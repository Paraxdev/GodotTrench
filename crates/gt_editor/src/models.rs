//! Loading and placing of model files: glTF (.glb, .gltf), Wavefront (.obj), Blockbench (.bbmodel),
//! STL (.stl) and id Software models (.md2, .md3). Used both for point-entity previews and for the
//! Models panel, which drops any of these into the scene as an editable mesh.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use glam::{Mat4, Vec3};
use gt_core::{Aabb, DVec2, DVec3};

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
    /// Bumped only when an already loaded model is dropped or replaced, so its uploaded textures go stale.
    pub reloads: u64,
}

pub fn is_model_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    MODEL_EXTS.iter().any(|e| lower.ends_with(&format!(".{e}")))
}

/// Model file extensions the editor can load and place. FBX is intentionally excluded: it cannot be
/// loaded in pure Rust, so it would never preview here.
pub const MODEL_EXTS: [&str; 7] = ["glb", "gltf", "obj", "bbmodel", "stl", "md2", "md3"];

impl ModelCache {
    pub fn get(&mut self, path: &Path, units_per_meter: f64) -> Option<Arc<Model>> {
        let mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        match self.entries.get(path) {
            Some((cached_time, model)) if *cached_time == mtime => return model.clone(),
            Some(_) => self.reloads += 1,
            None => {}
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
        self.reloads += 1;
    }

    /// The source file's mtime as of the last load, for callers that key their own cache on it (the
    /// renderer's uploaded textures, thumbnails) without re-reading the model themselves.
    pub fn mtime(&self, path: &Path) -> Option<SystemTime> {
        self.entries.get(path).and_then(|(m, _)| *m)
    }

    /// The model loaded earlier for `path`, without touching the file system.
    pub fn peek(&self, path: &Path) -> Option<Arc<Model>> {
        self.entries.get(path).and_then(|(_, m)| m.clone())
    }
}

fn load(path: &Path, units_per_meter: f64) -> Result<Model, String> {
    let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
    match ext.as_str() {
        "bbmodel" => load_bbmodel(path, units_per_meter),
        "glb" | "gltf" => load_gltf(path, units_per_meter),
        "obj" => load_obj(path, units_per_meter),
        "stl" => load_stl(path, units_per_meter),
        "md2" => {
            let data = std::fs::read(path).map_err(|e| e.to_string())?;
            let mesh = gt_formats::idmodel::parse_md2(&data).map_err(|e| e.to_string())?;
            Ok(id_model_to_model(path, mesh, units_per_meter))
        }
        "md3" => {
            let data = std::fs::read(path).map_err(|e| e.to_string())?;
            let mesh = gt_formats::idmodel::parse_md3(&data).map_err(|e| e.to_string())?;
            Ok(id_model_to_model(path, mesh, units_per_meter))
        }
        _ => Err(format!("unsupported model format {ext}")),
    }
}

/// STL stores raw triangles with no UVs or materials, so the whole model is one untextured part with
/// per-face normals. STL is unitless, its numbers are read as metres to match OBJ and glTF.
fn load_stl(path: &Path, units_per_meter: f64) -> Result<Model, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let scale = units_per_meter as f32;
    let tris = parse_stl(&data)?;
    if tris.is_empty() {
        return Err("stl has no triangles".into());
    }

    let mut vertices = Vec::with_capacity(tris.len() * 3);
    let mut indices = Vec::with_capacity(tris.len() * 3);
    let mut bounds = Aabb::EMPTY;
    for tri in tris {
        let ps = [Vec3::from(tri[0]) * scale, Vec3::from(tri[1]) * scale, Vec3::from(tri[2]) * scale];
        let n = (ps[1] - ps[0]).cross(ps[2] - ps[0]).normalize_or(Vec3::Y);
        for p in ps {
            bounds.include_point(p.as_dvec3());
            indices.push(vertices.len() as u32);
            vertices.push(ModelVertex { pos: p, normal: n, uv: [0.0, 0.0] });
        }
    }

    let part = ModelPart { material: gt_render::WHITE_MATERIAL.to_string(), vertices, indices };
    Ok(Model { parts: vec![part], textures: Vec::new(), bounds })
}

/// Triangles of an STL file, ascii or binary. Binary is detected by the exact size the triangle
/// count in the header implies, since binary files can also start with the ascii keyword "solid".
fn parse_stl(data: &[u8]) -> Result<Vec<[[f32; 3]; 3]>, String> {
    if data.len() >= 84 {
        let count = u32::from_le_bytes([data[80], data[81], data[82], data[83]]) as usize;
        if data.len() == 84 + count * 50 {
            let mut tris = Vec::with_capacity(count);
            let f = |b: &[u8]| f32::from_le_bytes([b[0], b[1], b[2], b[3]]);
            for i in 0..count {
                let o = 84 + i * 50 + 12; // skip the per-facet normal, it is recomputed
                let v = |k: usize| [f(&data[o + k * 12..]), f(&data[o + k * 12 + 4..]), f(&data[o + k * 12 + 8..])];
                tris.push([v(0), v(1), v(2)]);
            }

            return Ok(tris);
        }
    }

    let text = std::str::from_utf8(data).map_err(|_| "stl is neither valid binary nor ascii".to_string())?;
    let mut tris = Vec::new();
    let mut verts: Vec<[f32; 3]> = Vec::new();
    for token in text.split_whitespace().collect::<Vec<_>>().windows(4) {
        if token[0] == "vertex" {
            let p = [token[1].parse().unwrap_or(0.0), token[2].parse().unwrap_or(0.0), token[3].parse().unwrap_or(0.0)];
            verts.push(p);
            if verts.len() == 3 {
                tris.push([verts[0], verts[1], verts[2]]);
                verts.clear();
            }
        }
    }

    Ok(tris)
}

/// Converts a parsed id Software model (md2/md3) into a placeable model, saving the skin as a texture
/// when the file names one that sits next to it.
fn id_model_to_model(path: &Path, mesh: gt_formats::idmodel::IdModel, units_per_meter: f64) -> Model {
    let base = format!("model:{}", path.display());
    let scale = units_per_meter as f32;
    let mut textures = Vec::new();
    let mut parts = Vec::new();
    let mut bounds = Aabb::EMPTY;
    for (si, surf) in mesh.surfaces.iter().enumerate() {
        let material = surf
            .skin
            .as_deref()
            .and_then(|skin| skin_image(path, skin))
            .map(|img| {
                let key = format!("{base}#skin{si}");
                textures.push((key.clone(), img, false));
                key
            })
            .unwrap_or_else(|| gt_render::WHITE_MATERIAL.to_string());
        let mut vertices = Vec::with_capacity(surf.vertices.len());
        for v in &surf.vertices {
            let pos = Vec3::from(v.pos) * scale;
            bounds.include_point(pos.as_dvec3());
            vertices.push(ModelVertex { pos, normal: Vec3::from(v.normal).normalize_or(Vec3::Y), uv: v.uv });
        }

        parts.push(ModelPart { material, vertices, indices: surf.indices.clone() });
    }

    Model { parts, textures, bounds }
}

/// Loads an md2/md3 skin: the path the model names, tried as given and relative to the model's folder.
fn skin_image(model: &Path, skin: &str) -> Option<image::RgbaImage> {
    let skin = skin.trim_matches(char::from(0));
    if skin.is_empty() {
        return None;
    }

    let candidates = [PathBuf::from(skin), model.parent().map(|d| d.join(skin)).unwrap_or_default()];
    let try_exts = |p: &Path| image::open(p).ok().or_else(|| ["png", "tga", "jpg", "jpeg", "bmp"].iter().find_map(|e| image::open(p.with_extension(e)).ok()));
    candidates.iter().filter(|p| !p.as_os_str().is_empty()).find_map(|p| try_exts(p)).map(|i| i.to_rgba8())
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

fn load_obj(path: &Path, units_per_meter: f64) -> Result<Model, String> {
    let opts = tobj::LoadOptions { triangulate: true, single_index: true, ..Default::default() };
    let (models, mats_res) = tobj::load_obj(path, &opts).map_err(|e| e.to_string())?;
    let mats = mats_res.unwrap_or_default();
    let base = format!("model:{}", path.display());
    let scale = units_per_meter as f32;
    let dir = path.parent();
    let mut textures: Vec<(String, image::RgbaImage, bool)> = Vec::new();
    let mut mat_keys: Vec<String> = Vec::with_capacity(mats.len());
    for (i, m) in mats.iter().enumerate() {
        let key = format!("{base}#mat{i}");
        let from_file = m
            .diffuse_texture
            .as_ref()
            .filter(|t| !t.is_empty())
            .and_then(|t| dir.map(|d| d.join(t)))
            .and_then(|p| image::open(p).ok())
            .map(|img| img.to_rgba8());
        let img = from_file.unwrap_or_else(|| {
            let c = m.diffuse.unwrap_or([0.8, 0.8, 0.8]);
            image::RgbaImage::from_pixel(4, 4, image::Rgba([(c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, 255]))
        });
        textures.push((key.clone(), img, false));
        mat_keys.push(key);
    }

    let mut parts = Vec::new();
    let mut bounds = Aabb::EMPTY;
    for model in &models {
        let mesh = &model.mesh;
        if mesh.positions.is_empty() {
            continue;
        }

        let material = mesh.material_id.and_then(|id| mat_keys.get(id).cloned()).unwrap_or_else(|| gt_render::WHITE_MATERIAL.to_string());
        let count = mesh.positions.len() / 3;
        let mut vertices = Vec::with_capacity(count);
        for i in 0..count {
            let pos = Vec3::new(mesh.positions[3 * i], mesh.positions[3 * i + 1], mesh.positions[3 * i + 2]) * scale;
            bounds.include_point(pos.as_dvec3());
            let normal = if mesh.normals.len() >= 3 * i + 3 {
                Vec3::new(mesh.normals[3 * i], mesh.normals[3 * i + 1], mesh.normals[3 * i + 2]).normalize_or(Vec3::Y)
            } else {
                Vec3::Y
            };
            // OBJ texture coordinates use a bottom-left origin, images a top-left one.
            let uv = if mesh.texcoords.len() >= 2 * i + 2 { [mesh.texcoords[2 * i], 1.0 - mesh.texcoords[2 * i + 1]] } else { [0.0, 0.0] };
            vertices.push(ModelVertex { pos, normal, uv });
        }

        parts.push(ModelPart { material, vertices, indices: mesh.indices.clone() });
    }

    if parts.is_empty() {
        return Err("obj has no triangles".into());
    }

    Ok(Model { parts, textures, bounds })
}

/// Uniform scale to place a model at, from its natural (unit-scaled) bounds and the import prefs. Auto-fit
/// rescales pathologically small or large models to a usable size; the manual multiplier always applies.
pub fn placement_scale(bounds: &Aabb, units_per_meter: f64, autofit: bool, manual: f64) -> f64 {
    let dim = bounds.size().max_element();
    let fit = if autofit && dim > 1e-6 {
        let (target, min, max) = (units_per_meter * 2.0, units_per_meter * 0.25, units_per_meter * 256.0);
        if dim < min || dim > max { target / dim } else { 1.0 }
    } else {
        1.0
    };
    (fit * manual.max(0.0)).max(1e-4)
}

/// Converts a loaded model into an editable polygon mesh, welding coincident vertices and keeping the
/// model's UVs. `scale` sizes it about its origin. `material_of` maps each part's renderer material key to
/// a material name in the library.
pub fn model_to_mesh(model: &Model, offset: DVec3, scale: f64, material_of: impl Fn(&str) -> String) -> gt_geom::Mesh {
    use gt_geom::{FaceData, FaceUv, Mesh, MeshFace};
    let mut mesh = Mesh::default();
    let mut lookup: HashMap<(i64, i64, i64), u32> = HashMap::new();
    for part in &model.parts {
        let material = material_of(&part.material);
        for tri in part.indices.as_chunks::<3>().0 {
            let corners = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
            if corners.iter().any(|&c| c >= part.vertices.len()) {
                continue;
            }

            let mut indices = Vec::with_capacity(3);
            let mut positions = Vec::with_capacity(3);
            for &c in &corners {
                let p = part.vertices[c].pos.as_dvec3() * scale + offset;
                positions.push(p);
                let key = ((p.x * 1e4).round() as i64, (p.y * 1e4).round() as i64, (p.z * 1e4).round() as i64);
                let idx = *lookup.entry(key).or_insert_with(|| {
                    mesh.vertices.push(p);
                    (mesh.vertices.len() - 1) as u32
                });
                indices.push(idx);
            }

            let unique: std::collections::BTreeSet<u32> = indices.iter().copied().collect();
            if unique.len() < 3 {
                continue;
            }

            let normal = gt_geom::polygon::newell(&positions).normalize_or(DVec3::Y);
            let mut face = MeshFace::new(indices, FaceData::new(material.clone(), FaceUv::paraxial(normal, DVec2::ONE)));
            face.uvs = corners.iter().map(|&c| part.vertices[c].uv).collect();
            mesh.faces.push(face);
        }
    }

    // Models come smooth shaded; a moderate angle keeps rounded surfaces without over-smoothing hard edges.
    mesh.smooth_angle = 45.0;
    mesh
}

#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct PackCredit {
    pub asset: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub url: Option<String>,
}

/// A model pack's provenance, read from a `pack.json` in a model folder, or synthesized from the folder
/// name when none exists. Every `ModelEntry` carries one, shared with an `Arc` so a whole pack's models
/// clone cheaply.
#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct PackInfo {
    pub name: String,
    /// A short display name for tight spaces like a thumbnail slug, e.g. "Poly Haven" for a pack whose
    /// full `name` is longer, or a name sharing a prefix with every other pack ("GodotTrench low poly").
    /// Falls back to an abbreviation of `name` when absent.
    pub short: Option<String>,
    pub author: Option<String>,
    pub license: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub credits: Vec<PackCredit>,
}

impl PackInfo {
    /// Per-model author/url override, matched by the model file's stem (its name without extension).
    fn credit_for(&self, stem: &str) -> Option<&PackCredit> {
        self.credits.iter().find(|c| c.asset == stem)
    }
}

#[derive(Clone, Debug)]
pub struct ModelEntry {
    /// Path relative to the models root without extension, shown in the panel.
    pub name: String,
    pub folder: String,
    pub path: PathBuf,
    pub ext: String,
    pub source: Arc<PackInfo>,
    pub credit: Option<PackCredit>,
    pub mtime: Option<SystemTime>,
}

/// The placeable models under `res://models`, listed in the Models panel.
#[derive(Default)]
pub struct ModelLibrary {
    pub entries: Vec<ModelEntry>,
    pub root: Option<PathBuf>,
}

impl ModelLibrary {
    pub fn new(game: &gt_formats::GameConfig) -> Self {
        let mut lib = Self::default();
        lib.rescan(game);
        lib
    }

    pub fn rescan(&mut self, game: &gt_formats::GameConfig) {
        self.entries.clear();
        let mut packs = HashMap::new();
        self.root = game.resolve_res("res://models").filter(|p| p.is_dir());
        if let Some(root) = self.root.clone() {
            scan_models(&root, &root, "", "Project", &mut packs, &mut self.entries);
        }

        // The nature models the scatter presets install live outside res://models. They are listed under a
        // "nature" folder too, so trees and rocks can be placed by hand as props, not only scattered.
        if let Some(nature) = game.resolve_res(gt_doc::scatter::NATURE_DIR).filter(|p| p.is_dir()) {
            scan_models(&nature, &nature, "nature", "Nature", &mut packs, &mut self.entries);
        }

        self.entries.sort_by(|a, b| a.name.cmp(&b.name));
        self.entries.dedup_by(|a, b| a.path == b.path);
    }

    pub fn folders(&self) -> Vec<String> {
        let mut f: Vec<String> = self.entries.iter().map(|e| e.folder.clone()).collect();
        f.sort();
        f.dedup();
        f
    }

    pub fn sources(&self) -> Vec<String> {
        let mut s: Vec<String> = self.entries.iter().map(|e| e.source.name.clone()).collect();
        s.sort();
        s.dedup();
        s
    }
}

/// Collects the models under `dir`. `prefix` is put in front of their names and folders, for roots that are
/// not the models root itself. `root_fallback` names the pack a loose file at the scan root falls back to
/// when nothing provides a `pack.json`.
fn scan_models(root: &Path, dir: &Path, prefix: &str, root_fallback: &str, packs: &mut HashMap<PathBuf, Option<Arc<PackInfo>>>, out: &mut Vec<ModelEntry>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let source = resolve_pack(root, dir, packs).unwrap_or_else(|| Arc::new(fallback_pack(root, dir, root_fallback)));
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_models(root, &path, prefix, root_fallback, packs, out);
            continue;
        }

        let Some(ext) = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()) else { continue };
        if !MODEL_EXTS.contains(&ext.as_str()) {
            continue;
        }

        let Ok(rel) = path.strip_prefix(root) else { continue };
        let join = |part: String| if prefix.is_empty() || part.is_empty() { format!("{prefix}{part}") } else { format!("{prefix}/{part}") };
        let name = join(rel.with_extension("").to_string_lossy().replace('\\', "/"));
        let folder = join(rel.parent().map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_default());
        let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let credit = source.credit_for(&stem).cloned();
        let mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        out.push(ModelEntry { name, folder, path: path.clone(), ext, source: source.clone(), credit, mtime });
    }
}

/// Finds the nearest `pack.json` from `dir` up to (and including) `root`, memoized per directory since a
/// folder with many models would otherwise re-read and re-parse the same file for each of them.
fn resolve_pack(root: &Path, dir: &Path, cache: &mut HashMap<PathBuf, Option<Arc<PackInfo>>>) -> Option<Arc<PackInfo>> {
    if let Some(hit) = cache.get(dir) {
        return hit.clone();
    }

    let here = std::fs::read_to_string(dir.join("pack.json")).ok().and_then(|s| serde_json::from_str::<PackInfo>(&s).ok()).map(Arc::new);
    let result = here.or_else(|| if dir == root { None } else { dir.parent().and_then(|p| resolve_pack(root, p, cache)) });
    cache.insert(dir.to_path_buf(), result.clone());
    result
}

/// A synthesized pack for a folder with no `pack.json` anywhere from it up to the scan root: the top
/// folder under the root, or `root_fallback` for files loose in the root itself.
fn fallback_pack(root: &Path, dir: &Path, root_fallback: &str) -> PackInfo {
    let rel = dir.strip_prefix(root).ok().filter(|p| !p.as_os_str().is_empty());
    let name = match rel.and_then(|p| p.components().next()) {
        Some(c) => c.as_os_str().to_string_lossy().into_owned(),
        None => root_fallback.to_string(),
    };
    PackInfo { name, ..Default::default() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aabb(dim: f64) -> Aabb {
        Aabb::new(DVec3::ZERO, DVec3::splat(dim))
    }

    #[test]
    fn shipped_nature_gltf_assets_load() {
        // The scatter presets reference these, so a broken export must fail here, not silently scatter nothing.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot/godottrench/nature");
        let mut count = 0;
        for dir in ["trees", "trees_detailed", "bushes"] {
            for entry in std::fs::read_dir(root.join(dir)).unwrap_or_else(|e| panic!("{dir}: {e}")) {
                let path = entry.unwrap().path();
                if path.extension().is_none_or(|e| e != "glb") {
                    continue;
                }

                let rel = path.display().to_string();
                let model = load(&path, 16.0).unwrap_or_else(|e| panic!("{rel}: {e}"));
                assert!(model.parts.iter().any(|p| !p.indices.is_empty()), "{rel} has no geometry");
                // The textures live in the shared nature/textures folder, referenced by relative URI.
                assert!(model.parts.iter().all(|p| p.material.contains("#img")), "{rel} lost a texture");
                count += 1;
            }
        }

        assert!(count >= 30, "only {count} nature glTF models");
    }

    #[test]
    fn nature_models_are_listed_for_placing_by_hand() {
        // The demo project ships the nature pack, which lives outside res://models.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot");
        let game = gt_formats::GameConfig { project_root: Some(root), ..Default::default() };
        let lib = ModelLibrary::new(&game);
        let names: Vec<&str> = lib.entries.iter().map(|e| e.name.as_str()).collect();
        // The pack has loose .bbmodel props at its root and the procedural glTF trees in a subfolder, and
        // both keep their place under the "nature" prefix.
        assert!(names.contains(&"nature/pine"), "loose nature props are listed: {names:?}");
        let tree = lib.entries.iter().find(|e| e.name == "nature/trees/pine").unwrap_or_else(|| panic!("no nature/trees/pine among {names:?}"));
        assert_eq!(tree.folder, "nature/trees", "subfolders keep their place under the prefix");
        assert!(tree.path.is_file(), "the entry points at the real file so it can be placed");
        assert!(lib.folders().contains(&"nature/trees".to_string()));
    }

    /// Builds `root/pack.json` plus a `packA` subfolder with its own `pack.json`, a nested `sub` folder
    /// under it with no `pack.json` of its own, and a loose file at the root.
    fn pack_fixture(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("gt_models_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("packA/sub")).unwrap();
        std::fs::write(root.join("pack.json"), r#"{"name": "Root Pack", "license": "CC0"}"#).unwrap();
        std::fs::write(
            root.join("packA/pack.json"),
            r#"{"name": "Pack A", "author": "Alice", "license": "CC-BY", "url": "https://a.example",
                "credits": [{"asset": "widget", "author": "Bob", "url": "https://a.example/widget"}]}"#,
        )
        .unwrap();
        std::fs::write(root.join("loose.glb"), "").unwrap();
        std::fs::write(root.join("packA/widget.glb"), "").unwrap();
        std::fs::write(root.join("packA/sub/gadget.glb"), "").unwrap();
        root
    }

    #[test]
    fn nearest_ancestor_pack_json_wins() {
        let root = pack_fixture("nearest");
        let mut packs = HashMap::new();
        let mut out = Vec::new();
        scan_models(&root, &root, "", "Project", &mut packs, &mut out);

        let loose = out.iter().find(|e| e.name == "loose").unwrap();
        assert_eq!(loose.source.name, "Root Pack", "the root's own pack.json applies to loose files");

        let widget = out.iter().find(|e| e.name == "packA/widget").unwrap();
        assert_eq!(widget.source.name, "Pack A", "packA's pack.json overrides the root's");
        assert_eq!(widget.source.author.as_deref(), Some("Alice"));

        let gadget = out.iter().find(|e| e.name == "packA/sub/gadget").unwrap();
        assert_eq!(gadget.source.name, "Pack A", "sub has no pack.json of its own, so packA's nearer one wins over root's");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn credits_match_by_asset_stem() {
        let root = pack_fixture("credits");
        let mut packs = HashMap::new();
        let mut out = Vec::new();
        scan_models(&root, &root, "", "Project", &mut packs, &mut out);

        let widget = out.iter().find(|e| e.name == "packA/widget").unwrap();
        let credit = widget.credit.as_ref().unwrap_or_else(|| panic!("widget's asset stem matches the pack's credits entry"));
        assert_eq!(credit.author.as_deref(), Some("Bob"));
        assert_eq!(credit.url.as_deref(), Some("https://a.example/widget"));

        let gadget = out.iter().find(|e| e.name == "packA/sub/gadget").unwrap();
        assert!(gadget.credit.is_none(), "gadget has no matching credits entry");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn no_pack_json_falls_back_to_folder_name_or_root_fallback() {
        let root = std::env::temp_dir().join(format!("gt_models_fallback_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("crates")).unwrap();
        std::fs::write(root.join("loose.glb"), "").unwrap();
        std::fs::write(root.join("crates/wood.glb"), "").unwrap();

        let mut packs = HashMap::new();
        let mut out = Vec::new();
        scan_models(&root, &root, "", "Project", &mut packs, &mut out);

        let loose = out.iter().find(|e| e.name == "loose").unwrap();
        assert_eq!(loose.source.name, "Project", "loose files at the scan root fall back to root_fallback");
        let wood = out.iter().find(|e| e.name == "crates/wood").unwrap();
        assert_eq!(wood.source.name, "crates", "a subfolder with no pack.json falls back to its own name");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn library_sources_are_sorted_and_deduped() {
        let root = pack_fixture("sources");
        let mut lib = ModelLibrary { root: Some(root.clone()), ..Default::default() };
        let mut packs = HashMap::new();
        scan_models(&root, &root, "", "Project", &mut packs, &mut lib.entries);

        assert_eq!(lib.sources(), vec!["Pack A".to_string(), "Root Pack".to_string()]);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn autofit_grows_tiny_metric_models() {
        // A 2.4u toy car at 32 units/metre fits to the 2m (64u) target.
        let s = placement_scale(&aabb(2.4), 32.0, true, 1.0);
        assert!((2.4 * s - 64.0).abs() < 1e-6, "scaled dim {}", 2.4 * s);
    }

    #[test]
    fn autofit_leaves_level_scale_models_alone() {
        assert_eq!(placement_scale(&aabb(48.0), 32.0, true, 1.0), 1.0);
    }

    #[test]
    fn autofit_shrinks_huge_models() {
        let s = placement_scale(&aabb(20000.0), 32.0, true, 1.0);
        assert!((20000.0 * s - 64.0).abs() < 1e-6);
    }

    #[test]
    fn manual_multiplier_stacks_on_fit_and_overrides_when_off() {
        // Fit (26.67x) times the manual 2x.
        let both = placement_scale(&aabb(2.4), 32.0, true, 2.0);
        assert!((2.4 * both - 128.0).abs() < 1e-6);
        // With fit off, only the manual multiplier applies to the real size.
        assert_eq!(placement_scale(&aabb(2.4), 32.0, false, 3.0), 3.0);
    }
}
