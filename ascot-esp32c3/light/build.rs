#[toml_cfg::toml_config]
pub struct WifiConfig {
    #[default("")]
    ssid: &'static str,
    #[default("")]
    password: &'static str,
}

fn main() {
    let cfg = std::path::Path::new("cfg.toml");

    // Checks whether Wi-FI configuration exists
    if !cfg.exists() {
        panic!("A `wifi_config.toml` file with Wi-Fi credentials is required! Use `wifi_config.toml.example` as a template.");
    }

    // Track the `cfg.toml` file so that a rebuild is done upon changes to it
    embuild::cargo::track_file(cfg);

    let wifi_config = WIFI_CONFIG;
    if wifi_config.ssid == "your Wi-Fi SSID" || wifi_config.password == "your Wi-Fi password" {
        panic!("Wi-Fi credentials should be set up in `wifi_config.toml` file!");
    }

    embuild::espidf::sysenv::output();
}
