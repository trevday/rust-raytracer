use crate::vector::Vector3;
use rand;

pub fn random_unit_disk() -> Vector3 {
    let x = 2.0_f32 * rand::random::<f32>() - 1.0_f32;
    let y = (1.0_f32 - x * x).sqrt();
    Vector3::new(x, y, 0.0_f32)
}

pub fn unit_sphere_random() -> Vector3 {
    let azimuth = rand::random::<f32>() * std::f32::consts::PI * 2.0_f32;
    let y = rand::random::<f32>();
    let sin_elevation = (1.0_f32 - y * y).sqrt();
    let x = sin_elevation * azimuth.cos();
    let z = sin_elevation * azimuth.sin();

    Vector3::new(x, y, z)
}
