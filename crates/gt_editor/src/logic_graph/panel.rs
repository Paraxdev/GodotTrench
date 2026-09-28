//! The Logic panel: a pan and zoom canvas that draws the graph model and turns clicks and drags into map edits.

use std::collections::{BTreeSet, HashSet};

use egui::epaint::CubicBezierShape;
use egui::{Align2, Color32, FontId, Id, Order, PointerButton, Pos2, Rect, RichText, Sense, Shape, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};
use gt_core::{DVec3, NodeId};
use gt_doc::IoConnection;
use gt_doc::issues::BUILTIN_INPUTS;

use super::edit::{self, NewLink, Placement};
use super::model::{ConnectionId, EdgeKind, GraphEdge, GraphModel, GraphNode, HEADER, Inputs, NodeKey, PIN_ROW, pin_label};
use crate::commands::Action;
use crate::logic_sim::{self, SimResult};
use crate::state::EditorState;
use crate::theme;

const MIN_ZOOM: f32 = 0.2;
const MAX_ZOOM: f32 = 2.0;
const PIN_RADIUS: f32 = 4.5;
/// How close in screen points the pointer has to be to grab a pin or an edge.
const PIN_GRAB: f32 = 10.0;
const EDGE_GRAB: f32 = 6.0;
/// Below this zoom pin names and edge labels are left out, they would be too small to read.
const DETAIL_ZOOM: f32 = 0.45;

const CONTROLS: &str = "Drag from an output pin on the right of a node to an input pin on the left of another to connect them. \
Drop on empty space to add a logic entity there, or right click the canvas.\n\
Click a node to select its entity, double click to frame it in the views. Drag empty space to box select.\n\
Click a wire to edit its delay, parameter and times, Delete removes it.\n\
Wheel zooms, middle or right drag pans.";

/// One pin of a node, by name so it survives the model being built again.
#[derive(Clone, Debug, PartialEq)]
pub struct PinRef {
    pub key: NodeKey,
    pub output: bool,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
enum Hit {
    Pin(PinRef),
    Node(NodeKey),
    Edge(usize),
    Empty,
}

enum Drag {
    /// Moving nodes, `offset` in graph points.
    Nodes {
        keys: HashSet<NodeKey>,
        offset: Vec2,
    },
    /// A new wire from a pin, `at` is the pointer.
    Wire {
        from: PinRef,
        at: Pos2,
    },
    Box {
        start: Pos2,
        at: Pos2,
        add: bool,
    },
    Pan,
}

enum Menu {
    /// Adds an entity at `at` in the graph, wired by `link`.
    Create {
        screen: Pos2,
        at: Pos2,
        link: Option<NewLink>,
        near: Option<NodeId>,
    },
    /// Picks the pin of `to` a wire dropped on its body connects to.
    Pin {
        screen: Pos2,
        from: PinRef,
        to: NodeKey,
    },
    Context {
        screen: Pos2,
        hit: Hit,
    },
}

/// A fired output and the cascade it set off, highlighted in the graph.
pub struct Simulation {
    pub start: NodeId,
    pub output: String,
    pub result: SimResult,
}

pub struct GraphState {
    nodes: Option<imbl::OrdMap<NodeId, gt_doc::Node>>,
    revision: u64,
    game: (usize, usize),
    overlay: u64,
    doc: Option<gt_doc::document::HistoryMark>,
    pub model: GraphModel,
    pan: Vec2,
    zoom: f32,
    refit: bool,
    pub selected_edge: Option<ConnectionId>,
    drag: Option<Drag>,
    menu: Option<Menu>,
    menu_filter: String,
    menu_opened: u64,
    pub sim: Option<Simulation>,
    hovered: bool,
    canvas: Rect,
}

impl Default for GraphState {
    fn default() -> Self {
        Self {
            nodes: None,
            revision: 0,
            game: (0, 0),
            overlay: 0,
            doc: None,
            model: GraphModel::default(),
            pan: vec2(24.0, 24.0),
            zoom: 1.0,
            refit: true,
            selected_edge: None,
            drag: None,
            menu: None,
            menu_filter: String::new(),
            menu_opened: 0,
            sim: None,
            hovered: false,
            canvas: Rect::NOTHING,
        }
    }
}

/// The entities the selection touches, a selected brush counting for its entity.
fn selected_entities(state: &EditorState) -> BTreeSet<NodeId> {
    let map = &state.doc.map;
    state.doc.selection.nodes.iter().map(|id| map.owning_entity(*id).unwrap_or(*id)).filter(|id| map.entity(*id).is_some()).collect()
}

fn select(state: &mut EditorState, ids: &[NodeId], add: bool) {
    state.doc.select(|_, s| {
        if !add {
            s.clear();
        }

        for id in ids {
            if add {
                s.toggle_node(*id);
            } else {
                s.select_node(*id);
            }
        }
    });
    if let Some(id) = ids.last() {
        state.outliner_reveal = Some(*id);
    }
}

impl GraphState {
    /// Builds the model again when the map, the selection, the entity definitions or the overlay changed.
    pub fn refresh(&mut self, state: &EditorState) {
        let game = (state.game.entities.len(), state.game.entities.as_ptr() as usize);
        let same_doc = self.doc.as_ref().is_some_and(|m| state.doc.owns(m));
        let current = self.nodes.as_ref().is_some_and(|n| n.ptr_eq(&state.doc.map.nodes))
            && same_doc
            && self.revision == state.doc.revision
            && self.game == game
            && self.overlay == state.overlay_ghosts.generation;
        if current {
            return;
        }

        if !same_doc {
            self.doc = Some(state.doc.mark());
            self.refit = true;
            self.selected_edge = None;
            self.sim = None;
            self.drag = None;
            self.menu = None;
        }

        self.nodes = Some(state.doc.map.nodes.clone());
        self.revision = state.doc.revision;
        self.game = game;
        self.overlay = state.overlay_ghosts.generation;
        let selected = selected_entities(state);
        let external = state.overlay_ghosts.targetnames();
        self.model = GraphModel::build(&state.doc.map, &Inputs { game: &state.game, selected: &selected, external: &external });
        self.model.place();

        let map = &state.doc.map;
        if let Some((id, i)) = self.selected_edge
            && (map.entity(id).is_none_or(|e| i >= e.outputs.len()) || !selected.contains(&id))
        {
            self.selected_edge = None;
        }

        if let Some(sim) = &mut self.sim {
            if map.entity(sim.start).is_some() {
                sim.result = logic_sim::simulate(map, sim.start, &sim.output);
            } else {
                self.sim = None;
            }
        }
    }

    fn to_screen(&self, p: Pos2) -> Pos2 {
        self.canvas.min + self.pan + p.to_vec2() * self.zoom
    }

    fn to_graph(&self, p: Pos2) -> Pos2 {
        ((p - self.canvas.min - self.pan) / self.zoom).to_pos2()
    }

    /// Where a node is drawn, following a drag in progress.
    fn node_pos(&self, n: &GraphNode) -> Pos2 {
        match &self.drag {
            Some(Drag::Nodes { keys, offset }) if keys.contains(&n.key) => n.pos + *offset,
            _ => n.pos,
        }
    }

    fn node_rect(&self, n: &GraphNode) -> Rect {
        Rect::from_min_size(self.to_screen(self.node_pos(n)), n.size * self.zoom)
    }

    fn pin_screen(&self, n: &GraphNode, output: bool, pin: usize) -> Pos2 {
        let p = self.node_pos(n);
        let x = if output { p.x + n.size.x } else { p.x };
        self.to_screen(pos2(x, p.y + HEADER + PIN_ROW * (pin as f32 + 0.5)))
    }

    /// Screen position of a pin as last drawn, for tests that drive the canvas with the pointer.
    pub fn pin_pos(&self, key: &NodeKey, output: bool, name: &str) -> Option<Pos2> {
        let n = self.model.node(key)?;
        let pin = if output { n.output(name) } else { n.input(name) }?;
        Some(self.pin_screen(n, output, pin))
    }

    /// Screen rect of a node as last drawn.
    pub fn node_screen_rect(&self, key: &NodeKey) -> Option<Rect> {
        self.model.node(key).map(|n| self.node_rect(n))
    }

    fn edge_curve(&self, e: &GraphEdge) -> [Pos2; 4] {
        let a = self.pin_screen(&self.model.nodes[e.from], true, e.from_pin);
        let b = self.pin_screen(&self.model.nodes[e.to], false, e.to_pin);
        curve(a, b, self.zoom)
    }

    fn fit(&mut self) {
        let bounds = self.model.nodes.iter().fold(Rect::NOTHING, |r, n| r.union(n.rect()));
        if !bounds.is_positive() || !self.canvas.is_positive() {
            self.zoom = 1.0;
            self.pan = vec2(24.0, 24.0);
            return;
        }

        let room = self.canvas.shrink(24.0);
        self.zoom = (room.width() / bounds.width()).min(room.height() / bounds.height()).clamp(MIN_ZOOM, 1.0);
        self.pan = room.center() - self.canvas.min - bounds.center().to_vec2() * self.zoom;
        if bounds.height() * self.zoom > room.height() {
            self.pan.y = room.min.y - self.canvas.min.y - bounds.min.y * self.zoom;
        }
    }

    fn hit(&self, p: Pos2, visible: &[usize]) -> Hit {
        let grab = PIN_GRAB.max(PIN_RADIUS * self.zoom * 1.5);
        for &i in visible.iter().rev() {
            let n = &self.model.nodes[i];
            let pins = [(false, n.inputs.len()), (true, n.outputs.len())];
            for (output, count) in pins {
                for pin in 0..count {
                    if self.pin_screen(n, output, pin).distance(p) <= grab {
                        let name = if output { &n.outputs[pin].name } else { &n.inputs[pin].name };
                        return Hit::Pin(PinRef { key: n.key.clone(), output, name: name.clone() });
                    }
                }
            }
        }

        if let Some(&i) = visible.iter().rev().find(|i| self.node_rect(&self.model.nodes[**i]).contains(p)) {
            return Hit::Node(self.model.nodes[i].key.clone());
        }

        let mut best = (EDGE_GRAB, None);
        for (i, e) in self.model.edges.iter().enumerate() {
            let points = self.edge_curve(e);
            if !Rect::from_points(&points).expand(EDGE_GRAB).contains(p) {
                continue;
            }

            let d = distance_to_curve(&points, p);
            if d < best.0 {
                best = (d, Some(i));
            }
        }

        best.1.map_or(Hit::Empty, Hit::Edge)
    }

    /// The pin a wire from `from` would connect to when dropped at `p`: a pin of the other kind within reach, else the
    /// row of the node under the pointer. None over a node without a matching row, where a menu asks which pin.
    fn drop_pin(&self, from: &PinRef, p: Pos2, visible: &[usize]) -> Option<PinRef> {
        if let Hit::Pin(pin) = self.hit(p, visible)
            && pin.output != from.output
        {
            return Some(pin);
        }

        let n = visible.iter().rev().map(|i| &self.model.nodes[*i]).find(|n| self.node_rect(n).contains(p))?;
        let top = self.node_rect(n).min.y + HEADER * self.zoom;
        let row = ((p.y - top) / (PIN_ROW * self.zoom)).floor();
        let pins = if from.output { &n.inputs } else { &n.outputs };
        (row >= 0.0).then(|| pins.get(row as usize)).flatten().map(|pin| PinRef { key: n.key.clone(), output: !from.output, name: pin.name.clone() })
    }

    /// Claims the keys that act on the graph while the pointer is over it: Delete removes the selected wire, Escape
    /// deselects it or closes a menu. Runs before the editor's shortcuts, so Delete does not also delete the entity.
    pub fn take_keys(&mut self, ctx: &egui::Context, state: &mut EditorState) -> bool {
        if !self.hovered {
            return false;
        }

        let busy = self.menu.is_some() || self.drag.is_some() || self.selected_edge.is_some();
        if busy && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            if self.menu.take().is_none() && self.drag.take().is_none() {
                self.selected_edge = None;
            }

            return true;
        }

        let Some(id) = self.selected_edge else { return false };
        let delete = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Delete) || i.consume_key(egui::Modifiers::NONE, egui::Key::Backspace));
        if delete {
            edit::disconnect(state, &[id]);
            self.selected_edge = None;
        }

        delete
    }

    fn simulate(&mut self, state: &EditorState, start: NodeId, output: &str) {
        let result = logic_sim::simulate(&state.doc.map, start, output);
        self.sim = Some(Simulation { start, output: output.to_string(), result });
    }

    fn open_menu(&mut self, ctx: &egui::Context, menu: Menu) {
        self.menu = Some(menu);
        self.menu_filter.clear();
        self.menu_opened = ctx.cumulative_pass_nr();
    }

    /// Graph positions of the wired entity nodes that still follow the automatic layout, stored along with a first
    /// hand placed node so the rest of the picture holds still.
    fn unpinned(&self) -> Vec<(NodeId, Pos2)> {
        self.model.nodes.iter().filter(|n| n.stored.is_none() && !n.unwired).filter_map(|n| Some((n.key.entity()?, n.pos))).collect()
    }
}

fn curve(a: Pos2, b: Pos2, zoom: f32) -> [Pos2; 4] {
    let dx = ((b.x - a.x).abs() * 0.5).max(50.0 * zoom);
    [a, a + vec2(dx, 0.0), b - vec2(dx, 0.0), b]
}

fn curve_at(p: &[Pos2; 4], t: f32) -> Pos2 {
    let u = 1.0 - t;
    let v = p[0].to_vec2() * (u * u * u) + p[1].to_vec2() * (3.0 * u * u * t) + p[2].to_vec2() * (3.0 * u * t * t) + p[3].to_vec2() * (t * t * t);
    v.to_pos2()
}

fn distance_to_curve(points: &[Pos2; 4], p: Pos2) -> f32 {
    let mut best = f32::MAX;
    let mut last = points[0];
    for i in 1..=24 {
        let next = curve_at(points, i as f32 / 24.0);
        let seg = next - last;
        let t = if seg.length_sq() > 0.0 { ((p - last).dot(seg) / seg.length_sq()).clamp(0.0, 1.0) } else { 0.0 };
        best = best.min((last + seg * t).distance(p));
        last = next;
    }

    best
}

fn edge_color(kind: EdgeKind) -> Color32 {
    match kind {
        EdgeKind::Resolved => theme::GRAY_6,
        EdgeKind::Dynamic | EdgeKind::Overlay => theme::INFO,
        EdgeKind::Broken => theme::ERROR,
    }
}

fn marker_color(key: &NodeKey) -> Option<Color32> {
    match key {
        NodeKey::Entity(_) => None,
        NodeKey::Dynamic(_) | NodeKey::Overlay(_) => Some(theme::INFO),
        NodeKey::Missing(_) => Some(theme::ERROR),
    }
}

/// Classes the Add menu offers: the logic entities first, then other point entities that take part in I/O.
fn creatable(state: &EditorState) -> Vec<(String, String)> {
    let mut logic: Vec<(String, String)> = Vec::new();
    let mut other: Vec<(String, String)> = Vec::new();
    for d in state.game.point_entities() {
        let entry = (d.classname.clone(), d.description.clone());
        if d.classname.starts_with("logic_") {
            logic.push(entry);
        } else if !d.inputs.is_empty() || !d.outputs.is_empty() {
            other.push(entry);
        }
    }

    logic.sort();
    other.sort();
    logic.extend(other);
    logic
}

fn pack_missing(state: &EditorState) -> bool {
    !state.game.entities.iter().any(|d| crate::entity_pack::in_pack(&d.classname))
}

pub fn show(ui: &mut Ui, state: &mut EditorState, gs: &mut GraphState, actions: &mut Vec<Action>) {
    gs.refresh(state);
    toolbar(ui, state, gs, actions);
    let canvas = ui.available_rect_before_wrap();
    let canvas = Rect::from_min_size(canvas.min, canvas.size().max(vec2(120.0, 120.0)));
    let response = ui.allocate_rect(canvas, Sense::click_and_drag());
    gs.canvas = canvas;
    if gs.refit && !gs.model.nodes.is_empty() {
        gs.fit();
        gs.refit = false;
    }

    gs.hovered = response.hovered() || response.dragged() || gs.drag.is_some();
    navigate(ui, &response, gs);
    let visible: Vec<usize> = (0..gs.model.nodes.len()).filter(|i| gs.node_rect(&gs.model.nodes[*i]).intersects(canvas)).collect();
    let hover = response.hover_pos().filter(|_| gs.drag.is_none() && gs.menu.is_none()).map(|p| gs.hit(p, &visible));
    interact(ui, &response, state, gs, actions, &visible);
    gs.refresh(state);
    let visible: Vec<usize> = (0..gs.model.nodes.len()).filter(|i| gs.node_rect(&gs.model.nodes[*i]).intersects(canvas)).collect();
    draw(ui, state, gs, &visible, hover.as_ref());
    overlays(ui, state, gs, actions);
}

fn toolbar(ui: &mut Ui, state: &mut EditorState, gs: &mut GraphState, actions: &mut Vec<Action>) {
    ui.horizontal_wrapped(|ui| {
        ui.menu_button("Add", |ui| {
            let at = gs.to_graph(gs.canvas.center()) - vec2(80.0, HEADER);
            let near = selected_entities(state).into_iter().next();
            create_list(ui, state, gs, at, None, near, actions);
        })
        .response
        .on_hover_text("Adds a logic entity on the Logic layer, next to the selected entity in the map");

        let selected: Vec<NodeId> = selected_entities(state).into_iter().filter(|id| gs.model.node(&NodeKey::Entity(*id)).is_some()).collect();
        let arrange_tip =
            "Lays out the selected nodes left to right by signal flow. With fewer than two selected, every node goes back to the automatic layout";
        if ui.button("Arrange").on_hover_text(arrange_tip).clicked() {
            arrange(state, gs, &selected);
        }

        if ui.button("Fit").on_hover_text("Zooms to show every node").clicked() {
            gs.fit();
        }

        let start = match selected.as_slice() {
            [id] => state.doc.map.entity(*id).map(|e| (*id, logic_sim::start_outputs(state.game.entity(&e.classname), e))),
            _ => None,
        };
        let tip = "Fires an output of the selected entity on paper and highlights every connection that would follow";
        ui.add_enabled_ui(start.as_ref().is_some_and(|(_, o)| !o.is_empty()), |ui| {
            ui.menu_button("Simulate", |ui| {
                for output in start.as_ref().map(|(_, o)| o.as_slice()).unwrap_or_default() {
                    if ui.button(output).clicked() {
                        gs.simulate(state, start.as_ref().expect("enabled with a start").0, output);
                        ui.close();
                    }
                }
            })
            .response
            .on_hover_text(tip)
            .on_disabled_hover_text(format!("{tip}. Select one entity that has outputs first"));
        });

        ui.label(RichText::new(format!("{:.0}%", gs.zoom * 100.0)).weak()).on_hover_text("Wheel zooms, middle or right drag pans");
        crate::help::line(ui, state.prefs.help_text, "Drag from an output pin to an input pin to connect", CONTROLS);
    });
}

fn arrange(state: &mut EditorState, gs: &mut GraphState, selected: &[NodeId]) {
    if selected.len() < 2 {
        let stored: Vec<(NodeId, Option<Pos2>)> = gs.model.nodes.iter().filter(|n| n.stored.is_some()).filter_map(|n| Some((n.key.entity()?, None))).collect();
        edit::set_positions(state, "Arrange Logic Graph", &stored);
        gs.refit = true;
        return;
    }

    let nodes: Vec<usize> = selected.iter().filter_map(|id| gs.model.index_of(&NodeKey::Entity(*id))).collect();
    let local = |i: usize| nodes.iter().position(|n| *n == i);
    let links: Vec<(usize, usize)> = gs.model.edges.iter().filter_map(|e| Some((local(e.from)?, local(e.to)?))).collect();
    let sizes: Vec<Vec2> = nodes.iter().map(|i| gs.model.nodes[*i].size).collect();
    let laid = super::layout::layered(&sizes, &links);
    let corner = nodes.iter().fold(pos2(f32::MAX, f32::MAX), |c, i| c.min(gs.model.nodes[*i].pos));
    let mut positions: Vec<(NodeId, Option<Pos2>)> = gs.unpinned().into_iter().map(|(id, p)| (id, Some(p))).collect();
    for (k, i) in nodes.iter().enumerate() {
        if let Some(id) = gs.model.nodes[*i].key.entity() {
            positions.retain(|(other, _)| *other != id);
            positions.push((id, Some(corner + laid[k].to_vec2())));
        }
    }

    edit::set_positions(state, "Arrange Logic Nodes", &positions);
}

fn navigate(ui: &Ui, response: &egui::Response, gs: &mut GraphState) {
    if response.hovered()
        && let Some(p) = response.hover_pos()
    {
        let (scroll, pinch) = ui.input(|i| (i.smooth_scroll_delta.y, i.zoom_delta()));
        let factor = pinch * (scroll * 0.002).exp();
        if factor != 1.0 {
            let anchor = gs.to_graph(p);
            gs.zoom = (gs.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
            gs.pan = p - gs.canvas.min - anchor.to_vec2() * gs.zoom;
        }
    }

    if response.dragged_by(PointerButton::Middle) || response.dragged_by(PointerButton::Secondary) {
        gs.pan += response.drag_delta();
        if gs.drag.is_none() {
            gs.drag = Some(Drag::Pan);
        }
    }
}

fn interact(ui: &Ui, response: &egui::Response, state: &mut EditorState, gs: &mut GraphState, actions: &mut Vec<Action>, visible: &[usize]) {
    let ctx = ui.ctx().clone();
    let modifiers = ui.input(|i| i.modifiers);
    let add = modifiers.shift || modifiers.command;
    let pointer = response.interact_pointer_pos().or(response.hover_pos());

    // A click outside an open menu closes it and does nothing else.
    if gs.menu.is_some() && (response.clicked() || response.drag_started() || response.secondary_clicked()) && ctx.cumulative_pass_nr() > gs.menu_opened {
        gs.menu = None;
        return;
    }

    if response.drag_started_by(PointerButton::Primary) {
        let origin = ui.input(|i| i.pointer.press_origin()).unwrap_or_default();
        gs.drag = match gs.hit(origin, visible) {
            Hit::Pin(from) => Some(Drag::Wire { from, at: origin }),
            Hit::Node(NodeKey::Entity(id)) => {
                let selected = selected_entities(state);
                if !selected.contains(&id) {
                    select(state, &[id], add);
                }

                let keys: HashSet<NodeKey> = selected_entities(state).into_iter().map(NodeKey::Entity).collect();
                Some(Drag::Nodes { keys, offset: Vec2::ZERO })
            }
            Hit::Node(_) => None,
            Hit::Edge(_) | Hit::Empty => Some(Drag::Box { start: origin, at: origin, add }),
        };
    }

    if let Some(p) = pointer {
        match &mut gs.drag {
            Some(Drag::Wire { at, .. }) | Some(Drag::Box { at, .. }) => *at = p,
            Some(Drag::Nodes { offset, .. }) => *offset += response.drag_delta() / gs.zoom,
            _ => {}
        }
    }

    let released = response.drag_stopped() || (gs.drag.is_some() && !ui.input(|i| i.pointer.any_down()));
    if released && let Some(drag) = gs.drag.take() {
        match drag {
            Drag::Nodes { keys, offset } if offset.length() >= 0.5 => {
                let mut positions: Vec<(NodeId, Option<Pos2>)> = gs.unpinned().into_iter().map(|(id, p)| (id, Some(p))).collect();
                for n in gs.model.nodes.iter().filter(|n| keys.contains(&n.key)) {
                    if let Some(id) = n.key.entity() {
                        positions.retain(|(other, _)| *other != id);
                        positions.push((id, Some(n.pos + offset)));
                    }
                }

                let label = if keys.len() == 1 { "Move Logic Node".to_string() } else { format!("Move {} Logic Nodes", keys.len()) };
                edit::set_positions(state, &label, &positions);
            }
            Drag::Wire { from, at } => finish_wire(&ctx, state, gs, from, at, visible),
            Drag::Box { start, at, add } => {
                let area = Rect::from_two_pos(start, at);
                let ids: Vec<NodeId> =
                    visible.iter().map(|i| &gs.model.nodes[*i]).filter(|n| gs.node_rect(n).intersects(area)).filter_map(|n| n.key.entity()).collect();
                if add {
                    state.doc.select(|_, s| s.nodes.extend(ids.iter().copied()));
                } else {
                    select(state, &ids, false);
                }
            }
            _ => {}
        }
    }

    let Some(p) = pointer else { return };
    if response.double_clicked() {
        if let Hit::Node(NodeKey::Entity(id)) = gs.hit(p, visible) {
            select(state, &[id], false);
            actions.push(Action::FocusSelection);
        }
    } else if response.clicked() {
        match gs.hit(p, visible) {
            Hit::Pin(PinRef { key: NodeKey::Entity(id), .. }) | Hit::Node(NodeKey::Entity(id)) => {
                gs.selected_edge = None;
                select(state, &[id], add);
            }
            Hit::Pin(_) | Hit::Node(_) => gs.selected_edge = None,
            Hit::Edge(i) => {
                let e = &gs.model.edges[i];
                gs.selected_edge = Some(e.id());
                select(state, &[e.source], false);
            }
            Hit::Empty => {
                gs.selected_edge = None;
                if !add && !state.doc.selection.is_empty() {
                    state.doc.select(|_, s| s.clear());
                }
            }
        }
    } else if response.secondary_clicked() {
        let hit = gs.hit(p, visible);
        let menu = match hit {
            Hit::Empty => Menu::Create { screen: p, at: gs.to_graph(p), link: None, near: selected_entities(state).into_iter().next() },
            hit => Menu::Context { screen: p, hit },
        };
        gs.open_menu(&ctx, menu);
    }
}

fn finish_wire(ctx: &egui::Context, state: &mut EditorState, gs: &mut GraphState, from: PinRef, at: Pos2, visible: &[usize]) {
    if let Some(to) = gs.drop_pin(&from, at, visible) {
        let (out, input) = if from.output { (&from, &to) } else { (&to, &from) };
        if let Some(source) = out.key.entity()
            && let Err(e) = edit::connect(state, source, &out.name, &input.key, &input.name)
        {
            state.set_status(e);
        }

        return;
    }

    let on = visible.iter().rev().map(|i| &gs.model.nodes[*i]).find(|n| gs.node_rect(n).contains(at)).map(|n| n.key.clone());
    let menu = match on {
        Some(to) if to != from.key && (from.output || to.entity().is_some()) => Menu::Pin { screen: at, from, to },
        Some(_) => return,
        None => {
            let (link, near) = if from.output {
                (from.key.entity().map(|source| NewLink::From { source, output: from.name.clone() }), from.key.entity())
            } else {
                (Some(NewLink::To { target: from.key.clone(), input: from.name.clone() }), from.key.entity())
            };
            // The new node lands with the pin the wire ends on under the pointer.
            let graph = gs.to_graph(at);
            let at = if from.output { graph - vec2(0.0, HEADER + PIN_ROW * 0.5) } else { graph - vec2(160.0, HEADER + PIN_ROW * 0.5) };
            Menu::Create { screen: gs.to_screen(graph), at, link, near }
        }
    };
    gs.open_menu(ctx, menu);
}

fn draw(ui: &Ui, state: &EditorState, gs: &GraphState, visible: &[usize], hover: Option<&Hit>) {
    let canvas = gs.canvas;
    let painter = ui.painter_at(canvas);
    painter.rect_filled(canvas, 0.0, theme::GRAY_0);
    let step = 32.0 * gs.zoom;
    if step >= 10.0 {
        let grid = Stroke::new(1.0, theme::GRAY_1);
        let origin = canvas.min + gs.pan;
        let mut x = canvas.min.x + (origin.x - canvas.min.x).rem_euclid(step);
        while x < canvas.max.x {
            painter.line_segment([pos2(x, canvas.min.y), pos2(x, canvas.max.y)], grid);
            x += step;
        }

        let mut y = canvas.min.y + (origin.y - canvas.min.y).rem_euclid(step);
        while y < canvas.max.y {
            painter.line_segment([pos2(canvas.min.x, y), pos2(canvas.max.x, y)], grid);
            y += step;
        }
    }

    let selected = selected_entities(state);
    let sim_steps = |id: ConnectionId| -> Vec<usize> {
        gs.sim
            .as_ref()
            .map(|s| s.result.events.iter().enumerate().filter(|(_, e)| (e.source, e.connection) == id).map(|(i, _)| i + 1).collect())
            .unwrap_or_default()
    };
    let sim_nodes: BTreeSet<NodeId> =
        gs.sim.as_ref().map(|s| s.result.events.iter().flat_map(|e| std::iter::once(e.source).chain(e.resolved.iter().copied())).collect()).unwrap_or_default();
    let detail = gs.zoom >= DETAIL_ZOOM;
    let font = |size: f32| FontId::proportional(size * gs.zoom);

    let hovered_edge = match hover {
        Some(Hit::Edge(i)) => Some(*i),
        _ => None,
    };
    let mut badges = Vec::new();
    for (i, e) in gs.model.edges.iter().enumerate() {
        let points = gs.edge_curve(e);
        if !Rect::from_points(&points).intersects(canvas) {
            continue;
        }

        let steps = sim_steps(e.id());
        let chosen = gs.selected_edge == Some(e.id());
        let (width, color) = if chosen {
            (3.0, theme::ACCENT)
        } else if !steps.is_empty() {
            (3.0, theme::YELLOW)
        } else if hovered_edge == Some(i) {
            (2.5, theme::GRAY_7)
        } else {
            (1.6, edge_color(e.kind))
        };
        let stroke = Stroke::new(width, color);
        if e.kind == EdgeKind::Broken && !chosen {
            let line = CubicBezierShape::from_points_stroke(points, false, Color32::TRANSPARENT, stroke).flatten(Some(0.5));
            painter.extend(Shape::dashed_line(&line, stroke, 6.0, 4.0));
        } else {
            painter.add(CubicBezierShape::from_points_stroke(points, false, Color32::TRANSPARENT, stroke));
        }

        let mid = curve_at(&points, 0.5);
        if !steps.is_empty() {
            badges.push((mid, steps.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(",")));
        } else if detail && !e.label.is_empty() {
            let galley = painter.layout_no_wrap(e.label.clone(), font(11.0), theme::GRAY_7);
            let r = Rect::from_center_size(mid, galley.size() + vec2(8.0, 2.0));
            painter.rect_filled(r, 3.0, theme::GRAY_2);
            painter.galley(r.min + vec2(4.0, 1.0), galley, theme::GRAY_7);
        }
    }

    for &i in visible {
        let n = &gs.model.nodes[i];
        let r = gs.node_rect(n);
        let rounding = 6.0 * gs.zoom;
        let marker = marker_color(&n.key);
        painter.rect_filled(r, rounding, if marker.is_some() { theme::GRAY_1 } else { theme::GRAY_2 });
        let header = Rect::from_min_size(r.min, vec2(r.width(), HEADER * gs.zoom));
        if let Some(c) = n.color {
            let c = Color32::from_rgb((c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8);
            let strip = Rect::from_min_size(r.min, vec2(r.width(), 4.0 * gs.zoom));
            painter.rect_filled(strip, egui::CornerRadius { nw: rounding as u8, ne: rounding as u8, sw: 0, se: 0 }, c);
        }

        let text = |ui_pos: Pos2, align: Align2, s: &str, size: f32, color: Color32| {
            if size * gs.zoom >= 5.0 {
                painter.with_clip_rect(r.intersect(canvas)).text(ui_pos, align, s, font(size), color);
            }
        };
        text(header.min + vec2(10.0, 8.0) * gs.zoom, Align2::LEFT_TOP, &n.title, 13.0, theme::GRAY_7);
        if detail {
            text(header.min + vec2(10.0, 24.0) * gs.zoom, Align2::LEFT_TOP, &n.subtitle, 11.0, marker.unwrap_or(theme::GRAY_5));
        }

        let connected =
            |output: bool, pin: usize| gs.model.edges.iter().any(|e| if output { e.from == i && e.from_pin == pin } else { e.to == i && e.to_pin == pin });
        let hovered_pin = match hover {
            Some(Hit::Pin(p)) if p.key == n.key => Some((p.output, p.name.as_str())),
            _ => None,
        };
        for (output, pins) in [(false, &n.inputs), (true, &n.outputs)] {
            for (k, pin) in pins.iter().enumerate() {
                let at = gs.pin_screen(n, output, k);
                let hot = hovered_pin == Some((output, pin.name.as_str()));
                let color = if hot { theme::GRAY_7 } else { theme::GRAY_5 };
                let radius = PIN_RADIUS * gs.zoom.max(0.6) * if hot { 1.4 } else { 1.0 };
                if connected(output, k) {
                    painter.circle_filled(at, radius, if output { theme::ACCENT } else { color });
                } else {
                    painter.circle_stroke(at, radius, Stroke::new(1.5, color));
                }

                if detail {
                    let warn = n.color.is_some() && !pin.declared;
                    let label_color = if warn { theme::WARNING } else { theme::FG };
                    let (align, dx) = if output { (Align2::RIGHT_CENTER, -10.0) } else { (Align2::LEFT_CENTER, 10.0) };
                    text(at + vec2(dx * gs.zoom, 0.0), align, pin_label(&pin.name), 12.0, label_color);
                }
            }
        }

        let border = if n.key.entity().is_some_and(|id| selected.contains(&id)) {
            Stroke::new(2.0, theme::ACCENT)
        } else if n.key.entity().is_some_and(|id| sim_nodes.contains(&id)) {
            Stroke::new(2.0, theme::YELLOW)
        } else if let Some(c) = marker {
            Stroke::new(1.5, c)
        } else if matches!(hover, Some(Hit::Node(k)) if *k == n.key) {
            Stroke::new(1.0, theme::GRAY_5)
        } else {
            Stroke::new(1.0, theme::GRAY_3)
        };
        painter.rect_stroke(r, rounding, border, StrokeKind::Inside);
    }

    for (at, label) in badges {
        let galley = painter.layout_no_wrap(label, FontId::proportional(11.0), theme::GRAY_0);
        let r = Rect::from_center_size(at, galley.size() + vec2(10.0, 4.0));
        painter.rect_filled(r, r.height() * 0.5, theme::YELLOW);
        painter.galley(r.min + vec2(5.0, 2.0), galley, theme::GRAY_0);
    }

    match &gs.drag {
        Some(Drag::Wire { from, at }) => {
            if let Some(n) = gs.model.node(&from.key) {
                let pin = if from.output { n.output(&from.name) } else { n.input(&from.name) }.unwrap_or(0);
                let a = gs.pin_screen(n, from.output, pin);
                let points = if from.output { curve(a, *at, gs.zoom) } else { curve(*at, a, gs.zoom) };
                painter.add(CubicBezierShape::from_points_stroke(points, false, Color32::TRANSPARENT, Stroke::new(2.0, theme::ACCENT)));
            }
        }
        Some(Drag::Box { start, at, .. }) => {
            let r = Rect::from_two_pos(*start, *at);
            painter.rect_filled(r, 0.0, theme::ACCENT.gamma_multiply(0.12));
            painter.rect_stroke(r, 0.0, Stroke::new(1.0, theme::ACCENT), StrokeKind::Inside);
        }
        _ => {}
    }
}

fn overlays(ui: &mut Ui, state: &mut EditorState, gs: &mut GraphState, actions: &mut Vec<Action>) {
    let ctx = ui.ctx().clone();
    let canvas = gs.canvas;
    if gs.model.nodes.is_empty() {
        egui::Area::new(Id::new("logic_graph_empty")).order(Order::Middle).fixed_pos(canvas.min + vec2(16.0, 16.0)).constrain_to(canvas).show(&ctx, |ui| {
            ui.set_max_width((canvas.width() - 32.0).max(120.0));
            ui.label(RichText::new("No wiring yet").strong());
            ui.label(
                RichText::new(
                    "Select an entity to show it here, then drag from an output pin on its right edge to an input pin of another. \
                     Right click here to add a logic entity such as a relay or a timer.",
                )
                .weak(),
            );
            if pack_missing(state) {
                install_hint(ui, actions);
            }
        });
    }

    if let Some(id) = gs.selected_edge {
        edge_editor(&ctx, state, gs, id);
    }

    if gs.sim.is_some() {
        sim_list(&ctx, state, gs);
    }

    let Some(menu) = gs.menu.take() else { return };
    let screen = match &menu {
        Menu::Create { screen, .. } | Menu::Pin { screen, .. } | Menu::Context { screen, .. } => *screen,
    };
    let mut keep = true;
    let area = egui::Area::new(Id::new("logic_graph_menu")).order(Order::Foreground).fixed_pos(screen).constrain_to(ctx.content_rect()).show(&ctx, |ui| {
        egui::Frame::menu(ui.style()).show(ui, |ui| {
            keep = match &menu {
                Menu::Create { at, link, near, .. } => !create_list(ui, state, gs, *at, link.clone(), *near, actions),
                Menu::Pin { from, to, .. } => !pin_list(ui, state, gs, from, to),
                Menu::Context { hit, .. } => !context_menu(ui, state, gs, hit, actions),
            };
        });
    });
    let pressed_at = ctx.input(|i| if i.pointer.any_pressed() { i.pointer.interact_pos() } else { None });
    let clicked_away = pressed_at.is_some_and(|p| !area.response.rect.contains(p)) && ctx.cumulative_pass_nr() > gs.menu_opened;
    if keep && !clicked_away && !ctx.input(|i| i.key_pressed(egui::Key::Escape)) && gs.menu.is_none() {
        gs.menu = Some(menu);
    }
}

fn install_hint(ui: &mut Ui, actions: &mut Vec<Action>) {
    ui.label(RichText::new("Relays, timers, counters and the other logic entities come with the Gameplay entities pack.").weak());
    let tip = Action::InstallGameplayEntities.help().unwrap_or_default();
    if ui.button("Install Gameplay Entities").on_hover_text(tip).clicked() {
        actions.push(Action::InstallGameplayEntities);
    }
}

/// The searchable list of classes to add. Returns true when it is done.
fn create_list(
    ui: &mut Ui,
    state: &mut EditorState,
    gs: &mut GraphState,
    at: Pos2,
    link: Option<NewLink>,
    near: Option<NodeId>,
    actions: &mut Vec<Action>,
) -> bool {
    ui.set_min_width(220.0);
    let field = ui.add(egui::TextEdit::singleline(&mut gs.menu_filter).hint_text("Search entities").desired_width(220.0));
    if !field.has_focus() && ui.ctx().cumulative_pass_nr() <= gs.menu_opened + 1 {
        field.request_focus();
    }

    let filter = gs.menu_filter.to_lowercase();
    let classes: Vec<(String, String)> = creatable(state).into_iter().filter(|(c, d)| c.contains(&filter) || d.to_lowercase().contains(&filter)).collect();
    let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let mut chosen = enter.then(|| classes.first().map(|(c, _)| c.clone())).flatten();
    if pack_missing(state) {
        let asked = actions.len();
        install_hint(ui, actions);
        if actions.len() > asked {
            ui.close();
            return true;
        }
    }

    egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
        let mut logic = true;
        for (classname, description) in &classes {
            if logic && !classname.starts_with("logic_") {
                logic = false;
                ui.separator();
            }

            let button = ui.selectable_label(false, classname);
            let button = if description.is_empty() { button } else { button.on_hover_text(description) };
            if button.clicked() {
                chosen = Some(classname.clone());
            }
        }

        if classes.is_empty() {
            ui.label(RichText::new("No entity matches").weak());
        }
    });

    let Some(classname) = chosen else { return false };
    let fallback = state.view_focus.or(state.cursor_world).unwrap_or(DVec3::ZERO);
    let pin = gs.unpinned();
    match edit::create(state, &classname, Placement { at, near, fallback, pin: &pin }, link) {
        Ok(id) => state.outliner_reveal = Some(id),
        Err(e) => state.set_status(e),
    }

    ui.close();
    true
}

/// The pins of `to` a wire from `from` can end on, and a field for any other name. Returns true when it is done.
fn pin_list(ui: &mut Ui, state: &mut EditorState, gs: &mut GraphState, from: &PinRef, to: &NodeKey) -> bool {
    let Some(node) = gs.model.node(to) else { return true };
    let mut names: Vec<String> = if from.output { &node.inputs } else { &node.outputs }.iter().map(|p| p.name.clone()).collect();
    if from.output {
        for b in BUILTIN_INPUTS {
            if !names.iter().any(|n| n == b) {
                names.push(b.to_string());
            }
        }
    }

    ui.label(RichText::new(format!("Connect to {}", node.title)).strong());
    let field = ui.add(egui::TextEdit::singleline(&mut gs.menu_filter).hint_text(if from.output { "Input name" } else { "Output name" }).desired_width(200.0));
    if !field.has_focus() && ui.ctx().cumulative_pass_nr() <= gs.menu_opened + 1 {
        field.request_focus();
    }

    let typed = gs.menu_filter.trim().to_string();
    let shown: Vec<&String> = names.iter().filter(|n| n.contains(&typed)).collect();
    let mut chosen = (field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !typed.is_empty())
        .then(|| shown.first().filter(|n| ***n == typed).map_or(typed.clone(), |n| (*n).clone()));
    for name in shown {
        if ui.selectable_label(false, pin_label(name)).clicked() {
            chosen = Some(name.clone());
        }
    }

    let Some(name) = chosen else { return false };
    let other = PinRef { key: to.clone(), output: !from.output, name };
    let (out, input) = if from.output { (from, &other) } else { (&other, from) };
    if let Some(source) = out.key.entity()
        && let Err(e) = edit::connect(state, source, &out.name, &input.key, &input.name)
    {
        state.set_status(e);
    }

    true
}

fn context_menu(ui: &mut Ui, state: &mut EditorState, gs: &mut GraphState, hit: &Hit, actions: &mut Vec<Action>) -> bool {
    let key = match hit {
        Hit::Edge(i) => {
            let Some(e) = gs.model.edges.get(*i) else { return true };
            let id = e.id();
            if ui.button("Edit Connection").clicked() {
                gs.selected_edge = Some(id);
                select(state, &[id.0], false);
                return true;
            }

            if ui.button("Delete Connection").clicked() {
                edit::disconnect(state, &[id]);
                gs.selected_edge = None;
                return true;
            }

            return false;
        }
        Hit::Pin(p) => p.key.clone(),
        Hit::Node(k) => k.clone(),
        Hit::Empty => return true,
    };
    let Some(id) = key.entity() else {
        ui.label(RichText::new("Found when the map runs, or not at all").weak());
        return false;
    };

    if ui.button("Frame in Views").clicked() {
        select(state, &[id], false);
        actions.push(Action::FocusSelection);
        return true;
    }

    let outputs = state.doc.map.entity(id).map(|e| logic_sim::start_outputs(state.game.entity(&e.classname), e)).unwrap_or_default();
    if !outputs.is_empty() {
        ui.separator();
        ui.label(RichText::new("Simulate").weak());
        for output in outputs {
            if ui.button(&output).clicked() {
                gs.simulate(state, id, &output);
                return true;
            }
        }
    }

    false
}

fn edge_editor(ctx: &egui::Context, state: &mut EditorState, gs: &mut GraphState, id: ConnectionId) {
    let Some(entity) = state.doc.map.entity(id.0) else { return };
    let Some(mut conn) = entity.outputs.get(id.1).cloned() else { return };
    let source_name = entity.targetname().unwrap_or(&entity.classname).to_string();
    let outputs: Vec<(String, &str)> =
        state.game.entity(&entity.classname).map(|d| d.outputs.iter().map(|o| (o.name.clone(), "")).collect()).unwrap_or_default();
    let mut inputs: Vec<(String, &str)> = state
        .doc
        .map
        .entities()
        .filter(|(_, e)| e.targetname() == Some(conn.target.as_str()))
        .filter_map(|(_, e)| state.game.entity(&e.classname))
        .flat_map(|d| d.inputs.iter().map(|i| (i.name.clone(), "")))
        .collect();
    inputs.extend(BUILTIN_INPUTS.iter().map(|b| (b.to_string(), "any node")));
    inputs.dedup_by(|a, b| a.0 == b.0);
    let targets = crate::panels::target_options(state);
    let canvas = gs.canvas;
    let mut delete = false;
    let mut close = false;
    let before = conn.clone();
    egui::Area::new(Id::new("logic_edge_editor")).order(Order::Middle).fixed_pos(pos2(canvas.max.x - 290.0, canvas.min.y + 8.0)).constrain_to(canvas).show(
        ctx,
        |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_width(270.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Connection from {source_name}")).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| close = ui.small_button("\u{d7}").on_hover_text("Close").clicked());
                });
                egui::Grid::new("logic_edge_fields").num_columns(2).show(ui, |ui| {
                    ui.label("output");
                    crate::panels::combo_or_text(ui, Id::new("logic_edge_output"), &mut conn.output, outputs, 150.0);
                    ui.end_row();
                    ui.label("target");
                    crate::panels::combo_or_text(ui, Id::new("logic_edge_target"), &mut conn.target, targets, 150.0);
                    ui.end_row();
                    ui.label("input");
                    crate::panels::combo_or_text(ui, Id::new("logic_edge_input"), &mut conn.input, inputs, 150.0);
                    ui.end_row();
                    ui.label("parameter");
                    ui.add(egui::TextEdit::singleline(&mut conn.parameter).desired_width(176.0));
                    ui.end_row();
                    ui.label("delay").on_hover_text("Seconds before the input is called");
                    ui.add(egui::DragValue::new(&mut conn.delay).speed(0.05).range(0.0..=3600.0).suffix(" s"));
                    ui.end_row();
                    ui.label("times").on_hover_text("How often it may fire, -1 is every time");
                    ui.add(
                        egui::DragValue::new(&mut conn.times)
                            .speed(0.1)
                            .range(-1..=9999)
                            .custom_formatter(|v, _| if v < 0.0 { "every time".into() } else { format!("{v}") }),
                    );
                    ui.end_row();
                });
                ui.horizontal(|ui| {
                    delete = ui.button("Delete").on_hover_text("Removes this connection, Delete over the graph does the same").clicked();
                });
            });
        },
    );
    if delete {
        edit::disconnect(state, &[id]);
        gs.selected_edge = None;
    } else if conn != before {
        edit::update(state, id, IoConnection { times: conn.times.max(-1), ..conn });
    }

    if close {
        gs.selected_edge = None;
    }
}

fn sim_list(ctx: &egui::Context, state: &mut EditorState, gs: &mut GraphState) {
    let Some(sim) = &gs.sim else { return };
    let canvas = gs.canvas;
    let name = state.doc.map.entity(sim.start).map(|e| e.targetname().unwrap_or(&e.classname).to_string()).unwrap_or_default();
    let mut clear = false;
    let mut pick = None;
    egui::Area::new(Id::new("logic_sim_list")).order(Order::Middle).fixed_pos(pos2(canvas.min.x + 8.0, canvas.max.y - 200.0)).constrain_to(canvas).show(
        ctx,
        |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_max_width(360.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(format!("{name}.{}", sim.output)).strong());
                    let r = &sim.result;
                    ui.label(format!("{} steps", r.events.len()));
                    if r.broken() > 0 {
                        ui.label(RichText::new(format!("{} broken", r.broken())).color(theme::ERROR));
                    }

                    if r.truncated {
                        ui.label(RichText::new("truncated, feedback loop").weak());
                    }

                    clear = ui.small_button("Clear").clicked();
                });
                if sim.result.events.is_empty() {
                    ui.label(RichText::new("That output is not wired to anything.").weak());
                }

                egui::ScrollArea::vertical().max_height(130.0).show(ui, |ui| {
                    for (i, ev) in sim.result.events.iter().enumerate() {
                        let mut text = format!("{}. {}.{} \u{2192} {}.{}", i + 1, ev.source_name, ev.output, ev.target, ev.input);
                        if ev.time > 0.0 {
                            text.push_str(&format!("  at {}s", crate::widgets::format_number(ev.time)));
                        }

                        let color = if ev.broken() {
                            theme::ERROR
                        } else if ev.dynamic {
                            theme::INFO
                        } else {
                            ui.visuals().text_color()
                        };
                        let hint = if ev.dynamic {
                            "Found when the map runs, not followed further"
                        } else if ev.broken() {
                            "No entity has this targetname"
                        } else {
                            "Selects this connection"
                        };
                        if ui.selectable_label(false, RichText::new(text).color(color)).on_hover_text(hint).clicked() {
                            pick = Some((ev.source, ev.connection));
                        }
                    }
                });
            });
        },
    );
    if clear {
        gs.sim = None;
    }

    if let Some(id) = pick {
        gs.selected_edge = Some(id);
        select(state, &[id.0], false);
    }
}
