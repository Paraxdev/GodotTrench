//! Linked groups: groups sharing a link id mirror the contents of the one that was edited last.

use std::collections::{BTreeMap, BTreeSet};

use gt_core::{DMat4, DVec3, NodeId};

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

fn kinds_match(a: &NodeKind, b: &NodeKind) -> bool {
    match (a, b) {
        (NodeKind::Brush(x), NodeKind::Brush(y)) => {
            x.vertices.len() == y.vertices.len()
                && x.faces.len() == y.faces.len()
                && x.vertices.iter().zip(&y.vertices).all(|(p, q)| close(*p, *q))
                && x.faces.iter().zip(&y.faces).all(|(f, g)| f.indices == g.indices && f.data.material == g.data.material)
        }
        (NodeKind::Mesh(x), NodeKind::Mesh(y)) => {
            x.vertices.len() == y.vertices.len()
                && x.faces.len() == y.faces.len()
                && x.vertices.iter().zip(&y.vertices).all(|(p, q)| close(*p, *q))
                && x.faces.iter().zip(&y.faces).all(|(f, g)| f.indices == g.indices && f.data.material == g.data.material)
        }
        (NodeKind::Entity(x), NodeKind::Entity(y)) => {
            x.classname == y.classname && x.properties == y.properties && x.outputs == y.outputs && close(x.origin, y.origin) && close(x.angles, y.angles)
        }
        (NodeKind::Group(x), NodeKind::Group(y)) => x.name == y.name && x.link_id == y.link_id,
        (NodeKind::Terrain(x), NodeKind::Terrain(y)) => close(x.origin, y.origin) && x.heights == y.heights && x.layers == y.layers,
        (NodeKind::Scatter(x), NodeKind::Scatter(y)) => {
            x.items == y.items && x.instances.len() == y.instances.len() && x.instances.iter().zip(&y.instances).all(|(p, q)| close(p.position, q.position))
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
    for members in link_sets(after).into_values() {
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
}
