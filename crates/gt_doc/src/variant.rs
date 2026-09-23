//! The value encoding inside `.gtm` chunks: the part of Godot's binary Variant serialization that JSON values need,
//! so the Godot addon decodes a whole chunk with one `bytes_to_var` call. See docs/format-gtm.md.

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

/// Terrain keys holding base64 in JSON. They are stored as raw bytes and turned back into base64 on reading.
const BYTE_KEYS: [&str; 3] = ["heights", "splat", "holes"];

fn base64() -> base64::engine::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
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

    let Some(Value::Array(flat)) = node.get("instances") else { return };
    if flat.first().is_none_or(Value::is_array) || flat.len() % 8 != 0 {
        return;
    }

    let n = flat.len() / 8;
    let at = |c: usize, i: usize| flat[c * n + i].as_f64().unwrap_or(0.0) / INSTANCE_SCALES[c];
    let instances: Vec<Value> = (0..n).map(|i| Value::Array((0..8).map(|c| float(at(c, i))).collect())).collect();
    node.insert("instances".into(), Value::Array(instances));
}

/// The bytes of `s` when writing them back as base64 gives exactly `s`, so storing them raw loses nothing.
fn canonical_base64(s: &str) -> Option<Vec<u8>> {
    let bytes = base64().decode(s).ok()?;
    (base64().encode(&bytes) == s).then_some(bytes)
}

pub fn decode(bytes: &[u8]) -> Result<Value, String> {
    let mut r = Reader { bytes, pos: 0 };
    r.value(0)
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let end = self.pos.checked_add(n).filter(|e| *e <= self.bytes.len()).ok_or("value runs past the end of its chunk")?;
        let s = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(s)
    }

    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap_or_default()))
    }

    fn fixed<const N: usize>(&mut self) -> Result<[u8; N], String> {
        Ok(self.take(N)?.try_into().unwrap_or([0; N]))
    }

    fn padded(&mut self) -> Result<Vec<u8>, String> {
        let len = self.u32()? as usize;
        let data = self.take(len)?.to_vec();
        self.take((4 - len % 4) % 4)?;
        Ok(data)
    }

    fn count(&mut self, item_size: usize) -> Result<usize, String> {
        let n = (self.u32()? & COUNT_MASK) as usize;
        if n.saturating_mul(item_size) > self.bytes.len() - self.pos {
            return Err("array count runs past the end of its chunk".into());
        }

        Ok(n)
    }

    fn value(&mut self, depth: usize) -> Result<Value, String> {
        if depth > MAX_DEPTH {
            return Err("values nested too deep".into());
        }

        let header = self.u32()?;
        let wide = header & FLAG_64 != 0;
        Ok(match header & TYPE_MASK {
            NIL => Value::Null,
            BOOL => Value::Bool(self.u32()? != 0),
            INT if wide => Value::from(i64::from_le_bytes(self.fixed()?)),
            INT => Value::from(i32::from_le_bytes(self.fixed()?)),
            FLOAT if wide => float(f64::from_le_bytes(self.fixed()?)),
            FLOAT => float(f32::from_le_bytes(self.fixed()?).into()),
            STRING => Value::String(String::from_utf8(self.padded()?).map_err(|_| "string is not UTF-8")?),
            DICTIONARY => {
                let n = self.count(8)?;
                let mut map = Map::with_capacity(n);
                for _ in 0..n {
                    let Value::String(key) = self.value(depth + 1)? else { return Err("dictionary key is not a string".into()) };
                    map.insert(key, self.value(depth + 1)?);
                }

                Value::Object(map)
            }
            ARRAY => {
                let n = self.count(4)?;
                (0..n).map(|_| self.value(depth + 1)).collect::<Result<_, _>>()?
            }
            BYTES => Value::String(base64().encode(self.padded()?)),
            INT32S => {
                let n = self.count(4)?;
                (0..n).map(|_| self.fixed().map(|b| Value::from(i32::from_le_bytes(b)))).collect::<Result<_, _>>()?
            }
            INT64S => {
                let n = self.count(8)?;
                (0..n).map(|_| self.fixed().map(|b| Value::from(i64::from_le_bytes(b)))).collect::<Result<_, _>>()?
            }
            FLOAT32S => {
                let n = self.count(4)?;
                (0..n).map(|_| self.fixed().map(|b| float(f32::from_le_bytes(b).into()))).collect::<Result<_, _>>()?
            }
            FLOAT64S => {
                let n = self.count(8)?;
                (0..n).map(|_| self.fixed().map(|b| float(f64::from_le_bytes(b)))).collect::<Result<_, _>>()?
            }
            other => return Err(format!("unsupported value type {other}")),
        })
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
        }

        let mut huge = Vec::new();
        huge.extend_from_slice(&ARRAY.to_le_bytes());
        huge.extend_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode(&huge).is_err());
    }
}
