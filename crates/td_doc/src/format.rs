//! `.tdm` TrenchDot map files. See docs/format/README.md.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use td_core::NodeId;
use td_geom::{Brush, Mesh, Terrain};

use crate::entity::Entity;
use crate::map::{EditorData, Group, Instance, Layer, Map, Node, NodeKind};
use crate::scatter::{Scatter, ScatterKind};

pub const FORMAT_NAME: &str = "trenchdot-map";
pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not a TrenchDot map (format = {0:?})")]
    WrongFormat(String),
    #[error("map version {0} is newer than this editor supports ({FORMAT_VERSION})")]
    TooNew(u32),
    #[error("node {id}: {reason}")]
    Invalid { id: u64, reason: String },
    #[error("paste target {0} does not exist")]
    MissingParent(u64),
    #[error("{0}")]
    Container(#[from] crate::binary::ContainerError),
}

/// A map read from a file, with what was lost or moved while reading a damaged one.
pub struct Loaded {
    pub map: Map,
    pub problems: Vec<String>,
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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    textures: BTreeMap<String, crate::textures::TextureSettings>,
    /// Read by [`node_list`].
    #[serde(skip_deserializing)]
    layers: Vec<FileNode>,
}

fn is_default_editor(e: &EditorData) -> bool {
    *e == EditorData::default()
}

#[derive(Serialize)]
struct FileNode {
    id: u64,
    #[serde(flatten)]
    kind: FileKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    graph: Option<[f32; 2]>,
    #[serde(skip_serializing_if = "is_false")]
    hidden: bool,
    #[serde(skip_serializing_if = "is_false")]
    locked: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    children: Vec<FileNode>,
}

/// The keys of a node beside `children` and those of its kind.
#[derive(Deserialize)]
struct NodeFields {
    id: u64,
    #[serde(default)]
    label: Option<String>,
    #[serde(default, deserialize_with = "graph_pos")]
    graph: Option<[f32; 2]>,
    #[serde(default)]
    hidden: bool,
    #[serde(default)]
    locked: bool,
}

const NODE_FIELDS: [&str; 5] = ["id", "label", "graph", "hidden", "locked"];

/// Reads a node and its subtree by hand. Serde's `flatten` buffers the content of a node twice on the way to its kind,
/// strings included, so one big terrain took several times its size in memory.
fn file_node(value: Value) -> Result<FileNode, serde_json::Error> {
    let Value::Object(mut obj) = value else { return Err(invalid_type(&value, "struct FileNode")) };
    let children = obj.remove("children").map(node_list).transpose()?.unwrap_or_default();
    let fields: serde_json::Map<String, Value> = NODE_FIELDS.iter().filter_map(|k| obj.remove_entry(*k)).collect();
    let NodeFields { id, label, graph, hidden, locked } = NodeFields::deserialize(Value::Object(fields))?;
    let kind = FileKind::deserialize(Value::Object(obj))?;
    Ok(FileNode { id, kind, label, graph, hidden, locked, children })
}

fn node_list(value: Value) -> Result<Vec<FileNode>, serde_json::Error> {
    match value {
        Value::Array(items) => items.into_iter().map(file_node).collect(),
        other => Err(invalid_type(&other, "a sequence")),
    }
}

fn invalid_type(value: &Value, expected: &str) -> serde_json::Error {
    use serde::de::Unexpected;
    let unexpected = match value {
        Value::Null => Unexpected::Unit,
        Value::Bool(b) => Unexpected::Bool(*b),
        Value::Number(n) => match (n.as_u64(), n.as_i64()) {
            (Some(u), _) => Unexpected::Unsigned(u),
            (_, Some(i)) => Unexpected::Signed(i),
            _ => Unexpected::Float(n.as_f64().unwrap_or_default()),
        },
        Value::String(s) => Unexpected::Str(s),
        Value::Array(_) => Unexpected::Seq,
        Value::Object(_) => Unexpected::Map,
    };
    serde::de::Error::invalid_type(unexpected, &expected)
}

/// The map's top level with its layers.
fn file_map(mut value: Value) -> Result<FileMap, serde_json::Error> {
    let layers = value.as_object_mut().and_then(|o| o.remove("layers"));
    let mut file = FileMap::deserialize(value)?;
    file.layers = node_list(layers.ok_or_else(|| serde::de::Error::missing_field("layers"))?)?;
    Ok(file)
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Only the editor reads a graph position, so a hand edited one that is not two numbers is dropped rather than refusing
/// the whole map.
fn graph_pos<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<[f32; 2]>, D::Error> {
    Ok(Option::<[f32; 2]>::deserialize(Value::deserialize(d)?).ok().flatten())
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
    FileNode { id: node.id.0, kind, label: node.label.clone(), graph: node.graph, hidden: node.hidden, locked: node.locked, children: Vec::new() }
}

pub fn to_value(map: &Map) -> Value {
    let file = FileMap {
        format: FORMAT_NAME.into(),
        version: FORMAT_VERSION,
        properties: map.properties.clone(),
        editor: map.editor.clone(),
        textures: map.textures.clone(),
        layers: map.layers.iter().filter_map(|l| map.get(*l)).map(|l| to_file_node(map, l)).collect(),
    };
    serde_json::to_value(file).expect("map serializes")
}

/// The map as readable JSON, the layout of older `.tdm` files and of the clipboard and live link.
pub fn to_string(map: &Map) -> String {
    crate::json_fmt::to_string(&to_value(map))
}

/// [`to_string`] with the bake under `lightmap`, the readable layout of a whole map file and of the live link.
pub fn to_string_with_lightmap(map: &Map) -> String {
    crate::json_fmt::to_string(&value_with_lightmap(map))
}

fn value_with_lightmap(map: &Map) -> Value {
    let mut value = to_value(map);
    if let Some(lm) = &map.lightmap
        && let Some(obj) = value.as_object_mut()
    {
        obj.insert("lightmap".into(), lm.to_json(&lm.stale_nodes(map)));
    }

    value
}

/// A value in the readable JSON layout, written to `out` as it is made.
pub fn write_json(out: &mut impl std::io::Write, value: &Value) -> std::io::Result<()> {
    crate::json_fmt::write(out, value)
}

/// The map in the binary container that `.tdm` files are saved in. Refuses a map too big for the editor to read back.
pub fn to_bytes(map: &Map) -> Result<Vec<u8>, FormatError> {
    let encoded = match &map.lightmap {
        Some(lm) => {
            let mut chunks = map.unknown_chunks.clone();
            chunks.push(crate::binary::bulk_chunk(crate::lightmap::TAG, &lm.to_chunk(&lm.stale_nodes(map))));
            crate::binary::encode(to_value(map), &chunks)
        }
        None => crate::binary::encode(to_value(map), &map.unknown_chunks),
    };
    Ok(encoded?)
}

/// Identifies the map content the way the END chunk of `to_bytes` records it.
pub fn content_id(map: &Map) -> i64 {
    crate::binary::content_id(&to_value(map)).as_i64()
}

/// One node as it appears in the file, without its children.
pub fn node_to_json(map: &Map, id: NodeId) -> Option<Value> {
    serde_json::to_value(leaf_file_node(map.get(id)?)).ok()
}

/// Serializes only the given subtrees, for the clipboard. Parents are implied.
pub fn nodes_to_string(map: &Map, ids: &[NodeId]) -> String {
    let nodes: Vec<FileNode> = ids.iter().filter_map(|id| map.get(*id)).map(|n| to_file_node(map, n)).collect();
    let value = serde_json::json!({ "format": "trenchdot-clipboard", "version": FORMAT_VERSION, "nodes": nodes });
    crate::json_fmt::to_string(&value)
}

/// Parses clipboard text and inserts the nodes under `parent` with fresh ids. Returns the new root ids.
pub fn paste_nodes(map: &mut Map, parent: NodeId, text: &str) -> Result<Vec<NodeId>, FormatError> {
    #[derive(Deserialize)]
    struct Clip {
        format: String,
        #[serde(default)]
        version: u32,
    }

    let mut value: Value = serde_json::from_str(text)?;
    fill_defaults(&mut value);
    let nodes = value.as_object_mut().and_then(|o| o.remove("nodes"));
    let clip: Clip = serde_json::from_value(value)?;
    let nodes = node_list(nodes.ok_or_else(|| <serde_json::Error as serde::de::Error>::missing_field("nodes"))?)?;
    if clip.format != "trenchdot-clipboard" {
        return Err(FormatError::WrongFormat(clip.format));
    }

    if clip.version > FORMAT_VERSION {
        return Err(FormatError::TooNew(clip.version));
    }

    nodes.iter().try_for_each(validate)?;
    if !map.contains(parent) {
        return Err(FormatError::MissingParent(parent.0));
    }

    let mut out = Vec::new();
    let mut copies = BTreeMap::new();
    for node in nodes {
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

/// How far pasted nodes move in the Logic graph, so the copies cascade off the originals instead of hiding them.
pub const PASTE_GRAPH_OFFSET: f32 = 40.0;

fn insert_fresh(map: &mut Map, parent: NodeId, node: FileNode, copies: &mut BTreeMap<NodeId, NodeId>) -> NodeId {
    let id = map.insert(parent, file_kind_to_node(node.kind));
    copies.insert(NodeId(node.id), id);
    if let Some(n) = map.get_mut(id) {
        n.hidden = node.hidden;
        n.locked = node.locked;
        n.set_label(node.label);
        n.set_graph(node.graph.map(|p| p.map(|v| v + PASTE_GRAPH_OFFSET)));
    }

    for c in node.children {
        insert_fresh(map, id, c, copies);
    }

    id
}

fn max_file_id(node: &FileNode) -> u64 {
    node.children.iter().map(max_file_id).fold(node.id, u64::max)
}

/// Expects `map.next_id` past every id in the file, so a repaired id never takes one a later node owns.
fn insert_file_node(map: &mut Map, parent: Option<NodeId>, node: FileNode) {
    let id = NodeId(node.id);
    // Duplicate ids can only come from hand edited files, give those a fresh id.
    let id = if map.contains(id) || id.0 == 0 { map.alloc_id() } else { id };
    map.next_id = map.next_id.max(id.0 + 1);
    let kind = file_kind_to_node(node.kind);
    match parent {
        Some(p) => {
            map.insert_with_id(id, p, kind);
        }
        None => {
            map.nodes.insert(id, Node { id, parent: None, children: Vec::new(), kind, hidden: false, locked: false, label: None, graph: None });
            map.layers.push(id);
        }
    }

    if let Some(n) = map.get_mut(id) {
        n.hidden = node.hidden;
        n.locked = node.locked;
        n.set_label(node.label);
        n.set_graph(node.graph);
    }

    for c in node.children {
        insert_file_node(map, Some(id), c);
    }
}

/// Defaults that depend on another key, which serde cannot express. A scatter set missing a setting gets what
/// `Scatter::new` gives its kind, so a foliage set without `collision` has none.
fn fill_defaults(node: &mut Value) {
    if node.get("type").and_then(Value::as_str) == Some("scatter")
        && let Some(obj) = node.as_object_mut()
    {
        let kind = if obj.get("kind").and_then(Value::as_str) == Some("foliage") { ScatterKind::Foliage } else { ScatterKind::Props };
        if let Ok(Value::Object(defaults)) = serde_json::to_value(Scatter::new("", kind, Vec::new())) {
            for key in ["collision", "cast_shadows", "visibility_range", "chunk_size"] {
                if let Some(v) = defaults.get(key) {
                    obj.entry(key).or_insert_with(|| v.clone());
                }
            }
        }
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

/// Reads a map saved as JSON text.
pub fn from_str(text: &str) -> Result<Map, FormatError> {
    from_value(serde_json::from_str(text.trim_start_matches('\u{feff}'))?)
}

/// Reads a map file of either kind, told apart by its first bytes.
pub fn from_bytes(bytes: &[u8]) -> Result<Loaded, FormatError> {
    let (value, mut problems, unknown, lightmap) = file_value(bytes)?;
    let mut map = from_value(value)?;
    if let Some(lightmap) = lightmap {
        map.lightmap = crate::lightmap::Lightmap::from_json(&lightmap).map(std::sync::Arc::new);
        if map.lightmap.is_none() {
            problems.push(LIGHTMAP_DROPPED.into());
        }
    }

    map.unknown_chunks = unknown.into_iter().filter(|c| c.tag != crate::lightmap::TAG).collect();
    Ok(Loaded { map, problems })
}

const LIGHTMAP_DROPPED: &str = "the baked lighting is damaged or from a newer editor and was dropped, bake the lighting again";

/// The value tree, the problems, the chunks kept as they are stored and the value of the LMAP chunk.
type FileValue = (Value, Vec<String>, Vec<crate::binary::RawChunk>, Option<Value>);

/// The file's content as the JSON value tree, without building a map.
fn file_value(bytes: &[u8]) -> Result<FileValue, FormatError> {
    if crate::binary::is_binary(bytes) {
        let decoded = crate::binary::decode(bytes)?;
        return Ok((decoded.value, decoded.problems, decoded.unknown, decoded.lightmap));
    }

    let text = std::str::from_utf8(bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok((serde_json::from_str(text.trim_start_matches('\u{feff}'))?, Vec::new(), Vec::new(), None))
}

/// Any map file in the readable JSON layout, for `trenchdot --dump` and `--to-json`.
pub fn file_to_json(bytes: &[u8]) -> Result<(String, Vec<String>), FormatError> {
    let (value, problems) = file_json_value(bytes)?;
    Ok((crate::json_fmt::to_string(&value), problems))
}

/// The value [`file_to_json`] writes out, with what a damaged file lost. [`write_json`] writes it without holding the
/// text, which can be several times the size of the map in memory.
pub fn file_json_value(bytes: &[u8]) -> Result<(Value, Vec<String>), FormatError> {
    let (mut value, mut problems, _, lightmap) = file_value(bytes)?;
    if let Some(lightmap) = lightmap {
        // The chunk decodes to the JSON layout already, `save` to a .json path writes the same key.
        match crate::lightmap::Lightmap::from_json(&lightmap) {
            Some(_) => {
                if let Some(obj) = value.as_object_mut() {
                    obj.insert("lightmap".into(), lightmap);
                }
            }
            None => problems.push(LIGHTMAP_DROPPED.into()),
        }
    }

    Ok((value, problems))
}

/// Any map file as the binary container with exactly its content, for `trenchdot --to-tdm`. Refuses a file the
/// editor could not open, found by reading the result back rather than by building a map from a copy of the content.
pub fn file_to_binary(bytes: &[u8]) -> Result<(Vec<u8>, Vec<String>), FormatError> {
    let (value, problems, unknown, _) = file_value(bytes)?;
    let out = crate::binary::encode(value, &unknown)?;
    from_bytes(&out)?;
    Ok((out, problems))
}

fn from_value(mut value: Value) -> Result<Map, FormatError> {
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
    let lightmap = value.get("lightmap").and_then(crate::lightmap::Lightmap::from_json).map(std::sync::Arc::new);
    let file = file_map(value)?;
    file.layers.iter().try_for_each(validate)?;

    let next_id = file.layers.iter().map(max_file_id).max().unwrap_or(0) + 1;
    let mut map = Map {
        nodes: imbl::OrdMap::new(),
        layers: Vec::new(),
        properties: file.properties,
        next_id,
        editor: file.editor,
        unknown_chunks: Vec::new(),
        instance_extents: Default::default(),
        lightmap,
        textures: file.textures.into_iter().map(|(k, v)| (k, v.sanitized())).filter(|(_, v)| !v.is_default()).collect(),
    };
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

pub fn load(path: &Path) -> Result<Loaded, FormatError> {
    from_bytes(&std::fs::read(path)?)
}

/// Writes atomically through a temp file so a crash never leaves a truncated map. A `.json` path gets the readable
/// JSON layout for hand editing, anything else the binary container.
pub fn save(map: &Map, path: &Path) -> Result<(), FormatError> {
    let tmp = path.with_extension("tdm.tmp");
    // Whatever sits at the temp path, a link included, is replaced rather than written through.
    if tmp.symlink_metadata().is_ok() {
        std::fs::remove_file(&tmp)?;
    }

    let file = || std::fs::OpenOptions::new().write(true).create_new(true).open(&tmp);
    if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("json")) {
        let mut out = std::io::BufWriter::new(file()?);
        write_json(&mut out, &value_with_lightmap(map))?;
        std::io::Write::flush(&mut out)?;
    } else {
        let bytes = to_bytes(map)?;
        std::io::Write::write_all(&mut file()?, &bytes)?;
    }

    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IoConnection;
    use td_core::{Aabb, DVec3};

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
    fn texture_settings_survive_both_layouts() {
        let mut m = sample();
        let wallpaper = crate::textures::TextureSettings { size: Some([256.0, 128.0]), projection: crate::textures::Projection::World, ..Default::default() };
        m.set_texture("backrooms/wallpaper", wallpaper.clone());
        m.set_texture("glass", crate::textures::TextureSettings { bake: false, casts: false, ..Default::default() });
        m.set_texture("plain", crate::textures::TextureSettings::default());
        assert_eq!(m.textures.len(), 2, "default settings are not stored");
        let back = from_bytes(&to_bytes(&m).unwrap()).unwrap().map;
        assert_eq!(back.textures, m.textures);
        let json = from_bytes(to_string(&m).as_bytes()).unwrap().map;
        assert_eq!(json.texture("backrooms/wallpaper"), wallpaper);
        assert!(!json.texture("glass").bake && json.texture("missing").bake);
    }

    #[test]
    fn baked_lighting_survives_both_layouts() {
        let mut m = sample();
        let mut lm = crate::lightmap::Lightmap { width: 4, height: 2, texel_size: 16.0, scene: 5, ..Default::default() };
        lm.light = (0..24).map(|i| crate::lightmap::f32_to_f16(i as f32 * 0.25)).collect();
        lm.shadow = vec![255; 8];
        lm.ao = (0..8).map(|i| i as u8 * 30).collect();
        lm.charts.push(crate::lightmap::Chart { node: 2, face: 4, rows: [[0.25, 0.0, 0.0, 0.5], [0.0, 0.0, 0.5, 0.125]] });
        lm.nodes.insert(2, 77);
        m.lightmap = Some(std::sync::Arc::new(lm));
        let back = from_bytes(&to_bytes(&m).unwrap()).unwrap();
        assert!(back.problems.is_empty(), "{:?}", back.problems);
        assert_eq!(back.map.lightmap, m.lightmap);
        assert!(back.map.unknown_chunks.is_empty(), "the chunk is read, not kept as unknown");
        let (dumped, problems) = file_to_json(&to_bytes(&m).unwrap()).unwrap();
        assert!(problems.is_empty(), "{problems:?}");
        let dumped: Value = serde_json::from_str(&dumped).unwrap();
        assert_eq!(dumped["lightmap"]["stale"], serde_json::json!([2]));
        assert_eq!(from_str(&dumped.to_string()).unwrap().lightmap, m.lightmap, "the JSON dump keeps the bake");

        let dir = std::env::temp_dir().join(format!("td_lightmap_json_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("lit.json");
        save(&m, &path).unwrap();
        assert_eq!(load(&path).unwrap().map.lightmap, m.lightmap);
        let text = std::fs::read_to_string(&path).unwrap();
        assert_eq!(text, to_string_with_lightmap(&m), "the file is written as it is made, in the same layout");
        let saved: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(saved["lightmap"]["stale"], serde_json::json!([2]), "node 2 changed since its fingerprint was taken");
        std::fs::remove_dir_all(&dir).ok();

        let lm = m.lightmap.as_ref().unwrap();
        let mut fresh = (**lm).clone();
        let print = m.get(NodeId(2)).and_then(|n| crate::lightmap::fingerprint(&n.kind));
        fresh.nodes.insert(2, print.expect("node 2 is geometry"));
        assert!(fresh.stale_nodes(&m).is_empty());
        let mut gone = m.clone();
        gone.remove(NodeId(2));
        assert_eq!(fresh.stale_nodes(&gone), vec![2], "a deleted node is stale too");
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
    fn labels_round_trip_in_both_layouts() {
        let mut m = sample();
        let layer = m.default_layer();
        let brush = m.get(layer).unwrap().children[0];
        assert!(m.rename(brush, " north wall "));
        assert_eq!(m.get(brush).unwrap().name(), "north wall");
        let value = to_value(&m);
        assert_eq!(value["layers"][0]["children"][0]["label"], "north wall");
        assert!(value["layers"][0].get("label").is_none(), "no key without a label");

        let from_text = from_str(&to_string(&m)).unwrap();
        let from_binary = from_bytes(&to_bytes(&m).unwrap()).unwrap().map;
        for back in [from_text, from_binary] {
            assert_eq!(back.get(brush).unwrap().label.as_deref(), Some("north wall"));
            assert_eq!(back.get(layer).unwrap().label, None);
        }

        assert!(m.rename(brush, ""));
        assert_eq!(m.get(brush).unwrap().name(), format!("brush{}", brush.0), "an empty name brings the default back");
        assert!(!m.rename(layer, ""), "a layer keeps its name");
    }

    #[test]
    fn graph_positions_round_trip_in_both_layouts() {
        let mut m = sample();
        let layer = m.default_layer();
        let brush = m.get(layer).unwrap().children[0];
        m.get_mut(brush).unwrap().set_graph(Some([120.4, -36.6]));
        let value = to_value(&m);
        assert_eq!(value["layers"][0]["children"][0]["graph"], serde_json::json!([120.0, -37.0]), "rounded to whole points");
        assert!(value["layers"][0].get("graph").is_none(), "no key without a position");

        let from_text = from_str(&to_string(&m)).unwrap();
        let from_binary = from_bytes(&to_bytes(&m).unwrap()).unwrap().map;
        for back in [from_text, from_binary] {
            assert_eq!(back.get(brush).unwrap().graph, Some([120.0, -37.0]));
            assert_eq!(back.get(layer).unwrap().graph, None);
        }

        let mut value = to_value(&m);
        value["layers"][0]["children"][0]["graph"] = serde_json::json!("left");
        let back = from_str(&value.to_string()).expect("a damaged position does not refuse the map");
        assert_eq!(back.get(brush).unwrap().graph, None);
        value["layers"][0]["children"][0].as_object_mut().unwrap().remove("graph");
        assert_eq!(from_str(&value.to_string()).unwrap().get(brush).unwrap().graph, None, "files from before the key read as before");
        m.get_mut(brush).unwrap().set_graph(Some([f32::NAN, 1.0]));
        assert_eq!(m.get(brush).unwrap().graph, None, "never written as null");
    }

    #[test]
    fn paste_keeps_graph_positions_beside_the_originals() {
        let mut m = sample();
        let layer = m.default_layer();
        let brush = m.get(layer).unwrap().children[0];
        m.get_mut(brush).unwrap().set_graph(Some([120.0, -40.0]));
        let text = nodes_to_string(&m, &[brush]);
        let pasted = paste_nodes(&mut m, layer, &text).unwrap();
        let o = PASTE_GRAPH_OFFSET;
        assert_eq!(m.get(pasted[0]).unwrap().graph, Some([120.0 + o, -40.0 + o]));
        assert_eq!(m.get(brush).unwrap().graph, Some([120.0, -40.0]));
        m.get_mut(brush).unwrap().set_graph(None);
        let text = nodes_to_string(&m, &[brush]);
        let pasted = paste_nodes(&mut m, layer, &text).unwrap();
        assert_eq!(m.get(pasted[0]).unwrap().graph, None, "nodes the automatic layout places stay that way");
    }

    #[test]
    fn graph_frames_round_trip_in_both_layouts() {
        let mut m = sample();
        let frame = crate::map::GraphFrame { title: "Door logic".into(), rect: [10.0, 20.0, 300.0, 200.0], color: td_core::Color::rgb(0.2, 0.4, 0.8) };
        m.editor.graph_frames.push(frame.clone());
        let value = to_value(&m);
        assert_eq!(value["editor"]["graph_frames"][0]["title"], "Door logic");
        for back in [from_str(&to_string(&m)).unwrap(), from_bytes(&to_bytes(&m).unwrap()).unwrap().map] {
            assert_eq!(back.editor.graph_frames, vec![frame.clone()]);
        }

        let mut value = to_value(&m);
        value["editor"]["graph_frames"] = serde_json::json!([{ "title": 3 }, value["editor"]["graph_frames"][0].clone()]);
        let back = from_str(&value.to_string()).expect("a damaged frame does not refuse the map");
        assert_eq!(back.editor.graph_frames, vec![frame], "only the damaged frame is dropped");
        assert!(to_value(&sample())["editor"].get("graph_frames").is_none(), "no key without frames");
    }

    #[test]
    fn hand_edited_labels_that_name_nothing_are_dropped() {
        let m = sample();
        let layer = m.default_layer();
        let brush = m.get(layer).unwrap().children[0];
        let mut value = to_value(&m);
        value["layers"][0]["label"] = "walls".into();
        value["layers"][0]["children"][0]["label"] = "  ".into();
        let back = from_str(&value.to_string()).unwrap();
        assert_eq!((back.get(layer).unwrap().label.as_deref(), back.get(brush).unwrap().label.as_deref()), (None, None));
        assert_eq!(back.get(brush).unwrap().name(), format!("brush{}", brush.0));
    }

    #[test]
    fn converting_to_binary_keeps_the_content_and_refuses_what_cannot_open() {
        let m = sample();
        let (bytes, problems) = file_to_binary(to_string(&m).as_bytes()).unwrap();
        assert!(problems.is_empty());
        assert_eq!(bytes, to_bytes(&m).unwrap());
        assert_eq!(file_to_binary(&bytes).unwrap().0, bytes);

        let mut v = to_value(&m);
        v["layers"][0]["children"][0]["faces"][0]["indices"][0] = serde_json::json!(99);
        assert!(matches!(file_to_binary(v.to_string().as_bytes()), Err(FormatError::Invalid { .. })));
        v["format"] = "something-else".into();
        assert!(matches!(file_to_binary(v.to_string().as_bytes()), Err(FormatError::WrongFormat(_))));
    }

    #[test]
    fn hand_edited_node_keys_read_as_they_always_have() {
        let layer = |extra: Value| {
            let mut v = to_value(&sample());
            v["layers"][0]["children"][0].as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
            v
        };
        let read = |v: &Value| from_str(&v.to_string());
        let back = read(&layer(serde_json::json!({ "label": null, "hidden": true, "graph": "left", "unknown": [1, 2] }))).unwrap();
        let brush = back.get(back.default_layer()).unwrap().children[0];
        let node = back.get(brush).unwrap();
        assert!(node.hidden && node.label.is_none() && node.graph.is_none());
        assert!(matches!(back.get(brush).unwrap().kind, NodeKind::Brush(_)));

        for (extra, error) in [
            (serde_json::json!({ "children": 3 }), "invalid type: integer `3`, expected a sequence"),
            (serde_json::json!({ "hidden": "yes" }), "invalid type: string \"yes\", expected a boolean"),
            (serde_json::json!({ "children": ["x"] }), "invalid type: string \"x\", expected struct FileNode"),
            (serde_json::json!({ "type": "box" }), "unknown variant `box`"),
        ] {
            let err = read(&layer(extra)).unwrap_err().to_string();
            assert!(err.contains(error), "{err}");
        }

        let mut v = to_value(&sample());
        v["layers"][0]["children"][0].as_object_mut().unwrap().remove("id");
        assert!(read(&v).unwrap_err().to_string().contains("missing field `id`"));
        v.as_object_mut().unwrap().remove("layers");
        assert!(read(&v).unwrap_err().to_string().contains("missing field `layers`"));
        let clip = serde_json::json!({ "format": "trenchdot-clipboard", "version": 1 });
        assert!(paste_nodes(&mut sample(), sample().default_layer(), &clip.to_string()).unwrap_err().to_string().contains("missing field `nodes`"));
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
    fn terrain_cell_sizes_out_of_range_are_an_error() {
        let mut m = Map::new();
        let layer = m.default_layer();
        let t = m.insert(layer, NodeKind::Terrain(td_geom::Terrain::new(DVec3::ZERO, [3, 3], 32.0, "base/floor")));
        for size in [1e308, 1e-300] {
            let mut v = to_value(&m);
            v["layers"][0]["children"][0]["cell_size"] = serde_json::json!(size);
            assert!(matches!(from_str(&v.to_string()), Err(FormatError::Invalid { .. })), "{size}");
            assert!(matches!(from_bytes(&crate::binary::encode(v.clone(), &[]).unwrap()), Err(FormatError::Invalid { .. })), "{size}");

            let mut clip: Value = serde_json::from_str(&nodes_to_string(&m, &[t])).unwrap();
            clip["nodes"][0]["cell_size"] = serde_json::json!(size);
            assert!(matches!(paste_nodes(&mut m.clone(), layer, &clip.to_string()), Err(FormatError::Invalid { .. })), "{size}");
        }
    }

    #[test]
    fn repaired_ids_do_not_take_later_ids() {
        let mut v = to_value(&Map::new());
        let brush = |id: u64| {
            let mut b = serde_json::to_value(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(8.0)), "m").unwrap()).unwrap();
            b["id"] = id.into();
            b["type"] = "brush".into();
            b
        };
        let scatter = serde_json::json!({ "id": 7, "type": "scatter", "name": "s", "kind": "props", "items": [], "targets": [6] });
        v["layers"][0]["children"] = serde_json::json!([brush(5), brush(5), brush(6), scatter]);
        let m = from_str(&v.to_string()).unwrap();
        let ids: Vec<u64> = m.get(m.default_layer()).unwrap().children.iter().map(|c| c.0).collect();
        assert_eq!(ids, [5, 8, 6, 7], "the duplicate gets an id past the file's highest");
        assert_eq!(m.scatter(NodeId(7)).unwrap().targets, [NodeId(6)]);
        assert_eq!(m.next_id, 9);
    }

    #[test]
    fn paste_refuses_a_missing_parent() {
        let mut m = sample();
        let layer = m.default_layer();
        let text = nodes_to_string(&m, &m.get(layer).unwrap().children.clone());
        let before = m.nodes.len();
        assert!(matches!(paste_nodes(&mut m, NodeId(9999), &text), Err(FormatError::MissingParent(9999))));
        assert_eq!(m.nodes.len(), before);
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
    fn scatter_box_and_decomposition_collision_round_trip() {
        let m = Map::new();
        let layer = m.default_layer();
        let mut v = to_value(&m);
        for (name, collision) in [("box", crate::scatter::ScatterCollision::Box), ("decomposition", crate::scatter::ScatterCollision::Decomposition)] {
            v["layers"][0]["children"] = serde_json::json!([{ "id": 50, "type": "scatter", "name": "s", "kind": "props", "items": [], "collision": name }]);
            let back = from_str(&v.to_string()).unwrap();
            let id = back.get(layer).unwrap().children[0];
            assert_eq!(back.scatter(id).unwrap().collision, collision);
            assert_eq!(to_value(&back)["layers"][0]["children"][0]["collision"], name);
        }
    }

    #[test]
    fn scatter_missing_settings_match_a_new_set() {
        let m = Map::new();
        let layer = m.default_layer();
        let mut v = to_value(&m);
        for kind in ScatterKind::ALL {
            v["layers"][0]["children"] = serde_json::json!([{ "id": 50, "type": "scatter", "name": "s", "kind": kind.label(), "items": [] }]);
            let back = from_str(&v.to_string()).unwrap();
            let loaded = back.scatter(back.get(layer).unwrap().children[0]).unwrap();
            assert_eq!(*loaded, Scatter::new("s", kind, Vec::new()), "{kind:?}");
        }
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

    #[test]
    fn saving_replaces_a_link_at_the_temp_path_instead_of_writing_through_it() {
        let dir = std::env::temp_dir().join(format!("td_save_link_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (path, other) = (dir.join("a.tdm"), dir.join("other.txt"));
        std::fs::write(&other, "keep").unwrap();
        // A hard link stands in for a symbolic one, which Windows only lets an administrator make.
        std::fs::hard_link(&other, path.with_extension("tdm.tmp")).unwrap();

        save(&sample(), &path).unwrap();
        assert_eq!(std::fs::read_to_string(&other).unwrap(), "keep");
        assert_eq!(load(&path).unwrap().map.nodes.len(), sample().nodes.len());
        std::fs::write(path.with_extension("tdm.tmp"), "left by a crash").unwrap();
        save(&sample(), &path).unwrap();
        assert!(!path.with_extension("tdm.tmp").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
