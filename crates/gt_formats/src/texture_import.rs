//! Converts textures Godot cannot load into project PNGs and material resources: Valve `.vmt`/`.vtf` materials,
//! Quake and Half-Life `.wad` packs and Quake 2 `.wal` files. Output files are named after the face material, so
//! faces of imported `.vmf` and `.map` files find them, and a `.tres` carries what a plain image cannot say, such as
//! alpha testing or a normal map.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::game::to_res_path;
use crate::quake_texture::{self, Paletted, SURF_TRANS33, SURF_TRANS66};
use crate::vmt::{self, Vmt};
use crate::vtf::{self, Image};

/// Where converted files go.
#[derive(Clone, Debug)]
pub struct Target {
    pub project_root: PathBuf,
    /// The project's texture folder, face material names are relative to it.
    pub texture_dir: PathBuf,
    /// Where material resources go, the texture folder unless the project sets its own.
    pub material_dir: PathBuf,
    pub material_ext: String,
}

#[derive(Clone, Debug)]
enum Source {
    Valve {
        vmt: PathBuf,
        root: usize,
    },
    Paletted(Paletted),
    /// A loose image Godot can load already, as FuncGodot and Qodot projects ship them. It is copied as it is.
    Image(PathBuf),
}

const IMAGE_EXTS: [&str; 6] = ["png", "jpg", "jpeg", "tga", "bmp", "webp"];

/// Textures found in a set of folders and files, by face material name (lowercase).
#[derive(Default)]
pub struct Library {
    sources: BTreeMap<String, Source>,
    /// Per Valve `materials` folder, its files by lowercased relative path.
    roots: Vec<HashMap<String, PathBuf>>,
    /// Files that could not be read while scanning, with the reason.
    pub errors: Vec<(PathBuf, String)>,
}

#[derive(Debug, Default)]
pub struct Report {
    pub converted: Vec<String>,
    /// Already in the project and left alone.
    pub existing: Vec<String>,
    pub failed: Vec<(String, String)>,
}

impl Report {
    pub fn summary(&self) -> String {
        let mut s = format!("{} converted", self.converted.len());
        if !self.existing.is_empty() {
            let _ = write!(s, ", {} already in the project", self.existing.len());
        }

        if !self.failed.is_empty() {
            let _ = write!(s, ", {} failed ({}: {})", self.failed.len(), self.failed[0].0, self.failed[0].1);
        }

        s
    }
}

/// The file name a face material is written under. `*` marks Quake liquids and cannot be in a file name, FuncGodot
/// drops it when it looks for the material file.
pub fn file_stem(material: &str) -> String {
    material.to_ascii_lowercase().replace('*', "")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else {
            out.push(p);
        }
    }
}

fn ext(p: &Path) -> String {
    p.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default()
}

/// The nearest folder named `name` that holds `file`, else `fallback`.
fn root_named(file: &Path, name: &str, fallback: &Path) -> PathBuf {
    file.ancestors().skip(1).find(|a| a.file_name().is_some_and(|n| n.eq_ignore_ascii_case(name))).unwrap_or(fallback).to_path_buf()
}

fn rel_name(path: &Path, root: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?.with_extension("");
    Some(rel.to_string_lossy().replace('\\', "/").to_ascii_lowercase())
}

/// Scans folders and files for convertible textures. A Valve material is named by its path below the nearest
/// `materials` folder and a `.wal` by its path below the nearest `textures` folder, the way the games name them,
/// falling back to the scanned folder. The first source of a name wins.
pub fn scan(paths: &[PathBuf]) -> Library {
    let mut lib = Library::default();
    let mut root_ids: HashMap<PathBuf, usize> = HashMap::new();
    for base in paths {
        let mut files = Vec::new();
        if base.is_dir() {
            walk(base, &mut files);
        } else if base.is_file() {
            files.push(base.clone());
        }

        let scan_root = if base.is_dir() { base.clone() } else { base.parent().map(Path::to_path_buf).unwrap_or_default() };
        files.sort();
        for f in files {
            match ext(&f).as_str() {
                "vmt" | "vtf" => {
                    let root = root_named(&f, "materials", &scan_root);
                    let next = root_ids.len();
                    let id = *root_ids.entry(root.clone()).or_insert(next);
                    if id == lib.roots.len() {
                        lib.roots.push(HashMap::new());
                    }

                    let Ok(rel) = f.strip_prefix(&root) else { continue };
                    lib.roots[id].insert(rel.to_string_lossy().replace('\\', "/").to_ascii_lowercase(), f.clone());
                    if ext(&f) == "vmt"
                        && let Some(name) = rel_name(&f, &root)
                    {
                        lib.sources.entry(name).or_insert(Source::Valve { vmt: f.clone(), root: id });
                    }
                }
                "wad" => match std::fs::read(&f).map_err(|e| e.to_string()).and_then(|b| quake_texture::read_wad(&b).map_err(|e| e.to_string())) {
                    Ok(textures) => {
                        for t in textures {
                            lib.sources.entry(t.name.to_ascii_lowercase()).or_insert(Source::Paletted(t));
                        }
                    }
                    Err(e) => lib.errors.push((f.clone(), e)),
                },
                "wal" => {
                    let Some(name) = rel_name(&f, &root_named(&f, "textures", &scan_root)) else { continue };
                    match std::fs::read(&f).map_err(|e| e.to_string()).and_then(|b| quake_texture::read_wal(&b, &name).map_err(|e| e.to_string())) {
                        Ok(t) => {
                            lib.sources.entry(name).or_insert(Source::Paletted(t));
                        }
                        Err(e) => lib.errors.push((f.clone(), e)),
                    }
                }

                // Only below a `textures` folder, so screenshots and other pictures in a game folder stay out.
                e if IMAGE_EXTS.contains(&e) => {
                    let Some(root) = f.ancestors().skip(1).find(|a| a.file_name().is_some_and(|n| n.eq_ignore_ascii_case("textures"))) else { continue };
                    if let Some(name) = rel_name(&f, root) {
                        lib.sources.entry(name).or_insert(Source::Image(f.clone()));
                    }
                }
                _ => {}
            }
        }
    }

    lib
}

/// Folders and files near a map that likely hold its textures: `materials` folders next to it or up to four
/// folders up, and the WADs its worldspawn `wad` key lists. A WAD that is not where the key says is looked for by
/// name in the same places and in their subfolders.
pub fn sources_near(map: &Path, wad_key: Option<&str>) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let dirs: Vec<&Path> = map.ancestors().skip(1).take(5).collect();
    for d in &dirs {
        for sub in ["materials", "textures"] {
            let p = d.join(sub);
            if p.is_dir() && !out.contains(&p) {
                out.push(p);
            }
        }
    }

    for wad in wad_key.unwrap_or_default().split(';').map(str::trim).filter(|w| !w.is_empty()) {
        let wad = wad.replace('\\', "/");
        let direct = Path::new(&wad);
        let direct = if direct.is_absolute() { direct.to_path_buf() } else { dirs.first().map(|d| d.join(direct)).unwrap_or_default() };
        let found = direct.is_file().then_some(direct).or_else(|| {
            let name = Path::new(&wad).file_name()?.to_owned();
            dirs.iter().find_map(|d| {
                let here = d.join(&name);
                if here.is_file() {
                    return Some(here);
                }

                std::fs::read_dir(d).ok()?.flatten().map(|e| e.path().join(&name)).find(|p| p.is_file())
            })
        });
        if let Some(p) = found
            && !out.contains(&p)
        {
            out.push(p);
        }
    }

    out
}

/// The worldspawn `wad` key of a Quake `.map`, read without parsing the whole file.
pub fn wad_key(map_text: &str) -> Option<String> {
    map_text.lines().take(200).find_map(|l| {
        let l = l.trim();
        let rest = l.strip_prefix("\"wad\"")?.trim();
        Some(rest.trim_matches('"').to_string())
    })
}

/// Material settings a `.tres` is written for.
#[derive(Default)]
struct MaterialOut {
    albedo: Option<String>,
    normal: Option<String>,
    emission: Option<String>,
    /// transparency: 1 alpha blend, 2 scissor.
    transparency: u8,
    scissor: f32,
    additive: bool,
    double_sided: bool,
    unshaded: bool,
    color: Option<[f32; 4]>,
    surface_prop: Option<String>,
}

impl MaterialOut {
    fn needs_file(&self) -> bool {
        self.normal.is_some()
            || self.emission.is_some()
            || self.transparency != 0
            || self.additive
            || self.double_sided
            || self.unshaded
            || self.color.is_some()
            || self.surface_prop.is_some()
    }

    fn tres(&self, name: &str) -> String {
        let mut ext = String::new();
        let mut body = format!("resource_name = \"{name}\"\n");
        let mut n = 0;
        let mut texture = |key: &str, path: &Option<String>, body: &mut String| {
            if let Some(p) = path {
                n += 1;
                let _ = writeln!(ext, "[ext_resource type=\"Texture2D\" path=\"{p}\" id=\"{n}_{key}\"]");
                let _ = writeln!(body, "{key}_texture = ExtResource(\"{n}_{key}\")");
            }
        };
        texture("albedo", &self.albedo, &mut body);
        if self.normal.is_some() {
            body += "normal_enabled = true\n";
            texture("normal", &self.normal, &mut body);
        }

        if self.emission.is_some() {
            body += "emission_enabled = true\nemission = Color(0, 0, 0, 1)\n";
            texture("emission", &self.emission, &mut body);
        }

        if let Some(c) = self.color {
            let _ = writeln!(body, "albedo_color = Color({}, {}, {}, {})", c[0], c[1], c[2], c[3]);
        }

        match self.transparency {
            1 => body += "transparency = 1\n",
            2 => {
                let _ = writeln!(body, "transparency = 2\nalpha_scissor_threshold = {}", self.scissor);
            }
            _ => {}
        }

        if self.additive {
            body += "blend_mode = 1\n";
        }

        if self.double_sided {
            body += "cull_mode = 2\n";
        }

        if self.unshaded {
            body += "shading_mode = 0\n";
        }

        if let Some(s) = &self.surface_prop {
            let _ = writeln!(body, "metadata/surfaceprop = \"{}\"", s.replace('"', ""));
        }

        let head = if ext.is_empty() { String::new() } else { format!("\n{ext}") };
        format!("[gd_resource type=\"StandardMaterial3D\" format=3]\n{head}\n[resource]\n{body}")
    }
}

fn save_png(img: &Image, path: &Path) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }

    let pixels = img.rgba.as_chunks::<4>().0;
    let opaque = pixels.iter().all(|p| p[3] == 255);
    let result = if opaque {
        let rgb: Vec<u8> = pixels.iter().flat_map(|p| [p[0], p[1], p[2]]).collect();
        image::RgbImage::from_raw(img.width, img.height, rgb).ok_or("bad image size")?.save(path)
    } else {
        image::RgbaImage::from_raw(img.width, img.height, img.rgba.clone()).ok_or("bad image size")?.save(path)
    };
    result.map_err(|e| e.to_string())
}

impl Library {
    /// Face material names this library can provide.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.sources.keys()
    }

    pub fn contains(&self, material: &str) -> bool {
        self.sources.contains_key(&material.to_ascii_lowercase())
    }

    pub fn len(&self) -> usize {
        self.sources.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    fn valve_file(&self, root: usize, rel: &str) -> Option<&PathBuf> {
        self.roots.get(root)?.get(&rel.to_ascii_lowercase())
    }

    fn vtf(&self, root: usize, name: &str) -> Result<Image, String> {
        let path = self.valve_file(root, &format!("{name}.vtf")).ok_or_else(|| format!("{name}.vtf not found"))?;
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        vtf::decode(&bytes).map_err(|e| format!("{name}.vtf: {e}"))
    }

    /// Converts `only` (face material names), or everything when None. Existing outputs are kept unless `overwrite`.
    pub fn convert(&self, only: Option<&BTreeSet<String>>, target: &Target, overwrite: bool) -> Report {
        let mut report = Report::default();
        for (name, source) in &self.sources {
            if only.is_some_and(|o| !o.contains(name)) {
                continue;
            }

            let stem = file_stem(name);
            let image_ext = match source {
                Source::Image(p) => ext(p),
                _ => "png".into(),
            };
            let png = target.texture_dir.join(format!("{stem}.{image_ext}"));
            let tres = target.material_dir.join(format!("{stem}.{}", target.material_ext));
            let in_place = matches!(source, Source::Image(p) if p.canonicalize().ok().is_some_and(|p| png.canonicalize().ok() == Some(p)));
            if in_place || !overwrite && (png.is_file() || tres.is_file()) {
                report.existing.push(name.clone());
                continue;
            }

            let result = match source {
                Source::Valve { vmt, root } => self.convert_valve(name, vmt, *root, target, &png),
                Source::Paletted(t) => convert_paletted(t, target, &png),
                Source::Image(p) => png
                    .parent()
                    .map_or(Ok(()), std::fs::create_dir_all)
                    .and_then(|()| std::fs::copy(p, &png))
                    .map(|_| MaterialOut::default())
                    .map_err(|e| e.to_string()),
            };
            let result = result.and_then(|mat| {
                if mat.needs_file() {
                    if let Some(dir) = tres.parent() {
                        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
                    }

                    std::fs::write(&tres, mat.tres(name)).map_err(|e| e.to_string())?;
                } else if overwrite && tres.is_file() {
                    let _ = std::fs::remove_file(&tres);
                }

                Ok(())
            });
            match result {
                Ok(()) => report.converted.push(name.clone()),
                Err(e) => report.failed.push((name.clone(), e)),
            }
        }

        report
    }

    fn convert_valve(&self, name: &str, vmt_path: &Path, root: usize, target: &Target, png: &Path) -> Result<MaterialOut, String> {
        let text = crate::vmf::read_text(vmt_path).map_err(|e| e.to_string())?;
        let vmt: Vmt = vmt::parse(&text, &mut |include| {
            let path = self.valve_file(root, &format!("{}.vmt", vmt::normalize(include)))?;
            crate::vmf::read_text(path).ok()
        })
        .map_err(|e| e.to_string())?;
        let res = |p: &Path| to_res_path(&target.project_root, p);
        let stem = file_stem(name);
        let mut out =
            MaterialOut { surface_prop: vmt.surface_prop().map(str::to_string), double_sided: vmt.no_cull(), unshaded: vmt.unlit(), ..Default::default() };

        // A normal map is a nicety, one missing from the source folders does not stop the material.
        let normal = vmt.normal_map().and_then(|n| self.vtf(root, &n).ok());
        let mut base = match vmt.base_texture() {
            Some(b) => self.vtf(root, &b)?,
            // Water has no base texture, a tinted flat image the size of its normal map keeps the UVs right.
            None if vmt.water() || normal.is_some() => {
                let (w, h) = normal.as_ref().map(|n| (n.width, n.height)).unwrap_or((64, 64));
                out.color = Some([0.25, 0.4, 0.45, 0.6]);
                out.transparency = 1;
                Image { width: w, height: h, rgba: [255u8; 4].repeat((w * h) as usize) }
            }
            None => return Err("no $basetexture".into()),
        };

        if let Some(c) = vmt.color() {
            out.color = Some([c[0], c[1], c[2], out.color.map(|c| c[3]).unwrap_or(1.0)]);
        }

        if vmt.self_illum() {
            let mask = vmt.self_illum_mask().map(|m| self.vtf(root, &m)).transpose()?;
            let mask_at = |i: usize| match &mask {
                Some(m) if m.width == base.width && m.height == base.height => m.rgba[i * 4],
                Some(_) => 255,
                None => base.rgba[i * 4 + 3],
            };
            let rgba = (0..(base.width * base.height) as usize)
                .flat_map(|i| {
                    let m = mask_at(i) as u16;
                    [
                        (base.rgba[i * 4] as u16 * m / 255) as u8,
                        (base.rgba[i * 4 + 1] as u16 * m / 255) as u8,
                        (base.rgba[i * 4 + 2] as u16 * m / 255) as u8,
                        255,
                    ]
                })
                .collect();
            let path = target.texture_dir.join(format!("{stem}_emission.png"));
            save_png(&Image { width: base.width, height: base.height, rgba }, &path)?;
            out.emission = res(&path);
        }

        if vmt.translucent() || vmt.additive() {
            out.transparency = 1;
            out.additive = vmt.additive();
        } else if vmt.alpha_test() {
            out.transparency = 2;
            out.scissor = vmt.alpha_test_reference();
        } else if out.transparency == 0 {
            // The alpha of an opaque material holds a specular or self illumination mask, not coverage.
            for p in base.rgba.as_chunks_mut::<4>().0 {
                p[3] = 255;
            }
        }

        save_png(&base, png)?;
        out.albedo = res(png);
        if let Some(mut n) = normal {
            // Source normal maps point green down (DirectX), Godot expects it up, and the alpha is a specular mask.
            for p in n.rgba.as_chunks_mut::<4>().0 {
                p[1] = 255 - p[1];
                p[3] = 255;
            }

            let path = target.texture_dir.join(format!("{stem}_normal.png"));
            save_png(&n, &path)?;
            out.normal = res(&path);
        }

        Ok(out)
    }
}

fn convert_paletted(t: &Paletted, target: &Target, png: &Path) -> Result<MaterialOut, String> {
    let sky = t.is_sky();
    save_png(&if sky { t.sky_image() } else { t.to_rgba() }, png)?;
    let mut out = MaterialOut { albedo: to_res_path(&target.project_root, png), unshaded: sky, ..Default::default() };
    if t.alpha_test() {
        out.transparency = 2;
        out.scissor = 0.5;
    }

    let alpha = if t.flags & SURF_TRANS33 != 0 {
        Some(0.33)
    } else if t.flags & SURF_TRANS66 != 0 {
        Some(0.66)
    } else {
        None
    };
    if let Some(a) = alpha {
        out.transparency = 1;
        out.color = Some([1.0, 1.0, 1.0, a]);
    }

    // A `*liquid` name has no file of its own name, FuncGodot only finds it through the material file.
    if t.name.contains('*') && !out.needs_file() {
        out.color = Some([1.0; 4]);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::godot_material::{self, Transparency};

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gt_texture_import_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn target(project: &Path) -> Target {
        Target {
            project_root: project.to_path_buf(),
            texture_dir: project.join("textures"),
            material_dir: project.join("textures"),
            material_ext: "tres".into(),
        }
    }

    fn write(path: &Path, bytes: impl AsRef<[u8]>) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    #[test]
    fn converts_valve_materials_with_their_settings() {
        let dir = temp("valve");
        let game = dir.join("game/materials");
        // BGRA8888 2 x 2, alpha 0 where the self illumination mask is off.
        let bgra = crate::vtf::tests::build(2, 12, 2, 2, 1, 1, |_, _, _| vec![10, 20, 200, 255, 10, 20, 200, 0, 10, 20, 200, 255, 10, 20, 200, 0]);
        let normal = crate::vtf::tests::build(2, 12, 2, 2, 1, 1, |_, _, _| [255, 64, 128, 7].repeat(4));
        write(&game.join("Brick/Wall.vtf"), &bgra);
        write(&game.join("brick/wall_normal.vtf"), &normal);
        write(&game.join("brick/wall.vmt"), "LightmappedGeneric { $basetexture brick/wall $bumpmap brick/wall_normal $surfaceprop brick }");
        write(&game.join("brick/fence.vmt"), "\"LightmappedGeneric\" { \"$basetexture\" \"brick/wall\" \"$alphatest\" 1 \"$alphatestreference\" 0.4 }");
        write(&game.join("brick/sign.vmt"), "UnlitGeneric { $basetexture brick/wall $selfillum 1 }");
        write(&game.join("brick/sign_patch.vmt"), "patch { include \"materials/brick/sign.vmt\" insert { $translucent 1 } }");
        write(&game.join("brick/broken.vmt"), "LightmappedGeneric { $basetexture brick/nothing }");

        let lib = scan(&[dir.join("game")]);
        let names: Vec<&String> = lib.names().collect();
        assert_eq!(names, ["brick/broken", "brick/fence", "brick/sign", "brick/sign_patch", "brick/wall"]);
        let project = dir.join("project");
        let report = lib.convert(None, &target(&project), false);
        assert_eq!(report.converted.len(), 4, "{report:?}");
        assert_eq!(report.failed, vec![("brick/broken".to_string(), "brick/nothing.vtf not found".to_string())]);

        let tex = project.join("textures");
        let wall = image::open(tex.join("brick/wall.png")).unwrap().to_rgba8();
        assert_eq!(wall.get_pixel(0, 0).0, [200, 20, 10, 255]);
        assert_eq!(wall.get_pixel(1, 0).0[3], 255, "an opaque material drops the mask alpha");
        let n = image::open(tex.join("brick/wall_normal.png")).unwrap().to_rgba8();
        assert_eq!(n.get_pixel(0, 0).0, [128, 191, 255, 255], "green flipped, alpha dropped");
        let wall_mat = godot_material::parse(&std::fs::read_to_string(tex.join("brick/wall.tres")).unwrap()).unwrap();
        assert_eq!(wall_mat.albedo_texture.as_deref(), Some("res://textures/brick/wall.png"));
        assert_eq!(wall_mat.normal_texture.as_deref(), Some("res://textures/brick/wall_normal.png"));
        assert!(std::fs::read_to_string(tex.join("brick/wall.tres")).unwrap().contains("metadata/surfaceprop = \"brick\""));

        let fence = godot_material::parse(&std::fs::read_to_string(tex.join("brick/fence.tres")).unwrap()).unwrap();
        assert_eq!(fence.transparency, Transparency::Scissor(0.4));
        assert_eq!(image::open(tex.join("brick/fence.png")).unwrap().to_rgba8().get_pixel(1, 0).0[3], 0, "alpha test keeps the alpha");

        let sign = godot_material::parse(&std::fs::read_to_string(tex.join("brick/sign.tres")).unwrap()).unwrap();
        assert!(sign.unshaded && sign.is_emissive());
        let glow = image::open(tex.join("brick/sign_emission.png")).unwrap().to_rgba8();
        assert_eq!((glow.get_pixel(0, 0).0, glow.get_pixel(1, 0).0), ([200, 20, 10, 255], [0, 0, 0, 255]), "alpha masks the glow");
        let patched = godot_material::parse(&std::fs::read_to_string(tex.join("brick/sign_patch.tres")).unwrap()).unwrap();
        assert!(patched.is_transparent() && patched.unshaded, "patch keeps the included shader and adds $translucent");

        let again = lib.convert(Some(&BTreeSet::from(["brick/wall".to_string()])), &target(&project), false);
        assert_eq!((again.converted.len(), again.existing.len()), (0, 1));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn converts_wad_and_wal_textures() {
        let dir = temp("quake");
        write(
            &dir.join("src/wads/base.wad"),
            quake_texture::tests::wad(b"WAD2", &[("wall1", 8, 8, 3), ("{fence", 8, 8, 4), ("*water1", 8, 8, 5), ("sky1", 16, 8, 6)]),
        );
        let mut wal = vec![0u8; 100];
        wal[32..36].copy_from_slice(&2u32.to_le_bytes());
        wal[36..40].copy_from_slice(&2u32.to_le_bytes());
        wal[40..44].copy_from_slice(&100u32.to_le_bytes());
        wal[88..92].copy_from_slice(&SURF_TRANS33.to_le_bytes());
        wal.extend([1, 2, 3, 4]);
        write(&dir.join("baseq2/textures/e1u1/Glass.wal"), &wal);

        std::fs::create_dir_all(dir.join("baseq2/textures/base")).unwrap();
        image::RgbImage::from_pixel(2, 2, image::Rgb([1, 2, 3])).save(dir.join("baseq2/textures/base/grid.jpg")).unwrap();

        let lib = scan(&[dir.join("src"), dir.join("baseq2")]);
        let project = dir.join("project");
        let only: BTreeSet<String> = ["wall1", "{fence", "*water1", "sky1", "e1u1/glass", "base/grid"].map(String::from).into();
        let report = lib.convert(Some(&only), &target(&project), false);
        assert_eq!(report.converted.len(), 6, "{report:?}");
        let tex = project.join("textures");
        assert!(tex.join("base/grid.jpg").is_file(), "a loose image is copied with its format");
        assert!(tex.join("wall1.png").is_file() && !tex.join("wall1.tres").exists(), "a plain texture needs no material file");
        let fence = godot_material::parse(&std::fs::read_to_string(tex.join("{fence.tres")).unwrap()).unwrap();
        assert!(matches!(fence.transparency, Transparency::Scissor(_)));
        assert!(tex.join("water1.png").is_file() && tex.join("water1.tres").is_file(), "* dropped from the file name");
        assert!(godot_material::parse(&std::fs::read_to_string(tex.join("sky1.tres")).unwrap()).unwrap().unshaded);
        assert_eq!(image::image_dimensions(tex.join("sky1.png")).unwrap(), (8, 8), "the back half of the sky");
        let glass = godot_material::parse(&std::fs::read_to_string(tex.join("e1u1/glass.tres")).unwrap()).unwrap();
        assert!(glass.is_transparent() && (glass.albedo_color[3] - 0.33).abs() < 1e-6);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn finds_textures_near_a_map() {
        let dir = temp("near");
        write(&dir.join("mod/maps/src/e1/e1m1.map"), "");
        write(&dir.join("mod/texture-wads/tech.wad"), "WAD2");
        write(&dir.join("mod/maps/src/local.wad"), "WAD2");
        std::fs::create_dir_all(dir.join("mod/materials")).unwrap();
        let map = dir.join("mod/maps/src/e1/e1m1.map");
        let found = sources_near(&map, Some("../../../../texture-wads/tech.wad; ../local.wad;missing.wad"));
        assert!(found.contains(&dir.join("mod/materials")));
        assert!(found.contains(&dir.join("mod/texture-wads/tech.wad")), "found by name one folder over: {found:?}");
        assert!(found.iter().any(|p| p.ends_with("local.wad")));
        assert_eq!(wad_key("{\n\"classname\" \"worldspawn\"\n\"wad\" \"a.wad;b.wad\"\n").as_deref(), Some("a.wad;b.wad"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
