use crate::aggregate::AABB;
use crate::color::RGB;
use crate::material::Material;
use crate::ray::Ray;
use crate::shape::HitProperties;
use crate::shape::Shape;
use crate::texture::Texture;
use crate::utils::unit_sphere_random;

use rand;
use std::rc::Rc;

// TODO: Separate Phase Functions from Materials, and make them specific to Mediums
trait PhaseFunction: Material {}

pub struct Isotropic {
    albedo: Rc<dyn Texture>,
}

impl Isotropic {
    pub fn new(albedo: Rc<dyn Texture>) -> Isotropic {
        Isotropic { albedo: albedo }
    }
}

impl Material for Isotropic {
    fn scatter(&self, _in_ray: &Ray, hit_props: &HitProperties) -> Option<(RGB, Ray)> {
        Some((
            self.albedo
                .value(hit_props.u, hit_props.v, &hit_props.hit_point),
            Ray::new(hit_props.hit_point, unit_sphere_random()),
        ))
    }
}

// TODO: Separate Mediums from shapes, such that a shape can have a medium, but a medium
// does not need to be a shape; add Medium trait
pub struct ConstantMedium {
    boundary: Box<dyn Shape>,
    density: f32,
    phase_func: Rc<dyn Material>,
}

impl ConstantMedium {
    pub fn new(
        boundary: Box<dyn Shape>,
        density: f32,
        phase_func: Rc<dyn Material>,
    ) -> ConstantMedium {
        ConstantMedium {
            boundary: boundary,
            density: density,
            phase_func: phase_func,
        }
    }
}

impl Shape for ConstantMedium {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<f32> {
        let mut t_hit1 = match self.boundary.hit(r, -std::f32::MAX, std::f32::MAX) {
            Some(t) => t,
            None => return None,
        };

        let mut t_hit2 = match self
            .boundary
            .hit(r, t_hit1 + std::f32::EPSILON, std::f32::MAX)
        {
            Some(t) => t,
            None => return None,
        };

        if t_hit1 < t_min {
            t_hit1 = t_min;
        }
        if t_hit2 < t_max {
            t_hit2 = t_max;
        }

        if t_hit1 >= t_hit2 {
            return None;
        }

        let distance_inside_boundary = (t_hit2 - t_hit1) * r.dir.length();
        let hit_dist = (-1.0_f32 / self.density) * rand::random::<f32>().ln();

        if hit_dist > distance_inside_boundary {
            return None;
        }

        return Some(t_hit1 + (hit_dist / r.dir.length()));
    }

    fn get_hit_properties(&self, r: &Ray, t_hit: f32) -> HitProperties {
        self.boundary.get_hit_properties(r, t_hit)
    }

    fn get_material(&self) -> &Rc<dyn Material> {
        &self.phase_func
    }

    fn get_bounding_box(&self) -> AABB {
        self.boundary.get_bounding_box()
    }
}