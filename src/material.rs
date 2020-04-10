use crate::color::RGB;
use crate::point::Point3;
use crate::ray::Ray;
use crate::texture::Texture;
use crate::utils;
use crate::vector::Vector3;

use rand;
use serde::Deserialize;
use std::rc::Rc;

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

pub trait Material {
    // Because of the Rust compiler's optimizations, including use of inlining, implicit pointer
    // arguments, and return value optimization, I think it is ok for functions like this to use
    // multiple return values, some of which are structs, instead of "out" parameters.
    // See: https://stackoverflow.com/questions/35033806/how-does-rust-deal-with-structs-as-function-parameters-and-return-values
    fn scatter(
        &self,
        in_ray: &Ray,
        hit_point: &Point3,
        normal: &Vector3,
        u: f32,
        v: f32,
    ) -> Option<(RGB, Ray)>;

    fn emit(
        &self,
        _in_ray: &Ray,
        _hit_point: &Point3,
        _normal: &Vector3,
        _u: f32,
        _v: f32,
    ) -> Option<RGB> {
        None
    }
}

pub struct Lambert {
    albedo: Rc<dyn Texture>,
}

impl Lambert {
    pub fn new(albedo: Rc<dyn Texture>) -> Lambert {
        Lambert { albedo: albedo }
    }
}

impl Material for Lambert {
    fn scatter(
        &self,
        _in_ray: &Ray,
        hit_point: &Point3,
        normal: &Vector3,
        u: f32,
        v: f32,
    ) -> Option<(RGB, Ray)> {
        let target = *hit_point + *normal + utils::unit_sphere_random();
        Some((
            self.albedo.value(u, v, hit_point),
            Ray::new(*hit_point, target - *hit_point),
        ))
    }
}

pub struct Metal {
    albedo: Rc<dyn Texture>,
    roughness: f32,
}

impl Metal {
    pub fn new(albedo: Rc<dyn Texture>, roughness: f32) -> Metal {
        // Clamp roughness
        let mut r = roughness;
        if r < 0_f32 {
            r = 0_f32;
        } else if r > 1_f32 {
            r = 1_f32;
        }

        Metal {
            albedo: albedo,
            roughness: r,
        }
    }
}

impl Material for Metal {
    fn scatter(
        &self,
        in_ray: &Ray,
        hit_point: &Point3,
        normal: &Vector3,
        u: f32,
        v: f32,
    ) -> Option<(RGB, Ray)> {
        let reflected = reflect(in_ray.dir.normalized(), *normal);
        let out_ray_dir = reflected + self.roughness * utils::unit_sphere_random();

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

#[derive(Deserialize)]
pub struct Dielectric {
    refractive_index: f32,
}

impl Material for Dielectric {
    fn scatter(
        &self,
        in_ray: &Ray,
        hit_point: &Point3,
        normal: &Vector3,
        _u: f32,
        _v: f32,
    ) -> Option<(RGB, Ray)> {
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
            RGB::new(1.0_f32, 1.0_f32, 1.0_f32), // Attenuation is perfect
            out_ray,
        ))
    }
}

pub struct DiffuseLight {
    emission: Rc<dyn Texture>,
}

impl DiffuseLight {
    pub fn new(emission: Rc<dyn Texture>) -> DiffuseLight {
        DiffuseLight { emission: emission }
    }
}

impl Material for DiffuseLight {
    fn scatter(
        &self,
        _in_ray: &Ray,
        _hit_point: &Point3,
        _normal: &Vector3,
        _u: f32,
        _v: f32,
    ) -> Option<(RGB, Ray)> {
        None
    }

    fn emit(
        &self,
        _in_ray: &Ray,
        hit_point: &Point3,
        _normal: &Vector3,
        u: f32,
        v: f32,
    ) -> Option<RGB> {
        Some(self.emission.value(u, v, hit_point))
    }
}
