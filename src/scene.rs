use crate::camera::Camera;
use crate::shape::Shape;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Scene {
	pub logistics: Logistics,
	pub camera: Camera,
	pub shapes: Vec<Box<dyn Shape>>,
}

#[derive(Deserialize)]
pub struct Logistics {
	pub resolution_x: u32,
	pub resolution_y: u32,
	pub samples: u32,
}
