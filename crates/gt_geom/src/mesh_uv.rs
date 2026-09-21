//! Explicit UV layouts for mesh faces: box, cylinder, sphere and view projections, unfolding and normalizing.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use gt_core::{DVec2, DVec3};
use serde::{Deserialize, Serialize};

use crate::mesh::{Mesh, edge_key};
use crate::uv::{FaceUv, paraxial_axes};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum UvProjection {
    /// Each face projected along its dominant axis.
    Box,
    /// Around an axis through the selection center, u follows the circumference.
    Cylinder {
        axis: DVec3,
    },
    Sphere,
    /// Straight along a view: u along `right`, v along `-up`.
    View {
        right: DVec3,
        up: DVec3,
    },
    /// Faces laid out flat edge by edge from the first face, so strips and grids stay continuous.
    Unfold,
}

impl UvProjection {
    pub fn label(&self) -> &'static str {
        match self {
            UvProjection::Box => "Box",
            UvProjection::Cylinder { .. } => "Cylinder",
            UvProjection::Sphere => "Sphere",
            UvProjection::View { .. } => "View",
            UvProjection::Unfold => "Unfold",
        }
    }
}

fn to_f32(uv: DVec2) -> [f32; 2] {
    [uv.x as f32, uv.y as f32]
}

/// Keeps angles of one face within half a turn of its first corner, so faces crossing the seam do not smear.
fn unwrap_angles(values: &mut [f64], period: f64) {
    if let Some(&first) = values.first() {
        for v in values.iter_mut() {
            while *v - first > period * 0.5 {
                *v -= period;
            }
            while first - *v > period * 0.5 {
                *v += period;
            }
        }
    }
}

impl Mesh {
    /// Writes explicit UVs for `faces` (all faces when empty). `repeat` is the world size of one texture repeat.
    pub fn project_uvs(&mut self, faces: &[usize], projection: UvProjection, repeat: DVec2) {
        let faces: Vec<usize> =
            if faces.is_empty() { (0..self.faces.len()).collect() } else { faces.iter().copied().filter(|f| *f < self.faces.len()).collect() };
        if faces.is_empty() {
            return;
        }
        let repeat = repeat.max(DVec2::splat(1e-6));
        let points: Vec<DVec3> = faces.iter().flat_map(|f| self.face_points(*f)).collect();
        let center = points.iter().copied().sum::<DVec3>() / points.len() as f64;
        match projection {
            UvProjection::Box => {
                for &fi in &faces {
                    let (u, v) = paraxial_axes(self.face_normal(fi));
                    let uvs = self.face_points(fi).iter().map(|p| to_f32(DVec2::new(p.dot(u), p.dot(v)) / repeat)).collect();
                    self.faces[fi].uvs = uvs;
                }
            }
            UvProjection::View { right, up } => {
                let (u, v) = (right.normalize_or(DVec3::X), -up.normalize_or(DVec3::Y));
                for &fi in &faces {
                    let uvs = self.face_points(fi).iter().map(|p| to_f32(DVec2::new(p.dot(u), p.dot(v)) / repeat)).collect();
                    self.faces[fi].uvs = uvs;
                }
            }
            UvProjection::Cylinder { axis } => {
                let axis = axis.normalize_or(DVec3::Y);
                let basis = gt_core::Plane::from_point_normal(center, axis).basis();
                let radius = points.iter().map(|p| (*p - center - axis * (*p - center).dot(axis)).length()).sum::<f64>() / points.len() as f64;
                let circumference = std::f64::consts::TAU * radius.max(1e-6);
                for &fi in &faces {
                    let pts = self.face_points(fi);
                    let mut angles: Vec<f64> = pts.iter().map(|p| (*p - center).dot(basis.1).atan2((*p - center).dot(basis.0))).collect();
                    unwrap_angles(&mut angles, std::f64::consts::TAU);
                    let uvs = pts
                        .iter()
                        .zip(&angles)
                        .map(|(p, a)| to_f32(DVec2::new(-a / std::f64::consts::TAU * circumference, -(*p - center).dot(axis)) / repeat))
                        .collect();
                    self.faces[fi].uvs = uvs;
                }
            }
            UvProjection::Sphere => {
                let radius = points.iter().map(|p| (*p - center).length()).sum::<f64>() / points.len() as f64;
                for &fi in &faces {
                    let pts = self.face_points(fi);
                    let dirs: Vec<DVec3> = pts.iter().map(|p| (*p - center).normalize_or(DVec3::Y)).collect();
                    let mut lon: Vec<f64> = dirs.iter().map(|d| d.x.atan2(d.z)).collect();
                    unwrap_angles(&mut lon, std::f64::consts::TAU);
                    let uvs = dirs.iter().zip(&lon).map(|(d, l)| to_f32(DVec2::new(l * radius, -d.y.clamp(-1.0, 1.0).asin() * radius) / repeat)).collect();
                    self.faces[fi].uvs = uvs;
                }
            }
            UvProjection::Unfold => self.unfold_uvs(&faces, repeat),
        }
    }

    fn unfold_uvs(&mut self, faces: &[usize], repeat: DVec2) {
        let selected: BTreeSet<usize> = faces.iter().copied().collect();
        let edge_faces = self.edge_faces();
        let mut done: BTreeMap<usize, Vec<DVec2>> = BTreeMap::new();
        for &seed in faces {
            if done.contains_key(&seed) {
                continue;
            }
            // Each island starts from a face aligned projection, in world units.
            let n = self.face_normal(seed);
            let base = FaceUv::face_aligned(n, DVec2::ONE);
            let flip = if n.cross(base.u_axis).dot(base.v_axis) < 0.0 { -1.0 } else { 1.0 };
            let seed_uvs: Vec<DVec2> = self.face_points(seed).iter().map(|p| DVec2::new(p.dot(base.u_axis), p.dot(base.v_axis))).collect();
            done.insert(seed, seed_uvs);
            let mut queue = VecDeque::from([seed]);
            while let Some(fi) = queue.pop_front() {
                let idx = self.faces[fi].indices.clone();
                for k in 0..idx.len() {
                    let (a, b) = (idx[k], idx[(k + 1) % idx.len()]);
                    let (uva, uvb) = (done[&fi][k], done[&fi][(k + 1) % idx.len()]);
                    for &other in edge_faces.get(&edge_key(a, b)).map(|v| v.as_slice()).unwrap_or(&[]) {
                        if other == fi || !selected.contains(&other) || done.contains_key(&other) {
                            continue;
                        }
                        let (pa, pb) = (self.vertices[a as usize], self.vertices[b as usize]);
                        let edge = pb - pa;
                        let euv = uvb - uva;
                        if edge.length() < 1e-9 || euv.length() < 1e-12 {
                            continue;
                        }
                        let on = self.face_normal(other);
                        let x = edge / edge.length();
                        let y = on.cross(x) * flip;
                        let scale = euv.length() / edge.length();
                        let (c, s) = (euv.x / euv.length(), euv.y / euv.length());
                        let uvs = self
                            .face_points(other)
                            .iter()
                            .map(|p| {
                                let l = DVec2::new((*p - pa).dot(x), (*p - pa).dot(y)) * scale;
                                uva + DVec2::new(l.x * c - l.y * s, l.x * s + l.y * c)
                            })
                            .collect();
                        done.insert(other, uvs);
                        queue.push_back(other);
                    }
                }
            }
        }
        for (fi, uvs) in done {
            self.faces[fi].uvs = uvs.into_iter().map(|u| to_f32(u / repeat)).collect();
        }
    }

    /// Groups `faces` into UV islands: faces whose explicit UVs already join along a shared edge (what
    /// Box and Unfold produce). Faces without matching explicit UVs are dropped.
    fn uv_islands(&self, faces: &[usize]) -> Vec<Vec<usize>> {
        let valid: Vec<usize> = faces.iter().copied().filter(|&f| f < self.faces.len() && self.faces[f].uvs.len() == self.faces[f].indices.len()).collect();
        let mut parent: BTreeMap<usize, usize> = valid.iter().map(|&f| (f, f)).collect();
        fn find(parent: &mut BTreeMap<usize, usize>, x: usize) -> usize {
            let mut r = x;
            while parent[&r] != r {
                r = parent[&r];
            }
            let mut c = x;
            while parent[&c] != r {
                let next = parent[&c];
                parent.insert(c, r);
                c = next;
            }
            r
        }
        let in_set: BTreeSet<usize> = valid.iter().copied().collect();
        let edge_faces = self.edge_faces();
        for &f in &valid {
            let idx = self.faces[f].indices.clone();
            for k in 0..idx.len() {
                let (a, b) = (idx[k], idx[(k + 1) % idx.len()]);
                let (uva, uvb) = (self.faces[f].uvs[k], self.faces[f].uvs[(k + 1) % idx.len()]);
                for &other in edge_faces.get(&edge_key(a, b)).map(|v| v.as_slice()).unwrap_or(&[]) {
                    if other == f || !in_set.contains(&other) {
                        continue;
                    }
                    // The islands join only where the shared edge carries the same UV on both faces.
                    let of = &self.faces[other];
                    let uv_at = |vertex: u32| of.indices.iter().position(|v| *v == vertex).map(|p| of.uvs[p]);
                    let matches = |want: [f32; 2], got: Option<[f32; 2]>| got.is_some_and(|g| (g[0] - want[0]).abs() < 1e-4 && (g[1] - want[1]).abs() < 1e-4);
                    if matches(uva, uv_at(a)) && matches(uvb, uv_at(b)) {
                        let (ra, rb) = (find(&mut parent, f), find(&mut parent, other));
                        if ra != rb {
                            parent.insert(ra, rb);
                        }
                    }
                }
            }
        }
        let mut groups: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
        for &f in &valid {
            let r = find(&mut parent, f);
            groups.entry(r).or_default().push(f);
        }
        groups.into_values().collect()
    }

    /// Packs the explicit-UV islands of `faces` into the 0..1 square with a consistent texel density,
    /// so an atlas wastes little space. When `stack` is set, islands of the same shape and size are laid
    /// on top of each other to share one region, which is ideal for repeated trims and tiles.
    pub fn pack_uv_islands(&mut self, faces: &[usize], stack: bool) {
        let islands = self.uv_islands(faces);
        if islands.is_empty() {
            return;
        }
        // Per island: UV min, size, and a shape signature (corner offsets from the min, rounded).
        struct Island {
            faces: Vec<usize>,
            min: DVec2,
            size: DVec2,
            sig: Vec<(i32, i32)>,
        }
        let mut items: Vec<Island> = islands
            .into_iter()
            .map(|fs| {
                let (lo, hi) = fs
                    .iter()
                    .flat_map(|f| self.faces[*f].uvs.iter())
                    .fold((DVec2::MAX, DVec2::MIN), |(lo, hi), u| (lo.min(DVec2::new(u[0] as f64, u[1] as f64)), hi.max(DVec2::new(u[0] as f64, u[1] as f64))));
                let min = lo;
                let size = (hi - lo).max(DVec2::splat(1e-6));
                let mut sig: Vec<(i32, i32)> = fs
                    .iter()
                    .flat_map(|f| self.faces[*f].uvs.iter())
                    .map(|u| (((u[0] as f64 - min.x) * 1e3).round() as i32, ((u[1] as f64 - min.y) * 1e3).round() as i32))
                    .collect();
                sig.sort_unstable();
                Island { faces: fs, min, size, sig }
            })
            .collect();
        items.sort_by(|a, b| a.faces[0].cmp(&b.faces[0]));

        // Representatives: unique shapes when stacking, every island otherwise. Each gets one cell.
        let mut reps: Vec<usize> = Vec::new();
        let mut cell_of: Vec<usize> = vec![0; items.len()];
        for i in 0..items.len() {
            let found = stack.then(|| reps.iter().position(|&r| items[r].sig == items[i].sig)).flatten();
            match found {
                Some(slot) => cell_of[i] = slot,
                None => {
                    cell_of[i] = reps.len();
                    reps.push(i);
                }
            }
        }

        let cols = (reps.len() as f64).sqrt().ceil().max(1.0) as usize;
        let cell = 1.0 / cols as f64;
        let margin = 0.96;
        // One scale for every island keeps texel density uniform across the atlas.
        let max_dim = reps.iter().map(|&r| items[r].size.max_element()).fold(1e-6, f64::max);
        let scale = cell * margin / max_dim;

        for (i, item) in items.iter().enumerate() {
            let rep = &items[reps[cell_of[i]]];
            let (col, row) = (cell_of[i] % cols, cell_of[i] / cols);
            let cell_origin = DVec2::new(col as f64 * cell, row as f64 * cell);
            // Centre the shape in its cell using the representative's size, so stacked twins line up.
            let pad = (DVec2::splat(cell) - rep.size * scale) * 0.5;
            let offset = cell_origin + pad - item.min * scale;
            for &f in &item.faces {
                for u in &mut self.faces[f].uvs {
                    let p = DVec2::new(u[0] as f64, u[1] as f64) * scale + offset;
                    *u = [p.x as f32, p.y as f32];
                }
            }
        }
    }

    /// Scales and moves the explicit UVs of `faces` together so they span 0..1, keeping their aspect ratio when asked.
    pub fn normalize_uvs(&mut self, faces: &[usize], keep_aspect: bool) {
        let faces: Vec<usize> = faces.iter().copied().filter(|f| *f < self.faces.len() && self.faces[*f].uvs.len() == self.faces[*f].indices.len()).collect();
        let (lo, hi) = faces
            .iter()
            .flat_map(|f| self.faces[*f].uvs.iter())
            .fold(([f32::MAX; 2], [f32::MIN; 2]), |(lo, hi), u| ([lo[0].min(u[0]), lo[1].min(u[1])], [hi[0].max(u[0]), hi[1].max(u[1])]));
        let mut size = [(hi[0] - lo[0]).max(1e-6), (hi[1] - lo[1]).max(1e-6)];
        if keep_aspect {
            let m = size[0].max(size[1]);
            size = [m, m];
        }
        for f in faces {
            for u in &mut self.faces[f].uvs {
                *u = [(u[0] - lo[0]) / size[0], (u[1] - lo[1]) / size[1]];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh_shapes;
    use gt_core::Aabb;

    fn shared_corner_uvs(m: &Mesh, f1: usize, f2: usize) -> Vec<([f32; 2], [f32; 2])> {
        let mut out = Vec::new();
        for (k1, v1) in m.faces[f1].indices.iter().enumerate() {
            for (k2, v2) in m.faces[f2].indices.iter().enumerate() {
                if v1 == v2 {
                    out.push((m.faces[f1].uvs[k1], m.faces[f2].uvs[k2]));
                }
            }
        }
        out
    }

    #[test]
    fn unfold_keeps_neighbouring_faces_continuous() {
        let mut grid = mesh_shapes::grid(&Aabb::new(DVec3::ZERO, DVec3::new(128.0, 0.0, 128.0)), 4, 4, "m");
        // Bend the grid so the unfold has to rotate faces.
        for v in &mut grid.vertices {
            v.y = (v.x / 32.0).floor() * 8.0;
        }
        grid.project_uvs(&[], UvProjection::Unfold, DVec2::splat(64.0));
        let edges = grid.edge_faces();
        let mut checked = 0;
        for faces in edges.values().filter(|f| f.len() == 2) {
            for (a, b) in shared_corner_uvs(&grid, faces[0], faces[1]) {
                assert!((a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4, "{a:?} {b:?}");
                checked += 1;
            }
        }
        assert!(checked > 20);
        // Every face keeps the same UV winding, nothing is folded back over its neighbour.
        let signed_area = |uvs: &[[f32; 2]]| {
            (0..uvs.len())
                .map(|k| {
                    let (a, b) = (uvs[k], uvs[(k + 1) % uvs.len()]);
                    a[0] * b[1] - b[0] * a[1]
                })
                .sum::<f32>()
        };
        let signs: Vec<bool> = grid.faces.iter().map(|f| signed_area(&f.uvs) > 0.0).collect();
        assert!(signs.iter().all(|s| *s == signs[0]), "{signs:?}");
    }

    fn quad(mesh: &mut Mesh, at: DVec3) {
        let base = mesh.vertices.len() as u32;
        for c in [DVec3::ZERO, DVec3::new(1.0, 0.0, 0.0), DVec3::new(1.0, 1.0, 0.0), DVec3::new(0.0, 1.0, 0.0)] {
            mesh.vertices.push(at + c);
        }
        let mut f = crate::MeshFace::new(vec![base, base + 1, base + 2, base + 3], crate::FaceData::new("m", FaceUv::default()));
        // A 2x2 texture-space island, identical for every quad so stacking can fold them together.
        f.uvs = vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]];
        mesh.faces.push(f);
    }

    #[test]
    fn pack_stacks_identical_islands_and_fills_unit_square() {
        let mut mesh = Mesh::default();
        quad(&mut mesh, DVec3::ZERO);
        quad(&mut mesh, DVec3::new(10.0, 0.0, 0.0)); // disconnected, same shape

        let mut stacked = mesh.clone();
        stacked.pack_uv_islands(&[0, 1], true);
        assert_eq!(stacked.faces[0].uvs, stacked.faces[1].uvs, "identical islands overlap when stacked");
        for u in stacked.faces.iter().flat_map(|f| f.uvs.iter()) {
            assert!((-1e-4..=1.0001).contains(&u[0]) && (-1e-4..=1.0001).contains(&u[1]), "packed into 0..1: {u:?}");
        }

        let mut apart = mesh.clone();
        apart.pack_uv_islands(&[0, 1], false);
        assert_ne!(apart.faces[0].uvs, apart.faces[1].uvs, "without stacking each island gets its own cell");
        let max0 = apart.faces[0].uvs.iter().map(|u| u[0]).fold(f32::MIN, f32::max);
        let min1 = apart.faces[1].uvs.iter().map(|u| u[0]).fold(f32::MAX, f32::min);
        assert!(max0 <= min1 + 1e-4, "cells do not overlap along u");
        for u in apart.faces.iter().flat_map(|f| f.uvs.iter()) {
            assert!((-1e-4..=1.0001).contains(&u[0]) && (-1e-4..=1.0001).contains(&u[1]), "packed into 0..1: {u:?}");
        }
    }

    #[test]
    fn cylinder_projection_wraps_the_circumference() {
        let mut c = mesh_shapes::cylinder(&Aabb::new(DVec3::new(-32.0, 0.0, -32.0), DVec3::new(32.0, 64.0, 32.0)), 16, "m");
        let sides: Vec<usize> = (0..c.faces.len()).filter(|f| c.face_normal(*f).y.abs() < 0.5).collect();
        let circumference = std::f64::consts::TAU * 32.0;
        c.project_uvs(&sides, UvProjection::Cylinder { axis: DVec3::Y }, DVec2::new(circumference, 64.0));
        for &f in &sides {
            let us: Vec<f32> = c.faces[f].uvs.iter().map(|u| u[0]).collect();
            let span = us.iter().cloned().fold(f32::MIN, f32::max) - us.iter().cloned().fold(f32::MAX, f32::min);
            assert!(span < 0.1, "each side covers about 1/16 of the texture, got {span}");
        }
        c.normalize_uvs(&sides, false);
        let all: Vec<[f32; 2]> = sides.iter().flat_map(|f| c.faces[*f].uvs.clone()).collect();
        assert!(all.iter().all(|u| (-1e-4..=1.0001).contains(&u[0]) && (-1e-4..=1.0001).contains(&u[1])));
    }
}
