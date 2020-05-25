use crate::base::BasicThreeTuple;
use crate::vector::Axis;
use crate::vector::Vector3;

use serde::Deserialize;
use std::convert;
use std::ops;
use wavefront_obj::obj;

#[derive(Deserialize)]
#[serde(try_from = "Vec<f32>")]
pub struct Point3(pub BasicThreeTuple<f32>);

impl Copy for Point3 {}
impl Clone for Point3 {
    fn clone(&self) -> Point3 {
        *self
    }
}

impl Point3 {
    pub fn origin() -> Point3 {
        Point3(BasicThreeTuple::new(0_f32, 0_f32, 0_f32))
    }

    pub fn new(x: f32, y: f32, z: f32) -> Point3 {
        Point3(BasicThreeTuple::new(x, y, z))
    }

    pub fn x(&self) -> f32 {
        self.0.x
    }
    pub fn y(&self) -> f32 {
        self.0.y
    }
    pub fn z(&self) -> f32 {
        self.0.z
    }

    pub fn min(v1: Point3, v2: Point3) -> Point3 {
        Point3(BasicThreeTuple::min(v1.0, v2.0))
    }

    pub fn max(v1: Point3, v2: Point3) -> Point3 {
        Point3(BasicThreeTuple::max(v1.0, v2.0))
    }
}

impl ops::Add<Vector3> for Point3 {
    type Output = Point3;
    fn add(self, rhs: Vector3) -> Point3 {
        Point3(self.0.add(rhs.0))
    }
}

impl ops::Add for Point3 {
    type Output = Point3;
    fn add(self, rhs: Point3) -> Point3 {
        Point3(self.0.add(rhs.0))
    }
}

impl ops::Sub for Point3 {
    type Output = Vector3;
    fn sub(self, rhs: Point3) -> Vector3 {
        Vector3(self.0.sub(rhs.0))
    }
}

impl ops::Sub<Vector3> for Point3 {
    type Output = Point3;
    fn sub(self, rhs: Vector3) -> Point3 {
        Point3(self.0.sub(rhs.0))
    }
}

impl ops::Mul<f32> for Point3 {
    type Output = Point3;
    fn mul(self, rhs: f32) -> Point3 {
        Point3(self.0.mul(rhs))
    }
}

impl convert::TryFrom<Vec<f32>> for Point3 {
    type Error = &'static str;

    fn try_from(vec: Vec<f32>) -> Result<Self, Self::Error> {
        if vec.len() != 3 {
            Err("Deserializing in to Point3 requires a Vec of length 3!")
        } else {
            Ok(Point3::new(vec[0], vec[1], vec[2]))
        }
    }
}

impl convert::From<obj::Vertex> for Point3 {
    fn from(vertex: obj::Vertex) -> Self {
        Point3::new(vertex.x as f32, vertex.y as f32, vertex.z as f32)
    }
}

impl ops::Index<Axis> for Point3 {
    type Output = f32;
    fn index(&self, index: Axis) -> &f32 {
        match index {
            Axis::X => &self.0.x,
            Axis::Y => &self.0.y,
            Axis::Z => &self.0.z,
        }
    }
}
