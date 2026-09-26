//! MCP tools for building content: scatter sets, blends, gameplay setups, code references, hierarchy, terrain shaping,
//! array duplication and scripts of tool calls.

use std::collections::BTreeMap;

use gt_core::{Aabb, DVec2, DVec3, NodeId};
use gt_doc::scatter::Rng;
use gt_doc::{IoConnection, NodeKind, ScatterItem, ops};
use serde_json::{Value, json};

use super::ToolResult;
use super::tools::{Child, check_container, editable_ids, face_list, id_list, optional_id, require_id, resolve_parent, uint};
use crate::app::App;

fn vec3(v: &Value) -> Option<DVec3> {
    let a = v.as_array()?;
    Some(DVec3::new(a.first()?.as_f64()?, a.get(1)?.as_f64()?, a.get(2)?.as_f64()?))
}

/// [x, z] or [x, y, z] as a world point with y from the second form only.
fn point(v: &Value) -> Option<DVec3> {
    let a = v.as_array()?;
    match a.len() {
        2 => Some(DVec3::new(a[0].as_f64()?, 0.0, a[1].as_f64()?)),
        _ => vec3(v),
    }
}

fn arr(v: DVec3) -> Value {
    json!([v.x, v.y, v.z])
}

fn bounds_json(b: &Aabb) -> Value {
    if b.is_empty() { Value::Null } else { json!({ "min": arr(b.min), "max": arr(b.max) }) }
}

fn err(msg: impl Into<String>) -> ToolResult {
    ToolResult::Error(msg.into())
}

fn ok(v: Value) -> ToolResult {
    ToolResult::Json(v)
}

/// Points along a polyline every `spacing` units, the end points included.
pub fn resample(points: &[DVec3], spacing: f64) -> Vec<DVec3> {
    let mut out = Vec::new();
    let Some(first) = points.first() else { return out };
    out.push(*first);
    let spacing = spacing.max(1.0);
    for w in points.windows(2) {
        let len = (w[1] - w[0]).length();
        let n = (len / spacing).ceil().max(1.0) as usize;
        for k in 1..=n {
            out.push(w[0].lerp(w[1], k as f64 / n as f64));
        }
    }

    out
}

fn string_map(v: &Value) -> Vec<(String, String)> {
    v.as_object()
        .map(|o| {
            o.iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| {
                    let text = match v {
                        Value::String(s) => s.clone(),
                        Value::Array(a) => a.iter().map(|x| x.as_str().map(str::to_string).unwrap_or_else(|| x.to_string())).collect::<Vec<_>>().join(" "),
                        other => other.to_string(),
                    };
                    (k.clone(), text)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn outputs(v: &Value) -> Result<Vec<IoConnection>, String> {
    if v.is_null() {
        return Ok(Vec::new());
    }

    serde_json::from_value(v.clone()).map_err(|e| format!("outputs must be objects with output, target and input: {e}"))
}

fn scatter_item(v: &Value) -> Result<ScatterItem, String> {
    match v {
        Value::String(s) => Ok(ScatterItem::new(s.clone())),
        Value::Object(_) => {
            let mut fields = v.clone();
            // A single number scales uniformly, which the [min, max] field cannot parse on its own.
            let uniform = v.get("scale").and_then(|s| s.as_f64());
            if uniform.is_some() {
                fields["scale"] = json!([1.0, 1.0]);
            }

            let mut item: ScatterItem = serde_json::from_value(fields).map_err(|e| e.to_string())?;
            if let Some(s) = uniform {
                item.scale = [s, s];
            }

            Ok(item)
        }
        other => Err(format!("expected a model path or an object, got {other}")),
    }
}

/// Picks the link name to use: the given one, else the only candidate. `builtin` names never count as the only candidate.
fn link_name(given: Option<&str>, options: &[String], builtin: &[&str], what: &str, class: &str) -> Result<String, String> {
    if let Some(g) = given.filter(|g| !g.is_empty()) {
        return Ok(g.to_string());
    }

    let own: Vec<&String> = options.iter().filter(|o| !builtin.contains(&o.as_str())).collect();
    match own.as_slice() {
        [only] => Ok((*only).clone()),
        [] if options.is_empty() => Err(format!("{what} required, {class} defines none")),
        _ => Err(format!("{what} required, {class} offers {}", options.join(", "))),
    }
}

/// Default name for a new layer, matching the editor's Add Layer command.
fn next_layer_name(map: &gt_doc::Map) -> String {
    format!("Layer {}", map.layers.len() + 1)
}

/// The reply to project_content {install} once the install is done.
pub fn content_result(outcome: &crate::content::Outcome) -> ToolResult {
    if outcome.error.is_some() {
        return err(outcome.summary());
    }

    ok(json!({
        "written": outcome.added.written.len(), "kept": outcome.added.kept, "releases": outcome.tags, "summary": outcome.summary(),
    }))
}

impl App {
    pub(crate) fn content_status(&self) -> Value {
        let root = self.state.game.project_root.clone();
        let has = |f: fn(&std::path::Path) -> bool| root.as_deref().is_some_and(f);
        let nature = root.as_deref().map(crate::content::nature_dir);
        let models = nature.as_ref().is_some_and(|dir| gt_formats::nature::names().iter().all(|n| dir.join(format!("{n}.bbmodel")).is_file()));
        json!({
            "project_root": root.as_ref().map(|p| p.to_string_lossy().replace('\\', "/")),
            "blockbench_models": models, "nature_pack": has(crate::content::has_nature_pack), "demo": has(crate::content::has_demo),
            "wizard_open": self.content_wizard.open,
            "running": self.content_wizard.progress().map(|p| json!({ "stage": p.stage, "done": p.done, "total": p.total, "bytes": p.bytes })),
        })
    }

    /// Starts project_content {install}, which is answered when the install is done.
    pub(crate) fn start_content(&mut self, args: &Value, ctx: &egui::Context) -> Result<(), String> {
        let choice = match args["install"].as_str() {
            Some("nature") => crate::content::Choice::Nature,
            Some("demo") => crate::content::Choice::Demo,
            _ => return Err("install must be nature or demo".into()),
        };
        self.content_wizard.start(&self.state, choice, Some(ctx.clone()))
    }

    /// Applies scatter settings present in `args` to the tool settings.
    fn apply_scatter_args(&mut self, a: &Value) -> Result<(), String> {
        if let Some(preset) = a["preset"].as_str() {
            use crate::scatter_tool::PresetProblem;
            match crate::scatter_tool::prepare_preset(&mut self.state, preset) {
                Ok(_) => {}
                Err(PresetProblem::Unknown) => return Err(format!("unknown preset {preset}, use one of {}", gt_doc::scatter::PRESETS.join(", "))),
                Err(PresetProblem::NeedsPack) => {
                    return Err(format!(
                        "the {preset} preset uses the glTF models of the nature pack, which this project lacks, install it with project_content {{install: \"nature\"}}"
                    ));
                }
                Err(PresetProblem::Write(e)) => return Err(format!("could not add the {preset} models to the project: {e}")),
            }

            crate::scatter_tool::apply_preset(&mut self.state, preset);
        }

        let s = &mut self.state.prefs.scatter;
        if let Some(items) = a["items"].as_array() {
            s.palette = items.iter().enumerate().map(|(i, v)| scatter_item(v).map_err(|e| format!("items[{i}]: {e}"))).collect::<Result<_, _>>()?;
            s.preset.clear();
        }

        if let Some(k) = a["kind"].as_str() {
            s.kind = serde_json::from_value(json!(k)).map_err(|_| "kind must be props or foliage".to_string())?;
        }

        if let Some(r) = a["radius"].as_f64() {
            s.radius = r;
        }

        if let Some(d) = a["density"].as_f64() {
            s.rules.density = d;
        }

        if let Some(sl) = a["slope"].as_array() {
            s.rules.slope = [sl.first().and_then(|v| v.as_f64()).unwrap_or(0.0), sl.get(1).and_then(|v| v.as_f64()).unwrap_or(90.0)];
        }

        if a.get("height").is_some() {
            s.rules.height =
                a["height"].as_array().map(|h| [h.first().and_then(|v| v.as_f64()).unwrap_or(-1e9), h.get(1).and_then(|v| v.as_f64()).unwrap_or(1e9)]);
        }

        if let Some(b) = a["only_targets"].as_bool() {
            s.rules.only_targets = b;
        }

        if let Some(f) = a["falloff"].as_f64() {
            s.rules.falloff = f;
        }

        if let Some(b) = a["exposed_only"].as_bool() {
            s.rules.exposed_only = b;
        }

        if let Some(c) = a["clearance"].as_f64() {
            s.rules.clearance = c.max(0.0);
        }

        if let Some(b) = a["avoid_other_sets"].as_bool() {
            s.avoid_other_sets = b;
        }

        if let Some(amount) = a["amount"].as_f64() {
            s.erase_amount = amount;
        }

        if let Some(o) = a["output"].as_str() {
            s.output = serde_json::from_value(json!(o)).map_err(|_| "output must be set or entities".to_string())?;
        }

        Ok(())
    }

    fn scatter_summary(&self, id: NodeId) -> Value {
        let Some(s) = self.state.doc.map.scatter(id) else { return Value::Null };
        let counts = s.counts();
        json!({
            "id": id.0, "name": s.name, "kind": s.kind.label(), "instances": s.instances.len(),
            "targets": s.targets.iter().map(|t| t.0).collect::<Vec<_>>(),
            "items": s.items.iter().zip(counts).map(|(i, c)| json!({ "source": i.source, "weight": i.weight, "spacing": i.spacing, "enabled": i.enabled, "count": c })).collect::<Vec<_>>(),
            "layer": self.state.doc.map.layer_of(id).0,
            "bounds": bounds_json(&s.bounds()),
        })
    }

    pub(crate) fn tool_scatter(&mut self, args: &Value) -> ToolResult {
        let op = args["op"].as_str().unwrap_or_default();
        let (set_id, targets) = match (optional_id(args, "id"), id_list(args, "targets")) {
            (Ok(i), Ok(t)) => (i, t),
            (Err(e), _) | (_, Err(e)) => return err(e),
        };
        let saved = self.state.prefs.scatter.clone();
        if let Err(e) = self.apply_scatter_args(args) {
            self.state.prefs.scatter = saved;
            return err(e);
        }

        if let Some(id) = set_id {
            if self.state.doc.map.scatter(id).is_none() {
                self.state.prefs.scatter = saved;
                return err(format!("{id} is not a scatter set"));
            }

            self.state.active_scatter = Some(id);
        }

        let mut rng = Rng::new(args["seed"].as_u64().unwrap_or_else(crate::commands::time_seed));
        // Settings passed for one call apply to that call only, the palette and preset stay for later calls.
        let persistent = matches!(op, "palette" | "preset");
        let items_given = args.get("items").is_some() || args.get("preset").is_some();
        // Models passed to an op that paints into an existing set become that set's enabled models, the set is the
        // palette. Without an active set they are the template the painted set starts from.
        if items_given
            && matches!(op, "palette" | "preset" | "paint" | "stroke" | "fill")
            && let Some(id) = crate::scatter_tool::active_set(&self.state)
        {
            let palette = self.state.prefs.scatter.palette.clone();
            crate::scatter_tool::set_palette(&mut self.state, id, &palette);
        }

        let result = match op {
            "palette" | "preset" => {
                let set = match crate::scatter_tool::active_set(&self.state) {
                    Some(id) => Some(id),
                    None if items_given => Some(crate::scatter_tool::new_set(&mut self.state, args["name"].as_str())),
                    None => None,
                };
                if let (Some(id), Some(kind)) = (set, args["kind"].as_str().and_then(|k| serde_json::from_value::<gt_doc::ScatterKind>(json!(k)).ok())) {
                    self.state.doc.edit("Scatter Kind", |m, _| {
                        if let Some(s) = m.scatter_mut(id) {
                            s.kind = kind;
                        }
                    });
                }

                let palette = crate::scatter_tool::brush_items(&self.state);
                ok(json!({ "palette": palette, "kind": self.state.prefs.scatter.kind.label(), "set": set.map(|id| self.scatter_summary(id)) }))
            }
            "install_models" => match crate::scatter_tool::install_nature(&mut self.state, args["overwrite"].as_bool().unwrap_or(false)) {
                Ok(written) => {
                    let pack = self.state.game.project_root.as_deref().is_some_and(crate::content::has_nature_pack);
                    let models: Vec<String> = gt_formats::nature::names().iter().map(|n| format!("{n}.bbmodel")).collect();
                    let mut result = json!({ "written": written.len(), "dir": gt_doc::scatter::NATURE_DIR, "models": models, "gltf_pack": pack });
                    if !pack {
                        result["note"] = json!("the glTF trees and bushes are a download, project_content {install: \"nature\"} adds them");
                    }

                    ok(result)
                }
                Err(e) => err(e),
            },
            "new_set" => {
                let name = args["name"].as_str().map(str::to_string);
                let id = crate::scatter_tool::new_set(&mut self.state, name.as_deref());
                let props = args.clone();
                self.state.doc.edit("Scatter Set Settings", |m, _| {
                    if let Some(s) = m.scatter_mut(id) {
                        s.targets = targets;
                        if let Some(c) = props["collision"].as_str().and_then(|c| serde_json::from_value(json!(c)).ok()) {
                            s.collision = c;
                        }

                        if let Some(b) = props["cast_shadows"].as_bool() {
                            s.cast_shadows = b;
                        }

                        if let Some(r) = props["visibility_range"].as_f64() {
                            s.visibility_range = r;
                        }

                        if let Some(c) = props["chunk_size"].as_f64() {
                            s.chunk_size = c;
                        }

                        if props.get("material").is_some() {
                            s.material = props["material"].as_str().filter(|m| !m.is_empty()).map(str::to_string);
                        }
                    }
                });
                ok(self.scatter_summary(id))
            }
            "material" => match self.state.active_scatter {
                None => err("id required, or activate a set first"),
                Some(id) => {
                    let material = args["material"].as_str().filter(|m| !m.is_empty()).map(str::to_string);
                    let item = args["item"].as_u64().map(|i| i as usize);
                    let count = self.state.doc.map.scatter(id).map(|s| s.items.len()).unwrap_or(0);
                    match item {
                        Some(k) if k >= count => err(format!("the set has no palette entry {k}, it has {count}")),
                        _ => {
                            self.state.doc.edit("Scatter Material", |m, _| {
                                if let Some(s) = m.scatter_mut(id) {
                                    match item.and_then(|k| s.items.get_mut(k)) {
                                        // One palette entry, so a single model of the set is retextured.
                                        Some(entry) => entry.material = material.clone(),
                                        None => s.material = material.clone(),
                                    }
                                }
                            });
                            ok(self.scatter_summary(id))
                        }
                    }
                }
            },
            "activate" => match self.state.active_scatter {
                Some(id) => ok(self.scatter_summary(id)),
                None => err("id required"),
            },
            "paint" | "erase" | "stroke" => 'paint: {
                let points: Vec<DVec3> = if op == "stroke" {
                    args["points"].as_array().into_iter().flatten().filter_map(point).collect()
                } else {
                    point(&args["center"]).into_iter().collect()
                };
                if points.is_empty() {
                    break 'paint err("center (paint, erase) or points (stroke) required");
                }

                let erase = op == "erase" || args["erase"].as_bool().unwrap_or(false);
                if !erase && crate::scatter_tool::brush_items(&self.state).is_empty() {
                    break 'paint err("nothing to paint: the active set has no enabled models, pass items or a preset (or start a set with new_set)");
                }

                let normal = vec3(&args["normal"]).unwrap_or(DVec3::Y);
                let spacing = self.state.prefs.scatter.radius * 0.5;
                let mut changed = 0;
                self.state.doc.begin(if erase { "Erase Scatter" } else { "Scatter" });
                for p in resample(&points, spacing) {
                    // Points without a height are dropped onto what the brush would sit on there, scattered props
                    // included, like the tool under the mouse.
                    let (center, under) = if args["center"].as_array().is_some_and(|a| a.len() == 2)
                        || args["points"].as_array().is_some_and(|a| a.first().and_then(|f| f.as_array()).is_some_and(|f| f.len() == 2))
                    {
                        match crate::scatter_tool::cursor_hit(&self.state, &gt_core::Ray::new(DVec3::new(p.x, 1.0e5, p.z), DVec3::NEG_Y)) {
                            Some(h) => (h.point, Some(h.node)),
                            None => (p, None),
                        }
                    } else {
                        (p, None)
                    };
                    changed += if erase {
                        crate::scatter_tool::erase(&mut self.state, center, &mut rng)
                    } else {
                        crate::scatter_tool::paint(&mut self.state, center, normal, under, &mut rng)
                    };
                }

                self.state.doc.commit();
                let set = crate::scatter_tool::active_set(&self.state);
                ok(json!({ if erase { "removed" } else { "placed" }: changed, "set": set.map(|id| self.scatter_summary(id)) }))
            }
            "fill" => {
                if args["targets"].is_array() {
                    let id =
                        crate::scatter_tool::active_set(&self.state).unwrap_or_else(|| crate::scatter_tool::new_set(&mut self.state, args["name"].as_str()));
                    self.state.doc.edit("Scatter Targets", |m, _| {
                        if let Some(s) = m.scatter_mut(id) {
                            s.targets = targets;
                        }
                    });
                }

                match crate::scatter_tool::fill(&mut self.state, &mut rng) {
                    Ok(n) => {
                        let id = crate::scatter_tool::active_set(&self.state);
                        ok(json!({ "placed": n, "set": id.map(|id| self.scatter_summary(id)) }))
                    }
                    Err(e) => err(e),
                }
            }
            "toggle_target" => match uint(&args["target"]).map(NodeId) {
                Some(t) if self.state.doc.map.contains(t) => {
                    let added = crate::scatter_tool::toggle_target(&mut self.state, t);
                    ok(json!({ "added": added }))
                }
                Some(t) => err(format!("no node {t}")),
                None => err("target node id required"),
            },
            "clear" => match crate::scatter_tool::active_set(&self.state) {
                Some(id) => {
                    self.state.doc.edit("Clear Scatter", |m, _| {
                        if let Some(s) = m.scatter_mut(id) {
                            s.instances.clear();
                        }
                    });
                    ok(self.scatter_summary(id))
                }
                None => err("no scatter set, pass id"),
            },
            "get" => {
                let sets: Vec<Value> = match set_id {
                    Some(id) => vec![self.scatter_summary(id)],
                    None => self.state.doc.map.scatters().map(|(id, _)| id).collect::<Vec<_>>().into_iter().map(|id| self.scatter_summary(id)).collect(),
                };
                ok(json!({ "sets": sets, "active": self.state.active_scatter.map(|i| i.0), "palette": crate::scatter_tool::brush_items(&self.state) }))
            }
            "to_entities" => match crate::scatter_tool::active_set(&self.state) {
                Some(id) => ok(json!({ "entities": crate::scatter_tool::bake_to_entities(&mut self.state, id) })),
                None => err("no scatter set, pass id"),
            },
            other => err(format!(
                "unknown scatter op {other}, use palette, install_models, new_set, activate, paint, stroke, erase, fill, toggle_target, material, clear, get or to_entities"
            )),
        };
        if !persistent {
            let palette = self.state.prefs.scatter.palette.clone();
            let kind = self.state.prefs.scatter.kind;
            let preset = self.state.prefs.scatter.preset.clone();
            self.state.prefs.scatter = saved;
            if items_given {
                self.state.prefs.scatter.palette = palette;
                self.state.prefs.scatter.kind = kind;
                self.state.prefs.scatter.preset = preset;
            }
        }

        result
    }

    pub(crate) fn tool_blend(&mut self, args: &Value) -> ToolResult {
        let op = args["op"].as_str().unwrap_or("dab");
        let mut brush = self.state.blend;
        for (key, slot) in [("mode", 0), ("falloff", 1)] {
            if let Some(v) = args[key].as_str() {
                match slot {
                    0 => match serde_json::from_value(json!(v)) {
                        Ok(m) => brush.mode = m,
                        Err(_) => return err("mode must be paint, erase, smooth, sharpen, noise, slope or height"),
                    },
                    _ => match serde_json::from_value(json!(v)) {
                        Ok(f) => brush.falloff = f,
                        Err(_) => return err("falloff must be smooth, linear, constant or spray"),
                    },
                }
            }
        }

        brush.radius = args["radius"].as_f64().unwrap_or(brush.radius);
        brush.strength = args["strength"].as_f64().unwrap_or(brush.strength);
        brush.layer = args["layer"].as_u64().map(|l| l as usize).unwrap_or(brush.layer);
        brush.noise_scale = args["noise_scale"].as_f64().unwrap_or(brush.noise_scale);
        brush.seed = args["seed"].as_u64().map(|s| s as u32).unwrap_or(brush.seed);
        if let Some(s) = args["slope"].as_array() {
            brush.slope = [s.first().and_then(|v| v.as_f64()).unwrap_or(0.0), s.get(1).and_then(|v| v.as_f64()).unwrap_or(90.0)];
        }

        if let Some(h) = args["height"].as_array() {
            brush.height = [h.first().and_then(|v| v.as_f64()).unwrap_or(-1e9), h.get(1).and_then(|v| v.as_f64()).unwrap_or(1e9)];
        }

        let select_ids = match id_list(args, "ids").and_then(|i| editable_ids(&self.state, &i)) {
            Ok(i) => i,
            Err(e) => return err(e),
        };
        if !select_ids.is_empty() {
            self.state.doc.select(|_, s| {
                s.clear();
                s.nodes.extend(select_ids.iter().copied());
            });
        }

        match op {
            "set_material" | "clear_material" => {
                let material = if op == "clear_material" {
                    None
                } else {
                    Some(args["material"].as_str().map(str::to_string).unwrap_or_else(|| self.state.current_material.clone()))
                };
                let faces = match face_list(args, "faces") {
                    Ok(f) => f,
                    Err(e) => return err(e),
                };
                if let Err(e) = editable_ids(&self.state, &faces.iter().map(|(i, _)| *i).collect::<Vec<_>>()) {
                    return err(e);
                }

                if !faces.is_empty() {
                    self.state.doc.select(|_, s| {
                        s.clear();
                        s.faces.extend(faces.iter().copied());
                    });
                }

                let n = crate::blend_tool::set_blend_material(&mut self.state, material.as_deref());
                ok(json!({ "faces": n }))
            }
            "dab" | "stroke" => {
                let points: Vec<DVec3> = if op == "stroke" {
                    args["points"].as_array().into_iter().flatten().filter_map(point).collect()
                } else {
                    point(&args["center"]).into_iter().collect()
                };
                if points.is_empty() {
                    return err("center (dab) or points (stroke) required");
                }

                let targets = crate::blend_tool::targets(&self.state);
                if targets.is_empty() {
                    return err("nothing to blend: select terrains, displacements, or faces with a blend material");
                }

                let spacing = (brush.radius * 0.25).max(1.0);
                let mut changed = false;
                self.state.doc.begin("Blend");
                for (k, p) in resample(&points, spacing).into_iter().enumerate() {
                    let mut b = brush;
                    b.seed = brush.seed.wrapping_add(k as u32);
                    changed |= crate::blend_tool::dab(&mut self.state, p, &b);
                }

                self.state.doc.commit();
                ok(
                    json!({ "changed": changed, "terrains": targets.terrains.len(), "displacements": targets.displacements.len(), "faces": targets.faces.len() }),
                )
            }
            "weights" => {
                let Some(p) = point(&args["center"]) else { return err("center required") };
                let given = match optional_id(args, "id") {
                    Ok(i) => i,
                    Err(e) => return err(e),
                };
                let map = &self.state.doc.map;
                let terrain = given.or_else(|| map.terrains().next().map(|(id, _)| id));
                let Some(t) = terrain.and_then(|id| map.terrain(id)) else { return err("no terrain") };
                let i = ((p.x - t.origin.x) / t.cell_size).round().clamp(0.0, (t.resolution[0] - 1) as f64) as u32;
                let j = ((p.z - t.origin.z) / t.cell_size).round().clamp(0.0, (t.resolution[1] - 1) as f64) as u32;
                ok(json!({ "weights": t.weights(i, j), "height": t.vertex(i, j).y, "layers": t.layers.iter().map(|l| l.material.clone()).collect::<Vec<_>>() }))
            }
            other => err(format!("unknown blend op {other}, use dab, stroke, set_material, clear_material or weights")),
        }
    }

    /// Extra keys for an entity a gameplay wizard just made. Outputs other entities aim at it follow a new targetname.
    fn apply_wizard_properties(&mut self, id: NodeId, props: &Value, label: &str) {
        let Some(props) = props.as_object() else { return };
        let props = string_map(&Value::Object(props.clone()));
        let old_name = self.state.doc.map.entity(id).and_then(|e| e.targetname()).map(str::to_string);
        self.state.doc.edit(label, |m, _| {
            if let Some(e) = m.entity_mut(id) {
                e.properties.extend(props);
            }

            let new_name = m.entity(id).and_then(|e| e.targetname()).map(str::to_string);
            if let (Some(old), Some(new)) = (old_name, new_name)
                && old != new
            {
                let ids: Vec<NodeId> = m.entities().map(|(eid, _)| eid).collect();
                for eid in ids {
                    if let Some(e) = m.entity_mut(eid) {
                        e.outputs.iter_mut().filter(|o| o.target == old).for_each(|o| o.target = new.clone());
                    }
                }
            }
        });
    }

    /// Adds `outputs` to an entity a wizard made, after the connections the wizard wired itself.
    fn apply_wizard_outputs(&mut self, id: NodeId, extra: &[IoConnection]) {
        if extra.is_empty() {
            return;
        }

        self.state.doc.edit("Entity Outputs", |m, _| {
            if let Some(e) = m.entity_mut(id) {
                e.outputs.extend(extra.iter().cloned());
            }
        });
    }

    pub(crate) fn tool_gameplay(&mut self, args: &Value) -> ToolResult {
        use crate::entity_wizards as wiz;
        let op = args["op"].as_str().unwrap_or_default();
        let wizard_outputs = if matches!(op, "make_door" | "make_platform" | "make_button") {
            match outputs(&args["outputs"]) {
                Ok(o) => o,
                Err(e) => return err(e),
            }
        } else {
            Vec::new()
        };
        let given = match id_list(args, "ids").and_then(|i| editable_ids(&self.state, &i)) {
            Ok(i) => i,
            Err(e) => return err(e),
        };
        let target_ids: Vec<NodeId> = if given.is_empty() { self.state.doc.selection.nodes.iter().copied().collect() } else { given };
        let creates = matches!(op, "make_door" | "make_platform" | "make_button" | "brush_entity" | "volume" | "place");
        if creates && let Err(e) = resolve_parent(&self.state, &Value::Null, Child::Other) {
            return err(e);
        }

        let entity_json = |app: &App, id: NodeId| {
            let e = app.state.doc.map.entity(id);
            json!({ "id": id.0, "classname": e.map(|e| e.classname.clone()), "properties": e.map(|e| e.properties.clone()), "outputs": e.map(|e| e.outputs.clone()), "bounds": bounds_json(&app.state.doc.map.bounds(id)) })
        };
        let result: Result<Value, String> = match op {
            "make_door" => {
                let kind = match args["kind"].as_str().unwrap_or("hinged") {
                    "hinged" | "rotating" => wiz::DoorKind::Hinged {
                        side: serde_json::from_value(args["side"].clone()).unwrap_or_default(),
                        angle: args["angle"].as_f64().unwrap_or(95.0),
                    },
                    "sliding" => wiz::DoorKind::Sliding {
                        direction: serde_json::from_value(args["direction"].clone()).unwrap_or_default(),
                        lip: args["lip"].as_f64().unwrap_or(4.0),
                    },
                    other => return err(format!("kind must be hinged or sliding, got {other}")),
                };
                wiz::make_door(&mut self.state, &target_ids, &kind, args["trigger"].as_bool().unwrap_or(false)).map(|id| {
                    self.apply_wizard_properties(id, &args["properties"], "Door Properties");
                    self.apply_wizard_outputs(id, &wizard_outputs);
                    entity_json(self, id)
                })
            }
            "make_platform" => {
                let travel = vec3(&args["travel"]).unwrap_or(DVec3::new(0.0, 128.0, 0.0));
                wiz::make_platform(&mut self.state, &target_ids, travel, args["mode"].as_u64().unwrap_or(0) as u8).map(|id| {
                    self.apply_wizard_properties(id, &args["properties"], "Platform Properties");
                    self.apply_wizard_outputs(id, &wizard_outputs);
                    entity_json(self, id)
                })
            }
            "make_button" => {
                wiz::make_button(&mut self.state, &target_ids, args["target"].as_str().unwrap_or_default(), args["input"].as_str().unwrap_or("toggle")).map(
                    |id| {
                        self.apply_wizard_properties(id, &args["properties"], "Button Properties");
                        self.apply_wizard_outputs(id, &wizard_outputs);
                        entity_json(self, id)
                    },
                )
            }
            "brush_entity" => {
                let classname = args["classname"].as_str().unwrap_or("func_detail").to_string();
                let parent = self.state.insert_parent();
                let props = string_map(&args["properties"]);
                match outputs(&args["outputs"]) {
                    Ok(outs) => self
                        .state
                        .doc
                        .try_edit("Create Brush Entity", |m, s| {
                            s.clear();
                            s.nodes.extend(target_ids.iter().copied());
                            let id = ops::create_brush_entity(m, s, &classname, parent).ok_or_else(|| "select or pass the brushes (ids)".to_string())?;
                            if let Some(e) = m.entity_mut(id) {
                                e.properties.extend(props);
                                e.outputs = outs;
                            }

                            Ok(id)
                        })
                        .map(|id| entity_json(self, id)),
                    Err(e) => Err(e),
                }
            }
            "volume" => {
                let (Some(min), Some(max)) = (vec3(&args["min"]), vec3(&args["max"])) else { return err("min and max required") };
                let classname = args["classname"].as_str().unwrap_or("trigger_once");
                let mut props = crate::volume_tool::default_props(classname);
                props.retain(|(k, _)| args["properties"].get(k).is_none());
                props.extend(string_map(&args["properties"]));
                match outputs(&args["outputs"]) {
                    Ok(outs) => wiz::make_volume(&mut self.state, classname, &Aabb::new(min, max), &props, outs).map(|id| entity_json(self, id)),
                    Err(e) => Err(e),
                }
            }
            "link" => {
                let (from, to) = match (require_id(args, "from"), require_id(args, "to")) {
                    (Ok(f), Ok(t)) => (f, t),
                    (Err(e), _) | (_, Err(e)) => return err(e),
                };
                let class = |id: NodeId| self.state.doc.map.entity(id).map(|e| e.classname.clone()).unwrap_or_default();
                let (outs, ins) = wiz::link_options(&self.state, from, to);
                let names = link_name(args["output"].as_str(), &outs, &[], "output", &class(from))
                    .and_then(|o| link_name(args["input"].as_str(), &ins, &["kill", "show", "hide", "enable", "disable"], "input", &class(to)).map(|i| (o, i)));
                match names {
                    Ok((output, input)) => wiz::link(
                        &mut self.state,
                        from,
                        to,
                        &output,
                        &input,
                        args["parameter"].as_str().unwrap_or_default(),
                        args["delay"].as_f64().unwrap_or(0.0),
                    )
                    .map(|c| serde_json::to_value(c).unwrap_or_default()),
                    Err(e) => Err(e),
                }
            }
            "place" => {
                let classname = args["classname"].as_str().unwrap_or("info_null");
                let Some(mut origin) = point(&args["origin"]) else { return err("origin required") };
                let outs = match outputs(&args["outputs"]) {
                    Ok(o) => o,
                    Err(e) => return err(e),
                };
                if args["origin"].as_array().is_some_and(|a| a.len() == 2) || args["snap_to_ground"].as_bool().unwrap_or(false) {
                    let caster = crate::picking::SurfaceCaster::new(&self.state, &Aabb::from_center_size(origin, DVec3::new(2.0, 1.0e6, 2.0)));
                    if let Some(h) = caster.cast(DVec3::new(origin.x, origin.y.max(0.0) + 1.0e5, origin.z), DVec3::NEG_Y) {
                        origin = h.point;
                    }
                }

                let props = string_map(&args["properties"]);
                let angles = vec3(&args["angles"]);
                self.state.doc.begin("Place Entity");
                let id = wiz::place_entity(&mut self.state, classname, origin, &props);
                self.state.doc.edit("Entity Setup", |m, _| {
                    if let Some(e) = m.entity_mut(id) {
                        e.outputs = outs;
                        if let Some(a) = angles {
                            e.angles = a;
                        }
                    }
                });
                self.state.doc.commit();
                Ok(entity_json(self, id))
            }
            "gizmos" => {
                if !target_ids.is_empty() {
                    self.state.doc.select(|_, s| {
                        s.clear();
                        s.nodes.extend(target_ids.iter().copied());
                    });
                }

                let handles: Vec<Value> = crate::gizmos::handles(&self.state)
                    .into_iter()
                    .map(|(h, p, label)| json!({ "entity": h.entity.0, "gizmo": h.gizmo, "part": h.part, "position": arr(p), "label": label }))
                    .collect();
                Ok(json!({ "handles": handles }))
            }
            other => Err(format!("unknown gameplay op {other}, use make_door, make_platform, make_button, brush_entity, volume, link, place or gizmos")),
        };
        match result {
            Ok(v) => ok(v),
            Err(e) => err(e),
        }
    }

    pub(crate) fn tool_code_reference(&mut self, args: &Value) -> ToolResult {
        use crate::code_refs::{self, CodeKind};
        let Some(classname) = args["classname"].as_str() else {
            let list: Vec<Value> = self
                .state
                .game
                .entities
                .iter()
                .map(|d| json!({ "classname": d.classname, "type": d.kind, "description": d.description, "script": d.script }))
                .collect();
            return ok(json!({ "entities": list, "tools": crate::panels::TOOL_HELP.iter().map(|(t, h)| json!({ "tool": t, "help": h })).collect::<Vec<_>>() }));
        };
        let Some(def) = self.state.game.entity(classname).cloned() else { return err(format!("no definition for {classname}")) };
        let kinds: Vec<CodeKind> = match args["kind"].as_str() {
            None | Some("all") => CodeKind::ALL.to_vec(),
            Some(k) => match CodeKind::from_name(k) {
                Some(kind) => vec![kind],
                None => return err("kind must be gdscript, csharp, gdscript_usage, csharp_usage, fgd or all"),
            },
        };
        let code: serde_json::Map<String, Value> = kinds.iter().map(|k| (k.label().to_string(), json!(code_refs::generate(&def, *k)))).collect();
        ok(
            json!({ "classname": classname, "definition": def, "gizmos": def.gizmos(self.state.game.units_per_meter), "code": code, "csharp_helper": code_refs::CSHARP_HELPER }),
        )
    }

    pub(crate) fn tool_hierarchy(&mut self, args: &Value) -> ToolResult {
        let op = args["op"].as_str().unwrap_or_default();
        let name = args["name"].as_str().filter(|n| !n.trim().is_empty()).map(str::to_string);
        match op {
            "add_layer" => {
                let name = name.unwrap_or_else(|| next_layer_name(&self.state.doc.map));
                let id = self.state.doc.edit("Add Layer", |m, _| m.add_layer(&name));
                self.state.current_layer = id;
                self.state.open_groups.clear();
                ok(json!({ "id": id.0, "name": name }))
            }
            "add_group" => {
                let parent = match resolve_parent(&self.state, &args["parent"], Child::Other) {
                    Ok(p) => p,
                    Err(e) => return err(e),
                };
                let name = name.unwrap_or_else(|| "Group".to_string());
                let id = self.state.doc.edit("Add Group", |m, _| m.insert(parent, NodeKind::Group(gt_doc::Group::new(name.clone()))));
                if args["open"].as_bool().unwrap_or(true) {
                    self.state.open_groups.push(id);
                }

                ok(json!({ "id": id.0, "parent": parent.0 }))
            }
            "open_group" => {
                let id = match require_id(args, "id") {
                    Ok(id) => id,
                    Err(e) => return err(e),
                };
                match self.state.doc.map.get(id).map(|n| &n.kind) {
                    Some(NodeKind::Group(_)) => {
                        self.state.open_groups.push(id);
                        ok(json!({ "insert_parent": self.state.insert_parent().0 }))
                    }
                    Some(kind) => err(format!("{id} is a {}, open_group needs a group", kind.type_name())),
                    None => err(format!("no node {id}")),
                }
            }
            "close_group" => {
                if args["all"].as_bool().unwrap_or(false) {
                    self.state.open_groups.clear();
                } else {
                    self.state.open_groups.pop();
                }

                ok(json!({ "insert_parent": self.state.insert_parent().0 }))
            }
            "set_current_layer" => match require_id(args, "id") {
                Ok(l) if self.state.doc.map.layers.contains(&l) => {
                    self.state.current_layer = l;
                    self.state.open_groups.clear();
                    ok(json!({ "insert_parent": l.0 }))
                }
                Ok(l) => err(format!("{l} is not a layer")),
                Err(e) => err(e),
            },
            "reparent" => {
                let (parent, list) = match (require_id(args, "parent"), id_list(args, "ids")) {
                    (Ok(p), Ok(l)) => (p, l),
                    (Err(e), _) | (_, Err(e)) => return err(e),
                };
                if let Err(e) = editable_ids(&self.state, &list) {
                    return err(e);
                }

                for id in &list {
                    let geometry = self.state.doc.map.get(*id).is_some_and(|n| n.kind.is_geometry());
                    if let Err(e) = check_container(&self.state, parent, if geometry { Child::Geometry } else { Child::Other }) {
                        return err(format!("cannot move {id}: {e}"));
                    }

                    if *id == parent || self.state.doc.map.is_ancestor(*id, parent) {
                        return err(format!("cannot move {id} into itself or one of its children"));
                    }

                    if self.state.doc.map.layers.contains(id) {
                        return err(format!("{id} is a layer, layers have no parent"));
                    }
                }

                if self.state.doc.map.is_locked(parent) {
                    return err(format!("{parent} is locked"));
                }

                self.state.doc.edit("Reparent", |m, _| list.iter().for_each(|id| m.reparent(*id, parent)));
                ok(json!({ "moved": list.len() }))
            }
            "rename" => {
                let id = match require_id(args, "id") {
                    Ok(id) => id,
                    Err(e) => return err(e),
                };
                // Unlike the other ops, an empty name counts: it takes a given name off again.
                let Some(name) = args["name"].as_str() else { return err("name required") };
                let Some(node) = self.state.doc.map.get(id) else { return err(format!("no node {id}")) };
                if name.trim().is_empty() && node.kind.has_own_name() {
                    return err("layers, groups and scatter sets need a name");
                }

                let _ = self.state.doc.try_edit("Rename", |m, _| if m.rename(id, name) { Ok(()) } else { Err(()) });
                let name = self.state.doc.map.get(id).map(|n| n.name()).unwrap_or_default();
                ok(json!({ "id": id.0, "name": name }))
            }
            "set_flags" => {
                let list = match id_list(args, "ids") {
                    Ok(l) => l,
                    Err(e) => return err(e),
                };
                let missing: Vec<u64> = list.iter().filter(|i| !self.state.doc.map.contains(**i)).map(|i| i.0).collect();
                if !missing.is_empty() {
                    return err(format!("no nodes with ids {missing:?}"));
                }

                let (hidden, locked, omit) = (args["hidden"].as_bool(), args["locked"].as_bool(), args["omit_from_export"].as_bool());
                self.state.doc.edit("Set Flags", |m, _| {
                    for id in &list {
                        if let Some(n) = m.get_mut(*id) {
                            if let Some(h) = hidden {
                                n.hidden = h;
                            }

                            if let Some(l) = locked {
                                n.locked = l;
                            }

                            if let (Some(o), NodeKind::Layer(layer)) = (omit, &mut n.kind) {
                                layer.omit_from_export = o;
                            }
                        }
                    }
                });
                ok(json!({ "changed": list.len() }))
            }
            other => err(format!(
                "unknown hierarchy op {other}, use add_layer, add_group, open_group, close_group, set_current_layer, reparent, rename or set_flags"
            )),
        }
    }

    pub(crate) fn tool_set_map_properties(&mut self, args: &Value) -> ToolResult {
        let Some(props) = args["properties"].as_object().cloned() else { return err("properties object required") };
        self.state.doc.edit("Set Map Properties", |m, _| {
            m.properties.insert("classname".into(), "worldspawn".into());
            for (k, v) in &props {
                if v.is_null() {
                    m.properties.remove(k);
                }
            }

            m.properties.extend(string_map(&Value::Object(props.clone())));
        });
        ok(json!({ "properties": self.state.doc.map.properties }))
    }

    pub(crate) fn tool_terrain_edit(&mut self, args: &Value) -> ToolResult {
        use gt_doc::terrain::{SculptBrush, SculptMode};
        let given = match optional_id(args, "id") {
            Ok(i) => i,
            Err(e) => return err(e),
        };
        let map = &self.state.doc.map;
        let id = given.or_else(|| self.state.doc.selection.terrains(map).first().copied()).or_else(|| map.terrains().next().map(|(id, _)| id));
        let Some(id) = id.filter(|id| map.terrain(*id).is_some()) else { return err("no terrain, pass id or create one") };
        let probe = match &args["probe"] {
            Value::Null => None,
            v => match point(v) {
                Some(p) => Some(p),
                None => return err("probe must be [x, z]"),
            },
        };
        let probe_height = |app: &App| probe.and_then(|p| app.state.doc.map.terrain(id).map(|t| t.height_at(p.x, p.z)));
        let op = args["op"].as_str().unwrap_or_default().to_string();
        if op.is_empty() || op == "probe" {
            return match probe {
                Some(_) => ok(json!({ "id": id.0, "probe_height": probe_height(self) })),
                None => err("op required, or probe [x, z] to read the height"),
            };
        }

        if !self.state.doc.map.is_editable(id) {
            return err(format!("terrain {id} is hidden or locked"));
        }

        let a = args.clone();
        let result: Result<Value, String> = self.state.doc.try_edit(&format!("Terrain {op}"), |m, _| {
            let Some(t) = m.terrain_mut(id) else { return Err("terrain vanished".to_string()) };
            let radius = a["radius"].as_f64().unwrap_or(256.0);
            let strength = a["strength"].as_f64().unwrap_or(16.0);
            let changed = match op.as_str() {
                "sculpt" | "sculpt_path" => {
                    let mode: SculptMode = serde_json::from_value(a["mode"].clone())
                        .map_err(|_| "mode must be raise, lower, smooth, flatten, noise, terrace, paint_layer, hole or unhole".to_string())?;
                    let points: Vec<DVec3> = if op == "sculpt" {
                        point(&a["center"]).into_iter().collect()
                    } else {
                        a["points"].as_array().into_iter().flatten().filter_map(point).collect()
                    };
                    if points.is_empty() {
                        return Err(if op == "sculpt" { "center [x, z] required" } else { "points [[x, z], ...] required" }.to_string());
                    }

                    let mut brush = SculptBrush {
                        mode,
                        radius,
                        strength,
                        flatten_height: a["height"].as_f64().unwrap_or(0.0),
                        terrace_step: a["step"].as_f64().unwrap_or(32.0),
                        layer: a["layer"].as_u64().unwrap_or(1) as u8,
                        seed: 0,
                    };
                    let seed = a["seed"].as_u64().unwrap_or(7) as u32;
                    let mut any = false;
                    for (k, p) in resample(&points, a["spacing"].as_f64().unwrap_or(radius * 0.5)).into_iter().enumerate() {
                        brush.seed = seed.wrapping_add(k as u32);
                        any |= gt_doc::terrain::sculpt_terrains_single(t, p, &brush);
                    }

                    any
                }
                "paint_path" => {
                    let layer = a["layer"].as_u64().unwrap_or(1) as usize;
                    let amount = a["strength"].as_f64().unwrap_or(1.0);
                    let points: Vec<DVec3> = a["points"].as_array().into_iter().flatten().filter_map(point).collect();
                    if points.is_empty() {
                        return Err("points [[x, z], ...] required".to_string());
                    }

                    let mut any = false;
                    for p in resample(&points, radius * 0.4) {
                        any |= t.paint_layer(p, radius, layer, amount);
                    }

                    any
                }
                "flatten_rect" => {
                    let (Some(min), Some(max)) = (point(&a["min"]), point(&a["max"])) else { return Err("min and max [x, z] required".to_string()) };
                    t.flatten_rect(
                        DVec2::new(min.x, min.z),
                        DVec2::new(max.x, max.z),
                        a["height"].as_f64().unwrap_or(0.0),
                        a["margin"].as_f64().unwrap_or(64.0),
                    )
                }
                "ramp" => {
                    let (Some(from), Some(to)) = (vec3(&a["from"]), vec3(&a["to"])) else { return Err("from and to [x, y, z] required".to_string()) };
                    t.ramp(from, to, a["width"].as_f64().unwrap_or(96.0), a["margin"].as_f64().unwrap_or(64.0))
                }
                "erode" => {
                    t.erode(a["iterations"].as_u64().unwrap_or(4) as u32, a["talus"].as_f64().unwrap_or(t.cell_size));
                    true
                }
                "auto_paint" => {
                    let custom = ["rock_slope", "top_height", "low_height"].iter().any(|k| !a[*k].is_null());
                    if custom {
                        let rock = a["rock_slope"].as_f64().unwrap_or(0.35);
                        let top = a["top_height"].as_f64().unwrap_or(1.0e6) - t.origin.y;
                        let low = a["low_height"].as_f64().unwrap_or(-1.0e6) - t.origin.y;
                        t.auto_paint(rock, top, low);
                    } else {
                        t.auto_paint_from(a["sea_level"].as_f64());
                    }

                    true
                }
                "set_layers" => {
                    let layers: Vec<gt_geom::TerrainLayer> = a["layers"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|l| {
                            let mut layer = gt_geom::TerrainLayer::new(l.get(0)?.as_str()?.to_string(), l.get(1).and_then(|v| v.as_f64()).unwrap_or(256.0));
                            layer.detile = l.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0).clamp(0.0, 1.0);
                            layer.detile_sharpen = l.get(3).and_then(|v| v.as_f64()).unwrap_or(0.5).clamp(0.0, 1.0);
                            Some(layer)
                        })
                        .take(gt_geom::heightfield::MAX_LAYERS)
                        .collect();
                    if layers.is_empty() {
                        return Err("layers [[material, tile, detile?, sharpen?], ...] required".to_string());
                    }

                    t.layers = layers;
                    true
                }
                "holes" => {
                    let hole = a["hole"].as_bool().unwrap_or(true);
                    match (point(&a["min"]), point(&a["max"])) {
                        (Some(min), Some(max)) => t.set_holes_rect(DVec2::new(min.x, min.z), DVec2::new(max.x, max.z), hole),
                        _ => {
                            let Some(c) = point(&a["center"]) else { return Err("center [x, z], or min and max [x, z], required".to_string()) };
                            t.set_holes(c, radius, hole)
                        }
                    }
                }
                "clear_layer" => t.clear_layer(a["layer"].as_u64().unwrap_or(1) as usize),
                other => {
                    return Err(format!(
                        "unknown terrain op {other}, use sculpt, sculpt_path, paint_path, flatten_rect, ramp, erode, auto_paint, set_layers, holes or clear_layer, or only probe"
                    ));
                }
            };
            Ok(json!({ "changed": changed, "bounds": bounds_json(&t.bounds()) }))
        });
        match result {
            Ok(mut v) => {
                v["id"] = json!(id.0);
                if probe.is_some() {
                    v["probe_height"] = json!(probe_height(self));
                }

                ok(v)
            }
            Err(e) => err(e),
        }
    }

    pub(crate) fn tool_duplicate(&mut self, args: &Value) -> ToolResult {
        let given = match id_list(args, "ids") {
            Ok(i) => i,
            Err(e) => return err(e),
        };
        let source = if given.is_empty() { self.state.doc.selection.nodes.iter().copied().collect::<Vec<_>>() } else { given };
        if source.is_empty() {
            return err("nothing to duplicate, pass ids or select objects");
        }

        if let Err(e) = editable_ids(&self.state, &source) {
            return err(e);
        }

        let offset = vec3(&args["offset"]).unwrap_or(DVec3::new(self.state.grid, 0.0, 0.0));
        let count = args["count"].as_u64().unwrap_or(1).clamp(1, 512) as usize;
        let linked = args["linked"].as_bool().unwrap_or(false);
        let rotate = args["rotate_y"].as_f64().unwrap_or(0.0);
        let pivot = vec3(&args["pivot"]);
        let opts = ops::EditOptions { uv_lock: true, grid: 0.0 };
        let parent = match resolve_parent(&self.state, &Value::Null, Child::Other) {
            Ok(p) => p,
            Err(e) if linked => return err(e),
            Err(_) => self.state.insert_parent(),
        };
        let mut all = Vec::new();
        self.state.doc.begin(if linked { "Duplicate Linked" } else { "Array Duplicate" });
        self.state.doc.select(|_, s| {
            s.clear();
            s.nodes.extend(source.iter().copied());
        });
        if linked && !source.iter().any(|id| matches!(self.state.doc.map.get(*id).map(|n| &n.kind), Some(NodeKind::Group(_)))) {
            self.state.doc.edit("Group", |m, s| ops::group_selection(m, s, "Linked", parent));
        }

        for _ in 0..count {
            let copies = if linked {
                self.state.doc.edit("Duplicate Linked", |m, s| ops::duplicate_linked(m, s, offset, opts))
            } else {
                self.state.doc.edit("Duplicate", |m, s| ops::duplicate_selection(m, s, offset, opts))
            };
            if rotate != 0.0 {
                let center = pivot.unwrap_or_else(|| self.state.doc.map.bounds_of(copies.iter().copied()).center());
                // Each copy is duplicated from the previous one, so one step of rotation accumulates.
                let turn = ops::rotation_about(center, DVec3::Y, rotate);
                let only = copies.clone();
                self.state.doc.edit("Rotate Copy", |m, s| {
                    let mut sel = gt_doc::Selection::default();
                    sel.nodes.extend(only.iter().copied());
                    ops::transform_selection(m, &sel, &turn, opts);
                    s.nodes = only.iter().copied().collect();
                });
            }

            all.push(copies.iter().map(|i| i.0).collect::<Vec<_>>());
        }

        self.state.doc.commit();
        ok(json!({ "copies": all, "selection": self.state.doc.selection.nodes.iter().map(|i| i.0).collect::<Vec<_>>() }))
    }

    pub(crate) fn tool_run_script(&mut self, args: &Value, ctx: &egui::Context) -> ToolResult {
        let project = self.state.game.project_root.as_ref().map(super::tools::path_text);
        let cwd = || std::env::current_dir().map(super::tools::path_text).unwrap_or_default();
        let (doc, dir) = match args["path"].as_str() {
            Some(path) => {
                let text = match std::fs::read_to_string(path) {
                    Ok(t) => t,
                    Err(e) => return err(format!("cannot read {path}: {e}")),
                };
                match serde_json::from_str::<Value>(&text) {
                    Ok(v) => {
                        let parent = std::path::Path::new(path).parent().map(super::tools::path_text).unwrap_or_default();
                        (v, if parent.is_empty() { cwd() } else { parent })
                    }
                    Err(e) => return err(format!("{path}: {e}")),
                }
            }

            // Inline steps have no file, so relative paths resolve against the project, else the editor's working directory.
            None => (args.get("steps").map(|s| json!({ "steps": s })).unwrap_or(Value::Null), project.clone().unwrap_or_else(cwd)),
        };
        let steps = match super::script::parse(&doc) {
            Ok(s) => s,
            Err(e) => return err(e),
        };
        let notes = super::script::count_notes(&doc);
        let mut vars: BTreeMap<String, Value> = BTreeMap::new();
        vars.insert("script_dir".into(), json!(dir));
        if let Some(p) = project {
            vars.insert("project".into(), json!(p));
        }

        if let Some(extra) = args["vars"].as_object() {
            vars.extend(extra.iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        if let Some(defaults) = doc["vars"].as_object() {
            for (k, v) in defaults {
                if vars.contains_key(k) {
                    continue;
                }

                match super::script::resolve(v, &vars) {
                    Ok(v) => {
                        vars.insert(k.clone(), v);
                    }
                    Err(e) => return err(format!("script var {k}: {e}")),
                }
            }
        }

        let label = args["label"]
            .as_str()
            .or(doc["label"].as_str())
            .map(str::to_string)
            .or_else(|| args["path"].as_str().and_then(|p| std::path::Path::new(p).file_name()).map(|n| n.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "script".into());
        let mut mark = self.state.doc.mark();
        let continue_on_error = args["continue_on_error"].as_bool().unwrap_or(false);
        // Only results of steps follow replaced brushes, variables passed in stay as written.
        let mut saved_results: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        let mut errors = Vec::new();
        let mut ran = 0;
        for (i, step) in steps.iter().enumerate() {
            if matches!(step.tool.as_str(), "run_script" | "screenshot" | "simulate_input") {
                errors.push(json!({ "step": i, "tool": step.tool, "error": "not allowed inside scripts" }));
                if !continue_on_error {
                    break;
                }

                continue;
            }

            let resolved = match super::script::resolve(&step.args, &vars) {
                Ok(a) => a,
                Err(e) => {
                    errors.push(json!({ "step": i, "tool": step.tool, "error": e }));
                    if continue_on_error {
                        continue;
                    }

                    break;
                }
            };
            ran += 1;
            let result = self.run_tool(&step.tool, resolved, ctx);
            // A step that opens or starts another map begins the undo step there.
            if !self.state.doc.owns(&mark) {
                mark = self.state.doc.mark();
            }

            match result {
                ToolResult::Json(v) => {
                    let replaced = super::script::replacements(&v);
                    for name in saved_results.iter().chain(["last".to_string()].iter()) {
                        if let Some(saved) = vars.get_mut(name) {
                            super::script::follow_replacements(saved, &replaced);
                        }
                    }

                    if let Some(name) = &step.save {
                        vars.insert(name.clone(), v.clone());
                        saved_results.insert(name.clone());
                    }

                    vars.insert("last".into(), v);
                }
                ToolResult::Error(e) => {
                    errors.push(json!({ "step": i, "tool": step.tool, "error": e }));
                    if !continue_on_error {
                        break;
                    }
                }
                ToolResult::Image { .. } => {}
            }
        }

        if let Some(bounds) = self.state.focus_request.take() {
            for v in &mut self.viewports {
                v.focus(&bounds);
            }
        }

        let undo = self.state.doc.squash_since(mark, |_| format!("MCP: {label}, {ran} steps"));
        let saved: serde_json::Map<String, Value> = vars.into_iter().filter(|(k, _)| k != "last").collect();
        let summary =
            json!({ "steps": steps.len(), "notes": notes, "ran": ran, "errors": errors, "undo": undo, "vars": saved, "state": self.state_summary()["map"] });
        if errors.is_empty() || continue_on_error { ok(summary) } else { ToolResult::Error(serde_json::to_string_pretty(&summary).unwrap_or_default()) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_keeps_ends_and_spacing() {
        let pts = resample(&[DVec3::ZERO, DVec3::new(100.0, 0.0, 0.0)], 30.0);
        assert_eq!(pts.len(), 5);
        assert_eq!(*pts.last().unwrap(), DVec3::new(100.0, 0.0, 0.0));
        assert_eq!(point(&json!([3, 4])), Some(DVec3::new(3.0, 0.0, 4.0)));
    }

    #[test]
    fn link_names_default_only_when_unambiguous() {
        let one = vec!["trigger".to_string()];
        assert_eq!(link_name(None, &one, &[], "output", "button"), Ok("trigger".into()));
        assert_eq!(link_name(Some("custom"), &one, &[], "output", "button"), Ok("custom".into()));
        let two = vec!["open".to_string(), "close".to_string()];
        assert!(link_name(Some(""), &two, &[], "output", "door").unwrap_err().contains("open, close"));
        let with_builtins = vec!["toggle".to_string(), "kill".to_string()];
        assert_eq!(link_name(None, &with_builtins, &["kill"], "input", "door"), Ok("toggle".into()));
        assert!(link_name(None, &[], &[], "output", "info_null").is_err());
    }

    #[test]
    fn new_layers_are_numbered_like_the_add_layer_command() {
        let mut map = gt_doc::Map::new();
        assert_eq!(next_layer_name(&map), "Layer 2");
        map.add_layer("x");
        assert_eq!(next_layer_name(&map), "Layer 3");
    }

    #[test]
    fn scatter_items_parse_or_say_why() {
        let item = scatter_item(&json!({ "source": "res://a.glb", "scale": 1.5, "align": 1 })).unwrap();
        assert_eq!(item.scale, [1.5, 1.5], "a single number scales uniformly");
        assert_eq!(item.align, 1.0);
        assert_eq!(scatter_item(&json!({ "source": "res://a.glb", "scale": [0.5, 2] })).unwrap().scale, [0.5, 2.0]);
        assert_eq!(scatter_item(&json!("res://b.glb")).unwrap().source, "res://b.glb");
        let e = scatter_item(&json!({ "source": "res://a.glb", "align": true })).unwrap_err();
        assert!(e.contains("align") || e.contains("bool"), "a wrong field type is reported, not dropped: {e}");
        assert!(scatter_item(&json!(3)).is_err());
    }
}
