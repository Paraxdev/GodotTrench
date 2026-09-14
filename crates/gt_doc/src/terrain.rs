//! Displacement, heightmap terrain and vertex paint editing.

use std::collections::HashMap;

use gt_core::{DVec3, NodeId, Ray};
use gt_geom::displacement::{self, Displacement};
use serde::{Deserialize, Serialize};

use crate::map::Map;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SculptMode {
    Raise,
    Lower,
    Smooth,
    /// Pulls heights towards the height under the stroke start.
    Flatten,
    Noise,
    PaintAlpha,
    EraseAlpha,
    /// Quantizes heights into steps of `terrace_step`.
    Terrace,
    /// Terrains: paints blend layer `layer`.
    PaintLayer,
    /// Terrains: cuts holes.
    Hole,
    /// Terrains: fills holes.
    Unhole,
}

impl SculptMode {
    pub const ALL: [SculptMode; 11] = [
        SculptMode::Raise,
        SculptMode::Lower,
        SculptMode::Smooth,
        SculptMode::Flatten,
        SculptMode::Noise,
        SculptMode::Terrace,
        SculptMode::PaintAlpha,
        SculptMode::EraseAlpha,
        SculptMode::PaintLayer,
        SculptMode::Hole,
        SculptMode::Unhole,
    ];

    pub fn is_paint(&self) -> bool {
        matches!(self, SculptMode::PaintAlpha | SculptMode::EraseAlpha | SculptMode::PaintLayer | SculptMode::Hole | SculptMode::Unhole)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SculptBrush {
    pub mode: SculptMode,
    pub radius: f64,
    /// Units per application for heights, 0..1 per application for alpha.
    pub strength: f64,
    /// Target height for flatten, in world units along the face normal.
    pub flatten_height: f64,
    pub terrace_step: f64,
    /// Terrain blend layer painted by `PaintLayer`.
    pub layer: u8,
}

impl Default for SculptBrush {
    fn default() -> Self {
        Self { mode: SculptMode::Raise, radius: 48.0, strength: 4.0, flatten_height: 0.0, terrace_step: 32.0, layer: 1 }
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

/// Converts quad faces into displacements. Returns how many faces were converted.
pub fn create_displacements(map: &mut Map, faces: &[(NodeId, usize)], power: u8) -> usize {
    let mut count = 0;
    for (id, face) in faces {
        let Some(brush) = map.brush_mut(*id) else { continue };
        let Some(f) = brush.faces.get_mut(*face) else { continue };
        if f.indices.len() != 4 {
            continue;
        }
        f.data.disp = Some(match &f.data.disp {
            Some(existing) if existing.power != power => existing.resample(power),
            Some(existing) => existing.clone(),
            None => Displacement::new(power),
        });
        count += 1;
    }
    count
}

pub fn remove_displacements(map: &mut Map, faces: &[(NodeId, usize)]) {
    for (id, face) in faces {
        if let Some(f) = map.brush_mut(*id).and_then(|b| b.faces.get_mut(*face)) {
            f.data.disp = None;
        }
    }
}

/// Every displacement face of the given brushes (or the whole map when `brushes` is empty).
pub fn displacement_faces(map: &Map, brushes: &[NodeId]) -> Vec<(NodeId, usize)> {
    let mut out = Vec::new();
    let ids: Vec<NodeId> = if brushes.is_empty() { map.brushes().map(|(id, _)| id).collect() } else { brushes.to_vec() };
    for id in ids {
        if let Some(b) = map.brush(id) {
            for (fi, f) in b.faces.iter().enumerate() {
                if f.data.disp.is_some() {
                    out.push((id, fi));
                }
            }
        }
    }
    out
}

/// Nearest hit of a ray on displacement surfaces: (distance, point, brush, face).
pub fn ray_cast(map: &Map, faces: &[(NodeId, usize)], ray: &Ray) -> Option<(f64, DVec3, NodeId, usize)> {
    let mut best: Option<(f64, DVec3, NodeId, usize)> = None;
    for (id, face) in faces {
        if map.is_hidden(*id) {
            continue;
        }
        let Some(brush) = map.brush(*id) else { continue };
        let Some(grid) = displacement::grid(brush, *face) else { continue };
        for (a, b, c) in displacement::triangles(grid.size) {
            if let Some(t) = ray.intersect_triangle(grid.positions[a], grid.positions[b], grid.positions[c])
                && best.is_none_or(|(bt, ..)| t < bt)
            {
                best = Some((t, ray.at(t), *id, *face));
            }
        }
    }
    best
}

fn hash_noise(p: DVec3) -> f64 {
    let v = (p.x * 12.9898 + p.y * 78.233 + p.z * 37.719).sin() * 43758.5453;
    v.fract() * 2.0 - 1.0
}

/// Applies one sculpt dab at `center`. Returns true if anything changed.
pub fn sculpt(map: &mut Map, faces: &[(NodeId, usize)], center: DVec3, brush: &SculptBrush) -> bool {
    let mut changed = false;
    for (id, face) in faces {
        let Some(b) = map.brush(*id) else { continue };
        let Some(grid) = displacement::grid(b, *face) else { continue };
        let normal = b.faces[*face].plane.normal;
        let n = grid.size;
        let weights: Vec<f64> = grid.positions.iter().map(|p| falloff((*p - center).length(), brush.radius)).collect();
        if weights.iter().all(|w| *w <= 0.0) {
            continue;
        }
        let old_heights = b.faces[*face].data.disp.as_ref().unwrap().heights.clone();
        let Some(disp) = map.brush_mut(*id).and_then(|b| b.faces[*face].data.disp.as_mut()) else { continue };
        if matches!(brush.mode, SculptMode::PaintAlpha | SculptMode::EraseAlpha) && disp.alphas.is_empty() {
            disp.alphas = vec![0.0; n * n];
        }
        for k in 0..n * n {
            let w = weights[k];
            if w <= 0.0 {
                continue;
            }
            let (i, j) = (k % n, k / n);
            match brush.mode {
                SculptMode::Raise => disp.heights[k] += (brush.strength * w) as f32,
                SculptMode::Lower => disp.heights[k] -= (brush.strength * w) as f32,
                SculptMode::Smooth => {
                    let mut sum = 0.0;
                    let mut cnt = 0.0;
                    for (di, dj) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
                        let (ni, nj) = (i as i64 + di, j as i64 + dj);
                        if ni >= 0 && nj >= 0 && (ni as usize) < n && (nj as usize) < n {
                            sum += old_heights[nj as usize * n + ni as usize] as f64;
                            cnt += 1.0;
                        }
                    }
                    let avg = sum / cnt;
                    let t = (w * brush.strength.clamp(0.0, 16.0) / 16.0).clamp(0.0, 1.0);
                    disp.heights[k] = (old_heights[k] as f64 * (1.0 - t) + avg * t) as f32;
                }
                SculptMode::Flatten => {
                    let base_height = grid.base[k].dot(normal);
                    let target = brush.flatten_height - base_height;
                    let t = (w * brush.strength.clamp(0.0, 16.0) / 16.0).clamp(0.0, 1.0);
                    disp.heights[k] = (old_heights[k] as f64 * (1.0 - t) + target * t) as f32;
                }
                SculptMode::Noise => disp.heights[k] += (hash_noise(grid.base[k]) * brush.strength * w) as f32,
                SculptMode::PaintAlpha | SculptMode::PaintLayer => disp.alphas[k] = (disp.alphas[k] as f64 + brush.strength * w).clamp(0.0, 1.0) as f32,
                SculptMode::EraseAlpha => disp.alphas[k] = (disp.alphas[k] as f64 - brush.strength * w).clamp(0.0, 1.0) as f32,
                SculptMode::Terrace => {
                    let step = brush.terrace_step.max(1e-3) as f32;
                    let h = old_heights[k];
                    let t = (w * brush.strength.clamp(0.0, 16.0) / 16.0).clamp(0.0, 1.0) as f32;
                    disp.heights[k] = h + ((h / step).round() * step - h) * t;
                }
                SculptMode::Hole | SculptMode::Unhole => {}
            }
            changed = true;
        }
    }
    changed
}

/// Editable terrains among the given nodes (every terrain in the map when `ids` is empty).
pub fn terrain_targets(map: &Map, ids: &[NodeId]) -> Vec<NodeId> {
    let candidates: Vec<NodeId> = if ids.is_empty() { map.terrains().map(|(id, _)| id).collect() } else { ids.to_vec() };
    candidates.into_iter().filter(|id| map.terrain(*id).is_some() && map.is_editable(*id)).collect()
}

/// Nearest terrain hit: (distance, point, terrain).
pub fn terrain_ray_cast(map: &Map, terrains: &[NodeId], ray: &Ray) -> Option<(f64, DVec3, NodeId)> {
    terrains.iter().filter_map(|id| map.terrain(*id).and_then(|t| t.ray_cast(ray)).map(|(d, p)| (d, p, *id))).min_by(|a, b| a.0.total_cmp(&b.0))
}

/// Applies one dab to terrains. Returns true if anything changed.
pub fn sculpt_terrains(map: &mut Map, terrains: &[NodeId], center: DVec3, brush: &SculptBrush) -> bool {
    let mut changed = false;
    for id in terrains {
        if let Some(t) = map.terrain_mut(*id) {
            changed |= sculpt_terrains_single(t, center, brush);
        }
    }
    changed
}

/// One dab on one terrain.
pub fn sculpt_terrains_single(t: &mut gt_geom::Terrain, center: DVec3, brush: &SculptBrush) -> bool {
    let r = brush.radius;
    let soft = (brush.strength.clamp(0.0, 16.0) / 16.0).clamp(0.0, 1.0);
    match brush.mode {
        SculptMode::Raise => t.raise(center, r, brush.strength),
        SculptMode::Lower => t.raise(center, r, -brush.strength),
        SculptMode::Smooth => t.smooth(center, r, soft),
        SculptMode::Flatten => t.flatten(center, r, brush.flatten_height, soft),
        SculptMode::Noise => t.add_noise(center, r, brush.strength, 7),
        SculptMode::Terrace => t.terrace(center, r, brush.terrace_step, soft),
        SculptMode::PaintLayer | SculptMode::PaintAlpha => t.paint_layer(center, r, brush.layer as usize, brush.strength.min(1.0)),
        SculptMode::EraseAlpha => t.paint_layer(center, r, 0, brush.strength.min(1.0)),
        SculptMode::Hole => t.set_holes(center, r, true),
        SculptMode::Unhole => t.set_holes(center, r, false),
    }
}

/// A boundary vertex of a displacement: (face slot, grid index, base position, position, normal).
type SeamVertex = (usize, usize, DVec3, DVec3, DVec3);

/// Makes coincident edge and corner vertices of neighbouring displacements meet at their average position.
pub fn sew(map: &mut Map, faces: &[(NodeId, usize)]) -> usize {
    let key = |p: DVec3| ((p.x * 16.0).round() as i64, (p.y * 16.0).round() as i64, (p.z * 16.0).round() as i64);
    let mut groups: HashMap<(i64, i64, i64), Vec<SeamVertex>> = HashMap::new();
    for (fi, (id, face)) in faces.iter().enumerate() {
        let Some(b) = map.brush(*id) else { continue };
        let Some(grid) = displacement::grid(b, *face) else { continue };
        let normal = b.faces[*face].plane.normal;
        let n = grid.size;
        for k in 0..n * n {
            let (i, j) = (k % n, k / n);
            if i == 0 || j == 0 || i == n - 1 || j == n - 1 {
                groups.entry(key(grid.base[k])).or_default().push((fi, k, grid.base[k], grid.positions[k], normal));
            }
        }
    }
    let mut sewn = 0;
    for entries in groups.values().filter(|e| e.len() > 1) {
        let avg = entries.iter().map(|e| e.3).sum::<DVec3>() / entries.len() as f64;
        for (fi, k, base, _, normal) in entries {
            let (id, face) = faces[*fi];
            if let Some(d) = map.brush_mut(id).and_then(|b| b.faces[face].data.disp.as_mut()) {
                d.heights[*k] = (avg - *base).dot(*normal) as f32;
            }
        }
        sewn += 1;
    }
    sewn
}

/// Vertex paint on regular faces: blends face corner colors towards `color` around `center`.
pub fn paint_vertices(map: &mut Map, brushes: &[NodeId], center: DVec3, radius: f64, color: [f32; 4], strength: f64) -> bool {
    let mut changed = false;
    for id in brushes {
        let Some(b) = map.brush(*id).cloned() else { continue };
        let Some(bm) = map.brush_mut(*id) else { continue };
        for (fi, f) in b.faces.iter().enumerate() {
            if f.data.disp.is_some() {
                continue;
            }
            for (k, vi) in f.indices.iter().enumerate() {
                let w = falloff((b.vertices[*vi as usize] - center).length(), radius) * strength;
                if w <= 0.0 {
                    continue;
                }
                let face = &mut bm.faces[fi];
                if face.data.colors.len() != face.indices.len() {
                    face.data.colors = vec![[1.0; 4]; face.indices.len()];
                }
                let c = &mut face.data.colors[k];
                for ch in 0..4 {
                    c[ch] += (color[ch] - c[ch]) * w.clamp(0.0, 1.0) as f32;
                }
                changed = true;
            }
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeKind;
    use gt_core::Aabb;
    use gt_geom::Brush;

    fn terrain_map() -> (Map, NodeId, NodeId) {
        let mut m = Map::new();
        let l = m.default_layer();
        let a = m.insert(l, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::new(0.0, -16.0, 0.0), DVec3::new(64.0, 0.0, 64.0)), "m").unwrap()));
        let b = m.insert(l, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::new(64.0, -16.0, 0.0), DVec3::new(128.0, 0.0, 64.0)), "m").unwrap()));
        let top = |m: &Map, id| m.brush(id).unwrap().faces.iter().position(|f| f.plane.normal.y > 0.5).unwrap();
        let faces = vec![(a, top(&m, a)), (b, top(&m, b))];
        assert_eq!(create_displacements(&mut m, &faces, 3), 2);
        (m, a, b)
    }

    #[test]
    fn raise_then_sew_closes_seam() {
        let (mut m, _, _) = terrain_map();
        let faces = displacement_faces(&m, &[]);
        let brush = SculptBrush { mode: SculptMode::Raise, radius: 40.0, strength: 10.0, flatten_height: 0.0, ..Default::default() };
        // Dab near the left brush only, so its edge rises while the neighbour stays flat.
        assert!(sculpt(&mut m, &faces[..1], DVec3::new(60.0, 0.0, 32.0), &brush));
        let hit = ray_cast(&m, &faces, &Ray::new(DVec3::new(62.0, 100.0, 32.0), DVec3::NEG_Y)).unwrap();
        assert!(hit.1.y > 5.0, "surface raised: {:?}", hit.1);
        assert!(sew(&mut m, &faces) > 0);
        let grids: Vec<_> = faces.iter().map(|(id, f)| displacement::grid(m.brush(*id).unwrap(), *f).unwrap()).collect();
        for (k, p) in grids[0].positions.iter().enumerate() {
            if (grids[0].base[k].x - 64.0).abs() < 1e-6 {
                let other = grids[1].base.iter().position(|q| (*q - grids[0].base[k]).length() < 1e-6).unwrap();
                assert!((grids[1].positions[other] - *p).length() < 1e-4);
            }
        }
    }

    #[test]
    fn alpha_paint_and_smooth() {
        let (mut m, a, _) = terrain_map();
        let faces = displacement_faces(&m, &[a]);
        let paint = SculptBrush { mode: SculptMode::PaintAlpha, radius: 20.0, strength: 0.5, flatten_height: 0.0, ..Default::default() };
        sculpt(&mut m, &faces, DVec3::new(32.0, 0.0, 32.0), &paint);
        let d = m.brush(a).unwrap().faces[faces[0].1].data.disp.clone().unwrap();
        assert!(d.alphas.iter().any(|v| *v > 0.4) && d.alphas.contains(&0.0));

        let raise = SculptBrush { mode: SculptMode::Raise, radius: 8.0, strength: 16.0, ..paint };
        sculpt(&mut m, &faces, DVec3::new(32.0, 0.0, 32.0), &raise);
        let peak = |m: &Map| m.brush(a).unwrap().faces[faces[0].1].data.disp.as_ref().unwrap().heights.iter().cloned().fold(0.0f32, f32::max);
        let before = peak(&m);
        let smooth = SculptBrush { mode: SculptMode::Smooth, radius: 32.0, strength: 16.0, ..paint };
        sculpt(&mut m, &faces, DVec3::new(32.0, 0.0, 32.0), &smooth);
        assert!(peak(&m) < before);
    }

    #[test]
    fn vertex_paint_blends_corners() {
        let mut m = Map::new();
        let l = m.default_layer();
        let id = m.insert(l, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(32.0)), "m").unwrap()));
        assert!(paint_vertices(&mut m, &[id], DVec3::ZERO, 8.0, [1.0, 0.0, 0.0, 1.0], 1.0));
        let b = m.brush(id).unwrap();
        let painted = b.faces.iter().filter(|f| f.data.colors.iter().any(|c| c[1] < 0.5)).count();
        assert_eq!(painted, 3, "the three faces meeting at the origin corner");
    }
}
