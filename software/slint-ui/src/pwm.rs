use log::info;
use prisma::Xyz;

use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, Debug)]
pub struct XyColor {
	pub x: f32,
	pub y: f32
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

	#[allow(non_snake_case)]
	pub fn with_brightness(&self, Y: f32) -> Xyz<f32> {
		let z = 1.0 - (self.x+self.y);
		let X = (Y / self.y) * self.x;
		let Z = (Y / self.y) * z;
	
		return Xyz::new(X, Y, Z);
	}
}

pub struct Pwm<'p> {
	//timer_driver: LedcTimerDriver<'p>,
	leds: Vec<Rc<RefCell<Led<'p>>>>,
	triangles: Vec<LedTriangle<'p>>,
	gamma: f32,
}

pub struct Led<'p> {
	//driver: LD,
	xy_color: XyColor,
	//xyz_color: Xyz<f32>,
	max_brightness: f32,
	name: &'p str,
}

impl<'p> Led<'p> {
	pub fn new(name: &'p str, x: f32, y: f32, max_brightness: f32) -> Self {
		let xy_color = XyColor {x, y};
		//let xyz_color = xy_color.with_brightness(max_brightness);
		return Self { name, xy_color, max_brightness };
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

impl<'p> Pwm<'p> {
	pub fn new() -> Result<Self, anyhow::Error> {

		// TODO: For some advanced features I'd need to re-assign a LED to another driver
		// but keep it on the same pin, or configure it as `off`. So if I have at most
		// 4 LEDs active at all times, I could use up to 2 drivers for non-LED pins.


		let led_r  = Rc::new(RefCell::new(Led::new( "R", 0.630, 0.295,  25.0)));
		let led_g  = Rc::new(RefCell::new(Led::new( "G", 0.153, 0.682,  98.0)));
		let led_b  = Rc::new(RefCell::new(Led::new( "B", 0.146, 0.058,  48.0)));
		let led_cw = Rc::new(RefCell::new(Led::new("CW", 0.317, 0.318,  40.0)));
		let led_ww = Rc::new(RefCell::new(Led::new("WW", 0.485, 0.394,  29.0)));
		let led_a  = Rc::new(RefCell::new(Led::new( "A", 0.573, 0.421, 110.0)));

		let leds: Vec<Rc<RefCell<Led<'_>>>> = [
			led_r.clone(),
			led_g.clone(),
			led_b.clone(),
			led_cw.clone(),
			led_ww.clone(),
			led_a.clone(),
		].to_vec();

		let t0 = LedTriangle::new(led_r.clone(), led_a.clone() , led_ww.clone());
		let t1 = LedTriangle::new(led_g.clone(), led_a.clone() , led_ww.clone());
		let t2 = LedTriangle::new(led_r.clone(), led_cw.clone(), led_ww.clone());
		let t3 = LedTriangle::new(led_g.clone(), led_cw.clone(), led_ww.clone());
		let t4 = LedTriangle::new(led_r.clone(), led_cw.clone(), led_b.clone() );
		let t5 = LedTriangle::new(led_g.clone(), led_cw.clone(), led_b.clone() );

		let triangles = [
			t0, t1, t2, t3, t4, t5
		].to_vec();
	
		return Ok(Self {
			//timer_driver,
			leds,
			triangles,
			gamma: 2.0
		});
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

	pub fn temperature_to_xy(mut t: f32) -> Result<XyColor, anyhow::Error> {
		let x: f32;
		let y: f32;

		if t < 1667.0 {
			info!("Can't use temperatures below 1667.0: {}", t);
			t = 1667.0;
		}
		if t > 25000.0 {
			info!("Can't use temperatures above 25000.0: {}", t);
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

	pub fn set_color(self: &mut Self, color: Xyz<f32>) -> Result<(), anyhow::Error>  {
		let xy_color: XyColor = color.into();
		info!("Target color: {:?} from {}", xy_color, color);
		for triangle in &mut self.triangles {
			//info!("Checking triangle...");
			let bc = triangle.get_barycentric(xy_color.clone());
			if bc[0] >= 0.0 && bc[1] >= 0.0 && bc[2] >= 0.0 {
				//info!("Triangle matches: {:?}", bc);
				for i in 0..3 {
					let mut led = triangle.leds[i].borrow_mut();
					let max_duty: u32 = 2_u32.pow(16); // led.driver.get_max_duty();
					let max_brightness = led.max_brightness;
					let duty: u32 = (bc[i] * color.y() * max_duty as f32 / max_brightness) as u32;
					
					//info!("Setting duty for led {} to {} of {}. ", led.name, duty, max_duty);
					//info!("Brightness of color {} is {}, max_brightness of LED is {}, which makes {}% of maximum.", 
					//	color, color.y(), max_brightness, color.y() / max_brightness * 100.0);
					//led.driver.set_duty(duty)?;
				}
				return Ok(());
			}
		}

		info!("No triangle matched color {:?}!", xy_color);
		return Ok(()); // Not really ok.
	}

	pub fn set_temperature_and_brightness(self: &mut Self, temperature: f32, brightness: f32) -> Result<(), anyhow::Error> {
		let target_xy = Self::temperature_to_xy(temperature)?;
		let target_xyz: Xyz<f32> = target_xy.with_brightness(self.gamma_correct(brightness));
		info!("set_temperature_and_brightness temperature: {}, brightness: {} (gamma corrected: {}) results in target_xy: {:?}", temperature, brightness, self.gamma_correct(brightness), target_xy);
		//info!("target_xyz: {}", target_xyz);
		return self.set_color(target_xyz);
	}
}