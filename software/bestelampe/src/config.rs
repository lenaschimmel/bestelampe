#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,   
    #[default("Etc/GMT")]
    time_zone: &'static str,
}