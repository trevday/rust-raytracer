use crate::base::BasicTwoTuple;
use crate::color::RGB;
use crate::point::Point3;
use crate::utils::{clamp, noise, turbulence};

use image::{DynamicImage, GenericImageView};
use serde::Deserialize;
use std::{convert::TryFrom, ops, sync::Arc};

#[derive(Deserialize)]
pub struct TexCoord(pub BasicTwoTuple<f32>);

impl Copy for TexCoord {}
impl Clone for TexCoord {
    fn clone(&self) -> TexCoord {
        *self
    }
}

impl TexCoord {
    pub fn new(x: f32, y: f32) -> TexCoord {
        TexCoord(BasicTwoTuple::new(x, y))
    }

    pub fn u(&self) -> f32 {
        self.0.x
    }
    pub fn v(&self) -> f32 {
        self.0.y
    }

    pub fn clamp_to_valid_coords(&self) -> TexCoord {
        TexCoord::new(
            clamp(self.u(), 0.0_f32, 1.0_f32),
            clamp(self.v(), 0.0_f32, 1.0_f32),
        )
    }
}

impl ops::Add for TexCoord {
    type Output = TexCoord;
    fn add(self, rhs: TexCoord) -> TexCoord {
        TexCoord(self.0.add(rhs.0))
    }
}

impl ops::Sub for TexCoord {
    type Output = TexCoord;
    fn sub(self, rhs: TexCoord) -> TexCoord {
        TexCoord(self.0.sub(rhs.0))
    }
}

impl ops::Mul<f32> for TexCoord {
    type Output = TexCoord;
    fn mul(self, rhs: f32) -> TexCoord {
        TexCoord(self.0.mul(rhs))
    }
}

pub trait Texture {
    fn value(&self, uv: &TexCoord, p: &Point3) -> RGB;
    fn bump_value(&self, uv: &TexCoord, p: &Point3) -> f32 {
        let bump = self.value(uv, p);
        (bump.r() + bump.g() + bump.b()) / 3.0_f32
    }
}
pub type SyncTexture = dyn Texture + Send + Sync;

#[derive(Deserialize)]
pub struct Constant {
    color: RGB,
}
impl Texture for Constant {
    fn value(&self, _uv: &TexCoord, _p: &Point3) -> RGB {
        self.color
    }
}

pub struct Test;
impl Texture for Test {
    fn value(&self, uv: &TexCoord, _p: &Point3) -> RGB {
        RGB::new(
            uv.u(),
            uv.v(),
            if 1.0_f32 - uv.u() - uv.v() < 0.0_f32 {
                0.0_f32
            } else {
                1.0_f32 - uv.u() - uv.v()
            },
        )
    }
}

pub struct Checker {
    repeat: f32,
    odd: Arc<SyncTexture>,
    even: Arc<SyncTexture>,
}
impl Checker {
    pub fn new(repeat: f32, odd: Arc<SyncTexture>, even: Arc<SyncTexture>) -> Checker {
        Checker {
            repeat: repeat,
            odd: odd,
            even: even,
        }
    }
}
impl Texture for Checker {
    fn value(&self, uv: &TexCoord, p: &Point3) -> RGB {
        let sines =
            (self.repeat * p.x()).sin() * (self.repeat * p.y()).sin() * (self.repeat * p.z()).sin();
        if sines < 0.0_f32 {
            self.odd.value(uv, p)
        } else {
            self.even.value(uv, p)
        }
    }
}

pub struct Image {
    img: Arc<DynamicImage>,
}
impl Image {
    pub fn new(img: Arc<DynamicImage>) -> Image {
        Image { img: img }
    }
}
impl Texture for Image {
    fn value(&self, uv: &TexCoord, _p: &Point3) -> RGB {
        let i = (uv.u() * self.img.width() as f32) as u32 % self.img.width();
        let j = ((1_f32 - uv.v()) * self.img.height() as f32) as u32 % self.img.height();
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
    fn value(&self, _uv: &TexCoord, p: &Point3) -> RGB {
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
    fn value(&self, _uv: &TexCoord, p: &Point3) -> RGB {
        return RGB::new(1.0_f32, 1.0_f32, 1.0_f32)
            * turbulence(&(*p * self.scale), self.depth, self.omega.0);
    }
}
