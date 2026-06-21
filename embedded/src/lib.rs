#![no_std]

use dht_sensor;


pub fn read_dht22(delay: &mut esp_hal::delay::Delay, pin: &mut esp_hal::gpio::Flex) -> dht_sensor::dht22::Reading {
    match dht_sensor::dht22::blocking::read(delay, pin) {
        Ok(result) => {
            return result;
        }
        Err(e) => {
            esp_println::println!("DHT22 error: {:?}", e);
            panic!("Uh oh");
        } // Fix
    }
}