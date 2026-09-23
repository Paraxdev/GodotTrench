//! Live mode: edits reach a Godot editor that has the map open as small node level deltas, before the map is saved.
//!
//! Messages (one JSON line each, see `live_link`):
//! - `live_begin {path, text, content}` and `live_resync {path, text}` hand Godot the whole map. `content` is the id
//!   the map's file records in its END chunk, so a scene built from that file is not rebuilt.
//! - `live_delta {path, ops}` with `remove {id}`, `set {id, parent, index, node}`, `translate {ids, offset}` and
//!   `properties {properties}`. `node` is the node as stored in the `.gtm` file, without its children.

use std::collections::{BTreeMap, BTreeSet};

use gt_core::{DVec3, NodeId};
use gt_doc::{Map, Node, NodeKind, format};
use gt_geom::FaceData;
use serde_json::{Value, json};

/// Past this many operations the whole map is sent instead.
pub const RESYNC_OPS: usize = 500;
/// Nodes bigger than this (terrains while sculpting, huge scatter sets) wait until editing pauses.
pub const HOLD_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    Remove(NodeId),
    Set(NodeId),
    Translate(Vec<NodeId>, DVec3),
    Properties,
}

/// The map to sync, with whether a drag is in progress and whether editing has paused.
pub struct Frame {
    pub map: Map,
    pub dragging: bool,
    pub idle: bool,
}

pub struct LiveSession {
    pub path: String,
    pub epoch: u64,
    base: Map,
    /// Moved with `translate` only, their UVs are sent once editing pauses.
    unsettled: BTreeSet<NodeId>,
    /// Too big to send while editing.
    held: BTreeSet<NodeId>,
}

impl LiveSession {
    pub fn begin(path: String, map: Map) -> (Self, Value) {
        let message = json!({ "event": "live_begin", "path": path, "text": format::to_string(&map), "content": format::content_id(&map) });
        (Self { path, epoch: 0, base: map, unsettled: BTreeSet::new(), held: BTreeSet::new() }, message)
    }

    /// Nothing is waiting to be sent exactly.
    pub fn settled(&self) -> bool {
        self.unsettled.is_empty() && self.held.is_empty()
    }

    /// The message that brings Godot from the last sent map to `frame.map`, if anything changed.
    pub fn step(&mut self, frame: Frame) -> Option<Value> {
        let Frame { map, dragging, idle } = frame;
        let mut ops = diff(&self.base, &map, dragging);
        if ops.len() > RESYNC_OPS {
            self.unsettled.clear();
            self.held.clear();
            let text = format::to_string(&map);
            self.base = map;
            return Some(json!({ "event": "live_resync", "path": self.path, "text": text }));
        }

        let removed: BTreeSet<NodeId> = ops.iter().filter_map(|op| if let Op::Remove(id) = op { Some(*id) } else { None }).collect();
        let mut sent: BTreeSet<NodeId> = BTreeSet::new();
        for op in &ops {
            match op {
                Op::Set(id) => {
                    sent.insert(*id);
                }
                Op::Translate(ids, _) => self.unsettled.extend(ids.iter().copied()),
                _ => {}
            }
        }

        if idle {
            let pending: Vec<NodeId> =
                self.unsettled.iter().chain(&self.held).copied().filter(|id| map.contains(*id) && !sent.contains(id) && !removed.contains(id)).collect();
            for id in pending {
                if sent.insert(id) {
                    ops.push(Op::Set(id));
                }
            }

            self.unsettled.clear();
            self.held.clear();
        }

        let mut json_ops = Vec::with_capacity(ops.len());
        for op in ops {
            match op {
                Op::Remove(id) => {
                    self.unsettled.remove(&id);
                    self.held.remove(&id);
                    json_ops.push(json!({ "op": "remove", "id": id.0 }));
                }
                Op::Set(id) => {
                    self.unsettled.remove(&id);
                    let Some(node) = format::node_to_json(&map, id) else { continue };
                    if !idle && node.to_string().len() > HOLD_BYTES {
                        self.held.insert(id);
                        continue;
                    }

                    self.held.remove(&id);
                    let (parent, index) = placement(&map, id);
                    json_ops.push(json!({ "op": "set", "id": id.0, "parent": parent, "index": index, "node": node }));
                }
                Op::Translate(ids, offset) => {
                    json_ops.push(json!({ "op": "translate", "ids": ids.iter().map(|i| i.0).collect::<Vec<_>>(), "offset": [offset.x, offset.y, offset.z] }))
                }
                Op::Properties => json_ops.push(json!({ "op": "properties", "properties": map.properties })),
            }
        }

        self.base = map;
        (!json_ops.is_empty()).then(|| json!({ "event": "live_delta", "path": self.path, "ops": json_ops }))
    }
}

fn placement(map: &Map, id: NodeId) -> (Option<u64>, usize) {
    let parent = map.get(id).and_then(|n| n.parent);
    let siblings = match parent {
        Some(p) => map.get(p).map(|n| n.children.as_slice()).unwrap_or_default(),
        None => map.layers.as_slice(),
    };
    (parent.map(|p| p.0), siblings.iter().position(|s| *s == id).unwrap_or(siblings.len()))
}

/// Operations that turn `base` into `new`. Removals name only the top most removed node, parents are set before their
/// children, and while `dragging` pure moves become `translate`.
pub fn diff(base: &Map, new: &Map, dragging: bool) -> Vec<Op> {
    let mut removed = BTreeSet::new();
    let mut set = Vec::new();
    let mut moved: BTreeMap<[i64; 3], (DVec3, Vec<NodeId>)> = BTreeMap::new();
    for item in base.nodes.diff(&new.nodes) {
        match item {
            imbl::ordmap::DiffItem::Add(id, _) => set.push(*id),
            imbl::ordmap::DiffItem::Remove(id, _) => {
                removed.insert(*id);
            }
            imbl::ordmap::DiffItem::Update { old: (id, old), new: (_, node) } => {
                if same_for_godot(old, node) {
                    continue;
                }

                match translation(old, node).filter(|_| dragging) {
                    Some(offset) => {
                        let key = [offset.x, offset.y, offset.z].map(|v| (v * 1e4).round() as i64);
                        moved.entry(key).or_insert((offset, Vec::new())).1.push(*id);
                    }
                    None => set.push(*id),
                }
            }
        }
    }

    let mut ops: Vec<Op> =
        removed.iter().filter(|id| base.get(**id).and_then(|n| n.parent).is_none_or(|p| !removed.contains(&p))).map(|id| Op::Remove(*id)).collect();
    set.sort_by_key(|id| (depth(new, *id), *id));
    ops.extend(set.into_iter().map(Op::Set));
    ops.extend(moved.into_values().map(|(offset, ids)| Op::Translate(ids, offset)));
    if base.properties != new.properties {
        ops.push(Op::Properties);
    }

    ops
}

fn depth(map: &Map, id: NodeId) -> usize {
    std::iter::successors(map.get(id).and_then(|n| n.parent), |p| map.get(*p).and_then(|n| n.parent)).count()
}

/// Differences Godot does not build: child order, hidden, locked, layer colors and linked group bookkeeping.
fn same_for_godot(old: &Node, new: &Node) -> bool {
    if old.parent != new.parent {
        return false;
    }

    match (&old.kind, &new.kind) {
        (NodeKind::Layer(a), NodeKind::Layer(b)) => a.name == b.name && a.omit_from_export == b.omit_from_export,
        (NodeKind::Group(a), NodeKind::Group(b)) => a.name == b.name,
        (a, b) => a == b,
    }
}

fn same_face_data(a: &FaceData, b: &FaceData) -> bool {
    a.material == b.material && a.props == b.props && a.disp == b.disp && a.colors == b.colors
}

fn shifted(a: &[DVec3], b: &[DVec3]) -> Option<DVec3> {
    let offset = *b.first()? - *a.first()?;
    (a.len() == b.len() && a.iter().zip(b).all(|(p, q)| (*q - *p - offset).abs().max_element() < 1e-6)).then_some(offset)
}

/// The offset when `new` is `old` moved without any other change Godot builds. UV offsets are ignored, UV lock changes
/// them on every move and they are sent exactly once editing pauses.
pub fn translation(old: &Node, new: &Node) -> Option<DVec3> {
    let offset = match (&old.kind, &new.kind) {
        (NodeKind::Entity(a), NodeKind::Entity(b)) => {
            let same = a.classname == b.classname && a.angles == b.angles && a.properties == b.properties && a.outputs == b.outputs;
            (same && old.children.is_empty() && new.children.is_empty()).then(|| b.origin - a.origin)
        }
        (NodeKind::Instance(a), NodeKind::Instance(b)) => (a.path == b.path && a.angles == b.angles && a.fixup == b.fixup).then(|| b.origin - a.origin),
        (NodeKind::Brush(a), NodeKind::Brush(b)) => {
            let faces = a.faces.len() == b.faces.len() && a.faces.iter().zip(&b.faces).all(|(x, y)| x.indices == y.indices && same_face_data(&x.data, &y.data));
            if faces { shifted(&a.vertices, &b.vertices) } else { None }
        }
        (NodeKind::Mesh(a), NodeKind::Mesh(b)) => {
            let faces = a.smooth_angle == b.smooth_angle
                && a.faces.len() == b.faces.len()
                && a.faces.iter().zip(&b.faces).all(|(x, y)| x.indices == y.indices && x.uvs == y.uvs && same_face_data(&x.data, &y.data));
            if faces { shifted(&a.vertices, &b.vertices) } else { None }
        }
        (NodeKind::Terrain(a), NodeKind::Terrain(b)) => {
            let same = a.resolution == b.resolution
                && a.cell_size == b.cell_size
                && a.chunk_cells == b.chunk_cells
                && a.layers == b.layers
                && a.heights == b.heights
                && a.splat == b.splat
                && a.holes == b.holes;
            same.then(|| b.origin - a.origin)
        }
        (NodeKind::Scatter(a), NodeKind::Scatter(b)) => {
            let same = a.name == b.name
                && a.kind == b.kind
                && a.items == b.items
                && a.collision == b.collision
                && a.cast_shadows == b.cast_shadows
                && a.visibility_range == b.visibility_range
                && a.instances.len() == b.instances.len()
                && a.instances.iter().zip(&b.instances).all(|(x, y)| x.item == y.item && x.angles == y.angles && x.scale == y.scale);
            if same {
                let from: Vec<DVec3> = a.instances.iter().map(|i| i.position).collect();
                let to: Vec<DVec3> = b.instances.iter().map(|i| i.position).collect();
                shifted(&from, &to)
            } else {
                None
            }
        }
        _ => None,
    };
    offset.filter(|o| o.length_squared() > 0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_core::Aabb;
    use gt_doc::{Entity, Group};
    use gt_geom::Brush;

    fn sample() -> (Map, NodeId, NodeId, NodeId) {
        let mut m = Map::new();
        let layer = m.default_layer();
        let brush = m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::splat(-16.0), DVec3::splat(16.0)), "base/floor").unwrap()));
        let group = m.insert(layer, NodeKind::Group(Group::new("room")));
        let light = m.insert(group, NodeKind::Entity(Entity::new("light")));
        (m, brush, group, light)
    }

    fn edit(map: &Map, f: impl FnOnce(&mut Map)) -> Map {
        let mut m = map.clone();
        f(&mut m);
        m
    }

    #[test]
    fn moves_become_translate_only_while_dragging() {
        let (base, brush, _, light) = sample();
        let moved = edit(&base, |m| {
            let offset = DVec3::new(16.0, 0.0, 8.0);
            if let Some(NodeKind::Brush(b)) = m.get_mut(brush).map(|n| &mut n.kind) {
                *b = b.translated(offset, true);
            }

            if let Some(NodeKind::Entity(e)) = m.get_mut(light).map(|n| &mut n.kind) {
                e.origin += offset;
            }
        });
        assert_eq!(diff(&base, &moved, true), vec![Op::Translate(vec![brush, light], DVec3::new(16.0, 0.0, 8.0))]);
        assert_eq!(diff(&base, &moved, false), vec![Op::Set(brush), Op::Set(light)]);
    }

    #[test]
    fn material_changes_set_and_order_or_visibility_changes_send_nothing() {
        let (base, brush, group, _) = sample();
        let painted = edit(&base, |m| {
            if let Some(NodeKind::Brush(b)) = m.get_mut(brush).map(|n| &mut n.kind) {
                b.faces[0].data.material = "base/wall".into();
            }
        });
        assert_eq!(diff(&base, &painted, true), vec![Op::Set(brush)]);
        let layer = base.default_layer();
        let cosmetic = edit(&base, |m| {
            m.get_mut(layer).unwrap().children.reverse();
            m.get_mut(group).unwrap().hidden = true;
            m.get_mut(brush).unwrap().locked = true;
        });
        assert!(diff(&base, &cosmetic, false).is_empty());
    }

    #[test]
    fn removing_a_group_names_only_the_group_and_new_parents_come_first() {
        let (base, _, group, _) = sample();
        let removed = edit(&base, |m| m.remove(group));
        assert_eq!(diff(&base, &removed, false), vec![Op::Remove(group)]);

        let layer = base.default_layer();
        let mut added_ids = (NodeId(0), NodeId(0));
        let added = edit(&base, |m| {
            let g = m.insert(layer, NodeKind::Group(Group::new("new")));
            let e = m.insert(g, NodeKind::Entity(Entity::new("light")));
            added_ids = (g, e);
        });
        assert_eq!(diff(&base, &added, false), vec![Op::Set(added_ids.0), Op::Set(added_ids.1)]);
    }

    #[test]
    fn session_settles_translated_nodes_and_resyncs_big_changes() {
        let (base, brush, _, _) = sample();
        let (mut session, begin) = LiveSession::begin("c:/game/maps/a.gtm".into(), base.clone());
        assert_eq!(begin["event"], "live_begin");
        assert!(begin["text"].as_str().unwrap().contains("godottrench-map"));

        let moved = edit(&base, |m| {
            if let Some(NodeKind::Brush(b)) = m.get_mut(brush).map(|n| &mut n.kind) {
                *b = b.translated(DVec3::X * 32.0, true);
            }
        });
        let msg = session.step(Frame { map: moved.clone(), dragging: true, idle: false }).unwrap();
        assert_eq!(msg["ops"][0]["op"], "translate");
        assert!(!session.settled());
        assert!(session.step(Frame { map: moved.clone(), dragging: false, idle: false }).is_none());
        let settle = session.step(Frame { map: moved.clone(), dragging: false, idle: true }).unwrap();
        assert_eq!(settle["ops"][0]["op"], "set");
        assert_eq!(settle["ops"][0]["id"], brush.0);
        assert_eq!(settle["ops"][0]["node"]["type"], "brush");
        assert_eq!(settle["ops"][0]["index"], 0);
        assert!(session.settled());
        assert!(session.step(Frame { map: moved.clone(), dragging: false, idle: true }).is_none());

        let layer = base.default_layer();
        let many = edit(&moved, |m| {
            for i in 0..=RESYNC_OPS {
                let mut e = Entity::new("light");
                e.origin = DVec3::X * i as f64;
                m.insert(layer, NodeKind::Entity(e));
            }
        });
        assert_eq!(session.step(Frame { map: many, dragging: false, idle: false }).unwrap()["event"], "live_resync");
    }

    #[test]
    fn big_nodes_wait_for_idle() {
        let (base, _, _, _) = sample();
        let (mut session, _) = LiveSession::begin("a.gtm".into(), base.clone());
        let layer = base.default_layer();
        let mut terrain_id = NodeId(0);
        let big = edit(&base, |m| {
            let terrain = gt_geom::Terrain::new(DVec3::ZERO, [400, 400], 32.0, "base/grass");
            terrain_id = m.insert(layer, NodeKind::Terrain(terrain));
        });
        assert!(session.step(Frame { map: big.clone(), dragging: true, idle: false }).is_none());
        let msg = session.step(Frame { map: big, dragging: false, idle: true }).unwrap();
        assert_eq!(msg["ops"][0]["id"], terrain_id.0);
    }
}
