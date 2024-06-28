//! HTTP Server with JSON POST handler
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
    /* 1. Define device passing closure (we need to Box I suppose!!!)
     * Light::new(
     * closure
     * closure
     * )
     * .add_action(closure)
     * .build() --> get Device
     *
     * 1. Initialization of peripherals, logs and device.
     * esp_idf_svc::sys::link_patches();
     * esp_idf_svc::log::EspLogger::initialize_default();
     AscotServer::new(device)
     *
     * 2. Wifi-connection
     * let peripherals = peripherals::take()?;
     * let sys_loop = espsystemeventloop::take()?;
     * let nvs = espdefaultnvspartition::take()?;
     * // connects to wi-fi
     * let _wifi = connect_wifi(peripherals.modem, sys_loop, nvs)?;
      .connect_wifi()?
     *
     * 3. Run everything else
    */

    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // Retrieves wifi ssid and password
    let wifi_config = WIFI_CONFIG;

    // Connects to Wi-Fi
    let _wifi = connect_wifi(
        wifi_config.ssid,
        wifi_config.password,
        peripherals.modem,
        sys_loop,
        nvs,
    )?;

    // Configuration for the `PUT` turn light on route.
    let light_on_config = Route::put("/on").description("Turn light on.");

    let light_on_action = DeviceAction::no_hazards(light_on_config, |req| {
        req.into_ok_response()?
            .write_all(INDEX_HTML.as_bytes())
            .map(|_| ())
    });

    // Configuration for the `POST` turn light on route.
    let light_on_post_config = Route::post("/on").description("Turn light on.").inputs([
        Input::rangef64("brightness", (0., 20., 0.1, 0.)),
        Input::boolean("save-energy", false),
    ]);

    // Configuration for the turn light off route.
    let light_off_config = Route::put("/off").description("Turn light off.");

    // Configuration for the toggle route.
    let toggle_config = Route::put("/toggle").description("Toggle a light.");

    let device = Device::new(DeviceKind::Light).add_action(light_on_action);

    AscotServer::new(device).run()?;

    Ok(())
}
