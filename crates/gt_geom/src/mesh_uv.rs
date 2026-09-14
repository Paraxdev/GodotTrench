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
