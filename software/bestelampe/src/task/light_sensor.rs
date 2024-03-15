use esp_idf_hal::{
    prelude::*,
    gpio::AnyIOPin, i2c::*
};

use simple_error::SimpleError;
use veml6040::{Veml6040, IntegrationTime, MeasurementMode};

use ::function_name::named;
use anyhow::Result;
use log::*;

use crate::config::CONFIG;

const DARK_THRESHOLD_SOFT: u16 = 500;
const DARK_THRESHOLD_HARD: u16 = 10;
const BRIGHT_THRESHOLD_SOFT: u16 = 20_000;
const BRIGHT_THRESHOLD_HARD: u16 = 64_000;

const INTEGRATION_TIMES: [IntegrationTime; 6] = [
    IntegrationTime::_40ms,
    IntegrationTime::_80ms,
    IntegrationTime::_160ms,
    IntegrationTime::_320ms,
    IntegrationTime::_640ms,
    IntegrationTime::_1280ms,
];

const WAIT_TIMES: [u16; 6] = [
    40 + 40,
    40 + 80,
    40 + 160,
    40 + 320,
    40 + 640,
    40 + 1280,
];

const SENSITIVITIES: [f32; 6] = [
    0.25168,
    0.12584,
    0.06292,
    0.03146,
    0.01573,
    0.007865,
];

#[named]
pub fn test_light_sensor(i2c: I2C0, scl: AnyIOPin, sda: AnyIOPin) -> Result<()> {
    let config = I2cConfig::new().baudrate(100.kHz().into()).scl_enable_pullup(false).sda_enable_pullup(false);
    let mut i2c = I2cDriver::new(i2c, sda, scl, &config)?;

    info!(target: function_name!(), "Creating the Veml device...");

    let mut sensor: Veml6040<I2cDriver<'_>> = Veml6040::new(i2c);
    info!(target: function_name!(), "Trying to enable and set config...");
    sensor.enable()?;

    let mut integration_time_index = 3;
    sensor.set_integration_time(INTEGRATION_TIMES[integration_time_index])?;
    sensor.set_measurement_mode(MeasurementMode::Manual)?;

    let mut index_changed = false;

    info!(target: function_name!(), "Reading values...");
    loop {
        // This is my attempt at something like try-catch in Rust, so that an error in the
        // loop iteration just skips to the next iteration. Is there an easier way?
        let mut iteration = || -> Result<(), SimpleError> {
            sensor.trigger_measurement().map_err(|_e| SimpleError::new("Failed triggering measurement."))?;
            let min_wait_time = if index_changed { 0 } else { 1000 };
            let wait_time = u16::max(min_wait_time, WAIT_TIMES[integration_time_index]);
            index_changed = false;

            std::thread::sleep(core::time::Duration::from_millis(wait_time as u64));

            let reading = sensor.read_all_channels().map_err(|_e| SimpleError::new("Failed reading all channels for the first time."))?;
            
            // println!("Combined measurements: red = {}, green = {}, blue = {}, white = {}",
            //    reading.red, reading.green, reading.blue, reading.white);

            let green = reading.green;

            if green < DARK_THRESHOLD_HARD {
                debug!(target: function_name!(), "Too dark for accurate lux measurement");
            } else if green > BRIGHT_THRESHOLD_HARD {
                debug!(target: function_name!(), "Too bright for accurate lux measurement");
            } else {
                let lux = green as f32 * SENSITIVITIES[integration_time_index];
            
                let blue = reading.blue;
                let red = reading.red;
                if red > DARK_THRESHOLD_HARD && red < BRIGHT_THRESHOLD_HARD 
                    && blue > DARK_THRESHOLD_HARD && blue < BRIGHT_THRESHOLD_HARD 
                {
                    let ccti = (red as f32 - blue as f32) / (green as f32) + 0.5;
                    let cct = 4278.6 * ccti.powf(-1.2455);
                    info!(target: function_name!(), "Brightness: {} lx, color temperature: {} K", lux, cct);
                } else {
                    info!(target: function_name!(), "Brightness: {} lx, color temperature unknown", lux);
                }
            }
            
            if green < DARK_THRESHOLD_SOFT && integration_time_index < 5 {
                integration_time_index += 1;
                sensor.set_integration_time(INTEGRATION_TIMES[integration_time_index]).map_err(|_e| SimpleError::new("Failed setting integration time."))?;
                debug!(target: function_name!(), "Switching to longer integration time {:?}...", INTEGRATION_TIMES[integration_time_index]);
                index_changed = true;
            }
            if green > BRIGHT_THRESHOLD_SOFT  && integration_time_index > 0 {
                integration_time_index -= 1;
                sensor.set_integration_time(INTEGRATION_TIMES[integration_time_index]).map_err(|_e| SimpleError::new("Failed setting integration time."))?;
                debug!(target: function_name!(), "Switching to shorter integration time {:?}...", INTEGRATION_TIMES[integration_time_index]);
                index_changed = true;
            }     
            Ok(())   
        };
        if let Err(err) = iteration() {
            warn!(target: function_name!(), "Error in sensor loop iteration: {}", err);
            std::thread::sleep(core::time::Duration::from_millis(750));
        }
    }

    return Ok(());
}
