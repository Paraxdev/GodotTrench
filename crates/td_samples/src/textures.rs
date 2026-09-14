//! Procedural pixel art textures for the showcase maps. Every texture is 64x64 and tiles seamlessly.

use image::{Rgba, RgbaImage};

pub const SIZE: u32 = 64;

/// Integer hash noise in 0..1, wrapping at `SIZE` so textures tile.
pub fn hash(x: i64, y: i64, seed: u32) -> f32 {
    let x = x.rem_euclid(SIZE as i64) as u32;
    let y = y.rem_euclid(SIZE as i64) as u32;
    let mut h = x.wrapping_mul(0x27d4_eb2d) ^ y.wrapping_mul(0x1656_67b1) ^ seed.wrapping_mul(0x9e37_79b9);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    (h & 0xffff) as f32 / 65535.0
}

/// Smooth value noise with `cells` cells across the texture, tiling.
pub fn value_noise(x: u32, y: u32, cells: u32, seed: u32) -> f32 {
    let scale = SIZE as f32 / cells as f32;
    let fx = x as f32 / scale;
    let fy = y as f32 / scale;
    let (ix, iy) = (fx.floor() as i64, fy.floor() as i64);
    let (tx, ty) = (fx - ix as f32, fy - iy as f32);
    let cell = |cx: i64, cy: i64| hash(cx.rem_euclid(cells as i64) * 7, cy.rem_euclid(cells as i64) * 13, seed);
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let a = cell(ix, iy) + (cell(ix + 1, iy) - cell(ix, iy)) * s(tx);
    let b = cell(ix, iy + 1) + (cell(ix + 1, iy + 1) - cell(ix, iy + 1)) * s(tx);
    a + (b - a) * s(ty)
}

fn rgb(c: [f32; 3], k: f32) -> Rgba<u8> {
    Rgba([(c[0] * k).clamp(0.0, 255.0) as u8, (c[1] * k).clamp(0.0, 255.0) as u8, (c[2] * k).clamp(0.0, 255.0) as u8, 255])
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn speckled(base: [f32; 3], spread: f32, seed: u32) -> RgbaImage {
    RgbaImage::from_fn(SIZE, SIZE, |x, y| {
        let n = value_noise(x, y, 8, seed) * 0.6 + hash(x as i64, y as i64, seed + 1) * 0.4;
        rgb(base, 1.0 - spread + n * spread * 2.0)
    })
}

fn grass(base: [f32; 3], seed: u32) -> RgbaImage {
    RgbaImage::from_fn(SIZE, SIZE, |x, y| {
        let patch = value_noise(x, y, 4, seed);
        let blade = hash(x as i64, (y / 3) as i64, seed + 2);
        let k = 0.8
            + patch * 0.3
            + if blade > 0.82 {
                0.18
            } else if blade < 0.1 {
                -0.15
            } else {
                0.0
            };
        rgb(base, k)
    })
}

fn rock(base: [f32; 3], strata: bool, seed: u32) -> RgbaImage {
    RgbaImage::from_fn(SIZE, SIZE, |x, y| {
        let n = value_noise(x, y, 8, seed) * 0.5 + value_noise(x, y, 16, seed + 3) * 0.3 + hash(x as i64, y as i64, seed) * 0.2;
        let crack = (value_noise(x, y, 4, seed + 9) - 0.5).abs() < 0.03;
        let band = if strata { (y as f32 / 8.0 + value_noise(x, 0, 4, seed) * 2.0).sin() * 0.08 } else { 0.0 };
        rgb(base, if crack { 0.55 } else { 0.72 + n * 0.45 + band })
    })
}

fn planks(base: [f32; 3], board: u32, vertical: bool, seed: u32) -> RgbaImage {
    RgbaImage::from_fn(SIZE, SIZE, |x, y| {
        let (along, across) = if vertical { (y, x) } else { (x, y) };
        let row = across / board;
        let offset = (hash(row as i64, 0, seed) * SIZE as f32) as u32;
        let seam_across = across % board == 0;
        let seam_along = (along + offset).is_multiple_of(SIZE);
        let grain = value_noise(if vertical { x * 4 } else { x }, if vertical { y } else { y * 4 }, 16, seed + row);
        let tone = 0.85 + hash(row as i64, 1, seed) * 0.25;
        let nail = (along + offset) % SIZE == 3 && across % board == board / 2;
        rgb(
            base,
            if seam_across || seam_along {
                0.5
            } else if nail {
                0.4
            } else {
                tone * (0.85 + grain * 0.3)
            },
        )
    })
}

fn bricks(base: [f32; 3], mortar: [f32; 3], w: u32, h: u32, seed: u32) -> RgbaImage {
    RgbaImage::from_fn(SIZE, SIZE, |x, y| {
        let row = y / h;
        let shift = if row.is_multiple_of(2) { 0 } else { w / 2 };
        let col = (x + shift) / w;
        if y % h == 0 || (x + shift) % w == 0 {
            return rgb(mortar, 0.9 + hash(x as i64, y as i64, seed) * 0.2);
        }
        let tone = 0.8 + hash(col as i64, row as i64, seed) * 0.35;
        rgb(base, tone * (0.9 + hash(x as i64, y as i64, seed + 5) * 0.2))
    })
}

fn tiles_roof(base: [f32; 3], seed: u32) -> RgbaImage {
    RgbaImage::from_fn(SIZE, SIZE, |x, y| {
        let row = y / 8;
        let shift = if row % 2 == 0 { 0 } else { 4 };
        let lx = (x + shift) % 8;
        let ly = y % 8;
        let curve = ((lx as f32 - 3.5) / 4.0).powi(2);
        let edge = ly as f32 / 8.0 < 0.2 + curve * 0.3;
        let tone = 0.8 + hash(((x + shift) / 8) as i64, row as i64, seed) * 0.3;
        rgb(base, if edge { 0.55 } else { tone * (0.75 + ly as f32 / 8.0 * 0.35) })
    })
}

fn cobble(base: [f32; 3], seed: u32) -> RgbaImage {
    // Worley style cells from jittered points on an 8x8 grid.
    let points: Vec<(f32, f32)> =
        (0..64).map(|i| ((i % 8) as f32 * 8.0 + 1.0 + hash(i, 1, seed) * 6.0, (i / 8) as f32 * 8.0 + 1.0 + hash(i, 2, seed) * 6.0)).collect();
    RgbaImage::from_fn(SIZE, SIZE, |x, y| {
        let mut d1 = f32::MAX;
        let mut d2 = f32::MAX;
        let mut id = 0;
        for (i, (px, py)) in points.iter().enumerate() {
            let dx = ((x as f32 - px + 32.0).rem_euclid(64.0)) - 32.0;
            let dy = ((y as f32 - py + 32.0).rem_euclid(64.0)) - 32.0;
            let d = dx * dx + dy * dy;
            if d < d1 {
                d2 = d1;
                d1 = d;
                id = i;
            } else if d < d2 {
                d2 = d;
            }
        }
        if d2.sqrt() - d1.sqrt() < 1.2 {
            return rgb(base, 0.45);
        }
        rgb(base, 0.8 + hash(id as i64, 3, seed) * 0.35 - d1.sqrt() * 0.02)
    })
}

fn with_alpha(mut img: RgbaImage, keep: impl Fn(u32, u32) -> bool) -> RgbaImage {
    for (x, y, p) in img.enumerate_pixels_mut() {
        if !keep(x, y) {
            p.0[3] = 0;
        }
    }
    img
}

fn stained_glass(seed: u32) -> RgbaImage {
    let colors = [[190.0, 40.0, 50.0], [40.0, 80.0, 190.0], [220.0, 170.0, 40.0], [50.0, 140.0, 70.0], [120.0, 50.0, 150.0]];
    RgbaImage::from_fn(SIZE, SIZE, |x, y| {
        if x % 16 == 0 || y % 12 == 0 || (x + y) % 23 == 0 {
            return rgb([40.0, 40.0, 45.0], 1.0);
        }
        let c = colors[(hash((x / 16) as i64, (y / 12 + (x + y) / 23) as i64, seed) * colors.len() as f32) as usize % colors.len()];
        rgb(c, 0.85 + value_noise(x, y, 8, seed) * 0.3)
    })
}

/// Every showcase texture as (material name, image). Names are relative to the texture root.
pub fn all() -> Vec<(&'static str, RgbaImage)> {
    let mut out: Vec<(&'static str, RgbaImage)> = vec![
        ("showcase/grass", grass([78.0, 128.0, 56.0], 1)),
        ("showcase/grass_dark", grass([52.0, 96.0, 44.0], 2)),
        ("showcase/dirt", speckled([112.0, 84.0, 56.0], 0.18, 3)),
        ("showcase/sand", speckled([214.0, 198.0, 142.0], 0.08, 4)),
        ("showcase/snow", speckled([236.0, 240.0, 248.0], 0.04, 5)),
        ("showcase/rock", rock([128.0, 124.0, 116.0], false, 6)),
        ("showcase/cliff", rock([104.0, 96.0, 88.0], true, 7)),
        ("showcase/bark", planks([96.0, 66.0, 42.0], 6, true, 8)),
        ("showcase/planks", planks([150.0, 108.0, 66.0], 8, false, 9)),
        ("showcase/planks_dark", planks([92.0, 64.0, 42.0], 8, false, 10)),
        ("showcase/logs", planks([124.0, 86.0, 52.0], 16, false, 11)),
        ("showcase/beam", planks([78.0, 54.0, 36.0], 32, false, 12)),
        ("showcase/stone_bricks", bricks([150.0, 146.0, 136.0], [96.0, 92.0, 86.0], 16, 8, 13)),
        ("showcase/red_bricks", bricks([150.0, 72.0, 56.0], [180.0, 170.0, 160.0], 16, 8, 14)),
        ("showcase/cobble", cobble([132.0, 128.0, 122.0], 15)),
        ("showcase/plaster", speckled([224.0, 216.0, 196.0], 0.05, 16)),
        ("showcase/white_paint", planks([232.0, 232.0, 226.0], 16, true, 17)),
        ("showcase/red_paint", planks([176.0, 40.0, 36.0], 16, true, 18)),
        ("showcase/roof_red", tiles_roof([160.0, 70.0, 50.0], 19)),
        ("showcase/roof_slate", tiles_roof([74.0, 82.0, 96.0], 20)),
        ("showcase/metal", speckled([80.0, 84.0, 90.0], 0.1, 21)),
        ("showcase/gold", speckled([212.0, 170.0, 60.0], 0.12, 22)),
        ("showcase/lamp", speckled([255.0, 236.0, 170.0], 0.03, 23)),
        ("showcase/stained_glass", stained_glass(24)),
        (
            "showcase/carpet",
            RgbaImage::from_fn(SIZE, SIZE, |x, y| {
                let border = !(4..60).contains(&x);
                rgb(if border { [200.0, 160.0, 60.0] } else { [140.0, 30.0, 40.0] }, 0.9 + hash(x as i64, y as i64, 25) * 0.15)
            }),
        ),
        (
            "showcase/church_tiles",
            RgbaImage::from_fn(SIZE, SIZE, |x, y| {
                let dark = ((x / 16) + (y / 16)) % 2 == 0;
                rgb(if dark { [60.0, 58.0, 56.0] } else { [210.0, 204.0, 190.0] }, 0.92 + hash(x as i64, y as i64, 26) * 0.12)
            }),
        ),
        (
            "showcase/chalkboard",
            RgbaImage::from_fn(SIZE, SIZE, |x, y| {
                let chalk = (y % 12 == 5 && hash((x / 3) as i64, (y / 12) as i64, 27) > 0.35) || (x == y / 2 + 10 && y > 30);
                rgb(if chalk { [220.0, 225.0, 220.0] } else { [38.0, 72.0, 56.0] }, 0.9 + value_noise(x, y, 8, 27) * 0.15)
            }),
        ),
        (
            "showcase/water",
            RgbaImage::from_fn(SIZE, SIZE, |x, y| {
                let wave = ((x as f32 / 64.0 * std::f32::consts::TAU * 2.0 + y as f32 * 0.3).sin() * 0.5 + 0.5) * value_noise(x, y, 4, 28);
                rgb(mix([34.0, 84.0, 132.0], [90.0, 150.0, 190.0], wave * 0.6), 1.0)
            }),
        ),
        (
            "showcase/glass",
            RgbaImage::from_fn(SIZE, SIZE, |x, y| {
                let frame = x < 2 || y < 2 || x > 61 || y > 61;
                let streak = (x + y) % 40 < 3;
                rgb(
                    if frame {
                        [60.0, 60.0, 64.0]
                    } else if streak {
                        [220.0, 240.0, 250.0]
                    } else {
                        [150.0, 196.0, 214.0]
                    },
                    1.0,
                )
            }),
        ),
    ];
    out.push(("showcase/iron_bars", with_alpha(speckled([54.0, 54.0, 58.0], 0.15, 29), |x, y| x % 16 < 4 || y < 4 || (28..36).contains(&y))));
    out.push(("showcase/leaves", with_alpha(grass([60.0, 118.0, 48.0], 30), |x, y| value_noise(x, y, 16, 31) > 0.35)));
    out.push(("showcase/pine", with_alpha(grass([36.0, 82.0, 46.0], 32), |x, y| hash(x as i64, y as i64 / 2, 33) > 0.25)));
    out
}

/// Textures that get a generated `<name>_normal.png` referenced by their material.
pub const NORMAL_MAPPED: [&str; 7] =
    ["showcase/stone_bricks", "showcase/red_bricks", "showcase/cobble", "showcase/rock", "showcase/cliff", "showcase/roof_slate", "showcase/roof_red"];

/// Tangent space normal map (green up, Godot convention) from the image brightness, wrapping at the edges.
pub fn normal_map(img: &RgbaImage, strength: f32) -> RgbaImage {
    let (w, h) = img.dimensions();
    let height = |x: i64, y: i64| {
        let p = img.get_pixel(x.rem_euclid(w as i64) as u32, y.rem_euclid(h as i64) as u32);
        (p[0] as f32 * 0.3 + p[1] as f32 * 0.59 + p[2] as f32 * 0.11) / 255.0
    };
    RgbaImage::from_fn(w, h, |x, y| {
        let (x, y) = (x as i64, y as i64);
        let dx = (height(x + 1, y) - height(x - 1, y)) * strength;
        // Image rows grow downwards while the normal map's green channel points up.
        let dy = (height(x, y - 1) - height(x, y + 1)) * strength;
        let n = [-dx, -dy, 1.0];
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        let c = |v: f32| ((v / len * 0.5 + 0.5) * 255.0).round() as u8;
        image::Rgba([c(n[0]), c(n[1]), c(n[2]), 255])
    })
}

/// Godot material overrides for textures that need transparency, emission or normal maps (FuncGodot loads `<name>.tres` when present).
pub fn material_overrides() -> Vec<(&'static str, String)> {
    let material = |texture: &str, extra: &str| {
        format!(
            "[gd_resource type=\"StandardMaterial3D\" load_steps=2 format=3]\n\n[ext_resource type=\"Texture2D\" path=\"res://demo/textures/{texture}.png\" id=\"1_tex\"]\n\n[resource]\nalbedo_texture = ExtResource(\"1_tex\")\ntexture_filter = 2\n{extra}"
        )
    };
    let normal_mapped = |texture: &str| {
        format!(
            "[gd_resource type=\"StandardMaterial3D\" load_steps=3 format=3]\n\n[ext_resource type=\"Texture2D\" path=\"res://demo/textures/{texture}.png\" id=\"1_tex\"]\n[ext_resource type=\"Texture2D\" path=\"res://demo/textures/{texture}_normal.png\" id=\"2_normal\"]\n\n[resource]\nalbedo_texture = ExtResource(\"1_tex\")\ntexture_filter = 2\nnormal_enabled = true\nnormal_scale = 1.0\nnormal_texture = ExtResource(\"2_normal\")\n"
        )
    };
    let mut out: Vec<(&'static str, String)> = NORMAL_MAPPED.iter().map(|t| (*t, normal_mapped(t))).collect();
    out.extend([
        ("showcase/iron_bars", material("showcase/iron_bars", "transparency = 2\nalpha_scissor_threshold = 0.5\ncull_mode = 2\n")),
        ("showcase/leaves", material("showcase/leaves", "transparency = 2\nalpha_scissor_threshold = 0.5\ncull_mode = 2\n")),
        ("showcase/pine", material("showcase/pine", "transparency = 2\nalpha_scissor_threshold = 0.5\ncull_mode = 2\n")),
        ("showcase/glass", material("showcase/glass", "transparency = 1\nalbedo_color = Color(1, 1, 1, 0.35)\nroughness = 0.1\nmetallic_specular = 0.9\n")),
        ("showcase/water", material("showcase/water", "transparency = 1\nalbedo_color = Color(1, 1, 1, 0.82)\nroughness = 0.05\n")),
        ("showcase/lamp", material("showcase/lamp", "emission_enabled = true\nemission = Color(1, 0.9, 0.6, 1)\nemission_energy_multiplier = 3.0\n")),
        (
            "showcase/stained_glass",
            material(
                "showcase/stained_glass",
                "emission_enabled = true\nemission = Color(1, 1, 1, 1)\nemission_energy_multiplier = 0.6\nemission_texture = ExtResource(\"1_tex\")\n",
            ),
        ),
    ]);
    out
}
