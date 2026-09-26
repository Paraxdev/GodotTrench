//! Materials and images of an export, resolved the way the Godot build and the editor preview find them.

use std::collections::{BTreeSet, HashMap};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use gt_core::DVec2;
use gt_formats::GameConfig;
use gt_formats::godot_material::{GodotMaterial, Transparency};

use crate::materials::MaterialLibrary;
use crate::models::Model;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Alpha {
    Opaque,
    Mask(f32),
    Blend,
}

/// An image as it goes into the file: PNG or JPEG bytes, copied as found on disk when possible.
pub struct Image {
    pub name: String,
    pub bytes: Vec<u8>,
    pub mime: &'static str,
}

impl Image {
    pub fn extension(&self) -> &'static str {
        if self.mime == "image/jpeg" { "jpg" } else { "png" }
    }
}

pub struct Material {
    pub name: String,
    /// Linear color multiplied with the albedo, alpha included.
    pub base_color: [f32; 4],
    pub albedo: Option<usize>,
    /// Pixel art that should stay sharp.
    pub nearest: bool,
    pub normal: Option<usize>,
    pub normal_scale: f32,
    pub metallic: f32,
    pub roughness: f32,
    /// glTF's packed map, roughness in green and metallic in blue.
    pub metal_rough: Option<usize>,
    /// The plain roughness image, for OBJ's `map_Pr`.
    pub roughness_map: Option<usize>,
    pub occlusion: Option<usize>,
    /// Linear emission color, times `emissive_strength`.
    pub emissive: [f32; 3],
    pub emissive_strength: f32,
    pub emissive_texture: Option<usize>,
    pub alpha: Alpha,
    pub double_sided: bool,
    pub unlit: bool,
}

impl Material {
    fn plain(name: String) -> Self {
        Self {
            name,
            base_color: [1.0; 4],
            albedo: None,
            nearest: false,
            normal: None,
            normal_scale: 1.0,
            metallic: 0.0,
            roughness: 1.0,
            metal_rough: None,
            roughness_map: None,
            occlusion: None,
            emissive: [0.0; 3],
            emissive_strength: 1.0,
            emissive_texture: None,
            alpha: Alpha::Opaque,
            double_sided: false,
            unlit: false,
        }
    }

    pub fn is_emissive(&self) -> bool {
        self.emissive_strength > 0.0 && self.emissive.iter().any(|c| *c > 0.0)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum Key {
    /// A face material by name, the flag set for its decal variant.
    Face(String, bool),
    /// A texture key of a loaded model.
    Model(String),
    Plain(String),
}

#[derive(Default)]
pub struct Materials {
    pub list: Vec<Material>,
    pub images: Vec<Image>,
    /// Face materials the project has no image for.
    pub missing: BTreeSet<String>,
    by_key: HashMap<Key, (usize, DVec2)>,
    by_file: HashMap<PathBuf, Option<usize>>,
    by_pixels: HashMap<u64, usize>,
}

fn linear(c: f32) -> f32 {
    gt_render::srgb_to_linear(c)
}

pub fn to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.003_130_8 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
}

/// Godot's texture channel setting, 0 to 3 for red to alpha and 4 for grayscale.
fn channel(p: &image::Rgba<u8>, which: u8) -> u8 {
    match which {
        0..=3 => p.0[which as usize],
        _ => ((p.0[0] as u16 + p.0[1] as u16 + p.0[2] as u16) / 3) as u8,
    }
}

impl Materials {
    pub fn plain(&mut self, name: &str) -> usize {
        let key = Key::Plain(name.to_string());
        if let Some((i, _)) = self.by_key.get(&key) {
            return *i;
        }

        let mut m = Material::plain(name.to_string());
        m.base_color = [0.6, 0.6, 0.6, 1.0];
        self.add(key, m, DVec2::ONE)
    }

    fn add(&mut self, key: Key, material: Material, size: DVec2) -> usize {
        let index = self.list.len();
        self.list.push(material);
        self.by_key.insert(key, (index, size));
        index
    }

    /// An image file as it is, PNG and JPEG bytes unchanged and other formats turned into PNG. None when it cannot be
    /// read.
    fn file(&mut self, path: &Path) -> Option<usize> {
        if let Some(i) = self.by_file.get(path) {
            return *i;
        }

        let name = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "texture".into());
        let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        let index = match ext.as_str() {
            "png" | "jpg" | "jpeg" => std::fs::read(path).ok().map(|bytes| {
                let mime = if ext == "png" { "image/png" } else { "image/jpeg" };
                self.images.push(Image { name, bytes, mime });
                self.images.len() - 1
            }),
            _ => image::open(path).ok().and_then(|img| self.encode(&name, &img.to_rgba8())),
        };
        self.by_file.insert(path.to_path_buf(), index);
        index
    }

    /// A decoded image as PNG. The same pixels are stored once, models share one albedo between several materials.
    fn encode(&mut self, name: &str, img: &image::RgbaImage) -> Option<usize> {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        img.dimensions().hash(&mut h);
        img.as_raw().hash(&mut h);
        let hash = h.finish();
        if let Some(i) = self.by_pixels.get(&hash) {
            return Some(*i);
        }

        let mut bytes = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png).ok()?;
        self.images.push(Image { name: name.to_string(), bytes, mime: "image/png" });
        self.by_pixels.insert(hash, self.images.len() - 1);
        Some(self.images.len() - 1)
    }

    /// glTF's metallic roughness map from Godot's separate roughness and metallic images.
    fn bake_metal_rough(&mut self, name: &str, rough: Option<(&Path, u8)>, metal: Option<(&Path, u8)>) -> Option<usize> {
        let open = |p: &Path| image::open(p).ok().map(|i| i.to_rgba8());
        let rough = rough.and_then(|(p, c)| Some((open(p)?, c)));
        let metal = metal.and_then(|(p, c)| Some((open(p)?, c)));
        let (w, h) = rough.as_ref().or(metal.as_ref())?.0.dimensions();
        let fit =
            |img: image::RgbaImage| if img.dimensions() == (w, h) { img } else { image::imageops::resize(&img, w, h, image::imageops::FilterType::Triangle) };
        let rough = rough.map(|(i, c)| (fit(i), c));
        let metal = metal.map(|(i, c)| (fit(i), c));
        let baked = image::RgbaImage::from_fn(w, h, |x, y| {
            let g = rough.as_ref().map(|(i, c)| channel(i.get_pixel(x, y), *c)).unwrap_or(255);
            let b = metal.as_ref().map(|(i, c)| channel(i.get_pixel(x, y), *c)).unwrap_or(255);
            image::Rgba([255, g, b, 255])
        });
        self.encode(&format!("{name}_metal_rough"), &baked)
    }

    /// A face material and the size in map units one repeat of its texture covers, which face UVs divide by.
    pub fn face(&mut self, lib: &mut MaterialLibrary, game: &GameConfig, name: &str, decal: bool) -> (usize, DVec2) {
        let key = Key::Face(name.to_string(), decal);
        if let Some(found) = self.by_key.get(&key) {
            return *found;
        }

        let files = lib.material_files(name);
        let info = files.info.clone().unwrap_or_else(|| GodotMaterial {
            // What FuncGodot generates next to an emission map: a black color the texture is added to.
            emission: files.emission.is_some().then_some([0.0; 3]),
            ..Default::default()
        });
        let mut m = Material::plain(if decal { format!("{name} (decal)") } else { name.to_string() });
        let mut pixels = None;
        if let Some(path) = &files.albedo {
            m.albedo = self.file(path);
            pixels = image::image_dimensions(path).ok();
        } else if !lib.is_color_only(name)
            && let Some(img) = lib.load_image(name)
        {
            pixels = Some(img.dimensions());
            m.albedo = self.encode(name, &img);
        }

        if m.albedo.is_none() && !lib.is_color_only(name) {
            self.missing.insert(name.to_string());
        }

        let c = info.albedo_color;
        m.base_color = [linear(c[0]), linear(c[1]), linear(c[2]), c[3]];
        m.nearest = info.nearest.unwrap_or(false);
        m.normal = files.normal.as_deref().and_then(|p| self.file(p));
        m.normal_scale = info.normal_scale;
        m.metallic = info.metallic;
        m.roughness = info.roughness;
        if let Some(orm) = files.orm.as_deref().and_then(|p| self.file(p)) {
            m.metal_rough = Some(orm);
            m.occlusion = Some(orm);
        } else {
            let rough = files.roughness.as_deref().map(|p| (p, info.roughness_channel));
            let metal = files.metallic.as_deref().map(|p| (p, info.metallic_channel));
            if rough.is_some() || metal.is_some() {
                m.metal_rough = self.bake_metal_rough(name, rough, metal);
            }

            m.roughness_map = files.roughness.as_deref().and_then(|p| self.file(p));
            m.occlusion = files.ao.as_deref().and_then(|p| self.file(p));
        }

        if info.is_emissive() {
            let color = info.emission.unwrap_or_default().map(linear);
            m.emissive_texture = files.emission.as_deref().and_then(|p| self.file(p));
            // Godot adds the emission texture to the color by default, glTF multiplies them.
            m.emissive = if m.emissive_texture.is_some() && !info.emission_multiply { [1.0; 3] } else { color };
            m.emissive_strength = info.emission_energy;
        }

        m.alpha = match info.transparency {
            Transparency::Opaque => Alpha::Opaque,
            Transparency::Alpha => Alpha::Blend,
            Transparency::Scissor(t) => Alpha::Mask(t),
            Transparency::Hash => Alpha::Mask(0.5),
        };
        m.double_sided = info.double_sided;
        m.unlit = info.unshaded;
        if decal {
            // Like the addon draws decal sheets: blended over the surface and seen from both sides.
            if !matches!(m.alpha, Alpha::Mask(_)) {
                m.alpha = Alpha::Blend;
            }

            m.double_sided = true;
        }

        let size = info
            .texture_size
            .map(|[w, h]| DVec2::new(w as f64, h as f64))
            .or(pixels.map(|(w, h)| DVec2::new(w as f64, h as f64)))
            .unwrap_or(DVec2::splat(game.textures.fallback_size as f64));
        (self.add(key, m, size), size)
    }

    /// A material of a loaded model, from its texture `key`. `label` names it, usually the model's file name.
    pub fn model(&mut self, model: &Model, key: &str, label: &str) -> usize {
        let map_key = Key::Model(key.to_string());
        if let Some((i, _)) = self.by_key.get(&map_key) {
            return *i;
        }

        let suffix = key
            .rsplit_once('#')
            .map(|(_, s)| s.to_string())
            .unwrap_or_else(|| Path::new(key).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| key.to_string()));
        let name = format!("{label} {suffix}");
        let mut m = Material::plain(name.clone());
        if let Some((key, img, pixelated)) = model.textures.iter().find(|(k, ..)| k == key) {
            let desc = model.material_desc(key, img, *pixelated);
            m.albedo = self.encode(&name, img);
            m.nearest = *pixelated;
            m.base_color[3] = desc.tint[3];
            m.double_sided = desc.double_sided;
            m.alpha = match desc.alpha {
                gt_render::AlphaMode::Opaque => Alpha::Opaque,
                gt_render::AlphaMode::Blend => Alpha::Blend,
                gt_render::AlphaMode::Scissor(t) => Alpha::Mask(t),
                gt_render::AlphaMode::Hash => Alpha::Mask(0.5),
            };
            if desc.emission_energy > 0.0 {
                m.emissive_texture = desc.emission_texture.and_then(|e| self.encode(&format!("{name}_emission"), e));
                m.emissive = if m.emissive_texture.is_some() && !desc.emission_multiply { [1.0; 3] } else { desc.emission };
                m.emissive_strength = desc.emission_energy;
            }
        } else {
            m.base_color = [0.6, 0.6, 0.6, 1.0];
        }

        self.add(map_key, m, DVec2::ONE)
    }
}
