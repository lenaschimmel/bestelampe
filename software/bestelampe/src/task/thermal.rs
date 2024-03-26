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

use std::sync::{Arc, RwLock};

use ds18b20::{Ds18b20, Resolution};
use esp_idf_hal::{delay::Delay, gpio::{InputOutput, Pin, PinDriver}};
use esp_idf_sys::EspError;
use one_wire_bus::{OneWire, OneWireError};

#[named]
pub fn test_thermal_sensor<PinType: Pin>(one_wire_pin: PinDriver<'_, PinType, InputOutput>, thermal: Arc<RwLock<f32>>)  -> ! {
    info!(target: function_name!(), "Before temperature sensor init...");
    let mut delay = Delay::new_default();

    let mut one_wire_bus = OneWire::new(one_wire_pin).unwrap();

    // initiate a temperature measurement for all connected devices
    ds18b20::start_simultaneous_temp_measurement(&mut one_wire_bus, &mut delay).expect("Start temp measurement");

    // wait until the measurement is done. This depends on the resolution you specified
    // If you don't know the resolution, you can obtain it from reading the sensor data,
    // or just wait the longest time, which is the 12-bit resolution (750ms)
    
    // Don't block, use non-blocking sleep instead
    // Resolution::Bits12.delay_for_measurement_time(&mut delay);
    std::thread::sleep(core::time::Duration::from_millis(Resolution::Bits12.max_measurement_time_millis() as u64));

    let mut sensors: Vec<Ds18b20> = Vec::new();

    info!(target: function_name!(), "Before temperature sensor search loop...");
    // iterate over all the devices, and report their temperature
    let mut search_state = None;
    loop {
        if let Ok(Some((device_address, state))) = one_wire_bus.device_search(search_state.as_ref(), false, &mut delay) {
            search_state = Some(state);
            if device_address.family_code() != ds18b20::FAMILY_CODE {
                // skip other devices
                continue; 
            }
            // You will generally create the sensor once, and save it for later
            let sensor = Ds18b20::new::<OneWireError<EspError>>(device_address).expect("Create device by address");
            
            // contains the read temperature, as well as config info such as the resolution used
            match sensor.read_data(&mut one_wire_bus, &mut delay) {
                Ok(sensor_data) => {
                    info!(target: function_name!(), "Device at {:?} is {}°C. Resolution is {:?}.", device_address, sensor_data.temperature, sensor_data.resolution);
                    sensors.push(sensor);
                },
                Err(e) => warn!(target: function_name!(), "Error reading sensor data for the first time: {:?}", e),
            }
            
        } else {
            break;
        }
    }

    info!(target: function_name!(), "Before continuos temperature sensor measurement loop...");
    loop {
        // Start all measurements
        for sensor in &sensors {
            sensor.start_temp_measurement(&mut one_wire_bus, &mut delay).expect("Start single measurement");
        }
        // Wait for the longest possible measurement time
        //std::thread::sleep(core::time::Duration::from_millis(Resolution::Bits12.max_measurement_time_millis() as u64));

        // Wait two seconds. We don't need more frequent measurements.
        std::thread::sleep(core::time::Duration::from_millis(2000));

        // Read all measurements
        let mut thermal_write = thermal.write().unwrap();
        for sensor in &sensors {
            match sensor.read_data(&mut one_wire_bus, &mut delay) {
                Ok(sensor_data) => {
                    *thermal_write = sensor_data.temperature;
                    info!(target: function_name!(), "Device at {:?} is {}°C.", sensor.address(), sensor_data.temperature);
                },
                Err(e) => warn!(target: function_name!(), "Error while reading thermal temperature: {:?}", e),
            }
        }
    }
}

