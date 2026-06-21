#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use dht_sensor::dht22;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Output, OutputConfig};
use esp_hal::timer::timg::TimerGroup;
use core::fmt::Write;
use reqwless::request::RequestBuilder;
use esp_hal::delay::Delay;
use esp_println;
use embassy_net;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32c3 -o alloc -o unstable-hal -o wifi -o embassy -o vscode
    
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 66320);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let (mut _wifi_controller, _interfaces) =
        esp_radio::wifi::new(peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    let station_config = esp_radio::wifi::Config::Station(
        esp_radio::wifi::sta::StationConfig::default()
        .with_ssid(env!("WIFI_SSID"))
        .with_password(env!("WIFI_PASSWORD").into()),
    );

    match _wifi_controller.set_config(&station_config) {
        Ok(_) => (),
        Err(e) => esp_println::println!("Error setting config: {:?}", e),
    }

    esp_println::println!("Wifi configured and started!");
    let wifi_config = embassy_net::Config::dhcpv4(Default::default());
    let rng: esp_hal::rng::Rng = esp_hal::rng::Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    let (stack, runner) = embassy_net::new(
        _interfaces.station,
        wifi_config,
        mk_static!(embassy_net::StackResources<3>, embassy_net::StackResources::<3>::new()),
        seed
    );

    esp_println::println!("Scan");
    let scan_config = esp_radio::wifi::scan::ScanConfig::default().with_max(10);
    let result = _wifi_controller.scan_async(&scan_config).await.unwrap();
    for ap in result {
        esp_println::println!("{:?}", ap);
    }

    spawner.spawn(connection(_wifi_controller).unwrap());
    spawner.spawn(net_task(runner).unwrap());

    stack.wait_config_up().await;

    if let Some(config) = stack.config_v4() {
        esp_println::println!("Got IP: {}", config.address);
    }

    // Init HTTP client
    let tcp_client = embassy_net::tcp::client::TcpClient::new(
        stack,
        mk_static!(
            embassy_net::tcp::client::TcpClientState<1,1500,1500> ,
            embassy_net::tcp::client::TcpClientState::<1,1500,1500>::new()
        ),
    );
    let dns_client = embassy_net::dns::DnsSocket::new(stack);
    

    // TODO: Spawn some tasks
    let mut delay = Delay::new();
    delay.delay_millis(2000);

    // let _ = spawner;
    let mut dht22_sensor = esp_hal::gpio::Flex::new(peripherals.GPIO3);
    dht22_sensor.set_input_enable(true);
    dht22_sensor.set_output_enable(true);
    dht22_sensor.set_high();
    
    loop {
        Timer::after(Duration::from_secs(3)).await;
        match dht_sensor::dht22::blocking::read(&mut delay, &mut dht22_sensor) {
            Ok(read) => {
                esp_println::println!("Temp: {}, Humidity: {}", read.temperature, read.relative_humidity);
                let mut client = reqwless::client::HttpClient::new(&tcp_client, &dns_client);
                let mut rx_buf = [0u8; 4096];
                match embassy_time::with_timeout(Duration::from_secs(5), client
                    .request(reqwless::request::Method::POST, env!("SERVER_URL")))
                    .await {
                        Ok(Ok(builder)) => {
                            let mut body: heapless::String<64> =  heapless::String::new();
                            write!(body, "{{\"temp\":{},\"humidity\":{}}}", read.temperature, read.relative_humidity).unwrap();

                            match builder
                            .body(body.as_bytes())
                            .content_type(reqwless::headers::ContentType::ApplicationJson)
                            .send(&mut rx_buf)
                            .await {
                                Ok(_) => esp_println::println!("Posted successfully"),
                                Err(e) => esp_println::println!("Error sending: {}", e),
                            }
                        },
                        Ok(Err(e)) => esp_println::println!("Request error: {:?}", e),
                        Err(_) => esp_println::println!("Request timed out"),
                    }

            },
            Err(e) => {
                esp_println::println!("DHT22 error: {:?}", e);
            }
        }
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.1.0/examples
}


#[embassy_executor::task]
async fn connection(mut controller: esp_radio::wifi::WifiController<'static>) {
    esp_println::println!("start connection task");

    loop {
        esp_println::println!("About to connect...");

        match controller.connect_async().await {
            Ok(info) => {
                esp_println::println!("Wifi connected to {:?}", info);

                // wait until we're no longer connected
                let info = controller.wait_for_disconnect_async().await.ok();
                esp_println::println!("Disconnected: {:?}", info);
            }
            Err(e) => {
                esp_println::println!("Failed to connect to wifi: {e:?}");
            }
        }

        Timer::after(Duration::from_millis(5000)).await
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, esp_radio::wifi::Interface<'static>>) {
    runner.run().await
}
