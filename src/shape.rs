use crate::aggregate::AABB;
use crate::material::Material;
use crate::ray::Ray;
use crate::vector::Vector3;

use std::f32;
use std::rc::Rc;

pub trait Shape {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<f32>;
    fn derive_normal(&self, r: &Ray, t_hit: f32) -> Vector3;
    fn get_uv_coords(&self, r: &Ray, t_hit: f32) -> (f32, f32);
    fn get_material(&self) -> &Rc<dyn Material>;
    fn get_bounding_box(&self) -> AABB;
}

pub struct Sphere {
    center: Vector3,
    radius: f32,
    // NOTE: There is a tradeoff here between making an enum struct and a pointer to a trait object.
    // The enum struct would be slightly more efficient as it is immediately available
    // for use without having to reach into the Heap, but adding new variants is more
    // troublesome, and especially large variants may make the required size of each
    // Material too large. The Rc + trait object allows easier creation of Material
    // variants, but introduces a small performance penalty to read from Heap memory.
    //
    // TODO: Further investigate Pointer-Enum, performance vs. memory tradeoff if
    // optimization is required.
    material: Rc<dyn Material>,
}

impl Sphere {
    pub fn new(center: Vector3, radius: f32, mat: Rc<dyn Material>) -> Sphere {
        Sphere {
            center: center,
            radius: radius,
            material: mat,
        }
    }
}

const ONE_OVER_2_PI: f32 = 1.0_f32 / (2.0_f32 * f32::consts::PI);
impl Shape for Sphere {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<f32> {
        let towards_origin = r.origin - self.center;
        let a = r.dir.dot(r.dir);
        let b = 2.0_f32 * towards_origin.dot(r.dir);
        let c = towards_origin.dot(towards_origin) - (self.radius * self.radius);
        let discriminant = b * b - 4.0_f32 * a * c;

        if discriminant > 0.0_f32 {
            let mut t_hit = (-b - discriminant.sqrt()) / (2.0_f32 * a);
            if t_hit >= t_max || t_hit <= t_min {
                t_hit = (-b + discriminant.sqrt()) / (2.0_f32 * a);
            }

            if t_hit < t_max && t_hit > t_min {
                return Some(t_hit);
            }
        }
        return None;
    }

    fn derive_normal(&self, r: &Ray, t_hit: f32) -> Vector3 {
        ((r.point_at(t_hit) - self.center) / self.radius).normalized()
    }

    fn get_uv_coords(&self, r: &Ray, t_hit: f32) -> (f32, f32) {
        let unit_sphere_point = (r.point_at(t_hit) - self.center) / self.radius;
        (
            // u
            (1.0_f32
                - ((unit_sphere_point.z.atan2(unit_sphere_point.x) + f32::consts::PI)
                    * ONE_OVER_2_PI)),
            // v
            ((unit_sphere_point.y.asin() + f32::consts::FRAC_PI_2) * f32::consts::FRAC_1_PI),
        )
    }

    fn get_material(&self) -> &Rc<dyn Material> {
        &self.material
    }

    fn get_bounding_box(&self) -> AABB {
        AABB::new(
            self.center - Vector3::new(self.radius, self.radius, self.radius),
            self.center + Vector3::new(self.radius, self.radius, self.radius),
        )
    }
}

pub struct TriangleMesh {
    vertices: Vec<Vector3>,
    // TODO: Decide if I have enough need for a real Vector2 struct.
    tex_coords: Vec<(f32, f32)>,
    enable_backface_culling: bool,
    material: Rc<dyn Material>,
}

impl TriangleMesh {
    pub fn new(
        vertices: Vec<Vector3>,
        tex_coords: Vec<(f32, f32)>,
        enable_backface_culling: bool,
        material: Rc<dyn Material>,
    ) -> TriangleMesh {
        TriangleMesh {
            vertices: vertices,
            tex_coords: tex_coords,
            enable_backface_culling: enable_backface_culling,
            material: material,
        }
    }
}

pub struct Triangle {
    triangle_mesh: Rc<TriangleMesh>,
    // TODO: Make Vector generic over the data type,
    // and use it here.
    v0: usize,
    v1: usize,
    v2: usize,
    t0: Option<usize>,
    t1: Option<usize>,
    t2: Option<usize>,
}

impl Triangle {
    pub fn new(
        mesh: Rc<TriangleMesh>,
        v0: usize,
        v1: usize,
        v2: usize,
        t0: Option<usize>,
        t1: Option<usize>,
        t2: Option<usize>,
    ) -> Result<Triangle, String> {
        if mesh.vertices.is_empty()
            || mesh.vertices.len() - 1 < v0
            || mesh.vertices.len() - 1 < v1
            || mesh.vertices.len() - 1 < v2
        {
            return Err(format!(
					"Triangle mesh has length {} but attempted to make a Triangle with indices {}, {}, {}.",
					mesh.vertices.len(),
					v0,
					v1,
					v2));
        }
        match t0 {
            Some(t) => {
                if t >= mesh.tex_coords.len() {
                    return Err(format!("Triangle texture coordinates have length {} but attempted to make a Triangle with texture index {}.",
            mesh.tex_coords.len(),
            t));
                }
            }
            None => {}
        }
        match t1 {
            Some(t) => {
                if t >= mesh.tex_coords.len() {
                    return Err(format!("Triangle texture coordinates have length {} but attempted to make a Triangle with texture index {}.",
            mesh.tex_coords.len(),
            t));
                }
            }
            None => {}
        }
        match t2 {
            Some(t) => {
                if t >= mesh.tex_coords.len() {
                    return Err(format!("Triangle texture coordinates have length {} but attempted to make a Triangle with texture index {}.",
            mesh.tex_coords.len(),
            t));
                }
            }
            None => {}
        }
        Ok(Triangle {
            triangle_mesh: mesh,
            v0: v0,
            v1: v1,
            v2: v2,
            t0: t0,
            t1: t1,
            t2: t2,
        })
    }
}

impl Shape for Triangle {
    // Uses Moller-Trumbore ray-triangle intersection algorithm.
    // https://en.wikipedia.org/wiki/M%C3%B6ller%E2%80%93Trumbore_intersection_algorithm
    //
    // Backface culling expects a counter-clockwise winding order.
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32) -> Option<f32> {
        let vertex0 = self.triangle_mesh.vertices[self.v0];
        let vertex1 = self.triangle_mesh.vertices[self.v1];
        let vertex2 = self.triangle_mesh.vertices[self.v2];

        let edge_1 = vertex1 - vertex0;
        let edge_2 = vertex2 - vertex0;
        let p_vec = r.dir.cross(edge_2);
        let determinant = edge_1.dot(p_vec);

        if !self.triangle_mesh.enable_backface_culling
            && determinant > -std::f32::EPSILON
            && determinant < std::f32::EPSILON
        {
            return None; // Indicates parallel ray and triangle
        } else if self.triangle_mesh.enable_backface_culling && determinant < std::f32::EPSILON {
            return None; // Either parallel or ray approaching triangle from back
        }

        let inverse_determinant = 1.0_f32 / determinant;
        let t_vec = r.origin - vertex0;
        let u = t_vec.dot(p_vec) * inverse_determinant;
        if u < 0.0_f32 || u > 1.0_f32 {
            return None;
        }

        let q_vec = t_vec.cross(edge_1);
        let v = r.dir.dot(q_vec) * inverse_determinant;
        if v < 0.0_f32 || u + v > 1.0_f32 {
            return None;
        }

        let t_hit = edge_2.dot(q_vec) * inverse_determinant;
        if t_hit < t_max && t_hit > t_min {
            return Some(t_hit);
        }
        return None;
    }

    fn derive_normal(&self, r: &Ray, _t_hit: f32) -> Vector3 {
        let vertex0 = self.triangle_mesh.vertices[self.v0];
        let vertex1 = self.triangle_mesh.vertices[self.v1];
        let vertex2 = self.triangle_mesh.vertices[self.v2];

        // TODO: Some repeated work here to derive the normal.
        // Is it worth combining the normal calculation logic
        // into the hit function? Other shapes do not have
        // repeated work (Sphere) so it's a tradeoff
        // for different types of shapes.
        let edge_1 = vertex1 - vertex0;
        let edge_2 = vertex2 - vertex0;
        let p_vec = r.dir.cross(edge_2);
        let determinant = edge_1.dot(p_vec);

        let mut normal = edge_1.cross(edge_2).normalized();
        if determinant < 0.0_f32 {
            normal = -normal; // Ray came from the back so reverse the normal
        }
        return normal;
    }

    fn get_uv_coords(&self, r: &Ray, _t_hit: f32) -> (f32, f32) {
        let vertex0 = self.triangle_mesh.vertices[self.v0];
        let vertex1 = self.triangle_mesh.vertices[self.v1];
        let vertex2 = self.triangle_mesh.vertices[self.v2];

        // TODO: More repeated work here; see TODO in derive_normal.
        // There may be a way to avoid repeating work here and
        // reverse calculate data from the hit point.
        let edge_1 = vertex1 - vertex0;
        let edge_2 = vertex2 - vertex0;
        let p_vec = r.dir.cross(edge_2);
        let determinant = edge_1.dot(p_vec);

        let inverse_determinant = 1.0_f32 / determinant;
        let t_vec = r.origin - vertex0;
        let u = t_vec.dot(p_vec) * inverse_determinant;

        let q_vec = t_vec.cross(edge_1);
        let v = r.dir.dot(q_vec) * inverse_determinant;

        let w = 1.0_f32 - u - v;

        let (u0, v0) = match self.t0 {
            Some(t) => self.triangle_mesh.tex_coords[t],
            None => (0_f32, 0_f32),
        };
        let (u1, v1) = match self.t1 {
            Some(t) => self.triangle_mesh.tex_coords[t],
            None => (1_f32, 0_f32),
        };
        let (u2, v2) = match self.t2 {
            Some(t) => self.triangle_mesh.tex_coords[t],
            None => (1_f32, 1_f32),
        };

        // Apply to UV coordinates from mesh
        return ((u0 * u + u1 * v + u2 * w), (v0 * u + v1 * v + v2 * w));
    }

    fn get_material(&self) -> &Rc<dyn Material> {
        &self.triangle_mesh.material
    }

    fn get_bounding_box(&self) -> AABB {
        let vertex0 = self.triangle_mesh.vertices[self.v0];
        let vertex1 = self.triangle_mesh.vertices[self.v1];
        let vertex2 = self.triangle_mesh.vertices[self.v2];

        AABB::new(
            Vector3::min(vertex0, Vector3::min(vertex1, vertex2)),
            Vector3::max(vertex0, Vector3::max(vertex1, vertex2)),
        )
    }
}
