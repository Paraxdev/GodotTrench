//! Built-in nature pack: low poly Blockbench trees, rocks and foliage with embedded pixel textures.
//! The editor installs these into a project so the scatter presets work without any assets.

use base64::Engine;
use image::{Rgba, RgbaImage};
use serde_json::{Value, json};

/// Every model as (name, bbmodel text).
pub fn all() -> Vec<(&'static str, String)> {
    vec![
        ("pine", pine()),
        ("oak", oak()),
        ("birch", birch()),
        ("bush", bush()),
        ("fern", fern()),
        ("rock", rock()),
        ("boulder", boulder()),
        ("grass", grass()),
        ("flowers", flowers()),
    ]
}

pub fn names() -> Vec<&'static str> {
    all().into_iter().map(|(n, _)| n).collect()
}

/// Writes missing (or all, with `overwrite`) models into `dir`. Returns the files written.
pub fn install(dir: &std::path::Path, overwrite: bool) -> std::io::Result<Vec<std::path::PathBuf>> {
    std::fs::create_dir_all(dir)?;
    let mut out = Vec::new();
    for (name, text) in all() {
        let path = dir.join(format!("{name}.bbmodel"));
        if overwrite || !path.exists() {
            std::fs::write(&path, text)?;
            out.push(path);
        }
    }
    Ok(out)
}

fn hash(x: i64, y: i64, seed: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1) ^ seed.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    (h & 0xffff) as f32 / 65535.0
}

fn value_noise(x: u32, y: u32, cells: u32, seed: u32) -> f32 {
    let scale = 32.0 / cells as f32;
    let (fx, fy) = (x as f32 / scale, y as f32 / scale);
    let (ix, iy) = (fx.floor() as i64, fy.floor() as i64);
    let (tx, ty) = (fx - ix as f32, fy - iy as f32);
    let cell = |cx: i64, cy: i64| hash(cx.rem_euclid(cells as i64) * 7, cy.rem_euclid(cells as i64) * 13, seed);
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let a = cell(ix, iy) + (cell(ix + 1, iy) - cell(ix, iy)) * s(tx);
    let b = cell(ix, iy + 1) + (cell(ix + 1, iy + 1) - cell(ix, iy + 1)) * s(tx);
    a + (b - a) * s(ty)
}

fn shade(c: [f32; 3], k: f32, a: u8) -> Rgba<u8> {
    Rgba([(c[0] * k).clamp(0.0, 255.0) as u8, (c[1] * k).clamp(0.0, 255.0) as u8, (c[2] * k).clamp(0.0, 255.0) as u8, a])
}

fn png_data_url(img: &RgbaImage) -> String {
    let mut bytes = std::io::Cursor::new(Vec::new());
    img.write_to(&mut bytes, image::ImageFormat::Png).expect("png encodes");
    format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes.into_inner()))
}

/// Atlas regions in a 32x32 texture: bark on the left quarter, foliage in the rest.
const BARK: [f64; 4] = [0.0, 0.0, 8.0, 32.0];
const FOLIAGE: [f64; 4] = [8.0, 0.0, 32.0, 32.0];
const FULL: [f64; 4] = [0.0, 0.0, 32.0, 32.0];

fn tree_atlas(bark: [f32; 3], leaf: [f32; 3], striped: bool, seed: u32) -> RgbaImage {
    RgbaImage::from_fn(32, 32, |x, y| {
        if x < 8 {
            let stripe = striped && hash(0, y as i64 / 3, seed) > 0.72 && x % 7 != 0;
            let k = 0.8 + ((x % 4 == 0) as u8 as f32) * -0.25 + hash(x as i64, y as i64, seed) * 0.2;
            return if stripe { shade([40.0, 36.0, 34.0], 1.0, 255) } else { shade(bark, k, 255) };
        }
        let n = value_noise(x * 2, y * 2, 8, seed + 1);
        let k = 0.7 + n * 0.5 + if hash(x as i64, y as i64, seed + 2) > 0.85 { 0.2 } else { 0.0 };
        let hole = hash(x as i64 * 3, y as i64 * 5, seed + 3) < 0.08;
        shade(leaf, k, if hole { 0 } else { 255 })
    })
}

fn stone_atlas(base: [f32; 3], moss: bool, seed: u32) -> RgbaImage {
    RgbaImage::from_fn(32, 32, |x, y| {
        let k = 0.7 + value_noise(x * 2, y * 2, 8, seed) * 0.4 + hash(x as i64, y as i64, seed) * 0.15;
        if moss && y < 9 && value_noise(x, y, 4, seed + 5) > 0.45 {
            return shade([78.0, 112.0, 58.0], k, 255);
        }
        shade(base, k, 255)
    })
}

/// Blades or fronds on transparent background: `blade(x, y)` returns a brightness where the card is solid.
fn card_atlas(color: [f32; 3], seed: u32, blade: impl Fn(u32, u32) -> Option<f32>) -> RgbaImage {
    RgbaImage::from_fn(32, 32, |x, y| match blade(x, y) {
        Some(k) => shade(color, k * (0.85 + hash(x as i64, y as i64, seed) * 0.3), 255),
        None => Rgba([0, 0, 0, 0]),
    })
}

struct Model {
    elements: Vec<Value>,
    uuids: Vec<String>,
}

impl Model {
    fn new() -> Self {
        Self { elements: Vec::new(), uuids: Vec::new() }
    }

    fn uuid(&self) -> String {
        format!("{:08x}-0000-4000-8000-{:012x}", self.elements.len() + 1, self.elements.len() * 7919 + 17)
    }

    /// A cube with every face mapped to the same atlas region, rotated about its center.
    fn cube(&mut self, name: &str, from: [f64; 3], to: [f64; 3], rotation: [f64; 3], region: [f64; 4]) {
        let uuid = self.uuid();
        let origin = [(from[0] + to[0]) * 0.5, (from[1] + to[1]) * 0.5, (from[2] + to[2]) * 0.5];
        let face = json!({ "uv": region, "texture": 0 });
        self.elements.push(json!({
            "name": name, "type": "cube", "uuid": uuid, "from": from, "to": to, "origin": origin, "rotation": rotation,
            "faces": { "north": face, "south": face, "east": face, "west": face, "up": face, "down": face }
        }));
        self.uuids.push(uuid);
    }

    /// A double sided card: a quad from `a` to `b` along the ground, `height` tall, leaning by `lean` along its normal.
    fn card(&mut self, name: &str, a: [f64; 2], b: [f64; 2], height: f64, lean: f64, region: [f64; 4]) {
        let uuid = self.uuid();
        let (dx, dz) = (b[0] - a[0], b[1] - a[1]);
        let len = (dx * dx + dz * dz).sqrt().max(1e-6);
        let (nx, nz) = (-dz / len * lean, dx / len * lean);
        let verts = json!({
            "p0": [a[0], 0.0, a[1]], "p1": [b[0], 0.0, b[1]],
            "p2": [b[0] + nx, height, b[1] + nz], "p3": [a[0] + nx, height, a[1] + nz]
        });
        let [u0, v0, u1, v1] = region;
        let uv = json!({ "p0": [u0, v1], "p1": [u1, v1], "p2": [u1, v0], "p3": [u0, v0] });
        self.elements.push(json!({
            "name": name, "type": "mesh", "uuid": uuid, "origin": [0, 0, 0], "rotation": [0, 0, 0],
            "vertices": verts,
            "faces": {
                "front": { "vertices": ["p0", "p1", "p2", "p3"], "uv": uv, "texture": 0 },
                "back": { "vertices": ["p3", "p2", "p1", "p0"], "uv": uv, "texture": 0 }
            }
        }));
        self.uuids.push(uuid);
    }

    fn finish(self, name: &str, texture: &RgbaImage) -> String {
        let model = json!({
            "meta": { "format_version": "4.10", "model_format": "free", "box_uv": false },
            "name": name,
            "model_identifier": "",
            "visible_box": [1, 1, 0],
            "resolution": { "width": 32, "height": 32 },
            "elements": self.elements,
            "outliner": [{ "name": name, "origin": [0, 0, 0], "rotation": [0, 0, 0], "uuid": "ffffffff-0000-4000-8000-000000000000", "children": self.uuids }],
            "textures": [{
                "path": "", "name": format!("{name}_atlas.png"), "folder": "block", "namespace": "", "id": "0",
                "width": 32, "height": 32, "uv_width": 32, "uv_height": 32, "particle": false, "render_mode": "default",
                "source": png_data_url(texture)
            }]
        });
        serde_json::to_string_pretty(&model).expect("model serializes")
    }
}

/// Tall pine, about 11 m.
pub fn pine() -> String {
    let mut m = Model::new();
    m.cube("trunk", [-6.0, 0.0, -6.0], [6.0, 64.0, 6.0], [0.0, 0.0, 0.0], BARK);
    let tiers = [(40.0, 52.0, 88.0), (72.0, 42.0, 116.0), (104.0, 32.0, 140.0), (132.0, 20.0, 162.0)];
    for (i, (bottom, half, top)) in tiers.iter().enumerate() {
        let yaw = if i % 2 == 0 { 0.0 } else { 45.0 };
        m.cube(&format!("tier{i}"), [-half, *bottom, -half], [*half, *top - 8.0, *half], [0.0, yaw, 0.0], FOLIAGE);
    }
    m.cube("tip", [-5.0, 150.0, -5.0], [5.0, 176.0, 5.0], [0.0, 22.5, 0.0], FOLIAGE);
    m.finish("pine", &tree_atlas([98.0, 68.0, 44.0], [46.0, 96.0, 54.0], false, 40))
}

/// Round broadleaf tree, about 8 m.
pub fn oak() -> String {
    let mut m = Model::new();
    m.cube("trunk", [-8.0, 0.0, -8.0], [8.0, 56.0, 8.0], [0.0, 0.0, 0.0], BARK);
    m.cube("branch_a", [-4.0, 40.0, -30.0], [4.0, 48.0, 0.0], [-30.0, 0.0, 0.0], BARK);
    m.cube("branch_b", [0.0, 44.0, -4.0], [30.0, 52.0, 4.0], [0.0, 0.0, -25.0], BARK);
    let blobs = [
        ([-40.0, 56.0, -40.0], [40.0, 104.0, 40.0], 0.0),
        ([-30.0, 88.0, -26.0], [26.0, 128.0, 30.0], 30.0),
        ([-48.0, 64.0, -10.0], [-8.0, 96.0, 30.0], 15.0),
        ([6.0, 70.0, -44.0], [44.0, 102.0, -6.0], 50.0),
    ];
    for (i, (from, to, yaw)) in blobs.iter().enumerate() {
        m.cube(&format!("crown{i}"), *from, *to, [0.0, *yaw, 0.0], FOLIAGE);
    }
    m.finish("oak", &tree_atlas([98.0, 68.0, 44.0], [70.0, 128.0, 52.0], false, 41))
}

/// Slender birch with a light striped trunk, about 9 m.
pub fn birch() -> String {
    let mut m = Model::new();
    m.cube("trunk", [-4.0, 0.0, -4.0], [4.0, 110.0, 4.0], [0.0, 0.0, 3.0], BARK);
    m.cube("branch", [0.0, 70.0, -3.0], [22.0, 76.0, 3.0], [0.0, 30.0, 35.0], BARK);
    let blobs =
        [([-22.0, 78.0, -20.0], [22.0, 120.0, 20.0], 10.0), ([-16.0, 108.0, -14.0], [14.0, 142.0, 16.0], 40.0), ([4.0, 88.0, -6.0], [30.0, 112.0, 20.0], 25.0)];
    for (i, (from, to, yaw)) in blobs.iter().enumerate() {
        m.cube(&format!("crown{i}"), *from, *to, [0.0, *yaw, 0.0], FOLIAGE);
    }
    m.finish("birch", &tree_atlas([226.0, 222.0, 208.0], [132.0, 168.0, 70.0], true, 44))
}

pub fn bush() -> String {
    let mut m = Model::new();
    m.cube("a", [-18.0, 0.0, -18.0], [18.0, 22.0, 18.0], [0.0, 20.0, 0.0], FOLIAGE);
    m.cube("b", [-12.0, 10.0, -14.0], [14.0, 30.0, 10.0], [0.0, 55.0, 0.0], FOLIAGE);
    m.finish("bush", &tree_atlas([98.0, 68.0, 44.0], [62.0, 120.0, 50.0], false, 42))
}

/// Five fronds leaning outwards.
pub fn fern() -> String {
    let mut m = Model::new();
    for k in 0..5 {
        let a = k as f64 * std::f64::consts::TAU / 5.0 + 0.3;
        let (c, s) = (a.cos(), a.sin());
        let (w, reach) = (5.0, 3.0);
        m.card(&format!("frond{k}"), [-s * w + c * reach, c * w + s * reach], [s * w + c * reach, -c * w + s * reach], 16.0, 9.0, FULL);
    }
    let atlas = card_atlas([70.0, 132.0, 60.0], 50, |x, y| {
        let spine = 16.0;
        let t = y as f32 / 31.0;
        let half = 14.0 * (1.0 - t) * (0.6 + 0.4 * ((y as f32 * 1.3).sin().abs()));
        ((x as f32 - spine).abs() <= half.max(1.0)).then_some(0.8 + t * 0.4)
    });
    m.finish("fern", &atlas)
}

pub fn rock() -> String {
    let mut m = Model::new();
    m.cube("base", [-24.0, -4.0, -18.0], [22.0, 22.0, 20.0], [8.0, 20.0, -6.0], FULL);
    m.cube("top", [-14.0, 12.0, -12.0], [12.0, 30.0, 10.0], [-10.0, 55.0, 12.0], FULL);
    m.cube("chip", [10.0, -2.0, 6.0], [28.0, 10.0, 24.0], [0.0, 35.0, 18.0], FULL);
    m.finish("rock", &stone_atlas([130.0, 126.0, 118.0], false, 43))
}

pub fn boulder() -> String {
    let mut m = Model::new();
    m.cube("core", [-40.0, -8.0, -34.0], [38.0, 44.0, 36.0], [6.0, 12.0, -4.0], FULL);
    m.cube("cap", [-28.0, 30.0, -24.0], [26.0, 58.0, 22.0], [-8.0, 40.0, 10.0], FULL);
    m.cube("side", [20.0, -6.0, -20.0], [52.0, 26.0, 18.0], [0.0, -20.0, 14.0], FULL);
    m.cube("chunk", [-50.0, -6.0, 6.0], [-22.0, 18.0, 36.0], [10.0, 25.0, 0.0], FULL);
    m.finish("boulder", &stone_atlas([118.0, 116.0, 112.0], true, 45))
}

/// Three crossed double sided grass cards.
pub fn grass() -> String {
    let mut m = Model::new();
    for k in 0..3 {
        let a = k as f64 * std::f64::consts::PI / 3.0;
        let (c, s) = (a.cos() * 9.0, a.sin() * 9.0);
        m.card(&format!("tuft{k}"), [-c, -s], [c, s], 12.0, 0.0, FULL);
    }
    let atlas = card_atlas([92.0, 150.0, 64.0], 51, |x, y| {
        let blade = hash(x as i64 / 2, 0, 52);
        let top = 31.0 - blade * 26.0;
        ((y as f32) >= top && (x % 2 == 0 || hash(x as i64, y as i64, 53) > 0.6)).then_some(0.7 + (31.0 - y as f32) / 31.0 * 0.6)
    });
    m.finish("grass", &atlas)
}

/// Crossed cards with small flower heads on stems.
pub fn flowers() -> String {
    let mut m = Model::new();
    for k in 0..2 {
        let a = k as f64 * std::f64::consts::FRAC_PI_2 + 0.4;
        let (c, s) = (a.cos() * 7.0, a.sin() * 7.0);
        m.card(&format!("stems{k}"), [-c, -s], [c, s], 11.0, 0.0, FULL);
    }
    let petals = [[236.0, 214.0, 70.0], [230.0, 110.0, 160.0], [240.0, 240.0, 235.0]];
    let img = RgbaImage::from_fn(32, 32, |x, y| {
        let stem_x = (x / 8) * 8 + 4;
        let head_y = 6 + (hash(x as i64 / 8, 0, 54) * 10.0) as u32;
        let color = petals[(hash(x as i64 / 8, 1, 55) * 3.0) as usize % 3];
        let (dx, dy) = (x as i32 - stem_x as i32, y as i32 - head_y as i32);
        if dx * dx + dy * dy <= 5 {
            return shade(color, 0.9 + hash(x as i64, y as i64, 56) * 0.2, 255);
        }
        if x == stem_x && y > head_y {
            return shade([70.0, 128.0, 58.0], 1.0, 255);
        }
        Rgba([0, 0, 0, 0])
    });
    m.finish("flowers", &img)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_model_parses_with_textures() {
        for (name, text) in all() {
            let model = crate::bbmodel::parse(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(!model.textures[0].png.is_empty(), "{name} embeds its texture");
            assert!(model.polygons().count() >= 4, "{name}");
            let b = model.bounds();
            assert!(b.size().y > 8.0 && b.min.y >= -16.0, "{name} stands on the ground: {b:?}");
        }
    }
}
