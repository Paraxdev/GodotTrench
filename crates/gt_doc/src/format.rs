//! `.gtm` GodotTrench map files. See docs/format-gtm.md.

use std::collections::BTreeMap;
use std::path::Path;

use gt_core::NodeId;
use gt_geom::{Brush, Mesh, Terrain};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::entity::Entity;
use crate::map::{EditorData, Group, Instance, Layer, Map, Node, NodeKind};
use crate::scatter::Scatter;

pub const FORMAT_NAME: &str = "godottrench-map";
pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not a GodotTrench map (format = {0:?})")]
    WrongFormat(String),
    #[error("map version {0} is newer than this editor supports ({FORMAT_VERSION})")]
    TooNew(u32),
    #[error("node {id}: {reason}")]
    Invalid { id: u64, reason: String },
}

#[derive(Serialize, Deserialize)]
struct FileMap {
    format: String,
    #[serde(default)]
    version: u32,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    properties: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "is_default_editor")]
    editor: EditorData,
    layers: Vec<FileNode>,
}

fn is_default_editor(e: &EditorData) -> bool {
    *e == EditorData::default()
}

#[derive(Serialize, Deserialize)]
struct FileNode {
    id: u64,
    #[serde(flatten)]
    kind: FileKind,
    #[serde(default, skip_serializing_if = "is_false")]
    hidden: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    locked: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    children: Vec<FileNode>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum FileKind {
    Layer(Layer),
    Group(Group),
    Entity(Entity),
    Brush(Brush),
    Instance(Instance),
    Mesh(Mesh),
    Terrain(Terrain),
    Scatter(Scatter),
}

fn to_file_node(map: &Map, node: &Node) -> FileNode {
    FileNode { children: node.children.iter().filter_map(|c| map.get(*c)).map(|c| to_file_node(map, c)).collect(), ..leaf_file_node(node) }
}

fn leaf_file_node(node: &Node) -> FileNode {
    let kind = match &node.kind {
        NodeKind::Layer(l) => FileKind::Layer(l.clone()),
        NodeKind::Group(g) => FileKind::Group(g.clone()),
        NodeKind::Entity(e) => FileKind::Entity(e.clone()),
        NodeKind::Brush(b) => FileKind::Brush(b.clone()),
        NodeKind::Instance(i) => FileKind::Instance(i.clone()),
        NodeKind::Mesh(m) => FileKind::Mesh(m.clone()),
        NodeKind::Terrain(t) => FileKind::Terrain(t.clone()),
        NodeKind::Scatter(s) => FileKind::Scatter(s.clone()),
    };
    FileNode { id: node.id.0, kind, hidden: node.hidden, locked: node.locked, children: Vec::new() }
}

pub fn to_value(map: &Map) -> Value {
    let file = FileMap {
        format: FORMAT_NAME.into(),
        version: FORMAT_VERSION,
        properties: map.properties.clone(),
        editor: map.editor.clone(),
        layers: map.layers.iter().filter_map(|l| map.get(*l)).map(|l| to_file_node(map, l)).collect(),
    };
    serde_json::to_value(file).expect("map serializes")
}

pub fn to_string(map: &Map) -> String {
    crate::json_fmt::to_string(&to_value(map))
}

/// One node as it appears in the file, without its children.
pub fn node_to_json(map: &Map, id: NodeId) -> Option<Value> {
    serde_json::to_value(leaf_file_node(map.get(id)?)).ok()
}

/// Serializes only the given subtrees, for the clipboard. Parents are implied.
pub fn nodes_to_string(map: &Map, ids: &[NodeId]) -> String {
    let nodes: Vec<FileNode> = ids.iter().filter_map(|id| map.get(*id)).map(|n| to_file_node(map, n)).collect();
    let value = serde_json::json!({ "format": "godottrench-clipboard", "version": FORMAT_VERSION, "nodes": nodes });
    crate::json_fmt::to_string(&value)
}

/// Parses clipboard text and inserts the nodes under `parent` with fresh ids. Returns the new root ids.
pub fn paste_nodes(map: &mut Map, parent: NodeId, text: &str) -> Result<Vec<NodeId>, FormatError> {
    #[derive(Deserialize)]
    struct Clip {
        format: String,
        #[serde(default)]
        version: u32,
        nodes: Vec<FileNode>,
    }

    let mut value: Value = serde_json::from_str(text)?;
    fill_defaults(&mut value);
    let clip: Clip = serde_json::from_value(value)?;
    if clip.format != "godottrench-clipboard" {
        return Err(FormatError::WrongFormat(clip.format));
    }

    if clip.version > FORMAT_VERSION {
        return Err(FormatError::TooNew(clip.version));
    }

    clip.nodes.iter().try_for_each(validate)?;
    let mut out = Vec::new();
    let mut copies = BTreeMap::new();
    for node in clip.nodes {
        if matches!(node.kind, FileKind::Layer(_)) {
            for child in node.children {
                out.push(insert_fresh(map, parent, child, &mut copies));
            }
        } else {
            out.push(insert_fresh(map, parent, node, &mut copies));
        }
    }

    map.retarget_scatter_copies(&copies);
    Ok(out)
}

/// Rejects geometry the editor would otherwise index out of bounds, which only a hand edited file can contain.
fn validate(node: &FileNode) -> Result<(), FormatError> {
    let checked = match &node.kind {
        FileKind::Brush(b) => b.check_data(),
        FileKind::Mesh(m) => m.check_data(),
        FileKind::Terrain(t) => t.check_data(),
        _ => Ok(()),
    };
    checked.map_err(|reason| FormatError::Invalid { id: node.id, reason })?;
    node.children.iter().try_for_each(validate)
}

fn file_kind_to_node(kind: FileKind) -> NodeKind {
    match kind {
        FileKind::Layer(l) => NodeKind::Layer(l),
        FileKind::Group(g) => NodeKind::Group(g),
        FileKind::Entity(e) => NodeKind::Entity(e),
        FileKind::Brush(b) => NodeKind::Brush(b),
        FileKind::Instance(i) => NodeKind::Instance(i),
        FileKind::Mesh(m) => NodeKind::Mesh(m),
        FileKind::Terrain(t) => NodeKind::Terrain(t),
        FileKind::Scatter(s) => NodeKind::Scatter(s),
    }
}

fn insert_fresh(map: &mut Map, parent: NodeId, node: FileNode, copies: &mut BTreeMap<NodeId, NodeId>) -> NodeId {
    let id = map.insert(parent, file_kind_to_node(node.kind));
    copies.insert(NodeId(node.id), id);
    if let Some(n) = map.get_mut(id) {
        n.hidden = node.hidden;
        n.locked = node.locked;
    }

    for c in node.children {
        insert_fresh(map, id, c, copies);
    }

    id
}

fn insert_file_node(map: &mut Map, parent: Option<NodeId>, node: FileNode) {
    let id = NodeId(node.id);
    // Duplicate ids can only come from hand edited files, give those a fresh id.
    let id = if map.contains(id) || id.0 == 0 { map.alloc_id() } else { id };
    map.next_id = map.next_id.max(id.0 + 1);
    let kind = file_kind_to_node(node.kind);
    match parent {
        Some(p) => map.insert_with_id(id, p, kind),
        None => {
            map.nodes.insert(id, Node { id, parent: None, children: Vec::new(), kind, hidden: false, locked: false });
            map.layers.push(id);
        }
    }

    if let Some(n) = map.get_mut(id) {
        n.hidden = node.hidden;
        n.locked = node.locked;
    }

    for c in node.children {
        insert_file_node(map, Some(id), c);
    }
}

/// Defaults that depend on another key, which serde cannot express. A foliage set without `collision` has none, as
/// in `Scatter::new` and the Godot scatter reader.
fn fill_defaults(node: &mut Value) {
    if node.get("type").and_then(Value::as_str) == Some("scatter")
        && node.get("kind").and_then(Value::as_str) == Some("foliage")
        && let Some(obj) = node.as_object_mut()
    {
        obj.entry("collision").or_insert_with(|| Value::from("none"));
    }

    for list in ["children", "layers", "nodes"] {
        if let Some(children) = node.get_mut(list).and_then(Value::as_array_mut) {
            children.iter_mut().for_each(fill_defaults);
        }
    }
}

fn migrate(value: &mut Value, version: u32) {
    // Version 1 is the first format, future migrations go here in ascending order.
    let _ = (value, version);
}

pub fn from_str(text: &str) -> Result<Map, FormatError> {
    let mut value: Value = serde_json::from_str(text)?;
    let format = value.get("format").and_then(|v| v.as_str()).unwrap_or_default().to_string();
    if format != FORMAT_NAME {
        return Err(FormatError::WrongFormat(format));
    }

    let version = value.get("version").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    if version > FORMAT_VERSION {
        return Err(FormatError::TooNew(version));
    }

    migrate(&mut value, version);
    fill_defaults(&mut value);
    let file: FileMap = serde_json::from_value(value)?;
    file.layers.iter().try_for_each(validate)?;

    let mut map = Map { nodes: imbl::OrdMap::new(), layers: Vec::new(), properties: file.properties, next_id: 1, editor: file.editor };
    for layer in file.layers {
        if matches!(layer.kind, FileKind::Layer(_)) {
            insert_file_node(&mut map, None, layer);
        }
    }

    if map.layers.is_empty() {
        map.add_layer("Default");
    }

    Ok(map)
}

pub fn load(path: &Path) -> Result<Map, FormatError> {
    from_str(&std::fs::read_to_string(path)?)
}

/// Writes atomically through a temp file so a crash never leaves a truncated map.
pub fn save(map: &Map, path: &Path) -> Result<(), FormatError> {
    let tmp = path.with_extension("gtm.tmp");
    std::fs::write(&tmp, to_string(map))?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IoConnection;
    use gt_core::{Aabb, DVec3};

    fn sample() -> Map {
        let mut m = Map::new();
        m.properties.insert("classname".into(), "worldspawn".into());
        let layer = m.default_layer();
        let b = Brush::from_aabb(&Aabb::new(DVec3::splat(-16.0), DVec3::splat(16.0)), "base/floor").unwrap();
        m.insert(layer, NodeKind::Brush(b.clone()));
        let g = m.insert(layer, NodeKind::Group(Group { link_id: Some(3), ..Group::new("room") }));
        let mut e = Entity::new("func_door");
        e.properties.insert("targetname".into(), "door1".into());
        e.outputs.push(IoConnection {
            output: "opened".into(),
            target: "light1".into(),
            input: "turn_on".into(),
            parameter: String::new(),
            delay: 0.5,
            times: 1,
        });
        let e = m.insert(g, NodeKind::Entity(e));
        m.insert(e, NodeKind::Brush(b.translated(DVec3::new(64.0, 0.0, 0.0), true)));
        let l2 = m.add_layer("Details");
        m.get_mut(l2).unwrap().hidden = true;
        m
    }

    #[test]
    fn round_trip() {
        let m = sample();
        let text = to_string(&m);
        let back = from_str(&text).unwrap();
        assert_eq!(back.layers, m.layers);
        assert_eq!(back.properties, m.properties);
        for (id, node) in m.nodes.iter() {
            let other = back.get(*id).unwrap();
            assert_eq!(node.parent, other.parent);
            assert_eq!(node.children, other.children);
            assert_eq!(node.hidden, other.hidden);
            assert_eq!(node.kind.type_name(), other.kind.type_name());
        }

        assert_eq!(to_string(&back), text);
    }

    #[test]
    fn broken_geometry_is_an_error_not_a_panic() {
        let mut v = to_value(&sample());
        v["layers"][0]["children"][0]["faces"][0]["indices"][0] = serde_json::json!(99);
        let err = from_str(&v.to_string()).unwrap_err().to_string();
        assert!(err.contains("face 0 uses vertex 99"), "{err}");

        let mut v = to_value(&sample());
        v["layers"][0]["children"][0]["faces"][1]["indices"] = serde_json::json!([0, 1]);
        assert!(matches!(from_str(&v.to_string()), Err(FormatError::Invalid { .. })));

        let mut v = to_value(&sample());
        v.as_object_mut().unwrap().remove("version");
        assert!(from_str(&v.to_string()).is_ok(), "a missing version reads as the oldest format");
    }

    #[test]
    fn foliage_without_collision_has_none() {
        let m = Map::new();
        let layer = m.default_layer();
        let mut v = to_value(&m);
        let set = |kind: &str| serde_json::json!({ "id": 50, "type": "scatter", "name": "s", "kind": kind, "items": [] });
        v["layers"][0]["children"] = serde_json::json!([set("foliage")]);
        let back = from_str(&v.to_string()).unwrap();
        let id = back.get(layer).unwrap().children[0];
        assert_eq!(back.scatter(id).unwrap().collision, crate::scatter::ScatterCollision::None);
        v["layers"][0]["children"] = serde_json::json!([set("props")]);
        let back = from_str(&v.to_string()).unwrap();
        let id = back.get(layer).unwrap().children[0];
        assert_eq!(back.scatter(id).unwrap().collision, crate::scatter::ScatterCollision::Convex);
    }

    #[test]
    fn faces_are_single_lines() {
        let text = to_string(&sample());
        let face_lines = text.lines().filter(|l| l.trim_start().starts_with("{\"indices\"")).count();
        assert_eq!(face_lines, 12);
    }

    #[test]
    fn single_node_json_matches_the_file_without_children() {
        let m = sample();
        let file = to_value(&m);
        let group = &file["layers"][0]["children"][1];
        let json = node_to_json(&m, NodeId(group["id"].as_u64().unwrap())).unwrap();
        assert_eq!(json["type"], "group");
        assert_eq!(json["name"], "room");
        assert!(json.get("children").is_none());
        let door = &group["children"][0];
        let mut expected = door.clone();
        expected.as_object_mut().unwrap().remove("children");
        assert_eq!(node_to_json(&m, NodeId(door["id"].as_u64().unwrap())).unwrap(), expected);
        assert!(node_to_json(&m, NodeId(9999)).is_none());
    }

    #[test]
    fn clipboard_round_trip() {
        let mut m = sample();
        let layer = m.default_layer();
        let ids: Vec<NodeId> = m.get(layer).unwrap().children.clone();
        let text = nodes_to_string(&m, &ids);
        let before = m.nodes.len();
        let pasted = paste_nodes(&mut m, layer, &text).unwrap();
        assert_eq!(pasted.len(), ids.len());
        assert_eq!(m.nodes.len(), before + 4);
        let newer = text.replacen(&format!("\"version\": {FORMAT_VERSION}"), &format!("\"version\": {}", FORMAT_VERSION + 1), 1);
        assert!(matches!(paste_nodes(&mut m, layer, &newer), Err(FormatError::TooNew(_))));
    }

    #[test]
    fn copied_scatter_sets_follow_copied_surfaces() {
        let mut m = Map::new();
        let layer = m.default_layer();
        let ground = m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(64.0)), "g").unwrap()));
        let other = m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(8.0)), "o").unwrap()));
        let mut set = Scatter::new("trees", crate::scatter::ScatterKind::Props, Vec::new());
        set.targets = vec![ground, other];
        let scatter = m.insert(layer, NodeKind::Scatter(set));

        let text = nodes_to_string(&m, &[ground, scatter]);
        let pasted = paste_nodes(&mut m, layer, &text).unwrap();
        assert_eq!(m.scatter(pasted[1]).unwrap().targets, vec![pasted[0], other], "the pasted ground is targeted, the uncopied brush kept");

        let mut sel = crate::Selection::default();
        sel.nodes.extend([ground, scatter]);
        let copies = crate::ops::duplicate_selection(&mut m, &mut sel, DVec3::ZERO, crate::ops::EditOptions { uv_lock: true, grid: 0.0 });
        assert_eq!(m.scatter(copies[1]).unwrap().targets, vec![copies[0], other]);
    }
}
