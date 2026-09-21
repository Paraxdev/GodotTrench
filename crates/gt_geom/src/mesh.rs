//! Editable polygon meshes. Unlike brushes they may be concave, open or non-planar, and are edited per vertex, edge and face.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use gt_core::{Aabb, DMat4, DVec2, DVec3, Plane, Ray};
use serde::{Deserialize, Serialize};

use crate::brush::{Brush, BrushError, FaceData};
use crate::polygon;
use crate::uv::FaceUv;

const WELD: f64 = 1e-4;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum MeshError {
    #[error("mesh has no faces")]
    Empty,
    #[error("face {0} references a missing vertex")]
    BadIndex(usize),
    #[error("face {0} has fewer than three distinct vertices")]
    Degenerate(usize),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MeshFace {
    /// Counter-clockwise when seen from the front.
    pub indices: Vec<u32>,
    #[serde(flatten)]
    pub data: FaceData,
    /// Explicit corner UVs in texture space (1.0 is one texture width). Empty uses the planar projection in `data.uv`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub uvs: Vec<[f32; 2]>,
}

impl MeshFace {
    pub fn new(indices: Vec<u32>, data: FaceData) -> Self {
        Self { indices, data, uvs: Vec::new() }
    }

    fn corner_uv(&self, k: usize) -> Option<[f32; 2]> {
        (self.uvs.len() == self.indices.len()).then(|| self.uvs[k])
    }

    fn corner_color(&self, k: usize) -> Option<[f32; 4]> {
        (self.data.colors.len() == self.indices.len()).then(|| self.data.colors[k])
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Mesh {
    pub vertices: Vec<DVec3>,
    pub faces: Vec<MeshFace>,
    /// Corner normals are shared between faces meeting at less than this angle in degrees. 0 is flat shading.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub smooth_angle: f32,
    /// A decal sheet: a thin quad drawn with its texture's alpha cut out and double sided, laid over a
    /// surface. Set on meshes made by the decal tool, cleared on ordinary geometry.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub decal: bool,
}

fn is_zero(v: &f32) -> bool {
    *v == 0.0
}

pub fn edge_key(a: u32, b: u32) -> (u32, u32) {
    (a.min(b), a.max(b))
}

fn weld_key(p: DVec3) -> (i64, i64, i64) {
    ((p.x / WELD).round() as i64, (p.y / WELD).round() as i64, (p.z / WELD).round() as i64)
}

fn lerp_uv(a: [f32; 2], b: [f32; 2], t: f64) -> [f32; 2] {
    let t = t as f32;
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

fn lerp_color(a: [f32; 4], b: [f32; 4], t: f64) -> [f32; 4] {
    let t = t as f32;
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, a[3] + (b[3] - a[3]) * t]
}

/// Attributes of a new corner placed on the edge between corners `ka` and `kb` of a face.
#[derive(Clone, Copy)]
struct CornerAttr {
    uv: Option<[f32; 2]>,
    color: Option<[f32; 4]>,
}

impl CornerAttr {
    fn of(face: &MeshFace, k: usize) -> Self {
        Self { uv: face.corner_uv(k), color: face.corner_color(k) }
    }

    fn lerp(a: Self, b: Self, t: f64) -> Self {
        Self { uv: a.uv.zip(b.uv).map(|(x, y)| lerp_uv(x, y, t)), color: a.color.zip(b.color).map(|(x, y)| lerp_color(x, y, t)) }
    }

    fn average(items: &[Self]) -> Self {
        let n = items.len().max(1) as f32;
        let uv = items.iter().map(|c| c.uv).collect::<Option<Vec<_>>>().map(|v| {
            let s = v.iter().fold([0.0f32; 2], |acc, x| [acc[0] + x[0], acc[1] + x[1]]);
            [s[0] / n, s[1] / n]
        });
        let color = items.iter().map(|c| c.color).collect::<Option<Vec<_>>>().map(|v| {
            let s = v.iter().fold([0.0f32; 4], |acc, x| [acc[0] + x[0], acc[1] + x[1], acc[2] + x[2], acc[3] + x[3]]);
            [s[0] / n, s[1] / n, s[2] / n, s[3] / n]
        });
        Self { uv, color }
    }
}

/// Builds a face from corners and their attributes, keeping uvs and colors only if every corner has them.
fn face_from(indices: Vec<u32>, attrs: &[CornerAttr], data: &FaceData) -> MeshFace {
    let mut f = MeshFace::new(indices, data.clone());
    f.uvs = attrs.iter().map(|a| a.uv).collect::<Option<Vec<_>>>().unwrap_or_default();
    f.data.colors = attrs.iter().map(|a| a.color).collect::<Option<Vec<_>>>().unwrap_or_default();
    f.data.disp = None;
    f
}

impl Mesh {
    pub fn from_brush(brush: &Brush) -> Mesh {
        let has_disp = brush.faces.iter().any(|f| f.data.disp.is_some());
        if !has_disp {
            return Mesh {
                vertices: brush.vertices.clone(),
                faces: brush.faces.iter().map(|f| MeshFace::new(f.indices.clone(), f.data.clone())).collect(),
                smooth_angle: 0.0,
                decal: false,
            };
        }
        // Like Hammer, only the displacement surfaces of a displacement brush become geometry.
        let mut polys = Vec::new();
        for fi in 0..brush.faces.len() {
            let Some(grid) = crate::displacement::grid(brush, fi) else { continue };
            let n = grid.size;
            let mut data = brush.faces[fi].data.clone();
            data.disp = None;
            data.colors.clear();
            for j in 0..n - 1 {
                for i in 0..n - 1 {
                    let k = j * n + i;
                    polys.push((vec![grid.positions[k], grid.positions[k + 1], grid.positions[k + n + 1], grid.positions[k + n]], data.clone()));
                }
            }
        }
        let mut mesh = Mesh::from_polygons(polys);
        mesh.smooth_angle = 60.0;
        mesh
    }

    /// Builds a mesh from polygons, welding coincident vertices.
    pub fn from_polygons(polys: impl IntoIterator<Item = (Vec<DVec3>, FaceData)>) -> Mesh {
        let mut mesh = Mesh::default();
        let mut lookup: HashMap<(i64, i64, i64), u32> = HashMap::new();
        for (pts, data) in polys {
            let mut indices: Vec<u32> = Vec::with_capacity(pts.len());
            for p in pts {
                let idx = *lookup.entry(weld_key(p)).or_insert_with(|| {
                    mesh.vertices.push(p);
                    (mesh.vertices.len() - 1) as u32
                });
                if indices.last() != Some(&idx) {
                    indices.push(idx);
                }
            }
            while indices.len() > 1 && indices.first() == indices.last() {
                indices.pop();
            }
            if indices.len() >= 3 {
                mesh.faces.push(MeshFace::new(indices, data));
            }
        }
        mesh
    }

    pub fn bounds(&self) -> Aabb {
        Aabb::from_points(self.vertices.iter().copied())
    }

    pub fn center(&self) -> DVec3 {
        self.bounds().center()
    }

    pub fn face_points(&self, fi: usize) -> Vec<DVec3> {
        self.faces[fi].indices.iter().map(|i| self.vertices[*i as usize]).collect()
    }

    pub fn face_normal(&self, fi: usize) -> DVec3 {
        polygon::newell(&self.face_points(fi)).normalize_or(DVec3::Y)
    }

    pub fn face_center(&self, fi: usize) -> DVec3 {
        polygon::centroid(&self.face_points(fi))
    }

    pub fn face_plane(&self, fi: usize) -> Plane {
        Plane::from_point_normal(self.face_center(fi), self.face_normal(fi))
    }

    pub fn face_area(&self, fi: usize) -> f64 {
        polygon::newell(&self.face_points(fi)).length() * 0.5
    }

    fn face_edges(&self, fi: usize) -> impl Iterator<Item = (u32, u32)> + '_ {
        let idx = &self.faces[fi].indices;
        (0..idx.len()).map(move |k| (idx[k], idx[(k + 1) % idx.len()]))
    }

    /// Unique undirected edges.
    pub fn edges(&self) -> Vec<(u32, u32)> {
        let mut set = BTreeSet::new();
        for fi in 0..self.faces.len() {
            for (a, b) in self.face_edges(fi) {
                set.insert(edge_key(a, b));
            }
        }
        set.into_iter().collect()
    }

    pub fn edge_faces(&self) -> HashMap<(u32, u32), Vec<usize>> {
        let mut map: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
        for fi in 0..self.faces.len() {
            for (a, b) in self.face_edges(fi) {
                map.entry(edge_key(a, b)).or_default().push(fi);
            }
        }
        map
    }

    pub fn vertex_faces(&self) -> Vec<Vec<usize>> {
        let mut out = vec![Vec::new(); self.vertices.len()];
        for (fi, f) in self.faces.iter().enumerate() {
            for v in &f.indices {
                if let Some(list) = out.get_mut(*v as usize)
                    && list.last() != Some(&fi)
                {
                    list.push(fi);
                }
            }
        }
        out
    }

    fn directed_edges(&self) -> HashMap<(u32, u32), usize> {
        let mut map = HashMap::new();
        for fi in 0..self.faces.len() {
            for e in self.face_edges(fi) {
                map.insert(e, fi);
            }
        }
        map
    }

    /// Edges used by only one face, in that face's direction.
    pub fn boundary_edges(&self) -> Vec<(u32, u32)> {
        let directed = self.directed_edges();
        let mut out: Vec<(u32, u32)> = directed.keys().filter(|(a, b)| !directed.contains_key(&(*b, *a))).copied().collect();
        out.sort();
        out
    }

    pub fn is_closed(&self) -> bool {
        !self.faces.is_empty() && self.boundary_edges().is_empty()
    }

    /// Vertex index triples of the face triangulation, with the face's winding.
    pub fn triangulate_face(&self, fi: usize) -> Vec<[u32; 3]> {
        let f = &self.faces[fi];
        let pts = self.face_points(fi);
        polygon::triangulate(&pts, polygon::newell(&pts)).into_iter().map(|[a, b, c]| [f.indices[a], f.indices[b], f.indices[c]]).collect()
    }

    /// Corner triangles of the face: indices into the face's corners.
    pub fn triangulate_corners(&self, fi: usize) -> Vec<[usize; 3]> {
        let pts = self.face_points(fi);
        polygon::triangulate(&pts, polygon::newell(&pts))
    }

    pub fn triangle_count(&self) -> usize {
        self.faces.iter().map(|f| f.indices.len().saturating_sub(2)).sum()
    }

    /// Shading normal of every face corner, honouring `smooth_angle`.
    pub fn corner_normals(&self) -> Vec<Vec<DVec3>> {
        let raw: Vec<DVec3> = (0..self.faces.len()).map(|fi| polygon::newell(&self.face_points(fi))).collect();
        let unit: Vec<DVec3> = raw.iter().map(|n| n.normalize_or(DVec3::Y)).collect();
        if self.smooth_angle <= 0.0 {
            return self.faces.iter().enumerate().map(|(fi, f)| vec![unit[fi]; f.indices.len()]).collect();
        }
        let cos = (self.smooth_angle as f64).to_radians().cos() - 1e-6;
        let vertex_faces = self.vertex_faces();
        self.faces
            .iter()
            .enumerate()
            .map(|(fi, f)| {
                f.indices
                    .iter()
                    .map(|v| {
                        let sum: DVec3 = vertex_faces[*v as usize].iter().filter(|g| unit[**g].dot(unit[fi]) >= cos).map(|g| raw[*g]).sum();
                        sum.normalize_or(unit[fi])
                    })
                    .collect()
            })
            .collect()
    }

    /// Texture coordinate of a corner for a texture of the given pixel size.
    pub fn corner_uv(&self, fi: usize, k: usize, tex_size: DVec2) -> DVec2 {
        let f = &self.faces[fi];
        match f.corner_uv(k) {
            Some(uv) => DVec2::new(uv[0] as f64, uv[1] as f64),
            None => f.data.uv.uv(self.vertices[f.indices[k] as usize], tex_size),
        }
    }

    /// Nearest double sided hit: (distance, face).
    pub fn ray_cast(&self, ray: &Ray) -> Option<(f64, usize)> {
        let b = self.bounds().expanded(0.01);
        if !b.contains_point(ray.origin) && ray.intersect_aabb(&b).is_none() {
            return None;
        }
        let mut best: Option<(f64, usize)> = None;
        for fi in 0..self.faces.len() {
            for [a, bb, c] in self.triangulate_face(fi) {
                if let Some(t) = ray.intersect_triangle(self.vertices[a as usize], self.vertices[bb as usize], self.vertices[c as usize])
                    && best.is_none_or(|(bt, _)| t < bt)
                {
                    best = Some((t, fi));
                }
            }
        }
        best
    }

    pub fn transformed(&self, m: &DMat4, uv_lock: bool) -> Mesh {
        let mut out = self.clone();
        let mirror = m.determinant() < 0.0;
        for v in &mut out.vertices {
            *v = gt_core::snap_vec(m.transform_point3(*v));
        }
        for f in &mut out.faces {
            if mirror {
                f.indices.reverse();
                f.uvs.reverse();
                f.data.colors.reverse();
            }
            if uv_lock {
                f.data.uv = f.data.uv.transformed(m);
            }
        }
        if !uv_lock {
            out.refresh_degenerate_uvs();
        }
        out
    }

    pub fn translated(&self, offset: DVec3, uv_lock: bool) -> Mesh {
        let mut out = self.clone();
        for v in &mut out.vertices {
            *v = gt_core::snap_vec(*v + offset);
        }
        if uv_lock {
            for f in &mut out.faces {
                f.data.uv = f.data.uv.translated(offset);
            }
        }
        out
    }

    /// Faces whose planar projection collapsed (after vertex edits) get a fresh paraxial projection.
    pub fn refresh_degenerate_uvs(&mut self) {
        for fi in 0..self.faces.len() {
            let n = self.face_normal(fi);
            if self.faces[fi].data.uv.is_degenerate_for(n) && self.face_area(fi) > 1e-9 {
                let scale = self.faces[fi].data.uv.scale;
                self.faces[fi].data.uv = FaceUv::paraxial(n, scale);
            }
        }
    }

    pub fn validate(&self) -> Result<(), MeshError> {
        if self.faces.is_empty() {
            return Err(MeshError::Empty);
        }
        for (fi, f) in self.faces.iter().enumerate() {
            if f.indices.iter().any(|i| *i as usize >= self.vertices.len()) {
                return Err(MeshError::BadIndex(fi));
            }
            let unique: BTreeSet<u32> = f.indices.iter().copied().collect();
            if unique.len() < 3 {
                return Err(MeshError::Degenerate(fi));
            }
        }
        Ok(())
    }

    /// Signed volume, meaningful for closed meshes.
    pub fn volume(&self) -> f64 {
        let mut vol = 0.0;
        for fi in 0..self.faces.len() {
            for [a, b, c] in self.triangulate_face(fi) {
                let (pa, pb, pc) = (self.vertices[a as usize], self.vertices[b as usize], self.vertices[c as usize]);
                vol += pa.dot(pb.cross(pc)) / 6.0;
            }
        }
        vol
    }

    /// Removes repeated corners, faces with fewer than three distinct vertices and unused vertices.
    pub fn cleanup(&mut self) {
        for f in &mut self.faces {
            let mut keep = Vec::with_capacity(f.indices.len());
            for k in 0..f.indices.len() {
                if f.indices[k] != f.indices[(k + 1) % f.indices.len()] {
                    keep.push(k);
                }
            }
            if keep.len() != f.indices.len() {
                let uvs_ok = f.uvs.len() == f.indices.len();
                let colors_ok = f.data.colors.len() == f.indices.len();
                f.uvs = if uvs_ok { keep.iter().map(|k| f.uvs[*k]).collect() } else { Vec::new() };
                f.data.colors = if colors_ok { keep.iter().map(|k| f.data.colors[*k]).collect() } else { Vec::new() };
                f.indices = keep.iter().map(|k| f.indices[*k]).collect();
            }
        }
        self.faces.retain(|f| f.indices.iter().collect::<BTreeSet<_>>().len() >= 3);
        self.remove_unused_vertices();
    }

    /// Returns the old to new index map.
    pub fn remove_unused_vertices(&mut self) -> Vec<Option<u32>> {
        let mut used = vec![false; self.vertices.len()];
        for f in &self.faces {
            for i in &f.indices {
                used[*i as usize] = true;
            }
        }
        let mut remap = vec![None; self.vertices.len()];
        let mut out = Vec::with_capacity(self.vertices.len());
        for (i, v) in self.vertices.iter().enumerate() {
            if used[i] {
                remap[i] = Some(out.len() as u32);
                out.push(*v);
            }
        }
        for f in &mut self.faces {
            for i in &mut f.indices {
                *i = remap[*i as usize].unwrap_or(0);
            }
        }
        self.vertices = out;
        remap
    }

    /// Merges vertices closer than `eps`. Returns how many were merged.
    pub fn weld(&mut self, eps: f64) -> usize {
        let all: Vec<u32> = (0..self.vertices.len() as u32).collect();
        self.merge_by_distance(&all, eps)
    }

    /// Closed and every vertex behind every face plane.
    pub fn is_convex(&self) -> bool {
        self.is_closed()
            && (0..self.faces.len()).all(|fi| {
                let plane = self.face_plane(fi);
                self.vertices.iter().all(|v| plane.distance(*v) <= 1e-3)
            })
    }

    /// Convex hull as a brush, face attributes taken from the closest mesh faces.
    pub fn to_brush(&self) -> Result<Brush, BrushError> {
        let template: Vec<(Plane, FaceData)> = (0..self.faces.len()).map(|fi| (self.face_plane(fi), self.faces[fi].data.clone())).collect();
        Brush::from_points(&self.vertices, &template, "")
    }

    // ------------------------------------------------------------------ component edits

    pub fn transform_vertices(&mut self, verts: &[u32], m: &DMat4) {
        let set: BTreeSet<u32> = verts.iter().copied().collect();
        for v in set {
            if let Some(p) = self.vertices.get_mut(v as usize) {
                *p = gt_core::snap_vec(m.transform_point3(*p));
            }
        }
        self.refresh_degenerate_uvs();
    }

    /// Vertices used by the faces.
    pub fn face_vertices(&self, faces: &[usize]) -> Vec<u32> {
        let set: BTreeSet<u32> = faces.iter().filter_map(|f| self.faces.get(*f)).flat_map(|f| f.indices.iter().copied()).collect();
        set.into_iter().collect()
    }

    /// Faces whose every vertex is in `verts`.
    pub fn faces_within(&self, verts: &BTreeSet<u32>) -> Vec<usize> {
        self.faces.iter().enumerate().filter(|(_, f)| f.indices.iter().all(|v| verts.contains(v))).map(|(i, _)| i).collect()
    }

    /// Edges whose both vertices are in `verts`.
    pub fn edges_within(&self, verts: &BTreeSet<u32>) -> Vec<(u32, u32)> {
        self.edges().into_iter().filter(|(a, b)| verts.contains(a) && verts.contains(b)).collect()
    }

    /// Region extrude. The faces are detached along their outer boundary and joined back with side quads.
    /// Returns the vertices of the extruded region, still in place and ready to be moved.
    pub fn extrude_faces(&mut self, faces: &[usize]) -> Vec<u32> {
        let region: BTreeSet<usize> = faces.iter().copied().filter(|f| *f < self.faces.len()).collect();
        if region.is_empty() {
            return Vec::new();
        }
        let mut directed: HashMap<(u32, u32), usize> = HashMap::new();
        for &fi in &region {
            for e in self.face_edges(fi) {
                directed.insert(e, fi);
            }
        }
        let mut boundary: Vec<(u32, u32)> = directed.keys().filter(|(a, b)| !directed.contains_key(&(*b, *a))).copied().collect();
        boundary.sort();
        let verts: BTreeSet<u32> = region.iter().flat_map(|fi| self.faces[*fi].indices.iter().copied()).collect();
        let mut map: BTreeMap<u32, u32> = BTreeMap::new();
        for v in verts {
            map.insert(v, self.vertices.len() as u32);
            self.vertices.push(self.vertices[v as usize]);
        }
        for (a, b) in boundary {
            let src = &self.faces[directed[&(a, b)]];
            let ka = src.indices.iter().position(|v| *v == a).unwrap_or(0);
            let kb = src.indices.iter().position(|v| *v == b).unwrap_or(0);
            let (ca, cb) = (CornerAttr::of(src, ka), CornerAttr::of(src, kb));
            let mut side = face_from(vec![a, b, map[&b], map[&a]], &[ca, cb, cb, ca], &src.data);
            side.uvs.clear();
            self.faces.push(side);
        }
        for &fi in &region {
            for v in &mut self.faces[fi].indices {
                *v = map[v];
            }
        }
        map.values().copied().collect()
    }

    /// Extrudes boundary edges into new quads. Returns (original, copy) vertex pairs.
    pub fn extrude_edges(&mut self, edges: &[(u32, u32)]) -> Vec<(u32, u32)> {
        let directed = self.directed_edges();
        let mut map: BTreeMap<u32, u32> = BTreeMap::new();
        let mut new_faces = Vec::new();
        for (a, b) in edges {
            // Keep manifold orientation: the new quad uses the edge opposite to its existing face.
            let (a, b, fi) = if let Some(fi) = directed.get(&(*a, *b)) {
                if directed.contains_key(&(*b, *a)) {
                    continue;
                }
                (*b, *a, *fi)
            } else if let Some(fi) = directed.get(&(*b, *a)) {
                (*a, *b, *fi)
            } else {
                continue;
            };
            for v in [a, b] {
                map.entry(v).or_insert_with(|| {
                    self.vertices.push(self.vertices[v as usize]);
                    (self.vertices.len() - 1) as u32
                });
            }
            let data = self.faces[fi].data.clone();
            new_faces.push(face_from(vec![a, b, map[&b], map[&a]], &[CornerAttr { uv: None, color: None }; 4], &data));
        }
        self.faces.extend(new_faces);
        map.into_iter().collect()
    }

    /// Insets each face individually by `thickness`. Returns the inner faces.
    pub fn inset_faces(&mut self, faces: &[usize], thickness: f64) -> Vec<usize> {
        let targets: BTreeSet<usize> = faces.iter().copied().filter(|f| *f < self.faces.len()).collect();
        let mut new_faces = Vec::new();
        for &fi in &targets {
            let pts = self.face_points(fi);
            let n = polygon::newell(&pts).normalize_or(DVec3::Y);
            let m = pts.len();
            let mut inner = Vec::with_capacity(m);
            for k in 0..m {
                let prev = pts[(k + m - 1) % m];
                let cur = pts[k];
                let next = pts[(k + 1) % m];
                let in1 = n.cross((cur - prev).normalize_or_zero());
                let in2 = n.cross((next - cur).normalize_or_zero());
                let bis = (in1 + in2).normalize_or(in1);
                let s = bis.dot(in1).max(0.25);
                inner.push(cur + bis * (thickness / s));
            }
            let face = self.faces[fi].clone();
            let centroid_attr = CornerAttr::average(&(0..m).map(|k| CornerAttr::of(&face, k)).collect::<Vec<_>>());
            let mut inner_idx = Vec::with_capacity(m);
            let mut inner_attr = Vec::with_capacity(m);
            for (k, p) in inner.into_iter().enumerate() {
                inner_idx.push(self.vertices.len() as u32);
                self.vertices.push(p);
                inner_attr.push(CornerAttr::lerp(CornerAttr::of(&face, k), centroid_attr, 0.1));
            }
            for k in 0..m {
                let j = (k + 1) % m;
                let attrs = [CornerAttr::of(&face, k), CornerAttr::of(&face, j), inner_attr[j], inner_attr[k]];
                new_faces.push(face_from(vec![face.indices[k], face.indices[j], inner_idx[j], inner_idx[k]], &attrs, &face.data));
            }
            self.faces[fi] = face_from(inner_idx, &inner_attr, &face.data);
        }
        self.faces.extend(new_faces);
        targets.into_iter().collect()
    }

    /// Edges crossed by a loop cut through `edge`, walking across quads both ways. Each rung is oriented consistently.
    pub fn edge_ring(&self, edge: (u32, u32)) -> Vec<(u32, u32)> {
        let efaces = self.edge_faces();
        if !efaces.contains_key(&edge_key(edge.0, edge.1)) {
            return Vec::new();
        }
        let mut ring = vec![edge];
        let mut visited: BTreeSet<usize> = BTreeSet::new();
        for pass in 0..2 {
            let mut cur = edge;
            loop {
                let candidates = efaces.get(&edge_key(cur.0, cur.1)).cloned().unwrap_or_default();
                let Some(fi) = candidates.into_iter().find(|f| !visited.contains(f) && self.faces[*f].indices.len() == 4) else { break };
                visited.insert(fi);
                let idx = &self.faces[fi].indices;
                let next = (0..4).find_map(|k| {
                    let (p, q) = (idx[k], idx[(k + 1) % 4]);
                    if (p, q) == cur {
                        Some((idx[(k + 3) % 4], idx[(k + 2) % 4]))
                    } else if (q, p) == cur {
                        Some((idx[(k + 2) % 4], idx[(k + 3) % 4]))
                    } else {
                        None
                    }
                });
                let Some(next) = next else { break };
                if edge_key(next.0, next.1) == edge_key(edge.0, edge.1) {
                    break;
                }
                if pass == 0 {
                    ring.push(next);
                } else {
                    ring.insert(0, next);
                }
                cur = next;
            }
        }
        ring
    }

    /// Splits the quad ring through `edge` with `cuts` parallel loops. Returns the new vertices.
    pub fn loop_cut(&mut self, edge: (u32, u32), cuts: usize) -> Vec<u32> {
        let ring = self.edge_ring(edge);
        if ring.is_empty() {
            return Vec::new();
        }
        let cuts = cuts.max(1);
        let mut rungs: HashMap<(u32, u32), (u32, Vec<u32>)> = HashMap::new();
        let mut created = Vec::new();
        for (a, b) in &ring {
            let key = edge_key(*a, *b);
            if rungs.contains_key(&key) {
                continue;
            }
            let (pa, pb) = (self.vertices[*a as usize], self.vertices[*b as usize]);
            let mut ids = Vec::with_capacity(cuts);
            for c in 1..=cuts {
                let t = c as f64 / (cuts + 1) as f64;
                ids.push(self.vertices.len() as u32);
                self.vertices.push(pa.lerp(pb, t));
            }
            created.extend(ids.iter().copied());
            rungs.insert(key, (*a, ids));
        }
        // New points along a rung, ordered from `from` to the other end.
        let along = |from: u32, to: u32| -> Option<Vec<u32>> {
            let (start, ids) = rungs.get(&edge_key(from, to))?;
            let mut out = ids.clone();
            if *start != from {
                out.reverse();
            }
            Some(out)
        };
        let t_of = |s: usize| s as f64 / (cuts + 1) as f64;

        let mut result: Vec<MeshFace> = Vec::with_capacity(self.faces.len() + ring.len() * cuts);
        for face in &self.faces {
            let idx = &face.indices;
            let split =
                (idx.len() == 4).then(|| (0..2).find(|k| along(idx[*k], idx[k + 1]).is_some() && along(idx[(k + 3) % 4], idx[k + 2]).is_some())).flatten();
            if let Some(k) = split {
                let (a, b, c, d) = (idx[k], idx[(k + 1) % 4], idx[(k + 2) % 4], idx[(k + 3) % 4]);
                let (ka, kb, kc, kd) = (k, (k + 1) % 4, (k + 2) % 4, (k + 3) % 4);
                let top = [vec![a], along(a, b).unwrap(), vec![b]].concat();
                let bottom = [vec![d], along(d, c).unwrap(), vec![c]].concat();
                for s in 0..=cuts {
                    let (t0, t1) = (t_of(s), t_of(s + 1));
                    let attrs = [
                        CornerAttr::lerp(CornerAttr::of(face, ka), CornerAttr::of(face, kb), t0),
                        CornerAttr::lerp(CornerAttr::of(face, ka), CornerAttr::of(face, kb), t1),
                        CornerAttr::lerp(CornerAttr::of(face, kd), CornerAttr::of(face, kc), t1),
                        CornerAttr::lerp(CornerAttr::of(face, kd), CornerAttr::of(face, kc), t0),
                    ];
                    result.push(face_from(vec![top[s], top[s + 1], bottom[s + 1], bottom[s]], &attrs, &face.data));
                }
                continue;
            }
            // Faces that only touch the ring get the new points inserted to stay connected.
            let mut indices = Vec::new();
            let mut attrs = Vec::new();
            for k in 0..idx.len() {
                let j = (k + 1) % idx.len();
                indices.push(idx[k]);
                attrs.push(CornerAttr::of(face, k));
                if let Some(pts) = along(idx[k], idx[j]) {
                    for (s, p) in pts.into_iter().enumerate() {
                        indices.push(p);
                        attrs.push(CornerAttr::lerp(CornerAttr::of(face, k), CornerAttr::of(face, j), t_of(s + 1)));
                    }
                }
            }
            result.push(face_from(indices, &attrs, &face.data));
        }
        self.faces = result;
        created
    }

    /// Edge loop through `edge`, following the straight continuation at vertices with four edges.
    pub fn edge_loop(&self, edge: (u32, u32)) -> Vec<(u32, u32)> {
        let efaces = self.edge_faces();
        let mut neighbours: HashMap<u32, BTreeSet<u32>> = HashMap::new();
        for (a, b) in efaces.keys() {
            neighbours.entry(*a).or_default().insert(*b);
            neighbours.entry(*b).or_default().insert(*a);
        }
        let mut out = vec![edge];
        let mut seen: BTreeSet<(u32, u32)> = BTreeSet::from([edge_key(edge.0, edge.1)]);
        for (from, to) in [(edge.0, edge.1), (edge.1, edge.0)] {
            let (mut prev, mut cur) = (from, to);
            while neighbours.get(&cur).is_some_and(|n| n.len() == 4) {
                let prev_faces: BTreeSet<usize> = efaces.get(&edge_key(prev, cur)).map(|f| f.iter().copied().collect()).unwrap_or_default();
                let next = neighbours[&cur]
                    .iter()
                    .copied()
                    .find(|c| *c != prev && efaces.get(&edge_key(cur, *c)).is_some_and(|fs| fs.iter().all(|f| !prev_faces.contains(f))));
                let Some(next) = next else { break };
                if !seen.insert(edge_key(cur, next)) {
                    break;
                }
                out.push((cur, next));
                prev = cur;
                cur = next;
            }
        }
        out
    }

    /// Cuts faces (all when `faces` is None) with a plane, splitting every crossed polygon. Returns the vertices on the cut.
    pub fn bisect(&mut self, plane: &Plane, faces: Option<&[usize]>) -> Vec<u32> {
        const EPS: f64 = 1e-6;
        let targets: BTreeSet<usize> = match faces {
            Some(f) => f.iter().copied().filter(|i| *i < self.faces.len()).collect(),
            None => (0..self.faces.len()).collect(),
        };
        let mut dist: Vec<f64> = self.vertices.iter().map(|v| plane.distance(*v)).collect();
        for d in &mut dist {
            if d.abs() < EPS {
                *d = 0.0;
            }
        }
        let mut edge_points: HashMap<(u32, u32), (u32, f64, u32)> = HashMap::new();
        for &fi in &targets {
            for (a, b) in self.face_edges(fi).collect::<Vec<_>>() {
                let (da, db) = (dist[a as usize], dist[b as usize]);
                if da * db < 0.0 && !edge_points.contains_key(&edge_key(a, b)) {
                    let t = da / (da - db);
                    let p = self.vertices[a as usize].lerp(self.vertices[b as usize], t);
                    edge_points.insert(edge_key(a, b), (a, t, self.vertices.len() as u32));
                    self.vertices.push(p);
                    dist.push(0.0);
                }
            }
        }
        let mut on_cut: BTreeSet<u32> = BTreeSet::new();
        let mut result = Vec::with_capacity(self.faces.len());
        for (fi, face) in self.faces.iter().enumerate() {
            let mut indices = Vec::new();
            let mut attrs = Vec::new();
            let n = face.indices.len();
            for k in 0..n {
                let j = (k + 1) % n;
                let (a, b) = (face.indices[k], face.indices[j]);
                indices.push(a);
                attrs.push(CornerAttr::of(face, k));
                if let Some((start, t, p)) = edge_points.get(&edge_key(a, b)) {
                    let t = if *start == a { *t } else { 1.0 - *t };
                    indices.push(*p);
                    attrs.push(CornerAttr::lerp(CornerAttr::of(face, k), CornerAttr::of(face, j), t));
                }
            }
            if !targets.contains(&fi) {
                result.push(face_from(indices, &attrs, &face.data));
                continue;
            }
            let on: Vec<usize> = (0..indices.len()).filter(|k| dist[indices[*k] as usize] == 0.0).collect();
            on_cut.extend(on.iter().map(|k| indices[*k]));
            let has_front = indices.iter().any(|v| dist[*v as usize] > 0.0);
            let has_back = indices.iter().any(|v| dist[*v as usize] < 0.0);
            let m = indices.len();
            if has_front && has_back && on.len() == 2 && (on[1] - on[0]) % m != 1 && (on[0] + m - on[1]) % m != 1 {
                let (s0, s1) = (on[0], on[1]);
                let first: Vec<usize> = (s0..=s1).collect();
                let second: Vec<usize> = (s1..s0 + m + 1).map(|k| k % m).collect();
                for part in [first, second] {
                    let idx: Vec<u32> = part.iter().map(|k| indices[*k]).collect();
                    let at: Vec<CornerAttr> = part.iter().map(|k| attrs[*k]).collect();
                    result.push(face_from(idx, &at, &face.data));
                }
            } else {
                result.push(face_from(indices, &attrs, &face.data));
            }
        }
        self.faces = result;
        on_cut.into_iter().collect()
    }

    /// Removes faces lying entirely on the dropped side of the plane.
    pub fn delete_side(&mut self, plane: &Plane, keep_front: bool) {
        let sign = if keep_front { 1.0 } else { -1.0 };
        let verts = self.vertices.clone();
        self.faces.retain(|f| {
            let d: Vec<f64> = f.indices.iter().map(|v| plane.distance(verts[*v as usize]) * sign).collect();
            !(d.iter().all(|x| *x <= 1e-5) && d.iter().any(|x| *x < -1e-5))
        });
        self.cleanup();
    }

    /// Closes boundary loops made only of the given vertices with new faces. Returns the new faces.
    pub fn fill_boundary_loops(&mut self, verts: &BTreeSet<u32>, data: &FaceData) -> Vec<usize> {
        let mut next: BTreeMap<u32, u32> = BTreeMap::new();
        for (a, b) in self.boundary_edges() {
            if verts.contains(&a) && verts.contains(&b) {
                next.insert(b, a);
            }
        }
        let mut out = Vec::new();
        while let Some((&start, _)) = next.iter().next() {
            let mut lp = vec![start];
            let mut cur = next.remove(&start).unwrap();
            while cur != start {
                lp.push(cur);
                match next.remove(&cur) {
                    Some(n) => cur = n,
                    None => break,
                }
            }
            if cur == start && lp.len() >= 3 {
                let pts: Vec<DVec3> = lp.iter().map(|v| self.vertices[*v as usize]).collect();
                let mut d = data.clone();
                d.uv = FaceUv::paraxial(polygon::newell(&pts).normalize_or(DVec3::Y), data.uv.scale);
                self.faces.push(MeshFace::new(lp, d));
                out.push(self.faces.len() - 1);
            }
        }
        out
    }

    /// Splits each face into quads around its center, inserting edge midpoints into neighbours.
    pub fn subdivide_faces(&mut self, faces: &[usize]) -> Vec<usize> {
        let targets: BTreeSet<usize> = faces.iter().copied().filter(|f| *f < self.faces.len()).collect();
        let mut mids: HashMap<(u32, u32), u32> = HashMap::new();
        for &fi in &targets {
            for (a, b) in self.face_edges(fi).collect::<Vec<_>>() {
                mids.entry(edge_key(a, b)).or_insert_with(|| {
                    self.vertices.push((self.vertices[a as usize] + self.vertices[b as usize]) * 0.5);
                    (self.vertices.len() - 1) as u32
                });
            }
        }
        let mut result = Vec::with_capacity(self.faces.len() + targets.len() * 3);
        let mut created = Vec::new();
        for (fi, face) in self.faces.clone().iter().enumerate() {
            let n = face.indices.len();
            if targets.contains(&fi) {
                let center_attr = CornerAttr::average(&(0..n).map(|k| CornerAttr::of(face, k)).collect::<Vec<_>>());
                let center = self.vertices.len() as u32;
                self.vertices.push(self.face_center(fi));
                for k in 0..n {
                    let prev = (k + n - 1) % n;
                    let j = (k + 1) % n;
                    let (a, b, p) = (face.indices[k], face.indices[j], face.indices[prev]);
                    let attrs = [
                        CornerAttr::of(face, k),
                        CornerAttr::lerp(CornerAttr::of(face, k), CornerAttr::of(face, j), 0.5),
                        center_attr,
                        CornerAttr::lerp(CornerAttr::of(face, prev), CornerAttr::of(face, k), 0.5),
                    ];
                    created.push(result.len());
                    result.push(face_from(vec![a, mids[&edge_key(a, b)], center, mids[&edge_key(p, a)]], &attrs, &face.data));
                }
                continue;
            }
            let mut indices = Vec::new();
            let mut attrs = Vec::new();
            for k in 0..n {
                let j = (k + 1) % n;
                indices.push(face.indices[k]);
                attrs.push(CornerAttr::of(face, k));
                if let Some(m) = mids.get(&edge_key(face.indices[k], face.indices[j])) {
                    indices.push(*m);
                    attrs.push(CornerAttr::lerp(CornerAttr::of(face, k), CornerAttr::of(face, j), 0.5));
                }
            }
            result.push(face_from(indices, &attrs, &face.data));
        }
        self.faces = result;
        created
    }

    /// Collapses the vertices into one at `target`.
    pub fn merge_vertices(&mut self, verts: &[u32], target: DVec3) {
        let Some(&keep) = verts.first() else { return };
        let set: BTreeSet<u32> = verts.iter().copied().collect();
        self.vertices[keep as usize] = target;
        for f in &mut self.faces {
            for v in &mut f.indices {
                if set.contains(v) {
                    *v = keep;
                }
            }
        }
        self.cleanup();
    }

    /// Merges vertices of the set that lie within `dist` of each other. Returns how many vertices were removed.
    pub fn merge_by_distance(&mut self, verts: &[u32], dist: f64) -> usize {
        let before = self.vertices.len();
        let mut target: HashMap<u32, u32> = HashMap::new();
        let mut buckets: HashMap<(i64, i64, i64), Vec<u32>> = HashMap::new();
        let cell = dist.max(1e-9);
        let set: BTreeSet<u32> = verts.iter().copied().collect();
        for &v in &set {
            let p = self.vertices[v as usize];
            let key = ((p.x / cell).floor() as i64, (p.y / cell).floor() as i64, (p.z / cell).floor() as i64);
            let mut found = None;
            'search: for dx in -1..=1 {
                for dy in -1..=1 {
                    for dz in -1..=1 {
                        if let Some(list) = buckets.get(&(key.0 + dx, key.1 + dy, key.2 + dz)) {
                            for &o in list {
                                if (self.vertices[o as usize] - p).length() <= dist {
                                    found = Some(o);
                                    break 'search;
                                }
                            }
                        }
                    }
                }
            }
            match found {
                Some(o) => {
                    target.insert(v, o);
                }
                None => buckets.entry(key).or_default().push(v),
            }
        }
        if target.is_empty() {
            return 0;
        }
        for f in &mut self.faces {
            for v in &mut f.indices {
                if let Some(t) = target.get(v) {
                    *v = *t;
                }
            }
        }
        self.cleanup();
        before - self.vertices.len()
    }

    pub fn delete_faces(&mut self, faces: &[usize]) {
        let set: BTreeSet<usize> = faces.iter().copied().collect();
        let mut i = 0;
        self.faces.retain(|_| {
            let keep = !set.contains(&i);
            i += 1;
            keep
        });
        self.cleanup();
    }

    /// Deletes every face using one of the vertices.
    pub fn delete_vertices(&mut self, verts: &[u32]) {
        let set: BTreeSet<u32> = verts.iter().copied().collect();
        self.faces.retain(|f| !f.indices.iter().any(|v| set.contains(v)));
        self.cleanup();
    }

    /// Deletes every face using one of the edges.
    pub fn delete_edges(&mut self, edges: &[(u32, u32)]) {
        let set: BTreeSet<(u32, u32)> = edges.iter().map(|(a, b)| edge_key(*a, *b)).collect();
        self.faces.retain(|f| (0..f.indices.len()).all(|k| !set.contains(&edge_key(f.indices[k], f.indices[(k + 1) % f.indices.len()]))));
        self.cleanup();
    }

    /// Dissolves vertices shared by exactly two edges into the surrounding faces.
    pub fn dissolve_vertices(&mut self, verts: &[u32]) {
        let set: BTreeSet<u32> = verts.iter().copied().collect();
        for f in &mut self.faces {
            if f.indices.len() <= 3 {
                continue;
            }
            let keep: Vec<usize> = (0..f.indices.len()).filter(|k| !set.contains(&f.indices[*k])).collect();
            if keep.len() >= 3 && keep.len() != f.indices.len() {
                let uvs_ok = f.uvs.len() == f.indices.len();
                let colors_ok = f.data.colors.len() == f.indices.len();
                f.uvs = if uvs_ok { keep.iter().map(|k| f.uvs[*k]).collect() } else { Vec::new() };
                f.data.colors = if colors_ok { keep.iter().map(|k| f.data.colors[*k]).collect() } else { Vec::new() };
                f.indices = keep.iter().map(|k| f.indices[*k]).collect();
            }
        }
        self.cleanup();
    }

    /// Creates a face from the vertices (Blender F). Boundary loops are followed when possible, otherwise the
    /// vertices are ordered around their center. Returns the new face.
    pub fn fill(&mut self, verts: &[u32], data: &FaceData) -> Option<usize> {
        let set: BTreeSet<u32> = verts.iter().copied().filter(|v| (*v as usize) < self.vertices.len()).collect();
        if set.len() < 3 {
            return None;
        }
        if let Some(f) = self.fill_boundary_loops(&set, data).last() {
            return Some(*f);
        }
        let pts: Vec<DVec3> = set.iter().map(|v| self.vertices[*v as usize]).collect();
        let center = polygon::centroid(&pts);
        let vertex_faces = self.vertex_faces();
        let mut normal: DVec3 = set.iter().flat_map(|v| vertex_faces[*v as usize].iter()).map(|f| self.face_normal(*f)).sum();
        if normal.length_squared() < 1e-12 {
            let given: Vec<DVec3> = verts.iter().map(|v| self.vertices[*v as usize]).collect();
            normal = polygon::newell(&given).normalize_or(DVec3::Y);
        }
        let plane = Plane::from_point_normal(center, normal);
        let (u, v) = plane.basis();
        let mut order: Vec<(f64, u32)> = set
            .iter()
            .map(|i| {
                let d = self.vertices[*i as usize] - center;
                (d.dot(v).atan2(d.dot(u)), *i)
            })
            .collect();
        order.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut indices: Vec<u32> = order.into_iter().map(|(_, i)| i).collect();
        let directed = self.directed_edges();
        let clashes = (0..indices.len()).any(|k| directed.contains_key(&(indices[k], indices[(k + 1) % indices.len()])));
        if clashes {
            indices.reverse();
        }
        let pts: Vec<DVec3> = indices.iter().map(|i| self.vertices[*i as usize]).collect();
        let mut d = data.clone();
        d.uv = FaceUv::paraxial(polygon::newell(&pts).normalize_or(DVec3::Y), data.uv.scale);
        self.faces.push(MeshFace::new(indices, d));
        Some(self.faces.len() - 1)
    }

    pub fn flip_faces(&mut self, faces: &[usize]) {
        for fi in faces {
            if let Some(f) = self.faces.get_mut(*fi) {
                f.indices.reverse();
                f.uvs.reverse();
                f.data.colors.reverse();
            }
        }
    }

    /// Laplacian smoothing of the vertices towards their neighbours.
    pub fn smooth_vertices(&mut self, verts: &[u32], factor: f64, iterations: usize) {
        let mut neighbours: HashMap<u32, BTreeSet<u32>> = HashMap::new();
        for (a, b) in self.edges() {
            neighbours.entry(a).or_default().insert(b);
            neighbours.entry(b).or_default().insert(a);
        }
        let set: BTreeSet<u32> = verts.iter().copied().collect();
        for _ in 0..iterations {
            let snapshot = self.vertices.clone();
            for &v in &set {
                let Some(n) = neighbours.get(&v).filter(|n| !n.is_empty()) else { continue };
                let avg = n.iter().map(|o| snapshot[*o as usize]).sum::<DVec3>() / n.len() as f64;
                self.vertices[v as usize] = snapshot[v as usize].lerp(avg, factor);
            }
        }
    }

    /// Copies the faces with their own vertices. Returns the new faces.
    pub fn duplicate_faces(&mut self, faces: &[usize]) -> Vec<usize> {
        let targets: BTreeSet<usize> = faces.iter().copied().filter(|f| *f < self.faces.len()).collect();
        let mut map: BTreeMap<u32, u32> = BTreeMap::new();
        let mut out = Vec::new();
        for fi in targets {
            let mut face = self.faces[fi].clone();
            for v in &mut face.indices {
                *v = *map.entry(*v).or_insert_with(|| {
                    self.vertices.push(self.vertices[*v as usize]);
                    (self.vertices.len() - 1) as u32
                });
            }
            self.faces.push(face);
            out.push(self.faces.len() - 1);
        }
        out
    }

    /// Moves the faces into a new mesh.
    pub fn separate(&mut self, faces: &[usize]) -> Mesh {
        let set: BTreeSet<usize> = faces.iter().copied().collect();
        let mut part = Mesh { smooth_angle: self.smooth_angle, ..Default::default() };
        let mut map: BTreeMap<u32, u32> = BTreeMap::new();
        for fi in &set {
            let Some(src) = self.faces.get(*fi) else { continue };
            let mut face = src.clone();
            for v in &mut face.indices {
                *v = *map.entry(*v).or_insert_with(|| {
                    part.vertices.push(self.vertices[*v as usize]);
                    (part.vertices.len() - 1) as u32
                });
            }
            part.faces.push(face);
        }
        self.delete_faces(&set.into_iter().collect::<Vec<_>>());
        part
    }

    pub fn join(&mut self, other: &Mesh) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&other.vertices);
        for f in &other.faces {
            let mut face = f.clone();
            for v in &mut face.indices {
                *v += base;
            }
            self.faces.push(face);
        }
    }

    /// Faces connected to the seeds through shared vertices.
    pub fn linked_faces(&self, seeds: &[usize]) -> Vec<usize> {
        let vertex_faces = self.vertex_faces();
        let mut seen: BTreeSet<usize> = BTreeSet::new();
        let mut stack: Vec<usize> = seeds.iter().copied().filter(|f| *f < self.faces.len()).collect();
        while let Some(fi) = stack.pop() {
            if !seen.insert(fi) {
                continue;
            }
            for v in &self.faces[fi].indices {
                stack.extend(vertex_faces[*v as usize].iter().filter(|f| !seen.contains(f)));
            }
        }
        seen.into_iter().collect()
    }

    /// Gives the surface thickness: an inner shell offset along vertex normals plus rims on open edges.
    pub fn solidify(&mut self, thickness: f64) {
        let n_verts = self.vertices.len();
        let mut normals = vec![DVec3::ZERO; n_verts];
        for fi in 0..self.faces.len() {
            let n = polygon::newell(&self.face_points(fi));
            for v in &self.faces[fi].indices {
                normals[*v as usize] += n;
            }
        }
        let boundary = self.boundary_edges();
        let base = n_verts as u32;
        for (i, n) in normals.iter().enumerate() {
            self.vertices.push(self.vertices[i] - n.normalize_or_zero() * thickness);
        }
        let shell: Vec<MeshFace> = self
            .faces
            .iter()
            .map(|f| {
                let mut g = f.clone();
                g.indices = f.indices.iter().rev().map(|v| v + base).collect();
                g.uvs.reverse();
                g.data.colors.reverse();
                g
            })
            .collect();
        let rim_data = self.faces.first().map(|f| f.data.clone()).unwrap_or_default();
        self.faces.extend(shell);
        for (a, b) in boundary {
            self.faces.push(face_from(vec![b, a, a + base, b + base], &[CornerAttr { uv: None, color: None }; 4], &rim_data));
        }
        self.refresh_degenerate_uvs();
    }

    /// Chamfers vertices: each vertex is replaced by points along its edges, joined by a new face.
    pub fn bevel_vertices(&mut self, verts: &[u32], width: f64) -> Vec<usize> {
        let mut created = Vec::new();
        for &v in verts.iter().collect::<BTreeSet<_>>() {
            if v as usize >= self.vertices.len() {
                continue;
            }
            let faces_using: Vec<usize> = (0..self.faces.len()).filter(|f| self.faces[*f].indices.contains(&v)).collect();
            if faces_using.len() < 2 {
                continue;
            }
            let center = self.vertices[v as usize];
            let mut edge_point: BTreeMap<u32, u32> = BTreeMap::new();
            let mut cap_next: BTreeMap<u32, u32> = BTreeMap::new();
            for &fi in &faces_using {
                let face = self.faces[fi].clone();
                let n = face.indices.len();
                let k = face.indices.iter().position(|x| *x == v).unwrap();
                let (prev, next) = (face.indices[(k + n - 1) % n], face.indices[(k + 1) % n]);
                let mut point = |other: u32, mesh: &mut Mesh| -> u32 {
                    *edge_point.entry(other).or_insert_with(|| {
                        let dir = mesh.vertices[other as usize] - center;
                        let t = (width / dir.length().max(1e-9)).min(0.49);
                        mesh.vertices.push(center + dir * t);
                        (mesh.vertices.len() - 1) as u32
                    })
                };
                let p_prev = point(prev, self);
                let p_next = point(next, self);
                let mut indices = face.indices.clone();
                indices.splice(k..=k, [p_prev, p_next]);
                let mut f = face.clone();
                f.indices = indices;
                f.uvs.clear();
                f.data.colors.clear();
                self.faces[fi] = f;
                // The cap walks the new points in the opposite direction of the faces around it.
                cap_next.insert(p_next, p_prev);
            }
            let Some((&start, _)) = cap_next.iter().next() else { continue };
            let mut lp = vec![start];
            let mut cur = cap_next[&start];
            while cur != start && lp.len() <= cap_next.len() {
                lp.push(cur);
                match cap_next.get(&cur) {
                    Some(n) => cur = *n,
                    None => break,
                }
            }
            if cur == start && lp.len() >= 3 {
                let data = self.faces[faces_using[0]].data.clone();
                let pts: Vec<DVec3> = lp.iter().map(|i| self.vertices[*i as usize]).collect();
                let mut d = data;
                d.uv = FaceUv::paraxial(polygon::newell(&pts).normalize_or(DVec3::Y), d.uv.scale);
                self.faces.push(MeshFace::new(lp, d));
                created.push(self.faces.len() - 1);
            }
        }
        let before = self.faces.len();
        self.cleanup();
        created.retain(|f| *f < before.min(self.faces.len()));
        created
    }

    /// Chamfers edges between two faces by splitting their vertices along the adjacent edges.
    /// Works on vertices where every other edge belongs to one of the two faces or leads away from the bevel.
    pub fn bevel_edges(&mut self, edges: &[(u32, u32)], width: f64) -> Vec<usize> {
        let mut created = Vec::new();
        for &(a, b) in edges {
            let efaces = self.edge_faces();
            let Some(fs) = efaces.get(&edge_key(a, b)).filter(|f| f.len() == 2).cloned() else { continue };
            // f1 uses a->b, f2 uses b->a.
            let uses = |fi: usize, x: u32, y: u32| {
                let idx = &self.faces[fi].indices;
                (0..idx.len()).any(|k| idx[k] == x && idx[(k + 1) % idx.len()] == y)
            };
            let (f1, f2) = if uses(fs[0], a, b) { (fs[0], fs[1]) } else { (fs[1], fs[0]) };
            if !uses(f1, a, b) || !uses(f2, b, a) {
                continue;
            }
            // Offset point of `v` sliding along the edge of face `fi` that is not the beveled edge.
            let slide = |mesh: &Mesh, fi: usize, v: u32, other: u32| -> Option<(u32, DVec3)> {
                let idx = &mesh.faces[fi].indices;
                let n = idx.len();
                let k = idx.iter().position(|x| *x == v)?;
                let (prev, next) = (idx[(k + n - 1) % n], idx[(k + 1) % n]);
                let along = if prev == other { next } else { prev };
                let dir = mesh.vertices[along as usize] - mesh.vertices[v as usize];
                let edge_dir = (mesh.vertices[other as usize] - mesh.vertices[v as usize]).normalize_or_zero();
                let sin = dir.normalize_or_zero().cross(edge_dir).length().max(0.2);
                let t = (width / sin / dir.length().max(1e-9)).min(0.49);
                Some((along, mesh.vertices[v as usize] + dir * t))
            };
            let (Some((a1_along, a1)), Some((b1_along, b1)), Some((a2_along, a2)), Some((b2_along, b2))) =
                (slide(self, f1, a, b), slide(self, f1, b, a), slide(self, f2, a, b), slide(self, f2, b, a))
            else {
                continue;
            };
            let base = self.vertices.len() as u32;
            self.vertices.extend([a1, b1, a2, b2]);
            let (ia1, ib1, ia2, ib2) = (base, base + 1, base + 2, base + 3);
            let replace = |mesh: &mut Mesh, fi: usize, v: u32, with: u32| {
                for x in &mut mesh.faces[fi].indices {
                    if *x == v {
                        *x = with;
                    }
                }
                mesh.faces[fi].uvs.clear();
                mesh.faces[fi].data.colors.clear();
            };
            replace(self, f1, a, ia1);
            replace(self, f1, b, ib1);
            replace(self, f2, a, ia2);
            replace(self, f2, b, ib2);
            // Neighbouring faces along the slid edges get the new point inserted next to the old vertex.
            let fix = |mesh: &mut Mesh, v: u32, along: u32, new: u32, skip: usize| {
                for fi in 0..mesh.faces.len() {
                    if fi == skip {
                        continue;
                    }
                    let idx = mesh.faces[fi].indices.clone();
                    let n = idx.len();
                    for k in 0..n {
                        let j = (k + 1) % n;
                        if (idx[k], idx[j]) == (along, v) || (idx[k], idx[j]) == (v, along) {
                            let mut ni = idx.clone();
                            ni.insert(j, new);
                            mesh.faces[fi].indices = ni;
                            mesh.faces[fi].uvs.clear();
                            mesh.faces[fi].data.colors.clear();
                            break;
                        }
                    }
                }
            };
            fix(self, a, a1_along, ia1, f1);
            fix(self, b, b1_along, ib1, f1);
            fix(self, a, a2_along, ia2, f2);
            fix(self, b, b2_along, ib2, f2);
            let data = self.faces[f1].data.clone();
            let strip = [ia2, ib2, ib1, ia1];
            let pts: Vec<DVec3> = strip.iter().map(|i| self.vertices[*i as usize]).collect();
            let mut d = data;
            d.uv = FaceUv::paraxial(polygon::newell(&pts).normalize_or(DVec3::Y), d.uv.scale);
            self.faces.push(MeshFace::new(vec![ia1, ib1, ib2, ia2], d));
            // Orientation: f1 now runs a1->b1, so the strip must use b1->a1.
            let last = self.faces.len() - 1;
            self.faces[last].indices = vec![ib1, ia1, ia2, ib2];
            created.push(last);
            // Remove the old vertices from faces that still reference both inserted points (end caps).
            for fi in 0..self.faces.len() {
                let f = &self.faces[fi];
                for (v, p, q) in [(a, ia1, ia2), (b, ib1, ib2)] {
                    if f.indices.contains(&v) && f.indices.contains(&p) && f.indices.contains(&q) {
                        let idx: Vec<u32> = self.faces[fi].indices.iter().copied().filter(|x| *x != v).collect();
                        self.faces[fi].indices = idx;
                        break;
                    }
                }
            }
        }
        self.cleanup();
        created
    }

    /// Splits every face into triangles.
    pub fn triangulate_faces(&mut self, faces: &[usize]) {
        let set: BTreeSet<usize> = faces.iter().copied().collect();
        let mut result = Vec::with_capacity(self.faces.len());
        for (fi, face) in self.faces.iter().enumerate() {
            if !set.contains(&fi) || face.indices.len() <= 3 {
                result.push(face.clone());
                continue;
            }
            for [a, b, c] in self.triangulate_corners(fi) {
                let attrs = [CornerAttr::of(face, a), CornerAttr::of(face, b), CornerAttr::of(face, c)];
                result.push(face_from(vec![face.indices[a], face.indices[b], face.indices[c]], &attrs, &face.data));
            }
        }
        self.faces = result;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cube() -> Mesh {
        Mesh::from_brush(&Brush::from_aabb(&Aabb::new(DVec3::splat(-16.0), DVec3::splat(16.0)), "m").unwrap())
    }

    fn top_face(m: &Mesh) -> usize {
        (0..m.faces.len()).max_by(|a, b| m.face_normal(*a).y.total_cmp(&m.face_normal(*b).y)).unwrap()
    }

    #[test]
    fn triangulates_concave_polygon() {
        let l = [
            DVec3::new(0.0, 0.0, 0.0),
            DVec3::new(2.0, 0.0, 0.0),
            DVec3::new(2.0, 0.0, -1.0),
            DVec3::new(1.0, 0.0, -1.0),
            DVec3::new(1.0, 0.0, -2.0),
            DVec3::new(0.0, 0.0, -2.0),
        ];
        let n = polygon::newell(&l);
        let tris = polygon::triangulate(&l, n);
        assert_eq!(tris.len(), 4);
        let area: f64 = tris.iter().map(|[a, b, c]| (l[*b] - l[*a]).cross(l[*c] - l[*a]).dot(n.normalize()) * 0.5).sum();
        assert!((area - 3.0).abs() < 1e-9, "area {area}");
    }

    #[test]
    fn extrude_keeps_mesh_closed() {
        let mut m = cube();
        let top = top_face(&m);
        let verts = m.extrude_faces(&[top]);
        assert_eq!(verts.len(), 4);
        m.transform_vertices(&verts, &DMat4::from_translation(DVec3::new(0.0, 32.0, 0.0)));
        assert!(m.is_closed());
        assert_eq!(m.faces.len(), 10);
        assert!((m.volume() - 32.0 * 32.0 * 64.0).abs() < 1e-6, "volume {}", m.volume());
    }

    #[test]
    fn loop_cut_splits_ring() {
        let mut m = cube();
        let edge = m.edges()[0];
        let new = m.loop_cut(edge, 2);
        assert_eq!(new.len(), 8);
        assert_eq!(m.faces.len(), 6 + 4 * 2);
        assert!(m.is_closed());
        assert!((m.volume() - 32f64.powi(3)).abs() < 1e-6);
    }

    #[test]
    fn bisect_and_delete_side_then_fill() {
        let mut m = cube();
        let plane = Plane::new(DVec3::X, 4.0);
        let on = m.bisect(&plane, None);
        assert_eq!(on.len(), 4);
        assert!(m.is_closed());
        assert_eq!(m.faces.len(), 10);
        m.delete_side(&plane, false);
        assert!(!m.is_closed());
        let set: BTreeSet<u32> = m.vertices.iter().enumerate().filter(|(_, v)| (v.x - 4.0).abs() < 1e-6).map(|(i, _)| i as u32).collect();
        assert_eq!(m.fill_boundary_loops(&set, &FaceData::default()).len(), 1);
        assert!(m.is_closed());
        assert!((m.volume() - 20.0 * 32.0 * 32.0).abs() < 1e-6, "volume {}", m.volume());
    }

    #[test]
    fn inset_and_subdivide() {
        let mut m = cube();
        let top = top_face(&m);
        m.inset_faces(&[top], 4.0);
        assert_eq!(m.faces.len(), 10);
        assert!(m.is_closed());
        let pts = m.face_points(top);
        assert!(pts.iter().all(|p| p.x.abs() <= 12.0 + 1e-9 && p.z.abs() <= 12.0 + 1e-9));
        let mut c = cube();
        c.subdivide_faces(&(0..6).collect::<Vec<_>>());
        assert_eq!(c.faces.len(), 24);
        assert!(c.is_closed());
    }

    #[test]
    fn merge_fill_flip_and_convexity() {
        let mut m = cube();
        assert!(m.is_convex());
        m.to_brush().unwrap().validate().unwrap();
        let top = top_face(&m);
        let verts = m.faces[top].indices.clone();
        m.delete_faces(&[top]);
        assert!(!m.is_closed());
        let f = m.fill(&verts, &FaceData::default());
        assert!(f.is_some());
        assert!(m.is_closed());
        assert!(m.volume() > 0.0);
        let top = top_face(&m);
        let vs = m.faces[top].indices.clone();
        m.merge_vertices(&vs, DVec3::new(0.0, 40.0, 0.0));
        assert_eq!(m.vertices.len(), 5);
        assert!(m.is_closed());
    }

    #[test]
    fn bevel_edge_on_cube() {
        let mut m = cube();
        let top = top_face(&m);
        let idx = m.faces[top].indices.clone();
        let created = m.bevel_edges(&[(idx[0], idx[1])], 4.0);
        assert_eq!(created.len(), 1);
        assert_eq!(m.faces.len(), 7);
        assert!(m.is_closed(), "boundary {:?}", m.boundary_edges());
        assert!(m.volume() < 32f64.powi(3) && m.volume() > 32f64.powi(3) - 32.0 * 16.0);
    }

    #[test]
    fn bevel_vertex_on_cube() {
        let mut m = cube();
        m.bevel_vertices(&[0], 4.0);
        assert_eq!(m.faces.len(), 7);
        assert!(m.is_closed());
    }

    #[test]
    fn solidify_plane() {
        let mut m = Mesh::from_polygons([(
            vec![DVec3::ZERO, DVec3::new(0.0, 0.0, 32.0), DVec3::new(32.0, 0.0, 32.0), DVec3::new(32.0, 0.0, 0.0)],
            FaceData::default(),
        )]);
        m.solidify(8.0);
        assert!(m.is_closed());
        assert!((m.volume().abs() - 32.0 * 32.0 * 8.0).abs() < 1e-6);
    }

    #[test]
    fn edge_loop_on_subdivided_face() {
        let mut m = cube();
        m.subdivide_faces(&(0..6).collect::<Vec<_>>());
        let e = m.edges()[0];
        let lp = m.edge_loop(e);
        assert!(lp.len() >= 2);
    }

    #[test]
    fn smooth_normals_average_across_shallow_angles() {
        let mut m = cube();
        m.smooth_angle = 95.0;
        let normals = m.corner_normals();
        assert!(normals.iter().flatten().all(|n| (n.abs().x - n.abs().y).abs() < 1e-9));
        m.smooth_angle = 0.0;
        assert!(m.corner_normals().iter().flatten().all(|n| n.abs().max_element() > 0.999));
    }

    #[test]
    fn serde_round_trip() {
        let mut m = cube();
        m.faces[0].uvs = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let json = serde_json::to_string(&m).unwrap();
        let back: Mesh = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }
}
