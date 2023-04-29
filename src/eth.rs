use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::netif::EspNetifWait;
use esp_idf_svc::netif::EspNetif;
use esp_idf_hal::gpio;
use std::time::Duration;
use std::error::Error;
use esp_idf_hal::gpio::PinDriver;
use std::net::UdpSocket;
use esp_idf_hal::mac::MAC;

pub struct EthernetComms<'a> {
  _eth: Box<esp_idf_svc::eth::EspEth<'a>>,
  socket: UdpSocket
}

pub struct EthernetPins {
  pub gpio5: gpio::Gpio5,
  pub gpio12: gpio::Gpio12,
  pub gpio17: gpio::Gpio17,
  pub gpio18: gpio::Gpio18,
  pub gpio19: gpio::Gpio19,
  pub gpio21: gpio::Gpio21,
  pub gpio22: gpio::Gpio22,
  pub gpio23: gpio::Gpio23,
  pub gpio25: gpio::Gpio25,
  pub gpio26: gpio::Gpio26,
  pub gpio27: gpio::Gpio27,
}

fn eth_configure(
  sysloop: &EspSystemEventLoop,
  mut eth: Box<esp_idf_svc::eth::EspEth<'static>>,
) -> Result<Box<esp_idf_svc::eth::EspEth<'static>>, Box<dyn Error>> {
  use std::net::Ipv4Addr;

  println!("Eth created");

  eth.start()?;
  println!("Starting eth...");

  if !esp_idf_svc::eth::EthWait::new(eth.driver(), sysloop).expect("Failed to create new eth driver")
      .wait_with_timeout(Duration::from_secs(20), || eth.is_started().expect("Failed to check eth started"))
  {
      panic!("Eth did not start");
  }

  if !EspNetifWait::new::<EspNetif>(eth.netif(), &sysloop).expect("Failed to get EspNetIf")
      .wait_with_timeout(Duration::from_secs(20), || {
          eth.netif().get_ip_info().expect("Failed to get IP info").ip != Ipv4Addr::new(0, 0, 0, 0)
      })
  {
      panic!("Eth did not receive a DHCP lease");
  }

  let ip_info = eth.netif().get_ip_info().expect("Failed to get IP info");
  println!("Eth DHCP info: {:?}", ip_info);

  Ok(eth)
}

impl EthernetComms<'_> {

  pub fn new(
    sysloop: &EspSystemEventLoop,
    mac: MAC,
    pins: EthernetPins
  ) -> Self {

    let mut phy_power = PinDriver::output(pins.gpio12).expect("Could not create PHY power driver.");
    phy_power.set_high().expect("Failed to turn on PHY power.");

    let eth = eth_configure(
      &sysloop,
      Box::new(esp_idf_svc::eth::EspEth::wrap(
        esp_idf_svc::eth::EthDriver::new_rmii(
          mac,
          pins.gpio25, // rdx0
          pins.gpio26, // rdx1
          pins.gpio27, // crs_dv
          pins.gpio23, // mdc
          pins.gpio22, // txd1
          pins.gpio21, // tx_en
          pins.gpio19, // txd0
          pins.gpio18, // mdio
          esp_idf_svc::eth::RmiiClockConfig::<gpio::Gpio0, gpio::Gpio16, gpio::Gpio17>::OutputInvertedGpio17(
              pins.gpio17,
          ), // clk_config
          Some(pins.gpio5), // rst
          esp_idf_svc::eth::RmiiEthChipset::LAN87XX, // chipset
          Some(0x00), // phy_addr
          sysloop.clone(),
        ).expect("Failed to create new RMII driver"),
      ).expect("Failed to wrap EspEth")),
    ).expect("Failed to configure Ethernet");

    let socket = UdpSocket::bind("0.0.0.0:1234").expect("Failed to create UDP socket.");

    EthernetComms {
      _eth: eth,
      socket
    }

  }

  pub fn send(&self, buf: &[u8]) -> Result<(), Box<dyn Error>> {
    self.socket.send_to(buf, "239.1.2.3:1234")?;
    Ok(())
  }

}