use crate::color::RGB;
use crate::pdf;
use crate::pdf::PDF;
use crate::ray::Ray;
use crate::shape::HitProperties;
use crate::texture::SyncTexture;
use crate::utils;
use crate::vector::Vector3;

use rand;
use serde::Deserialize;
use std::sync::Arc;

fn reflect(v: Vector3, n: Vector3) -> Vector3 {
    v - 2.0_f32 * v.dot(n) * n
}

fn refract(v: Vector3, n: Vector3, refracted_index: f32) -> Vector3 {
    let cos_theta = (-v).dot(n);
    let r_out_parallel = refracted_index * (v + cos_theta * n);
    let r_out_perp = (-(1.0_f32 - r_out_parallel.squared_length()).sqrt()) * n;
    return r_out_parallel + r_out_perp;
}

fn schlick(cosine: f32, index: f32) -> f32 {
    let mut r0 = (1.0_f32 - index) / (1.0_f32 + index);
    r0 = r0 * r0;
    r0 + (1.0_f32 - r0) * (1.0_f32 - cosine).powi(5)
}

pub enum Reflectance {
    Specular(Ray),
    PDF(PDF),
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
pub type SyncMaterial = dyn Material + Send + Sync;

pub struct Lambert {
    albedo: Arc<SyncTexture>,
    // TODO: Expose to other materials, such as Metal
    bump_map: Option<Arc<SyncTexture>>,
}

impl Lambert {
    pub fn new(albedo: Arc<SyncTexture>, bump_map: Option<Arc<SyncTexture>>) -> Lambert {
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
            reflectance: Reflectance::PDF(PDF::Cosine(pdf::Cosine::new(bump_modified_normal))),
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
    albedo: Arc<SyncTexture>,
    roughness: f32,
}

impl Metal {
    pub fn new(albedo: Arc<SyncTexture>, roughness: f32) -> Metal {
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
        let attenuation = RGB::new(1.0_f32, 1.0_f32, 1.0_f32); // Attenuation is perfect
        let (etai_over_etat, normal_for_use) = if in_ray.dir.dot(hit_props.normal) < 0.0_f32 {
            (1.0 / self.refractive_index, hit_props.normal)
        } else {
            (self.refractive_index, -hit_props.normal)
        };

        let unit_direction = in_ray.dir.normalized();
        let cos_theta = utils::float_min((-unit_direction).dot(normal_for_use), 1.0_f32);
        let sin_theta = (1.0_f32 - cos_theta * cos_theta).sqrt();

        if etai_over_etat * sin_theta > 1.0_f32 {
            let reflected = reflect(unit_direction, normal_for_use);
            return Some(ScatterProperties {
                reflectance: Reflectance::Specular(Ray::new(hit_props.hit_point, reflected)),
                attenuation: attenuation,
            });
        }

        let reflect_prob = schlick(cos_theta, etai_over_etat);
        if rand::random::<f32>() < reflect_prob {
            let reflected = reflect(unit_direction, normal_for_use);
            return Some(ScatterProperties {
                reflectance: Reflectance::Specular(Ray::new(hit_props.hit_point, reflected)),
                attenuation: attenuation,
            });
        }

        let refracted = refract(unit_direction, normal_for_use, etai_over_etat);
        return Some(ScatterProperties {
            reflectance: Reflectance::Specular(Ray::new(hit_props.hit_point, refracted)),
            attenuation: attenuation,
        });
    }

    fn is_important(&self) -> bool {
        true
    }
}

pub struct DiffuseLight {
    emission: Arc<SyncTexture>,
}

impl DiffuseLight {
    pub fn new(emission: Arc<SyncTexture>) -> DiffuseLight {
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
