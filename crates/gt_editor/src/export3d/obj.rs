//! Wavefront OBJ with its `.mtl` and the textures copied into a folder next to it. OBJ has no hierarchy or
//! instancing, so every object is written in world space.

use std::collections::{HashMap, HashSet};
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::Path;

use gt_core::{DMat3, DMat4, DVec3};

use super::collect::Scene;
use super::textures::{Alpha, to_srgb};
use super::{CANCELLED, Progress};

/// A name OBJ and MTL readers take as one word.
fn word(name: &str) -> String {
    let w: String = name.trim().chars().map(|c| if c.is_whitespace() { '_' } else { c }).collect();
    if w.is_empty() { "unnamed".into() } else { w }
}

/// Writes `path`, `<stem>.mtl` and `<stem>_textures/` next to it. Returns the bytes written.
pub fn write(scene: &Scene, path: &Path, progress: &Progress) -> Result<u64, String> {
    let dir = path.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let stem = word(&path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "map".into()));
    let mtl_name = format!("{stem}.mtl");
    let tex_dir = format!("{stem}_textures");
    let fail = |p: &Path, e: std::io::Error| format!("cannot write {}: {e}", p.display());

    // The geometry goes first, the bulk of the work. Streamed through a temporary file, a cancel or a full disk leaves
    // neither a half written OBJ nor materials without one.
    progress.begin(true, scene.triangles());
    let mut written = gt_formats::write_atomic_with(path, |out| geometry(scene, &mtl_name, out, progress))
        .map_err(|e| if progress.is_cancelled() { CANCELLED.to_string() } else { fail(path, e) })?;

    // Texture files, named after their source and made unique.
    let mut files: HashMap<usize, String> = HashMap::new();
    let mut taken: HashSet<String> = HashSet::new();
    for m in &scene.materials.list {
        for image in [m.albedo, m.normal, m.roughness_map, m.emissive_texture].into_iter().flatten() {
            if files.contains_key(&image) {
                continue;
            }

            let img = &scene.materials.images[image];
            let base = word(&img.name.replace(['/', '\\', ':'], "_"));
            let mut file = format!("{base}.{}", img.extension());
            let mut n = 2;
            while !taken.insert(file.to_ascii_lowercase()) {
                file = format!("{base}_{n}.{}", img.extension());
                n += 1;
            }

            let target = dir.join(&tex_dir).join(&file);
            std::fs::create_dir_all(dir.join(&tex_dir)).map_err(|e| fail(&target, e))?;
            std::fs::write(&target, &img.bytes).map_err(|e| fail(&target, e))?;
            written += img.bytes.len() as u64;
            files.insert(image, format!("{tex_dir}/{file}"));
        }
    }

    let mut mtl = format!("# GodotTrench {} materials\n", crate::VERSION);
    for m in &scene.materials.list {
        let c = m.base_color;
        let _ = writeln!(mtl, "\nnewmtl {}", word(&m.name));
        let _ = writeln!(mtl, "Kd {:.4} {:.4} {:.4}", to_srgb(c[0]), to_srgb(c[1]), to_srgb(c[2]));
        let _ = writeln!(mtl, "Ks 0 0 0\nillum 2");
        let _ = writeln!(mtl, "Pr {:.3}\nPm {:.3}", m.roughness, m.metallic);
        if m.alpha == Alpha::Blend {
            let _ = writeln!(mtl, "d {:.3}", c[3]);
        }

        if m.is_emissive() {
            let e = m.emissive.map(|v| to_srgb(v * m.emissive_strength.min(1.0)));
            let _ = writeln!(mtl, "Ke {:.4} {:.4} {:.4}", e[0], e[1], e[2]);
        }

        let map = |image: Option<usize>| image.and_then(|i| files.get(&i));
        if let Some(f) = map(m.albedo) {
            let _ = writeln!(mtl, "map_Kd {f}");
            if m.alpha != Alpha::Opaque {
                let _ = writeln!(mtl, "map_d {f}");
            }
        }

        match map(m.normal) {
            Some(f) if m.normal_scale != 1.0 => {
                let _ = writeln!(mtl, "map_Bump -bm {:.3} {f}", m.normal_scale);
            }
            Some(f) => {
                let _ = writeln!(mtl, "map_Bump {f}");
            }
            None => {}
        }

        if let Some(f) = map(m.roughness_map) {
            let _ = writeln!(mtl, "map_Pr {f}");
        }

        if let Some(f) = map(m.emissive_texture) {
            let _ = writeln!(mtl, "map_Ke {f}");
        }
    }

    let mtl_path = dir.join(&mtl_name);
    std::fs::write(&mtl_path, &mtl).map_err(|e| fail(&mtl_path, e))?;
    Ok(written + mtl.len() as u64)
}

/// The objects in world space, a scatter instance as a full copy of its model. Returns the bytes written.
fn geometry(scene: &Scene, mtl_name: &str, out: &mut impl std::io::Write, progress: &Progress) -> std::io::Result<u64> {
    let mut out = Counted { out, bytes: 0 };
    writeln!(
        out,
        "# {} exported by GodotTrench {}. Geometry and materials only, entity logic is not included.\n# Meters, Y up.\nmtllib {mtl_name}",
        scene.name,
        crate::VERSION
    )?;
    let mut next = 1usize;
    let mut stack: Vec<(usize, DMat4)> = scene.roots.iter().rev().map(|r| (*r, DMat4::IDENTITY)).collect();
    while let Some((index, parent)) = stack.pop() {
        if progress.is_cancelled() {
            return Err(std::io::Error::other(CANCELLED));
        }

        let node = &scene.nodes[index];
        let world = parent * node.matrix();
        stack.extend(node.children.iter().rev().map(|c| (*c, world)));
        let Some(mesh) = node.mesh.map(|m| &scene.meshes[m]) else { continue };
        let normal_matrix = DMat3::from_mat4(world).inverse().transpose();
        let flip = world.determinant() < 0.0;
        writeln!(out, "o {}", word(&node.name))?;
        for p in &mesh.primitives {
            for v in &p.positions {
                let w = world.transform_point3(DVec3::from(v.map(f64::from)));
                writeln!(out, "v {:.5} {:.5} {:.5}", w.x, w.y, w.z)?;
            }

            for t in &p.uvs {
                writeln!(out, "vt {:.5} {:.5}", t[0], 1.0 - t[1])?;
            }

            for n in &p.normals {
                let w = (normal_matrix * DVec3::from(n.map(f64::from))).normalize_or(DVec3::Y);
                writeln!(out, "vn {:.4} {:.4} {:.4}", w.x, w.y, w.z)?;
            }

            writeln!(out, "usemtl {}", word(&scene.materials.list[p.material].name))?;
            for tri in p.indices.as_chunks::<3>().0 {
                let [a, b, c] = tri.map(|i| i as usize + next);
                let (b, c) = if flip { (c, b) } else { (b, c) };
                writeln!(out, "f {a}/{a}/{a} {b}/{b}/{b} {c}/{c}/{c}")?;
            }

            next += p.positions.len();
            progress.advance(p.indices.len() / 3);
        }
    }

    Ok(out.bytes)
}

/// Counts the bytes going through to `out`.
struct Counted<W> {
    out: W,
    bytes: u64,
}

impl<W: std::io::Write> std::io::Write for Counted<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.out.write(buf)?;
        self.bytes += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.out.flush()
    }
}
