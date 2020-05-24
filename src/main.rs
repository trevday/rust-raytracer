// Local modules
mod aggregate;
mod camera;
mod color;
mod material;
mod matrix;
mod pdf;
mod point;
mod progress;
mod ray;
mod resources;
mod scene;
mod shape;
mod texture;
mod transform;
mod utils;
mod vector;
mod volume;

// External/std libraries for main
use clap::{App, Arg};
use image::png::PNGEncoder;
use image::ColorType;
use rand;
use std::{
    fs, fs::OpenOptions, io, path, process, sync::mpsc, sync::Arc, sync::Mutex, thread,
    time::Instant,
};

// Use statements for local modules
use crate::color::RGB;
use crate::progress::Progress;
use crate::ray::Ray;
use crate::resources::Resources;
use crate::scene::Scene;

// Constants
const COLOR_SPACE: f32 = 255.99_f32;

fn main() {
    // Define command line args
    let matches = App::new("Raytracer")
        .arg(
            Arg::with_name("thread-count")
                .short("t")
                .long("thread-count")
                .value_name("THREAD_COUNT")
                .help("Number of threads to use while tracing")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("IN_SCENE_FILE")
                .help("The scene specification to render")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("OUT_FILEPATH")
                .help("The relative filepath to write the output image to")
                .required(true)
                .index(2),
        )
        .get_matches();

    // Grab a stamp for the start of the run
    let program_start = Instant::now();

    // Grab the number of threads we want to use from arguments,
    // default to 2
    let num_threads = matches
        .value_of("thread-count")
        .unwrap_or("2")
        .parse::<u32>()
        .expect("thread-count requires a valid positive integer");
    if num_threads == 0_u32 {
        panic!("Need a thread count greater than zero");
    }

    // Read the scene spec file
    let mut res = Resources::new();
    let scene_spec_path = path::Path::new(matches.value_of("IN_SCENE_FILE").unwrap());
    let scene_str = fs::read_to_string(&scene_spec_path).expect("Failed to read scene spec file.");
    let scene_spec = Arc::new(
        scene::deserialize(
            &scene_str,
            match scene_spec_path.parent() {
                Some(p) => p,
                None => path::Path::new("/"),
            },
            &mut res,
        )
        .expect("Failed to parse scene spec JSON."),
    );

    // Create the output file according to input path
    let out_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(matches.value_of("OUT_FILEPATH").unwrap())
        .expect("Failed to create new file");
    let png_encoder = PNGEncoder::new(out_file);

    // Specifications
    let res_x = scene_spec.logistics.resolution_x;
    let res_y = scene_spec.logistics.resolution_y;
    let samples = scene_spec.logistics.samples;

    // Init output color float data with empty values.
    let colors = Arc::new(Mutex::new(Vec::new()));
    {
        (*colors
            .lock()
            .expect("Failed to acquire output data lock for resizing."))
        .resize_with((res_x * res_y) as usize, RGB::black);
    }

    // Set up a queue of input pixels + samples for threads to process
    let (tx, rx) = {
        let (temp_tx, temp_rx) = mpsc::channel();
        (temp_tx, Arc::new(Mutex::new(temp_rx)))
    };

    // Set up a structure to track progress and print to standard out
    let progress_tracker = Arc::new(Mutex::new(Progress::new(
        res_x as u64 * res_y as u64 * samples as u64,
        Arc::new(Mutex::new(io::stdout())),
        20_u32,
    )));

    // Spawn threads up to the desired amount (minus one,
    // because the main thread is a thread too)
    let mut threads = Vec::new();
    for _ in 0..(num_threads - 1_u32) {
        let thread_scene = Arc::clone(&scene_spec);
        let thread_rx = Arc::clone(&rx);
        let thread_colors = Arc::clone(&colors);
        let thread_progress = Arc::clone(&progress_tracker);
        threads.push(thread::spawn(move || {
            thread_work(&thread_scene, &thread_rx, &thread_colors, &thread_progress)
        }))
    }

    // Fill queue with data
    for x in 0..res_x {
        for y in 0..res_y {
            for _ in 0..samples {
                tx.send((x, y))
                    .expect("Main thread failed to send pixel data into queue.");
            }
        }
    }
    // Drop Sender so threads can close on their own
    drop(tx);
    // Start having the main thread do some work too
    thread_work(&scene_spec, &rx, &colors, &progress_tracker);
    // Wait for tracing threads to complete if the main thread completes early
    for t in threads {
        t.join().expect("Failed to finalize a tracing thread.");
    }
    (*progress_tracker).lock().unwrap().done();

    // Once all tracing has been done, finalize data and convert to
    // 8 bit unsigned integer
    let mut data = Vec::with_capacity((res_x * res_y * 3_u32) as usize);
    let locked_colors = &mut (*colors
        .lock()
        .expect("Main thread failed to lock output color data for writing to image."));
    for y in 0..res_y {
        for x in 0..res_x {
            let mut col = locked_colors[((x * res_y) + y) as usize] / samples as f32;
            col = RGB::new(col.r.sqrt(), col.g.sqrt(), col.b.sqrt());

            data.push((col.r * COLOR_SPACE) as u8);
            data.push((col.g * COLOR_SPACE) as u8);
            data.push((col.b * COLOR_SPACE) as u8);
        }
    }
    // Write the image to disk
    match png_encoder.encode(&data, res_x, res_y, ColorType::RGB(8)) {
        Ok(()) => println!(
            "Success! Took {} seconds",
            program_start.elapsed().as_secs_f64()
        ),
        Err(e) => {
            eprintln!("Failed to encode the png for output: {}", e);
            process::exit(1);
        }
    }
}

fn thread_work(
    thread_scene: &Scene,
    thread_rx: &Mutex<mpsc::Receiver<(u32, u32)>>,
    thread_colors: &Mutex<Vec<RGB>>,
    thread_progress: &Mutex<Progress>,
) {
    let res_x = thread_scene.logistics.resolution_x;
    let res_y = thread_scene.logistics.resolution_y;
    let mut aggregate_workspace = thread_scene.shape_aggregate.get_workspace();

    loop {
        let (x, y) = {
            match thread_rx
                .lock()
                .expect("Thread failed acquiring lock on input data queue.")
                .iter()
                .next()
            {
                Some((x, y)) => (x, y),
                None => break,
            }
        };

        // Note the use of rand::random. Consider switching to an explicit
        // use of SmallRng, which is a non-secure, but fast, pseudo-RNG.
        // The default implementation may not be as performant, and this
        // program does not need the extra security benefits.
        let u = (x as f32 + rand::random::<f32>()) / res_x as f32;
        let v = ((res_y - y) as f32 + rand::random::<f32>()) / res_y as f32;
        let r = thread_scene.camera.get_ray(u, v);

        let pixel_color = aggregate::trace(
            &r,
            &(*thread_scene.shape_aggregate),
            &thread_scene.important_samples,
            &mut aggregate_workspace,
            &black_background,
            0,
        );

        {
            let out_colors = &mut (*thread_colors
                .lock()
                .expect("Thread failed to acquire output data lock."));
            out_colors[((x * res_y) + y) as usize] =
                out_colors[((x * res_y) + y) as usize] + pixel_color;
        }

        {
            thread_progress.lock().unwrap().update(1);
        }
    }
}

/*
fn background(r: &Ray) -> RGB {
    // Sky blend
    let dir_normal = r.dir.normalized();
    let t = 0.5_f32 * (dir_normal.y + 1.0_f32);

    RGB::new(1.0_f32, 1.0_f32, 1.0_f32) * (1.0_f32 - t) + RGB::new(0.5_f32, 0.7_f32, 1.0_f32) * t
}
*/
fn black_background(_: &Ray) -> RGB {
    RGB::black()
}
