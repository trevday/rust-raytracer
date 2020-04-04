use crate::vector::Vector3;

use image::{DynamicImage, GenericImageView};
use serde::Deserialize;
use std::rc::Rc;

pub trait Texture {
    fn value(&self, u: f32, v: f32, p: &Vector3) -> Vector3;
}

#[derive(Deserialize)]
pub struct Constant {
    color: Vector3,
}
impl Texture for Constant {
    fn value(&self, _u: f32, _v: f32, _p: &Vector3) -> Vector3 {
        self.color
    }
}

pub struct Test;
impl Texture for Test {
    fn value(&self, u: f32, v: f32, _p: &Vector3) -> Vector3 {
        Vector3::new(
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
    fn value(&self, u: f32, v: f32, p: &Vector3) -> Vector3 {
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
    fn value(&self, u: f32, v: f32, _p: &Vector3) -> Vector3 {
        let i = (u * self.img.width() as f32) as u32;
        let j = ((1_f32 - v) * self.img.height() as f32) as u32;
        let pixel = self.img.get_pixel(i, j);
        Vector3::new(
            pixel[0] as f32 / 255_f32,
            pixel[1] as f32 / 255_f32,
            pixel[2] as f32 / 255_f32,
        )
    }
}
