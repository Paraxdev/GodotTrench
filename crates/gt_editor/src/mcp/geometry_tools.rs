//! MCP tools for meshes and terrains.

use gt_core::{Aabb, DMat4, DVec2, DVec3, NodeId, Plane};
use gt_doc::NodeKind;
use gt_geom::mesh_shapes;
use serde_json::{Value, json};

use super::ToolResult;
use crate::app::App;

fn vec3(v: &Value) -> Option<DVec3> {
    let a = v.as_array()?;
    Some(DVec3::new(a.first()?.as_f64()?, a.get(1)?.as_f64()?, a.get(2)?.as_f64()?))
}

fn vec2(v: &Value) -> Option<DVec2> {
    let a = v.as_array()?;
    Some(DVec2::new(a.first()?.as_f64()?, a.get(1)?.as_f64()?))
}

fn bounds_json(b: &Aabb) -> Value {
    if b.is_empty() { Value::Null } else { json!({ "min": [b.min.x, b.min.y, b.min.z], "max": [b.max.x, b.max.y, b.max.z] }) }
}

fn err(msg: impl Into<String>) -> Option<ToolResult> {
    Some(ToolResult::Error(msg.into()))
}

fn ok(v: Value) -> Option<ToolResult> {
    Some(ToolResult::Json(v))
}

impl App {
    pub(crate) fn tool_create_mesh(&mut self, args: &Value) -> Option<ToolResult> {
        let material = args["material"].as_str().map(str::to_string).unwrap_or_else(|| self.state.current_material.clone());
        let bounds = match (vec3(&args["min"]), vec3(&args["max"])) {
            (Some(a), Some(b)) => Aabb::new(a, b),
            _ => Aabb::new(DVec3::splat(-32.0), DVec3::splat(32.0)),
        };
        let sides = args["sides"].as_u64().unwrap_or(16) as usize;
        let size = bounds.size();
        let thickness = args["thickness"].as_f64().unwrap_or(8.0);
        let divisions = args["divisions"].as_u64().unwrap_or(4) as usize;
        let mut mesh = match args["shape"].as_str().unwrap_or("cuboid") {
            "cuboid" | "box" => mesh_shapes::cuboid(&bounds, &material),
            "cylinder" => mesh_shapes::cylinder(&bounds, sides, &material),
            "cone" => mesh_shapes::cone(&bounds, sides, &material),
            "sphere" => mesh_shapes::sphere(&bounds, sides, (sides / 2).max(3), &material),
            "torus" => mesh_shapes::torus(bounds.center(), (size.x * 0.5 - thickness).max(1.0), thickness, sides, (sides / 2).max(3), &material),
            "grid" => mesh_shapes::grid(&bounds, divisions, divisions, &material),
            "arch_wall" => mesh_shapes::arch_wall(
                &bounds,
                args["opening_width"].as_f64().unwrap_or(size.x * 0.5),
                args["opening_height"].as_f64().unwrap_or(size.y * 0.75),
                sides,
                &material,
            ),
            "gable_roof" => mesh_shapes::gable_roof(
                &bounds,
                args["ridge_z"].as_bool().unwrap_or(false),
                args["thickness"].as_f64().unwrap_or(4.0),
                args["overhang"].as_f64().unwrap_or(8.0),
                &material,
            ),
            "spire" => mesh_shapes::spire(&bounds, sides.max(3), &material),
            "lathe" => {
                let profile: Vec<DVec2> = args["profile"].as_array().into_iter().flatten().filter_map(vec2).collect();
                if profile.len() < 2 {
                    return err("lathe needs a profile of at least two [radius, height] points");
                }

                mesh_shapes::lathe(vec3(&args["center"]).unwrap_or_default(), &profile, sides, args["caps"].as_bool().unwrap_or(true), &material, 40.0)
            }
            "prism" => {
                let footprint: Vec<DVec2> = args["footprint"].as_array().into_iter().flatten().filter_map(vec2).collect();
                if footprint.len() < 3 {
                    return err("prism needs a footprint of at least three [x, z] points");
                }

                mesh_shapes::prism(&footprint, args["bottom"].as_f64().unwrap_or(bounds.min.y), args["top"].as_f64().unwrap_or(bounds.max.y), &material)
            }
            other => return err(format!("unknown mesh shape {other}")),
        };
        if let Some(angle) = args["smooth_angle"].as_f64() {
            mesh.smooth_angle = angle as f32;
        }

        if let Some(r) = args["roughen"].as_object() {
            let amount = r.get("amount").and_then(|v| v.as_f64()).unwrap_or(16.0);
            let seed = r.get("seed").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
            let center = mesh.bounds().center();
            for (i, p) in mesh.vertices.iter_mut().enumerate() {
                let n = gt_geom::heightfield::noise2(DVec2::new(p.x * 0.013 + i as f64 * 0.1, p.z * 0.013 + p.y * 0.011), seed);
                *p += (*p - center).normalize_or(DVec3::Y) * n * amount;
            }
        }

        if let Some(deg) = args["rotate_y"].as_f64() {
            let c = mesh.bounds().center();
            mesh = mesh.transformed(&(DMat4::from_translation(c) * DMat4::from_rotation_y(deg.to_radians()) * DMat4::from_translation(-c)), false);
        }

        if let Some(t) = vec3(&args["translate"]) {
            mesh = mesh.transformed(&DMat4::from_translation(t), false);
        }

        if let Some(scale) = args["uv_scale"].as_f64() {
            for fi in 0..mesh.faces.len() {
                let n = mesh.face_normal(fi);
                mesh.faces[fi].data.uv = gt_geom::FaceUv::face_aligned(n, DVec2::splat(scale));
                mesh.faces[fi].uvs.clear();
            }
        }

        let parent = args["parent"].as_u64().map(NodeId).filter(|p| self.state.doc.map.contains(*p)).unwrap_or_else(|| self.state.insert_parent());
        let faces = mesh.faces.len();
        let id = self.state.doc.edit("Create Mesh", |m, s| {
            let id = m.insert(parent, NodeKind::Mesh(mesh));
            s.clear();
            s.select_node(id);
            id
        });
        ok(json!({ "id": id.0, "faces": faces }))
    }

    pub(crate) fn tool_mesh_edit(&mut self, args: &Value) -> Option<ToolResult> {
        let id = NodeId(args["id"].as_u64()?);
        if self.state.doc.map.mesh(id).is_none() {
            return err(format!("{id} is not a mesh"));
        }

        let op = args["op"].as_str().unwrap_or_default().to_string();
        let faces: Vec<usize> = args["faces"].as_array().into_iter().flatten().filter_map(|v| v.as_u64()).map(|v| v as usize).collect();
        let verts: Vec<u32> = args["verts"].as_array().into_iter().flatten().filter_map(|v| v.as_u64()).map(|v| v as u32).collect();
        let edges: Vec<(u32, u32)> =
            args["edges"].as_array().into_iter().flatten().filter_map(|e| Some((e.get(0)?.as_u64()? as u32, e.get(1)?.as_u64()? as u32))).collect();
        let a = args.clone();
        let parent = self.state.doc.map.get(id).and_then(|n| n.parent).unwrap_or_else(|| self.state.insert_parent());
        let result: Result<Value, String> = self.state.doc.edit(&format!("Mesh {op}"), |m, s| {
            let Some(mesh) = m.mesh_mut(id) else { return Err("mesh vanished".to_string()) };
            let out = match op.as_str() {
                "extrude_faces" => {
                    let new = mesh.extrude_faces(&faces);
                    let offset = vec3(&a["offset"]).unwrap_or_else(|| {
                        let n: DVec3 = faces.iter().map(|f| mesh.face_normal(*f)).sum::<DVec3>().normalize_or(DVec3::Y);
                        n * a["distance"].as_f64().unwrap_or(16.0)
                    });
                    mesh.transform_vertices(&new, &DMat4::from_translation(offset));
                    json!({ "verts": new })
                }
                "extrude_edges" => {
                    let new: Vec<u32> = mesh.extrude_edges(&edges).into_iter().map(|(_, n)| n).collect();
                    mesh.transform_vertices(&new, &DMat4::from_translation(vec3(&a["offset"]).unwrap_or(DVec3::Y * 16.0)));
                    json!({ "verts": new })
                }
                "inset" => json!({ "faces": mesh.inset_faces(&faces, a["thickness"].as_f64().unwrap_or(4.0)) }),
                "loop_cut" => {
                    let Some(edge) = edges.first().copied() else { return Err("edges needs one edge".to_string()) };
                    json!({ "verts": mesh.loop_cut(edge, a["cuts"].as_u64().unwrap_or(1) as usize) })
                }
                "bisect" => {
                    let (Some(point), Some(normal)) = (vec3(&a["point"]), vec3(&a["normal"])) else { return Err("point and normal required".to_string()) };
                    let plane = Plane::from_point_normal(point, normal);
                    let on = mesh.bisect(&plane, (!faces.is_empty()).then_some(faces.as_slice()));
                    match a["delete"].as_str() {
                        Some("front") => mesh.delete_side(&plane, false),
                        Some("back") => mesh.delete_side(&plane, true),
                        _ => {}
                    }

                    if a["fill"].as_bool().unwrap_or(false) {
                        let set = mesh.vertices.iter().enumerate().filter(|(_, v)| plane.distance(**v).abs() < 1e-4).map(|(i, _)| i as u32).collect();
                        let data = mesh.faces.first().map(|f| f.data.clone()).unwrap_or_default();
                        mesh.fill_boundary_loops(&set, &data);
                    }

                    json!({ "cut_vertices": on.len() })
                }
                "subdivide" => json!({ "faces": mesh.subdivide_faces(&faces) }),
                "merge" => {
                    let at = vec3(&a["at"]).unwrap_or_else(|| verts.iter().map(|v| mesh.vertices[*v as usize]).sum::<DVec3>() / verts.len().max(1) as f64);
                    mesh.merge_vertices(&verts, at);
                    json!({})
                }
                "weld" => json!({ "merged": mesh.weld(a["distance"].as_f64().unwrap_or(0.01)) }),
                "fill" => {
                    let data = mesh.faces.first().map(|f| f.data.clone()).unwrap_or_default();
                    json!({ "face": mesh.fill(&verts, &data) })
                }
                "delete_faces" => {
                    mesh.delete_faces(&faces);
                    json!({})
                }
                "delete_faces_in_box" => {
                    let (Some(min), Some(max)) = (vec3(&a["min"]), vec3(&a["max"])) else { return Err("min and max required".to_string()) };
                    let region = Aabb::new(min, max);
                    let inside: Vec<usize> = (0..mesh.faces.len()).filter(|f| region.contains_point(mesh.face_center(*f))).collect();
                    mesh.delete_faces(&inside);
                    json!({ "deleted": inside.len() })
                }
                "set_material_in_box" => {
                    let (Some(min), Some(max)) = (vec3(&a["min"]), vec3(&a["max"])) else { return Err("min and max required".to_string()) };
                    let region = Aabb::new(min, max);
                    let material = a["material"].as_str().unwrap_or_default().to_string();
                    let mut n = 0;
                    for f in 0..mesh.faces.len() {
                        if region.contains_point(mesh.face_center(f)) {
                            mesh.faces[f].data.material = material.clone();
                            n += 1;
                        }
                    }

                    json!({ "changed": n })
                }
                "delete_vertices" => {
                    mesh.delete_vertices(&verts);
                    json!({})
                }
                "bevel_edges" => json!({ "faces": mesh.bevel_edges(&edges, a["width"].as_f64().unwrap_or(4.0)) }),
                "bevel_vertices" => json!({ "faces": mesh.bevel_vertices(&verts, a["width"].as_f64().unwrap_or(4.0)) }),
                "translate" => {
                    mesh.transform_vertices(&verts, &DMat4::from_translation(vec3(&a["offset"]).unwrap_or_default()));
                    json!({})
                }
                "smooth" => {
                    let targets: Vec<u32> = if verts.is_empty() { (0..mesh.vertices.len() as u32).collect() } else { verts.clone() };
                    mesh.smooth_vertices(&targets, a["factor"].as_f64().unwrap_or(0.5), a["iterations"].as_u64().unwrap_or(1) as usize);
                    json!({})
                }
                "solidify" => {
                    mesh.solidify(a["thickness"].as_f64().unwrap_or(8.0));
                    json!({})
                }
                "flip" => {
                    mesh.flip_faces(&faces);
                    json!({})
                }
                "triangulate" => {
                    mesh.triangulate_faces(&faces);
                    json!({})
                }
                "smooth_angle" => {
                    mesh.smooth_angle = a["angle"].as_f64().unwrap_or(60.0) as f32;
                    json!({})
                }
                "set_material" => {
                    let material = a["material"].as_str().unwrap_or_default();
                    for f in &faces {
                        if let Some(face) = mesh.faces.get_mut(*f) {
                            face.data.material = material.to_string();
                        }
                    }

                    json!({})
                }
                "separate" => {
                    let part = mesh.separate(&faces);
                    let new_id = m.insert(parent, NodeKind::Mesh(part));
                    s.nodes.insert(new_id);
                    json!({ "id": new_id.0 })
                }
                other => return Err(format!("unknown mesh_edit op {other}")),
            };
            Ok(out)
        });
        match result {
            Ok(mut v) => {
                if let Some(mesh) = self.state.doc.map.mesh(id) {
                    v["vertices"] = json!(mesh.vertices.len());
                    v["face_count"] = json!(mesh.faces.len());
                    v["closed"] = json!(mesh.is_closed());
                    v["bounds"] = bounds_json(&mesh.bounds());
                }

                ok(v)
            }
            Err(e) => {
                self.state.doc.undo();
                err(e)
            }
        }
    }

    pub(crate) fn tool_texture(&mut self, args: &Value) -> Option<ToolResult> {
        use crate::texture_ops as tex;
        let face_list = |v: &Value| -> Vec<(NodeId, usize)> {
            v.as_array().into_iter().flatten().filter_map(|f| Some((NodeId(f.get(0)?.as_u64()?), f.get(1)?.as_u64()? as usize))).collect()
        };
        let mut faces = face_list(&args["faces"]);
        if faces.is_empty() {
            faces = tex::target_faces(&self.state);
        }

        let vec2 = |v: &Value| -> Option<DVec2> {
            match v {
                Value::Number(n) => n.as_f64().map(DVec2::splat),
                _ => vec2(v),
            }
        };
        let op = args["op"].as_str().unwrap_or_default();
        let changed = match op {
            "apply" => {
                let material = args["material"].as_str().map(str::to_string).unwrap_or_else(|| self.state.current_material.clone());
                tex::apply_material(&mut self.state, &faces, &material, args["reset"].as_bool().unwrap_or(false))
            }
            "justify" => {
                let Ok(mode) = serde_json::from_value::<gt_geom::Justify>(args["mode"].clone()) else {
                    return err("mode must be left, right, top, bottom, center, fit, fit_width or fit_height");
                };
                let as_one = args["treat_as_one"].as_bool().unwrap_or(self.state.treat_as_one);
                tex::justify(&mut self.state, &faces, mode, as_one)
            }
            "align_view" => {
                let cam = &self.viewports[0].camera;
                let (right, up) = (cam.right(), cam.up());
                tex::align_to_view(&mut self.state, &faces, right, up)
            }
            "reset" => tex::reset(&mut self.state, &faces, vec2(&args["scale"]).unwrap_or(DVec2::ONE)),
            "wrap" => {
                let source = face_list(&json!([args["source"].clone()]));
                let Some(src) = source.first().copied() else { return err("source [node, face] required") };
                let targets: Vec<(NodeId, usize)> = faces.iter().copied().filter(|f| *f != src).collect();
                tex::wrap_from(&mut self.state, src, &targets, args["material"].as_str())
            }
            "shift" => tex::shift(&mut self.state, &faces, vec2(&args["texels"]).unwrap_or_default()),
            "scale" => tex::scale(&mut self.state, &faces, vec2(&args["factor"]).unwrap_or(DVec2::ONE)),
            "rotate" => tex::rotate(&mut self.state, &faces, args["degrees"].as_f64().unwrap_or(90.0)),
            "density" => tex::set_density(&mut self.state, &faces, args["units_per_texel"].as_f64().unwrap_or(1.0)),
            "pick" => usize::from(faces.first().is_some_and(|f| tex::eyedropper(&mut self.state, *f))),
            "paste" => tex::paste_alignment(&mut self.state, &faces, args["with_material"].as_bool().unwrap_or(false)),
            "mesh_uv" => {
                let Some(kind) = args["kind"].as_str().and_then(tex::MeshUvKind::from_name) else {
                    return err("kind must be planar, box, cylinder, sphere, view, unfold or normalize");
                };
                let cam = &self.viewports[0].camera;
                let view = (cam.right(), cam.up());
                tex::mesh_uv(&mut self.state, &faces, kind, view)
            }
            "set_hotspots" => {
                let material = args["material"].as_str().map(str::to_string).unwrap_or_else(|| self.state.current_material.clone());
                let rects: Vec<[f64; 4]> = args["rects"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|r| {
                        let a = r.as_array()?;
                        Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?, a.get(2)?.as_f64()?, a.get(3)?.as_f64()?])
                    })
                    .collect();
                if let Err(e) = crate::commands::write_hotspots(&self.state, &material, &rects) {
                    return err(e);
                }

                rects.len()
            }
            "material_info" => {
                let material = args["material"].as_str().map(str::to_string).unwrap_or_else(|| self.state.current_material.clone());
                let info = self.state.materials.info(&material);
                let size = self.state.materials.load_material(&material).map(|m| [m.albedo.width(), m.albedo.height()]);
                return ok(json!({
                    "material": material,
                    "size": size,
                    "material_file": self.state.materials.find(&material).and_then(|e| e.material_file.clone()),
                    "summary": info.as_ref().map(crate::panels::material_summary).unwrap_or_default().trim_start_matches(", "),
                    "transparent": info.as_ref().is_some_and(|i| i.is_transparent()),
                    "emissive": info.as_ref().is_some_and(|i| i.emission.is_some()),
                    "nearest": info.as_ref().and_then(|i| i.nearest),
                    "hotspots": crate::commands::hotspot_rects(&self.state, &material),
                }));
            }
            "" | "get" => 0,
            other => return err(format!("unknown texture op {other}")),
        };
        let map = &self.state.doc.map;
        let details: Vec<Value> = faces
            .iter()
            .take(64)
            .filter_map(|(id, f)| tex::face_info(map, *id, *f))
            .map(|i| {
                json!({
                    "face": [i.id.0, i.face], "material": i.material,
                    "uv": serde_json::to_value(&i.uv).unwrap_or_default(),
                    "explicit": i.explicit, "normal": [i.plane.normal.x, i.plane.normal.y, i.plane.normal.z],
                })
            })
            .collect();
        ok(json!({ "changed": changed, "faces": details, "clipboard": self.state.uv_clipboard.as_ref().map(|c| c.material.clone()) }))
    }

    pub(crate) fn tool_create_terrain(&mut self, args: &Value) -> Option<ToolResult> {
        let resolution = args["resolution"].as_u64().unwrap_or(65).clamp(3, 2049) as u32;
        let cell = args["cell_size"].as_f64().unwrap_or(64.0);
        let origin = vec3(&args["origin"]).unwrap_or_else(|| {
            let side = (resolution - 1) as f64 * cell;
            DVec3::new(-side * 0.5, 0.0, -side * 0.5)
        });
        let mut params = gt_geom::heightfield::TerrainGen::default();
        if let Some(shape) = args["shape"].as_str() {
            match serde_json::from_value(json!(shape)) {
                Ok(s) => params.shape = s,
                Err(_) => return err("shape must be flat, hills, mountain, island, valley or ridges"),
            }
        }

        params.height = args["height"].as_f64().unwrap_or(params.height);
        params.seed = args["seed"].as_u64().unwrap_or(1) as u32;
        params.feature_size = args["feature_size"].as_f64().unwrap_or(params.feature_size);
        params.erosion_iterations = args["erosion"].as_u64().unwrap_or(params.erosion_iterations as u64) as u32;
        let layers: Vec<(String, f64)> = match args["layers"].as_array() {
            Some(list) => list.iter().filter_map(|l| Some((l.get(0)?.as_str()?.to_string(), l.get(1).and_then(|t| t.as_f64()).unwrap_or(256.0)))).collect(),
            None => vec![(self.state.current_material.clone(), 256.0)],
        };
        let t = crate::dialogs::make_terrain(origin, resolution, cell, &params, &layers, None, args["auto_paint"].as_bool().unwrap_or(true));
        let bounds = t.bounds();
        let parent = args["parent"].as_u64().map(NodeId).filter(|p| self.state.doc.map.contains(*p)).unwrap_or_else(|| self.state.insert_parent());
        let id = self.state.doc.edit("Create Terrain", |m, s| {
            let id = m.insert(parent, NodeKind::Terrain(t));
            s.clear();
            s.select_node(id);
            id
        });
        ok(json!({ "id": id.0, "bounds": bounds_json(&bounds) }))
    }
}
