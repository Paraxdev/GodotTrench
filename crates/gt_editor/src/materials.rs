use std::collections::HashMap;
use std::path::{Path, PathBuf};

use gt_formats::GameConfig;
use gt_formats::godot_material::{self, GodotMaterial};

pub const DEV_PREFIX: &str = "dev/";

/// Filename suffixes that mark a texture as a normal map, longest first so stripping matches the
/// most specific one. Compared case-insensitively. Single-letter suffixes like `_n` are common in
/// texture packs, so they are matched too.
pub const NORMAL_SUFFIXES: [&str; 7] = ["_normalmap", "_normal", "_nrm", "_nmap", "_norm", "_nm", "_n"];

/// Suffixes marking the albedo/diffuse map of a PBR set, for packs that name the colour map `x_d`
/// rather than a plain `x`. Longest first.
pub const DIFFUSE_SUFFIXES: [&str; 6] = ["_basecolor", "_diffuse", "_albedo", "_color", "_col", "_d"];

/// Every non-diffuse PBR companion suffix (normal, specular, roughness, metallic, ao, height,
/// emission, packed orm), longest first so `_normal` wins over `_n` and `_specular` over `_s`.
/// A texture ending in one of these is a companion map, hidden when its set's albedo is present.
const COMPANION_SUFFIXES: [&str; 24] = [
    "_normalmap",
    "_specular",
    "_roughness",
    "_metallic",
    "_emissive",
    "_emission",
    "_normal",
    "_height",
    "_occlusion",
    "_metal",
    "_rough",
    "_disp",
    "_glow",
    "_nrm",
    "_nmap",
    "_norm",
    "_spec",
    "_orm",
    "_ao",
    "_nm",
    "_n",
    "_s",
    "_h",
    "_e",
];

/// Companion map that FuncGodot's generated materials use as the emission texture (`emission_map_pattern`).
pub const EMISSION_SUFFIX: &str = "_emission";

/// The normal-map suffix a lowercased texture name ends with, if any.
pub fn normal_suffix(name_lower: &str) -> Option<&'static str> {
    NORMAL_SUFFIXES.iter().copied().find(|s| name_lower.ends_with(s) && name_lower.len() > s.len())
}

/// The diffuse-map suffix a lowercased texture name ends with, if any.
pub fn diffuse_suffix(name_lower: &str) -> Option<&'static str> {
    DIFFUSE_SUFFIXES.iter().copied().find(|s| name_lower.ends_with(s) && name_lower.len() > s.len())
}

/// The base name of a PBR companion map (its name without a normal/spec/rough/etc. suffix), if it is
/// one. Diffuse maps are not companions, they anchor a set.
fn companion_base(name_lower: &str) -> Option<String> {
    COMPANION_SUFFIXES.iter().find(|s| name_lower.ends_with(**s) && name_lower.len() > s.len()).map(|s| name_lower[..name_lower.len() - s.len()].to_string())
}

/// The PBR set base a lowercased albedo name belongs to: its name minus a diffuse suffix, else the
/// name itself. Companion maps share this base, so it locates their siblings on disk.
fn set_base(name_lower: &str) -> String {
    diffuse_suffix(name_lower).map(|s| name_lower[..name_lower.len() - s.len()].to_string()).unwrap_or_else(|| name_lower.to_string())
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
    /// Companion PBR maps (normal, spec, roughness, height, ao, emission) were found for this
    /// texture, so it forms a full material set. Shown with a PBR badge, its companions are hidden.
    pub is_pbr: bool,
    /// Glows: its material file enables emission, or without one an `_emission` companion map sits next to it.
    pub is_emissive: bool,
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
    /// `metadata/texture_size` of every material file, keyed by lowercased material name.
    world_sizes: HashMap<String, [f64; 2]>,
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
            world_sizes: HashMap::new(),
            infos: HashMap::new(),
        };
        lib.rescan(game);
        lib
    }

    pub fn rescan(&mut self, game: &GameConfig) {
        self.entries.clear();
        self.thumbnails.clear();
        self.sizes.clear();
        self.world_sizes.clear();
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

        // FuncGodot tries the extensions in order, so with wall.png and wall.jpg side by side the first extension is the material.
        let rank = |e: &MaterialEntry| {
            let ext = e.path.as_ref().and_then(|p| p.extension()).map(|x| x.to_string_lossy().to_ascii_lowercase());
            ext.and_then(|x| self.image_exts.iter().position(|y| *y == x)).unwrap_or(usize::MAX)
        };
        found.sort_by_cached_key(|e| (e.name.to_ascii_lowercase(), rank(e)));
        found.dedup_by(|later, first| later.name.eq_ignore_ascii_case(&first.name));

        // Material resources without an image of the same name still show up, previewed with their albedo texture.
        if let Some(mat_root) = self.material_root.clone() {
            let mut files = Vec::new();
            scan_dir(&mat_root, &mat_root, &[self.material_ext.clone(), "material".into()], &mut files);
            for f in files {
                let file = f.path.clone();
                let parsed = file.as_ref().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| godot_material::parse(&t));
                if let Some([w, h]) = parsed.as_ref().and_then(|m| m.texture_size) {
                    self.world_sizes.insert(f.name.to_ascii_lowercase(), [w as f64, h as f64]);
                }

                let emissive = parsed.as_ref().is_some_and(|m| m.is_emissive());
                match found.iter_mut().find(|e| e.name.eq_ignore_ascii_case(&f.name)) {
                    Some(e) => {
                        e.material_file = file;
                        e.is_emissive = emissive;
                    }
                    None => {
                        let albedo = parsed.and_then(|m| m.albedo_texture).and_then(|res| self.resolve_res(&res));
                        if albedo.is_some() {
                            found.push(MaterialEntry {
                                name: f.name,
                                folder: f.folder,
                                path: albedo,
                                material_file: file,
                                has_normal: false,
                                missing_albedo: false,
                                is_pbr: false,
                                is_emissive: emissive,
                            });
                        }
                    }
                }
            }
        }

        pair_pbr_maps(&mut found);
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
                    is_pbr: false,
                    is_emissive: false,
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

    /// Companion map such as `wall_normal.png` next to the albedo image. When the albedo is itself a
    /// diffuse-suffixed file (`wall_d.png`), the companions share the base `wall`, not `wall_d`.
    fn companion(&self, name: &str, suffix: &str) -> Option<PathBuf> {
        let albedo = self.find(name)?.path.clone()?;
        let stem = albedo.file_stem()?.to_string_lossy().into_owned();
        let base = match diffuse_suffix(&stem.to_ascii_lowercase()) {
            Some(s) => stem[..stem.len() - s.len()].to_string(),
            None => stem,
        };
        let dir = albedo.parent()?;
        self.image_exts.iter().map(|ext| dir.join(format!("{base}{suffix}.{ext}"))).find(|p| p.is_file())
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
        let emission_path = self.companion(name, EMISSION_SUFFIX);
        let emission = emission_path.as_deref().and_then(open);
        // What FuncGodot generates: the default black emission color plus the texture.
        let info = GodotMaterial {
            emission: emission.is_some().then_some([0.0; 3]),
            emission_texture: emission.as_ref().and(emission_path).map(|p| p.to_string_lossy().replace('\\', "/")),
            ..Default::default()
        };
        Some(LoadedMaterial { albedo, normal, emission, info })
    }

    pub fn remember_size(&mut self, name: &str, size: [u32; 2]) {
        self.sizes.insert(name.to_string(), size);
    }

    /// Pixel size of the albedo image, read from the file header when no texture has been loaded yet.
    pub fn pixel_size(&self, name: &str) -> Option<[u32; 2]> {
        if let Some(s) = self.sizes.get(name) {
            return Some(*s);
        }

        let path = self.find(name)?.path.as_ref()?;
        image::image_dimensions(path).ok().map(|(w, h)| [w, h])
    }

    /// The `metadata/texture_size` world size of a material, when its material file sets one.
    pub fn world_size(&self, name: &str) -> Option<[f64; 2]> {
        self.world_sizes.get(&name.to_ascii_lowercase()).copied()
    }

    /// Size in map units that one repeat of the texture covers, which is what face UVs divide by: the
    /// material's world size when set, else the albedo's pixel size.
    pub fn size(&self, name: &str) -> Option<[f64; 2]> {
        self.world_size(name).or_else(|| self.pixel_size(name).map(|[w, h]| [w as f64, h as f64]))
    }

    /// "1024 x 1024 px, 64 x 64 units a repeat" for the inspector, just the pixels without an override.
    pub fn size_label(&self, name: &str) -> Option<String> {
        let px = self.pixel_size(name).map(|[w, h]| format!("{w} x {h} px"));
        let world = self.world_size(name).map(|[w, h]| format!("{w} x {h} units a repeat"));
        match (px, world) {
            (Some(p), Some(w)) => Some(format!("{p}, {w}")),
            (p, w) => p.or(w),
        }
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

/// Groups textures into PBR sets: an albedo (a plain `wall` or a diffuse-suffixed `wall_d`) plus its
/// companion maps (`wall_normal`, `wall_s`, `wall_h`, `wall_ao`, …). Companion maps that belong to a
/// present albedo are removed and the albedo is flagged (`is_pbr`, and `has_normal` when a normal was
/// among them). Companion maps with no matching albedo stay visible; lone normals are flagged as
/// missing their diffuse. The albedo keeps its own name so face material references never change.
fn pair_pbr_maps(entries: &mut Vec<MaterialEntry>) {
    use std::collections::HashMap;
    // Anchors are the albedos: every texture that is not itself a companion map. Keyed by set base
    // (a diffuse-suffixed name maps to the base its companions share), first anchor of a base wins.
    let mut anchor: HashMap<String, usize> = HashMap::new();
    for (i, e) in entries.iter().enumerate() {
        let lower = e.name.to_ascii_lowercase();
        if companion_base(&lower).is_some() {
            continue;
        }

        anchor.entry(set_base(&lower)).or_insert(i);
    }

    let mut remove: Vec<usize> = Vec::new();
    for i in 0..entries.len() {
        let lower = entries[i].name.to_ascii_lowercase();
        let Some(base) = companion_base(&lower) else { continue };
        match anchor.get(&base) {
            Some(&ai) => {
                entries[ai].is_pbr = true;
                if normal_suffix(&lower).is_some() {
                    entries[ai].has_normal = true;
                }

                // FuncGodot only generates emission from this suffix, and a material file is used as it is.
                if lower.ends_with(EMISSION_SUFFIX) && entries[ai].material_file.is_none() {
                    entries[ai].is_emissive = true;
                }

                remove.push(i);
            }
            None if normal_suffix(&lower).is_some() => entries[i].missing_albedo = true,
            None => {}
        }
    }

    for i in remove.into_iter().rev() {
        entries.remove(i);
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
        let folder = rel.parent().map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_default();
        out.push(MaterialEntry {
            name,
            folder,
            path: Some(path),
            material_file: None,
            has_normal: false,
            missing_albedo: false,
            is_pbr: false,
            is_emissive: false,
        });
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

    #[test]
    fn same_name_in_two_formats_is_one_material() {
        let dir = std::env::temp_dir().join(format!("gt_material_formats_{}", std::process::id()));
        let tex = dir.join("textures");
        std::fs::create_dir_all(&tex).unwrap();
        std::fs::write(dir.join("project.godot"), "").unwrap();
        image::RgbImage::from_pixel(8, 8, image::Rgb([200, 10, 10])).save(tex.join("bark.jpg")).unwrap();
        image::RgbaImage::from_pixel(4, 4, image::Rgba([10, 200, 10, 255])).save(tex.join("bark.png")).unwrap();
        image::RgbImage::from_pixel(4, 4, image::Rgb([128, 128, 255])).save(tex.join("bark_normal.jpg")).unwrap();
        image::RgbaImage::from_pixel(4, 4, image::Rgba([128, 128, 255, 255])).save(tex.join("bark_normal.png")).unwrap();
        let mut game = GameConfig::builtin();
        game.project_root = Some(dir.clone());
        game.textures.base_dir = "res://textures".into();
        let lib = MaterialLibrary::new(&game);
        let barks: Vec<_> = lib.entries.iter().filter(|e| e.name == "bark").collect();
        assert_eq!(barks.len(), 1, "one entry per material name");
        assert_eq!(barks[0].path.as_ref().unwrap().extension().unwrap(), "png", "the first extension in the list wins, as in FuncGodot");
        assert!(barks[0].is_pbr && barks[0].has_normal);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn texture_size_metadata_overrides_the_pixel_size() {
        let dir = std::env::temp_dir().join(format!("gt_texture_size_{}", std::process::id()));
        let tex = dir.join("textures");
        std::fs::create_dir_all(&tex).unwrap();
        std::fs::write(dir.join("project.godot"), "").unwrap();
        image::RgbaImage::from_pixel(64, 32, image::Rgba([90, 60, 40, 255])).save(tex.join("photo.png")).unwrap();
        image::RgbaImage::from_pixel(16, 16, image::Rgba([90, 60, 40, 255])).save(tex.join("plain.png")).unwrap();
        std::fs::write(
            tex.join("photo.tres"),
            "[gd_resource type=\"StandardMaterial3D\" format=3]\n[ext_resource type=\"Texture2D\" path=\"res://textures/photo.png\" id=\"1\"]\n[resource]\nalbedo_texture = ExtResource(\"1\")\nmetadata/texture_size = Vector2(128, 48)\n",
        )
        .unwrap();
        let mut game = GameConfig::builtin();
        game.project_root = Some(dir.clone());
        game.textures.base_dir = "res://textures".into();
        let mut lib = MaterialLibrary::new(&game);
        assert_eq!(lib.size("photo"), Some([128.0, 48.0]), "the world size wins before anything is loaded");
        assert_eq!(lib.size("Photo"), Some([128.0, 48.0]), "names match without case, like everywhere else");
        assert_eq!(lib.pixel_size("photo"), Some([64, 32]), "the pixel size is still known");
        assert_eq!(lib.size("plain"), Some([16.0, 16.0]), "without metadata UVs follow the pixel size");
        lib.remember_size("photo", [64, 32]);
        assert_eq!(lib.size("photo"), Some([128.0, 48.0]), "loading the texture does not undo the override");
        assert_eq!(lib.info("photo").and_then(|m| m.texture_size), Some([128.0, 48.0]));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn pbr_sets_hide_companions_and_flag_the_albedo() {
        let dir = std::env::temp_dir().join(format!("gt_pbr_{}", std::process::id()));
        let tex = dir.join("textures");
        std::fs::create_dir_all(&tex).unwrap();
        std::fs::write(dir.join("project.godot"), "").unwrap();
        let px = |c: [u8; 4]| image::RgbaImage::from_pixel(4, 4, image::Rgba(c));
        // Convention A: a plain base with normal, spec and height maps.
        for (file, c) in
            [("201.png", [200, 0, 0, 255]), ("201_norm.png", [128, 128, 255, 255]), ("201_s.png", [80, 80, 80, 255]), ("201_h.png", [40, 40, 40, 255])]
        {
            px(c).save(tex.join(file)).unwrap();
        }

        // Convention B: the colour map is named _d, companions share the base without it.
        px([10, 120, 30, 255]).save(tex.join("moss_d.png")).unwrap();
        px([128, 128, 255, 255]).save(tex.join("moss_n.png")).unwrap();
        px([60, 60, 60, 255]).save(tex.join("moss_ao.png")).unwrap();
        // A lone normal map with no albedo stays, flagged.
        px([128, 128, 255, 255]).save(tex.join("orphan_normal.png")).unwrap();

        let mut game = GameConfig::builtin();
        game.project_root = Some(dir.clone());
        game.textures.base_dir = "res://textures".into();
        let lib = MaterialLibrary::new(&game);

        let names: Vec<&str> = lib.entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"201"), "plain albedo shows");
        assert!(!names.contains(&"201_norm") && !names.contains(&"201_s") && !names.contains(&"201_h"), "companions hidden: {names:?}");
        assert!(names.contains(&"moss_d"), "diffuse-suffixed albedo shows under its own name");
        assert!(!names.contains(&"moss_n") && !names.contains(&"moss_ao"), "diffuse-set companions hidden: {names:?}");
        assert!(names.contains(&"orphan_normal"), "lone normal stays visible");

        let a = lib.find("201").unwrap();
        assert!(a.is_pbr && a.has_normal);
        let b = lib.find("moss_d").unwrap();
        assert!(b.is_pbr && b.has_normal, "diffuse-suffixed albedo is flagged from its base-named companions");
        assert!(lib.find("orphan_normal").unwrap().missing_albedo);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn emissive_materials_are_flagged_and_companions_picked_up() {
        let dir = std::env::temp_dir().join(format!("gt_emissive_{}", std::process::id()));
        let tex = dir.join("textures");
        std::fs::create_dir_all(&tex).unwrap();
        std::fs::write(dir.join("project.godot"), "").unwrap();
        let px = |c: [u8; 4]| image::RgbaImage::from_pixel(4, 4, image::Rgba(c));
        px([90, 70, 60, 255]).save(tex.join("facade.png")).unwrap();
        px([255, 200, 120, 255]).save(tex.join("facade_emission.png")).unwrap();
        px([90, 90, 90, 255]).save(tex.join("wall.png")).unwrap();
        px([200, 40, 40, 255]).save(tex.join("sign.png")).unwrap();
        std::fs::write(
            tex.join("sign.tres"),
            "[gd_resource type=\"StandardMaterial3D\" format=3]\n[ext_resource type=\"Texture2D\" path=\"res://textures/sign.png\" id=\"1\"]\n[resource]\nalbedo_texture = ExtResource(\"1\")\nemission_enabled = true\nemission = Color(1, 0.3, 0.2, 1)\nemission_energy_multiplier = 4.0\n",
        )
        .unwrap();
        // A material file wins over companions: this one does not enable emission, so it does not glow.
        px([60, 60, 60, 255]).save(tex.join("vent.png")).unwrap();
        px([255, 255, 255, 255]).save(tex.join("vent_emission.png")).unwrap();
        std::fs::write(
            tex.join("vent.tres"),
            "[gd_resource type=\"StandardMaterial3D\" format=3]\n[ext_resource type=\"Texture2D\" path=\"res://textures/vent.png\" id=\"1\"]\n[resource]\nalbedo_texture = ExtResource(\"1\")\n",
        )
        .unwrap();
        let mut game = GameConfig::builtin();
        game.project_root = Some(dir.clone());
        game.textures.base_dir = "res://textures".into();
        let mut lib = MaterialLibrary::new(&game);
        assert!(lib.find("facade_emission").is_none(), "the emission map is a companion, not a material");
        assert!(lib.find("facade").unwrap().is_emissive);
        assert!(lib.find("sign").unwrap().is_emissive);
        assert!(!lib.find("wall").unwrap().is_emissive);
        assert!(!lib.find("vent").unwrap().is_emissive);

        let facade = lib.load_material("facade").unwrap();
        assert_eq!(facade.emission.as_ref().map(|e| e.dimensions()), Some((4, 4)));
        assert!(facade.info.is_emissive() && !facade.info.emission_multiply);
        assert_eq!(facade.info.emission, Some([0.0; 3]), "like FuncGodot: black color, texture added");
        let sign = lib.load_material("sign").unwrap();
        assert!(sign.emission.is_none() && sign.info.is_emissive());
        assert_eq!(sign.info.emission_energy, 4.0);
        let vent = lib.load_material("vent").unwrap();
        assert!(vent.emission.is_none() && !vent.info.is_emissive());
        assert!(lib.load_material("wall").unwrap().emission.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
