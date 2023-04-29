use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_hal::prelude::Peripherals;
use esp_idf_hal::delay::FreeRtos;

use std::sync::mpsc::channel;
use std::time::Duration;

use serde::{Serialize, Deserialize};

mod eth;
use crate::eth::{EthernetComms, EthernetPins};

mod espnow;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
struct EspNowPacket<'a> {
    packet: &'a [u8]
}

fn main() -> () {
    esp_idf_sys::link_patches();
    
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;
    let sysloop = EspSystemEventLoop::take().expect("Failed to get system loop");

    println!("Waiting for clock start...");
    FreeRtos::delay_ms(1000u32);

    let ethernet = EthernetPins {
        gpio5: pins.gpio5,
        gpio12: pins.gpio12,
        gpio17: pins.gpio17,
        gpio18: pins.gpio18,
        gpio19: pins.gpio19,
        gpio21: pins.gpio21,
        gpio22: pins.gpio22,
        gpio23: pins.gpio23,
        gpio25: pins.gpio25,
        gpio26: pins.gpio26,
        gpio27: pins.gpio27,
    };

    let network = EthernetComms::new(&sysloop,
        peripherals.mac,
        ethernet
    );

    // Configure wifi / radio link
    let (tx, rx) = channel();
    let radio = espnow::init(&sysloop, peripherals.modem);
    radio.espnow.register_recv_cb(move |_mac, data| {
        let packet: Vec<u8> = data.to_vec();
        tx.send(packet).expect("Couldn't send packet data.");
        println!("Packet sent!");
    }).expect("Failed to register ESP-NOW recv callback");


    loop {
        if let Ok(packet) = rx.recv_timeout(Duration::from_secs(1)) {
            let buf = &bincode::serialize(&EspNowPacket { packet: &packet }).expect("Failed to serialize packet.");
            network.send(buf).expect("Couldn't send UDP packet.");
        }
        FreeRtos::delay_ms(10u32);
    }

}
