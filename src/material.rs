use crate::color::RGB;
use crate::pdf;
use crate::pdf::PDF;
use crate::ray::Ray;
use crate::shape::HitProperties;
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

pub enum Reflectance {
    Specular(Ray),
    PDF(Rc<dyn PDF>),
}
pub struct ScatterProperties {
    pub reflectance: Reflectance,
    pub attenuation: RGB,
}

pub trait Material {
    // Because of the Rust compiler's optimizations, including use of inlining, implicit pointer
    // arguments, and return value optimization, I think it is ok for functions like this to use
    // multiple return values, some of which are structs, instead of "out" parameters.
    // See: https://stackoverflow.com/questions/35033806/how-does-rust-deal-with-structs-as-function-parameters-and-return-values
    fn scatter(&self, in_ray: &Ray, hit_props: &HitProperties) -> Option<ScatterProperties>;

    fn emit(&self, _in_ray: &Ray, _hit_props: &HitProperties) -> Option<RGB> {
        None
    }

    // Reflects whether a Material has some importance for shading in a scene,
    // usually indicates that a Material emits light or that it will reflect
    // other sources of light. If true, more rays will be sent in this Material's
    // direction during tracing.
    fn is_important(&self) -> bool;
}

pub struct Lambert {
    albedo: Rc<dyn Texture>,
    // TODO: Expose to other materials, such as Metal
    bump_map: Option<Rc<dyn Texture>>,
}

impl Lambert {
    pub fn new(albedo: Rc<dyn Texture>, bump_map: Option<Rc<dyn Texture>>) -> Lambert {
        Lambert {
            albedo: albedo,
            bump_map: bump_map,
        }
    }
}

const BUMP_DELTA: f32 = 0.005_f32; // TODO: Make bump delta dynamic
impl Material for Lambert {
    fn scatter(&self, _in_ray: &Ray, hit_props: &HitProperties) -> Option<ScatterProperties> {
        // Apply bump map if present
        // https://www.microsoft.com/en-us/research/wp-content/uploads/1978/01/p286-blinn.pdf
        let bump_modified_normal = match &self.bump_map {
            None => hit_props.normal,
            Some(b) => {
                // Get base value of bump at u, v, p
                let displacement = b.bump_value(hit_props.u, hit_props.v, &hit_props.hit_point);
                // Create partial derivatives for bump
                // by shifting u, v, and p
                let displacement_u = b.bump_value(
                    hit_props.u + BUMP_DELTA,
                    hit_props.v,
                    &(hit_props.hit_point + BUMP_DELTA * hit_props.pu),
                );
                let displacement_v = b.bump_value(
                    hit_props.u,
                    hit_props.v + BUMP_DELTA,
                    &(hit_props.hit_point + BUMP_DELTA * hit_props.pv),
                );

                // Determine new Pu and Pv
                let new_pu = hit_props.pu
                    + ((displacement_u - displacement) / BUMP_DELTA) * hit_props.normal;
                let new_pv = hit_props.pv
                    + ((displacement_v - displacement) / BUMP_DELTA) * hit_props.normal;

                // Cross product of displaced Pu and Pv yields the new normal
                new_pu.cross(new_pv).normalized()
            }
        };

        Some(ScatterProperties {
            // TODO: Avoid an allocation here?
            reflectance: Reflectance::PDF(Rc::new(pdf::Cosine::new(&bump_modified_normal))),
            attenuation: self
                .albedo
                .value(hit_props.u, hit_props.v, &hit_props.hit_point),
        })
    }

    fn is_important(&self) -> bool {
        false
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
    fn scatter(&self, in_ray: &Ray, hit_props: &HitProperties) -> Option<ScatterProperties> {
        let reflected = reflect(in_ray.dir.normalized(), hit_props.normal);
        let out_ray_dir = reflected + self.roughness * utils::unit_sphere_random();

        Some(ScatterProperties {
            reflectance: Reflectance::Specular(Ray::new(hit_props.hit_point, out_ray_dir)),
            attenuation: self
                .albedo
                .value(hit_props.u, hit_props.v, &hit_props.hit_point),
        })
    }

    fn is_important(&self) -> bool {
        true
    }
}

#[derive(Deserialize)]
pub struct Dielectric {
    refractive_index: f32,
}

impl Material for Dielectric {
    fn scatter(&self, in_ray: &Ray, hit_props: &HitProperties) -> Option<ScatterProperties> {
        let normal_dot = in_ray.dir.dot(hit_props.normal);

        let (outward_normal, index, cosine) = if normal_dot > 0.0_f32 {
            let normal_dot_div = normal_dot / in_ray.dir.length();
            (
                -(hit_props.normal),
                self.refractive_index,
                (1.0_f32
                    - self.refractive_index
                        * self.refractive_index
                        * (1.0_f32 - normal_dot_div * normal_dot_div))
                    .sqrt(),
            )
        } else {
            (
                hit_props.normal,
                1.0_f32 / self.refractive_index,
                -normal_dot / in_ray.dir.length(),
            )
        };

        let out_ray = match refract(&in_ray.dir, &outward_normal, index) {
            Some(refracted) => {
                let prob = schlick(cosine, self.refractive_index);

                if rand::random::<f32>() < prob {
                    Ray::new(hit_props.hit_point, reflect(in_ray.dir, hit_props.normal))
                } else {
                    Ray::new(hit_props.hit_point, refracted)
                }
            }
            None => Ray::new(hit_props.hit_point, reflect(in_ray.dir, hit_props.normal)),
        };

        Some(ScatterProperties {
            reflectance: Reflectance::Specular(out_ray),
            attenuation: RGB::new(1.0_f32, 1.0_f32, 1.0_f32), // Attenuation is perfect
        })
    }

    fn is_important(&self) -> bool {
        true
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
    fn scatter(&self, _in_ray: &Ray, _hit_props: &HitProperties) -> Option<ScatterProperties> {
        None
    }

    fn emit(&self, _in_ray: &Ray, hit_props: &HitProperties) -> Option<RGB> {
        Some(
            self.emission
                .value(hit_props.u, hit_props.v, &hit_props.hit_point),
        )
    }

    fn is_important(&self) -> bool {
        true
    }
}
