// SPDX-FileCopyrightText: 2024 Lena Schimmel <mail@lenaschimmel.de>
// SPDX-License-Identifier: CERN-OHL-S-2.0+
// This file is part of besteLampe!.
// 
// besteLampe! is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software Foundation, 
// either version 3 of the License, or (at your option) any later version.
// 
// besteLampe! is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; 
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. 
// See the GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License along with besteLampe!.
// If not, see <https://www.gnu.org/licenses/>. 

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