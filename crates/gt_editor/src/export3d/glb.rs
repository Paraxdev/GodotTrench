//! glTF 2.0 binary container: the JSON document and one buffer holding the geometry and the images.

use std::collections::{BTreeSet, HashMap};
use std::io::Write;
use std::path::Path;

use serde_json::{Value, json};

use super::collect::Scene;
use super::size_text;
use super::textures::Alpha;

const GLB_MAGIC: u32 = 0x4654_6C67;
const CHUNK_JSON: u32 = 0x4E4F_534A;
const CHUNK_BIN: u32 = 0x004E_4942;
const ARRAY_BUFFER: u32 = 34962;
const ELEMENT_ARRAY_BUFFER: u32 = 34963;
const FLOAT: u32 = 5126;
const UNSIGNED_SHORT: u32 = 5123;
const UNSIGNED_INT: u32 = 5125;
const NEAREST: u32 = 9728;
const LINEAR: u32 = 9729;
const LINEAR_MIPMAP_LINEAR: u32 = 9987;
const NEAREST_MIPMAP_LINEAR: u32 = 9986;
const EMISSIVE_STRENGTH: &str = "KHR_materials_emissive_strength";
const UNLIT: &str = "KHR_materials_unlit";

#[derive(Default)]
struct Buffer {
    bin: Vec<u8>,
    views: Vec<Value>,
    accessors: Vec<Value>,
}

impl Buffer {
    fn view(&mut self, data: &[u8], target: Option<u32>) -> usize {
        while !self.bin.len().is_multiple_of(4) {
            self.bin.push(0);
        }

        let mut view = json!({ "buffer": 0, "byteOffset": self.bin.len(), "byteLength": data.len() });
        if let Some(t) = target {
            view["target"] = json!(t);
        }

        self.bin.extend_from_slice(data);
        self.views.push(view);
        self.views.len() - 1
    }

    fn floats<const N: usize>(&mut self, items: &[[f32; N]], bounds: bool) -> usize {
        let bytes: Vec<u8> = items.iter().flatten().flat_map(|v| v.to_le_bytes()).collect();
        let view = self.view(&bytes, Some(ARRAY_BUFFER));
        let kind = match N {
            2 => "VEC2",
            3 => "VEC3",
            _ => "VEC4",
        };
        let mut accessor = json!({ "bufferView": view, "componentType": FLOAT, "count": items.len(), "type": kind });
        if bounds {
            let mut min = [f32::INFINITY; N];
            let mut max = [f32::NEG_INFINITY; N];
            for item in items {
                for k in 0..N {
                    min[k] = min[k].min(item[k]);
                    max[k] = max[k].max(item[k]);
                }
            }

            accessor["min"] = json!(min.to_vec());
            accessor["max"] = json!(max.to_vec());
        }

        self.accessors.push(accessor);
        self.accessors.len() - 1
    }

    fn indices(&mut self, indices: &[u32], vertex_count: usize) -> usize {
        let (bytes, component): (Vec<u8>, u32) = if vertex_count <= u16::MAX as usize {
            (indices.iter().flat_map(|i| (*i as u16).to_le_bytes()).collect(), UNSIGNED_SHORT)
        } else {
            (indices.iter().flat_map(|i| i.to_le_bytes()).collect(), UNSIGNED_INT)
        };
        let view = self.view(&bytes, Some(ELEMENT_ARRAY_BUFFER));
        self.accessors.push(json!({ "bufferView": view, "componentType": component, "count": indices.len(), "type": "SCALAR" }));
        self.accessors.len() - 1
    }
}

fn is_zero(v: &[f64]) -> bool {
    v.iter().all(|x| *x == 0.0)
}

/// The header keeps the file length in 32 bits.
const MAX_FILE: u64 = u32::MAX as u64;

fn too_big(bytes: u64) -> String {
    format!(
        "the .glb would be {}, more than the 4 GB a .glb file can hold. Export part of the map with Selection only, or leave \
         out the prop models or scatter instances",
        size_text(bytes)
    )
}

/// The size of the binary chunk, as [`Buffer`] lays it out.
pub(super) fn buffer_size(scene: &Scene) -> u64 {
    let padded = |n: usize| (n as u64).next_multiple_of(4);
    let mut size = 0;
    for p in scene.meshes.iter().flat_map(|m| &m.primitives).filter(|p| !p.indices.is_empty()) {
        let index = if p.positions.len() <= u16::MAX as usize { 2 } else { 4 };
        size += padded(p.positions.len() * 12) + padded(p.normals.len() * 12) + padded(p.uvs.len() * 8) + padded(p.indices.len() * index);
    }

    let used: BTreeSet<usize> = scene
        .materials
        .list
        .iter()
        .flat_map(|m| [m.albedo, m.metal_rough, m.normal, m.occlusion, m.emissive_texture.filter(|_| m.is_emissive())])
        .flatten()
        .collect();
    size + used.iter().map(|i| padded(scene.materials.images[*i].bytes.len())).sum::<u64>()
}

/// Writes the scene as a `.glb` file to `path`. Returns the bytes written.
pub fn write(scene: &Scene, path: &Path) -> Result<u64, String> {
    // Refused before the buffer is built, a file past the limit would take as much memory.
    let estimate = 28 + buffer_size(scene);
    if estimate > MAX_FILE {
        return Err(too_big(estimate));
    }

    let mut buf = Buffer::default();
    let mut extensions: Vec<&str> = Vec::new();
    let mut meshes: Vec<Value> = Vec::with_capacity(scene.meshes.len());
    for mesh in &scene.meshes {
        let primitives: Vec<Value> = mesh
            .primitives
            .iter()
            .filter(|p| !p.indices.is_empty())
            .map(|p| {
                let position = buf.floats(&p.positions, true);
                let normal = buf.floats(&p.normals, false);
                let uv = buf.floats(&p.uvs, false);
                let indices = buf.indices(&p.indices, p.positions.len());
                json!({
                    "attributes": { "POSITION": position, "NORMAL": normal, "TEXCOORD_0": uv },
                    "indices": indices,
                    "material": p.material,
                    "mode": 4,
                })
            })
            .collect();
        meshes.push(json!({ "name": mesh.name, "primitives": primitives }));
    }

    // Images go into the buffer only when a glTF texture uses them, and each image gets one texture per filter.
    let mut images: Vec<Value> = Vec::new();
    let mut image_index: HashMap<usize, usize> = HashMap::new();
    let mut textures: Vec<Value> = Vec::new();
    let mut texture_index: HashMap<(usize, bool), usize> = HashMap::new();
    let mut samplers: Vec<Value> = Vec::new();
    let mut sampler_index: HashMap<bool, usize> = HashMap::new();
    let mut texture = |image: usize, nearest: bool| -> usize {
        let sampler = *sampler_index.entry(nearest).or_insert_with(|| {
            samplers.push(if nearest {
                json!({ "magFilter": NEAREST, "minFilter": NEAREST_MIPMAP_LINEAR })
            } else {
                json!({ "magFilter": LINEAR, "minFilter": LINEAR_MIPMAP_LINEAR })
            });
            samplers.len() - 1
        });
        let source = *image_index.entry(image).or_insert_with(|| {
            let img = &scene.materials.images[image];
            let view = buf.view(&img.bytes, None);
            images.push(json!({ "name": img.name, "bufferView": view, "mimeType": img.mime }));
            images.len() - 1
        });
        *texture_index.entry((image, nearest)).or_insert_with(|| {
            textures.push(json!({ "source": source, "sampler": sampler }));
            textures.len() - 1
        })
    };

    let materials: Vec<Value> = scene
        .materials
        .list
        .iter()
        .map(|m| {
            let mut pbr = json!({ "baseColorFactor": m.base_color, "metallicFactor": m.metallic, "roughnessFactor": m.roughness });
            if let Some(i) = m.albedo {
                pbr["baseColorTexture"] = json!({ "index": texture(i, m.nearest) });
            }

            if let Some(i) = m.metal_rough {
                pbr["metallicRoughnessTexture"] = json!({ "index": texture(i, m.nearest) });
            }

            let mut out = json!({ "name": m.name, "pbrMetallicRoughness": pbr });
            if let Some(i) = m.normal {
                out["normalTexture"] = json!({ "index": texture(i, m.nearest), "scale": m.normal_scale });
            }

            if let Some(i) = m.occlusion {
                out["occlusionTexture"] = json!({ "index": texture(i, m.nearest) });
            }

            let mut ext = serde_json::Map::new();
            if m.is_emissive() {
                // glTF keeps the factor within 0 to 1 and carries brighter glows in the strength extension.
                let peak = m.emissive.iter().copied().fold(1.0f32, f32::max);
                let strength = m.emissive_strength * peak;
                let factor = m.emissive.map(|c| c / peak * strength.min(1.0));
                out["emissiveFactor"] = json!(factor);
                if let Some(i) = m.emissive_texture {
                    out["emissiveTexture"] = json!({ "index": texture(i, m.nearest) });
                }

                if strength > 1.0 {
                    ext.insert(EMISSIVE_STRENGTH.into(), json!({ "emissiveStrength": strength }));
                }
            }

            if m.unlit {
                ext.insert(UNLIT.into(), json!({}));
            }

            for name in [EMISSIVE_STRENGTH, UNLIT] {
                if ext.contains_key(name) && !extensions.contains(&name) {
                    extensions.push(name);
                }
            }

            if !ext.is_empty() {
                out["extensions"] = Value::Object(ext);
            }

            match m.alpha {
                Alpha::Opaque => {}
                Alpha::Mask(cutoff) => {
                    out["alphaMode"] = json!("MASK");
                    out["alphaCutoff"] = json!(cutoff);
                }
                Alpha::Blend => out["alphaMode"] = json!("BLEND"),
            }

            if m.double_sided {
                out["doubleSided"] = json!(true);
            }

            out
        })
        .collect();

    let nodes: Vec<Value> = scene
        .nodes
        .iter()
        .map(|n| {
            let mut out = json!({ "name": n.name });
            let t = n.translation.to_array();
            if !is_zero(&t) {
                out["translation"] = json!(t);
            }

            if n.rotation != gt_core::DQuat::IDENTITY {
                out["rotation"] = json!(n.rotation.normalize().to_array());
            }

            if n.scale != gt_core::DVec3::ONE {
                out["scale"] = json!(n.scale.to_array());
            }

            if let Some(m) = n.mesh {
                out["mesh"] = json!(m);
            }

            if !n.children.is_empty() {
                out["children"] = json!(n.children);
            }

            out
        })
        .collect();

    let mut doc = json!({
        "asset": { "version": "2.0", "generator": format!("GodotTrench {}", crate::VERSION) },
        "scene": 0,
        "scenes": [{ "name": scene.name, "nodes": scene.roots }],
        "nodes": nodes,
    });
    let mut set = |key: &str, list: Vec<Value>| {
        if !list.is_empty() {
            doc[key] = Value::Array(list);
        }
    };
    set("meshes", meshes);
    set("materials", materials);
    set("samplers", samplers);
    set("textures", textures);
    set("images", images);
    if !buf.bin.is_empty() {
        while !buf.bin.len().is_multiple_of(4) {
            buf.bin.push(0);
        }

        set("accessors", std::mem::take(&mut buf.accessors));
        set("bufferViews", std::mem::take(&mut buf.views));
        set("buffers", vec![json!({ "byteLength": buf.bin.len() })]);
    }

    if !extensions.is_empty() {
        doc["extensionsUsed"] = json!(extensions);
    }

    let json = serde_json::to_vec(&doc).unwrap_or_default();
    let total = container_len(json.len(), buf.bin.len());
    if total > MAX_FILE {
        return Err(too_big(total));
    }

    gt_formats::write_atomic_with(path, |out| write_container(out, &json, &buf.bin)).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    Ok(total)
}

fn container_len(json: usize, bin: usize) -> u64 {
    let json = (json as u64).next_multiple_of(4);
    let bin = (bin as u64).next_multiple_of(4);
    12 + 8 + json + if bin == 0 { 0 } else { 8 + bin }
}

fn write_container(out: &mut impl Write, json: &[u8], bin: &[u8]) -> std::io::Result<()> {
    let total = container_len(json.len(), bin.len());
    let json_len = json.len().next_multiple_of(4);
    for word in [GLB_MAGIC, 2, total as u32, json_len as u32, CHUNK_JSON] {
        out.write_all(&word.to_le_bytes())?;
    }

    out.write_all(json)?;
    out.write_all(&b"   "[..json_len - json.len()])?;
    if !bin.is_empty() {
        let bin_len = bin.len().next_multiple_of(4);
        out.write_all(&(bin_len as u32).to_le_bytes())?;
        out.write_all(&CHUNK_BIN.to_le_bytes())?;
        out.write_all(bin)?;
        out.write_all(&[0; 3][..bin_len - bin.len()])?;
    }

    Ok(())
}
