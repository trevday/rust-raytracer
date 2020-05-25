use crate::matrix::Matrix4;
use crate::vector::Vector3;

use serde::Deserialize;

// Note on usage of Transforms: All calculations in this program are typically
// done in terms of world space. If an object can cache data in terms of
// world space and still function correctly, that is the approach that is
// preferred. Most Transforms, therefore, will not make it past the
// deserialization and loading step before being consumed. For any objects
// that require Transformations during runtime, this should be handled
// internally in the implementation of that object, and all inputs and
// outputs should be assumed to be world space unless otherwise specified.
#[derive(Deserialize)]
pub struct Transform {
    #[serde(default = "Vector3::new_empty")]
    translate: Vector3,
    #[serde(default = "Vector3::new_empty")]
    rotate: Vector3,
    #[serde(default = "Vector3::new_identity")]
    scale: Vector3,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            translate: Vector3::new_empty(),
            rotate: Vector3::new_empty(),
            scale: Vector3::new_identity(),
        }
    }

    pub fn create_matrix(&self) -> Matrix4 {
        return Matrix4::new_translation(&self.translate)
            * Matrix4::new_rotation_x(self.rotate.x())
            * Matrix4::new_rotation_y(self.rotate.y())
            * Matrix4::new_rotation_z(self.rotate.z())
            * Matrix4::new_scale(&self.scale);
    }
}
