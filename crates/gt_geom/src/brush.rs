use std::collections::BTreeMap;

use gt_core::{Aabb, DMat4, DVec2, DVec3, EPSILON, Plane, PlaneSide, Ray};
use serde::{Deserialize, Serialize};

use crate::polygon;
use crate::uv::FaceUv;

const WORLD_EXTENT: f64 = 1_048_576.0;
const WELD_EPSILON: f64 = 1e-4;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum BrushError {
    #[error("brush has fewer than four faces")]
    TooFewFaces,
    #[error("brush is empty or has no volume")]
    Empty,
    #[error("brush is not closed")]
    NotClosed,
    #[error("brush is not convex")]
    NotConvex,
    #[error("face {0} is degenerate")]
    DegenerateFace(usize),
}

/// Surface attributes carried by a face through geometry operations.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FaceData {
    #[serde(default)]
    pub material: String,
    #[serde(default)]
    pub uv: FaceUv,
    /// Free-form per-face attributes (surface flags, collision layers, smoothing groups).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub props: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disp: Option<crate::displacement::Displacement>,
    /// Vertex paint, one RGBA color per entry of the face's `indices`. Empty means unpainted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub colors: Vec<[f32; 4]>,
}

impl FaceData {
    pub fn new(material: impl Into<String>, uv: FaceUv) -> Self {
        Self { material: material.into(), uv, props: BTreeMap::new(), disp: None, colors: Vec::new() }
    }

    pub fn with_normal(material: impl Into<String>, normal: DVec3) -> Self {
        Self::new(material, FaceUv::paraxial(normal, DVec2::ONE))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Face {
    /// Counter-clockwise when seen from outside the brush.
    pub indices: Vec<u32>,
    #[serde(flatten)]
    pub data: FaceData,
    #[serde(skip)]
    pub plane: Plane,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(from = "RawBrush")]
pub struct Brush {
    pub vertices: Vec<DVec3>,
    pub faces: Vec<Face>,
}

#[derive(Deserialize)]
struct RawBrush {
    vertices: Vec<DVec3>,
    faces: Vec<Face>,
}

impl From<RawBrush> for Brush {
    fn from(raw: RawBrush) -> Self {
        let mut b = Brush { vertices: raw.vertices, faces: raw.faces };
        b.update_planes();
        b
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RayHit {
    pub distance: f64,
    pub face: usize,
    pub point: DVec3,
}

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub positions: [DVec3; 3],
    pub uvs: [DVec2; 3],
    pub normal: DVec3,
}

impl Brush {
    /// Intersection of half-spaces. Each plane's normal points out of the brush.
    pub fn from_planes(planes: Vec<(Plane, FaceData)>) -> Result<Brush, BrushError> {
        let mut unique: Vec<(Plane, FaceData)> = Vec::with_capacity(planes.len());
        for (plane, data) in planes {
            if !unique.iter().any(|(p, _)| p.approx_eq(&plane, 1e-9, 1e-6)) {
                unique.push((plane, data));
            }
        }

        let mut vertices: Vec<DVec3> = Vec::new();
        let mut faces: Vec<Face> = Vec::new();
        for (i, (plane, data)) in unique.iter().enumerate() {
            let mut winding = polygon::base_winding(plane, WORLD_EXTENT);
            for (j, (other, _)) in unique.iter().enumerate() {
                if i == j {
                    continue;
                }

                winding = polygon::clip_back(&winding, other);
                if winding.len() < 3 {
                    break;
                }
            }

            if winding.len() < 3 || polygon::area(&winding) < 1e-8 {
                continue;
            }

            let mut indices: Vec<u32> = Vec::with_capacity(winding.len());
            for p in winding {
                let p = gt_core::snap_vec(p);
                let idx = match vertices.iter().position(|v| (*v - p).abs().max_element() < WELD_EPSILON) {
                    Some(idx) => idx,
                    None => {
                        vertices.push(p);
                        vertices.len() - 1
                    }
                } as u32;
                if indices.last() != Some(&idx) {
                    indices.push(idx);
                }
            }

            while indices.len() > 1 && indices.first() == indices.last() {
                indices.pop();
            }

            if indices.len() < 3 {
                continue;
            }

            faces.push(Face { indices, data: data.clone(), plane: *plane });
        }

        let mut brush = Brush { vertices, faces };
        brush.remove_unused_vertices();
        brush.check()?;
        Ok(brush)
    }

    pub fn from_aabb(bounds: &Aabb, material: &str) -> Result<Brush, BrushError> {
        let planes = [
            (DVec3::X, bounds.max.x),
            (DVec3::NEG_X, -bounds.min.x),
            (DVec3::Y, bounds.max.y),
            (DVec3::NEG_Y, -bounds.min.y),
            (DVec3::Z, bounds.max.z),
            (DVec3::NEG_Z, -bounds.min.z),
        ];
        Brush::from_planes(planes.iter().map(|(n, d)| (Plane::new(*n, *d), FaceData::with_normal(material, *n))).collect())
    }

    /// Convex hull of the points; face data comes from `template` faces with the closest plane.
    pub fn from_points(points: &[DVec3], template: &[(Plane, FaceData)], default_material: &str) -> Result<Brush, BrushError> {
        let planes = crate::hull::convex_hull_planes(points).ok_or(BrushError::Empty)?;
        let with_data = planes
            .into_iter()
            .map(|plane| {
                let data = best_face_data(&plane, template).cloned().unwrap_or_else(|| FaceData::with_normal(default_material, plane.normal));
                (plane, data)
            })
            .collect();
        Brush::from_planes(with_data)
    }

    pub fn update_planes(&mut self) {
        for face in &mut self.faces {
            let pts: Vec<DVec3> = face.indices.iter().map(|i| self.vertices[*i as usize]).collect();
            if let Some(p) = Plane::from_polygon(&pts) {
                face.plane = p;
            }
        }
    }

    fn remove_unused_vertices(&mut self) {
        let mut used = vec![false; self.vertices.len()];
        for f in &self.faces {
            for i in &f.indices {
                used[*i as usize] = true;
            }
        }

        let mut remap = vec![0u32; self.vertices.len()];
        let mut out = Vec::with_capacity(self.vertices.len());
        for (i, v) in self.vertices.iter().enumerate() {
            if used[i] {
                remap[i] = out.len() as u32;
                out.push(*v);
            }
        }

        for f in &mut self.faces {
            for i in &mut f.indices {
                *i = remap[*i as usize];
            }
        }

        self.vertices = out;
    }

    fn check(&self) -> Result<(), BrushError> {
        if self.faces.is_empty() {
            return Err(BrushError::Empty);
        }

        if self.faces.len() < 4 {
            return Err(BrushError::TooFewFaces);
        }

        Ok(())
    }

    /// Full validation: closed manifold, planar faces, convex.
    pub fn validate(&self) -> Result<(), BrushError> {
        self.check()?;
        let mut edges: BTreeMap<(u32, u32), i32> = BTreeMap::new();
        for (fi, f) in self.faces.iter().enumerate() {
            if f.indices.len() < 3 {
                return Err(BrushError::DegenerateFace(fi));
            }

            for k in 0..f.indices.len() {
                let a = f.indices[k];
                let b = f.indices[(k + 1) % f.indices.len()];
                *edges.entry((a, b)).or_default() += 1;
                let v = self.vertices[a as usize];
                if f.plane.distance(v).abs() > 1e-3 {
                    return Err(BrushError::NotConvex);
                }
            }
        }

        for (a, b) in edges.keys() {
            if edges.get(&(*b, *a)) != Some(&1) {
                return Err(BrushError::NotClosed);
            }
        }

        for f in &self.faces {
            for v in &self.vertices {
                if f.plane.distance(*v) > 1e-3 {
                    return Err(BrushError::NotConvex);
                }
            }
        }

        Ok(())
    }

    pub fn planes(&self) -> Vec<(Plane, FaceData)> {
        self.faces.iter().map(|f| (f.plane, f.data.clone())).collect()
    }

    pub fn bounds(&self) -> Aabb {
        Aabb::from_points(self.vertices.iter().copied())
    }

    pub fn center(&self) -> DVec3 {
        polygon::centroid(&self.vertices)
    }

    pub fn face_points(&self, face: usize) -> Vec<DVec3> {
        self.faces[face].indices.iter().map(|i| self.vertices[*i as usize]).collect()
    }

    pub fn face_center(&self, face: usize) -> DVec3 {
        polygon::centroid(&self.face_points(face))
    }

    /// Unique undirected edges as vertex index pairs.
    pub fn edges(&self) -> Vec<(u32, u32)> {
        let mut out: Vec<(u32, u32)> = Vec::new();
        for f in &self.faces {
            for k in 0..f.indices.len() {
                let a = f.indices[k];
                let b = f.indices[(k + 1) % f.indices.len()];
                let e = (a.min(b), a.max(b));
                if !out.contains(&e) {
                    out.push(e);
                }
            }
        }

        out
    }

    pub fn volume(&self) -> f64 {
        let c = self.center();
        let mut vol = 0.0;
        for f in &self.faces {
            let p0 = self.vertices[f.indices[0] as usize];
            for k in 1..f.indices.len() - 1 {
                let p1 = self.vertices[f.indices[k] as usize];
                let p2 = self.vertices[f.indices[k + 1] as usize];
                vol += (p0 - c).dot((p1 - c).cross(p2 - c)) / 6.0;
            }
        }

        vol
    }

    pub fn contains_point(&self, p: DVec3) -> bool {
        self.faces.iter().all(|f| f.plane.distance(p) <= EPSILON)
    }

    /// True if the two brushes share interior volume (touching does not count).
    pub fn intersects(&self, other: &Brush) -> bool {
        if !self.bounds().overlaps(&other.bounds(), EPSILON) {
            return false;
        }

        let mut planes = self.planes();
        planes.extend(other.planes());
        Brush::from_planes(planes).map(|b| b.volume() > 1e-6).unwrap_or(false)
    }

    pub fn contains_brush(&self, other: &Brush) -> bool {
        other.vertices.iter().all(|v| self.contains_point(*v))
    }

    pub fn ray_cast(&self, ray: &Ray) -> Option<RayHit> {
        let mut t_enter = f64::NEG_INFINITY;
        let mut t_exit = f64::INFINITY;
        let mut enter_face = usize::MAX;
        for (i, f) in self.faces.iter().enumerate() {
            let denom = f.plane.normal.dot(ray.dir);
            let dist = f.plane.distance(ray.origin);
            if denom.abs() < 1e-12 {
                if dist > EPSILON {
                    return None;
                }

                continue;
            }

            let t = -dist / denom;
            if denom < 0.0 {
                if t > t_enter {
                    t_enter = t;
                    enter_face = i;
                }
            } else if t < t_exit {
                t_exit = t;
            }

            if t_enter > t_exit {
                return None;
            }
        }

        if enter_face == usize::MAX || t_enter < 0.0 {
            return None;
        }

        Some(RayHit { distance: t_enter, face: enter_face, point: ray.at(t_enter) })
    }

    /// Applies an affine transform. With `uv_lock`, textures stay attached to the surfaces.
    pub fn transformed(&self, m: &DMat4, uv_lock: bool) -> Brush {
        let mirror = m.determinant() < 0.0;
        let vertices = self.vertices.iter().map(|v| gt_core::snap_vec(m.transform_point3(*v))).collect();
        let faces = self
            .faces
            .iter()
            .map(|f| {
                let mut indices = f.indices.clone();
                let mut data = f.data.clone();
                if mirror {
                    indices.reverse();
                    data.colors.reverse();
                    // [a,b,c,d] -> [d,c,b,a] keeps the u direction and flips v.
                    if let Some(d) = &mut data.disp {
                        d.flip_v();
                    }
                }

                if uv_lock {
                    data.uv = data.uv.transformed(m);
                }

                Face { indices, data, plane: f.plane }
            })
            .collect();
        let mut b = Brush { vertices, faces };
        b.update_planes();
        if !uv_lock {
            for f in &mut b.faces {
                if f.data.uv.is_degenerate_for(f.plane.normal) {
                    let scale = f.data.uv.scale;
                    f.data.uv = FaceUv::paraxial(f.plane.normal, scale);
                }
            }
        }

        b
    }

    pub fn translated(&self, offset: DVec3, uv_lock: bool) -> Brush {
        let mut b = self.clone();
        for v in &mut b.vertices {
            *v = gt_core::snap_vec(*v + offset);
        }

        for f in &mut b.faces {
            f.plane = f.plane.translated(offset);
            if uv_lock {
                f.data.uv = f.data.uv.translated(offset);
            }
        }

        b
    }

    /// Splits by a plane into (front, back). The new cap face uses `cap` data.
    pub fn split(&self, plane: &Plane, cap: &FaceData) -> (Option<Brush>, Option<Brush>) {
        let mut front = 0;
        let mut back = 0;
        for v in &self.vertices {
            match plane.side(*v) {
                PlaneSide::Front => front += 1,
                PlaneSide::Back => back += 1,
                PlaneSide::On => {}
            }
        }

        if back == 0 {
            return (Some(self.clone()), None);
        }

        if front == 0 {
            return (None, Some(self.clone()));
        }

        let mut back_planes = self.planes();
        back_planes.push((*plane, cap.clone()));
        let mut front_planes = self.planes();
        front_planes.push((plane.flipped(), cap.clone()));
        (Brush::from_planes(front_planes).ok(), Brush::from_planes(back_planes).ok())
    }

    pub fn set_material(&mut self, material: &str) {
        for f in &mut self.faces {
            f.data.material = material.to_string();
        }
    }

    pub fn find_face_by_plane(&self, plane: &Plane) -> Option<usize> {
        self.faces.iter().position(|f| f.plane.approx_eq(plane, 1e-6, 1e-4))
    }

    /// Moves one face along its normal and rebuilds. Fails if the result would be invalid.
    pub fn move_face(&self, face: usize, offset: DVec3, uv_lock: bool) -> Result<Brush, BrushError> {
        let mut planes = self.planes();
        planes[face].0 = planes[face].0.translated(offset);
        if uv_lock {
            planes[face].1.uv = planes[face].1.uv.translated(offset);
        }

        let b = Brush::from_planes(planes)?;
        if b.faces.len() != self.faces.len() {
            return Err(BrushError::NotConvex);
        }

        Ok(b)
    }

    /// Fan triangulation with texel UVs. `tex_size` maps materials to pixel sizes.
    pub fn triangles(&self, mut tex_size: impl FnMut(&str) -> DVec2) -> Vec<(usize, Triangle)> {
        let mut out = Vec::new();
        for (fi, f) in self.faces.iter().enumerate() {
            let size = tex_size(&f.data.material);
            let p0 = self.vertices[f.indices[0] as usize];
            for k in 1..f.indices.len() - 1 {
                let p1 = self.vertices[f.indices[k] as usize];
                let p2 = self.vertices[f.indices[k + 1] as usize];
                out.push((
                    fi,
                    Triangle { positions: [p0, p1, p2], uvs: [f.data.uv.uv(p0, size), f.data.uv.uv(p1, size), f.data.uv.uv(p2, size)], normal: f.plane.normal },
                ));
            }
        }

        out
    }
}

pub fn best_face_data<'a>(plane: &Plane, template: &'a [(Plane, FaceData)]) -> Option<&'a FaceData> {
    if let Some((_, d)) = template.iter().find(|(p, _)| p.approx_eq(plane, 1e-6, 1e-3)) {
        return Some(d);
    }

    template.iter().max_by(|(a, _), (b, _)| a.normal.dot(plane.normal).total_cmp(&b.normal.dot(plane.normal))).map(|(_, d)| d)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube(size: f64) -> Brush {
        Brush::from_aabb(&Aabb::new(DVec3::splat(-size), DVec3::splat(size)), "test").unwrap()
    }

    #[test]
    fn cube_topology() {
        let b = cube(16.0);
        assert_eq!(b.vertices.len(), 8);
        assert_eq!(b.faces.len(), 6);
        assert_eq!(b.edges().len(), 12);
        b.validate().unwrap();
        assert!((b.volume() - 32.0f64.powi(3)).abs() < 1e-6);
    }

    #[test]
    fn faces_wind_outward() {
        let b = cube(8.0);
        let c = b.center();
        for f in &b.faces {
            let pts: Vec<DVec3> = f.indices.iter().map(|i| b.vertices[*i as usize]).collect();
            let n = Plane::from_polygon(&pts).unwrap().normal;
            assert!(n.dot(polygon::centroid(&pts) - c) > 0.0);
        }
    }

    #[test]
    fn split_conserves_volume() {
        let b = cube(16.0);
        let plane = Plane::from_point_normal(DVec3::new(3.0, 0.0, 0.0), DVec3::new(1.0, 1.0, 0.3));
        let (f, k) = b.split(&plane, &FaceData::with_normal("cap", plane.normal));
        let (f, k) = (f.unwrap(), k.unwrap());
        f.validate().unwrap();
        k.validate().unwrap();
        assert!((f.volume() + k.volume() - b.volume()).abs() < 1e-3);
    }

    #[test]
    fn ray_hits_nearest_face() {
        let b = cube(16.0);
        let hit = b.ray_cast(&Ray::new(DVec3::new(0.0, 0.0, 100.0), DVec3::NEG_Z)).unwrap();
        assert!((hit.distance - 84.0).abs() < 1e-9);
        assert!(gt_core::vec_approx_eq(b.faces[hit.face].plane.normal, DVec3::Z));
        assert!(b.ray_cast(&Ray::new(DVec3::new(40.0, 0.0, 100.0), DVec3::NEG_Z)).is_none());
    }

    #[test]
    fn serde_round_trip() {
        let b = cube(4.0);
        let json = serde_json::to_string(&b).unwrap();
        let back: Brush = serde_json::from_str(&json).unwrap();
        assert_eq!(b.vertices, back.vertices);
        for (a, c) in b.faces.iter().zip(&back.faces) {
            assert!(a.plane.approx_eq(&c.plane, 1e-9, 1e-9));
        }
    }

    #[test]
    fn mirror_keeps_outward_winding() {
        let b = cube(8.0).transformed(&DMat4::from_scale(DVec3::new(-1.0, 1.0, 1.0)), true);
        b.validate().unwrap();
        assert!(b.volume() > 0.0);
    }
}
