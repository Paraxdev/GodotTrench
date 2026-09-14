use std::collections::BTreeSet;

use gt_core::NodeId;

use crate::map::Map;

/// Either objects or faces are selected, never both (TrenchBroom semantics).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection {
    pub nodes: BTreeSet<NodeId>,
    pub faces: BTreeSet<(NodeId, usize)>,
}

impl Selection {
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.faces.is_empty()
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.faces.clear();
    }

    pub fn select_node(&mut self, id: NodeId) {
        self.faces.clear();
        self.nodes.insert(id);
    }

    pub fn toggle_node(&mut self, id: NodeId) {
        self.faces.clear();
        if !self.nodes.remove(&id) {
            self.nodes.insert(id);
        }
    }

    pub fn select_face(&mut self, brush: NodeId, face: usize) {
        self.nodes.clear();
        self.faces.insert((brush, face));
    }

    pub fn toggle_face(&mut self, brush: NodeId, face: usize) {
        self.nodes.clear();
        if !self.faces.remove(&(brush, face)) {
            self.faces.insert((brush, face));
        }
    }

    pub fn has_faces(&self) -> bool {
        !self.faces.is_empty()
    }

    /// Drops ids that no longer exist or no longer point at valid faces.
    pub fn prune(&mut self, map: &Map) {
        self.nodes.retain(|id| map.contains(*id));
        self.faces.retain(|(id, f)| map.brush(*id).map(|b| b.faces.len()).or_else(|| map.mesh(*id).map(|m| m.faces.len())).is_some_and(|n| *f < n));
    }

    /// Selected nodes expanded to all brushes they contain (groups and brush entities recurse).
    pub fn brushes(&self, map: &Map) -> Vec<NodeId> {
        let mut out = BTreeSet::new();
        for id in &self.nodes {
            if map.brush(*id).is_some() {
                out.insert(*id);
            }
            for d in map.descendants(*id) {
                if map.brush(d).is_some() {
                    out.insert(d);
                }
            }
        }
        out.into_iter().collect()
    }

    /// Selected nodes expanded to every leaf that can be transformed: brushes, meshes, terrains, point entities and instances.
    pub fn transformables(&self, map: &Map) -> Vec<NodeId> {
        let mut out = BTreeSet::new();
        for id in &self.nodes {
            let mut consider = vec![*id];
            consider.extend(map.descendants(*id));
            for c in consider {
                let Some(n) = map.get(c) else { continue };
                match &n.kind {
                    crate::NodeKind::Layer(_) | crate::NodeKind::Group(_) => {}
                    _ => {
                        out.insert(c);
                    }
                }
            }
        }
        out.into_iter().collect()
    }

    /// Selected nodes expanded to the meshes they contain.
    pub fn meshes(&self, map: &Map) -> Vec<NodeId> {
        self.expand(map, |n| matches!(n.kind, crate::NodeKind::Mesh(_)))
    }

    pub fn terrains(&self, map: &Map) -> Vec<NodeId> {
        self.expand(map, |n| matches!(n.kind, crate::NodeKind::Terrain(_)))
    }

    /// Brushes, meshes and terrains.
    pub fn geometry(&self, map: &Map) -> Vec<NodeId> {
        self.expand(map, |n| n.kind.is_geometry())
    }

    fn expand(&self, map: &Map, keep: impl Fn(&crate::map::Node) -> bool) -> Vec<NodeId> {
        let mut out = BTreeSet::new();
        for id in &self.nodes {
            for c in std::iter::once(*id).chain(map.descendants(*id)) {
                if map.get(c).is_some_and(&keep) {
                    out.insert(c);
                }
            }
        }
        out.into_iter().collect()
    }
}
