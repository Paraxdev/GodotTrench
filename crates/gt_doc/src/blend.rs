//! Material blending: terrain layer weights, displacement alpha and per corner blend weights on brush and mesh faces
//! that carry a second material in their `blend_material` face property.

use std::collections::HashMap;

use gt_core::{DVec2, DVec3, NodeId};
use gt_geom::displacement;
use gt_geom::heightfield::{MAX_LAYERS, noise2};
use serde::{Deserialize, Serialize};

use crate::map::Map;

/// Face property naming the second material that vertex color alpha blends towards.
pub const BLEND_MATERIAL: &str = "blend_material";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlendMode {
    /// Pushes weight towards the chosen layer.
    #[default]
    Paint,
    /// Pushes weight back to the base layer.
    Erase,
    /// Averages weights with neighbours, softening transitions.
    Smooth,
    /// Hardens transitions towards the dominant layer.
    Sharpen,
    /// Paints the layer in noise shaped patches.
    Noise,
    /// Paints the layer only where the surface slope is within range.
    Slope,
    /// Paints the layer only within a height band.
    Height,
}

impl BlendMode {
    pub const ALL: [BlendMode; 7] =
        [BlendMode::Paint, BlendMode::Erase, BlendMode::Smooth, BlendMode::Sharpen, BlendMode::Noise, BlendMode::Slope, BlendMode::Height];

    pub fn label(&self) -> &'static str {
        match self {
            BlendMode::Paint => "paint",
            BlendMode::Erase => "erase",
            BlendMode::Smooth => "smooth",
            BlendMode::Sharpen => "sharpen",
            BlendMode::Noise => "noise",
            BlendMode::Slope => "slope",
            BlendMode::Height => "height",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Falloff {
    #[default]
    Smooth,
    Linear,
    Constant,
    /// Random speckles thinning out towards the rim.
    Spray,
}

impl Falloff {
    pub const ALL: [Falloff; 4] = [Falloff::Smooth, Falloff::Linear, Falloff::Constant, Falloff::Spray];

    pub fn label(&self) -> &'static str {
        match self {
            Falloff::Smooth => "smooth",
            Falloff::Linear => "linear",
            Falloff::Constant => "constant",
            Falloff::Spray => "spray",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BlendBrush {
    pub mode: BlendMode,
    pub falloff: Falloff,
    pub radius: f64,
    /// 0..1 per application.
    pub strength: f64,
    /// Terrain layer 0..3. Two texture targets treat 0 as the base material and anything else as the blend material.
    pub layer: usize,
    /// Slope range in degrees for `Slope`.
    pub slope: [f64; 2],
    /// World height range for `Height`.
    pub height: [f64; 2],
    /// World size of the noise patches for `Noise`.
    pub noise_scale: f64,
    /// Changes the spray and noise pattern, bump it per dab for fresh speckles.
    pub seed: u32,
}

impl Default for BlendBrush {
    fn default() -> Self {
        Self {
            mode: BlendMode::Paint,
            falloff: Falloff::Smooth,
            radius: 128.0,
            strength: 0.5,
            layer: 1,
            slope: [30.0, 90.0],
            height: [-1.0e5, 1.0e5],
            noise_scale: 256.0,
            seed: 1,
        }
    }
}

fn hash(p: DVec3, seed: u32) -> f64 {
    let k = ((p.x * 7.13).floor() as i64).wrapping_mul(73_856_093)
        ^ ((p.y * 5.71).floor() as i64).wrapping_mul(19_349_663)
        ^ ((p.z * 6.37).floor() as i64).wrapping_mul(83_492_791);
    let mut h = (k as u64 ^ seed as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 31;
    (h % 10_000) as f64 / 10_000.0
}

fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

impl BlendBrush {
    /// Brush influence at a point, 0 outside the radius.
    pub fn influence(&self, p: DVec3, center: DVec3, horizontal: bool) -> f64 {
        let d = if horizontal { DVec2::new(p.x - center.x, p.z - center.z).length() } else { (p - center).length() };
        if d >= self.radius || self.radius <= 0.0 {
            return 0.0;
        }
        let t = d / self.radius;
        let w = match self.falloff {
            Falloff::Smooth => (1.0 - t * t).powi(2),
            Falloff::Linear => 1.0 - t,
            Falloff::Constant => 1.0,
            Falloff::Spray => {
                if hash(p, self.seed) < 0.35 * (1.0 - t) {
                    1.0
                } else {
                    0.0
                }
            }
        };
        w * self.strength.clamp(0.0, 1.0)
    }

    /// How much a per point mode applies at a point, 1 for plain painting.
    fn mask(&self, p: DVec3, normal: DVec3) -> f64 {
        match self.mode {
            BlendMode::Noise => {
                let n = noise2(DVec2::new(p.x, p.z) / self.noise_scale.max(1.0), self.seed ^ 0x6e6f) * 0.5 + 0.5;
                smoothstep(0.42, 0.58, n)
            }
            BlendMode::Slope => {
                let s = normal.normalize_or(DVec3::Y).y.clamp(-1.0, 1.0).acos().to_degrees();
                smoothstep(self.slope[0] - 4.0, self.slope[0] + 4.0, s) * (1.0 - smoothstep(self.slope[1] - 4.0, self.slope[1] + 4.0, s))
            }
            BlendMode::Height => {
                let soft = ((self.height[1] - self.height[0]).abs() * 0.1).clamp(1.0, 64.0);
                smoothstep(self.height[0] - soft, self.height[0] + soft, p.y) * (1.0 - smoothstep(self.height[1] - soft, self.height[1] + soft, p.y))
            }
            _ => 1.0,
        }
    }

    fn target_layer(&self) -> usize {
        if self.mode == BlendMode::Erase { 0 } else { self.layer }
    }
}

fn normalize(w: &mut [f64]) {
    let sum: f64 = w.iter().sum();
    if sum <= 1e-9 {
        w.iter_mut().enumerate().for_each(|(i, v)| *v = if i == 0 { 1.0 } else { 0.0 });
    } else {
        w.iter_mut().for_each(|v| *v /= sum);
    }
}

/// New weights for one point. `neighbours` is the average of nearby weights, used by smoothing.
fn apply(brush: &BlendBrush, influence: f64, p: DVec3, normal: DVec3, current: &[f64], neighbours: Option<&[f64]>) -> Vec<f64> {
    let mut w = current.to_vec();
    match brush.mode {
        BlendMode::Smooth => {
            if let Some(avg) = neighbours {
                for (v, a) in w.iter_mut().zip(avg) {
                    *v += (a - *v) * influence;
                }
            }
        }
        BlendMode::Sharpen => {
            let power = 1.0 + influence * 3.0;
            w.iter_mut().for_each(|v| *v = v.max(0.0).powf(power));
            normalize(&mut w);
        }
        _ => {
            let amount = influence * brush.mask(p, normal);
            let layer = brush.target_layer().min(w.len() - 1);
            for (i, v) in w.iter_mut().enumerate() {
                let target = if i == layer { 1.0 } else { 0.0 };
                *v += (target - *v) * amount;
            }
        }
    }
    normalize(&mut w);
    w
}

/// Applies one dab to terrain layer weights. Returns true if anything changed.
pub fn blend_terrain(t: &mut gt_geom::Terrain, center: DVec3, brush: &BlendBrush) -> bool {
    let [w, d] = t.resolution;
    let n = t.heights.len();
    if !t.is_valid() {
        return false;
    }
    let local = center - t.origin;
    let r = brush.radius / t.cell_size;
    let (x0, x1) = (((local.x / t.cell_size) - r).floor().max(0.0) as u32, (((local.x / t.cell_size) + r).ceil().max(0.0) as u32).min(w - 1));
    let (z0, z1) = (((local.z / t.cell_size) - r).floor().max(0.0) as u32, (((local.z / t.cell_size) + r).ceil().max(0.0) as u32).min(d - 1));
    if x0 > x1 || z0 > z1 || local.x + brush.radius < 0.0 || local.z + brush.radius < 0.0 {
        return false;
    }
    let layers = t.layers.len().clamp(1, MAX_LAYERS);
    if t.splat.len() != n * 4 {
        t.splat = (0..n).flat_map(|_| [255u8, 0, 0, 0]).collect();
    }
    let old: Vec<[f32; 4]> = (z0..=z1).flat_map(|j| (x0..=x1).map(move |i| (i, j))).map(|(i, j)| t.weights(i, j)).collect();
    let row = (x1 - x0 + 1) as usize;
    let get = |i: u32, j: u32| old[(j - z0) as usize * row + (i - x0) as usize];
    let mut changed = false;
    for j in z0..=z1 {
        for i in x0..=x1 {
            let p = t.vertex(i, j);
            let influence = brush.influence(p, center, true);
            if influence <= 0.0 {
                continue;
            }
            let current: Vec<f64> = get(i, j).iter().take(layers).map(|v| *v as f64).collect();
            let neighbours = (brush.mode == BlendMode::Smooth).then(|| {
                let mut sum = vec![0.0; layers];
                let mut count = 0.0;
                for (di, dj) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
                    let (ni, nj) = (i as i64 + di, j as i64 + dj);
                    if ni >= x0 as i64 && nj >= z0 as i64 && ni <= x1 as i64 && nj <= z1 as i64 {
                        let nw = get(ni as u32, nj as u32);
                        for (s, v) in sum.iter_mut().zip(nw) {
                            *s += v as f64;
                        }
                        count += 1.0;
                    }
                }
                sum.iter().map(|s| s / f64::max(count, 1.0)).collect::<Vec<_>>()
            });
            let new = apply(brush, influence, p, t.normal(i, j), &current, neighbours.as_deref());
            let k = t.index(i, j) * 4;
            for l in 0..4 {
                let v = (new.get(l).copied().unwrap_or(0.0) * 255.0).round().clamp(0.0, 255.0) as u8;
                if t.splat[k + l] != v {
                    t.splat[k + l] = v;
                    changed = true;
                }
            }
        }
    }
    changed
}

/// Applies one dab to displacement alphas (0 base, 1 blend texture).
pub fn blend_displacements(map: &mut Map, faces: &[(NodeId, usize)], center: DVec3, brush: &BlendBrush) -> bool {
    let mut changed = false;
    for (id, face) in faces {
        let Some(b) = map.brush(*id) else { continue };
        let Some(grid) = displacement::grid(b, *face) else { continue };
        let n = grid.size;
        let influences: Vec<f64> = grid.positions.iter().map(|p| brush.influence(*p, center, false)).collect();
        if influences.iter().all(|w| *w <= 0.0) {
            continue;
        }
        let normal = b.faces[*face].plane.normal;
        let Some(disp) = map.brush_mut(*id).and_then(|b| b.faces[*face].data.disp.as_mut()) else { continue };
        if disp.alphas.len() != n * n {
            disp.alphas = vec![0.0; n * n];
        }
        let old = disp.alphas.clone();
        for k in 0..n * n {
            if influences[k] <= 0.0 {
                continue;
            }
            let (i, j) = (k % n, k / n);
            let a = old[k] as f64;
            let neighbours = (brush.mode == BlendMode::Smooth).then(|| {
                let mut sum = 0.0;
                let mut count = 0.0;
                for (di, dj) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
                    let (ni, nj) = (i as i64 + di, j as i64 + dj);
                    if ni >= 0 && nj >= 0 && (ni as usize) < n && (nj as usize) < n {
                        sum += old[nj as usize * n + ni as usize] as f64;
                        count += 1.0;
                    }
                }
                let avg = sum / f64::max(count, 1.0);
                vec![1.0 - avg, avg]
            });
            let new = apply(brush, influences[k], grid.positions[k], grid.normals.get(k).copied().unwrap_or(normal), &[1.0 - a, a], neighbours.as_deref());
            disp.alphas[k] = new[1] as f32;
            changed |= (new[1] - a).abs() > 1e-6;
        }
    }
    changed
}

/// Brush and mesh faces carrying a blend material among `nodes` (every node when empty).
pub fn blend_faces(map: &Map, nodes: &[NodeId]) -> Vec<(NodeId, usize)> {
    let ids: Vec<NodeId> = if nodes.is_empty() { map.nodes.keys().copied().collect() } else { nodes.to_vec() };
    let mut out = Vec::new();
    for id in ids {
        if let Some(b) = map.brush(id) {
            out.extend(b.faces.iter().enumerate().filter(|(_, f)| f.data.disp.is_none() && f.data.props.contains_key(BLEND_MATERIAL)).map(|(i, _)| (id, i)));
        } else if let Some(m) = map.mesh(id) {
            out.extend(m.faces.iter().enumerate().filter(|(_, f)| f.data.props.contains_key(BLEND_MATERIAL)).map(|(i, _)| (id, i)));
        }
    }
    out
}

/// Per corner blend weights of brush and mesh faces. Smoothing averages corners that meet at the same position.
pub fn blend_face_corners(map: &mut Map, faces: &[(NodeId, usize)], center: DVec3, brush: &BlendBrush) -> bool {
    struct Corner {
        id: NodeId,
        face: usize,
        corner: usize,
        pos: DVec3,
        normal: DVec3,
        alpha: f64,
    }
    let mut corners = Vec::new();
    for (id, face) in faces {
        let (points, normal, colors) = if let Some(b) = map.brush(*id) {
            let Some(f) = b.faces.get(*face) else { continue };
            (f.indices.iter().map(|i| b.vertices[*i as usize]).collect::<Vec<_>>(), f.plane.normal, f.data.colors.clone())
        } else if let Some(m) = map.mesh(*id) {
            let Some(f) = m.faces.get(*face) else { continue };
            (m.face_points(*face), m.face_normal(*face), f.data.colors.clone())
        } else {
            continue;
        };
        for (k, p) in points.iter().enumerate() {
            let alpha = if colors.len() == points.len() { colors[k][3] as f64 } else { 0.0 };
            corners.push(Corner { id: *id, face: *face, corner: k, pos: *p, normal, alpha });
        }
    }
    let key = |p: DVec3| ((p.x * 8.0).round() as i64, (p.y * 8.0).round() as i64, (p.z * 8.0).round() as i64);
    let mut shared: HashMap<(i64, i64, i64), (f64, f64)> = HashMap::new();
    for c in &corners {
        let e = shared.entry(key(c.pos)).or_insert((0.0, 0.0));
        e.0 += c.alpha;
        e.1 += 1.0;
    }
    let mut writes = Vec::new();
    for c in &corners {
        let influence = brush.influence(c.pos, center, false);
        if influence <= 0.0 {
            continue;
        }
        let neighbours = (brush.mode == BlendMode::Smooth).then(|| {
            // Neighbouring corners inside the brush, so a brush wide stroke evens out whole faces.
            let (mut sum, mut count) = shared.get(&key(c.pos)).copied().unwrap_or((c.alpha, 1.0));
            for o in &corners {
                if (o.pos - c.pos).length() <= brush.radius * 0.5 && key(o.pos) != key(c.pos) {
                    sum += o.alpha;
                    count += 1.0;
                }
            }
            let avg = sum / count.max(1.0);
            vec![1.0 - avg, avg]
        });
        let new = apply(brush, influence, c.pos, c.normal, &[1.0 - c.alpha, c.alpha], neighbours.as_deref());
        if (new[1] - c.alpha).abs() > 1e-6 {
            writes.push((c.id, c.face, c.corner, new[1] as f32));
        }
    }
    let changed = !writes.is_empty();
    for (id, face, corner, alpha) in writes {
        let data = if let Some(b) = map.brush_mut(id) {
            b.faces.get_mut(face).map(|f| (f.indices.len(), &mut f.data))
        } else {
            map.mesh_mut(id).and_then(|m| m.faces.get_mut(face)).map(|f| (f.indices.len(), &mut f.data))
        };
        let Some((len, data)) = data else { continue };
        if data.colors.len() != len {
            data.colors = vec![[1.0, 1.0, 1.0, 0.0]; len];
        }
        data.colors[corner][3] = alpha;
    }
    changed
}

/// Sets or clears the blend material of faces. Clearing also drops the blend weights stored in vertex alpha.
pub fn set_blend_material(map: &mut Map, faces: &[(NodeId, usize)], material: Option<&str>) -> usize {
    let mut n = 0;
    for (id, face) in faces {
        let data = if let Some(b) = map.brush_mut(*id) {
            b.faces.get_mut(*face).map(|f| &mut f.data)
        } else {
            map.mesh_mut(*id).and_then(|m| m.faces.get_mut(*face)).map(|f| &mut f.data)
        };
        let Some(data) = data else { continue };
        match material {
            Some(m) => {
                data.props.insert(BLEND_MATERIAL.into(), m.to_string());
            }
            None => {
                data.props.remove(BLEND_MATERIAL);
            }
        }
        n += 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeKind;
    use gt_core::Aabb;
    use gt_geom::{Brush, Terrain, TerrainLayer};

    fn terrain() -> Terrain {
        let mut t = Terrain::new(DVec3::ZERO, [33, 33], 16.0, "grass");
        t.layers.push(TerrainLayer { material: "rock".into(), tile: 128.0 });
        t.layers.push(TerrainLayer { material: "dirt".into(), tile: 128.0 });
        t
    }

    #[test]
    fn paint_erase_smooth_and_sharpen_on_terrain() {
        let mut t = terrain();
        let brush = BlendBrush { layer: 2, strength: 1.0, radius: 64.0, falloff: Falloff::Constant, ..Default::default() };
        assert!(blend_terrain(&mut t, DVec3::new(256.0, 0.0, 256.0), &brush));
        assert!(t.weights(16, 16)[2] > 0.99);
        assert!(t.weights(0, 0)[0] > 0.99, "outside the brush stays base");

        let smooth = BlendBrush { mode: BlendMode::Smooth, radius: 200.0, strength: 1.0, ..brush };
        blend_terrain(&mut t, DVec3::new(256.0, 0.0, 256.0), &smooth);
        let edge = t.weights(20, 16)[2];
        assert!(edge > 0.05 && edge < 0.95, "smoothing spreads the edge, got {edge}");

        let sharpen = BlendBrush { mode: BlendMode::Sharpen, ..smooth };
        let before = t.weights(20, 16);
        blend_terrain(&mut t, DVec3::new(256.0, 0.0, 256.0), &sharpen);
        let after = t.weights(20, 16);
        let dominant = if before[0] > before[2] { 0 } else { 2 };
        assert!(after[dominant] >= before[dominant]);

        let erase = BlendBrush { mode: BlendMode::Erase, radius: 400.0, ..brush };
        blend_terrain(&mut t, DVec3::new(256.0, 0.0, 256.0), &erase);
        assert!(t.weights(16, 16)[0] > 0.99);
    }

    #[test]
    fn slope_mode_only_paints_steep_ground() {
        let mut t = terrain();
        for j in 0..33 {
            for i in 17..33 {
                let k = t.index(i, j);
                t.heights[k] = (i as f32 - 16.0) * 32.0;
            }
        }
        let brush = BlendBrush {
            mode: BlendMode::Slope,
            layer: 1,
            strength: 1.0,
            radius: 10_000.0,
            falloff: Falloff::Constant,
            slope: [35.0, 90.0],
            ..Default::default()
        };
        blend_terrain(&mut t, DVec3::new(256.0, 0.0, 256.0), &brush);
        assert!(t.weights(4, 10)[1] < 0.01, "flat ground untouched");
        assert!(t.weights(25, 10)[1] > 0.9, "steep ramp painted");
    }

    #[test]
    fn face_blend_weights_and_material() {
        let mut m = Map::new();
        let l = m.default_layer();
        let id = m.insert(l, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(64.0)), "bricks").unwrap()));
        let top = m.brush(id).unwrap().faces.iter().position(|f| f.plane.normal.y > 0.5).unwrap();
        assert_eq!(set_blend_material(&mut m, &[(id, top)], Some("moss")), 1);
        let faces = blend_faces(&m, &[]);
        assert_eq!(faces, vec![(id, top)]);
        let brush = BlendBrush { strength: 1.0, radius: 20.0, falloff: Falloff::Constant, ..Default::default() };
        assert!(blend_face_corners(&mut m, &faces, DVec3::new(0.0, 64.0, 0.0), &brush));
        let colors = &m.brush(id).unwrap().faces[top].data.colors;
        assert_eq!(colors.iter().filter(|c| c[3] > 0.99).count(), 1, "only the corner under the brush");
        assert!(colors.iter().all(|c| c[0] == 1.0), "tint stays white");
    }
}
