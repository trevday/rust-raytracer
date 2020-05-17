use crate::point::Point3;
use crate::ray::Ray;
use crate::shape;
use crate::utils;
use crate::utils::OrthonormalBasis;
use crate::vector::Vector3;

use rand::seq::SliceRandom;
use std::f32;
use std::rc::Rc;

pub trait PDF {
    fn value(&self, dir: &Vector3) -> f32;
    fn generate(&self) -> Vector3;
}

pub struct Cosine {
    onb: OrthonormalBasis,
}

impl Cosine {
    pub fn new(v: &Vector3) -> Cosine {
        Cosine {
            onb: OrthonormalBasis::new(v),
        }
    }
}

impl PDF for Cosine {
    fn value(&self, dir: &Vector3) -> f32 {
        let cosine = dir.normalized().dot(self.onb.w());
        if cosine < 0.0_f32 {
            0.0_f32
        } else {
            cosine / f32::consts::PI
        }
    }

    fn generate(&self) -> Vector3 {
        self.onb.local(&utils::random_cosine_direction())
    }
}

pub struct Shape {
    shape: Rc<dyn shape::Shape>,
    origin: Point3,
}

impl Shape {
    pub fn new(shape: &Rc<dyn shape::Shape>, origin: &Point3) -> Shape {
        Shape {
            shape: Rc::clone(shape),
            origin: *origin,
        }
    }
}

impl PDF for Shape {
    fn value(&self, dir: &Vector3) -> f32 {
        self.shape.pdf(&Ray::new(self.origin, *dir))
    }

    fn generate(&self) -> Vector3 {
        self.shape.random_dir_towards(&self.origin)
    }
}

pub struct Mixture {
    members: Vec<Rc<dyn PDF>>,
}

impl Mixture {
    pub fn new(members: Vec<Rc<dyn PDF>>) -> Result<Mixture, &'static str> {
        if members.is_empty() {
            Err("Tried to construct a Mixture PDF with no members!")
        } else {
            Ok(Mixture { members: members })
        }
    }
}

impl PDF for Mixture {
    fn value(&self, dir: &Vector3) -> f32 {
        let weight = 1.0_f32 / self.members.len() as f32;
        let mut sum = 0.0_f32;

        for pdf in &self.members {
            sum += weight * pdf.value(dir);
        }

        return sum;
    }

    fn generate(&self) -> Vector3 {
        match self.members.choose(&mut rand::thread_rng()) {
            Some(m) => m.generate(),
            None => panic!("Mixture PDF had no members, this is not expected behavior."),
        }
    }
}
