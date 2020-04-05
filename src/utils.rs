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

pub fn clamp(v: f32, min: f32, max: f32) -> f32 {
    if v > max {
        max
    } else if v < min {
        min
    } else {
        v
    }
}

pub fn lerp(t: f32, a: f32, b: f32) -> f32 {
    return (1_f32 - t) * a + t * b;
}

// Data for noise, duplicated twice for efficient lookup
const NOISE_SIZE: usize = 256;
const NOISE_DATA: [usize; NOISE_SIZE * 2] = [
    63, 147, 186, 78, 92, 53, 229, 76, 14, 204, 183, 99, 237, 241, 59, 167, 118, 23, 29, 44, 82,
    37, 6, 249, 131, 253, 210, 28, 71, 96, 3, 207, 115, 32, 158, 61, 215, 220, 116, 40, 48, 93,
    179, 196, 141, 0, 165, 185, 145, 217, 139, 216, 250, 235, 39, 232, 124, 146, 5, 77, 180, 4, 31,
    203, 154, 178, 226, 25, 20, 130, 22, 240, 252, 163, 75, 90, 51, 89, 151, 193, 33, 69, 21, 149,
    208, 244, 238, 191, 161, 36, 38, 81, 181, 56, 43, 127, 34, 243, 65, 200, 97, 247, 79, 231, 98,
    11, 100, 142, 15, 166, 45, 209, 223, 66, 119, 155, 49, 153, 113, 41, 133, 197, 157, 112, 46,
    91, 74, 27, 128, 228, 16, 248, 174, 187, 87, 95, 30, 110, 212, 175, 144, 135, 225, 172, 221,
    170, 67, 9, 111, 224, 239, 176, 117, 109, 177, 202, 132, 80, 125, 62, 251, 108, 148, 103, 227,
    50, 17, 35, 24, 126, 164, 42, 156, 10, 182, 218, 70, 246, 150, 73, 213, 138, 129, 189, 188, 84,
    160, 134, 105, 83, 169, 121, 233, 194, 19, 114, 55, 211, 58, 104, 254, 57, 18, 123, 102, 140,
    8, 171, 68, 206, 168, 86, 136, 152, 47, 60, 88, 101, 26, 122, 13, 192, 94, 198, 64, 234, 195,
    52, 245, 54, 236, 219, 12, 106, 143, 120, 7, 190, 1, 2, 205, 222, 159, 162, 173, 85, 107, 201,
    184, 214, 137, 230, 255, 242, 72, 199, // Second set of duplicate data starts here
    63, 147, 186, 78, 92, 53, 229, 76, 14, 204, 183, 99, 237, 241, 59, 167, 118, 23, 29, 44, 82,
    37, 6, 249, 131, 253, 210, 28, 71, 96, 3, 207, 115, 32, 158, 61, 215, 220, 116, 40, 48, 93,
    179, 196, 141, 0, 165, 185, 145, 217, 139, 216, 250, 235, 39, 232, 124, 146, 5, 77, 180, 4, 31,
    203, 154, 178, 226, 25, 20, 130, 22, 240, 252, 163, 75, 90, 51, 89, 151, 193, 33, 69, 21, 149,
    208, 244, 238, 191, 161, 36, 38, 81, 181, 56, 43, 127, 34, 243, 65, 200, 97, 247, 79, 231, 98,
    11, 100, 142, 15, 166, 45, 209, 223, 66, 119, 155, 49, 153, 113, 41, 133, 197, 157, 112, 46,
    91, 74, 27, 128, 228, 16, 248, 174, 187, 87, 95, 30, 110, 212, 175, 144, 135, 225, 172, 221,
    170, 67, 9, 111, 224, 239, 176, 117, 109, 177, 202, 132, 80, 125, 62, 251, 108, 148, 103, 227,
    50, 17, 35, 24, 126, 164, 42, 156, 10, 182, 218, 70, 246, 150, 73, 213, 138, 129, 189, 188, 84,
    160, 134, 105, 83, 169, 121, 233, 194, 19, 114, 55, 211, 58, 104, 254, 57, 18, 123, 102, 140,
    8, 171, 68, 206, 168, 86, 136, 152, 47, 60, 88, 101, 26, 122, 13, 192, 94, 198, 64, 234, 195,
    52, 245, 54, 236, 219, 12, 106, 143, 120, 7, 190, 1, 2, 205, 222, 159, 162, 173, 85, 107, 201,
    184, 214, 137, 230, 255, 242, 72, 199,
];
// Perlin noise
pub fn noise(p: &Vector3) -> f32 {
    let mut ix = p.x.floor() as i32;
    let mut iy = p.y.floor() as i32;
    let mut iz = p.z.floor() as i32;

    let dx = p.x - ix as f32;
    let dy = p.y - iy as f32;
    let dz = p.z - iz as f32;

    // Reduce to the size of our noise data
    ix &= NOISE_SIZE as i32 - 1;
    iy &= NOISE_SIZE as i32 - 1;
    iz &= NOISE_SIZE as i32 - 1;

    // Compute gradients
    let w000 = gradient(ix, iy, iz, dx, dy, dz);
    let w100 = gradient(ix + 1, iy, iz, dx - 1_f32, dy, dz);
    let w010 = gradient(ix, iy + 1, iz, dx, dy - 1_f32, dz);
    let w001 = gradient(ix, iy, iz + 1, dx, dy, dz - 1_f32);
    let w110 = gradient(ix + 1, iy + 1, iz, dx - 1_f32, dy - 1_f32, dz);
    let w101 = gradient(ix + 1, iy, iz + 1, dx - 1_f32, dy, dz - 1_f32);
    let w011 = gradient(ix, iy + 1, iz + 1, dx, dy - 1_f32, dz - 1_f32);
    let w111 = gradient(ix + 1, iy + 1, iz + 1, dx - 1_f32, dy - 1_f32, dz - 1_f32);

    let wx = smooth(dx);
    let wy = smooth(dy);
    let wz = smooth(dz);

    // Linear interpolation
    let x00 = lerp(wx, w000, w100);
    let x10 = lerp(wx, w010, w110);
    let x01 = lerp(wx, w001, w101);
    let x11 = lerp(wx, w011, w111);
    let y0 = lerp(wy, x00, x10);
    let y1 = lerp(wy, x01, x11);
    return lerp(wz, y0, y1);
}
fn gradient(x: i32, y: i32, z: i32, dx: f32, dy: f32, dz: f32) -> f32 {
    let mut val = NOISE_DATA[NOISE_DATA[NOISE_DATA[x as usize] + y as usize] + z as usize];
    // Only the lower 4 bits of the value are considered
    val &= 15;
    let mut u = if val < 8 || val == 12 || val == 13 {
        dx
    } else {
        dy
    };
    let mut v = if val < 4 || val == 12 || val == 13 {
        dy
    } else {
        dz
    };
    if val & 1 > 0 {
        u = -u;
    }
    if val & 2 > 0 {
        v = -v;
    }
    return u + v;
}
fn smooth(f: f32) -> f32 {
    let f_3 = f * f * f;
    let f_4 = f_3 * f;
    return 6_f32 * f_4 * f - 15_f32 * f_4 + 10_f32 * f_3;
}

pub fn turbulence(p: &Vector3, depth: u32, omega: f32) -> f32 {
    let mut sum = 0.0_f32;
    let mut p_copy = *p;
    let mut weight = 1.0_f32;

    for _ in 0..depth {
        sum += weight * noise(&p_copy);
        weight *= omega;
        p_copy = p_copy * 1.99_f32;
    }

    return sum.abs();
}
