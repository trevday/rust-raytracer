use crate::ray::Ray;
use crate::utils;
use crate::vector::Vector3;

use serde::Deserialize;
use std::convert;

#[derive(Deserialize)]
#[serde(from = "CameraDescription")]
pub struct Camera {
	origin: Vector3,
	lower_left_corner: Vector3,
	horizontal: Vector3,
	vertical: Vector3,
	u: Vector3,
	v: Vector3,
	lens_radius: f32,
}

impl Camera {
	pub fn new(
		pos: &Vector3,
		look_at: &Vector3,
		up: &Vector3,
		vertical_fov: f32,
		aspect: f32,
		aperture: f32,
		focus_dist: f32,
	) -> Camera {
		let theta = vertical_fov * (std::f32::consts::PI / 180.0_f32);
		let half_height = (theta / 2.0_f32).tan();
		let half_width = aspect * half_height;

		let w = (*pos - *look_at).normalized();
		let u = up.cross(w).normalized();
		let v = w.cross(u);

		Camera {
			origin: *pos,
			lower_left_corner: *pos
				- (half_width * focus_dist * u)
				- (half_height * focus_dist * v)
				- (w * focus_dist),
			horizontal: 2.0_f32 * half_width * focus_dist * u,
			vertical: 2.0_f32 * half_height * focus_dist * v,
			u: u,
			v: v,
			lens_radius: aperture / 2.0_f32,
		}
	}

	pub fn get_ray(&self, s: f32, t: f32) -> Ray {
		let ray_disk = self.lens_radius * utils::random_unit_disk();
		let offset = self.u * ray_disk.x + self.v * ray_disk.y;

		Ray::new(
			self.origin + offset,
			self.lower_left_corner + (self.horizontal * s) + (self.vertical * t)
				- self.origin - offset,
		)
	}
}

#[derive(Deserialize)]
struct CameraDescription {
	position: Vector3,
	look_at: Vector3,
	up: Vector3,
	fov: f32,
	aspect_ratio: f32,
	aperture: f32,
	focus_distance: f32,
}

impl convert::From<CameraDescription> for Camera {
	fn from(camera_desc: CameraDescription) -> Self {
		Camera::new(
			&camera_desc.position,
			&camera_desc.look_at,
			&camera_desc.up,
			camera_desc.fov,
			camera_desc.aspect_ratio,
			camera_desc.aperture,
			camera_desc.focus_distance,
		)
	}
}
