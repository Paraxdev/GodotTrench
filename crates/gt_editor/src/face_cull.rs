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

const COPLANAR_DIST: f64 = 0.02;
const SAME_NORMAL: f64 = 0.9995;

type PlaneKey = (i64, i64, i64, i64);

#[derive(Clone)]
struct CullFace {
    face: usize,
    key: PlaneKey,
    normal: DVec3,
    dist: f64,
    bounds: Aabb,
    polygon: Vec<DVec3>,
    closed: bool,
    is_mesh: bool,
}

#[derive(Default)]
pub struct FaceCull {
    faces: HashMap<NodeId, Vec<CullFace>>,
    planes: HashMap<PlaneKey, BTreeSet<(NodeId, usize)>>,
    /// Bounds of every closed solid, for the broad phase of the interior pass.
    solids: HashMap<NodeId, Aabb>,
    pub pieces: HashMap<NodeId, FacePieces>,
}

/// Bounds of a node that can swallow another solid's faces: a closed, visible solid. Same eligibility as
/// `FaceCull::node_faces`, but the material does not matter, a tool textured box still hides what is inside it.
fn solid_bounds(map: &Map, game: &GameConfig, id: NodeId) -> Option<Aabb> {
    let node = map.get(id)?;
    if map.is_hidden(id) || !map.in_cordon(id) {
        return None;
    }

    let entity = map.owning_entity(id).and_then(|e| map.entity(e));
    if entity.is_some_and(|e| e.classname.starts_with("trigger") || game.entity(&e.classname).is_some_and(|d| d.node_class == "Area3D")) {
        return None;
    }

    match &node.kind {
        NodeKind::Brush(b) if b.faces.iter().all(|f| f.data.disp.is_none()) => Some(b.bounds()),
        NodeKind::Mesh(m) if m.edge_faces().values().all(|f| f.len() == 2) => Some(m.bounds()),
        _ => None,
    }
}

/// True when every corner of `face` sits well inside the solid `id`. Each sample is probed a little to both
/// sides of the face: a face resting on the other solid's surface leaves it on one side and is left to the
/// coplanar pass, only a face with solid material on both sides counts as buried.
fn buried_in(map: &Map, id: NodeId, face: &CullFace) -> bool {
    let Some(node) = map.get(id) else { return false };
    let nudge = face.normal * COPLANAR_DIST;
    let mut samples: Vec<DVec3> = Vec::with_capacity((face.polygon.len() + 1) * 2);
    for p in face.polygon.iter().chain(std::iter::once(&polygon::centroid(&face.polygon))) {
        samples.push(*p + nudge);
        samples.push(*p - nudge);
    }

    match &node.kind {
        NodeKind::Brush(b) => samples.iter().all(|p| b.contains_point(*p)),
        NodeKind::Mesh(m) => samples.iter().all(|p| m.contains_point(*p)),
        _ => false,
    }
}

/// Opposite normals share a key so back to back faces land in one group.
fn plane_key(normal: DVec3, dist: f64) -> PlaneKey {
    let flip = [normal.x, normal.y, normal.z].into_iter().find(|c| c.abs() > 1e-6).is_some_and(|c| c < 0.0);
    let (n, d) = if flip { (-normal, -dist) } else { (normal, dist) };
    ((n.x * 1000.0).round() as i64, (n.y * 1000.0).round() as i64, (n.z * 1000.0).round() as i64, (d * 8.0).round() as i64)
}

impl FaceCull {
    /// Faces of a node that take part: opaque, single sided, not tool textures, triggers or displacements.
    fn node_faces(map: &Map, game: &GameConfig, opaque: &(dyn Fn(&str) -> bool + Sync), id: NodeId) -> Vec<CullFace> {
        let Some(node) = map.get(id) else { return Vec::new() };
        if map.is_hidden(id) || !map.in_cordon(id) {
            return Vec::new();
        }

        let entity = map.owning_entity(id).and_then(|e| map.entity(e));
        let volume = entity.is_some_and(|e| e.classname.starts_with("trigger") || game.entity(&e.classname).is_some_and(|d| d.node_class == "Area3D"));
        if volume {
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
    fn with_neighbours(&self, map: &Map, game: &GameConfig, dirty: &BTreeSet<NodeId>) -> Vec<NodeId> {
        let mut regions: Vec<Aabb> = Vec::new();
        for id in dirty {
            regions.extend(self.solids.get(id).copied());
            regions.extend(solid_bounds(map, game, *id));
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
                    !already_hidden && containers.iter().any(|other| self.solids.get(other).is_some_and(|b| b.contains(&f.bounds)) && buried_in(map, *other, f))
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
            self.with_neighbours(map, game, dirty)
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
            if let Some(b) = solid_bounds(map, game, *id) {
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
fn priority(face: &CullFace, id: NodeId) -> (u8, NodeId, usize) {
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
