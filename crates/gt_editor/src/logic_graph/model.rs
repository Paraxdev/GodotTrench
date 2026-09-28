//! The graph as a projection of the map's entity I/O. Nothing in it is stored: the outputs on the entities are the only
//! source of truth, and the model is built again from them whenever the map changes.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use egui::{Pos2, Vec2, pos2, vec2};
use gt_core::NodeId;
use gt_doc::issues::{BUILTIN_INPUTS, io_name_matches, is_dynamic_target};
use gt_doc::{IoConnection, Map};
use gt_formats::GameConfig;

/// What a graph node stands for.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NodeKey {
    Entity(NodeId),
    /// A target only the running game resolves: `!player`, `@group`, a node path or a wildcard.
    Dynamic(String),
    /// A node the Godot overlay adds on top of the built map.
    Overlay(String),
    /// A targetname no entity has, or an empty target.
    Missing(String),
}

impl NodeKey {
    pub fn entity(&self) -> Option<NodeId> {
        match self {
            NodeKey::Entity(id) => Some(*id),
            _ => None,
        }
    }

    /// The text an output targets to reach this node, None for an entity without a targetname.
    pub fn target<'a>(&'a self, map: &'a Map) -> Option<&'a str> {
        match self {
            NodeKey::Entity(id) => map.entity(*id).and_then(|e| e.targetname()),
            NodeKey::Dynamic(t) | NodeKey::Overlay(t) | NodeKey::Missing(t) => Some(t),
        }
    }
}

/// What a pin carries, Blueprint style: a pulse is only an event, the others pass a value along.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PinType {
    Pulse,
    Bool,
    Int,
    Float,
    String,
    Vector3,
    Color,
    Node,
    Variant,
}

impl PinType {
    pub const ALL: [PinType; 9] =
        [PinType::Pulse, PinType::Bool, PinType::Int, PinType::Float, PinType::String, PinType::Vector3, PinType::Color, PinType::Node, PinType::Variant];

    pub fn name(self) -> &'static str {
        match self {
            PinType::Pulse => "pulse",
            PinType::Bool => "bool",
            PinType::Int => "int",
            PinType::Float => "float",
            PinType::String => "string",
            PinType::Vector3 => "vector3",
            PinType::Color => "color",
            PinType::Node => "node",
            PinType::Variant => "variant",
        }
    }

    /// A definition's type name. Names this editor does not know still carry some value.
    pub fn parse(name: &str) -> Self {
        let name = name.trim().to_lowercase();
        PinType::ALL.into_iter().find(|t| t.name() == name).unwrap_or(if name.is_empty() { PinType::Pulse } else { PinType::Variant })
    }

    /// Whether a value of this type fits an input of type `to`. A pulse carries no value and takes any, a variant is any
    /// value, and numbers convert. Other pairs are no natural fit, though the wire can still be made.
    pub fn fits(self, to: PinType) -> bool {
        use PinType::*;
        self == to || matches!((self, to), (Pulse, _) | (_, Pulse) | (Variant, _) | (_, Variant) | (Int, Float) | (Float, Int) | (Bool, Int) | (Int, Bool))
    }

    /// A declared pin's type: a pulse without parameters, else the type of its first one.
    fn of(def: &gt_formats::game::IoDef) -> Self {
        def.parameter_types().first().map_or(PinType::Pulse, |t| PinType::parse(t))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Pin {
    pub name: String,
    /// From the entity definition. Other pins are names the map already uses.
    pub declared: bool,
    pub description: String,
    pub ty: PinType,
    /// The values it passes or takes with their types, like `activator: node`, empty for a pulse.
    pub values: String,
}

impl Pin {
    fn declared(def: &gt_formats::game::IoDef) -> Self {
        let names = def.parameter.split(',').map(str::trim).filter(|p| !p.is_empty());
        let values: Vec<String> = names.zip(def.parameter_types()).map(|(n, t)| format!("{n}: {t}")).collect();
        Pin { name: def.name.clone(), declared: true, description: def.description.clone(), ty: PinType::of(def), values: values.join(", ") }
    }

    fn used(name: &str, ty: PinType) -> Self {
        Pin { name: name.to_string(), declared: false, description: String::new(), ty, values: String::new() }
    }
}

/// A logic or math entity, which only exists for its wiring, so it always has a node.
pub fn is_helper(classname: &str) -> bool {
    classname.starts_with("logic_") || classname.starts_with("math_")
}

/// What an entity is called in the graph: its targetname, else the name it was given in the Outliner, else its classname.
pub fn node_title(map: &Map, id: NodeId) -> String {
    let label = map.get(id).and_then(|n| n.label.clone());
    match map.entity(id) {
        Some(e) => e.targetname().map(str::to_string).or(label).unwrap_or_else(|| e.classname.clone()),
        None => label.unwrap_or_default(),
    }
}

/// The category a node is colored and grouped by: the definition's group, else the classname's first word, such as
/// logic for logic_relay.
pub fn category(game: &GameConfig, classname: &str) -> String {
    match game.entity(classname).map(|d| d.group.trim()).filter(|g| !g.is_empty()) {
        Some(group) => group.to_lowercase(),
        None => classname.split('_').next().unwrap_or(classname).to_lowercase(),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNode {
    pub key: NodeKey,
    pub title: String,
    /// The category and classname, or what kind of marker this is.
    pub subtitle: String,
    pub inputs: Vec<Pin>,
    pub outputs: Vec<Pin>,
    /// What the entity is grouped and colored by, empty for markers.
    pub category: String,
    /// The entity has a definition, so pins it does not declare are worth a warning.
    pub defined: bool,
    /// A logic_* or math_* helper entity, which only exists for its wiring.
    pub logic: bool,
    /// Shown only because it is selected, it has no wiring yet.
    pub unwired: bool,
    /// The position saved in the map.
    pub stored: Option<[f32; 2]>,
    /// Key settings under the title, up to [`SETTING_LINES`] lines: the properties set away from their defaults.
    pub settings: Vec<String>,
    /// Height of the title and settings, where the pin rows start.
    pub head: f32,
    /// Top left corner in graph space, the stored position or the automatic layout's.
    pub pos: Pos2,
    pub size: Vec2,
}

impl GraphNode {
    pub fn rect(&self) -> egui::Rect {
        egui::Rect::from_min_size(self.pos, self.size)
    }

    /// Graph space position of an input pin, on the left edge.
    pub fn input_pos(&self, pin: usize) -> Pos2 {
        pos2(self.pos.x, self.pos.y + self.head + PIN_ROW * (pin as f32 + 0.5))
    }

    pub fn output_pos(&self, pin: usize) -> Pos2 {
        pos2(self.pos.x + self.size.x, self.pos.y + self.head + PIN_ROW * (pin as f32 + 0.5))
    }

    pub fn input(&self, name: &str) -> Option<usize> {
        find_pin(&self.inputs, name)
    }

    pub fn output(&self, name: &str) -> Option<usize> {
        find_pin(&self.outputs, name)
    }
}

/// Height of a node's title area and of one pin row, in graph points.
pub const HEADER: f32 = 42.0;
pub const PIN_ROW: f32 = 22.0;
/// Height of one line of settings, and how many a node shows.
pub const SETTING_ROW: f32 = 14.0;
pub const SETTING_LINES: usize = 2;
const MIN_WIDTH: f32 = 160.0;
const MAX_WIDTH: f32 = 320.0;
/// Rough width of a character, the layout must not depend on fonts to stay the same everywhere.
const CHAR: f32 = 7.5;

fn find_pin(pins: &[Pin], name: &str) -> Option<usize> {
    pins.iter().position(|p| p.name == name).or_else(|| pins.iter().position(|p| io_name_matches(&p.name, name)))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EdgeKind {
    Resolved,
    Dynamic,
    Overlay,
    /// The target resolves to nothing.
    Broken,
}

/// One output connection to one of the nodes its target finds.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphEdge {
    pub source: NodeId,
    /// Index of the connection in the source entity's outputs.
    pub connection: usize,
    pub from: usize,
    pub from_pin: usize,
    pub to: usize,
    pub to_pin: usize,
    pub kind: EdgeKind,
    /// Delay, times and parameter when they are set, empty otherwise.
    pub label: String,
}

impl GraphEdge {
    pub fn id(&self) -> ConnectionId {
        (self.source, self.connection)
    }
}

/// An output connection: the entity that has it and its index among that entity's outputs.
pub type ConnectionId = (NodeId, usize);

#[derive(Clone, Debug, Default)]
pub struct GraphModel {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    index: HashMap<NodeKey, usize>,
}

/// Everything besides the map the projection reads.
pub struct Inputs<'a> {
    pub game: &'a GameConfig,
    /// Entities shown even without wiring, the selected ones.
    pub selected: &'a BTreeSet<NodeId>,
    /// Targetnames outside the map, from the Godot overlay.
    pub external: &'a BTreeSet<String>,
}

impl GraphModel {
    pub fn node(&self, key: &NodeKey) -> Option<&GraphNode> {
        self.index.get(key).map(|i| &self.nodes[*i])
    }

    pub fn index_of(&self, key: &NodeKey) -> Option<usize> {
        self.index.get(key).copied()
    }

    pub fn edges_of(&self, id: ConnectionId) -> impl Iterator<Item = &GraphEdge> {
        self.edges.iter().filter(move |e| e.id() == id)
    }

    pub fn build(map: &Map, cx: &Inputs) -> Self {
        let mut names: BTreeMap<&str, Vec<NodeId>> = BTreeMap::new();
        for (id, e) in map.entities() {
            if let Some(name) = e.targetname() {
                names.entry(name).or_default().push(id);
            }
        }

        let resolve = |target: &str| -> &[NodeId] { if is_dynamic_target(target) { &[] } else { names.get(target).map(Vec::as_slice).unwrap_or_default() } };

        let mut shown: BTreeSet<NodeId> = BTreeSet::new();
        let mut incoming: BTreeMap<NodeKey, Vec<(&str, PinType)>> = BTreeMap::new();
        for (id, e) in map.entities() {
            if !e.outputs.is_empty() || is_helper(&e.classname) {
                shown.insert(id);
            }

            for conn in &e.outputs {
                let targets = resolve(&conn.target);
                shown.extend(targets);
                for key in target_keys(&conn.target, targets, cx.external) {
                    let ty = if conn.parameter.is_empty() { PinType::Pulse } else { PinType::Variant };
                    incoming.entry(key).or_default().push((&conn.input, ty));
                }
            }
        }

        let wired = shown.clone();
        for id in cx.selected {
            let owner = map.owning_entity(*id).unwrap_or(*id);
            if map.entity(owner).is_some() {
                shown.insert(owner);
            }
        }

        let mut model = GraphModel::default();
        for id in &shown {
            let e = map.entity(*id).expect("shown ids are entities");
            let def = cx.game.entity(&e.classname);
            let mut outputs: Vec<Pin> = def.map(|d| d.outputs.iter().map(Pin::declared).collect()).unwrap_or_default();
            for conn in &e.outputs {
                if find_pin(&outputs, &conn.output).is_none() {
                    outputs.push(Pin::used(&conn.output, PinType::Pulse));
                }
            }

            let mut inputs: Vec<Pin> = def.map(|d| d.inputs.iter().map(Pin::declared).collect()).unwrap_or_default();
            for (name, ty) in incoming.get(&NodeKey::Entity(*id)).into_iter().flatten() {
                if find_pin(&inputs, name).is_none() {
                    inputs.push(Pin::used(name, *ty));
                }
            }

            let title = node_title(map, *id);
            let category = category(cx.game, &e.classname);
            let subtitle = match (title == e.classname, category == e.classname) {
                (true, true) => String::new(),
                (true, false) => category.clone(),
                (false, true) => e.classname.clone(),
                (false, false) => format!("{category} \u{b7} {}", e.classname),
            };
            let stored = map.get(*id).and_then(|n| n.graph);
            model.push(GraphNode {
                key: NodeKey::Entity(*id),
                title,
                subtitle,
                inputs,
                outputs,
                category,
                defined: def.is_some(),
                logic: is_helper(&e.classname),
                unwired: !wired.contains(id),
                stored,
                settings: def.map(|d| setting_tokens(e, d)).unwrap_or_default(),
                head: HEADER,
                pos: Pos2::ZERO,
                size: Vec2::ZERO,
            });
        }

        for (key, used) in &incoming {
            let subtitle = match key {
                NodeKey::Entity(_) => continue,
                NodeKey::Dynamic(t) => dynamic_kind(t),
                NodeKey::Overlay(_) => "Godot overlay node",
                NodeKey::Missing(t) if t.is_empty() => "output without a target",
                NodeKey::Missing(_) => "no entity has this name",
            };
            let mut inputs: Vec<Pin> = Vec::new();
            for (name, ty) in used {
                if find_pin(&inputs, name).is_none() {
                    inputs.push(Pin::used(name, *ty));
                }
            }

            let title = match key.target(map) {
                Some("") | None => "(no target)".to_string(),
                Some(t) => t.to_string(),
            };
            model.push(GraphNode {
                key: key.clone(),
                title,
                subtitle: subtitle.to_string(),
                inputs,
                outputs: Vec::new(),
                category: String::new(),
                defined: false,
                logic: false,
                unwired: false,
                stored: None,
                settings: Vec::new(),
                head: HEADER,
                pos: Pos2::ZERO,
                size: Vec2::ZERO,
            });
        }

        for (id, e) in map.entities() {
            let Some(&from) = model.index.get(&NodeKey::Entity(id)) else { continue };
            for (i, conn) in e.outputs.iter().enumerate() {
                let from_pin = model.nodes[from].output(&conn.output).unwrap_or(0);
                let targets = resolve(&conn.target);
                let label = edge_label(conn);
                for key in target_keys(&conn.target, targets, cx.external) {
                    let kind = match &key {
                        NodeKey::Entity(_) => EdgeKind::Resolved,
                        NodeKey::Dynamic(_) => EdgeKind::Dynamic,
                        NodeKey::Overlay(_) => EdgeKind::Overlay,
                        NodeKey::Missing(_) => EdgeKind::Broken,
                    };
                    let to = model.index[&key];
                    let to_pin = model.nodes[to].input(&conn.input).unwrap_or(0);
                    model.edges.push(GraphEdge { source: id, connection: i, from, from_pin, to, to_pin, kind, label: label.clone() });
                }
            }
        }

        for node in &mut model.nodes {
            node_size(node);
        }

        model
    }

    /// Gives every node its position: the stored one, or the automatic layout's. While some nodes have a stored
    /// position, the others keep their automatic offset to a wired neighbour that has one, so a new node shows up next
    /// to what it is wired to, and whatever is left goes below.
    pub fn place(&mut self) {
        let sizes: Vec<Vec2> = self.nodes.iter().map(|n| n.size).collect();
        let links: Vec<(usize, usize)> = self.edges.iter().map(|e| (e.from, e.to)).collect();
        let auto = super::layout::layered(&sizes, &links);
        let mut pos: Vec<Option<Pos2>> = self.nodes.iter().map(|n| n.stored.map(|p| pos2(p[0], p[1]))).collect();
        if pos.iter().all(Option::is_none) {
            for (node, p) in self.nodes.iter_mut().zip(auto) {
                node.pos = p;
            }

            return;
        }

        let mut neighbours: Vec<Vec<usize>> = vec![Vec::new(); self.nodes.len()];
        for e in &self.edges {
            neighbours[e.to].push(e.from);
        }

        for e in &self.edges {
            neighbours[e.from].push(e.to);
        }

        let mut taken: Vec<egui::Rect> = pos.iter().zip(&sizes).filter_map(|(p, s)| p.map(|p| egui::Rect::from_min_size(p, *s))).collect();
        let mut free: Vec<usize> = (0..self.nodes.len()).filter(|i| pos[*i].is_none()).collect();
        free.sort_by(|a, b| auto[*a].x.total_cmp(&auto[*b].x).then(auto[*a].y.total_cmp(&auto[*b].y)).then(a.cmp(b)));
        loop {
            let mut progress = false;
            for &u in &free {
                if pos[u].is_some() {
                    continue;
                }

                if let Some(&v) = neighbours[u].iter().find(|v| pos[**v].is_some()) {
                    let p = clear_spot(pos[v].expect("found placed") + (auto[u] - auto[v]), sizes[u], &taken);
                    taken.push(egui::Rect::from_min_size(p, sizes[u]));
                    pos[u] = Some(p);
                    progress = true;
                }
            }

            if !progress {
                break;
            }
        }

        let rest: Vec<usize> = free.into_iter().filter(|u| pos[*u].is_none()).collect();
        if !rest.is_empty() {
            let placed = taken.iter().fold(egui::Rect::NOTHING, |a, b| a.union(*b));
            let origin = rest.iter().fold(pos2(f32::MAX, f32::MAX), |a, u| a.min(auto[*u]));
            for u in rest {
                let p = pos2(placed.min.x, placed.max.y + 60.0) + (auto[u] - origin);
                let p = clear_spot(p, sizes[u], &taken);
                taken.push(egui::Rect::from_min_size(p, sizes[u]));
                pos[u] = Some(p);
            }
        }

        for (node, p) in self.nodes.iter_mut().zip(pos) {
            node.pos = p.unwrap_or_default();
        }
    }

    fn push(&mut self, node: GraphNode) {
        self.index.insert(node.key.clone(), self.nodes.len());
        self.nodes.push(node);
    }
}

/// `p` moved down past every rect it would overlap.
pub(crate) fn clear_spot(mut p: Pos2, size: Vec2, taken: &[egui::Rect]) -> Pos2 {
    for _ in 0..=taken.len() {
        let r = egui::Rect::from_min_size(p, size).expand(8.0);
        match taken.iter().filter(|t| t.intersects(r)).map(|t| t.max.y).reduce(f32::max) {
            Some(bottom) => p.y = bottom + 9.0,
            None => break,
        }
    }

    p
}

/// The nodes one output target reaches: the entities it names, else one marker for the target text.
fn target_keys(target: &str, resolved: &[NodeId], external: &BTreeSet<String>) -> Vec<NodeKey> {
    if !resolved.is_empty() {
        return resolved.iter().map(|id| NodeKey::Entity(*id)).collect();
    }

    let key = if target.is_empty() {
        NodeKey::Missing(String::new())
    } else if is_dynamic_target(target) {
        NodeKey::Dynamic(target.to_string())
    } else if external.contains(target) {
        NodeKey::Overlay(target.to_string())
    } else {
        NodeKey::Missing(target.to_string())
    };
    vec![key]
}

fn dynamic_kind(target: &str) -> &'static str {
    if target.contains('*') {
        "every name that matches"
    } else if target.starts_with('@') {
        "every node in the group"
    } else if target.starts_with('!') {
        "found when the map runs"
    } else {
        "node path"
    }
}

/// Delay, times and parameter, the parts of a connection that are not its ends.
pub fn edge_label(conn: &IoConnection) -> String {
    let mut parts: Vec<String> = Vec::new();
    if conn.delay > 0.0 {
        parts.push(format!("{}s", crate::widgets::format_number(conn.delay)));
    }

    match conn.times {
        -1 => {}
        1 => parts.push("once".into()),
        n => parts.push(format!("{n}\u{d7}")),
    }

    if !conn.parameter.is_empty() {
        let mut p: String = conn.parameter.chars().take(18).collect();
        if p.len() < conn.parameter.len() {
            p.push('\u{2026}');
        }

        parts.push(format!("\"{p}\""));
    }

    parts.join("  ")
}

/// What hovering a wire tells: the ends of the connection and the delay, with the times and parameter when set.
pub fn edge_tip(conn: &IoConnection) -> String {
    let mut tip = format!("output: {}\ntarget: {}\ninput: {}\ndelay: ", pin_label(&conn.output), conn.target, pin_label(&conn.input));
    tip.push_str(&if conn.delay > 0.0 { format!("{} s", crate::widgets::format_number(conn.delay)) } else { "none".into() });
    match conn.times {
        -1 => {}
        1 => tip.push_str("\nfires: once"),
        n => tip.push_str(&format!("\nfires: {n} times")),
    }

    if !conn.parameter.is_empty() {
        tip.push_str(&format!("\nparameter: {}", conn.parameter));
    }

    tip
}

/// Sets a node's size and packs its settings into the lines that fit.
fn node_size(node: &mut GraphNode) {
    let rows = node.inputs.len().max(node.outputs.len()).max(1);
    let text = |s: &str| s.chars().count() as f32 * CHAR;
    let mut width = text(&node.title).max(text(&node.subtitle)) + 28.0;
    for row in 0..rows {
        let left = node.inputs.get(row).map(|p| text(pin_label(&p.name))).unwrap_or(0.0);
        let right = node.outputs.get(row).map(|p| text(pin_label(&p.name))).unwrap_or(0.0);
        width = width.max(left + right + 44.0);
    }

    let width = width.clamp(MIN_WIDTH, MAX_WIDTH).round();
    node.settings = pack_settings(&node.settings, ((width - 20.0) / SETTING_CHAR) as usize);
    node.head = HEADER + SETTING_ROW * node.settings.len() as f32 + if node.settings.is_empty() { 0.0 } else { 3.0 };
    node.size = vec2(width, node.head + PIN_ROW * rows as f32 + 8.0);
}

/// Rough width of a character of the smaller settings text.
const SETTING_CHAR: f32 = 6.2;

/// The properties of an entity set away from the definition's defaults, each as a short phrase like `max 3`, `wait 2 s`
/// or `once`, in the order of the definition. Only kinds of value that read well in a few characters.
fn setting_tokens(e: &gt_doc::Entity, def: &gt_formats::EntityDef) -> Vec<String> {
    use gt_formats::PropertyType as T;
    let mut tokens = Vec::new();
    for p in &def.properties {
        let Some(value) = e.property(&p.name).map(str::trim) else { continue };
        let default = p.default.trim();
        let number = |v: &str| v.parse::<f64>().ok();
        let truthy = |v: &str| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on");
        let name = &p.name;
        let token = match p.ty {
            T::Bool if truthy(value) != truthy(default) => Some(if truthy(value) { name.clone() } else { format!("{name} off") }),
            T::Int | T::Float if number(value).is_some() && number(value) != number(default) => {
                let unit = if p.description.to_lowercase().starts_with("seconds") { " s" } else { "" };
                Some(format!("{name} {}{unit}", crate::widgets::format_number(number(value).unwrap_or_default())))
            }
            T::Choices if value != default => {
                let label = p.options.iter().find(|(_, v)| v == value).map_or(value, |(l, _)| l.as_str());
                Some(format!("{name} {label}"))
            }
            T::String | T::TargetDestination if value != default && !value.is_empty() => Some(format!("{name} {}", value.lines().next().unwrap_or_default())),
            _ => None,
        };
        tokens.extend(token);
    }

    tokens
}

/// Fills lines of at most `width` characters with the settings, `\u{b7}` between them, and ends the last line with an
/// ellipsis when some do not fit.
fn pack_settings(tokens: &[String], width: usize) -> Vec<String> {
    let width = width.max(8);
    let clip = |s: &str, at: usize| -> String {
        if s.chars().count() <= at { s.to_string() } else { s.chars().take(at.saturating_sub(1)).collect::<String>() + "\u{2026}" }
    };
    let mut lines: Vec<String> = Vec::new();
    let mut left = tokens.iter().peekable();
    while lines.len() < SETTING_LINES && left.peek().is_some() {
        let mut line = String::new();
        while let Some(t) = left.peek() {
            let joined = if line.is_empty() { (*t).clone() } else { format!("{line} \u{b7} {t}") };
            if joined.chars().count() > width && !line.is_empty() {
                break;
            }

            line = clip(&joined, width);
            left.next();
        }

        lines.push(line);
    }

    if left.peek().is_some()
        && let Some(last) = lines.last_mut()
        && !last.ends_with('\u{2026}')
    {
        let keep: String = last.chars().take(width - 2).collect();
        *last = format!("{} \u{2026}", keep.trim_end());
    }

    lines
}

/// How a pin name is shown, an empty one being a connection without an output or input.
pub fn pin_label(name: &str) -> &str {
    if name.is_empty() { "(none)" } else { name }
}

/// The input a new connection into a node of `classname` uses: the first that makes it fire, else its first input.
pub fn default_input(game: &GameConfig, classname: &str) -> String {
    let inputs: Vec<&str> = game.entity(classname).map(|d| d.inputs.iter().map(|i| i.name.as_str()).collect()).unwrap_or_default();
    inputs
        .iter()
        .find(|i| !crate::logic_sim::triggered_outputs(classname, i).is_empty())
        .or(inputs.first())
        .map(|i| i.to_string())
        .unwrap_or_else(|| BUILTIN_INPUTS[0].to_string())
}

/// The output a new connection out of a node of `classname` uses.
pub fn default_output(game: &GameConfig, classname: &str) -> String {
    game.entity(classname).and_then(|d| d.outputs.first()).map(|o| o.name.clone()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_doc::{Entity, NodeKind};

    pub(crate) fn conn(output: &str, target: &str, input: &str) -> IoConnection {
        IoConnection { output: output.into(), target: target.into(), input: input.into(), parameter: String::new(), delay: 0.0, times: -1 }
    }

    pub(crate) fn add(map: &mut Map, classname: &str, name: &str, outputs: Vec<IoConnection>) -> NodeId {
        let layer = map.default_layer();
        let mut e = Entity::new(classname);
        if !name.is_empty() {
            e.properties.insert("targetname".into(), name.into());
        }

        e.outputs = outputs;
        map.insert(layer, NodeKind::Entity(e))
    }

    fn build(map: &Map, selected: &[NodeId], external: &[&str]) -> GraphModel {
        let game = GameConfig::with_gameplay_pack();
        let selected: BTreeSet<NodeId> = selected.iter().copied().collect();
        let external: BTreeSet<String> = external.iter().map(|s| s.to_string()).collect();
        GraphModel::build(map, &Inputs { game: &game, selected: &selected, external: &external })
    }

    #[test]
    fn projects_entities_pins_and_edges() {
        let mut m = Map::new();
        let mut delayed = conn("pressed", "door", "open");
        delayed.delay = 0.5;
        delayed.times = 1;
        let button = add(&mut m, "func_button", "", vec![delayed, conn("pressed", "door", "Custom_Thing")]);
        let door = add(&mut m, "func_door", "door", vec![]);
        let relay = add(&mut m, "logic_relay", "lonely", vec![]);
        let light = add(&mut m, "light", "lamp", vec![]);

        let g = build(&m, &[], &[]);
        let keys: Vec<&NodeKey> = g.nodes.iter().map(|n| &n.key).collect();
        assert_eq!(keys, [&NodeKey::Entity(button), &NodeKey::Entity(door), &NodeKey::Entity(relay)], "an unwired light is left out, a logic entity is not");
        assert!(g.node(&NodeKey::Entity(light)).is_none());

        let b = g.node(&NodeKey::Entity(button)).unwrap();
        assert_eq!((b.title.as_str(), b.subtitle.as_str()), ("func_button", "func"), "no targetname, the classname titles it once");
        assert_eq!(b.outputs.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["pressed", "released", "locked_use"]);
        let d = g.node(&NodeKey::Entity(door)).unwrap();
        assert_eq!(d.title, "door");
        assert!(d.inputs.iter().any(|p| p.name == "open" && p.declared));
        let custom = d.input("Custom_Thing").unwrap();
        assert!(!d.inputs[custom].declared, "a name the map uses gets a pin of its own");

        assert_eq!(g.edges.len(), 2);
        let e = &g.edges[0];
        assert_eq!((e.source, e.connection, e.kind), (button, 0, EdgeKind::Resolved));
        assert_eq!((g.nodes[e.from].key.clone(), e.from_pin), (NodeKey::Entity(button), 0));
        assert_eq!(d.inputs[e.to_pin].name, "open");
        assert_eq!(e.label, "0.5s  once");
        assert_eq!(g.edges[1].to_pin, custom);
    }

    #[test]
    fn io_names_match_pins_loosely_like_the_runtime() {
        let mut m = Map::new();
        add(&mut m, "func_button", "b", vec![conn("Pressed", "d", "OPEN")]);
        let door = add(&mut m, "func_door", "d", vec![]);
        let g = build(&m, &[], &[]);
        let d = g.node(&NodeKey::Entity(door)).unwrap();
        assert_eq!(d.inputs[g.edges[0].to_pin].name, "open");
        assert!(d.inputs.iter().all(|p| p.declared), "no extra pin for a differently written name");
    }

    #[test]
    fn dynamic_overlay_and_broken_targets_get_markers() {
        let mut m = Map::new();
        let relay = add(
            &mut m,
            "logic_relay",
            "r",
            vec![
                conn("triggered", "!player", "kill"),
                conn("triggered", "@enemies", "kill"),
                conn("triggered", "door_*", "open"),
                conn("triggered", "ghost", "open"),
                conn("triggered", "ghost", "close"),
                conn("triggered", "overlay_npc", "wave"),
                conn("triggered", "", "open"),
            ],
        );
        let door = add(&mut m, "func_door", "door_1", vec![]);

        let g = build(&m, &[], &["overlay_npc"]);
        assert!(g.node(&NodeKey::Entity(door)).is_none(), "a wildcard resolves at runtime, it does not pull in the door");
        let kinds: Vec<EdgeKind> = g.edges.iter().map(|e| e.kind).collect();
        use EdgeKind::*;
        assert_eq!(kinds, [Dynamic, Dynamic, Dynamic, Broken, Broken, Overlay, Broken]);
        let ghost = g.node(&NodeKey::Missing("ghost".into())).unwrap();
        assert_eq!(ghost.inputs.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["open", "close"], "one marker per name, with the inputs used on it");
        assert_eq!(g.node(&NodeKey::Missing(String::new())).unwrap().title, "(no target)");
        assert!(g.node(&NodeKey::Dynamic("!player".into())).is_some());
        assert!(g.node(&NodeKey::Overlay("overlay_npc".into())).is_some());
        assert!(g.edges.iter().all(|e| e.source == relay));
    }

    #[test]
    fn selected_entities_show_without_wiring() {
        let mut m = Map::new();
        let light = add(&mut m, "light", "", vec![]);
        let g = build(&m, &[light], &[]);
        let n = g.node(&NodeKey::Entity(light)).unwrap();
        assert!(n.unwired);
        assert_eq!(n.inputs.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["turn_on", "turn_off", "toggle"]);
    }

    #[test]
    fn a_selected_brush_shows_its_entity() {
        let mut m = Map::new();
        let door = add(&mut m, "func_door", "", vec![]);
        let brush = gt_geom::Brush::from_aabb(&gt_core::Aabb::new(gt_core::DVec3::ZERO, gt_core::DVec3::splat(16.0)), "a").unwrap();
        let child = m.insert(door, NodeKind::Brush(brush));
        let g = build(&m, &[child], &[]);
        assert!(g.node(&NodeKey::Entity(door)).is_some());
    }

    #[test]
    fn unplaced_nodes_join_their_placed_neighbours() {
        let mut m = Map::new();
        let button = add(&mut m, "func_button", "b", vec![conn("pressed", "d", "open"), conn("pressed", "!player", "kill")]);
        let door = add(&mut m, "func_door", "d", vec![]);
        let lone = add(&mut m, "logic_auto", "", vec![]);
        let mut auto = build(&m, &[], &[]);
        auto.place();
        let at = |g: &GraphModel, key: NodeKey| g.node(&key).unwrap().pos;
        assert!(at(&auto, NodeKey::Entity(button)).x < at(&auto, NodeKey::Entity(door)).x);

        m.get_mut(button).unwrap().set_graph(Some([1000.0, 500.0]));
        let mut g = build(&m, &[], &[]);
        g.place();
        assert_eq!(at(&g, NodeKey::Entity(button)), pos2(1000.0, 500.0));
        let offset = at(&auto, NodeKey::Entity(door)) - at(&auto, NodeKey::Entity(button));
        assert_eq!(at(&g, NodeKey::Entity(door)), pos2(1000.0, 500.0) + offset, "the door keeps its place next to the button");
        let player = g.node(&NodeKey::Dynamic("!player".into())).unwrap();
        assert!(player.pos.x > 1000.0 && !player.rect().intersects(g.node(&NodeKey::Entity(door)).unwrap().rect()));
        let bottom = g.nodes.iter().filter(|n| n.key != NodeKey::Entity(lone)).map(|n| n.rect().max.y).fold(0.0, f32::max);
        assert!(at(&g, NodeKey::Entity(lone)).y > bottom, "an unwired node goes below the placed ones");

        let mut again = build(&m, &[], &[]);
        again.place();
        assert_eq!(again.nodes.iter().map(|n| n.pos).collect::<Vec<_>>(), g.nodes.iter().map(|n| n.pos).collect::<Vec<_>>());
    }

    #[test]
    fn nodes_have_a_category_and_typed_pins() {
        let mut m = Map::new();
        let mut with_value = conn("pressed", "c", "custom_amount");
        with_value.parameter = "2".into();
        let button = add(&mut m, "func_button", "b", vec![with_value, conn("pressed", "c", "custom")]);
        let counter = add(&mut m, "logic_counter", "c", vec![]);
        let g = build(&m, &[], &[]);
        let b = g.node(&NodeKey::Entity(button)).unwrap();
        assert_eq!((b.category.as_str(), b.subtitle.as_str()), ("func", "func \u{b7} func_button"), "the category is named, not only colored");
        let pressed = &b.outputs[b.output("pressed").unwrap()];
        assert_eq!((pressed.ty, pressed.values.as_str()), (PinType::Node, "activator: node"), "pressed passes the activator along");
        assert_eq!(b.outputs[b.output("released").unwrap()].ty, PinType::Pulse);
        let c = g.node(&NodeKey::Entity(counter)).unwrap();
        assert_eq!(c.category, "logic");
        assert_eq!(c.inputs[c.input("add").unwrap()].ty, PinType::Int, "typed by the definition");
        assert_eq!(c.inputs[c.input("custom_amount").unwrap()].ty, PinType::Variant, "the map hands it a parameter");
        assert_eq!(c.inputs[c.input("custom").unwrap()].ty, PinType::Pulse);

        let mut game = GameConfig::with_gameplay_pack();
        game.entities.iter_mut().find(|d| d.classname == "logic_counter").unwrap().group = "Puzzles".into();
        assert_eq!(category(&game, "logic_counter"), "puzzles", "a definition's group wins over the prefix");
        assert_eq!(PinType::parse("Vector3"), PinType::Vector3);
        assert_eq!(PinType::parse("matrix"), PinType::Variant, "an unknown type still carries a value");
    }

    #[test]
    fn nodes_show_the_settings_that_differ_from_the_defaults() {
        let mut m = Map::new();
        let set = |m: &mut Map, id: NodeId, props: &[(&str, &str)]| {
            for (k, v) in props {
                m.entity_mut(id).unwrap().properties.insert((*k).into(), (*v).into());
            }
        };
        let plain = add(&mut m, "logic_counter", "plain", vec![]);
        let counter = add(&mut m, "logic_counter", "count", vec![]);
        set(&mut m, counter, &[("max", "5"), ("min", "0"), ("start_value", "2")]);
        let timer = add(&mut m, "logic_timer", "tick", vec![]);
        set(&mut m, timer, &[("interval", "2.5"), ("once", "1"), ("start_on", "0"), ("random_max", "0")]);
        let door = add(&mut m, "func_door", "d", vec![]);
        set(&mut m, door, &[("wait", "2"), ("speed", "5"), ("targetname", "d")]);
        let unknown = add(&mut m, "prop_unknown", "u", vec![]);
        set(&mut m, unknown, &[("max", "9")]);

        let g = build(&m, &[plain, counter, timer, door, unknown], &[]);
        let lines = |id: NodeId| g.node(&NodeKey::Entity(id)).unwrap().settings.clone();
        assert!(lines(plain).is_empty(), "nothing set, nothing shown");
        assert_eq!(lines(counter), ["max 5 \u{b7} start_value 2"], "a value equal to its default is left out");
        assert_eq!(lines(timer), ["interval 2.5 s", "start_on off \u{b7} once"], "a bool is its name, or off when the default is on");
        assert_eq!(lines(door), ["speed 5 \u{b7} wait 2 s"], "only a time in seconds gets the unit, not a speed per second");
        assert!(lines(unknown).is_empty(), "no definition, no defaults to compare with");

        let plain_node = g.node(&NodeKey::Entity(plain)).unwrap();
        let shown = g.node(&NodeKey::Entity(counter)).unwrap();
        assert_eq!(plain_node.head, HEADER);
        assert_eq!(shown.head, HEADER + SETTING_ROW + 3.0);
        assert_eq!(shown.size.y, shown.head + PIN_ROW * 4.0 + 8.0, "the node grows by the lines");
        assert_eq!(shown.input_pos(0).y - shown.pos.y, shown.head + PIN_ROW * 0.5, "the pins start below the settings");
    }

    #[test]
    fn settings_are_packed_into_two_short_lines() {
        let tokens: Vec<String> = ["max 3", "min -1", "start_value 2", "steps 5", "interval 1.5 s", "loop"].map(String::from).to_vec();
        let lines = pack_settings(&tokens, 24);
        assert_eq!(lines.len(), SETTING_LINES);
        assert!(lines.iter().all(|l| l.chars().count() <= 24), "{lines:?}");
        assert_eq!(lines[0], "max 3 \u{b7} min -1");
        assert!(lines[1].ends_with('\u{2026}'), "what does not fit is marked: {lines:?}");
        assert_eq!(pack_settings(&tokens[..2], 24), ["max 3 \u{b7} min -1"]);

        let long = pack_settings(&["message a rather long piece of text that goes on".to_string()], 20);
        assert_eq!(long.len(), 1);
        assert!(long[0].chars().count() <= 20 && long[0].ends_with('\u{2026}'), "{long:?}");
        assert!(pack_settings(&[], 20).is_empty());
    }

    #[test]
    fn a_wire_tip_names_both_ends_and_the_delay() {
        let mut c = conn("pressed", "door_1", "open");
        assert_eq!(edge_tip(&c), "output: pressed\ntarget: door_1\ninput: open\ndelay: none");
        c.delay = 2.5;
        c.times = 1;
        c.parameter = "5".into();
        assert_eq!(edge_tip(&c), "output: pressed\ntarget: door_1\ninput: open\ndelay: 2.5 s\nfires: once\nparameter: 5");
        c.times = 3;
        assert!(edge_tip(&c).contains("fires: 3 times"));
    }

    #[test]
    fn pins_fit_when_a_value_can_go_into_them() {
        use PinType::*;
        assert!(Pulse.fits(Int) && Node.fits(Pulse), "a pulse takes and gives any");
        assert!(Int.fits(Float) && Variant.fits(String) && String.fits(Variant));
        assert!(!Node.fits(Int) && !String.fits(Vector3) && !Color.fits(Bool), "no natural fit");
        assert!(PinType::ALL.iter().all(|t| t.fits(*t)));
    }

    #[test]
    fn math_entities_always_have_a_node_like_logic_ones() {
        let mut m = Map::new();
        let calc = add(&mut m, "math_calc", "calc", vec![]);
        let relay = add(&mut m, "logic_relay", "r", vec![]);
        let light = add(&mut m, "light", "l", vec![]);
        let g = build(&m, &[], &[]);
        assert!(g.node(&NodeKey::Entity(calc)).is_some_and(|n| n.logic) && g.node(&NodeKey::Entity(relay)).is_some_and(|n| n.logic));
        assert!(g.node(&NodeKey::Entity(light)).is_none());
    }

    #[test]
    fn default_pins_for_new_connections() {
        let game = GameConfig::with_gameplay_pack();
        assert_eq!(default_input(&game, "logic_branch"), "test", "the input that makes it fire");
        assert_eq!(default_input(&game, "func_door"), "open");
        assert_eq!(default_input(&game, "logic_debug"), "write", "no input fires, the first one");
        assert_eq!(default_input(&game, "unknown_class"), "kill");
        assert_eq!(default_output(&game, "logic_timer"), "timer");
    }
}
