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

use esp_idf_hal::ledc::LedcDriver;

use esp_idf_sys::EspError;

use log::*;
use prisma::Xyz;

use std::rc::Rc;
use std::cell::RefCell;

/// A chromaticity on the CIE 1931 xy chromaticity diagram.
/// This can be interpreted as the hue and saturation of light, without brightness information.
#[derive(Clone, Debug)]
pub struct XyColor {
	x: f32,
	y: f32
}

impl From<XyColor> for Xyz<f32> {
	fn from(xy: XyColor) -> Xyz<f32> {
		return xy.with_brightness(1.0);
	}
}

impl From<Xyz<f32>> for XyColor {
	fn from(xyz: Xyz<f32>) -> XyColor {
		let sum = xyz.x() + xyz.y() + xyz.z();
		return XyColor{x: xyz.x() / sum, y: xyz.y() / sum};
	}
}

impl XyColor {
	pub fn new(x: f32, y:f32) -> Self {
		return XyColor{x, y};
	}

	/// Combine the xy value with a Y value for brightness in the xyY color space 
	/// and convert the result to a color in the XYZ color space.
	#[allow(non_snake_case)]
	pub fn with_brightness(&self, Y: f32) -> Xyz<f32> {
		let z = 1.0 - (self.x+self.y);
		let X = (Y / self.y) * self.x;
		let Z = (Y / self.y) * z;
	
		return Xyz::new(X, Y, Z);
	}
}

pub struct Led<'p> {
	index: usize,
	driver: LedcDriver<'p>,
	xy_color: XyColor,
	//xyz_color: Xyz<f32>,
	max_brightness: f32,
	name: &'p str,
}

impl<'p> Led<'p> {
	pub fn new(index: usize, name: &'p str, driver: LedcDriver<'p>, x: f32, y: f32, max_brightness: f32) -> Self {
		let xy_color = XyColor {x, y};
		//let xyz_color = xy_color.with_brightness(max_brightness);
		return Self { index, name, driver, xy_color, max_brightness };
	}
}

#[derive(Clone)]
pub struct LedTriangle<'p> {
	leds: [Rc<RefCell<Led<'p>>>; 3],
}

impl<'p> LedTriangle<'p> {
	pub fn new(a: Rc<RefCell<Led<'p>>>, b: Rc<RefCell<Led<'p>>>, c: Rc<RefCell<Led<'p>>>) -> Self {
		return Self { leds: [a, b, c]};
	}

	pub fn get_barycentric(&self, xy: XyColor) -> [f32; 3] {
		let XyColor{x: x1, y: y1} = self.leds[0].borrow().xy_color;
		let XyColor{x: x2, y: y2} = self.leds[1].borrow().xy_color;
		let XyColor{x: x3, y: y3} = self.leds[2].borrow().xy_color;
		
		let a = ((y2 - y3) * (xy.x - x3) + (x3 - x2) * (xy.y - y3)) / ((y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3));
		let b = ((y3 - y1) * (xy.x - x3) + (x1 - x3) * (xy.y - y3)) / ((y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3));
		let c = 1.0 - a - b;
		//info!("Barycentric for color {:?}: {},{},{}", xy, a,b,c);
		return [a, b, c];
	}
}


/// Pwm controller for a specific set of LEDs
pub struct Pwm<'p> {
	//timer_driver: LedcTimerDriver<'p>,
	leds: Vec<Rc<RefCell<Led<'p>>>>,
	triangles: Vec<LedTriangle<'p>>,
	gamma: f32,
}

impl<'p> Pwm<'p> {
	/// create a Pwm object with 6 LED driver channels
	pub fn new(
		driver_0: LedcDriver<'p>,
		driver_1: LedcDriver<'p>,
		driver_2: LedcDriver<'p>,
		driver_3: LedcDriver<'p>,
		driver_4: LedcDriver<'p>,
		driver_5: LedcDriver<'p>,
	) -> Result<Self, EspError> {



		// let channel = self.get_channel_by_name(channel_name);


		// TODO: For some advanced features I'd need to re-assign a LED to another driver
		// but keep it on the same pin, or configure it as `off`. So if I have at most
		// 4 LEDs active at all times, I could use up to 2 drivers for non-LED pins.

		// with these 5 leds, only temperatures from 2150 to 6800 can be mapped

		let led_r  = Rc::new(RefCell::new(Led::new(0, "R", driver_0, 0.6400, 0.3500, 165.0)));
		let led_g  = Rc::new(RefCell::new(Led::new(1, "G", driver_1, 0.4070, 0.5370, 460.0)));
		let led_b  = Rc::new(RefCell::new(Led::new(2, "B", driver_2, 0.1470, 0.1100, 130.0)));
		let led_cw = Rc::new(RefCell::new(Led::new(3,"CW", driver_3, 0.3447, 0.3553, 310.0)));
		//let led_ww = Rc::new(RefCell::new(Led::new(4,"WW", driver_4, 0.4334, 0.4030, 220.0)));
		let led_ww = Rc::new(RefCell::new(Led::new(4,"WW", driver_4, 0.5066, 0.4158, 170.0)));
		let led_pa  = Rc::new(RefCell::new(Led::new(5, "PA", driver_5, 0.5650, 0.4250, 230.0)));

		let leds: Vec<Rc<RefCell<Led<'_>>>> = [
			led_r.clone(),
			led_g.clone(),
			led_b.clone(),
			led_cw.clone(),
			led_ww.clone(),
			led_pa.clone(),
		].to_vec();

		// TODO: implement triangulation for an arbitrary number of LED channels 
		// (given as xy color). For now, these are just hardcoded values for the first prototype.
		let t0 = LedTriangle::new(led_r.clone(), led_pa.clone() , led_ww.clone());
		let t1 = LedTriangle::new(led_g.clone(), led_pa.clone() , led_ww.clone());
		let t2 = LedTriangle::new(led_r.clone(), led_cw.clone(), led_ww.clone());
		let t3 = LedTriangle::new(led_g.clone(), led_cw.clone(), led_ww.clone());
		let t4 = LedTriangle::new(led_r.clone(), led_cw.clone(), led_b.clone() );
		let t5 = LedTriangle::new(led_g.clone(), led_cw.clone(), led_b.clone() );	

		let triangles = [
			//t1, t3, t5
			t0, t1, t2, t3, t4, t5
		].to_vec();
	
		return Ok(Self {
			//timer_driver,
			leds,
			triangles,
			gamma: 2.0
		});
	}

	pub fn report(&self) {
		let l_0 = self.leds[0].borrow();
		let l_1 = self.leds[1].borrow();
		let l_2 = self.leds[2].borrow();
		let l_3 = self.leds[3].borrow();
		let l_4 = self.leds[4].borrow();
		let l_5 = self.leds[5].borrow();
		info!("{}: {}, {}: {}, {}: {}, {}: {}, {}: {}, {}: {}, ",
			l_0.name, l_0.driver.get_duty(),
			l_1.name, l_1.driver.get_duty(),
			l_2.name, l_2.driver.get_duty(),
			l_3.name, l_3.driver.get_duty(),
			l_4.name, l_4.driver.get_duty(),
			l_5.name, l_5.driver.get_duty(),
		);
	}

	fn gamma_correct(&self, brightness: f32) -> f32 {
		return brightness.powf(self.gamma);
	}

	// fn channel_share_for_temperature(&self, channel: usize, temperature: f32) -> f32 {
	// 	if channel > 0 && temperature < self.temperatures[channel - 1] {
	// 		return 0.0;
	// 	} else if temperature < self.temperatures[channel] {
	// 		if channel == 0 {
	// 			return 1.0;
	// 		} else {
	// 			return (temperature - self.temperatures[channel - 1]) / (self.temperatures[channel] - self.temperatures[channel - 1])
	// 		}
	// 	} else if channel < 4 && temperature < self.temperatures[channel + 1] {
	// 		return 1.0 - (temperature - self.temperatures[channel]) / (self.temperatures[channel + 1] - self.temperatures[channel])
	// 	} else {
	// 		if channel == 4 {
	// 			return 1.0;
	// 		} else {
	// 			return 0.0;
	// 		}
	// 	}
	// }

	// pub fn set_brightness(self: &mut Self, channel: usize, brightness: f32) {
	// 	let max_duty: u32 = self.drivers[channel].get_max_duty();
    //     let duty: u32 = (brightness * max_duty as f32) as u32;
	// 	self.drivers[channel].set_duty(duty);
	// }

	// pub fn set_brightness_all(self: &mut Self, brightness: f32) {
	// 	for i in 0..self.channel_count {
	// 		self.set_brightness(i, brightness);
	// 	}
	// }

	pub fn temperature_to_xy(mut t: f32) -> Result<XyColor, EspError> {
		let x: f32;
		let y: f32;

		if t < 1005.0 {
			warn!("Can't use temperatures below 1005.0: {}", t);
			t = 1005.0;
		}
		if t < 1667.0 {
			// Interpolate between computation for 1667 K and pure red.
			let a = (1667.0 - t) / 667.0; // amount of red
			let color_1667 = Self::temperature_to_xy(1667.0)?;
			// FIXME these are the xy coordinates of our current red LEDs.
			// According to https://www.waveformlighting.com/tech/calculate-cie-1931-xy-coordinates-from-cct/
			// it should be 0.65275, 0.34446 but the triangulation cannot handle values
			// outside the polygon (yet).
			return Ok(XyColor::new(
				0.628 * a + color_1667.x * (1.0 - a),
				0.295 * a + color_1667.y * (1.0 - a),
			));
		}
		if t > 25000.0 {
			warn!("Can't use temperatures above 25000.0: {}", t);
			t = 25000.0;
		}

		// Formula taken from https://en.wikipedia.org/wiki/Planckian_locus#Approximation
		if 1667.0 <= t && t <= 4000.0 {
			x = -266123900.0 / (t*t*t) - 234359.0 / (t*t) + 877.6956 / t  + 0.179910;
		} else if 4000.0 <= t && t <= 25000.0 {
			x = -3025846900.0 / (t*t*t) + 2107038.0 / (t*t) + 222.6347 / t + 0.240390;
		} else {
			panic!();
		}
		
		if 1667.0 <= t && t <= 2222.0 {
			y = -1.1063814 * (x*x*x) - 1.34811020 * (x*x) + 2.18555832 * x - 0.20219683;
		} else if 2000.0 <= t && t <= 4000.0 {
			y = -0.9549476 * (x*x*x) - 1.37418593 * (x*x) + 2.09137015 * x - 0.16748867;
		} else if 4000.0 <= t && t <= 25000.0 {
			y =  3.0817580 * (x*x*x) - 5.87338670 * (x*x) + 3.75112997 * x - 0.37001483;
		} else {
			panic!();
		}

		// TODO provide some fallback for t < 1667 ?
		// I could not find any textual or mathematical source for it,
		// but most graphics imply that the planck locus ends at
		// x=0.654, y=0.35 which should coincide with the Draper
		// point at 798 K

		return Ok(XyColor {x, y});
	}

	// pub fn set_temperature_and_brightness(self: &mut Self, temperature: f32, brightness: f32) {
	// 	// 1m RGBCCT (2300K - 7000K, 800 lm/m)
	// 	// 1m CCT (2200K - 6800K, 1000 lm/m)
	// 	//   WW sieht eher aus wie 3200K
	// 	//   CC ist noch nicht gemessen
	// 	// 1m Amber (593nm, 1100 lm/m, entspricht 1600K)

	// 	let corrected_brightness = self.gamma_correct(brightness);
	// 	let min_power = 25.0;
	// 	//self.powers.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).expect("could determine max power");

	// 	for i in 0..5 {
	// 		let share = self.channel_share_for_temperature(i, temperature);
	// 		let power_correction = min_power / self.powers[i] * 3.0;
	// 		//info!("  Channel {}: {:.3}", i, share);
	// 		self.set_brightness(i, corrected_brightness * share * power_correction);
	// 	}
	// }

	/// Set the LEDs to the specified color. 
	/// Returns ok if the requested color is invalid (outside of the displayable range) - 
	/// this is not really ok and should be changed to return an error.
	/// Returns an EspError if the color is valid, but some kind of hardware failure happened.
	pub fn set_color(self: &mut Self, color: Xyz<f32>) -> Result<(), EspError>  {
		let xy_color: XyColor = color.into();
		let mut active_leds = Vec::<usize>::new();
		let mut all_duties = [0u32; 8];
		//info!("Target color: {:?} from {}", xy_color, color);
		for triangle in &mut self.triangles {
			//info!("Checking triangle...");
			let bc = triangle.get_barycentric(xy_color.clone());
			if bc[0] >= 0.0 && bc[1] >= 0.0 && bc[2] >= 0.0 {
				//info!("Triangle matches: {:?}", bc);
				for i in 0..3 {
					let mut led = triangle.leds[i].borrow_mut();
					let max_duty: u32 = led.driver.get_max_duty();
					let max_brightness = led.max_brightness;
					let duty: u32 = (bc[i] * color.y() * max_duty as f32 / max_brightness) as u32;
					
					//info!("Setting duty for led {} to {} of {}. ", led.name, duty, max_duty);
					//info!("Brightness of color {} is {}, max_brightness of LED is {}, which makes {}% of maximum.", 
					//	color, color.y(), max_brightness, color.y() / max_brightness * 100.0);
					let hpoint = led.index as u32 * 1000;
					led.driver.set_duty_with_hpoint(duty, hpoint)?;
					all_duties[led.index] = duty;

					active_leds.push(led.index);
				}
				break;
				//return Ok(());
			}
		}

		//info!("D: {:?}", all_duties);

		// Turn off all LEDs that were not turned on just now.
		for i in 0..5 {
			if !active_leds.contains(&i) {
				self.leds[i].borrow_mut().driver.set_duty(0)?;
			}
		}

		if active_leds.is_empty() {
			//warn!("No triangle matched color {:?}!", xy_color);
		}
		return Ok(()); 
	}

	/// Set the LEDs to light with a given color temperature (in K) and brightness ().
	/// Currently, temperatures between _ and _ K are supported.
	/// 
	/// TODO: Implement a check for the power consumption of the given brightness, 
	/// to prevent overheating of the LED module!
	pub fn set_temperature_and_brightness(
		self: &mut Self, 
		temperature: f32, 
		brightness: f32,
	) -> Result<(), EspError> {
		let target_xy: XyColor = Self::temperature_to_xy(temperature)?;
		let target_xyz: Xyz<f32> = target_xy.with_brightness(self.gamma_correct(brightness));
		//info!("set_temperature_and_brightness temperature: {}, brightness: {} (gamma corrected: {}) results in target_xy: {:?}", temperature, brightness, self.gamma_correct(brightness), target_xy);
		//info!("target_xyz: {}", target_xyz);
		return self.set_color(target_xyz);
	}

	pub fn set_amber(self: &mut Self, brightness_up_to_one: f32) -> Result<(), EspError> {
		for i in 0..6 {
			self.leds[i].borrow_mut().driver.set_duty(0)?;
		}
		let amber_driver = &mut self.leds[5].borrow_mut().driver;
		amber_driver.set_duty((amber_driver.get_max_duty() as f32 * brightness_up_to_one) as u32)?;
		Ok(())
	}

	pub fn set_duties(self: &mut Self, brightness_up_to_one: &Vec<f32>) -> Result<(), EspError> {
		info!("Set duties: {:?}", brightness_up_to_one);
		for i in 0..6 {
			let driver = &mut self.leds[i].borrow_mut().driver;
			driver.set_duty((driver.get_max_duty() as f32 * brightness_up_to_one[i]) as u32)?;
		}
		Ok(())
	}
}