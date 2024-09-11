use core::net::Ipv4Addr;

use anyhow::bail;

use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};

use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

use log::info;

pub struct Wifi {
    pub ip: Ipv4Addr,
    _wifi: AsyncWifi<EspWifi<'static>>,
}

impl Wifi {
    pub fn connect_wifi(
        ssid: &'static str,
        password: &'static str,
        modem: impl Peripheral<P = Modem> + 'static,
    ) -> anyhow::Result<Self> {
        if ssid.is_empty() {
            bail!("Missing Wi-Fi SSID")
        }

        if password.is_empty() {
            bail!("Missing Wifi password");
        }

        // Retrieve system loop (singleton)
        let sys_loop = EspSystemEventLoop::take()?;
        // Retrieve timer service (singleton)
        let timer_service = EspTaskTimerService::new()?;
        // Retrieve nvs partitions (singleton)
        let nvs = EspDefaultNvsPartition::take()?;

        let mut wifi = AsyncWifi::wrap(
            EspWifi::new(modem, sys_loop.clone(), Some(nvs))?,
            sys_loop,
            timer_service,
        )?;

        block_on(Self::connect(&mut wifi, ssid, password))?;

        let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

        info!("Wifi DHCP info: {:?}", ip_info);

        Ok(Self {
            ip: ip_info.ip,
            _wifi: wifi,
        })
    }

    async fn connect(
        wifi: &mut AsyncWifi<EspWifi<'static>>,
        ssid: &str,
        password: &str,
    ) -> anyhow::Result<()> {
        let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
            ssid: ssid.try_into().unwrap(),
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: password.try_into().unwrap(),
            channel: None,
            ..Default::default()
        });

        wifi.set_configuration(&wifi_configuration)?;

        wifi.start().await?;
        info!("Wifi started");

        wifi.connect().await?;
        info!("Wifi connected");

        wifi.wait_netif_up().await?;
        info!("Wifi netif up");

        Ok(())
    }
}
