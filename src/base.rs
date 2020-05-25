use serde::Deserialize;
use std::cmp;
use std::ops;

#[derive(Deserialize)]
pub struct BasicThreeTuple<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Copy> Copy for BasicThreeTuple<T> {}
impl<T: Copy> Clone for BasicThreeTuple<T> {
    fn clone(&self) -> BasicThreeTuple<T> {
        *self
    }
}

// Short functions in this file should always be inlined by the compiler.
// https://doc.rust-lang.org/1.25.0/reference/attributes.html#inline-attribute
impl<T> BasicThreeTuple<T>
where
    T: cmp::PartialOrd,
{
    pub fn new(x: T, y: T, z: T) -> BasicThreeTuple<T> {
        BasicThreeTuple { x: x, y: y, z: z }
    }

    pub fn min(v1: BasicThreeTuple<T>, v2: BasicThreeTuple<T>) -> BasicThreeTuple<T> {
        BasicThreeTuple {
            x: if v1.x < v2.x { v1.x } else { v2.x },
            y: if v1.y < v2.y { v1.y } else { v2.y },
            z: if v1.z < v2.z { v1.z } else { v2.z },
        }
    }

    pub fn max(v1: BasicThreeTuple<T>, v2: BasicThreeTuple<T>) -> BasicThreeTuple<T> {
        BasicThreeTuple {
            x: if v1.x > v2.x { v1.x } else { v2.x },
            y: if v1.y > v2.y { v1.y } else { v2.y },
            z: if v1.z > v2.z { v1.z } else { v2.z },
        }
    }
}

impl<T> ops::Add for BasicThreeTuple<T>
where
    T: ops::Add<Output = T>,
{
    type Output = BasicThreeTuple<T>;
    fn add(self, rhs: BasicThreeTuple<T>) -> BasicThreeTuple<T> {
        BasicThreeTuple {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T> ops::Sub for BasicThreeTuple<T>
where
    T: ops::Sub<Output = T>,
{
    type Output = BasicThreeTuple<T>;
    fn sub(self, rhs: BasicThreeTuple<T>) -> BasicThreeTuple<T> {
        BasicThreeTuple {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T> ops::Neg for BasicThreeTuple<T>
where
    T: ops::Neg<Output = T>,
{
    type Output = BasicThreeTuple<T>;
    fn neg(self) -> BasicThreeTuple<T> {
        BasicThreeTuple {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<T> ops::Mul for BasicThreeTuple<T>
where
    T: ops::Mul<Output = T>,
{
    type Output = BasicThreeTuple<T>;
    fn mul(self, rhs: BasicThreeTuple<T>) -> BasicThreeTuple<T> {
        BasicThreeTuple {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
        }
    }
}

impl<T> ops::Mul<T> for BasicThreeTuple<T>
where
    T: Copy + ops::Mul<Output = T>,
{
    type Output = BasicThreeTuple<T>;
    fn mul(self, rhs: T) -> BasicThreeTuple<T> {
        BasicThreeTuple {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl<T> ops::Div<T> for BasicThreeTuple<T>
where
    T: Copy + ops::Div<Output = T>,
{
    type Output = BasicThreeTuple<T>;
    fn div(self, rhs: T) -> BasicThreeTuple<T> {
        BasicThreeTuple {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

#[derive(Deserialize)]
pub struct BasicTwoTuple<T> {
    pub x: T,
    pub y: T,
}

impl<T: Copy> Copy for BasicTwoTuple<T> {}
impl<T: Copy> Clone for BasicTwoTuple<T> {
    fn clone(&self) -> BasicTwoTuple<T> {
        *self
    }
}

impl<T> BasicTwoTuple<T>
where
    T: cmp::PartialOrd,
{
    pub fn new(x: T, y: T) -> BasicTwoTuple<T> {
        BasicTwoTuple { x: x, y: y }
    }
}

impl<T> ops::Add for BasicTwoTuple<T>
where
    T: ops::Add<Output = T>,
{
    type Output = BasicTwoTuple<T>;
    fn add(self, rhs: BasicTwoTuple<T>) -> BasicTwoTuple<T> {
        BasicTwoTuple {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<T> ops::Sub for BasicTwoTuple<T>
where
    T: ops::Sub<Output = T>,
{
    type Output = BasicTwoTuple<T>;
    fn sub(self, rhs: BasicTwoTuple<T>) -> BasicTwoTuple<T> {
        BasicTwoTuple {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T> ops::Neg for BasicTwoTuple<T>
where
    T: ops::Neg<Output = T>,
{
    type Output = BasicTwoTuple<T>;
    fn neg(self) -> BasicTwoTuple<T> {
        BasicTwoTuple {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> ops::Mul for BasicTwoTuple<T>
where
    T: ops::Mul<Output = T>,
{
    type Output = BasicTwoTuple<T>;
    fn mul(self, rhs: BasicTwoTuple<T>) -> BasicTwoTuple<T> {
        BasicTwoTuple {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

impl<T> ops::Mul<T> for BasicTwoTuple<T>
where
    T: Copy + ops::Mul<Output = T>,
{
    type Output = BasicTwoTuple<T>;
    fn mul(self, rhs: T) -> BasicTwoTuple<T> {
        BasicTwoTuple {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl<T> ops::Div<T> for BasicTwoTuple<T>
where
    T: Copy + ops::Div<Output = T>,
{
    type Output = BasicTwoTuple<T>;
    fn div(self, rhs: T) -> BasicTwoTuple<T> {
        BasicTwoTuple {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}
