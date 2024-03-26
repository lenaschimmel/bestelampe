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

use esp_idf_svc::{
    http::client::{
        Configuration, EspHttpConnection
    }, 
    ota::EspOta
};

use std::sync::{Arc, RwLock};

#[named]
pub fn test_ota(
    update_requested: Arc<RwLock<bool>>,
) -> Result<()> {
    let buf = vec![0; 512];

    /*


    let mut client = Client::<EspHttpConnection>::wrap(&mut http_connection);
    let headers = [("accept", "text/plain")];
    */
    
    let url = "http://192.168.1.21:8000/esp32c6.upd";
    
    //let url = "http://shininggrandinnermorning.neverssl.com/online/";

    let mut client = EspHttpConnection::new(&Configuration {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
        ..Default::default()
    })
    .expect("creation of EspHttpConnection should have worked");



    loop {
        if *(update_requested.read().unwrap()) {


            let resp = match client.initiate_request(
                esp_idf_svc::http::Method::Get,
                url,
                &[],
            ) {
                Ok(c) => c,
                Err(err) => {
                    error!("Failed to initiate request {}", err);
                }
            };
            info!("-> GET {}", url);

            client.initiate_response()?;

            let len = client.header("Content-Length") .and_then(|v| v.parse::<u64>().ok()).unwrap_or(0) as usize;
           
            if len == 0 {
                warn!(target: function_name!(), "Got ota file without Content-Length header, aborting.");
                return Ok(());
            }
            info!(target: function_name!(), "Got ota file response with Content-Length header {}.", len);

            let mut ota = EspOta::new()?;
            info!("ota A");
            let mut ota_update = ota.initiate_update()?;
            info!("ota B");
            let mut buf = vec![0; 512];
            let mut total_bytes_read: usize = 0;
            loop {
                let bytes_read = client.read(&mut buf)?;
                if bytes_read == 0 {
                    break;
                }
                total_bytes_read += bytes_read;
                //info!(target: function_name!(), "Read {} bytes from the ota request, in total {}: {:?}", bytes_read, total_bytes_read, &buf[0..bytes_read]);
                
                ota_update.write(&buf[0..bytes_read])?;
                //info!(target: function_name!(), "Written into the updater.");   
                debug!(".");
            }

            info!(target: function_name!(), "Reading and writing done: {} bytes", total_bytes_read);

            let finisher = ota_update.finish()?;
            info!(target: function_name!(), "Finished.");
            
            finisher.activate()?;
            info!(target: function_name!(), "Activated.");
            
            return Ok(());
        }
        trace!(target: function_name!(), "Inside ota keep-alive loop.");
        std::thread::sleep(core::time::Duration::from_millis(2000));
    }
}