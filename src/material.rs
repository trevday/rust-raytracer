use crate::ray::Ray;
use crate::texture::Texture;
use crate::utils;
use crate::vector::Vector3;

use rand;
use serde::{Deserialize, Serialize};
use typetag;

fn reflect(v: Vector3, n: Vector3) -> Vector3 {
    v - 2.0_f32 * v.dot(n) * n
}

fn refract(v: &Vector3, n: &Vector3, refracted_index: f32) -> Option<Vector3> {
    let normalized = v.normalized();
    let dt = normalized.dot(*n);
    let discriminant = 1.0_f32 - refracted_index * refracted_index * (1.0_f32 - dt * dt);

    if discriminant > 0.0_f32 {
        Some(refracted_index * (normalized - (*n) * dt) - (*n) * discriminant.sqrt())
    } else {
        None
    }
}

fn schlick(cosine: f32, index: f32) -> f32 {
    let mut r0 = (1.0_f32 - index) / (1.0_f32 + index);
    r0 = r0 * r0;
    r0 + (1.0_f32 - r0) * (1.0_f32 - cosine).powi(5)
}

#[typetag::serde(tag = "type")]
pub trait Material {
    // Because of the Rust compiler's optimizations, including use of inlining, implicit pointer
    // arguments, and return value optimization, I think it is ok for functions like this to use
    // multiple return values, some of which are structs, instead of "out" parameters.
    // See: https://stackoverflow.com/questions/35033806/how-does-rust-deal-with-structs-as-function-parameters-and-return-values
    fn scatter(
        &self,
        in_ray: &Ray,
        hit_point: &Vector3,
        normal: &Vector3,
        u: f32,
        v: f32,
    ) -> Option<(Vector3, Ray)>;
}

#[derive(Serialize, Deserialize)]
pub struct Lambert {
    albedo: Box<dyn Texture>,
}

#[typetag::serde]
impl Material for Lambert {
    fn scatter(
        &self,
        _in_ray: &Ray,
        hit_point: &Vector3,
        normal: &Vector3,
        u: f32,
        v: f32,
    ) -> Option<(Vector3, Ray)> {
        let target = *hit_point + *normal + utils::unit_sphere_random();
        Some((
            self.albedo.value(u, v, hit_point),
            Ray::new(*hit_point, target - *hit_point),
        ))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Metal {
    albedo: Box<dyn Texture>,
    roughness: f32,
}

#[typetag::serde]
impl Material for Metal {
    fn scatter(
        &self,
        in_ray: &Ray,
        hit_point: &Vector3,
        normal: &Vector3,
        u: f32,
        v: f32,
    ) -> Option<(Vector3, Ray)> {
        // TODO: Perform this check and constraint upon JSON
        // deserialization.
        let r = if self.roughness < 0_f32 {
            0_f32
        } else if self.roughness > 1_f32 {
            1_f32
        } else {
            self.roughness
        };

        let reflected = reflect(in_ray.dir.normalized(), *normal);
        let out_ray_dir = reflected + r * utils::unit_sphere_random();

        if out_ray_dir.dot(*normal) > 0.0_f32 {
            Some((
                self.albedo.value(u, v, hit_point),
                Ray::new(*hit_point, out_ray_dir),
            ))
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Dielectric {
    refractive_index: f32,
}

#[typetag::serde]
impl Material for Dielectric {
    fn scatter(
        &self,
        in_ray: &Ray,
        hit_point: &Vector3,
        normal: &Vector3,
        _u: f32,
        _v: f32,
    ) -> Option<(Vector3, Ray)> {
        let normal_dot = in_ray.dir.dot(*normal);

        let (outward_normal, index, cosine) = if normal_dot > 0.0_f32 {
            let normal_dot_div = normal_dot / in_ray.dir.length();
            (
                -(*normal),
                self.refractive_index,
                (1.0_f32
                    - self.refractive_index
                        * self.refractive_index
                        * (1.0_f32 - normal_dot_div * normal_dot_div))
                    .sqrt(),
            )
        } else {
            (
                *normal,
                1.0_f32 / self.refractive_index,
                -normal_dot / in_ray.dir.length(),
            )
        };

        let out_ray = match refract(&in_ray.dir, &outward_normal, index) {
            Some(refracted) => {
                let prob = schlick(cosine, self.refractive_index);

                if rand::random::<f32>() < prob {
                    Ray::new(*hit_point, reflect(in_ray.dir, *normal))
                } else {
                    Ray::new(*hit_point, refracted)
                }
            }
            None => Ray::new(*hit_point, reflect(in_ray.dir, *normal)),
        };

        Some((
            Vector3::new(1.0_f32, 1.0_f32, 1.0_f32), // Attenuation is perfect
            out_ray,
        ))
    }
}
