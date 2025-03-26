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

use std::error::Error;

use crate::prelude::*;

use esp_idf_hal::{
    prelude::*,
    gpio::AnyIOPin, i2c::*
};

use veml6040::wrapper::AutoVeml6040;

#[named]
pub fn test_light_sensor(i2c: I2C0, scl: AnyIOPin, sda: AnyIOPin) -> Result<(), Box<dyn Error>> {
    let config = I2cConfig::new().baudrate(100.kHz().into()).scl_enable_pullup(false).sda_enable_pullup(false);
    let i2c = I2cDriver::new(i2c, sda, scl, &config)?;

    info!(target: function_name!(), "Creating the Veml device...");

    let mut sensor_wrapper = AutoVeml6040::new(i2c)?;

    info!(target: function_name!(), "Sensor initialized, starting to read values continuously...");
    loop {
        let result = sensor_wrapper.read_absolute_retry();
        match result {
            Ok(measurement) => {
                let ccti = (measurement.red as f32 - measurement.blue as f32) / (measurement.green as f32) + 0.5;
                let cct = 4278.6 * ccti.powf(-1.2455);
                // TODO it seems strange to use `measurement.green` as the brightness, but I think I got that
                // from the datasheet or some official example. Anyway, I should try using `measurement.white``
                // instead at some time.
                info!(target: function_name!(), "Brightness: {} lx, color temperature: {} K", measurement.green, cct);
            }
            Err(err) => {
                warn!(target: function_name!(), "Error in sensor loop iteration: {:?}", err);
                std::thread::sleep(core::time::Duration::from_millis(750));
            }
        }
    }
    
}
