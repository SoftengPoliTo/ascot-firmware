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

use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

use embedded_svc::io::Write;

use anyhow::anyhow;

use std::sync::{Arc, Mutex};

// Max payload length
const MAX_LEN: usize = 128;

#[toml_cfg::toml_config]
pub struct WifiConfig {
    #[default("")]
    ssid: &'static str,
    #[default("")]
    password: &'static str,
}

// TODO:
//
// Developer define how to contact the device and should do that through a json
// file.
//
// - Define how to send data through POST method
//
//
// . Convert Wifi access into a Rust struct and not function.

fn main() -> anyhow::Result<()> {
    // A hack to make sure that a few patches to the ESP-IDF which are
    // implemented in Rust are linked to the final executable
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Retrieve all esp32c3 peripherals
    let peripherals = Peripherals::take()?;

    // Retrieve system loop
    let sys_loop = EspSystemEventLoop::take()?;
    // Retrieve nvs partitions
    let nvs = EspDefaultNvsPartition::take()?;

    // Connects to Wi-Fi
    let (_wifi, ip) = connect_wifi(
        WIFI_CONFIG.ssid,
        WIFI_CONFIG.password,
        peripherals.modem,
        sys_loop,
        nvs,
    )?;

    // Configuration for the `PUT` turn light on route.
    let light_on_config = Route::put("/on").description("Turn light on.");

    let light_on_action = DeviceAction::no_hazards(light_on_config, |req| {
        let peripherals = Peripherals::take()?;

        // Retrieve built-in LED on the microcontroller
        let mut pin = PinDriver::output(peripherals.pins.gpio8)?;

        // Set pin high.
        pin.set_high()?;

        // Use a delay smaller than 10ms.
        Ets::delay_ms(1u32);

        req.into_ok_response()?
            .write_all(b"All went well!")
            .map(|_| ())

        // TODO: An action should return a reply to notify that the operation
        // goes well. Ask upstream.
    });

    let device = Device::new(DeviceKind::Light).add_action(light_on_action);

    AscotServer::new(device, ip).run()
}
