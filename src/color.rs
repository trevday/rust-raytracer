use serde::Deserialize;
use std::ops;

#[derive(Deserialize)]
pub struct RGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Copy for RGB {}
impl Clone for RGB {
    fn clone(&self) -> RGB {
        *self
    }
}

impl RGB {
    pub fn new(r: f32, g: f32, b: f32) -> RGB {
        RGB { r: r, g: g, b: b }
    }

    pub fn black() -> RGB {
        RGB {
            r: 0.0_f32,
            g: 0.0_f32,
            b: 0.0_f32,
        }
    }
}

// TODO: See if there is a way to reduce duplication of common functions for these
// "three float" structures, like Point, Vector, and RGB, but while maintaining
// the strong types.
impl ops::Mul for RGB {
    type Output = RGB;
    fn mul(self, rhs: RGB) -> RGB {
        RGB {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl ops::Add for RGB {
    type Output = RGB;
    fn add(self, rhs: RGB) -> RGB {
        RGB {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl ops::Mul<f32> for RGB {
    type Output = RGB;
    fn mul(self, rhs: f32) -> RGB {
        RGB {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl ops::Div<f32> for RGB {
    type Output = RGB;
    fn div(self, rhs: f32) -> RGB {
        RGB {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
        }
    }
}
