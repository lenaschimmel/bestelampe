use esp_idf_hal::spi::config::MODE_0;

use esp_idf_hal::gpio;
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::*;
use esp_idf_hal::units::FromValueType;
use embedded_hal::digital::v1::OutputPin;
use std::thread;
use std::time::Duration;
use num::clamp;
use num::traits::Pow;
use rand::Rng;
use fastapprox::faster;


use tlc59xxx::TLC59711;

fn main() -> anyhow::Result<()>  {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let spi = peripherals.spi2;

    let rst = PinDriver::output(peripherals.pins.gpio3)?;
    let dc = PinDriver::output(peripherals.pins.gpio4)?;
    let mut backlight = PinDriver::output(peripherals.pins.gpio5)?;
    let sclk: AnyIOPin = peripherals.pins.gpio10.into();
    let sdo : AnyIOPin = peripherals.pins.gpio11.into();
    // Latch pin is not attached, but the lib needs it anyway
    let lat = PinDriver::output(peripherals.pins.gpio7)?;


    let delay = Ets;

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

    let mut rng = rand::thread_rng();

    let tmax = 30_000;
    let mut t: i32 = -tmax;
    let max = (((2 << 15)-1) as f32);
    let mut err: f32 = 0.0;
    let mut offset : f32 = 0.2;
    loop {
        let tf = (t as f32) * 0.00004;
        let perception = 0.02 - faster::cosfull(tf) * 0.02;
        let corrected = faster::pow(perception, 2.2);
        //let ran = rng.gen::<f32>();
        let pwm_f = corrected * max; // + offset
        let mut pwm_r = pwm_f.round();
        let diff = (pwm_f - pwm_r);

        if (diff.abs() > offset) {
            err += diff;
        }
        
        if (err > 1.0) {
            pwm_r += 1.0;
            err -= 1.0;
        } else if (err < -1.0) {
            pwm_r -= 1.0;
            err += 1.0;
        }
        let pwm = (pwm_r as u16).clamp(0, 65525);
        //let pwm = 5;

        tlc.set_pwm(8, pwm);
        
        tlc.write()?;

        if (t > tmax) {
            offset += 0.015;
            if (offset > 0.2) {
                offset = 0.0;
            }
            t = -tmax;
            println!("Offset: {}", offset);
        }
        if (t % 2500 == 0) {
            println!("   {:.4}", pwm_f);
        }

        //thread::sleep(Duration::from_millis(10));
        // currently updates 6740 times per second
        // each packet has 224 bits + 10 empty cycles, so 234 data clocks.
        // This should update about 42.000 times per second,
        // about six times faster than it does.

        t += 1;
    }

    Ok(())
}
