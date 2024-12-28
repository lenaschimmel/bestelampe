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

use std::thread;
use std::sync::{Arc, RwLock};

use esp_idf_hal::gpio::DriveStrength;
use esp_idf_hal::{
    prelude::*,
    gpio::{AnyIOPin, PinDriver},
};

use esp_idf_svc::{
    log::EspLogger,
    sntp,
};

use ::function_name::named;
use chrono_tz::Tz;
use chrono::Utc;

use log::*;
mod pwm;

mod task;
use crate::task::buttons::test_buttons;
use crate::task::leds::test_leds;
use crate::task::ota::test_ota;
use crate::task::presence::test_presence_sensor;
use crate::task::server::run_server;
use crate::task::thermal::test_thermal_sensor;
use crate::task::wifi::start_wifi;
use crate::task::light_sensor::test_light_sensor;
use crate::task::i2c::test_i2c;
use crate::task::uart::test_uart;

extern crate ina219;

mod config;
use crate::config::CONFIG;

mod prelude {
    pub use log::*;
    pub use ::function_name::named;
    pub use crate::config::CONFIG;
    pub use anyhow::{ Result, anyhow };
}

#[named]
fn main() -> ! {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();
    debug!(target: function_name!(), "Logger initialized.");
    
    // Thread safe globals for communication across tasks
    let thermal: Arc<RwLock<f32>> = Arc::new(RwLock::new(0.0));
    let voltage: Arc<RwLock<f32>> = Arc::new(RwLock::new(0.0));
    let current: Arc<RwLock<f32>> = Arc::new(RwLock::new(0.0));
    let light_temperature_target: Arc<RwLock<f32>> = Arc::new(RwLock::new(3000.0));
    let light_brightness_target: Arc<RwLock<f32>> = Arc::new(RwLock::new(0.001));
    let light_dim_speed: Arc<RwLock<f32>> = Arc::new(RwLock::new(0.01));
    let update_requested: Arc<RwLock<bool>> = Arc::new(RwLock::new(false));
    let time_offset: Arc<RwLock<i64>> = Arc::new(RwLock::new(0));
    
    let peripherals: Peripherals = Peripherals::take().expect("Need Peripherals.");


    // I2C
    let pin_i2c_pwr : AnyIOPin = peripherals.pins.gpio20.into();
    let mut out_i2c_pwr = PinDriver::output(pin_i2c_pwr).unwrap();
    out_i2c_pwr.set_high();
    

    let thermal_for_i2c = thermal.clone();
    let voltage_for_i2c = voltage.clone();
    let current_for_i2c = current.clone();
    let time_offset_for_i2c = time_offset.clone();
    let i2c = peripherals.i2c0;
    let _i2c_thread = thread::spawn(|| {
        test_i2c(
            i2c, 
            peripherals.pins.gpio18.into(), 
            peripherals.pins.gpio19.into(), 
            thermal_for_i2c, 
            voltage_for_i2c, 
            current_for_i2c,
            time_offset_for_i2c,
        ).unwrap_or_default();
        error!(target: function_name!(), "I2C thread has ended :(");
    });

    // // Serial / UART, configured for GPS time
    // let time_offset_for_uart = time_offset.clone();
    // let _uart_thread = thread::spawn(|| {
    //     test_uart(
    //         peripherals.pins.gpio8.into(),
    //         peripherals.pins.gpio0.into(),
    //         peripherals.uart1, time_offset_for_uart
    //     ).unwrap_or_default();
    //     error!(target: function_name!(), "UART thread has ended :(");
    // });

    // // Light sensor
    // let i2c = peripherals.i2c0;
    // let _light_sensor_thread = thread::spawn(|| {
    //     test_light_sensor(i2c, peripherals.pins.gpio6.into(), peripherals.pins.gpio7.into()).unwrap_or_default();
    //     error!(target: function_name!(), "Light sensor has ended :(");
    // });

    // Buttons
    // let light_brightness_target_clone_for_buttons = light_brightness_target.clone();
    // let light_temperature_target_clone_for_buttons = light_temperature_target.clone();
    // let light_dim_speed_for_buttons = light_dim_speed.clone();
    // let _button_thread = thread::spawn(|| {
    //     let pin_a : AnyIOPin = peripherals.pins.gpio22.into();
    //     let pin_b : AnyIOPin = peripherals.pins.gpio23.into();
    //     let pin_c : AnyIOPin = peripherals.pins.gpio0.into();
    //     test_buttons(pin_a, pin_b, pin_c, light_brightness_target_clone_for_buttons, light_temperature_target_clone_for_buttons, light_dim_speed_for_buttons).unwrap_or_default();
    //     error!(target: function_name!(), "Button thread has ended :(");
    // });

//    // Temperature sensor
//     let _thermal_thread = thread::spawn(|| {
//         let pin_driver = PinDriver::input_output(peripherals.pins.gpio15).expect("Should be able to take Gpio15 for temperature measurement.");
//         test_thermal_sensor(pin_driver, thermal);
//     });

    // To use the RMT periferal as a fake UART controller, we could use peripherals.rmt.channel2, // Only channel 2 and 3 can RX

    // // Presence sensor
    let light_brightness_target_clone_for_presence = light_brightness_target.clone();
    let _presence_thread = thread::spawn(|| {
        test_presence_sensor(
            peripherals.pins.gpio17.into(),
            peripherals.pins.gpio16.into(), 
            peripherals.uart1,
            light_brightness_target_clone_for_presence).unwrap();
        warn!("Presence sensor thread has ended :(");
    });
    
    // LED control
    let light_temperature_target_clone = light_temperature_target.clone();
    let light_brightness_target_clone = light_brightness_target.clone();
    let light_dim_speed_clone = light_dim_speed.clone();
    let _led_thread = thread::spawn(|| {
        let ledc = peripherals.ledc;
        let pin_r  : AnyIOPin = peripherals.pins.gpio22.into();  // LED 8
        let pin_g  : AnyIOPin = peripherals.pins.gpio2.into();   // LED 6
        let pin_b  : AnyIOPin = peripherals.pins.gpio6.into();   // LED 3
        let pin_cw : AnyIOPin = peripherals.pins.gpio5.into();   // LED 4 
        let pin_nw : AnyIOPin = peripherals.pins.gpio21.into();   // LED 7
        let pin_ww : AnyIOPin = peripherals.pins.gpio1.into();   // LED 1
        let pin_a  : AnyIOPin = peripherals.pins.gpio4.into();   // LED 2
        let pin_pa : AnyIOPin = peripherals.pins.gpio3.into();   // LED 5
  
        test_leds(
            ledc,
            pin_r,
            pin_g,
            pin_b,
            pin_cw,
            pin_nw,
            pin_ww,
            pin_a,
            pin_pa,
            light_temperature_target_clone,
            light_brightness_target_clone,
            light_dim_speed_clone
        ).expect("LEDs should just work.");
    });

    // // OTA
    // let update_requested_clone = update_requested.clone();
    // let _ota_thread = thread::spawn(|| {
    //     if let Err(err) = test_ota(update_requested_clone) {
    //         println!("test_ota returned {:#?}", err);
    //     }
    // });

    // Wifi & web interface server
    let light_brightness_target_for_server = light_brightness_target.clone();
    let _wifi_thread = thread::spawn(|| {
        start_wifi(peripherals.modem, CONFIG.wifi_ap_active).unwrap();
        let _sntp = sntp::EspSntp::new_default().unwrap();
        info!(target: function_name!(), "SNTP initialized");
        run_server(light_temperature_target, light_brightness_target_for_server, light_dim_speed, update_requested, thermal, voltage, current).unwrap();
    });

    let tz: Tz = CONFIG.time_zone.parse().unwrap();


    let led_en_pin : AnyIOPin = peripherals.pins.gpio14.into();
    let mut led_en = PinDriver::output(led_en_pin).unwrap();
    led_en.set_high(); // high = disabled

    // Keep the main thread alive
    info!(target: function_name!(), "Entering infinite loop in main thread...");
    let mut count: u8 = 0;
    loop {
        let now = Utc::now();
        let local_now = now.with_timezone(&tz);
        //trace!("Current time: {:?}", local_now);
        std::thread::sleep(core::time::Duration::from_millis(1000));
        //info!(target: function_name!(), "LED enable / alert : {:?}", led_en.get_level());
        if(count < 10) {
            count += 1;
            if (count == 4) {
                led_en.set_low(); // low = enabled    
                *light_brightness_target.write().unwrap() = 0.0;
            }
        }
    }
}
