// Local modules
mod camera;
mod material;
mod ray;
mod shape;
mod utils;
mod vector;

// External/std libraries for main
use image::png::PNGEncoder;
use image::ColorType;
use rand;
use std::{convert::TryInto, env, fs::File, process};

// Use statements for local modules
use crate::camera::Camera;
use crate::ray::Ray;
use crate::vector::Vector3;

// Inputs
const SIZE_X: u32 = 800;
const SIZE_Y: u32 = 600;
const NUM_SAMPLES: u32 = 100;
// Constants
const COLOR_SPACE: f32 = 255.99_f32;

fn main() {
    // Read args: <filepath>
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing required argument: output filepath");
        process::exit(0);
    }

    // Create the file according to input
    let file = match File::create(&args[1]) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create or append to file: {}", e);
            process::exit(1);
        }
    };
    let png_encoder = PNGEncoder::new(file);

    // Set up parameters
    let cam = Camera::new(
        &Vector3::new(0_f32, 0_f32, 1_f32),
        &Vector3::new_empty(),
        &Vector3::new(0_f32, 1_f32, 0_f32),
        90_f32,
        SIZE_X as f32 / SIZE_Y as f32,
        0.3_f32,
        2.0_f32,
    );

    // TODO: take in world data in a defined format, so that scenes
    // are easily user customizable
    let s1 = shape::Sphere::new(
        Vector3::new(0_f32, 0_f32, -1_f32),
        0.5_f32,
        Box::new(material::Lambert::new(Vector3::new(
            0.1_f32, 0.2_f32, 0.5_f32,
        ))),
    );
    let s2 = shape::Sphere::new(
        Vector3::new(0_f32, -100.5_f32, -1_f32),
        100_f32,
        Box::new(material::Lambert::new(Vector3::new(
            0.8_f32, 0.8_f32, 0_f32,
        ))),
    );
    let s3 = shape::Sphere::new(
        Vector3::new(1_f32, 0_f32, -1_f32),
        0.5_f32,
        Box::new(material::Metal::new(
            Vector3::new(0.8_f32, 0.6_f32, 0.2_f32),
            0.0_f32,
        )),
    );
    let s4 = shape::Sphere::new(
        Vector3::new(-1_f32, 0_f32, -1_f32),
        0.5_f32,
        Box::new(material::Dielectric::new(1.5_f32)),
    );
    let s5 = shape::Sphere::new(
        Vector3::new(-1_f32, 0_f32, -1_f32),
        -0.45_f32,
        Box::new(material::Dielectric::new(1.5_f32)),
    );
    let world_shapes: Vec<&dyn shape::Shape> = vec![&s1, &s2, &s3, &s4, &s5];

    let mut data = Vec::with_capacity((SIZE_X * SIZE_Y * 3u32).try_into().unwrap());
    // Execute path trace for each pixel of our output
    for y in 0..SIZE_Y {
        for x in 0..SIZE_X {
            let mut color = Vector3::new_empty();
            for _ in 0..NUM_SAMPLES {
                // Note the use of rand::random. Consider switching to an explicit
                // use of SmallRng, which is a non-secure, but fast, pseudo-RNG.
                // The default implementation may not be as performant, and this
                // program does not need the extra security benefits.
                let u = (x as f32 + rand::random::<f32>()) / SIZE_X as f32;
                let v = ((SIZE_Y - y) as f32 + rand::random::<f32>()) / SIZE_Y as f32;

                let r = cam.get_ray(u, v);
                color = color + shape::trace(&r, &world_shapes, &background, 0);
            }
            color = color / NUM_SAMPLES as f32;
            color = Vector3::new(color.x.sqrt(), color.y.sqrt(), color.z.sqrt());

            data.push((color.x * COLOR_SPACE) as u8);
            data.push((color.y * COLOR_SPACE) as u8);
            data.push((color.z * COLOR_SPACE) as u8);
        }
    }

    // Write the image to disk
    match png_encoder.encode(&data, SIZE_X, SIZE_Y, ColorType::RGB(8)) {
        Ok(()) => println!("Success!"),
        Err(e) => {
            eprintln!("Failed to encode the png for output: {}", e);
            process::exit(1);
        }
    }
}

fn background(r: &Ray) -> Vector3 {
    // Sky blend
    let dir_normal = r.dir.normalized();
    let t = 0.5_f32 * (dir_normal.y + 1.0_f32);

    Vector3::new(1.0_f32, 1.0_f32, 1.0_f32) * (1.0_f32 - t)
        + Vector3::new(0.5_f32, 0.7_f32, 1.0_f32) * t
}
