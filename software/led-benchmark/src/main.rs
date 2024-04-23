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
//
// -----
//
// This project is used to drive LEDs using different driver circuits and PWM generators,
// and benchmark several properties. See the README.md for mor information.

use std::cell::RefCell;

use esp_idf_sys::{esp, esp_vfs_dev_uart_use_driver, uart_driver_install};
use esp_idf_sys::ESP_FAIL;

use esp_idf_hal::{
    prelude::*,
    gpio::*,
    peripherals::Peripherals,
    i2c::{I2cConfig, I2cDriver},
    ledc::{LedcDriver, LedcTimerDriver, config::TimerConfig},
};

use embedded_hal::pwm::SetDutyCycle;

use embedded_hal_bus::{
    i2c as i2c_bus,
};

use std::{
    io::{stdin, stdout},
    time::Duration,
    ptr::null_mut,
    thread,
};

use veml6040::wrapper::AutoVeml6040;

use lm75::{Lm75, Address};

extern crate ina219_rs as ina219;

use ina219::ina219::{INA219, Calibration};

fn main() -> anyhow::Result<()>  {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Needed to accept input via serial connection:
    unsafe {   
        esp!(uart_driver_install(0, 512, 512, 10, null_mut(), 0)).unwrap();
        esp_vfs_dev_uart_use_driver(0);
    }

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    run_benchmark()?;

    Ok(())
}


fn run_benchmark() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;

    println!("Initialize the LED and LEDC for direct PWM output...");
    let mut led_pin = peripherals.pins.gpio11;    
    let ledc = peripherals.ledc;
    let mut timer_driver: LedcTimerDriver<'_> = LedcTimerDriver::new(
        ledc.timer0, 
        &TimerConfig::default().frequency(2400.Hz().into()).resolution(esp_idf_hal::ledc::Resolution::Bits8)
    ).expect("Get LEDC timer.");

    let mut driver_0 = LedcDriver::new(ledc.channel0, &timer_driver, led_pin ).expect("Get LEDC driver.");
    println!("LED and LEDC is initialized.");

    
    println!("Initialize I2C bus...");
    let scl: AnyIOPin = peripherals.pins.gpio22.into();
    let sda: AnyIOPin = peripherals.pins.gpio23.into();
    let config = I2cConfig::new().baudrate(100.kHz().into()).scl_enable_pullup(false).sda_enable_pullup(false);
    let mut i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &config)?;

    // Perform a quick bus scan. I currently can't put this into a method and/or call it 
    // at a later time, because... complicated. Let me explain:
    // - The scan needs the timeout parameter to work, which is not a 
    //   standard feature of the i2c trait, but an ESP-specific addition.
    // - To access multiple devices on the I2C bus, we need to wrap the bus 
    //   in a RefCell and then create a RefCellDevice for each driver.
    // - But RefCellDevice does not offer the timeout parameter
    // - So we make the scan on the raw i2c before wrapping it into RefCell
    for addr in 1..=127 {
        //info!(target: function_name!(), "Scanning Address {}", addr as u8);

        // Scan Address
        let res = i2c.read(addr as u8, &mut [0], 100);

        // Check and Print Result
        match res {
            Ok(_) => 
                println!("Address {}: found something!", addr as u8),
            Err(e) if e.code() == ESP_FAIL => 
                {}, // Usual error if nothing is present
            Err(e)
                => println!("Address {}: error: {}", addr as u8, e),
        }
    }

    let i2c_ref_cell = RefCell::new(i2c);
    println!("I2C bus is initialized.");


    println!("Initialize light sensor...");
    let mut light_sensor = AutoVeml6040::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell))?;
    println!("Light sensor is initialized.");

    println!("Initialize temperature sensor...");
    let tmp1075_address = Address::from(77);
    let mut tempeature_sensor = Lm75::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell), tmp1075_address);
    println!("Temperature sensor is initialized.");

    println!("Initialize power sensor...");
    //let ina_options = Opts::new(64, 100 * physic::MilliOhm, 1 * physic::Ampere);
    //let ina_options = Opts::default();
    let mut ina = INA219::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell));
    ina.init(Calibration::Calibration_32V_2A).unwrap();
    println!("Power sensor is initialized.");


    for dc in [2,3,4,5,6,7,8,9,10,15,20,30,40,60,80,120,160,200,250] {
        for frequency in [350, 400, 600, 800, 1000, 1200, 1400, 1600, 1800, 2000, 2200, 2400, 2600, 2800, 3000, 3500, 4000, 4500, 5000, 6000, 8000, 10000]{
            timer_driver.set_frequency(Hertz(frequency as u32));
            driver_0.set_duty_cycle(dc);

            thread::sleep(Duration::from_millis(10));
        
            let mut sum: f32 = 0.0;
            let mut count: f32 = 0.0;
            for i in 1..10 {
                let result = light_sensor.read_absolute_retry();
                if let Ok(measurement) = result {
                    sum += measurement.white;
                    count += 1.0;
                }
                if i > 3 && count < 2.0 || i > 6 && count < 4.0 {
                    break;
                }
            }


            let temperature = match tempeature_sensor.read_temperature() {
                Ok(temp_celsius) => temp_celsius,
                Err(_) => 0.0
            };

            let brightness = if count > 7.0 {
                sum / count
            } else {
                0.0
            };

            let voltage = ina.getBusVoltage_V().unwrap();
            let current = ina.getCurrent_mA().unwrap();

            println!("{}; {}; {}; {}; {} V; {} A", frequency, dc, brightness, temperature, voltage, current);
            
            thread::sleep(Duration::from_millis(10));
        }
    }

    println!("Benchmark is done.");
    Ok(())
}