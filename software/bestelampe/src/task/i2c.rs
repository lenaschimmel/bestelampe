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

use std::{error::Error, thread};
use std::cell::RefCell;
use std::sync::{Arc, RwLock};

use crate::prelude::*;

use chrono::{DateTime, FixedOffset, TimeZone};
use esp_idf_hal::{
    prelude::*,
    gpio::AnyIOPin, i2c::*,
};
use esp_idf_sys::{EspError, ESP_FAIL};
use embedded_hal::digital::OutputPin;

use lm75::{Lm75, Address};

// use ina219::{INA219, Calibration};
use ina219::address::Address as InaAddress;
use ina219::calibration::{IntCalibration, MicroAmpere};
use ina219::SyncIna219;

use embedded_hal_bus::i2c as i2c_bus;

use embedded_graphics::{
    mono_font::{iso_8859_1::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
use microchip_24aa02e48::Microchip24AA02E48;
use ssd1306::{mode::BufferedGraphicsMode, prelude::*, I2CDisplayInterface, Ssd1306};

use tinyqoi::Qoi;
use embedded_graphics::{prelude::*, image::Image,  primitives::{Rectangle, PrimitiveStyleBuilder},};

use port_expander::dev::pi4ioe5v6408::Pi4ioe5v6408;
use port_expander::dev::tca6408a::Tca6408a;

use chrono::Utc;
use chrono::LocalResult::Single;

use veml6040::wrapper::AutoVeml6040;

#[named]
pub fn test_i2c(
    i2c: I2C0, 
    scl: AnyIOPin, 
    sda: AnyIOPin, 
    thermal: Arc<RwLock<f32>>, 
    voltage: Arc<RwLock<f32>>, 
    current: Arc<RwLock<f32>>,
    time_offset: Arc<RwLock<i64>>,
) -> Result<(), Box<dyn Error>> {
    let config = I2cConfig::new().baudrate(100.kHz().into()).scl_enable_pullup(true).sda_enable_pullup(true);
    let mut i2c = I2cDriver::new(i2c, sda, scl, &config)?;

    info!(target: function_name!(), "Sleeping a while so that other messages can scroll by...");
    std::thread::sleep(core::time::Duration::from_millis(1500));
    
    info!(target: function_name!(), "Start device scan...");

    // Start Scan at Address 1 going up to 127
    for addr in 1..=127 {
        //info!(target: function_name!(), "Scanning Address {}", addr as u8);
        info!(target: function_name!(), "Try to read address {}...", addr as u8);
        // Scan Address
        let res = i2c.read(addr as u8, &mut [0], 1000);
        // Notes:
        // - if no device is present, ESP_FAIL will be returned
        // - the VEML6040 (addr. hex10) will also return an error, but it's TIMEOUT
        // - Most other I2C devices that I tested return Ok (BME260, INA219, TMP1075)
        // - Tca6408a will return ESP_FAIL. It needs an 

        // Decimal addresses
        // - 16 VEML6040 light sensor
        // - 64..95 TMP1075 temperature sensor
        // - 64..79 INA219 power sensor
        // - 80..87 24AA025E EEPROM
        // - 118..119 BME680 

        // Check and Print Result
        match res {
            Ok(_) => info!(target: function_name!(), "Address {}: found something!", addr as u8),
            Err(e) if e.code() == ESP_FAIL => {}, // Usual error if nothing is present
            Err(e) => info!(target: function_name!(), "Address {}: error: {}", addr as u8, e),
        }
    }

    info!(target: function_name!(), "Sleeping a bit in case you want to stop the output...");
    std::thread::sleep(core::time::Duration::from_millis(1500));

    info!(target: function_name!(), "Specifically seatch for GPIO-Expanders: Address 33");
    let res = i2c.write_read(33u8, &[0u8],  &mut [0], 1000);
    match res {
        Ok(_) => info!(target: function_name!(), "Address 33: found something!"),
        Err(e) if e.code() == ESP_FAIL => info!(target: function_name!(), "Address 33: FAIL!"),
        Err(e) => info!(target: function_name!(), "Address 33: error: {}", e),
    }

    info!(target: function_name!(), "Specifically seatch for GPIO-Expanders: Address 32");
    let res = i2c.write_read(32u8, &[0u8],  &mut [0], 1000);
    match res {
        Ok(_) => info!(target: function_name!(), "Address 32: found something!"),
        Err(e) if e.code() == ESP_FAIL => info!(target: function_name!(), "Address 32: FAIL!"),
        Err(e) => info!(target: function_name!(), "Address 32: error: {}", e),
    }



    let i2c_ref_cell = RefCell::new(i2c);
    println!("I2C bus (for multiple devices) is initialized.");

   
    


    info!(target: function_name!(), "Initialize temperature sensors...");
    let mut tmp1075_power = Lm75::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell), Address::from(79));
    let mut tmp1075_led = Lm75::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell), Address::from(75));

    tmp1075_led.set_os_temperature(90.1).unwrap();
    tmp1075_led.set_hysteresis_temperature(70.0).unwrap();
    tmp1075_led.set_fault_queue(lm75::FaultQueue::_2);
    tmp1075_led.set_os_polarity(lm75::OsPolarity::ActiveHigh);

    tmp1075_power.set_os_temperature(50.0).unwrap();
    tmp1075_power.set_hysteresis_temperature(40.0).unwrap();
    tmp1075_power.set_fault_queue(lm75::FaultQueue::_2);
    tmp1075_power.set_os_polarity(lm75::OsPolarity::ActiveHigh);

    info!(target: function_name!(), "Initialize power sensors...");
    //let ina_options = Opts::new(64, 100 * physic::MilliOhm, 1 * physic::Ampere);
    //let ina_options = Opts::default();
    
    // Using the forked driver:
    //let mut ina = INA219::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell));
    //ina.init(Calibration::Calibration_32V_2A).unwrap();
    
    // Using the original driver:
    // Resolution of 1A, and a shunt of 1mOhm
    let calib = IntCalibration::new(MicroAmpere(10_000), 2_000).unwrap();

    //let ina_address: u8 = 64; // 20W power supply
    let ina_address: u8 = 65; // 30W power supply
    //let ina_address: u8 = 69; // controller board (optional)
    let mut ina_power_30   = SyncIna219::new_calibrated(i2c_bus::RefCellDevice::new(&i2c_ref_cell), InaAddress::from_byte(ina_address).unwrap(), calib).unwrap();
    //let mut ina_controller = SyncIna219::new_calibrated(i2c_bus::RefCellDevice::new(&i2c_ref_cell), InaAddress::from_byte(69).unwrap(), calib).unwrap();
    info!(target: function_name!(), "Power sensors is initialized.");

    info!(target: function_name!(), "Try to initialize 3.3V port expander...");
    let mut fx_3 = Tca6408a::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell), false);
    let fx_3_pins = fx_3.split();
    let mut fx_3_0 = fx_3_pins.io0.into_output().unwrap();
    let mut fx_3_1 = fx_3_pins.io1.into_output().unwrap();
    let mut fx_3_5 = fx_3_pins.io5.into_output().unwrap();
    fx_3_5.set_high().unwrap();
    info!(target: function_name!(), "3.3V port expander is initialized.");

    info!(target: function_name!(), "Try to initialize 5V port expander...");
    let mut fx_5 = Tca6408a::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell), true);
    let fx_5_pins = fx_5.split();
    let mut fx_5_6 = fx_5_pins.io6.into_output().unwrap();
    let mut fx_5_7 = fx_5_pins.io7.into_output().unwrap();
    info!(target: function_name!(), "5V port expander is initialized.");
    fx_5_6.set_low().unwrap(); // power for left presence sensor, inverted
    fx_5_7.set_low().unwrap(); // power for right presence sensor, inverted
    info!(target: function_name!(), "Enabled power supply for right presence sensor (actually GPS), and left presence sensor");


    info!(target: function_name!(), "Creating the VEML device...");
    let mut veml = AutoVeml6040::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell))?;
    info!(target: function_name!(), "VEML initialized..");
    
    // EEPROM is not usable with and on LED board (<= v2.0) and new power board (>= 1.2) because both EEPROMS will react on address 82
    // info!(target: function_name!(), "Creating EEPROM device...");
    // let mut eeprom = Microchip24AA02E48::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell)).unwrap();
    // let mut uid = [0u8; 6];
    // eeprom.read_eui48(&mut uid);
    // info!(target: function_name!(), "EEPROM initialized. UID: {:?}", uid);

    // let mut data = [0u8; 8];
    // eeprom.read(0, &mut data).unwrap();

    // info!(target: function_name!(), "Read first 8 bytes from EEPROM: {:?}", data);

    // data = [1u8, 2u8, 3u8, 1u8, 3u8, 1u8, 2u8, 0u8];
    // eeprom.write(0, &data).unwrap();;
    // info!(target: function_name!(), "Written some stuff to the EEPROM");


    info!(target: function_name!(), "Try to test the display...");

    let interface = I2CDisplayInterface::new_alternate_address(i2c_bus::RefCellDevice::new(&i2c_ref_cell));
   
    // let mut display = Ssd1306::new(
    //     interface,
    //     DisplaySize128x64,
    //     DisplayRotation::Rotate0,
    // ).into_buffered_graphics_mode();
    // display.init().unwrap();
    // info!(target: function_name!(), "Display initialized. Try to draw image.");

    // // Parse QOI image.
    // let data = include_bytes!("../../bestelampe.qoi");
    // let qoi = Qoi::new(data).unwrap();

    // Image::new(&qoi, Point::zero()).draw(&mut display.color_converted()).unwrap();

    // let text_style = MonoTextStyleBuilder::new()
    //     .font(&FONT_6X10)
    //     .text_color(BinaryColor::On)
    //     .build();

    // Text::with_baseline("Hello world!", Point::zero(), text_style, Baseline::Top)
    //     .draw(&mut display)
    //     .unwrap();

    // let mut text_tmp = Text::with_baseline("Temp:", Point::new(0, 53), text_style, Baseline::Top);
    // text_tmp.draw(&mut display).unwrap();
    // display.flush().unwrap();

    // let style = PrimitiveStyleBuilder::new()
    // // .stroke_color(BinaryColor::Off)
    // // .stroke_width(3)
    // .fill_color(BinaryColor::Off)
    // .build();

    // let rect = Rectangle::new(Point::new(0, 0), Size::new(127, 63)).into_styled(style);
   
    let german_time_zone = FixedOffset::east_opt(3600).unwrap();
    info!(target: function_name!(), "Start sensor reading...");

    let mut count: u8 = 0;
    loop {
        //rect.draw(&mut display).unwrap();

        match(tmp1075_power.read_temperature()) {
            Ok(temp_celsius) => { 
                // let mut thermal_write = thermal.write().unwrap();
                // *thermal_write = temp_celsius;
                let text_cnt = format!("Temp Power: {}°C", temp_celsius);
                info!(target: function_name!(), "Temp Power: {}°C", temp_celsius);
                // let text_tmp = Text::with_baseline(text_cnt.as_str(), Point::new(0, 10), text_style, Baseline::Top);
                // text_tmp.draw(&mut display).unwrap();
            },
            Err(_) => {} // println!("Could not read temp1075_power.")
        }

        match(tmp1075_led.read_temperature()) {
            Ok(temp_celsius) => { 
                let mut thermal_write = thermal.write().unwrap();
                *thermal_write = temp_celsius;
                let text_cnt = format!("Temp LED: {}°C", temp_celsius);
                info!(target: function_name!(), "Temp LED: {}°C", temp_celsius);
                // let text_tmp = Text::with_baseline(text_cnt.as_str(), Point::new(0, 20), text_style, Baseline::Top);
                // text_tmp.draw(&mut display).unwrap();
            },
            Err(_) => println!("Could not read temp1075_led.")
        }

        count = (count + 1) % 4;

        // if (fx_3_0.is_high().unwrap()) {
        //     text_tmp = Text::with_baseline("0", Point::new(64, 53), text_style, Baseline::Top);
        //     text_tmp.draw(&mut display).unwrap();
        // }
        // if (fx_3_1.is_high().unwrap()) {
        //     text_tmp = Text::with_baseline(" 1", Point::new(64, 53), text_style, Baseline::Top);
        //     text_tmp.draw(&mut display).unwrap();
        // }

        // if (count == 0) {
        //     // pull EN low after 4 seconds
        //     fx_3_5.set_low().unwrap();
        // }

        // fx_3_0.set_state((count & 1 == 0).into()).unwrap();
        // fx_3_1.set_state((count & 2 == 0).into()).unwrap();

        //let ina_measurement_controller = ina_controller.next_measurement().expect("An ina measurement is ready (controller)");
        let mut ina_measurement_power_30 = ina_power_30.next_measurement().expect("An ina measurement is ready (power 30)");

        // if let Some(measure) = ina_measurement_controller {
        //     let power = (measure.bus_voltage.voltage_mv() as f32 / 1000.0) * (measure.current.0 as f32 / 1_000_000.0);
        //     let text_cnt = format!("Power USB: {} W", power);
        //     let text_pwr = Text::with_baseline(text_cnt.as_str(), Point::new(0, 30), text_style, Baseline::Top);
        //     text_pwr.draw(&mut display).unwrap();
        // }

        if let Some(measure) = ina_measurement_power_30 {
            let mut power_sum = (measure.bus_voltage.voltage_mv() as f32 / 1000.0) * (measure.current.0 as f32 / 1_000_000.0);
            let mut voltage_sum = (measure.bus_voltage.voltage_mv() as f32 / 1000.0);
            let mut power_cnt = 1u8;
            // This sensor is availble in general, now try to make 5 measurements
            for i in 0..5 {
                ina_measurement_power_30 = ina_power_30.next_measurement().expect("An ina measurement is ready (power 30)");
                if let Some(loop_measure) = ina_measurement_power_30 {
                    power_sum += (loop_measure.bus_voltage.voltage_mv() as f32 / 1000.0) * (loop_measure.current.0 as f32 / 1_000_000.0);
                    voltage_sum += (loop_measure.bus_voltage.voltage_mv() as f32 / 1000.0);
                    power_cnt += 1u8;
                }
            }
            
            let text_cnt = format!("AC: {:.02}W, {:.02}V ({} S.)", power_sum / (power_cnt as f32), voltage_sum / (power_cnt as f32), power_cnt);
            info!(target: function_name!(), "Power - {}", text_cnt);

            //let text_pwr = Text::with_baseline(text_cnt.as_str(), Point::new(0, 40), text_style, Baseline::Top);
            //text_pwr.draw(&mut display).unwrap();
        }

        //println!("Ina: {:#?}", ina_measurement);

        // let mut voltage_write = voltage.write().unwrap();
        // *voltage_write = (ina_measurement.unwrap().bus_voltage.voltage_mv() as f32) / 1000.0;

        // let mut current_write = current.write().unwrap();
        // *current_write = ina_measurement.unwrap().current.0 as f32 / 1_000_000.0;

        // voltage = ina_measurement.shunt_voltage.voltage_mv();
        // current = ina_measurement.current
        // info!(target: function_name!(), "Voltage: {} V, Current: {} A, Power: {} W", voltage, current, voltage * current);

        let computed_timestamp = Utc::now().timestamp() + *time_offset.read().unwrap();
        match Utc.timestamp_opt(computed_timestamp, 0) {
            (Single(computed_time)) => {
                let text_cnt = format!("{}", computed_time.with_timezone(&german_time_zone).format("%H:%M:%S"));
                //info!(target: function_name!(), "Try to write time to the display: {}", text_cnt);
                // let text_tmp = Text::with_baseline(text_cnt.as_str(), Point::new(0, 53), text_style, Baseline::Top);
                // text_tmp.draw(&mut display).unwrap();
            },
            _ => {}
        }


        let veml_result = veml.read_absolute_retry();
        match veml_result {
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

        //display.flush().unwrap();

        thread::sleep(core::time::Duration::from_millis(1_000));
    }
}
