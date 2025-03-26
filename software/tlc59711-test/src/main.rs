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
// This file contains lots of small, quick and dirty methods to try the TLC59711
// hardware, the driver, attached LEDs and the LED Board C. It also has a simple
// text-based serial interface that allows me to control it with a MIDI Controller,
// which makes it easier to test the color mixing. See also the project `midi-light`.

use esp_idf_sys::{esp, esp_vfs_dev_uart_use_driver, uart_driver_install, vTaskDelay};

use esp_idf_hal::{
    prelude::*,
    gpio,
    gpio::*,
    peripherals::Peripherals,
    spi::*,
    spi::config::MODE_0,
    units::*,
    delay::Delay,
    i2c::{I2c, I2cConfig, I2cDriver, I2cError},
    ledc::{LedcDriver, LedcTimerDriver, LEDC, config::TimerConfig},
};

use embedded_hal::pwm::SetDutyCycle;

use fastapprox::faster;
// use num::{
//     clamp,
//     traits::Pow,
// };

//use rand::Rng;
use std::{
    io::{stdin, stdout},
    thread,
    thread::spawn,
    time::Duration,
    ptr::null_mut,
    sync::{
        Arc, 
        RwLock,
    },
};


use tlc59xxx::TLC59711;

use veml6040::wrapper::AutoVeml6040;

fn main() -> anyhow::Result<()>  {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    unsafe {   
        esp!(uart_driver_install(0, 512, 512, 10, null_mut(), 0)).unwrap();
        esp_vfs_dev_uart_use_driver(0);
    }

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    //test_spi()?;
    //test_direct()?;
    //test_spi_multi_color()?;
    test_spi_multi_color_serial()?;
    Ok(())
}


fn test_direct() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let mut led_pin = peripherals.pins.gpio11;
    //led_pin.set_low()?;

    let ledc = peripherals.ledc;

    let mut timer_driver: LedcTimerDriver<'_> = LedcTimerDriver::new(
        ledc.timer0, 
        &TimerConfig::default().frequency(2400.Hz().into()).resolution(esp_idf_hal::ledc::Resolution::Bits8)
    ).expect("Get LEDC timer.");

    let mut driver_0 = LedcDriver::new(ledc.channel0, &timer_driver, led_pin ).expect("Get LEDC driver.");

    println!("LED is LEDC now. Let's connect the light sensor...");

    let scl: AnyIOPin = peripherals.pins.gpio22.into(); // Pin 1 on Ext
    let sda: AnyIOPin = peripherals.pins.gpio23.into(); // Pin 3 (above 1) on Ext

    let config = I2cConfig::new().baudrate(100.kHz().into()).scl_enable_pullup(false).sda_enable_pullup(false);
    let mut i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &config)?;

    let mut sensor_wrapper = AutoVeml6040::new(i2c);
  

    // let on_time: Arc<RwLock<u64>> = Arc::new(RwLock::new(500));
    // let off_time: Arc<RwLock<u64>> = Arc::new(RwLock::new(500));

    // let  on_time_clone =  on_time.clone();
    // let off_time_clone = off_time.clone();
    // spawn(|| {
    //     //echo(on_time_clone, off_time_clone);
    //     auto_measurements(on_time_clone, off_time_clone, sensor_wrapper);
    // });


    // for on_t in 5..20 {
    //     for off_t in 5..20 {

        for dc in [2,3,4,5,6,7,8,9,10,15,20,30,40,60,80,120,160,200,250] {
             for frequency in [350, 400, 600, 800, 1000, 1200, 1400, 1600, 1800, 2000, 2200, 2400, 2600, 2800, 3000, 3500, 4000, 4500, 5000, 6000, 8000, 10000]{
        //for dc in [1,3,5,8,14,60] {
        //    for frequency in [400, 800, 1600, 3200, 6000]{
                    //let frequency = 1_000_000.0 / (on_t + off_t) as f32;

            timer_driver.set_frequency(Hertz(frequency as u32));
            //let dc = ((2 ^ 8 - 1) * (on_t / on_t + off_t)) as u16;
            driver_0.set_duty_cycle(dc);
            //println!("F: {} Hz, Duty cycle: {} / 255", frequency, dc);

            thread::sleep(Duration::from_millis(10));
        
            let mut sum: f32 = 0.0;
            let mut count: f32 = 0.0;
            for i in 1..10 {
                let result = sensor_wrapper.read_absolute_retry();
                if let Ok(measurement) = result {
                    sum += measurement.white;
                    count += 1.0;
                }
                if i > 3 && count < 2.0 || i > 6 && count < 4.0 {
                    break;
                }
            }

            if count > 7.0 {
                println!("{}; {}; {}", frequency, dc, sum / count);
            } else {
                println!("{}; {}; Error", frequency, dc);
            }
            

            thread::sleep(Duration::from_millis(10));
        }
    }

    Ok(())

    // let delay: Delay = Default::default();
    // loop {
    //     let on = *(on_time.read().unwrap());
    //     let off = *(off_time.read().unwrap());

    //     let frequency = 1_000_000.0 / (on + off) as f32;

    //     timer_driver.set_frequency(Hertz(frequency as u32));
    //     let dc = ((2 ^ 8 - 1) * (on / on + off)) as u16;
    //     driver_0.set_duty_cycle(dc);
    //     println!("F: {} Hz, Duty cycle: {} / 255", frequency, dc);
    //     thread::sleep(Duration::from_millis(40));
    //     // led_pin.set_high()?;
    //     // if on > 100 {
    //     //     thread::sleep(Duration::from_micros(on));
    //     // } else {
    //     //     // The impl of delay_us will use different ways for the delay for t > 10µs and t <= 10µs.
    //     //     // My own special handling for t > 100µs might be useless, but should not hurt anyway.
    //     //     delay.delay_us(on as u32);
    //     // }
    //     // led_pin.set_low()?;
    //     // if off > 100 {
    //     //     thread::sleep(Duration::from_micros(off));
    //     // } else {
    //     //     delay.delay_us(off as u32);
    //     // }
    // }
}


fn auto_measurements(
    on_time: Arc<RwLock<u64>>,
    off_time: Arc<RwLock<u64>>,
    mut sensor_wrapper: AutoVeml6040<I2cDriver, I2cError>,
) {
    for on_t in 5..20 {
        for off_t in 5..20 {
            *on_time.write().unwrap() = on_t;
            *off_time.write().unwrap() = off_t;

            thread::sleep(Duration::from_micros(100));
        
            let mut sum: f32 = 0.0;
            let mut count: f32 = 0.0;
            for i in 1..10 {
                let result = sensor_wrapper.read_absolute_retry();
                if let Ok(measurement) = result {
                    sum += measurement.white;
                    count += 1.0;
                }
                if i > 3 && count < 2.0 || i > 6 && count < 4.0 {
                    break;
                }
            }

            if count > 40.0 {
                println!("{}; {}; {}", on_t, off_t, sum / count);
            } else {
                println!("{}; {}; Error", on_t, off_t);
            }
            

            thread::sleep(Duration::from_millis(100));
        }
    }
}

fn echo(
    on_time: Arc<RwLock<u64>>,
    off_time: Arc<RwLock<u64>>,
) {
    let reader = stdin();
    //let mut writer = stdout();

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(_) => {
                let line_trimmed = line.trim_end_matches(&['\r', '\n']);
                if let Some(c) = line_trimmed.chars().nth(0) {
                    let num_str = line_trimmed[1..].trim();
                    if let Ok(num) = num_str.parse::<u64>() {
                        if c == 'h' {
                            *on_time.write().unwrap() = num;
                        } else if c == 'l' {
                            *off_time.write().unwrap() = num;
                        } else {
                            println!("C is '{}', num is {}, but C must be 'h' or 'l'.", c, num);
                        }
                        let on =  *on_time.read().unwrap();
                        let off = *off_time.read().unwrap();
                        let pct = (on as f32) / ((on + off) as f32) * 100.0;
                        let hz = 1_000_000.0 / ((on + off) as f32);
                        println!("On time: {:5} µs, off time: {:5} µs, which makes {:3.2}% at {:5.1} Hz", on, off, pct, hz);
                    }
                }
            }
            Err(e) => {
                print!("Error: {e}\r\n");
                loop {
                    unsafe { vTaskDelay(1000) };
                }
            }
        }
    }
}

fn test_spi_special_dim() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let spi = peripherals.spi2;

    let sclk: AnyIOPin = peripherals.pins.gpio22.into(); // Pin 1 on Ext
    let sdo : AnyIOPin = peripherals.pins.gpio23.into(); // Pin 3 (above 1) on Ext
    // Latch pin is not attached, but the lib needs it anyway
    let lat = PinDriver::output(peripherals.pins.gpio7)?;

    // PWM pulse we be input into GOIP0.

    // let mut _led_0 = PinDriver::disabled(peripherals.pins.gpio10)?;
    // //let mut led_0 = PinDriver::input(peripherals.pins.gpio10)?;
    // let mut led_1 = PinDriver::output(peripherals.pins.gpio11)?;
    // let mut led_2 = PinDriver::output(peripherals.pins.gpio18)?;
    // let mut led_3 = PinDriver::output(peripherals.pins.gpio19)?;
    // let mut led_4 = PinDriver::output(peripherals.pins.gpio20)?;
    // let mut led_5 = PinDriver::output(peripherals.pins.gpio21)?;

    // //led_0.set_pull(Pull::Down);
    // led_1.set_low()?;
    // led_2.set_low()?;
    // led_3.set_low()?;
    // led_4.set_low()?;
    // led_5.set_low()?;

    println!("All LEDs should be low now.");

    // configuring the spi interface
    let config = config::Config::new()
        .baudrate(10.MHz().into())
        .data_mode(MODE_0);

    let spi_device = SpiDeviceDriver::new_single(
        spi,
        sclk,
        sdo,
        Option::<gpio::Gpio0>::None, // Pin is not needed, but it needs a type
        Option::<gpio::Gpio1>::None, // this is how the official esp-idf-hal examples to it
        &SpiDriverConfig::new(),
        &config,
    )?;

    let mut tlc = TLC59711::new(spi_device, lat, 1);

    println!("Setting the pwm soon...");

    let tmax = 90_000;
    let mut t: i32 = -tmax;
    let max = ((2 << 15)-1) as f32;
    let mut err: f32 = 0.0;
    let mut offset : f32 = 0.2;
    loop {
        let tf = (t as f32) * 0.0001;
        // Zwichen 84% und 90% passieren spannende Dinge, also...
        let perception = 0.87 - 0.03 * faster::cosfull(tf);
        //let perception = 0.5 - 0.5 * faster::cosfull(tf);
        //let corrected = faster::pow(perception, 2.2) + 0.001926;
        let corrected = perception;
        //let ran = rng.gen::<f32>();
        let pwm_f = corrected * max; // + offset
        let mut pwm_r = pwm_f.round();
        let diff = pwm_f - pwm_r;

        if diff.abs() > offset {
            err += diff;
        }
        
        if err > 1.0 {
            pwm_r += 1.0;
            err -= 1.0;
        } else if err < -1.0 {
            pwm_r -= 1.0;
            err += 1.0;
        }
        let pwm = (pwm_r as u16).clamp(0, 65535);
        //let pwm = 5;

        tlc.set_pwm(8, pwm);
        
        tlc.write()?;

        if t > tmax {
            offset += 0.015;
            if offset > 0.2 {
                offset = 0.0;
            }
            t = -tmax;
            println!("Offset: {}", offset);
        }
        if t % 2500 == 0 {
            println!("   {:6.1}", pwm);
            //led_1.toggle();
        }

        //thread::sleep(Duration::from_millis(10));
        // currently updates 6740 times per second
        // each packet has 224 bits + 10 empty cycles, so 234 data clocks.
        // This should update about 42.000 times per second,
        // about six times faster than it does.

        t += 1;
    }
}


fn test_spi_multi_color() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let spi = peripherals.spi2;

    let sclk: AnyIOPin = peripherals.pins.gpio22.into(); // Pin 1 on Ext
    let sdo : AnyIOPin = peripherals.pins.gpio23.into(); // Pin 3 (above 1) on Ext
    // Latch pin is not attached, but the lib needs it anyway
    let lat = PinDriver::output(peripherals.pins.gpio7)?;

    // configuring the spi interface
    let config = config::Config::new()
        .baudrate(10.MHz().into())
        .data_mode(MODE_0);

    let spi_device = SpiDeviceDriver::new_single(
        spi,
        sclk,
        sdo,
        Option::<gpio::Gpio0>::None, // Pin is not needed, but it needs a type
        Option::<gpio::Gpio1>::None, // this is how the official esp-idf-hal examples to it
        &SpiDriverConfig::new(),
        &config,
    )?;

    let mut tlc = TLC59711::new(spi_device, lat, 1);

    println!("Setting the pwm soon...");

    let mut t: i32 = 0;
    let max = ((2 << 15)-1) as f32;
    let mut i = 4;
    loop {
        let tf = (t as f32) * 0.0001; 
        let mut perception = 0.5 - 0.5 * faster::cosfull(tf);
        let corrected = faster::pow(perception, 2.2);
        let pwm_f = corrected * max;
        let mut pwm_r = pwm_f.round();
        let pwm = (pwm_r as u16).clamp(0, 65535);
        tlc.set_pwm(i, pwm);
        if t % 2500 == 0 {
            println!("   {}: {:6.1}", t, pwm);
        }
        
        tlc.write()?;

        if(t > 62_831) { // 2*pi
            t = 0;
            i = (i + 1) % 12;
            println!("CHANNEL: {}", i);
        }

        //thread::sleep(Duration::from_millis(10));
        // currently updates 6740 times per second
        // each packet has 224 bits + 10 empty cycles, so 234 data clocks.
        // This should update about 42.000 times per second,
        // about six times faster than it does.

        t += 1;
    }
}

fn test_spi_multi_color_serial() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let spi = peripherals.spi2;

    let sclk: AnyIOPin = peripherals.pins.gpio22.into(); // Pin 1 on Ext
    let sdo : AnyIOPin = peripherals.pins.gpio23.into(); // Pin 3 (above 1) on Ext
    // Latch pin is not attached, but the lib needs it anyway
    let lat = PinDriver::output(peripherals.pins.gpio7)?;

    // configuring the spi interface
    let config = config::Config::new()
        .baudrate(10.MHz().into())
        .data_mode(MODE_0);

    let spi_device = SpiDeviceDriver::new_single(
        spi,
        sclk,
        sdo,
        Option::<gpio::Gpio0>::None, // Pin is not needed, but it needs a type
        Option::<gpio::Gpio1>::None, // this is how the official esp-idf-hal examples to it
        &SpiDriverConfig::new(),
        &config,
    )?;

    let mut tlc = TLC59711::new(spi_device, lat, 1);

    println!("Waiting for your input...");
    let reader = stdin();
    //let mut writer = stdout();

    let mut values: [i16; 12] = [0; 12];
    let max = ((2 << 15)-1) as f32;

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(_) => {
                let line_trimmed = line.trim_end_matches(&['\r', '\n']);
                println!("Read line: {}", line_trimmed);
                if let Some(colon) = line_trimmed.find(':') {
                    if let Ok(channel) = line_trimmed[.. colon].parse::<u64>() {
                        if let Ok(value) = line_trimmed[colon + 1 ..].parse::<u64>() {
                            println!("Set channel '{}' to value '{}'", channel, value);
                            if channel >= 0 && channel < 12 {
                                values[channel as usize] = value as i16;
                                let mut perception = (value as f32) / 1024.0;
                                let corrected = faster::pow(perception, 2.2);
                                let pwm_f = corrected * max;
                                let mut pwm_r = pwm_f.round();
                                let pwm = (pwm_r as u16).clamp(0, 65535);
                                println!("Duty cycle is {}", pwm);
                                tlc.set_pwm(channel as usize, pwm);
                                tlc.write()?;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                print!("Error: {e}\r\n");
                loop {
                    unsafe { vTaskDelay(1000) };
                }
            }
        }
    }
}
