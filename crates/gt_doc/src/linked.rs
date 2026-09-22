//! Linked groups: groups sharing a link id mirror the contents of the one that was edited last.

use std::collections::{BTreeMap, BTreeSet};

use gt_core::{DMat4, DVec3, NodeId};
use serde::Serialize;
use serde_json::Value;

use crate::format;
use crate::map::{Map, NodeKind};
use crate::ops::{self, EditOptions};
use crate::selection::Selection;

fn link_sets(map: &Map) -> BTreeMap<u64, Vec<NodeId>> {
    let mut sets: BTreeMap<u64, Vec<NodeId>> = BTreeMap::new();
    for (id, node) in map.nodes.iter() {
        if let NodeKind::Group(g) = &node.kind
            && let Some(link) = g.link_id
        {
            sets.entry(link).or_default().push(*id);
        }
    }

    sets
}

fn group_transform(map: &Map, id: NodeId) -> DMat4 {
    match map.get(id).map(|n| &n.kind) {
        Some(NodeKind::Group(g)) => g.transform,
        _ => DMat4::IDENTITY,
    }
}

/// Descendants in depth-first order with their depth, leaves expressed in the link set's shared space.
fn normalized(map: &Map, group: NodeId) -> Vec<(usize, NodeKind)> {
    let inverse = group_transform(map, group).inverse();
    let mut out = Vec::new();
    let mut stack: Vec<(NodeId, usize)> = map.get(group).map(|n| n.children.iter().rev().map(|c| (*c, 1)).collect()).unwrap_or_default();
    while let Some((id, depth)) = stack.pop() {
        let Some(node) = map.get(id) else { continue };
        let leaf = node.children.is_empty();
        let kind = match &node.kind {
            NodeKind::Brush(b) => NodeKind::Brush(b.transformed(&inverse, true)),
            NodeKind::Mesh(m) => NodeKind::Mesh(m.transformed(&inverse, true)),
            NodeKind::Terrain(t) => NodeKind::Terrain(t.transformed(&inverse)),
            NodeKind::Scatter(s) => {
                let mut s = s.clone();
                s.transform(&inverse);
                NodeKind::Scatter(s)
            }
            NodeKind::Entity(e) if leaf => {
                let mut e = e.clone();
                e.transform_by(&inverse);
                NodeKind::Entity(e)
            }
            NodeKind::Instance(i) => {
                let mut e = crate::Entity::new("");
                e.origin = i.origin;
                e.angles = i.angles;
                e.transform_by(&inverse);
                NodeKind::Instance(crate::map::Instance { origin: e.origin, angles: e.angles, ..i.clone() })
            }
            other => other.clone(),
        };
        out.push((depth, kind));
        stack.extend(node.children.iter().rev().map(|c| (*c, depth + 1)));
    }

    out
}

fn close(a: DVec3, b: DVec3) -> bool {
    (a - b).abs().max_element() < 1e-3
}

/// Numbers equal up to the rounding a round trip through the group transforms leaves behind.
fn values_close(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => match (x.as_f64(), y.as_f64()) {
            (Some(x), Some(y)) => (x - y).abs() < 1e-3 || (x - y).abs() <= 1e-6 * x.abs().max(y.abs()),
            _ => x == y,
        },
        (Value::Array(x), Value::Array(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| values_close(p, q)),
        (Value::Object(x), Value::Object(y)) => x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| values_close(v, w))),
        _ => a == b,
    }
}

fn content<T: Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap_or(Value::Null)
}

/// Whole content of two normalized nodes, so any edit (UVs, face properties, paint, entity keys) counts.
fn kinds_match(a: &NodeKind, b: &NodeKind) -> bool {
    match (a, b) {
        (NodeKind::Brush(x), NodeKind::Brush(y)) => values_close(&content(x), &content(y)),
        (NodeKind::Mesh(x), NodeKind::Mesh(y)) => values_close(&content(x), &content(y)),
        (NodeKind::Entity(x), NodeKind::Entity(y)) => values_close(&content(x), &content(y)),
        (NodeKind::Instance(x), NodeKind::Instance(y)) => values_close(&content(x), &content(y)),
        (NodeKind::Scatter(x), NodeKind::Scatter(y)) => values_close(&content(x), &content(y)),
        // Nested groups differ in their placement by design.
        (NodeKind::Group(x), NodeKind::Group(y)) => x.name == y.name && x.link_id == y.link_id,
        // Terrains only move and scale, which leaves everything but origin and cell size exact.
        (NodeKind::Terrain(x), NodeKind::Terrain(y)) => {
            close(x.origin, y.origin)
                && (x.cell_size - y.cell_size).abs() < 1e-6
                && gt_geom::Terrain { origin: y.origin, cell_size: y.cell_size, ..x.clone() } == *y
        }
        _ => a == b,
    }
}

fn subtree_changed(before: &Map, after: &Map, group: NodeId) -> bool {
    let now: BTreeSet<NodeId> = std::iter::once(group).chain(after.descendants(group)).collect();
    let then: BTreeSet<NodeId> = if before.contains(group) { std::iter::once(group).chain(before.descendants(group)).collect() } else { BTreeSet::new() };
    if now != then {
        return true;
    }

    now.iter().any(|id| match (before.get(*id), after.get(*id)) {
        (Some(a), Some(b)) => a.kind != b.kind || a.children != b.children,
        _ => true,
    })
}

/// Copies the contents of edited linked groups into the other members of their sets. Returns how many groups were rewritten.
pub fn sync(before: &Map, after: &mut Map) -> usize {
    let mut updated = 0;
    let links: Vec<u64> = link_sets(after).into_keys().collect();
    for link in links {
        // Rebuilding an outer set replaces the linked groups nested in its copies, so members are looked up fresh.
        let members = link_sets(after).remove(&link).unwrap_or_default();
        if members.len() < 2 {
            continue;
        }

        let Some(&source) = members.iter().find(|g| subtree_changed(before, after, **g)) else { continue };
        let source_content = normalized(after, source);
        let source_transform = group_transform(after, source);
        let children = after.get(source).map(|n| n.children.clone()).unwrap_or_default();
        let text = format::nodes_to_string(after, &children);
        for &target in &members {
            if target == source {
                continue;
            }

            let target_content = normalized(after, target);
            let same = target_content.len() == source_content.len()
                && target_content.iter().zip(&source_content).all(|((da, ka), (db, kb))| da == db && kinds_match(ka, kb));
            if same {
                continue;
            }

            for c in after.get(target).map(|n| n.children.clone()).unwrap_or_default() {
                after.remove(c);
            }

            let Ok(ids) = format::paste_nodes(after, target, &text) else { continue };
            let mut sel = Selection::default();
            sel.nodes.extend(ids);
            let m = group_transform(after, target) * source_transform.inverse();
            let leaves = sel.transformables(after);
            ops::transform_nodes(after, &leaves, &m, EditOptions { uv_lock: true, grid: 0.0 });
            updated += 1;
        }
    }

    updated
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Document;
    use gt_core::Aabb;
    use gt_geom::Brush;

    #[test]
    fn edits_propagate_to_linked_copies() {
        let mut doc = Document::new();
        let layer = doc.map.default_layer();
        let brush = doc.edit("add", |m, s| {
            let id = m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(16.0)), "a").unwrap()));
            s.select_node(id);
            id
        });
        let group = doc.edit("group", |m, s| ops::group_selection(m, s, "g", layer)).unwrap();
        let copies = doc.edit("link", |m, s| ops::duplicate_linked(m, s, DVec3::new(100.0, 0.0, 0.0), EditOptions::default()));
        assert_eq!(copies.len(), 1);
        let copy = copies[0];
        assert!((doc.map.bounds(copy).min.x - 100.0).abs() < 1e-9);

        // Moving a linked group as a whole does not touch the others.
        doc.select(|_, s| {
            s.clear();
            s.select_node(copy);
        });
        doc.edit("move", |m, s| ops::translate_selection(m, s, DVec3::new(0.0, 0.0, 50.0), EditOptions::default()));
        assert!((doc.map.bounds(group).min.z).abs() < 1e-9);

        // Editing inside the first group rebuilds the copy at its own placement.
        doc.select(|_, s| {
            s.clear();
            s.select_node(brush);
        });
        doc.edit("scale", |m, s| {
            let b = m.brush(brush).unwrap().bounds();
            let bigger = Aabb::new(b.min, b.max + DVec3::new(16.0, 0.0, 0.0));
            ops::transform_selection(m, s, &ops::scale_bounds(&b, &bigger), EditOptions::default());
        });
        let cb = doc.map.bounds(copy);
        assert!((cb.size().x - 32.0).abs() < 1e-6, "copy resized: {cb:?}");
        assert!((cb.min.x - 100.0).abs() < 1e-6 && (cb.min.z - 50.0).abs() < 1e-6, "copy kept its placement: {cb:?}");
    }

    fn linked_pair(doc: &mut Document) -> (NodeId, NodeId, NodeId) {
        let layer = doc.map.default_layer();
        let brush = doc.edit("add", |m, s| {
            let id = m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(16.0)), "a").unwrap()));
            s.select_node(id);
            id
        });
        let group = doc.edit("group", |m, s| ops::group_selection(m, s, "window", layer)).unwrap();
        let copy = doc.edit("link", |m, s| ops::duplicate_linked(m, s, DVec3::new(100.0, 0.0, 0.0), EditOptions::default()))[0];
        (brush, group, copy)
    }

    #[test]
    fn face_edits_propagate() {
        let mut doc = Document::new();
        let (brush, _, copy) = linked_pair(&mut doc);
        doc.edit("retexture", |m, _| m.brush_mut(brush).unwrap().faces[0].data.material = "glass".into());
        let other = doc.map.get(copy).unwrap().children[0];
        assert_eq!(doc.map.brush(other).unwrap().faces[0].data.material, "glass");

        let offset = |doc: &Document| doc.map.brush(doc.map.get(copy).unwrap().children[0]).unwrap().faces[1].data.uv.offset;
        let before = offset(&doc);
        doc.edit("shift", |m, _| m.brush_mut(brush).unwrap().faces[1].data.uv.offset.x += 8.0);
        assert!((offset(&doc) - before - gt_core::DVec2::new(8.0, 0.0)).length() < 1e-6, "{:?} then {:?}", before, offset(&doc));

        doc.edit("props", |m, _| {
            m.brush_mut(brush).unwrap().faces[2].data.props.insert("surface".into(), "metal".into());
        });
        let other = doc.map.get(copy).unwrap().children[0];
        assert_eq!(doc.map.brush(other).unwrap().faces[2].data.props.get("surface").map(String::as_str), Some("metal"));

        let before = doc.map.get(copy).unwrap().children.clone();
        doc.edit("move", |m, s| {
            s.clear();
            s.select_node(copy);
            ops::translate_selection(m, s, DVec3::new(0.0, 32.0, 0.0), EditOptions::default());
        });
        assert_eq!(doc.map.get(copy).unwrap().children, before, "moving a whole linked group rebuilds nothing");
    }

    fn assert_tree_consistent(map: &Map) {
        for (id, node) in map.nodes.iter() {
            match node.parent {
                Some(p) => assert!(map.get(p).is_some_and(|p| p.children.contains(id)), "{id} is an orphan"),
                None => assert!(map.layers.contains(id)),
            }
        }
    }

    #[test]
    fn nested_linked_groups_leave_no_orphans() {
        let mut doc = Document::new();
        let layer = doc.map.default_layer();
        let (brush, window, _) = linked_pair(&mut doc);
        let house = doc.edit("group", |m, s| {
            s.clear();
            s.nodes.extend(m.get(layer).unwrap().children.clone());
            ops::group_selection(m, s, "house", layer)
        });
        let house = house.unwrap();
        let house_copy = doc.edit("link", |m, s| ops::duplicate_linked(m, s, DVec3::new(0.0, 0.0, 200.0), EditOptions::default()))[0];
        // The outer set syncs first when its link id is the lower one, which is when stale inner members bit.
        let groups: Vec<NodeId> = doc.map.nodes.keys().copied().collect();
        for g in groups {
            if let Some(NodeKind::Group(group)) = doc.map.get_mut(g).map(|n| &mut n.kind)
                && group.link_id == Some(1)
            {
                group.link_id = Some(9);
            }
        }

        assert_tree_consistent(&doc.map);

        doc.edit("retexture", |m, _| m.brush_mut(brush).unwrap().faces[0].data.material = "glass".into());
        assert_tree_consistent(&doc.map);
        let materials: Vec<String> = doc.map.brushes().map(|(_, b)| b.faces[0].data.material.clone()).collect();
        assert_eq!(materials.len(), 4, "two windows in each of two houses");
        assert!(materials.iter().all(|m| m == "glass"), "{materials:?}");
        assert!(doc.map.contains(window) && doc.map.contains(house) && doc.map.contains(house_copy));
        assert_eq!(format::from_str(&format::to_string(&doc.map)).unwrap().nodes.len(), doc.map.nodes.len(), "every node is saved");
    }
}
