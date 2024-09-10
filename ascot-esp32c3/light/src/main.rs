//! HTTP Server with REST API handler
//!
//! Go to 192.168.71.1 to test

// Library.
use ascot_library::device::{DeviceErrorKind, DeviceKind};
use ascot_library::hazards::Hazard;
use ascot_library::input::Input;
use ascot_library::route::Route;

// Esp32
use ascot_esp32c3::device::{Device, DeviceAction};
use ascot_esp32c3::server::AscotServer;
use ascot_esp32c3::wifi::connect_wifi;

use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

use embedded_svc::io::Write;

// Max payload length
const MAX_LEN: usize = 128;

static INDEX_HTML: &str = include_str!("../http_server_page.html");

#[toml_cfg::toml_config]
pub struct WifiConfig {
    #[default("")]
    ssid: &'static str,
    #[default("")]
    password: &'static str,
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // Retrieves wifi ssid and password
    let wifi_config = WIFI_CONFIG;

    // Connects to Wi-Fi
    let (_wifi, ip) = connect_wifi(
        wifi_config.ssid,
        wifi_config.password,
        peripherals.modem,
        sys_loop,
        nvs,
    )?;

    // Configuration for the `PUT` turn light on route.
    let light_on_config = Route::get("/on").description("Turn light on.");

    let light_on_action = DeviceAction::no_hazards(light_on_config, |req| {
        req.into_ok_response()?
            .write_all(INDEX_HTML.as_bytes())
            .map(|_| ())
    });

    let device = Device::new(DeviceKind::Light).add_action(light_on_action);

    AscotServer::new(device, ip).run()
}
