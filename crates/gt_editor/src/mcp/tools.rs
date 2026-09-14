//! MCP tool implementations. These run on the UI thread with full access to the app.

use std::collections::VecDeque;
use std::io::Cursor;
use std::sync::mpsc::Sender;

use egui::{Event, Key, Modifiers, PointerButton, Pos2, Vec2};
use gt_core::{Aabb, DVec2, DVec3, NodeId};
use gt_doc::{IoConnection, NodeKind, format, issues, ops};
use gt_geom::{Brush, shapes};
use serde_json::{Value, json};

use super::ToolResult;
use crate::app::App;
use crate::camera::ViewKind;
use crate::commands::Action;
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
    WindowScreenshot(Sender<ToolResult>),
}

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

fn err(msg: impl Into<String>) -> Option<ToolResult> {
    Some(ToolResult::Error(msg.into()))
}

fn ok(v: Value) -> Option<ToolResult> {
    Some(ToolResult::Json(v))
}

impl App {
    /// Executes queued MCP calls. Must run at the start of the frame, before UI and input handling.
    pub(crate) fn process_mcp(&mut self, ctx: &egui::Context) {
        let Some(host) = &self.mcp else { return };
        let calls: Vec<_> = host.rx.try_iter().collect();
        for call in calls {
            if let Some(result) = self.call_tool(&call.name, call.args, ctx, &call.reply) {
                let _ = call.reply.send(result);
            }
        }
        for e in ctx.input(|i| i.raw.events.clone()) {
            if let Event::Screenshot { image, .. } = e {
                let rgba = image::RgbaImage::from_raw(image.width() as u32, image.height() as u32, image.as_raw().to_vec());
                for d in self.deferred.drain(..) {
                    let Deferred::WindowScreenshot(reply) = d;
                    let result = match &rgba {
                        Some(img) => ToolResult::Image { png: png(img), note: format!("window {}x{}", img.width(), img.height()) },
                        None => ToolResult::Error("screenshot conversion failed".into()),
                    };
                    let _ = reply.send(result);
                }
            }
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

    pub(crate) fn call_tool(&mut self, name: &str, args: Value, ctx: &egui::Context, reply: &Sender<ToolResult>) -> Option<ToolResult> {
        match name {
            "get_state" => ok(self.state_summary()),
            "list_nodes" => ok(self.list_nodes(&args)),
            "get_node" => {
                let id = NodeId(args["id"].as_u64()?);
                if !self.state.doc.map.contains(id) {
                    return err(format!("no node {id}"));
                }
                let text = format::nodes_to_string(&self.state.doc.map, &[id]);
                let v: Value = serde_json::from_str(&text).ok()?;
                ok(v["nodes"][0].clone())
            }
            "run_action" => self.tool_run_action(&args, ctx),
            "create_brush" => self.tool_create_brush(&args),
            "create_entity" => {
                let classname = args["classname"].as_str().unwrap_or_default().to_string();
                let origin = vec3(&args["origin"]).unwrap_or_default();
                let angles = vec3(&args["angles"]).unwrap_or_default();
                let props: Vec<(String, String)> =
                    args["properties"].as_object().map(|o| o.iter().map(|(k, v)| (k.clone(), value_string(v))).collect()).unwrap_or_default();
                let outputs: Vec<IoConnection> = match args.get("outputs").filter(|o| o.is_array()) {
                    Some(o) => match serde_json::from_value(o.clone()) {
                        Ok(list) => list,
                        Err(e) => return err(format!("outputs must be objects with output, target and input: {e}")),
                    },
                    None => Vec::new(),
                };
                let parent = args["parent"].as_u64().map(NodeId).filter(|p| self.state.doc.map.contains(*p)).unwrap_or_else(|| self.state.insert_parent());
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
            "run_script" => self.tool_run_script(&args, ctx, reply),
            "import_model" => {
                let Some(path) = args["path"].as_str() else { return err("path required") };
                let mode = match args["mode"].as_str().unwrap_or("mesh") {
                    "mesh" => crate::commands::ModelImport::Mesh,
                    "brushes" => crate::commands::ModelImport::Brushes,
                    "prop" => crate::commands::ModelImport::Prop,
                    other => return err(format!("unknown mode {other}")),
                };
                let origin = vec3(&args["origin"]).unwrap_or_default();
                match crate::commands::import_model(&mut self.state, std::path::Path::new(path), mode, origin) {
                    Ok(n) => ok(json!({ "objects": n, "selection": self.state.doc.selection.nodes.iter().map(|i| i.0).collect::<Vec<_>>() })),
                    Err(e) => err(e),
                }
            }
            "select" => {
                let ids: Vec<NodeId> = args["ids"].as_array().map(|a| a.iter().filter_map(|v| v.as_u64()).map(NodeId).collect()).unwrap_or_default();
                let faces: Vec<(NodeId, usize)> = args["faces"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|p| Some((NodeId(p.get(0)?.as_u64()?), p.get(1)?.as_u64()? as usize))).collect())
                    .unwrap_or_default();
                let mode = args["mode"].as_str().unwrap_or("replace").to_string();
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
                ok(
                    json!({ "selected": self.state.doc.selection.nodes.iter().map(|i| i.0).collect::<Vec<_>>(), "faces": self.state.doc.selection.faces.iter().map(|(i, f)| json!([i.0, f])).collect::<Vec<_>>() }),
                )
            }
            "transform" => self.tool_transform(&args),
            "set_face" => self.tool_set_face(&args),
            "map_file" => {
                let op = args["op"].as_str().unwrap_or_default();
                let path = args["path"].as_str().map(std::path::PathBuf::from);
                let result = match op {
                    "new" => {
                        self.state.reset_document(gt_doc::Document::new());
                        Ok(())
                    }
                    "open" => match path {
                        Some(p) => self.state.open_map(&p),
                        None => Err("path required".into()),
                    },
                    "save" => match path.or_else(|| self.state.doc.path.clone()) {
                        Some(p) => self.state.save_map(&p),
                        None => Err("map has no path yet, pass one".into()),
                    },
                    "import_map" => match path {
                        Some(p) => crate::commands::import_quake_map(&mut self.state, &p),
                        None => Err("path required".into()),
                    },
                    "import_vmf" => match path {
                        Some(p) => crate::commands::import_vmf(&mut self.state, &p),
                        None => Err("path required".into()),
                    },
                    "open_tab" => match path {
                        Some(p) => crate::commands::open_map_in_tab(&mut self.state, &p),
                        None => Err("path required".into()),
                    },
                    "export_map" => match path {
                        Some(p) => std::fs::write(&p, gt_formats::quake_map::export(&self.state.doc.map)).map_err(|e| e.to_string()),
                        None => Err("path required".into()),
                    },
                    _ => Err(format!("unknown op {op}")),
                };
                self.project_generation += 1;
                match result {
                    Ok(()) => ok(json!({ "ok": true, "path": self.state.doc.path })),
                    Err(e) => err(e),
                }
            }
            "open_project" => {
                let path = std::path::PathBuf::from(args["path"].as_str().unwrap_or_default());
                match gt_formats::game::find_project_root(&path) {
                    Some(root) => {
                        self.state.load_project(&root);
                        self.project_generation += 1;
                        ok(
                            json!({ "project_root": root, "game": self.state.game.name, "entities": self.state.game.entities.len(), "materials": self.state.materials.entries.len(), "status": self.state.status }),
                        )
                    }
                    None => err("no project.godot found"),
                }
            }
            "set_editor" => {
                if let Some(g) = args["grid"].as_f64() {
                    self.state.grid = g.clamp(0.125, 1024.0);
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
                if let Some(s) = args["shade"].as_str() {
                    self.state.prefs.shade = match crate::state::Shade::from_name(s) {
                        Some(shade) => shade,
                        None => return err(format!("unknown shade {s}")),
                    };
                }
                if let Some(c) = args["mesh_component"].as_str() {
                    self.tools.mesh.component = match c {
                        "vertex" => crate::mesh_tool::Component::Vertex,
                        "edge" => crate::mesh_tool::Component::Edge,
                        _ => crate::mesh_tool::Component::Face,
                    };
                }
                if let Some(items) = args["scatter_items"].as_array().or(args["sprinkle_items"].as_array()) {
                    self.state.prefs.scatter.palette = items.iter().filter_map(|i| i.as_str().map(gt_doc::ScatterItem::new)).collect();
                    self.state.prefs.scatter.preset.clear();
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
                if let Some(mode) = args["sculpt_mode"].as_str() {
                    match serde_json::from_value(json!(mode)) {
                        Ok(m) => self.state.sculpt.mode = m,
                        Err(_) => return err(format!("unknown sculpt mode {mode}")),
                    }
                }
                if let Some(m) = args["material"].as_str() {
                    self.state.current_material = m.to_string();
                }
                if let Some(t) = args["tool"].as_str() {
                    match ToolKind::from_name(t) {
                        Some(k) => self.state.tool = k,
                        None => return err(format!("unknown tool {t}")),
                    }
                }
                ok(self.state_summary()["editor"].clone())
            }
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
            "screenshot" => {
                let target = args["target"].as_str().unwrap_or("window");
                if target == "window" {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
                    self.deferred.push(Deferred::WindowScreenshot(reply.clone()));
                    ctx.request_repaint();
                    return None;
                }
                let Some(kind) = view_kind(target) else { return err("unknown target") };
                let Some(vp) = self.viewports.iter().find(|v| v.kind() == kind) else { return err("view not open") };
                let Some(t) = vp.target() else { return err("view has not rendered yet, is its tab visible?") };
                match self.renderer.read_target(t) {
                    Some(img) => Some(ToolResult::Image { png: png(&img), note: format!("{} view {}x{}", kind.label(), img.width(), img.height()) }),
                    None => err("readback failed"),
                }
            }
            "simulate_input" => match self.build_input_script(&args, reply.clone()) {
                Ok(script) => {
                    if self.input_script.is_some() {
                        return err("another input script is still running");
                    }
                    self.input_script = Some(script);
                    ctx.request_repaint();
                    None
                }
                Err(e) => err(e),
            },
            "validate_map" => {
                let mut list: Vec<Value> = issues::check(&self.state.doc.map).into_iter().map(|i| serde_json::to_value(i).unwrap_or_default()).collect();
                for (id, e) in self.state.doc.map.entities() {
                    if self.state.game.entity(&e.classname).is_none() {
                        list.push(json!({ "node": id.0, "severity": "warning", "code": "unknown_class", "message": format!("No entity definition for '{}'", e.classname) }));
                    }
                }
                ok(json!({ "count": list.len(), "issues": list }))
            }
            "get_game_config" => match args["classname"].as_str() {
                Some(c) => match self.state.game.entity(c) {
                    Some(def) => ok(serde_json::to_value(def).unwrap_or_default()),
                    None => err(format!("no definition for {c}")),
                },
                None => ok(json!({
                    "name": self.state.game.name,
                    "project_root": self.state.game.project_root,
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
                "path": s.doc.path, "modified": s.doc.is_modified(), "revision": s.doc.revision,
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
            },
            "game": { "name": s.game.name, "project_root": s.game.project_root, "entity_definitions": s.game.entities.len(), "materials": s.materials.entries.len() },
            "undo": s.doc.history.undo_labels().take(10).collect::<Vec<_>>(),
            "redo": s.doc.history.redo_labels().take(10).collect::<Vec<_>>(),
            "status": s.status,
        })
    }

    fn list_nodes(&self, args: &Value) -> Value {
        let map = &self.state.doc.map;
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
            total += 1;
            if nodes.len() >= limit {
                continue;
            }
            let bounds = self.state.instance_bounds.get(&id).copied().unwrap_or_else(|| map.bounds(id));
            let mut v = json!({
                "id": id.0, "type": n.kind.type_name(), "name": n.name(), "parent": n.parent.map(|p| p.0),
                "bounds": bounds_json(&bounds), "hidden": n.hidden, "locked": n.locked, "selected": selected, "children": n.children.len(),
            });
            if let Some(e) = n.entity() {
                v["classname"] = json!(e.classname);
            }
            nodes.push(v);
        }
        json!({ "total": total, "nodes": nodes })
    }

    fn tool_run_action(&mut self, args: &Value, ctx: &egui::Context) -> Option<ToolResult> {
        let name = args["action"].as_str().unwrap_or_default();
        let a = &args["args"];
        let action = match name {
            "new_map" => Action::NewMap,
            "save" => Action::Save,
            "undo" => Action::Undo,
            "redo" => Action::Redo,
            "delete" => Action::Delete,
            "duplicate" => Action::Duplicate,
            "select_all" => Action::SelectAll,
            "select_none" => Action::SelectNone,
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
            "csg_subtract" => Action::CsgSubtract,
            "csg_merge" => Action::CsgMerge,
            "csg_intersect" => Action::CsgIntersect,
            "csg_hollow" => Action::CsgHollow,
            "rotate" => Action::Rotate { axis: axis_index(&a["axis"]).unwrap_or(1), degrees: a["degrees"].as_f64().unwrap_or(90.0) },
            "flip" => Action::Flip { axis: axis_index(&a["axis"]).unwrap_or(0) },
            "focus_selection" => Action::FocusSelection,
            "create_brush_entity" => Action::CreateBrushEntity(a["classname"].as_str().unwrap_or("func_detail").to_string()),
            "move_to_world" => Action::MoveToWorld,
            "add_layer" => Action::AddLayer,
            "snap_vertices" => Action::SnapVertices,
            "apply_material" => Action::ApplyMaterial(a["material"].as_str().unwrap_or_default().to_string()),
            "copy" => Action::Copy,
            "cut" => Action::Cut,
            "paste" => Action::Paste(a["text"].as_str().unwrap_or_default().to_string()),
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
                };
                let targets = self.state.doc.selection.brushes(&self.state.doc.map);
                let faces = gt_doc::terrain::displacement_faces(&self.state.doc.map, &targets);
                let selected_terrains = self.state.doc.selection.terrains(&self.state.doc.map);
                let terrains = gt_doc::terrain::terrain_targets(&self.state.doc.map, &selected_terrains);
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
                    .unwrap_or_else(|| self.state.prefs.scatter.palette.clone());
                let saved = self.state.prefs.scatter.clone();
                self.state.prefs.scatter.palette = items;
                let radius = a["radius"].as_f64().unwrap_or(saved.radius);
                let rules =
                    gt_doc::scatter::ScatterRules { density: a["density"].as_f64().unwrap_or(1.0), slope: [0.0, 90.0], falloff: 0.0, ..Default::default() };
                let mut rng = gt_doc::scatter::Rng::new(a["seed"].as_u64().unwrap_or(1));
                let normal = vec3(&a["normal"]).unwrap_or(DVec3::Y);
                let ids = crate::scatter_tool::paint_entities(&mut self.state, center, normal, radius, &rules, &mut rng);
                self.state.prefs.scatter = saved;
                return ok(json!({ "ids": ids.iter().map(|i| i.0).collect::<Vec<_>>() }));
            }
            "mesh_op" => {
                let op_name = a["op"].as_str().unwrap_or_default().replace('_', " ");
                let Some(op) = crate::mesh_tool::MeshOp::ALL.into_iter().find(|o| o.label().eq_ignore_ascii_case(&op_name)) else {
                    return err(format!("unknown mesh op {op_name}"));
                };
                let _ = self.run_app_action(Action::MeshOp(op));
                return ok(json!({ "ok": true, "status": self.state.status }));
            }
            "store_camera" | "recall_camera" => {
                let slot = a["slot"].as_u64().unwrap_or(1).clamp(1, 9) as u8;
                let action = if name == "store_camera" { Action::StoreCamera(slot) } else { Action::RecallCamera(slot) };
                let _ = self.run_app_action(action);
                return ok(json!({ "ok": true, "status": self.state.status }));
            }
            "set_shade" => Action::SetShade(crate::state::Shade::from_name(a["shade"].as_str().unwrap_or("lit")).unwrap_or(crate::state::Shade::Lit)),
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
            "close_tab" => Action::CloseTab,
            "hotspot_texture" => Action::HotspotTexture,
            "terrain_auto_paint" => Action::TerrainAutoPaint,
            "reload_models" => Action::ReloadModels,
            "insert_prefab" => {
                let Some(path) = a["path"].as_str() else { return err("path required") };
                let reference =
                    crate::commands::prefab_reference(std::path::Path::new(path), self.state.doc.path.as_deref(), self.state.game.project_root.as_deref());
                let origin = vec3(&a["origin"]).unwrap_or_default();
                let angles = vec3(&a["angles"]).unwrap_or_default();
                let fixup = a["fixup"].as_str().unwrap_or_default().to_string();
                let parent = self.state.insert_parent();
                let id = self.state.doc.edit("Insert Prefab", |m, s| {
                    let id = m.insert(parent, NodeKind::Instance(gt_doc::map::Instance { path: reference.clone(), origin, angles, fixup }));
                    s.clear();
                    s.select_node(id);
                    id
                });
                return ok(json!({ "id": id.0, "path": reference }));
            }
            "clip_apply" => {
                self.tools.sync(&self.state);
                let before = self.state.doc.map.brush_count();
                self.tools.apply_clip_public(&mut self.state);
                return ok(json!({ "brushes_before": before, "brushes_after": self.state.doc.map.brush_count() }));
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
        ok(
            json!({ "ok": true, "status": self.state.status, "brushes": self.state.doc.map.brush_count(), "selection": self.state.doc.selection.nodes.iter().map(|i| i.0).collect::<Vec<_>>() }),
        )
    }

    fn tool_create_brush(&mut self, args: &Value) -> Option<ToolResult> {
        let (Some(min), Some(max)) = (vec3(&args["min"]), vec3(&args["max"])) else { return err("min and max are required") };
        let bounds = Aabb::new(min, max);
        let material = args["material"].as_str().map(str::to_string).unwrap_or_else(|| self.state.current_material.clone());
        let sides = args["sides"].as_u64().unwrap_or(12) as usize;
        let thickness = args["thickness"].as_f64().unwrap_or(16.0);
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
            other => return err(format!("unknown shape {other}")),
        };
        if brushes.is_empty() {
            return err("shape produced no valid brushes, check the bounds");
        }
        let mut brushes = brushes;
        if let Some(t) = args["hollow"].as_f64().filter(|t| *t > 0.0) {
            brushes = brushes.iter().flat_map(|b| gt_geom::csg::hollow(b, t)).collect();
        }
        for opening in args["openings"].as_array().into_iter().flatten() {
            let (min, max) = match opening {
                Value::Array(pair) if pair.len() == 2 => (vec3(&pair[0]), vec3(&pair[1])),
                Value::Object(_) => (vec3(&opening["min"]), vec3(&opening["max"])),
                _ => (None, None),
            };
            let (Some(min), Some(max)) = (min, max) else { return err("openings are [[min], [max]] pairs") };
            let Ok(cutter) = Brush::from_aabb(&Aabb::new(min, max), &material) else { return err("opening has no volume") };
            brushes = brushes.iter().flat_map(|b| if b.intersects(&cutter) { gt_geom::csg::subtract(b, &cutter) } else { vec![b.clone()] }).collect();
        }
        if let Some(s) = args["uv_scale"].as_f64() {
            for b in &mut brushes {
                for f in &mut b.faces {
                    f.data.uv.scale = DVec2::splat(s);
                }
            }
        }
        let parent = args["parent"].as_u64().map(NodeId).filter(|p| self.state.doc.map.contains(*p)).unwrap_or_else(|| self.state.insert_parent());
        let entity = args["entity"].as_object().map(|e| {
            let mut ent = gt_doc::Entity::new(e.get("classname").and_then(|c| c.as_str()).unwrap_or("func_detail"));
            if let Some(props) = e.get("properties").and_then(|p| p.as_object()) {
                ent.properties.extend(props.iter().map(|(k, v)| (k.clone(), value_string(v))));
            }
            if let Some(outs) = e.get("outputs").and_then(|o| serde_json::from_value::<Vec<IoConnection>>(o.clone()).ok()) {
                ent.outputs = outs;
            }
            ent
        });
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
        ok(json!({ "ids": ids, "entity": entity_id.map(|e| e.0) }))
    }

    fn tool_update_entity(&mut self, args: &Value) -> Option<ToolResult> {
        let id = NodeId(args["id"].as_u64()?);
        if self.state.doc.map.entity(id).is_none() {
            return err(format!("{id} is not an entity"));
        }
        let classname = args["classname"].as_str().map(str::to_string);
        let origin = vec3(&args["origin"]);
        let angles = vec3(&args["angles"]);
        let props = args["properties"].as_object().cloned();
        let outputs: Option<Vec<IoConnection>> = args.get("outputs").filter(|v| v.is_array()).and_then(|v| serde_json::from_value(v.clone()).ok());
        if args.get("outputs").is_some_and(|v| v.is_array()) && outputs.is_none() {
            return err("outputs must be objects with output, target and input");
        }
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
        let e = self.state.doc.map.entity(id)?;
        ok(serde_json::to_value(e).unwrap_or_default())
    }

    fn tool_transform(&mut self, args: &Value) -> Option<ToolResult> {
        if self.state.doc.selection.nodes.is_empty() {
            return err("nothing selected");
        }
        let opts = self.state.opts();
        let grid = self.state.grid;
        let center = ops::selection_center(&self.state.doc.map, &self.state.doc.selection, grid);
        if let Some(t) = vec3(&args["translate"]) {
            self.state.doc.edit("Move", |m, s| ops::translate_selection(m, s, t, opts));
        }
        if args["rotate"].is_object() {
            let axis = axis_index(&args["rotate"]["axis"]).unwrap_or(1);
            let degrees = args["rotate"]["degrees"].as_f64().unwrap_or(0.0);
            let c = vec3(&args["rotate"]["center"]).unwrap_or(center);
            let mut dir = DVec3::ZERO;
            dir[axis] = 1.0;
            let mat = ops::rotation_about(c, dir, degrees);
            self.state.doc.edit("Rotate", |m, s| ops::transform_selection(m, s, &mat, opts));
        }
        if let Some(axis) = axis_index(&args["flip"]) {
            let mat = ops::flip_about(center, axis);
            self.state.doc.edit("Flip", |m, s| ops::transform_selection(m, s, &mat, opts));
        }
        if let (Some(min), Some(max)) = (vec3(&args["scale_to"]["min"]), vec3(&args["scale_to"]["max"])) {
            let old = self.state.doc.map.bounds_of(self.state.doc.selection.nodes.iter().copied());
            let mat = ops::scale_bounds(&old, &Aabb::new(min, max));
            self.state.doc.edit("Scale", |m, s| ops::transform_selection(m, s, &mat, opts));
        }
        ok(json!({ "bounds": bounds_json(&self.state.doc.map.bounds_of(self.state.doc.selection.nodes.iter().copied())) }))
    }

    fn tool_set_face(&mut self, args: &Value) -> Option<ToolResult> {
        let id = NodeId(args["id"].as_u64()?);
        let face = args["face"].as_u64()? as usize;
        let Some(brush) = self.state.doc.map.brush(id) else { return err(format!("{id} is not a brush")) };
        if face >= brush.faces.len() {
            return err(format!("brush has {} faces", brush.faces.len()));
        }
        let material = args["material"].as_str().map(str::to_string);
        let offset = vec2(&args["offset"]);
        let scale = vec2(&args["scale"]);
        let rotate = args["rotate_by"].as_f64();
        let fit = args["fit"].as_bool().unwrap_or(false);
        let mat_name = material.clone().unwrap_or_else(|| brush.faces[face].data.material.clone());
        let size = self
            .state
            .materials
            .size(&mat_name)
            .map(|s| DVec2::new(s[0] as f64, s[1] as f64))
            .unwrap_or(DVec2::splat(self.state.game.textures.fallback_size as f64));
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
        let b = self.state.doc.map.brush(id)?;
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

fn value_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Array(a) => a.iter().map(value_string).collect::<Vec<_>>().join(" "),
        other => other.to_string(),
    }
}
