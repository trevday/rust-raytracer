use crate::aggregate::AABB;
use crate::point::Point3;
use crate::ray::Ray;
use crate::vector::Vector3;

use std::ops;

pub struct Matrix4 {
    // Row first ordering
    data: [[f32; 4]; 4],
}

impl Clone for Matrix4 {
    fn clone(&self) -> Matrix4 {
        Matrix4 { data: self.data }
    }
}

impl Matrix4 {
    pub fn new() -> Matrix4 {
        Matrix4 {
            data: [[0_f32; 4]; 4],
        }
    }

    pub fn new_identity() -> Matrix4 {
        let mut data = [[0_f32; 4]; 4];
        data[0][0] = 1.0_f32;
        data[1][1] = 1.0_f32;
        data[2][2] = 1.0_f32;
        data[3][3] = 1.0_f32;
        Matrix4 { data: data }
    }

    pub fn new_translation(translate: &Vector3) -> Matrix4 {
        let mut m = Matrix4::new_identity();
        m.data[0][3] = translate.x;
        m.data[1][3] = translate.y;
        m.data[2][3] = translate.z;
        m
    }

    pub fn new_rotation(rotate: &Vector3) -> Matrix4 {
        let mut m = Matrix4::new_identity();
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

        m
    }

    pub fn new_scale(scale: &Vector3) -> Matrix4 {
        let mut m = Matrix4::new_identity();
        m.data[0][0] = scale.x;
        m.data[1][1] = scale.y;
        m.data[2][2] = scale.z;
        m
    }

    // Gauss-Jordan Elimination
    // from https://www.scratchapixel.com/lessons/mathematics-physics-for-computer-graphics/matrix-inverse
    pub fn inverse(&self) -> Result<Matrix4, &'static str> {
        let mut temp = self.clone();
        let mut res = Matrix4::new_identity();
        for col in 0..4 {
            if temp.data[col][col] == 0.0_f32 {
                let mut big = col;
                for row in 0..4 {
                    if temp.data[row][col].abs() > temp.data[big][col].abs() {
                        big = row;
                    }
                }
                if big == col {
                    return Err("Singular matrix");
                } else {
                    for j in 0..4 {
                        // mem::swap does not work here because we cannot have
                        // two mutable references to the array at once
                        let t = temp.data[col][j];
                        temp.data[col][j] = temp.data[big][j];
                        temp.data[big][j] = t;

                        let t = res.data[col][j];
                        res.data[col][j] = res.data[big][j];
                        res.data[big][j] = t;
                    }
                }
            }
            for row in 0..4 {
                if row != col {
                    let coeff = temp.data[row][col] / temp.data[col][col];
                    if coeff != 0.0_f32 {
                        for j in 0..4 {
                            temp.data[row][j] -= coeff * temp.data[col][j];
                            res.data[row][j] -= coeff * res.data[col][j];
                        }
                        temp.data[row][col] = 0.0_f32;
                    }
                }
            }
        }
        for row in 0..4 {
            for col in 0..4 {
                res.data[row][col] /= temp.data[row][row];
            }
        }
        return Ok(res);
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
        Vector3::new(
            self.data[0][0] * rhs.x + self.data[0][1] * rhs.y + self.data[0][2] * rhs.z,
            self.data[1][0] * rhs.x + self.data[1][1] * rhs.y + self.data[1][2] * rhs.z,
            self.data[2][0] * rhs.x + self.data[2][1] * rhs.y + self.data[2][2] * rhs.z,
        )
    }
}

impl ops::Mul<Point3> for &Matrix4 {
    type Output = Point3;
    fn mul(self, rhs: Point3) -> Point3 {
        Point3::new(
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

impl ops::Mul<&Ray> for &Matrix4 {
    type Output = Ray;
    fn mul(self, rhs: &Ray) -> Ray {
        Ray::new(self * rhs.origin, self * rhs.dir)
    }
}

impl ops::Mul<&AABB> for &Matrix4 {
    type Output = AABB;
    fn mul(self, rhs: &AABB) -> AABB {
        AABB::new(self * rhs.min, self * rhs.max)
    }
}
