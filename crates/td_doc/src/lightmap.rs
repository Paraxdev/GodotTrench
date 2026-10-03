//! Baked lighting stored with the map: the light, shadow and ambient occlusion atlases and where each surface sits in
//! them. Binary maps keep it in an `LMAP` chunk, JSON maps under a top-level `lightmap` key. See
//! docs/format/container.md.

use std::collections::BTreeMap;

use base64::Engine;
use serde_json::{Value, json};

use crate::variant::{self, Writer};
use crate::{Map, NodeKind};

pub const TAG: [u8; 4] = *b"LMAP";

/// The bake settings last used on a map, kept with its editor state so the next bake starts from them. Stored as
/// names so an editor that does not know a value falls back to its default.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BakeOptions {
    #[serde(default)]
    pub quality: String,
    /// Map units per texel.
    #[serde(default)]
    pub texel_size: f64,
    /// 0 for hard shadows to 1 for the softest.
    #[serde(default = "default_softness")]
    pub softness: f64,
    #[serde(default)]
    pub backend: String,
}

/// Shadow softness of a bake that sets none.
pub const DEFAULT_SOFTNESS: f64 = 0.3;

fn default_softness() -> f64 {
    DEFAULT_SOFTNESS
}

pub const VERSION: u32 = 1;

/// Where one surface lies in the atlas. `rows` map a position in map units to its 0..1 atlas coordinate,
/// `uv = (dot(rows[0].xyz, p) + rows[0].w, dot(rows[1].xyz, p) + rows[1].w)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Chart {
    pub node: u64,
    /// Face of a brush or mesh, 0 for a terrain.
    pub face: u32,
    pub rows: [[f32; 4]; 2],
}

/// Light probes for objects that move, in the columns Godot's `LightmapGIData` takes them in. Positions and plane
/// distances are in map units, everything else is as Godot stores it: nine RGB spherical harmonics coefficients per
/// point, four point indices per tetrahedron, and per BSP node a plane (normal, distance) with its children over and
/// under it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Probes {
    pub points: Vec<f32>,
    pub sh: Vec<f32>,
    pub tetrahedra: Vec<i32>,
    pub bsp_planes: Vec<f32>,
    pub bsp_children: Vec<i32>,
}

impl Probes {
    pub fn len(&self) -> usize {
        self.points.len() / 3
    }

    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    fn is_consistent(&self) -> bool {
        let (n, nodes) = (self.len(), self.bsp_planes.len() / 4);
        self.points.len() == n * 3
            && self.sh.len() == n * 27
            && self.tetrahedra.len().is_multiple_of(4)
            && self.tetrahedra.iter().all(|i| (*i as usize) < n)
            && self.bsp_planes.len() == nodes * 4
            && self.bsp_children.len() == nodes * 2
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lightmap {
    pub width: u32,
    pub height: u32,
    /// Map units per texel.
    pub texel_size: f32,
    /// Linear light as half floats, red, green and blue per texel, row by row.
    pub light: Vec<u16>,
    /// Share of the sun reaching each texel, 0 to 255.
    pub shadow: Vec<u8>,
    /// Ambient occlusion, 255 where nothing is near.
    pub ao: Vec<u8>,
    /// Sorted by node and face.
    pub charts: Vec<Chart>,
    /// Fingerprint of every baked node's geometry at bake time. A node that changed since has no valid charts.
    pub nodes: BTreeMap<u64, u64>,
    /// Fingerprint of everything the bake depended on, lights and all geometry, to tell when it is out of date.
    pub scene: u64,
    pub probes: Probes,
}

impl Lightmap {
    pub fn texels(&self) -> usize {
        self.width as usize * self.height as usize
    }

    /// The charts of a node, when its geometry is the one that was baked.
    pub fn charts_of(&self, node: u64, fingerprint: u64) -> &[Chart] {
        if self.nodes.get(&node) != Some(&fingerprint) {
            return &[];
        }

        let start = self.charts.partition_point(|c| c.node < node);
        let end = self.charts.partition_point(|c| c.node <= node);
        &self.charts[start..end]
    }

    fn is_consistent(&self) -> bool {
        let n = self.texels();
        self.light.len() == n * 3 && self.shadow.len() == n && self.ao.len() == n && n > 0
    }

    fn chart_columns(&self) -> (Vec<i64>, Vec<f32>) {
        let keys = self.charts.iter().flat_map(|c| [c.node as i64, c.face as i64]).collect();
        let rows = self.charts.iter().flat_map(|c| c.rows.iter().flatten().copied().collect::<Vec<_>>()).collect();
        (keys, rows)
    }

    fn set_columns(&mut self, keys: &[i64], rows: &[f32], nodes: &[i64]) {
        self.charts = keys
            .as_chunks::<2>()
            .0
            .iter()
            .zip(rows.as_chunks::<8>().0)
            .map(|(k, r)| Chart { node: k[0] as u64, face: k[1] as u32, rows: [[r[0], r[1], r[2], r[3]], [r[4], r[5], r[6], r[7]]] })
            .collect();
        self.charts.sort_by_key(|c| (c.node, c.face));
        self.nodes = nodes.as_chunks::<2>().0.iter().map(|p| (p[0] as u64, p[1] as u64)).collect();
    }

    /// Average light of the texels that hold any, for surfaces a mesh has without a chart of their own.
    pub fn fallback_light(&self) -> [f32; 3] {
        let mut sum = [0.0f64; 3];
        let mut n = 0usize;
        for c in self.light.as_chunks::<3>().0 {
            if c.iter().any(|h| *h != 0) {
                for k in 0..3 {
                    sum[k] += f16_to_f32(c[k]) as f64;
                }

                n += 1;
            }
        }

        sum.map(|s| if n > 0 { (s / n as f64) as f32 } else { 0.0 })
    }

    /// Baked nodes of `map` whose geometry changed since the bake, or that are gone. Saved with the bake so the Godot
    /// addon, which cannot fingerprint geometry, leaves their charts out.
    pub fn stale_nodes(&self, map: &Map) -> Vec<i64> {
        self.nodes
            .iter()
            .filter(|(id, print)| map.get(crate::NodeId(**id)).and_then(|n| fingerprint(&n.kind)) != Some(**print))
            .map(|(id, _)| *id as i64)
            .collect()
    }

    /// The payload of the `LMAP` chunk: a Godot Variant dictionary. `stale` is [`Lightmap::stale_nodes`].
    pub fn to_chunk(&self, stale: &[i64]) -> Vec<u8> {
        let (keys, rows) = self.chart_columns();
        let nodes: Vec<i64> = self.nodes.iter().flat_map(|(k, v)| [*k as i64, *v as i64]).collect();
        let mut w = Writer::new();
        w.dict_header(18);
        for (key, value) in [("version", VERSION as i64), ("width", self.width as i64), ("height", self.height as i64), ("scene", self.scene as i64)] {
            w.key(key);
            w.value(&Value::from(value));
        }

        w.key("texel_size");
        w.value(&json!(self.texel_size as f64));
        w.key("light");
        w.bytes(&u16_bytes(&self.light));
        w.key("shadow");
        w.bytes(&self.shadow);
        w.key("ao");
        w.bytes(&self.ao);
        w.key("chart_keys");
        w.int64s(&keys);
        w.key("chart_rows");
        w.float32s(&rows);
        w.key("nodes");
        w.int64s(&nodes);
        w.key("stale");
        w.int64s(stale);
        w.key("fallback");
        w.float32s(&self.fallback_light());
        let p = &self.probes;
        w.key("probe_points");
        w.float32s(&p.points);
        w.key("probe_sh");
        w.float32s(&p.sh);
        w.key("probe_tetrahedra");
        w.int32s(&p.tetrahedra);
        w.key("probe_bsp_planes");
        w.float32s(&p.bsp_planes);
        w.key("probe_bsp_children");
        w.int32s(&p.bsp_children);
        w.out
    }

    pub fn from_chunk(raw: &[u8]) -> Option<Lightmap> {
        Self::from_json(&variant::decode(raw).ok()?)
    }

    /// The `lightmap` key of a JSON map, the byte fields as base64.
    pub fn to_json(&self, stale: &[i64]) -> Value {
        let (keys, rows) = self.chart_columns();
        let nodes: Vec<i64> = self.nodes.iter().flat_map(|(k, v)| [*k as i64, *v as i64]).collect();
        let b64 = |b: &[u8]| base64::engine::general_purpose::STANDARD.encode(b);
        json!({
            "version": VERSION,
            "width": self.width,
            "height": self.height,
            "scene": self.scene as i64,
            "texel_size": self.texel_size,
            "light": b64(&u16_bytes(&self.light)),
            "shadow": b64(&self.shadow),
            "ao": b64(&self.ao),
            "chart_keys": keys,
            "chart_rows": rows,
            "nodes": nodes,
            "stale": stale,
            "fallback": self.fallback_light(),
            "probe_points": self.probes.points,
            "probe_sh": self.probes.sh,
            "probe_tetrahedra": self.probes.tetrahedra,
            "probe_bsp_planes": self.probes.bsp_planes,
            "probe_bsp_children": self.probes.bsp_children,
        })
    }

    /// Reads the JSON layout, which is also what the chunk decodes to with its bytes as base64.
    pub fn from_json(value: &Value) -> Option<Lightmap> {
        let v = value.as_object()?;
        let int = |k: &str| v.get(k).and_then(Value::as_i64);
        let bytes = |k: &str| v.get(k).and_then(Value::as_str).and_then(|s| base64::engine::general_purpose::STANDARD.decode(s).ok());
        let ints = |k: &str| v.get(k).and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_i64).collect::<Vec<_>>());
        if int("version")? > VERSION as i64 {
            return None;
        }

        let floats = |k: &str| v.get(k).and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_f64).map(|f| f as f32).collect::<Vec<_>>());
        let int32s = |k: &str| ints(k).unwrap_or_default().into_iter().filter_map(|i| i32::try_from(i).ok()).collect();
        let rows = floats("chart_rows")?;
        let mut map = Lightmap {
            width: int("width")? as u32,
            height: int("height")? as u32,
            texel_size: v.get("texel_size")?.as_f64()? as f32,
            light: bytes("light")?.as_chunks::<2>().0.iter().map(|b| u16::from_le_bytes(*b)).collect(),
            shadow: bytes("shadow")?,
            ao: bytes("ao")?,
            scene: int("scene").unwrap_or(0) as u64,
            ..Lightmap::default()
        };
        map.set_columns(&ints("chart_keys")?, &rows, &ints("nodes").unwrap_or_default());
        let probes = Probes {
            points: floats("probe_points").unwrap_or_default(),
            sh: floats("probe_sh").unwrap_or_default(),
            tetrahedra: int32s("probe_tetrahedra"),
            bsp_planes: floats("probe_bsp_planes").unwrap_or_default(),
            bsp_children: int32s("probe_bsp_children"),
        };
        // Probes that do not add up are left out, the light map itself still works without them.
        if probes.is_consistent() {
            map.probes = probes;
        }

        map.is_consistent().then_some(map)
    }
}

fn u16_bytes(v: &[u16]) -> Vec<u8> {
    v.iter().flat_map(|x| x.to_le_bytes()).collect()
}

/// IEEE half float bits of `f`, rounded to nearest. Values past the half range become infinity.
pub fn f32_to_f16(f: f32) -> u16 {
    let x = f.to_bits();
    let sign = ((x >> 16) & 0x8000) as u16;
    let exp = ((x >> 23) & 0xff) as i32;
    let man = x & 0x7f_ffff;
    if exp == 0xff {
        return sign | 0x7c00 | if man != 0 { 0x200 } else { 0 };
    }

    let e = exp - 127 + 15;
    if e >= 0x1f {
        return sign | 0x7c00;
    }

    if e <= 0 {
        if e < -10 {
            return sign;
        }

        let m = (man | 0x80_0000) >> (1 - e);
        let round = (m >> 12) & 1;
        return sign | ((m >> 13) + round) as u16;
    }

    let half = sign as u32 | ((e as u32) << 10) | (man >> 13);
    let round = (man >> 12) & 1;
    (half + round) as u16
}

pub fn f16_to_f32(h: u16) -> f32 {
    let sign = ((h & 0x8000) as u32) << 16;
    let exp = ((h >> 10) & 0x1f) as u32;
    let man = (h & 0x3ff) as u32;
    let bits = match exp {
        0 if man == 0 => sign,
        0 => {
            let mut e = 127 - 15 + 1;
            let mut m = man;
            while m & 0x400 == 0 {
                m <<= 1;
                e -= 1;
            }

            sign | (e << 23) | ((m & 0x3ff) << 13)
        }
        0x1f => sign | 0x7f80_0000 | (man << 13),
        _ => sign | ((exp + 127 - 15) << 23) | (man << 13),
    };
    f32::from_bits(bits)
}

/// FNV-1a, stable across runs and Rust versions since fingerprints are saved with the map.
pub struct Fnv(u64);

impl Default for Fnv {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Fnv {
    pub fn bytes(&mut self, b: &[u8]) -> &mut Self {
        for x in b {
            self.0 = (self.0 ^ u64::from(*x)).wrapping_mul(0x0100_0000_01b3);
        }

        self
    }

    pub fn f64(&mut self, v: f64) -> &mut Self {
        self.bytes(&v.to_le_bytes())
    }

    pub fn u32(&mut self, v: u32) -> &mut Self {
        self.bytes(&v.to_le_bytes())
    }

    pub fn finish(&self) -> u64 {
        self.0
    }
}

/// Fingerprint of the geometry a node's charts depend on: moving or reshaping it invalidates them, retexturing does not.
pub fn fingerprint(kind: &NodeKind) -> Option<u64> {
    let mut h = Fnv::default();
    match kind {
        NodeKind::Brush(b) => {
            b.vertices.iter().for_each(|v| {
                h.f64(v.x).f64(v.y).f64(v.z);
            });
            for f in &b.faces {
                h.u32(f.indices.len() as u32);
                f.indices.iter().for_each(|i| {
                    h.u32(*i);
                });
                if let Some(d) = &f.data.disp {
                    h.bytes(format!("{d:?}").as_bytes());
                }
            }
        }
        NodeKind::Mesh(m) => {
            m.vertices.iter().for_each(|v| {
                h.f64(v.x).f64(v.y).f64(v.z);
            });
            for f in &m.faces {
                h.u32(f.indices.len() as u32);
                f.indices.iter().for_each(|i| {
                    h.u32(*i);
                });
            }

            h.f64(m.smooth_angle as f64);
        }
        NodeKind::Terrain(t) => {
            h.f64(t.origin.x).f64(t.origin.y).f64(t.origin.z).f64(t.cell_size).u32(t.resolution[0]).u32(t.resolution[1]);
            t.heights.iter().for_each(|v| {
                h.bytes(&v.to_le_bytes());
            });
            h.bytes(&t.holes);
        }
        _ => return None,
    }

    Some(h.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bake_options_without_softness_get_the_default() {
        let o: BakeOptions = serde_json::from_str(r#"{ "quality": "high" }"#).unwrap();
        assert_eq!(o.softness, DEFAULT_SOFTNESS);
    }

    fn sample() -> Lightmap {
        let mut nodes = BTreeMap::new();
        nodes.insert(7, 0xdead_beef_u64);
        nodes.insert(3, 42);
        Lightmap {
            width: 2,
            height: 2,
            texel_size: 16.0,
            light: [0.0, 0.5, 1.0, 2.5, 100.0, 1e-3, 0.25, 0.75, 1.5, 3.0, 4.0, 5.0].map(f32_to_f16).to_vec(),
            shadow: vec![0, 64, 128, 255],
            ao: vec![255, 200, 100, 0],
            charts: vec![
                Chart { node: 3, face: 1, rows: [[0.1, 0.0, 0.0, 0.5], [0.0, 0.0, -0.1, 0.25]] },
                Chart { node: 7, face: 0, rows: [[0.0, 0.2, 0.0, 0.1], [0.3, 0.0, 0.0, 0.0]] },
            ],
            nodes,
            scene: 99,
            probes: Probes {
                points: vec![0.0, 1.0, 2.0, 64.0, 0.0, 0.0, 0.0, 64.0, 0.0, 0.0, 0.0, 64.0],
                sh: (0..4 * 27).map(|k| k as f32 * 0.25).collect(),
                tetrahedra: vec![0, 1, 2, 3],
                bsp_planes: vec![0.0, 1.0, 0.0, 16.5],
                bsp_children: vec![-1, i32::MIN],
            },
        }
    }

    #[test]
    fn half_floats_round_trip_the_values_a_light_map_holds() {
        for v in [0.0f32, 1.0, 0.5, 2.5, 65504.0, 1e-4, 0.1, 123.456] {
            let back = f16_to_f32(f32_to_f16(v));
            assert!((back - v).abs() <= v.abs() * 1e-3 + 1e-7, "{v} came back as {back}");
        }

        assert_eq!(f16_to_f32(f32_to_f16(1e9)), f32::INFINITY);
        assert_eq!(f32_to_f16(1.0), 0x3c00);
    }

    #[test]
    fn chunk_and_json_round_trip() {
        let map = sample();
        assert_eq!(Lightmap::from_chunk(&map.to_chunk(&[])), Some(map.clone()));
        let text = serde_json::to_string(&map.to_json(&[])).unwrap();
        assert_eq!(Lightmap::from_json(&serde_json::from_str(&text).unwrap()), Some(map));
    }

    #[test]
    fn a_changed_node_has_no_charts() {
        let map = sample();
        assert_eq!(map.charts_of(3, 42).len(), 1);
        assert!(map.charts_of(3, 43).is_empty());
        assert!(map.charts_of(5, 0).is_empty());
    }

    #[test]
    fn a_truncated_payload_is_refused() {
        let mut map = sample();
        map.ao.pop();
        assert_eq!(Lightmap::from_chunk(&map.to_chunk(&[])), None);

        let mut map = sample();
        map.probes.sh.pop();
        let back = Lightmap::from_chunk(&map.to_chunk(&[])).unwrap();
        assert!(back.probes.is_empty() && back.charts == map.charts, "broken probes are dropped, the light map stays");
    }
}
