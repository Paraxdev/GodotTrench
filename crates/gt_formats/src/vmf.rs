//! Valve Hammer `.vmf` import: world and entity solids, displacements, entity I/O and visgroup hidden blocks.
//! VMF is Z-up like Quake, so it shares the id space conversion of the `.map` importer.

use gt_core::{DVec2, DVec3, Plane};
use gt_doc::{Entity, IoConnection, Map, NodeKind};
use gt_geom::displacement::Displacement;
use gt_geom::{Brush, FaceData, FaceUv};

use crate::quake_map::{angles_from_quake, from_id};

#[derive(Debug, thiserror::Error)]
pub enum VmfError {
    #[error("line {0}: {1}")]
    Syntax(usize, String),
}

#[derive(Debug, Default, Clone)]
pub struct Block {
    pub name: String,
    pub props: Vec<(String, String)>,
    pub children: Vec<Block>,
}

impl Block {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.props.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v.as_str())
    }

    pub fn child(&self, name: &str) -> Option<&Block> {
        self.children.iter().find(|c| c.name.eq_ignore_ascii_case(name))
    }

    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Block> + 'a {
        self.children.iter().filter(move |c| c.name.eq_ignore_ascii_case(name))
    }
}

/// KeyValues text: `name { "key" "value" child { ... } }`.
pub fn parse_keyvalues(src: &str) -> Result<Vec<Block>, VmfError> {
    let mut tokens: Vec<(usize, String, bool)> = Vec::new();
    let mut line = 1;
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\n' => line += 1,
            c if c.is_whitespace() => {}
            '/' if chars.peek() == Some(&'/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        line += 1;
                        break;
                    }
                }
            }
            '{' | '}' => tokens.push((line, c.to_string(), false)),
            '"' => {
                let mut s = String::new();
                for c in chars.by_ref() {
                    if c == '"' {
                        break;
                    }
                    if c == '\n' {
                        line += 1;
                    }
                    s.push(c);
                }
                tokens.push((line, s, true));
            }
            _ => {
                let mut s = c.to_string();
                while let Some(&n) = chars.peek() {
                    if n.is_whitespace() || n == '{' || n == '}' || n == '"' {
                        break;
                    }
                    s.push(n);
                    chars.next();
                }
                tokens.push((line, s, false));
            }
        }
    }
    let mut pos = 0;
    fn block_body(tokens: &[(usize, String, bool)], pos: &mut usize, name: String) -> Result<Block, VmfError> {
        let mut block = Block { name, ..Default::default() };
        while *pos < tokens.len() {
            let (line, tok, quoted) = &tokens[*pos];
            if !quoted && tok == "}" {
                *pos += 1;
                return Ok(block);
            }
            let next = tokens.get(*pos + 1);
            match next {
                Some((_, n, false)) if n == "{" => {
                    *pos += 2;
                    let child = block_body(tokens, pos, tok.clone())?;
                    block.children.push(child);
                }
                Some((_, v, _)) => {
                    block.props.push((tok.clone(), v.clone()));
                    *pos += 2;
                }
                None => return Err(VmfError::Syntax(*line, format!("dangling token {tok}"))),
            }
        }
        Ok(block)
    }
    let mut out = Vec::new();
    while pos < tokens.len() {
        let (line, name, _) = tokens[pos].clone();
        if tokens.get(pos + 1).map(|t| t.1.as_str()) != Some("{") {
            return Err(VmfError::Syntax(line, format!("expected {{ after {name}")));
        }
        pos += 2;
        out.push(block_body(&tokens, &mut pos, name)?);
    }
    Ok(out)
}

fn nums(s: &str) -> Vec<f64> {
    s.split(|c: char| c.is_whitespace() || c == '(' || c == ')' || c == '[' || c == ']').filter_map(|p| p.parse().ok()).collect()
}

fn plane_points(s: &str) -> Option<[DVec3; 3]> {
    let n = nums(s);
    (n.len() >= 9).then(|| [DVec3::new(n[0], n[1], n[2]), DVec3::new(n[3], n[4], n[5]), DVec3::new(n[6], n[7], n[8])])
}

/// "[x y z offset] scale"
fn axis(s: Option<&str>) -> Option<(DVec3, f64, f64)> {
    let n = nums(s?);
    (n.len() >= 5).then(|| (DVec3::new(n[0], n[1], n[2]), n[3], n[4]))
}

fn parse_solid(solid: &Block) -> Option<Brush> {
    let mut planes = Vec::new();
    let mut disps: Vec<(Plane, Displacement, DVec3)> = Vec::new();
    for side in solid.children_named("side") {
        let Some([p1, p2, p3]) = side.get("plane").and_then(plane_points) else { continue };
        let Some(plane) = Plane::from_points(from_id(p1), from_id(p3), from_id(p2)) else { continue };
        let material = side.get("material").unwrap_or("").to_ascii_lowercase();
        let (u, ou, su) = axis(side.get("uaxis")).unwrap_or((DVec3::X, 0.0, 0.25));
        let (v, ov, sv) = axis(side.get("vaxis")).unwrap_or((DVec3::NEG_Z, 0.0, 0.25));
        let uv = FaceUv {
            u_axis: from_id(u),
            v_axis: from_id(v),
            offset: DVec2::new(ou, ov),
            scale: DVec2::new(if su == 0.0 { 1.0 } else { su }, if sv == 0.0 { 1.0 } else { sv }),
            rotation: side.get("rotation").and_then(|r| r.parse().ok()).unwrap_or(0.0),
        };
        if let Some(info) = side.child("dispinfo") {
            let power: u8 = info.get("power").and_then(|p| p.parse().ok()).unwrap_or(3).clamp(2, 4);
            let n = (1usize << power) + 1;
            let start = info.get("startposition").map(nums).filter(|v| v.len() >= 3).map(|v| from_id(DVec3::new(v[0], v[1], v[2]))).unwrap_or_default();
            let elevation = info.get("elevation").and_then(|e| e.parse::<f64>().ok()).unwrap_or(0.0);
            let rows = |name: &str| -> Vec<Vec<f64>> {
                let mut out = Vec::new();
                if let Some(b) = info.child(name) {
                    for r in 0..n {
                        out.push(b.get(&format!("row{r}")).map(nums).unwrap_or_default());
                    }
                }
                out
            };
            let normals = rows("normals");
            let distances = rows("distances");
            let offsets = rows("offsets");
            let alphas = rows("alphas");
            let mut disp = Displacement::new(power);
            let face_normal = plane.normal;
            for r in 0..n {
                for c in 0..n {
                    let k = r * n + c;
                    let get3 = |rows: &Vec<Vec<f64>>| {
                        rows.get(r).and_then(|row| (row.len() >= (c + 1) * 3).then(|| from_id(DVec3::new(row[c * 3], row[c * 3 + 1], row[c * 3 + 2]))))
                    };
                    let dir = get3(&normals).unwrap_or(DVec3::ZERO);
                    let dist = distances.get(r).and_then(|row| row.get(c)).copied().unwrap_or(0.0);
                    let offset = get3(&offsets).unwrap_or(DVec3::ZERO);
                    disp.heights[k] = ((dir * dist + offset).dot(face_normal) + elevation) as f32;
                    if let Some(a) = alphas.get(r).and_then(|row| row.get(c)) {
                        if disp.alphas.is_empty() {
                            disp.alphas = vec![0.0; n * n];
                        }
                        disp.alphas[k] = (*a / 255.0).clamp(0.0, 1.0) as f32;
                    }
                }
            }
            disps.push((plane, disp, start));
        }
        planes.push((plane, FaceData::new(material, uv)));
    }
    let mut brush = Brush::from_planes(planes).ok()?;
    for (plane, disp, start) in disps {
        let Some(fi) = brush.find_face_by_plane(&plane) else { continue };
        let face = &mut brush.faces[fi];
        if face.indices.len() != 4 {
            continue;
        }
        let first = (0..4).min_by(|a, b| {
            let da = (brush.vertices[face.indices[*a] as usize] - start).length();
            let db = (brush.vertices[face.indices[*b] as usize] - start).length();
            da.total_cmp(&db)
        })?;
        face.indices.rotate_left(first);
        face.data.disp = Some(disp);
    }
    Some(brush)
}

fn parse_connections(block: Option<&Block>) -> Vec<IoConnection> {
    let Some(block) = block else { return Vec::new() };
    block
        .props
        .iter()
        .filter_map(|(output, value)| {
            let sep = if value.contains('\u{1b}') { '\u{1b}' } else { ',' };
            let parts: Vec<&str> = value.split(sep).collect();
            (parts.len() >= 2).then(|| IoConnection {
                output: output.clone(),
                target: parts[0].to_string(),
                input: parts[1].to_string(),
                parameter: parts.get(2).unwrap_or(&"").to_string(),
                delay: parts.get(3).and_then(|d| d.parse().ok()).unwrap_or(0.0),
                times: parts.get(4).and_then(|t| t.parse().ok()).unwrap_or(-1),
            })
        })
        .collect()
}

fn solids(block: &Block) -> Vec<&Block> {
    let mut out: Vec<&Block> = block.children_named("solid").collect();
    for hidden in block.children_named("hidden") {
        out.extend(hidden.children_named("solid"));
    }
    out
}

/// Imports a `.vmf`. Units are kept as they are (1 Hammer unit = 1 map unit).
pub fn import(src: &str) -> Result<Map, VmfError> {
    let blocks = parse_keyvalues(src)?;
    let mut map = Map::new();
    let layer = map.default_layer();
    let mut entity_blocks: Vec<&Block> = Vec::new();
    for b in &blocks {
        match b.name.to_ascii_lowercase().as_str() {
            "world" => {
                for (k, v) in &b.props {
                    if !matches!(k.as_str(), "id" | "mapversion" | "detailmaterial" | "detailvbsp" | "maxpropscreenwidth" | "skyname") {
                        map.properties.insert(k.clone(), v.clone());
                    }
                }
                if let Some(sky) = b.get("skyname") {
                    map.properties.insert("skyname".into(), sky.into());
                }
                for s in solids(b) {
                    if let Some(brush) = parse_solid(s) {
                        map.insert(layer, NodeKind::Brush(brush));
                    }
                }
            }
            "entity" => entity_blocks.push(b),
            "hidden" => entity_blocks.extend(b.children_named("entity")),
            _ => {}
        }
    }
    map.properties.insert("classname".into(), "worldspawn".into());
    for b in entity_blocks {
        let classname = b.get("classname").unwrap_or("info_null").to_string();
        let mut e = Entity::new(classname);
        let brush_solids = solids(b);
        for (k, v) in &b.props {
            match k.as_str() {
                "classname" | "id" => {}
                "origin" if brush_solids.is_empty() => {
                    let n = nums(v);
                    if n.len() >= 3 {
                        e.origin = from_id(DVec3::new(n[0], n[1], n[2]));
                    }
                }
                "angles" if brush_solids.is_empty() => {
                    let n = nums(v);
                    if n.len() >= 3 {
                        e.angles = angles_from_quake(DVec3::new(n[0], n[1], n[2]));
                    }
                }
                _ => {
                    e.properties.insert(k.clone(), v.clone());
                }
            }
        }
        e.outputs = parse_connections(b.child("connections"));
        let id = map.insert(layer, NodeKind::Entity(e));
        for s in brush_solids {
            if let Some(brush) = parse_solid(s) {
                map.insert(id, NodeKind::Brush(brush));
            }
        }
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
versioninfo { "editorversion" "400" }
world
{
    "id" "1"
    "classname" "worldspawn"
    "skyname" "sky_day01_01"
    solid
    {
        "id" "2"
        side { "id" "1" "plane" "(-64 64 64) (64 64 64) (64 -64 64)" "material" "DEV/DEV_MEASUREGENERIC01B" "uaxis" "[1 0 0 0] 0.25" "vaxis" "[0 -1 0 0] 0.25" }
        side { "id" "2" "plane" "(-64 -64 0) (64 -64 0) (64 64 0)" "material" "DEV/DEV_MEASUREGENERIC01B" "uaxis" "[1 0 0 0] 0.25" "vaxis" "[0 -1 0 0] 0.25" }
        side { "id" "3" "plane" "(-64 64 64) (-64 -64 64) (-64 -64 0)" "material" "TOOLS/TOOLSNODRAW" "uaxis" "[0 1 0 0] 0.25" "vaxis" "[0 0 -1 0] 0.25" }
        side { "id" "4" "plane" "(64 64 0) (64 -64 0) (64 -64 64)" "material" "TOOLS/TOOLSNODRAW" "uaxis" "[0 1 0 0] 0.25" "vaxis" "[0 0 -1 0] 0.25" }
        side { "id" "5" "plane" "(64 64 64) (-64 64 64) (-64 64 0)" "material" "TOOLS/TOOLSNODRAW" "uaxis" "[1 0 0 0] 0.25" "vaxis" "[0 0 -1 0] 0.25" }
        side { "id" "6" "plane" "(64 -64 0) (-64 -64 0) (-64 -64 64)" "material" "TOOLS/TOOLSNODRAW" "uaxis" "[1 0 0 0] 0.25" "vaxis" "[0 0 -1 0] 0.25" }
    }
}
entity
{
    "id" "10"
    "classname" "logic_relay"
    "targetname" "relay"
    "origin" "16 32 48"
    "angles" "0 90 0"
    connections
    {
        "OnTrigger" "door,Open,,0.5,-1"
        "OnTrigger" "lamp,TurnOn,,0,1"
    }
}
"#;

    #[test]
    fn imports_world_brush_and_entity_io() {
        let map = import(SAMPLE).unwrap();
        assert_eq!(map.brush_count(), 1);
        let (_, b) = map.brushes().next().unwrap();
        b.validate().unwrap();
        let bounds = b.bounds();
        assert!((bounds.size() - DVec3::new(128.0, 64.0, 128.0)).length() < 1e-6, "{bounds:?}");
        assert!(b.faces.iter().any(|f| f.data.material == "dev/dev_measuregeneric01b"));
        let (_, e) = map.entities().next().unwrap();
        assert_eq!(e.classname, "logic_relay");
        assert!((e.origin - DVec3::new(32.0, 48.0, 16.0)).length() < 1e-9);
        assert_eq!(e.outputs.len(), 2);
        assert_eq!(e.outputs[0].input, "Open");
        assert!((e.outputs[0].delay - 0.5).abs() < 1e-9);
        assert_eq!(e.outputs[1].times, 1);
    }

    #[test]
    fn imports_displacement_heights() {
        let disp = SAMPLE.replace(
            r#""vaxis" "[0 -1 0 0] 0.25" }
        side { "id" "2""#,
            r#""vaxis" "[0 -1 0 0] 0.25"
            dispinfo
            {
                "power" "2" "startposition" "[-64 -64 64]" "elevation" "0"
                normals { "row0" "0 0 1 0 0 1 0 0 1 0 0 1 0 0 1" "row1" "0 0 1 0 0 1 0 0 1 0 0 1 0 0 1" "row2" "0 0 1 0 0 1 0 0 1 0 0 1 0 0 1" "row3" "0 0 1 0 0 1 0 0 1 0 0 1 0 0 1" "row4" "0 0 1 0 0 1 0 0 1 0 0 1 0 0 1" }
                distances { "row0" "0 0 0 0 0" "row1" "0 0 0 0 0" "row2" "0 0 32 0 0" "row3" "0 0 0 0 0" "row4" "0 0 0 0 0" }
            }
        }
        side { "id" "2""#,
        );
        let map = import(&disp).unwrap();
        let (id, b) = map.brushes().next().unwrap();
        let fi = b.faces.iter().position(|f| f.data.disp.is_some()).expect("displacement face");
        let grid = gt_geom::displacement::grid(map.brush(id).unwrap(), fi).unwrap();
        let top = grid.positions.iter().map(|p| p.y).fold(f64::MIN, f64::max);
        assert!((top - 96.0).abs() < 1e-6, "center raised by 32, got {top}");
    }
}
