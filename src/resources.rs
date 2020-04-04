use image;
use image::DynamicImage;
use std::{collections::HashMap, path::Path, rc::Rc};

pub struct Resources {
    loaded_images: HashMap<String, Rc<DynamicImage>>,
}

impl Resources {
    pub fn new() -> Resources {
        Resources {
            loaded_images: HashMap::new(),
        }
    }

    pub fn load_image(&mut self, image_path: &Path) -> Result<Rc<DynamicImage>, String> {
        let absolute_path = match image_path.canonicalize() {
            Ok(p) => p,
            Err(e) => {
                return Err(format!(
                    "There was a problem finding the given image path: {}",
                    e
                ))
            }
        };
        let path_str = match absolute_path.to_str() {
            Some(p) => p,
            None => {
                return Err(String::from(
                    "There was a problem using the given image path as a key.",
                ))
            }
        };
        if self.loaded_images.contains_key(path_str) {
            return match self.loaded_images.get(path_str) {
                Some(v) => Ok(Rc::clone(v)),
                None => Err(String::from("Unexpected issue loading from image map.")),
            };
        }

        let image_buffer = match image::open(&absolute_path) {
            Ok(i) => i,
            Err(e) => return Err(format!("Could not open image: {}", e)),
        };
        self.loaded_images
            .insert(String::from(path_str), Rc::new(image_buffer));
        return match self.loaded_images.get(path_str) {
            Some(v) => Ok(Rc::clone(v)),
            None => Err(String::from("Unexpected issue loading from image map.")),
        };
    }
}
