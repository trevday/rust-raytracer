use crate::base::BasicThreeTuple;

use serde::Deserialize;
use std::ops;

#[derive(Deserialize)]
pub struct RGB(pub BasicThreeTuple<f32>);

impl Copy for RGB {}
impl Clone for RGB {
    fn clone(&self) -> RGB {
        *self
    }
}

impl RGB {
    pub fn new(r: f32, g: f32, b: f32) -> RGB {
        RGB(BasicThreeTuple::new(r, g, b))
    }

    pub fn black() -> RGB {
        RGB(BasicThreeTuple::new(0_f32, 0_f32, 0_f32))
    }

    pub fn r(&self) -> f32 {
        self.0.x
    }
    pub fn g(&self) -> f32 {
        self.0.y
    }
    pub fn b(&self) -> f32 {
        self.0.z
    }
}

impl ops::Mul for RGB {
    type Output = RGB;
    fn mul(self, rhs: RGB) -> RGB {
        RGB(self.0.mul(rhs.0))
    }
}

impl ops::Add for RGB {
    type Output = RGB;
    fn add(self, rhs: RGB) -> RGB {
        RGB(self.0.add(rhs.0))
    }
}

impl ops::Mul<f32> for RGB {
    type Output = RGB;
    fn mul(self, rhs: f32) -> RGB {
        RGB(self.0.mul(rhs))
    }
}

impl ops::Div<f32> for RGB {
    type Output = RGB;
    fn div(self, rhs: f32) -> RGB {
        RGB(self.0.div(rhs))
    }
}
