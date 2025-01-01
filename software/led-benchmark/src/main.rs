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
    ledc::{LedcDriver, LedcTimerDriver, config::TimerConfig, LedcChannel,PwmError},
    peripheral::{Peripheral, PeripheralRef},
    sys::EspError,
};

use embedded_hal::pwm::SetDutyCycle;

use embedded_hal_bus::{
    i2c as i2c_bus,
};

use std::{
    time::Duration,
    ptr::null_mut,
    thread,
    io::stdin,
    borrow::{Borrow, BorrowMut},
    sync::{Arc, mpsc},
    collections::VecDeque,
};

use veml6040::wrapper::AutoVeml6040;

use lm75::{Lm75, Address};

extern crate ina219_rs as ina219;

use ina219::ina219::{INA219, Calibration};

use rand::Rng;

// from esp-idf-servo
use esp_idf_sys::{
    gpio_num_t, ledc_channel_config, ledc_channel_config_t, ledc_channel_t,
    ledc_get_duty, ledc_intr_type_t_LEDC_INTR_DISABLE, ledc_mode_t, ledc_set_duty, ledc_stop,
    ledc_timer_bit_t, ledc_timer_bit_t_LEDC_TIMER_10_BIT, ledc_timer_config, ledc_timer_config_t,
    ledc_timer_rst, ledc_timer_t, ledc_update_duty, soc_periph_ledc_clk_src_legacy_t_LEDC_AUTO_CLK,
};

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

    //run_benchmark()?;
    // let mut robot = Robot::new()?;
    // robot.run_angle_measurement()?;
    // robot.move_to_neutral()?;

    //interactive_pwm()?;
    //board_d();
    all_together()?;
    //board_e()?;

    Ok(())
}

struct Robot<'p> {
    pos_hori_neutral: i16,
    pos_hori: i16,
    pos_vert_neutral: i16,
    pos_vert: i16,
    driver_hori: LedcDriver<'p>,
    driver_vert: LedcDriver<'p>,
    light_sensor: AutoVeml6040<I2cDriver<'p>>,
}

impl<'p> Robot<'p> {
    pub fn new() -> anyhow::Result<Self> {
        let peripherals = Peripherals::take()?;
        //let rng = rand::thread_rng();

        println!("Initialize the LED and LEDC for direct PWM output...");
        let hori_pin = peripherals.pins.gpio10;    
        let vert_pin = peripherals.pins.gpio11;    
        let ledc = peripherals.ledc;
        let timer_driver: LedcTimerDriver<'_> = LedcTimerDriver::new(
            ledc.timer0, 
            &TimerConfig::default().frequency(50.Hz().into()).resolution(esp_idf_hal::ledc::Resolution::Bits14)
        ).expect("Get LEDC timer.");

                
        println!("Initialize I2C bus...");
        let scl: AnyIOPin = peripherals.pins.gpio2.into();
        let sda: AnyIOPin = peripherals.pins.gpio3.into();
        let config = I2cConfig::new().baudrate(100.kHz().into()).scl_enable_pullup(true).sda_enable_pullup(true);
        let mut i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &config)?;
        //let i2c_ref_cell = RefCell::new(i2c);
        println!("I2C bus is initialized.");

        for addr in 1..=127 {
            //info!(target: function_name!(), "Scanning Address {}", addr as u8);

            // Scan Address
            let res = i2c.read(addr as u8, &mut [0], 100);

            // Check and Print Result
            match res {
                Ok(_) => 
                    println!("Address {}: found something!", addr as u8),
                Err(e) if e.code() == ESP_FAIL => 
                    println!("Address {}: fail", addr as u8),
                    // {}, // Usual error if nothing is present
                Err(e)
                    => println!("Address {}: error: {}", addr as u8, e),
            }
        }

        println!("Initialize light sensor...");
        let light_sensor = AutoVeml6040::new(i2c)?;
        println!("Light sensor is initialized.");

        let driver_hori = LedcDriver::new(ledc.channel0, &timer_driver, hori_pin ).expect("Get LEDC driver.");
        let driver_vert = LedcDriver::new(ledc.channel1, &timer_driver, vert_pin ).expect("Get LEDC driver.");
        println!("LED and LEDC is initialized.");

		return Ok(Robot{
            pos_hori_neutral: 1600, 
            pos_vert_neutral: 1600,
            pos_hori: 1600,
            pos_vert: 1600,
            driver_hori,
            driver_vert,
            light_sensor,
        });
	}

    fn move_to(self: &mut Self, target_hori: i16, target_vert: i16, delay: u64) -> anyhow::Result<()> {
        let change = 1;
        loop {
            if self.pos_hori > target_hori {
                self.pos_hori -= change;
            }
            if self.pos_hori < target_hori {
                self.pos_hori += change;
            }

            if self.pos_vert > target_vert {
                self.pos_vert -= change;
            }
            if self.pos_vert < target_vert {
                self.pos_vert += change;
            }

            self.driver_hori.set_duty_cycle(self.pos_hori as u16)?;
            self.driver_vert.set_duty_cycle(self.pos_vert as u16)?;

            if self.pos_hori == target_hori && self.pos_vert == target_vert {
                return Ok(());
            }

            thread::sleep(Duration::from_millis(delay));
        }
    }

    fn move_to_neutral(self: &mut Self) -> anyhow::Result<()> {
        self.move_to(self.pos_hori_neutral, self.pos_vert_neutral, 30)?;
        Ok(())
    }

    fn run_angle_measurement(self: &mut Self) -> anyhow::Result<()> {
        self.move_to_neutral()?;
        thread::sleep(Duration::from_millis(2000));

        let h_min: i16 = -90;
        let h_max: i16 = 91;
        let h_stp = 5;

        let v_min: i16 = 0;
        let v_max: i16 = 56;
        let v_stp = 2;

        println!("line; horizontal; vertical; white");

        for v in (v_min..v_max).step_by(v_stp) {
            for h in (h_min..h_max).step_by(h_stp) {
                self.move_to(self.pos_hori_neutral - h * 5, self.pos_vert_neutral - v * 5, 30);
                thread::sleep(Duration::from_millis(250));

                let result = self.light_sensor.read_absolute_retry();
                if let Ok(measurement) = result {
                    println!("line; {}; {}; {}", h, v, measurement.white);
                }
        
            }
        }

        Ok(())
    }
}

// struct LedcWrapper<'t> {
//     //ledc_channel: PeripheralRef<'t, LedcChannel>,
//     //timer_driver: LedcTimerDriver<'t>,
//     ledc_driver: Option<Box<LedcDriver<'t>>>,
//     led_pin: AnyIOPin,
// }

// impl<'t, C: LedcChannel> LedcWrapper<'t> {
//     pub fn new(
//         mut timer_driver: LedcTimerDriver<'t>,
//         mut led_pin: AnyIOPin,
//         ledc_channel: impl Peripheral<P = C> + 't,
//     ) -> Result<Self, EspError> {
//         return Ok(Self {
//             //timer_driver: timer_driver,
//             ledc_driver: Some(Box::new(LedcDriver::new(ledc_channel, timer_driver.borrow_mut(), led_pin.borrow_mut()).expect("Get LEDC driver."))),
//             led_pin,
//         });
//     }

//     // pub fn set_channel<C: LedcChannel>(
//     //     &mut self,
//     //     ledc_channel: impl Peripheral<P = C> + 't,
//     // )  -> Result<(), EspError>  {
//     //     Ok(())
//     // }

//     pub fn set_duty_cycle(
//         &mut self,
//         dc: u16,
//     ) -> Result<(), PwmError>  {
//         match &mut self.ledc_driver {
//             Some(driver) => driver.set_duty_cycle(dc)?,
//             None => {},
//         }
//         Ok(())
//     }

//     pub fn deactivate(
//         &mut self,
//     ) -> Result<(), EspError>  {
//         match &mut self.ledc_driver {
//             Some(driver) => driver.set_duty_cycle(0).unwrap(),
//             None => {},
//         }
//         self.ledc_driver = None;
//         let mut pinDriver = PinDriver::output(self.led_pin.borrow_mut())?; 
//         pinDriver.set_low()?;
//         Ok(())
//     }
// }

// impl<'t> Drop for LedcWrapper<'t> {
//     fn drop(&mut self) {
//         self.ledc_driver = None;
//     }
// }

fn all_together() ->  anyhow::Result<()> {
    let peripherals = Peripherals::take()?;

    // GPIO_18 = J2, Pin 10 = LED 1  Amber, at 4000 draws only 2.54A, ca. 130 °C
    // GPIO_19 = J2, Pin  9 = LED 2  2200K, max 2750, ca. 120 °C
    // GPIO_20 = J2, Pin  8 = LED 3  3000K, at 4000 draws only 2.55A but gets > 160 °C
    // GPIO_21 = J2, Pin  7 = LED 4  Amber PC // takes the current wihtout illumination -> shorted
    // GPIO_22 = J2, Pin  6 = LED 5  Red-Orange, at 4000 draws only 2.45A, ca. 140 °C, looks just like Amber?
    // GPIO_23 = J2, Pin  5 = LED 6  Lime, at 4000 draws only 2.52A, ca. 140 °C
    // GPIO_10 = J1, Pin 10 = LED 7  Blue // works now
    // GPIO_11 = J1, Pin 11 = LED 8  5000K // works now
    //   (linked to LED 2) = LED 11  2200K 
    //   (linked to LED 1) = LED 11  Amber does not work

    // Schonleuchtgänge:
    // 2x Amber, 2x 2200K, 1x Red-Orange: max 1520, ca. 130 °C
    // 2x 2200K, 1x 3000K: max 1940, ca. 130 °C
    // 2x 2200K, 1x 3000K, 1x Red-Orange: max, 1550  ca. 122 °C

    //let mut pins: [AnyIOPin;10] = [
    let mut pins = Vec::<AnyIOPin>::from([
        //peripherals.pins.gpio18.into(),
        peripherals.pins.gpio19.into(),
        peripherals.pins.gpio20.into(),
        // peripherals.pins.gpio21.into(),
        //peripherals.pins.gpio22.into(),
        // peripherals.pins.gpio23.into(),
        // peripherals.pins.gpio10.into(),
        // peripherals.pins.gpio11.into(),
    ]);

    println!("Setting all pins to low...");
    // for pin in pins {
    //     let mut pinDriver = PinDriver::output(pin)?; 
    //     pinDriver.set_low()?;
    // }

    println!("Linking all pins to a single LEDC channel...");
    let mut ledc = peripherals.ledc;

    let timer_driver_0 = Arc::new(LedcTimerDriver::new(
        ledc.timer0, 
        &TimerConfig::default().frequency(2400.Hz().into()).resolution(esp_idf_hal::ledc::Resolution::Bits12)
    )?);

    
    // We re-assign this variable to 10 different instances, and then keep and use the last one. That's by design to link all 10 pins to the same driver.
    // Syntactically it would be easier to declare the variable before the loop, then loop over all 10 pins, and keep the value that was assigned last.
    // But in that case, the borrow checker does not release the driver from the previous iteration early enough to borrow channel0 again.
    let end_count = pins.len() - 1;
    for i in 0..end_count {
        let pin = pins.pop().unwrap();
        println!("Temporarily driving pin {}", pin.pin());
        let mut ledc_driver = LedcDriver::new(ledc.channel0.borrow_mut(), timer_driver_0.clone(), pin)?;
        ledc_driver.set_duty_cycle(0)?;
    }
    let pin = pins.pop().unwrap();
    println!("Permanently driving pin {}", pin.pin());
    let mut ledc_driver = LedcDriver::new(ledc.channel0.borrow_mut(), timer_driver_0.clone(), pin)?;

    // GPIO2 = J2, Pin 12 = Enable
    let enable_pin =   peripherals.pins.gpio2;
    let mut enable_driver = PinDriver::output(enable_pin)?; 
    enable_driver.set_low()?;

    println!("LEDs are enabled.");

    println!("Testing increasing duty cycles...");
    let mut dc_f: f32 = 1940.0;
    loop {
        dc_f *= 1.0;
        if dc_f > 4000.0 {
            break;
        }
        let dc: u16 = dc_f as u16;
        ledc_driver.set_duty_cycle(dc)?;
        println!("Duty cycle is {}.", dc);
        thread::sleep(Duration::from_millis(2000));
    }
     
    println!("Loop done!");
    loop {
        thread::sleep(Duration::from_millis(2000));
    }
   //Ok(())
}

fn multi_threading_moster() ->  anyhow::Result<()> {
    //println!("Try to initialize all 10 pins..");
    let peripherals = Peripherals::take()?;

    let mut pins = VecDeque::<AnyIOPin>::from([
        peripherals.pins.gpio18.into(), // Reset state: 3 = Input Enabled with weak Pull-Up
        peripherals.pins.gpio19.into(), // Reset state: 3 = Input Enabled with weak Pull-Up
        peripherals.pins.gpio20.into(), // Reset state: 3 = Input Enabled with weak Pull-Up
        peripherals.pins.gpio21.into(), // Reset state: 3 = Input Enabled with weak Pull-Up
        peripherals.pins.gpio22.into(), // Reset state: 3 = Input Enabled with weak Pull-Up
        peripherals.pins.gpio23.into(), // Reset state: 3 = Input Enabled with weak Pull-Up
        peripherals.pins.gpio10.into(), // Reset state: 1 = Input Enabled
        peripherals.pins.gpio11.into(), // Reset state: 1 = Input Enabled
        peripherals.pins.gpio2.into(),  // Reset state: 1 = Input Enabled
        peripherals.pins.gpio3.into(),  // Reset state: 1 = Input Enabled
    ]);

    // let mut ledc = peripherals.ledc;
    // let timer_driver: LedcTimerDriver<'_> = LedcTimerDriver::new(
    //          ledc.timer0.borrow_mut(), 
    //          &TimerConfig::default().frequency(2400.Hz().into()).resolution(esp_idf_hal::ledc::Resolution::Bits12)
    //      ).expect("Get LEDC timer.");

    // let mut wrapper = LedcWrapper::new(timer_driver, peripherals.pins.gpio18.into())?;

    // wrapper.set_channel(ledc.channel0.borrow_mut());

    // wrapper.set_duty_cycle(30);
    // thread::sleep(Duration::from_millis(500));
    // wrapper.set_duty_cycle(60);
    // thread::sleep(Duration::from_millis(500));
    // wrapper.set_duty_cycle(90);
    // thread::sleep(Duration::from_millis(500));
    // wrapper.deactivate();

    // loop {}

    // LED 1 = Lichtfarbe rot/pink    = Draht blau    = GPIO 18 - Pin kaputt?
    // LED 2 = Lichtfarbe amber       = Draht grün    = GPIO 19
    // LED 3 = Lichtfarbe amber       = Draht gelb    = GPIO 20
    // LED 4 = Lichtfarbe warmweiß    = Draht orange  = GPIO 21 - Pin kaputt?
    // LED 5 = Lichtfarbe weiß        = Draht weiß    = GPIO 22
    // LED 6 = Lichtfarbe weiß        = Draht schwarz = GPIO 23
    // LED 7 = Lichtfarbe kaltweiß    = Draht braun   = GPIO 10
    // LED 8 = Lichtfarbe grün (lime) = Draht rot     = GPIO 11
    // LED 9 = Lichtfarbe blau        = Draht grau    = GPIO 2
    // LED 10 = Lichtfarbe blau       = Draht lila    = GPIO 3

    // let mut pinDrivers: Vec<Option<PinDriver<AnyIOPin, Output>>> = Vec::new();

    // for pin in pins {
    //     let mut pinDriver = PinDriver::output(pin)?; 
    //     pinDriver.set_low()?;
    //     pinDrivers.push(Some(pinDriver));
    // }

    //println!("All pins should be low now.");
    let mut ledc = peripherals.ledc;

    let timer_driver_0 = Arc::new(LedcTimerDriver::new(
        ledc.timer0, 
        &TimerConfig::default().frequency(2400.Hz().into()).resolution(esp_idf_hal::ledc::Resolution::Bits12)
    )?);

    let timer_driver_0_0 = timer_driver_0.clone();
    let timer_driver_0_1 = timer_driver_0.clone();
    
    let (tx_inside_0, rx_inside_0) = mpsc::channel();
    let (tx_outside_0, rx_outside_0) = mpsc::channel();
    let thread0 = std::thread::Builder::new()
    .stack_size(7000)
    .spawn(move || use_channel(ledc.channel0, timer_driver_0_0, rx_inside_0, tx_outside_0, 50))?;

    let (tx_inside_1, rx_inside_1) = mpsc::channel();
    let (tx_outside_1, rx_outside_1) = mpsc::channel();
    let thread1 = std::thread::Builder::new()
    .stack_size(7000)
    .spawn(move || use_channel(ledc.channel1, timer_driver_0_1, rx_inside_1, tx_outside_1, 187))?;

    let pin_a = pins.pop_front().unwrap();
    tx_inside_0.send(pin_a);

    let pin_b = pins.pop_front().unwrap();
    tx_inside_1.send(pin_b);

    loop {
        match rx_outside_0.try_recv() {
            Ok(pin_out) => {
                pins.push_back(pin_out);
                let pin_in = pins.pop_front().unwrap();
                tx_inside_0.send(pin_in);
            },
            Err(_) => {},
        }

        match rx_outside_1.try_recv() {
            Ok(pin_out) => {
                pins.push_back(pin_out);
                let pin_in = pins.pop_front().unwrap();
                tx_inside_1.send(pin_in);
            },
            Err(_) => {},
        }

        thread::sleep(Duration::from_millis(10));
        
    }
    // let (tx1, rx1) = mpsc::channel();
    // let thread1 = std::thread::Builder::new()
    // .stack_size(7000)
    // .spawn(move || use_channel(ledc.channel1, peripherals.pins.gpio23.into(), timer_driver_0_1, tx1))?;

    thread0.join().unwrap()?;
    thread1.join().unwrap()?;

    //let p23 = rx1.recv().unwrap();

    println!("Got my pins back!");


    // {
    //     println!("Switch 18 low.");
    //     let mut pinDriver = PinDriver::output(led_pin.borrow_mut())?; 
    //     pinDriver.set_low()?;
    //     thread::sleep(Duration::from_millis(1500));
    // }

    // println!("Drive pin 18 with LEDC");
    // {
    //     let mut timer_driver: LedcTimerDriver<'_> = LedcTimerDriver::new(
    //         ledc.timer0.borrow_mut(), 
    //         &TimerConfig::default().frequency(2400.Hz().into()).resolution(esp_idf_hal::ledc::Resolution::Bits12)
    //     ).expect("Get LEDC timer.");

    //     let mut driver_0 = LedcDriver::new(ledc.channel0.borrow_mut(), &timer_driver, led_pin.borrow_mut()).expect("Get LEDC driver.");
    //     println!("LED and LEDC is initialized.");

    //     for i in 0..5 {
    //         driver_0.set_duty_cycle(30)?;
    //         println!("LED is on");
    //         thread::sleep(Duration::from_millis(500));
    //         driver_0.set_duty_cycle(0)?;
    //         println!("LED is off");
    //         thread::sleep(Duration::from_millis(500));
    //     }
    // }

    Ok(())
}

fn use_channel<C: LedcChannel>(
    mut channel: impl Peripheral<P = C>,
    timer_driver: Arc<LedcTimerDriver<'_>>,
    rx: mpsc::Receiver<AnyIOPin>,
    tx: mpsc::Sender<AnyIOPin>,
    millis: u64,
) -> Result<(), PwmError> {

    loop {

        let mut led_pin: AnyIOPin = rx.recv().unwrap();
        {
            println!("Thread waits for pin...");
            let mut ledc_driver = LedcDriver::new(channel.borrow_mut(), timer_driver.clone(), led_pin.borrow_mut()).expect("Get LEDC driver.");
            println!("Thred got the pin. Drive it with LEDC...");

            for i in 0..10 {
                ledc_driver.set_duty_cycle(i)?;
                thread::sleep(Duration::from_millis(millis));
            }
            for i in 0..10 {
                ledc_driver.set_duty_cycle(9-i)?;
                thread::sleep(Duration::from_millis(millis));
            }
        }
        // ledcDriver going out of scope does not unlink the pin from it in the GPIO Matrix.
        // We need to create a PinDriver (and switch it to low?) so that the next usage of the 
        // LEDC channel with another Pin and Driver will not also control the previous pin.
        // We *could* use this to control multiple pins with the same LEDC channel if we wanted,
        // in case we need more then 6 channels at once.

        println!("Thread is switching off the pin...");
        {
            let mut pin_driver = PinDriver::output(led_pin.borrow_mut())?; 
            println!("Switch {} low.", pin_driver.pin());
            pin_driver.set_low()?;
            
            //thread::sleep(Duration::from_millis(1500));
        }
        
        println!("Thread is giving the pin back...");
        tx.send(led_pin).unwrap();
    }
    Ok(())
}

fn interactive_pwm()  -> anyhow::Result<()> {
    
    println!("Initialize the LED and LEDC for direct PWM output...");
    let peripherals = Peripherals::take()?;

    let led_pin = peripherals.pins.gpio22;
    let ledc = peripherals.ledc;
    let mut timer_driver: LedcTimerDriver<'_> = LedcTimerDriver::new(
        ledc.timer0, 
        &TimerConfig::default().frequency(2400.Hz().into()).resolution(esp_idf_hal::ledc::Resolution::Bits12)
    ).expect("Get LEDC timer.");

    let mut driver_0 = LedcDriver::new(ledc.channel0, &timer_driver, led_pin ).expect("Get LEDC driver.");
    println!("LED and LEDC is initialized.");

    let frequency = 3000;
    let mut dc = 800;

    timer_driver.set_frequency(Hertz(frequency as u32))?;
    driver_0.set_duty_cycle(dc)?;
    println!("Duty cycle is: {}", dc);
    thread::sleep(Duration::from_millis(10));

    println!("Waiting for your input...");
    let reader = stdin();
    //let mut writer = stdout();

    //let values: [i16; 12] = [0; 12];
    //let max = ((2 << 15)-1) as f32;

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(_) => {
                let line_trimmed = line.trim_end_matches(&['\r', '\n']);
                println!("Read line: {}", line_trimmed);
                if let Ok(duty) = line_trimmed.parse::<u64>() {
                    dc = duty as u16;
                    driver_0.set_duty_cycle(dc)?;
                    println!("Duty cycle is: {}", dc);
                }
            }
            Err(e) => {
                print!("Error: {e}\r\n");
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn run_benchmark() -> anyhow::Result<()> {
    let peripherals = Peripherals::take()?;
    let mut rng = rand::thread_rng();

    println!("Initialize the LED and LEDC for direct PWM output...");
    let led_pin = peripherals.pins.gpio11;    
    let ledc = peripherals.ledc;
    let mut timer_driver: LedcTimerDriver<'_> = LedcTimerDriver::new(
        ledc.timer0, 
        &TimerConfig::default().frequency(2400.Hz().into()).resolution(esp_idf_hal::ledc::Resolution::Bits12)
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
    //loop {
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
    //}
    

    let i2c_ref_cell = RefCell::new(i2c);
    println!("I2C bus is initialized.");


    println!("Initialize light sensor...");
    let mut light_sensor = AutoVeml6040::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell))?;
    println!("Light sensor is initialized.");

    // 72 should be the temperature sensor on the driver
    // 84 should be the temperature sensor on the led

    println!("Initialize LED temperature sensor...");
    let tmp1075_led_address = Address::from(79);
    let mut led_tempeature_sensor = Lm75::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell), tmp1075_led_address);
    println!("LED temperature sensor is initialized.");

    println!("Initialize driver temperature sensor...");
    let tmp1075_driver_address = Address::from(72);
    let mut driver_tempeature_sensor = Lm75::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell), tmp1075_driver_address);
    println!("Driver temperature sensor is initialized.");


    println!("Initialize power sensor...");
    //let ina_options = Opts::new(64, 100 * physic::MilliOhm, 1 * physic::Ampere);
    //let ina_options = Opts::default();
    let mut ina = INA219::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell));
    ina.init(Calibration::Calibration_32V_2A).unwrap();
    println!("Power sensor is initialized.");

    println!("header; frequency; duty cycle; red; green; blue; white; led_temperature; voltage; current; power; efficiency; driver_temperature");

    //let mut t = 1.5;
    loop {
        //t += 0.005;
        //let max_duty = 2000 - (fastapprox::fast::cos(t) * 1800.0) as i16;
        let frequency = rng.gen_range(1_000..12_000);
        let dc = rng.gen_range(50..4000) as u16;

        // Even if we are in a low-power phase, let's sometimes test high powers so that we know how high powers perform on low temperatures
        // if rng.gen_range(0..100) < 10 {
        //     dc = rng.gen_range(max_duty..3000);
        // }
        

            timer_driver.set_frequency(Hertz(frequency as u32))?;
            driver_0.set_duty_cycle(dc)?;

            thread::sleep(Duration::from_millis(10));
        
            let mut sum_r = 0.0;
            let mut sum_g = 0.0;
            let mut sum_b = 0.0;
            let mut sum_w = 0.0;
            let mut sum_voltage = 0.0;
            let mut sum_current = 0.0;

            thread::sleep(Duration::from_millis(100));

            // The INA219 driver does not support multisampling, so I take many measurements and
            // average them on the ESP.
            // TODO use my own fork of the driver and enable multisampling.
            
            let mut light_count: f32 = 0.0;
            let mut power_count: f32 = 0.0;
            for i in 1..15 {
                if i < 5 {
                let result = light_sensor.read_absolute_retry();
                    if let Ok(measurement) = result {
                        sum_r += measurement.red;
                        sum_g += measurement.green;
                        sum_b += measurement.blue;
                        sum_w += measurement.white;
                        light_count += 1.0;
                    }
                    // if i > 3 && light_count < 2.0 || i > 6 && light_count < 4.0 {
                    //     break;
                    // }
                }

                sum_voltage += ina.getBusVoltage_V().unwrap();
                sum_current += ina.getCurrent_mA().unwrap();    
                power_count += 1.0;

                thread::sleep(Duration::from_millis(3));
            }


            let led_temperature = match led_tempeature_sensor.read_temperature() {
                Ok(temp_celsius) => temp_celsius,
                Err(e) => { 
                    println!("Temp sensor error: {:?}", e);
                    0.0
                }
            };

            let driver_temperature = match driver_tempeature_sensor.read_temperature() {
                Ok(temp_celsius) => temp_celsius,
                Err(_) => 0.0
            };

            let brightness = if light_count > 3.0 {
                (sum_r / light_count, sum_g / light_count, sum_b / light_count, sum_w / light_count)
            } else {
                (0.0, 0.0, 0.0, 0.0)
            };

            let (voltage, current) = if power_count > 10.0 {
                (sum_voltage / power_count, sum_current / power_count)
            } else {
                (0.0, 0.0)
            };

            
            println!("line; {}; {}; {}; {}; {}; {}; {}; {}; {}; {}; {}; {}", frequency, dc, brightness.0, brightness.1, brightness.2, brightness.3, led_temperature, voltage, current, voltage * current, brightness.3 / (voltage * current + 0.01), driver_temperature);        
            
            // if voltage * current > 17_500.0 {
            //     println!("Stopping measuement (at this frequency) to keep the power supply safe.");
            //     driver_0.set_duty_cycle(0)?;
            //     continue;
            // }   
        
    }

    //println!("Benchmark is done.");
}

fn board_e() -> anyhow::Result<()> {
    println!("Initialize the LED and LEDC for direct PWM output...");
    let peripherals = Peripherals::take()?;
    
    // GPIO_18 = J2, Pin 10 = LED 1  Amber
    // GPIO_19 = J2, Pin  9 = LED 2  2200K
    // GPIO_20 = J2, Pin  8 = LED 3  3000K
    // GPIO_21 = J2, Pin  7 = LED 4  Amber PC
    // GPIO_22 = J2, Pin  6 = LED 5  Red-Orange
    // GPIO_23 = J2, Pin  5 = LED 6  Lime
    // GPIO_10 = J1, Pin 10 = LED 7  Blue
    // GPIO_11 = J1, Pin 11 = LED 8  5000K
    //   (linked to LED 2) = LED 11  2200K 
    //   (linked to LED 1) = LED 11  Amber 

    let mut pins = VecDeque::<AnyIOPin>::from([
        peripherals.pins.gpio18.into(),
        // peripherals.pins.gpio19.into(),
        // peripherals.pins.gpio20.into(),
        // peripherals.pins.gpio21.into(),
        // peripherals.pins.gpio22.into(),
        // peripherals.pins.gpio23.into(),
        // peripherals.pins.gpio10.into(),
        // peripherals.pins.gpio11.into(),
    ]);

    let mut ledc = peripherals.ledc;
    let timer_driver = Arc::new(LedcTimerDriver::new(
        ledc.timer0, 
        &TimerConfig::default().frequency(3000.Hz().into()).resolution(esp_idf_hal::ledc::Resolution::Bits12)
    ).expect("Get LEDC timer."));
    
    //let frequency = 3000;
    //timer_driver.set_frequency(Hertz(frequency as u32))?;

    // let mut dc = 20; // up to 4095
    // println!("LED and LEDC is initialized.");
    // println!("Duty cycle is: {}", dc);
        

    // GPIO2 = J2, Pin 12 = Enable
    let enable_pin =   peripherals.pins.gpio2;
    let mut enable_driver = PinDriver::output(enable_pin)?; 
    enable_driver.set_low()?;

    println!("LEDs are enabled.");



    // println!("Initialize I2C bus...");
    // let sda: AnyIOPin = peripherals.pins.gpio6.into();
    // let scl: AnyIOPin = peripherals.pins.gpio7.into();
    // let config = I2cConfig::new().baudrate(100.kHz().into()).scl_enable_pullup(false).sda_enable_pullup(false);
    // let mut i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &config)?;

    // let i2c_ref_cell = RefCell::new(i2c);
    // println!("I2C bus is initialized.");


    // println!("Initialize LED temperature sensor...");
    // let tmp1075_led_address = Address::from(142);
    // let mut led_tempeature_sensor = Lm75::new(i2c_bus::RefCellDevice::new(&i2c_ref_cell), tmp1075_led_address);
    // println!("LED temperature sensor is initialized.");

    let mut dc = 1.0;
    
    loop {

        let led_pin = pins.pop_front().unwrap();
        {
            let mut pwm_driver = LedcDriver::new(ledc.channel0.borrow_mut(), timer_driver.clone(), led_pin).expect("Get LEDC driver.");
            pwm_driver.set_duty_cycle(dc as u16)?;
            thread::sleep(Duration::from_millis(100));
            pwm_driver.set_duty_cycle(0)?;
        }
        //pins.push_back(led_pin);

        dc *= 1.1;
        if (dc > 4000.0) {
            dc = 1.0;
        }
        // let led_temperature = match led_tempeature_sensor.read_temperature() {
        //     Ok(temp_celsius) => {
        //         println!("Temp: {:?}", temp_celsius);
        //         temp_celsius
        //     },
        //     Err(e) => { 
        //         println!("Temp sensor error: {:?}", e);
        //         0.0
        //     }
        // };
        thread::sleep(Duration::from_millis(1000));
    }
}