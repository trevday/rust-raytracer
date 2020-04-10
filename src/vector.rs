use serde::Deserialize;
use std::convert;
use std::ops;

#[derive(Deserialize)]
#[serde(try_from = "Vec<f32>")]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// Vector3 implements the Copy trait because it is a small, constant piece
// of data. Vector3's are, ideally, not widely mutated. The compiler
// will aid in optimizing the copy process, such that excess copies
// are not required at runtime.
// Further refactoring of the Vector class may make it unappealing to
// copy if the included data grows larger than three 32-bit floats,
// and at that time it should be considered whether this trait
// should be removed.
impl Copy for Vector3 {}
impl Clone for Vector3 {
    fn clone(&self) -> Vector3 {
        *self
    }
}

// Short functions in this file should always be inlined by the compiler.
// https://doc.rust-lang.org/1.25.0/reference/attributes.html#inline-attribute
impl Vector3 {
    pub fn new_empty() -> Vector3 {
        Vector3 {
            x: 0_f32,
            y: 0_f32,
            z: 0_f32,
        }
    }

    pub fn new_identity() -> Vector3 {
        Vector3 {
            x: 1_f32,
            y: 1_f32,
            z: 1_f32,
        }
    }

    pub fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3 { x: x, y: y, z: z }
    }

    pub fn min(v1: Vector3, v2: Vector3) -> Vector3 {
        Vector3 {
            x: if v1.x < v2.x { v1.x } else { v2.x },
            y: if v1.y < v2.y { v1.y } else { v2.y },
            z: if v1.z < v2.z { v1.z } else { v2.z },
        }
    }

    pub fn max(v1: Vector3, v2: Vector3) -> Vector3 {
        Vector3 {
            x: if v1.x > v2.x { v1.x } else { v2.x },
            y: if v1.y > v2.y { v1.y } else { v2.y },
            z: if v1.z > v2.z { v1.z } else { v2.z },
        }
    }

    pub fn dot(self, other: Vector3) -> f32 {
        (self.x * other.x) + (self.y * other.y) + (self.z * other.z)
    }

    pub fn squared_length(self) -> f32 {
        (self.x * self.x) + (self.y * self.y) + (self.z * self.z)
    }

    pub fn length(self) -> f32 {
        self.squared_length().sqrt()
    }

    pub fn normalized(self) -> Vector3 {
        self / self.length()
    }

    pub fn cross(self, other: Vector3) -> Vector3 {
        Vector3 {
            x: (self.y * other.z) - (self.z * other.y),
            y: (self.z * other.x) - (self.x * other.z),
            z: (self.x * other.y) - (self.y * other.x),
        }
    }
}

impl ops::Add for Vector3 {
    type Output = Vector3;
    fn add(self, rhs: Vector3) -> Vector3 {
        Vector3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl ops::Sub for Vector3 {
    type Output = Vector3;
    fn sub(self, rhs: Vector3) -> Vector3 {
        Vector3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl ops::Neg for Vector3 {
    type Output = Vector3;
    fn neg(self) -> Vector3 {
        Vector3 {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl ops::Mul for Vector3 {
    type Output = Vector3;
    fn mul(self, rhs: Vector3) -> Vector3 {
        Vector3 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl ops::Mul<f32> for Vector3 {
    type Output = Vector3;
    fn mul(self, rhs: f32) -> Vector3 {
        Vector3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl ops::Mul<Vector3> for f32 {
    type Output = Vector3;
    fn mul(self, rhs: Vector3) -> Vector3 {
        Vector3 {
            x: self * rhs.x,
            y: self * rhs.y,
            z: self * rhs.z,
        }
    }
}

impl ops::Div<f32> for Vector3 {
    type Output = Vector3;
    fn div(self, rhs: f32) -> Vector3 {
        Vector3 {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl ops::Div<Vector3> for f32 {
    type Output = Vector3;
    fn div(self, rhs: Vector3) -> Vector3 {
        Vector3 {
            x: self / rhs.x,
            y: self / rhs.y,
            z: self / rhs.z,
        }
    }
}

impl convert::TryFrom<Vec<f32>> for Vector3 {
    type Error = &'static str;

    fn try_from(vec: Vec<f32>) -> Result<Self, Self::Error> {
        if vec.len() != 3 {
            Err("Deserializing in to Vector3 requires a Vec of length 3!")
        } else {
            Ok(Vector3::new(vec[0], vec[1], vec[2]))
        }
    }
}

pub enum Axis {
    X,
    Y,
    Z,
}

impl Copy for Axis {}
impl Clone for Axis {
    fn clone(&self) -> Axis {
        *self
    }
}

impl ops::Index<Axis> for Vector3 {
    type Output = f32;
    fn index(&self, index: Axis) -> &f32 {
        match index {
            Axis::X => &self.x,
            Axis::Y => &self.y,
            Axis::Z => &self.z,
        }
    }
}
