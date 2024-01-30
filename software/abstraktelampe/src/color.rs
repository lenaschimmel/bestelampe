use core::fmt;
use delaunator::Point;

#[derive(Debug, Clone)]
pub struct ColorBoundsError;

impl fmt::Display for ColorBoundsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Color outside of valid bounds.")
    }
}

#[derive(Clone, Debug)]
pub struct XyColor {
	pub x: f32,
	pub y: f32
}

impl XyColor {
	pub fn new(x: f32, y:f32) -> Self {
		return XyColor{x, y};
	}
}

pub fn temperature_to_xy(mut t: f32) -> Result<XyColor, ColorBoundsError> {
	let x: f32;
	let y: f32;

	if t < 1667.0 {
		return Err(ColorBoundsError); // info!("Can't use temperatures below 1667.0: {}", t);
	}
	if t > 25000.0 {
		return Err(ColorBoundsError);  // info!("Can't use temperatures above 25000.0: {}", t);
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

// impl Into<Point> for XyColor {
//     fn into(self) -> Point {
//         Point { x: self.x as f64, y: self.y as f64 }
//     }
// }

impl From<XyColor> for Point {
    fn from(value: XyColor) -> Self {
        Point { x: value.x as f64, y: value.y as f64 }
    }
}
