//! Valve Hammer `.vmf` import: world and entity solids, displacements, entity I/O, visgroups, hidden objects and
//! `func_instance` maps, which are inlined as groups. VMF is Z-up like Quake, so it shares the id space conversion of
//! the `.map` importer.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use gt_core::{DMat3, DMat4, DVec2, DVec3, NodeId, Plane};
use gt_doc::map::Group;
use gt_doc::{Entity, IoConnection, Map, NodeKind};
use gt_geom::displacement::Displacement;
use gt_geom::mesh::{Mesh, MeshFace};
use gt_geom::{Brush, FaceData, FaceUv};

use crate::game::ToolTextures;
use crate::quake_map::{angles_from_quake, from_id, to_id};

#[derive(Debug, thiserror::Error)]
pub enum VmfError {
    #[error("line {0}: {1}")]
    Syntax(usize, String),
    #[error("{0}: {1}")]
    Io(String, std::io::Error),
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

/// Reads a Valve text file. Hammer writes the system code page, so text that is not UTF-8 is read as Latin-1.
pub fn read_text(path: &Path) -> Result<String, VmfError> {
    let bytes = std::fs::read(path).map_err(|e| VmfError::Io(path.display().to_string(), e))?;
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(&bytes);
    Ok(match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|b| *b as char).collect(),
    })
}

/// KeyValues text: `name { "key" "value" child { ... } }`.
pub fn parse_keyvalues(src: &str) -> Result<Vec<Block>, VmfError> {
    let src = src.strip_prefix('\u{feff}').unwrap_or(src);
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

fn vec3(s: &str) -> Option<DVec3> {
    let n = nums(s);
    (n.len() >= 3).then(|| DVec3::new(n[0], n[1], n[2]))
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

pub struct ImportOptions {
    /// The project's tool textures. Valve tool materials such as `tools/toolsnodraw` are renamed to them, so the faces
    /// they cover build the way they did in Source.
    pub tools: ToolTextures,
    /// The material brush entities use for trigger volumes.
    pub trigger: String,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self { tools: ToolTextures::default(), trigger: "special/trigger".into() }
    }
}

impl ImportOptions {
    /// The face material for a lowercased VMF material name.
    fn material(&self, name: &str) -> String {
        let Some(tool) = name.strip_prefix("tools/") else { return name.to_string() };
        match tool {
            "toolsnodraw" | "toolsskybox" | "toolsskybox2d" | "toolsskip" | "toolshint" | "toolsareaportal" | "toolsoccluder" | "toolsblocklight"
            | "toolsblock_los" | "toolsfog" | "toolsblack" => self.tools.skip.clone(),
            "toolsclip"
            | "toolsplayerclip"
            | "toolsnpcclip"
            | "toolscontrolclip"
            | "toolsgrenadeclip"
            | "toolsdotaclip"
            | "toolsinvisible"
            | "toolsinvisibleladder"
            | "toolsblockbullets"
            | "toolsblockbullets2" => self.tools.clip.clone(),
            "toolsorigin" => self.tools.origin.clone(),
            "toolstrigger" => self.trigger.clone(),
            _ => name.to_string(),
        }
    }
}

/// What an import found beyond the map itself.
#[derive(Debug, Default)]
pub struct ImportReport {
    /// `func_instance` maps that were inlined, nested ones included.
    pub instances: usize,
    /// `file` values of instances whose map could not be read. Those stay `func_instance` entities.
    pub missing_instances: Vec<String>,
}

/// A solid as a brush, or as meshes for displacements that move sideways.
fn parse_solid(solid: &Block, options: &ImportOptions) -> Option<Vec<NodeKind>> {
    let mut planes = Vec::new();
    let mut disps: Vec<(Plane, Displacement, DVec3, Vec<DVec3>)> = Vec::new();
    for side in solid.children_named("side") {
        let Some([p1, p2, p3]) = side.get("plane").and_then(plane_points) else { continue };
        let Some(plane) = Plane::from_points(from_id(p1), from_id(p3), from_id(p2)) else { continue };
        let material = options.material(&side.get("material").unwrap_or("").to_ascii_lowercase());
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
            let start = info.get("startposition").and_then(vec3).map(from_id).unwrap_or_default();
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
            let mut exact = vec![DVec3::ZERO; n * n];
            let face_normal = plane.normal;
            for r in 0..n {
                for c in 0..n {
                    // Source winds faces clockwise and walks a row from corner 0 towards its corner 1, which is our
                    // counter-clockwise corner 3, the v direction of grid(). So VMF rows are our v rows.
                    let k = r * n + c;
                    let get3 = |rows: &Vec<Vec<f64>>| {
                        rows.get(r).and_then(|row| (row.len() >= (c + 1) * 3).then(|| from_id(DVec3::new(row[c * 3], row[c * 3 + 1], row[c * 3 + 2]))))
                    };
                    let dir = get3(&normals).unwrap_or(DVec3::ZERO);
                    let dist = distances.get(r).and_then(|row| row.get(c)).copied().unwrap_or(0.0);
                    let offset = get3(&offsets).unwrap_or(DVec3::ZERO);
                    exact[k] = dir * dist + offset + face_normal * elevation;
                    disp.heights[k] = exact[k].dot(face_normal) as f32;
                    if let Some(a) = alphas.get(r).and_then(|row| row.get(c)) {
                        if disp.alphas.is_empty() {
                            disp.alphas = vec![0.0; n * n];
                        }

                        disp.alphas[k] = (*a / 255.0).clamp(0.0, 1.0) as f32;
                    }
                }
            }

            disps.push((plane, disp, start, exact));
        }

        planes.push((plane, FaceData::new(material, uv)));
    }

    let mut brush = Brush::from_planes(planes).ok()?;
    let mut sideways = false;
    let mut exact_offsets = Vec::new();
    for (plane, disp, start, exact) in disps {
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
        sideways |= exact.iter().zip(&disp.heights).any(|(e, h)| (*e - plane.normal * *h as f64).length() > SIDEWAYS_TOLERANCE);
        face.data.disp = Some(disp);
        exact_offsets.push((fi, exact));
    }

    if sideways {
        return Some(exact_offsets.into_iter().filter_map(|(fi, exact)| displacement_mesh(&brush, fi, &exact)).map(NodeKind::Mesh).collect());
    }

    Some(vec![NodeKind::Brush(brush)])
}

/// Map units a displacement vertex may move off its face normal and still import as a displacement.
const SIDEWAYS_TOLERANCE: f64 = 1.0;

/// A displacement whose vertices move sideways, which the height grid of GodotTrench displacements cannot hold,
/// as a smooth triangle mesh with the exact Hammer positions. Its texture keeps the face projection.
fn displacement_mesh(brush: &Brush, face: usize, exact: &[DVec3]) -> Option<Mesh> {
    let grid = gt_geom::displacement::grid(brush, face)?;
    let data = FaceData { disp: None, ..brush.faces[face].data.clone() };
    let vertices = grid.base.iter().zip(exact).map(|(b, e)| *b + *e).collect();
    let faces = gt_geom::displacement::triangles(grid.size).map(|(a, b, c)| MeshFace::new(vec![a as u32, b as u32, c as u32], data.clone())).collect();
    Some(Mesh { vertices, faces, smooth_angle: 75.0, decal: false })
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

/// Solids of a world or entity block, with whether Hammer hides them.
fn solids(block: &Block) -> Vec<(&Block, bool)> {
    let mut out: Vec<(&Block, bool)> = block.children_named("solid").map(|s| (s, hidden_by_visgroup(s))).collect();
    for hidden in block.children_named("hidden") {
        out.extend(hidden.children_named("solid").map(|s| (s, true)));
    }

    out
}

fn hidden_by_visgroup(b: &Block) -> bool {
    b.child("editor").and_then(|e| e.get("visgroupshown")).is_some_and(|v| v == "0")
}

/// Entity keys that name another entity and so get an instance's fixup.
const FIXUP_KEYS: [&str; 12] = [
    "targetname",
    "parentname",
    "target",
    "filtername",
    "damagefilter",
    "lightingorigin",
    "measuretarget",
    "attach1",
    "attach2",
    "sourceentityname",
    "destination",
    "call_target",
];

#[derive(Clone)]
struct Fixup {
    name: String,
    /// `fixup_style`: 0 prefix, 1 postfix, 2 none.
    style: u8,
}

impl Fixup {
    fn apply(&self, name: &str) -> String {
        if name.is_empty() || name.starts_with(['!', '@']) {
            return name.to_string();
        }

        match self.style {
            0 => format!("{}-{name}", self.name),
            1 => format!("{name}-{}", self.name),
            _ => name.to_string(),
        }
    }
}

/// Maps Hammer (id space) to GodotTrench space, where `from_id` is a fixed axis permutation.
fn id_to_godot(m: DMat4) -> DMat4 {
    let p = DMat4::from_mat3(DMat3::from_cols(from_id(DVec3::X), from_id(DVec3::Y), from_id(DVec3::Z)));
    p * m * p.inverse()
}

/// Placement of a `func_instance` in GodotTrench space. Hammer's angles are pitch around Y, yaw around Z and roll
/// around X, applied as yaw, then pitch, then roll.
fn instance_transform(origin: DVec3, angles: DVec3) -> DMat4 {
    let [pitch, yaw, roll] = [angles.x.to_radians(), angles.y.to_radians(), angles.z.to_radians()];
    let rot = DMat4::from_rotation_z(yaw) * DMat4::from_rotation_y(pitch) * DMat4::from_rotation_x(roll);
    id_to_godot(DMat4::from_translation(origin) * rot)
}

/// Where Hammer looks for an instance: next to the map, then up the folders, since instance paths are often relative
/// to the game's map source folder rather than to the map.
fn resolve_instance(file: &str, dir: Option<&Path>) -> Option<PathBuf> {
    let file = file.replace('\\', "/");
    let rel = Path::new(&file);
    if rel.is_absolute() {
        return rel.is_file().then(|| rel.to_path_buf());
    }

    let mut candidates = vec![rel.to_path_buf()];
    if rel.extension().is_none() {
        candidates.push(rel.with_extension("vmf"));
    }

    let mut dir = dir?;
    loop {
        for c in &candidates {
            let p = dir.join(c);
            if p.is_file() {
                return Some(p);
            }
        }

        dir = dir.parent()?;
    }
}

/// `$variable` replacements of an instance: its own `replaceNN` keys, then the defaults of the instance map's
/// `func_instance_parms`. Longest first, so `$a` never eats the start of `$ab`.
fn instance_replacements(instance: &Block, blocks: &[Block]) -> Vec<(String, String)> {
    let mut vars: BTreeMap<String, String> = BTreeMap::new();
    let parms = blocks.iter().filter(|b| b.name.eq_ignore_ascii_case("entity") && b.get("classname") == Some("func_instance_parms"));
    for p in parms {
        for (k, v) in &p.props {
            if !k.to_ascii_lowercase().starts_with("parm") {
                continue;
            }

            let mut parts = v.splitn(3, char::is_whitespace);
            if let (Some(var), Some(_ty), Some(default)) = (parts.next(), parts.next(), parts.next()) {
                vars.insert(var.to_string(), default.to_string());
            }
        }
    }

    for (k, v) in &instance.props {
        if k.to_ascii_lowercase().starts_with("replace")
            && let Some((var, value)) = v.split_once(char::is_whitespace)
        {
            vars.insert(var.to_string(), value.trim_start().to_string());
        }
    }

    let mut out: Vec<(String, String)> = vars.into_iter().collect();
    out.sort_by_key(|(k, _)| std::cmp::Reverse(k.len()));
    out
}

/// Applies `$var` replacements to entity values, and `#material` ones to face materials.
fn apply_replacements(block: &mut Block, vars: &[(String, String)]) {
    if block.name.eq_ignore_ascii_case("side") {
        for (k, v) in &mut block.props {
            if k.eq_ignore_ascii_case("material")
                && let Some((_, to)) = vars.iter().find(|(var, _)| var.strip_prefix('#').is_some_and(|m| m.eq_ignore_ascii_case(v)))
            {
                *v = to.clone();
            }
        }
    } else if block.name.eq_ignore_ascii_case("entity") || block.name.eq_ignore_ascii_case("connections") {
        for (_, v) in &mut block.props {
            for (var, to) in vars.iter().filter(|(var, _)| var.starts_with('$')) {
                if v.contains(var.as_str()) {
                    *v = v.replace(var.as_str(), to);
                }
            }
        }
    }

    for c in &mut block.children {
        apply_replacements(c, vars);
    }
}

struct Importer<'a> {
    options: &'a ImportOptions,
    report: ImportReport,
    /// Files being inlined, to stop an instance that includes itself.
    stack: Vec<PathBuf>,
    auto_names: usize,
}

/// Imports a `.vmf`. Units are kept as they are (1 Hammer unit = 1 map unit). Instances are only resolved by
/// [`import_file`], which knows where the map lives.
pub fn import(src: &str) -> Result<Map, VmfError> {
    import_with(src, None, &ImportOptions::default()).map(|(m, _)| m)
}

/// Imports a `.vmf` file, inlining its `func_instance` maps.
pub fn import_file(path: &Path, options: &ImportOptions) -> Result<(Map, ImportReport), VmfError> {
    import_with(&read_text(path)?, Some(path), options)
}

pub fn import_with(src: &str, path: Option<&Path>, options: &ImportOptions) -> Result<(Map, ImportReport), VmfError> {
    let blocks = parse_keyvalues(src)?;
    let mut importer = Importer { options, report: ImportReport::default(), stack: path.map(|p| vec![p.to_path_buf()]).unwrap_or_default(), auto_names: 0 };
    let map = importer.import_blocks(&blocks, path.and_then(Path::parent))?;
    Ok((map, importer.report))
}

impl Importer<'_> {
    fn import_blocks(&mut self, blocks: &[Block], dir: Option<&Path>) -> Result<Map, VmfError> {
        let mut map = Map::new();
        let default_layer = map.default_layer();
        let visgroups = visgroup_names(blocks);
        let mut layers: HashMap<String, NodeId> = HashMap::new();
        let mut layer_of = |map: &mut Map, b: &Block| -> NodeId {
            let Some(name) = b.child("editor").and_then(|e| e.get("visgroupid")).and_then(|id| visgroups.get(id)) else { return default_layer };
            *layers.entry(name.clone()).or_insert_with(|| map.add_layer(name))
        };

        let mut entity_blocks: Vec<(&Block, bool)> = Vec::new();
        for b in blocks {
            match b.name.to_ascii_lowercase().as_str() {
                "world" => {
                    for (k, v) in &b.props {
                        if !matches!(k.as_str(), "id" | "mapversion" | "detailmaterial" | "detailvbsp" | "maxpropscreenwidth") {
                            map.properties.insert(k.clone(), v.clone());
                        }
                    }

                    for (s, hidden) in solids(b) {
                        let layer = layer_of(&mut map, s);
                        for node in parse_solid(s, self.options).unwrap_or_default() {
                            let id = map.insert(layer, node);
                            set_hidden(&mut map, id, hidden);
                        }
                    }
                }
                "entity" => entity_blocks.push((b, hidden_by_visgroup(b))),
                "hidden" => entity_blocks.extend(b.children_named("entity").map(|e| (e, true))),
                _ => {}
            }
        }

        map.properties.insert("classname".into(), "worldspawn".into());
        let mut instances: HashMap<String, Fixup> = HashMap::new();
        for (b, hidden) in entity_blocks {
            let layer = layer_of(&mut map, b);
            if b.get("classname").is_some_and(|c| c.eq_ignore_ascii_case("func_instance"))
                && let Some((group, fixup)) = self.inline_instance(&mut map, layer, b, dir)?
            {
                set_hidden(&mut map, group, hidden);
                if let Some(name) = b.get("targetname").filter(|n| !n.is_empty()) {
                    instances.insert(name.to_string(), fixup);
                }

                continue;
            }

            let classname = b.get("classname").unwrap_or("info_null").to_string();
            let mut e = Entity::new(classname);
            let brush_solids = solids(b);
            for (k, v) in &b.props {
                match k.as_str() {
                    "classname" | "id" => {}
                    "origin" if brush_solids.is_empty() => e.origin = vec3(v).map(from_id).unwrap_or_default(),
                    "angles" if brush_solids.is_empty() => e.angles = vec3(v).map(angles_from_quake).unwrap_or_default(),
                    _ => {
                        e.properties.insert(k.clone(), v.clone());
                    }
                }
            }

            e.outputs = parse_connections(b.child("connections"));
            let id = map.insert(layer, NodeKind::Entity(e));
            set_hidden(&mut map, id, hidden);
            for (s, solid_hidden) in brush_solids {
                for node in parse_solid(s, self.options).unwrap_or_default() {
                    let brush_id = map.insert(id, node);
                    set_hidden(&mut map, brush_id, solid_hidden && !hidden);
                }
            }
        }

        // `instance:relay;Trigger` sends Trigger to the entity relay inside the instance the output targets.
        if !instances.is_empty() {
            let ids: Vec<NodeId> = map.entities().map(|(id, _)| id).collect();
            for id in ids {
                let Some(e) = map.entity_mut(id) else { continue };
                for o in &mut e.outputs {
                    if let (Some(fixup), Some((inner, input))) =
                        (instances.get(&o.target), o.input.strip_prefix("instance:").and_then(|rest| rest.split_once(';')))
                    {
                        o.target = fixup.apply(inner);
                        o.input = input.to_string();
                    }
                }
            }
        }

        Ok(map)
    }

    /// Inlines the map of a `func_instance` as a group under `parent`. None when the file cannot be found or read,
    /// the entity is then imported as it is.
    fn inline_instance(&mut self, map: &mut Map, parent: NodeId, instance: &Block, dir: Option<&Path>) -> Result<Option<(NodeId, Fixup)>, VmfError> {
        let Some(file) = instance.get("file").filter(|f| !f.is_empty()) else { return Ok(None) };
        let path = resolve_instance(file, dir).filter(|p| !self.stack.contains(p) && self.stack.len() < 16);
        let blocks = path.as_deref().and_then(|p| read_text(p).ok()).and_then(|text| parse_keyvalues(&text).ok());
        let (Some(path), Some(mut blocks)) = (path, blocks) else {
            self.report.missing_instances.push(file.to_string());
            return Ok(None);
        };

        let vars = instance_replacements(instance, &blocks);
        for b in &mut blocks {
            apply_replacements(b, &vars);
        }

        self.stack.push(path.clone());
        let inner = self.import_blocks(&blocks, path.parent());
        self.stack.pop();
        let mut inner = inner?;
        self.report.instances += 1;
        let parms: Vec<NodeId> = inner.entities().filter(|(_, e)| e.classname.eq_ignore_ascii_case("func_instance_parms")).map(|(id, _)| id).collect();
        for id in parms {
            inner.remove(id);
        }

        let name = match instance.get("targetname").filter(|n| !n.is_empty()) {
            Some(n) => n.to_string(),
            None => {
                self.auto_names += 1;
                format!("AutoInstance{}", self.auto_names)
            }
        };
        let fixup = Fixup { name: name.clone(), style: instance.get("fixup_style").and_then(|s| s.parse().ok()).unwrap_or(0) };
        let ids: Vec<NodeId> = inner.entities().map(|(id, _)| id).collect();
        for id in ids {
            let Some(e) = inner.entity_mut(id) else { continue };
            for (k, v) in e.properties.iter_mut() {
                if FIXUP_KEYS.contains(&k.to_ascii_lowercase().as_str()) {
                    *v = fixup.apply(v);
                }
            }

            for o in &mut e.outputs {
                o.target = fixup.apply(&o.target);
            }
        }

        let m = instance_transform(instance.get("origin").and_then(vec3).unwrap_or_default(), instance.get("angles").and_then(vec3).unwrap_or_default());
        let label = Path::new(&file.replace('\\', "/")).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let group = map.insert(parent, NodeKind::Group(Group::new(format!("{name} ({label})"))));
        let mut copied = Vec::new();
        let roots: Vec<NodeId> = inner.layers.iter().filter_map(|l| inner.get(*l)).flat_map(|l| l.children.clone()).collect();
        for root in roots {
            graft(map, group, &inner, root, &mut copied);
        }

        gt_doc::ops::transform_nodes(map, &copied, &m, gt_doc::ops::EditOptions { uv_lock: true, grid: 0.0 });
        for id in &copied {
            let is_brush_entity = map.get(*id).is_some_and(|n| !n.children.is_empty());
            if let Some(e) = map.entity_mut(*id).filter(|_| is_brush_entity)
                && let Some(origin) = e.properties.get("origin").and_then(|v| vec3(v))
            {
                let moved = to_id(m.transform_point3(from_id(origin)));
                e.properties.insert("origin".into(), format!("{} {} {}", short(moved.x), short(moved.y), short(moved.z)));
            }
        }

        // Outputs on the func_instance itself, `instance:relay;OnTrigger`, belong to that entity inside.
        for o in parse_connections(instance.child("connections")) {
            let Some((inner_name, output)) = o.output.strip_prefix("instance:").and_then(|rest| rest.split_once(';')) else { continue };
            let target = fixup.apply(inner_name);
            let found = copied.iter().copied().find(|id| map.entity(*id).is_some_and(|e| e.targetname() == Some(target.as_str())));
            if let Some(e) = found.and_then(|id| map.entity_mut(id)) {
                e.outputs.push(IoConnection { output: output.to_string(), ..o });
            }
        }

        Ok(Some((group, fixup)))
    }
}

fn short(v: f64) -> String {
    let r = (v * 1e4).round() / 1e4;
    if r == 0.0 { "0".into() } else { format!("{r}") }
}

fn set_hidden(map: &mut Map, id: NodeId, hidden: bool) {
    if hidden && let Some(n) = map.get_mut(id) {
        n.hidden = true;
    }
}

/// Copies `id` and its subtree from `src` under `parent`, appending every new id to `out`.
fn graft(map: &mut Map, parent: NodeId, src: &Map, id: NodeId, out: &mut Vec<NodeId>) {
    let Some(node) = src.get(id) else { return };
    let new = map.insert(parent, node.kind.clone());
    set_hidden(map, new, node.hidden);
    out.push(new);
    for child in &node.children {
        graft(map, new, src, *child, out);
    }
}

/// `visgroupid` to visgroup name, nested visgroups included.
fn visgroup_names(blocks: &[Block]) -> HashMap<String, String> {
    fn walk(b: &Block, out: &mut HashMap<String, String>) {
        for g in b.children_named("visgroup") {
            if let (Some(id), Some(name)) = (g.get("visgroupid"), g.get("name")) {
                out.insert(id.to_string(), name.to_string());
            }

            walk(g, out);
        }
    }

    let mut out = HashMap::new();
    for b in blocks.iter().filter(|b| b.name.eq_ignore_ascii_case("visgroups")) {
        walk(b, &mut out);
    }

    out
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

    /// A box solid in Hammer's plane point order, `top` goes into the +Z side (for a dispinfo).
    fn solid(min: [f64; 3], max: [f64; 3], material: &str, top: &str) -> String {
        let ([x0, y0, z0], [x1, y1, z1]) = (min, max);
        let side = |p: [[f64; 3]; 3], extra: &str| {
            let pts: Vec<String> = p.iter().map(|v| format!("({} {} {})", v[0], v[1], v[2])).collect();
            format!(
                "side {{ \"plane\" \"{}\" \"material\" \"{material}\" \"uaxis\" \"[1 0 0 0] 0.25\" \"vaxis\" \"[0 -1 0 0] 0.25\" {extra} }}\n",
                pts.join(" ")
            )
        };
        let mut s = String::from("solid {\n");
        s += &side([[x0, y1, z1], [x1, y1, z1], [x1, y0, z1]], top);
        s += &side([[x0, y0, z0], [x1, y0, z0], [x1, y1, z0]], "");
        s += &side([[x0, y1, z1], [x0, y0, z1], [x0, y0, z0]], "");
        s += &side([[x1, y1, z0], [x1, y0, z0], [x1, y0, z1]], "");
        s += &side([[x1, y1, z1], [x0, y1, z1], [x0, y1, z0]], "");
        s += &side([[x1, y0, z0], [x0, y0, z0], [x0, y0, z1]], "");
        s + "}\n"
    }

    fn dispinfo(start: [f64; 3], distances: impl Fn(usize, usize) -> f64) -> String {
        let row = |f: &dyn Fn(usize) -> String| (0..5).map(|r| format!("\"row{r}\" \"{}\"", f(r))).collect::<Vec<_>>().join(" ");
        let normals = row(&|_| "0 0 1 ".repeat(5).trim().to_string());
        let dists = row(&|r| (0..5).map(|c| distances(r, c).to_string()).collect::<Vec<_>>().join(" "));
        format!(
            "dispinfo {{ \"power\" \"2\" \"startposition\" \"[{} {} {}]\" \"elevation\" \"0\" normals {{ {normals} }} distances {{ {dists} }} }}",
            start[0], start[1], start[2]
        )
    }

    fn top_grid(map: &Map, brush: usize) -> gt_geom::displacement::DisplacementGrid {
        let (id, b) = map.brushes().nth(brush).unwrap();
        let fi = b.faces.iter().position(|f| f.data.disp.is_some()).expect("displacement face");
        gt_geom::displacement::grid(map.brush(id).unwrap(), fi).unwrap()
    }

    #[test]
    fn displacement_rows_follow_hammer_and_neighbours_sew() {
        // Hammer walks a row from the start corner towards the next corner of Source's clockwise winding, +Y for a top
        // face starting at its minimum. Both neighbours raise their shared edge x = 128 by 8 a row, so a transposed
        // reader both misplaces the ramp and tears the seam, which real maps showed as unsewn terrain.
        let a = solid([0.0, 0.0, 0.0], [128.0, 128.0, 64.0], "nature/dirt", &dispinfo([0.0, 0.0, 64.0], |r, c| if c == 4 { 8.0 * r as f64 } else { 0.0 }));
        let b = solid([128.0, 0.0, 0.0], [256.0, 128.0, 64.0], "nature/dirt", &dispinfo([128.0, 0.0, 64.0], |r, c| if c == 0 { 8.0 * r as f64 } else { 0.0 }));
        let map = import(&format!("world {{ {a} {b} }}")).unwrap();
        let (ga, gb) = (top_grid(&map, 0), top_grid(&map, 1));
        let mut seam = 0;
        for (base, pos) in ga.base.iter().zip(&ga.positions) {
            // Godot space: id x is z, id y is x, id z is y.
            if (base.z - 128.0).abs() > 1e-6 {
                assert!((pos.y - 64.0).abs() < 1e-6, "only the seam is raised, {pos} is not flat");
                continue;
            }

            seam += 1;
            assert!((pos.y - 64.0 - base.x / 4.0).abs() < 1e-6, "rows run along +Y: {base} rose to {pos}");
            assert!(gb.positions.iter().any(|p| (*p - *pos).length() < 1e-6), "{pos} has no twin on the neighbour");
        }

        assert_eq!(seam, 5);
    }

    #[test]
    fn sideways_displacements_become_exact_meshes() {
        // One vertex pushed 20 units along +X on a +Z face, which a height grid cannot hold.
        let info = dispinfo([0.0, 0.0, 64.0], |r, c| if (r, c) == (2, 2) { 20.0 } else { 0.0 }).replacen(
            "\"row2\" \"0 0 1 0 0 1 0 0 1",
            "\"row2\" \"0 0 1 0 0 1 1 0 0",
            1,
        );
        let src =
            format!("world {{ {} {} }}", solid([0.0; 3], [128.0, 128.0, 64.0], "nature/cliff", &info), solid([256.0, 0.0, 0.0], [320.0, 64.0, 64.0], "a", ""));
        let map = import(&src).unwrap();
        assert_eq!(map.brush_count(), 1, "the displacement brush is replaced, the plain one stays");
        let (_, mesh) = map.meshes().next().expect("displacement mesh");
        assert_eq!((mesh.vertices.len(), mesh.faces.len()), (25, 32));
        assert!(mesh.faces.iter().all(|f| f.data.material == "nature/cliff" && f.data.disp.is_none()));
        // The middle vertex, id (64, 64, 64) moved to id x 84: Godot (64, 64, 84).
        assert!(mesh.vertices.iter().any(|v| (*v - DVec3::new(64.0, 64.0, 84.0)).length() < 1e-6), "{:?}", mesh.vertices[12]);
    }

    #[test]
    fn tool_materials_become_project_tool_textures() {
        let map = import(SAMPLE).unwrap();
        let (_, b) = map.brushes().next().unwrap();
        assert_eq!(b.faces.iter().filter(|f| f.data.material == "special/skip").count(), 4, "nodraw is skipped");
        let tools = ImportOptions::default();
        assert_eq!(tools.material("tools/toolsplayerclip"), "special/clip");
        assert_eq!(tools.material("tools/toolstrigger"), "special/trigger");
        assert_eq!(tools.material("tools/toolsorigin"), "special/origin");
        assert_eq!(tools.material("tools/toolsskybox"), "special/skip");
        assert_eq!(tools.material("brick/brickwall001a"), "brick/brickwall001a");
    }

    #[test]
    fn visgroups_become_layers_and_hidden_objects_stay_hidden() {
        let src = format!(
            "visgroups {{ visgroup {{ \"name\" \"Detail\" \"visgroupid\" \"7\" visgroup {{ \"name\" \"Lamps\" \"visgroupid\" \"9\" }} }} }}
world {{ {} hidden {{ {} }} }}
entity {{ \"classname\" \"light\" \"origin\" \"0 0 0\" editor {{ \"visgroupid\" \"9\" \"visgroupshown\" \"0\" }} }}",
            solid([0.0; 3], [64.0; 3], "a", "").replace("solid {", "solid { editor { \"visgroupid\" \"7\" }"),
            solid([128.0, 0.0, 0.0], [192.0, 64.0, 64.0], "a", "")
        );
        let map = import(&src).unwrap();
        let layer_names: Vec<String> = map.layers.iter().filter_map(|l| map.get(*l)).map(|n| n.name()).collect();
        assert!(layer_names.iter().any(|n| n.contains("Detail")) && layer_names.iter().any(|n| n.contains("Lamps")), "{layer_names:?}");
        let (detail, _) = map.brushes().find(|(id, _)| map.get(map.layer_of(*id)).is_some_and(|l| l.name().contains("Detail"))).expect("brush in Detail");
        assert!(!map.is_hidden(detail));
        assert_eq!(map.brushes().filter(|(id, _)| map.is_hidden(*id)).count(), 1, "the brush in the hidden block");
        let (lamp, _) = map.entities().next().unwrap();
        assert!(map.is_hidden(lamp), "visgroupshown 0 hides the entity");
    }

    #[test]
    fn reads_latin1_files() {
        let dir = std::env::temp_dir().join(format!("gt_vmf_latin1_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("old.vmf");
        let mut bytes = b"entity { \"classname\" \"info_target\" \"targetname\" \"caf".to_vec();
        bytes.extend([0xE9, b'"', b' ', b'}']);
        std::fs::write(&path, bytes).unwrap();
        let (map, _) = import_file(&path, &ImportOptions::default()).unwrap();
        assert_eq!(map.entities().next().unwrap().1.targetname(), Some("caf\u{e9}"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
