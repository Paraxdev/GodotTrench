//! The value encoding inside `.tdm` chunks: the part of Godot's binary Variant serialization that JSON values need,
//! so the Godot addon decodes a whole chunk with one `bytes_to_var` call. See docs/format/container.md.

use base64::Engine;
use serde_json::{Map, Number, Value};

const NIL: u32 = 0;
const BOOL: u32 = 1;
const INT: u32 = 2;
const FLOAT: u32 = 3;
const STRING: u32 = 4;
const DICTIONARY: u32 = 27;
const ARRAY: u32 = 28;
const BYTES: u32 = 29;
const INT32S: u32 = 30;
const INT64S: u32 = 31;
const FLOAT32S: u32 = 32;
const FLOAT64S: u32 = 33;
const TYPE_MASK: u32 = 0xff;
const FLAG_64: u32 = 1 << 16;
const COUNT_MASK: u32 = 0x7fff_ffff;

/// Shorter number arrays stay plain arrays: Godot decodes many tiny packed arrays slower than tiny arrays, and zstd
/// shrinks both to the same size.
pub const PACK_MIN: usize = 32;
const MAX_DEPTH: usize = 128;
/// What the values of one file may take in memory once read, the map and its baked lighting together. About sixteen
/// times what the biggest showcase map takes, and room for one and a half to two million scatter instances.
pub const MAX_MEMORY: usize = 1200 << 20;
/// The size of a `serde_json::Value`. Dictionary keys and the items of packed arrays become values too.
const VALUE_COST: usize = 80;
/// A string written as JSON text takes up to six bytes per byte, `\u0000`, and Godot keeps four bytes per character.
const STRING_BYTE_COST: usize = 6;

/// Bytes are read as base64 text, 4 bytes per 3, which terrain and baked lighting then decode into a copy of their own.
fn bytes_cost(len: usize) -> usize {
    len.div_ceil(3) * 4 + len
}

/// Terrain keys holding base64 in JSON. They are stored as raw bytes and turned back into base64 on reading.
const BYTE_KEYS: [&str; 3] = ["heights", "splat", "holes"];

fn base64() -> base64::engine::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}

/// Decodes base64 a block at a time and hands each block to `sink`, 3072 bytes but for the last, so the decoded bytes
/// are never held whole. None where decoding the text whole fails.
pub(crate) fn decode_base64(text: &str, mut sink: impl FnMut(&[u8])) -> Option<()> {
    const BLOCK: usize = 4096;
    let mut out = [0; BLOCK / 4 * 3];
    let mut blocks = text.as_bytes().chunks(BLOCK).peekable();
    while let Some(block) = blocks.next() {
        // Padding before the end makes the whole text invalid, but would end a block of its own cleanly.
        if blocks.peek().is_some() && block.ends_with(b"=") {
            return None;
        }

        let n = base64().decode_slice(block, &mut out).ok()?;
        sink(&out[..n]);
    }

    Some(())
}

pub struct Writer {
    pub out: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Self { out: Vec::new() }
    }

    fn u32(&mut self, v: u32) {
        self.out.extend_from_slice(&v.to_le_bytes());
    }

    fn padded(&mut self, bytes: &[u8]) {
        self.u32(bytes.len() as u32);
        self.out.extend_from_slice(bytes);
        self.out.resize(self.out.len().next_multiple_of(4), 0);
    }

    fn string(&mut self, s: &str) {
        self.u32(STRING);
        self.padded(s.as_bytes());
    }

    pub fn int64s(&mut self, values: &[i64]) {
        self.u32(INT64S);
        self.u32(values.len() as u32);
        values.iter().for_each(|v| self.out.extend_from_slice(&v.to_le_bytes()));
    }

    pub fn int32s(&mut self, values: &[i32]) {
        self.u32(INT32S);
        self.u32(values.len() as u32);
        values.iter().for_each(|v| self.out.extend_from_slice(&v.to_le_bytes()));
    }

    pub fn bytes(&mut self, bytes: &[u8]) {
        self.u32(BYTES);
        self.padded(bytes);
    }

    pub fn float32s(&mut self, values: &[f32]) {
        self.u32(FLOAT32S);
        self.u32(values.len() as u32);
        values.iter().for_each(|v| self.out.extend_from_slice(&v.to_le_bytes()));
    }

    pub fn array_header(&mut self, len: usize) {
        self.u32(ARRAY);
        self.u32(len as u32);
    }

    pub fn dict_header(&mut self, len: usize) {
        self.u32(DICTIONARY);
        self.u32(len as u32);
    }

    pub fn key(&mut self, key: &str) {
        self.string(key);
    }

    pub fn value(&mut self, v: &Value) {
        match v {
            Value::Null => self.u32(NIL),
            Value::Bool(b) => {
                self.u32(BOOL);
                self.u32(*b as u32);
            }
            Value::Number(n) => self.number(n),
            Value::String(s) => self.string(s),
            Value::Array(items) => {
                if !self.packed(items) {
                    self.array_header(items.len());
                    items.iter().for_each(|i| self.value(i));
                }
            }
            Value::Object(map) => self.object(map, false),
        }
    }

    /// A node object without its `children`. Terrain byte fields are written raw instead of as base64 text, and
    /// scatter instances as fixed point columns when that is exact.
    pub fn node(&mut self, map: &Map<String, Value>) {
        self.object(map, true);
    }

    fn object(&mut self, map: &Map<String, Value>, node: bool) {
        let skipped = node && map.contains_key("children");
        let kind = if node { map.get("type").and_then(Value::as_str) } else { None };
        self.dict_header(map.len() - skipped as usize);
        for (k, v) in map {
            if skipped && k == "children" {
                continue;
            }

            self.key(k);
            match (kind, k.as_str(), v) {
                (Some("terrain"), key, Value::String(s)) if BYTE_KEYS.contains(&key) && canonical_base64(s).is_some() => {
                    self.u32(BYTES);
                    self.padded(&canonical_base64(s).unwrap_or_default());
                }
                (Some("scatter"), "instances", Value::Array(items)) if let Some(columns) = instance_columns(items) => {
                    self.u32(INT32S);
                    self.u32(columns.len() as u32);
                    columns.iter().for_each(|v| self.out.extend_from_slice(&v.to_le_bytes()));
                }
                _ => self.value(v),
            }
        }
    }

    fn number(&mut self, n: &Number) {
        if let Some(i) = n.as_i64() {
            match i32::try_from(i) {
                Ok(small) => {
                    self.u32(INT);
                    self.out.extend_from_slice(&small.to_le_bytes());
                }
                Err(_) => {
                    self.u32(INT | FLAG_64);
                    self.out.extend_from_slice(&i.to_le_bytes());
                }
            }
        } else {
            // Integers past i64::MAX have no Godot type and are stored as floats, nothing in a map uses them.
            self.u32(FLOAT | FLAG_64);
            self.out.extend_from_slice(&n.as_f64().unwrap_or_default().to_le_bytes());
        }
    }

    /// Writes a long all-integer or all-float array as a packed array, floats as 32 bit only when that is exact.
    fn packed(&mut self, items: &[Value]) -> bool {
        if items.len() < PACK_MIN {
            return false;
        }

        let ints: Option<Vec<i32>> = items.iter().map(|v| v.as_i64().and_then(|i| i32::try_from(i).ok())).collect();
        if let Some(ints) = ints {
            self.u32(INT32S);
            self.u32(ints.len() as u32);
            ints.iter().for_each(|v| self.out.extend_from_slice(&v.to_le_bytes()));
            return true;
        }

        let floats: Option<Vec<f64>> = items.iter().map(|v| v.as_number().filter(|n| n.is_f64()).and_then(Number::as_f64)).collect();
        let Some(floats) = floats else { return false };
        if floats.iter().all(|f| f64::from(*f as f32) == *f) {
            self.u32(FLOAT32S);
            self.u32(floats.len() as u32);
            floats.iter().for_each(|f| self.out.extend_from_slice(&(*f as f32).to_le_bytes()));
        } else {
            self.u32(FLOAT64S);
            self.u32(floats.len() as u32);
            floats.iter().for_each(|f| self.out.extend_from_slice(&f.to_le_bytes()));
        }

        true
    }
}

/// Scatter instances are saved rounded to these steps (`[item, x, y, z, pitch, yaw, roll, scale]`, see
/// `ScatterInstance`), so whole multiples of them store losslessly as integers. As 64 bit floats their mantissas would
/// not compress.
pub const INSTANCE_SCALES: [f64; 8] = [1.0, 100.0, 100.0, 100.0, 10.0, 10.0, 10.0, 1000.0];

/// All instances as integer columns, every item of the first field, then every x and so on, when each value is
/// exactly `k / scale` for an integer `k`.
fn instance_columns(items: &[Value]) -> Option<Vec<i32>> {
    if items.is_empty() {
        return None;
    }

    let mut columns = vec![0; items.len() * 8];
    for (i, item) in items.iter().enumerate() {
        let values = item.as_array().filter(|a| a.len() == 8)?;
        for (c, (v, scale)) in values.iter().zip(INSTANCE_SCALES).enumerate() {
            let v = v.as_number().filter(|n| n.is_f64())?.as_f64()?;
            let k = (v * scale).round();
            let k = (k.abs() < f64::from(i32::MAX)).then_some(k as i32)?;
            columns[c * items.len() + i] = (f64::from(k) / scale).to_bits().eq(&v.to_bits()).then_some(k)?;
        }
    }

    Some(columns)
}

/// Undoes the node level storage choices of [`Writer::node`] on a decoded node.
pub fn restore_node(node: &mut Map<String, Value>) {
    if node.get("type").and_then(Value::as_str) != Some("scatter") {
        return;
    }

    let Some(Value::Array(flat)) = node.get_mut("instances") else { return };
    if flat.first().is_none_or(Value::is_array) || flat.len() % 8 != 0 {
        return;
    }

    // The columns are copied out as plain numbers and freed first, so they and the instances are never both held as
    // values, which take ten times the memory.
    let columns: Vec<f64> = std::mem::take(flat).iter().map(|v| v.as_f64().unwrap_or(0.0)).collect();
    let n = columns.len() / 8;
    let at = |c: usize, i: usize| columns[c * n + i] / INSTANCE_SCALES[c];
    *flat = (0..n).map(|i| Value::Array((0..8).map(|c| float(at(c, i))).collect())).collect();
}

/// The bytes of `s` when writing them back as base64 gives exactly `s`, so storing them raw loses nothing.
fn canonical_base64(s: &str) -> Option<Vec<u8>> {
    let bytes = base64().decode(s).ok()?;
    (base64().encode(&bytes) == s).then_some(bytes)
}

#[cfg(test)]
pub fn decode(bytes: &[u8]) -> Result<Value, Refused> {
    let mut budget = MAX_MEMORY;
    decode_within(bytes, &mut budget)
}

/// Why a chunk is not read.
#[derive(Debug, PartialEq)]
pub enum Refused {
    /// Its values would take more memory than is left of the file's budget.
    TooBig,
    Damaged(&'static str),
}

/// Decodes one chunk and takes what its values take in memory from `budget`, which the chunks of a file share. The
/// chunk's bytes are held while its values are built, so they have to fit in the budget too, but are not taken from
/// it. The values are counted before any is built, so a chunk that does not fit is refused without taking memory or
/// budget.
pub fn decode_within(bytes: &[u8], budget: &mut usize) -> Result<Value, Refused> {
    let room = budget.checked_sub(bytes.len()).ok_or(Refused::TooBig)?;
    let mut count = Reader { bytes, pos: 0, left: room };
    count.charge(VALUE_COST)?;
    count.skip(0)?;
    let mut r = Reader { bytes, pos: 0, left: room };
    r.charge(VALUE_COST)?;
    let value = r.value(0)?;
    *budget = r.left + bytes.len();
    Ok(value)
}

/// What [`decode_within`] would take from the budget for `bytes`, counted without building the values.
pub fn cost(bytes: &[u8]) -> Result<usize, Refused> {
    let mut r = Reader { bytes, pos: 0, left: usize::MAX };
    r.charge(VALUE_COST)?;
    r.skip(0)?;
    Ok(usize::MAX - r.left)
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
    /// What is left of the memory budget.
    left: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], Refused> {
        let end = self.pos.checked_add(n).filter(|e| *e <= self.bytes.len()).ok_or(Refused::Damaged("value runs past the end of its chunk"))?;
        let s = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(s)
    }

    fn u32(&mut self) -> Result<u32, Refused> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap_or_default()))
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], Refused> {
        Ok(self.take(N)?.try_into().unwrap_or([0; N]))
    }

    fn charge(&mut self, cost: usize) -> Result<(), Refused> {
        self.left = self.left.checked_sub(cost).ok_or(Refused::TooBig)?;
        Ok(())
    }

    /// The type of the next value and whether it is 64 bit.
    fn header(&mut self, depth: usize) -> Result<(u32, bool), Refused> {
        if depth > MAX_DEPTH {
            return Err(Refused::Damaged("values nested too deep"));
        }

        let header = self.u32()?;
        let kind = header & TYPE_MASK;
        let wide = header == kind | FLAG_64 && matches!(kind, INT | FLOAT);
        if header != kind && !wide {
            return Err(Refused::Damaged("value header has unknown flags"));
        }

        Ok((kind, wide))
    }

    /// The bytes of a string or byte array, charged by what they grow to once decoded.
    fn text(&mut self, kind: u32) -> Result<&'a [u8], Refused> {
        let len = self.u32()? as usize;
        let data = self.take(len)?;
        self.take((4 - len % 4) % 4)?;
        self.charge(if kind == STRING { len.saturating_mul(STRING_BYTE_COST) } else { bytes_cost(len) })?;
        Ok(data)
    }

    fn string(&mut self) -> Result<&'a str, Refused> {
        std::str::from_utf8(self.text(STRING)?).map_err(|_| Refused::Damaged("string is not UTF-8"))
    }

    /// The item count of a container or packed array, once it is clear the items fit in the chunk and a value for each
    /// item is charged, two for a dictionary entry. Godot sets bit 31 of a shared container's count.
    fn count(&mut self, kind: u32) -> Result<usize, Refused> {
        let raw = self.u32()?;
        let (n, item_size, values) = match kind {
            DICTIONARY => (raw & COUNT_MASK, 8, 2),
            ARRAY => (raw & COUNT_MASK, 4, 1),
            INT32S | FLOAT32S => (raw, 4, 1),
            _ => (raw, 8, 1),
        };
        let n = n as usize;
        if n.saturating_mul(item_size) > self.bytes.len() - self.pos {
            return Err(Refused::Damaged("array count runs past the end of its chunk"));
        }

        self.charge(n * values * VALUE_COST)?;
        Ok(n)
    }

    fn packed<const N: usize>(&mut self, kind: u32) -> Result<&'a [[u8; N]], Refused> {
        let n = self.count(kind)?;
        Ok(self.take(n * N)?.as_chunks::<N>().0)
    }

    fn value(&mut self, depth: usize) -> Result<Value, Refused> {
        let (kind, wide) = self.header(depth)?;
        Ok(match kind {
            NIL => Value::Null,
            BOOL => Value::Bool(self.u32()? != 0),
            INT if wide => Value::from(i64::from_le_bytes(self.fixed()?)),
            INT => Value::from(i32::from_le_bytes(self.fixed()?)),
            FLOAT if wide => float(f64::from_le_bytes(self.fixed()?)),
            FLOAT => float(f32::from_le_bytes(self.fixed()?).into()),
            STRING => Value::String(self.string()?.to_owned()),
            DICTIONARY => {
                let n = self.count(kind)?;
                let mut map = Map::with_capacity(n);
                for _ in 0..n {
                    let Value::String(key) = self.value(depth + 1)? else { return Err(Refused::Damaged("dictionary key is not a string")) };
                    map.insert(key, self.value(depth + 1)?);
                }

                Value::Object(map)
            }
            ARRAY => {
                let n = self.count(kind)?;
                let mut items = Vec::with_capacity(n);
                for _ in 0..n {
                    items.push(self.value(depth + 1)?);
                }

                Value::Array(items)
            }
            BYTES => Value::String(base64().encode(self.text(kind)?)),
            INT32S => self.packed::<4>(kind)?.iter().map(|b| Value::from(i32::from_le_bytes(*b))).collect(),
            INT64S => self.packed::<8>(kind)?.iter().map(|b| Value::from(i64::from_le_bytes(*b))).collect(),
            FLOAT32S => self.packed::<4>(kind)?.iter().map(|b| float(f32::from_le_bytes(*b).into())).collect(),
            FLOAT64S => self.packed::<8>(kind)?.iter().map(|b| float(f64::from_le_bytes(*b))).collect(),
            _ => return Err(Refused::Damaged("unsupported value type")),
        })
    }

    /// Reads past a value the way [`Reader::value`] reads it, charging the same, and returns its type.
    fn skip(&mut self, depth: usize) -> Result<u32, Refused> {
        let (kind, wide) = self.header(depth)?;
        match kind {
            NIL => {}
            BOOL => {
                self.u32()?;
            }
            INT | FLOAT => {
                self.take(if wide { 8 } else { 4 })?;
            }
            STRING => {
                self.string()?;
            }
            BYTES => {
                self.text(kind)?;
            }
            DICTIONARY => {
                for _ in 0..self.count(kind)? {
                    if self.skip(depth + 1)? != STRING {
                        return Err(Refused::Damaged("dictionary key is not a string"));
                    }

                    self.skip(depth + 1)?;
                }
            }
            ARRAY => {
                for _ in 0..self.count(kind)? {
                    self.skip(depth + 1)?;
                }
            }
            INT32S | FLOAT32S => {
                self.packed::<4>(kind)?;
            }
            INT64S | FLOAT64S => {
                self.packed::<8>(kind)?;
            }
            _ => return Err(Refused::Damaged("unsupported value type")),
        }

        Ok(kind)
    }
}

fn float(f: f64) -> Value {
    Number::from_f64(f).map_or(Value::Null, Value::Number)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn round_trip(v: &Value) -> Value {
        let mut w = Writer::new();
        w.value(v);
        assert_eq!(w.out.len() % 4, 0);
        decode(&w.out).unwrap()
    }

    #[test]
    fn values_round_trip() {
        let long_ints: Vec<i64> = (0..40).map(|i| i * 1000 - 7).collect();
        let long_f32: Vec<f64> = (0..40).map(|i| i as f64 * 0.5).collect();
        let long_f64: Vec<f64> = (0..40).map(|i| i as f64 * 0.1).collect();
        let v = json!({
            "null": null, "yes": true, "no": false, "small": -5, "big": 1_i64 << 40, "neg": -(1_i64 << 40), "f": 0.1, "whole": 2.0,
            "s": "héllo", "odd": "abc", "empty": "", "list": [1, 2.5, "x", [], {}], "ints": long_ints, "f32": long_f32,
            "f64": long_f64, "mixed": (0..40).map(|i| if i == 3 { json!(1.5) } else { json!(i) }).collect::<Vec<_>>(),
        });
        assert_eq!(round_trip(&v), v);
    }

    #[test]
    fn terrain_bytes_are_raw() {
        let heights = base64().encode([1u8, 2, 3, 4, 5]);
        let v = json!({ "id": 1, "type": "terrain", "heights": heights, "splat": "not base64!" });
        let mut w = Writer::new();
        w.node(v.as_object().unwrap());
        assert!(w.out.windows(5).any(|s| s == [1, 2, 3, 4, 5]));
        assert_eq!(decode(&w.out).unwrap(), v);
    }

    fn node_round_trip(v: &Value) -> (Value, usize) {
        let mut w = Writer::new();
        w.node(v.as_object().unwrap());
        let Value::Object(mut back) = decode(&w.out).unwrap() else { panic!() };
        restore_node(&mut back);
        (Value::Object(back), w.out.len())
    }

    #[test]
    fn scatter_instances_are_fixed_point_only_when_exact() {
        let exact = json!([[0.0, 12.34, -5.0, 1024.5, 0.0, 359.9, -0.1, 1.25], [2.0, -0.01, 0.0, 3.0, 45.0, 0.0, 0.0, 0.8]]);
        let set = |instances: &Value| json!({ "id": 3, "type": "scatter", "name": "s", "instances": instances, "targets": [1] });
        let (back, fixed_len) = node_round_trip(&set(&exact));
        assert_eq!(back, set(&exact));

        for odd in [json!(0.123456), json!(-0.0), json!(2), json!(1e12)] {
            let mut inexact = exact.clone();
            inexact[1][3] = odd;
            let (back, len) = node_round_trip(&set(&inexact));
            assert_eq!(back, set(&inexact));
            assert!(len > fixed_len, "{inexact} is written as plain arrays");
        }

        let mut short = exact.clone();
        short[0].as_array_mut().unwrap().pop();
        assert_eq!(node_round_trip(&set(&short)).0, set(&short));
        assert_eq!(node_round_trip(&set(&json!([]))).0, set(&json!([])));
    }

    #[test]
    fn broken_input_is_an_error() {
        let mut w = Writer::new();
        w.value(&json!({ "a": [1, 2, 3], "b": "text" }));
        for cut in 0..w.out.len() {
            assert!(decode(&w.out[..cut]).is_err(), "cut at {cut}");
            assert!(cost(&w.out[..cut]).is_err(), "cut at {cut}");
        }

        let mut huge = Vec::new();
        huge.extend_from_slice(&ARRAY.to_le_bytes());
        huge.extend_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode(&huge).is_err());
    }

    /// `{"a": [1, 2, 3], "s": "héllo", "b": <5 bytes>, "p": <40 int32>}`
    fn mixed() -> Vec<u8> {
        let mut w = Writer::new();
        w.dict_header(4);
        w.key("a");
        w.value(&json!([1, 2, 3]));
        w.key("s");
        w.value(&json!("héllo"));
        w.key("b");
        w.bytes(&[7; 5]);
        w.key("p");
        w.int32s(&[9; 40]);
        w.out
    }

    #[test]
    fn values_are_charged_by_what_they_take_in_memory() {
        let raw = mixed();
        // The dictionary, its four keys and four values, the three numbers and the forty packed ones at 80 bytes each,
        // six for each byte of the keys and of the six byte string, and for the five bytes 8 as base64 and 5 decoded.
        let expected = (1 + 4 + 4 + 3 + 40) * 80 + (4 + 6) * 6 + 8 + 5;
        assert_eq!(cost(&raw), Ok(expected));

        // The chunk's own bytes have to fit next to its values while they are built.
        let mut budget = raw.len() + expected + 1;
        assert!(decode_within(&raw, &mut budget).is_ok());
        assert_eq!(budget, raw.len() + 1, "only the values are taken");
        assert_eq!(decode_within(&raw, &mut budget), Err(Refused::TooBig), "nothing is left for a second chunk");
        assert_eq!(budget, raw.len() + 1, "a refused chunk takes nothing");
        let mut budget = raw.len() + expected - 1;
        assert_eq!(decode_within(&raw, &mut budget), Err(Refused::TooBig));
        assert_eq!(budget, raw.len() + expected - 1);
    }

    /// Checks that `raw` needs exactly `cost` next to its own bytes.
    fn fits_exactly(raw: &[u8], cost: usize) {
        let mut budget = raw.len() + cost - 1;
        assert_eq!(decode_within(raw, &mut budget), Err(Refused::TooBig));
        budget += 1;
        assert!(decode_within(raw, &mut budget).is_ok());
        assert_eq!(budget, raw.len());
    }

    #[test]
    fn strings_and_bytes_are_charged_before_they_are_copied() {
        let mut w = Writer::new();
        w.bytes(&vec![0; 3 << 20]);
        fits_exactly(&w.out, 80 + (4 << 20) + (3 << 20));
        let mut w = Writer::new();
        w.value(&Value::String("\0".repeat(1 << 20)));
        fits_exactly(&w.out, 80 + (6 << 20));
    }

    #[test]
    fn base64_decodes_a_block_at_a_time_like_it_does_whole() {
        let data: Vec<u8> = (0..20_000u32).map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8).collect();
        for len in [0, 1, 2, 3, 3071, 3072, 3073, 6144, 6145, 20_000] {
            let text = base64().encode(&data[..len]);
            let mut back = Vec::new();
            assert_eq!(decode_base64(&text, |b| back.extend_from_slice(b)), Some(()), "{len}");
            assert_eq!(back, data[..len]);
        }

        let block = base64().encode(&data[..3071]);
        assert_eq!(block.len(), 4096);
        let broken = [format!("{block}QUJD"), "QQ".into(), "QQ=".into(), "QR==".into(), "QUJD\n".into(), "Q===".into(), format!("{}!", &block[..4095])];
        for text in &broken {
            assert!(base64().decode(text).is_err(), "{text:?}");
            assert_eq!(decode_base64(text, |_| {}), None, "{text:?}");
        }
    }

    fn header(kind: u32, body: &[u8]) -> Vec<u8> {
        let mut out = kind.to_le_bytes().to_vec();
        out.extend_from_slice(body);
        out
    }

    fn nested(n: usize) -> Vec<u8> {
        let mut out = Vec::new();
        for i in 0..n {
            out.extend_from_slice(&ARRAY.to_le_bytes());
            out.extend_from_slice(&u32::from(i + 1 < n).to_le_bytes());
        }

        out
    }

    #[test]
    fn values_the_godot_addon_refuses_are_refused() {
        let refused = [
            header(NIL | FLAG_64, &[]),
            header(STRING | FLAG_64, &[0; 4]),
            header(ARRAY | 1 << 17, &[0; 4]),
            header(INT32S, &(1u32 << 31 | 1).to_le_bytes()),
            header(DICTIONARY, &[1, 0, 0, 0, 2, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0]),
            nested(130),
        ];
        for raw in &refused {
            assert!(matches!(decode(raw), Err(Refused::Damaged(_))), "{raw:?}");
            assert!(matches!(cost(raw), Err(Refused::Damaged(_))), "{raw:?}");
        }

        let read = [header(INT | FLAG_64, &[0; 8]), header(FLOAT | FLAG_64, &[0; 8]), header(ARRAY, &(1u32 << 31).to_le_bytes()), nested(129)];
        for raw in &read {
            assert!(decode(raw).is_ok(), "{raw:?}");
            assert!(cost(raw).is_ok(), "{raw:?}");
        }
    }
}
