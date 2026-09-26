//! Editing operations on a map and selection. Callers wrap these in `Document::edit` for undo.

use std::collections::{BTreeMap, BTreeSet};

use gt_core::{Aabb, DMat4, DQuat, DVec3, NodeId};
use gt_geom::{Brush, FaceData, Mesh, csg};

use crate::entity::Entity;
use crate::map::{Group, Map, Node, NodeKind};
use crate::selection::Selection;

/// Nodes an edit replaced, each with the nodes that took its place, empty when it was removed.
pub type Replaced = BTreeMap<NodeId, Vec<NodeId>>;

#[derive(Clone, Copy, Debug)]
pub struct EditOptions {
    pub uv_lock: bool,
    pub grid: f64,
}

impl Default for EditOptions {
    fn default() -> Self {
        Self { uv_lock: true, grid: 16.0 }
    }
}

/// Top level selected nodes that are not inside another selected node.
pub fn selection_roots(map: &Map, sel: &Selection) -> Vec<NodeId> {
    sel.nodes.iter().copied().filter(|id| !map.ancestors(*id).iter().any(|a| sel.nodes.contains(a))).collect()
}

pub fn delete_selection(map: &mut Map, sel: &mut Selection) -> usize {
    let roots = selection_roots(map, sel);
    let parents = remove_nodes(map, &roots);
    remove_empty_containers(map, parents);
    sel.clear();
    roots.len()
}

/// Removes the nodes and returns their former parents, for [`remove_empty_containers`] once any replacements are in.
fn remove_nodes(map: &mut Map, ids: &[NodeId]) -> BTreeSet<NodeId> {
    let mut parents = BTreeSet::new();
    for id in ids {
        if let Some(p) = map.get(*id).and_then(|n| n.parent) {
            parents.insert(p);
        }

        map.remove(*id);
    }

    parents
}

/// Groups and brush entities that lost all children are removed, like TrenchBroom does.
/// Candidates are former parents of removed nodes, so an empty entity among them was a brush entity.
fn remove_empty_containers(map: &mut Map, mut candidates: BTreeSet<NodeId>) {
    while let Some(id) = candidates.pop_first() {
        let Some(node) = map.get(id) else { continue };
        if node.children.is_empty() && matches!(node.kind, NodeKind::Group(_) | NodeKind::Entity(_)) {
            let parent = node.parent;
            map.remove(id);
            if let Some(p) = parent {
                candidates.insert(p);
            }
        }
    }
}

pub fn translate_nodes(map: &mut Map, ids: &[NodeId], offset: DVec3, opts: EditOptions) {
    for id in ids {
        let Some(node) = map.get_mut(*id) else { continue };
        match &mut node.kind {
            NodeKind::Brush(b) => *b = b.translated(offset, opts.uv_lock),
            NodeKind::Mesh(m) => *m = m.translated(offset, opts.uv_lock),
            NodeKind::Terrain(t) => *t = t.translated(offset),
            NodeKind::Scatter(s) => s.transform(&DMat4::from_translation(offset)),
            NodeKind::Entity(e) => {
                if node.children.is_empty() {
                    e.origin = gt_core::snap_vec(e.origin + offset);
                }
            }
            NodeKind::Instance(i) => i.origin = gt_core::snap_vec(i.origin + offset),
            _ => {}
        }
    }
}

pub fn transform_nodes(map: &mut Map, ids: &[NodeId], m: &DMat4, opts: EditOptions) {
    for id in ids {
        let Some(node) = map.get_mut(*id) else { continue };
        let leaf = node.children.is_empty();
        match &mut node.kind {
            NodeKind::Brush(b) => *b = b.transformed(m, opts.uv_lock),
            NodeKind::Mesh(mesh) => *mesh = mesh.transformed(m, opts.uv_lock),
            NodeKind::Terrain(t) => *t = t.transformed(m),
            NodeKind::Scatter(s) => s.transform(m),
            NodeKind::Entity(e) if leaf => e.transform_by(m),
            NodeKind::Instance(i) => {
                let mut e = Entity::new("");
                e.origin = i.origin;
                e.angles = i.angles;
                e.transform_by(m);
                i.origin = e.origin;
                i.angles = e.angles;
            }
            _ => {}
        }
    }
}

pub fn translate_selection(map: &mut Map, sel: &Selection, offset: DVec3, opts: EditOptions) {
    let ids = sel.transformables(map);
    translate_nodes(map, &ids, offset, opts);
    update_group_transforms(map, sel, &DMat4::from_translation(offset));
}

pub fn transform_selection(map: &mut Map, sel: &Selection, m: &DMat4, opts: EditOptions) {
    let ids = sel.transformables(map);
    transform_nodes(map, &ids, m, opts);
    update_group_transforms(map, sel, m);
}

/// Linked groups moved as a whole remember their placement, so edits inside them propagate with the right offset.
fn update_group_transforms(map: &mut Map, sel: &Selection, m: &DMat4) {
    let mut groups: BTreeSet<NodeId> = BTreeSet::new();
    for id in &sel.nodes {
        for g in std::iter::once(*id).chain(map.descendants(*id)) {
            if matches!(map.get(g).map(|n| &n.kind), Some(NodeKind::Group(gr)) if gr.link_id.is_some()) {
                groups.insert(g);
            }
        }
    }

    for g in groups {
        if let Some(Node { kind: NodeKind::Group(group), .. }) = map.get_mut(g) {
            group.transform = *m * group.transform;
        }
    }
}

/// Rotation about `center` around `axis` by `degrees`.
pub fn rotation_about(center: DVec3, axis: DVec3, degrees: f64) -> DMat4 {
    DMat4::from_translation(center) * DMat4::from_quat(DQuat::from_axis_angle(axis.normalize(), degrees.to_radians())) * DMat4::from_translation(-center)
}

pub fn flip_about(center: DVec3, axis: usize) -> DMat4 {
    let mut s = DVec3::ONE;
    s[axis] = -1.0;
    DMat4::from_translation(center) * DMat4::from_scale(s) * DMat4::from_translation(-center)
}

/// Scales from `old` bounds to `new` bounds.
pub fn scale_bounds(old: &Aabb, new: &Aabb) -> DMat4 {
    let os = old.size().max(DVec3::splat(1e-9));
    let s = new.size() / os;
    DMat4::from_translation(new.min) * DMat4::from_scale(s) * DMat4::from_translation(-old.min)
}

pub fn selection_center(map: &Map, sel: &Selection, grid: f64) -> DVec3 {
    let b = map.bounds_of(sel.nodes.iter().copied());
    if b.is_empty() {
        return DVec3::ZERO;
    }

    gt_core::snap_vec_to_grid(b.center(), grid.max(1.0))
}

pub fn duplicate_selection(map: &mut Map, sel: &mut Selection, offset: DVec3, opts: EditOptions) -> Vec<NodeId> {
    let roots = selection_roots(map, sel);
    let mut new_ids = Vec::new();
    let mut copies = std::collections::BTreeMap::new();
    for id in roots {
        let parent = map.get(id).and_then(|n| n.parent).unwrap_or(map.default_layer());
        if let Some(new_id) = map.duplicate_subtree_into(id, parent, &mut copies) {
            new_ids.push(new_id);
        }
    }

    map.retarget_scatter_copies(&copies);

    sel.clear();
    for id in &new_ids {
        sel.nodes.insert(*id);
    }

    translate_selection(map, sel, offset, opts);
    new_ids
}

pub fn create_brush(map: &mut Map, parent: NodeId, brush: Brush) -> NodeId {
    map.insert(parent, NodeKind::Brush(brush))
}

pub fn group_selection(map: &mut Map, sel: &mut Selection, name: &str, parent: NodeId) -> Option<NodeId> {
    let roots = selection_roots(map, sel);
    if roots.is_empty() {
        return None;
    }

    let group = map.insert(parent, NodeKind::Group(Group::new(name)));
    for id in roots {
        let target = match map.owning_entity(id) {
            Some(e) => e,
            None => id,
        };
        map.reparent(target, group);
    }

    sel.clear();
    sel.nodes.insert(group);
    Some(group)
}

pub fn ungroup_selection(map: &mut Map, sel: &mut Selection) {
    let groups: Vec<NodeId> = sel.nodes.iter().copied().filter(|id| matches!(map.get(*id).map(|n| &n.kind), Some(NodeKind::Group(_)))).collect();
    for g in groups {
        let Some(node) = map.get(g).cloned() else { continue };
        let parent = node.parent.unwrap_or(map.default_layer());
        sel.nodes.remove(&g);
        for c in node.children {
            map.reparent(c, parent);
            sel.nodes.insert(c);
        }

        map.remove(g);
    }
}

pub fn set_hidden(map: &mut Map, ids: &[NodeId], hidden: bool) {
    for id in ids {
        if let Some(n) = map.get_mut(*id) {
            n.hidden = hidden;
        }
    }
}

pub fn set_locked(map: &mut Map, ids: &[NodeId], locked: bool) {
    for id in ids {
        if let Some(n) = map.get_mut(*id) {
            n.locked = locked;
        }
    }
}

pub fn hide_selection(map: &mut Map, sel: &mut Selection) {
    let roots = selection_roots(map, sel);
    set_hidden(map, &roots, true);
    sel.clear();
}

/// Hides every object that is not selected or an ancestor of a selected object.
pub fn isolate_selection(map: &mut Map, sel: &Selection) {
    let mut keep: BTreeSet<NodeId> = BTreeSet::new();
    for id in &sel.nodes {
        keep.insert(*id);
        keep.extend(map.ancestors(*id));
        keep.extend(map.descendants(*id));
    }

    let all: Vec<NodeId> = map.nodes.keys().copied().collect();
    for id in all {
        let Some(n) = map.get(id) else { continue };
        if matches!(n.kind, NodeKind::Layer(_)) {
            continue;
        }

        let hide = !keep.contains(&id);
        let parent_kept = n.parent.is_none_or(|p| keep.contains(&p) || matches!(map.get(p).map(|n| &n.kind), Some(NodeKind::Layer(_))));
        if hide && parent_kept {
            map.get_mut(id).unwrap().hidden = true;
        }
    }
}

pub fn unhide_all(map: &mut Map) {
    let ids: Vec<NodeId> = map.nodes.iter().filter(|(_, n)| n.hidden && !matches!(n.kind, NodeKind::Layer(_))).map(|(id, _)| *id).collect();
    set_hidden(map, &ids, false);
}

/// Every selectable object: brushes (world or in brush entities), point entities, instances, respecting closed groups.
pub fn selectable_objects(map: &Map, open_groups: &[NodeId]) -> Vec<NodeId> {
    let mut out = BTreeSet::new();
    for (id, node) in map.nodes.iter() {
        let leaf = match &node.kind {
            NodeKind::Brush(_) | NodeKind::Instance(_) | NodeKind::Mesh(_) | NodeKind::Terrain(_) | NodeKind::Scatter(_) => true,
            NodeKind::Entity(_) => node.children.is_empty(),
            _ => false,
        };
        if leaf && map.is_editable(*id) {
            out.insert(map.selection_target(*id, open_groups));
        }
    }

    out.into_iter().collect()
}

pub fn select_all(map: &Map, sel: &mut Selection, open_groups: &[NodeId]) {
    sel.clear();
    sel.nodes.extend(selectable_objects(map, open_groups));
}

pub fn select_inverse(map: &Map, sel: &mut Selection, open_groups: &[NodeId]) {
    let current = std::mem::take(&mut sel.nodes);
    sel.faces.clear();
    sel.nodes.extend(selectable_objects(map, open_groups).into_iter().filter(|id| !current.contains(id)));
}

/// Objects intersecting any selected brush. The selected brushes are removed (TrenchBroom "select touching").
pub fn select_touching(map: &mut Map, sel: &mut Selection, open_groups: &[NodeId], inside_only: bool) {
    let selectors: Vec<(NodeId, Brush)> = sel.brushes(map).into_iter().filter_map(|id| map.brush(id).cloned().map(|b| (id, b))).collect();
    if selectors.is_empty() {
        return;
    }

    let selector_ids: BTreeSet<NodeId> = selectors.iter().map(|(id, _)| *id).collect();
    let mut result = BTreeSet::new();
    for obj in selectable_objects(map, open_groups) {
        if selector_ids.contains(&obj) || sel.nodes.contains(&obj) {
            continue;
        }

        let mut leaves = vec![obj];
        leaves.extend(map.descendants(obj));
        let hit = leaves.iter().any(|leaf| {
            if let Some(b) = map.brush(*leaf) {
                selectors.iter().any(|(_, s)| if inside_only { s.contains_brush(b) } else { s.intersects(b) })
            } else if map.mesh(*leaf).is_some() || map.terrain(*leaf).is_some() {
                let lb = map.bounds(*leaf);
                selectors.iter().any(|(_, s)| {
                    let sb = s.bounds();
                    if inside_only { sb.contains(&lb) } else { sb.overlaps(&lb, 1e-6) }
                })
            } else if let Some(e) = map.entity(*leaf) {
                selectors.iter().any(|(_, s)| s.contains_point(e.origin))
            } else {
                false
            }
        });
        if hit {
            result.insert(obj);
        }
    }

    let roots = selection_roots(map, sel);
    let parents = remove_nodes(map, &roots);
    remove_empty_containers(map, parents);
    result.retain(|id| map.contains(*id));
    sel.clear();
    sel.nodes = result;
}

pub fn select_by_material(map: &Map, sel: &mut Selection, material: &str, open_groups: &[NodeId]) {
    sel.clear();
    for (id, b) in map.brushes() {
        if map.is_editable(id) && b.faces.iter().any(|f| f.data.material.eq_ignore_ascii_case(material)) {
            sel.nodes.insert(map.selection_target(id, open_groups));
        }
    }

    for (id, m) in map.meshes() {
        if map.is_editable(id) && m.faces.iter().any(|f| f.data.material.eq_ignore_ascii_case(material)) {
            sel.nodes.insert(map.selection_target(id, open_groups));
        }
    }
}

pub fn select_by_classname(map: &Map, sel: &mut Selection, classname: &str) {
    sel.clear();
    for (id, e) in map.entities() {
        if map.is_editable(id) && e.classname == classname {
            sel.nodes.insert(id);
        }
    }
}

/// Selects every sibling of the selected objects inside their parent (group or brush entity).
pub fn select_siblings(map: &Map, sel: &mut Selection) {
    let parents: BTreeSet<NodeId> = sel.nodes.iter().filter_map(|id| map.get(*id).and_then(|n| n.parent)).collect();
    for p in parents {
        if let Some(n) = map.get(p) {
            sel.nodes.extend(n.children.iter().copied().filter(|c| map.is_editable(*c)));
        }
    }
}

pub fn apply_material(map: &mut Map, sel: &Selection, material: &str) {
    if sel.has_faces() {
        for (id, f) in &sel.faces {
            if let Some(b) = map.brush_mut(*id)
                && let Some(face) = b.faces.get_mut(*f)
            {
                face.data.material = material.to_string();
            } else if let Some(face) = map.mesh_mut(*id).and_then(|m| m.faces.get_mut(*f)) {
                face.data.material = material.to_string();
            }
        }
    } else {
        for id in sel.geometry(map) {
            match map.get_mut(id).map(|n| &mut n.kind) {
                Some(NodeKind::Brush(b)) => b.set_material(material),
                Some(NodeKind::Mesh(m)) => m.faces.iter_mut().for_each(|f| f.data.material = material.to_string()),
                Some(NodeKind::Terrain(t)) => {
                    if let Some(l) = t.layers.first_mut() {
                        l.material = material.to_string();
                    }
                }
                _ => {}
            }
        }
    }
}

/// The name of the first of `ids` that was given one, for the node that replaces them all.
fn first_label(map: &Map, ids: &[NodeId]) -> Option<String> {
    ids.iter().find_map(|id| map.get(*id).and_then(|n| n.label.clone()))
}

/// Subtracts the selected brushes from every other intersecting editable brush, then removes the selected ones.
/// Returns how many brushes were cut, and every replaced brush (the cutters with no replacement).
pub fn csg_subtract(map: &mut Map, sel: &mut Selection, carve: csg::CarveMaterial) -> (usize, Replaced) {
    let mut replaced = Replaced::new();
    let cutters: Vec<(NodeId, Brush)> = sel.brushes(map).into_iter().filter_map(|id| map.brush(id).cloned().map(|b| (id, b))).collect();
    if cutters.is_empty() {
        return (0, replaced);
    }

    let cutter_ids: Vec<NodeId> = cutters.iter().map(|(id, _)| *id).collect();
    let targets: Vec<NodeId> = map.brushes().filter(|(id, _)| !cutter_ids.contains(id) && map.is_editable(*id)).map(|(id, _)| id).collect();
    let mut changed = 0;
    let mut parents = BTreeSet::new();
    for t in targets {
        let Some(original) = map.brush(t).cloned() else { continue };
        let mut pieces = vec![original.clone()];
        for (_, c) in &cutters {
            pieces = pieces.iter().flat_map(|p| csg::subtract_with(p, c, carve)).collect();
        }

        if pieces.len() == 1 && pieces[0] == original {
            continue;
        }

        changed += 1;
        let parent = map.get(t).and_then(|n| n.parent).unwrap_or(map.default_layer());
        let label = map.get(t).and_then(|n| n.label.clone());
        parents.extend(remove_nodes(map, &[t]));
        let new_ids = pieces.into_iter().map(|p| map.insert_labeled(parent, NodeKind::Brush(p), label.clone())).collect();
        replaced.insert(t, new_ids);
    }

    parents.extend(remove_nodes(map, &cutter_ids));
    remove_empty_containers(map, parents);
    replaced.extend(cutter_ids.into_iter().map(|c| (c, Vec::new())));
    sel.clear();
    (changed, replaced)
}

pub fn csg_merge(map: &mut Map, sel: &mut Selection, default_material: &str) -> Option<NodeId> {
    let ids = sel.brushes(map);
    if ids.len() < 2 {
        return None;
    }

    let brushes: Vec<Brush> = ids.iter().filter_map(|id| map.brush(*id).cloned()).collect();
    let refs: Vec<&Brush> = brushes.iter().collect();
    let merged = csg::convex_merge(&refs, default_material).ok()?;
    let parent = map.get(ids[0]).and_then(|n| n.parent).unwrap_or(map.default_layer());
    let label = first_label(map, &ids);
    let parents = remove_nodes(map, &ids);
    let new_id = map.insert_labeled(parent, NodeKind::Brush(merged), label);
    remove_empty_containers(map, parents);
    sel.clear();
    sel.nodes.insert(new_id);
    Some(new_id)
}

pub fn csg_intersect(map: &mut Map, sel: &mut Selection) -> Option<NodeId> {
    let ids = sel.brushes(map);
    if ids.len() < 2 {
        return None;
    }

    let mut result = map.brush(ids[0])?.clone();
    for id in &ids[1..] {
        result = csg::intersect(&result, map.brush(*id)?).ok()?;
    }

    let parent = map.get(ids[0]).and_then(|n| n.parent).unwrap_or(map.default_layer());
    let label = first_label(map, &ids);
    let parents = remove_nodes(map, &ids);
    let new_id = map.insert_labeled(parent, NodeKind::Brush(result), label);
    remove_empty_containers(map, parents);
    sel.clear();
    sel.nodes.insert(new_id);
    Some(new_id)
}

/// Returns each hollowed brush with the walls that replaced it.
pub fn csg_hollow(map: &mut Map, sel: &mut Selection, thickness: f64) -> Replaced {
    let ids = sel.brushes(map);
    let mut replaced = Replaced::new();
    for id in ids {
        let Some(b) = map.brush(id).cloned() else { continue };
        let walls = csg::hollow(&b, thickness);
        if walls.len() <= 1 {
            continue;
        }

        let parent = map.get(id).and_then(|n| n.parent).unwrap_or(map.default_layer());
        let label = map.get(id).and_then(|n| n.label.clone());
        map.remove(id);
        let new_ids: Vec<NodeId> = walls.into_iter().map(|w| map.insert_labeled(parent, NodeKind::Brush(w), label.clone())).collect();
        replaced.insert(id, new_ids);
    }

    sel.clear();
    sel.nodes.extend(replaced.values().flatten().copied());
    replaced
}

/// Moves the selected brushes into a new brush entity.
pub fn create_brush_entity(map: &mut Map, sel: &mut Selection, classname: &str, parent: NodeId) -> Option<NodeId> {
    let brushes: Vec<NodeId> = sel.geometry(map).into_iter().filter(|id| map.terrain(*id).is_none()).collect();
    if brushes.is_empty() {
        return None;
    }

    let e = Entity::new(classname);
    let old_parents: BTreeSet<NodeId> = brushes.iter().filter_map(|b| map.get(*b).and_then(|n| n.parent)).collect();
    let entity = map.insert(parent, NodeKind::Entity(e));
    for b in brushes {
        map.reparent(b, entity);
    }

    remove_empty_containers(map, old_parents);
    sel.clear();
    sel.nodes.insert(entity);
    Some(entity)
}

pub fn move_brushes_to_world(map: &mut Map, sel: &mut Selection, layer: NodeId) {
    let brushes = sel.geometry(map);
    let old_parents: BTreeSet<NodeId> = brushes.iter().filter_map(|b| map.get(*b).and_then(|n| n.parent)).collect();
    for b in &brushes {
        map.reparent(*b, layer);
    }

    remove_empty_containers(map, old_parents);
    sel.clear();
    sel.nodes.extend(brushes);
}

pub fn create_point_entity(map: &mut Map, parent: NodeId, classname: &str, origin: DVec3) -> NodeId {
    let mut e = Entity::new(classname);
    e.origin = origin;
    map.insert(parent, NodeKind::Entity(e))
}

pub fn move_to_layer(map: &mut Map, sel: &Selection, layer: NodeId) {
    for id in selection_roots(map, sel) {
        let target = map.owning_entity(id).unwrap_or(id);
        map.reparent(target, layer);
    }
}

pub fn snap_vertices(map: &mut Map, sel: &Selection, grid: f64) {
    for id in sel.brushes(map) {
        let Some(b) = map.brush(id).cloned() else { continue };
        let pts: Vec<DVec3> = b.vertices.iter().map(|v| gt_core::snap_vec_to_grid(*v, grid)).collect();
        if let Ok(nb) = Brush::from_points(&pts, &b.planes(), "")
            && let Some(slot) = map.brush_mut(id)
        {
            *slot = nb;
        }
    }
}

pub fn default_face(material: &str, normal: DVec3) -> FaceData {
    FaceData::with_normal(material, normal)
}

/// Replaces selected brushes with editable meshes. With `join`, all of them become one mesh.
pub fn convert_to_mesh(map: &mut Map, sel: &mut Selection, join: bool) -> Vec<NodeId> {
    let brushes = sel.brushes(map);
    let mut out = Vec::new();
    let mut joined: Option<(NodeId, Mesh)> = None;
    let joined_label = first_label(map, &brushes);
    let mut parents = BTreeSet::new();
    for id in brushes {
        let Some(b) = map.brush(id).cloned() else { continue };
        let parent = map.get(id).and_then(|n| n.parent).unwrap_or(map.default_layer());
        let label = map.get(id).and_then(|n| n.label.clone());
        let mesh = Mesh::from_brush(&b);
        parents.extend(remove_nodes(map, &[id]));
        if join {
            match &mut joined {
                Some((_, m)) => m.join(&mesh),
                None => joined = Some((parent, mesh)),
            }
        } else {
            out.push(map.insert_labeled(parent, NodeKind::Mesh(mesh), label));
        }
    }

    if let Some((parent, mut mesh)) = joined {
        mesh.weld(1e-4);
        out.push(map.insert_labeled(parent, NodeKind::Mesh(mesh), joined_label));
    }

    remove_empty_containers(map, parents);
    sel.clear();
    sel.nodes.extend(out.iter().copied());
    out
}

/// Replaces selected meshes with their convex hulls as brushes.
pub fn convert_to_brushes(map: &mut Map, sel: &mut Selection) -> Vec<NodeId> {
    let mut out = Vec::new();
    for id in sel.meshes(map) {
        let Some(m) = map.mesh(id).cloned() else { continue };
        let Ok(b) = m.to_brush() else { continue };
        let parent = map.get(id).and_then(|n| n.parent).unwrap_or(map.default_layer());
        let label = map.get(id).and_then(|n| n.label.clone());
        map.remove(id);
        out.push(map.insert_labeled(parent, NodeKind::Brush(b), label));
    }

    sel.clear();
    sel.nodes.extend(out.iter().copied());
    out
}

/// Joins the selected meshes (and brushes) into the first mesh.
pub fn join_meshes(map: &mut Map, sel: &mut Selection) -> Option<NodeId> {
    let ids: Vec<NodeId> = sel.geometry(map).into_iter().filter(|id| map.terrain(*id).is_none()).collect();
    if ids.len() < 2 {
        return None;
    }

    let first = ids.iter().copied().find(|id| map.mesh(*id).is_some());
    let mut result = first.and_then(|id| map.mesh(id)).cloned().unwrap_or_default();
    for id in &ids {
        if Some(*id) == first {
            continue;
        }

        match map.get(*id).map(|n| &n.kind) {
            Some(NodeKind::Mesh(m)) => {
                result.smooth_angle = result.smooth_angle.max(m.smooth_angle);
                result.join(m);
            }
            Some(NodeKind::Brush(b)) => result.join(&Mesh::from_brush(b)),
            _ => {}
        }
    }

    result.weld(1e-4);
    let others: Vec<NodeId> = ids.iter().copied().filter(|id| Some(*id) != first).collect();
    let target = match first {
        Some(id) => {
            if let Some(slot) = map.mesh_mut(id) {
                *slot = result;
            }

            id
        }
        None => {
            let parent = map.get(ids[0]).and_then(|n| n.parent).unwrap_or(map.default_layer());
            let label = first_label(map, &ids);
            map.insert_labeled(parent, NodeKind::Mesh(result), label)
        }
    };
    let parents = remove_nodes(map, &others);
    remove_empty_containers(map, parents);
    sel.clear();
    sel.nodes.insert(target);
    Some(target)
}

/// Duplicates the selected groups as linked copies offset by `offset`.
pub fn duplicate_linked(map: &mut Map, sel: &mut Selection, offset: DVec3, opts: EditOptions) -> Vec<NodeId> {
    let groups: Vec<NodeId> = selection_roots(map, sel).into_iter().filter(|id| matches!(map.get(*id).map(|n| &n.kind), Some(NodeKind::Group(_)))).collect();
    let mut next_link = map
        .nodes
        .values()
        .filter_map(|n| match &n.kind {
            NodeKind::Group(g) => g.link_id,
            _ => None,
        })
        .max()
        .unwrap_or(0)
        + 1;
    let mut new_ids = Vec::new();
    for g in groups {
        let link = match map.get_mut(g).map(|n| &mut n.kind) {
            Some(NodeKind::Group(group)) => *group.link_id.get_or_insert_with(|| {
                next_link += 1;
                next_link - 1
            }),
            _ => continue,
        };
        let parent = map.get(g).and_then(|n| n.parent).unwrap_or(map.default_layer());
        if let Some(copy) = map.duplicate_subtree(g, parent) {
            if let Some(NodeKind::Group(group)) = map.get_mut(copy).map(|n| &mut n.kind) {
                group.link_id = Some(link);
            }

            new_ids.push(copy);
        }
    }

    sel.clear();
    sel.nodes.extend(new_ids.iter().copied());
    translate_selection(map, sel, offset, opts);
    new_ids
}

/// Turns selected linked groups back into independent groups.
pub fn unlink_groups(map: &mut Map, sel: &Selection) {
    for id in selection_roots(map, sel) {
        if let Some(NodeKind::Group(g)) = map.get_mut(id).map(|n| &mut n.kind) {
            g.link_id = None;
            g.transform = DMat4::IDENTITY;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world_with_boxes() -> (Map, Vec<NodeId>) {
        let mut m = Map::new();
        let l = m.default_layer();
        let a = create_brush(&mut m, l, Brush::from_aabb(&Aabb::new(DVec3::splat(-32.0), DVec3::splat(32.0)), "a").unwrap());
        let b = create_brush(&mut m, l, Brush::from_aabb(&Aabb::new(DVec3::new(-8.0, -64.0, -8.0), DVec3::new(8.0, 64.0, 8.0)), "b").unwrap());
        let c = create_brush(&mut m, l, Brush::from_aabb(&Aabb::new(DVec3::splat(100.0), DVec3::splat(120.0)), "c").unwrap());
        (m, vec![a, b, c])
    }

    #[test]
    fn subtract_splits_target() {
        let (mut m, ids) = world_with_boxes();
        let mut sel = Selection::default();
        sel.nodes.insert(ids[1]);
        let (changed, replaced) = csg_subtract(&mut m, &mut sel, csg::CarveMaterial::Cutter);
        assert_eq!(changed, 1);
        assert_eq!(m.brush_count(), 5);
        assert_eq!(replaced[&ids[1]], Vec::<NodeId>::new(), "the cutter is gone without a replacement");
        assert_eq!(replaced[&ids[0]].len(), 4, "the target maps to its pieces");
        assert!(replaced[&ids[0]].iter().all(|id| m.brush(*id).is_some()));
        assert!(!replaced.contains_key(&ids[2]), "brushes the cutter missed keep their ids");
    }

    #[test]
    fn hollow_maps_each_brush_to_its_walls() {
        let (mut m, ids) = world_with_boxes();
        let mut sel = Selection::default();
        sel.nodes.insert(ids[0]);
        let replaced = csg_hollow(&mut m, &mut sel, 4.0);
        assert_eq!(replaced.len(), 1);
        assert_eq!(replaced[&ids[0]].len(), sel.nodes.len());
        assert!(!m.contains(ids[0]));
    }

    fn in_entity(m: &mut Map, id: NodeId, classname: &str) -> NodeId {
        let l = m.default_layer();
        let e = m.insert(l, NodeKind::Entity(Entity::new(classname)));
        m.reparent(id, e);
        e
    }

    #[test]
    fn csg_and_join_remove_emptied_brush_entities() {
        let (mut m, ids) = world_with_boxes();
        let cutter = in_entity(&mut m, ids[1], "func_detail");
        let mut sel = Selection::default();
        sel.nodes.insert(ids[1]);
        csg_subtract(&mut m, &mut sel, csg::CarveMaterial::Cutter);
        assert!(!m.contains(cutter), "the cutter's entity went with it");

        for op in 0..3 {
            let (mut m, ids) = world_with_boxes();
            let a = in_entity(&mut m, ids[0], "func_wall");
            let b = in_entity(&mut m, ids[1], "func_door");
            let mut sel = Selection::default();
            sel.nodes.extend([ids[0], ids[1]]);
            let result = match op {
                0 => csg_merge(&mut m, &mut sel, "m"),
                1 => csg_intersect(&mut m, &mut sel),
                _ => join_meshes(&mut m, &mut sel),
            };
            let result = result.unwrap();
            assert_eq!(m.get(result).unwrap().parent, Some(a), "op {op}");
            assert!(!m.contains(b), "op {op} left an empty brush entity");
        }

        let (mut m, ids) = world_with_boxes();
        let door = in_entity(&mut m, ids[1], "func_door");
        let mut sel = Selection::default();
        sel.nodes.insert(ids[1]);
        select_touching(&mut m, &mut sel, &[], false);
        assert!(!m.contains(door));
    }

    #[test]
    fn replacing_brushes_keeps_their_name() {
        let named = |m: &Map, ids: &[NodeId]| ids.iter().all(|id| m.get(*id).unwrap().label.as_deref() == Some("pillar"));
        for op in 0..6 {
            let (mut m, ids) = world_with_boxes();
            m.rename(ids[1], "pillar");
            let mut sel = Selection::default();
            sel.nodes.extend([ids[0], ids[1]]);
            let result = match op {
                0 => vec![csg_merge(&mut m, &mut sel, "m").unwrap()],
                1 => vec![csg_intersect(&mut m, &mut sel).unwrap()],
                2 => convert_to_mesh(&mut m, &mut sel, true),
                3 => {
                    let meshes = convert_to_mesh(&mut m, &mut sel, false);
                    assert_eq!(m.get(meshes[0]).unwrap().label, None, "the unnamed brush stays unnamed");
                    sel.nodes = meshes[1..].iter().copied().collect();
                    convert_to_brushes(&mut m, &mut sel)
                }
                4 => {
                    sel.nodes.remove(&ids[0]);
                    csg_hollow(&mut m, &mut sel, 4.0).remove(&ids[1]).unwrap()
                }
                _ => {
                    sel.nodes.remove(&ids[1]);
                    csg_subtract(&mut m, &mut sel, csg::CarveMaterial::Cutter).1.remove(&ids[1]).unwrap()
                }
            };
            assert!(!result.is_empty() && named(&m, &result), "op {op}");
        }
    }

    #[test]
    fn join_keeps_the_first_mesh() {
        let (mut m, ids) = world_with_boxes();
        let mut sel = Selection::default();
        sel.nodes.extend([ids[0], ids[1]]);
        let meshes = convert_to_mesh(&mut m, &mut sel, false);
        m.mesh_mut(meshes[0]).unwrap().decal = true;
        sel.nodes.extend(meshes.iter().copied());
        let joined = join_meshes(&mut m, &mut sel).unwrap();
        assert_eq!(joined, meshes[0]);
        assert!(m.mesh(joined).unwrap().decal);
        assert!(!m.contains(meshes[1]));
        assert_eq!(m.mesh(joined).unwrap().faces.len(), 12);
    }

    #[test]
    fn duplicate_and_delete() {
        let (mut m, ids) = world_with_boxes();
        let mut sel = Selection::default();
        sel.nodes.insert(ids[2]);
        let new = duplicate_selection(&mut m, &mut sel, DVec3::new(16.0, 0.0, 0.0), EditOptions::default());
        assert_eq!(new.len(), 1);
        assert!((m.bounds(new[0]).min.x - 116.0).abs() < 1e-9);
        delete_selection(&mut m, &mut sel);
        assert_eq!(m.brush_count(), 3);
    }

    #[test]
    fn deleting_a_selected_layer_removes_it_and_its_contents() {
        let mut m = Map::new();
        let l2 = m.add_layer("Trees");
        let brush = create_brush(&mut m, l2, Brush::from_aabb(&Aabb::new(DVec3::splat(-8.0), DVec3::splat(8.0)), "t").unwrap());
        let mut sel = Selection::default();
        sel.nodes.insert(l2);
        delete_selection(&mut m, &mut sel);
        assert!(m.get(l2).is_none(), "the layer is gone");
        assert!(m.get(brush).is_none(), "its brush went with it");
        assert!(!m.layers.contains(&l2));
    }

    #[test]
    fn select_touching_finds_intersections() {
        let (mut m, ids) = world_with_boxes();
        let mut sel = Selection::default();
        sel.nodes.insert(ids[1]);
        select_touching(&mut m, &mut sel, &[], false);
        assert_eq!(sel.nodes.iter().copied().collect::<Vec<_>>(), vec![ids[0]]);
        assert!(!m.contains(ids[1]));
    }

    #[test]
    fn brush_entity_round_trip() {
        let (mut m, ids) = world_with_boxes();
        let layer = m.default_layer();
        let mut sel = Selection::default();
        sel.nodes.insert(ids[2]);
        let e = create_brush_entity(&mut m, &mut sel, "func_door", layer).unwrap();
        assert_eq!(m.owning_entity(ids[2]), Some(e));
        sel.nodes.clear();
        sel.nodes.insert(e);
        move_brushes_to_world(&mut m, &mut sel, layer);
        assert!(!m.contains(e));
        assert_eq!(m.owning_entity(ids[2]), None);
    }

    #[test]
    fn rotate_entity_angles() {
        let mut m = Map::new();
        let l = m.default_layer();
        let e = create_point_entity(&mut m, l, "light", DVec3::new(16.0, 0.0, 0.0));
        transform_nodes(&mut m, &[e], &rotation_about(DVec3::ZERO, DVec3::Y, 90.0), EditOptions::default());
        let ent = m.entity(e).unwrap();
        assert!(gt_core::vec_approx_eq(ent.origin, DVec3::new(0.0, 0.0, -16.0)));
        assert!((ent.angles.y - 90.0).abs() < 1e-6);
    }

    #[test]
    fn group_ungroup_and_isolate() {
        let (mut m, ids) = world_with_boxes();
        let layer = m.default_layer();
        let mut sel = Selection::default();
        sel.nodes.insert(ids[0]);
        sel.nodes.insert(ids[1]);
        let g = group_selection(&mut m, &mut sel, "g", layer).unwrap();
        assert_eq!(selectable_objects(&m, &[]), vec![ids[2], g]);
        isolate_selection(&mut m, &sel);
        assert!(m.is_hidden(ids[2]) && !m.is_hidden(ids[0]));
        unhide_all(&mut m);
        ungroup_selection(&mut m, &mut sel);
        assert!(!m.contains(g));
        assert_eq!(sel.nodes.len(), 2);
    }
}
