//! Hides the parts of brush and mesh faces that sit on the same plane as another solid's face. Back to back faces
//! between closed solids are inside the shape and never visible, and overlapping faces facing the same way z-fight,
//! so only the one with priority draws the shared area.

use std::collections::{BTreeSet, HashMap};

use gt_core::{Aabb, DVec3, NodeId};
use gt_doc::{Map, NodeKind};
use gt_formats::GameConfig;
use gt_geom::polygon;

/// Pieces to draw instead of a face, empty when the whole face is hidden. Faces without an entry draw unchanged.
pub type FacePieces = HashMap<usize, Pieces>;
pub type Pieces = Vec<Vec<DVec3>>;

pub(crate) const COPLANAR_DIST: f64 = 0.02;
pub(crate) const SAME_NORMAL: f64 = 0.9995;

pub(crate) type PlaneKey = (i64, i64, i64, i64);

#[derive(Clone)]
pub(crate) struct CullFace {
    pub face: usize,
    pub key: PlaneKey,
    pub normal: DVec3,
    pub dist: f64,
    pub bounds: Aabb,
    pub polygon: Vec<DVec3>,
    pub closed: bool,
    pub is_mesh: bool,
}

#[derive(Default)]
pub struct FaceCull {
    faces: HashMap<NodeId, Vec<CullFace>>,
    planes: HashMap<PlaneKey, BTreeSet<(NodeId, usize)>>,
    /// Bounds of every closed solid, for the broad phase of the interior pass.
    solids: HashMap<NodeId, Aabb>,
    pub pieces: HashMap<NodeId, FacePieces>,
}

/// Node classes the Godot build leaves out of culling, their brushes move or do not draw.
const MOVING_CLASSES: [&str; 5] = ["Area3D", "AnimatableBody3D", "RigidBody3D", "CharacterBody3D", "VehicleBody3D"];

/// Brushes of triggers and of moving or volume entities take no part in culling, like godottrench_face_cull.gd.
pub(crate) fn skipped_entity(map: &Map, game: &GameConfig, id: NodeId) -> bool {
    map.owning_entity(id)
        .and_then(|e| map.entity(e))
        .is_some_and(|e| e.classname.starts_with("trigger") || game.entity(&e.classname).is_some_and(|d| MOVING_CLASSES.contains(&d.node_class.as_str())))
}

/// Bounds of a node that can swallow another solid's faces: a closed, visible solid drawn opaque on every face.
/// A tool textured or see-through box (clip, water) shows what is inside it, so it buries nothing.
fn solid_bounds(map: &Map, game: &GameConfig, opaque: &(dyn Fn(&str) -> bool + Sync), id: NodeId) -> Option<Aabb> {
    let node = map.get(id)?;
    if map.is_hidden(id) || !map.in_cordon(id) || skipped_entity(map, game, id) {
        return None;
    }

    let solid = |material: &str| !game.is_tool_texture(material) && opaque(material);
    match &node.kind {
        NodeKind::Brush(b) if b.faces.iter().all(|f| f.data.disp.is_none() && solid(&f.data.material)) => Some(b.bounds()),
        NodeKind::Mesh(m) if m.edge_faces().values().all(|f| f.len() == 2) && m.faces.iter().all(|f| solid(&f.data.material)) => Some(m.bounds()),
        _ => None,
    }
}

/// True when every corner of `face` sits well inside the solid `id`. Each sample is probed a little to both
/// sides of the face: a face resting on the other solid's surface leaves it on one side and is left to the
/// coplanar pass, only a face with solid material on both sides counts as buried. Brushes are convex, so their
/// corners decide it. A closed mesh can be concave, there the face must also not cross the mesh surface.
pub(crate) fn buried_in(map: &Map, id: NodeId, face: &CullFace, mesh_tris: &mut HashMap<NodeId, Vec<[DVec3; 3]>>) -> bool {
    let Some(node) = map.get(id) else { return false };
    let nudge = face.normal * COPLANAR_DIST;
    let mut samples: Vec<DVec3> = Vec::with_capacity((face.polygon.len() + 1) * 2);
    for p in face.polygon.iter().chain(std::iter::once(&polygon::centroid(&face.polygon))) {
        samples.push(*p + nudge);
        samples.push(*p - nudge);
    }

    match &node.kind {
        NodeKind::Brush(b) => samples.iter().all(|p| b.contains_point(*p)),
        NodeKind::Mesh(m) => {
            if !samples.iter().all(|p| m.contains_point(*p)) {
                return false;
            }

            let tris = mesh_tris.entry(id).or_insert_with(|| {
                (0..m.faces.len())
                    .flat_map(|fi| m.triangulate_face(fi))
                    .filter_map(|t| {
                        let [a, b, c] = t.map(|i| m.vertices.get(i as usize).copied());
                        Some([a?, b?, c?])
                    })
                    .collect()
            });
            !crosses_surface(face, tris)
        }
        _ => false,
    }
}

/// Whether the surface triangles cut through the face polygon anywhere, meaning part of it lies outside.
fn crosses_surface(face: &CullFace, tris: &[[DVec3; 3]]) -> bool {
    let poly = &face.polygon;
    let face_tris: Vec<[DVec3; 3]> = polygon::triangulate(poly, face.normal).into_iter().map(|[a, b, c]| [poly[a], poly[b], poly[c]]).collect();
    let reach = face.bounds.expanded(COPLANAR_DIST);
    tris.iter().any(|tri| {
        if !Aabb::from_points(tri.iter().copied()).intersects(&reach) {
            return false;
        }

        (0..poly.len()).any(|k| segment_hits_triangle(poly[k], poly[(k + 1) % poly.len()], tri))
            || (0..3).any(|k| face_tris.iter().any(|ft| segment_hits_triangle(tri[k], tri[(k + 1) % 3], ft)))
    })
}

/// Segment a-b passing through the inside of triangle `tri`. Touching an edge or ending on the plane does not count.
fn segment_hits_triangle(a: DVec3, b: DVec3, tri: &[DVec3; 3]) -> bool {
    const EPS: f64 = 1e-6;
    let dir = b - a;
    let (e1, e2) = (tri[1] - tri[0], tri[2] - tri[0]);
    let p = dir.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-12 {
        return false;
    }

    let inv = 1.0 / det;
    let s = a - tri[0];
    let u = s.dot(p) * inv;
    let q = s.cross(e1);
    let v = dir.dot(q) * inv;
    let t = e2.dot(q) * inv;
    u > EPS && v > EPS && u + v < 1.0 - EPS && t > EPS && t < 1.0 - EPS
}

/// Opposite normals share a key so back to back faces land in one group.
pub(crate) fn plane_key(normal: DVec3, dist: f64) -> PlaneKey {
    let flip = [normal.x, normal.y, normal.z].into_iter().find(|c| c.abs() > 1e-6).is_some_and(|c| c < 0.0);
    let (n, d) = if flip { (-normal, -dist) } else { (normal, dist) };
    ((n.x * 1000.0).round() as i64, (n.y * 1000.0).round() as i64, (n.z * 1000.0).round() as i64, (d * 8.0).round() as i64)
}

impl FaceCull {
    /// Faces of a node that take part: opaque, single sided, not tool textures, triggers or displacements.
    fn node_faces(map: &Map, game: &GameConfig, opaque: &(dyn Fn(&str) -> bool + Sync), id: NodeId) -> Vec<CullFace> {
        let Some(node) = map.get(id) else { return Vec::new() };
        if map.is_hidden(id) || !map.in_cordon(id) || skipped_entity(map, game, id) {
            return Vec::new();
        }

        let usable = |material: &str| !game.is_tool_texture(material) && opaque(material);
        let make = |face: usize, normal: DVec3, polygon: Vec<DVec3>, closed: bool, is_mesh: bool| -> Option<CullFace> {
            if polygon.len() < 3 || normal == DVec3::ZERO {
                return None;
            }

            let dist = normal.dot(polygon::centroid(&polygon));
            let bounds = Aabb::from_points(polygon.iter().copied());
            Some(CullFace { face, key: plane_key(normal, dist), normal, dist, bounds, polygon, closed, is_mesh })
        };
        match &node.kind {
            NodeKind::Brush(b) if b.faces.iter().all(|f| f.data.disp.is_none()) => (0..b.faces.len())
                .filter_map(|fi| {
                    let f = &b.faces[fi];
                    usable(&f.data.material)
                        .then(|| make(fi, f.plane.normal, f.indices.iter().map(|i| b.vertices[*i as usize]).collect(), true, false))
                        .flatten()
                })
                .collect(),
            NodeKind::Mesh(m) => {
                let closed = m.edge_faces().values().all(|f| f.len() == 2);
                let face_of = |fi: usize| -> Option<CullFace> {
                    let f = &m.faces[fi];
                    if f.indices.len() >= 3 && f.indices.iter().all(|i| (*i as usize) < m.vertices.len()) && usable(&f.data.material) {
                        make(fi, m.face_normal(fi), m.face_points(fi), closed, true)
                    } else {
                        None
                    }
                };
                // A dense mesh's per-face work is independent, so build the cull faces across threads.
                let n = m.faces.len();
                let threads = std::thread::available_parallelism().map(|t| t.get()).unwrap_or(1);
                if threads <= 1 || n < 2000 {
                    (0..n).filter_map(face_of).collect()
                } else {
                    let chunk = n.div_ceil(threads);
                    std::thread::scope(|s| {
                        (0..threads)
                            .map(|t| {
                                let range = (t * chunk).min(n)..((t + 1) * chunk).min(n);
                                s.spawn(|| range.filter_map(&face_of).collect::<Vec<_>>())
                            })
                            .collect::<Vec<_>>()
                            .into_iter()
                            .flat_map(|h| h.join().unwrap())
                            .collect()
                    })
                }
            }
            _ => Vec::new(),
        }
    }

    /// Drops the cached visible pieces of `ids` so they draw whole, returning those that had any.
    /// Used to defer culling of geometry that is being dragged: the culled result is invisible mid-drag
    /// and recomputing it every frame is the dominant cost on dense meshes.
    pub fn clear_pieces(&mut self, ids: &BTreeSet<NodeId>) -> BTreeSet<NodeId> {
        let mut changed = BTreeSet::new();
        for id in ids {
            if self.pieces.remove(id).is_some() {
                changed.insert(*id);
            }
        }

        changed
    }

    /// The changed nodes plus every closed solid sharing space with them. Moving a solid buries or uncovers the
    /// faces of its neighbours, so those are recomputed in the same pass and stay consistent.
    fn with_neighbours(&self, map: &Map, game: &GameConfig, opaque: &(dyn Fn(&str) -> bool + Sync), dirty: &BTreeSet<NodeId>) -> Vec<NodeId> {
        let mut regions: Vec<Aabb> = Vec::new();
        for id in dirty {
            regions.extend(self.solids.get(id).copied());
            regions.extend(solid_bounds(map, game, opaque, *id));
        }

        let mut nodes = dirty.clone();
        if !regions.is_empty() {
            for (id, b) in &self.solids {
                if !nodes.contains(id) && regions.iter().any(|r| r.intersects(b)) {
                    nodes.insert(*id);
                }
            }
        }

        nodes.into_iter().collect()
    }

    /// Hides faces buried inside another closed solid, so a pile of intersecting solids draws as one outer
    /// shell instead of every solid's whole surface. Whole faces only, like the coplanar pass.
    fn hide_buried_faces(&mut self, map: &Map, nodes: &[NodeId], changed: &mut BTreeSet<NodeId>) {
        let solids: Vec<(NodeId, Aabb)> = self.solids.iter().map(|(id, b)| (*id, *b)).collect();
        let mut mesh_tris: HashMap<NodeId, Vec<[DVec3; 3]>> = HashMap::new();
        for id in nodes {
            let Some(faces) = self.faces.get(id) else { continue };
            let reach = faces.iter().fold(Aabb::EMPTY, |mut b, f| {
                b.include(&f.bounds);
                b
            });
            // Only solids overlapping this node can contain any of its faces, and most nodes have none.
            let containers: Vec<NodeId> = solids.iter().filter(|(other, b)| other != id && b.intersects(&reach)).map(|(other, _)| *other).collect();
            if containers.is_empty() {
                continue;
            }

            let buried: Vec<usize> = faces
                .iter()
                .filter(|f| {
                    let already_hidden = self.pieces.get(id).and_then(|p| p.get(&f.face)).is_some_and(|p| p.is_empty());
                    !already_hidden
                        && containers
                            .iter()
                            .any(|other| self.solids.get(other).is_some_and(|b| b.contains(&f.bounds)) && buried_in(map, *other, f, &mut mesh_tris))
                })
                .map(|f| f.face)
                .collect();
            for fi in buried {
                self.pieces.entry(*id).or_default().insert(fi, Vec::new());
                changed.insert(*id);
            }
        }
    }

    /// Refreshes the faces of `dirty` nodes and everything sharing a plane with them. Returns the nodes whose visible pieces changed.
    pub fn update(&mut self, map: &Map, game: &GameConfig, opaque: &(dyn Fn(&str) -> bool + Sync), dirty: &BTreeSet<NodeId>, full: bool) -> BTreeSet<NodeId> {
        let nodes: Vec<NodeId> = if full {
            *self = Self::default();
            map.nodes.keys().copied().collect()
        } else {
            self.with_neighbours(map, game, opaque, dirty)
        };
        // Areas on each plane that changed, from the old and the new faces of the refreshed nodes.
        let mut touched: HashMap<PlaneKey, Vec<Aabb>> = HashMap::new();
        for id in &nodes {
            if let Some(old) = self.faces.remove(id) {
                for f in old {
                    if let Some(members) = self.planes.get_mut(&f.key) {
                        members.remove(&(*id, f.face));
                    }

                    touched.entry(f.key).or_default().push(f.bounds);
                }
            }

            let new = Self::node_faces(map, game, opaque, *id);
            for f in &new {
                self.planes.entry(f.key).or_default().insert((*id, f.face));
                touched.entry(f.key).or_default().push(f.bounds);
            }

            if !new.is_empty() {
                self.faces.insert(*id, new);
            }

            self.solids.remove(id);
            if let Some(b) = solid_bounds(map, game, opaque, *id) {
                self.solids.insert(*id, b);
            }
        }

        self.planes.retain(|_, members| !members.is_empty());

        let mut changed = BTreeSet::new();
        for id in &nodes {
            if self.pieces.remove(id).is_some() {
                changed.insert(*id);
            }
        }

        // Direct lookup from (node, face) to its CullFace. Without it, resolving a plane's members
        // scans the whole node face list per member, which is O(faces^2) on a dense model. Only the
        // nodes that share a touched plane are indexed, so ordinary small edits stay cheap.
        let member_ids: BTreeSet<NodeId> = touched.keys().filter_map(|k| self.planes.get(k)).flat_map(|m| m.iter().map(|(id, _)| *id)).collect();
        let mut index: HashMap<(NodeId, usize), &CullFace> = HashMap::new();
        for id in &member_ids {
            if let Some(faces) = self.faces.get(id) {
                for f in faces {
                    index.insert((*id, f.face), f);
                }
            }
        }

        // Each touched plane's visible pieces are computed independently from immutable data, so they fan
        // out across threads; only the final apply into `self.pieces` runs on the calling thread.
        let touched: Vec<(PlaneKey, Vec<Aabb>)> = touched.into_iter().collect();
        let planes = &self.planes;
        let index = &index;
        let per_key = |key: &PlaneKey, areas: &[Aabb]| -> Vec<(NodeId, usize, Option<Pieces>)> {
            let Some(members) = planes.get(key) else { return Vec::new() };
            let members: Vec<(NodeId, &CullFace)> = members.iter().filter_map(|(id, fi)| index.get(&(*id, *fi)).map(|f| (*id, *f))).collect();
            let near = |b: &Aabb| areas.iter().any(|a| a.expanded(COPLANAR_DIST).intersects(b));
            let mut results = Vec::new();
            for (id, face) in members.iter().filter(|(_, f)| near(&f.bounds)) {
                let mut overlapping: Vec<&[DVec3]> = Vec::new();
                let mut backing: Vec<&[DVec3]> = Vec::new();
                for (other_id, other) in &members {
                    match covers(other, *other_id, face, *id) {
                        Cover::Overlap => overlapping.push(&other.polygon),
                        Cover::Backing => backing.push(&other.polygon),
                        Cover::None => {}
                    }
                }

                // Back to back faces are only dropped when fully covered, cutting holes into a floor under every wall adds
                // triangles and T-junction cracks without hiding anything visible.
                let hidden = !backing.is_empty() && polygon::visible_pieces(&face.polygon, face.normal, &backing).is_some_and(|p| p.is_empty());
                let pieces = if hidden {
                    Some(Vec::new())
                } else if overlapping.is_empty() {
                    None
                } else {
                    polygon::visible_pieces(&face.polygon, face.normal, &overlapping)
                };
                results.push((*id, face.face, pieces));
            }

            results
        };
        let threads = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);
        let all_results: Vec<(NodeId, usize, Option<Pieces>)> = if threads <= 1 || touched.len() < 512 {
            touched.iter().flat_map(|(k, a)| per_key(k, a)).collect()
        } else {
            let chunk = touched.len().div_ceil(threads);
            std::thread::scope(|s| {
                touched
                    .chunks(chunk)
                    .map(|c| s.spawn(|| c.iter().flat_map(|(k, a)| per_key(k, a)).collect::<Vec<_>>()))
                    .collect::<Vec<_>>()
                    .into_iter()
                    .flat_map(|h| h.join().unwrap())
                    .collect()
            })
        };
        for (id, fi, pieces) in all_results {
            let entry = self.pieces.entry(id).or_default();
            let before = entry.get(&fi).cloned();
            match pieces {
                Some(p) => {
                    entry.insert(fi, p);
                }
                None => {
                    entry.remove(&fi);
                }
            }

            if entry.get(&fi) != before.as_ref() {
                changed.insert(id);
            }

            if entry.is_empty() {
                self.pieces.remove(&id);
            }
        }

        self.hide_buried_faces(map, &nodes, &mut changed);
        changed
    }
}

enum Cover {
    None,
    /// Faces the same way on the same plane and keeps the shared area, see `priority`.
    Overlap,
    /// Back to back with another closed solid.
    Backing,
}

/// Open meshes are sheets laid over a solid (blend layers, decals), so they draw over brushes, and brushes over closed
/// meshes. The Godot build (godottrench_face_cull.gd) uses the same order.
pub(crate) fn priority(face: &CullFace, id: NodeId) -> (u8, NodeId, usize) {
    let rank = match (face.is_mesh, face.closed) {
        (true, false) => 0,
        (false, _) => 1,
        (true, true) => 2,
    };
    (rank, id, face.face)
}

fn covers(other: &CullFace, other_id: NodeId, face: &CullFace, id: NodeId) -> Cover {
    if other_id == id || !other.bounds.expanded(COPLANAR_DIST).intersects(&face.bounds) {
        return Cover::None;
    }

    let dot = other.normal.dot(face.normal);
    if dot > SAME_NORMAL && (other.dist - face.dist).abs() < COPLANAR_DIST && priority(other, other_id) < priority(face, id) {
        Cover::Overlap
    } else if dot < -SAME_NORMAL && (other.dist + face.dist).abs() < COPLANAR_DIST && other.closed && face.closed {
        Cover::Backing
    } else {
        Cover::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_geom::Brush;

    fn add_box(map: &mut Map, min: DVec3, max: DVec3) -> NodeId {
        let layer = map.default_layer();
        map.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(min, max), "dev/grey").unwrap()))
    }

    fn run(map: &Map) -> FaceCull {
        let mut cull = FaceCull::default();
        cull.update(map, &GameConfig::default(), &|_| true, &BTreeSet::new(), true);
        cull
    }

    fn face_towards(map: &Map, id: NodeId, normal: DVec3) -> usize {
        map.brush(id).unwrap().faces.iter().position(|f| f.plane.normal.dot(normal) > 0.999).unwrap()
    }

    #[test]
    fn touching_boxes_hide_the_faces_between_them() {
        let mut map = Map::new();
        let a = add_box(&mut map, DVec3::ZERO, DVec3::splat(64.0));
        let b = add_box(&mut map, DVec3::new(64.0, 0.0, 0.0), DVec3::new(128.0, 64.0, 64.0));
        let cull = run(&map);
        assert_eq!(cull.pieces[&a][&face_towards(&map, a, DVec3::X)], Vec::<Vec<DVec3>>::new());
        assert_eq!(cull.pieces[&b][&face_towards(&map, b, DVec3::NEG_X)], Vec::<Vec<DVec3>>::new());
        assert_eq!(cull.pieces[&a].len(), 1, "the tops only share an edge and stay whole");
    }

    #[test]
    fn a_floor_under_a_wall_keeps_its_face() {
        let mut map = Map::new();
        let floor = add_box(&mut map, DVec3::new(0.0, -16.0, 0.0), DVec3::new(256.0, 0.0, 256.0));
        let wall = add_box(&mut map, DVec3::new(64.0, 0.0, 64.0), DVec3::new(80.0, 128.0, 192.0));
        let cull = run(&map);
        assert!(!cull.pieces.contains_key(&floor), "partly covered back to back faces are not cut up");
        assert_eq!(cull.pieces[&wall][&face_towards(&map, wall, DVec3::NEG_Y)], Vec::<Vec<DVec3>>::new(), "the wall bottom is fully covered");
    }

    #[test]
    fn overlapping_coplanar_faces_draw_the_shared_area_once() {
        let mut map = Map::new();
        let a = add_box(&mut map, DVec3::ZERO, DVec3::new(64.0, 64.0, 16.0));
        let b = add_box(&mut map, DVec3::new(48.0, 0.0, 0.0), DVec3::new(112.0, 64.0, 16.0));
        let cull = run(&map);
        let front = face_towards(&map, b, DVec3::Z);
        assert!(!cull.pieces.get(&a).is_some_and(|p| p.contains_key(&face_towards(&map, a, DVec3::Z))), "the older brush keeps its face");
        let pieces = &cull.pieces[&b][&front];
        let area: f64 = pieces.iter().map(|p| polygon::area(p)).sum();
        assert!((area - 48.0 * 64.0).abs() < 1e-6, "only the part past the first brush is drawn, got {area}");
    }

    #[test]
    fn an_open_sheet_draws_over_the_brush_below() {
        let mut map = Map::new();
        let floor = add_box(&mut map, DVec3::new(0.0, -24.0, 0.0), DVec3::new(128.0, -2.0, 128.0));
        let layer = map.default_layer();
        let sheet_bounds = Aabb::new(DVec3::new(0.0, -2.0, 0.0), DVec3::new(128.0, -2.0, 128.0));
        let sheet = map.insert(layer, NodeKind::Mesh(gt_geom::mesh_shapes::grid(&sheet_bounds, 4, 4, "dev/grey")));
        let cull = run(&map);
        assert_eq!(cull.pieces[&floor][&face_towards(&map, floor, DVec3::Y)], Vec::<Vec<DVec3>>::new(), "the floor top gives way to the sheet");
        assert!(!cull.pieces.contains_key(&sheet), "every sheet face draws");
    }

    #[test]
    fn a_solid_buried_in_another_loses_every_face() {
        let mut map = Map::new();
        let big = add_box(&mut map, DVec3::splat(-128.0), DVec3::splat(128.0));
        let inner = add_box(&mut map, DVec3::splat(-16.0), DVec3::splat(16.0));
        let cull = run(&map);
        let hidden = &cull.pieces[&inner];
        assert_eq!(hidden.len(), 6, "every face of the swallowed box is dropped");
        assert!(hidden.values().all(|p| p.is_empty()));
        assert!(!cull.pieces.contains_key(&big), "the outer shell keeps all of its faces");
    }

    #[test]
    fn half_overlapping_solids_keep_their_faces() {
        let mut map = Map::new();
        let a = add_box(&mut map, DVec3::ZERO, DVec3::splat(64.0));
        let b = add_box(&mut map, DVec3::splat(32.0), DVec3::splat(96.0));
        let cull = run(&map);
        // Each box pokes out of the other, so no whole face is buried and nothing is dropped.
        for id in [a, b] {
            assert!(!cull.pieces.get(&id).is_some_and(|p| p.values().any(|v| v.is_empty())), "{id} lost a face it still shows");
        }
    }

    #[test]
    fn a_closed_mesh_inside_a_brush_is_dropped_and_restored_when_it_moves_out() {
        let mut map = Map::new();
        let layer = map.default_layer();
        let hill = add_box(&mut map, DVec3::splat(-256.0), DVec3::splat(256.0));
        let rock_bounds = Aabb::new(DVec3::splat(-32.0), DVec3::splat(32.0));
        let rock = map.insert(layer, NodeKind::Mesh(gt_geom::mesh_shapes::sphere(&rock_bounds, 8, 6, "dev/grey")));
        let mut cull = run(&map);
        let faces = map.mesh(rock).unwrap().faces.len();
        assert_eq!(cull.pieces[&rock].len(), faces, "the whole buried rock is dropped");

        // Move it clear of the brush: every face comes back.
        let moved = gt_geom::mesh_shapes::sphere(&rock_bounds.translated(DVec3::new(4096.0, 0.0, 0.0)), 8, 6, "dev/grey");
        *map.mesh_mut(rock).unwrap() = moved;
        let changed = cull.update(&map, &GameConfig::default(), &|_| true, &BTreeSet::from([rock]), false);
        assert!(changed.contains(&rock));
        assert!(!cull.pieces.contains_key(&rock), "the rock draws again once it is outside");
        let _ = hill;
    }

    fn add_box_with(map: &mut Map, parent: NodeId, min: DVec3, max: DVec3, material: &str) -> NodeId {
        map.insert(parent, NodeKind::Brush(Brush::from_aabb(&Aabb::new(min, max), material).unwrap()))
    }

    #[test]
    fn tool_and_see_through_solids_bury_nothing() {
        let mut map = Map::new();
        let layer = map.default_layer();
        let clip = add_box_with(&mut map, layer, DVec3::splat(-128.0), DVec3::splat(128.0), "special/clip");
        let water = add_box_with(&mut map, layer, DVec3::new(512.0, -128.0, -128.0), DVec3::new(768.0, 128.0, 128.0), "liquids/water");
        let in_clip = add_box(&mut map, DVec3::splat(-16.0), DVec3::splat(16.0));
        let in_water = add_box(&mut map, DVec3::new(624.0, -16.0, -16.0), DVec3::new(656.0, 16.0, 16.0));
        let mut cull = FaceCull::default();
        cull.update(&map, &GameConfig::default(), &|m| m != "liquids/water", &BTreeSet::new(), true);
        assert!(!cull.pieces.contains_key(&in_clip), "detail inside a clip box still draws");
        assert!(!cull.pieces.contains_key(&in_water), "a rock under water still draws");
        let _ = (clip, water);
    }

    #[test]
    fn moving_bodies_take_no_part() {
        let mut map = Map::new();
        let layer = map.default_layer();
        let mut game = GameConfig::default();
        game.entities.push(serde_json::from_str(r#"{"classname": "prop_physics", "type": "solid", "node_class": "RigidBody3D"}"#).unwrap());
        let body = map.insert(layer, NodeKind::Entity(gt_doc::Entity::new("prop_physics")));
        let crate_box = add_box_with(&mut map, body, DVec3::splat(-128.0), DVec3::splat(128.0), "dev/grey");
        let inner = add_box(&mut map, DVec3::splat(-16.0), DVec3::splat(16.0));
        let wall = add_box(&mut map, DVec3::new(128.0, -128.0, -128.0), DVec3::new(160.0, 128.0, 128.0));
        let mut cull = FaceCull::default();
        cull.update(&map, &game, &|_| true, &BTreeSet::new(), true);
        assert!(!cull.pieces.contains_key(&inner), "a rigid body moves away, what it covers must still draw");
        assert!(!cull.pieces.contains_key(&crate_box) && !cull.pieces.contains_key(&wall), "no back to back culling against a body");
    }

    /// A closed U shaped prism, x 0..96, y 0..64, z 0..64, with a notch at x 32..64, y 32..64.
    fn u_mesh() -> gt_geom::Mesh {
        let outline = [(0.0, 0.0), (96.0, 0.0), (96.0, 32.0), (96.0, 64.0), (64.0, 64.0), (64.0, 32.0), (32.0, 32.0), (32.0, 64.0), (0.0, 64.0), (0.0, 32.0)];
        let at = |k: usize, z: f64| DVec3::new(outline[k].0, outline[k].1, z);
        let data = || gt_geom::FaceData::new("dev/grey", Default::default());
        let mut polys: Vec<(Vec<DVec3>, gt_geom::FaceData)> = Vec::new();
        for cap in [vec![0, 1, 2, 5, 6, 9], vec![9, 6, 7, 8], vec![5, 2, 3, 4]] {
            polys.push((cap.iter().map(|k| at(*k, 0.0)).rev().collect(), data()));
            polys.push((cap.iter().map(|k| at(*k, 64.0)).collect(), data()));
        }

        for k in 0..outline.len() {
            let n = (k + 1) % outline.len();
            polys.push((vec![at(k, 0.0), at(n, 0.0), at(n, 64.0), at(k, 64.0)], data()));
        }

        gt_geom::Mesh::from_polygons(polys)
    }

    #[test]
    fn a_face_crossing_the_opening_of_a_concave_mesh_stays() {
        let mut map = Map::new();
        let layer = map.default_layer();
        let u = map.insert(layer, NodeKind::Mesh(u_mesh()));
        assert!(map.mesh(u).unwrap().edge_faces().values().all(|f| f.len() == 2), "the test shape is closed");
        // Its front face has every corner and its centre inside the U, but the middle of its top edge runs through the notch.
        let bar = add_box(&mut map, DVec3::new(8.0, 8.0, 8.0), DVec3::new(88.0, 40.0, 56.0));
        let buried = add_box(&mut map, DVec3::new(4.0, 4.0, 58.0), DVec3::new(20.0, 20.0, 62.0));
        let cull = run(&map);
        let front = face_towards(&map, bar, DVec3::NEG_Z);
        assert!(!cull.pieces.get(&bar).is_some_and(|p| p.get(&front).is_some_and(|v| v.is_empty())), "the face shows through the notch");
        assert_eq!(cull.pieces[&buried].len(), 6, "a box wholly inside the U is still dropped");
    }

    #[test]
    fn moving_a_brush_away_restores_its_neighbour() {
        let mut map = Map::new();
        let a = add_box(&mut map, DVec3::ZERO, DVec3::splat(64.0));
        let b = add_box(&mut map, DVec3::new(64.0, 0.0, 0.0), DVec3::new(128.0, 64.0, 64.0));
        let mut cull = run(&map);
        *map.brush_mut(b).unwrap() = Brush::from_aabb(&Aabb::new(DVec3::new(96.0, 0.0, 0.0), DVec3::new(160.0, 64.0, 64.0)), "dev/grey").unwrap();
        let changed = cull.update(&map, &GameConfig::default(), &|_| true, &BTreeSet::from([b]), false);
        assert!(changed.contains(&a) && changed.contains(&b));
        assert!(cull.pieces.is_empty());
    }
}
