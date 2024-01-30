use crate::color::XyColor;
use std::{rc::Rc, error::Error};
use std::cell::RefCell;
use delaunator::{Point, triangulate};

pub struct Led<'p> {
	xy_color: XyColor,
	max_brightness: f32,
	name: &'p str,
}

impl<'p> Led<'p> {
	pub fn new(name: &'p str, x: f32, y: f32, max_brightness: f32) -> Self {
		let xy_color = XyColor {x, y};
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

		return [a, b, c];
	}
}


pub struct LedGroup<'p> {
	leds: Vec<Led<'p>>,
	triangles: Vec<LedTriangle<'p>>,
}

impl<'p> LedGroup<'p> {
	pub fn new() -> Result<Self, i32> {
		Ok(LedGroup{
			leds: Vec::new(),
			triangles: Vec::new(),
		})
	}

	pub fn add_led(&mut self, led: Led<'p>) {
		self.leds.push(led);
		self.triangulate();
	}

	fn triangulate(&mut self) {
		let points : Vec<Point>  = self.leds.iter().map(|led| Into::<Point>::into(led.xy_color.clone())).collect();
		let triangulation = triangulate(&points);
		for geo_triangle in triangulation.triangles.chunks(3) {
			self.triangles.push(LedTriangle::new(
				 Rc::new(RefCell::new(self.leds[geo_triangle[0]])),
				 Rc::new(RefCell::new(self.leds[geo_triangle[1]])),
				 Rc::new(RefCell::new(self.leds[geo_triangle[2]])),
			));
		}
	}
}