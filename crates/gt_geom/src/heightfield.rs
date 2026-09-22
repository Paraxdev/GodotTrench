//! Heightmap terrain for large outdoor areas. Heights live on a regular grid, split into chunks for rendering and collision.

use gt_core::{Aabb, DVec2, DVec3, Ray};
use serde::{Deserialize, Serialize};

pub const MAX_LAYERS: usize = 4;

fn default_tile() -> f64 {
    256.0
}

fn default_chunk() -> u32 {
    32
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TerrainLayer {
    pub material: String,
    /// World units covered by one texture repeat.
    #[serde(default = "default_tile")]
    pub tile: f64,
    /// Breaks up the repeat: every tile is turned and shifted by a fixed random amount and the joins are
    /// blended, so a tileable texture stops showing a grid. 0 leaves it repeating as authored, 1 is the
    /// strongest. Costs four texture reads per projection, so it is off unless asked for.
    #[serde(default)]
    pub detile: f64,
    /// How crisp the de-tiled result stays: 0 mixes the four cells evenly, which hides the joins best but
    /// softens the texture, 1 mixes only in a narrow band around the joins and keeps the detail. Ignored
    /// while `detile` is 0.
    #[serde(default = "default_sharpen")]
    pub detile_sharpen: f64,
}

fn default_sharpen() -> f64 {
    0.5
}

impl TerrainLayer {
    pub fn new(material: impl Into<String>, tile: f64) -> Self {
        Self { material: material.into(), tile, detile: 0.0, detile_sharpen: default_sharpen() }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Terrain {
    /// World position of the first vertex. Heights are added to its y.
    pub origin: DVec3,
    /// Vertex count along x and z.
    pub resolution: [u32; 2],
    pub cell_size: f64,
    #[serde(with = "b64_f32")]
    pub heights: Vec<f32>,
    pub layers: Vec<TerrainLayer>,
    /// Blend weights, four bytes per vertex. Empty means the first layer everywhere.
    #[serde(default, skip_serializing_if = "Vec::is_empty", with = "b64_u8")]
    pub splat: Vec<u8>,
    /// One byte per cell, non-zero cells are holes.
    #[serde(default, skip_serializing_if = "Vec::is_empty", with = "b64_u8")]
    pub holes: Vec<u8>,
    /// Cells per chunk side for rendering and collision.
    #[serde(default = "default_chunk")]
    pub chunk_cells: u32,
}

mod b64_f32 {
    use base64::Engine;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &[f32], s: S) -> Result<S::Ok, S::Error> {
        let bytes: Vec<u8> = v.iter().flat_map(|f| f.to_le_bytes()).collect();
        s.serialize_str(&base64::engine::general_purpose::STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<f32>, D::Error> {
        let text = String::deserialize(d)?;
        let bytes = base64::engine::general_purpose::STANDARD.decode(text).map_err(serde::de::Error::custom)?;
        Ok(bytes.as_chunks::<4>().0.iter().map(|c| f32::from_le_bytes(*c)).collect())
    }
}

mod b64_u8 {
    use base64::Engine;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &[u8], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&base64::engine::general_purpose::STANDARD.encode(v))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let text = String::deserialize(d)?;
        base64::engine::general_purpose::STANDARD.decode(text).map_err(serde::de::Error::custom)
    }
}

/// Smooth falloff, 1 at the center and 0 at the radius.
pub fn falloff(distance: f64, radius: f64) -> f64 {
    if distance >= radius || radius <= 0.0 {
        return 0.0;
    }

    let t = distance / radius;
    (1.0 - t * t).powi(2)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TerrainShape {
    Flat,
    #[default]
    Hills,
    Mountain,
    Island,
    Valley,
    Ridges,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TerrainGen {
    pub shape: TerrainShape,
    pub seed: u32,
    /// Peak height in world units.
    pub height: f64,
    /// Size of the largest features in world units.
    pub feature_size: f64,
    pub octaves: u32,
    /// Amplitude kept per octave.
    pub roughness: f64,
    pub erosion_iterations: u32,
}

impl Default for TerrainGen {
    fn default() -> Self {
        Self { shape: TerrainShape::Hills, seed: 1, height: 512.0, feature_size: 2048.0, octaves: 5, roughness: 0.5, erosion_iterations: 20 }
    }
}

/// Seeded gradient noise in roughly -1..1.
pub fn noise2(p: DVec2, seed: u32) -> f64 {
    fn hash(x: i64, y: i64, seed: u32) -> u32 {
        let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841) ^ seed.wrapping_mul(0xcb1a_b31f);
        h ^= h >> 13;
        h = h.wrapping_mul(0x5bd1_e995);
        h ^ (h >> 15)
    }

    let grad = |ix: i64, iy: i64, d: DVec2| {
        let a = hash(ix, iy, seed) as f64 / u32::MAX as f64 * std::f64::consts::TAU;
        a.cos() * d.x + a.sin() * d.y
    };
    let i = p.floor();
    let f = p - i;
    let (ix, iy) = (i.x as i64, i.y as i64);
    let fade = |t: f64| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let (u, v) = (fade(f.x), fade(f.y));
    let n00 = grad(ix, iy, f);
    let n10 = grad(ix + 1, iy, f - DVec2::new(1.0, 0.0));
    let n01 = grad(ix, iy + 1, f - DVec2::new(0.0, 1.0));
    let n11 = grad(ix + 1, iy + 1, f - DVec2::new(1.0, 1.0));
    let x0 = n00 + (n10 - n00) * u;
    let x1 = n01 + (n11 - n01) * u;
    (x0 + (x1 - x0) * v) * 1.414
}

pub fn fbm(p: DVec2, seed: u32, octaves: u32, roughness: f64) -> f64 {
    let mut sum = 0.0;
    let mut amp = 1.0;
    let mut norm = 0.0;
    let mut q = p;
    for o in 0..octaves.max(1) {
        sum += noise2(q, seed.wrapping_add(o * 7919)) * amp;
        norm += amp;
        amp *= roughness;
        q = q * 2.03 + DVec2::new(17.1, -9.7);
    }

    sum / norm
}

impl Terrain {
    pub fn check_data(&self) -> Result<(), String> {
        let [w, d] = self.resolution;
        if w < 2 || d < 2 {
            return Err(format!("terrain resolution {w} x {d} is below 2 x 2"));
        }

        let verts = w as usize * d as usize;
        if self.heights.len() != verts {
            return Err(format!("terrain of {w} x {d} vertices needs {verts} heights, has {}", self.heights.len()));
        }

        if !self.splat.is_empty() && self.splat.len() != verts * 4 {
            return Err(format!("terrain splat has {} bytes, expected {}", self.splat.len(), verts * 4));
        }

        let cells = (w as usize - 1) * (d as usize - 1);
        if !self.holes.is_empty() && self.holes.len() != cells {
            return Err(format!("terrain holes have {} cells, expected {cells}", self.holes.len()));
        }

        if self.cell_size.is_nan() || self.cell_size <= 0.0 {
            return Err(format!("terrain cell size {} must be positive", self.cell_size));
        }

        Ok(())
    }

    pub fn new(origin: DVec3, resolution: [u32; 2], cell_size: f64, material: &str) -> Self {
        let res = [resolution[0].max(2), resolution[1].max(2)];
        Self {
            origin,
            resolution: res,
            cell_size: cell_size.max(1e-3),
            heights: vec![0.0; (res[0] * res[1]) as usize],
            layers: vec![TerrainLayer::new(material, default_tile())],
            splat: Vec::new(),
            holes: Vec::new(),
            chunk_cells: default_chunk(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.resolution[0] >= 2 && self.resolution[1] >= 2 && self.heights.len() == (self.resolution[0] * self.resolution[1]) as usize
    }

    pub fn cells(&self) -> [u32; 2] {
        [self.resolution[0] - 1, self.resolution[1] - 1]
    }

    pub fn size(&self) -> DVec2 {
        DVec2::new(self.cells()[0] as f64 * self.cell_size, self.cells()[1] as f64 * self.cell_size)
    }

    pub fn index(&self, i: u32, j: u32) -> usize {
        (j * self.resolution[0] + i) as usize
    }

    pub fn height(&self, i: u32, j: u32) -> f64 {
        self.heights.get(self.index(i, j)).copied().unwrap_or(0.0) as f64
    }

    pub fn vertex(&self, i: u32, j: u32) -> DVec3 {
        self.origin + DVec3::new(i as f64 * self.cell_size, self.height(i, j), j as f64 * self.cell_size)
    }

    pub fn normal(&self, i: u32, j: u32) -> DVec3 {
        let [w, d] = self.resolution;
        let h = |x: i64, z: i64| self.height(x.clamp(0, w as i64 - 1) as u32, z.clamp(0, d as i64 - 1) as u32);
        let (x, z) = (i as i64, j as i64);
        let dx = h(x + 1, z) - h(x - 1, z);
        let dz = h(x, z + 1) - h(x, z - 1);
        DVec3::new(-dx, 2.0 * self.cell_size, -dz).normalize()
    }

    pub fn bounds(&self) -> Aabb {
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for h in &self.heights {
            lo = lo.min(*h);
            hi = hi.max(*h);
        }

        if self.heights.is_empty() {
            lo = 0.0;
            hi = 0.0;
        }

        let s = self.size();
        Aabb::new(self.origin + DVec3::new(0.0, lo as f64, 0.0), self.origin + DVec3::new(s.x, hi as f64, s.y))
    }

    pub fn is_hole(&self, ci: u32, cj: u32) -> bool {
        let k = (cj * self.cells()[0] + ci) as usize;
        self.holes.get(k).is_some_and(|h| *h != 0)
    }

    /// Normalized blend weights of a vertex.
    pub fn weights(&self, i: u32, j: u32) -> [f32; 4] {
        let k = self.index(i, j) * 4;
        if self.splat.len() < k + 4 {
            return [1.0, 0.0, 0.0, 0.0];
        }

        let w = [self.splat[k] as f32, self.splat[k + 1] as f32, self.splat[k + 2] as f32, self.splat[k + 3] as f32];
        let sum = w.iter().sum::<f32>();
        if sum <= 0.0 { [1.0, 0.0, 0.0, 0.0] } else { [w[0] / sum, w[1] / sum, w[2] / sum, w[3] / sum] }
    }

    /// The two triangles of a cell as (i, j) vertex coordinates, counter-clockwise seen from above.
    /// The diagonal alternates like displacements do, which keeps slopes symmetric.
    pub fn cell_triangles(ci: u32, cj: u32) -> [[(u32, u32); 3]; 2] {
        let p00 = (ci, cj);
        let p01 = (ci, cj + 1);
        let p11 = (ci + 1, cj + 1);
        let p10 = (ci + 1, cj);
        if (ci + cj).is_multiple_of(2) { [[p00, p01, p11], [p00, p11, p10]] } else { [[p00, p01, p10], [p01, p11, p10]] }
    }

    /// Height of the surface at a world x/z position, following the triangulation.
    pub fn height_at(&self, x: f64, z: f64) -> Option<f64> {
        let local = DVec2::new(x - self.origin.x, z - self.origin.z) / self.cell_size;
        let [cw, cd] = self.cells();
        if local.x < 0.0 || local.y < 0.0 || local.x > cw as f64 || local.y > cd as f64 {
            return None;
        }

        let ci = (local.x.floor() as u32).min(cw - 1);
        let cj = (local.y.floor() as u32).min(cd - 1);
        let ray = Ray::new(DVec3::new(x, self.origin.y + 1e7, z), DVec3::NEG_Y);
        for tri in Self::cell_triangles(ci, cj) {
            let [a, b, c] = tri.map(|(i, j)| self.vertex(i, j));
            if let Some(t) = ray.intersect_triangle(a, b, c) {
                return Some(ray.at(t).y);
            }
        }

        Some(self.vertex(ci, cj).y)
    }

    /// Nearest hit along the ray, walking the cells under it.
    pub fn ray_cast(&self, ray: &Ray) -> Option<(f64, DVec3)> {
        if !self.is_valid() {
            return None;
        }

        let bounds = self.bounds().expanded(1.0);
        let t0 = if bounds.contains_point(ray.origin) { 0.0 } else { ray.intersect_aabb(&bounds)? };
        let [cw, cd] = self.cells();
        let flat = DVec2::new(ray.dir.x, ray.dir.z);
        let test_cell = |ci: i64, cj: i64| -> Option<f64> {
            if ci < 0 || cj < 0 || ci >= cw as i64 || cj >= cd as i64 || self.is_hole(ci as u32, cj as u32) {
                return None;
            }

            let mut best: Option<f64> = None;
            for tri in Self::cell_triangles(ci as u32, cj as u32) {
                let [a, b, c] = tri.map(|(i, j)| self.vertex(i, j));
                if let Some(t) = ray.intersect_triangle(a, b, c)
                    && best.is_none_or(|bt| t < bt)
                {
                    best = Some(t);
                }
            }

            best
        };
        let start = ray.at(t0);
        let cell_of = |p: DVec3| (((p.x - self.origin.x) / self.cell_size).floor() as i64, ((p.z - self.origin.z) / self.cell_size).floor() as i64);
        if flat.length() < 1e-9 {
            let (ci, cj) = cell_of(start);
            return test_cell(ci, cj).map(|t| (t, ray.at(t)));
        }

        // 2D DDA over the grid.
        let (mut ci, mut cj) = cell_of(start);
        let step_x: i64 = if flat.x > 0.0 { 1 } else { -1 };
        let step_z: i64 = if flat.y > 0.0 { 1 } else { -1 };
        let local = DVec2::new(start.x - self.origin.x, start.z - self.origin.z) / self.cell_size;
        let next_boundary = |pos: f64, cell: i64, step: i64| if step > 0 { (cell + 1) as f64 - pos } else { pos - cell as f64 };
        let inv = DVec2::new(
            if flat.x.abs() > 1e-12 { (self.cell_size / flat.x).abs() } else { f64::MAX },
            if flat.y.abs() > 1e-12 { (self.cell_size / flat.y).abs() } else { f64::MAX },
        );
        let mut t_max_x = t0 + next_boundary(local.x, ci, step_x) * inv.x;
        let mut t_max_z = t0 + next_boundary(local.y, cj, step_z) * inv.y;
        let max_steps = (cw + cd) as usize * 2 + 4;
        for _ in 0..max_steps {
            // Neighbouring cells are tested too, a triangle edge on the boundary would otherwise slip through.
            let mut best: Option<f64> = None;
            for (dx, dz) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                if let Some(t) = test_cell(ci + dx, cj + dz)
                    && best.is_none_or(|b| t < b)
                {
                    best = Some(t);
                }
            }

            if let Some(t) = best {
                return Some((t, ray.at(t)));
            }

            if t_max_x < t_max_z {
                ci += step_x;
                t_max_x += inv.x;
            } else {
                cj += step_z;
                t_max_z += inv.y;
            }

            let outside_x = (step_x > 0 && ci > cw as i64) || (step_x < 0 && ci < -1);
            let outside_z = (step_z > 0 && cj > cd as i64) || (step_z < 0 && cj < -1);
            if outside_x || outside_z {
                break;
            }
        }

        None
    }

    /// Vertex coordinate range within `radius` of a world point.
    fn affected(&self, center: DVec3, radius: f64) -> Option<(u32, u32, u32, u32)> {
        let [w, d] = self.resolution;
        let lo_x = ((center.x - radius - self.origin.x) / self.cell_size).floor().max(0.0) as i64;
        let hi_x = ((center.x + radius - self.origin.x) / self.cell_size).ceil() as i64;
        let lo_z = ((center.z - radius - self.origin.z) / self.cell_size).floor().max(0.0) as i64;
        let hi_z = ((center.z + radius - self.origin.z) / self.cell_size).ceil() as i64;
        if hi_x < 0 || hi_z < 0 || lo_x >= w as i64 || lo_z >= d as i64 {
            return None;
        }

        Some((lo_x as u32, (hi_x as u32).min(w - 1), lo_z as u32, (hi_z as u32).min(d - 1)))
    }

    fn weight_at(&self, i: u32, j: u32, center: DVec3, radius: f64) -> f64 {
        let p = self.origin + DVec3::new(i as f64 * self.cell_size, 0.0, j as f64 * self.cell_size);
        falloff(DVec2::new(p.x - center.x, p.z - center.z).length(), radius)
    }

    /// Adds `amount * falloff` to the heights. Returns true if anything changed.
    pub fn raise(&mut self, center: DVec3, radius: f64, amount: f64) -> bool {
        let Some((x0, x1, z0, z1)) = self.affected(center, radius) else { return false };
        let mut changed = false;
        for j in z0..=z1 {
            for i in x0..=x1 {
                let w = self.weight_at(i, j, center, radius);
                if w > 0.0 {
                    let k = self.index(i, j);
                    self.heights[k] += (amount * w) as f32;
                    changed = true;
                }
            }
        }

        changed
    }

    /// Relaxes heights towards their neighbour average by `t` (0..1).
    pub fn smooth(&mut self, center: DVec3, radius: f64, t: f64) -> bool {
        let Some((x0, x1, z0, z1)) = self.affected(center, radius) else { return false };
        let old = self.heights.clone();
        let [w, d] = self.resolution;
        let mut changed = false;
        for j in z0..=z1 {
            for i in x0..=x1 {
                let wgt = self.weight_at(i, j, center, radius) * t.clamp(0.0, 1.0);
                if wgt <= 0.0 {
                    continue;
                }

                let mut sum = 0.0;
                let mut n = 0.0;
                for (dx, dz) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1), (-1, -1), (1, 1), (-1, 1), (1, -1)] {
                    let (x, z) = (i as i64 + dx, j as i64 + dz);
                    if x >= 0 && z >= 0 && x < w as i64 && z < d as i64 {
                        sum += old[(z as u32 * w + x as u32) as usize] as f64;
                        n += 1.0;
                    }
                }

                let k = self.index(i, j);
                let avg = sum / n;
                self.heights[k] = (old[k] as f64 * (1.0 - wgt) + avg * wgt) as f32;
                changed = true;
            }
        }

        changed
    }

    /// Pulls heights towards `target` (world y) by `t` (0..1).
    pub fn flatten(&mut self, center: DVec3, radius: f64, target: f64, t: f64) -> bool {
        let Some((x0, x1, z0, z1)) = self.affected(center, radius) else { return false };
        let local_target = (target - self.origin.y) as f32;
        let mut changed = false;
        for j in z0..=z1 {
            for i in x0..=x1 {
                let w = (self.weight_at(i, j, center, radius) * t.clamp(0.0, 1.0)) as f32;
                if w > 0.0 {
                    let k = self.index(i, j);
                    self.heights[k] += (local_target - self.heights[k]) * w;
                    changed = true;
                }
            }
        }

        changed
    }

    pub fn add_noise(&mut self, center: DVec3, radius: f64, amount: f64, seed: u32) -> bool {
        let Some((x0, x1, z0, z1)) = self.affected(center, radius) else { return false };
        let mut changed = false;
        for j in z0..=z1 {
            for i in x0..=x1 {
                let w = self.weight_at(i, j, center, radius);
                if w > 0.0 {
                    let p = DVec2::new(i as f64, j as f64) * 0.37;
                    let k = self.index(i, j);
                    self.heights[k] += (noise2(p, seed) * amount * w) as f32;
                    changed = true;
                }
            }
        }

        changed
    }

    /// Quantizes heights into steps of `step` units, blended by `t`.
    pub fn terrace(&mut self, center: DVec3, radius: f64, step: f64, t: f64) -> bool {
        let Some((x0, x1, z0, z1)) = self.affected(center, radius) else { return false };
        let step = step.max(1e-3);
        let mut changed = false;
        for j in z0..=z1 {
            for i in x0..=x1 {
                let w = self.weight_at(i, j, center, radius) * t.clamp(0.0, 1.0);
                if w > 0.0 {
                    let k = self.index(i, j);
                    let h = self.heights[k] as f64;
                    let q = (h / step).round() * step;
                    self.heights[k] = (h + (q - h) * w) as f32;
                    changed = true;
                }
            }
        }

        changed
    }

    /// Sets heights inside an XZ rectangle to world height `height`, easing back to the old ground over `margin` units.
    /// Building pads, courtyards and plateaus. Returns true if anything changed.
    pub fn flatten_rect(&mut self, min: DVec2, max: DVec2, height: f64, margin: f64) -> bool {
        let (lo, hi) = (min.min(max), min.max(max));
        let center = DVec3::new((lo.x + hi.x) * 0.5, 0.0, (lo.y + hi.y) * 0.5);
        let reach = ((hi - lo) * 0.5).length() + margin;
        let Some((x0, x1, z0, z1)) = self.affected(center, reach) else { return false };
        let target = (height - self.origin.y) as f32;
        let mut changed = false;
        for j in z0..=z1 {
            for i in x0..=x1 {
                let p = self.vertex(i, j);
                let dx = (lo.x - p.x).max(p.x - hi.x).max(0.0);
                let dz = (lo.y - p.z).max(p.z - hi.y).max(0.0);
                let d = (dx * dx + dz * dz).sqrt();
                let w = if d <= 0.0 {
                    1.0
                } else if margin > 0.0 && d < margin {
                    let t = 1.0 - d / margin;
                    t * t * (3.0 - 2.0 * t)
                } else {
                    0.0
                } as f32;
                if w > 0.0 {
                    let k = self.index(i, j);
                    self.heights[k] += (target - self.heights[k]) * w;
                    changed = true;
                }
            }
        }

        changed
    }

    /// Carves or fills a straight ramp from `a` to `b` (world positions), `width` wide with soft `margin` sides.
    /// Roads, paths and slopes up to a door. Returns true if anything changed.
    pub fn ramp(&mut self, a: DVec3, b: DVec3, width: f64, margin: f64) -> bool {
        let seg = DVec2::new(b.x - a.x, b.z - a.z);
        let len2 = seg.length_squared().max(1e-9);
        let center = (a + b) * 0.5;
        let reach = seg.length() * 0.5 + width * 0.5 + margin;
        let Some((x0, x1, z0, z1)) = self.affected(center, reach) else { return false };
        let mut changed = false;
        for j in z0..=z1 {
            for i in x0..=x1 {
                let p = self.vertex(i, j);
                let rel = DVec2::new(p.x - a.x, p.z - a.z);
                let t = (rel.dot(seg) / len2).clamp(0.0, 1.0);
                let d = (rel - seg * t).length() - width * 0.5;
                let w = if d <= 0.0 {
                    1.0
                } else if margin > 0.0 && d < margin {
                    let s = 1.0 - d / margin;
                    s * s * (3.0 - 2.0 * s)
                } else {
                    0.0
                };
                if w > 0.0 {
                    let target = (a.y + (b.y - a.y) * t - self.origin.y) as f32;
                    let k = self.index(i, j);
                    self.heights[k] += (target - self.heights[k]) * w as f32;
                    changed = true;
                }
            }
        }

        changed
    }

    /// Adds weight to a blend layer (0..3) by `amount` (0..1 per application).
    pub fn paint_layer(&mut self, center: DVec3, radius: f64, layer: usize, amount: f64) -> bool {
        let layer = layer.min(MAX_LAYERS - 1);
        let Some((x0, x1, z0, z1)) = self.affected(center, radius) else { return false };
        let n = self.heights.len();
        if self.splat.len() != n * 4 {
            self.splat = (0..n).flat_map(|_| [255u8, 0, 0, 0]).collect();
        }

        let mut changed = false;
        for j in z0..=z1 {
            for i in x0..=x1 {
                let w = self.weight_at(i, j, center, radius) * amount.clamp(0.0, 1.0);
                if w <= 0.0 {
                    continue;
                }

                let k = self.index(i, j) * 4;
                let mut weights = self.weights(i, j);
                for (l, wl) in weights.iter_mut().enumerate() {
                    let target = if l == layer { 1.0 } else { 0.0 };
                    *wl += (target - *wl) * w as f32;
                }

                for (dst, wl) in self.splat[k..k + 4].iter_mut().zip(weights) {
                    *dst = (wl * 255.0).round().clamp(0.0, 255.0) as u8;
                }

                changed = true;
            }
        }

        changed
    }

    /// Cuts or restores holes in the cells whose center lies within `radius`.
    pub fn set_holes(&mut self, center: DVec3, radius: f64, hole: bool) -> bool {
        let [cw, cd] = self.cells();
        if self.holes.len() != (cw * cd) as usize {
            self.holes = vec![0; (cw * cd) as usize];
        }

        let mut changed = false;
        for cj in 0..cd {
            for ci in 0..cw {
                let c = self.origin + DVec3::new((ci as f64 + 0.5) * self.cell_size, 0.0, (cj as f64 + 0.5) * self.cell_size);
                if DVec2::new(c.x - center.x, c.z - center.z).length() <= radius {
                    let k = (cj * cw + ci) as usize;
                    let v = hole as u8;
                    if self.holes[k] != v {
                        self.holes[k] = v;
                        changed = true;
                    }
                }
            }
        }

        if self.holes.iter().all(|h| *h == 0) {
            self.holes.clear();
        }

        changed
    }

    /// Replaces the heights with procedural terrain.
    pub fn generate(&mut self, params: &TerrainGen) {
        let [w, d] = self.resolution;
        let size = self.size();
        let center = size * 0.5;
        let radius = size.min_element() * 0.5;
        let freq = 1.0 / params.feature_size.max(1.0);
        for j in 0..d {
            for i in 0..w {
                let p = DVec2::new(i as f64 * self.cell_size, j as f64 * self.cell_size);
                let q = p * freq;
                let base = fbm(q, params.seed, params.octaves, params.roughness);
                let r = ((p - center).length() / radius).min(1.5);
                let h = match params.shape {
                    TerrainShape::Flat => 0.0,
                    TerrainShape::Hills => (base * 0.5 + 0.5) * params.height,
                    TerrainShape::Mountain => {
                        let ridge = 1.0 - fbm(q * 1.7, params.seed ^ 0x5eed, params.octaves, params.roughness).abs();
                        let dome = (1.0 - r).max(0.0).powf(1.6);
                        (dome * (0.65 + 0.35 * ridge) + base * 0.08 * dome.sqrt()) * params.height
                    }
                    TerrainShape::Island => {
                        let dome = (1.0 - r * r).max(-0.3);
                        (dome * 0.7 + base * 0.3) * params.height
                    }
                    TerrainShape::Valley => {
                        let across = ((p.x - center.x) / radius).abs().min(1.0);
                        (across.powf(1.5) * 0.8 + base * 0.2) * params.height
                    }
                    TerrainShape::Ridges => (1.0 - fbm(q, params.seed, params.octaves, params.roughness).abs()).powi(2) * params.height,
                };
                let k = self.index(i, j);
                self.heights[k] = h as f32;
            }
        }

        self.erode(params.erosion_iterations, self.cell_size * 0.9);
    }

    /// Thermal erosion: material slides down slopes steeper than `talus` height per cell.
    pub fn erode(&mut self, iterations: u32, talus: f64) {
        let [w, d] = self.resolution;
        for _ in 0..iterations {
            let old = self.heights.clone();
            for j in 1..d - 1 {
                for i in 1..w - 1 {
                    let k = self.index(i, j);
                    let h = old[k] as f64;
                    let mut lowest = (0usize, 0.0f64);
                    for (dx, dz) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
                        let nk = ((j as i64 + dz) as u32 * w + (i as i64 + dx) as u32) as usize;
                        let diff = h - old[nk] as f64;
                        if diff > lowest.1 {
                            lowest = (nk, diff);
                        }
                    }

                    if lowest.1 > talus {
                        let moved = ((lowest.1 - talus) * 0.25) as f32;
                        self.heights[k] -= moved;
                        self.heights[lowest.0] += moved;
                    }
                }
            }
        }
    }

    /// Paints layers from the shape of the terrain: `rock` on steep slopes, `top` above `top_height`, `low` below `low_height`.
    pub fn auto_paint(&mut self, rock_slope: f64, top_height: f64, low_height: f64) {
        let n = self.heights.len();
        self.splat = vec![0; n * 4];
        let [w, d] = self.resolution;
        for j in 0..d {
            for i in 0..w {
                let normal = self.normal(i, j);
                let h = self.height(i, j);
                let slope = 1.0 - normal.y;
                let mut weights = [1.0f64, 0.0, 0.0, 0.0];
                let rock = ((slope - rock_slope) / 0.08).clamp(0.0, 1.0);
                let top = ((h - top_height) / (self.cell_size * 2.0)).clamp(0.0, 1.0) * (1.0 - rock * 0.7);
                let low = ((low_height - h) / (self.cell_size * 1.5)).clamp(0.0, 1.0) * (1.0 - rock);
                weights[1] = rock;
                weights[2] = top;
                weights[3] = low;
                weights[0] = (1.0 - rock - top - low).max(0.0);
                let sum: f64 = weights.iter().sum::<f64>().max(1e-6);
                let k = self.index(i, j) * 4;
                for (dst, wl) in self.splat[k..k + 4].iter_mut().zip(weights) {
                    *dst = (wl / sum * 255.0).round() as u8;
                }
            }
        }
    }

    /// Resamples a grayscale image (values 0..1, row major, `size` pixels) into heights scaled by `height`.
    pub fn import_heightmap(&mut self, values: &[f32], size: [u32; 2], height: f64) {
        let [iw, ih] = size;
        if iw < 2 || ih < 2 || values.len() < (iw * ih) as usize {
            return;
        }

        let [w, d] = self.resolution;
        for j in 0..d {
            for i in 0..w {
                let u = i as f64 / (w - 1) as f64 * (iw - 1) as f64;
                let v = j as f64 / (d - 1) as f64 * (ih - 1) as f64;
                let (x0, y0) = (u.floor() as u32, v.floor() as u32);
                let (x1, y1) = ((x0 + 1).min(iw - 1), (y0 + 1).min(ih - 1));
                let (fx, fy) = (u - x0 as f64, v - y0 as f64);
                let s = |x: u32, y: u32| values[(y * iw + x) as usize] as f64;
                let top = s(x0, y0) + (s(x1, y0) - s(x0, y0)) * fx;
                let bottom = s(x0, y1) + (s(x1, y1) - s(x0, y1)) * fx;
                let k = self.index(i, j);
                self.heights[k] = ((top + (bottom - top) * fy) * height) as f32;
            }
        }
    }

    /// Chunks as (first cell i, first cell j, cells i, cells j).
    pub fn chunks(&self) -> Vec<(u32, u32, u32, u32)> {
        let [cw, cd] = self.cells();
        let size = self.chunk_cells.max(1);
        let mut out = Vec::new();
        let mut cj = 0;
        while cj < cd {
            let mut ci = 0;
            while ci < cw {
                out.push((ci, cj, size.min(cw - ci), size.min(cd - cj)));
                ci += size;
            }

            cj += size;
        }

        out
    }

    pub fn translated(&self, offset: DVec3) -> Terrain {
        let mut t = self.clone();
        t.origin = gt_core::snap_vec(t.origin + offset);
        t
    }

    /// Terrains stay axis aligned: translation and scale are applied, rotation is ignored.
    pub fn transformed(&self, m: &gt_core::DMat4) -> Terrain {
        let (scale, _, _) = m.to_scale_rotation_translation();
        let mut t = self.clone();
        let size = self.size();
        let center = self.origin + DVec3::new(size.x * 0.5, 0.0, size.y * 0.5);
        let new_center = m.transform_point3(center);
        let horizontal = ((scale.x.abs() + scale.z.abs()) * 0.5).max(1e-3);
        t.cell_size = self.cell_size * horizontal;
        if (scale.y - 1.0).abs() > 1e-9 {
            for h in &mut t.heights {
                *h *= scale.y.abs() as f32;
            }
        }

        let new_size = t.size();
        t.origin = gt_core::snap_vec(new_center - DVec3::new(new_size.x * 0.5, 0.0, new_size.y * 0.5));
        t
    }

    /// Resizes the grid, resampling heights, weights and holes. Cells stay square, so `resolution` bounds the vertex
    /// count per side: the axis that needs the larger cells sets the cell size and the other axis gets as many
    /// vertices as keep its extent (to the nearest cell).
    pub fn resample(&self, resolution: [u32; 2]) -> Terrain {
        let size = self.size();
        let cell = (size.x / (resolution[0].max(2) - 1) as f64).max(size.y / (resolution[1].max(2) - 1) as f64).max(1e-3);
        let res = [((size.x / cell).round() as u32 + 1).max(2), ((size.y / cell).round() as u32 + 1).max(2)];
        let mut out = Terrain::new(self.origin, res, cell, "");
        out.layers = self.layers.clone();
        out.chunk_cells = self.chunk_cells;
        let values: Vec<f32> = self.heights.clone();
        out.import_heightmap(&values, self.resolution, 1.0);
        if !self.splat.is_empty() {
            out.splat = vec![0; out.heights.len() * 4];
            for j in 0..res[1] {
                for i in 0..res[0] {
                    let si = ((i as f64 / (res[0] - 1) as f64) * (self.resolution[0] - 1) as f64).round() as u32;
                    let sj = ((j as f64 / (res[1] - 1) as f64) * (self.resolution[1] - 1) as f64).round() as u32;
                    let (dst, src) = (out.index(i, j) * 4, self.index(si, sj) * 4);
                    out.splat[dst..dst + 4].copy_from_slice(&self.splat[src..src + 4]);
                }
            }
        }

        if self.holes.iter().any(|h| *h != 0) {
            let ([cw, cd], [ow, od]) = (out.cells(), self.cells());
            out.holes = vec![0; (cw * cd) as usize];
            for cj in 0..cd {
                for ci in 0..cw {
                    let si = (((ci as f64 + 0.5) / cw as f64 * ow as f64) as u32).min(ow - 1);
                    let sj = (((cj as f64 + 0.5) / cd as f64 * od as f64) as u32).min(od - 1);
                    if self.is_hole(si, sj) {
                        out.holes[(cj * cw + ci) as usize] = 1;
                    }
                }
            }
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat() -> Terrain {
        Terrain::new(DVec3::new(-320.0, 0.0, -320.0), [33, 33], 20.0, "grass")
    }

    #[test]
    fn flatten_rect_and_ramp() {
        let mut t = flat();
        t.raise(DVec3::ZERO, 400.0, 100.0);
        assert!(t.flatten_rect(DVec2::new(-60.0, -60.0), DVec2::new(60.0, 60.0), 40.0, 80.0));
        assert!((t.height_at(0.0, 0.0).unwrap() - 40.0).abs() < 1e-3);
        assert!((t.height_at(59.0, -59.0).unwrap() - 40.0).abs() < 1e-3, "pad corner is flat");
        let outside = t.height_at(-300.0, -300.0).unwrap();
        assert!(outside.abs() < 1e-3, "far ground untouched, got {outside}");

        let mut r = flat();
        assert!(r.ramp(DVec3::new(-200.0, 0.0, 0.0), DVec3::new(200.0, 80.0, 0.0), 40.0, 20.0));
        let mid = r.height_at(0.0, 0.0).unwrap();
        assert!((mid - 40.0).abs() < 1.0, "ramp midpoint halfway up, got {mid}");
        assert!(r.height_at(0.0, 200.0).unwrap().abs() < 1e-3, "beside the ramp stays flat");
    }

    #[test]
    fn raise_and_ray_cast() {
        let mut t = flat();
        assert!(t.raise(DVec3::ZERO, 100.0, 64.0));
        let hit = t.ray_cast(&Ray::new(DVec3::new(0.0, 500.0, 0.0), DVec3::NEG_Y)).unwrap();
        assert!((hit.1.y - 64.0).abs() < 1e-3, "{:?}", hit);
        let slanted = t.ray_cast(&Ray::new(DVec3::new(-300.0, 300.0, -300.0), DVec3::new(1.0, -1.0, 1.0))).unwrap();
        assert!((slanted.1.y - t.height_at(slanted.1.x, slanted.1.z).unwrap()).abs() < 1e-3);
        assert!(t.ray_cast(&Ray::new(DVec3::new(0.0, 500.0, 0.0), DVec3::Y)).is_none());
    }

    #[test]
    fn smooth_flatten_terrace() {
        let mut t = flat();
        t.raise(DVec3::ZERO, 60.0, 50.0);
        let peak = |t: &Terrain| t.heights.iter().cloned().fold(0.0f32, f32::max);
        let before = peak(&t);
        t.smooth(DVec3::ZERO, 100.0, 1.0);
        assert!(peak(&t) < before);
        t.flatten(DVec3::ZERO, 400.0, 10.0, 1.0);
        let center = t.height(16, 16);
        assert!((center - 10.0).abs() < 1e-3, "center {center}");
        t.terrace(DVec3::ZERO, 1000.0, 16.0, 1.0);
    }

    #[test]
    fn paint_layers_and_holes() {
        let mut t = flat();
        t.layers.push(TerrainLayer::new("rock", 128.0));
        assert!(t.paint_layer(DVec3::ZERO, 50.0, 1, 1.0));
        let w = t.weights(16, 16);
        assert!(w[1] > 0.99, "{w:?}");
        assert_eq!(t.weights(0, 0), [1.0, 0.0, 0.0, 0.0]);
        assert!(t.set_holes(DVec3::ZERO, 15.0, true));
        assert!(t.ray_cast(&Ray::new(DVec3::new(5.0, 100.0, 5.0), DVec3::NEG_Y)).is_none());
        t.set_holes(DVec3::ZERO, 15.0, false);
        assert!(t.holes.is_empty());
    }

    #[test]
    fn generate_mountain_and_serde() {
        let mut t = flat();
        t.generate(&TerrainGen { shape: TerrainShape::Mountain, height: 400.0, ..Default::default() });
        let b = t.bounds();
        assert!(b.max.y > 150.0 && b.min.y > -50.0, "{b:?}");
        t.auto_paint(0.3, 300.0, 20.0);
        let json = serde_json::to_string(&t).unwrap();
        let back: Terrain = serde_json::from_str(&json).unwrap();
        assert_eq!(t, back);
        assert_eq!(t.chunks().len(), 1);
        t.chunk_cells = 16;
        assert_eq!(t.chunks().len(), 4);
    }

    #[test]
    fn transform_keeps_center() {
        let t = flat();
        let m = gt_core::DMat4::from_scale(DVec3::new(2.0, 3.0, 2.0));
        let moved = t.transformed(&m);
        assert!((moved.size() - t.size() * 2.0).length() < 1e-9);
        let c = |t: &Terrain| t.origin + DVec3::new(t.size().x * 0.5, 0.0, t.size().y * 0.5);
        assert!((c(&moved) - c(&t)).length() < 1e-9);
        let r = t.resample([17, 17]);
        assert!((r.size() - t.size()).length() < 1e-9);
    }

    #[test]
    fn resample_keeps_extents_and_holes() {
        let mut t = Terrain::new(DVec3::ZERO, [33, 17], 20.0, "grass");
        assert!(t.set_holes(DVec3::new(10.0, 0.0, 10.0), 5.0, true));
        let r = t.resample([65, 65]);
        assert_eq!(r.resolution, [65, 33]);
        assert!((r.size() - t.size()).length() < 1e-9, "{:?} vs {:?}", r.size(), t.size());
        assert_eq!(r.holes.len(), (64 * 32) as usize);
        assert!(r.is_hole(0, 0) && r.is_hole(1, 1) && !r.is_hole(2, 2));
        assert!(r.check_data().is_ok());
    }
}
