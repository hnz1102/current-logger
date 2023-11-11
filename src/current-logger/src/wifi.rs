use std::time::Duration;
use std::thread;

use esp_idf_hal::peripheral;
use esp_idf_svc::{eventloop::EspSystemEventLoop, wifi::EspWifi};

use embedded_svc::wifi::{ClientConfiguration, Wifi, Configuration, AccessPointInfo};
use anyhow::bail;
use anyhow::Result;
use log::*;

pub fn wifi_connect<'d> (
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    ssid: &'d str,
    pass: &'d str,
) -> Result<Box<EspWifi<'d>>> {
  
    let sys_event_loop = EspSystemEventLoop::take().unwrap();
    let mut wifi = Box::new(EspWifi::new(modem, sys_event_loop.clone(), None).unwrap());

    let ap_list = wifi.scan().unwrap();
    let find_ap = ap_list.into_iter().find(|ap| ap.ssid == ssid);
    if find_ap == None {
        bail!("AP not found.");
    }
    let ap_info : AccessPointInfo = find_ap.unwrap();
    info!("{:?}", ap_info);

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.into(),
        password: pass.into(),
        ..Default::default()
    })).unwrap();

    wifi.start().unwrap();
    wifi.connect().unwrap();
    let mut timeout = 0;
    while !wifi.is_connected().unwrap(){
        thread::sleep(Duration::from_secs(1));
        timeout += 1;
        if timeout > 30 {
            bail!("Wifi could not be connected.");
        }
    }

    info!("Wifi connected");
    Ok(wifi)
}