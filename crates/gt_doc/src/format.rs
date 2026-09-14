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
}

#[derive(Serialize, Deserialize)]
struct FileMap {
    format: String,
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
        nodes: Vec<FileNode>,
    }
    let clip: Clip = serde_json::from_str(text)?;
    if clip.format != "godottrench-clipboard" {
        return Err(FormatError::WrongFormat(clip.format));
    }
    let mut out = Vec::new();
    for node in clip.nodes {
        if matches!(node.kind, FileKind::Layer(_)) {
            for child in node.children {
                out.push(insert_fresh(map, parent, child));
            }
        } else {
            out.push(insert_fresh(map, parent, node));
        }
    }
    Ok(out)
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

fn insert_fresh(map: &mut Map, parent: NodeId, node: FileNode) -> NodeId {
    let id = map.insert(parent, file_kind_to_node(node.kind));
    if let Some(n) = map.get_mut(id) {
        n.hidden = node.hidden;
        n.locked = node.locked;
    }
    for c in node.children {
        insert_fresh(map, id, c);
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
    let file: FileMap = serde_json::from_value(value)?;

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
    }
}
