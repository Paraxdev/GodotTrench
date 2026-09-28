//! The Logic panel: a pan and zoom canvas that draws the graph model and turns clicks and drags into map edits.

use std::collections::{BTreeSet, HashSet};

use egui::epaint::CubicBezierShape;
use egui::{Align2, Color32, FontId, Id, Order, PointerButton, Pos2, Rect, RichText, Sense, Shape, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};
use gt_core::{DVec3, NodeId};
use gt_doc::IoConnection;
use gt_doc::issues::BUILTIN_INPUTS;
use gt_doc::map::GraphFrame;

use super::edit::{self, NewLink, Placement};
use super::model::{ConnectionId, EdgeKind, GraphEdge, GraphModel, GraphNode, HEADER, Inputs, NodeKey, PIN_ROW, PinType, SETTING_ROW, node_title, pin_label};
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
/// The least a graph opens at, where names can still be read.
const READABLE_ZOOM: f32 = 0.6;
/// Titles stay at least this big when zoomed out, clipped to their node, so an overview still says what is what.
const MIN_TITLE: f32 = 10.0;
/// Height of the simulation's step list below the canvas.
const SIM_STRIP: f32 = 140.0;
/// Other text shrinks with the zoom only down to this share of its size.
const MIN_TEXT: f32 = 0.7;

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
    /// The title bar of a frame.
    Frame(usize),
    /// The corner that resizes a frame.
    FrameCorner(usize),
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
    /// Moving a frame and the nodes inside it.
    Frame {
        index: usize,
        keys: HashSet<NodeKey>,
        offset: Vec2,
    },
    FrameSize {
        index: usize,
        size: Vec2,
    },
    Pan,
}

/// Height of a frame's title bar in graph points.
const FRAME_TITLE: f32 = 30.0;
const FRAME_MIN: Vec2 = vec2(120.0, 60.0);

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
    pub frames: Vec<GraphFrame>,
    pub selected_frame: Option<usize>,
    /// The frame whose title is being edited, and the text so far.
    renaming: Option<(usize, String)>,
    drag: Option<Drag>,
    menu: Option<Menu>,
    menu_filter: String,
    /// The highlighted row of the search list, moved by the arrow keys.
    menu_cursor: usize,
    menu_opened: u64,
    pub sim: Option<Simulation>,
    hovered: bool,
    canvas: Rect,
    /// The view follows the panel's size until the user zooms or pans.
    navigated: bool,
    fitted: Vec2,
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
            frames: Vec::new(),
            selected_frame: None,
            renaming: None,
            drag: None,
            menu: None,
            menu_filter: String::new(),
            menu_cursor: 0,
            menu_opened: 0,
            sim: None,
            hovered: false,
            canvas: Rect::NOTHING,
            navigated: false,
            fitted: Vec2::ZERO,
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
            self.navigated = false;
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
        self.frames = state.doc.map.editor.graph_frames.clone();
        if self.selected_frame.is_some_and(|i| i >= self.frames.len()) {
            self.selected_frame = None;
        }

        if self.renaming.as_ref().is_some_and(|(i, _)| *i >= self.frames.len()) {
            self.renaming = None;
        }

        let map = &state.doc.map;
        if let Some((id, i)) = self.selected_edge
            && (map.entity(id).is_none_or(|e| i >= e.outputs.len()) || !selected.contains(&id))
        {
            self.selected_edge = None;
        }

        if let Some(sim) = &mut self.sim {
            if map.entity(sim.start).is_some() {
                sim.result = logic_sim::simulate(map, &state.game, sim.start, &sim.output);
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
            Some(Drag::Nodes { keys, offset } | Drag::Frame { keys, offset, .. }) if keys.contains(&n.key) => n.pos + *offset,
            _ => n.pos,
        }
    }

    /// A frame's rect in graph points, following a drag in progress.
    fn frame_graph_rect(&self, i: usize) -> Rect {
        let r = self.frames[i].rect;
        let (mut min, mut size) = (pos2(r[0], r[1]), vec2(r[2], r[3]));
        match &self.drag {
            Some(Drag::Frame { index, offset, .. }) if *index == i => min += *offset,
            Some(Drag::FrameSize { index, size: s }) if *index == i => size = *s,
            _ => {}
        }

        Rect::from_min_size(min, size)
    }

    fn frame_rect(&self, i: usize) -> Rect {
        let r = self.frame_graph_rect(i);
        Rect::from_min_max(self.to_screen(r.min), self.to_screen(r.max))
    }

    fn frame_title_height(&self) -> f32 {
        (FRAME_TITLE * self.zoom).max(18.0)
    }

    /// Entity nodes whose middle lies inside frame `i`, which move with it.
    fn framed(&self, i: usize) -> HashSet<NodeKey> {
        let r = self.frame_graph_rect(i);
        self.model.nodes.iter().filter(|n| n.key.entity().is_some() && r.contains(n.rect().center())).map(|n| n.key.clone()).collect()
    }

    /// Entity nodes of the selected entities.
    fn selected_nodes(&self, state: &EditorState) -> Vec<&GraphNode> {
        let selected = selected_entities(state);
        self.model.nodes.iter().filter(|n| n.key.entity().is_some_and(|id| selected.contains(&id))).collect()
    }

    /// Puts a new frame behind the selected nodes and starts editing its title.
    fn frame_selection(&mut self, state: &mut EditorState) {
        let nodes = self.selected_nodes(state);
        if nodes.is_empty() {
            return;
        }

        let bounds = nodes.iter().fold(Rect::NOTHING, |r, n| r.union(n.rect())).expand(24.0);
        let bounds = Rect::from_min_max(bounds.min - vec2(0.0, FRAME_TITLE), bounds.max);
        let used: Vec<Color32> = self.frames.iter().map(|f| color32(f.color)).collect();
        let color = theme::FRAME_COLORS.into_iter().find(|c| !used.contains(c)).unwrap_or(theme::FRAME_COLORS[self.frames.len() % theme::FRAME_COLORS.len()]);
        let frame = GraphFrame { title: "Frame".into(), rect: [bounds.min.x, bounds.min.y, bounds.width(), bounds.height()], color: doc_color(color) };
        let index = self.frames.len();
        edit::edit_frames(state, "Add Frame", &[], &self.unpinned(), |frames| frames.push(frame));
        self.selected_frame = Some(index);
        self.selected_edge = None;
        self.renaming = Some((index, "Frame".into()));
    }

    fn node_rect(&self, n: &GraphNode) -> Rect {
        Rect::from_min_size(self.to_screen(self.node_pos(n)), n.size * self.zoom)
    }

    fn pin_screen(&self, n: &GraphNode, output: bool, pin: usize) -> Pos2 {
        let p = self.node_pos(n);
        let x = if output { p.x + n.size.x } else { p.x };
        self.to_screen(pos2(x, p.y + n.head + PIN_ROW * (pin as f32 + 0.5)))
    }

    /// The pointer is over the graph, so keys act on it.
    pub fn hovered(&self) -> bool {
        self.hovered
    }

    /// Screen position of a pin as last drawn, for tests that drive the canvas with the pointer.
    pub fn pin_pos(&self, key: &NodeKey, output: bool, name: &str) -> Option<Pos2> {
        let n = self.model.node(key)?;
        let pin = if output { n.output(name) } else { n.input(name) }?;
        Some(self.pin_screen(n, output, pin))
    }

    /// Screen rect of a frame as last drawn.
    pub fn frame_screen_rect(&self, i: usize) -> Option<Rect> {
        (i < self.frames.len()).then(|| self.frame_rect(i))
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

    /// Zooms to show every node, but not below `min`, where the view shows the top left of the graph instead.
    fn fit(&mut self, min: f32) {
        let bounds = self.model.nodes.iter().fold(Rect::NOTHING, |r, n| r.union(n.rect()));
        if !bounds.is_positive() || !self.canvas.is_positive() {
            self.zoom = 1.0;
            self.pan = vec2(24.0, 24.0);
            return;
        }

        let room = self.canvas.shrink(24.0);
        self.zoom = (room.width() / bounds.width()).min(room.height() / bounds.height()).clamp(min, 1.0);
        self.pan = room.center() - self.canvas.min - bounds.center().to_vec2() * self.zoom;
        if bounds.width() * self.zoom > room.width() {
            self.pan.x = room.min.x - self.canvas.min.x - bounds.min.x * self.zoom;
        }

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

        if let Some(i) = best.1 {
            return Hit::Edge(i);
        }

        for i in (0..self.frames.len()).rev() {
            let r = self.frame_rect(i);
            if r.right_bottom().distance(p) <= PIN_GRAB * 1.2 {
                return Hit::FrameCorner(i);
            }

            if Rect::from_min_size(r.min, vec2(r.width(), self.frame_title_height())).contains(p) {
                return Hit::Frame(i);
            }
        }

        Hit::Empty
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
        let top = self.node_rect(n).min.y + n.head * self.zoom;
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

        let busy = self.menu.is_some() || self.drag.is_some() || self.selected_edge.is_some() || self.selected_frame.is_some();
        if busy && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            if self.menu.take().is_none() && self.drag.take().is_none() {
                self.selected_edge = None;
                self.selected_frame = None;
            }

            return true;
        }

        if !self.selected_nodes(state).is_empty() && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::C)) {
            self.frame_selection(state);
            return true;
        }

        if let Some(i) = self.selected_frame {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F2)) {
                self.renaming = Some((i, self.frames[i].title.clone()));
                return true;
            }

            let delete =
                ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Delete) || i.consume_key(egui::Modifiers::NONE, egui::Key::Backspace));
            if delete {
                edit::edit_frames(state, "Delete Frame", &[], &[], |frames| {
                    frames.remove(i);
                });
                self.selected_frame = None;
            }

            return delete;
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
        let result = logic_sim::simulate(&state.doc.map, &state.game, start, output);
        self.sim = Some(Simulation { start, output: output.to_string(), result });
    }

    fn open_menu(&mut self, ctx: &egui::Context, menu: Menu) {
        self.menu = Some(menu);
        self.menu_filter.clear();
        self.menu_cursor = 0;
        self.menu_opened = ctx.cumulative_pass_nr();
    }

    /// Graph positions of the wired entity nodes that still follow the automatic layout, stored along with a first
    /// hand placed node so the rest of the picture holds still.
    fn unpinned(&self) -> Vec<(NodeId, Pos2)> {
        self.unpinned_with(&[])
    }

    /// [`Self::unpinned`] plus the nodes of `keys`, such as both ends of a new wire.
    fn unpinned_with(&self, keys: &[&NodeKey]) -> Vec<(NodeId, Pos2)> {
        let pinned = |n: &&GraphNode| n.stored.is_none() && (!n.unwired || keys.contains(&&n.key));
        self.model.nodes.iter().filter(pinned).filter_map(|n| Some((n.key.entity()?, n.pos))).collect()
    }

    /// A graph position near `p` where a new node overlaps none.
    fn free_spot(&self, p: Pos2) -> Pos2 {
        let taken: Vec<Rect> = self.model.nodes.iter().map(|n| n.rect()).collect();
        super::model::clear_spot(p, vec2(200.0, HEADER + PIN_ROW * 4.0 + 8.0), &taken)
    }
}

fn color32(c: gt_core::Color) -> Color32 {
    let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Color32::from_rgb(b(c.r), b(c.g), b(c.b))
}

fn doc_color(c: Color32) -> gt_core::Color {
    gt_core::Color::rgb(c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0)
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
/// A category of the Add menu and its classes with their descriptions.
type Group = (String, Vec<(String, String)>);

/// Classes the Add menu offers, point entities that take part in I/O, by category with logic and math first.
fn creatable(state: &EditorState) -> Vec<Group> {
    let mut groups: std::collections::BTreeMap<(u8, String), Vec<(String, String)>> = Default::default();
    for d in state.game.point_entities() {
        let helper = d.classname.starts_with("logic_") || d.classname.starts_with("math_");
        if !helper && d.inputs.is_empty() && d.outputs.is_empty() {
            continue;
        }

        let category = super::model::category(&state.game, &d.classname);
        let rank = match category.as_str() {
            "logic" => 0,
            "math" => 1,
            _ => 2,
        };
        groups.entry((rank, category)).or_default().push((d.classname.clone(), d.description.clone()));
    }

    groups
        .into_iter()
        .map(|((_, category), mut classes)| {
            classes.sort();
            (category, classes)
        })
        .collect()
}

fn pack_missing(state: &EditorState) -> bool {
    !state.game.entities.iter().any(|d| crate::entity_pack::in_pack(&d.classname))
}

pub fn show(ui: &mut Ui, state: &mut EditorState, gs: &mut GraphState, actions: &mut Vec<Action>) {
    gs.refresh(state);
    toolbar(ui, state, gs, actions);
    let room = ui.available_rect_before_wrap();
    let strip = if gs.sim.is_some() { SIM_STRIP.min(room.height() * 0.4) } else { 0.0 };
    let canvas = Rect::from_min_size(room.min, (room.size() - vec2(0.0, strip)).max(vec2(120.0, 120.0)));
    let response = ui.allocate_rect(canvas, Sense::click_and_drag());
    gs.canvas = canvas;
    let resized = !gs.navigated && (gs.fitted - canvas.size()).length() > 1.0;
    if (gs.refit || resized) && !gs.model.nodes.is_empty() {
        gs.fit(READABLE_ZOOM);
        gs.refit = false;
        gs.fitted = canvas.size();
    }

    gs.hovered = response.hovered() || response.dragged() || gs.drag.is_some();
    navigate(ui, &response, gs);
    let visible: Vec<usize> = (0..gs.model.nodes.len()).filter(|i| gs.node_rect(&gs.model.nodes[*i]).intersects(canvas)).collect();
    let hover = response.hover_pos().filter(|_| gs.drag.is_none() && gs.menu.is_none()).map(|p| gs.hit(p, &visible));
    interact(ui, &response, state, gs, actions, &visible);
    gs.refresh(state);
    let visible: Vec<usize> = (0..gs.model.nodes.len()).filter(|i| gs.node_rect(&gs.model.nodes[*i]).intersects(canvas)).collect();
    draw(ui, state, gs, &visible, hover.as_ref());
    if let Some(Hit::Pin(p)) = &hover
        && let Some(n) = gs.model.node(&p.key)
        && let Some(pin) = if p.output { n.output(&p.name).map(|i| &n.outputs[i]) } else { n.input(&p.name).map(|i| &n.inputs[i]) }
    {
        let kind = if p.output { "output" } else { "input" };
        let mut tip = format!("{} {kind}, {}", pin_label(&pin.name), pin.ty.name());
        if !pin.values.is_empty() {
            tip.push_str(&format!(" ({})", pin.values));
        }

        if !pin.description.is_empty() {
            tip.push_str(&format!("\n{}", pin.description));
        } else if !pin.declared {
            tip.push_str("\nNot in the entity definition, used by the map");
        }

        response.clone().on_hover_text_at_pointer(tip);
    }

    if strip > 0.0 {
        sim_list(ui, Rect::from_min_max(pos2(room.min.x, canvas.max.y + 4.0), room.max), state, gs);
    }

    overlays(ui, state, gs, actions);
}

fn toolbar(ui: &mut Ui, state: &mut EditorState, gs: &mut GraphState, actions: &mut Vec<Action>) {
    ui.horizontal_wrapped(|ui| {
        ui.menu_button("Add", |ui| {
            let at = gs.free_spot(gs.to_graph(gs.canvas.center()) - vec2(100.0, HEADER));
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
            gs.fit(MIN_ZOOM);
        }

        let tip = "Puts a titled box behind the selected nodes, C over the graph does the same. Dragging its title moves everything inside";
        let framed = ui.add_enabled(!gs.selected_nodes(state).is_empty(), egui::Button::new("Frame")).on_hover_text(tip).on_disabled_hover_text(tip);
        if framed.clicked() {
            gs.frame_selection(state);
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
            gs.navigated = true;
            let anchor = gs.to_graph(p);
            gs.zoom = (gs.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
            gs.pan = p - gs.canvas.min - anchor.to_vec2() * gs.zoom;
        }
    }

    if response.dragged_by(PointerButton::Middle) || response.dragged_by(PointerButton::Secondary) {
        gs.navigated = true;
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
            Hit::Frame(index) => {
                gs.selected_frame = Some(index);
                Some(Drag::Frame { index, keys: gs.framed(index), offset: Vec2::ZERO })
            }
            Hit::FrameCorner(index) => Some(Drag::FrameSize { index, size: gs.frame_graph_rect(index).size() }),
            Hit::Edge(_) | Hit::Empty => Some(Drag::Box { start: origin, at: origin, add }),
        };
    }

    if let Some(p) = pointer {
        match &mut gs.drag {
            Some(Drag::Wire { at, .. }) | Some(Drag::Box { at, .. }) => *at = p,
            Some(Drag::Nodes { offset, .. } | Drag::Frame { offset, .. }) => *offset += response.drag_delta() / gs.zoom,
            Some(Drag::FrameSize { size, .. }) => *size += response.drag_delta() / gs.zoom,
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
            Drag::Frame { index, keys, offset } if offset.length() >= 0.5 => {
                let moved: Vec<(NodeId, Pos2)> =
                    gs.model.nodes.iter().filter(|n| keys.contains(&n.key)).filter_map(|n| Some((n.key.entity()?, n.pos + offset))).collect();
                edit::edit_frames(state, "Move Frame", &moved, &gs.unpinned(), |frames| {
                    if let Some(f) = frames.get_mut(index) {
                        f.rect[0] += offset.x;
                        f.rect[1] += offset.y;
                    }
                });
            }
            Drag::FrameSize { index, size } => {
                let size = size.max(FRAME_MIN);
                edit::edit_frames(state, "Resize Frame", &[], &gs.unpinned(), |frames| {
                    if let Some(f) = frames.get_mut(index) {
                        (f.rect[2], f.rect[3]) = (size.x, size.y);
                    }
                });
            }
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
        match gs.hit(p, visible) {
            Hit::Node(NodeKey::Entity(id)) => {
                select(state, &[id], false);
                actions.push(Action::FocusSelection);
            }
            Hit::Frame(i) => gs.renaming = Some((i, gs.frames[i].title.clone())),
            _ => {}
        }
    } else if response.clicked() {
        let hit = gs.hit(p, visible);
        if !matches!(hit, Hit::Frame(_) | Hit::FrameCorner(_)) {
            gs.selected_frame = None;
        }

        match hit {
            Hit::Frame(i) | Hit::FrameCorner(i) => {
                gs.selected_frame = Some(i);
                gs.selected_edge = None;
            }
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
            Hit::Empty => Menu::Create { screen: p, at: gs.free_spot(gs.to_graph(p)), link: None, near: selected_entities(state).into_iter().next() },
            hit => Menu::Context { screen: p, hit },
        };
        gs.open_menu(&ctx, menu);
    }
}

fn finish_wire(ctx: &egui::Context, state: &mut EditorState, gs: &mut GraphState, from: PinRef, at: Pos2, visible: &[usize]) {
    if let Some(to) = gs.drop_pin(&from, at, visible) {
        let (out, input) = if from.output { (&from, &to) } else { (&to, &from) };
        if let Some(source) = out.key.entity()
            && let Err(e) = edit::connect(state, source, &out.name, &input.key, &input.name, &gs.unpinned_with(&[&out.key, &input.key]))
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
            let at = if from.output { graph - vec2(0.0, HEADER + PIN_ROW * 0.5) } else { graph - vec2(200.0, HEADER + PIN_ROW * 0.5) };
            let at = gs.free_spot(at);
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

    for i in 0..gs.frames.len() {
        let r = gs.frame_rect(i);
        if !r.intersects(canvas) {
            continue;
        }

        let color = color32(gs.frames[i].color);
        let chosen = gs.selected_frame == Some(i);
        painter.rect_filled(r, 6.0, color.gamma_multiply(0.08));
        let title = Rect::from_min_size(r.min, vec2(r.width(), gs.frame_title_height()));
        painter.rect_filled(title, egui::CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 }, color.gamma_multiply(0.35));
        painter.rect_stroke(r, 6.0, Stroke::new(if chosen { 2.5 } else { 1.5 }, if chosen { color } else { color.gamma_multiply(0.7) }), StrokeKind::Inside);
        let font = FontId::proportional((15.0 * gs.zoom).max(MIN_TITLE));
        painter.with_clip_rect(title.intersect(canvas)).text(
            title.left_center() + vec2(10.0, 0.0),
            Align2::LEFT_CENTER,
            &gs.frames[i].title,
            font,
            theme::GRAY_7,
        );
        let corner = r.right_bottom();
        let grip = 10.0;
        painter.add(Shape::convex_polygon(vec![corner, corner - vec2(grip, 0.0), corner - vec2(0.0, grip)], color.gamma_multiply(0.7), Stroke::NONE));
    }

    let selected = selected_entities(state);
    let sim_steps = |id: ConnectionId| -> Vec<String> {
        gs.sim
            .as_ref()
            .map(|s| {
                let steps = s.result.events.iter().enumerate().filter(|(_, e)| (e.source, e.connection) == id);
                steps.map(|(i, e)| if e.possible() { format!("{}?", i + 1) } else { (i + 1).to_string() }).collect()
            })
            .unwrap_or_default()
    };
    let sim_nodes: BTreeSet<NodeId> =
        gs.sim.as_ref().map(|s| s.result.events.iter().flat_map(|e| std::iter::once(e.source).chain(e.resolved.iter().copied())).collect()).unwrap_or_default();
    let detail = gs.zoom >= DETAIL_ZOOM;
    // Text scales with the zoom, but never below a size that can still be read.
    let font = |size: f32| FontId::proportional((size * gs.zoom).max(size * MIN_TEXT));

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
        } else if e.kind == EdgeKind::Resolved {
            let pin = gs.model.nodes[e.from].outputs.get(e.from_pin).map_or(PinType::Pulse, |p| p.ty);
            (1.6, theme::pin_type_color(pin.name()))
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
            badges.push((mid, steps.join(",")));
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
        let category = (!n.category.is_empty()).then(|| theme::category_color(&n.category));
        if let Some(c) = category {
            let strip = Rect::from_min_size(r.min, vec2(r.width(), 5.0 * gs.zoom.max(0.6)));
            painter.rect_filled(strip, egui::CornerRadius { nw: rounding as u8, ne: rounding as u8, sw: 0, se: 0 }, c);
        }

        let clip = painter.with_clip_rect(r.shrink(2.0).intersect(canvas));
        let text = |at: Pos2, align: Align2, s: &str, size: f32, color: Color32| {
            clip.text(at, align, s, font(size), color);
        };
        let title = FontId::proportional((14.0 * gs.zoom).max(MIN_TITLE));
        clip.text(header.min + vec2(10.0, 7.0) * gs.zoom.max(0.4), Align2::LEFT_TOP, &n.title, title, theme::GRAY_7);
        if detail {
            text(header.min + vec2(10.0, 25.0) * gs.zoom, Align2::LEFT_TOP, &n.subtitle, 11.0, marker.or(category).unwrap_or(theme::GRAY_5));
            for (row, line) in n.settings.iter().enumerate() {
                text(header.min + vec2(10.0, HEADER + SETTING_ROW * row as f32 + 1.0) * gs.zoom, Align2::LEFT_TOP, line, 11.0, theme::GRAY_6);
            }
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
                let color = theme::pin_type_color(pin.ty.name());
                let radius = PIN_RADIUS * gs.zoom.max(0.6) * if hot { 1.4 } else { 1.0 };
                let (fill, stroke) = if connected(output, k) { (color, Stroke::NONE) } else { (Color32::TRANSPARENT, Stroke::new(1.5, color)) };
                // A pulse is a triangle pointing the way the signal flows, a value a circle, so the shape tells them
                // apart as well as the color.
                if pin.ty == PinType::Pulse {
                    let r = radius * 1.2;
                    let points = vec![at + vec2(-r * 0.8, -r), at + vec2(r, 0.0), at + vec2(-r * 0.8, r)];
                    painter.add(Shape::convex_polygon(points, fill, stroke));
                } else {
                    painter.circle(at, radius, fill, stroke);
                }

                if detail {
                    let warn = n.defined && !pin.declared;
                    let label_color = if warn { theme::WARNING } else { theme::FG };
                    let (align, dx) = if output { (Align2::RIGHT_CENTER, -10.0) } else { (Align2::LEFT_CENTER, 10.0) };
                    text(at + vec2(dx * gs.zoom, 0.0), align, pin_label(&pin.name), 13.0, label_color);
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

    if gs.renaming.is_some() {
        rename_frame(&ctx, state, gs);
    }

    let Some(menu) = gs.menu.take() else { return };
    let screen = match &menu {
        Menu::Create { screen, .. } | Menu::Pin { screen, .. } | Menu::Context { screen, .. } => *screen,
    };
    let mut keep = true;
    // Every menu is a new area. A reused one keeps the size of the last menu, so after a short list the next one would
    // scroll in a few rows.
    let area = egui::Area::new(Id::new(("logic_graph_menu", gs.menu_opened))).order(Order::Foreground).fixed_pos(screen).constrain_to(ctx.content_rect()).show(
        &ctx,
        |ui| {
            egui::Frame::menu(ui.style()).show(ui, |ui| {
                keep = match &menu {
                    Menu::Create { at, link, near, .. } => !create_list(ui, state, gs, *at, link.clone(), *near, actions),
                    Menu::Pin { from, to, .. } => !pin_list(ui, state, gs, from, to),
                    Menu::Context { hit, .. } => !context_menu(ui, state, gs, hit, actions),
                };
            });
        },
    );
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

/// One class of the Add menu.
struct Entry {
    category: String,
    classname: String,
    description: String,
}

/// How well a class fits the search text, lower is better: the classname or one of its words starts with it, the
/// classname contains it, the category is it, the description contains it. None when it does not match at all.
fn match_rank(e: &Entry, filter: &str) -> Option<u8> {
    let name = e.classname.to_lowercase();
    if name.starts_with(filter) || name.split('_').any(|w| w.starts_with(filter)) {
        Some(0)
    } else if name.contains(filter) {
        Some(1)
    } else if e.category.contains(filter) {
        Some(2)
    } else if e.description.to_lowercase().contains(filter) {
        Some(3)
    } else {
        None
    }
}

/// The classes to list for the search text. Without any they follow the categories of the Add menu, with some they
/// are sorted by how well they match, so the first row is the best guess.
fn listed(groups: Vec<Group>, filter: &str) -> Vec<Entry> {
    let all = groups
        .into_iter()
        .flat_map(|(category, classes)| classes.into_iter().map(move |(classname, description)| Entry { category: category.clone(), classname, description }));
    if filter.is_empty() {
        return all.collect();
    }

    let mut ranked: Vec<(u8, Entry)> = all.filter_map(|e| Some((match_rank(&e, filter)?, e))).collect();
    ranked.sort_by_key(|(rank, _)| *rank);
    ranked.into_iter().map(|(_, e)| e).collect()
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

    let filter = gs.menu_filter.trim().to_lowercase();
    let entries = listed(creatable(state), &filter);
    if field.changed() {
        gs.menu_cursor = 0;
    }

    let last = entries.len().saturating_sub(1);
    let mut moved = false;
    if field.has_focus() && !entries.is_empty() {
        let (down, up) =
            ui.input_mut(|i| (i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown), i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)));
        moved = down || up;
        gs.menu_cursor = if down {
            gs.menu_cursor + 1
        } else if up {
            gs.menu_cursor.saturating_sub(1)
        } else {
            gs.menu_cursor
        };
    }

    gs.menu_cursor = gs.menu_cursor.min(last);
    let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let mut chosen = enter.then(|| entries.get(gs.menu_cursor).map(|e| e.classname.clone())).flatten();
    if pack_missing(state) {
        let asked = actions.len();
        install_hint(ui, actions);
        if actions.len() > asked {
            ui.close();
            return true;
        }
    }

    let height = (ui.ctx().content_rect().height() - 90.0).clamp(120.0, 320.0);
    egui::ScrollArea::vertical().max_height(height).show(ui, |ui| {
        let mut heading = None;
        for (i, e) in entries.iter().enumerate() {
            if filter.is_empty() && heading != Some(&e.category) {
                heading = Some(&e.category);
                ui.label(RichText::new(&e.category).strong().color(theme::category_color(&e.category)));
            }

            let row = ui.horizontal(|ui| {
                if filter.is_empty() {
                    ui.add_space(ui.spacing().indent);
                }

                let button = ui.selectable_label(i == gs.menu_cursor, &e.classname);
                if !filter.is_empty() {
                    ui.label(RichText::new(&e.category).small().color(theme::category_color(&e.category)));
                }

                button
            });
            let button = if e.description.is_empty() { row.inner } else { row.inner.on_hover_text(&e.description) };
            if moved && i == gs.menu_cursor {
                button.scroll_to_me(None);
            }

            if button.clicked() {
                chosen = Some(e.classname.clone());
            }
        }

        if entries.is_empty() {
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
        && let Err(e) = edit::connect(state, source, &out.name, &input.key, &input.name, &gs.unpinned_with(&[&out.key, &input.key]))
    {
        state.set_status(e);
    }

    true
}

/// The title field over a frame being renamed. Enter or a click elsewhere keeps the text, Escape drops it.
fn rename_frame(ctx: &egui::Context, state: &mut EditorState, gs: &mut GraphState) {
    let Some((i, mut text)) = gs.renaming.take() else { return };
    let r = gs.frame_rect(i);
    let mut done = None;
    egui::Area::new(Id::new("logic_frame_title")).order(Order::Foreground).fixed_pos(r.min + vec2(4.0, 2.0)).show(ctx, |ui| {
        let field = ui.add(egui::TextEdit::singleline(&mut text).desired_width((r.width() - 8.0).clamp(80.0, 400.0)));
        if !field.has_focus() && !field.lost_focus() {
            field.request_focus();
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            done = Some(false);
        } else if field.lost_focus() {
            done = Some(true);
        }
    });
    match done {
        Some(true) => {
            let title = text.trim().to_string();
            if !title.is_empty() && gs.frames.get(i).is_some_and(|f| f.title != title) {
                edit::edit_frames(state, "Rename Frame", &[], &[], |frames| frames[i].title = title);
            }
        }
        Some(false) => {}
        None => gs.renaming = Some((i, text)),
    }
}

fn frame_menu(ui: &mut Ui, state: &mut EditorState, gs: &mut GraphState, i: usize) -> bool {
    if i >= gs.frames.len() {
        return true;
    }

    if ui.button("Rename Frame").clicked() {
        gs.renaming = Some((i, gs.frames[i].title.clone()));
        return true;
    }

    let mut picked = None;
    ui.horizontal(|ui| {
        for c in theme::FRAME_COLORS {
            let (rect, response) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::click());
            ui.painter().rect_filled(rect, 3.0, c);
            if color32(gs.frames[i].color) == c {
                ui.painter().rect_stroke(rect, 3.0, Stroke::new(2.0, theme::GRAY_7), StrokeKind::Outside);
            }

            if response.on_hover_text("Frame color").clicked() {
                picked = Some(c);
            }
        }
    });
    if let Some(c) = picked {
        edit::edit_frames(state, "Recolor Frame", &[], &[], |frames| frames[i].color = doc_color(c));
        return true;
    }

    if ui.button("Delete Frame").on_hover_text("Removes the frame, the nodes stay").clicked() {
        edit::edit_frames(state, "Delete Frame", &[], &[], |frames| {
            frames.remove(i);
        });
        gs.selected_frame = None;
        return true;
    }

    false
}

fn context_menu(ui: &mut Ui, state: &mut EditorState, gs: &mut GraphState, hit: &Hit, actions: &mut Vec<Action>) -> bool {
    let key = match hit {
        Hit::Frame(i) | Hit::FrameCorner(i) => return frame_menu(ui, state, gs, *i),
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
    let source_name = node_title(&state.doc.map, id.0);
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

/// The steps of a simulation, in a strip below the canvas so they never cover the highlighted wires.
fn sim_list(ui: &mut Ui, rect: Rect, state: &mut EditorState, gs: &mut GraphState) {
    let Some(sim) = &gs.sim else { return };
    let name = node_title(&state.doc.map, sim.start);
    let mut clear = false;
    let mut pick = None;
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
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

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            for (i, ev) in sim.result.events.iter().enumerate() {
                let mut text = format!("{}. {}.{} -> {}.{}", i + 1, ev.source_name, ev.output, ev.target, ev.input);
                if ev.time > 0.0 {
                    text.push_str(&format!("  at {}s", crate::widgets::format_number(ev.time)));
                }

                if ev.possible() {
                    text.push_str("  (maybe)");
                }

                let color = if ev.broken() {
                    theme::ERROR
                } else if ev.dynamic {
                    theme::INFO
                } else if ev.possible() {
                    theme::GRAY_6
                } else {
                    ui.visuals().text_color()
                };
                let mut hint = if ev.dynamic {
                    "Found when the map runs, not followed further".to_string()
                } else if ev.broken() {
                    "No entity has this targetname".to_string()
                } else {
                    "Selects this connection".to_string()
                };
                if let Some(why) = &ev.condition {
                    hint = format!("{why}. {hint}");
                }

                if ui.selectable_label(false, RichText::new(text).color(color)).on_hover_text(hint).clicked() {
                    pick = Some((ev.source, ev.connection));
                }
            }
        });
    });
    if clear {
        gs.sim = None;
    }

    if let Some(id) = pick {
        gs.selected_edge = Some(id);
        select(state, &[id.0], false);
    }
}
