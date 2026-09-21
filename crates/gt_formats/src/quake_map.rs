//! Quake `.map` import and export (Standard and Valve 220 formats, TrenchBroom layers and groups).
//! id Tech maps are Z-up, GodotTrench is Y-up: `id = (g.z, g.x, g.y)` and `g = (id.y, id.z, id.x)`.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

use gt_core::{DVec2, DVec3, NodeId, Plane};
use gt_doc::map::{Group, Layer};
use gt_doc::{Entity, Map, NodeKind};
use gt_geom::{Brush, FaceData, FaceUv};

pub fn to_id(v: DVec3) -> DVec3 {
    DVec3::new(v.z, v.x, v.y)
}

pub fn from_id(v: DVec3) -> DVec3 {
    DVec3::new(v.y, v.z, v.x)
}

fn num(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{v:.6}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn vec_str(v: DVec3) -> String {
    format!("{} {} {}", num(v.x), num(v.y), num(v.z))
}

/// GodotTrench angles (Godot rotation degrees) to Quake "angles" as FuncGodot interprets them.
pub fn angles_to_quake(a: DVec3) -> DVec3 {
    DVec3::new(-a.x, a.y - 180.0, -a.z)
}

pub fn angles_from_quake(q: DVec3) -> DVec3 {
    let wrap = |d: f64| {
        let r = (d + 180.0).rem_euclid(360.0) - 180.0;
        if (r + 180.0).abs() < 1e-9 { 180.0 } else { r }
    };
    DVec3::new(wrap(-q.x), wrap(q.y + 180.0), wrap(-q.z))
}

fn write_brush(out: &mut String, brush: &Brush) {
    out.push_str("{\n");
    for face in &brush.faces {
        let pts: Vec<DVec3> = face.indices.iter().map(|i| brush.vertices[*i as usize]).collect();
        // Three points spanning the plane, written in the clockwise order .map files expect.
        let (a, b, c) = (pts[0], pts[1], pts[pts.len() - 1]);
        let (a, b, c) = (to_id(a), to_id(c), to_id(b));
        let uv = &face.data.uv;
        let tex = if face.data.material.is_empty() { "__TB_empty" } else { face.data.material.as_str() };
        let tex = if tex.contains(' ') { format!("\"{tex}\"") } else { tex.to_string() };
        let _ = writeln!(
            out,
            "( {} ) ( {} ) ( {} ) {} [ {} {} ] [ {} {} ] {} {} {}",
            vec_str(a),
            vec_str(b),
            vec_str(c),
            tex,
            vec_str(to_id(uv.u_axis)),
            num(uv.offset.x),
            vec_str(to_id(uv.v_axis)),
            num(uv.offset.y),
            num(uv.rotation),
            num(uv.scale.x),
            num(uv.scale.y)
        );
    }

    out.push_str("}\n");
}

fn write_props(out: &mut String, props: &[(String, String)]) {
    for (k, v) in props {
        let _ = writeln!(out, "\"{}\" \"{}\"", k, v.replace('"', "'"));
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ExportOptions {
    /// Leave out layers marked "omit from export", like TrenchBroom's Export command.
    /// Off keeps them (tagged) so the file round-trips through TrenchBroom losslessly.
    pub skip_omitted_layers: bool,
    /// Leave out objects outside the map's enabled cordon.
    pub cordon: bool,
}

/// Brushes export as they are, convex meshes as their hull. Other meshes and terrains have no `.map` equivalent.
fn exportable(map: &Map, id: NodeId) -> Option<Brush> {
    match &map.get(id)?.kind {
        NodeKind::Brush(b) => Some(b.clone()),
        NodeKind::Mesh(m) if m.is_convex() => m.to_brush().ok(),
        _ => None,
    }
}

/// Exports a map as a Valve 220 `.map` readable by TrenchBroom and FuncGodot.
pub fn export(map: &Map) -> String {
    export_with(map, ExportOptions::default())
}

pub fn export_with(map: &Map, options: ExportOptions) -> String {
    let mut map = map.clone();
    if options.skip_omitted_layers {
        let omitted: Vec<NodeId> =
            map.layers.iter().copied().filter(|l| matches!(map.get(*l).map(|n| &n.kind), Some(NodeKind::Layer(layer)) if layer.omit_from_export)).collect();
        for l in omitted {
            map.remove(l);
        }
    }

    if options.cordon {
        let outside: Vec<NodeId> = map
            .nodes
            .iter()
            .filter(|(id, n)| (n.kind.is_geometry() || n.children.is_empty() && n.entity().is_some()) && !map.in_cordon(**id))
            .map(|(id, _)| *id)
            .collect();
        for id in outside {
            map.remove(id);
        }
    }

    let map = &map;
    let mut out = String::from("// Game: Godot\n// Format: Valve\n// Exported by GodotTrench\n");
    let mut tb_ids: HashMap<NodeId, u64> = HashMap::new();
    let mut next_id = 1u64;
    for id in map.walk() {
        if matches!(map.get(id).map(|n| &n.kind), Some(NodeKind::Layer(_) | NodeKind::Group(_))) && id != map.default_layer() {
            tb_ids.insert(id, next_id);
            next_id += 1;
        }
    }

    // The nearest enclosing TB container (group or non-default layer) of a node.
    let container = |id: NodeId| -> Option<(String, String)> {
        let parent = map.get(id)?.parent?;
        let pid = tb_ids.get(&parent)?.to_string();
        match map.get(parent)?.kind {
            NodeKind::Layer(_) => Some(("_tb_layer".into(), pid)),
            _ => Some(("_tb_group".into(), pid)),
        }
    };

    // Worldspawn with brushes that live directly in the default layer.
    out.push_str("// entity 0\n{\n");
    let mut world_props: Vec<(String, String)> = vec![("classname".into(), "worldspawn".into()), ("mapversion".into(), "220".into())];
    world_props.extend(map.properties.iter().filter(|(k, _)| *k != "classname" && *k != "mapversion").map(|(k, v)| (k.clone(), v.clone())));
    write_props(&mut out, &world_props);
    for id in map.get(map.default_layer()).map(|n| n.children.clone()).unwrap_or_default() {
        if let Some(brush) = exportable(map, id) {
            write_brush(&mut out, &brush);
        }
    }

    out.push_str("}\n");

    let mut entity_index = 1;
    for id in map.walk() {
        let Some(node) = map.get(id) else { continue };
        match &node.kind {
            NodeKind::Layer(l) if tb_ids.contains_key(&id) => {
                let mut props = vec![
                    ("classname".to_string(), "func_group".to_string()),
                    ("_tb_type".into(), "_tb_layer".into()),
                    ("_tb_name".into(), l.name.clone()),
                    ("_tb_id".into(), tb_ids[&id].to_string()),
                ];
                if l.omit_from_export {
                    props.push(("_tb_layer_omit_from_export".into(), "1".into()));
                }

                let _ = writeln!(out, "// entity {entity_index}\n{{");
                write_props(&mut out, &props);
                for c in &node.children {
                    if let Some(b) = exportable(map, *c) {
                        write_brush(&mut out, &b);
                    }
                }

                out.push_str("}\n");
                entity_index += 1;
            }
            NodeKind::Group(g) => {
                let mut props = vec![
                    ("classname".to_string(), "func_group".to_string()),
                    ("_tb_type".into(), "_tb_group".into()),
                    ("_tb_name".into(), g.name.clone()),
                    ("_tb_id".into(), tb_ids[&id].to_string()),
                ];
                props.extend(container(id));
                let _ = writeln!(out, "// entity {entity_index}\n{{");
                write_props(&mut out, &props);
                for c in &node.children {
                    if let Some(b) = exportable(map, *c) {
                        write_brush(&mut out, &b);
                    }
                }

                out.push_str("}\n");
                entity_index += 1;
            }
            NodeKind::Entity(e) => {
                let mut props = vec![("classname".to_string(), e.classname.clone())];
                if node.children.is_empty() {
                    props.push(("origin".into(), vec_str(to_id(e.origin))));
                    if e.angles != DVec3::ZERO || !e.properties.contains_key("angles") {
                        props.push(("angles".into(), vec_str(angles_to_quake(e.angles))));
                    }
                }

                props.extend(e.properties.iter().filter(|(k, _)| k.as_str() != "origin" && k.as_str() != "angles").map(|(k, v)| (k.clone(), v.clone())));
                props.extend(container(id));
                let _ = writeln!(out, "// entity {entity_index}\n{{");
                write_props(&mut out, &props);
                for c in &node.children {
                    if let Some(b) = exportable(map, *c) {
                        write_brush(&mut out, &b);
                    }
                }

                out.push_str("}\n");
                entity_index += 1;
            }
            _ => {}
        }
    }

    out
}

#[derive(Debug, thiserror::Error)]
pub enum MapError {
    #[error("line {line}: {message}")]
    Syntax { line: usize, message: String },
}

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    src: &'a str,
    line: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Open,
    Close,
    ParenOpen,
    ParenClose,
    BracketOpen,
    BracketClose,
    Str(String),
    Word(String),
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self { chars: src.char_indices().peekable(), src, line: 1 }
    }

    fn next(&mut self) -> Option<Tok> {
        loop {
            let (i, c) = *self.chars.peek()?;
            if c == '\n' {
                self.line += 1;
                self.chars.next();
            } else if c.is_whitespace() {
                self.chars.next();
            } else if c == '/' && self.src[i..].starts_with("//") {
                while let Some((_, c)) = self.chars.peek() {
                    if *c == '\n' {
                        break;
                    }

                    self.chars.next();
                }
            } else {
                break;
            }
        }

        let (start, c) = self.chars.next()?;
        Some(match c {
            '{' => Tok::Open,
            '}' => Tok::Close,
            '(' => Tok::ParenOpen,
            ')' => Tok::ParenClose,
            '[' => Tok::BracketOpen,
            ']' => Tok::BracketClose,
            '"' => {
                let mut s = String::new();
                while let Some((_, c)) = self.chars.next() {
                    match c {
                        '"' => break,
                        '\\' if self.chars.peek().is_some_and(|(_, n)| *n == '"') => {
                            s.push('"');
                            self.chars.next();
                        }
                        '\n' => {
                            self.line += 1;
                            s.push(c);
                        }
                        _ => s.push(c),
                    }
                }

                Tok::Str(s)
            }
            _ => {
                let mut end = start + c.len_utf8();
                while let Some((i, c)) = self.chars.peek() {
                    if c.is_whitespace() || matches!(c, '{' | '}' | '(' | ')' | '[' | ']' | '"') {
                        break;
                    }

                    end = *i + c.len_utf8();
                    self.chars.next();
                }

                Tok::Word(self.src[start..end].to_string())
            }
        })
    }
}

struct RawEntity {
    props: Vec<(String, String)>,
    brushes: Vec<Brush>,
}

// qbsp's paraxial base axes in id space: (normal, u, v).
const BASE_AXES: [([f64; 3], [f64; 3], [f64; 3]); 6] = [
    ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, -1.0, 0.0]),
    ([0.0, 0.0, -1.0], [1.0, 0.0, 0.0], [0.0, -1.0, 0.0]),
    ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]),
    ([-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, -1.0]),
    ([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]),
    ([0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]),
];

/// Converts a Standard format texture projection into Valve style axes (id space).
fn standard_axes(id_normal: DVec3, rotation: f64) -> (DVec3, DVec3) {
    let best = BASE_AXES.iter().max_by(|a, b| DVec3::from(a.0).dot(id_normal).total_cmp(&DVec3::from(b.0).dot(id_normal))).unwrap();
    let (mut u, mut v) = (DVec3::from(best.1), DVec3::from(best.2));
    let (s, c) = rotation.to_radians().sin_cos();
    let sv = if u.x != 0.0 {
        0
    } else if u.y != 0.0 {
        1
    } else {
        2
    };
    let tv = if v.x != 0.0 {
        0
    } else if v.y != 0.0 {
        1
    } else {
        2
    };
    for vec in [&mut u, &mut v] {
        let (ns, nt) = (c * vec[sv] - s * vec[tv], s * vec[sv] + c * vec[tv]);
        vec[sv] = ns;
        vec[tv] = nt;
    }

    (u, v)
}

fn parse_f(tok: Option<Tok>, line: usize) -> Result<f64, MapError> {
    match tok {
        Some(Tok::Word(w)) => w.parse().map_err(|_| MapError::Syntax { line, message: format!("expected number, got {w}") }),
        other => Err(MapError::Syntax { line, message: format!("expected number, got {other:?}") }),
    }
}

fn parse_brush(lx: &mut Lexer) -> Result<Option<Brush>, MapError> {
    let mut planes: Vec<(Plane, FaceData)> = Vec::new();
    loop {
        let line = lx.line;
        match lx.next() {
            Some(Tok::Close) => break,
            Some(Tok::ParenOpen) => {
                let mut pts = [DVec3::ZERO; 3];
                for (k, p) in pts.iter_mut().enumerate() {
                    if k > 0 && lx.next() != Some(Tok::ParenOpen) {
                        return Err(MapError::Syntax { line, message: "expected (".into() });
                    }

                    *p = DVec3::new(parse_f(lx.next(), line)?, parse_f(lx.next(), line)?, parse_f(lx.next(), line)?);
                    if lx.next() != Some(Tok::ParenClose) {
                        return Err(MapError::Syntax { line, message: "expected )".into() });
                    }
                }

                let texture = match lx.next() {
                    Some(Tok::Word(w)) | Some(Tok::Str(w)) => w,
                    other => return Err(MapError::Syntax { line, message: format!("expected texture, got {other:?}") }),
                };
                let (p1, p2, p3) = (pts[0], pts[1], pts[2]);
                let Some(id_plane) = Plane::from_points(p1, p3, p2) else { continue };
                let plane =
                    Plane::from_points(from_id(p1), from_id(p3), from_id(p2)).unwrap_or(Plane::from_point_normal(from_id(p1), from_id(id_plane.normal)));
                let mut lookahead = lx.next();
                let uv = if lookahead == Some(Tok::BracketOpen) {
                    let u = DVec3::new(parse_f(lx.next(), line)?, parse_f(lx.next(), line)?, parse_f(lx.next(), line)?);
                    let ou = parse_f(lx.next(), line)?;
                    lx.next();
                    lx.next();
                    let v = DVec3::new(parse_f(lx.next(), line)?, parse_f(lx.next(), line)?, parse_f(lx.next(), line)?);
                    let ov = parse_f(lx.next(), line)?;
                    lx.next();
                    let rotation = parse_f(lx.next(), line)?;
                    let sx = parse_f(lx.next(), line)?;
                    let sy = parse_f(lx.next(), line)?;
                    lookahead = None;
                    FaceUv { u_axis: from_id(u), v_axis: from_id(v), offset: DVec2::new(ou, ov), scale: DVec2::new(sx, sy), rotation }
                } else {
                    let ou = parse_f(lookahead.take(), line)?;
                    let ov = parse_f(lx.next(), line)?;
                    let rotation = parse_f(lx.next(), line)?;
                    let sx = parse_f(lx.next(), line)?;
                    let sy = parse_f(lx.next(), line)?;
                    let (u, v) = standard_axes(id_plane.normal, rotation);
                    FaceUv {
                        u_axis: from_id(u),
                        v_axis: from_id(v),
                        offset: DVec2::new(ou, ov),
                        scale: DVec2::new(if sx == 0.0 { 1.0 } else { sx }, if sy == 0.0 { 1.0 } else { sy }),
                        rotation,
                    }
                };
                // Quake 2 / 3 surface flags trail the face line, skip up to the next face or the brush end.
                let _ = lookahead;
                let material = if texture == "__TB_empty" { String::new() } else { texture };
                planes.push((plane, FaceData::new(material, uv)));
            }
            Some(Tok::Word(_)) => {}
            None => return Err(MapError::Syntax { line, message: "unexpected end of file in brush".into() }),
            Some(other) => return Err(MapError::Syntax { line, message: format!("unexpected {other:?} in brush") }),
        }
    }

    Ok(Brush::from_planes(planes).ok())
}

fn parse_entities(src: &str) -> Result<Vec<RawEntity>, MapError> {
    let mut lx = Lexer::new(src);
    let mut out = Vec::new();
    while let Some(tok) = lx.next() {
        if tok != Tok::Open {
            return Err(MapError::Syntax { line: lx.line, message: format!("expected {{, got {tok:?}") });
        }

        let mut ent = RawEntity { props: Vec::new(), brushes: Vec::new() };
        loop {
            let line = lx.line;
            match lx.next() {
                Some(Tok::Close) => break,
                Some(Tok::Str(k)) => match lx.next() {
                    Some(Tok::Str(v)) => ent.props.push((k, v)),
                    other => return Err(MapError::Syntax { line, message: format!("expected value for {k}, got {other:?}") }),
                },
                Some(Tok::Open) => {
                    if let Some(b) = parse_brush(&mut lx)? {
                        ent.brushes.push(b);
                    }
                }
                None => return Err(MapError::Syntax { line, message: "unexpected end of file in entity".into() }),
                Some(other) => return Err(MapError::Syntax { line, message: format!("unexpected {other:?}") }),
            }
        }

        out.push(ent);
    }

    Ok(out)
}

fn vec_prop(s: &str) -> Option<DVec3> {
    let p: Vec<f64> = s.split_whitespace().filter_map(|x| x.parse().ok()).collect();
    (p.len() >= 3).then(|| DVec3::new(p[0], p[1], p[2]))
}

/// Imports a Quake `.map`. TrenchBroom layers and groups become GodotTrench layers and groups.
pub fn import(src: &str) -> Result<Map, MapError> {
    let entities = parse_entities(src)?;
    let mut map = Map::new();
    let default_layer = map.default_layer();
    let mut containers: BTreeMap<String, NodeId> = BTreeMap::new();
    let mut pending_parent: Vec<(NodeId, String)> = Vec::new();

    // Layers and groups first so everything can be attached to them.
    for ent in &entities {
        let get = |k: &str| ent.props.iter().find(|(pk, _)| pk == k).map(|(_, v)| v.as_str());
        if get("classname") != Some("func_group") {
            continue;
        }

        let Some(tb_id) = get("_tb_id") else { continue };
        let name = get("_tb_name").unwrap_or("Unnamed").to_string();
        let id = match get("_tb_type") {
            Some("_tb_layer") => {
                let l = map.add_layer(&name);
                if let Some(NodeKind::Layer(layer)) = map.get_mut(l).map(|n| &mut n.kind) {
                    *layer = Layer { name, color: layer.color, omit_from_export: get("_tb_layer_omit_from_export") == Some("1") };
                }

                l
            }
            _ => {
                let g = map.insert(default_layer, NodeKind::Group(Group::new(name)));
                if let Some(parent) = get("_tb_group").or(get("_tb_layer")) {
                    pending_parent.push((g, parent.to_string()));
                }

                g
            }
        };
        containers.insert(tb_id.to_string(), id);
        for b in &ent.brushes {
            map.insert(id, NodeKind::Brush(b.clone()));
        }
    }

    for (node, parent) in pending_parent {
        if let Some(p) = containers.get(&parent) {
            map.reparent(node, *p);
        }
    }

    for ent in &entities {
        let get = |k: &str| ent.props.iter().find(|(pk, _)| pk == k).map(|(_, v)| v.as_str());
        let classname = get("classname").unwrap_or("").to_string();
        if classname == "func_group" && get("_tb_id").is_some() {
            continue;
        }

        let parent = get("_tb_group").or(get("_tb_layer")).and_then(|id| containers.get(id).copied()).unwrap_or(default_layer);
        if classname == "worldspawn" {
            for (k, v) in &ent.props {
                if k != "mapversion" && !k.starts_with("_tb_") {
                    map.properties.insert(k.clone(), v.clone());
                }
            }

            for b in &ent.brushes {
                map.insert(default_layer, NodeKind::Brush(b.clone()));
            }

            continue;
        }

        let mut e = Entity::new(classname);
        for (k, v) in &ent.props {
            match k.as_str() {
                "classname" | "_tb_group" | "_tb_layer" => {}
                "origin" if ent.brushes.is_empty() => e.origin = vec_prop(v).map(from_id).unwrap_or_default(),
                "angles" | "mangle" if ent.brushes.is_empty() => e.angles = vec_prop(v).map(angles_from_quake).unwrap_or_default(),
                "angle" if ent.brushes.is_empty() => {
                    let a: f64 = v.parse().unwrap_or(0.0);
                    e.angles = if a == -1.0 {
                        DVec3::new(90.0, 0.0, 0.0)
                    } else if a == -2.0 {
                        DVec3::new(-90.0, 0.0, 0.0)
                    } else {
                        angles_from_quake(DVec3::new(0.0, a, 0.0))
                    };
                }
                _ => {
                    e.properties.insert(k.clone(), v.clone());
                }
            }
        }

        let id = map.insert(parent, NodeKind::Entity(e));
        for b in &ent.brushes {
            map.insert(id, NodeKind::Brush(b.clone()));
        }
    }

    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_core::Aabb;
    use gt_doc::ops;

    fn sample() -> Map {
        let mut m = Map::new();
        m.properties.insert("message".into(), "hello".into());
        let layer = m.default_layer();
        let floor = Brush::from_aabb(&Aabb::new(DVec3::new(-64.0, -16.0, -64.0), DVec3::new(64.0, 0.0, 64.0)), "base/floor").unwrap();
        m.insert(layer, NodeKind::Brush(floor.clone()));
        let rotated = floor.transformed(&ops::rotation_about(DVec3::ZERO, DVec3::Y, 30.0), true).translated(DVec3::new(0.0, 64.0, 0.0), true);
        let details = m.add_layer("Details");
        let g = m.insert(details, NodeKind::Group(Group::new("crates")));
        m.insert(g, NodeKind::Brush(rotated));
        let mut door = Entity::new("func_door");
        door.properties.insert("targetname".into(), "door1".into());
        let d = m.insert(layer, NodeKind::Entity(door));
        m.insert(d, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::new(0.0, 0.0, -8.0), DVec3::new(32.0, 64.0, 8.0)), "base/metal").unwrap()));
        let mut light = Entity::new("light");
        light.origin = DVec3::new(8.0, 96.0, -24.0);
        light.angles = DVec3::new(10.0, 45.0, 0.0);
        m.insert(g, NodeKind::Entity(light));
        m
    }

    #[test]
    fn round_trip_preserves_geometry_and_entities() {
        let original = sample();
        let text = export(&original);
        assert!(text.contains("\"_tb_type\" \"_tb_layer\"") && text.contains("\"_tb_type\" \"_tb_group\""));
        let back = import(&text).unwrap();
        assert_eq!(back.brush_count(), original.brush_count());
        assert_eq!(back.entity_count(), original.entity_count());
        assert_eq!(back.properties.get("message").map(String::as_str), Some("hello"));

        let total = |m: &Map| m.bounds_of(m.layers.clone());
        let (a, b) = (total(&original), total(&back));
        assert!(gt_core::vec_approx_eq(a.min, b.min) && gt_core::vec_approx_eq(a.max, b.max), "{a:?} {b:?}");
        let vol = |m: &Map| m.brushes().map(|(_, b)| b.volume()).sum::<f64>();
        // Points are written with 6 decimals, rotated brushes lose a few millionths of a unit.
        let (vo, vb) = (vol(&original), vol(&back));
        assert!((vo - vb).abs() / vo < 1e-6, "volume {vo} vs {vb}");

        let light = back.entities().find(|(_, e)| e.classname == "light").unwrap();
        assert!(gt_core::vec_approx_eq(light.1.origin, DVec3::new(8.0, 96.0, -24.0)));
        assert!((light.1.angles - DVec3::new(10.0, 45.0, 0.0)).length() < 1e-6, "{:?}", light.1.angles);
        assert!(matches!(back.get(back.get(light.0).unwrap().parent.unwrap()).map(|n| &n.kind), Some(NodeKind::Group(_))), "light stays in its group");

        // Texture coordinates survive: compare texels at every vertex of the rotated brush.
        let find = |m: &Map| m.brushes().find(|(_, b)| b.bounds().min.y > 40.0).map(|(_, b)| b.clone()).unwrap();
        let (bo, bb) = (find(&original), find(&back));
        for fo in &bo.faces {
            let fb = bb.faces.iter().find(|f| f.plane.approx_eq(&fo.plane, 1e-6, 1e-3)).expect("matching face");
            for i in &fo.indices {
                let p = bo.vertices[*i as usize];
                assert!((fo.data.uv.texel(p) - fb.data.uv.texel(p)).length() < 1e-3);
            }
        }
    }

    #[test]
    fn imports_standard_format_and_angles() {
        let src = r#"
// Game: Quake
{
"classname" "worldspawn"
{
( -64 -64 -16 ) ( -64 -63 -16 ) ( -64 -64 -15 ) wall 0 0 0 1 1
( -64 -64 -16 ) ( -64 -64 -15 ) ( -63 -64 -16 ) wall 0 0 0 1 1
( -64 -64 -16 ) ( -63 -64 -16 ) ( -64 -63 -16 ) floor 16 8 45 0.5 0.5
( 64 64 16 ) ( 64 65 16 ) ( 65 64 16 ) floor 0 0 0 1 1
( 64 64 16 ) ( 65 64 16 ) ( 64 64 17 ) wall 0 0 0 1 1
( 64 64 16 ) ( 64 64 17 ) ( 64 65 16 ) wall 0 0 0 1 1
}
}
{
"classname" "info_player_start"
"origin" "32 0 24"
"angle" "90"
}
"#;
        let m = import(src).unwrap();
        assert_eq!(m.brush_count(), 1);
        let (_, b) = m.brushes().next().unwrap();
        let bounds = b.bounds();
        assert!(gt_core::vec_approx_eq(bounds.min, DVec3::new(-64.0, -16.0, -64.0)) && gt_core::vec_approx_eq(bounds.max, DVec3::new(64.0, 16.0, 64.0)));
        let bottom = b.faces.iter().find(|f| f.plane.normal.y < -0.5).unwrap();
        assert_eq!(bottom.data.material, "floor");
        assert_eq!(bottom.data.uv.scale, DVec2::new(0.5, 0.5));
        let (_, start) = m.entities().next().unwrap();
        assert!(gt_core::vec_approx_eq(start.origin, DVec3::new(0.0, 24.0, 32.0)));
        // Quake angle 90 faces +Y (id), which is +X in GodotTrench.
        let facing = start.rotation() * DVec3::NEG_Z;
        assert!(gt_core::vec_approx_eq(facing, DVec3::X), "{facing}");
    }
}
