use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use crate::map::Map;
use crate::selection::Selection;

#[derive(Clone)]
struct Snapshot {
    label: String,
    map: Map,
    selection: Selection,
    /// Content state id of `map`, see `Document::state`.
    state: u64,
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

    /// The map as it was `steps` undo steps ago, 1 being before the latest step.
    pub fn map_before(&self, steps: usize) -> Option<&Map> {
        self.undo.iter().rev().nth(steps.checked_sub(1)?).map(|s| &s.map)
    }
}

/// Where the undo history stood, see [`Document::squash_since`].
#[derive(Clone, Copy, Debug)]
pub struct HistoryMark {
    doc: u64,
    top: u64,
    rewinds: u64,
}

static NEXT_DOCUMENT: AtomicU64 = AtomicU64::new(1);

pub struct Document {
    pub map: Map,
    pub selection: Selection,
    pub path: Option<PathBuf>,
    /// The autosave this document was loaded from, until it is saved to `path`.
    pub recovered_from: Option<PathBuf>,
    /// What reading a damaged file lost or moved, until the map is saved.
    pub load_problems: Vec<String>,
    pub history: History,
    /// Incremented on every change, used by renderers and caches. Code that changes `map` directly bumps it, which
    /// also marks the document modified.
    pub revision: u64,
    /// Identifies the map content: every change gets a fresh id and undo or redo bring back the id of the state they
    /// return to, so going back to the saved state reads as unmodified.
    state: u64,
    next_state: u64,
    saved_state: Option<u64>,
    /// `revision` as of the last change made through this type, a newer revision means `map` was changed directly.
    accounted_revision: u64,
    transaction: Option<Snapshot>,
    last_command: Option<String>,
    last_edit: Option<(String, Instant)>,
    /// The map as last saved or opened.
    saved_map: Map,
    uid: u64,
    /// Undo steps dropped off the bottom of the history, and undo or redo calls, both for [`HistoryMark`].
    evicted: u64,
    rewinds: u64,
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
            saved_map: map.clone(),
            map,
            selection: Selection::default(),
            path,
            recovered_from: None,
            load_problems: Vec::new(),
            history: History { limit: 512, ..Default::default() },
            revision: 1,
            state: 1,
            next_state: 2,
            saved_state: Some(1),
            accounted_revision: 1,
            transaction: None,
            last_command: None,
            last_edit: None,
            uid: NEXT_DOCUMENT.fetch_add(1, Ordering::Relaxed),
            evicted: 0,
            rewinds: 0,
        }
    }

    pub fn is_modified(&self) -> bool {
        self.revision != self.accounted_revision || self.saved_state != Some(self.state)
    }

    pub fn mark_saved(&mut self) {
        self.absorb_direct_changes();
        self.saved_state = Some(self.state);
        self.saved_map = self.map.clone();
        self.recovered_from = None;
        self.load_problems.clear();
        // The saved state has to stay reachable by undo, so the next coalesced edit starts its own step.
        self.last_edit = None;
    }

    pub fn mark_unsaved(&mut self) {
        self.saved_state = None;
    }

    pub fn saved_map(&self) -> &Map {
        &self.saved_map
    }

    pub fn title(&self) -> String {
        let name = self.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "untitled.gtm".into());
        let name = if self.is_modified() { format!("{name}*") } else { name };
        if self.recovered_from.is_some() { format!("{name} (recovered)") } else { name }
    }

    /// A revision bumped from outside turns into a content state of its own.
    fn absorb_direct_changes(&mut self) {
        if self.revision != self.accounted_revision {
            self.new_state();
            self.accounted_revision = self.revision;
        }
    }

    fn new_state(&mut self) {
        self.state = self.next_state;
        self.next_state += 1;
    }

    fn bump(&mut self) {
        self.revision += 1;
        self.accounted_revision = self.revision;
    }

    fn changed(&mut self) {
        self.new_state();
        self.bump();
    }

    fn snapshot(&self, label: &str) -> Snapshot {
        Snapshot { label: label.into(), map: self.map.clone(), selection: self.selection.clone(), state: self.state }
    }

    fn push_undo(&mut self, snap: Snapshot) {
        self.history.undo.push(snap);
        if self.history.undo.len() > self.history.limit {
            self.history.undo.remove(0);
            self.evicted += 1;
        }

        self.history.redo.clear();
    }

    /// Runs an undoable edit. Returns whatever the closure returns.
    pub fn edit<R>(&mut self, label: &str, f: impl FnOnce(&mut Map, &mut Selection) -> R) -> R {
        self.absorb_direct_changes();
        self.last_edit = None;
        if self.transaction.is_some() {
            let r = f(&mut self.map, &mut self.selection);
            self.changed();
            return r;
        }

        let snap = self.snapshot(label);
        let r = f(&mut self.map, &mut self.selection);
        crate::linked::sync(&snap.map, &mut self.map);
        self.selection.prune(&self.map);
        self.push_undo(snap);
        self.changed();
        self.last_command = Some(label.into());
        r
    }

    /// Like `edit`, but an `Err` restores the map and selection and records nothing: no undo step,
    /// no revision bump, and the redo stack is kept.
    pub fn try_edit<T, E>(&mut self, label: &str, f: impl FnOnce(&mut Map, &mut Selection) -> Result<T, E>) -> Result<T, E> {
        self.absorb_direct_changes();
        if self.transaction.is_some() {
            let before = self.snapshot(label);
            let r = f(&mut self.map, &mut self.selection);
            match &r {
                Ok(_) => {
                    self.last_edit = None;
                    self.changed();
                }
                Err(_) => {
                    self.map = before.map;
                    self.selection = before.selection;
                }
            }

            return r;
        }

        let snap = self.snapshot(label);
        match f(&mut self.map, &mut self.selection) {
            Ok(v) => {
                crate::linked::sync(&snap.map, &mut self.map);
                self.selection.prune(&self.map);
                self.push_undo(snap);
                self.changed();
                self.last_edit = None;
                self.last_command = Some(label.into());
                Ok(v)
            }
            Err(e) => {
                self.map = snap.map;
                self.selection = snap.selection;
                Err(e)
            }
        }
    }

    /// Like `edit`, but consecutive edits with the same label within a short time form one undo step.
    /// Used for continuous inputs such as dragging a number field.
    pub fn edit_coalesced<R>(&mut self, label: &str, f: impl FnOnce(&mut Map, &mut Selection) -> R) -> R {
        let now = Instant::now();
        self.absorb_direct_changes();
        let recent = self.last_edit.as_ref().is_some_and(|(l, t)| l == label && now.duration_since(*t).as_secs_f32() < 1.0);
        if recent && self.transaction.is_none() && self.history.redo.is_empty() && !self.history.undo.is_empty() {
            let before = self.map.clone();
            let r = f(&mut self.map, &mut self.selection);
            crate::linked::sync(&before, &mut self.map);
            self.selection.prune(&self.map);
            self.changed();
            self.last_edit = Some((label.into(), now));
            return r;
        }

        let r = self.edit(label, f);
        self.last_edit = Some((label.into(), now));
        r
    }

    /// Selection changes are not recorded in history and leave the modified flag alone, but still bump the revision.
    pub fn select(&mut self, f: impl FnOnce(&Map, &mut Selection)) {
        self.absorb_direct_changes();
        f(&self.map, &mut self.selection);
        self.selection.prune(&self.map);
        self.bump();
    }

    /// Starts an interactive edit (drag). Intermediate changes go straight to the map.
    pub fn begin(&mut self, label: &str) {
        self.absorb_direct_changes();
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
            self.state = t.state;
            self.bump();
        }
    }

    pub fn transaction_base(&self) -> Option<&Map> {
        self.transaction.as_ref().map(|t| &t.map)
    }

    pub fn commit(&mut self) {
        self.absorb_direct_changes();
        if let Some(snap) = self.transaction.take() {
            self.last_edit = None;
            crate::linked::sync(&snap.map, &mut self.map);
            self.selection.prune(&self.map);
            if snap.map.nodes != self.map.nodes || snap.map.properties != self.map.properties || snap.map.layers != self.map.layers {
                self.last_command = Some(snap.label.clone());
                self.push_undo(snap);
                self.changed();
            } else {
                self.state = snap.state;
            }
        }
    }

    pub fn cancel(&mut self) {
        if let Some(snap) = self.transaction.take() {
            self.map = snap.map;
            self.selection = snap.selection;
            self.state = snap.state;
            self.bump();
        }
    }

    pub fn undo(&mut self) -> Option<String> {
        self.commit();
        let snap = self.history.undo.pop()?;
        self.last_edit = None;
        self.rewinds += 1;
        let current = self.swap_in(snap);
        let label = current.label.clone();
        self.history.redo.push(current);
        Some(label)
    }

    pub fn redo(&mut self) -> Option<String> {
        self.absorb_direct_changes();
        let snap = self.history.redo.pop()?;
        self.last_edit = None;
        self.rewinds += 1;
        let current = self.swap_in(snap);
        let label = current.label.clone();
        self.history.undo.push(current);
        Some(label)
    }

    /// Makes `snap` current and returns the state it replaced under the same label.
    fn swap_in(&mut self, snap: Snapshot) -> Snapshot {
        let current = Snapshot {
            label: snap.label,
            map: std::mem::replace(&mut self.map, snap.map),
            selection: std::mem::replace(&mut self.selection, snap.selection),
            state: std::mem::replace(&mut self.state, snap.state),
        };
        self.bump();
        current
    }

    pub fn last_command(&self) -> Option<&str> {
        self.last_command.as_deref()
    }

    /// Whether `mark` was taken on this document.
    pub fn owns(&self, mark: &HistoryMark) -> bool {
        mark.doc == self.uid
    }

    pub fn mark(&self) -> HistoryMark {
        HistoryMark { doc: self.uid, top: self.evicted + self.history.undo.len() as u64, rewinds: self.rewinds }
    }

    /// Merges the undo steps added since `mark` into one, named by `label` from their labels, oldest first. A merged
    /// step that changed nothing is dropped. Returns the step's label, or None when no step was added, the mark is from
    /// another document, or an undo or redo ran since, which leaves the history as it is.
    pub fn squash_since(&mut self, mark: HistoryMark, label: impl FnOnce(&[&str]) -> String) -> Option<String> {
        if !self.owns(&mark) || mark.rewinds != self.rewinds {
            return None;
        }

        let added = (self.evicted + self.history.undo.len() as u64).checked_sub(mark.top).filter(|n| *n > 0)? as usize;
        let undo = &mut self.history.undo;
        let first = undo.len().saturating_sub(added);
        let name = label(&undo[first..].iter().map(|s| s.label.as_str()).collect::<Vec<_>>());
        undo.truncate(first + 1);
        let step = undo.last_mut()?;
        let m = &self.map;
        if step.map.nodes == m.nodes && step.map.properties == m.properties && step.map.layers == m.layers && step.map.editor == m.editor {
            self.state = undo.pop()?.state;
            return None;
        }

        step.label = name.clone();
        Some(name)
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

    #[test]
    fn failed_try_edit_leaves_no_trace() {
        let mut doc = Document::new();
        let layer = doc.map.default_layer();
        doc.edit("add", |m, _| m.insert(layer, NodeKind::Entity(Entity::new("light"))));
        doc.undo();
        doc.mark_saved();
        let r: Result<(), &str> = doc.try_edit("fail", |m, _| {
            m.insert(layer, NodeKind::Entity(Entity::new("light")));
            Err("nope")
        });
        assert!(r.is_err());
        assert_eq!(doc.map.entity_count(), 0);
        assert!(!doc.is_modified());
        assert!(doc.history.can_redo());
    }

    #[test]
    fn undo_and_redo_to_the_saved_state_are_unmodified() {
        let mut doc = Document::new();
        let layer = doc.map.default_layer();
        let add = |doc: &mut Document| doc.edit("add", |m, _| m.insert(layer, NodeKind::Entity(Entity::new("light"))));
        add(&mut doc);
        doc.mark_saved();
        add(&mut doc);
        assert!(doc.is_modified());
        doc.undo();
        assert!(!doc.is_modified(), "undo back to the saved state");
        doc.undo();
        assert!(doc.is_modified(), "undo past the saved state");
        doc.redo();
        assert!(!doc.is_modified(), "redo back to the saved state");
        doc.redo();
        assert!(doc.is_modified());

        doc.undo();
        doc.select(|_, s| s.clear());
        assert!(!doc.is_modified(), "selecting keeps a saved document saved");
        doc.revision += 1;
        assert!(doc.is_modified(), "a direct change with a revision bump marks it modified");
        doc.select(|_, s| s.clear());
        assert!(doc.is_modified(), "and selecting afterwards keeps it modified");
        doc.mark_saved();
        doc.mark_unsaved();
        assert!(doc.is_modified());
    }

    #[test]
    fn empty_transaction_keeps_the_saved_state() {
        let mut doc = Document::new();
        let layer = doc.map.default_layer();
        let id = doc.edit("add", |m, _| m.insert(layer, NodeKind::Entity(Entity::new("light"))));
        doc.mark_saved();
        doc.begin("move");
        doc.edit("drag", |m, _| m.entity_mut(id).unwrap().origin.x = 5.0);
        assert!(doc.is_modified());
        doc.edit("drag", |m, _| m.entity_mut(id).unwrap().origin.x = 0.0);
        doc.commit();
        assert!(!doc.is_modified(), "dragged back to where it was");
        doc.begin("move");
        doc.edit("drag", |m, _| m.entity_mut(id).unwrap().origin.x = 5.0);
        doc.cancel();
        assert!(!doc.is_modified());
    }

    #[test]
    fn coalesced_edits_do_not_merge_into_other_steps() {
        let mut doc = Document::new();
        let layer = doc.map.default_layer();
        let id = doc.edit("add", |m, _| m.insert(layer, NodeKind::Entity(Entity::new("light"))));
        doc.edit_coalesced("x", |m, _| m.entity_mut(id).unwrap().origin.x = 1.0);
        doc.edit_coalesced("x", |m, _| m.entity_mut(id).unwrap().origin.x = 2.0);
        assert_eq!(doc.history.undo_labels().count(), 2);
        doc.begin("move");
        doc.edit("drag", |m, _| m.entity_mut(id).unwrap().origin.y = 8.0);
        doc.commit();
        doc.edit_coalesced("x", |m, _| m.entity_mut(id).unwrap().origin.x = 3.0);
        assert_eq!(doc.history.undo_labels().collect::<Vec<_>>(), ["x", "move", "x", "add"]);
        doc.undo();
        assert_eq!(doc.map.entity(id).unwrap().origin, gt_core::DVec3::new(2.0, 8.0, 0.0));
    }

    #[test]
    fn squashed_steps_undo_as_one() {
        let mut doc = Document::new();
        let layer = doc.map.default_layer();
        let add = |doc: &mut Document, label: &str| doc.edit(label, |m, _| m.insert(layer, NodeKind::Entity(Entity::new("light"))));
        add(&mut doc, "before");
        doc.mark_saved();
        let mark = doc.mark();
        add(&mut doc, "a");
        let b = add(&mut doc, "b");
        assert_eq!(doc.squash_since(mark, |steps| format!("batch of {}", steps.join(" "))), Some("batch of a b".into()));
        assert_eq!(doc.history.undo_labels().collect::<Vec<_>>(), ["batch of a b", "before"]);
        assert_eq!(doc.history.map_before(1).unwrap().entity_count(), 1);
        assert!(doc.history.map_before(3).is_none());
        doc.undo();
        assert_eq!(doc.map.entity_count(), 1);
        assert!(!doc.is_modified());
        doc.redo();
        assert!(doc.map.contains(b));

        let mark = doc.mark();
        assert_eq!(doc.squash_since(mark, |_| "nothing".into()), None);
        let id = add(&mut doc, "add");
        doc.edit("remove", |m, _| m.remove(id));
        assert_eq!(doc.squash_since(mark, |_| "no-op".into()), None, "a batch that changed nothing leaves no step");
        assert_eq!(doc.history.undo_labels().next(), Some("batch of a b"));

        let mark = doc.mark();
        add(&mut doc, "c");
        doc.undo();
        add(&mut doc, "d");
        assert_eq!(doc.squash_since(mark, |_| "x".into()), None, "undo inside the batch keeps the steps apart");
        let other = Document::new();
        assert_eq!(doc.squash_since(other.mark(), |_| "x".into()), None);
        assert_eq!(doc.saved_map().entity_count(), 1);
    }

    #[test]
    fn nested_squashes_and_a_full_history() {
        let mut doc = Document::new();
        doc.history.limit = 3;
        let layer = doc.map.default_layer();
        let add = |doc: &mut Document| doc.edit("add", |m, _| m.insert(layer, NodeKind::Entity(Entity::new("light"))));
        add(&mut doc);
        let outer = doc.mark();
        let inner = doc.mark();
        for _ in 0..5 {
            add(&mut doc);
        }

        assert_eq!(doc.squash_since(inner, |s| format!("inner {}", s.len())), Some("inner 3".into()), "steps that fell off the bottom are not listed");
        add(&mut doc);
        assert_eq!(doc.squash_since(outer, |s| s.join(", ")), Some("inner 3, add".into()));
        assert_eq!(doc.history.undo_labels().count(), 1);
    }

    #[test]
    fn coalesced_typing_reaches_linked_copies() {
        let mut doc = Document::new();
        let layer = doc.map.default_layer();
        let group = doc.edit("add", |m, s| {
            let g = m.insert(layer, NodeKind::Group(crate::Group::new("g")));
            m.insert(g, NodeKind::Entity(Entity::new("func_door")));
            s.select_node(g);
            g
        });
        let copy = doc.edit("link", |m, s| crate::ops::duplicate_linked(m, s, gt_core::DVec3::X * 64.0, Default::default()))[0];
        let door = doc.map.get(copy).unwrap().children[0];
        for typed in ["d", "do", "doo", "door"] {
            doc.edit_coalesced("Edit Property", |m, _| {
                m.entity_mut(door).unwrap().properties.insert("targetname".into(), typed.into());
            });
        }

        let original = doc.map.get(group).unwrap().children[0];
        assert_eq!(doc.map.entity(original).unwrap().targetname(), Some("door"));
        assert_eq!(doc.map.entity(doc.map.get(copy).unwrap().children[0]).unwrap().targetname(), Some("door"));
    }
}
