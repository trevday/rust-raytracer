use crate::color::RGB;
use crate::point::Point3;
use crate::utils::{noise, turbulence};

use image::{DynamicImage, GenericImageView};
use serde::Deserialize;
use std::{convert::TryFrom, rc::Rc};

pub trait Texture {
    fn value(&self, u: f32, v: f32, p: &Point3) -> RGB;
    fn bump_value(&self, u: f32, v: f32, p: &Point3) -> f32 {
        let bump = self.value(u, v, p);
        (bump.r + bump.g + bump.b) / 3.0_f32
    }
}

#[derive(Deserialize)]
pub struct Constant {
    color: RGB,
}
impl Texture for Constant {
    fn value(&self, _u: f32, _v: f32, _p: &Point3) -> RGB {
        self.color
    }
}

pub struct Test;
impl Texture for Test {
    fn value(&self, u: f32, v: f32, _p: &Point3) -> RGB {
        RGB::new(
            u,
            v,
            if 1.0_f32 - u - v < 0.0_f32 {
                0.0_f32
            } else {
                1.0_f32 - u - v
            },
        )
    }
}

pub struct Checker {
    repeat: f32,
    odd: Rc<dyn Texture>,
    even: Rc<dyn Texture>,
}
impl Checker {
    pub fn new(repeat: f32, odd: Rc<dyn Texture>, even: Rc<dyn Texture>) -> Checker {
        Checker {
            repeat: repeat,
            odd: odd,
            even: even,
        }
    }
}
impl Texture for Checker {
    fn value(&self, u: f32, v: f32, p: &Point3) -> RGB {
        let sines =
            (self.repeat * p.x).sin() * (self.repeat * p.y).sin() * (self.repeat * p.z).sin();
        if sines < 0.0_f32 {
            self.odd.value(u, v, p)
        } else {
            self.even.value(u, v, p)
        }
    }
}

pub struct Image {
    img: Rc<DynamicImage>,
}
impl Image {
    pub fn new(img: Rc<DynamicImage>) -> Image {
        Image { img: img }
    }
}
impl Texture for Image {
    fn value(&self, u: f32, v: f32, _p: &Point3) -> RGB {
        let i = (u * self.img.width() as f32) as u32 % self.img.width();
        let j = ((1_f32 - v) * self.img.height() as f32) as u32 % self.img.height();
        let pixel = self.img.get_pixel(i, j);
        // TODO: Probably need to undo gamma correction here after reading the image
        RGB::new(
            pixel[0] as f32 / 255_f32,
            pixel[1] as f32 / 255_f32,
            pixel[2] as f32 / 255_f32,
        )
    }
}

#[derive(Deserialize)]
pub struct Noise {
    scale: f32,
}
impl Texture for Noise {
    fn value(&self, _u: f32, _v: f32, p: &Point3) -> RGB {
        return RGB::new(0.5_f32, 0.5_f32, 0.5_f32) * (1.0_f32 + noise(&(*p * self.scale)));
    }
}

#[derive(Deserialize)]
pub struct Turbulence {
    scale: f32,
    depth: u32,
    omega: Omega,
}
#[derive(Deserialize)]
#[serde(try_from = "f32")]
struct Omega(f32);
impl TryFrom<f32> for Omega {
    type Error = &'static str;
    fn try_from(v: f32) -> Result<Self, Self::Error> {
        if v > 1.0_f32 {
            Err("Turbulence omega is greater than 1.")
        } else if v < 0.0_f32 {
            Err("Turbulence omega is less than 0.")
        } else {
            Ok(Omega(v))
        }
    }
}
impl Texture for Turbulence {
    fn value(&self, _u: f32, _v: f32, p: &Point3) -> RGB {
        return RGB::new(1.0_f32, 1.0_f32, 1.0_f32)
            * turbulence(&(*p * self.scale), self.depth, self.omega.0);
    }
}
