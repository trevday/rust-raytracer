use crate::material::Material;
use crate::ray::Ray;
use crate::vector::Vector3;

use serde::{Deserialize, Serialize};
use typetag;

#[typetag::serde(tag = "type")]
pub trait Shape {
	fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<f32>;
	fn derive_normal(&self, point: &Vector3) -> Vector3;
	fn get_material(&self) -> &Box<dyn Material>;
}

const MAX_DEPTH: i32 = 50;
const T_MIN: f32 = 0.001_f32;

pub fn trace(
	r: &Ray,
	shapes: &Vec<Box<dyn Shape>>,
	bg_func: &dyn Fn(&Ray) -> Vector3,
	depth: i32,
) -> Vector3 {
	let mut t_max = std::f32::MAX;
	let mut hit_shape: Option<&dyn Shape> = None;

	// Check shapes to see if we have a hit
	for shape in shapes {
		match shape.hit(r, T_MIN, t_max) {
			Some(t) => {
				t_max = t;
				hit_shape = Some(&(*(*shape)));
			}
			// No-op
			None => {}
		}
	}

	if depth < MAX_DEPTH {
		match hit_shape {
			// Some if we have a hit
			Some(s) => {
				let hit_point = r.point_at(t_max);
				let normal = s.derive_normal(&hit_point);

				match s.get_material().scatter(r, &hit_point, &normal) {
					// Some if we scattered
					Some((attenuation, scattered_ray)) => {
						// Recursive case
						return attenuation * trace(&scattered_ray, shapes, bg_func, depth + 1);
					}
					None => {
						return Vector3::new_empty();
					}
				}
			}
			// None if we don't, no-op
			None => {}
		}
	}

	// Return BG color
	return bg_func(r);
}

#[derive(Serialize, Deserialize)]
pub struct Sphere {
	center: Vector3,
	radius: f32,
	// NOTE: There is a tradeoff here between making an enum struct and a Box to a trait object.
	// The enum struct would be slightly more efficient as it is immediately available
	// for use without having to reach into the Heap, but adding new variants is more
	// troublesome, and especially large variants may make the required size of each
	// Material too large. The Box + trait object allows easier creation of Material
	// variants, but introduces a small performance penalty to read from Heap memory.
	//
	// TODO: Further investigate Box-Enum, performance vs. memory tradeoff if
	// optimization is required.
	material: Box<dyn Material>,
}

#[typetag::serde]
impl Shape for Sphere {
	fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<f32> {
		let towards_origin = r.origin - self.center;
		let a = r.dir.dot(r.dir);
		let b = 2.0_f32 * towards_origin.dot(r.dir);
		let c = towards_origin.dot(towards_origin) - (self.radius * self.radius);
		let discriminant = b * b - 4.0_f32 * a * c;

		if discriminant > 0.0_f32 {
			let mut t_hit = (-b - discriminant.sqrt()) / (2.0_f32 * a);
			if t_hit >= t_max || t_hit <= t_min {
				t_hit = (-b + discriminant.sqrt()) / (2.0_f32 * a);
			}

			if t_hit < t_max && t_hit > t_min {
				return Some(t_hit);
			}
		}
		return None;
	}

	fn derive_normal(&self, point: &Vector3) -> Vector3 {
		((*point - self.center) / self.radius).normalized()
	}

	fn get_material(&self) -> &Box<dyn Material> {
		&self.material
	}
}
