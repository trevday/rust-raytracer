use crate::vector::Vector3;

use serde::{Deserialize, Serialize};
use typetag;

#[typetag::serde(tag = "type")]
pub trait Texture {
	fn value(&self, u: f32, v: f32, p: &Vector3) -> Vector3;
}

#[derive(Serialize, Deserialize)]
pub struct Constant {
	color: Vector3,
}
#[typetag::serde]
impl Texture for Constant {
	fn value(&self, _u: f32, _v: f32, _p: &Vector3) -> Vector3 {
		self.color
	}
}

#[derive(Serialize, Deserialize)]
pub struct Test {}
#[typetag::serde]
impl Texture for Test {
	fn value(&self, u: f32, v: f32, _p: &Vector3) -> Vector3 {
		Vector3::new(
			u,
			v,
			if 1.0_f32 - u - v < 0.0_f32 {
				0.0_f32
			} else {
				1.0_f32 - u - v
			},
		)
	}
}

#[derive(Serialize, Deserialize)]
pub struct Checker {
	repeat: f32,
	odd: Box<dyn Texture>,
	even: Box<dyn Texture>,
}
#[typetag::serde]
impl Texture for Checker {
	fn value(&self, u: f32, v: f32, p: &Vector3) -> Vector3 {
		let sines =
			(self.repeat * p.x).sin() * (self.repeat * p.y).sin() * (self.repeat * p.z).sin();
		if sines < 0.0_f32 {
			self.odd.value(u, v, p)
		} else {
			self.even.value(u, v, p)
		}
	}
}
