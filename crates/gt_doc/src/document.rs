use std::path::PathBuf;

use crate::map::Map;
use crate::selection::Selection;

#[derive(Clone)]
struct Snapshot {
    label: String,
    map: Map,
    selection: Selection,
}

/// Undo history of whole-map snapshots. Cheap because `Map::nodes` is a persistent map with structural sharing.
#[derive(Default)]
pub struct History {
    undo: Vec<Snapshot>,
    redo: Vec<Snapshot>,
    pub limit: usize,
}

impl History {
    pub fn undo_labels(&self) -> impl Iterator<Item = &str> {
        self.undo.iter().rev().map(|s| s.label.as_str())
    }

    pub fn redo_labels(&self) -> impl Iterator<Item = &str> {
        self.redo.iter().rev().map(|s| s.label.as_str())
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

pub struct Document {
    pub map: Map,
    pub selection: Selection,
    pub path: Option<PathBuf>,
    /// The autosave this document was loaded from, until it is saved to `path`.
    pub recovered_from: Option<PathBuf>,
    pub history: History,
    /// Incremented on every change, used by renderers and caches.
    pub revision: u64,
    saved_revision: u64,
    transaction: Option<Snapshot>,
    last_command: Option<String>,
    last_edit: Option<(String, std::time::Instant)>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    pub fn new() -> Self {
        Self::from_map(Map::new(), None)
    }

    pub fn from_map(map: Map, path: Option<PathBuf>) -> Self {
        Self {
            map,
            selection: Selection::default(),
            path,
            recovered_from: None,
            history: History { limit: 512, ..Default::default() },
            revision: 1,
            saved_revision: 1,
            transaction: None,
            last_command: None,
            last_edit: None,
        }
    }

    pub fn is_modified(&self) -> bool {
        self.revision != self.saved_revision
    }

    pub fn mark_saved(&mut self) {
        self.saved_revision = self.revision;
        self.recovered_from = None;
    }

    pub fn mark_unsaved(&mut self) {
        self.saved_revision = 0;
    }

    pub fn title(&self) -> String {
        let name = self.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "untitled.gtm".into());
        let name = if self.is_modified() { format!("{name}*") } else { name };
        if self.recovered_from.is_some() { format!("{name} (recovered)") } else { name }
    }

    fn snapshot(&self, label: &str) -> Snapshot {
        Snapshot { label: label.into(), map: self.map.clone(), selection: self.selection.clone() }
    }

    fn push_undo(&mut self, snap: Snapshot) {
        self.history.undo.push(snap);
        if self.history.undo.len() > self.history.limit {
            self.history.undo.remove(0);
        }
        self.history.redo.clear();
    }

    /// Runs an undoable edit. Returns whatever the closure returns.
    pub fn edit<R>(&mut self, label: &str, f: impl FnOnce(&mut Map, &mut Selection) -> R) -> R {
        if self.transaction.is_some() {
            let r = f(&mut self.map, &mut self.selection);
            self.revision += 1;
            return r;
        }
        let snap = self.snapshot(label);
        let r = f(&mut self.map, &mut self.selection);
        crate::linked::sync(&snap.map, &mut self.map);
        self.selection.prune(&self.map);
        self.push_undo(snap);
        self.revision += 1;
        self.last_command = Some(label.into());
        r
    }

    /// Like `edit`, but consecutive edits with the same label within a short time form one undo step.
    /// Used for continuous inputs such as dragging a number field.
    pub fn edit_coalesced<R>(&mut self, label: &str, f: impl FnOnce(&mut Map, &mut Selection) -> R) -> R {
        let now = std::time::Instant::now();
        let recent = self.last_edit.as_ref().is_some_and(|(l, t)| l == label && now.duration_since(*t).as_secs_f32() < 1.0);
        if recent && self.transaction.is_none() && self.history.redo.is_empty() && !self.history.undo.is_empty() {
            let r = f(&mut self.map, &mut self.selection);
            self.selection.prune(&self.map);
            self.revision += 1;
            self.last_edit = Some((label.into(), now));
            return r;
        }
        let r = self.edit(label, f);
        self.last_edit = Some((label.into(), now));
        r
    }

    /// Selection changes are not recorded in history, but still bump the revision.
    pub fn select(&mut self, f: impl FnOnce(&Map, &mut Selection)) {
        f(&self.map, &mut self.selection);
        self.selection.prune(&self.map);
        let was_saved = !self.is_modified();
        self.revision += 1;
        if was_saved {
            self.saved_revision = self.revision;
        }
    }

    /// Starts an interactive edit (drag). Intermediate changes go straight to the map.
    pub fn begin(&mut self, label: &str) {
        if self.transaction.is_none() {
            self.transaction = Some(self.snapshot(label));
        }
    }

    pub fn in_transaction(&self) -> bool {
        self.transaction.is_some()
    }

    /// Restores the map to the state at `begin`, keeping the transaction open.
    pub fn reset_transaction(&mut self) {
        if let Some(t) = &self.transaction {
            self.map = t.map.clone();
            self.selection = t.selection.clone();
            self.revision += 1;
        }
    }

    pub fn transaction_base(&self) -> Option<&Map> {
        self.transaction.as_ref().map(|t| &t.map)
    }

    pub fn commit(&mut self) {
        if let Some(snap) = self.transaction.take() {
            crate::linked::sync(&snap.map, &mut self.map);
            self.selection.prune(&self.map);
            if snap.map.nodes != self.map.nodes || snap.map.properties != self.map.properties || snap.map.layers != self.map.layers {
                self.last_command = Some(snap.label.clone());
                self.push_undo(snap);
                self.revision += 1;
            }
        }
    }

    pub fn cancel(&mut self) {
        if let Some(snap) = self.transaction.take() {
            self.map = snap.map;
            self.selection = snap.selection;
            self.revision += 1;
        }
    }

    pub fn undo(&mut self) -> Option<String> {
        self.commit();
        let snap = self.history.undo.pop()?;
        let label = snap.label.clone();
        let current = Snapshot {
            label: snap.label.clone(),
            map: std::mem::replace(&mut self.map, snap.map),
            selection: std::mem::replace(&mut self.selection, snap.selection),
        };
        self.history.redo.push(current);
        self.revision += 1;
        Some(label)
    }

    pub fn redo(&mut self) -> Option<String> {
        let snap = self.history.redo.pop()?;
        let label = snap.label.clone();
        let current = Snapshot {
            label: snap.label.clone(),
            map: std::mem::replace(&mut self.map, snap.map),
            selection: std::mem::replace(&mut self.selection, snap.selection),
        };
        self.history.undo.push(current);
        self.revision += 1;
        Some(label)
    }

    pub fn last_command(&self) -> Option<&str> {
        self.last_command.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Entity, NodeKind};

    #[test]
    fn undo_redo_restores_state() {
        let mut doc = Document::new();
        let layer = doc.map.default_layer();
        let id = doc.edit("add", |m, s| {
            let id = m.insert(layer, NodeKind::Entity(Entity::new("light")));
            s.select_node(id);
            id
        });
        assert!(doc.map.contains(id));
        doc.undo();
        assert!(!doc.map.contains(id));
        assert!(doc.selection.is_empty());
        doc.redo();
        assert!(doc.map.contains(id));
        assert!(doc.selection.nodes.contains(&id));
    }

    #[test]
    fn transaction_is_single_step() {
        let mut doc = Document::new();
        let layer = doc.map.default_layer();
        let id = doc.edit("add", |m, _| m.insert(layer, NodeKind::Entity(Entity::new("light"))));
        doc.begin("move");
        for i in 0..10 {
            doc.edit("drag", |m, _| m.entity_mut(id).unwrap().origin.x = i as f64);
        }
        doc.commit();
        assert_eq!(doc.map.entity(id).unwrap().origin.x, 9.0);
        doc.undo();
        assert_eq!(doc.map.entity(id).unwrap().origin.x, 0.0);
    }
}
