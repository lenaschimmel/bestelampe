use crate::prelude::*;
use std::sync::{Arc, RwLock};
use esp_idf_hal::gpio::{AnyIOPin, PinDriver};

#[named]
pub fn test_buttons(
    pin_a: AnyIOPin, 
    pin_b: AnyIOPin,
    pin_c: AnyIOPin,
    light_brightness_target: Arc<RwLock<f32>>,
    light_temperature_target: Arc<RwLock<f32>>,
    light_dim_speed:  Arc<RwLock<f32>>,
) -> Result<()> 
    {
    let in_a = PinDriver::input(pin_a)?;
    let in_b = PinDriver::input(pin_b)?;
    let in_c = PinDriver::input(pin_c)?;

    let mut temperature_index = 0;

    let temperatures: [f32; 8] = [1050.0, 1700.0, 2300.0, 2700.0, 3500.0, 5700.0, 10_000.0, 20_000.0];
    let mut b_high_before: bool = false;

    loop {
        if in_a.is_high() {
            let brightness = *light_brightness_target.read().unwrap();
            let new_brightness = f32::min(30.0, brightness + 0.2);
            *light_brightness_target.write().unwrap() = new_brightness;
            *light_dim_speed.write().unwrap() = 0.1;
            println!("Touch-dim to {}", new_brightness);
        } else if in_c.is_high() {
            let brightness = *light_brightness_target.read().unwrap();
            let new_brightness = f32::max(0.0, brightness - 0.2);
            *light_brightness_target.write().unwrap() = new_brightness;
            *light_dim_speed.write().unwrap() = 0.1;
            println!("Touch-dim to {}", new_brightness);
        } else if in_b.is_high() && !b_high_before {
            temperature_index = (temperature_index + 1) % temperatures.len();
            *light_temperature_target.write().unwrap() = temperatures[temperature_index];
            println!("Touch-temperatrue to {}", temperatures[temperature_index]);
            *light_dim_speed.write().unwrap() = 0.1;
        }
        b_high_before = in_b.is_high();
        //println!("Buttons: {}, {}, {}", in_a.is_high(), in_b.is_high(), in_c.is_high());
        std::thread::sleep(core::time::Duration::from_millis(100));
    }
    Ok(())
}