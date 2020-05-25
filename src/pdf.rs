use crate::point::Point3;
use crate::ray::Ray;
use crate::shape;
use crate::utils;
use crate::utils::OrthonormalBasis;
use crate::vector::Vector3;

use rand::seq::SliceRandom;
use std::f32;
use std::sync::Arc;

pub enum PDF {
    Cosine(Cosine),
    Shape(Shape),
    Mixture(Mixture),
}

impl PDF {
    pub fn value(&self, r: &Ray) -> f32 {
        match self {
            PDF::Cosine(c) => c.value(r),
            PDF::Shape(s) => s.value(r),
            PDF::Mixture(m) => m.value(r),
        }
    }
    pub fn generate(&self, origin: &Point3) -> Vector3 {
        match self {
            PDF::Cosine(c) => c.generate(),
            PDF::Shape(s) => s.generate(origin),
            PDF::Mixture(m) => m.generate(origin),
        }
    }
    pub fn is_valid(&self) -> bool {
        match self {
            PDF::Cosine(_) => true,
            PDF::Shape(_) => true,
            PDF::Mixture(m) => !m.is_empty(),
        }
    }
}

pub struct Cosine {
    normal: Vector3,
}

impl Cosine {
    pub fn new(v: Vector3) -> Cosine {
        Cosine { normal: v }
    }

    fn value(&self, r: &Ray) -> f32 {
        let cosine = r.dir.normalized().dot(self.normal);
        if cosine < 0.0_f32 {
            0.0_f32
        } else {
            cosine / f32::consts::PI
        }
    }

    fn generate(&self) -> Vector3 {
        OrthonormalBasis::new(&self.normal).local(&utils::random_cosine_direction())
    }
}

pub struct Shape {
    shape: Arc<shape::SyncShape>,
}

impl Shape {
    pub fn new(shape: &Arc<shape::SyncShape>) -> Shape {
        Shape {
            shape: Arc::clone(shape),
        }
    }

    fn value(&self, r: &Ray) -> f32 {
        self.shape.pdf(r)
    }

    fn generate(&self, origin: &Point3) -> Vector3 {
        self.shape.random_dir_towards(origin)
    }
}

pub struct Mixture {
    members: Vec<PDF>,
}

impl Mixture {
    pub fn new(members: Vec<PDF>) -> Mixture {
        Mixture { members: members }
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    fn value(&self, r: &Ray) -> f32 {
        let weight = 1.0_f32 / self.members.len() as f32;
        let mut sum = 0.0_f32;

        for pdf in &self.members {
            sum += weight * pdf.value(r);
        }

        return sum;
    }

    fn generate(&self, origin: &Point3) -> Vector3 {
        match self.members.choose(&mut rand::thread_rng()) {
            Some(m) => m.generate(origin),
            None => panic!("Mixture PDF had no members!"),
        }
    }
}

pub fn pair_value(first: &PDF, second: &PDF, r: &Ray) -> f32 {
    first.value(r) * 0.5_f32 + second.value(r) * 0.5_f32
}

pub fn pair_generate(first: &PDF, second: &PDF, origin: &Point3) -> Vector3 {
    let r = rand::random::<f32>();
    if r < 0.5_f32 {
        first.generate(origin)
    } else {
        second.generate(origin)
    }
}
