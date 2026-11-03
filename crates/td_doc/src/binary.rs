//! The chunked, zstd compressed container `.tdm` maps are saved in. It converts between file bytes and the same JSON
//! value tree `format.rs` reads and writes, and recovers what it can from a damaged file. See docs/format/container.md.

use std::sync::Arc;

use serde_json::{Map, Value, json};

use crate::variant::{self, Writer};

pub const MAGIC: [u8; 8] = *b"\x89GTM\r\n\x1a\n";
pub const CONTAINER_VERSION: u32 = 1;
const SYNC: [u8; 4] = *b"\x89GTC";
const HEADER_LEN: usize = 24;
const CHECK_SEED: u32 = 0x4754_4d43;
pub const FLAG_ZSTD: u32 = 1;
const BATCH_BYTES: usize = 64 * 1024;
/// A chunk header that claims more is treated as damaged rather than allocated.
const MAX_RAW: u32 = 1 << 30;
/// What the HEAD, NODE, END and LMAP chunks of a file may unpack to together. Other chunks are never unpacked.
const MAX_FILE_RAW: usize = 1 << 30;
/// Past 15 zstd gets several times slower for a few percent, and autosave writes big maps every few minutes.
const LEVEL: i32 = 15;
/// Baked light maps run to tens of megabytes, where the higher levels take seconds.
const BULK_LEVEL: i32 = 3;

pub const HEAD: [u8; 4] = *b"HEAD";
pub const NODE: [u8; 4] = *b"NODE";
pub const END: [u8; 4] = *b"END ";
const LMAP: [u8; 4] = crate::lightmap::TAG;

pub const RECOVERED_LAYER: &str = "Recovered";
/// About as deep as JSON maps can nest before serde_json's recursion limit refuses them.
const MAX_TREE_DEPTH: usize = 60;

/// A chunk this version does not know, kept as stored so saving writes it back unchanged.
#[derive(Clone, Debug, PartialEq)]
pub struct RawChunk {
    pub tag: [u8; 4],
    pub flags: u32,
    pub raw_size: u32,
    pub data: Arc<[u8]>,
}

pub struct Decoded {
    pub value: Value,
    /// The chunks other than HEAD, NODE and END as they are stored, LMAP included, to write them back unchanged.
    pub unknown: Vec<RawChunk>,
    /// What was lost or moved, empty for an intact file.
    pub problems: Vec<String>,
    /// The value of the first LMAP chunk that could be read.
    pub lightmap: Option<Value>,
}

#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    #[error("the file was saved or transferred as text and its line endings were changed, restore it from version control or a backup")]
    Mangled,
    #[error("container version {0} is newer than this editor supports ({CONTAINER_VERSION})")]
    TooNew(u32),
    #[error("the map header is damaged and the file has no intact end chunk to take the map version from")]
    NoHeader,
    #[error("the map is too big to save, its file would unpack to {0} MiB and a map file may unpack to {max} MiB", max = MAX_FILE_RAW >> 20)]
    TooLarge(usize),
    #[error("the map is too big to save, reading it back would take {0} MiB of memory and a map file may take {max} MiB", max = variant::MAX_MEMORY >> 20)]
    TooMuchMemory(usize),
}

pub fn is_binary(bytes: &[u8]) -> bool {
    bytes.starts_with(&MAGIC[..4])
}

/// A 64 bit FNV-1a hash over the raw HEAD and NODE payloads, stored in the END chunk. Godot keeps it with a scene
/// built from the file, so a live session can tell the editor's map is what the scene was built from.
pub struct ContentId(u64);

impl ContentId {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }

    fn add(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = (self.0 ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3);
        }
    }

    /// The low 52 bits, which survive the live link's JSON where Godot parses every number as a double.
    pub fn as_i64(&self) -> i64 {
        (self.0 & ((1 << 52) - 1)) as i64
    }
}

/// Raw payloads of the HEAD and NODE chunks for a map value as `format::to_value` makes it.
fn payloads(value: &Value) -> (Vec<u8>, Vec<Vec<u8>>, usize) {
    let mut w = Writer::new();
    let head = value.as_object().into_iter().flatten().filter(|(k, _)| k.as_str() != "layers");
    w.dict_header(head.clone().count());
    for (k, v) in head {
        w.key(k);
        w.value(v);
    }

    let head = w.out;

    let mut flat = Vec::new();
    for layer in value.get("layers").and_then(Value::as_array).into_iter().flatten() {
        flatten(layer, 0, &mut flat);
    }

    let mut batches = Vec::new();
    let mut parents = Vec::new();
    let mut encoded = Vec::new();
    let mut size = 0;
    let mut flush = |parents: &mut Vec<i64>, encoded: &mut Vec<Vec<u8>>, size: &mut usize| {
        if encoded.is_empty() {
            return;
        }

        let mut w = Writer { out: Vec::with_capacity(64 + 8 * parents.len() + encoded.iter().map(Vec::len).sum::<usize>()) };
        w.dict_header(2);
        w.key("parents");
        w.int64s(parents);
        w.key("nodes");
        w.array_header(encoded.len());
        encoded.iter().for_each(|n| w.out.extend_from_slice(n));
        batches.push(w.out);
        parents.clear();
        encoded.clear();
        *size = 0;
    };
    for (parent, node) in &flat {
        let mut w = Writer::new();
        w.node(node);
        size += w.out.len();
        parents.push(*parent);
        encoded.push(w.out);
        if size >= BATCH_BYTES {
            flush(&mut parents, &mut encoded, &mut size);
        }
    }

    flush(&mut parents, &mut encoded, &mut size);
    (head, batches, flat.len())
}

/// Nodes in tree order, each with its parent id, 0 for the top level.
fn flatten<'a>(node: &'a Value, parent: i64, out: &mut Vec<(i64, &'a Map<String, Value>)>) {
    let Some(obj) = node.as_object() else { return };
    let id = obj.get("id").and_then(Value::as_i64).unwrap_or(0);
    out.push((parent, obj));
    for child in obj.get("children").and_then(Value::as_array).into_iter().flatten() {
        flatten(child, id, out);
    }
}

fn id_of(head: &[u8], batches: &[Vec<u8>]) -> ContentId {
    let mut id = ContentId::new();
    id.add(head);
    batches.iter().for_each(|b| id.add(b));
    id
}

pub fn content_id(value: &Value) -> ContentId {
    let (head, batches, _) = payloads(value);
    id_of(&head, &batches)
}

/// The file for a map value as `format::to_value` makes it, with the `unknown` chunks written back as they are.
/// Refuses a map that the reader would not read in full. The value is dropped once it is written out, and each stage
/// is freed as the next is made, so a big map is not held several times over.
pub fn encode(value: Value, unknown: &[RawChunk]) -> Result<Vec<u8>, ContainerError> {
    let (head, batches, node_count) = payloads(&value);
    let version = value.get("version").cloned().unwrap_or(Value::from(0));
    drop(value);
    let id = id_of(&head, &batches);
    let raw: Vec<&[u8]> = std::iter::once(head.as_slice()).chain(batches.iter().map(Vec::as_slice)).collect();
    let end = json!({ "version": version, "nodes": node_count, "chunks": raw.len() + unknown.len(), "content": id.as_i64() });
    let mut w = Writer::new();
    w.value(&end);
    check_fits(&raw, unknown, &w.out)?;

    let sizes: Vec<u32> = raw.iter().map(|r| r.len() as u32).collect();
    let compressed = compress_all(&raw);
    drop(raw);
    drop((head, batches));
    let mut out = Vec::with_capacity(compressed.iter().map(Vec::len).sum::<usize>() + 64);
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&CONTAINER_VERSION.to_le_bytes());
    for (i, data) in compressed.into_iter().enumerate() {
        write_chunk(&mut out, if i == 0 { HEAD } else { NODE }, FLAG_ZSTD, sizes[i], &data);
    }

    for chunk in unknown {
        write_chunk(&mut out, chunk.tag, chunk.flags, chunk.raw_size, &chunk.data);
    }

    write_chunk(&mut out, END, FLAG_ZSTD, w.out.len() as u32, &compress(&w.out));
    Ok(out)
}

/// Refuses a file holding the HEAD and NODE `payloads`, then the `unknown` chunks, then the END payload `end`, when
/// `decode` would not read all of it.
fn check_fits(payloads: &[&[u8]], unknown: &[RawChunk], end: &[u8]) -> Result<(), ContainerError> {
    let (mut raw, mut taken, mut peak) = (0usize, 0usize, 0usize);
    let mut read = |len: usize, payload: Option<&[u8]>| {
        raw = raw.saturating_add(len);
        if let Some(payload) = payload {
            let cost = variant::cost(payload).unwrap_or(usize::MAX);
            peak = peak.max(taken.saturating_add(len).saturating_add(cost));
            taken = taken.saturating_add(cost);
        }
    };
    payloads.iter().for_each(|p| read(p.len(), Some(p)));
    // A light map that does not unpack is dropped by the reader whether or not it is saved again.
    for chunk in unknown.iter().filter(|c| c.tag == LMAP) {
        read(chunk.raw_size as usize, chunk_payload(chunk).as_deref());
    }

    read(end.len(), Some(end));
    let mib = |n: usize| n.div_ceil(1 << 20);
    if raw > MAX_FILE_RAW {
        return Err(ContainerError::TooLarge(mib(raw)));
    }

    if peak > variant::MAX_MEMORY {
        return Err(ContainerError::TooMuchMemory(mib(peak)));
    }

    Ok(())
}

/// A chunk holding `raw`, compressed at a level that keeps saving quick for payloads of many megabytes.
pub fn bulk_chunk(tag: [u8; 4], raw: &[u8]) -> RawChunk {
    let data = compress_at(raw, BULK_LEVEL);
    RawChunk { tag, flags: FLAG_ZSTD, raw_size: raw.len() as u32, data: data.into() }
}

/// The uncompressed payload of a chunk, None when it is damaged.
pub fn chunk_payload(chunk: &RawChunk) -> Option<Vec<u8>> {
    let h = Header { tag: chunk.tag, flags: chunk.flags, raw: chunk.raw_size, stored: chunk.data.len() };
    payload(&h, &chunk.data)
}

fn compress(raw: &[u8]) -> Vec<u8> {
    compress_at(raw, LEVEL)
}

fn compress_at(raw: &[u8], level: i32) -> Vec<u8> {
    let mut c = zstd::bulk::Compressor::new(level).expect("zstd level is valid");
    c.include_checksum(true).expect("zstd accepts the checksum flag");
    c.compress(raw).expect("zstd compresses into a growing buffer")
}

/// Chunks are independent zstd frames, so they compress in parallel.
fn compress_all(raw: &[&[u8]]) -> Vec<Vec<u8>> {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get()).min(raw.len().max(1));
    let per = raw.len().div_ceil(threads).max(1);
    std::thread::scope(|s| {
        let handles: Vec<_> = raw.chunks(per).map(|group| s.spawn(move || group.iter().map(|r| compress(r)).collect::<Vec<_>>())).collect();
        handles.into_iter().flat_map(|h| h.join().expect("compression thread")).collect()
    })
}

fn header_check(tag: [u8; 4], flags: u32, raw: u32, stored: u32) -> u32 {
    u32::from_le_bytes(tag) ^ flags ^ raw ^ stored ^ CHECK_SEED
}

fn write_chunk(out: &mut Vec<u8>, tag: [u8; 4], flags: u32, raw: u32, data: &[u8]) {
    let stored = data.len() as u32;
    out.extend_from_slice(&SYNC);
    out.extend_from_slice(&tag);
    for v in [flags, raw, stored, header_check(tag, flags, raw, stored)] {
        out.extend_from_slice(&v.to_le_bytes());
    }

    out.extend_from_slice(data);
}

fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap_or_default())
}

struct Header {
    tag: [u8; 4],
    flags: u32,
    raw: u32,
    stored: usize,
}

fn header_at(bytes: &[u8], at: usize) -> Option<Header> {
    if at + HEADER_LEN > bytes.len() || bytes[at..at + 4] != SYNC {
        return None;
    }

    let tag: [u8; 4] = bytes[at + 4..at + 8].try_into().ok()?;
    let (flags, raw, stored) = (u32_at(bytes, at + 8), u32_at(bytes, at + 12), u32_at(bytes, at + 16));
    let valid = header_check(tag, flags, raw, stored) == u32_at(bytes, at + 20) && raw <= MAX_RAW && tag.iter().all(|c| c.is_ascii_graphic() || *c == b' ');
    valid.then_some(Header { tag, flags, raw, stored: stored as usize })
}

fn next_sync(bytes: &[u8], from: usize) -> Option<usize> {
    let from = from.min(bytes.len());
    bytes[from..].windows(SYNC.len()).position(|w| w == SYNC).map(|p| p + from)
}

fn payload(h: &Header, data: &[u8]) -> Option<Vec<u8>> {
    if h.flags & FLAG_ZSTD == 0 {
        return (data.len() == h.raw as usize).then(|| data.to_vec());
    }

    zstd::bulk::decompress(data, h.raw as usize).ok().filter(|raw| raw.len() == h.raw as usize)
}

fn tag_name(tag: [u8; 4]) -> String {
    String::from_utf8_lossy(&tag).trim_end().to_string()
}

pub fn decode(bytes: &[u8]) -> Result<Decoded, ContainerError> {
    decode_with(bytes, MAX_FILE_RAW, variant::MAX_MEMORY)
}

/// Reads the HEAD, NODE, END and LMAP chunks in file order. A chunk is unpacked when its raw size fits in what is left
/// of `raw_left` and of `memory_left`, and read when its raw size and its values fit in what is left of `memory_left`.
/// Only the values are taken from `memory_left`, a chunk that is not read takes nothing from it.
fn decode_with(bytes: &[u8], mut raw_left: usize, mut memory_left: usize) -> Result<Decoded, ContainerError> {
    if bytes.starts_with(b"\x89GTM\n") || bytes.starts_with(b"\x89GTM\r\r\n") {
        return Err(ContainerError::Mangled);
    }

    let version = if bytes.len() >= 12 { u32_at(bytes, 8) } else { CONTAINER_VERSION };
    if version > CONTAINER_VERSION {
        return Err(ContainerError::TooNew(version));
    }

    let mut problems = Vec::new();
    let mut head: Option<Map<String, Value>> = None;
    let mut end: Option<Value> = None;
    let mut unknown = Vec::new();
    let mut tree = Tree::default();
    let mut chunks = 0;
    let mut damaged = 0;
    let mut lightmap = None;
    let mut pos = MAGIC.len() + 4;
    while pos < bytes.len() {
        let Some(h) = header_at(bytes, pos) else {
            let next = next_sync(bytes, pos + 1).unwrap_or(bytes.len());
            problems.push(format!("bytes {pos} to {next} are damaged and were skipped"));
            damaged += 1;
            pos = next;
            continue;
        };
        let start = pos + HEADER_LEN;
        let Some(data) = bytes.get(start..start + h.stored) else {
            problems.push(format!("the file ends inside the {} chunk at byte {pos}", tag_name(h.tag)));
            damaged += 1;
            pos = next_sync(bytes, start).unwrap_or(bytes.len());
            continue;
        };
        let at = pos;
        pos = start + h.stored;
        chunks += 1;
        if !matches!(h.tag, HEAD | NODE | END | LMAP) {
            unknown.push(RawChunk { tag: h.tag, flags: h.flags, raw_size: h.raw, data: data.into() });
            continue;
        }

        let lost = match h.tag {
            NODE => ", the nodes stored in it are lost",
            HEAD => ", the worldspawn properties and editor state are lost",
            LMAP => ", the baked lighting is lost, bake it again",
            _ => "",
        };
        let tag = tag_name(h.tag);
        let Some(left) = raw_left.checked_sub(h.raw as usize) else {
            problems.push(format!("the {tag} chunk at byte {at} would unpack past the {} MiB a map may hold and was not read{lost}", MAX_FILE_RAW >> 20));
            damaged += 1;
            continue;
        };
        raw_left = left;
        if h.tag == LMAP {
            unknown.push(RawChunk { tag: h.tag, flags: h.flags, raw_size: h.raw, data: data.into() });
        }

        let read = if h.raw as usize > memory_left {
            Err(variant::Refused::TooBig)
        } else {
            payload(&h, data).ok_or(variant::Refused::Damaged("payload does not unpack"))
        };
        let value = match read.and_then(|raw| variant::decode_within(&raw, &mut memory_left)) {
            Ok(value) => value,
            Err(refused) => {
                let why = if refused == variant::Refused::TooBig { "takes more memory than a map may and was not read" } else { "is damaged" };
                problems.push(format!("the {tag} chunk at byte {at} {why}{lost}"));
                damaged += 1;
                continue;
            }
        };
        match h.tag {
            HEAD => head = if let Value::Object(map) = value { Some(map) } else { None },
            NODE => tree.add_batch(value),
            END => end = Some(value),
            _ => lightmap = lightmap.or(Some(value)),
        }
    }

    let mut head = match (head, &end) {
        (Some(h), _) => h,
        (None, Some(end)) => {
            let mut h = Map::new();
            h.insert("format".into(), Value::from(crate::format::FORMAT_NAME));
            h.insert("version".into(), end.get("version").cloned().unwrap_or(Value::from(0)));
            h
        }
        (None, None) => return Err(ContainerError::NoHeader),
    };

    match &end {
        None if damaged == 0 => problems.push("the file ends early, anything saved after the last complete chunk is lost".into()),
        None => problems.push("the end chunk is missing, so the number of lost nodes is unknown".into()),
        Some(end) => {
            let expected = end.get("nodes").and_then(Value::as_u64).unwrap_or(0) as usize;
            if expected > tree.nodes.len() {
                problems.push(format!("{} of {expected} nodes could not be read", expected - tree.nodes.len()));
            }

            let expected_chunks = end.get("chunks").and_then(Value::as_u64).unwrap_or(0) as usize;
            if damaged == 0 && expected_chunks > chunks - 1 {
                problems.push(format!("{} chunks are missing", expected_chunks - (chunks - 1)));
            }
        }
    }

    let (layers, orphans) = tree.build();
    if orphans > 0 {
        problems.push(format!("{orphans} nodes whose parent was lost were moved to the layer \"{RECOVERED_LAYER}\""));
    }

    head.insert("layers".into(), Value::Array(layers));
    Ok(Decoded { value: Value::Object(head), unknown, problems, lightmap })
}

#[derive(Default)]
struct Tree {
    nodes: Vec<Map<String, Value>>,
    parents: Vec<i64>,
}

impl Tree {
    fn add_batch(&mut self, batch: Value) {
        let Value::Object(mut batch) = batch else { return };
        let Some(Value::Array(parents)) = batch.remove("parents") else { return };
        let Some(Value::Array(nodes)) = batch.remove("nodes") else { return };
        for (node, parent) in nodes.into_iter().zip(parents.iter().map(|v| v.as_i64().unwrap_or(0))) {
            if let Value::Object(mut node) = node {
                variant::restore_node(&mut node);
                self.nodes.push(node);
                self.parents.push(parent);
            }
        }
    }

    /// The top level nodes with their subtrees, the orphans in a `Recovered` layer at the end, and how many orphans
    /// there were. A parent id refers to the latest node with that id read so far, as ids are only repeated in hand
    /// edited files. Nesting past `MAX_TREE_DEPTH`, which only a crafted file has, is cut into orphans so that
    /// building the map cannot run out of stack.
    fn build(self) -> (Vec<Value>, usize) {
        let mut index: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); self.nodes.len()];
        let mut depth = vec![0; self.nodes.len()];
        let mut roots = Vec::new();
        let mut orphans = Vec::new();
        for (i, node) in self.nodes.iter().enumerate() {
            match self.parents[i] {
                0 => roots.push(i),
                p => match index.get(&p) {
                    Some(&at) if depth[at] < MAX_TREE_DEPTH => {
                        children[at].push(i);
                        depth[i] = depth[at] + 1;
                    }
                    _ => {
                        orphans.push(i);
                        depth[i] = 1;
                    }
                },
            }

            index.insert(node.get("id").and_then(Value::as_i64).unwrap_or(0), i);
        }

        let mut slots: Vec<Option<Map<String, Value>>> = self.nodes.into_iter().map(Some).collect();
        let mut out: Vec<Value> = roots.iter().map(|&i| assemble(i, &mut slots, &children)).collect();
        if !orphans.is_empty() {
            let kids: Vec<Value> = orphans.iter().map(|&i| assemble(i, &mut slots, &children)).collect();
            out.push(json!({ "id": 0, "type": "layer", "name": RECOVERED_LAYER, "color": "#e0a030ff", "omit_from_export": false, "children": kids }));
        }

        (out, orphans.len())
    }
}

fn assemble(i: usize, slots: &mut [Option<Map<String, Value>>], children: &[Vec<usize>]) -> Value {
    let mut node = slots[i].take().unwrap_or_default();
    if !children[i].is_empty() {
        let kids = children[i].iter().map(|&c| assemble(c, slots, children)).collect();
        node.insert("children".into(), Value::Array(kids));
    }

    Value::Object(node)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format;
    use crate::lightmap::{Chart, Lightmap, Probes};
    use crate::map::{Group, Map as DocMap, NodeKind};
    use std::collections::BTreeSet;
    use td_core::{Aabb, DVec3};
    use td_geom::{Brush, Terrain};

    fn boxb(x: f64) -> Brush {
        Brush::from_aabb(&Aabb::new(DVec3::new(x, 0.0, 0.0), DVec3::new(x + 7.3, 16.0, 16.0)), "base/wall").unwrap()
    }

    /// Two groups of brushes big enough to span several NODE chunks, and a terrain.
    fn big_map() -> DocMap {
        let mut m = DocMap::new();
        m.properties.insert("message".into(), "big".into());
        let layer = m.default_layer();
        for name in ["a", "b"] {
            let g = m.insert(layer, NodeKind::Group(Group::new(name)));
            for i in 0..120 {
                m.insert(g, NodeKind::Brush(boxb(i as f64 * 9.1)));
            }
        }

        m.insert(layer, NodeKind::Terrain(Terrain::new(DVec3::ZERO, [9, 9], 32.0, "base/floor")));
        m
    }

    /// Start offset and header of every chunk in an intact file.
    fn chunks(bytes: &[u8]) -> Vec<(usize, Header)> {
        let mut out = Vec::new();
        let mut pos = MAGIC.len() + 4;
        while let Some(h) = header_at(bytes, pos) {
            let next = pos + HEADER_LEN + h.stored;
            out.push((pos, h));
            pos = next;
        }

        assert_eq!(pos, bytes.len());
        out
    }

    fn chunk_ids(bytes: &[u8], at: usize, h: &Header) -> Vec<i64> {
        let raw = payload(h, &bytes[at + HEADER_LEN..at + HEADER_LEN + h.stored]).unwrap();
        let v = variant::decode(&raw).unwrap();
        v["nodes"].as_array().unwrap().iter().map(|n| n["id"].as_i64().unwrap()).collect()
    }

    fn ids(n: &Value, out: &mut BTreeSet<i64>) {
        out.insert(n["id"].as_i64().unwrap());
        n.get("children").and_then(Value::as_array).into_iter().flatten().for_each(|c| ids(c, out));
    }

    fn all_ids(v: &Value) -> BTreeSet<i64> {
        let mut out = BTreeSet::new();
        v["layers"].as_array().unwrap().iter().for_each(|l| ids(l, &mut out));
        out
    }

    #[test]
    fn intact_file_round_trips_and_is_small() {
        let m = big_map();
        let value = format::to_value(&m);
        let bytes = format::to_bytes(&m).unwrap();
        assert!(bytes.starts_with(&MAGIC));
        let tags: Vec<[u8; 4]> = chunks(&bytes).iter().map(|(_, h)| h.tag).collect();
        assert_eq!(tags.first(), Some(&HEAD));
        assert_eq!(tags.last(), Some(&END));
        assert!(tags.iter().filter(|t| **t == NODE).count() >= 3, "{tags:?}");

        let back = decode(&bytes).unwrap();
        assert!(back.problems.is_empty(), "{:?}", back.problems);
        assert_eq!(back.value, value);
        assert!(bytes.len() * 10 < format::to_string(&m).len(), "{} bytes", bytes.len());
        assert_eq!(format::to_bytes(&format::from_bytes(&bytes).unwrap().map).unwrap(), bytes, "saving is deterministic");

        let (at, end) = chunks(&bytes).pop().unwrap();
        let end = variant::decode(&payload(&end, &bytes[at + HEADER_LEN..]).unwrap()).unwrap();
        assert_eq!(end["content"].as_i64(), Some(format::content_id(&m)));
        assert_eq!(end["nodes"].as_u64(), Some(m.nodes.len() as u64));
    }

    #[test]
    fn committed_maps_round_trip() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot");
        let mut seen = 0;
        for dir in ["demo/maps", "demo/maps/showcase", "tests/maps"] {
            for entry in std::fs::read_dir(root.join(dir)).unwrap().flatten() {
                let path = entry.path();
                if path.extension().is_none_or(|e| e != "tdm") {
                    continue;
                }

                let bytes = std::fs::read(&path).unwrap();
                let (text, problems) = format::file_to_json(&bytes).unwrap();
                assert!(problems.is_empty(), "{}: {problems:?}", path.display());
                let value: Value = serde_json::from_str(&text).unwrap();
                let binary = encode(value.clone(), &[]).unwrap();
                assert_eq!(decode(&binary).unwrap().value, value, "{}", path.display());

                let from_json = format::from_str(&text).unwrap();
                let from_binary = format::from_bytes(&binary).unwrap();
                assert!(from_binary.problems.is_empty());
                assert_eq!(format::to_value(&from_binary.map), format::to_value(&from_json), "{}", path.display());
                seen += 1;
            }
        }

        assert!(seen >= 10, "{seen} maps");
    }

    #[test]
    fn a_damaged_node_chunk_loses_only_its_nodes() {
        let m = big_map();
        let bytes = format::to_bytes(&m).unwrap();
        let list = chunks(&bytes);
        let group_b = m.nodes.values().find(|n| matches!(&n.kind, NodeKind::Group(g) if g.name == "b")).unwrap().id.0 as i64;
        let (k, (at, h)) = list.iter().enumerate().find(|(_, (at, h))| h.tag == NODE && chunk_ids(&bytes, *at, h).contains(&group_b)).unwrap();
        let lost: BTreeSet<i64> = chunk_ids(&bytes, *at, h).into_iter().collect();
        let later: Vec<i64> = list[k + 1..].iter().filter(|(_, h)| h.tag == NODE).flat_map(|(at, h)| chunk_ids(&bytes, *at, h)).collect();
        let orphans: BTreeSet<i64> = later
            .iter()
            .copied()
            .filter(|id| m.get(td_core::NodeId(*id as u64)).and_then(|n| n.parent).is_some_and(|p| lost.contains(&(p.0 as i64))))
            .collect();
        assert!(!orphans.is_empty(), "group b's brushes continue in the next chunk");

        let mut broken = bytes.clone();
        broken[at + HEADER_LEN + h.stored / 2] ^= 0x5a;
        let back = decode(&broken).unwrap();
        let expected: BTreeSet<i64> = m.nodes.keys().map(|id| id.0 as i64).filter(|id| !lost.contains(id)).chain([0]).collect();
        assert_eq!(all_ids(&back.value), expected);
        assert!(back.problems.iter().any(|p| p.contains("nodes stored in it are lost")), "{:?}", back.problems);
        assert!(back.problems.iter().any(|p| p.contains(&format!("{} of {} nodes", lost.len(), m.nodes.len()))), "{:?}", back.problems);

        let layers = back.value["layers"].as_array().unwrap();
        let recovered = layers.last().unwrap();
        assert_eq!(recovered["name"], RECOVERED_LAYER);
        let moved: BTreeSet<i64> = recovered["children"].as_array().unwrap().iter().map(|n| n["id"].as_i64().unwrap()).collect();
        assert_eq!(moved, orphans);

        let loaded = format::from_bytes(&broken).unwrap();
        assert_eq!(loaded.map.nodes.len(), expected.len());
        assert!(!loaded.problems.is_empty());
        let layer = loaded.map.layers.last().copied().unwrap();
        assert_eq!(loaded.map.get(layer).unwrap().name(), RECOVERED_LAYER);
        assert!(loaded.map.nodes.keys().all(|id| id.0 != 0));
    }

    #[test]
    fn a_damaged_header_skips_to_the_next_chunk() {
        let m = big_map();
        let bytes = format::to_bytes(&m).unwrap();
        let list = chunks(&bytes);
        let (at, h) = &list[2];
        let lost = chunk_ids(&bytes, *at, h).len();
        let mut broken = bytes.clone();
        broken[at + 17] ^= 0x01;
        let back = decode(&broken).unwrap();
        assert_eq!(all_ids(&back.value).into_iter().filter(|id| *id != 0).count(), m.nodes.len() - lost, "{:?}", back.problems);
        assert_eq!(back.value["properties"]["message"], "big");
    }

    #[test]
    fn a_damaged_head_keeps_the_nodes() {
        let m = big_map();
        let mut bytes = format::to_bytes(&m).unwrap();
        bytes[MAGIC.len() + 4 + HEADER_LEN + 3] ^= 0xff;
        let loaded = format::from_bytes(&bytes).unwrap();
        assert_eq!(loaded.map.nodes.len(), m.nodes.len());
        assert!(loaded.map.properties.is_empty());
        assert!(loaded.problems.iter().any(|p| p.contains("worldspawn properties")), "{:?}", loaded.problems);
    }

    #[test]
    fn a_truncated_file_keeps_what_came_before_the_cut() {
        let m = big_map();
        let bytes = format::to_bytes(&m).unwrap();
        let list = chunks(&bytes);
        let (at, h) = &list[3];
        let cut = at + HEADER_LEN + h.stored / 2;
        let kept: BTreeSet<i64> = list[1..3].iter().flat_map(|(at, h)| chunk_ids(&bytes, *at, h)).collect();
        let back = decode(&bytes[..cut]).unwrap();
        assert_eq!(all_ids(&back.value), kept);
        assert!(back.problems.iter().any(|p| p.contains("ends inside")), "{:?}", back.problems);

        let back = decode(&bytes[..list[3].0]).unwrap();
        assert_eq!(all_ids(&back.value), kept);
        assert!(back.problems.iter().any(|p| p.contains("ends early")), "{:?}", back.problems);
        assert!(format::from_bytes(&bytes[..list[3].0]).is_ok());
    }

    #[test]
    fn random_damage_never_panics() {
        let bytes = format::to_bytes(&big_map()).unwrap();
        for i in (0..bytes.len()).step_by(97) {
            let mut broken = bytes.clone();
            broken[i] = broken[i].wrapping_add(1 + (i % 250) as u8);
            let _ = format::from_bytes(&broken);
            let _ = format::from_bytes(&bytes[..i]);
        }
    }

    /// A list of `n` nils, which zstd packs into a few kilobytes however long it is.
    fn nils(n: usize) -> Vec<u8> {
        let mut w = Writer::new();
        w.array_header(n);
        let mut raw = w.out;
        raw.resize(raw.len() + n * 4, 0);
        raw
    }

    /// A list of nils taking just more than the whole memory budget. Its count alone passes the budget, so it is
    /// refused before any of it is built.
    fn too_many_nils() -> Vec<u8> {
        nils(variant::MAX_MEMORY / 80)
    }

    /// `bytes` with `extra` chunks before its END chunk.
    fn insert(bytes: &[u8], extra: &[([u8; 4], Vec<u8>)]) -> Vec<u8> {
        let (end_at, _) = *chunks(bytes).last().unwrap();
        let mut out = bytes[..end_at].to_vec();
        for (tag, raw) in extra {
            write_chunk(&mut out, *tag, FLAG_ZSTD, raw.len() as u32, &compress_at(raw, 1));
        }

        out.extend_from_slice(&bytes[end_at..]);
        out
    }

    /// The smallest memory budget an intact file reads in full with: what its chunks take, plus the bytes of the chunk
    /// being read where that is the most.
    fn need_of(bytes: &[u8]) -> usize {
        let (mut taken, mut peak) = (0, 0);
        for (at, h) in chunks(bytes) {
            let raw = payload(&h, &bytes[at + HEADER_LEN..at + HEADER_LEN + h.stored]).unwrap();
            let cost = variant::cost(&raw).unwrap();
            peak = usize::max(peak, taken + raw.len() + cost);
            taken += cost;
        }

        peak
    }

    /// A NODE batch holding one layer.
    fn layer_batch(id: i64, extra: Option<&dyn Fn(&mut Writer)>) -> Vec<u8> {
        let mut w = Writer::new();
        w.dict_header(2);
        w.key("parents");
        w.int64s(&[0]);
        w.key("nodes");
        w.array_header(1);
        w.dict_header(if extra.is_some() { 3 } else { 2 });
        w.key("id");
        w.value(&json!(id));
        w.key("type");
        w.value(&json!("layer"));
        if let Some(extra) = extra {
            w.key("x");
            extra(&mut w);
        }

        w.out
    }

    fn refused(problems: &[String]) -> usize {
        problems.iter().filter(|p| p.contains("takes more memory than a map may and was not read")).count()
    }

    #[test]
    fn a_few_kilobytes_cannot_take_a_gigabyte() {
        let m = big_map();
        let bytes = format::to_bytes(&m).unwrap();
        let (end_at, _) = *chunks(&bytes).last().unwrap();
        let mut crafted = bytes[..end_at].to_vec();
        // Neither padding nor an unknown chunk claiming the whole unpacked size lends the file more budget.
        let padding: Vec<u8> = (0..250_000u32).map(|i| (i.wrapping_mul(2_654_435_761) >> 24) as u8).collect();
        write_chunk(&mut crafted, *b"PADX", 0, padding.len() as u32, &padding);
        write_chunk(&mut crafted, *b"XTRA", FLAG_ZSTD, MAX_RAW, &[1, 2, 3, 4]);
        let bomb = too_many_nils();
        write_chunk(&mut crafted, NODE, FLAG_ZSTD, bomb.len() as u32, &compress_at(&bomb, 1));
        crafted.extend_from_slice(&bytes[end_at..]);
        assert!(crafted.len() < 300 << 10, "{} bytes", crafted.len());

        let started = std::time::Instant::now();
        let back = decode(&crafted).unwrap();
        assert!(started.elapsed() < std::time::Duration::from_secs(2), "{:?}", started.elapsed());
        assert_eq!(refused(&back.problems), 1, "{:?}", back.problems);
        assert_eq!(back.problems.len(), 1, "the END chunk after it is still read, {:?}", back.problems);
        assert_eq!(back.unknown.len(), 2);
        assert_eq!(all_ids(&back.value), all_ids(&format::to_value(&m)), "the map itself still loads");
    }

    #[test]
    fn chunks_share_the_budgets_in_file_order() {
        let m = big_map();
        let bytes = format::to_bytes(&m).unwrap();
        // Big enough that these chunks need more than any of the map's own.
        let fill = |w: &mut Writer| w.value(&json!("f".repeat(1 << 20)));
        let batch = |id| (NODE, layer_batch(id, Some(&fill)));
        let crafted = insert(&bytes, &[batch(901), batch(902), (NODE, layer_batch(903, None))]);
        let ids = |back: &Decoded| all_ids(&back.value).into_iter().filter(|id| *id > 900).collect::<Vec<_>>();

        let back = decode_with(&crafted, MAX_FILE_RAW, need_of(&crafted)).unwrap();
        assert!(back.problems.is_empty(), "{:?}", back.problems);
        assert_eq!(ids(&back), [901, 902, 903]);
        let back = decode_with(&crafted, MAX_FILE_RAW, need_of(&crafted) - 1).unwrap();
        assert_eq!(refused(&back.problems), 1, "{:?}", back.problems);
        let without = insert(&bytes, &[batch(901), (NODE, layer_batch(903, None))]);
        let back = decode_with(&crafted, MAX_FILE_RAW, need_of(&without)).unwrap();
        assert_eq!(ids(&back), [901, 903], "the chunk that does not fit takes nothing from the next");
        assert_eq!(refused(&back.problems), 1, "{:?}", back.problems);

        let raw = |b: &[u8]| chunks(b).iter().map(|(_, h)| h.raw as usize).sum::<usize>();
        let back = decode_with(&crafted, raw(&crafted) - 1, variant::MAX_MEMORY).unwrap();
        assert!(back.problems.iter().any(|p| p.contains("END chunk at byte") && p.contains("would unpack past the 1024 MiB")), "{:?}", back.problems);
    }

    #[test]
    fn strings_and_bytes_take_what_they_grow_to() {
        let m = big_map();
        let bytes = format::to_bytes(&m).unwrap();
        let n = 1 << 20;
        let one = |extra: &dyn Fn(&mut Writer)| (NODE, layer_batch(901, Some(extra)));
        // A gigabyte of bytes or NULs packs into a few kilobytes. Bytes turn into base64 and then into a copy of their
        // own, a NUL takes six bytes as JSON text.
        for (chunk, grows_to) in [(one(&|w| w.bytes(&vec![0; n])), n / 3 * 4 + n), (one(&|w| w.value(&Value::String("\0".repeat(n)))), 6 * n)] {
            assert!(variant::cost(&chunk.1).unwrap() > grows_to);
            let crafted = insert(&bytes, &[chunk]);
            assert!(crafted.len() < bytes.len() + 4096, "{} bytes", crafted.len());
            let back = decode_with(&crafted, MAX_FILE_RAW, need_of(&crafted)).unwrap();
            assert!(back.problems.is_empty() && all_ids(&back.value).contains(&901), "{:?}", back.problems);
            let back = decode_with(&crafted, MAX_FILE_RAW, need_of(&crafted) - 1).unwrap();
            assert_eq!(refused(&back.problems), 1, "{:?}", back.problems);
            assert!(!all_ids(&back.value).contains(&901) && back.problems.iter().all(|p| !p.contains("END")), "{:?}", back.problems);
        }
    }

    #[test]
    fn a_chunk_is_read_only_when_its_bytes_fit_beside_its_values() {
        let m = big_map();
        let bytes = format::to_bytes(&m).unwrap();
        // A nil with 64 MiB of padding after it takes almost nothing once read, but all of it is held while it is.
        let padded = vec![0; 64 << 20];
        assert_eq!(variant::cost(&padded), Ok(80));
        assert!(need_of(&bytes) < 16 << 20);
        let crafted = insert(&bytes, &[(NODE, padded)]);
        let back = decode_with(&crafted, MAX_FILE_RAW, need_of(&crafted) - 1).unwrap();
        assert_eq!(refused(&back.problems), 1, "{:?}", back.problems);
        assert!(back.problems[0].contains("NODE"), "{:?}", back.problems);
        assert_eq!(all_ids(&back.value), all_ids(&format::to_value(&m)));
        let back = decode_with(&crafted, MAX_FILE_RAW, need_of(&crafted)).unwrap();
        assert!(back.problems.is_empty(), "{:?}", back.problems);
    }

    #[test]
    fn a_light_map_past_the_budget_is_dropped() {
        let m = big_map();
        let bytes = format::to_bytes(&m).unwrap();
        let mut w = Writer::new();
        w.dict_header(1);
        w.key("chart_rows");
        w.float32s(&vec![0.0; variant::MAX_MEMORY / 80]);
        let crafted = insert(&bytes, &[(LMAP, w.out)]);

        let loaded = format::from_bytes(&crafted).unwrap();
        assert!(loaded.map.lightmap.is_none());
        assert_eq!(loaded.map.nodes.len(), m.nodes.len());
        assert!(loaded.map.unknown_chunks.is_empty(), "a refused bake is not saved again");
        assert_eq!(refused(&loaded.problems), 1, "{:?}", loaded.problems);
        assert!(loaded.problems.iter().all(|p| p.contains("baked lighting")), "{:?}", loaded.problems);
        let (_, problems) = format::file_to_json(&crafted).unwrap();
        assert_eq!(problems, loaded.problems);
    }

    #[test]
    fn the_light_map_is_charged_where_it_sits() {
        let mut m = big_map();
        let mut lm = Lightmap { width: 4, height: 2, texel_size: 16.0, ..Default::default() };
        lm.light = vec![0; 24];
        lm.shadow = vec![255; 8];
        lm.ao = vec![255; 8];
        lm.charts = (0..2_000).map(|face| Chart { node: 2, face, rows: [[0.5; 4]; 2] }).collect();
        m.lightmap = Some(Arc::new(lm));
        let bytes = format::to_bytes(&m).unwrap();
        let base = need_of(&bytes);
        let filler = (NODE, layer_batch(901, Some(&|w: &mut Writer| w.value(&json!("f".repeat(100_000))))));
        let after = insert(&bytes, std::slice::from_ref(&filler));
        let (lmap_at, _) = *chunks(&bytes).iter().find(|(_, h)| h.tag == LMAP).unwrap();
        let mut before = bytes[..lmap_at].to_vec();
        write_chunk(&mut before, filler.0, FLAG_ZSTD, filler.1.len() as u32, &compress_at(&filler.1, 1));
        before.extend_from_slice(&bytes[lmap_at..]);

        let back = decode_with(&after, MAX_FILE_RAW, base).unwrap();
        assert!(back.lightmap.is_some() && !all_ids(&back.value).contains(&901), "{:?}", back.problems);
        let back = decode_with(&before, MAX_FILE_RAW, base).unwrap();
        assert!(back.lightmap.is_none() && all_ids(&back.value).contains(&901), "{:?}", back.problems);
        assert!(back.problems.iter().any(|p| p.contains("LMAP") && p.contains("baked lighting")), "{:?}", back.problems);
    }

    #[test]
    fn a_file_that_unpacks_past_its_total_is_refused_quickly() {
        let m = big_map();
        let bytes = format::to_bytes(&m).unwrap();
        let list = chunks(&bytes);
        let (end_at, _) = list.last().unwrap();
        let intact: usize = list.iter().map(|(_, h)| h.raw as usize).sum();

        let zeros = vec![0u8; 64 << 20];
        let bombs = 64;
        let mut crafted = bytes[..*end_at].to_vec();
        let packed = compress_at(&zeros, 1);
        for _ in 0..bombs {
            write_chunk(&mut crafted, NODE, FLAG_ZSTD, zeros.len() as u32, &packed);
        }

        crafted.extend_from_slice(&bytes[*end_at..]);
        assert!(crafted.len() < 1 << 20, "{} bytes", crafted.len());

        let started = std::time::Instant::now();
        let back = decode(&crafted).unwrap();
        assert!(started.elapsed() < std::time::Duration::from_secs(10), "{:?}", started.elapsed());
        // Each chunk of zeros decodes to a single nil, so only its unpacked size counts.
        let fit = (MAX_FILE_RAW - intact) / zeros.len();
        let past = back.problems.iter().filter(|p| p.contains("would unpack past")).count();
        assert_eq!(past, bombs - fit, "{:?}", back.problems);
        assert_eq!(all_ids(&back.value), all_ids(&format::to_value(&m)), "the map itself still loads");
    }

    #[test]
    fn saving_refuses_a_map_the_reader_would_refuse() {
        let nil = nils(0);
        assert!(check_fits(&[], &[], &nil).is_ok());
        // Nils just inside the budget fit on their own, but not beside the bytes they are read from.
        let at = nils((variant::MAX_MEMORY - 80) / 80);
        let need = |p: &[u8]| (p.len() + variant::cost(p).unwrap()).div_ceil(1 << 20);
        assert!(variant::cost(&at).unwrap() <= variant::MAX_MEMORY);
        assert!(matches!(check_fits(&[&at], &[], &nil), Err(ContainerError::TooMuchMemory(n)) if n == need(&at)));
        let fits = nils((variant::MAX_MEMORY - 80) / 84 - 2);
        assert!(check_fits(&[&fits], &[], &nil).is_ok());
        assert!(matches!(check_fits(&[&fits, &fits], &[], &nil), Err(ContainerError::TooMuchMemory(_))), "the chunks share the budget");
        let claims_it_all = RawChunk { tag: LMAP, flags: FLAG_ZSTD, raw_size: MAX_RAW, data: Arc::from(&[1u8, 2, 3][..]) };
        assert!(matches!(check_fits(&[], std::slice::from_ref(&claims_it_all), &nil), Err(ContainerError::TooLarge(1025))));
        let kept = RawChunk { tag: *b"XTRA", ..claims_it_all };
        assert!(check_fits(&[], &[kept], &nil).is_ok(), "chunks the reader keeps as stored are never unpacked");

        let dir = std::env::temp_dir().join(format!("td_too_big_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("big.tdm");
        let mut m = big_map();
        let probes = |n: usize| Probes { sh: vec![0.0; n], ..Default::default() };
        let lm = |n| Lightmap { width: 1, height: 1, light: vec![0; 3], shadow: vec![0], ao: vec![0], probes: probes(n), ..Default::default() };
        m.lightmap = Some(Arc::new(lm(variant::MAX_MEMORY / 80)));
        let error = format::save(&m, &path).unwrap_err().to_string();
        assert!(error.starts_with("the map is too big to save, reading it back would take") && error.ends_with("a map file may take 1200 MiB"), "{error}");
        assert!(!path.exists() && !path.with_extension("tdm.tmp").exists());

        m.lightmap = Some(Arc::new(lm(1 << 20)));
        format::save(&m, &path).unwrap();
        assert!(format::load(&path).unwrap().problems.is_empty());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn committed_maps_cost_what_the_reader_takes() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut dirs = vec![root.join("godot/demo/maps"), root.join("godot/demo/maps/showcase"), root.join("godot/tests/maps")];
        dirs.extend(std::fs::read_dir(root.join("crates/td_editor/assets/prefab_library")).unwrap().flatten().map(|e| e.path()).filter(|p| p.is_dir()));
        let mut seen = 0;
        for path in dirs.iter().flat_map(|d| std::fs::read_dir(d).unwrap().flatten().map(|e| e.path())) {
            if path.extension().is_none_or(|e| e != "tdm") {
                continue;
            }

            let bytes = std::fs::read(&path).unwrap();
            if !is_binary(&bytes) {
                continue;
            }

            let mut total = 0;
            for (at, h) in chunks(&bytes) {
                let raw = payload(&h, &bytes[at + HEADER_LEN..at + HEADER_LEN + h.stored]).unwrap();
                let mut left = usize::MAX;
                variant::decode_within(&raw, &mut left).unwrap();
                assert_eq!(variant::cost(&raw), Ok(usize::MAX - left), "{}", path.display());
                total += usize::MAX - left;
            }

            assert!(total < variant::MAX_MEMORY / 8, "{} takes {total}", path.display());
            seen += 1;
        }

        assert!(seen >= 30, "{seen} maps");
    }

    #[test]
    fn deep_nesting_is_cut_instead_of_overflowing() {
        let mut node = json!({ "id": 1000, "type": "group", "name": "g" });
        for id in (2..1000).rev() {
            node = json!({ "id": id, "type": "group", "name": "g", "children": [node] });
        }

        let value = json!({ "format": "trenchdot-map", "version": 1, "layers": [
            { "id": 1, "type": "layer", "name": "L", "color": "#ffffffff", "omit_from_export": false, "children": [node] }
        ] });
        let loaded = std::thread::Builder::new()
            .stack_size(1 << 20)
            .spawn(move || format::from_bytes(&encode(value, &[]).unwrap()).map(|l| (l.map.nodes.len(), l.problems)))
            .unwrap()
            .join()
            .unwrap()
            .unwrap();
        assert!(loaded.0 >= 1000, "every node is kept, {}", loaded.0);
        assert!(loaded.1.iter().any(|p| p.contains(RECOVERED_LAYER)));
    }

    #[test]
    fn unknown_chunks_survive_a_save() {
        let m = big_map();
        let extra = RawChunk { tag: *b"XTRA", flags: 7, raw_size: 5, data: Arc::from(&b"\x01\x02\x03\x04\x05\x06"[..]) };
        let bytes = encode(format::to_value(&m), std::slice::from_ref(&extra)).unwrap();
        let loaded = format::from_bytes(&bytes).unwrap();
        assert!(loaded.problems.is_empty(), "{:?}", loaded.problems);
        assert_eq!(loaded.map.unknown_chunks, vec![extra.clone()]);
        let mut edited = loaded.map.clone();
        edited.properties.insert("message".into(), "edited".into());
        let again = format::from_bytes(&format::to_bytes(&edited).unwrap()).unwrap();
        assert_eq!(again.map.unknown_chunks, vec![extra]);
        assert_eq!(again.map.properties["message"], "edited");
    }

    #[test]
    fn old_json_and_mangled_files() {
        let m = big_map();
        let json = format::to_string(&m);
        let loaded = format::from_bytes(json.as_bytes()).unwrap();
        assert_eq!(format::to_value(&loaded.map), format::to_value(&m));
        assert!(format::from_bytes(format!("\u{feff}{json}").as_bytes()).is_ok());

        let mut mangled = format::to_bytes(&m).unwrap();
        mangled.remove(4);
        assert!(matches!(format::from_bytes(&mangled), Err(format::FormatError::Container(ContainerError::Mangled))));

        let mut newer = format::to_bytes(&m).unwrap();
        newer[8] = 2;
        assert!(matches!(format::from_bytes(&newer), Err(format::FormatError::Container(ContainerError::TooNew(2)))));
        assert!(matches!(decode(&MAGIC), Err(ContainerError::NoHeader)));
    }
}
