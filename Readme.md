# Ray Tracer in Rust
This is a moderately complex ray tracer with a fair number of features. This project fulfilled two primary personal goals. One was to familiarize myself with Rust by building a complete program that touches on a number of different language features (architecture, performance, multithreading, etc.). The other was to construct a more mature ray tracer than my previous attempts. While there are many more features, optimizations, and refactors I would like to implement at some point, I am satisfied with the current code for "V1" of this project.

## Gallery
![Cornell Box](/img/cornell_box.png)
![Globe](/img/globe.png)
![Example of Materials](/img/materials.png)
![Spaceship](/img/ship.png)
Ship model is from [here](https://www.turbosquid.com/FullPreview/Index.cfm/ID/798902).

## Features
* Basic shapes
	* Sphere
	* Triangle
* Basic materials library
	* Lambert
	* Metal
	* Dielectric
* Custom JSON scene specification format
	* Supports reading from `.obj` meshes
* Bounding volume hierarchy accelerates collision detection
	* Uses Surface Area Heuristic (SAH)
* Diffuse lights
* Textures
	* Solid
	* Perlin Noise
	* Images
* Transformations allow scene manipulation
* Basic volumes
* Bump mapping
* Monte Carlo importance sampling
* Multithreaded
	* Number of threads is an optional command line argument
* Basic stats and progress report

## Instructions
I would recommend building using the official Rust package manager, `cargo`. For more information, see the official [Getting Started](https://www.rust-lang.org/learn/get-started). Once built, the basic command is `rust-raytracer [OPTIONS] <IN_SCENE_FILE> <OUT_FILEPATH>`. `IN_SCENE_FILE` is the relative path to the scene specification, and `OUT_FILEPATH` is the relative filepath you wish to write the output image to. By default output images are in the `.png` image format. `--help` will also print this information.

### Scene Specification Format
There are example scene specifications available in `assets/`.
#### (TODO: Scene Specification Documentation)

### Dependencies
* [image](https://crates.io/crates/image)
* [serde](https://crates.io/crates/serde)
* [serde_json](https://crates.io/crates/serde_json)
* [typetag](https://crates.io/crates/typetag)
* [wavefront_obj](https://crates.io/crates/wavefront_obj)
* [clap](https://crates.io/crates/clap)

## Resources
I never would have built this ray tracer without the invaluable knowledge presented by Peter Shirley in the [Ray Tracing Book Series](https://raytracing.github.io/) and Matt Pharr, Wenzel Jakob, and Greg Humphreys in [Physically Based Rendering](https://www.pbrt.org/). The feature set and implementation of this ray tracer is based off these texts.

## Wishlist
Just some things I am thinking about implementing:
* More shapes
	* Cylinders
	* Disks
* Normal mapping
* "Non-physical" lights
	* Point
	* Spot
	* Directional
* Different texture mappings
	* Spherical
	* Cylindrical
* True BSDF support
* Read `.mtl` files
* More robust statistics reporting