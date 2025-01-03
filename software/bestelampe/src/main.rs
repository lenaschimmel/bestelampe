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
    let light_temperature_target: Arc<RwLock<f32>> = Arc::new(RwLock::new(3000.0));
    let light_brightness_target: Arc<RwLock<f32>> = Arc::new(RwLock::new(2.0));
    let light_dim_speed: Arc<RwLock<f32>> = Arc::new(RwLock::new(0.01));
    let update_requested: Arc<RwLock<bool>> = Arc::new(RwLock::new(false));

    let peripherals: Peripherals = Peripherals::take().expect("Need Peripherals.");

    // Light sensor
    let i2c = peripherals.i2c0;
    let _light_sensor_thread = thread::spawn(|| {
        test_light_sensor(i2c, peripherals.pins.gpio6.into(), peripherals.pins.gpio7.into()).unwrap_or_default();
        error!(target: function_name!(), "Light sensor has ended :(");
    // Pin configuration
    


    // I2C
    // GPIO 20 is "Neopixel_i2c_power" on Adafruit Feather ESP32-C6 
    #[cfg(feature = "feather-adafruit")] {
        let pin_i2c_pwr : AnyIOPin = peripherals.pins.gpio20.into();
        let mut out_i2c_pwr = PinDriver::output(pin_i2c_pwr).unwrap();
        out_i2c_pwr.set_high();
    }

    let pin_cfg_scl: AnyIOPin = 
        if cfg!(feature = "feather-adafruit") { 
            peripherals.pins.gpio18.into() 
        } else /*if cfg!(feature = "feather-sparkfun")*/ {
            peripherals.pins.gpio7.into()
        };

    let pin_cfg_sda: AnyIOPin = 
        if cfg!(feature = "feather-adafruit") { 
            peripherals.pins.gpio19.into() 
        } else /*if cfg!(feature = "feather-sparkfun")*/ {
            peripherals.pins.gpio6.into()
        };

    let thermal_for_i2c = thermal.clone();
    let voltage_for_i2c = voltage.clone();
    let current_for_i2c = current.clone();
    let time_offset_for_i2c = time_offset.clone();
    let i2c = peripherals.i2c0;
    let _i2c_thread = thread::spawn(|| {
        test_i2c(
            i2c, 
            pin_cfg_scl, 
            pin_cfg_sda, 
            thermal_for_i2c, 
            voltage_for_i2c, 
            current_for_i2c,
            time_offset_for_i2c,
        ).unwrap_or_default();
        error!(target: function_name!(), "I2C thread has ended :(");
    });

    // Buttons
    let light_brightness_target_clone_for_buttons = light_brightness_target.clone();
    let light_temperature_target_clone_for_buttons = light_temperature_target.clone();
    let light_dim_speed_for_buttons = light_dim_speed.clone();
    let _button_thread = thread::spawn(|| {
        let pin_a : AnyIOPin = peripherals.pins.gpio22.into();
        let pin_b : AnyIOPin = peripherals.pins.gpio23.into();
        let pin_c : AnyIOPin = peripherals.pins.gpio0.into();
        test_buttons(pin_a, pin_b, pin_c, light_brightness_target_clone_for_buttons, light_temperature_target_clone_for_buttons, light_dim_speed_for_buttons).unwrap_or_default();
        error!(target: function_name!(), "Button thread has ended :(");
    });

   // Temperature sensor
    let _thermal_thread = thread::spawn(|| {
        let pin_driver = PinDriver::input_output(peripherals.pins.gpio15).expect("Should be able to take Gpio15 for temperature measurement.");
        test_thermal_sensor(pin_driver, thermal);
    });

    // Presence sensor
    let light_brightness_target_clone_for_presence = light_brightness_target.clone();
    let _presence_thread = thread::spawn(|| {
        test_presence_sensor(
            peripherals.pins.gpio17.into(), 
            peripherals.pins.gpio16.into(), 
            peripherals.uart1,
            light_brightness_target_clone_for_presence).unwrap();
        warn!("Presence senspr has ended :(");
    });
    
    // LED control
    let light_temperature_target_clone = light_temperature_target.clone();
    let light_brightness_target_clone = light_brightness_target.clone();
    let light_dim_speed_clone = light_dim_speed.clone();
    let _led_thread = thread::spawn(|| {
        let ledc = peripherals.ledc;

        if cfg!(feature = "feather-adafruit") { 
            test_leds(
                ledc,
                peripherals.pins.gpio22.into(),
                peripherals.pins.gpio2.into(),
                peripherals.pins.gpio6.into(),
                peripherals.pins.gpio5.into(),
                peripherals.pins.gpio21.into(),
                peripherals.pins.gpio1.into(),
                peripherals.pins.gpio4.into(),
                peripherals.pins.gpio3.into(),
                light_temperature_target_clone,
                light_brightness_target_clone,
                light_dim_speed_clone
            ).expect("LEDs should just work.");
        } else /*if cfg!(feature = "feather-sparkfun")*/ {
            test_leds(
                ledc,
                peripherals.pins.gpio20.into(),
                peripherals.pins.gpio5.into(),
                peripherals.pins.gpio2.into(),
                peripherals.pins.gpio3.into(),
                peripherals.pins.gpio19.into(),
                peripherals.pins.gpio0.into(),
                peripherals.pins.gpio1.into(),
                peripherals.pins.gpio4.into(),
                light_temperature_target_clone,
                light_brightness_target_clone,
                light_dim_speed_clone
            ).expect("LEDs should just work.");
        };
    });

    // OTA
    let update_requested_clone = update_requested.clone();
    let _ota_thread = thread::spawn(|| {
        if let Err(err) = test_ota(update_requested_clone) {
            println!("test_ota returned {:#?}", err);
        }
    });

    // Wifi & web interface server
    let _wifi_thread = thread::spawn(|| {
        start_wifi(peripherals.modem, CONFIG.wifi_ap_active).unwrap();
        let _sntp = sntp::EspSntp::new_default().unwrap();
        info!(target: function_name!(), "SNTP initialized");
        run_server(light_temperature_target, light_brightness_target, light_dim_speed, update_requested).unwrap();
    });

    let tz: Tz = CONFIG.time_zone.parse().unwrap();


    let led_en_pin : AnyIOPin = if cfg!(feature = "feather-adafruit") { 
        peripherals.pins.gpio14.into() 
    } else /*if cfg!(feature = "feather-sparkfun")*/ {
        peripherals.pins.gpio22.into()
    };

    let mut led_en = PinDriver::output(led_en_pin).unwrap();
    led_en.set_high(); // high = disabled

    // Keep the main thread alive
    info!(target: function_name!(), "Entering infinite loop in main thread...");
    loop {
        let now = Utc::now();
        let local_now = now.with_timezone(&tz);
        trace!("Current time: {:?}", local_now);
        std::thread::sleep(core::time::Duration::from_millis(1000));
    }
}
