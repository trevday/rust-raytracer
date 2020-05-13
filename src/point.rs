use crate::vector::Axis;
use crate::vector::Vector3;

use serde::{Deserialize, Serialize};
use std::convert;
use std::ops;
use wavefront_obj::obj;

#[derive(Serialize, Deserialize)]
#[serde(try_from = "Vec<f32>")]
pub struct Point3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Copy for Point3 {}
impl Clone for Point3 {
    fn clone(&self) -> Point3 {
        *self
    }
}

impl Point3 {
    pub fn origin() -> Point3 {
        Point3 {
            x: 0_f32,
            y: 0_f32,
            z: 0_f32,
        }
    }

    pub fn new(x: f32, y: f32, z: f32) -> Point3 {
        Point3 { x: x, y: y, z: z }
    }

    pub fn min(v1: Point3, v2: Point3) -> Point3 {
        Point3 {
            x: if v1.x < v2.x { v1.x } else { v2.x },
            y: if v1.y < v2.y { v1.y } else { v2.y },
            z: if v1.z < v2.z { v1.z } else { v2.z },
        }
    }

    pub fn max(v1: Point3, v2: Point3) -> Point3 {
        Point3 {
            x: if v1.x > v2.x { v1.x } else { v2.x },
            y: if v1.y > v2.y { v1.y } else { v2.y },
            z: if v1.z > v2.z { v1.z } else { v2.z },
        }
    }
}

impl ops::Add<Vector3> for Point3 {
    type Output = Point3;
    fn add(self, rhs: Vector3) -> Point3 {
        Point3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::Sub for Point3 {
    type Output = Vector3;
    fn sub(self, rhs: Point3) -> Vector3 {
        Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Sub<Vector3> for Point3 {
    type Output = Point3;
    fn sub(self, rhs: Vector3) -> Point3 {
        Point3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Mul<f32> for Point3 {
    type Output = Point3;
    fn mul(self, rhs: f32) -> Point3 {
        Point3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
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
        // TODO: Expand precision?
        Point3::new(vertex.x as f32, vertex.y as f32, vertex.z as f32)
    }
}

impl ops::Index<Axis> for Point3 {
    type Output = f32;
    fn index(&self, index: Axis) -> &f32 {
        match index {
            Axis::X => &self.x,
            Axis::Y => &self.y,
            Axis::Z => &self.z,
        }
    }
}