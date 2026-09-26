//! MCP tool implementations. These run on the UI thread with full access to the app.

use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::time::Instant;

use egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use gt_core::{Aabb, DVec2, DVec3, NodeId};
use gt_doc::{IoConnection, NodeKind, format, issues, ops};
use gt_geom::{Brush, shapes};
use serde_json::{Value, json};

use super::{Reply, ToolResult};
use crate::app::App;
use crate::camera::ViewKind;
use crate::commands::Action;
use crate::state::EditorState;
use crate::tools::ToolKind;

pub struct InputStep {
    pub events: Vec<Event>,
    pub modifiers: Modifiers,
}

pub struct InputScript {
    pub steps: VecDeque<InputStep>,
    pub reply: Sender<ToolResult>,
    pub frames: usize,
}

/// A tool call that finishes in a later frame.
pub enum Deferred {
    /// Answered by the screenshot event that carries the same token, so pipelined requests each get their own frame.
    WindowScreenshot { token: u64, reply: Sender<ToolResult> },
}

/// Screenshot user data, a private type so screenshots requested elsewhere in the app are never mistaken for ours.
struct ShotToken(u64);

static NEXT_SHOT: AtomicU64 = AtomicU64::new(1);

fn vec3(v: &Value) -> Option<DVec3> {
    let a = v.as_array()?;
    Some(DVec3::new(a.first()?.as_f64()?, a.get(1)?.as_f64()?, a.get(2)?.as_f64()?))
}

fn vec2(v: &Value) -> Option<DVec2> {
    let a = v.as_array()?;
    Some(DVec2::new(a.first()?.as_f64()?, a.get(1)?.as_f64()?))
}

fn arr(v: DVec3) -> Value {
    json!([v.x, v.y, v.z])
}

fn bounds_json(b: &Aabb) -> Value {
    if b.is_empty() { Value::Null } else { json!({ "min": arr(b.min), "max": arr(b.max) }) }
}

fn axis_index(v: &Value) -> Option<usize> {
    match v.as_str()?.to_ascii_lowercase().as_str() {
        "x" => Some(0),
        "y" => Some(1),
        "z" => Some(2),
        _ => None,
    }
}

fn view_kind(name: &str) -> Option<ViewKind> {
    match name.to_ascii_lowercase().as_str() {
        "3d" | "perspective" => Some(ViewKind::Perspective),
        "top" => Some(ViewKind::Top),
        "front" => Some(ViewKind::Front),
        "side" => Some(ViewKind::Side),
        _ => None,
    }
}

fn png(img: &image::RgbaImage) -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    let _ = img.write_to(&mut out, image::ImageFormat::Png);
    out.into_inner()
}

fn godot_capture_result(result: Result<Value, String>, file: &str, source: &str) -> ToolResult {
    use base64::Engine;
    let reply = match result {
        Ok(reply) => reply,
        Err(e) => return err(e),
    };
    let Some(png) = reply["png"].as_str().and_then(|d| base64::engine::general_purpose::STANDARD.decode(d).ok()) else { return err("Godot sent no image") };
    let mut note = format!("Godot view of {file} {}x{}, {source}", reply["width"], reply["height"]);
    let warnings: Vec<&str> = reply["warnings"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
    if !warnings.is_empty() {
        note += &format!("\nGodot logged:\n{}", warnings.join("\n"));
    }

    ToolResult::Image { png, note }
}

fn err(msg: impl Into<String>) -> ToolResult {
    ToolResult::Error(msg.into())
}

fn ok(v: Value) -> ToolResult {
    ToolResult::Json(v)
}

/// Issues as validate_map returns them, each with the bounds of its node so it can be found without the id.
fn issues_json(map: &gt_doc::Map, found: &[issues::Issue]) -> Vec<Value> {
    found
        .iter()
        .map(|i| {
            let mut v = serde_json::to_value(i).unwrap_or_default();
            if let Some(id) = i.node.filter(|id| map.contains(*id)) {
                v["bounds"] = bounds_json(&map.bounds_of([id]));
            }

            v
        })
        .collect()
}

/// A map path from an MCP call: `res://` and relative paths start at the open project.
fn project_path(game: &gt_formats::GameConfig, path: &str) -> std::path::PathBuf {
    let rel = path.strip_prefix("res://").unwrap_or(path);
    match &game.project_root {
        Some(root) if std::path::Path::new(rel).is_relative() => root.join(rel),
        _ => std::path::PathBuf::from(path),
    }
}

/// A filesystem path as MCP returns it, always with forward slashes whatever the platform.
pub(super) fn path_text(p: impl AsRef<std::path::Path>) -> String {
    p.as_ref().to_string_lossy().replace('\\', "/")
}

/// [`path_text`] of an optional path, null when there is none.
pub(super) fn path_value<P: AsRef<std::path::Path>>(p: Option<P>) -> Value {
    p.map_or(Value::Null, |p| Value::String(path_text(p)))
}

/// `{"old id": [new ids], ...}`, the form run_script reads to follow replaced brushes.
pub(super) fn replaced_json(replaced: &gt_doc::ops::Replaced) -> Value {
    Value::Object(replaced.iter().map(|(old, new)| (old.0.to_string(), json!(new.iter().map(|n| n.0).collect::<Vec<_>>()))).collect())
}

/// A non-negative integer. Numeric strings count too, since `"${x.id}"` in scripts produces text.
pub(super) fn uint(v: &Value) -> Option<u64> {
    match v {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

pub(super) fn node_id(v: &Value) -> Option<NodeId> {
    uint(v).map(NodeId)
}

pub(super) fn require_id(args: &Value, key: &str) -> Result<NodeId, String> {
    match &args[key] {
        Value::Null => Err(format!("{key} required")),
        v => node_id(v).ok_or_else(|| format!("{key} must be a node id, got {v}")),
    }
}

/// Optional id argument: absent is None, present but not an id is an error.
pub(super) fn optional_id(args: &Value, key: &str) -> Result<Option<NodeId>, String> {
    match &args[key] {
        Value::Null => Ok(None),
        _ => require_id(args, key).map(Some),
    }
}

/// Nested arrays are flattened, so a script can pass `["$a.ids", "$b.ids"]`.
pub(super) fn uint_list(args: &Value, key: &str) -> Result<Vec<u64>, String> {
    fn collect(v: &Value, key: &str, out: &mut Vec<u64>) -> Result<(), String> {
        match v {
            Value::Array(a) => a.iter().try_for_each(|x| collect(x, key, out)),
            _ => {
                out.push(uint(v).ok_or_else(|| format!("{key} must hold non-negative integers, got {v}"))?);
                Ok(())
            }
        }
    }

    match &args[key] {
        Value::Null => Ok(Vec::new()),
        v @ Value::Array(_) => {
            let mut out = Vec::new();
            collect(v, key, &mut out)?;
            Ok(out)
        }
        other => Err(format!("{key} must be an array, got {other}")),
    }
}

pub(super) fn id_list(args: &Value, key: &str) -> Result<Vec<NodeId>, String> {
    Ok(uint_list(args, key)?.into_iter().map(NodeId).collect())
}

/// `[[node, index], ...]` pairs such as faces.
pub(super) fn pair_list(args: &Value, key: &str) -> Result<Vec<(u64, u64)>, String> {
    match &args[key] {
        Value::Null => Ok(Vec::new()),
        Value::Array(a) => a
            .iter()
            .map(|p| match (p.get(0).and_then(uint), p.get(1).and_then(uint), p.as_array().map(Vec::len)) {
                (Some(x), Some(y), Some(2)) => Ok((x, y)),
                _ => Err(format!("{key} must hold [a, b] pairs of non-negative integers, got {p}")),
            })
            .collect(),
        other => Err(format!("{key} must be an array of pairs, got {other}")),
    }
}

pub(super) fn face_list(args: &Value, key: &str) -> Result<Vec<(NodeId, usize)>, String> {
    Ok(pair_list(args, key)?.into_iter().map(|(n, f)| (NodeId(n), f as usize)).collect())
}

/// What is being placed under a parent: geometry may also go into brush entities.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Child {
    Geometry,
    Other,
}

fn is_brush_entity(state: &EditorState, id: NodeId) -> bool {
    let map = &state.doc.map;
    let Some(n) = map.get(id) else { return false };
    let Some(e) = n.entity() else { return false };
    n.children.iter().any(|c| map.get(*c).is_some_and(|c| c.kind.is_geometry()))
        || state.game.entity(&e.classname).is_some_and(|d| d.kind == gt_formats::game::EntityKind::Solid)
}

/// Checks that `id` can hold a child of that kind: layers and groups, and brush entities for geometry.
pub(super) fn check_container(state: &EditorState, id: NodeId, child: Child) -> Result<(), String> {
    let Some(n) = state.doc.map.get(id) else { return Err(format!("no node {id}")) };
    match &n.kind {
        NodeKind::Layer(_) | NodeKind::Group(_) => Ok(()),
        NodeKind::Entity(_) if child == Child::Geometry && is_brush_entity(state, id) => Ok(()),
        kind => Err(format!(
            "{id} is a {}{}, a parent must be a layer or group{}",
            if matches!(kind, NodeKind::Entity(_)) { "point " } else { "" },
            kind.type_name(),
            if child == Child::Geometry { " or brush entity" } else { "" }
        )),
    }
}

/// Parent for new nodes: `parent` when given, else the innermost open group or current layer. Locked parents are refused,
/// since nothing inside them could be selected or edited afterwards.
pub(super) fn resolve_parent(state: &EditorState, parent: &Value, child: Child) -> Result<NodeId, String> {
    let id = match parent {
        Value::Null => state.insert_parent(),
        v => {
            let id = node_id(v).ok_or_else(|| format!("parent must be a node id, got {v}"))?;
            check_container(state, id, child)?;
            id
        }
    };
    if state.doc.map.is_locked(id) {
        return Err(format!(
            "{id} ({}) is locked, unlock it with hierarchy set_flags or pick another parent or current layer",
            state.doc.map.get(id).map(|n| n.name()).unwrap_or_default()
        ));
    }

    Ok(id)
}

/// Ids that exist and are neither hidden nor locked, the same rule the viewports and Select All use.
pub(super) fn editable_ids(state: &EditorState, ids: &[NodeId]) -> Result<Vec<NodeId>, String> {
    let map = &state.doc.map;
    let missing: Vec<u64> = ids.iter().filter(|i| !map.contains(**i)).map(|i| i.0).collect();
    if !missing.is_empty() {
        return Err(format!("no nodes with ids {missing:?}"));
    }

    let blocked: Vec<u64> = ids.iter().filter(|i| !map.is_editable(**i)).map(|i| i.0).collect();
    if !blocked.is_empty() {
        return Err(format!("{blocked:?} are hidden or locked, unhide or unlock them first"));
    }

    Ok(ids.to_vec())
}

/// Status messages `commands::execute` shows when an action did nothing, so MCP callers get an error instead.
fn is_failure_status(s: &str) -> bool {
    const PREFIXES: [&str; 19] = [
        "Save failed",
        "Save or discard",
        "Save the map",
        "Nothing to",
        "Select ",
        "Selected brushes do not",
        "No cordon",
        "No hotspot",
        "No project.godot",
        "Clipboard does not",
        "Open failed",
        "Import failed",
        "Export failed",
        "Cannot ",
        "Could not",
        "Place model failed",
        "Open a Godot project first",
        "Godot was not found",
        "Godot is not open",
    ];
    PREFIXES.iter().any(|p| s.starts_with(p))
}

/// Actions that open a native file dialog, which would block the UI thread, with the MCP call to use instead.
fn dialog_action(name: &str) -> Option<&'static str> {
    Some(match name {
        "save_as" => "map_file {op: save, path}",
        "open" | "open_map" => "map_file {op: open or open_tab, path}",
        "open_project" => "open_project {path}",
        "import_quake_map" | "import_map" => "map_file {op: import_map, path}",
        "import_vmf" => "map_file {op: import_vmf, path}",
        "convert_textures" => "map_file {op: convert_textures, path}",
        "import_model" => "import_model {path}",
        "export_quake_map" | "export_map" | "export_quake_map_cordon" => "map_file {op: export_map, path}",
        "export_glb" | "export_obj" => "map_file {op: export_glb or export_obj, path}",
        "create_prefab" => "copy the objects and save them with map_file into a new map",
        _ => return None,
    })
}

/// Pastes GodotTrench clipboard text under `parent` as one undo step, moved by `offset` or centered on `origin`.
pub(super) fn paste_text(state: &mut EditorState, text: &str, parent: NodeId, origin: Option<DVec3>, offset: Option<DVec3>) -> Result<Vec<NodeId>, String> {
    let opts = ops::EditOptions { uv_lock: state.uv_lock, grid: 0.0 };
    state.doc.try_edit("Paste", |m, s| {
        let ids = format::paste_nodes(m, parent, text).map_err(|e| format!("not GodotTrench clipboard text: {e}"))?;
        if ids.is_empty() {
            return Err("the text holds no objects to paste".to_string());
        }

        s.clear();
        s.nodes.extend(ids.iter().copied());
        let b = m.bounds_of(ids.iter().copied());
        let mut shift = offset.unwrap_or_default();
        if let Some(o) = origin
            && !b.is_empty()
        {
            shift += o - b.center();
        }

        if shift != DVec3::ZERO {
            ops::translate_selection(m, s, shift, opts);
        }

        Ok(ids)
    })
}

impl App {
    /// Executes queued MCP calls. Must run at the start of the frame, before UI and input handling.
    pub(crate) fn process_mcp(&mut self, ctx: &egui::Context) {
        let Some(host) = &self.mcp else { return };
        let calls: Vec<_> = host.rx.try_iter().collect();
        for call in calls {
            if let Reply::Now(result) = self.call_tool(&call.name, call.args, ctx, &call.reply) {
                let _ = call.reply.send(result);
            }
        }

        for e in ctx.input(|i| i.raw.events.clone()) {
            let Event::Screenshot { image, user_data, .. } = e else { continue };
            let Some(token) = user_data.data.as_ref().and_then(|d| d.downcast_ref::<ShotToken>()).map(|t| t.0) else { continue };
            let Some(i) = self.deferred.iter().position(|d| matches!(d, Deferred::WindowScreenshot { token: t, .. } if *t == token)) else { continue };
            let Deferred::WindowScreenshot { reply, .. } = self.deferred.remove(i);
            let rgba = image::RgbaImage::from_raw(image.width() as u32, image.height() as u32, image.as_raw().to_vec());
            let result = match &rgba {
                Some(img) => ToolResult::Image { png: png(img), note: format!("window {}x{}", img.width(), img.height()) },
                None => ToolResult::Error("screenshot conversion failed".into()),
            };
            let _ = reply.send(result);
        }
    }

    /// Called at the end of a frame: replies to input scripts that finished this frame.
    pub(crate) fn finish_input_script(&mut self, ctx: &egui::Context) {
        if let Some(script) = &self.input_script {
            if script.steps.is_empty() {
                let script = self.input_script.take().unwrap();
                let _ = script.reply.send(ToolResult::Json(json!({ "ok": true, "frames": script.frames, "state": self.state_summary() })));
            } else {
                ctx.request_repaint();
            }
        }

        if !self.deferred.is_empty() {
            ctx.request_repaint();
        }
    }

    pub(crate) fn feed_input_script(&mut self, raw: &mut egui::RawInput) {
        let Some(script) = &mut self.input_script else { return };
        if let Some(step) = script.steps.pop_front() {
            raw.events.push(Event::ModifiersChanged(step.modifiers));
            raw.events.extend(step.events);
            raw.focused = true;
            script.frames += 1;
        }
    }

    /// Runs a tool call from a transport. Window screenshots and input scripts keep `reply` and answer in a later frame.
    pub(crate) fn call_tool(&mut self, name: &str, args: Value, ctx: &egui::Context, reply: &Sender<ToolResult>) -> Reply {
        // The watcher only looks once a second, a call right after Godot exported the config must already see it.
        if self.state.reload_changed_game_config() {
            self.project_generation += 1;
        }

        match name {
            "screenshot" if args["source"].as_str() == Some("godot") => match self.godot_capture(&args, reply.clone()) {
                Ok(()) => Reply::Deferred,
                Err(e) => Reply::Now(err(e)),
            },
            "walkability" => match self.walkability(&args, reply.clone(), ctx) {
                Ok(()) => Reply::Deferred,
                Err(e) => Reply::Now(err(e)),
            },
            "screenshot" if args["target"].as_str().unwrap_or("window") == "window" => {
                let token = NEXT_SHOT.fetch_add(1, Ordering::Relaxed);
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(ShotToken(token))));
                self.deferred.push(Deferred::WindowScreenshot { token, reply: reply.clone() });
                ctx.request_repaint();
                Reply::Deferred
            }
            "simulate_input" => {
                if self.input_script.is_some() {
                    return Reply::Now(err("another input script is still running"));
                }

                match self.build_input_script(&args, reply.clone()) {
                    Ok(script) => {
                        self.input_script = Some(script);
                        ctx.request_repaint();
                        Reply::Deferred
                    }
                    Err(e) => Reply::Now(err(e)),
                }
            }
            _ => Reply::Now(self.run_tool_as_one_step(name, args, ctx)),
        }
    }

    /// Runs a tool as one undo step labelled "MCP: ...", so a person can take back an agent's call in one go. A script
    /// labels its own step.
    fn run_tool_as_one_step(&mut self, name: &str, args: Value, ctx: &egui::Context) -> ToolResult {
        if name == "run_script" {
            return self.run_tool(name, args, ctx);
        }

        let mark = self.state.doc.mark();
        let call = match args["op"].as_str().or(args["action"].as_str()) {
            Some(op) => format!("{name} {op}"),
            None => name.to_string(),
        };
        let result = self.run_tool(name, args, ctx);
        self.state.doc.squash_since(mark, |steps| match steps {
            [one] => format!("MCP: {one}"),
            _ => format!("MCP: {call}, {} edits", steps.len()),
        });
        result
    }

    /// Asks the Godot editor for a capture of the current map over the live link, answering `reply` when it arrives.
    fn godot_capture(&self, args: &Value, reply: Sender<ToolResult>) -> Result<(), String> {
        let s = &self.state;
        let Some(path) = s.doc.path.clone() else { return Err("save the map into the Godot project first, Godot builds maps from their file path".into()) };
        if s.game.project_root.is_none() {
            return Err("open the Godot project first with open_project".into());
        }

        let Some(link) = s.link.as_ref().filter(|_| s.prefs.live_link) else { return Err("the Godot live link is off, turn it on in Preferences".into()) };
        let Some(vp) = self.viewports.iter().find(|v| v.kind() == ViewKind::Perspective) else { return Err("3d view not open".into()) };
        let eye = vec3(&args["position"]).unwrap_or(vp.camera.position);
        let forward = match vec3(&args["look_at"]) {
            Some(target) => (target - eye).try_normalize().ok_or("look_at must differ from position")?,
            None => vp.camera.forward(),
        };
        let fov = args["fov"].as_f64().unwrap_or(vp.camera.fov);
        let size = [args["width"].as_u64().unwrap_or(1280).clamp(16, 4096) as u32, args["height"].as_u64().unwrap_or(720).clamp(16, 4096) as u32];
        let live = s.live_active();
        let build = args["build"].as_bool().unwrap_or(!live).then(|| s.doc.map.clone());
        let source = match (build.is_some(), live) {
            (true, _) => "built from the map as shown here",
            (false, true) => "with the live edits",
            (false, false) => "as Godot has it",
        };
        let file = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        link.request(crate::live_link::Request::Capture {
            path: crate::live_link::godot_path(&path),
            camera: json!({ "position": arr(eye), "forward": arr(forward), "fov": fov }),
            size,
            build,
            live,
            done: Box::new(move |result| {
                let _ = reply.send(godot_capture_result(result, &file, source));
            }),
        });
        Ok(())
    }

    /// Bakes the walkable area in Godot for the agent in `args` and answers `reply` with its summary when it arrives.
    fn walkability(&mut self, args: &Value, reply: Sender<ToolResult>, ctx: &egui::Context) -> Result<(), String> {
        let agent = &mut self.state.prefs.walk_agent;
        for (key, value, range) in [
            ("radius", &mut agent.radius, 0.05..=10.0),
            ("height", &mut agent.height, 0.2..=20.0),
            ("max_climb", &mut agent.max_climb, 0.0..=10.0),
            ("max_slope", &mut agent.max_slope, 0.0..=89.0),
        ] {
            if let Some(v) = args[key].as_f64() {
                *value = v.clamp(*range.start(), *range.end());
            }
        }

        let show = args["show"].as_bool().unwrap_or(true);
        crate::walkable::request(
            &mut self.state,
            Some(ctx.clone()),
            Some(Box::new(move |result| {
                let _ = reply.send(match result {
                    Ok(w) => ok(w.summary()),
                    Err(e) => err(e.clone()),
                });
            })),
        )?;
        if show != self.state.walkable.on {
            self.state.walkable.on = show;
            self.state.walkable.generation += 1;
        }

        Ok(())
    }

    /// Runs a tool that answers right away, as every tool inside run_script must.
    pub(crate) fn run_tool(&mut self, name: &str, args: Value, ctx: &egui::Context) -> ToolResult {
        if std::mem::take(&mut self.state.materials_rescanned) {
            self.project_generation += 1;
        }

        match name {
            "get_state" => ok(self.state_summary()),
            "list_nodes" => match self.list_nodes(&args) {
                Ok(v) => ok(v),
                Err(e) => err(e),
            },
            "summarize_map" => ok(super::review_tools::summarize(&self.state, args["limit"].as_u64().unwrap_or(100) as usize)),
            "changes_since" => self.tool_changes_since(&args),
            "get_node" => {
                let id = match require_id(&args, "id") {
                    Ok(id) => id,
                    Err(e) => return err(e),
                };
                if !self.state.doc.map.contains(id) {
                    return err(format!("no node {id}"));
                }

                if args["compact"].as_bool().unwrap_or(false) {
                    return ok(super::review_tools::compact_node(&self.state.doc.map, id));
                }

                let text = format::nodes_to_string(&self.state.doc.map, &[id]);
                match serde_json::from_str::<Value>(&text) {
                    Ok(v) => ok(v["nodes"][0].clone()),
                    Err(e) => err(format!("could not encode node {id}: {e}")),
                }
            }
            "run_action" => self.tool_run_action(&args, ctx),
            "create_brush" => self.tool_create_brush(&args),
            "create_entity" => self.tool_create_entity(&args),
            "update_entity" => self.tool_update_entity(&args),
            "create_mesh" => self.tool_create_mesh(&args),
            "mesh_edit" => self.tool_mesh_edit(&args),
            "texture" => self.tool_texture(&args),
            "create_terrain" => self.tool_create_terrain(&args),
            "scatter" => self.tool_scatter(&args),
            "blend" => self.tool_blend(&args),
            "gameplay" => self.tool_gameplay(&args),
            "code_reference" => self.tool_code_reference(&args),
            "hierarchy" => self.tool_hierarchy(&args),
            "set_map_properties" => self.tool_set_map_properties(&args),
            "terrain_edit" => self.tool_terrain_edit(&args),
            "duplicate" => self.tool_duplicate(&args),
            "run_script" => self.tool_run_script(&args, ctx),
            "import_model" => self.tool_import_model(&args),
            "select" => self.tool_select(&args),
            "transform" => self.tool_transform(&args),
            "set_face" => self.tool_set_face(&args),
            "map_file" => self.tool_map_file(&args),
            "open_project" => {
                let path = std::path::PathBuf::from(args["path"].as_str().unwrap_or_default());
                match gt_formats::game::find_project_root(&path) {
                    Some(root) => {
                        self.state.load_project(&root);
                        self.project_generation += 1;
                        ok(json!({
                            "project_root": path_text(&root), "game": self.state.game.name, "entities": self.state.game.entities.len(),
                            "materials": self.state.materials.entries.len(), "status": self.state.status,
                            "addon_version": self.state.game.addon_version,
                            "addon_version_mismatch": self.state.game.addon_version.as_deref().is_some_and(|v| v != crate::VERSION),
                        }))
                    }
                    None => err("no project.godot found"),
                }
            }
            "set_editor" => self.tool_set_editor(&args, ctx),
            "set_camera" => {
                let Some(kind) = args["view"].as_str().and_then(view_kind) else { return err("unknown view") };
                let Some(vp) = self.viewports.iter_mut().find(|v| v.kind() == kind) else { return err("view not open") };
                if let Some(p) = vec3(&args["position"]) {
                    vp.camera.position = p;
                }

                if let Some(t) = vec3(&args["look_at"]) {
                    vp.camera.look_at(t);
                }

                if let Some(c) = vec3(&args["center"]) {
                    vp.camera.center = c;
                }

                if let Some(z) = args["zoom"].as_f64() {
                    vp.camera.zoom = z.clamp(0.005, 64.0);
                }

                if let (Some(min), Some(max)) = (vec3(&args["focus"]["min"]), vec3(&args["focus"]["max"])) {
                    vp.focus(&Aabb::new(min, max));
                }

                ctx.request_repaint();
                ok(json!({ "ok": true }))
            }
            "screenshot" if args["source"].as_str() == Some("godot") => err("Godot captures cannot run inside scripts"),
            "walkability" => err("walkability cannot run inside scripts, call it after the script"),
            "screenshot" => {
                let target = args["target"].as_str().unwrap_or("window");
                let Some(kind) = view_kind(target) else { return err("unknown target, or window screenshots, which cannot run inside scripts") };
                let Some(vp) = self.viewports.iter().find(|v| v.kind() == kind) else { return err("view not open") };
                let sized = args["width"].as_u64().zip(args["height"].as_u64());
                // Offscreen captures are beauty shots unless asked otherwise, a docked capture keeps what the pane shows.
                let overlays = args["overlays"].as_bool().unwrap_or(sized.is_none());
                let size = match sized {
                    Some((w, h)) => Some([w.clamp(16, 4096) as u32, h.clamp(16, 4096) as u32]),
                    None if !overlays => vp.target().map(|t| t.size),
                    None => None,
                };
                if let Some(size) = size {
                    let clean = !overlays && !self.state.doc.selection.is_empty();
                    if clean {
                        self.scene.update_with(&mut self.renderer, &mut self.state, self.project_generation, false);
                    }

                    let shot = vp.render_offscreen(&mut self.renderer, &self.scene, &self.state, size, overlays);
                    if clean {
                        self.scene.update(&mut self.renderer, &mut self.state, self.project_generation);
                    }

                    return match shot {
                        Some(img) => ToolResult::Image { png: png(&img), note: format!("{} view {}x{}", kind.label(), img.width(), img.height()) },
                        None => err("readback failed"),
                    };
                }

                let Some(t) = vp.target() else { return err("view has not rendered yet, is its tab visible?") };
                match self.renderer.read_target(t) {
                    Some(img) => ToolResult::Image { png: png(&img), note: format!("{} view {}x{}", kind.label(), img.width(), img.height()) },
                    None => err("readback failed"),
                }
            }
            "simulate_input" => err("simulate_input cannot run inside scripts"),
            "validate_map" => {
                if args["project"].as_bool() == Some(true) {
                    let Some(root) = self.state.game.project_root.clone() else { return err("no project is open, use open_project") };
                    let mut count = 0;
                    let mut maps = Vec::new();
                    for path in crate::validate::project_maps(&root) {
                        maps.push(match crate::validate::file_issues(&mut self.state, &path) {
                            Ok((map, found)) => {
                                count += found.len();
                                json!({ "path": path_text(&path), "count": found.len(), "issues": issues_json(&map, &found) })
                            }
                            Err(e) => json!({ "path": path_text(&path), "error": e }),
                        });
                    }

                    return ok(json!({ "count": count, "maps": maps }));
                }

                let path = args["path"].as_str().map(|p| project_path(&self.state.game, p));
                let (map, found) = match path {
                    Some(p) => match crate::validate::file_issues(&mut self.state, &p) {
                        Ok(checked) => checked,
                        Err(e) => return err(e),
                    },
                    None => {
                        let found = crate::validate::open_map_issues(&mut self.state);
                        (self.state.doc.map.clone(), found)
                    }
                };
                ok(json!({ "count": found.len(), "issues": issues_json(&map, &found) }))
            }
            "get_game_config" => match args["classname"].as_str() {
                Some(c) => match self.state.game.entity(c) {
                    Some(def) => ok(serde_json::to_value(def).unwrap_or_default()),
                    None => err(format!("no definition for {c}")),
                },
                None => ok(json!({
                    "name": self.state.game.name,
                    "project_root": path_value(self.state.game.project_root.as_ref()),
                    "units_per_meter": self.state.game.units_per_meter,
                    "textures": serde_json::to_value(&self.state.game.textures).unwrap_or_default(),
                    "point_entities": self.state.game.point_entities().map(|e| e.classname.clone()).collect::<Vec<_>>(),
                    "solid_entities": self.state.game.solid_entities().map(|e| e.classname.clone()).collect::<Vec<_>>(),
                    "materials": self.state.materials.entries.iter().take(500).map(|m| m.name.clone()).collect::<Vec<_>>(),
                })),
            },
            _ => err(format!("tool {name} not implemented")),
        }
    }

    pub(crate) fn state_summary(&self) -> Value {
        let s = &self.state;
        let map = &s.doc.map;
        let sel = &s.doc.selection;
        let cameras: Vec<Value> = self
            .viewports
            .iter()
            .map(|v| {
                let c = &v.camera;
                let rect = json!([v.rect.min.x, v.rect.min.y, v.rect.width(), v.rect.height()]);
                if c.kind.is_2d() {
                    json!({ "view": c.kind.label(), "center": arr(c.center), "zoom": c.zoom, "rect": rect })
                } else {
                    json!({ "view": "3d", "position": arr(c.position), "forward": arr(c.forward()), "rect": rect })
                }
            })
            .collect();
        json!({
            "map": {
                "path": path_value(s.doc.path.as_ref()), "modified": s.doc.is_modified(), "revision": s.doc.revision,
                "brushes": map.brush_count(), "entities": map.entity_count(), "layers": map.layers.len(),
                "meshes": map.meshes().count(), "terrains": map.terrains().count(),
                "cordon": map.editor.cordon.map(|c| bounds_json(&c)), "cordon_enabled": map.editor.cordon_enabled,
                "cameras": map.editor.cameras.keys().collect::<Vec<_>>(),
            },
            "tabs": { "titles": s.tab_titles().0, "active": s.tab_titles().1 },
            "selection": {
                "nodes": sel.nodes.iter().map(|i| i.0).collect::<Vec<_>>(),
                "faces": sel.faces.iter().map(|(i, f)| json!([i.0, f])).collect::<Vec<_>>(),
                "bounds": bounds_json(&map.bounds_of(sel.nodes.iter().copied())),
            },
            "editor": {
                "tool": s.tool.label(), "grid": s.grid, "snap": s.snap, "uv_lock": s.uv_lock, "material": s.current_material,
                "shade": s.prefs.shade.label(), "current_layer": s.current_layer.0, "open_groups": s.open_groups.iter().map(|g| g.0).collect::<Vec<_>>(),
                "keymap": s.prefs.keymap_preset,
                "mesh_component": self.tools.mesh.component.label(),
                "mesh_selection": self.tools.mesh.selection.iter().map(|(id, sel)| json!({
                    "id": id.0, "verts": sel.verts, "edges": sel.edges.iter().map(|(a, b)| [a, b]).collect::<Vec<_>>(), "faces": sel.faces,
                })).collect::<Vec<_>>(),
                "measure": self.tools.measure.start.zip(self.tools.measure.end).map(|(a, b)| json!({ "start": arr(a), "end": arr(b), "distance": (b - a).length() })),
                "clip_points": self.tools.clip.points.iter().map(|p| arr(*p)).collect::<Vec<_>>(),
                "selected_vertices": self.tools.vertex.selected.iter().map(|p| arr(*p)).collect::<Vec<_>>(),
            },
            "cameras": cameras,
            "ui": {
                "scale": (s.prefs.ui_scale as f64 * 100.0).round() / 100.0, "follow_display_scaling": s.prefs.follow_display_scaling,
                "pixels_per_point": self.viewports.first().map(|v| (v.pixels_per_point as f64 * 1000.0).round() / 1000.0),
                "windows": self.open_windows.iter().map(|(title, r)| json!({ "title": title, "rect": [r.min.x, r.min.y, r.max.x, r.max.y] })).collect::<Vec<_>>(),
                "tool_buttons": self.tool_buttons.iter().map(|(t, r)| json!({ "tool": t.label(), "rect": [r.min.x, r.min.y, r.max.x, r.max.y] })).collect::<Vec<_>>(),
            },
            "game": {
                "name": s.game.name, "project_root": path_value(s.game.project_root.as_ref()), "entity_definitions": s.game.entities.len(),
                "materials": s.materials.entries.len(), "editor_version": crate::VERSION, "addon_version": s.game.addon_version,
                "addon_version_mismatch": s.game.addon_version.as_deref().is_some_and(|v| v != crate::VERSION),
            },
            "godot": {
                "executable": s.godot.exe, "connected": s.link_state.connected, "addon_outdated": s.link_state.outdated,
                "project_open": s.godot_has_project(), "version": s.link_state.godot, "busy": s.link_state.busy,
                "live_mode": s.prefs.live_mode, "live_session": s.live_active(),
                "maps": s.link_state.maps.keys().collect::<Vec<_>>(),
            },
            "undo": s.doc.history.undo_labels().take(10).collect::<Vec<_>>(),
            "redo": s.doc.history.redo_labels().take(10).collect::<Vec<_>>(),
            "status": s.status,
        })
    }

    fn list_nodes(&self, args: &Value) -> Result<Value, String> {
        let map = &self.state.doc.map;
        let filter = super::review_tools::NodeFilter::parse(map, args)?;
        let ty = args["type"].as_str();
        let classname = args["classname"].as_str();
        let selected_only = args["selected_only"].as_bool().unwrap_or(false);
        let limit = args["limit"].as_u64().unwrap_or(200) as usize;
        let mut nodes = Vec::new();
        let mut total = 0;
        for id in map.walk() {
            let Some(n) = map.get(id) else { continue };
            if ty.is_some_and(|t| t != n.kind.type_name()) {
                continue;
            }

            if classname.is_some_and(|c| n.entity().is_none_or(|e| e.classname != c)) {
                continue;
            }

            let selected = self.state.doc.selection.nodes.contains(&id);
            if selected_only && !selected {
                continue;
            }

            let bounds = self.state.instance_bounds.get(&id).copied().unwrap_or_else(|| map.bounds(id));
            if !filter.matches(map, id, &bounds) {
                continue;
            }

            total += 1;
            if nodes.len() >= limit {
                continue;
            }

            let mut v = json!({
                "id": id.0, "type": n.kind.type_name(), "name": n.name(), "parent": n.parent.map(|p| p.0),
                "bounds": bounds_json(&bounds), "hidden": n.hidden, "locked": n.locked, "selected": selected, "children": n.children.len(),
            });
            if let Some(e) = n.entity() {
                v["classname"] = json!(e.classname);
                if let Some(p) = filter.matched_properties(e) {
                    v["properties"] = p;
                }
            }

            nodes.push(v);
        }

        Ok(json!({ "total": total, "nodes": nodes }))
    }

    /// The status line when this action set it, so a caller never sees the message of an earlier command.
    fn fresh_status(&self, started: Instant) -> Option<String> {
        let age = self.state.status_age();
        (age <= started.elapsed().as_secs_f32()).then(|| self.state.status.clone())
    }

    fn tool_run_action(&mut self, args: &Value, ctx: &egui::Context) -> ToolResult {
        let name = args["action"].as_str().unwrap_or_default();
        let a = &args["args"];
        if let Some(instead) = dialog_action(name) {
            return err(format!("{name} opens a native file dialog, which MCP cannot answer, use {instead} instead"));
        }

        let started = Instant::now();
        self.state.replaced.clear();
        let action = match name {
            "new_map" => Action::NewMap,
            "save" => {
                let Some(path) = self.state.doc.path.clone() else {
                    return err("the map has no file yet, use map_file {op: save, path} (saving an untitled map opens a file dialog)");
                };
                return match self.state.save_map(&path) {
                    Ok(()) => ok(json!({ "ok": true, "path": path_text(&path), "status": self.fresh_status(started) })),
                    Err(e) => err(format!("Save failed: {e}")),
                };
            }
            "undo" | "redo" => {
                let action = if name == "undo" { Action::Undo } else { Action::Redo };
                for _ in 1..a["steps"].as_u64().unwrap_or(1) {
                    crate::commands::execute(&mut self.state, action.clone(), ctx);
                }

                action
            }
            "delete" => Action::Delete,
            "duplicate" => Action::Duplicate,
            "select_all" => Action::SelectAll,
            // Escape's Select None first leaves a tool, over MCP it always means an empty selection.
            "select_none" => {
                self.state.doc.select(|_, s| s.clear());
                return ok(json!({ "ok": true, "selected": 0 }));
            }
            "select_inverse" => Action::SelectInverse,
            "select_touching" => Action::SelectTouching,
            "select_inside" => Action::SelectInside,
            "select_siblings" => Action::SelectSiblings,
            "select_same_material" => Action::SelectSameMaterial,
            "group" => Action::Group,
            "ungroup" => Action::Ungroup,
            "hide_selected" => Action::HideSelected,
            "isolate_selected" => Action::IsolateSelected,
            "unhide_all" => Action::UnhideAll,
            "lock_selected" => Action::LockSelected,
            "unlock_all" => Action::UnlockAll,
            "grid_up" => Action::GridUp,
            "grid_down" => Action::GridDown,
            "toggle_snap" => Action::ToggleSnap,
            "toggle_uv_lock" => Action::ToggleUvLock,
            "toggle_textured" => Action::ToggleTextured,
            "csg_subtract" => {
                if let Some(name) = a["carve_material"].as_str() {
                    let Some(carve) = gt_geom::csg::CarveMaterial::from_name(name) else {
                        return err(format!("carve_material is target or cutter, not {name}"));
                    };
                    let saved = std::mem::replace(&mut self.state.carve_material, carve);
                    let result = self.tool_run_action(&json!({ "action": "csg_subtract" }), ctx);
                    self.state.carve_material = saved;
                    return result;
                }

                Action::CsgSubtract
            }
            "csg_merge" => Action::CsgMerge,
            "csg_intersect" => Action::CsgIntersect,
            "csg_hollow" => Action::CsgHollow,
            "rotate" => Action::Rotate { axis: axis_index(&a["axis"]).unwrap_or(1), degrees: a["degrees"].as_f64().unwrap_or(90.0) },
            "flip" => Action::Flip { axis: axis_index(&a["axis"]).unwrap_or(0) },
            "focus_selection" => Action::FocusSelection,
            "create_brush_entity" => Action::CreateBrushEntity(a["classname"].as_str().unwrap_or("func_detail").to_string()),
            "place_entities" => Action::PlaceEntities {
                classnames: a["classnames"].as_array().into_iter().flatten().filter_map(|c| c.as_str()).map(str::to_string).collect(),
                at: vec3(&a["at"]),
                normal: vec3(&a["normal"]),
                row: vec3(&a["row"]).unwrap_or(DVec3::X),
            },
            "move_to_world" => Action::MoveToWorld,
            "add_layer" => Action::AddLayer,
            "snap_vertices" => Action::SnapVertices,
            "apply_material" => match a["material"].as_str().filter(|m| !m.is_empty()) {
                Some(m) => Action::ApplyMaterial(m.to_string()),
                None => return err("apply_material needs args.material"),
            },
            "create_decal" => {
                let (Some(material), Some(at)) = (a["material"].as_str().filter(|m| !m.is_empty()), vec3(&a["at"])) else {
                    return err("create_decal needs args.material and args.at");
                };
                let size = a["size"].as_array().and_then(|s| Some([s.first()?.as_f64()?, s.get(1)?.as_f64()?]));
                Action::CreateDecal { material: material.to_string(), at, normal: vec3(&a["normal"]).unwrap_or(DVec3::Y), size }
            }
            "copy" | "cut" => {
                let roots = ops::selection_roots(&self.state.doc.map, &self.state.doc.selection);
                if roots.is_empty() {
                    return err(format!("nothing selected to {name}"));
                }

                let text = format::nodes_to_string(&self.state.doc.map, &roots);
                ctx.copy_text(text.clone());
                if name == "cut" {
                    self.state.doc.edit("Cut", ops::delete_selection);
                }

                self.state.set_status(format!("Copied {} objects", roots.len()));
                return ok(json!({ "ok": true, "objects": roots.len(), "text": text }));
            }
            "paste" => {
                let Some(text) = a["text"].as_str().filter(|t| !t.trim().is_empty()) else {
                    return err("paste needs args.text, the text that copy or cut returned");
                };
                let parent = match resolve_parent(&self.state, &a["parent"], Child::Other) {
                    Ok(p) => p,
                    Err(e) => return err(e),
                };
                return match paste_text(&mut self.state, text, parent, vec3(&a["origin"]), vec3(&a["offset"])) {
                    Ok(ids) => {
                        let b = self.state.doc.map.bounds_of(ids.iter().copied());
                        self.state.set_status(format!("Pasted {} objects", ids.len()));
                        ok(json!({ "ok": true, "ids": ids.iter().map(|i| i.0).collect::<Vec<_>>(), "bounds": bounds_json(&b) }))
                    }
                    Err(e) => err(e),
                };
            }
            "nudge" => Action::Nudge(vec3(&a["offset"]).unwrap_or_default()),
            "set_tool" => match a["tool"].as_str().and_then(ToolKind::from_name) {
                Some(t) => Action::SetTool(t),
                None => return err("unknown tool"),
            },
            "explode_instances" => Action::ExplodeInstances,
            "create_displacement" => Action::CreateDisplacement(a["power"].as_u64().unwrap_or(3) as u8),
            "remove_displacement" => Action::RemoveDisplacement,
            "sew_displacements" => Action::SewDisplacements,
            "sculpt" => {
                let Some(center) = vec3(&a["center"]) else { return err("center required") };
                let mode: gt_doc::terrain::SculptMode = match serde_json::from_value(a["mode"].clone()) {
                    Ok(m) => m,
                    Err(_) => return err("mode must be raise, lower, smooth, flatten, noise, terrace, paint_alpha, erase_alpha, paint_layer, hole or unhole"),
                };
                let brush = gt_doc::terrain::SculptBrush {
                    mode,
                    radius: a["radius"].as_f64().unwrap_or(self.state.sculpt.radius),
                    strength: a["strength"].as_f64().unwrap_or(self.state.sculpt.strength),
                    flatten_height: a["height"].as_f64().unwrap_or(center.y),
                    terrace_step: a["step"].as_f64().unwrap_or(self.state.sculpt.terrace_step),
                    layer: a["layer"].as_u64().unwrap_or(self.state.sculpt.layer as u64) as u8,
                    seed: a["seed"].as_u64().unwrap_or(7) as u32,
                };
                let scope = crate::blend_tool::StrokeScope::of(&self.state);
                let faces = scope.displacements(&self.state);
                let terrains = scope.terrains;
                let changed = self
                    .state
                    .doc
                    .edit("Sculpt", |m, _| gt_doc::terrain::sculpt(m, &faces, center, &brush) | gt_doc::terrain::sculpt_terrains(m, &terrains, center, &brush));
                return ok(json!({ "changed": changed, "faces": faces.len(), "terrains": terrains.len() }));
            }
            "sprinkle" => {
                let Some(center) = vec3(&a["center"]) else { return err("center required") };
                let spacing = a["min_spacing"].as_f64().unwrap_or(64.0);
                let items: Vec<gt_doc::ScatterItem> = a["items"]
                    .as_array()
                    .map(|i| i.iter().filter_map(|v| v.as_str()).map(|s| gt_doc::ScatterItem { spacing, ..gt_doc::ScatterItem::new(s) }).collect())
                    .unwrap_or_else(|| crate::scatter_tool::brush_items(&self.state));
                if items.is_empty() {
                    return err("sprinkle needs items, or an active scatter set with models (or set_editor scatter_items)");
                }

                let radius = a["radius"].as_f64().unwrap_or(self.state.prefs.scatter.radius);
                let rules =
                    gt_doc::scatter::ScatterRules { density: a["density"].as_f64().unwrap_or(1.0), slope: [0.0, 90.0], falloff: 0.0, ..Default::default() };
                let mut rng = gt_doc::scatter::Rng::new(a["seed"].as_u64().unwrap_or(1));
                let normal = vec3(&a["normal"]).unwrap_or(DVec3::Y);
                let ids = crate::scatter_tool::paint_entities(&mut self.state, &items, center, normal, radius, &rules, &mut rng);
                return ok(json!({ "ids": ids.iter().map(|i| i.0).collect::<Vec<_>>() }));
            }
            "mesh_op" => {
                let op_name = a["op"].as_str().unwrap_or_default().replace('_', " ");
                let Some(op) = crate::mesh_tool::MeshOp::ALL.into_iter().find(|o| o.label().eq_ignore_ascii_case(&op_name)) else {
                    return err(format!("unknown mesh op {op_name}, use one of {}", mesh_op_names().join(", ")));
                };
                let _ = self.run_app_action(Action::MeshOp(op));
                let status = self.fresh_status(started);
                if let Some(s) = status.as_deref().filter(|s| is_failure_status(s)) {
                    return err(s);
                }

                return ok(json!({ "ok": true, "status": status }));
            }
            "store_camera" | "recall_camera" => {
                let slot = a["slot"].as_u64().unwrap_or(1).clamp(1, 9) as u8;
                let action = if name == "store_camera" { Action::StoreCamera(slot) } else { Action::RecallCamera(slot) };
                let _ = self.run_app_action(action);
                return ok(json!({ "ok": true, "status": self.fresh_status(started) }));
            }
            "set_shade" => {
                let name = a["shade"].as_str().unwrap_or("lit");
                match crate::state::Shade::from_name(name) {
                    Some(s) => Action::SetShade(s),
                    None => return err(format!("unknown shade {name}, use textured, flat, lit or wireframe")),
                }
            }
            "edit_mesh" => Action::EditMesh,
            "convert_to_mesh" => Action::ConvertToMesh,
            "convert_to_brushes" => Action::ConvertToBrushes,
            "join_meshes" => Action::JoinMeshes,
            "duplicate_linked" => Action::DuplicateLinked,
            "unlink_groups" => Action::UnlinkGroups,
            "set_cordon" => Action::SetCordonFromSelection,
            "toggle_cordon" => Action::ToggleCordon,
            "clear_cordon" => Action::ClearCordon,
            "new_tab" => Action::NewTab,
            "next_tab" => Action::NextTab,
            "close_tab" => {
                if a["close_others"].as_bool().unwrap_or(false) {
                    return match self.state.close_other_tabs(a["discard"].as_bool().unwrap_or(false)) {
                        Ok(closed) => ok(json!({ "ok": true, "closed": closed, "tabs": self.state.tab_titles().0 })),
                        Err(e) => err(e),
                    };
                }

                if a["discard"].as_bool().unwrap_or(false) {
                    self.state.discard_tab();
                    return ok(json!({ "ok": true, "tabs": self.state.tab_titles().0 }));
                }

                if self.state.doc.is_modified() {
                    return err("the map has unsaved changes, save it with map_file save, or pass args.discard true to drop them");
                }

                Action::CloseTab
            }
            "hotspot_texture" => Action::HotspotTexture,
            "terrain_auto_paint" => {
                if let Some(base) = a["sea_level"].as_f64() {
                    self.state.auto_paint_base = Some(base);
                }

                Action::TerrainAutoPaint
            }
            "reload_models" => Action::ReloadModels,
            "reload_materials" => {
                let game = self.state.game.clone();
                self.state.materials.rescan(&game);
                self.project_generation += 1;
                return ok(json!({ "ok": true, "materials": self.state.materials.entries.len() }));
            }
            "reload_game_config" => {
                let Some(root) = self.state.game.project_root.clone() else { return err("no project is open, use open_project") };
                self.state.load_project(&root);
                self.project_generation += 1;
                return ok(json!({ "ok": true, "entities": self.state.game.entities.len(), "status": self.state.status }));
            }
            "open_godot_editor" => Action::OpenGodotEditor,
            "run_godot_project" => Action::RunGodotProject,
            "focus_godot" => Action::FocusGodot,
            "build_in_godot" => Action::BuildInGodot,
            "toggle_live_mode" => Action::ToggleLiveMode,
            "toggle_walkable" => Action::ToggleWalkable,
            "insert_prefab" => {
                let Some(path) = a["path"].as_str() else { return err("path required") };
                let parent = match resolve_parent(&self.state, &a["parent"], Child::Other) {
                    Ok(p) => p,
                    Err(e) => return err(e),
                };
                let reference =
                    crate::commands::prefab_reference(std::path::Path::new(path), self.state.doc.path.as_deref(), self.state.game.project_root.as_deref());
                let origin = vec3(&a["origin"]).unwrap_or_default();
                let angles = vec3(&a["angles"]).unwrap_or_default();
                let fixup = a["fixup"].as_str().unwrap_or_default().to_string();
                let id = self.state.doc.edit("Insert Prefab", |m, s| {
                    let id = m.insert(parent, NodeKind::Instance(gt_doc::map::Instance { path: reference.clone(), origin, angles, fixup }));
                    s.clear();
                    s.select_node(id);
                    id
                });
                return ok(json!({ "id": id.0, "path": reference }));
            }
            "move_vertices" => {
                let verts: Vec<DVec3> = a["vertices"].as_array().into_iter().flatten().filter_map(vec3).collect();
                let (Some(offset), false) = (vec3(&a["offset"]), verts.is_empty()) else {
                    return err("move_vertices needs vertices [[x, y, z], ...] and an offset");
                };
                if self.state.doc.selection.brushes(&self.state.doc.map).is_empty() {
                    return err("select the brushes whose vertices to move first");
                }

                if !crate::toolset::move_vertices(&mut self.state, &verts, offset) {
                    return err("no selected brush has those vertices, or moving them would make it concave");
                }

                return ok(json!({ "ok": true, "selection": self.state.doc.selection.nodes.iter().map(|i| i.0).collect::<Vec<_>>() }));
            }
            "clip_apply" => {
                let before = self.state.doc.map.brush_count();
                let plane = match (vec3(&a["point"]), vec3(&a["normal"]), a["points"].as_array()) {
                    (Some(p), Some(n), _) if n.length() > 1e-9 => Some(gt_core::Plane::from_point_normal(p, n.normalize())),
                    (_, _, Some(pts)) => match pts.iter().filter_map(vec3).collect::<Vec<_>>()[..] {
                        [p0, p1, p2] => match gt_core::Plane::from_points(p0, p1, p2) {
                            Some(plane) => Some(plane),
                            None => return err("the three clip points lie on one line"),
                        },
                        _ => return err("points needs three [x, y, z] points"),
                    },
                    (Some(_), _, _) | (_, Some(_), _) => return err("clip by plane needs both point and a non zero normal"),
                    _ => None,
                };
                let Some(plane) = plane else {
                    self.tools.sync(&self.state);
                    self.tools.apply_clip_public(&mut self.state);
                    return ok(
                        json!({ "brushes_before": before, "brushes_after": self.state.doc.map.brush_count(), "replaced": replaced_json(&self.state.replaced) }),
                    );
                };
                if self.state.doc.selection.brushes(&self.state.doc.map).is_empty() {
                    return err("select the brushes to clip first");
                }

                let side = match a["keep"].as_str().unwrap_or("front") {
                    "front" => crate::toolset::ClipSide::Front,
                    "back" => crate::toolset::ClipSide::Back,
                    "both" => crate::toolset::ClipSide::Both,
                    other => return err(format!("keep is front, back or both, not {other}")),
                };
                crate::toolset::clip_selection(&mut self.state, &plane, side);
                let selection: Vec<u64> = self.state.doc.selection.nodes.iter().map(|i| i.0).collect();
                return ok(
                    json!({ "brushes_before": before, "brushes_after": self.state.doc.map.brush_count(), "ids": selection, "replaced": replaced_json(&self.state.replaced) }),
                );
            }
            other => return err(format!("unknown action {other}")),
        };
        let project_before = self.state.game.project_root.clone();
        crate::commands::execute(&mut self.state, action, ctx);
        if self.state.game.project_root != project_before {
            self.project_generation += 1;
        }

        if let Some(bounds) = self.state.focus_request.take() {
            for v in &mut self.viewports {
                v.focus(&bounds);
            }
        }

        let status = self.fresh_status(started);
        if let Some(s) = status.as_deref().filter(|s| is_failure_status(s)) {
            return err(s);
        }

        let mut result = json!({ "ok": true, "status": status, "brushes": self.state.doc.map.brush_count(), "selection": self.state.doc.selection.nodes.iter().map(|i| i.0).collect::<Vec<_>>() });
        if !self.state.replaced.is_empty() {
            result["replaced"] = replaced_json(&self.state.replaced);
        }

        ok(result)
    }

    fn tool_create_entity(&mut self, args: &Value) -> ToolResult {
        let Some(classname) = args["classname"].as_str().map(str::trim).filter(|c| !c.is_empty()).map(str::to_string) else {
            return err("classname required");
        };
        let origin = vec3(&args["origin"]).unwrap_or_default();
        let angles = vec3(&args["angles"]).unwrap_or_default();
        let props: Vec<(String, String)> =
            args["properties"].as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), value_string(v))).collect()).unwrap_or_default();
        let outputs: Vec<IoConnection> = match args.get("outputs").filter(|o| !o.is_null()) {
            Some(o) => match serde_json::from_value(o.clone()) {
                Ok(list) => list,
                Err(e) => return err(format!("outputs must be objects with output, target and input: {e}")),
            },
            None => Vec::new(),
        };
        let parent = match resolve_parent(&self.state, &args["parent"], Child::Other) {
            Ok(p) => p,
            Err(e) => return err(e),
        };
        let id = self.state.doc.edit("Create Entity", |m, s| {
            let id = ops::create_point_entity(m, parent, &classname, origin);
            if let Some(e) = m.entity_mut(id) {
                e.angles = angles;
                e.properties.extend(props);
                e.outputs = outputs;
            }

            s.clear();
            s.select_node(id);
            id
        });
        ok(json!({ "id": id.0 }))
    }

    fn tool_import_model(&mut self, args: &Value) -> ToolResult {
        let Some(path) = args["path"].as_str() else { return err("path required") };
        let path = std::path::Path::new(path);
        if !path.is_file() {
            return err(format!("no file at {}", path_text(path)));
        }

        let ext = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()).unwrap_or_default();
        if !crate::models::MODEL_EXTS.contains(&ext.as_str()) {
            return err(format!("unsupported model type .{ext}, use one of {}", crate::models::MODEL_EXTS.join(", ")));
        }

        let mode = match args["mode"].as_str().unwrap_or("mesh") {
            "mesh" => crate::commands::ModelImport::Mesh,
            "brushes" => crate::commands::ModelImport::Brushes,
            "prop" => crate::commands::ModelImport::Prop,
            other => return err(format!("unknown mode {other}, use mesh, brushes or prop")),
        };
        if let Err(e) = resolve_parent(&self.state, &Value::Null, Child::Other) {
            return err(e);
        }

        let origin = vec3(&args["origin"]).unwrap_or_default();
        match crate::commands::import_model(&mut self.state, path, mode, origin) {
            Ok(n) => ok(json!({ "objects": n, "selection": self.state.doc.selection.nodes.iter().map(|i| i.0).collect::<Vec<_>>() })),
            Err(e) => err(e),
        }
    }

    fn tool_select(&mut self, args: &Value) -> ToolResult {
        let (ids, faces) = match (id_list(args, "ids"), face_list(args, "faces")) {
            (Ok(i), Ok(f)) => (i, f),
            (Err(e), _) | (_, Err(e)) => return err(e),
        };
        let mode = args["mode"].as_str().unwrap_or("replace").to_string();
        if !matches!(mode.as_str(), "replace" | "add" | "remove" | "clear") {
            return err(format!("unknown mode {mode}, use replace, add, remove or clear"));
        }

        let map = &self.state.doc.map;
        let mut missing: Vec<u64> = ids.iter().chain(faces.iter().map(|(i, _)| i)).filter(|i| !map.contains(**i)).map(|i| i.0).collect();
        missing.dedup();
        if !missing.is_empty() {
            return err(format!("no nodes with ids {missing:?}"));
        }

        for (id, f) in &faces {
            let count = match (map.brush(*id), map.mesh(*id)) {
                (Some(b), _) => b.faces.len(),
                (_, Some(m)) => m.faces.len(),
                _ => return err(format!("{id} is not a brush or mesh, faces are [brush or mesh id, face index] pairs")),
            };
            if *f >= count {
                return err(format!("{id} has faces 0..{}, got {f}", count.saturating_sub(1)));
            }
        }

        // Removing needs no editability, adding skips what the viewports could not pick either.
        let adding = mode != "remove";
        let skipped: Vec<u64> = if adding {
            let mut s: Vec<u64> = ids.iter().chain(faces.iter().map(|(i, _)| i)).filter(|i| !map.is_editable(**i)).map(|i| i.0).collect();
            s.dedup();
            s
        } else {
            Vec::new()
        };
        let ids: Vec<NodeId> = ids.into_iter().filter(|i| !skipped.contains(&i.0)).collect();
        let faces: Vec<(NodeId, usize)> = faces.into_iter().filter(|(i, _)| !skipped.contains(&i.0)).collect();
        self.state.doc.select(|_, s| match mode.as_str() {
            "clear" => s.clear(),
            "remove" => {
                for id in &ids {
                    s.nodes.remove(id);
                }

                for f in &faces {
                    s.faces.remove(f);
                }
            }
            m => {
                if m == "replace" {
                    s.clear();
                }

                if !faces.is_empty() {
                    s.nodes.clear();
                    s.faces.extend(faces.iter().copied());
                } else {
                    s.faces.clear();
                    s.nodes.extend(ids.iter().copied());
                }
            }
        });
        let b = self.state.doc.map.bounds_of(self.state.doc.selection.nodes.iter().copied());
        if !b.is_empty() {
            self.state.last_bounds = b;
        }

        ok(json!({
            "selected": self.state.doc.selection.nodes.iter().map(|i| i.0).collect::<Vec<_>>(),
            "faces": self.state.doc.selection.faces.iter().map(|(i, f)| json!([i.0, f])).collect::<Vec<_>>(),
            "skipped": skipped,
        }))
    }

    /// After an import: `textures: "auto"` converts the missing textures found near the map, a path converts them
    /// from there, and without it the reply only says what could be converted, the way the editor offers it.
    fn import_textures(&mut self, map_file: &std::path::Path, args: &Value) -> Result<Value, String> {
        let sources = match args["textures"].as_str() {
            Some("auto") | None => crate::texture_convert::sources_for_map(&self.state, map_file),
            Some(p) => vec![std::path::PathBuf::from(p)],
        };
        let lib = gt_formats::texture_import::scan(&sources);
        let wanted = crate::texture_convert::convertible(&self.state, &lib);
        let mut v = json!({
            "texture_sources": sources.iter().map(path_text).collect::<Vec<_>>(),
            "missing_materials": crate::texture_convert::missing_materials(&self.state).len(),
            "convertible": wanted.len(),
        });
        // Runs with nothing to convert too, the sky colors still come from the map's sky texture.
        if args["textures"].is_string() && (!wanted.is_empty() || self.state.game.project_root.is_some()) {
            let report = crate::texture_convert::convert(&mut self.state, &lib, Some(&wanted), false)?;
            v["textures"] = crate::texture_convert::report_json(&report);
        }

        Ok(v)
    }

    fn tool_map_file(&mut self, args: &Value) -> ToolResult {
        let op = args["op"].as_str().unwrap_or_default();
        // A relative path resolves against the process directory, which a later save should not depend on.
        let path = args["path"]
            .as_str()
            .map(std::path::PathBuf::from)
            .map(|p| if matches!(op, "open" | "open_tab") { std::path::absolute(&p).unwrap_or(p) } else { p });
        let need = |p: Option<std::path::PathBuf>| p.ok_or_else(|| format!("map_file {op} needs a path"));
        let modified = self.state.doc.is_modified();
        let result: Result<Value, String> = match op {
            // Like File > New: unsaved work stays open in its own tab.
            "new" => {
                if modified {
                    self.state.open_tab(gt_doc::Document::new());
                    self.state.set_status("New map opened in a tab, the current map has unsaved changes");
                } else {
                    self.state.reset_document(gt_doc::Document::new());
                    self.state.set_status("New map");
                }

                Ok(json!({ "new_tab": modified }))
            }
            "open" => need(path).and_then(|p| {
                if modified { crate::commands::open_map_in_tab(&mut self.state, &p) } else { self.state.open_map(&p) }
                    .map(|()| json!({ "new_tab": modified, "problems": self.state.doc.load_problems }))
            }),
            "save" => match path.or_else(|| self.state.doc.path.clone()) {
                Some(p) => self.state.save_map(&p).map(|()| json!({})),
                None => Err("map has no path yet, pass one".into()),
            },
            "import_map" => need(path).and_then(|p| {
                let report = crate::commands::import_quake_map(&mut self.state, &p)?;
                let mut v = self.import_textures(&p, args)?;
                v["skipped_patches"] = json!(report.skipped_patches);
                Ok(v)
            }),
            "import_vmf" => need(path).and_then(|p| {
                let report = crate::commands::import_vmf(&mut self.state, &p)?;
                let mut v = self.import_textures(&p, args)?;
                v["instances"] = json!(report.instances);
                v["missing_instances"] = json!(report.missing_instances);
                Ok(v)
            }),
            "convert_textures" => need(path).and_then(|p| {
                let lib = gt_formats::texture_import::scan(std::slice::from_ref(&p));
                let only = args["only_missing"].as_bool().unwrap_or(false).then(|| crate::texture_convert::convertible(&self.state, &lib));
                let report = crate::texture_convert::convert(&mut self.state, &lib, only.as_ref(), args["overwrite"].as_bool().unwrap_or(false))?;
                self.state.set_status(format!("Textures: {}", report.summary()));
                Ok(json!({ "found": lib.len(), "textures": crate::texture_convert::report_json(&report) }))
            }),
            "open_tab" => need(path).and_then(|p| crate::commands::open_map_in_tab(&mut self.state, &p)).map(|()| json!({})),
            "export_map" => need(path).and_then(|p| {
                std::fs::write(&p, gt_formats::quake_map::export(&self.state.doc.map))
                    .map(|()| json!({ "path": path_text(&p), "map_path": path_value(self.state.doc.path.as_ref()) }))
                    .map_err(|e| format!("cannot write {}: {e}", path_text(&p)))
            }),
            "export_glb" | "export_obj" => need(path).and_then(|p| {
                let format = if op == "export_glb" { crate::export3d::Format::Glb } else { crate::export3d::Format::Obj };
                let defaults = crate::export3d::Options::default();
                let flag = |key: &str, default: bool| args[key].as_bool().unwrap_or(default);
                let options = crate::export3d::Options {
                    selection_only: flag("selection_only", defaults.selection_only),
                    hidden: flag("hidden", defaults.hidden),
                    scatter: flag("scatter", defaults.scatter),
                    models: flag("models", defaults.models),
                    markers: flag("markers", defaults.markers),
                    merge_brushes: flag("merge_brushes", defaults.merge_brushes),
                };
                let report = crate::export3d::export(crate::export3d::Sources::of(&mut self.state), &p, format, &options)?;
                self.state.set_status(report.summary(&p));
                Ok(json!({
                    "path": path_text(&p),
                    "map_path": path_value(self.state.doc.path.as_ref()),
                    "objects": report.objects,
                    "meshes": report.meshes,
                    "materials": report.materials,
                    "images": report.images,
                    "triangles": report.triangles,
                    "bytes": report.bytes,
                    "missing_textures": report.missing,
                }))
            }),
            _ => Err(format!("unknown op {op}, use new, open, open_tab, save, import_map, import_vmf, convert_textures, export_map, export_glb or export_obj")),
        };
        self.project_generation += 1;
        match result {
            Ok(extra) => {
                let mut v = json!({ "ok": true, "path": path_value(self.state.doc.path.as_ref()) });
                if let Some(o) = extra.as_object() {
                    for (k, x) in o {
                        v[k] = x.clone();
                    }
                }

                ok(v)
            }
            Err(e) => err(e),
        }
    }

    fn tool_set_editor(&mut self, args: &Value, ctx: &egui::Context) -> ToolResult {
        // Everything is validated first, so a bad value changes nothing.
        let grid = match &args["grid"] {
            Value::Null => None,
            v => match v.as_f64().filter(|g| g.is_finite() && *g > 0.0) {
                Some(g) => Some(g.clamp(0.125, 1024.0)),
                None => return err(format!("grid must be a positive number, got {v}")),
            },
        };
        let shade = match args["shade"].as_str() {
            Some(s) => match crate::state::Shade::from_name(s) {
                Some(shade) => Some(shade),
                None => return err(format!("unknown shade {s}, use textured, flat, lit or wireframe")),
            },
            None => None,
        };
        let component = match args["mesh_component"].as_str() {
            Some("vertex") => Some(crate::mesh_tool::Component::Vertex),
            Some("edge") => Some(crate::mesh_tool::Component::Edge),
            Some("face") => Some(crate::mesh_tool::Component::Face),
            Some(other) => return err(format!("unknown mesh_component {other}, use vertex, edge or face")),
            None => None,
        };
        let sculpt_mode = match args["sculpt_mode"].as_str() {
            Some(mode) => match serde_json::from_value(json!(mode)) {
                Ok(m) => Some(m),
                Err(_) => return err(format!("unknown sculpt mode {mode}")),
            },
            None => None,
        };
        let tool = match args["tool"].as_str() {
            Some(t) => match ToolKind::from_name(t) {
                Some(k) => Some(k),
                None => return err(format!("unknown tool {t}")),
            },
            None => None,
        };

        if let Some(g) = grid {
            self.state.grid = g;
        }

        if let Some(b) = args["snap"].as_bool() {
            self.state.snap = b;
        }

        if let Some(b) = args["uv_lock"].as_bool() {
            self.state.uv_lock = b;
        }

        if let Some(b) = args["textured"].as_bool() {
            self.state.prefs.shade = if b { crate::state::Shade::Textured } else { crate::state::Shade::Flat };
        }

        if let Some(s) = shade {
            self.state.prefs.shade = s;
        }

        if let Some(c) = component {
            self.tools.mesh.component = c;
        }

        if let Some(items) = args["scatter_items"].as_array().or(args["sprinkle_items"].as_array()) {
            self.state.prefs.scatter.palette = items.iter().filter_map(|i| i.as_str().map(gt_doc::ScatterItem::new)).collect();
            self.state.prefs.scatter.preset.clear();
            if let Some(id) = crate::scatter_tool::active_set(&self.state) {
                let palette = self.state.prefs.scatter.palette.clone();
                crate::scatter_tool::set_palette(&mut self.state, id, &palette);
            }
        }

        if let Some(r) = args["brush_radius"].as_f64() {
            self.state.sculpt.radius = r;
            self.state.blend.radius = r;
            self.state.prefs.scatter.radius = r;
        }

        if let Some(scale) = args["ui_scale"].as_f64() {
            self.state.prefs.ui_scale = (scale as f32).clamp(crate::state::UI_SCALE_MIN, crate::state::UI_SCALE_MAX);
        }

        if let Some(follow) = args["follow_display_scaling"].as_bool() {
            self.state.prefs.set_follow_display_scaling(follow, ctx.native_pixels_per_point().unwrap_or(1.0));
        }

        if let Some(m) = sculpt_mode {
            self.state.sculpt.mode = m;
        }

        if let Some(m) = args["material"].as_str() {
            self.state.current_material = m.to_string();
        }

        if let Some(k) = tool {
            self.state.tool = k;
        }

        if let Some(live) = args["live_mode"].as_bool() {
            self.state.prefs.live_mode = live;
        }

        if let Some(port) = args["live_link_port"].as_u64() {
            self.state.prefs.live_link_port = port.clamp(1024, 65535) as u16;
        }

        let mut summary = self.state_summary();
        let mut editor = summary["editor"].take();
        editor["godot_live_mode"] = json!(self.state.prefs.live_mode);
        ok(editor)
    }

    fn tool_create_brush(&mut self, args: &Value) -> ToolResult {
        let (Some(min), Some(max)) = (vec3(&args["min"]), vec3(&args["max"])) else { return err("min and max are required") };
        let bounds = Aabb::new(min, max);
        let material = args["material"].as_str().map(str::to_string).unwrap_or_else(|| self.state.current_material.clone());
        let sides = args["sides"].as_u64().unwrap_or(12) as usize;
        let thickness = args["thickness"].as_f64().unwrap_or(16.0);
        let parent = match resolve_parent(&self.state, &args["parent"], Child::Geometry) {
            Ok(p) => p,
            Err(e) => return err(e),
        };
        let entity = match args["entity"].as_object() {
            None => None,
            Some(_) if self.state.doc.map.entity(parent).is_some() => return err("entity cannot wrap brushes whose parent is already a brush entity"),
            Some(e) => {
                let mut ent = gt_doc::Entity::new(e.get("classname").and_then(|c| c.as_str()).unwrap_or("func_detail"));
                if let Some(props) = e.get("properties").and_then(|p| p.as_object()) {
                    ent.properties.extend(props.iter().map(|(k, v)| (k.clone(), value_string(v))));
                }

                if let Some(outs) = e.get("outputs").filter(|o| !o.is_null()) {
                    match serde_json::from_value::<Vec<IoConnection>>(outs.clone()) {
                        Ok(list) => ent.outputs = list,
                        Err(e) => return err(format!("entity.outputs must be objects with output, target and input: {e}")),
                    }
                }

                Some(ent)
            }
        };
        let mut letter_bounds: Vec<Aabb> = Vec::new();
        let brushes: Vec<Brush> = match args["shape"].as_str().unwrap_or("box") {
            "box" => Brush::from_aabb(&bounds, &material).into_iter().collect(),
            "cylinder" => shapes::cylinder(&bounds, sides, &material).into_iter().collect(),
            "cone" => shapes::cone(&bounds, sides, &material).into_iter().collect(),
            "spike" => shapes::spike(&bounds, &material).into_iter().collect(),
            "wedge" => shapes::wedge(&bounds, &material).into_iter().collect(),
            "sphere" => shapes::sphere(&bounds, sides, (sides / 2).max(3), &material),
            "arch" => shapes::arch(&bounds, sides, thickness, &material),
            "pipe" => shapes::pipe(&bounds, sides, thickness, &material),
            "stairs" => shapes::stairs(&bounds, args["steps"].as_u64().unwrap_or(8) as usize, &material),
            "spiral_stairs" => {
                shapes::spiral_stairs(&bounds, args["steps"].as_u64().unwrap_or(16) as usize, thickness, args["turns"].as_f64().unwrap_or(1.0), &material)
            }
            "gable" => {
                let (a, b) = (bounds.min, bounds.max);
                let pts: Vec<DVec3> = if args["ridge_x"].as_bool().unwrap_or(false) {
                    let cz = (a.z + b.z) * 0.5;
                    vec![
                        DVec3::new(a.x, a.y, a.z),
                        DVec3::new(a.x, a.y, b.z),
                        DVec3::new(a.x, b.y, cz),
                        DVec3::new(b.x, a.y, a.z),
                        DVec3::new(b.x, a.y, b.z),
                        DVec3::new(b.x, b.y, cz),
                    ]
                } else {
                    let cx = (a.x + b.x) * 0.5;
                    vec![
                        DVec3::new(a.x, a.y, a.z),
                        DVec3::new(b.x, a.y, a.z),
                        DVec3::new(cx, b.y, a.z),
                        DVec3::new(a.x, a.y, b.z),
                        DVec3::new(b.x, a.y, b.z),
                        DVec3::new(cx, b.y, b.z),
                    ]
                };
                Brush::from_points(&pts, &[], &material).into_iter().collect()
            }
            "text" => {
                let Some(text) = args["text"].as_str() else { return err("text shape needs text") };
                match gt_geom::block_text::layout(text, &bounds, args["standing"].as_bool().unwrap_or(false), args["spacing"].as_f64().unwrap_or(1.0)) {
                    Ok(letters) => {
                        letter_bounds = letters.iter().map(|boxes| boxes.iter().fold(Aabb::EMPTY, |acc, b| acc.union(b))).collect();
                        letters.iter().flatten().filter_map(|b| Brush::from_aabb(b, &material).ok()).collect()
                    }
                    Err(e) => return err(e),
                }
            }
            other => return err(format!("unknown shape {other}")),
        };
        if brushes.is_empty() {
            return err("shape produced no valid brushes, check the bounds");
        }

        let mut brushes = brushes;
        if let Some(t) = args["hollow"].as_f64().filter(|t| *t > 0.0) {
            brushes = brushes.iter().flat_map(|b| gt_geom::csg::hollow(b, t)).collect();
        }

        let mut cutters = Vec::new();
        for opening in args["openings"].as_array().into_iter().flatten() {
            let (min, max) = match opening {
                Value::Array(pair) if pair.len() == 2 => (vec3(&pair[0]), vec3(&pair[1])),
                Value::Object(_) => (vec3(&opening["min"]), vec3(&opening["max"])),
                _ => (None, None),
            };
            let (Some(min), Some(max)) = (min, max) else { return err("openings are [[min], [max]] pairs or {min, max, count, step, rows, row_step}") };
            let (count, rows) = (opening["count"].as_u64().unwrap_or(1).max(1), opening["rows"].as_u64().unwrap_or(1).max(1));
            if count * rows > 1024 {
                return err("an opening repeats at most 1024 times");
            }

            let (step, row_step) = (vec3(&opening["step"]).unwrap_or_default(), vec3(&opening["row_step"]).unwrap_or_default());
            for r in 0..rows {
                for c in 0..count {
                    let offset = step * c as f64 + row_step * r as f64;
                    let Ok(cutter) = Brush::from_aabb(&Aabb::new(min + offset, max + offset), &material) else { return err("opening has no volume") };
                    cutters.push(cutter);
                }
            }
        }

        for cutter in &cutters {
            brushes = brushes.iter().flat_map(|b| if b.intersects(cutter) { gt_geom::csg::subtract(b, cutter) } else { vec![b.clone()] }).collect();
        }

        if let Some(s) = args["uv_scale"].as_f64() {
            for b in &mut brushes {
                for f in &mut b.faces {
                    f.data.uv.scale = DVec2::splat(s);
                }
            }
        }

        let (ids, entity_id) = self.state.doc.edit("Create Brush", |m, s| {
            s.clear();
            let (container, entity_id) = match entity {
                Some(e) => {
                    let id = m.insert(parent, NodeKind::Entity(e));
                    (id, Some(id))
                }
                None => (parent, None),
            };
            let ids = brushes
                .into_iter()
                .map(|b| {
                    let id = m.insert(container, NodeKind::Brush(b));
                    if entity_id.is_none() {
                        s.nodes.insert(id);
                    }

                    id.0
                })
                .collect::<Vec<_>>();
            if let Some(e) = entity_id {
                s.nodes.insert(e);
            }

            (ids, entity_id)
        });
        self.state.last_bounds = bounds;
        if letter_bounds.is_empty() {
            return ok(json!({ "ids": ids, "entity": entity_id.map(|e| e.0) }));
        }

        let map = &self.state.doc.map;
        let letters: Vec<Vec<u64>> = letter_bounds
            .iter()
            .map(|lb| {
                let inside = |id: &&u64| map.brush(NodeId(**id)).is_some_and(|b| lb.expanded(1e-6).contains_point(b.bounds().center()));
                ids.iter().filter(inside).copied().collect()
            })
            .collect();
        ok(json!({ "ids": ids, "entity": entity_id.map(|e| e.0), "letters": letters }))
    }

    fn tool_update_entity(&mut self, args: &Value) -> ToolResult {
        let id = match require_id(args, "id") {
            Ok(id) => id,
            Err(e) => return err(e),
        };
        if self.state.doc.map.entity(id).is_none() {
            return err(format!("{id} is not an entity"));
        }

        let classname = args["classname"].as_str().map(str::to_string);
        if classname.as_deref().is_some_and(|c| c.trim().is_empty()) {
            return err("classname cannot be empty");
        }

        let origin = vec3(&args["origin"]);
        let angles = vec3(&args["angles"]);
        let props = args["properties"].as_object().cloned();
        let outputs: Option<Vec<IoConnection>> = match args.get("outputs").filter(|v| !v.is_null()) {
            Some(v) => match serde_json::from_value(v.clone()) {
                Ok(list) => Some(list),
                Err(e) => return err(format!("outputs must be objects with output, target and input: {e}")),
            },
            None => None,
        };

        self.state.doc.edit("Update Entity", |m, _| {
            let Some(e) = m.entity_mut(id) else { return };
            if let Some(c) = classname {
                e.classname = c;
            }

            if let Some(o) = origin {
                e.origin = o;
            }

            if let Some(a) = angles {
                e.angles = a;
            }

            if let Some(p) = props {
                for (k, v) in p {
                    if v.is_null() {
                        e.properties.remove(&k);
                    } else {
                        e.properties.insert(k, value_string(&v));
                    }
                }
            }

            if let Some(o) = outputs {
                e.outputs = o;
            }
        });
        match self.state.doc.map.entity(id) {
            Some(e) => ok(serde_json::to_value(e).unwrap_or_default()),
            None => err(format!("{id} vanished")),
        }
    }

    /// Applied in order translate, rotate, flip, scale_to as one undo step. rotate and flip pivot on the selection
    /// center as it is at that step, so a translated selection flips in place at its new position.
    fn tool_transform(&mut self, args: &Value) -> ToolResult {
        if self.state.doc.selection.nodes.is_empty() {
            return err("nothing selected");
        }

        let blocked: Vec<u64> = self.state.doc.selection.nodes.iter().filter(|id| !self.state.doc.map.is_editable(**id)).map(|i| i.0).collect();
        if !blocked.is_empty() {
            return err(format!("selection holds hidden or locked nodes {blocked:?}"));
        }

        let translate = match &args["translate"] {
            Value::Null => None,
            v => match vec3(v) {
                Some(t) => Some(t),
                None => return err("translate must be [x, y, z]"),
            },
        };
        let rotate = match &args["rotate"] {
            Value::Null => None,
            Value::Object(r) => {
                let axis = match r.get("axis") {
                    None => 1,
                    Some(a) => match axis_index(a) {
                        Some(i) => i,
                        None => return err("rotate.axis must be x, y or z"),
                    },
                };
                Some((axis, r.get("degrees").and_then(|d| d.as_f64()).unwrap_or(0.0), r.get("center").and_then(vec3)))
            }
            _ => return err("rotate must be {axis, degrees, center}"),
        };
        let flip = match &args["flip"] {
            Value::Null => None,
            v => match axis_index(v) {
                Some(i) => Some(i),
                None => return err("flip must be x, y or z"),
            },
        };
        let scale_to = match &args["scale_to"] {
            Value::Null => None,
            s => match (vec3(&s["min"]), vec3(&s["max"])) {
                (Some(min), Some(max)) => Some(Aabb::new(min, max)),
                _ => return err("scale_to needs min and max"),
            },
        };
        if translate.is_none() && rotate.is_none() && flip.is_none() && scale_to.is_none() {
            return err("pass translate, rotate, flip or scale_to");
        }

        let opts = self.state.opts();
        let grid = self.state.grid;
        let started = !self.state.doc.in_transaction();
        self.state.doc.begin("Transform");
        if let Some(t) = translate {
            self.state.doc.edit("Move", |m, s| ops::translate_selection(m, s, t, opts));
        }

        if let Some((axis, degrees, center)) = rotate {
            let c = center.unwrap_or_else(|| ops::selection_center(&self.state.doc.map, &self.state.doc.selection, grid));
            let mut dir = DVec3::ZERO;
            dir[axis] = 1.0;
            let mat = ops::rotation_about(c, dir, degrees);
            self.state.doc.edit("Rotate", |m, s| ops::transform_selection(m, s, &mat, opts));
        }

        if let Some(axis) = flip {
            let center = ops::selection_center(&self.state.doc.map, &self.state.doc.selection, grid);
            let mat = ops::flip_about(center, axis);
            self.state.doc.edit("Flip", |m, s| ops::transform_selection(m, s, &mat, opts));
        }

        if let Some(target) = scale_to {
            let old = self.state.doc.map.bounds_of(self.state.doc.selection.nodes.iter().copied());
            let mat = ops::scale_bounds(&old, &target);
            self.state.doc.edit("Scale", |m, s| ops::transform_selection(m, s, &mat, opts));
        }

        if started {
            self.state.doc.commit();
        }

        ok(json!({ "bounds": bounds_json(&self.state.doc.map.bounds_of(self.state.doc.selection.nodes.iter().copied())) }))
    }

    fn tool_set_face(&mut self, args: &Value) -> ToolResult {
        let id = match require_id(args, "id") {
            Ok(id) => id,
            Err(e) => return err(e),
        };
        let Some(face) = uint(&args["face"]).map(|f| f as usize) else { return err("face required, a face index") };
        let Some(brush) = self.state.doc.map.brush(id) else { return err(format!("{id} is not a brush")) };
        if face >= brush.faces.len() {
            return err(format!("brush has {} faces", brush.faces.len()));
        }

        if !self.state.doc.map.is_editable(id) {
            return err(format!("{id} is hidden or locked"));
        }

        let material = args["material"].as_str().map(str::to_string);
        let offset = vec2(&args["offset"]);
        let scale = vec2(&args["scale"]);
        let rotate = args["rotate_by"].as_f64();
        let fit = args["fit"].as_bool().unwrap_or(false);
        let mat_name = material.clone().unwrap_or_else(|| brush.faces[face].data.material.clone());
        self.state.find_new_materials([mat_name.as_str()]);
        let size = crate::texture_ops::tex_size(&self.state, &mat_name);
        self.state.doc.edit("Set Face", |m, _| {
            let Some(b) = m.brush_mut(id) else { return };
            let pts: Vec<DVec3> = b.faces[face].indices.iter().map(|i| b.vertices[*i as usize]).collect();
            let f = &mut b.faces[face];
            if let Some(mat) = material {
                f.data.material = mat;
            }

            if let Some(o) = offset {
                f.data.uv.offset = o;
            }

            if let Some(s) = scale {
                f.data.uv.scale = s;
            }

            if let Some(r) = rotate {
                f.data.uv.rotate(r);
            }

            if fit {
                f.data.uv.fit(&pts, size, DVec2::ONE);
            }
        });
        let Some(b) = self.state.doc.map.brush(id) else { return err(format!("{id} vanished")) };
        ok(
            json!({ "material": b.faces[face].data.material, "uv": serde_json::to_value(&b.faces[face].data.uv).unwrap_or_default(), "normal": arr(b.faces[face].plane.normal) }),
        )
    }

    fn resolve_pos(&self, target: &str, ev: &Value, key_x: &str, key_world: &str) -> Result<Pos2, String> {
        if target == "window" {
            let (x, y) = if key_x == "x" { (ev["x"].as_f64(), ev["y"].as_f64()) } else { (ev[key_x][0].as_f64(), ev[key_x][1].as_f64()) };
            return match (x, y) {
                (Some(x), Some(y)) => Ok(Pos2::new(x as f32, y as f32)),
                _ => Err("window events need x and y".into()),
            };
        }

        let kind = view_kind(target).ok_or("unknown target")?;
        let vp = self.viewports.iter().find(|v| v.kind() == kind).ok_or("view not open")?;
        if let Some(w) = vec3(&ev[key_world]) {
            let p = vp.camera.project(vp.rect, w).ok_or_else(|| "world position is behind the camera".to_string())?;
            // Drags may end outside the view, but a press there would land on another panel.
            if key_world == "world" && !vp.rect.contains(p) {
                return Err(format!("world position [{}, {}, {}] is outside the {target} view, move its camera with set_camera first", w.x, w.y, w.z));
            }

            return Ok(p);
        }

        let (x, y) = if key_x == "x" { (ev["x"].as_f64(), ev["y"].as_f64()) } else { (ev[key_x][0].as_f64(), ev[key_x][1].as_f64()) };
        match (x, y) {
            (Some(x), Some(y)) => Ok(vp.rect.min + Vec2::new(x as f32, y as f32)),
            // No position: the view center.
            _ => Ok(vp.rect.center()),
        }
    }

    fn build_input_script(&self, args: &Value, reply: Sender<ToolResult>) -> Result<InputScript, String> {
        let target = args["target"].as_str().unwrap_or("window");
        let mut steps = VecDeque::new();
        let events = args["events"].as_array().ok_or("events must be an array")?;
        for ev in events {
            let mut modifiers = Modifiers::NONE;
            for m in ev["modifiers"].as_array().into_iter().flatten().filter_map(|m| m.as_str()) {
                match m {
                    "ctrl" => {
                        modifiers.ctrl = true;
                        modifiers.command = true;
                    }
                    "shift" => modifiers.shift = true,
                    "alt" => modifiers.alt = true,
                    _ => {}
                }
            }

            let button = match ev["button"].as_str().unwrap_or("left") {
                "right" => PointerButton::Secondary,
                "middle" => PointerButton::Middle,
                _ => PointerButton::Primary,
            };
            let step = |events: Vec<Event>| InputStep { events, modifiers };
            let press = |pos: Pos2, pressed: bool| Event::PointerButton { pos, button, pressed, modifiers };
            match ev["type"].as_str().unwrap_or_default() {
                "move" => {
                    let p = self.resolve_pos(target, ev, "x", "world")?;
                    steps.push_back(step(vec![Event::PointerMoved(p)]));
                }
                t @ ("click" | "double_click") => {
                    let p = self.resolve_pos(target, ev, "x", "world")?;
                    steps.push_back(step(vec![Event::PointerMoved(p)]));
                    let times = if t == "click" { 1 } else { 2 };
                    for _ in 0..times {
                        steps.push_back(step(vec![press(p, true)]));
                        steps.push_back(step(vec![press(p, false)]));
                    }

                    steps.push_back(step(vec![]));
                }
                "drag" => {
                    let from = self.resolve_pos(target, ev, "x", "world")?;
                    let to = self.resolve_pos(target, ev, "to", "to_world")?;
                    let n = ev["steps"].as_u64().unwrap_or(8).max(2) as usize;
                    steps.push_back(step(vec![Event::PointerMoved(from)]));
                    steps.push_back(step(vec![press(from, true)]));
                    for i in 1..=n {
                        steps.push_back(step(vec![Event::PointerMoved(from.lerp(to, i as f32 / n as f32))]));
                    }

                    steps.push_back(step(vec![press(to, false)]));
                    steps.push_back(step(vec![]));
                }
                "key" => {
                    let name = ev["key"].as_str().ok_or("key events need key")?;
                    let key = Key::from_name(name).ok_or_else(|| format!("unknown key {name}"))?;
                    steps.push_back(step(vec![Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers }]));
                    steps.push_back(step(vec![Event::Key { key, physical_key: None, pressed: false, repeat: false, modifiers }]));
                }
                "text" => steps.push_back(step(vec![Event::Text(ev["text"].as_str().unwrap_or_default().to_string())])),
                "scroll" => {
                    let p = self.resolve_pos(target, ev, "x", "world")?;
                    let d = ev["delta"].as_f64().unwrap_or(120.0) as f32;
                    steps.push_back(step(vec![Event::PointerMoved(p)]));
                    steps.push_back(step(vec![Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: Vec2::new(0.0, d),
                        modifiers,
                        phase: egui::TouchPhase::Move,
                    }]));
                    steps.push_back(step(vec![]));
                }
                other => return Err(format!("unknown event type {other}")),
            }
        }

        steps.push_back(InputStep { events: vec![], modifiers: Modifiers::NONE });
        Ok(InputScript { steps, reply, frames: 0 })
    }
}

/// run_action mesh_op names, derived from the mesh tool's menu so the list cannot drift.
pub fn mesh_op_names() -> Vec<String> {
    crate::mesh_tool::MeshOp::ALL.iter().map(|o| o.label().to_ascii_lowercase().replace(' ', "_")).collect()
}

fn value_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Array(a) => a.iter().map(value_string).collect::<Vec<_>>().join(" "),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> EditorState {
        EditorState::new(Default::default())
    }

    #[test]
    fn ids_accept_numeric_strings_and_reject_junk() {
        assert_eq!(require_id(&json!({ "id": "42" }), "id"), Ok(NodeId(42)));
        assert_eq!(require_id(&json!({ "id": 7 }), "id"), Ok(NodeId(7)));
        assert!(require_id(&json!({}), "id").unwrap_err().contains("required"));
        assert!(require_id(&json!({ "id": "abc" }), "id").is_err());
        assert_eq!(id_list(&json!({ "ids": [1, "2"] }), "ids"), Ok(vec![NodeId(1), NodeId(2)]));
        assert!(id_list(&json!({ "ids": [1, -3] }), "ids").is_err());
        assert!(face_list(&json!({ "faces": [[1]] }), "faces").is_err());
        assert_eq!(
            path_text(std::path::Path::new("D:/dev/TrenchWorld/godot\\demo/textures\\night\\lit.tres")),
            "D:/dev/TrenchWorld/godot/demo/textures/night/lit.tres"
        );
        assert_eq!(path_value(None::<&std::path::Path>), Value::Null);
        assert_eq!(id_list(&json!({ "ids": [[1, 2], 3, [[4]]] }), "ids"), Ok(vec![NodeId(1), NodeId(2), NodeId(3), NodeId(4)]), "joined id lists flatten");
    }

    #[test]
    fn godot_captures_carry_the_image_and_what_godot_logged() {
        use base64::Engine;
        let reply = json!({ "ok": true, "width": 4, "height": 2, "png": base64::engine::general_purpose::STANDARD.encode([1, 2, 3]), "warnings": ["warning: no model"] });
        let ToolResult::Image { png, note } = godot_capture_result(Ok(reply), "yard.gtm", "built from the map") else { panic!("an image") };
        assert_eq!(png, vec![1, 2, 3]);
        assert!(note.contains("yard.gtm 4x2") && note.contains("built from the map") && note.ends_with("warning: no model"), "{note}");
        let ToolResult::Error(e) = godot_capture_result(Ok(json!({ "ok": true })), "yard.gtm", "") else { panic!("an error") };
        assert!(e.contains("no image"));
        assert!(matches!(godot_capture_result(Err("no scene".into()), "yard.gtm", ""), ToolResult::Error(e) if e == "no scene"));
    }

    #[test]
    fn node_methods_are_valid_inputs_and_typos_are_not() {
        let game = gt_formats::GameConfig::builtin();
        let inputs = crate::validate::class_inputs(&game);
        let mut map = gt_doc::Map::new();
        let layer = map.default_layer();
        let mut light = gt_doc::Entity::new("light");
        let conn = |input: &str, target: &str| IoConnection {
            output: "switched".into(),
            target: target.into(),
            input: input.into(),
            parameter: String::new(),
            delay: 0.0,
            times: -1,
        };
        light.outputs =
            vec![conn("set_visible", "bulb"), conn("set_visibel", "bulb"), conn("hide", "bulb"), conn("turn_on", "lamp"), conn("omni_range", "lamp")];
        light.properties.insert("targetname".into(), "lamp".into());
        map.insert(layer, NodeKind::Entity(light));
        let mut bulb = gt_doc::Entity::new("func_illusionary");
        bulb.properties.insert("targetname".into(), "bulb".into());
        map.insert(layer, NodeKind::Entity(bulb));
        let found = issues::check_with(&map, |class| {
            game.entity(class).map(|d| issues::ClassIo {
                outputs: d.outputs.iter().map(|o| o.name.as_str()).collect(),
                inputs: inputs.get(class).map(|list| list.iter().map(String::as_str).collect()).unwrap_or_default(),
            })
        });
        let unknown: Vec<&str> = found.iter().filter(|i| i.code == "io_unknown_input").map(|i| i.message.as_str()).collect();
        assert_eq!(unknown.len(), 1, "{unknown:?}");
        assert!(unknown[0].contains("set_visibel"), "a func_illusionary is a Node3D, so set_visible passes and the typo does not");
    }

    #[test]
    fn csg_commands_report_the_brushes_they_replaced() {
        let mut s = state();
        let layer = s.doc.map.default_layer();
        let wall = s.doc.edit("w", |m, _| {
            m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::new(-64.0, 0.0, -8.0), DVec3::new(64.0, 128.0, 8.0)), "brick").unwrap()))
        });
        let cutter = s.doc.edit("c", |m, _| {
            m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::new(-16.0, 32.0, -16.0), DVec3::new(16.0, 96.0, 16.0)), "glass").unwrap()))
        });
        s.doc.select(|_, sel| sel.select_node(cutter));
        s.carve_material = gt_geom::csg::CarveMaterial::Target;
        crate::commands::execute(&mut s, Action::CsgSubtract, &egui::Context::default());
        assert!(s.replaced[&cutter].is_empty());
        let pieces = s.replaced[&wall].clone();
        assert_eq!(pieces.len(), 4);
        assert!(pieces.iter().all(|p| s.doc.map.brush(*p).unwrap().faces.iter().all(|f| f.data.material == "brick")), "carved faces kept the wall's material");
        let json = replaced_json(&s.replaced);
        assert_eq!(json[wall.0.to_string()].as_array().unwrap().len(), 4);

        s.doc.select(|_, sel| {
            sel.clear();
            sel.nodes.extend(pieces.iter().copied());
        });
        crate::commands::execute(&mut s, Action::CsgMerge, &egui::Context::default());
        let merged = s.replaced[&pieces[0]].clone();
        assert_eq!(merged.len(), 1);
        assert!(s.replaced.values().all(|v| *v == merged), "every merged brush maps to the one result");
    }

    #[test]
    fn decals_lie_on_their_surface_at_the_given_size() {
        let mut s = state();
        let ctx = egui::Context::default();
        crate::commands::execute(
            &mut s,
            Action::CreateDecal { material: "grime".into(), at: DVec3::new(0.0, 0.0, 0.0), normal: DVec3::Y, size: Some([64.0, 16.0]) },
            &ctx,
        );
        let id = *s.doc.selection.nodes.iter().next().unwrap();
        let mesh = s.doc.map.mesh(id).unwrap();
        assert!(mesh.decal);
        let (min, max) = mesh.vertices.iter().fold((DVec3::MAX, DVec3::MIN), |(lo, hi), v| (lo.min(*v), hi.max(*v)));
        assert!((max - min - DVec3::new(64.0, 0.0, 16.0)).length() < 1e-9, "{min} {max}");
        assert!((min.y - 0.1).abs() < 1e-9, "lifted just off the floor");

        crate::commands::execute(&mut s, Action::CreateDecal { material: "grime".into(), at: DVec3::ZERO, normal: DVec3::X, size: None }, &ctx);
        let id = *s.doc.selection.nodes.iter().next().unwrap();
        let mesh = s.doc.map.mesh(id).unwrap();
        let (min, max) = mesh.vertices.iter().fold((DVec3::MAX, DVec3::MIN), |(lo, hi), v| (lo.min(*v), hi.max(*v)));
        let metre = s.game.units_per_meter;
        assert!((max - min - DVec3::new(0.0, metre, metre)).length() < 1e-9, "one metre square by default, {min} {max}");
    }

    #[test]
    fn parents_must_be_containers_and_unlocked() {
        let mut s = state();
        let layer = s.doc.map.default_layer();
        let brush = s.doc.edit("b", |m, _| m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::ONE), "a").unwrap())));
        let point = s.doc.edit("e", |m, _| ops::create_point_entity(m, layer, "info_null", DVec3::ZERO));
        assert_eq!(resolve_parent(&s, &Value::Null, Child::Other), Ok(layer));
        assert!(resolve_parent(&s, &json!(brush.0), Child::Geometry).unwrap_err().contains("brush"));
        assert!(resolve_parent(&s, &json!(point.0), Child::Geometry).is_err());
        assert!(check_container(&s, NodeId(9999), Child::Other).is_err());

        s.doc.edit("lock", |m, _| m.get_mut(layer).unwrap().locked = true);
        assert!(resolve_parent(&s, &Value::Null, Child::Other).unwrap_err().contains("locked"));
        assert!(editable_ids(&s, &[brush]).is_err());
    }

    #[test]
    fn paste_round_trips_and_failures_leave_no_undo_step() {
        let mut s = state();
        let layer = s.doc.map.default_layer();
        let brush = s.doc.edit("b", |m, _| m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(16.0)), "a").unwrap())));
        let text = format::nodes_to_string(&s.doc.map, &[brush]);
        let undo_before = s.doc.history.undo_labels().count();
        assert!(paste_text(&mut s, "not a clipboard", layer, None, None).is_err());
        assert_eq!(s.doc.history.undo_labels().count(), undo_before);

        let ids = paste_text(&mut s, &text, layer, Some(DVec3::new(100.0, 8.0, 0.0)), None).unwrap();
        assert_eq!(ids.len(), 1);
        assert_eq!(s.doc.map.bounds_of(ids.iter().copied()).center(), DVec3::new(100.0, 8.0, 0.0));
        let moved = paste_text(&mut s, &text, layer, None, Some(DVec3::new(0.0, 0.0, 64.0))).unwrap();
        assert_eq!(s.doc.map.bounds_of(moved.iter().copied()).min, DVec3::new(0.0, 0.0, 64.0));
    }

    #[test]
    fn failure_statuses_and_dialog_actions() {
        assert!(is_failure_status("Save failed: disk full"));
        assert!(is_failure_status("Select brushes first"));
        assert!(!is_failure_status("Deleted 3 objects"));
        assert!(!is_failure_status("New map"));
        assert!(dialog_action("save_as").is_some());
        assert!(dialog_action("export_obj").is_some_and(|use_instead| use_instead.contains("export_obj")));
        assert!(dialog_action("undo").is_none());
        let names = mesh_op_names();
        for n in ["merge_by_distance", "dissolve_vertices", "select_linked", "mirror_y", "mirror_z"] {
            assert!(names.iter().any(|x| x == n), "{n}");
        }
    }
}
