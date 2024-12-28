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

use crate::prelude::*;

use esp_idf_hal::io::{Read, Write};
use esp_idf_svc::http::{
        Method,
        server::EspHttpServer,
};

use serde::Deserialize;
use std::sync::{Arc, RwLock};

#[derive(Deserialize)]
struct FormData {
    brightness: f32,
    temperature: f32,
    speed: f32,
}

#[derive(Deserialize)]
struct FormDataChannels {
    channels: [u16; 12],
}

static INDEX_HTML: &str = include_str!("../http_server_page.html");

// Max payload length
const MAX_LEN: usize = 128;

// Need lots of stack to parse JSON
const STACK_SIZE: usize = 10240;


#[named]
pub fn run_server(
    light_temperature_target: Arc<RwLock<f32>>,
    light_brightness_target: Arc<RwLock<f32>>,
    light_dim_speed: Arc<RwLock<f32>>,
    update_requested: Arc<RwLock<bool>>,
    thermal: Arc<RwLock<f32>>, 
    voltage: Arc<RwLock<f32>>, 
    current: Arc<RwLock<f32>>,
) -> Result<()> {
    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    };
    let mut server: EspHttpServer<'_> = EspHttpServer::new(&server_configuration).or(Err(anyhow!("Could not create server.")))?;
    info!(target: function_name!(), "Created the server. Attaching handlers.");

    server.fn_handler::<anyhow::Error, _>("/", Method::Get, |req| {
        req.into_ok_response()?.write_all(INDEX_HTML.as_bytes()).map(|_| ())?;
        return Ok(());
    })?;

    server.fn_handler::<anyhow::Error, _>("/values", Method::Get, |req| {
        let mut resp = req.into_ok_response()?;
        let t = *thermal.read().unwrap();
        let u = *voltage.read().unwrap();
        let i = *current.read().unwrap();
        let p = u * i;
        write!(resp, "Current values:\nTemperature: {} deg C, Voltage: {} V, Current: {} A, Power: {} W", t, u, i, p)?;
        return Ok(());
    })?;


    server.fn_handler::<anyhow::Error, _>("/channels", Method::Get, |mut req| {
        let len = req.header("Content-Length") .and_then(|v| v.parse::<u64>().ok()).unwrap_or(0) as usize;

        if len > MAX_LEN {
            req.into_status_response(413)?
                .write_all("Request too big".as_bytes())?;
            return Ok(());
        }

        let mut buf = vec![0; len];
        req.read_exact(&mut buf)?;
        let mut resp = req.into_ok_response()?;

        if let Ok(form) = serde_json::from_slice::<FormDataChannels>(&buf) {
            warn!(target: function_name!(),
                ": Got an HTTP request to set the channels to {:?}, but that is not yet implemented.",
                form.channels
            );
            write!(
                resp,
                "WARNING: Got an HTTP request to set the channels to {:?}, but that is not yet implemented.",
                form.channels
            )?;
        } else {
            resp.write_all("JSON error".as_bytes())?;
        }

        Ok(())
    })?;

    server.fn_handler::<anyhow::Error, _>("/post", Method::Post, |mut req| {
        let len = req.header("Content-Length") .and_then(|v| v.parse::<u64>().ok()).unwrap_or(0) as usize;

        if len > MAX_LEN {
            req.into_status_response(413)?
                .write_all("Request too big".as_bytes())?;
            return Ok(());
        }

        let mut buf = vec![0; len];
        req.read_exact(&mut buf)?;
        let mut resp = req.into_ok_response()?;

        if let Ok(form) = serde_json::from_slice::<FormData>(&buf) {
            write!(
                resp,
                "Set color temperature to {}K, brightness to {} with speed {}...",
                form.temperature, form.brightness, form.speed
            )?;
            *light_brightness_target.write().unwrap() = form.brightness;
            *light_temperature_target.write().unwrap() = form.temperature;
            *light_dim_speed.write().unwrap() = form.speed;   
        } else {
            resp.write_all("JSON error".as_bytes())?;
        }

        Ok(())
    })?;

    server.fn_handler::<anyhow::Error, _>("/ota/start", Method::Post, |req| {
        info!(target: function_name!(), "Got ota start request.");
        *update_requested.write().unwrap() = true;
        let mut resp = req.into_ok_response()?;
        resp.write_all("Seems to be ok.".as_bytes())?;
        Ok(())
    })?;

    info!(target: function_name!(), "Handlers attached.");

    loop {
        trace!(target: function_name!(), "Inside server keep-alive loop.");
        std::thread::sleep(core::time::Duration::from_millis(2000));
    }
}
