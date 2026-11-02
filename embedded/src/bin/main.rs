#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use embassy_executor::{Spawner};
use embassy_net::{dns::DnsSocket, tcp::client::{TcpClient, TcpClientState}};
use embassy_time::{Duration, Timer, with_timeout};
use esp_hal::{clock::CpuClock, gpio::Output};
use esp_hal::timer::timg::TimerGroup;
use esp_println;
use reqwless::{client::HttpClient, headers::ContentType};
use shared::Reading as SensorReading;
use shared::DoorCommand;
use shared::CommandAck;
use reqwless::request::{Method, RequestBuilder};
use static_cell::{StaticCell};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// Network Statics
static TCP_CLIENT_STATE: StaticCell<TcpClientState<1, 1500, 1500>> = StaticCell::new();
static TCP_CLIENT: StaticCell<TcpClient<'_, 1, 1500, 1500>> = StaticCell::new();
static DNS_CLIENT: StaticCell<DnsSocket<'_>> = StaticCell::new();
static STACK_RESOURCES: StaticCell<embassy_net::StackResources<3>> = StaticCell::new();

// Constants
const POLL_INTERVAL: Duration = Duration::from_secs(60);
const RETRY_INTERVAL: Duration = Duration::from_secs(2);
const NET_TIMEOUT: Duration = Duration::from_secs(5);
const DOOR_PULSE: Duration = Duration::from_millis(500); // how long the "button press" lasts

const SUBMIT_READING_ROUTE: &str = concat!(env!("SERVER_URL"), "/readings/submit");
const GET_DOOR_COMMAND_ROUTE: &str = concat!(env!("SERVER_URL"), "/commands/door/");
const FULL_DOOR_ACK_ROUTE: &str = concat!(env!("SERVER_URL"), "/commands/door/ack");

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65000);


    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    let (mut wifi_controller, interfaces) =
        esp_radio::wifi::new(peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");

    let station_config = esp_radio::wifi::Config::Station(
        esp_radio::wifi::sta::StationConfig::default()
        .with_ssid(env!("WIFI_SSID"))
        .with_password(env!("WIFI_PASSWORD").into()),
    );

    match wifi_controller.set_config(&station_config) {
        Ok(_) => (),
        Err(e) => esp_println::println!("Error setting config: {:?}", e),
    }

    esp_println::println!("Wifi configured and started!");
    let wifi_config = embassy_net::Config::dhcpv4(Default::default());
    let _ = wifi_controller.set_power_saving(esp_radio::wifi::PowerSaveMode::Minimum);
    let rng: esp_hal::rng::Rng = esp_hal::rng::Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    let (stack, runner) = embassy_net::new(
        interfaces.station,
        wifi_config,
        STACK_RESOURCES.init(embassy_net::StackResources::<3>::new()),
        seed,
    );

    spawner.spawn(connection(wifi_controller).unwrap());
    spawner.spawn(net_task(runner).unwrap());

    stack.wait_config_up().await;

    if let Some(config) = stack.config_v4() {
        esp_println::println!("Got IP: {}", config.address);
    }

    // Init HTTP client
    let tcp_client_state = TCP_CLIENT_STATE.init(embassy_net::tcp::client::TcpClientState::<1,1500,1500>::new());
    let tcp_client = TCP_CLIENT.init(embassy_net::tcp::client::TcpClient::new(
        stack,
        tcp_client_state,
    ));
    let dns_client = DNS_CLIENT.init(embassy_net::dns::DnsSocket::new(stack));
    
    // Output NEEDS to be open drain
    let pin_cfg = esp_hal::gpio::OutputConfig::default().with_drive_mode(esp_hal::gpio::DriveMode::OpenDrain);
    let mut dht22_pin = esp_hal::gpio::Flex::new(peripherals.GPIO18);
    dht22_pin.apply_output_config(&pin_cfg);

    dht22_pin.set_input_enable(true);
    dht22_pin.set_output_enable(true);
    dht22_pin.set_high();

    let open_opto = esp_hal::gpio::Output::new(peripherals.GPIO20, esp_hal::gpio::Level::Low, esp_hal::gpio::OutputConfig::default());
    let close_opto = Output::new(peripherals.GPIO19, esp_hal::gpio::Level::Low, esp_hal::gpio::OutputConfig::default());

    spawner.spawn(dht_task(dht22_pin, &*tcp_client, &*dns_client).expect("Should work"));
    spawner.spawn(door_poll_task(tcp_client, dns_client, open_opto, close_opto).expect("Should work"));
    loop {
        embassy_time::Timer::after_secs(1).await; // Yield control to let other tasks start
    }
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

#[embassy_executor::task]
pub async fn dht_task(mut pin: esp_hal::gpio::Flex<'static>,
    tcp_client: &'static TcpClient<'static, 1, 1500, 1500>,
    dns_socket: &'static DnsSocket<'static>,
) {    
    esp_println::println!("hello");
    let mut client = HttpClient::new(tcp_client, dns_socket);

    let mut buf = [0u8; 128];
    let mut rx_buf = [0u8; 4096];

    let mut delay = embassy_time::Delay;
    Timer::after_secs(2).await; // Give 2 seconds for the sensor to warm up
    loop {
        let sensor_read = match dht_sensor::dht22::r#async::read(&mut delay, &mut pin).await {
            Ok(sensor_read) => SensorReading{temperature: sensor_read.temperature, humidity: sensor_read.relative_humidity},
            Err(e) => {
                esp_println::println!("DHT22 error: {:?}", e);
                Timer::after(RETRY_INTERVAL).await;
                continue; // Try again after interval
            }
        };
        
        let dht_future = send_dht(&mut client, &sensor_read, &mut buf, &mut rx_buf);
        match with_timeout(NET_TIMEOUT, dht_future).await {
            Ok(Ok(_)) => esp_println::println!("Posted successfully!"),
            Ok(Err(_)) => esp_println::println!("HTTP post failed :("),
            Err(_) => esp_println::println!("Request timed out"),
        }

        Timer::after(POLL_INTERVAL).await;
    }
}

async fn send_dht(
    client: &mut HttpClient<'_, TcpClient<'_, 1, 1500, 1500>, DnsSocket<'_>>,
    sensor_read: &SensorReading,
    buf: &mut [u8],
    rx_buf: &mut [u8]
) -> Result<(), ()> {
    let len = serde_json_core::to_slice(&sensor_read, buf).map_err(|_| ())?;

    let request = client
    .request(Method::POST, SUBMIT_READING_ROUTE)
    .await
    .map_err(|_| ())?;

    request
        .body(&buf[..len])
        .content_type(ContentType::ApplicationJson)
        .send(rx_buf)
        .await
        .map_err(|_| ())?;

    Ok(())
}

#[embassy_executor::task]
async fn door_poll_task(
    tcp_client: &'static TcpClient<'static, 1, 1500, 1500>,
    dns_socket: &'static DnsSocket<'static>,
    mut open_opto: esp_hal::gpio::Output<'static>,
    mut close_opto: esp_hal::gpio::Output<'static>,
) {
    // Option so that id 0 is still a valid command id
    let mut last_command_id: Option<u32> = None;
    let mut rx_buf = [0u8; 4096];

    // Make sure both optos start off
    open_opto.set_low();
    close_opto.set_low();

    let mut client = HttpClient::new(tcp_client, dns_socket);

    loop {
        let command = match with_timeout(NET_TIMEOUT, fetch_door_command(&mut client, &mut rx_buf)).await {
            Ok(Ok(command)) => command,
            Ok(Err(_)) => {
                esp_println::println!("HTTP get failed :(");
                Timer::after(POLL_INTERVAL).await;
                continue;
            }
            Err(_) => {
                esp_println::println!("Request timed out");
                Timer::after(POLL_INTERVAL).await;
                continue;
            }
        };

        // Nothing to do
        if matches!(command.action, shared::DoorAction::None) {
            Timer::after(POLL_INTERVAL).await;
            continue;
        }

        if last_command_id == Some(command.id) {
            // Already attempted this one (e.g. our ack got lost). Don't touch the door, just re-ack.
            esp_println::println!("Command {} already executed, re-acking", command.id);
        } else {
            match command.action {
                shared::DoorAction::Open => pulse(&mut open_opto, DOOR_PULSE).await,
                shared::DoorAction::Close => pulse(&mut close_opto, DOOR_PULSE).await,
                shared::DoorAction::None => unreachable!(),
            }
            // Record BEFORE acking so a failed ack can never cause a second actuation
            last_command_id = Some(command.id);
        }

        match with_timeout(NET_TIMEOUT, ack_door_command(&mut client, command.id, &mut rx_buf)).await {
            Ok(Ok(())) => esp_println::println!("Ack sent for {}", command.id),
            // Not fatal: the server will resend the command and we'll re-ack next poll
            Ok(Err(_)) => esp_println::println!("Ack failed, will retry next poll"),
            Err(_) => esp_println::println!("Ack timed out, will retry next poll"),
        }

        Timer::after(POLL_INTERVAL).await;
    }
}

/// Drive an optocoupler LED for `duration`, emulating a button press.
async fn pulse(pin: &mut Output<'static>, duration: Duration) {
    pin.set_high();
    Timer::after(duration).await;
    pin.set_low();
}

async fn fetch_door_command(
    client: &mut HttpClient<'_, TcpClient<'_, 1, 1500, 1500>, DnsSocket<'_>>,
    rx_buf: &mut [u8],
) -> Result<DoorCommand, ()> {
    let mut request = client
        .request(Method::GET, GET_DOOR_COMMAND_ROUTE)
        .await
        .map_err(|_| ())?;

    let response = request.send(rx_buf).await.map_err(|_| ())?;

    if !response.status.is_successful() {
        esp_println::println!("Bad status: {:?}", response.status);
        return Err(());
    }

    let body = response.body().read_to_end().await.map_err(|_| ())?;

    let (cmd, _len) = serde_json_core::from_slice::<DoorCommand>(body).map_err(|_| ())?;
    Ok(cmd)
}

async fn ack_door_command(
    client: &mut HttpClient<'_, TcpClient<'_, 1, 1500, 1500>, DnsSocket<'_>>,
    id: u32,
    rx_buf: &mut [u8],
) -> Result<(), ()> {
    let mut body = [0u8; 32]; // request body, separate from rx_buf
    let len = serde_json_core::to_slice(&CommandAck { id }, &mut body).map_err(|_| ())?;

    let builder = client
        .request(Method::POST, FULL_DOOR_ACK_ROUTE)
        .await
        .map_err(|_| ())?;

    builder
        .body(&body[..len])
        .content_type(ContentType::ApplicationJson)
        .send(rx_buf)
        .await
        .map_err(|_| ())?;

    Ok(())
}