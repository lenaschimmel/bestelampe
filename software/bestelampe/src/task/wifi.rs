use crate::prelude::*;

use esp_idf_hal::modem::Modem;

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    netif::{EspNetif, NetifConfiguration, NetifStack},
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi, AuthMethod, WifiDriver},
};

// Wi-Fi channel, between 1 and 11
const CHANNEL: u8 = 11;

/// Starts the wifi either as station ("client") or access point.
/// Does not have any retry-loop or error handling.
/// Method returns when the wifi is ready to be used.
#[named]
pub fn start_wifi(modem: Modem, as_access_point: bool) -> Result<()> {
    info!(target: function_name!(), "Inside 'start_wifi'...");
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let ipv4_client_cfg =
        esp_idf_svc::ipv4::ClientConfiguration::DHCP(esp_idf_svc::ipv4::DHCPClientSettings {
            hostname: Some(heapless::String::<30>::try_from("besteLampe").unwrap()),
            ..Default::default()
        });
    let new_c = NetifConfiguration {
        ip_configuration: esp_idf_svc::ipv4::Configuration::Client(ipv4_client_cfg),
        ..NetifConfiguration::wifi_default_client()
    };

    let esp_wifi = EspWifi::wrap_all(
        WifiDriver::new(
            modem,
            sys_loop.clone(),
            Some(nvs),
        )?,
        EspNetif::new_with_conf(&new_c)?,
        EspNetif::new(NetifStack::Ap)?,
    )?;

    let mut wifi = BlockingWifi::wrap(
        esp_wifi,
        sys_loop,
    )?;

    let wifi_configuration = match as_access_point {
        false => 
            esp_idf_svc::wifi::Configuration::Client(esp_idf_svc::wifi::ClientConfiguration {
            ssid: CONFIG.wifi_ssid.try_into().or(Err(anyhow!("Invalid SSID config.")))?,
            password: CONFIG.wifi_psk.try_into().or(Err(anyhow!("Invalid PSK config.")))?,
            auth_method: AuthMethod::WPA2Personal,
            ..Default::default()
        }),
        true => esp_idf_svc::wifi::Configuration::AccessPoint(esp_idf_svc::wifi::AccessPointConfiguration {
            ssid: CONFIG.wifi_ssid.try_into().or(Err(anyhow!("Invalid SSID config.")))?,
            password: CONFIG.wifi_psk.try_into().or(Err(anyhow!("Invalid PSK config.")))?,
            ssid_hidden: false,
            auth_method: AuthMethod::WPA2Personal,
            channel: CHANNEL,
            ..Default::default()
        })
    };

    info!(target: function_name!(), "Setting configuration...");
    wifi.set_configuration(&wifi_configuration)?;
    info!(target: function_name!(), "Starting...");
    wifi.start()?;
    if !as_access_point {
        info!(target: function_name!(), "Connecting...");
        wifi.connect()?;
    }
    info!(target: function_name!(), "Waiting for netif...");
    wifi.wait_netif_up()?;

    info!(target: function_name!(), 
        "Joined Wi-Fi with WIFI_SSID `{}` and WIFI_PASS `{}` as {}",
        CONFIG.wifi_ssid, CONFIG.wifi_psk, if as_access_point { "access point" } else { "station" }
    );

    core::mem::forget(wifi);

    return Ok(());
}