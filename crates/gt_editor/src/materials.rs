use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gt_formats::GameConfig;
use gt_formats::godot_material::{self, GodotMaterial};

pub const DEV_PREFIX: &str = "dev/";

/// Suffixes of non-normal PBR companion maps next to an albedo texture (FuncGodot's map patterns).
/// These are never shown as their own material.
const COMPANIONS: [&str; 6] = ["_roughness", "_metallic", "_ao", "_emission", "_height", "_orm"];

/// Filename suffixes that mark a texture as a normal map, longest first so stripping matches the
/// most specific one. Compared case-insensitively.
pub const NORMAL_SUFFIXES: [&str; 6] = ["_normalmap", "_normal", "_nmap", "_norm", "_nrm", "_n"];

/// The normal-map suffix a lowercased texture name ends with, if any.
pub fn normal_suffix(name_lower: &str) -> Option<&'static str> {
    NORMAL_SUFFIXES.iter().copied().find(|s| name_lower.ends_with(s) && name_lower.len() > s.len())
}

/// The albedo/diffuse name a normal-map texture belongs to (its name without the normal suffix).
pub fn albedo_of_normal(name: &str) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    normal_suffix(&lower).map(|s| name[..name.len() - s.len()].to_string())
}

#[derive(Clone, Debug)]
pub struct MaterialEntry {
    /// Path relative to the texture root without extension, as stored on faces.
    pub name: String,
    pub folder: String,
    /// Albedo image.
    pub path: Option<PathBuf>,
    /// Godot material resource with the same name, if any.
    pub material_file: Option<PathBuf>,
    /// A normal map was found for this texture, so dropping it into the scene gives valid normals.
    pub has_normal: bool,
    /// This entry is itself a normal map with no matching albedo/diffuse texture of the same name.
    pub missing_albedo: bool,
}

/// Images and settings of one material, ready for the renderer.
pub struct LoadedMaterial {
    pub albedo: image::RgbaImage,
    pub normal: Option<image::RgbaImage>,
    pub emission: Option<image::RgbaImage>,
    pub info: GodotMaterial,
}

pub struct MaterialLibrary {
    pub entries: Vec<MaterialEntry>,
    pub root: Option<PathBuf>,
    project_root: Option<PathBuf>,
    material_root: Option<PathBuf>,
    material_ext: String,
    image_exts: Vec<String>,
    thumbnails: HashMap<String, Option<egui::TextureHandle>>,
    sizes: HashMap<String, [u32; 2]>,
    infos: HashMap<String, Option<GodotMaterial>>,
}

impl MaterialLibrary {
    pub fn new(game: &GameConfig) -> Self {
        let mut lib = Self {
            entries: Vec::new(),
            root: None,
            project_root: None,
            material_root: None,
            material_ext: "tres".into(),
            image_exts: Vec::new(),
            thumbnails: HashMap::new(),
            sizes: HashMap::new(),
            infos: HashMap::new(),
        };
        lib.rescan(game);
        lib
    }

    pub fn rescan(&mut self, game: &GameConfig) {
        self.entries.clear();
        self.thumbnails.clear();
        self.sizes.clear();
        self.infos.clear();
        self.root = game.texture_root().filter(|p| p.is_dir());
        self.project_root = game.project_root.clone();
        self.material_ext = game.textures.material_extension.trim_start_matches('.').to_ascii_lowercase();
        self.material_root =
            if game.textures.material_dir.is_empty() { self.root.clone() } else { game.resolve_res(&game.textures.material_dir).filter(|p| p.is_dir()) };
        self.image_exts = game.textures.extensions.iter().map(|e| e.to_ascii_lowercase()).collect();
        let mut found = Vec::new();
        if let Some(root) = self.root.clone() {
            scan_dir(&root, &root, &self.image_exts, &mut found);
        }
        // Material resources without an image of the same name still show up, previewed with their albedo texture.
        if let Some(mat_root) = self.material_root.clone() {
            let mut files = Vec::new();
            scan_dir(&mat_root, &mat_root, &[self.material_ext.clone(), "material".into()], &mut files);
            for f in files {
                let file = f.path.clone();
                match found.iter_mut().find(|e| e.name.eq_ignore_ascii_case(&f.name)) {
                    Some(e) => e.material_file = file,
                    None => {
                        let albedo = file
                            .as_ref()
                            .and_then(|p| std::fs::read_to_string(p).ok())
                            .and_then(|t| godot_material::parse(&t))
                            .and_then(|m| m.albedo_texture)
                            .and_then(|res| self.resolve_res(&res));
                        if albedo.is_some() {
                            found.push(MaterialEntry {
                                name: f.name,
                                folder: f.folder,
                                path: albedo,
                                material_file: file,
                                has_normal: false,
                                missing_albedo: false,
                            });
                        }
                    }
                }
            }
        }
        pair_normal_maps(&mut found);
        found.sort_by(|a, b| a.name.cmp(&b.name));
        // Project textures win over built-in placeholders with the same name.
        for (name, _) in dev_textures() {
            if !found.iter().any(|e| e.name.eq_ignore_ascii_case(name)) {
                self.entries.push(MaterialEntry {
                    name: name.to_string(),
                    folder: name.rsplit_once('/').map(|(f, _)| f.to_string()).unwrap_or_default(),
                    path: None,
                    material_file: None,
                    has_normal: false,
                    missing_albedo: false,
                });
            }
        }
        self.entries.extend(found);
    }

    fn resolve_res(&self, res: &str) -> Option<PathBuf> {
        let rel = res.strip_prefix("res://")?;
        Some(self.project_root.as_ref()?.join(rel)).filter(|p| p.is_file())
    }

    pub fn folders(&self) -> Vec<String> {
        let mut f: Vec<String> = self.entries.iter().map(|e| e.folder.clone()).collect();
        f.sort();
        f.dedup();
        f
    }

    pub fn find(&self, name: &str) -> Option<&MaterialEntry> {
        self.entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))
    }

    pub fn load_image(&self, name: &str) -> Option<image::RgbaImage> {
        if let Some(rel) = name.strip_prefix("res://") {
            return image::open(self.project_root.as_ref()?.join(rel)).ok().map(|img| img.to_rgba8());
        }
        if let Some(path) = self.find(name).and_then(|e| e.path.as_ref()) {
            return image::open(path).ok().map(|img| img.to_rgba8());
        }
        dev_textures().into_iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, make)| make())
    }

    /// Godot material settings for a face material, from its `.tres` file. Cached.
    pub fn info(&mut self, name: &str) -> Option<GodotMaterial> {
        if let Some(i) = self.infos.get(name) {
            return i.clone();
        }
        let info = self.read_info(name);
        self.infos.insert(name.to_string(), info.clone());
        info
    }

    fn read_info(&self, name: &str) -> Option<GodotMaterial> {
        let file = match self.find(name).and_then(|e| e.material_file.clone()) {
            Some(f) => f,
            None => {
                let root = self.material_root.as_ref()?;
                [self.material_ext.as_str(), "material"].iter().map(|ext| root.join(format!("{name}.{ext}"))).find(|p| p.is_file())?
            }
        };
        godot_material::parse(&std::fs::read_to_string(file).ok()?)
    }

    /// Companion map such as `wall_normal.png` next to the albedo image.
    fn companion(&self, name: &str, suffix: &str) -> Option<PathBuf> {
        let albedo = self.find(name)?.path.clone()?;
        let stem = albedo.file_stem()?.to_string_lossy().into_owned();
        let dir = albedo.parent()?;
        self.image_exts.iter().map(|ext| dir.join(format!("{stem}{suffix}.{ext}"))).find(|p| p.is_file())
    }

    /// The first normal-map companion next to the albedo, trying every supported suffix.
    fn normal_companion(&self, name: &str) -> Option<PathBuf> {
        NORMAL_SUFFIXES.iter().find_map(|s| self.companion(name, s))
    }

    /// Albedo plus normal and emission maps and the Godot settings, for the preview renderer.
    pub fn load_material(&mut self, name: &str) -> Option<LoadedMaterial> {
        let open = |p: &Path| image::open(p).ok().map(|i| i.to_rgba8());
        // Like FuncGodot, a material resource is used as is, companion maps only apply to generated materials.
        if let Some(info) = self.info(name) {
            let res = |r: &Option<String>| r.as_deref().and_then(|r| self.resolve_res(r)).and_then(|p| open(&p));
            let albedo = res(&info.albedo_texture).or_else(|| self.load_image(name))?;
            let (normal, emission) = (res(&info.normal_texture), res(&info.emission_texture));
            return Some(LoadedMaterial { albedo, normal, emission, info });
        }
        let albedo = self.load_image(name)?;
        let normal = self.normal_companion(name).and_then(|p| open(&p));
        let emission = self.companion(name, "_emission").and_then(|p| open(&p));
        let info = GodotMaterial { emission: emission.is_some().then_some([1.0; 3]), ..Default::default() };
        Some(LoadedMaterial { albedo, normal, emission, info })
    }

    pub fn remember_size(&mut self, name: &str, size: [u32; 2]) {
        self.sizes.insert(name.to_string(), size);
    }

    pub fn size(&self, name: &str) -> Option<[u32; 2]> {
        self.sizes.get(name).copied()
    }

    /// Lazily creates a small egui thumbnail. `budget` limits decoding work per frame.
    pub fn thumbnail(&mut self, ctx: &egui::Context, name: &str, budget: &mut u32) -> Option<egui::TextureHandle> {
        if let Some(t) = self.thumbnails.get(name) {
            return t.clone();
        }
        if *budget == 0 {
            return None;
        }
        *budget -= 1;
        let handle = self.load_image(name).map(|img| {
            self.sizes.insert(name.to_string(), [img.width(), img.height()]);
            let thumb = image::imageops::thumbnail(&img, 96, 96);
            let color = egui::ColorImage::from_rgba_unmultiplied([thumb.width() as usize, thumb.height() as usize], thumb.as_raw());
            ctx.load_texture(format!("mat:{name}"), color, egui::TextureOptions::LINEAR)
        });
        self.thumbnails.insert(name.to_string(), handle.clone());
        handle
    }

    /// Full resolution texture for the hotspot and UV editors.
    pub fn full_texture(&mut self, ctx: &egui::Context, name: &str) -> Option<egui::TextureHandle> {
        let key = format!("full:{name}");
        if let Some(t) = self.thumbnails.get(&key) {
            return t.clone();
        }
        let handle = self.load_image(name).map(|img| {
            self.sizes.insert(name.to_string(), [img.width(), img.height()]);
            let color = egui::ColorImage::from_rgba_unmultiplied([img.width() as usize, img.height() as usize], img.as_raw());
            ctx.load_texture(key.clone(), color, egui::TextureOptions::NEAREST_REPEAT)
        });
        self.thumbnails.insert(key, handle.clone());
        handle
    }
}

/// Removes normal-map textures that belong to an albedo of the same name, flagging that albedo as
/// having valid normals, and marks lone normal maps (no matching albedo) as missing their diffuse.
fn pair_normal_maps(entries: &mut Vec<MaterialEntry>) {
    use std::collections::HashSet;
    let albedos: HashSet<String> =
        entries.iter().filter(|e| normal_suffix(&e.name.to_ascii_lowercase()).is_none()).map(|e| e.name.to_ascii_lowercase()).collect();
    let mut with_normal: HashSet<String> = HashSet::new();
    let mut i = 0;
    while i < entries.len() {
        if let Some(base) = albedo_of_normal(&entries[i].name) {
            let base_lower = base.to_ascii_lowercase();
            if albedos.contains(&base_lower) {
                with_normal.insert(base_lower);
                entries.remove(i);
                continue;
            }
            entries[i].missing_albedo = true;
        }
        i += 1;
    }
    for e in entries.iter_mut() {
        if with_normal.contains(&e.name.to_ascii_lowercase()) {
            e.has_normal = true;
        }
    }
}

fn scan_dir(root: &Path, dir: &Path, exts: &[String], out: &mut Vec<MaterialEntry>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(root, &path, exts, out);
            continue;
        }
        let Some(ext) = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()) else { continue };
        if !exts.contains(&ext) {
            continue;
        }
        let Ok(rel) = path.strip_prefix(root) else { continue };
        let name = rel.with_extension("").to_string_lossy().replace('\\', "/");
        let lower = name.to_ascii_lowercase();
        if COMPANIONS.iter().any(|s| lower.ends_with(s)) {
            continue;
        }
        let folder = rel.parent().map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_default();
        out.push(MaterialEntry { name, folder, path: Some(path), material_file: None, has_normal: false, missing_albedo: false });
    }
}

type Generator = fn() -> image::RgbaImage;

fn grid_texture(base: [u8; 3], line: [u8; 3]) -> image::RgbaImage {
    image::RgbaImage::from_fn(128, 128, |x, y| {
        let major = x % 64 == 0 || y % 64 == 0;
        let minor = x % 16 == 0 || y % 16 == 0;
        let c = if major {
            line
        } else if minor {
            [(base[0] as u16 + line[0] as u16) as u8 / 2, (base[1] as u16 + line[1] as u16) as u8 / 2, (base[2] as u16 + line[2] as u16) as u8 / 2]
        } else {
            base
        };
        image::Rgba([c[0], c[1], c[2], 255])
    })
}

fn stripes(a: [u8; 3], b: [u8; 3]) -> image::RgbaImage {
    image::RgbaImage::from_fn(64, 64, |x, y| {
        let c = if ((x + y) / 8) % 2 == 0 { a } else { b };
        image::Rgba([c[0], c[1], c[2], 255])
    })
}

/// Procedural textures available without a Godot project.
pub fn dev_textures() -> Vec<(&'static str, Generator)> {
    vec![
        ("dev/grey", || grid_texture([120, 120, 124], [160, 160, 166])),
        ("dev/dark", || grid_texture([60, 62, 68], [90, 92, 100])),
        ("dev/orange", || grid_texture([200, 110, 40], [235, 150, 80])),
        ("dev/blue", || grid_texture([50, 90, 170], [90, 130, 210])),
        ("dev/green", || grid_texture([60, 140, 70], [100, 180, 110])),
        ("special/clip", || stripes([180, 40, 180], [90, 20, 90])),
        ("special/skip", || stripes([40, 40, 40], [90, 90, 90])),
        ("special/origin", || stripes([230, 140, 20], [120, 70, 10])),
        ("special/trigger", || stripes([230, 160, 30], [140, 90, 10])),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_files_and_companions() {
        let dir = std::env::temp_dir().join(format!("gt_materials_{}", std::process::id()));
        let tex = dir.join("textures");
        std::fs::create_dir_all(tex.join("props")).unwrap();
        std::fs::write(dir.join("project.godot"), "").unwrap();
        image::RgbaImage::from_pixel(8, 4, image::Rgba([200, 10, 10, 255])).save(tex.join("props/crate.png")).unwrap();
        image::RgbaImage::from_pixel(8, 4, image::Rgba([128, 128, 255, 255])).save(tex.join("props/crate_normal.png")).unwrap();
        image::RgbaImage::from_pixel(2, 2, image::Rgba([0, 0, 255, 128])).save(tex.join("props/pane.png")).unwrap();
        std::fs::write(
            tex.join("glass.tres"),
            "[gd_resource type=\"StandardMaterial3D\" format=3]\n[ext_resource type=\"Texture2D\" path=\"res://textures/props/pane.png\" id=\"1\"]\n[resource]\nalbedo_texture = ExtResource(\"1\")\ntransparency = 1\ntexture_filter = 0\n",
        )
        .unwrap();
        let mut game = GameConfig::builtin();
        game.project_root = Some(dir.clone());
        game.textures.base_dir = "res://textures".into();
        let mut lib = MaterialLibrary::new(&game);
        assert!(lib.find("props/crate_normal").is_none(), "companion maps are not materials");
        let crate_mat = lib.load_material("props/crate").unwrap();
        assert!(crate_mat.normal.is_some(), "companion normal map picked up");
        assert_eq!(crate_mat.albedo.dimensions(), (8, 4));
        let glass = lib.load_material("glass").expect("material file without image of the same name");
        assert!(glass.info.is_transparent());
        assert_eq!(glass.info.nearest, Some(true));
        assert_eq!(glass.albedo.dimensions(), (2, 2));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
