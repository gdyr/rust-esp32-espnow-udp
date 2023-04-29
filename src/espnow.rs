use esp_idf_svc::{espnow::EspNow, espnow::PeerInfo, wifi::EspWifi, wifi::WifiDeviceId, eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use embedded_svc::wifi::*;
use crate::FreeRtos;

pub const BROADCAST_ADDR: [u8; 6] = [0xFF,0xFF,0xFF,0xFF,0xFF,0xFF];

pub struct Comms<'a> {
    pub espnow: EspNow,
    pub wifi: EspWifi<'a>
}

pub fn init<'a>(sys_loop: &EspSystemEventLoop, modem: esp_idf_hal::modem::Modem) -> Comms<'a> {
    let nvs = EspDefaultNvsPartition::take().expect("Could not get NVS partition.");

    // unsafe { esp_idf_sys::esp_wifi_internal_set_log_level(5); }

    let mut wifi_driver = EspWifi::new(
        modem,
        sys_loop.clone(),
        Some(nvs)
    ).expect("Failed to init WiFi driver.");

    // let configuration = ClientConfiguration {
    //     ssid: "AlphaOne".into(),
    //     ssid_hidden: false,
    //     channel: 1,
    //     secondary_channel: None,
    //     protocols: Protocol::P802D11LR.into(),
    //     auth_method: AuthMethod::None,
    //     password: "".into(),
    //     max_connections: 16
    // };

    //wifi_driver.start().expect("Failed to start WiFi driver.");
    wifi_driver.set_configuration(
        &Configuration::Client(ClientConfiguration::default())
    ).expect("Failed to configure WiFi driver.");

    println!("WiFi reconfigured.");
    FreeRtos::delay_ms(500u32);

    wifi_driver.start().expect("Failed to start WiFi driver.");
    println!("WiFi driver started.");
    FreeRtos::delay_ms(500u32);

    let esp_now = EspNow::take().expect("Failed to init EspNow.");
    println!("ESP-Now started.");
    FreeRtos::delay_ms(2000u32);

    let peer_info = PeerInfo {
        peer_addr: BROADCAST_ADDR.clone(),
        ifidx: WifiDeviceId::Sta.into(),
        channel: 0,
        encrypt: false,
        lmk: [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        ..Default::default()
    };
    println!("ESP-NOW peer info: {:?}", peer_info);
    esp_now.add_peer(peer_info).expect("Failed to add broadcast peer.");

    Comms {
        espnow: esp_now,
        wifi: wifi_driver
    }

}