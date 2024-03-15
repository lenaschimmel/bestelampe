#[toml_cfg::toml_config]
pub struct Config {
    #[default(false)]
    wifi_ap_active: bool,

    #[default("")]
    wifi_client_ssid: &'static str,

    #[default("")]
    wifi_client_psk: &'static str,   

    #[default("")]
    wifi_ap_ssid: &'static str,

    #[default("")]
    wifi_ap_psk: &'static str,   

    #[default("Etc/GMT")]
    time_zone: &'static str,
}