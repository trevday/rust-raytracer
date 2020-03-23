use crate::vector::Vector3;

pub struct Ray {
    pub origin: Vector3,
    pub dir: Vector3,
}

impl Ray {
    pub fn new(origin: Vector3, dir: Vector3) -> Ray {
        Ray {
            origin: origin,
            dir: dir,
        }
    }

    pub fn point_at(&self, t: f32) -> Vector3 {
        self.origin + (self.dir * t)
    }
}
