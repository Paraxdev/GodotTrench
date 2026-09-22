//! Blockbench `.bbmodel` models: cubes and free meshes with embedded textures.
//! Blockbench is Y-up like GodotTrench, 16 units make a block. North is -Z.

use std::collections::BTreeMap;

use base64::Engine;
use gt_core::{Aabb, DMat4, DVec2, DVec3, Plane};
use gt_geom::{Brush, FaceData, FaceUv, Mesh, MeshFace};
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum BbError {
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not a Blockbench model")]
    NotBbModel,
}

#[derive(Clone, Debug)]
pub struct BbTexture {
    pub name: String,
    /// PNG bytes, empty when the texture only references an external file.
    pub png: Vec<u8>,
    /// External file the texture was loaded from in Blockbench, if any.
    pub path: String,
    /// Size of the UV space the face coordinates are expressed in.
    pub uv_size: DVec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CubeFace {
    North,
    South,
    East,
    West,
    Up,
    Down,
}

impl CubeFace {
    pub const ALL: [CubeFace; 6] = [CubeFace::North, CubeFace::South, CubeFace::East, CubeFace::West, CubeFace::Up, CubeFace::Down];

    pub fn key(&self) -> &'static str {
        match self {
            CubeFace::North => "north",
            CubeFace::South => "south",
            CubeFace::East => "east",
            CubeFace::West => "west",
            CubeFace::Up => "up",
            CubeFace::Down => "down",
        }
    }

    /// (outward normal, right, up) as seen from outside the face.
    pub fn basis(&self) -> (DVec3, DVec3, DVec3) {
        match self {
            CubeFace::North => (DVec3::NEG_Z, DVec3::NEG_X, DVec3::Y),
            CubeFace::South => (DVec3::Z, DVec3::X, DVec3::Y),
            CubeFace::East => (DVec3::X, DVec3::NEG_Z, DVec3::Y),
            CubeFace::West => (DVec3::NEG_X, DVec3::Z, DVec3::Y),
            CubeFace::Up => (DVec3::Y, DVec3::X, DVec3::NEG_Z),
            CubeFace::Down => (DVec3::NEG_Y, DVec3::X, DVec3::Z),
        }
    }
}

/// One textured polygon in model space (Blockbench units), counter-clockwise seen from the front.
#[derive(Clone, Debug)]
pub struct BbPolygon {
    pub positions: Vec<DVec3>,
    /// Normalized texture coordinates, v pointing down.
    pub uvs: Vec<DVec2>,
    pub texture: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct BbCube {
    pub name: String,
    pub from: DVec3,
    pub to: DVec3,
    /// Model space placement of the unrotated box.
    pub transform: DMat4,
    pub faces: Vec<(CubeFace, BbPolygon)>,
}

#[derive(Clone, Debug, Default)]
pub struct BbModel {
    pub name: String,
    pub textures: Vec<BbTexture>,
    pub cubes: Vec<BbCube>,
    /// Faces of free form mesh elements.
    pub mesh_polygons: Vec<BbPolygon>,
}

fn vec3(v: Option<&Value>) -> DVec3 {
    match v.and_then(|v| v.as_array()) {
        Some(a) if a.len() >= 3 => DVec3::new(a[0].as_f64().unwrap_or(0.0), a[1].as_f64().unwrap_or(0.0), a[2].as_f64().unwrap_or(0.0)),
        _ => DVec3::ZERO,
    }
}

/// Blockbench rotates elements about their origin with three.js "ZYX" Euler order.
fn pivot_rotation(origin: DVec3, rotation_deg: DVec3) -> DMat4 {
    let r = DMat4::from_rotation_z(rotation_deg.z.to_radians())
        * DMat4::from_rotation_y(rotation_deg.y.to_radians())
        * DMat4::from_rotation_x(rotation_deg.x.to_radians());
    DMat4::from_translation(origin) * r * DMat4::from_translation(-origin)
}

fn box_uv_rect(face: CubeFace, offset: DVec2, size: DVec3) -> [f64; 4] {
    let (w, h, d) = (size.x, size.y, size.z);
    let (u, v) = (offset.x, offset.y);
    match face {
        CubeFace::East => [u, v + d, u + d, v + d + h],
        CubeFace::North => [u + d, v + d, u + d + w, v + d + h],
        CubeFace::West => [u + d + w, v + d, u + d * 2.0 + w, v + d + h],
        CubeFace::South => [u + d * 2.0 + w, v + d, u + d * 2.0 + w * 2.0, v + d + h],
        CubeFace::Up => [u + d + w, v + d, u + d, v],
        CubeFace::Down => [u + d + w * 2.0, v, u + d + w, v + d],
    }
}

/// Parses a `.bbmodel` document.
pub fn parse(text: &str) -> Result<BbModel, BbError> {
    let root: Value = serde_json::from_str(text)?;
    let Some(elements) = root.get("elements").and_then(|e| e.as_array()) else { return Err(BbError::NotBbModel) };
    if root.get("meta").is_none() {
        return Err(BbError::NotBbModel);
    }

    let resolution = DVec2::new(
        root.pointer("/resolution/width").and_then(|v| v.as_f64()).unwrap_or(16.0),
        root.pointer("/resolution/height").and_then(|v| v.as_f64()).unwrap_or(16.0),
    );
    let global_box_uv = root.pointer("/meta/box_uv").and_then(|v| v.as_bool()).unwrap_or(false);

    let mut model = BbModel { name: root.get("name").and_then(|v| v.as_str()).unwrap_or("model").to_string(), ..Default::default() };
    let mut texture_index: BTreeMap<String, usize> = BTreeMap::new();
    for (i, t) in root.get("textures").and_then(|t| t.as_array()).into_iter().flatten().enumerate() {
        let source = t.get("source").and_then(|s| s.as_str()).unwrap_or("");
        let png = source.split_once("base64,").and_then(|(_, data)| base64::engine::general_purpose::STANDARD.decode(data.trim()).ok()).unwrap_or_default();
        let uv_size =
            DVec2::new(t.get("uv_width").and_then(|v| v.as_f64()).unwrap_or(resolution.x), t.get("uv_height").and_then(|v| v.as_f64()).unwrap_or(resolution.y));
        let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("texture").trim_end_matches(".png").to_string();
        if let Some(id) = t.get("id").and_then(|v| v.as_str()) {
            texture_index.insert(id.to_string(), i);
        }

        if let Some(uuid) = t.get("uuid").and_then(|v| v.as_str()) {
            texture_index.insert(uuid.to_string(), i);
        }

        model.textures.push(BbTexture { name, png, path: t.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string(), uv_size });
    }

    let uv_size = |tex: Option<usize>| tex.and_then(|t| model.textures.get(t)).map(|t| t.uv_size).unwrap_or(resolution);
    let texture_ref = |v: Option<&Value>| -> Option<usize> {
        match v? {
            Value::Number(n) => n.as_u64().map(|n| n as usize),
            Value::String(s) => texture_index.get(s).copied().or_else(|| s.parse().ok()),
            _ => None,
        }
    };

    // Group transforms from the outliner, applied to every element below them. Blockbench 5 keeps group data in a
    // separate "groups" list and the outliner only names the group's uuid.
    let groups: BTreeMap<&str, &Value> =
        root.get("groups").and_then(|g| g.as_array()).into_iter().flatten().filter_map(|g| Some((g.get("uuid")?.as_str()?, g))).collect();
    let mut element_xform: BTreeMap<String, DMat4> = BTreeMap::new();
    fn walk(node: &Value, parent: DMat4, groups: &BTreeMap<&str, &Value>, out: &mut BTreeMap<String, DMat4>) {
        match node {
            Value::String(uuid) => {
                out.insert(uuid.clone(), parent);
            }
            Value::Object(group) => {
                let data = group.get("uuid").and_then(|u| u.as_str()).and_then(|u| groups.get(u)).copied();
                let field = |key: &str| group.get(key).or_else(|| data.and_then(|d| d.get(key)));
                let m = parent * pivot_rotation(vec3(field("origin")), vec3(field("rotation")));
                for c in group.get("children").and_then(|c| c.as_array()).into_iter().flatten() {
                    walk(c, m, groups, out);
                }
            }
            _ => {}
        }
    }

    for node in root.get("outliner").and_then(|o| o.as_array()).into_iter().flatten() {
        walk(node, DMat4::IDENTITY, &groups, &mut element_xform);
    }

    for el in elements {
        if el.get("visibility").and_then(|v| v.as_bool()) == Some(false) {
            continue;
        }

        let uuid = el.get("uuid").and_then(|v| v.as_str()).unwrap_or("");
        let group = element_xform.get(uuid).copied().unwrap_or(DMat4::IDENTITY);
        let origin = vec3(el.get("origin"));
        let rotation = vec3(el.get("rotation"));
        match el.get("type").and_then(|v| v.as_str()).unwrap_or("cube") {
            "mesh" => {
                let xform = group * DMat4::from_translation(origin) * pivot_rotation(DVec3::ZERO, rotation);
                let verts: BTreeMap<String, DVec3> = el
                    .get("vertices")
                    .and_then(|v| v.as_object())
                    .into_iter()
                    .flatten()
                    .map(|(k, v)| (k.clone(), xform.transform_point3(vec3(Some(v)))))
                    .collect();
                for face in el.get("faces").and_then(|f| f.as_object()).into_iter().flat_map(|f| f.values()) {
                    let keys: Vec<String> =
                        face.get("vertices").and_then(|v| v.as_array()).into_iter().flatten().filter_map(|k| k.as_str().map(str::to_string)).collect();
                    if keys.len() < 3 || keys.iter().any(|k| !verts.contains_key(k)) {
                        continue;
                    }

                    let texture = texture_ref(face.get("texture"));
                    let size = uv_size(texture);
                    let order = sorted_face_order(&keys.iter().map(|k| verts[k]).collect::<Vec<_>>());
                    let positions: Vec<DVec3> = order.iter().map(|i| verts[&keys[*i]]).collect();
                    let uvs: Vec<DVec2> = order
                        .iter()
                        .map(|i| {
                            let uv = face.get("uv").and_then(|u| u.get(&keys[*i]));
                            let a = uv.and_then(|u| u.as_array());
                            let (u, v) = a
                                .map(|a| (a.first().and_then(|x| x.as_f64()).unwrap_or(0.0), a.get(1).and_then(|x| x.as_f64()).unwrap_or(0.0)))
                                .unwrap_or((0.0, 0.0));
                            DVec2::new(u / size.x, v / size.y)
                        })
                        .collect();
                    model.mesh_polygons.push(BbPolygon { positions, uvs, texture });
                }
            }
            "cube" => {
                let inflate = el.get("inflate").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let from = vec3(el.get("from")) - DVec3::splat(inflate);
                let to = vec3(el.get("to")) + DVec3::splat(inflate);
                let transform = group * pivot_rotation(origin, rotation);
                let box_uv = el.get("box_uv").and_then(|v| v.as_bool()).unwrap_or(global_box_uv);
                let uv_offset = {
                    let a = el.get("uv_offset").and_then(|v| v.as_array());
                    DVec2::new(
                        a.and_then(|a| a.first()).and_then(|v| v.as_f64()).unwrap_or(0.0),
                        a.and_then(|a| a.get(1)).and_then(|v| v.as_f64()).unwrap_or(0.0),
                    )
                };
                let size = (vec3(el.get("to")) - vec3(el.get("from"))).abs();
                let mut faces = Vec::new();
                for dir in CubeFace::ALL {
                    let Some(face) = el.get("faces").and_then(|f| f.get(dir.key())) else { continue };
                    if face.get("texture").is_some_and(|t| t.is_null()) {
                        continue;
                    }

                    let texture = texture_ref(face.get("texture"));
                    let tex_size = uv_size(texture);
                    let rect = if box_uv {
                        box_uv_rect(dir, uv_offset, size.floor())
                    } else {
                        let a = face.get("uv").and_then(|u| u.as_array());
                        let g = |i: usize| a.and_then(|a| a.get(i)).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        [g(0), g(1), g(2), g(3)]
                    };
                    let (normal, right, up) = dir.basis();
                    let corner = |sr: f64, su: f64| {
                        let mut p = DVec3::ZERO;
                        for a in 0..3 {
                            let pick = if normal[a] != 0.0 {
                                normal[a] > 0.0
                            } else if right[a] != 0.0 {
                                sr * right[a] > 0.0
                            } else {
                                su * up[a] > 0.0
                            };
                            p[a] = if pick { to[a] } else { from[a] };
                        }

                        transform.transform_point3(p)
                    };
                    let (tl, tr, bl, br) = (corner(-1.0, 1.0), corner(1.0, 1.0), corner(-1.0, -1.0), corner(1.0, -1.0));
                    let mut arr = [[rect[0], rect[1]], [rect[2], rect[1]], [rect[0], rect[3]], [rect[2], rect[3]]];
                    let mut rot = face.get("rotation").and_then(|v| v.as_f64()).unwrap_or(0.0) as i64;
                    while rot > 0 {
                        let a = arr[0];
                        arr[0] = arr[2];
                        arr[2] = arr[3];
                        arr[3] = arr[1];
                        arr[1] = a;
                        rot -= 90;
                    }

                    let n = |c: [f64; 2]| DVec2::new(c[0] / tex_size.x, c[1] / tex_size.y);
                    let poly = BbPolygon { positions: vec![tl, bl, br, tr], uvs: vec![n(arr[0]), n(arr[2]), n(arr[3]), n(arr[1])], texture };
                    faces.push((dir, poly));
                }

                model.cubes.push(BbCube { name: el.get("name").and_then(|v| v.as_str()).unwrap_or("cube").to_string(), from, to, transform, faces });
            }
            _ => {}
        }
    }

    Ok(model)
}

/// Blockbench stores quad corners in creation order, which is not always a loop.
fn sorted_face_order(pts: &[DVec3]) -> Vec<usize> {
    if pts.len() != 4 {
        return (0..pts.len()).collect();
    }

    let candidates = [[0usize, 1, 2, 3], [0, 1, 3, 2], [0, 2, 1, 3]];
    let area = |o: &[usize; 4]| gt_geom::polygon::newell(&o.iter().map(|i| pts[*i]).collect::<Vec<_>>()).length();
    candidates.into_iter().max_by(|a, b| area(a).total_cmp(&area(b))).map(|o| o.to_vec()).unwrap()
}

impl BbModel {
    /// Every textured polygon in model space.
    pub fn polygons(&self) -> impl Iterator<Item = &BbPolygon> {
        self.cubes.iter().flat_map(|c| c.faces.iter().map(|(_, p)| p)).chain(self.mesh_polygons.iter())
    }

    pub fn bounds(&self) -> Aabb {
        Aabb::from_points(self.polygons().flat_map(|p| p.positions.iter().copied()))
    }

    /// One mesh with explicit UVs. `scale` converts Blockbench units to map units, `offset` is added afterwards.
    pub fn to_mesh(&self, scale: f64, offset: DVec3, material: impl Fn(Option<usize>) -> String) -> Mesh {
        let polys = self.polygons().map(|p| (p.positions.iter().map(|v| *v * scale + offset).collect::<Vec<_>>(), p));
        let mut mesh = Mesh::default();
        let mut lookup: BTreeMap<(i64, i64, i64), u32> = BTreeMap::new();
        for (positions, poly) in polys {
            let mut indices = Vec::with_capacity(positions.len());
            for p in &positions {
                let key = ((p.x * 1e4).round() as i64, (p.y * 1e4).round() as i64, (p.z * 1e4).round() as i64);
                let idx = *lookup.entry(key).or_insert_with(|| {
                    mesh.vertices.push(*p);
                    (mesh.vertices.len() - 1) as u32
                });
                indices.push(idx);
            }

            let unique: std::collections::BTreeSet<u32> = indices.iter().copied().collect();
            if unique.len() < 3 || unique.len() != indices.len() {
                continue;
            }

            let normal = gt_geom::polygon::newell(&positions).normalize_or(DVec3::Y);
            let mut face = MeshFace::new(indices, FaceData::new(material(poly.texture), FaceUv::paraxial(normal, DVec2::ONE)));
            face.uvs = poly.uvs.iter().map(|uv| [uv.x as f32, uv.y as f32]).collect();
            mesh.faces.push(face);
        }

        mesh
    }

    /// Cubes as brushes with Valve UVs matching the Blockbench mapping. `tex_pixels` gives each texture's pixel size.
    pub fn to_brushes(&self, scale: f64, offset: DVec3, material: impl Fn(Option<usize>) -> String, tex_pixels: impl Fn(Option<usize>) -> DVec2) -> Vec<Brush> {
        let place = DMat4::from_translation(offset) * DMat4::from_scale(DVec3::splat(scale));
        let mut out = Vec::new();
        for cube in &self.cubes {
            if (cube.to - cube.from).abs().min_element() < 1e-6 {
                continue;
            }

            let m = place * cube.transform;
            let Ok(mut brush) = Brush::from_aabb(&Aabb::new(cube.from, cube.to), "").map(|b| b.transformed(&m, false)) else { continue };
            for face in &mut brush.faces {
                let Some((_, poly)) = cube.faces.iter().find(|(dir, _)| (m.transform_vector3(dir.basis().0).normalize() - face.plane.normal).length() < 1e-3)
                else {
                    face.data.material = "special/skip".into();
                    continue;
                };
                let px = tex_pixels(poly.texture);
                let world: Vec<DVec3> = poly.positions.iter().map(|p| place.transform_point3(*p)).collect();
                face.data.material = material(poly.texture);
                if let Some(uv) = uv_from_corners(&world, &poly.uvs, px) {
                    face.data.uv = uv;
                }
            }

            out.push(brush);
        }

        out
    }
}

/// Valve projection reproducing corner UVs (normalized) on a planar polygon for a texture of `px` pixels.
pub fn uv_from_corners(positions: &[DVec3], uvs: &[DVec2], px: DVec2) -> Option<FaceUv> {
    if positions.len() < 3 || uvs.len() < 3 {
        return None;
    }

    let normal = Plane::from_polygon(positions)?.normal;
    let (p0, p1, p2) = (positions[0], positions[1], positions[positions.len() - 1]);
    let (t0, t1, t2) = (uvs[0] * px, uvs[1] * px, uvs[uvs.len() - 1] * px);
    let (e1, e2) = (p1 - p0, p2 - p0);
    let (d1, d2) = (t1 - t0, t2 - t0);
    // Rows solve a.e1 = d1, a.e2 = d2, a.n = 0.
    let m = gt_core::DMat3::from_cols(e1, e2, normal).transpose();
    if m.determinant().abs() < 1e-12 {
        return None;
    }

    let inv = m.inverse();
    let a_u = inv * DVec3::new(d1.x, d2.x, 0.0);
    let a_v = inv * DVec3::new(d1.y, d2.y, 0.0);
    let (lu, lv) = (a_u.length(), a_v.length());
    if lu < 1e-12 || lv < 1e-12 {
        return None;
    }

    Some(FaceUv {
        u_axis: a_u / lu,
        v_axis: a_v / lv,
        offset: DVec2::new(t0.x - a_u.dot(p0), t0.y - a_v.dot(p0)),
        scale: DVec2::new(1.0 / lu, 1.0 / lv),
        rotation: 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "meta": {"format_version": "4.10", "model_format": "free", "box_uv": false},
        "name": "test",
        "resolution": {"width": 16, "height": 16},
        "elements": [
            {"name": "trunk", "type": "cube", "uuid": "a", "from": [6, 0, 6], "to": [10, 16, 10], "origin": [8, 0, 8],
             "faces": {"north": {"uv": [0, 0, 4, 16], "texture": 0}, "south": {"uv": [4, 0, 8, 16], "texture": 0},
                       "east": {"uv": [0, 0, 4, 16], "texture": 0}, "west": {"uv": [0, 0, 4, 16], "texture": 0},
                       "up": {"uv": [8, 0, 12, 4], "texture": 0}, "down": {"uv": [8, 4, 12, 8], "texture": 0}}},
            {"name": "tilted", "type": "cube", "uuid": "b", "from": [0, 16, 0], "to": [16, 20, 16], "origin": [8, 18, 8], "rotation": [0, 45, 0],
             "faces": {"up": {"uv": [0, 0, 16, 16], "texture": 0, "rotation": 90}}},
            {"name": "tri", "type": "mesh", "uuid": "c", "origin": [0, 24, 0], "rotation": [0, 0, 0],
             "vertices": {"v1": [0, 0, 0], "v2": [8, 0, 0], "v3": [0, 0, -8]},
             "faces": {"f": {"uv": {"v1": [0, 0], "v2": [16, 0], "v3": [0, 16]}, "vertices": ["v1", "v2", "v3"], "texture": 0}}}
        ],
        "outliner": [{"name": "root", "origin": [0, 0, 0], "rotation": [0, 0, 0], "children": ["a", "b", "c"]}],
        "textures": [{"name": "bark.png", "id": "0", "uv_width": 16, "uv_height": 16, "source": "data:image/png;base64,iVBORw0KGgo="}]
    }"#;

    #[test]
    fn parses_cubes_meshes_and_textures() {
        let m = parse(SAMPLE).unwrap();
        assert_eq!(m.cubes.len(), 2);
        assert_eq!(m.cubes[0].faces.len(), 6);
        assert_eq!(m.cubes[1].faces.len(), 1);
        assert_eq!(m.mesh_polygons.len(), 1);
        assert_eq!(m.textures[0].name, "bark");
        assert_eq!(&m.textures[0].png[..4], &[0x89, b'P', b'N', b'G']);
        let b = m.bounds();
        assert!((b.max.y - 24.0).abs() < 1e-9 && b.min.y.abs() < 1e-9);
    }

    #[test]
    fn blockbench5_group_transforms_come_from_the_groups_list() {
        let text = r#"{
            "meta": {"format_version": "5.0", "model_format": "free", "box_uv": false},
            "elements": [{"name": "c", "type": "cube", "uuid": "c", "from": [4, 0, -1], "to": [6, 2, 1], "origin": [5, 1, 0],
                          "faces": {"up": {"uv": [0, 0, 2, 2], "texture": 0}}}],
            "groups": [{"name": "g", "uuid": "g", "origin": [0, 0, 0], "rotation": [0, 90, 0]}],
            "outliner": [{"uuid": "g", "isOpen": false, "children": ["c"]}]
        }"#;
        let b = parse(text).unwrap().bounds();
        // Turning 90 degrees about Y carries +X to -Z.
        assert!((b.min.z + 6.0).abs() < 1e-9 && (b.max.z + 4.0).abs() < 1e-9 && b.max.x.abs() < 1.0 + 1e-9, "{b:?}");
    }

    #[test]
    fn cube_faces_wind_outwards_with_uv_corners() {
        let m = parse(SAMPLE).unwrap();
        let trunk = &m.cubes[0];
        let center = (trunk.from + trunk.to) * 0.5;
        for (dir, poly) in &trunk.faces {
            let n = gt_geom::polygon::newell(&poly.positions).normalize();
            assert!((n - dir.basis().0).length() < 1e-9, "{dir:?} normal {n}");
            assert!(n.dot(gt_geom::polygon::centroid(&poly.positions) - center) > 0.0);
        }

        let (_, south) = trunk.faces.iter().find(|(d, _)| *d == CubeFace::South).unwrap();
        // Top left corner of the south face is at -X, +Y seen from +Z.
        assert!((south.positions[0] - DVec3::new(6.0, 16.0, 10.0)).length() < 1e-9);
        assert!((south.uvs[0] - DVec2::new(0.25, 0.0)).length() < 1e-9);
    }

    #[test]
    fn mesh_and_brush_conversion_keep_uvs() {
        let m = parse(SAMPLE).unwrap();
        let mesh = m.to_mesh(2.0, DVec3::ZERO, |_| "bark".into());
        mesh.validate().unwrap();
        assert_eq!(mesh.faces.len(), 8);
        let brushes = m.to_brushes(2.0, DVec3::ZERO, |_| "bark".into(), |_| DVec2::splat(16.0));
        assert_eq!(brushes.len(), 2);
        let trunk = &brushes[0];
        trunk.validate().unwrap();
        let (_, south) = m.cubes[0].faces.iter().find(|(d, _)| *d == CubeFace::South).unwrap();
        let face = trunk.faces.iter().find(|f| f.plane.normal.z > 0.9).unwrap();
        for (p, uv) in south.positions.iter().zip(&south.uvs) {
            let got = face.data.uv.uv(*p * 2.0, DVec2::splat(16.0));
            assert!((got - *uv).length() < 1e-9, "uv {got} vs {uv}");
        }
    }
}
