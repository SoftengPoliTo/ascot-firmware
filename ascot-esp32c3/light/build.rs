#[toml_cfg::toml_config]
pub struct WifiConfig {
    #[default("")]
    ssid: &'static str,
    #[default("")]
    password: &'static str,
}

fn main() {
    // Checks whether Wi-FI configuration exists
    if !std::path::Path::new("cfg.toml").exists() {
        panic!("A `wifi_config.toml` file with Wi-Fi credentials is required! Use `wifi_config.toml.example` as a template.");
    }

    let wifi_config = WIFI_CONFIG;
    if wifi_config.ssid == "your Wi-Fi SSID" || wifi_config.password == "your Wi-Fi password" {
        panic!("Wi-Fi credentials should be set up in `wifi_config.toml` file!");
    }

    embuild::espidf::sysenv::output();
}
