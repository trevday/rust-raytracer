use crate::aggregate::{new_bvh, SyncAggregate};
use crate::camera::Camera;
use crate::material;
use crate::material::SyncMaterial;
use crate::pdf;
use crate::point::Point3;
use crate::resources::Resources;
use crate::shape;
use crate::shape::SyncShape;
use crate::texture;
use crate::texture::SyncTexture;
use crate::transform::Transform;
use crate::volume;

use serde::Deserialize;
use serde_json;
use std::{collections::HashMap, convert, fs, io, path, sync::Arc};
use wavefront_obj::obj;

pub struct Scene {
    pub logistics: Logistics,
    pub camera: Camera,
    pub shape_aggregate: Box<SyncAggregate>,
    pub important_samples: Arc<pdf::PDF>,
}

#[derive(Deserialize)]
pub struct Logistics {
    pub resolution_x: u32,
    pub resolution_y: u32,
    pub samples: u32,
}

// Package together third party library errors and
// Scene local errors to be returned from the
// deserialize function.
#[derive(Debug)]
pub enum DeserializeError {
    ObjLibraryError(wavefront_obj::ParseError),
    JsonLibraryError(serde_json::Error),
    IoError(io::Error),
    LocalError(String),
}
impl convert::From<wavefront_obj::ParseError> for DeserializeError {
    fn from(obj_error: wavefront_obj::ParseError) -> Self {
        DeserializeError::ObjLibraryError(obj_error)
    }
}
impl convert::From<serde_json::Error> for DeserializeError {
    fn from(serde_error: serde_json::Error) -> Self {
        DeserializeError::JsonLibraryError(serde_error)
    }
}
impl convert::From<io::Error> for DeserializeError {
    fn from(err: io::Error) -> Self {
        DeserializeError::IoError(err)
    }
}

// Deserializes a JSON scene specification correctly
// into a scene structure.
pub fn deserialize(
    data: &str,
    spec_dir: &path::Path,
    res: &mut Resources,
) -> Result<Scene, DeserializeError> {
    let top_level: serde_json::Value = serde_json::from_str(data)?;
    if !top_level.is_object() {
        return Err(DeserializeError::LocalError(String::from(
            "Top level scene spec is not a JSON object.",
        )));
    }

    // Pull out logistics struct
    let logistics_value = get_required_key(&top_level, "Logistics")?;
    let logistics: Logistics = serde_json::from_value(serde_json::Value::clone(logistics_value))?;

    // Pull out camera struct
    let camera_value = get_required_key(&top_level, "Camera")?;
    let camera: Camera = serde_json::from_value(serde_json::Value::clone(camera_value))?;

    // Create textures library
    let textures_value = match get_required_key(&top_level, "Textures")?.as_object() {
        Some(t) => t,
        None => {
            return Err(DeserializeError::LocalError(String::from(
                "'Textures' is not a JSON object.",
            )));
        }
    };
    let mut textures = HashMap::new();
    for (key, value) in textures_value.iter() {
        textures.insert(
            String::clone(key),
            deserialize_texture(value, spec_dir, res)?,
        );
    }

    // Create materials library
    let materials_value = match get_required_key(&top_level, "Materials")?.as_object() {
        Some(m) => m,
        None => {
            return Err(DeserializeError::LocalError(String::from(
                "'Materials' is not a JSON object.",
            )))
        }
    };
    let mut materials = HashMap::new();
    for (key, value) in materials_value.iter() {
        materials.insert(String::clone(key), deserialize_material(value, &textures)?);
    }

    // Set up shapes
    let shapes_value = match get_required_key(&top_level, "Shapes")?.as_array() {
        Some(s) => s,
        None => {
            return Err(DeserializeError::LocalError(String::from(
                "'Shapes' is not a JSON array.",
            )))
        }
    };
    // Iterate through the shapes and deserialize correctly
    let mut shapes: Vec<Arc<SyncShape>> = Vec::with_capacity(shapes_value.len());
    for shape in shapes_value {
        deserialize_shape(shape, spec_dir, &materials, &mut shapes)?;
    }

    // Pull out any important shapes for sampling in a separate list
    let mut samples = Vec::new();
    for shape in &shapes {
        if shape.get_material().is_important() {
            samples.push(pdf::PDF::Shape(pdf::Shape::new(&shape)));
        }
    }
    let important_samples = Arc::new(pdf::PDF::Mixture(pdf::Mixture::new(samples)));

    // Break the shapes down into the aggregate structure
    let aggregate_type = match get_required_key(&top_level, "Aggregate")?.as_str() {
        Some(t) => t,
        None => {
            return Err(DeserializeError::LocalError(String::from(
                "'Aggregate' is not a string.",
            )))
        }
    };
    let shape_aggregate = create_aggregate(aggregate_type, shapes)?;

    Ok(Scene {
        logistics: logistics,
        camera: camera,
        shape_aggregate: shape_aggregate,
        important_samples: important_samples,
    })
}

// Just a helper for getting a key expected in the JSON.
fn get_required_key<'a>(
    dict: &'a serde_json::Value,
    key: &str,
) -> Result<&'a serde_json::Value, DeserializeError> {
    match dict.get(key) {
        Some(v) => Ok(v),
        None => {
            return Err(DeserializeError::LocalError(format!(
                "Required key {} is missing.",
                key
            )))
        }
    }
}

fn identify_type(dict: &serde_json::Value) -> Result<&str, DeserializeError> {
    match get_required_key(dict, "type")?.as_str() {
        Some(t) => Ok(t),
        None => {
            return Err(DeserializeError::LocalError(format!(
                "Expected 'type' key to be a string: {}",
                serde_json::to_string(dict)?
            )))
        }
    }
}

fn deserialize_texture(
    json: &serde_json::Value,
    spec_dir: &path::Path,
    res: &mut Resources,
) -> Result<Arc<SyncTexture>, DeserializeError> {
    if !json.is_object() {
        return Err(DeserializeError::LocalError(format!(
            "Expected JSON object for value in Texture map: {}",
            serde_json::to_string(json)?
        )));
    }

    let tex_type = identify_type(json)?;
    match tex_type {
        "Constant" => Ok(serde_json::from_value::<Arc<texture::Constant>>(
            serde_json::Value::clone(json),
        )?),
        "Test" => Ok(Arc::new(texture::Test)),
        "Checker" => deserialize_checker(json, spec_dir, res),
        "Image" => deserialize_image(json, spec_dir, res),
        "Noise" => Ok(serde_json::from_value::<Arc<texture::Noise>>(
            serde_json::Value::clone(json),
        )?),
        "Turbulence" => Ok(serde_json::from_value::<Arc<texture::Turbulence>>(
            serde_json::Value::clone(json),
        )?),
        _ => Err(DeserializeError::LocalError(format!(
            "Unsupported texture type: {}",
            tex_type
        ))),
    }
}

// Checker
#[derive(Deserialize)]
struct CheckerDescription {
    repeat: f32,
    odd: serde_json::Value,
    even: serde_json::Value,
}

fn deserialize_checker(
    json: &serde_json::Value,
    spec_dir: &path::Path,
    res: &mut Resources,
) -> Result<Arc<SyncTexture>, DeserializeError> {
    let checker_desc: CheckerDescription = serde_json::from_value(serde_json::Value::clone(json))?;
    return Ok(Arc::new(texture::Checker::new(
        checker_desc.repeat,
        deserialize_texture(&checker_desc.odd, spec_dir, res)?,
        deserialize_texture(&checker_desc.even, spec_dir, res)?,
    )));
}

// Image
#[derive(Deserialize)]
struct ImageDescription {
    image_path: String,
}

fn deserialize_image(
    json: &serde_json::Value,
    spec_dir: &path::Path,
    res: &mut Resources,
) -> Result<Arc<SyncTexture>, DeserializeError> {
    let image_desc: ImageDescription = serde_json::from_value(serde_json::Value::clone(json))?;
    return Ok(Arc::new(texture::Image::new(
        match res.load_image(&spec_dir.join(image_desc.image_path)) {
            Ok(i) => i,
            Err(e) => return Err(DeserializeError::LocalError(e)),
        },
    )));
}

fn deserialize_material(
    json: &serde_json::Value,
    textures: &HashMap<String, Arc<SyncTexture>>,
) -> Result<Arc<SyncMaterial>, DeserializeError> {
    if !json.is_object() {
        return Err(DeserializeError::LocalError(format!(
            "Expected JSON object for value in Materials map: {}",
            serde_json::to_string(json)?
        )));
    }

    let material_type = identify_type(json)?;
    match material_type {
        "Lambert" => deserialize_lambert(json, textures),
        "Metal" => deserialize_metal(json, textures),
        "Dielectric" => Ok(serde_json::from_value::<Arc<material::Dielectric>>(
            serde_json::Value::clone(json),
        )?),
        "DiffuseLight" => deserialize_diffuse_light(json, textures),
        "Isotropic" => deserialize_isotropic(json, textures),
        _ => Err(DeserializeError::LocalError(format!(
            "Unsupported material type: {}",
            material_type
        ))),
    }
}

// Lambert
#[derive(Deserialize)]
struct LambertDescription {
    albedo: String,
    bump_map: Option<String>,
}

fn deserialize_lambert(
    json: &serde_json::Value,
    textures: &HashMap<String, Arc<SyncTexture>>,
) -> Result<Arc<SyncMaterial>, DeserializeError> {
    let lambert_desc: LambertDescription = serde_json::from_value(serde_json::Value::clone(json))?;
    if !textures.contains_key(&lambert_desc.albedo) {
        return Err(DeserializeError::LocalError(format!(
            "Missing Texture {} for Lambert.",
            lambert_desc.albedo
        )));
    }
    let bump_map = match &lambert_desc.bump_map {
        None => None,
        Some(b) => {
            if !textures.contains_key(b) {
                return Err(DeserializeError::LocalError(format!(
                    "Missing bump map Texture {} for Lambert.",
                    b
                )));
            }
            Some(Arc::clone(&textures[b]))
        }
    };
    return Ok(Arc::new(material::Lambert::new(
        Arc::clone(&textures[&lambert_desc.albedo]),
        bump_map,
    )));
}

// Metal
#[derive(Deserialize)]
struct MetalDescription {
    albedo: String,
    roughness: f32,
    bump_map: Option<String>,
}

fn deserialize_metal(
    json: &serde_json::Value,
    textures: &HashMap<String, Arc<SyncTexture>>,
) -> Result<Arc<SyncMaterial>, DeserializeError> {
    let metal_desc: MetalDescription = serde_json::from_value(serde_json::Value::clone(json))?;
    if !textures.contains_key(&metal_desc.albedo) {
        return Err(DeserializeError::LocalError(format!(
            "Missing Texture {} for Metal.",
            metal_desc.albedo
        )));
    }
    let bump_map = match &metal_desc.bump_map {
        None => None,
        Some(b) => {
            if !textures.contains_key(b) {
                return Err(DeserializeError::LocalError(format!(
                    "Missing bump map Texture {} for Lambert.",
                    b
                )));
            }
            Some(Arc::clone(&textures[b]))
        }
    };
    return Ok(Arc::new(material::Metal::new(
        Arc::clone(&textures[&metal_desc.albedo]),
        metal_desc.roughness,
        bump_map,
    )));
}

// Diffuse Light
#[derive(Deserialize)]
struct DiffuseLightDescription {
    emission: String,
}

fn deserialize_diffuse_light(
    json: &serde_json::Value,
    textures: &HashMap<String, Arc<SyncTexture>>,
) -> Result<Arc<SyncMaterial>, DeserializeError> {
    let diffuse_desc: DiffuseLightDescription =
        serde_json::from_value(serde_json::Value::clone(json))?;
    if !textures.contains_key(&diffuse_desc.emission) {
        return Err(DeserializeError::LocalError(format!(
            "Missing Texture {} for DiffuseLight.",
            diffuse_desc.emission
        )));
    }
    return Ok(Arc::new(material::DiffuseLight::new(Arc::clone(
        &textures[&diffuse_desc.emission],
    ))));
}

// Isotropic Phase Function
#[derive(Deserialize)]
struct IsotropicDescription {
    albedo: String,
}

fn deserialize_isotropic(
    json: &serde_json::Value,
    textures: &HashMap<String, Arc<SyncTexture>>,
) -> Result<Arc<SyncMaterial>, DeserializeError> {
    let iso_desc: IsotropicDescription = serde_json::from_value(serde_json::Value::clone(json))?;
    if !textures.contains_key(&iso_desc.albedo) {
        return Err(DeserializeError::LocalError(format!(
            "Missing Texture {} for Isotropic.",
            iso_desc.albedo
        )));
    }
    return Ok(Arc::new(volume::Isotropic::new(Arc::clone(
        &textures[&iso_desc.albedo],
    ))));
}

fn deserialize_shape(
    json: &serde_json::Value,
    spec_dir: &path::Path,
    materials: &HashMap<String, Arc<SyncMaterial>>,
    shapes: &mut Vec<Arc<SyncShape>>,
) -> Result<(), DeserializeError> {
    if !json.is_object() {
        return Err(DeserializeError::LocalError(format!(
            "Expected JSON object for value in Shapes array: {}",
            serde_json::to_string(json)?
        )));
    }

    let shape_type = identify_type(json)?;
    match shape_type {
        "Sphere" => {
            shapes.push(deserialize_sphere(json, materials)?);
            Ok(())
        }
        "Mesh" => deserialize_mesh(json, spec_dir, materials, shapes),
        "ConstantMedium" => deserialize_constant_medium(json, spec_dir, materials, shapes),
        _ => {
            return Err(DeserializeError::LocalError(format!(
                "Unknown Shape 'type' {} given.",
                shape_type
            )))
        }
    }
}

// Sphere
#[derive(Deserialize)]
struct SphereDescription {
    radius: f32,
    material: String,

    #[serde(default = "Transform::new")]
    transform: Transform,
}

fn deserialize_sphere(
    json: &serde_json::Value,
    materials: &HashMap<String, Arc<SyncMaterial>>,
) -> Result<Arc<shape::Sphere>, DeserializeError> {
    let sphere_desc: SphereDescription = serde_json::from_value(serde_json::Value::clone(json))?;
    if !materials.contains_key(&sphere_desc.material) {
        return Err(DeserializeError::LocalError(format!(
            "Missing Material {} for Sphere.",
            sphere_desc.material
        )));
    }
    return Ok(Arc::new(
        match shape::Sphere::new(
            &sphere_desc.transform.create_matrix(),
            sphere_desc.radius,
            Arc::clone(&materials[&sphere_desc.material]),
        ) {
            Ok(s) => s,
            Err(e) => return Err(DeserializeError::LocalError(String::from(e))),
        },
    ));
}

// Mesh
#[derive(Deserialize)]
struct MeshDescription {
    file_path: String,
    enable_backface_culling: bool,
    material: String,

    #[serde(default = "Transform::new")]
    transform: Transform,
}

fn deserialize_mesh(
    json: &serde_json::Value,
    spec_dir: &path::Path,
    materials: &HashMap<String, Arc<SyncMaterial>>,
    shapes: &mut Vec<Arc<SyncShape>>,
) -> Result<(), DeserializeError> {
    let mesh_desc: MeshDescription = serde_json::from_value(serde_json::Value::clone(json))?;
    if !materials.contains_key(&mesh_desc.material) {
        return Err(DeserializeError::LocalError(format!(
            "Missing Material {} for Mesh.",
            mesh_desc.material
        )));
    }

    let local_to_world = mesh_desc.transform.create_matrix();

    // TODO: Proper support for OBJ material (.mtl) files.
    let obj_string = fs::read_to_string(spec_dir.join(&mesh_desc.file_path))?;
    let obj_set = obj::parse(obj_string)?;
    // Pull apart the object set read from the OBJ file.
    for object in obj_set.objects {
        // Need to convert the library's vertex struct to ours.
        let mut converted_vertices = Vec::with_capacity(object.vertices.len());
        for vert in object.vertices {
            converted_vertices.push(&local_to_world * Point3::from(vert));
        }
        // Also need to convert the texture coordinates.
        let mut converted_tex_coords = Vec::with_capacity(object.tex_vertices.len());
        for tex in object.tex_vertices {
            converted_tex_coords.push((tex.u as f32, tex.v as f32));
        }
        // Create shared mesh, which all Triangles will reference.
        let t_mesh = Arc::new(shape::TriangleMesh::new(
            converted_vertices,
            converted_tex_coords,
            mesh_desc.enable_backface_culling,
            Arc::clone(&materials[&mesh_desc.material]),
        ));

        // Geometry -> Shape -> Primitive -> Triangle -> Vertices
        for geom in object.geometry {
            for obj_shape in geom.shapes {
                match obj_shape.primitive {
                    obj::Primitive::Triangle(v0, v1, v2) => {
                        let (v_index0, t_index0, _) = v0;
                        let (v_index1, t_index1, _) = v1;
                        let (v_index2, t_index2, _) = v2;

                        shapes.push(Arc::new(
                            match shape::Triangle::new(
                                Arc::clone(&t_mesh),
                                v_index0,
                                v_index1,
                                v_index2,
                                t_index0,
                                t_index1,
                                t_index2,
                            ) {
                                Ok(t) => t,
                                Err(e) => {
                                    return Err(DeserializeError::LocalError(format!(
                                        "Error creating Triangle for file {}, object {}: {}",
                                        mesh_desc.file_path, object.name, e
                                    )))
                                }
                            },
                        ));
                    }
                    _ => {
                        return Err(DeserializeError::LocalError(format!(
                            "Only triangles are allowed in meshes, 
								but file {}, object {} had another type of primitive.",
                            mesh_desc.file_path, object.name
                        )));
                    }
                }
            }
        }
    }
    return Ok(());
}

// ConstantMedium
#[derive(Deserialize)]
struct ConstantMediumDescription {
    boundary: serde_json::Value,
    density: f32,
    phase_func: String,
}

fn deserialize_constant_medium(
    json: &serde_json::Value,
    spec_dir: &path::Path,
    materials: &HashMap<String, Arc<SyncMaterial>>,
    shapes: &mut Vec<Arc<SyncShape>>,
) -> Result<(), DeserializeError> {
    let med_desc: ConstantMediumDescription =
        serde_json::from_value(serde_json::Value::clone(json))?;
    if !materials.contains_key(&med_desc.phase_func) {
        return Err(DeserializeError::LocalError(format!(
            "Missing Phase Function {} for ConstantMedium.",
            med_desc.phase_func
        )));
    }
    let mut shapes_temp = Vec::new();
    deserialize_shape(&med_desc.boundary, spec_dir, materials, &mut shapes_temp)?;
    // TODO: For now, just single shapes are valid for boundaries
    if shapes_temp.len() != 1 {
        return Err(DeserializeError::LocalError(String::from(
            "Only single shapes are allowed for constant mediums at the moment.",
        )));
    }

    shapes.push(Arc::new(volume::ConstantMedium::new(
        shapes_temp.remove(0_usize),
        med_desc.density,
        Arc::clone(&materials[&med_desc.phase_func]),
    )));
    return Ok(());
}

// Aggregates
fn create_aggregate(
    aggregate_type: &str,
    shapes: Vec<Arc<SyncShape>>,
) -> Result<Box<SyncAggregate>, DeserializeError> {
    match aggregate_type {
        "List" => return Ok(Box::new(shapes)),
        "BVH" => return Ok(new_bvh(shapes)),
        _ => {
            return Err(DeserializeError::LocalError(format!(
                "Unknown Aggregate 'type' {} given.",
                aggregate_type
            )))
        }
    }
}
