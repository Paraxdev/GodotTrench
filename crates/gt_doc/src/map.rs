use std::collections::BTreeMap;

use gt_core::{Aabb, Color, DMat4, DVec3, NodeId};
use gt_geom::{Brush, Mesh, Terrain};
use serde::{Deserialize, Serialize};

use crate::entity::Entity;
use crate::scatter::Scatter;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub color: Color,
    #[serde(default)]
    pub omit_from_export: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    /// Groups sharing a link id are kept identical up to their transforms (TrenchBroom linked groups).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_id: Option<u64>,
    /// Placement of a linked group relative to the others in its set.
    #[serde(default = "identity", skip_serializing_if = "is_identity")]
    pub transform: DMat4,
}

impl Group {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), link_id: None, transform: DMat4::IDENTITY }
    }
}

fn identity() -> DMat4 {
    DMat4::IDENTITY
}

fn is_identity(m: &DMat4) -> bool {
    *m == DMat4::IDENTITY
}

/// Saved viewpoint of the 3D view.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraBookmark {
    pub position: DVec3,
    pub yaw: f64,
    pub pitch: f64,
}

/// Editor state stored with the map but not exported.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EditorData {
    /// Slots 1 to 9.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub cameras: BTreeMap<u8, CameraBookmark>,
    /// Objects outside the cordon are hidden in the views and left out of cordoned exports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cordon: Option<Aabb>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub cordon_enabled: bool,
}

/// Reference to another map placed with a transform (Hammer func_instance / prefab).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    /// Path relative to the referencing map, or a res:// path.
    pub path: String,
    pub origin: DVec3,
    pub angles: DVec3,
    /// Prefix applied to targetnames inside the instance so several copies do not collide.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub fixup: String,
}

/// Entity keys that name other entities and so get an instance's fixup, besides any property an entity definition
/// declares as a target. Kept in step with `FIXUP_KEYS` in `gtm_parser.gd`.
pub const FIXUP_KEYS: [&str; 4] = ["targetname", "target", "destination", "call_target"];

impl Instance {
    pub fn transform(&self) -> DMat4 {
        let q = gt_core::DQuat::from_euler(gt_core::EulerRot::YXZ, self.angles.y.to_radians(), self.angles.x.to_radians(), self.angles.z.to_radians());
        DMat4::from_rotation_translation(q, self.origin)
    }

    /// `name` as it is inside this instance. Special (`!self`), group (`@doors`) and node path targets are left
    /// alone. The Godot importer applies the same rule in `gtm_parser.gd`.
    pub fn fixup_name(&self, name: &str) -> Option<String> {
        let special = name.is_empty() || name.starts_with(['!', '@', '/']);
        (!self.fixup.is_empty() && !special).then(|| format!("{}-{name}", self.fixup))
    }

    /// The fixup of an instance nested in this one, so exploding the outer then the inner names things the way a
    /// Godot build of the outer does.
    pub fn nested_fixup(&self, inner: &str) -> String {
        match (self.fixup.is_empty(), inner.is_empty()) {
            (true, _) => inner.to_string(),
            (false, true) => self.fixup.clone(),
            (false, false) => format!("{}-{inner}", self.fixup),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeKind {
    Layer(Layer),
    Group(Group),
    Entity(Entity),
    Brush(Brush),
    Instance(Instance),
    Mesh(Mesh),
    Terrain(Terrain),
    Scatter(Scatter),
}

impl NodeKind {
    pub fn type_name(&self) -> &'static str {
        match self {
            NodeKind::Layer(_) => "layer",
            NodeKind::Group(_) => "group",
            NodeKind::Entity(_) => "entity",
            NodeKind::Brush(_) => "brush",
            NodeKind::Instance(_) => "instance",
            NodeKind::Mesh(_) => "mesh",
            NodeKind::Terrain(_) => "terrain",
            NodeKind::Scatter(_) => "scatter",
        }
    }

    /// Layers, groups and scatter sets carry a name in their own data, other kinds use the node's label.
    pub fn has_own_name(&self) -> bool {
        matches!(self, NodeKind::Layer(_) | NodeKind::Group(_) | NodeKind::Scatter(_))
    }

    /// Geometry leaves that can live in the world or inside brush entities.
    pub fn is_geometry(&self) -> bool {
        matches!(self, NodeKind::Brush(_) | NodeKind::Mesh(_) | NodeKind::Terrain(_))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub kind: NodeKind,
    pub hidden: bool,
    pub locked: bool,
    /// Name given with Rename, shown in place of [`Node::default_name`]. Layers, groups and scatter sets keep their
    /// name in their own data instead.
    pub label: Option<String>,
}

impl Node {
    pub fn brush(&self) -> Option<&Brush> {
        match &self.kind {
            NodeKind::Brush(b) => Some(b),
            _ => None,
        }
    }

    pub fn entity(&self) -> Option<&Entity> {
        match &self.kind {
            NodeKind::Entity(e) => Some(e),
            _ => None,
        }
    }

    pub fn mesh(&self) -> Option<&Mesh> {
        match &self.kind {
            NodeKind::Mesh(m) => Some(m),
            _ => None,
        }
    }

    pub fn terrain(&self) -> Option<&Terrain> {
        match &self.kind {
            NodeKind::Terrain(t) => Some(t),
            _ => None,
        }
    }

    pub fn name(&self) -> String {
        match &self.label {
            Some(label) if !self.kind.has_own_name() => label.clone(),
            _ => self.default_name(),
        }
    }

    /// The name Rename starts from and stores: a layer's, group's or scatter set's own name without the extras
    /// [`Node::default_name`] adds, else the shown name.
    pub fn editable_name(&self) -> String {
        match &self.kind {
            NodeKind::Layer(Layer { name, .. }) | NodeKind::Group(Group { name, .. }) | NodeKind::Scatter(Scatter { name, .. }) => name.clone(),
            _ => self.name(),
        }
    }

    /// Sets the name given with Rename. A blank name, which only a hand edited file has, and a name on a kind that
    /// carries its own are dropped.
    pub fn set_label(&mut self, label: Option<String>) {
        self.label = label.filter(|l| !l.trim().is_empty() && !self.kind.has_own_name());
    }

    /// The name from the node's kind and content, used while it has no label.
    pub fn default_name(&self) -> String {
        match &self.kind {
            NodeKind::Layer(l) => l.name.clone(),
            NodeKind::Group(g) if g.link_id.is_some() => format!("{} (linked)", g.name),
            NodeKind::Group(g) => g.name.clone(),
            NodeKind::Entity(e) => match e.targetname() {
                Some(n) => format!("{} ({n})", e.classname),
                None => e.classname.clone(),
            },
            NodeKind::Brush(_) => format!("brush{}", self.id.0),
            NodeKind::Instance(i) => format!("instance {}", i.path),
            NodeKind::Mesh(_) => format!("mesh{}", self.id.0),
            NodeKind::Terrain(t) => format!("terrain {}x{}", t.resolution[0], t.resolution[1]),
            NodeKind::Scatter(s) => format!("scatter {} ({})", s.name, s.instances.len()),
        }
    }

    pub fn scatter(&self) -> Option<&Scatter> {
        match &self.kind {
            NodeKind::Scatter(s) => Some(s),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Map {
    pub nodes: imbl::OrdMap<NodeId, Node>,
    pub layers: Vec<NodeId>,
    /// Worldspawn key/values.
    pub properties: BTreeMap<String, String>,
    pub next_id: u64,
    pub editor: EditorData,
    /// Chunks of a newer editor read from the file, written back unchanged on save.
    pub unknown_chunks: Vec<crate::binary::RawChunk>,
    /// Bounds of the prefab behind each instance path, in the prefab's own space. The editor fills this from the
    /// prefab files so instance bounds match what renders. Not saved.
    pub instance_extents: std::sync::Arc<BTreeMap<String, Aabb>>,
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Map {
    pub fn new() -> Self {
        let mut map = Map {
            nodes: imbl::OrdMap::new(),
            layers: Vec::new(),
            properties: BTreeMap::new(),
            next_id: 1,
            editor: EditorData::default(),
            unknown_chunks: Vec::new(),
            instance_extents: Default::default(),
        };
        map.add_layer("Default");
        map
    }

    pub fn alloc_id(&mut self) -> NodeId {
        let id = NodeId(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn default_layer(&self) -> NodeId {
        self.layers[0]
    }

    pub fn add_layer(&mut self, name: &str) -> NodeId {
        let id = self.alloc_id();
        let color = Color::from_seed(id.0 + 7);
        self.nodes.insert(
            id,
            Node {
                id,
                parent: None,
                children: Vec::new(),
                kind: NodeKind::Layer(Layer { name: name.into(), color, omit_from_export: false }),
                hidden: false,
                locked: false,
                label: None,
            },
        );
        self.layers.push(id);
        id
    }

    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }

    pub fn brush(&self, id: NodeId) -> Option<&Brush> {
        self.get(id).and_then(|n| n.brush())
    }

    pub fn brush_mut(&mut self, id: NodeId) -> Option<&mut Brush> {
        match &mut self.get_mut(id)?.kind {
            NodeKind::Brush(b) => Some(b),
            _ => None,
        }
    }

    pub fn entity(&self, id: NodeId) -> Option<&Entity> {
        self.get(id).and_then(|n| n.entity())
    }

    pub fn entity_mut(&mut self, id: NodeId) -> Option<&mut Entity> {
        match &mut self.get_mut(id)?.kind {
            NodeKind::Entity(e) => Some(e),
            _ => None,
        }
    }

    pub fn mesh(&self, id: NodeId) -> Option<&Mesh> {
        self.get(id).and_then(|n| n.mesh())
    }

    pub fn mesh_mut(&mut self, id: NodeId) -> Option<&mut Mesh> {
        match &mut self.get_mut(id)?.kind {
            NodeKind::Mesh(m) => Some(m),
            _ => None,
        }
    }

    pub fn terrain(&self, id: NodeId) -> Option<&Terrain> {
        self.get(id).and_then(|n| n.terrain())
    }

    pub fn terrain_mut(&mut self, id: NodeId) -> Option<&mut Terrain> {
        match &mut self.get_mut(id)?.kind {
            NodeKind::Terrain(t) => Some(t),
            _ => None,
        }
    }

    pub fn scatter(&self, id: NodeId) -> Option<&Scatter> {
        self.get(id).and_then(|n| n.scatter())
    }

    pub fn scatter_mut(&mut self, id: NodeId) -> Option<&mut Scatter> {
        match &mut self.get_mut(id)?.kind {
            NodeKind::Scatter(s) => Some(s),
            _ => None,
        }
    }

    pub fn scatters(&self) -> impl Iterator<Item = (NodeId, &Scatter)> {
        self.nodes.iter().filter_map(|(id, n)| n.scatter().map(|s| (*id, s)))
    }

    pub fn meshes(&self) -> impl Iterator<Item = (NodeId, &Mesh)> {
        self.nodes.iter().filter_map(|(id, n)| n.mesh().map(|m| (*id, m)))
    }

    pub fn terrains(&self) -> impl Iterator<Item = (NodeId, &Terrain)> {
        self.nodes.iter().filter_map(|(id, n)| n.terrain().map(|t| (*id, t)))
    }

    /// Whether the node lies inside the enabled cordon (always true without one).
    pub fn in_cordon(&self, id: NodeId) -> bool {
        match (self.editor.cordon_enabled, self.editor.cordon) {
            (true, Some(c)) => {
                let b = self.bounds(id);
                b.is_empty() || c.intersects(&b)
            }
            _ => true,
        }
    }

    pub fn insert(&mut self, parent: NodeId, kind: NodeKind) -> NodeId {
        let id = self.alloc_id();
        self.insert_with_id(id, parent, kind);
        id
    }

    /// Inserts a node that replaces others, like a clip piece or a merged brush, keeping the name they were given.
    pub fn insert_labeled(&mut self, parent: NodeId, kind: NodeKind, label: Option<String>) -> NodeId {
        let id = self.insert(parent, kind);
        if let Some(n) = self.get_mut(id) {
            n.set_label(label);
        }

        id
    }

    /// Refuses, returning false, when `parent` does not exist, since an orphan would render but never be saved.
    pub fn insert_with_id(&mut self, id: NodeId, parent: NodeId, kind: NodeKind) -> bool {
        let Some(p) = self.nodes.get_mut(&parent) else { return false };
        p.children.push(id);
        self.next_id = self.next_id.max(id.0 + 1);
        self.nodes.insert(id, Node { id, parent: Some(parent), children: Vec::new(), kind, hidden: false, locked: false, label: None });
        true
    }

    /// Removes the node and all descendants. Empty layers are kept, layers themselves are removed only if not the last.
    pub fn remove(&mut self, id: NodeId) {
        let Some(node) = self.nodes.get(&id).cloned() else { return };
        if matches!(node.kind, NodeKind::Layer(_)) && self.layers.len() <= 1 {
            return;
        }

        for child in node.children.clone() {
            self.remove_subtree(child);
        }

        self.nodes.remove(&id);
        match node.parent {
            Some(p) => {
                if let Some(parent) = self.nodes.get_mut(&p) {
                    parent.children.retain(|c| *c != id);
                }
            }
            None => self.layers.retain(|l| *l != id),
        }
    }

    fn remove_subtree(&mut self, id: NodeId) {
        if let Some(node) = self.nodes.remove(&id) {
            for c in node.children {
                self.remove_subtree(c);
            }
        }
    }

    pub fn reparent(&mut self, id: NodeId, new_parent: NodeId) {
        if id == new_parent || self.is_ancestor(id, new_parent) {
            return;
        }

        let Some(old) = self.get(id).and_then(|n| n.parent) else { return };
        if old == new_parent {
            return;
        }

        if let Some(p) = self.nodes.get_mut(&old) {
            p.children.retain(|c| *c != id);
        }

        if let Some(p) = self.nodes.get_mut(&new_parent) {
            p.children.push(id);
        }

        if let Some(n) = self.nodes.get_mut(&id) {
            n.parent = Some(new_parent);
        }
    }

    pub fn is_ancestor(&self, ancestor: NodeId, mut node: NodeId) -> bool {
        while let Some(p) = self.get(node).and_then(|n| n.parent) {
            if p == ancestor {
                return true;
            }

            node = p;
        }

        false
    }

    pub fn ancestors(&self, id: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut cur = id;
        while let Some(p) = self.get(cur).and_then(|n| n.parent) {
            out.push(p);
            cur = p;
        }

        out
    }

    pub fn layer_of(&self, id: NodeId) -> NodeId {
        let mut cur = id;
        while let Some(p) = self.get(cur).and_then(|n| n.parent) {
            cur = p;
        }

        cur
    }

    /// Hidden itself or through any ancestor.
    pub fn is_hidden(&self, id: NodeId) -> bool {
        let mut cur = Some(id);
        while let Some(c) = cur {
            match self.get(c) {
                Some(n) if n.hidden => return true,
                Some(n) => cur = n.parent,
                None => return false,
            }
        }

        false
    }

    pub fn is_locked(&self, id: NodeId) -> bool {
        let mut cur = Some(id);
        while let Some(c) = cur {
            match self.get(c) {
                Some(n) if n.locked => return true,
                Some(n) => cur = n.parent,
                None => return false,
            }
        }

        false
    }

    pub fn is_editable(&self, id: NodeId) -> bool {
        !self.is_hidden(id) && !self.is_locked(id)
    }

    /// Depth-first descendants, not including `id`.
    pub fn descendants(&self, id: NodeId) -> Vec<NodeId> {
        let mut out = Vec::new();
        let mut stack: Vec<NodeId> = self.get(id).map(|n| n.children.iter().rev().copied().collect()).unwrap_or_default();
        while let Some(c) = stack.pop() {
            out.push(c);
            if let Some(n) = self.get(c) {
                stack.extend(n.children.iter().rev());
            }
        }

        out
    }

    /// All nodes in tree order.
    pub fn walk(&self) -> Vec<NodeId> {
        let mut out = Vec::with_capacity(self.nodes.len());
        for l in &self.layers {
            out.push(*l);
            out.extend(self.descendants(*l));
        }

        out
    }

    pub fn brushes(&self) -> impl Iterator<Item = (NodeId, &Brush)> {
        self.nodes.iter().filter_map(|(id, n)| n.brush().map(|b| (*id, b)))
    }

    pub fn entities(&self) -> impl Iterator<Item = (NodeId, &Entity)> {
        self.nodes.iter().filter_map(|(id, n)| n.entity().map(|e| (*id, e)))
    }

    /// Entity that owns this brush, if it is not a world brush.
    pub fn owning_entity(&self, brush: NodeId) -> Option<NodeId> {
        let p = self.get(brush)?.parent?;
        self.entity(p).map(|_| p)
    }

    pub fn is_point_entity(&self, id: NodeId) -> bool {
        self.get(id).is_some_and(|n| matches!(n.kind, NodeKind::Entity(_)) && n.children.is_empty())
    }

    pub fn bounds(&self, id: NodeId) -> Aabb {
        let Some(node) = self.get(id) else { return Aabb::EMPTY };
        match &node.kind {
            NodeKind::Brush(b) => b.bounds(),
            NodeKind::Mesh(m) => m.bounds(),
            NodeKind::Terrain(t) => t.bounds(),
            NodeKind::Scatter(s) => s.bounds(),
            NodeKind::Entity(e) if node.children.is_empty() => Aabb::from_center_size(e.origin, DVec3::splat(16.0)),
            NodeKind::Instance(i) => match self.instance_extents.get(&i.path).filter(|b| !b.is_empty()) {
                Some(b) => {
                    let m = i.transform();
                    Aabb::from_points(b.corners().iter().map(|c| m.transform_point3(*c)))
                }
                None => Aabb::from_center_size(i.origin, DVec3::splat(16.0)),
            },
            _ => {
                let mut b = Aabb::EMPTY;
                for c in &node.children {
                    b.include(&self.bounds(*c));
                }

                b
            }
        }
    }

    pub fn bounds_of(&self, ids: impl IntoIterator<Item = NodeId>) -> Aabb {
        let mut b = Aabb::EMPTY;
        for id in ids {
            b.include(&self.bounds(id));
        }

        b
    }

    /// Nearest ancestor group that is closed, which is what a click in the viewport selects.
    pub fn selection_target(&self, id: NodeId, open_groups: &[NodeId]) -> NodeId {
        let mut target = id;
        for a in self.ancestors(id) {
            if let Some(Node { kind: NodeKind::Group(_), .. }) = self.get(a)
                && !open_groups.contains(&a)
            {
                target = a;
            }
        }

        target
    }

    /// What a click in a view selects: the clicked object itself, or the outermost closed linked group around it, because
    /// linked groups are edited as a whole. [`Self::selection_target`] gives the whole group instead.
    pub fn click_target(&self, id: NodeId, open_groups: &[NodeId]) -> NodeId {
        let mut target = id;
        for a in self.ancestors(id) {
            if let Some(Node { kind: NodeKind::Group(g), .. }) = self.get(a)
                && g.link_id.is_some()
                && !open_groups.contains(&a)
            {
                target = a;
            }
        }

        target
    }

    /// Names a node. Layers, groups and scatter sets take it as their own name and ignore an empty one, other nodes get
    /// it as their label, and an empty or default name takes the label off again. A scatter set's layer named after it,
    /// as the Scatter tool makes one, is renamed along. Returns false when nothing changed.
    pub fn rename(&mut self, id: NodeId, name: &str) -> bool {
        let name = name.trim();
        let layer = self.layer_of(id);
        let Some(node) = self.get_mut(id) else { return false };
        let default = node.default_name();
        let scatter = matches!(node.kind, NodeKind::Scatter(_));
        let slot = match &mut node.kind {
            NodeKind::Layer(Layer { name: own, .. }) | NodeKind::Group(Group { name: own, .. }) | NodeKind::Scatter(Scatter { name: own, .. }) => Some(own),
            _ => None,
        };
        let old = match slot {
            Some(own) if name.is_empty() || own == name => return false,
            Some(own) => std::mem::replace(own, name.to_string()),
            None => {
                let label = (!name.is_empty() && name != default).then(|| name.to_string());
                let changed = node.label != label;
                node.label = label;
                return changed;
            }
        };
        if scatter
            && let Some(NodeKind::Layer(l)) = self.get_mut(layer).map(|n| &mut n.kind)
            && l.name == format!("Scatter: {old}")
        {
            l.name = format!("Scatter: {name}");
        }

        true
    }

    /// Deep copies a subtree under a new parent with fresh ids. Returns the new root id.
    pub fn duplicate_subtree(&mut self, id: NodeId, parent: NodeId) -> Option<NodeId> {
        let mut copies = BTreeMap::new();
        let new_id = self.duplicate_subtree_into(id, parent, &mut copies);
        self.retarget_scatter_copies(&copies);
        new_id
    }

    /// [`Map::duplicate_subtree`] that records each original id and its copy, for copying several subtrees whose
    /// scatter sets may target each other. Finish with [`Map::retarget_scatter_copies`].
    pub fn duplicate_subtree_into(&mut self, id: NodeId, parent: NodeId, copies: &mut BTreeMap<NodeId, NodeId>) -> Option<NodeId> {
        let node = self.get(id)?.clone();
        let new_id = self.insert(parent, node.kind.clone());
        copies.insert(id, new_id);
        if let Some(n) = self.get_mut(new_id) {
            n.hidden = node.hidden;
            n.locked = node.locked;
            n.label = node.label;
        }

        for c in node.children {
            self.duplicate_subtree_into(c, new_id, copies);
        }

        Some(new_id)
    }

    /// Copied scatter sets paint on the copies of their target surfaces when those were copied with them, and keep
    /// the original surfaces otherwise.
    pub fn retarget_scatter_copies(&mut self, copies: &BTreeMap<NodeId, NodeId>) {
        for copy in copies.values() {
            if let Some(s) = self.scatter_mut(*copy) {
                for t in &mut s.targets {
                    if let Some(new) = copies.get(t) {
                        *t = *new;
                    }
                }
            }
        }
    }

    pub fn brush_count(&self) -> usize {
        self.brushes().count()
    }

    pub fn entity_count(&self) -> usize {
        self.entities().count()
    }

    pub fn find_by_targetname(&self, name: &str) -> Vec<NodeId> {
        self.entities().filter(|(_, e)| e.targetname() == Some(name)).map(|(id, _)| id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_geom::Brush;

    #[test]
    fn rename_starts_from_the_bare_name() {
        let mut m = Map::new();
        let layer = m.default_layer();
        let room = m.insert(layer, NodeKind::Group(Group { link_id: Some(2), ..Group::new("room") }));
        let grass = m.insert(layer, NodeKind::Scatter(Scatter::new("grass", crate::scatter::ScatterKind::Foliage, Vec::new())));
        for id in [room, grass] {
            let node = m.get(id).unwrap();
            assert_ne!(node.name(), node.editable_name());
            let bare = node.editable_name();
            assert!(!m.rename(id, &bare), "confirming the field unchanged changes nothing");
        }

        assert_eq!(m.get(room).unwrap().name(), "room (linked)");
    }

    #[test]
    fn renaming_a_scatter_set_renames_its_layer() {
        let mut m = Map::new();
        let layer = m.add_layer("Scatter: grass");
        let grass = m.insert(layer, NodeKind::Scatter(Scatter::new("grass", crate::scatter::ScatterKind::Foliage, Vec::new())));
        assert!(m.rename(grass, " meadow "));
        assert_eq!(m.get(layer).unwrap().name(), "Scatter: meadow");
        assert!(!m.rename(grass, ""), "a set keeps its name");

        m.rename(layer, "Field");
        assert!(m.rename(grass, "lawn"));
        assert_eq!(m.get(layer).unwrap().name(), "Field", "a layer named by hand keeps its name");
    }

    #[test]
    fn instance_fixups() {
        let inst = |fixup: &str| Instance { path: "p.gtm".into(), origin: DVec3::ZERO, angles: DVec3::ZERO, fixup: fixup.into() };
        let a = inst("a");
        let names: Vec<Option<String>> = ["door", "door*", "@doors", "!self", "/root/Game", ""].iter().map(|n| a.fixup_name(n)).collect();
        assert_eq!(names, [Some("a-door".into()), Some("a-door*".into()), None, None, None, None]);
        assert_eq!(inst("").fixup_name("door"), None);
        assert_eq!(a.nested_fixup("b"), "a-b");
        assert_eq!(a.nested_fixup(""), "a");
        assert_eq!(inst("").nested_fixup("b"), "b");
    }

    #[test]
    fn insert_remove_subtree() {
        let mut m = Map::new();
        let layer = m.default_layer();
        let g = m.insert(layer, NodeKind::Group(Group::new("g")));
        let e = m.insert(g, NodeKind::Entity(Entity::new("func_door")));
        let b = Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::ONE), "x").unwrap();
        let bid = m.insert(e, NodeKind::Brush(b));
        assert_eq!(m.owning_entity(bid), Some(e));
        assert_eq!(m.layer_of(bid), layer);
        m.get_mut(g).unwrap().hidden = true;
        assert!(m.is_hidden(bid));
        m.remove(g);
        assert!(!m.contains(bid));
        assert!(m.get(layer).unwrap().children.is_empty());
    }

    #[test]
    fn insert_refuses_a_missing_parent() {
        let mut m = Map::new();
        assert!(!m.insert_with_id(NodeId(50), NodeId(40), NodeKind::Group(Group::new("g"))));
        assert!(!m.contains(NodeId(50)));
        let id = m.insert(NodeId(40), NodeKind::Group(Group::new("g")));
        assert!(!m.contains(id));
    }

    #[test]
    fn reparent_rejects_cycles() {
        let mut m = Map::new();
        let layer = m.default_layer();
        let a = m.insert(layer, NodeKind::Group(Group::new("a")));
        let b = m.insert(a, NodeKind::Group(Group::new("b")));
        m.reparent(a, b);
        assert_eq!(m.get(a).unwrap().parent, Some(layer));
    }

    #[test]
    fn clicks_pick_objects_inside_groups_but_keep_linked_groups_whole() {
        let mut m = Map::new();
        let layer = m.default_layer();
        let brush = || NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::ONE), "x").unwrap());
        let house = m.insert(layer, NodeKind::Group(Group::new("house")));
        let chair = m.insert(house, NodeKind::Group(Group::new("chair")));
        let leg = m.insert(chair, brush());
        assert_eq!(m.click_target(leg, &[]), leg);
        assert_eq!(m.selection_target(leg, &[]), house);

        let window = m.insert(house, NodeKind::Group(Group { link_id: Some(7), ..Group::new("window") }));
        let pane = m.insert(window, brush());
        assert_eq!(m.click_target(pane, &[]), window);
        assert_eq!(m.click_target(pane, &[window]), pane, "an opened linked group edits its contents");
    }
}
