//! Models shown for point entities: glTF (.glb, .gltf) and Blockbench (.bbmodel).

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use glam::{Mat4, Vec3};
use gt_core::{Aabb, DVec3};

#[derive(Clone, Copy, Debug)]
pub struct ModelVertex {
    pub pos: Vec3,
    pub normal: Vec3,
    pub uv: [f32; 2],
}

pub struct ModelPart {
    /// Renderer material key.
    pub material: String,
    pub vertices: Vec<ModelVertex>,
    pub indices: Vec<u32>,
}

/// A model in map units, placed at the entity origin.
pub struct Model {
    pub parts: Vec<ModelPart>,
    /// Textures to register with the renderer: (key, image, pixelated).
    pub textures: Vec<(String, image::RgbaImage, bool)>,
    pub bounds: Aabb,
}

#[derive(Default)]
pub struct ModelCache {
    entries: HashMap<PathBuf, (Option<SystemTime>, Option<Arc<Model>>)>,
    /// Bumped whenever a model is (re)loaded, so the scene rebuilds.
    pub generation: u64,
}

pub fn is_model_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".glb") || lower.ends_with(".gltf") || lower.ends_with(".bbmodel")
}

impl ModelCache {
    pub fn get(&mut self, path: &Path, units_per_meter: f64) -> Option<Arc<Model>> {
        let mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        if let Some((cached_time, model)) = self.entries.get(path)
            && *cached_time == mtime
        {
            return model.clone();
        }
        let model = load(path, units_per_meter).map(Arc::new);
        if let Err(e) = &model {
            eprintln!("model {}: {e}", path.display());
        }
        let model = model.ok();
        self.entries.insert(path.to_path_buf(), (mtime, model.clone()));
        self.generation += 1;
        model
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.generation += 1;
    }
}

fn load(path: &Path, units_per_meter: f64) -> Result<Model, String> {
    let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    match ext.as_str() {
        "bbmodel" => load_bbmodel(path, units_per_meter),
        "glb" | "gltf" => load_gltf(path, units_per_meter),
        _ => Err(format!("unsupported model format {ext}")),
    }
}

/// Blockbench uses 16 units per block, which the Godot importer maps to one meter.
pub const BB_UNITS_PER_METER: f64 = 16.0;

fn load_bbmodel(path: &Path, units_per_meter: f64) -> Result<Model, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let bb = gt_formats::bbmodel::parse(&text).map_err(|e| e.to_string())?;
    let scale = (units_per_meter / BB_UNITS_PER_METER) as f32;
    let base = format!("model:{}", path.display());
    let mut textures = Vec::new();
    for (i, t) in bb.textures.iter().enumerate() {
        let img = if !t.png.is_empty() {
            image::load_from_memory(&t.png).ok().map(|i| i.to_rgba8())
        } else {
            let p = path.parent().map(|d| d.join(&t.path)).filter(|p| p.is_file()).or_else(|| Some(PathBuf::from(&t.path)).filter(|p| p.is_file()));
            p.and_then(|p| image::open(p).ok()).map(|i| i.to_rgba8())
        };
        textures.push((format!("{base}#{i}"), img.unwrap_or_else(|| image::RgbaImage::from_pixel(2, 2, image::Rgba([200, 60, 200, 255]))), true));
    }
    let mut parts: HashMap<Option<usize>, ModelPart> = HashMap::new();
    let mut bounds = Aabb::EMPTY;
    for poly in bb.polygons() {
        let material = match poly.texture {
            Some(i) if i < textures.len() => format!("{base}#{i}"),
            _ => gt_render::WHITE_MATERIAL.to_string(),
        };
        let part = parts.entry(poly.texture).or_insert_with(|| ModelPart { material, vertices: Vec::new(), indices: Vec::new() });
        let n = gt_geom::polygon::newell(&poly.positions).normalize_or(DVec3::Y);
        let base_index = part.vertices.len() as u32;
        for (p, uv) in poly.positions.iter().zip(&poly.uvs) {
            let pos = p.as_vec3() * scale;
            bounds.include_point(pos.as_dvec3());
            part.vertices.push(ModelVertex { pos, normal: n.as_vec3(), uv: [uv.x as f32, uv.y as f32] });
        }
        for [a, b, c] in gt_geom::polygon::triangulate(&poly.positions, n) {
            part.indices.extend([base_index + a as u32, base_index + b as u32, base_index + c as u32]);
        }
    }
    Ok(Model { parts: parts.into_values().collect(), textures, bounds })
}

fn load_gltf(path: &Path, units_per_meter: f64) -> Result<Model, String> {
    let (doc, buffers, images) = gltf::import(path).map_err(|e| e.to_string())?;
    let base = format!("model:{}", path.display());
    let scale = units_per_meter as f32;
    let mut textures: Vec<(String, image::RgbaImage, bool)> = Vec::new();
    let mut image_keys: HashMap<usize, String> = HashMap::new();
    for (i, data) in images.iter().enumerate() {
        let rgba: Option<image::RgbaImage> = match data.format {
            gltf::image::Format::R8G8B8A8 => image::RgbaImage::from_raw(data.width, data.height, data.pixels.clone()),
            gltf::image::Format::R8G8B8 => {
                image::RgbImage::from_raw(data.width, data.height, data.pixels.clone()).map(|img| image::DynamicImage::ImageRgb8(img).to_rgba8())
            }
            gltf::image::Format::R8 => {
                image::GrayImage::from_raw(data.width, data.height, data.pixels.clone()).map(|img| image::DynamicImage::ImageLuma8(img).to_rgba8())
            }
            _ => None,
        };
        if let Some(img) = rgba {
            let key = format!("{base}#img{i}");
            textures.push((key.clone(), img, false));
            image_keys.insert(i, key);
        }
    }
    let mut parts: Vec<ModelPart> = Vec::new();
    let mut bounds = Aabb::EMPTY;
    let scene = doc.default_scene().or_else(|| doc.scenes().next()).ok_or("gltf has no scene")?;
    let mut stack: Vec<(gltf::Node, Mat4)> = scene.nodes().map(|n| (n, Mat4::from_scale(Vec3::splat(scale)))).collect();
    while let Some((node, parent)) = stack.pop() {
        let world = parent * Mat4::from_cols_array_2d(&node.transform().matrix());
        if let Some(mesh) = node.mesh() {
            let normal_matrix = glam::Mat3::from_mat4(world).inverse().transpose();
            for prim in mesh.primitives() {
                if prim.mode() != gltf::mesh::Mode::Triangles {
                    continue;
                }
                let reader = prim.reader(|b| Some(&buffers[b.index()]));
                let Some(positions) = reader.read_positions() else { continue };
                let positions: Vec<Vec3> = positions.map(|p| world.transform_point3(Vec3::from(p))).collect();
                let normals: Option<Vec<Vec3>> = reader.read_normals().map(|n| n.map(|n| (normal_matrix * Vec3::from(n)).normalize_or_zero()).collect());
                let uvs: Vec<[f32; 2]> = reader.read_tex_coords(0).map(|t| t.into_f32().collect()).unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);
                let indices: Vec<u32> = reader.read_indices().map(|i| i.into_u32().collect()).unwrap_or_else(|| (0..positions.len() as u32).collect());
                let pbr = prim.material().pbr_metallic_roughness();
                let material = match pbr.base_color_texture().and_then(|t| image_keys.get(&t.texture().source().index())) {
                    Some(key) => key.clone(),
                    None => {
                        let c = pbr.base_color_factor();
                        let key = format!("{base}#color{:02x}{:02x}{:02x}", (c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8);
                        if !textures.iter().any(|(k, _, _)| *k == key) {
                            let px = image::Rgba([(c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, 255]);
                            textures.push((key.clone(), image::RgbaImage::from_pixel(4, 4, px), false));
                        }
                        key
                    }
                };
                let mirrored = world.determinant() < 0.0;
                let vertices: Vec<ModelVertex> = positions
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        bounds.include_point(p.as_dvec3());
                        ModelVertex {
                            pos: *p,
                            normal: normals.as_ref().and_then(|n| n.get(i).copied()).unwrap_or(Vec3::Y),
                            uv: uvs.get(i).copied().unwrap_or([0.0, 0.0]),
                        }
                    })
                    .collect();
                let mut idx = indices;
                if mirrored {
                    for tri in idx.as_chunks_mut::<3>().0 {
                        tri.swap(1, 2);
                    }
                }
                if normals.is_none() {
                    let mut vs = vertices;
                    for tri in idx.as_chunks::<3>().0 {
                        let (a, b, c) = (vs[tri[0] as usize].pos, vs[tri[1] as usize].pos, vs[tri[2] as usize].pos);
                        let n = (b - a).cross(c - a).normalize_or_zero();
                        for k in tri {
                            vs[*k as usize].normal = n;
                        }
                    }
                    parts.push(ModelPart { material, vertices: vs, indices: idx });
                } else {
                    parts.push(ModelPart { material, vertices, indices: idx });
                }
            }
        }
        stack.extend(node.children().map(|c| (c, world)));
    }
    Ok(Model { parts, textures, bounds })
}

/// Model path of a point entity: its "model" property, or the definition's model.
pub fn entity_model_path(game: &gt_formats::GameConfig, e: &gt_doc::Entity) -> Option<PathBuf> {
    let from_prop = e.property("model").filter(|m| is_model_path(m));
    let from_def = game.entity(&e.classname).map(|d| d.model.as_str()).filter(|m| is_model_path(m));
    let path = from_prop.or(from_def)?;
    if path.starts_with("res://") { game.resolve_res(path) } else { Some(PathBuf::from(path)) }
}
