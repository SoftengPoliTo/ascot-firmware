//! HTTP Server with REST API handler
//!
//! Go to 192.168.1.126 to test

use std::sync::{Arc, Mutex};

// Ascot library
use ascot_library::device::DeviceKind;
use ascot_library::route::Route;

// Ascot Esp32
use ascot_esp32c3::device::{Device, DeviceAction};
use ascot_esp32c3::server::AscotServer;
use ascot_esp32c3::wifi::Wifi;

// Esp idf
use esp_idf_svc::hal::delay::Ets;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::io::EspIOError;
use esp_idf_svc::io::Write;
use esp_idf_svc::log::EspLogger;

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
// TODO: An action should return a reply to notify that the operation went well.
//  Ask upstream if it's possible to add a server.fn_handle() API which returns
//  a forced reply.

fn main() -> anyhow::Result<()> {
    // A hack to make sure that a few patches to the ESP-IDF which are
    // implemented in Rust are linked to the final executable
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    EspLogger::initialize_default();

    // `async-io` uses the ESP IDF `eventfd` syscall to implement async IO.
    // If you use `tokio`, you still have to do the same as it also uses the `eventfd` syscall
    esp_idf_svc::io::vfs::initialize_eventfd(5)?;

    // Retrieve all esp32c3 peripherals (singleton)
    let peripherals = Peripherals::take()?;

    // Retrieve ssid e password
    let wifi_config = WIFI_CONFIG;

    // Connects to Wi-Fi
    let wifi = Wifi::connect_wifi(wifi_config.ssid, wifi_config.password, peripherals.modem)?;

    // Create the driver for the built-in led in output mode
    let mut built_in_led_output = PinDriver::output(peripherals.pins.gpio8)?;
    // Turn the built-in led off setting the impedance high
    built_in_led_output.set_high()?;
    // Delay 1ms
    Ets::delay_ms(1u32);

    // Create an atomic reference counter accessible in mutual exclusion by
    // server route.
    let temp_led_main = Arc::new(Mutex::new(built_in_led_output));

    // Configuration for the main page route.
    let main_page_config = Route::get("/").description("Main page.");

    let main_page_action = DeviceAction::no_hazards(
        main_page_config,
        move |req| -> core::result::Result<(), EspIOError> {
            req.into_ok_response()?.write_all(b"Main page!")?;

            Ok(())
        },
    );

    // Configuration for the `PUT` turn light on route.
    let light_on_config = Route::put("/on").description("Turn light on.");

    let temp_led_on = temp_led_main.clone();
    let light_on_action = DeviceAction::no_hazards(
        light_on_config,
        move |req| -> core::result::Result<(), EspIOError> {
            // Turn built-in led on.
            temp_led_on.lock().unwrap().set_low().unwrap();

            // Add a delay of 1ms
            Ets::delay_ms(1u32);

            // Add a response
            req.into_ok_response()?
                .write_all(b"Turning led on went well!")?;

            Ok(())
        },
    );

    // Configuration for the `PUT` turn light off route.
    let light_off_config = Route::put("/off").description("Turn light off.");

    let temp_led_off = temp_led_main.clone();
    let light_off_action = DeviceAction::no_hazards(
        light_off_config,
        move |req| -> core::result::Result<(), EspIOError> {
            // Turn built-in led off.
            temp_led_off.lock().unwrap().set_high().unwrap();

            // Add a delay of 1ms
            Ets::delay_ms(1u32);

            req.into_ok_response()?
                .write_all(b"Turning led off went well!")?;

            Ok(())
        },
    );

    let device = Device::new(DeviceKind::Light)
        .add_action(main_page_action)
        .add_action(light_on_action)
        .add_action(light_off_action);

    AscotServer::new(device, wifi.ip).run()
}
