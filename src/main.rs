// Local modules
mod aggregate;
mod camera;
mod material;
mod ray;
mod scene;
mod shape;
mod utils;
mod vector;

// External/std libraries for main
use image::png::PNGEncoder;
use image::ColorType;
use rand;
use std::{convert::TryInto, env, fs, fs::OpenOptions, path, process};

// Use statements for local modules
use crate::ray::Ray;
use crate::vector::Vector3;

// Constants
const COLOR_SPACE: f32 = 255.99_f32;

fn main() {
    // Read args: <in-scene-file> <out-filepath>
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Missing required arguments: <in-scene-file> <out-filepath>");
        process::exit(0);
    }

    // Read the scene spec file
    let scene_spec_path = path::Path::new(&args[1]);
    let scene_str = fs::read_to_string(&scene_spec_path).expect("Failed to read scene spec file.");
    let scene_spec = scene::deserialize(
        &scene_str,
        match scene_spec_path.parent() {
            Some(p) => p,
            None => path::Path::new("/"),
        },
    )
    .expect("Failed to parse scene spec JSON.");

    if &args[2] == "CONSTRUCTION_ONLY" {
        println!("Finished construction!");
        process::exit(0);
    }

    // Create the file according to input
    let out_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])
        .expect("Failed to create new file");
    let png_encoder = PNGEncoder::new(out_file);

    let res_x = scene_spec.logistics.resolution_x;
    let res_y = scene_spec.logistics.resolution_y;
    let samples = scene_spec.logistics.samples;
    let mut data = Vec::with_capacity((res_x * res_y * 3u32).try_into().unwrap());
    // Execute path trace for each pixel of our output
    for y in 0..res_y {
        for x in 0..res_x {
            let mut color = Vector3::new_empty();
            for _ in 0..samples {
                // Note the use of rand::random. Consider switching to an explicit
                // use of SmallRng, which is a non-secure, but fast, pseudo-RNG.
                // The default implementation may not be as performant, and this
                // program does not need the extra security benefits.
                let u = (x as f32 + rand::random::<f32>()) / res_x as f32;
                let v = ((res_y - y) as f32 + rand::random::<f32>()) / res_y as f32;

                let r = scene_spec.camera.get_ray(u, v);
                color =
                    color + aggregate::trace(&r, &(*scene_spec.shape_aggregate), &background, 0);
            }
            color = color / samples as f32;
            color = Vector3::new(color.x.sqrt(), color.y.sqrt(), color.z.sqrt());

            data.push((color.x * COLOR_SPACE) as u8);
            data.push((color.y * COLOR_SPACE) as u8);
            data.push((color.z * COLOR_SPACE) as u8);
        }
    }

    // Write the image to disk
    match png_encoder.encode(&data, res_x, res_y, ColorType::RGB(8)) {
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
