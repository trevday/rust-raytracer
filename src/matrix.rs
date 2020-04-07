use crate::vector::Vector3;

use std::ops;

pub struct Matrix4 {
    // Row first ordering
    data: [[f32; 4]; 4],
}

impl Matrix4 {
    pub fn new() -> Matrix4 {
        Matrix4 {
            data: [[0_f32; 4]; 4],
        }
    }

    pub fn new_translation(translate: &Vector3) -> Matrix4 {
        let mut m = Matrix4::new();
        m.data[0][3] = translate.x;
        m.data[1][3] = translate.y;
        m.data[2][3] = translate.z;

        m.data[0][0] = 1_f32;
        m.data[1][1] = 1_f32;
        m.data[2][2] = 1_f32;
        m.data[3][3] = 1_f32;
        m
    }

    pub fn new_rotation(rotate: &Vector3) -> Matrix4 {
        let mut m = Matrix4::new();
        // First row
        m.data[0][0] = rotate.z.cos() * rotate.y.cos();
        m.data[0][1] =
            rotate.z.cos() * rotate.y.sin() * rotate.x.sin() - rotate.z.sin() * rotate.x.cos();
        m.data[0][2] =
            rotate.z.cos() * rotate.y.sin() * rotate.x.cos() + rotate.z.sin() * rotate.x.sin();

        // Second row
        m.data[1][0] = rotate.z.sin() * rotate.y.cos();
        m.data[1][1] =
            rotate.z.sin() * rotate.y.sin() * rotate.x.sin() + rotate.z.cos() * rotate.x.cos();
        m.data[1][2] =
            rotate.z.sin() * rotate.y.sin() * rotate.x.cos() - rotate.z.cos() * rotate.x.sin();

        // Third row
        m.data[2][0] = -rotate.y.sin();
        m.data[2][1] = rotate.y.cos() * rotate.x.sin();
        m.data[2][2] = rotate.y.cos() * rotate.x.cos();

        m.data[3][3] = 1_f32;
        m
    }

    pub fn new_scale(scale: &Vector3) -> Matrix4 {
        let mut m = Matrix4::new();
        m.data[0][0] = scale.x;
        m.data[1][1] = scale.y;
        m.data[2][2] = scale.z;
        m.data[3][3] = 1_f32;
        m
    }
}

// TODO (performance): Use SIMD?
impl ops::Mul for Matrix4 {
    type Output = Matrix4;
    fn mul(self, rhs: Matrix4) -> Matrix4 {
        let mut m = Matrix4::new();
        for row in 0..4 {
            for col in 0..4 {
                m.data[row][col] = self.data[row][0] * rhs.data[0][col]
                    + self.data[row][1] * rhs.data[1][col]
                    + self.data[row][2] * rhs.data[2][col]
                    + self.data[row][3] * rhs.data[3][col];
            }
        }
        m
    }
}

impl ops::Mul<Vector3> for &Matrix4 {
    type Output = Vector3;
    fn mul(self, rhs: Vector3) -> Vector3 {
        // TODO: I am very aware I am not doing homogeneous coordinates correctly right now,
        // this is a stopgap measure. I define a lot of what are really points as Vector3's,
        // so I am defaulting to Point behavior since I need to use the transformations for
        // them. Next commit will refactor out a point class and clean this up so it is right.
        Vector3::new(
            self.data[0][0] * rhs.x
                + self.data[0][1] * rhs.y
                + self.data[0][2] * rhs.z
                + self.data[0][3] * 1_f32,
            self.data[1][0] * rhs.x
                + self.data[1][1] * rhs.y
                + self.data[1][2] * rhs.z
                + self.data[1][3] * 1_f32,
            self.data[2][0] * rhs.x
                + self.data[2][1] * rhs.y
                + self.data[2][2] * rhs.z
                + self.data[2][3] * 1_f32,
        )
    }
}
