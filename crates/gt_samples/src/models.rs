//! Blockbench models for the showcase, written as real `.bbmodel` files with embedded textures.

use base64::Engine;
use image::RgbaImage;
use serde_json::{Value, json};

use crate::textures::{hash, value_noise};

/// Atlas regions in a 32x32 texture: bark on the left quarter, foliage in the rest.
const BARK: [f64; 4] = [0.0, 0.0, 8.0, 32.0];
const FOLIAGE: [f64; 4] = [8.0, 0.0, 32.0, 32.0];
const STONE: [f64; 4] = [0.0, 0.0, 32.0, 32.0];

fn png_data_url(img: &RgbaImage) -> String {
    let mut bytes = std::io::Cursor::new(Vec::new());
    img.write_to(&mut bytes, image::ImageFormat::Png).expect("png encodes");
    format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes.into_inner()))
}

fn tree_atlas(leaf: [f32; 3], seed: u32) -> RgbaImage {
    RgbaImage::from_fn(32, 32, |x, y| {
        if x < 8 {
            let k = 0.8 + ((x % 4 == 0) as u8 as f32) * -0.25 + hash(x as i64, y as i64, seed) * 0.2;
            return image::Rgba([(98.0 * k) as u8, (68.0 * k) as u8, (44.0 * k) as u8, 255]);
        }
        let n = value_noise(x * 2, y * 2, 8, seed + 1);
        let k = 0.7 + n * 0.5 + if hash(x as i64, y as i64, seed + 2) > 0.85 { 0.2 } else { 0.0 };
        let hole = hash(x as i64 * 3, y as i64 * 5, seed + 3) < 0.08;
        image::Rgba([(leaf[0] * k).min(255.0) as u8, (leaf[1] * k).min(255.0) as u8, (leaf[2] * k).min(255.0) as u8, if hole { 0 } else { 255 }])
    })
}

fn stone_atlas(seed: u32) -> RgbaImage {
    RgbaImage::from_fn(32, 32, |x, y| {
        let k = 0.7 + value_noise(x * 2, y * 2, 8, seed) * 0.4 + hash(x as i64, y as i64, seed) * 0.15;
        image::Rgba([(130.0 * k) as u8, (126.0 * k) as u8, (118.0 * k) as u8, 255])
    })
}

struct Builder {
    elements: Vec<Value>,
    uuids: Vec<String>,
}

impl Builder {
    fn new() -> Self {
        Self { elements: Vec::new(), uuids: Vec::new() }
    }

    /// A cube with every face mapped to the same atlas region, rotated about its center.
    fn cube(&mut self, name: &str, from: [f64; 3], to: [f64; 3], rotation: [f64; 3], region: [f64; 4]) {
        let uuid = format!("{:08x}-0000-4000-8000-{:012x}", self.elements.len() + 1, self.elements.len() * 7919 + 17);
        let origin = [(from[0] + to[0]) * 0.5, (from[1] + to[1]) * 0.5, (from[2] + to[2]) * 0.5];
        let face = json!({ "uv": region, "texture": 0 });
        self.elements.push(json!({
            "name": name, "type": "cube", "uuid": uuid, "from": from, "to": to, "origin": origin, "rotation": rotation,
            "faces": { "north": face, "south": face, "east": face, "west": face, "up": face, "down": face }
        }));
        self.uuids.push(uuid);
    }

    fn finish(self, name: &str, texture_name: &str, texture: &RgbaImage) -> String {
        let model = json!({
            "meta": { "format_version": "4.10", "model_format": "free", "box_uv": false },
            "name": name,
            "model_identifier": "",
            "visible_box": [1, 1, 0],
            "resolution": { "width": 32, "height": 32 },
            "elements": self.elements,
            "outliner": [{ "name": name, "origin": [0, 0, 0], "rotation": [0, 0, 0], "uuid": "ffffffff-0000-4000-8000-000000000000", "children": self.uuids }],
            "textures": [{
                "path": "", "name": format!("{texture_name}.png"), "folder": "block", "namespace": "", "id": "0",
                "width": 32, "height": 32, "uv_width": 32, "uv_height": 32, "particle": false, "render_mode": "default",
                "source": png_data_url(texture)
            }]
        });
        serde_json::to_string_pretty(&model).expect("model serializes")
    }
}

/// Tall pine, about 11 m (176 Blockbench units).
pub fn pine() -> String {
    let mut b = Builder::new();
    b.cube("trunk", [-6.0, 0.0, -6.0], [6.0, 64.0, 6.0], [0.0, 0.0, 0.0], BARK);
    let tiers = [(40.0, 52.0, 88.0), (72.0, 42.0, 116.0), (104.0, 32.0, 140.0), (132.0, 20.0, 162.0)];
    for (i, (bottom, half, top)) in tiers.iter().enumerate() {
        let yaw = if i % 2 == 0 { 0.0 } else { 45.0 };
        b.cube(&format!("tier{i}"), [-half, *bottom, -half], [*half, *top - 8.0, *half], [0.0, yaw, 0.0], FOLIAGE);
    }
    b.cube("tip", [-5.0, 150.0, -5.0], [5.0, 176.0, 5.0], [0.0, 22.5, 0.0], FOLIAGE);
    b.finish("pine", "pine_atlas", &tree_atlas([46.0, 96.0, 54.0], 40))
}

/// Round broadleaf tree, about 8 m.
pub fn oak() -> String {
    let mut b = Builder::new();
    b.cube("trunk", [-8.0, 0.0, -8.0], [8.0, 56.0, 8.0], [0.0, 0.0, 0.0], BARK);
    b.cube("branch_a", [-4.0, 40.0, -30.0], [4.0, 48.0, 0.0], [-30.0, 0.0, 0.0], BARK);
    b.cube("branch_b", [0.0, 44.0, -4.0], [30.0, 52.0, 4.0], [0.0, 0.0, -25.0], BARK);
    let blobs = [
        ([-40.0, 56.0, -40.0], [40.0, 104.0, 40.0], 0.0),
        ([-30.0, 88.0, -26.0], [26.0, 128.0, 30.0], 30.0),
        ([-48.0, 64.0, -10.0], [-8.0, 96.0, 30.0], 15.0),
        ([6.0, 70.0, -44.0], [44.0, 102.0, -6.0], 50.0),
    ];
    for (i, (from, to, yaw)) in blobs.iter().enumerate() {
        b.cube(&format!("crown{i}"), *from, *to, [0.0, *yaw, 0.0], FOLIAGE);
    }
    b.finish("oak", "oak_atlas", &tree_atlas([70.0, 128.0, 52.0], 41))
}

pub fn bush() -> String {
    let mut b = Builder::new();
    b.cube("a", [-18.0, 0.0, -18.0], [18.0, 22.0, 18.0], [0.0, 20.0, 0.0], FOLIAGE);
    b.cube("b", [-12.0, 10.0, -14.0], [14.0, 30.0, 10.0], [0.0, 55.0, 0.0], FOLIAGE);
    b.finish("bush", "bush_atlas", &tree_atlas([62.0, 120.0, 50.0], 42))
}

pub fn rock() -> String {
    let mut b = Builder::new();
    b.cube("base", [-24.0, -4.0, -18.0], [22.0, 22.0, 20.0], [8.0, 20.0, -6.0], STONE);
    b.cube("top", [-14.0, 12.0, -12.0], [12.0, 30.0, 10.0], [-10.0, 55.0, 12.0], STONE);
    b.cube("chip", [10.0, -2.0, 6.0], [28.0, 10.0, 24.0], [0.0, 35.0, 18.0], STONE);
    b.finish("rock", "rock_atlas", &stone_atlas(43))
}

/// Every model as (file name, bbmodel text).
pub fn all() -> Vec<(&'static str, String)> {
    vec![("pine.bbmodel", pine()), ("oak.bbmodel", oak()), ("bush.bbmodel", bush()), ("rock.bbmodel", rock())]
}
