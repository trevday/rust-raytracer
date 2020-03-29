use crate::ray::Ray;
use crate::shape::Shape;
use crate::vector::Axis;
use crate::vector::Vector3;

use std::cmp;
use std::mem;

const MAX_DEPTH: i32 = 50;

pub fn trace(
    r: &Ray,
    shape_aggregate: &dyn Aggregate,
    workspace: &mut Workspace,
    bg_func: &dyn Fn(&Ray) -> Vector3,
    depth: i32,
) -> Vector3 {
    let hit_shape = hit(shape_aggregate, workspace, r);

    if depth < MAX_DEPTH {
        match hit_shape {
            // Some if we have a hit
            Some((s, t)) => {
                let normal = s.derive_normal(r, t);

                let hit_point = r.point_at(t);
                match s.get_material().scatter(r, &hit_point, &normal) {
                    // Some if we scattered
                    Some((attenuation, scattered_ray)) => {
                        // Recursive case
                        return attenuation
                            * trace(
                                &scattered_ray,
                                shape_aggregate,
                                workspace,
                                bg_func,
                                depth + 1,
                            );
                    }
                    None => {
                        return Vector3::new_empty();
                    }
                }
            }
            // None if we don't, no-op
            None => {}
        }
    }

    // Return BG color
    return bg_func(r);
}

// Workspaces are optional, but some aggregate structures (like BVH)
// can use them to improve performance.
pub enum Workspace {
    Void,
    BVH(Vec<usize>),
}

pub trait Aggregate {
    // Hit for an aggregate returns the shape that was hit and the time
    // 't' at which it was hit along the path of the ray.
    // None is returned if no shape is hit.
    fn hit(
        &self,
        r: &Ray,
        t_min: f32,
        t_max: f32,
        workspaces: &mut Workspace,
    ) -> Option<(&dyn Shape, f32)>;

    fn get_workspace(&self) -> Workspace {
        return Workspace::Void;
    }
}

// Small convenience function
const T_MIN: f32 = 0.001_f32;
const T_MAX: f32 = std::f32::MAX;
fn hit<'a>(
    aggregate: &'a dyn Aggregate,
    workspace: &mut Workspace,
    r: &Ray,
) -> Option<(&'a dyn Shape, f32)> {
    aggregate.hit(r, T_MIN, T_MAX, workspace)
}

// Simple list aggregate
type List = Vec<Box<dyn Shape>>;

impl Aggregate for List {
    fn hit(&self, r: &Ray, t_min: f32, t_max: f32, _: &mut Workspace) -> Option<(&dyn Shape, f32)> {
        let mut modified_t_max = t_max;
        let mut hit_shape: Option<&dyn Shape> = None;

        for shape in self {
            match shape.hit(r, t_min, modified_t_max) {
                Some(t) => {
                    modified_t_max = t;
                    hit_shape = Some(&(*(*shape)));
                }
                // No-op
                None => {}
            }
        }

        match hit_shape {
            Some(s) => Some((s, modified_t_max)),
            None => None,
        }
    }
}

// Bounding Volume Hierarchy
type BVH = Vec<BVHTypes>;
enum BVHTypes {
    Leaf(BVHLeaf),
    Node(BVHNode),
}
struct BVHLeaf {
    bounding_box: AABB,
    shapes: List, // Just uses the simple shape list aggregate
}
struct BVHNode {
    bounding_box: AABB,
    cut_axis: Axis,
    // Left is implicitly always stored as this node's
    // index + 1. This is suitable for depth first
    // searching, which we will do for hit testing.
    right_offset: usize,
}

// Constructs a new BVH using the Surface Area Heuristic (SAH).
pub fn new_bvh(shapes: Vec<Box<dyn Shape>>) -> Box<dyn Aggregate> {
    let mut bvh = Box::new(Vec::new());
    new_bvh_helper(&mut (*bvh), shapes);
    return bvh;
}
// Helper for recursive case of BVH construction.
fn new_bvh_helper(bvh: &mut BVH, mut shapes: Vec<Box<dyn Shape>>) {
    // Calculate total bounds for this iteration
    let mut total_bounds = AABB::new_empty();
    for shape in &shapes {
        total_bounds = AABB::union(&total_bounds, &shape.get_bounding_box());
    }

    // If we only have a couple shapes, just make a leaf
    if (&shapes).len() <= 2 {
        bvh.push(BVHTypes::Leaf(BVHLeaf {
            bounding_box: total_bounds,
            shapes: shapes,
        }));
        return;
    }

    // Compute centroid (center of bounding boxes) bounds
    let mut centroid_bounds = AABB::new_empty();
    for shape in &shapes {
        centroid_bounds = AABB::union_vector(&centroid_bounds, &shape.get_bounding_box().center());
    }
    // We will cut over the dimension for which bounding box centers cover the
    // largest area
    let cut_axis = centroid_bounds.largest_axis();

    // If we have zero area to split over, just make a leaf
    if centroid_bounds.max[cut_axis] == centroid_bounds.min[cut_axis] {
        bvh.push(BVHTypes::Leaf(BVHLeaf {
            bounding_box: total_bounds,
            shapes: shapes,
        }));
        return;
    }

    // Sort shapes by centroids
    //
    // TODO (performance): It's unfortunate to do an n(log(n)) operation here, but
    // at the same time BVH construction has not proven to be the bottleneck of
    // the program. Should it become an issue, I can consider slightly less
    // optimal, but linear time, alternatives, such as partitioning with buckets.
    shapes.sort_by(|a, b| {
        let a_c = a.get_bounding_box().center()[cut_axis];
        let b_c = b.get_bounding_box().center()[cut_axis];
        if a_c < b_c {
            cmp::Ordering::Less
        } else if a_c > b_c {
            cmp::Ordering::Greater
        } else {
            cmp::Ordering::Equal
        }
    });

    // Apply SAH:
    // Start by calculating bounds at each possible split point in reverse,
    // a linear operation.
    let mut reverse_bounds = Vec::with_capacity(shapes.len());
    reverse_bounds.resize_with(shapes.len(), AABB::new_empty);
    for reverse_idx in (0..(shapes.len() - 1)).rev() {
        reverse_bounds[reverse_idx] = shapes[reverse_idx].get_bounding_box();
        if reverse_idx + 1 < shapes.len() {
            reverse_bounds[reverse_idx] = AABB::union(
                &reverse_bounds[reverse_idx],
                &reverse_bounds[reverse_idx + 1],
            );
        }
    }
    // Then iterate forward, applying SAH at each split point.
    let mut forward_bounds = AABB::new_empty();
    let mut min_cost = std::f32::MAX;
    let mut min_cost_index = 0;
    for idx in 0..shapes.len() - 1 {
        forward_bounds = AABB::union(&forward_bounds, &shapes[idx].get_bounding_box());
        let cost =
        // Extra cost incurred by the ray to bounding box intersection should we make a node
        1_f32 +
        // (Probability of going through A) * (Cost to iterate A (1 per element in A))
        ((forward_bounds.surface_area() / total_bounds.surface_area()) * (idx + 1) as f32) +
        // (Probability of going through B) * (Cost to iterate B (1 per element in B))
        ((reverse_bounds[idx + 1].surface_area() / total_bounds.surface_area()) * (shapes.len() - (idx + 1)) as f32);
        // Pick min cost
        if cost < min_cost {
            min_cost = cost;
            min_cost_index = idx;
        }
    }

    // Compare split cost to cost of creating a leaf,
    // which is 1 per element.
    if min_cost < shapes.len() as f32 {
        // Split the shape vector into two pieces at our split index
        let second_half = shapes.split_off(min_cost_index + 1);

        // NOTE: This is a bit of a workaround to handle Rust's safety guarantees
        // but also maintain the readability of just pushing to "bvh" most
        // of the time. I push a placeholder node that gets overwritten in
        // a moment when I know what my real right_offset value should be.
        bvh.push(BVHTypes::Node(BVHNode {
            bounding_box: AABB::new_empty(),
            cut_axis: cut_axis,
            right_offset: 0,
        }));
        let node_idx = bvh.len() - 1;

        // Add the left branch
        new_bvh_helper(bvh, shapes);

        // Now do the replacement of the node with
        // a correct right_offset
        bvh[node_idx] = BVHTypes::Node(BVHNode {
            bounding_box: total_bounds,
            cut_axis: cut_axis,
            // Offset is current length minus this node's index,
            // because we know we are going to add at least a
            // leaf to represent the right branch, and this leaf
            // will reside at the index currently represented by
            // bvh's length
            right_offset: bvh.len() - node_idx,
        });

        // Last, add the right branch
        new_bvh_helper(bvh, second_half);
        return;
    }
    // If it's cheap enough, just make the leaf
    bvh.push(BVHTypes::Leaf(BVHLeaf {
        bounding_box: total_bounds,
        shapes: shapes,
    }));
    return;
}

impl Aggregate for BVH {
    fn hit(
        &self,
        r: &Ray,
        t_min: f32,
        t_max: f32,
        workspace: &mut Workspace,
    ) -> Option<(&dyn Shape, f32)> {
        // Grab the workspace as the pre-allocated vector
        // we expect it to be.
        let to_explore = match workspace {
            Workspace::BVH(v) => v,
            _ => panic!("BVH Aggregate was given a bad workspace!"),
        };

        if self.is_empty() {
            return None;
        }

        let mut modified_t_max = t_max;
        let mut hit_shape: Option<&dyn Shape> = None;

        let mut to_explore_count = 1;
        to_explore[0] = 0;

        while to_explore_count > 0 {
            // "Pop" the top value
            to_explore_count -= 1;
            let cur_idx = to_explore[to_explore_count];

            match &self[cur_idx] {
                BVHTypes::Leaf(leaf) => {
                    if !leaf.bounding_box.intersect(r, t_min, modified_t_max) {
                        continue;
                    }
                    match leaf
                        .shapes
                        .hit(r, t_min, modified_t_max, &mut Workspace::Void)
                    {
                        Some((s, t)) => {
                            modified_t_max = t;
                            hit_shape = Some(s);
                        }
                        None => {}
                    }
                }
                BVHTypes::Node(node) => {
                    if !node.bounding_box.intersect(r, t_min, modified_t_max) {
                        continue;
                    }
                    // NOTE: This is a micro-optimization where the axis this node was
                    // split along is cached so that the ray can be inspected and it
                    // can be guessed which of the two branches is most likely to be
                    // hit first.
                    if r.dir[node.cut_axis] < 0.0_f32 {
                        // Right Branch
                        to_explore[to_explore_count] = cur_idx + node.right_offset;
                        to_explore_count += 1;
                        // Left Branch
                        to_explore[to_explore_count] = cur_idx + 1_usize;
                        to_explore_count += 1;
                    } else {
                        // Left Branch
                        to_explore[to_explore_count] = cur_idx + 1_usize;
                        to_explore_count += 1;
                        // Right Branch
                        to_explore[to_explore_count] = cur_idx + node.right_offset;
                        to_explore_count += 1;
                    }
                }
            }
        }

        match hit_shape {
            Some(s) => Some((s, modified_t_max)),
            None => None,
        }
    }

    // Allocate this conservatively, so that we never
    // have to allocate more space in our hit loop
    fn get_workspace(&self) -> Workspace {
        let mut v = Vec::with_capacity(self.len());
        v.resize(self.len(), 0_usize);
        return Workspace::BVH(v);
    }
}

// Axis Aligned Bounding Box
pub struct AABB {
    min: Vector3,
    max: Vector3,
}

impl AABB {
    pub fn new(min: Vector3, max: Vector3) -> AABB {
        AABB { min: min, max: max }
    }

    fn new_empty() -> AABB {
        AABB {
            min: Vector3::new_empty(),
            max: Vector3::new_empty(),
        }
    }

    fn union(box1: &AABB, box2: &AABB) -> AABB {
        AABB {
            min: Vector3::min(box1.min, box2.min),
            max: Vector3::max(box1.max, box2.max),
        }
    }

    fn union_vector(box1: &AABB, vector: &Vector3) -> AABB {
        AABB {
            min: Vector3::min(box1.min, *vector),
            max: Vector3::max(box1.max, *vector),
        }
    }

    fn center(&self) -> Vector3 {
        0.5_f32 * self.min + 0.5_f32 * self.max
    }

    fn largest_axis(&self) -> Axis {
        let diagonal = self.max - self.min;
        if diagonal.x > diagonal.y && diagonal.x > diagonal.z {
            Axis::X
        } else if diagonal.y > diagonal.z {
            Axis::Y
        } else {
            Axis::Z
        }
    }

    fn surface_area(&self) -> f32 {
        let diagonal = self.max - self.min;
        2_f32 * (diagonal.x * diagonal.y + diagonal.x * diagonal.z + diagonal.y * diagonal.z)
    }

    fn intersect(&self, r: &Ray, t_min: f32, t_max: f32) -> bool {
        // X
        let (t_min_temp, t_max_temp) = self.intersect_helper(r, t_min, t_max, Axis::X);
        if t_max_temp <= t_min_temp {
            return false;
        }
        // Y
        let (t_min_temp, t_max_temp) = self.intersect_helper(r, t_min_temp, t_max_temp, Axis::Y);
        if t_max_temp <= t_min_temp {
            return false;
        }
        // Z
        let (t_min_temp, t_max_temp) = self.intersect_helper(r, t_min_temp, t_max_temp, Axis::Z);
        if t_max_temp <= t_min_temp {
            return false;
        }

        return true;
    }

    fn intersect_helper(&self, r: &Ray, t_min: f32, t_max: f32, axis: Axis) -> (f32, f32) {
        let inverse_direction = 1.0_f32 / r.dir[axis];
        let mut t0 = (self.min[axis] - r.origin[axis]) * inverse_direction;
        let mut t1 = (self.max[axis] - r.origin[axis]) * inverse_direction;
        if inverse_direction < 0.0_f32 {
            mem::swap(&mut t0, &mut t1);
        }

        (
            if t0 > t_min { t0 } else { t_min },
            if t1 < t_min { t1 } else { t_max },
        )
    }
}
