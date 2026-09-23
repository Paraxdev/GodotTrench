//! Reading an unfamiliar map and reviewing what changed in it: summarize_map, changes_since, the list_nodes filters and
//! compact get_node.

use std::collections::{BTreeMap, HashSet};

use gt_core::{Aabb, DVec3, NodeId};
use gt_doc::diff::KeyChange;
use gt_doc::{Entity, Map, NodeKind};
use serde_json::{Value, json};

use super::ToolResult;
use super::tools::uint;
use crate::app::App;
use crate::state::EditorState;

fn vec3(v: &Value) -> Option<DVec3> {
    let a = v.as_array()?;
    Some(DVec3::new(a.first()?.as_f64()?, a.get(1)?.as_f64()?, a.get(2)?.as_f64()?))
}

fn arr(v: DVec3) -> Value {
    json!([v.x, v.y, v.z])
}

fn bounds_json(b: &Aabb) -> Value {
    if b.is_empty() { Value::Null } else { json!({ "min": arr(b.min), "max": arr(b.max) }) }
}

fn layer_name(map: &Map, id: NodeId) -> String {
    map.get(map.layer_of(id)).map(|l| l.name()).unwrap_or_default()
}

/// Id, type, name, layer and bounds, enough to find a node again.
fn brief(map: &Map, id: NodeId) -> Value {
    let Some(n) = map.get(id) else { return json!({ "id": id.0 }) };
    json!({ "id": id.0, "type": n.kind.type_name(), "name": n.name(), "layer": layer_name(map, id), "bounds": bounds_json(&map.bounds(id)) })
}

fn face_materials<'a>(kind: &'a NodeKind) -> Box<dyn Iterator<Item = &'a str> + 'a> {
    match kind {
        NodeKind::Brush(b) => Box::new(b.faces.iter().map(|f| f.data.material.as_str())),
        NodeKind::Mesh(m) => Box::new(m.faces.iter().map(|f| f.data.material.as_str())),
        NodeKind::Terrain(t) => Box::new(t.layers.iter().map(|l| l.material.as_str())),
        _ => Box::new(std::iter::empty()),
    }
}

/// get_node without vertices, faces or heights.
pub(super) fn compact_node(map: &Map, id: NodeId) -> Value {
    let Some(n) = map.get(id) else { return Value::Null };
    let mut v = brief(map, id);
    v["parent"] = json!(n.parent.map(|p| p.0));
    v["children"] = json!(n.children.len());
    v["hidden"] = json!(n.hidden);
    v["locked"] = json!(n.locked);
    let mut materials: Vec<&str> = face_materials(&n.kind).filter(|m| !m.is_empty()).collect();
    materials.sort_unstable();
    materials.dedup();
    match &n.kind {
        NodeKind::Entity(e) => {
            v["classname"] = json!(e.classname);
            v["origin"] = arr(e.origin);
            v["angles"] = arr(e.angles);
            v["properties"] = json!(e.properties);
            v["outputs"] = serde_json::to_value(&e.outputs).unwrap_or_default();
        }
        NodeKind::Brush(b) => v["faces"] = json!(b.faces.len()),
        NodeKind::Mesh(m) => v["faces"] = json!(m.faces.len()),
        NodeKind::Terrain(t) => v["resolution"] = json!(t.resolution),
        NodeKind::Instance(i) => {
            v["path"] = json!(i.path);
            v["origin"] = arr(i.origin);
        }
        NodeKind::Scatter(s) => v["instances"] = json!(s.instances.len()),
        NodeKind::Layer(_) | NodeKind::Group(_) => {}
    }

    if !materials.is_empty() {
        v["materials"] = json!(materials);
    }

    v
}

/// `*` matches any run of characters, case is ignored.
fn glob(pattern: &str, text: &str) -> bool {
    let (pattern, text) = (pattern.to_ascii_lowercase(), text.to_ascii_lowercase());
    let parts: Vec<&str> = pattern.split('*').collect();
    let [first, middle @ .., last] = parts.as_slice() else { return pattern == text };
    let Some(mut rest) = text.strip_prefix(first) else { return false };
    for part in middle {
        match rest.find(part) {
            Some(i) => rest = &rest[i + part.len()..],
            None => return false,
        }
    }

    rest.ends_with(last)
}

/// The list_nodes filters besides type and classname.
#[derive(Debug, Default)]
pub(super) struct NodeFilter {
    layer: Option<NodeId>,
    material: Option<String>,
    properties: Vec<(String, String)>,
    area: Option<Aabb>,
}

impl NodeFilter {
    pub(super) fn parse(map: &Map, args: &Value) -> Result<Self, String> {
        let layer = match &args["layer"] {
            Value::Null => None,
            Value::String(name) if uint(&args["layer"]).is_none() => {
                Some(map.layers.iter().copied().find(|l| map.get(*l).is_some_and(|n| n.name().eq_ignore_ascii_case(name))).ok_or_else(|| {
                    let names: Vec<String> = map.layers.iter().filter_map(|l| map.get(*l)).map(|n| n.name()).collect();
                    format!("no layer named {name}, the layers are {}", names.join(", "))
                })?)
            }
            v => match uint(v).map(NodeId) {
                Some(id) if map.layers.contains(&id) => Some(id),
                _ => return Err(format!("layer must be a layer id or name, got {v}")),
            },
        };
        let properties = match &args["property"] {
            Value::Null => Vec::new(),
            Value::Object(o) => o
                .iter()
                .map(|(k, v)| {
                    let pattern = match v {
                        Value::String(s) => s.clone(),
                        Value::Null => "*".into(),
                        other => other.to_string(),
                    };
                    (k.clone(), pattern)
                })
                .collect(),
            other => return Err(format!("property must be an object of key and value, such as {{\"model\": \"*crate*\"}}, got {other}")),
        };
        let area = match &args["box"] {
            Value::Null => None,
            b => match (vec3(&b["min"]), vec3(&b["max"])) {
                (Some(min), Some(max)) => Some(Aabb::new(min, max)),
                _ => return Err("box needs min and max, each [x, y, z]".into()),
            },
        };
        Ok(Self { layer, material: args["material"].as_str().map(str::to_string), properties, area })
    }

    pub(super) fn matches(&self, map: &Map, id: NodeId, bounds: &Aabb) -> bool {
        let Some(n) = map.get(id) else { return false };
        if self.layer.is_some_and(|l| map.layer_of(id) != l) {
            return false;
        }

        if self.material.as_ref().is_some_and(|m| !face_materials(&n.kind).any(|f| f.eq_ignore_ascii_case(m))) {
            return false;
        }

        if !self.properties.is_empty() && n.entity().is_none_or(|e| self.properties.iter().any(|(k, p)| e.property(k).is_none_or(|v| !glob(p, v)))) {
            return false;
        }

        self.area.is_none_or(|a| !bounds.is_empty() && a.intersects(bounds))
    }

    /// The entity keys the filter asked about, with their values.
    pub(super) fn matched_properties(&self, e: &Entity) -> Option<Value> {
        (!self.properties.is_empty()).then(|| json!(self.properties.iter().map(|(k, _)| (k.clone(), e.property(k))).collect::<BTreeMap<_, _>>()))
    }
}

fn key_changes_json(changes: &[KeyChange]) -> Value {
    Value::Object(changes.iter().map(|(k, a, b)| (k.clone(), json!([a, b]))).collect())
}

/// What changed from `old` to `new`, with nodes added or removed together listed once under their topmost node.
pub(super) fn changes(old: &Map, new: &Map, limit: usize) -> Value {
    let d = gt_doc::diff::diff(old, new);
    let changed_ids: Vec<NodeId> = d.changed.iter().map(|c| c.id).collect();
    let mut counts: BTreeMap<&str, BTreeMap<&str, usize>> = BTreeMap::new();
    let mut by_layer: BTreeMap<String, BTreeMap<&str, usize>> = BTreeMap::new();
    for (what, ids, map) in [("added", &d.added, new), ("removed", &d.removed, old), ("changed", &changed_ids, new)] {
        for id in ids {
            let ty = map.get(*id).map(|n| n.kind.type_name()).unwrap_or("node");
            *counts.entry(what).or_default().entry(ty).or_default() += 1;
            *by_layer.entry(layer_name(map, *id)).or_default().entry(what).or_default() += 1;
        }
    }

    let roots = |ids: &[NodeId], map: &Map| -> Vec<NodeId> {
        let set: HashSet<NodeId> = ids.iter().copied().collect();
        ids.iter().copied().filter(|id| map.get(*id).and_then(|n| n.parent).is_none_or(|p| !set.contains(&p))).collect()
    };
    let (added_roots, removed_roots) = (roots(&d.added, new), roots(&d.removed, old));
    let listed = |ids: &[NodeId], map: &Map| -> Vec<Value> {
        ids.iter()
            .take(limit)
            .map(|id| {
                let mut v = brief(map, *id);
                let inside = map.descendants(*id).len();
                if inside > 0 {
                    v["nodes_inside"] = json!(inside);
                }

                v
            })
            .collect()
    };
    let (added, removed) = (listed(&added_roots, new), listed(&removed_roots, old));
    let mut changed = Vec::new();
    for c in d.changed.iter().take(limit) {
        let mut v = brief(new, c.id);
        v["changes"] = json!(c.what);
        if let Some(o) = c.offset {
            v["offset"] = arr(o);
        }

        if !c.properties.is_empty() {
            v["properties"] = key_changes_json(&c.properties);
        }

        changed.push(v);
    }

    let total = |what: &str| counts.get(what).map_or(0, |c| c.values().sum::<usize>());
    let mut summary = format!("{} added, {} removed, {} changed", total("added"), total("removed"), total("changed"));
    if !d.properties.is_empty() {
        summary.push_str(&format!(", {} map properties", d.properties.len()));
    }

    let truncated = [added_roots.len(), removed_roots.len(), d.changed.len()].iter().any(|n| *n > limit);
    json!({
        "summary": summary, "counts": counts, "by_layer": by_layer, "truncated": truncated,
        "added": added, "removed": removed, "changed": changed, "map_properties": key_changes_json(&d.properties),
    })
}

/// Where an entity's I/O starts, its targetname or else its class and id.
fn io_source(id: NodeId, e: &Entity) -> String {
    e.targetname().map_or_else(|| format!("{}#{}", e.classname, id.0), str::to_string)
}

pub(super) fn summarize(state: &EditorState, limit: usize) -> Value {
    let map = &state.doc.map;
    let layers: Vec<Value> = map
        .layers
        .iter()
        .filter_map(|l| map.get(*l).map(|n| (*l, n)))
        .map(|(l, n)| {
            let mut contents: BTreeMap<&str, usize> = BTreeMap::new();
            for c in map.descendants(l) {
                if let Some(c) = map.get(c) {
                    *contents.entry(c.kind.type_name()).or_default() += 1;
                }
            }

            json!({ "id": l.0, "name": n.name(), "hidden": n.hidden, "locked": n.locked, "bounds": bounds_json(&map.bounds(l)), "contents": contents })
        })
        .collect();
    let mut classes: BTreeMap<&str, usize> = BTreeMap::new();
    let mut models: BTreeMap<&str, usize> = BTreeMap::new();
    let mut io = Vec::new();
    let mut io_count = 0;
    for (id, e) in map.entities() {
        *classes.entry(e.classname.as_str()).or_default() += 1;
        for v in e.properties.values().filter(|v| crate::models::is_model_path(v) || v.to_ascii_lowercase().ends_with(".tscn")) {
            *models.entry(v.as_str()).or_default() += 1;
        }

        for o in &e.outputs {
            io_count += 1;
            if io.len() < limit {
                let mut line = format!("{}.{} -> {}.{}", io_source(id, e), o.output, o.target, o.input);
                if !o.parameter.is_empty() {
                    line.push_str(&format!("({})", o.parameter));
                }

                if o.delay > 0.0 {
                    line.push_str(&format!(" after {} s", o.delay));
                }

                io.push(line);
            }
        }
    }

    let mut materials: BTreeMap<&str, usize> = BTreeMap::new();
    for n in map.nodes.values() {
        for m in face_materials(&n.kind).filter(|m| !m.is_empty()) {
            *materials.entry(m).or_default() += 1;
        }
    }

    let unknown: Vec<&str> = classes.keys().copied().filter(|c| state.game.entity(c).is_none()).collect();
    let count = |ty: &str| map.nodes.values().filter(|n| n.kind.type_name() == ty).count();
    json!({
        "path": super::tools::path_value(state.doc.path.as_ref()),
        "bounds": bounds_json(&map.bounds_of(map.layers.iter().copied())),
        "counts": {
            "brushes": map.brush_count(), "entities": map.entity_count(), "meshes": count("mesh"), "terrains": count("terrain"),
            "groups": count("group"), "instances": count("instance"), "scatter_sets": count("scatter"),
        },
        "layers": layers,
        "entities": classes,
        "io": io, "io_count": io_count,
        "materials": materials,
        "models": models,
        "missing": {
            "materials": crate::texture_convert::missing_materials(state).into_keys().collect::<Vec<_>>(),
            "entity_classes": unknown,
        },
        "map_properties": map.properties,
    })
}

impl App {
    pub(super) fn tool_changes_since(&self, args: &Value) -> ToolResult {
        let doc = &self.state.doc;
        let limit = args["limit"].as_u64().unwrap_or(100) as usize;
        let loaded;
        let (old, since) = if let Some(path) = args["file"].as_str() {
            loaded = match gt_doc::format::load(std::path::Path::new(path)) {
                Ok(l) => l.map,
                Err(e) => return ToolResult::Error(format!("cannot read {path}: {e}")),
            };
            (&loaded, format!("the map file {path}"))
        } else if !args["undo_steps"].is_null() {
            let Some(steps) = uint(&args["undo_steps"]).filter(|n| *n > 0) else { return ToolResult::Error("undo_steps must be 1 or more".into()) };
            let Some(old) = doc.history.map_before(steps as usize) else {
                return ToolResult::Error(format!("there are only {} undo steps", doc.history.undo_labels().count()));
            };
            let label = doc.history.undo_labels().nth(steps as usize - 1).unwrap_or_default();
            (old, format!("{steps} undo steps ago, before \"{label}\""))
        } else {
            (doc.saved_map(), if doc.path.is_some() { "the last save".to_string() } else { "the new map".to_string() })
        };
        let mut v = changes(old, &doc.map, limit);
        v["since"] = json!(since);
        ToolResult::Json(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_geom::Brush;

    fn cube(map: &mut Map, parent: NodeId, at: DVec3, material: &str) -> NodeId {
        map.insert(parent, NodeKind::Brush(Brush::from_aabb(&Aabb::new(at, at + DVec3::splat(32.0)), material).unwrap()))
    }

    #[test]
    fn globs_match_like_shell_wildcards() {
        assert!(glob("*", ""));
        assert!(glob("res://*.GLB", "res://models/crate.glb"));
        assert!(glob("*sketch*fab*", "res://sketchfab/a.glb"));
        assert!(!glob("door", "door_2"));
        assert!(glob("door*", "door_2"));
        assert!(!glob("a*b*c", "a_c_b"));
        assert!(!glob("ab*ba", "aba"), "the ends may not overlap");
    }

    #[test]
    fn filters_pick_nodes_by_layer_material_property_and_box() {
        let mut map = Map::new();
        let walls = map.add_layer("Walls");
        let base = map.default_layer();
        let brick = cube(&mut map, walls, DVec3::ZERO, "brick");
        let wood = cube(&mut map, base, DVec3::X * 100.0, "wood");
        let prop = gt_doc::ops::create_point_entity(&mut map, base, "prop_model", DVec3::new(0.0, 0.0, 500.0));
        map.entity_mut(prop).unwrap().properties.insert("model".into(), "res://sketchfab/chair.glb".into());
        let pick = |args: Value| -> Vec<NodeId> {
            let f = NodeFilter::parse(&map, &args).unwrap();
            map.walk()
                .into_iter()
                .filter(|id| f.matches(&map, *id, &map.bounds(*id)))
                .filter(|id| map.brush(*id).is_some() || map.entity(*id).is_some())
                .collect()
        };
        assert_eq!(pick(json!({ "layer": "walls" })), [brick]);
        assert_eq!(pick(json!({ "layer": walls.0 })), [brick]);
        assert_eq!(pick(json!({ "material": "WOOD" })), [wood]);
        assert_eq!(pick(json!({ "property": { "model": "*sketchfab*" } })), [prop]);
        assert!(pick(json!({ "property": { "model": "*.gltf" } })).is_empty());
        assert_eq!(pick(json!({ "box": { "min": [90, 0, 0], "max": [200, 10, 10] } })), [wood]);
        assert!(NodeFilter::parse(&map, &json!({ "layer": "nope" })).unwrap_err().contains("Walls"));
        assert!(NodeFilter::parse(&map, &json!({ "layer": brick.0 })).is_err());
        assert!(NodeFilter::parse(&map, &json!({ "box": { "min": [0, 0, 0] } })).is_err());
        let f = NodeFilter::parse(&map, &json!({ "property": { "model": null } })).unwrap();
        assert_eq!(f.matched_properties(map.entity(prop).unwrap()).unwrap()["model"], "res://sketchfab/chair.glb");
    }

    #[test]
    fn changes_list_new_groups_once_and_count_everything() {
        let mut old = Map::new();
        let layer = old.default_layer();
        let moved = cube(&mut old, layer, DVec3::ZERO, "brick");
        let mut new = old.clone();
        let group = new.insert(layer, NodeKind::Group(gt_doc::Group::new("crates")));
        for i in 0..3 {
            cube(&mut new, group, DVec3::X * (100.0 * i as f64), "wood");
        }

        for v in &mut new.brush_mut(moved).unwrap().vertices {
            v.y += 32.0;
        }

        let v = changes(&old, &new, 100);
        assert_eq!(v["summary"], "4 added, 0 removed, 1 changed");
        assert_eq!(v["added"].as_array().unwrap().len(), 1, "the group stands for its brushes: {v}");
        assert_eq!(v["added"][0]["nodes_inside"], 3);
        assert_eq!(v["counts"]["added"]["brush"], 3);
        assert_eq!(v["by_layer"]["Default"]["added"], 4);
        assert_eq!(v["changed"][0]["changes"], json!(["moved"]));
        assert_eq!(v["changed"][0]["offset"], json!([0.0, 32.0, 0.0]));
        assert_eq!(changes(&old, &new, 0)["truncated"], true);
    }

    #[test]
    fn summaries_count_layers_classes_materials_and_io() {
        let mut state = EditorState::new(Default::default());
        let layer = state.doc.map.default_layer();
        state.doc.edit("build", |m, _| {
            cube(m, layer, DVec3::ZERO, "brick");
            let button = gt_doc::ops::create_point_entity(m, layer, "func_button", DVec3::ZERO);
            let e = m.entity_mut(button).unwrap();
            e.outputs.push(gt_doc::IoConnection {
                output: "pressed".into(),
                target: "door".into(),
                input: "open".into(),
                parameter: String::new(),
                delay: 0.5,
                times: -1,
            });
            gt_doc::ops::create_point_entity(m, layer, "no_such_class", DVec3::ZERO);
        });
        let v = summarize(&state, 100);
        assert_eq!(v["counts"]["brushes"], 1);
        assert_eq!(v["layers"][0]["contents"]["entity"], 2);
        assert_eq!(v["entities"]["func_button"], 1);
        assert_eq!(v["materials"]["brick"], 6);
        assert_eq!(v["io"][0].as_str().unwrap(), format!("func_button#{}.pressed -> door.open after 0.5 s", layer.0 + 2));
        assert_eq!(v["missing"]["entity_classes"], json!(["no_such_class"]));
    }
}
