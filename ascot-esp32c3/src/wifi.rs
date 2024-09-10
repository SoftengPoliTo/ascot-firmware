use core::net::Ipv4Addr;

use anyhow::bail;

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripheral,
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};

use log::info;

pub fn connect_wifi(
    ssid: &'static str,
    password: &'static str,
    modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
    sys_loop: EspSystemEventLoop,
    nvs: EspDefaultNvsPartition,
) -> anyhow::Result<(EspWifi<'static>, Ipv4Addr)> {
    if ssid.is_empty() {
        bail!("Missing Wi-Fi SSID")
    }

    let auth_method = if password.is_empty() {
        info!("Wifi password is empty");
        AuthMethod::None
    } else {
        AuthMethod::WPA2Personal
    };

    let mut esp_wifi = EspWifi::new(modem, sys_loop.clone(), Some(nvs))?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sys_loop)?;

    // Set default Wi-Fi configuration.
    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

    // Start Wi-Fi
    wifi.start()?;

    // Scan the network looking for all possible access points information.
    let ap_infos = wifi.scan()?;

    // Iter on all access points looking for the one associated with
    // the input SSID.
    let input_ap = ap_infos.into_iter().find(|a| a.ssid == ssid);

    let channel = if let Some(input_ap) = input_ap {
        info!("Found input SSID {} on channel {}", ssid, input_ap.channel);
        Some(input_ap.channel)
    } else {
        info!(
            "Input access point {} not found during the scanning process, an unknown channel will be used.",
            ssid
        );
        None
    };

    // Configure Wi-FI
    let wifi_configuration = Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().unwrap(),
        password: password.try_into().unwrap(),
        auth_method,
        channel,
        ..Default::default()
    });

    // Sets the Wi-Fi configuration in order to connect to the access point.
    wifi.set_configuration(&wifi_configuration)?;

    // Connects to Wi-Fi
    wifi.connect()?;

    // DHCP lease phase
    wifi.wait_netif_up()?;

    // Gets the firmware IP information
    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    Ok((esp_wifi, ip_info.ip))
}
