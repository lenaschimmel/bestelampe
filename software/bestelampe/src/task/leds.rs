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

use esp_idf_hal::{
    prelude::*,
    gpio::AnyIOPin,
    ledc::{LedcDriver, LedcTimerDriver, LEDC, config::TimerConfig},
};

use prisma::Lerp;

use crate::pwm::Pwm;

#[named]
pub fn test_leds(
    ledc: LEDC,
    pin_r:  AnyIOPin,
    pin_g:  AnyIOPin,
    pin_b:  AnyIOPin,
    pin_cw: AnyIOPin,
    pin_ww: AnyIOPin,
    pin_a:  AnyIOPin,
    light_temperature_target: Arc<RwLock<f32>>,
    light_brightness_target: Arc<RwLock<f32>>,
    light_dim_speed: Arc<RwLock<f32>>,
) -> Result<()> {

    // FIXME For ESP32-C6 `Resolution::Bits14` is the largest enum that is defined. But the C6 supports resolutions up to Bits20.
    // I'd like to use Bits16 and 1000 Hz here, which should be okay.
    let timer_driver: LedcTimerDriver<'_> = LedcTimerDriver::new(
        ledc.timer0, 
        &TimerConfig::default().frequency(2400.Hz().into()).resolution(esp_idf_hal::ledc::Resolution::Bits14)
    ).expect("Get LEDC timer.");
    
    let driver_0 = LedcDriver::new(ledc.channel0, &timer_driver, pin_r ).expect("Get LEDC driver.");
    let driver_1 = LedcDriver::new(ledc.channel1, &timer_driver, pin_g ).expect("Get LEDC driver.");
    let driver_2 = LedcDriver::new(ledc.channel2, &timer_driver, pin_b ).expect("Get LEDC driver.");
    let driver_3 = LedcDriver::new(ledc.channel3, &timer_driver, pin_cw).expect("Get LEDC driver.");
    let driver_4 = LedcDriver::new(ledc.channel4, &timer_driver, pin_ww).expect("Get LEDC driver.");
    let driver_5 = LedcDriver::new(ledc.channel5, &timer_driver, pin_a ).expect("Get LEDC driver.");

    info!(target: function_name!(), "Before LED main loop...");
    std::thread::sleep(core::time::Duration::from_millis(50));
    
    let mut time: f32 = 0.0;
    let mut pwm = Pwm::new(
        driver_0,
        driver_1,
        driver_2,
        driver_3,
        driver_4,
        driver_5,
    )?;
    
    let (mut target_brightness, mut target_temperature, mut dim_speed) = {(
        *(light_brightness_target.read().unwrap()),
        *(light_temperature_target.read().unwrap()),
        *(light_dim_speed.read().unwrap()),

    )};
    let (mut brightness, mut temperature) = (target_brightness, target_temperature);

    let mut count: i32 = 0;
    loop {
        std::thread::sleep(core::time::Duration::from_millis(50));
        time += 50.0;
        count += 1;
        
        {
            target_brightness = *(light_brightness_target.read().unwrap());
            target_temperature = *(light_temperature_target.read().unwrap());
            dim_speed = *(light_dim_speed.read().unwrap());
        }
        
        brightness = brightness.lerp(&target_brightness, dim_speed);
        temperature = temperature.lerp(&target_temperature, dim_speed);

        if count % 40 == 0 {
            info!(target: function_name!(), "Current temp: {}, brightness: {}", temperature, brightness);
        }

        pwm.set_temperature_and_brightness(temperature, brightness)?;
    }
    
}