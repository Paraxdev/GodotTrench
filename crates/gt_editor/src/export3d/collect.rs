//! Turns a map into an export scene: named nodes in the Outliner's layout, triangle meshes in meters and materials.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gt_core::{Aabb, DMat4, DQuat, DVec2, DVec3, NodeId};
use gt_doc::{Entity, Map, NodeKind};
use gt_formats::GameConfig;
use gt_geom::{Brush, Mesh, Terrain};

use super::textures::Materials;
use super::{Options, Sources};
use crate::face_cull::{FaceCull, FacePieces};
use crate::materials::MaterialLibrary;
use crate::models::{Model, ModelCache};
use crate::prefabs::{self, PrefabCache};

pub struct Node {
    pub name: String,
    /// Relative to the parent, in meters.
    pub translation: DVec3,
    pub rotation: DQuat,
    pub scale: DVec3,
    pub mesh: Option<usize>,
    pub children: Vec<usize>,
}

impl Node {
    fn new(name: String) -> Self {
        Self { name, translation: DVec3::ZERO, rotation: DQuat::IDENTITY, scale: DVec3::ONE, mesh: None, children: Vec::new() }
    }

    pub fn matrix(&self) -> DMat4 {
        DMat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

/// Triangles of one material, in meters relative to the node.
pub struct Primitive {
    pub material: usize,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

pub struct MeshData {
    pub name: String,
    pub primitives: Vec<Primitive>,
}

pub struct Scene {
    pub name: String,
    pub nodes: Vec<Node>,
    pub roots: Vec<usize>,
    pub meshes: Vec<MeshData>,
    pub materials: Materials,
}

impl Scene {
    /// Triangles as placed, a mesh used by several nodes counts once per node.
    pub fn triangles(&self) -> usize {
        let per_mesh: Vec<usize> = self.meshes.iter().map(|m| m.primitives.iter().map(|p| p.indices.len() / 3).sum()).collect();
        self.nodes.iter().filter_map(|n| n.mesh).map(|m| per_mesh[m]).sum()
    }
}

/// Triangles of one material in map units, world space.
#[derive(Default)]
struct Triangles {
    material: usize,
    positions: Vec<DVec3>,
    normals: Vec<DVec3>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u32>,
}

/// Collects the triangles of one mesh, one list per material.
#[derive(Default)]
struct MeshBuilder {
    prims: Vec<Triangles>,
    bounds: Aabb,
}

fn uv(v: DVec2) -> [f32; 2] {
    [v.x as f32, v.y as f32]
}

impl MeshBuilder {
    fn triangles(&mut self, material: usize, positions: &[DVec3], normals: &[DVec3], uvs: &[[f32; 2]], indices: &[u32]) {
        if indices.is_empty() {
            return;
        }

        for p in positions {
            self.bounds.include_point(*p);
        }

        let i = match self.prims.iter().position(|p| p.material == material) {
            Some(i) => i,
            None => {
                self.prims.push(Triangles { material, ..Default::default() });
                self.prims.len() - 1
            }
        };
        let prim = &mut self.prims[i];
        let base = prim.positions.len() as u32;
        prim.positions.extend_from_slice(positions);
        prim.normals.extend_from_slice(normals);
        prim.uvs.extend_from_slice(uvs);
        prim.indices.extend(indices.iter().map(|i| i + base));
    }

    /// A convex polygon as a triangle fan.
    fn polygon(&mut self, material: usize, points: &[DVec3], normals: &[DVec3], uvs: &[[f32; 2]]) {
        if points.len() < 3 {
            return;
        }

        let fan: Vec<u32> = (1..points.len() as u32 - 1).flat_map(|k| [0, k, k + 1]).collect();
        self.triangles(material, points, normals, uvs, &fan);
    }

    fn is_empty(&self) -> bool {
        self.prims.is_empty()
    }

    /// The mesh in meters around `center`.
    fn finish(self, name: &str, center: DVec3, units_per_meter: f64) -> MeshData {
        let primitives = self
            .prims
            .into_iter()
            .map(|t| Primitive {
                material: t.material,
                positions: t.positions.iter().map(|p| ((*p - center) / units_per_meter).as_vec3().to_array()).collect(),
                normals: t.normals.iter().map(|n| n.try_normalize().unwrap_or(DVec3::Y).as_vec3().to_array()).collect(),
                uvs: t.uvs,
                indices: t.indices,
            })
            .collect();
        MeshData { name: name.to_string(), primitives }
    }
}

fn entity_scale(e: &Entity) -> f64 {
    e.property("scale").and_then(|s| s.parse::<f64>().ok()).filter(|s| *s > 0.0).unwrap_or(1.0)
}

struct Collector<'a> {
    game: &'a GameConfig,
    lib: &'a mut MaterialLibrary,
    models: &'a mut ModelCache,
    prefabs: &'a mut PrefabCache,
    selection: &'a BTreeSet<NodeId>,
    options: &'a Options,
    upm: f64,
    pieces: HashMap<NodeId, FacePieces>,
    nodes: Vec<Node>,
    meshes: Vec<MeshData>,
    materials: Materials,
    /// Mesh of a model file, per material override, shared by every prop and scatter instance showing it.
    model_meshes: HashMap<(PathBuf, Option<String>), usize>,
}

pub fn collect(src: Sources, options: &Options, name: String) -> Scene {
    let Sources { map, map_path, game, materials: lib, models, prefabs, selection } = src;
    // Hidden faces are left out like the Godot build does, which also keeps overlapping faces from z-fighting.
    let brush_faces = map.brushes().flat_map(|(_, b)| b.faces.iter().map(|f| f.data.material.as_str()));
    let face_materials: BTreeSet<&str> = brush_faces.chain(map.meshes().flat_map(|(_, m)| m.faces.iter().map(|f| f.data.material.as_str()))).collect();
    let opaque: HashMap<String, bool> = face_materials
        .into_iter()
        .map(|m| {
            let info = lib.info(m).unwrap_or_default();
            (m.to_string(), info.transparency != gt_formats::godot_material::Transparency::Alpha && info.albedo_color[3] >= 0.999 && !info.double_sided)
        })
        .collect();
    // Only what goes into the file hides faces: an unselected floor must not eat the base of a pillar exported alone.
    let mut cull = FaceCull::default();
    let exported = |id: NodeId| super::in_scope(map, id, selection, options);
    cull.update_among(map, game, &|m: &str| opaque.get(m).copied().unwrap_or(true), &exported, &BTreeSet::new(), true);
    let mut c = Collector {
        game,
        lib,
        models,
        prefabs,
        selection,
        options,
        upm: game.units_per_meter.max(1e-6),
        pieces: std::mem::take(&mut cull.pieces),
        nodes: Vec::new(),
        meshes: Vec::new(),
        materials: Materials::default(),
        model_meshes: HashMap::new(),
    };
    let roots = map.layers.iter().filter_map(|l| c.walk(map, map_path, *l, false)).collect();
    Scene { name, nodes: c.nodes, roots, meshes: c.meshes, materials: c.materials }
}

impl Collector<'_> {
    fn push(&mut self, node: Node) -> usize {
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    fn container(&mut self, name: String, children: Vec<usize>) -> Option<usize> {
        (!children.is_empty()).then(|| self.push(Node { children, ..Node::new(name) }))
    }

    fn mesh_node(&mut self, name: String, builder: MeshBuilder) -> Option<usize> {
        if builder.is_empty() {
            return None;
        }

        let center = builder.bounds.center();
        self.meshes.push(builder.finish(&name, center, self.upm));
        let mesh = Some(self.meshes.len() - 1);
        Some(self.push(Node { translation: center / self.upm, mesh, ..Node::new(name) }))
    }

    /// The node for `id` and what it holds, None when nothing in it goes out.
    fn walk(&mut self, map: &Map, map_path: Option<&Path>, id: NodeId, chosen: bool) -> Option<usize> {
        let node = map.get(id)?;
        if node.hidden && !self.options.hidden {
            return None;
        }

        let chosen = chosen || !self.options.selection_only || self.selection.contains(&id);
        let name = node.name();
        match &node.kind {
            NodeKind::Layer(l) if l.omit_from_export => None,
            NodeKind::Entity(e) if !node.children.is_empty() && crate::scene::is_volume(self.game, e) => None,
            NodeKind::Layer(_) | NodeKind::Group(_) | NodeKind::Entity(_) if !node.children.is_empty() => {
                let mut children = Vec::new();
                let mut merged = MeshBuilder::default();
                for child in &node.children {
                    let merge = self.options.merge_brushes
                        && chosen
                        && map.get(*child).is_some_and(|n| matches!(n.kind, NodeKind::Brush(_)) && n.label.is_none() && (self.options.hidden || !n.hidden));
                    if merge {
                        if let Some(brush) = map.brush(*child) {
                            let pieces = self.pieces.remove(child);
                            self.brush(&mut merged, brush, pieces.as_ref());
                        }
                    } else {
                        children.extend(self.walk(map, map_path, *child, chosen));
                    }
                }

                children.extend(self.mesh_node(format!("{name} brushes"), merged));
                self.container(name, children)
            }
            _ if !chosen => None,
            NodeKind::Entity(e) => self.point_entity(name, e),
            NodeKind::Brush(b) => {
                let mut builder = MeshBuilder::default();
                let pieces = self.pieces.remove(&id);
                self.brush(&mut builder, b, pieces.as_ref());
                self.mesh_node(name, builder)
            }
            NodeKind::Mesh(m) => {
                let mut builder = MeshBuilder::default();
                let pieces = self.pieces.remove(&id);
                self.mesh(&mut builder, m, pieces.as_ref());
                self.mesh_node(name, builder)
            }
            NodeKind::Terrain(t) => self.terrain(name, t),
            NodeKind::Scatter(set) => self.scatter(name, set),
            NodeKind::Instance(inst) => {
                let path = prefabs::resolve(&inst.path, map_path, self.prefabs.project_root.as_deref())?;
                self.prefab(name, &path, &prefabs::instance_transform(inst), 0)
            }
            NodeKind::Layer(_) | NodeKind::Group(_) => None,
        }
    }

    fn brush(&mut self, out: &mut MeshBuilder, brush: &Brush, pieces: Option<&FacePieces>) {
        // Like Hammer, only the displacement surfaces of a displacement brush are real geometry.
        let has_disp = brush.faces.iter().any(|f| f.data.disp.is_some());
        for (fi, face) in brush.faces.iter().enumerate() {
            if !crate::zfight::draws(self.game, &face.data.material) || (has_disp && face.data.disp.is_none()) {
                continue;
            }

            let whole = || vec![face.indices.iter().map(|i| brush.vertices[*i as usize]).collect::<Vec<DVec3>>()];
            let polygons = pieces.and_then(|p| p.get(&fi)).cloned().unwrap_or_else(whole);
            if polygons.iter().all(|p| p.len() < 3) {
                continue;
            }

            let (mat, size) = self.materials.face(self.lib, self.game, &face.data.material, false);
            if let Some(grid) = gt_geom::displacement::grid(brush, fi) {
                let uvs: Vec<[f32; 2]> = grid.base.iter().map(|p| uv(face.data.uv.uv(*p, size))).collect();
                let indices: Vec<u32> = gt_geom::displacement::triangles(grid.size).flat_map(|(a, b, c)| [a as u32, b as u32, c as u32]).collect();
                out.triangles(mat, &grid.positions, &grid.normals, &uvs, &indices);
                continue;
            }

            for poly in polygons {
                let normals = vec![face.plane.normal; poly.len()];
                let uvs: Vec<[f32; 2]> = poly.iter().map(|p| uv(face.data.uv.uv(*p, size))).collect();
                out.polygon(mat, &poly, &normals, &uvs);
            }
        }
    }

    fn mesh(&mut self, out: &mut MeshBuilder, mesh: &Mesh, pieces: Option<&FacePieces>) {
        let normals = mesh.corner_normals();
        for (fi, face) in mesh.faces.iter().enumerate() {
            if face.indices.len() < 3
                || face.indices.iter().any(|i| *i as usize >= mesh.vertices.len())
                || !crate::zfight::draws(self.game, &face.data.material)
            {
                continue;
            }

            let pieces = pieces.and_then(|p| p.get(&fi));
            if pieces.is_some_and(|p| p.iter().all(|piece| piece.len() < 3)) {
                continue;
            }

            let (mat, size) = self.materials.face(self.lib, self.game, &face.data.material, mesh.decal);
            let corners = mesh.face_points(fi);
            let uvs: Vec<DVec2> = (0..corners.len()).map(|k| mesh.corner_uv(fi, k, size)).collect();
            let tris = mesh.triangulate_corners(fi);
            match pieces {
                Some(pieces) => {
                    for piece in pieces {
                        let weights: Vec<[(usize, f64); 3]> = piece.iter().map(|p| gt_geom::polygon::corner_weights(&corners, &tris, *p)).collect();
                        let piece_normals: Vec<DVec3> = weights.iter().map(|w| w.iter().map(|(k, t)| normals[fi][*k] * *t).sum()).collect();
                        let piece_uvs: Vec<[f32; 2]> = weights.iter().map(|w| uv(w.iter().map(|(k, t)| uvs[*k] * *t).sum())).collect();
                        out.polygon(mat, piece, &piece_normals, &piece_uvs);
                    }
                }
                None => {
                    let uvs: Vec<[f32; 2]> = uvs.into_iter().map(uv).collect();
                    let indices: Vec<u32> = tris.iter().flat_map(|t| t.map(|k| k as u32)).collect();
                    out.triangles(mat, &corners, &normals[fi], &uvs, &indices);
                }
            }
        }
    }

    /// Each triangle in the material of its strongest layer, a stepped version of the blend Godot paints.
    fn terrain(&mut self, name: String, t: &Terrain) -> Option<usize> {
        if !t.is_valid() {
            return None;
        }

        let layer_count = t.layers.len().clamp(1, 4);
        let [cells_x, cells_z] = t.cells();
        // Each layer's triangles share vertices, `slots` maps a terrain vertex to its index there.
        let mut per_layer: Vec<(Vec<u32>, Triangles)> = (0..layer_count).map(|_| (vec![u32::MAX; t.heights.len()], Triangles::default())).collect();
        for cj in 0..cells_z {
            for ci in 0..cells_x {
                if t.is_hole(ci, cj) {
                    continue;
                }

                for tri in Terrain::cell_triangles(ci, cj) {
                    let mut sum = [0.0f32; 4];
                    for (i, j) in tri {
                        for (s, w) in sum.iter_mut().zip(t.weights(i, j)) {
                            *s += w;
                        }
                    }

                    let layer = (0..layer_count).max_by(|a, b| sum[*a].total_cmp(&sum[*b])).unwrap_or(0);
                    let tile = t.layers.get(layer).map(|l| l.tile.max(1.0)).unwrap_or(256.0);
                    let (slots, out) = &mut per_layer[layer];
                    for (i, j) in tri {
                        let k = t.index(i, j);
                        if slots[k] == u32::MAX {
                            let p = t.vertex(i, j);
                            slots[k] = out.positions.len() as u32;
                            out.positions.push(p);
                            out.normals.push(t.normal(i, j));
                            out.uvs.push([(p.x / tile) as f32, (p.z / tile) as f32]);
                        }

                        out.indices.push(slots[k]);
                    }
                }
            }
        }

        let mut builder = MeshBuilder::default();
        for (layer, (_, tris)) in per_layer.iter().enumerate().filter(|(_, (_, t))| !t.indices.is_empty()) {
            let material = match t.layers.get(layer) {
                Some(l) => self.materials.face(self.lib, self.game, &l.material, false).0,
                None => self.materials.plain("terrain"),
            };
            builder.triangles(material, &tris.positions, &tris.normals, &tris.uvs, &tris.indices);
        }

        self.mesh_node(name, builder)
    }

    /// The shared mesh of a model in meters, its parts in the model's own materials or in `material` when given.
    fn model_mesh(&mut self, path: &Path, model: &Model, material: Option<&str>) -> usize {
        let key = (path.to_path_buf(), material.map(str::to_string));
        if let Some(mesh) = self.model_meshes.get(&key) {
            return *mesh;
        }

        let (file, _) = crate::models::split_model_node(path);
        let label = file.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "model".into());
        let mut builder = MeshBuilder::default();
        for part in &model.parts {
            let mat = match material {
                Some(m) => self.materials.face(self.lib, self.game, m, false).0,
                None => self.materials.model(model, &part.material, &label),
            };
            let positions: Vec<DVec3> = part.vertices.iter().map(|v| v.pos.as_dvec3()).collect();
            let normals: Vec<DVec3> = part.vertices.iter().map(|v| v.normal.as_dvec3()).collect();
            let uvs: Vec<[f32; 2]> = part.vertices.iter().map(|v| v.uv).collect();
            builder.triangles(mat, &positions, &normals, &uvs, &part.indices);
        }

        self.meshes.push(builder.finish(&label, DVec3::ZERO, self.upm));
        let mesh = self.meshes.len() - 1;
        self.model_meshes.insert(key, mesh);
        mesh
    }

    fn point_entity(&mut self, name: String, e: &Entity) -> Option<usize> {
        let mut node = Node { translation: e.origin / self.upm, rotation: e.rotation(), ..Node::new(name) };
        if self.options.models
            && let Some(path) = crate::models::entity_model_path(self.game, e)
            && let Some(model) = self.models.get(&path, self.game.units_per_meter)
        {
            node.mesh = Some(self.model_mesh(&path, &model, None));
            node.scale = DVec3::splat(entity_scale(e));
            return Some(self.push(node));
        }

        self.options.markers.then(|| self.push(node))
    }

    fn scatter(&mut self, name: String, set: &gt_doc::Scatter) -> Option<usize> {
        if !self.options.scatter {
            return None;
        }

        let items: Vec<Option<(PathBuf, Arc<Model>)>> = set
            .items
            .iter()
            .map(|item| {
                let path = if item.source.starts_with("res://") { self.game.resolve_res(&item.source) } else { Some(PathBuf::from(&item.source)) };
                let path = path.filter(|p| crate::models::is_model_path(&p.to_string_lossy()))?;
                let model = self.models.get(&path, self.game.units_per_meter)?;
                Some((path, model))
            })
            .collect();
        let mut children = Vec::new();
        for inst in &set.instances {
            let Some(Some((path, model))) = items.get(inst.item as usize) else { continue };
            let mesh = self.model_mesh(path, model, set.item_material(inst.item as usize));
            let (scale, rotation, translation) = inst.transform().to_scale_rotation_translation();
            let label = set.items[inst.item as usize].label().to_string();
            children.push(self.push(Node { translation: translation / self.upm, rotation, scale, mesh: Some(mesh), ..Node::new(label) }));
        }

        self.container(name, children)
    }

    /// A prefab instance: what the prefab's exported layers hold, moved into place like the Godot build places it.
    fn prefab(&mut self, name: String, path: &Path, xform: &DMat4, depth: usize) -> Option<usize> {
        if depth > prefabs::MAX_DEPTH {
            return None;
        }

        let map = self.prefabs.get(path).map.clone()?;
        let mut children = Vec::new();
        for (id, node) in prefabs::exported_nodes(&map) {
            if (!self.options.hidden && map.is_hidden(id))
                || map.owning_entity(id).and_then(|e| map.entity(e)).is_some_and(|e| crate::scene::is_volume(self.game, e))
            {
                continue;
            }

            let child_name = node.name();
            let child = match &node.kind {
                NodeKind::Brush(b) => {
                    let mut builder = MeshBuilder::default();
                    self.brush(&mut builder, &b.transformed(xform, true), None);
                    self.mesh_node(child_name, builder)
                }
                NodeKind::Mesh(m) => {
                    let mut builder = MeshBuilder::default();
                    self.mesh(&mut builder, &m.transformed(xform, true), None);
                    self.mesh_node(child_name, builder)
                }
                NodeKind::Entity(e) if node.children.is_empty() => {
                    let mut placed = e.clone();
                    placed.transform_by(xform);
                    self.point_entity(child_name, &placed)
                }

                // Like the Godot build, instances move terrains but never rotate them.
                NodeKind::Terrain(t) => self.terrain(child_name, &t.translated(xform.w_axis.truncate())),
                NodeKind::Scatter(set) => {
                    let mut placed = set.clone();
                    placed.transform(xform);
                    self.scatter(child_name, &placed)
                }
                NodeKind::Instance(inner) => {
                    let root = self.prefabs.project_root.clone();
                    prefabs::resolve(&inner.path, Some(path), root.as_deref())
                        .and_then(|p| self.prefab(child_name, &p, &(*xform * prefabs::instance_transform(inner)), depth + 1))
                }
                _ => None,
            };
            children.extend(child);
        }

        self.container(name, children)
    }
}
