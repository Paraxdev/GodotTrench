use std::collections::{BTreeMap, BTreeSet, HashMap};

use glam::Vec3;
use gt_core::{Aabb, Color, DMat4, DVec2, DVec3, NodeId};
use gt_doc::{Entity, Map, NodeKind, Selection};
use gt_formats::{EntityDef, GameConfig};
use gt_geom::{Brush, Mesh, Terrain};
use gt_render::{Frame, GpuLines, GpuMesh, Lighting, LineVertex, MeshBatch, MeshVertex, PointLight, Renderer};

use crate::face_cull::{FaceCull, FacePieces};
use crate::prefabs::{self, PrefabCache};
use crate::state::EditorState;

pub const EDGE_COLOR: [f32; 4] = [0.85, 0.85, 0.85, 0.28];
pub const EDGE_COLOR_2D: [f32; 4] = [0.85, 0.85, 0.85, 0.75];
pub const SELECTED_COLOR: [f32; 4] = [1.0, 0.15, 0.1, 1.0];
pub const SELECTED_XRAY: [f32; 4] = [1.0, 0.2, 0.1, 0.35];
const SELECTED_TINT: [f32; 4] = [1.0, 0.72, 0.68, 1.0];
const LOCKED_TINT: [f32; 4] = [0.7, 0.75, 0.9, 1.0];
const INSTANCE_TINT: [f32; 4] = [0.8, 0.92, 1.0, 1.0];
const INSTANCE_EDGE: [f32; 4] = [0.45, 0.85, 1.0, 0.8];
const TARGET_LINK: [f32; 4] = [0.35, 0.95, 0.45, 0.55];
const IO_LINK: [f32; 4] = [1.0, 0.62, 0.2, 0.6];
const TERRAIN_EDGE: [f32; 4] = [0.55, 0.8, 0.45, 0.7];
const BUCKETS: u64 = 64;
/// Above this many faces in the moving selection, face culling is deferred to drag release. Ordinary
/// brush drags stay well under it and keep live culling.
const CULL_DEFER_FACES: usize = 4000;

fn node_face_count(node: &gt_doc::Node) -> usize {
    match &node.kind {
        NodeKind::Brush(b) => b.faces.len(),
        NodeKind::Mesh(m) => m.faces.len(),
        _ => 0,
    }
}

#[derive(Default, Clone, Copy)]
pub struct SceneStats {
    pub brushes: usize,
    pub meshes: usize,
    pub terrains: usize,
    pub entities: usize,
    pub triangles: usize,
}

impl std::ops::AddAssign for SceneStats {
    fn add_assign(&mut self, o: Self) {
        self.brushes += o.brushes;
        self.meshes += o.meshes;
        self.terrains += o.terrains;
        self.entities += o.entities;
        self.triangles += o.triangles;
    }
}

/// GPU data of the nodes whose id falls into one bucket. Only buckets with changed nodes are rebuilt.
#[derive(Default)]
struct Bucket {
    opaque: Option<GpuMesh>,
    double: Option<GpuMesh>,
    transparent: Option<GpuMesh>,
    volumes: Option<GpuMesh>,
    overlay: Option<GpuMesh>,
    edges: Option<GpuLines>,
    edges_2d: Option<GpuLines>,
    sel_edges: Option<GpuLines>,
    xray: Option<GpuLines>,
    stats: SceneStats,
    instance_bounds: HashMap<NodeId, Aabb>,
    model_bounds: HashMap<NodeId, Aabb>,
}

struct TerrainGpu {
    chunks: Vec<Option<GpuMesh>>,
    lines_2d: Option<GpuLines>,
    sel_lines: Option<GpuLines>,
    /// Height following grid lines, only built in wireframe mode.
    wire: Option<GpuLines>,
    triangles: usize,
}

/// Geometry being interactively moved, kept as its pre-drag batches so each frame only re-uploads a
/// translated copy instead of re-tessellating. These nodes are left out of the buckets while the drag runs.
struct DragLayer {
    nodes: BTreeSet<NodeId>,
    opaque: MeshBatch,
    double: MeshBatch,
    transparent: MeshBatch,
    edges: Vec<LineVertex>,
    gpu_opaque: Option<GpuMesh>,
    gpu_double: Option<GpuMesh>,
    gpu_transparent: Option<GpuMesh>,
    gpu_edges: Option<GpuLines>,
}

/// Builds the batches of the dragged nodes from their pre-drag geometry, drawn un-culled and selected.
fn build_drag_batches(renderer: &Renderer, game: &GameConfig, base: &Map, nodes: &BTreeSet<NodeId>) -> DragLayer {
    let mut b = Builder {
        renderer,
        game,
        fallback: game.textures.fallback_size as f64,
        opaque: MeshBatch::default(),
        double: MeshBatch::default(),
        transparent: MeshBatch::default(),
        volumes: MeshBatch::default(),
        volume: false,
        face_overlay: MeshBatch::default(),
        edges: Vec::new(),
        edges_2d: Vec::new(),
        sel_edges: Vec::new(),
        stats: SceneStats::default(),
        instance_bounds: HashMap::new(),
        model_bounds: HashMap::new(),
    };
    for id in nodes {
        match base.get(*id).map(|n| &n.kind) {
            Some(NodeKind::Brush(brush)) => b.brush(brush, SELECTED_TINT, true, |_| false, EDGE_COLOR_2D, false, EDGE_COLOR, None),
            Some(NodeKind::Mesh(mesh)) => b.mesh(mesh, SELECTED_TINT, true, |_| false, EDGE_COLOR_2D, false, None),
            _ => {}
        }
    }

    DragLayer {
        nodes: nodes.clone(),
        opaque: b.opaque,
        double: b.double,
        transparent: b.transparent,
        edges: b.sel_edges,
        gpu_opaque: None,
        gpu_double: None,
        gpu_transparent: None,
        gpu_edges: None,
    }
}

fn translate_lines(lines: &[LineVertex], offset: DVec3) -> Vec<LineVertex> {
    let o = v3(offset);
    lines.iter().map(|l| LineVertex { pos: [l.pos[0] + o[0], l.pos[1] + o[1], l.pos[2] + o[2]], color: l.color }).collect()
}

#[derive(Default)]
pub struct SceneCache {
    prev_map: Option<Map>,
    prev_selection: Selection,
    revision: u64,
    project_generation: u64,
    prefab_generation: u64,
    model_generation: u64,
    model_reloads: u64,
    lit: bool,
    wireframe: bool,
    buckets: Vec<Bucket>,
    terrains: HashMap<NodeId, TerrainGpu>,
    scatters: HashMap<NodeId, ScatterGpu>,
    links: Option<GpuLines>,
    cordon: Option<GpuLines>,
    pub stats: SceneStats,
    pub lighting: Lighting,
    shadow_dirty: bool,
    shadow_center: Option<DVec3>,
    scene_bounds: Aabb,
    face_cull: FaceCull,
    drag: Option<DragLayer>,
}

pub fn v3(v: DVec3) -> [f32; 3] {
    [v.x as f32, v.y as f32, v.z as f32]
}

fn def_color(def: Option<&EntityDef>) -> Color {
    def.map(|d| d.color).unwrap_or(Color::rgb(0.8, 0.5, 1.0))
}

pub fn is_decal(def: Option<&EntityDef>) -> bool {
    def.is_some_and(|d| d.node_class == "Decal" || d.classname.contains("decal"))
}

/// Decal projector size in map units from the "size" property ("x depth z").
pub fn decal_size(e: &Entity) -> DVec3 {
    let parts: Vec<f64> = e.property("size").unwrap_or("64 32 64").split_whitespace().filter_map(|p| p.parse().ok()).collect();
    if parts.len() >= 3 { DVec3::new(parts[0], parts[1], parts[2]) } else { DVec3::new(64.0, 32.0, 64.0) }
}

pub fn entity_box(game: &GameConfig, e: &Entity) -> Aabb {
    game.entity(&e.classname).map(|d| d.bounds()).unwrap_or(Aabb::new(DVec3::splat(-8.0), DVec3::splat(8.0))).translated(e.origin)
}

/// "r g b" in 0..255 (FuncGodot) or 0..1.
pub fn parse_color(s: &str) -> Option<Vec3> {
    let v: Vec<f32> = s.split_whitespace().filter_map(|p| p.parse().ok()).collect();
    if v.len() < 3 {
        return None;
    }

    let scale = if v.iter().take(3).any(|c| *c > 1.0) { 255.0 } else { 1.0 };
    Some(Vec3::new(v[0], v[1], v[2]) / scale)
}

fn linear(c: Vec3) -> Vec3 {
    Vec3::new(gt_render::srgb_to_linear(c.x), gt_render::srgb_to_linear(c.y), gt_render::srgb_to_linear(c.z))
}

/// Materials used by decal sheets are registered under this prefix with alpha cutout and double sided
/// on, so the same texture can still render opaque on ordinary geometry.
const DECAL_PREFIX: &str = "decal::";

pub fn decal_key(material: &str) -> String {
    format!("{DECAL_PREFIX}{material}")
}

/// Renderer description of a loaded Godot material.
pub fn material_desc(m: &crate::materials::LoadedMaterial, filter: crate::state::TextureFilter) -> gt_render::MaterialDesc<'_> {
    use gt_formats::godot_material::Transparency;
    use gt_render::AlphaMode;
    let i = &m.info;
    let tint = linear(Vec3::new(i.albedo_color[0], i.albedo_color[1], i.albedo_color[2]));
    let emission = i.emission.map(|e| linear(Vec3::from_array(e))).unwrap_or(Vec3::ZERO);
    gt_render::MaterialDesc {
        albedo: &m.albedo,
        normal: m.normal.as_ref(),
        emission_texture: m.emission.as_ref(),
        tint: [tint.x, tint.y, tint.z, i.albedo_color[3]],
        emission: emission.to_array(),
        emission_energy: if i.emission.is_some() { i.emission_energy } else { 0.0 },
        emission_multiply: i.emission_multiply,
        alpha: match i.transparency {
            Transparency::Opaque => AlphaMode::Opaque,
            Transparency::Alpha => AlphaMode::Blend,
            Transparency::Scissor(t) => AlphaMode::Scissor(t),
            Transparency::Hash => AlphaMode::Hash,
        },
        nearest: match filter {
            crate::state::TextureFilter::Auto => i.nearest.unwrap_or(false),
            crate::state::TextureFilter::Nearest => true,
            crate::state::TextureFilter::Linear => false,
        },
        unshaded: i.unshaded,
        double_sided: i.double_sided,
        normal_scale: i.normal_scale,
        world_size: i.texture_size,
    }
}

fn entity_scale(e: &Entity) -> f64 {
    e.property("scale").and_then(|s| s.parse::<f64>().ok()).filter(|s| *s > 0.0).unwrap_or(1.0)
}

struct Builder<'a> {
    renderer: &'a Renderer,
    game: &'a GameConfig,
    fallback: f64,
    opaque: MeshBatch,
    double: MeshBatch,
    transparent: MeshBatch,
    /// Trigger and other gameplay volumes, hidden in the lit preview.
    volumes: MeshBatch,
    /// Set while building the brushes of a volume entity.
    volume: bool,
    face_overlay: MeshBatch,
    edges: Vec<LineVertex>,
    edges_2d: Vec<LineVertex>,
    sel_edges: Vec<LineVertex>,
    stats: SceneStats,
    instance_bounds: HashMap<NodeId, Aabb>,
    model_bounds: HashMap<NodeId, Aabb>,
}

fn push_line(list: &mut Vec<LineVertex>, a: DVec3, b: DVec3, color: [f32; 4]) {
    list.push(LineVertex { pos: v3(a), color });
    list.push(LineVertex { pos: v3(b), color });
}

/// Striped stand-in textures for tool materials a project does not provide (special/trigger, special/clip, ...).
pub fn tool_texture(name: &str) -> Option<image::RgbaImage> {
    let tool = name.strip_prefix("special/").or_else(|| name.strip_prefix("gt/"))?;
    let (base, stripe) = match tool {
        "trigger" => ([230u8, 140, 40], [150u8, 80, 20]),
        "clip" | "playerclip" => ([170, 90, 220], [110, 50, 150]),
        "skip" | "nodraw" => ([120, 120, 128], [80, 80, 88]),
        "origin" => ([60, 200, 220], [30, 120, 140]),
        "hint" => ([230, 220, 60], [150, 140, 30]),
        "occluder" => ([60, 60, 70], [30, 30, 36]),
        _ => ([200, 60, 200], [120, 30, 120]),
    };
    Some(image::RgbaImage::from_fn(32, 32, |x, y| {
        let c = if (x + y) % 16 < 5 { stripe } else { base };
        image::Rgba([c[0], c[1], c[2], 255])
    }))
}

/// Vertex color of a face corner. With a blend material the alpha carries the blend weight and defaults to the base.
fn face_vertex_color(tint: [f32; 4], painted: Option<[f32; 4]>, blend: bool) -> [f32; 4] {
    match (painted, blend) {
        (Some(vc), true) => [tint[0] * vc[0], tint[1] * vc[1], tint[2] * vc[2], vc[3]],
        (None, true) => [tint[0], tint[1], tint[2], 0.0],
        (Some(vc), false) => [tint[0] * vc[0], tint[1] * vc[1], tint[2] * vc[2], tint[3] * vc[3]],
        (None, false) => tint,
    }
}

fn mix_corners<const N: usize>(weights: [(usize, f64); 3], value: impl Fn(usize) -> [f32; N]) -> [f32; N] {
    let mut out = [0.0; N];
    for (k, w) in weights {
        for (o, v) in out.iter_mut().zip(value(k)) {
            *o += v * w as f32;
        }
    }

    out
}

/// Registers a model's textures, with alpha scissor for textures that have transparent pixels (leaves, grass cards).
fn register_model_textures(renderer: &mut Renderer, model: &crate::models::Model) {
    for (key, img, pixelated) in &model.textures {
        if renderer.has_material(key) {
            continue;
        }

        let mut desc = gt_render::MaterialDesc::plain(img, *pixelated);
        if img.pixels().any(|p| p.0[3] < 128) {
            desc.alpha = gt_render::AlphaMode::Scissor(0.5);
        }

        renderer.set_material_desc(key, &desc);
    }
}

const SCATTER_CHUNK: f64 = 1024.0;

/// GPU data of one scatter set, split into spatial chunks so painting only rebuilds the chunks it touched.
#[derive(Default)]
struct ScatterGpu {
    chunks: HashMap<(i64, i64), (u64, Option<GpuMesh>)>,
    lines_2d: Option<GpuLines>,
    sel_lines: Option<GpuLines>,
    triangles: usize,
}

fn instance_hash(set: &gt_doc::Scatter, instances: &[&gt_doc::scatter::ScatterInstance], selected: bool) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    selected.hash(&mut h);
    for i in instances {
        i.item.hash(&mut h);
        set.items.get(i.item as usize).map(|it| it.source.as_str()).unwrap_or("").hash(&mut h);
        set.item_material(i.item as usize).hash(&mut h);
        for v in [i.position.x, i.position.y, i.position.z, i.angles.x, i.angles.y, i.angles.z, i.scale] {
            v.to_bits().hash(&mut h);
        }
    }

    h.finish()
}

fn scatter_item_models(game: &GameConfig, models: &mut crate::models::ModelCache, set: &gt_doc::Scatter) -> Vec<Option<std::sync::Arc<crate::models::Model>>> {
    set.items
        .iter()
        .map(|item| {
            let path = if item.source.starts_with("res://") { game.resolve_res(&item.source) } else { Some(std::path::PathBuf::from(&item.source)) };
            path.filter(|p| crate::models::is_model_path(&p.to_string_lossy())).and_then(|p| models.get(&p, game.units_per_meter))
        })
        .collect()
}

fn build_scatter(
    renderer: &mut Renderer,
    game: &GameConfig,
    models: &mut crate::models::ModelCache,
    set: &gt_doc::Scatter,
    selected: bool,
    previous: Option<ScatterGpu>,
) -> ScatterGpu {
    let mut by_chunk: HashMap<(i64, i64), Vec<&gt_doc::scatter::ScatterInstance>> = HashMap::new();
    for inst in &set.instances {
        by_chunk.entry(((inst.position.x / SCATTER_CHUNK).floor() as i64, (inst.position.z / SCATTER_CHUNK).floor() as i64)).or_default().push(inst);
    }

    let item_models = scatter_item_models(game, models, set);
    for m in item_models.iter().flatten() {
        register_model_textures(renderer, m);
    }

    let mut old = previous.map(|p| p.chunks).unwrap_or_default();
    let mut out = ScatterGpu::default();
    let tint = if selected { SELECTED_TINT } else { [1.0; 4] };
    for (key, instances) in by_chunk {
        let hash = instance_hash(set, &instances, selected);
        if let Some((old_hash, mesh)) = old.remove(&key)
            && old_hash == hash
        {
            out.chunks.insert(key, (hash, mesh));
            continue;
        }

        let mut batch = MeshBatch::default();
        for inst in instances {
            let xform = inst.transform().as_mat4();
            match item_models.get(inst.item as usize).and_then(|m| m.as_ref()) {
                Some(model) => {
                    let normal_m = glam::Mat3::from_mat4(xform);
                    let override_material = set.item_material(inst.item as usize);
                    for part in &model.parts {
                        let verts: Vec<MeshVertex> = part
                            .vertices
                            .iter()
                            .map(|v| MeshVertex {
                                pos: xform.transform_point3(v.pos).to_array(),
                                normal: (normal_m * v.normal).normalize_or_zero().to_array(),
                                uv: v.uv,
                                color: tint,
                            })
                            .collect();
                        out.triangles += part.indices.len() / 3;
                        batch.add_triangles(override_material.unwrap_or(&part.material), &verts, &part.indices);
                    }
                }
                None => {
                    let s = (12.0 * inst.scale) as f32;
                    let p = glam::Vec3::from_array(v3(inst.position));
                    batch.add_box(p - glam::Vec3::new(s * 0.5, 0.0, s * 0.5), p + glam::Vec3::new(s * 0.5, s * 2.0, s * 0.5), [0.45, 0.85, 0.4, 1.0]);
                }
            }
        }

        out.chunks.insert(key, (hash, renderer.upload_mesh(&batch)));
    }

    let bounds = set.bounds();
    let color = if selected { SELECTED_COLOR } else { [0.45, 0.95, 0.5, 0.6] };
    let mut lines = Vec::new();
    if set.kind == gt_doc::ScatterKind::Props && set.instances.len() <= 20_000 {
        for inst in &set.instances {
            let r = set.items.get(inst.item as usize).map(|i| i.spacing * 0.25).unwrap_or(8.0).clamp(4.0, 64.0);
            push_line(&mut lines, inst.position - DVec3::X * r, inst.position + DVec3::X * r, color);
            push_line(&mut lines, inst.position - DVec3::Z * r, inst.position + DVec3::Z * r, color);
        }
    }

    let mut outline = Vec::new();
    if !bounds.is_empty() {
        let c = bounds.corners();
        for (i, j) in Aabb::EDGES {
            push_line(&mut outline, c[i], c[j], [color[0], color[1], color[2], 0.5]);
        }
    }

    lines.extend(outline.iter().copied());
    out.lines_2d = renderer.upload_lines(&lines);
    out.sel_lines = if selected { renderer.upload_lines(&outline) } else { None };
    out
}

impl Builder<'_> {
    /// Composite material key for a face with a blend material, when both textures are loaded.
    fn blend_material(&self, data: &gt_geom::FaceData) -> Option<String> {
        let blend = data.props.get(gt_doc::blend::BLEND_MATERIAL)?;
        let key = gt_render::blend_key(&data.material, blend);
        self.renderer.has_material(&key).then_some(key)
    }

    fn tex_size(&self, mat: &str) -> DVec2 {
        self.renderer.material_size(mat).map(|s| DVec2::new(s[0] as f64, s[1] as f64)).unwrap_or(DVec2::splat(self.fallback))
    }

    /// Batch for a surface: blended materials and see-through tool faces sort as transparent, cull disabled materials skip culling.
    fn batch(&mut self, mat: &str, see_through: bool) -> &mut MeshBatch {
        let flags = self.renderer.material_flags(mat);
        if self.volume {
            &mut self.volumes
        } else if see_through || flags.transparent {
            &mut self.transparent
        } else if flags.double_sided {
            &mut self.double
        } else {
            &mut self.opaque
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn brush(
        &mut self,
        brush: &Brush,
        tint: [f32; 4],
        selected: bool,
        selected_face: impl Fn(usize) -> bool,
        edge_2d: [f32; 4],
        see_through: bool,
        edge_3d: [f32; 4],
        culled: Option<&FacePieces>,
    ) {
        self.stats.brushes += 1;
        let has_disp = brush.faces.iter().any(|f| f.data.disp.is_some());
        for (fi, face) in brush.faces.iter().enumerate() {
            let mat = face.data.material.as_str();
            let size = self.tex_size(mat);
            let n = v3(face.plane.normal);
            // Like Hammer, only the displacement surfaces of a displacement brush are real geometry.
            let tool = see_through || self.game.is_tool_texture(mat) || (has_disp && face.data.disp.is_none());
            let mut color = tint;
            if tool {
                color[3] = if has_disp && face.data.disp.is_none() { 0.12 } else { 0.45 };
            }

            if let Some(grid) = gt_geom::displacement::grid(brush, fi) {
                let blend = self.blend_material(&face.data);
                let verts: Vec<MeshVertex> = (0..grid.size * grid.size)
                    .map(|k| {
                        let uv = face.data.uv.uv(grid.base[k], size);
                        let a = grid.alphas[k];
                        let c = if blend.is_some() {
                            [color[0], color[1], color[2], a]
                        } else {
                            // Without a blend material the weight is shown as a green shift.
                            [color[0] * (1.0 - 0.35 * a), color[1], color[2] * (1.0 - 0.35 * a), color[3]]
                        };
                        MeshVertex { pos: v3(grid.positions[k]), normal: v3(grid.normals[k]), uv: [uv.x as f32, uv.y as f32], color: c }
                    })
                    .collect();
                let indices: Vec<u32> = gt_geom::displacement::triangles(grid.size).flat_map(|(a, b, c)| [a as u32, b as u32, c as u32]).collect();
                self.stats.triangles += indices.len() / 3;
                let key = blend.as_deref().unwrap_or(mat);
                self.batch(key, false).add_triangles(key, &verts, &indices);
                let n_side = grid.size;
                for j in 0..n_side {
                    for i in 0..n_side {
                        let k = j * n_side + i;
                        let line = if selected { [1.0, 0.4, 0.3, 0.5] } else { [0.9, 0.9, 0.9, 0.12] };
                        let list = if selected { &mut self.sel_edges } else { &mut self.edges };
                        if i + 1 < n_side {
                            push_line(list, grid.positions[k], grid.positions[k + 1], line);
                        }

                        if j + 1 < n_side {
                            push_line(list, grid.positions[k], grid.positions[k + n_side], line);
                        }
                    }
                }

                if selected_face(fi) {
                    let overlay: Vec<MeshVertex> = verts.iter().map(|v| MeshVertex { color: [1.0, 0.2, 0.2, 0.3], ..*v }).collect();
                    self.face_overlay.add_triangles(gt_render::WHITE_MATERIAL, &overlay, &indices);
                }

                continue;
            }

            let painted = face.data.colors.len() == face.indices.len();
            let blend = self.blend_material(&face.data);
            let verts: Vec<MeshVertex> = face
                .indices
                .iter()
                .enumerate()
                .map(|(k, i)| {
                    let p = brush.vertices[*i as usize];
                    let uv = face.data.uv.uv(p, size);
                    let c = face_vertex_color(color, painted.then(|| face.data.colors[k]), blend.is_some());
                    MeshVertex { pos: v3(p), normal: n, uv: [uv.x as f32, uv.y as f32], color: c }
                })
                .collect();
            let key = blend.as_deref().unwrap_or(mat);
            match culled.and_then(|c| c.get(&fi)) {
                Some(pieces) => {
                    let corners: Vec<DVec3> = face.indices.iter().map(|i| brush.vertices[*i as usize]).collect();
                    let tris = gt_geom::polygon::fan(corners.len());
                    for piece in pieces {
                        let piece_verts: Vec<MeshVertex> = piece
                            .iter()
                            .map(|p| {
                                let uv = face.data.uv.uv(*p, size);
                                let vc = painted.then(|| mix_corners(gt_geom::polygon::corner_weights(&corners, &tris, *p), |k| face.data.colors[k]));
                                MeshVertex { pos: v3(*p), normal: n, uv: [uv.x as f32, uv.y as f32], color: face_vertex_color(color, vc, blend.is_some()) }
                            })
                            .collect();
                        self.stats.triangles += piece_verts.len().saturating_sub(2);
                        self.batch(key, tool).add_polygon(key, &piece_verts);
                    }
                }
                None => {
                    self.stats.triangles += verts.len().saturating_sub(2);
                    self.batch(key, tool).add_polygon(key, &verts);
                }
            }

            if selected_face(fi) {
                let overlay: Vec<MeshVertex> = verts.iter().map(|v| MeshVertex { color: [1.0, 0.2, 0.2, 0.35], ..*v }).collect();
                self.face_overlay.add_polygon(gt_render::WHITE_MATERIAL, &overlay);
                for k in 0..face.indices.len() {
                    let a = brush.vertices[face.indices[k] as usize];
                    let b = brush.vertices[face.indices[(k + 1) % face.indices.len()] as usize];
                    push_line(&mut self.sel_edges, a, b, SELECTED_COLOR);
                }
            }
        }

        for (a, b) in brush.edges() {
            let (pa, pb) = (brush.vertices[a as usize], brush.vertices[b as usize]);
            if selected {
                push_line(&mut self.sel_edges, pa, pb, SELECTED_COLOR);
            } else {
                push_line(&mut self.edges, pa, pb, edge_3d);
                push_line(&mut self.edges_2d, pa, pb, edge_2d);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn mesh(
        &mut self,
        mesh: &Mesh,
        tint: [f32; 4],
        selected: bool,
        selected_face: impl Fn(usize) -> bool,
        edge_2d: [f32; 4],
        see_through: bool,
        culled: Option<&FacePieces>,
    ) {
        self.stats.meshes += 1;
        let normals = mesh.corner_normals();
        for (fi, face) in mesh.faces.iter().enumerate() {
            if face.indices.len() < 3 || face.indices.iter().any(|i| *i as usize >= mesh.vertices.len()) {
                continue;
            }

            let mat = face.data.material.as_str();
            let size = self.tex_size(mat);
            let tool = see_through || self.game.is_tool_texture(mat);
            let mut color = tint;
            if tool {
                color[3] = 0.45;
            }

            let painted = face.data.colors.len() == face.indices.len();
            let blend = self.blend_material(&face.data);
            let verts: Vec<MeshVertex> = face
                .indices
                .iter()
                .enumerate()
                .map(|(k, i)| {
                    let uv = mesh.corner_uv(fi, k, size);
                    let c = face_vertex_color(color, painted.then(|| face.data.colors[k]), blend.is_some());
                    MeshVertex { pos: v3(mesh.vertices[*i as usize]), normal: v3(normals[fi][k]), uv: [uv.x as f32, uv.y as f32], color: c }
                })
                .collect();
            let corner_tris = mesh.triangulate_corners(fi);
            let tris: Vec<u32> = corner_tris.iter().flat_map(|[a, b, c]| [*a as u32, *b as u32, *c as u32]).collect();
            let decal_mat;
            let key = if mesh.decal {
                decal_mat = decal_key(mat);
                decal_mat.as_str()
            } else {
                blend.as_deref().unwrap_or(mat)
            };
            match culled.and_then(|c| c.get(&fi)) {
                Some(pieces) => {
                    let corners = mesh.face_points(fi);
                    for piece in pieces {
                        let piece_verts: Vec<MeshVertex> = piece
                            .iter()
                            .map(|p| {
                                let w = gt_geom::polygon::corner_weights(&corners, &corner_tris, *p);
                                let normal = mix_corners(w, |k| verts[k].normal);
                                let vc = painted.then(|| mix_corners(w, |k| face.data.colors[k]));
                                MeshVertex {
                                    pos: v3(*p),
                                    normal: Vec3::from_array(normal).normalize_or_zero().to_array(),
                                    uv: mix_corners(w, |k| verts[k].uv),
                                    color: face_vertex_color(color, vc, blend.is_some()),
                                }
                            })
                            .collect();
                        self.stats.triangles += piece_verts.len().saturating_sub(2);
                        self.batch(key, tool).add_polygon(key, &piece_verts);
                    }
                }
                None => {
                    self.stats.triangles += tris.len() / 3;
                    self.batch(key, tool).add_triangles(key, &verts, &tris);
                }
            }

            if selected_face(fi) {
                let overlay: Vec<MeshVertex> = verts.iter().map(|v| MeshVertex { color: [1.0, 0.2, 0.2, 0.35], ..*v }).collect();
                self.face_overlay.add_triangles(gt_render::WHITE_MATERIAL, &overlay, &tris);
            }
        }

        // Smooth shaded meshes only show feature edges, every face edge would turn cylinders into a wire cage.
        let face_normals: Vec<DVec3> = (0..mesh.faces.len()).map(|f| mesh.face_normal(f)).collect();
        let cos = if mesh.smooth_angle > 0.0 { (mesh.smooth_angle as f64).to_radians().cos() } else { 2.0 };
        for (key, faces) in mesh.edge_faces() {
            let feature = faces.len() != 2 || face_normals[faces[0]].dot(face_normals[faces[1]]) < cos;
            if !feature && !selected {
                continue;
            }

            let (pa, pb) = (mesh.vertices[key.0 as usize], mesh.vertices[key.1 as usize]);
            if selected {
                let c = if feature { SELECTED_COLOR } else { [1.0, 0.4, 0.3, 0.35] };
                push_line(&mut self.sel_edges, pa, pb, c);
            } else {
                push_line(&mut self.edges, pa, pb, EDGE_COLOR);
                push_line(&mut self.edges_2d, pa, pb, edge_2d);
            }
        }
    }

    fn point_entity(&mut self, id: Option<NodeId>, e: &Entity, selected: bool, tint: Option<[f32; 4]>, model: Option<&crate::models::Model>) {
        self.stats.entities += 1;
        let def = self.game.entity(&e.classname);
        let c = def_color(def);
        let line_color = if selected { SELECTED_COLOR } else { [c.r, c.g, c.b, 0.9] };
        let bounds = match model {
            Some(m) => {
                let xform = DMat4::from_scale_rotation_translation(DVec3::splat(entity_scale(e)), e.rotation(), e.origin);
                let fill = if selected { SELECTED_TINT } else { tint.unwrap_or([1.0; 4]) };
                let m32 = xform.as_mat4();
                let normal_m = glam::Mat3::from_mat4(m32);
                for part in &m.parts {
                    let verts: Vec<MeshVertex> = part
                        .vertices
                        .iter()
                        .map(|v| MeshVertex {
                            pos: m32.transform_point3(v.pos).to_array(),
                            normal: (normal_m * v.normal).normalize_or_zero().to_array(),
                            uv: v.uv,
                            color: fill,
                        })
                        .collect();
                    self.stats.triangles += part.indices.len() / 3;
                    self.batch(&part.material, false).add_triangles(&part.material, &verts, &part.indices);
                }

                let corners = m.bounds.corners();
                let b = Aabb::from_points(corners.iter().map(|p| xform.transform_point3(*p)));
                if let Some(id) = id {
                    self.model_bounds.insert(id, b);
                }

                b
            }
            None => {
                let bounds = entity_box(self.game, e);
                let fill = if selected { [1.0, 0.45, 0.4, 1.0] } else { tint.unwrap_or([c.r, c.g, c.b, 1.0]) };
                self.opaque.add_box(Vec3::from_array(v3(bounds.min)), Vec3::from_array(v3(bounds.max)), fill);
                bounds
            }
        };
        let corners = bounds.corners();
        for (a, b) in Aabb::EDGES {
            if selected {
                push_line(&mut self.sel_edges, corners[a], corners[b], line_color);
            } else {
                if model.is_none() {
                    push_line(&mut self.edges, corners[a], corners[b], line_color);
                }

                push_line(&mut self.edges_2d, corners[a], corners[b], [c.r, c.g, c.b, 1.0]);
            }
        }

        if is_decal(def) {
            self.decal_preview(e, selected);
        }

        // Models already show their orientation, arrows on hundreds of scattered props only add noise.
        if model.is_none() || selected {
            let fwd = e.rotation() * DVec3::NEG_Z;
            let start = bounds.center();
            let end = start + fwd * (bounds.size().max_element() * 0.75);
            push_line(&mut self.edges_2d, start, end, line_color);
            push_line(if selected { &mut self.sel_edges } else { &mut self.edges }, start, end, line_color);
        }
    }

    /// Textured quad on the decal's local XZ plane plus its projector box.
    fn decal_preview(&mut self, e: &Entity, selected: bool) {
        let size = decal_size(e);
        let rot = e.rotation();
        let (x, y, z) = (rot * DVec3::X * size.x * 0.5, rot * DVec3::Y, rot * DVec3::Z * size.z * 0.5);
        let center = e.origin + y * 0.5;
        let material = e.property("texture").filter(|t| !t.is_empty()).unwrap_or(gt_render::MISSING_MATERIAL);
        let corners = [center - x - z, center + x - z, center + x + z, center - x + z];
        let uvs = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let color = if selected { [1.0, 0.7, 0.7, 0.95] } else { [1.0, 1.0, 1.0, 0.9] };
        let verts: Vec<MeshVertex> = corners.iter().zip(uvs).map(|(p, uv)| MeshVertex { pos: v3(*p), normal: v3(y), uv, color }).collect();
        // Both windings so the preview is visible from either side.
        self.transparent.add_polygon(material, &verts);
        let back: Vec<MeshVertex> = verts.iter().rev().copied().collect();
        self.transparent.add_polygon(material, &back);
        let half_depth = y * size.y * 0.5;
        let box_corners: Vec<DVec3> = [
            (-1.0, -1.0, -1.0),
            (1.0, -1.0, -1.0),
            (1.0, -1.0, 1.0),
            (-1.0, -1.0, 1.0),
            (-1.0, 1.0, -1.0),
            (1.0, 1.0, -1.0),
            (1.0, 1.0, 1.0),
            (-1.0, 1.0, 1.0),
        ]
        .iter()
        .map(|(a, b, c)| e.origin + x * *a + half_depth * *b + z * *c)
        .collect();
        let edge_color = if selected { SELECTED_COLOR } else { [0.9, 0.4, 0.9, 0.6] };
        for (i, j) in [(0, 1), (1, 2), (2, 3), (3, 0), (4, 5), (5, 6), (6, 7), (7, 4), (0, 4), (1, 5), (2, 6), (3, 7)] {
            push_line(if selected { &mut self.sel_edges } else { &mut self.edges }, box_corners[i], box_corners[j], edge_color);
            push_line(&mut self.edges_2d, box_corners[i], box_corners[j], edge_color);
        }
    }

    /// Draws a prefab's contents transformed into place. Returns its world bounds.
    fn prefab(&mut self, cx: &mut BuildCx, path: &std::path::Path, xform: &DMat4, selected: bool, depth: usize) -> Aabb {
        if depth > prefabs::MAX_DEPTH {
            return Aabb::EMPTY;
        }

        let Some(map) = cx.prefabs.get(path).map.clone() else { return Aabb::EMPTY };
        let mut bounds = Aabb::EMPTY;
        let tint = if selected { SELECTED_TINT } else { INSTANCE_TINT };
        for (_, node) in prefabs::exported_nodes(&map) {
            match &node.kind {
                NodeKind::Brush(b) => {
                    let placed = b.transformed(xform, true);
                    bounds.include(&placed.bounds());
                    self.brush(&placed, tint, selected, |_| false, INSTANCE_EDGE, false, INSTANCE_EDGE, None);
                }
                NodeKind::Mesh(m) => {
                    let placed = m.transformed(xform, true);
                    bounds.include(&placed.bounds());
                    self.mesh(&placed, tint, selected, |_| false, INSTANCE_EDGE, false, None);
                }
                NodeKind::Entity(e) if node.children.is_empty() => {
                    let mut placed = e.clone();
                    placed.transform_by(xform);
                    let model = crate::models::entity_model_path(self.game, &placed).and_then(|p| cx.models.get(&p, self.game.units_per_meter));
                    bounds.include(&entity_box(self.game, &placed));
                    self.point_entity(None, &placed, selected, Some([0.6, 0.85, 1.0, 1.0]), model.as_deref());
                }
                NodeKind::Terrain(t) if t.is_valid() => {
                    // Like the Godot build, instances move terrains but never rotate them.
                    let placed = t.translated(xform.w_axis.truncate());
                    bounds.include(&placed.bounds());
                    self.prefab_terrain(&placed, tint);
                }
                NodeKind::Scatter(set) if !set.instances.is_empty() => {
                    let mut placed = set.clone();
                    placed.transform(xform);
                    bounds.include(&placed.bounds());
                    self.prefab_scatter(cx, &placed, tint);
                }
                NodeKind::Instance(inner) => {
                    if let Some(p) = prefabs::resolve(&inner.path, Some(path), cx.project_root) {
                        let m = *xform * prefabs::instance_transform(inner);
                        bounds.include(&self.prefab(cx, &p, &m, selected, depth + 1));
                    }
                }
                _ => {}
            }
        }

        bounds
    }

    /// A prefab's terrain as plain triangles, each cell in the material of its strongest layer.
    fn prefab_terrain(&mut self, t: &Terrain, tint: [f32; 4]) {
        self.stats.terrains += 1;
        let layer_count = t.layers.len().clamp(1, 4);
        let mut per_layer: Vec<(Vec<MeshVertex>, Vec<u32>)> = vec![(Vec::new(), Vec::new()); layer_count];
        let [cells_x, cells_z] = t.cells();
        for cj in 0..cells_z {
            for ci in 0..cells_x {
                if t.is_hole(ci, cj) {
                    continue;
                }

                let mut sum = [0.0f32; 4];
                for (i, j) in [(ci, cj), (ci + 1, cj), (ci, cj + 1), (ci + 1, cj + 1)] {
                    for (s, w) in sum.iter_mut().zip(t.weights(i, j)) {
                        *s += w;
                    }
                }

                let layer = (0..layer_count).max_by(|a, b| sum[*a].total_cmp(&sum[*b])).unwrap_or(0);
                let tile = t.layers.get(layer).map(|l| l.tile.max(1.0)).unwrap_or(256.0);
                let (verts, indices) = &mut per_layer[layer];
                for tri in Terrain::cell_triangles(ci, cj) {
                    for (i, j) in tri {
                        let p = t.vertex(i, j);
                        indices.push(verts.len() as u32);
                        verts.push(MeshVertex { pos: v3(p), normal: v3(t.normal(i, j)), uv: [(p.x / tile) as f32, (p.z / tile) as f32], color: tint });
                    }
                }
            }
        }

        for (layer, (verts, indices)) in per_layer.iter().enumerate() {
            if indices.is_empty() {
                continue;
            }

            let mat = t.layers.get(layer).map(|l| l.material.as_str()).unwrap_or(gt_render::MISSING_MATERIAL);
            self.stats.triangles += indices.len() / 3;
            self.batch(mat, false).add_triangles(mat, verts, indices);
        }

        let corners = t.bounds().corners();
        for (i, j) in Aabb::EDGES {
            push_line(&mut self.edges_2d, corners[i], corners[j], INSTANCE_EDGE);
        }
    }

    /// A prefab's scatter set: instance models in place, or small boxes where a model is missing.
    fn prefab_scatter(&mut self, cx: &mut BuildCx, set: &gt_doc::Scatter, tint: [f32; 4]) {
        let item_models = scatter_item_models(self.game, cx.models, set);
        for inst in &set.instances {
            let xform = inst.transform().as_mat4();
            match item_models.get(inst.item as usize).and_then(|m| m.as_ref()) {
                Some(model) => {
                    let normal_m = glam::Mat3::from_mat4(xform);
                    let material = set.item_material(inst.item as usize);
                    for part in &model.parts {
                        let verts: Vec<MeshVertex> = part
                            .vertices
                            .iter()
                            .map(|v| MeshVertex {
                                pos: xform.transform_point3(v.pos).to_array(),
                                normal: (normal_m * v.normal).normalize_or_zero().to_array(),
                                uv: v.uv,
                                color: tint,
                            })
                            .collect();
                        let key = material.unwrap_or(&part.material);
                        self.stats.triangles += part.indices.len() / 3;
                        self.batch(key, false).add_triangles(key, &verts, &part.indices);
                    }
                }
                None => {
                    let s = (12.0 * inst.scale) as f32;
                    let p = Vec3::from_array(v3(inst.position));
                    self.opaque.add_box(p - Vec3::new(s * 0.5, 0.0, s * 0.5), p + Vec3::new(s * 0.5, s * 2.0, s * 0.5), [0.45, 0.85, 0.4, 1.0]);
                }
            }
        }
    }
}

struct BuildCx<'a> {
    prefabs: &'a mut PrefabCache,
    models: &'a mut crate::models::ModelCache,
    project_root: Option<&'a std::path::Path>,
}

fn entity_center(map: &Map, game: &GameConfig, id: NodeId) -> Option<DVec3> {
    let node = map.get(id)?;
    let e = node.entity()?;
    Some(if node.children.is_empty() { entity_box(game, e).center() } else { map.bounds(id).center() })
}

fn terrain_wire(t: &Terrain, color: [f32; 4]) -> Vec<LineVertex> {
    let [rx, rz] = t.resolution;
    let stride = ((rx.max(rz) as usize) / 128).max(1) as u32;
    let mut lines = Vec::new();
    for j in (0..rz).step_by(stride as usize) {
        for i in (0..rx.saturating_sub(stride)).step_by(stride as usize) {
            push_line(&mut lines, t.vertex(i, j), t.vertex(i + stride, j), color);
        }
    }

    for i in (0..rx).step_by(stride as usize) {
        for j in (0..rz.saturating_sub(stride)).step_by(stride as usize) {
            push_line(&mut lines, t.vertex(i, j), t.vertex(i, j + stride), color);
        }
    }

    lines
}

fn build_terrain(
    renderer: &mut Renderer,
    t: &Terrain,
    selected: bool,
    chunks_to_build: Option<&BTreeSet<usize>>,
    previous: Option<TerrainGpu>,
    wireframe: bool,
) -> TerrainGpu {
    let layers: Vec<(String, f32, f32, f32)> = t
        .layers
        .iter()
        .map(|l| (l.material.clone(), l.tile.max(1.0) as f32, l.detile.clamp(0.0, 1.0) as f32, l.detile_sharpen.clamp(0.0, 1.0) as f32))
        .collect();
    let key = renderer.prepare_terrain_material(&layers);
    let chunk_list = t.chunks();
    let mut out = previous.filter(|p| p.chunks.len() == chunk_list.len()).unwrap_or(TerrainGpu {
        chunks: Vec::new(),
        lines_2d: None,
        sel_lines: None,
        wire: None,
        triangles: 0,
    });
    out.wire = if wireframe { renderer.upload_lines(&terrain_wire(t, if selected { SELECTED_COLOR } else { TERRAIN_EDGE })) } else { None };
    out.chunks.resize_with(chunk_list.len(), || None);
    let flag = if selected { 1.0 } else { 0.0 };
    for (index, (ci, cj, w, h)) in chunk_list.iter().copied().enumerate() {
        if chunks_to_build.is_some_and(|set| !set.contains(&index)) {
            continue;
        }

        let mut verts = Vec::with_capacity(((w + 1) * (h + 1)) as usize);
        for j in cj..=cj + h {
            for i in ci..=ci + w {
                let weights = t.weights(i, j);
                verts.push(MeshVertex { pos: v3(t.vertex(i, j)), normal: v3(t.normal(i, j)), uv: [flag, 0.0], color: weights });
            }
        }

        let row = w + 1;
        let mut indices = Vec::with_capacity((w * h * 6) as usize);
        for y in cj..cj + h {
            for x in ci..ci + w {
                if t.is_hole(x, y) {
                    continue;
                }

                for tri in Terrain::cell_triangles(x, y) {
                    indices.extend(tri.iter().map(|(i, j)| (j - cj) * row + (i - ci)));
                }
            }
        }

        let mut batch = MeshBatch::default();
        batch.add_triangles(&key, &verts, &indices);
        out.chunks[index] = renderer.upload_mesh(&batch);
    }

    out.triangles = (t.cells()[0] * t.cells()[1] * 2) as usize;
    let b = t.bounds();
    let mut lines = Vec::new();
    let color = if selected { SELECTED_COLOR } else { TERRAIN_EDGE };
    let corners = b.corners();
    for (i, j) in Aabb::EDGES {
        push_line(&mut lines, corners[i], corners[j], color);
    }

    let size = t.size();
    let step = t.chunk_cells.max(1) as f64 * t.cell_size;
    let top = b.max.y - t.origin.y;
    let mut x = 0.0;
    while x <= size.x + 1e-6 {
        push_line(&mut lines, t.origin + DVec3::new(x, top, 0.0), t.origin + DVec3::new(x, top, size.y), [color[0], color[1], color[2], 0.35]);
        x += step;
    }

    let mut z = 0.0;
    while z <= size.y + 1e-6 {
        push_line(&mut lines, t.origin + DVec3::new(0.0, top, z), t.origin + DVec3::new(size.x, top, z), [color[0], color[1], color[2], 0.35]);
        z += step;
    }

    out.lines_2d = renderer.upload_lines(&lines);
    out.sel_lines = if selected { renderer.upload_lines(&lines[..24]) } else { None };
    out
}

/// Chunks whose heights, weights or holes differ between two versions of a terrain.
fn changed_chunks(old: &Terrain, new: &Terrain) -> Option<BTreeSet<usize>> {
    if old.resolution != new.resolution
        || old.cell_size != new.cell_size
        || old.origin != new.origin
        || old.layers != new.layers
        || old.chunk_cells != new.chunk_cells
    {
        return None;
    }

    let mut out = BTreeSet::new();
    for (index, (ci, cj, w, h)) in new.chunks().into_iter().enumerate() {
        // Normals look one vertex further, so the comparison window is one larger on each side.
        let i0 = ci.saturating_sub(1);
        let j0 = cj.saturating_sub(1);
        let i1 = (ci + w + 1).min(new.resolution[0] - 1);
        let j1 = (cj + h + 1).min(new.resolution[1] - 1);
        let differs = (j0..=j1).any(|j| {
            let a = new.index(i0, j);
            let b = new.index(i1, j);
            old.heights[a..=b] != new.heights[a..=b] || (!new.splat.is_empty() && old.splat.get(a * 4..=b * 4 + 3) != new.splat.get(a * 4..=b * 4 + 3))
        }) || (old.holes != new.holes);
        if differs {
            out.insert(index);
        }
    }

    Some(out)
}

pub fn compute_lighting(map: &Map, game: &GameConfig) -> Lighting {
    let mut lighting = Lighting::default();
    let upm = game.units_per_meter as f32;
    let props = &map.properties;
    if let Some(angles) =
        props.get("sun_angles").map(|s| s.split_whitespace().filter_map(|p| p.parse::<f64>().ok()).collect::<Vec<_>>()).filter(|v| v.len() >= 2)
    {
        let q = gt_core::DQuat::from_euler(gt_core::EulerRot::YXZ, angles[1].to_radians(), angles[0].to_radians(), 0.0);
        lighting.sun_direction = (q * DVec3::NEG_Z).as_vec3();
    }

    if let Some(c) = props.get("sun_color").and_then(|s| parse_color(s)) {
        lighting.sun_color = c;
    }

    if let Some(e) = props.get("sun_energy").and_then(|s| s.parse().ok()) {
        lighting.sun_energy = e;
    }

    if let Some(c) = props.get("ambient_color").and_then(|s| parse_color(s)) {
        lighting.ambient = c;
    }

    for (key, slot) in [
        ("sky_top_color", &mut lighting.sky_top),
        ("sky_horizon_color", &mut lighting.sky_horizon),
        ("sky_ground_color", &mut lighting.sky_ground),
        ("fog_color", &mut lighting.fog_color),
    ] {
        if let Some(c) = props.get(key).and_then(|s| parse_color(s)) {
            *slot = linear(c);
        }
    }

    let energy = |key: &str| props.get(key).and_then(|s| s.parse::<f32>().ok()).map(|e| e.max(0.0));
    if let Some(e) = energy("ambient_energy") {
        lighting.ambient *= e;
    }

    if let Some(e) = energy("sky_energy") {
        lighting.sky_top *= e;
        lighting.sky_horizon *= e;
        lighting.sky_ground *= e;
    }

    // Godot fog density is per meter.
    if let Some(d) = props.get("fog_density").and_then(|s| s.parse::<f32>().ok()) {
        lighting.fog_density = d.max(0.0) / upm.max(1.0);
    }

    for (id, e) in map.entities() {
        if map.is_hidden(id) || !map.is_point_entity(id) {
            continue;
        }

        let node_class = game.entity(&e.classname).map(|d| d.node_class.as_str()).unwrap_or("");
        let is_light =
            matches!(node_class, "OmniLight3D" | "SpotLight3D" | "DirectionalLight3D") || (node_class.is_empty() && e.classname.starts_with("light"));
        if !is_light || e.property("start_on") == Some("0") {
            continue;
        }

        let color = e.property("light_color").and_then(parse_color).unwrap_or(Vec3::ONE);
        let energy = e.property("light_energy").and_then(|s| s.parse().ok()).unwrap_or(1.0f32);
        let forward = (e.rotation() * DVec3::NEG_Z).as_vec3();
        if node_class == "DirectionalLight3D" || e.classname.contains("directional") || e.classname == "light_environment" {
            lighting.sun_direction = forward;
            lighting.sun_color = color;
            lighting.sun_energy = energy;
            continue;
        }

        let spot = node_class == "SpotLight3D" || e.classname.contains("spot");
        let range_m = e.property(if spot { "spot_range" } else { "omni_range" }).and_then(|s| s.parse().ok()).unwrap_or(10.0f32);
        let cone = e.property("spot_angle").and_then(|s| s.parse::<f32>().ok()).unwrap_or(45.0);
        lighting.lights.push(PointLight {
            position: e.origin.as_vec3(),
            range: range_m * upm,
            color,
            energy: energy * 1.6,
            spot: spot.then(|| (forward, cone.to_radians().cos())),
        });
    }

    lighting.lights.sort_by(|a, b| (b.energy * b.range).total_cmp(&(a.energy * a.range)));
    lighting.lights.truncate(gt_render::MAX_LIGHTS);
    lighting
}

/// Read-only context shared by the per-bucket builders, so buckets can build across threads.
struct BucketCtx<'a> {
    map: &'a Map,
    game: &'a GameConfig,
    renderer: &'a Renderer,
    selected_brush_like: &'a BTreeSet<NodeId>,
    selection: &'a Selection,
    selected_faces: &'a BTreeSet<(NodeId, usize)>,
    pieces: &'a HashMap<NodeId, FacePieces>,
    entity_models: &'a HashMap<NodeId, std::sync::Arc<crate::models::Model>>,
    drag_nodes: &'a BTreeSet<NodeId>,
}

/// Builds the batches for one bucket's brush, mesh, entity and terrain nodes. Prefab instances are left to
/// the caller because they need the mutable prefab and model caches.
fn build_bucket<'a>(ctx: &BucketCtx<'a>, ids: &[NodeId]) -> Builder<'a> {
    let map = ctx.map;
    let mut builder = Builder {
        renderer: ctx.renderer,
        game: ctx.game,
        fallback: ctx.game.textures.fallback_size as f64,
        opaque: MeshBatch::default(),
        double: MeshBatch::default(),
        transparent: MeshBatch::default(),
        volumes: MeshBatch::default(),
        volume: false,
        face_overlay: MeshBatch::default(),
        edges: Vec::new(),
        edges_2d: Vec::new(),
        sel_edges: Vec::new(),
        stats: SceneStats::default(),
        instance_bounds: HashMap::new(),
        model_bounds: HashMap::new(),
    };
    let is_selected = |id: NodeId| ctx.selection.nodes.contains(&id) || map.ancestors(id).iter().any(|a| ctx.selection.nodes.contains(a));
    for &id in ids {
        let Some(node) = map.get(id) else { continue };
        if ctx.drag_nodes.contains(&id) {
            continue;
        }

        if map.is_hidden(id) || (!matches!(node.kind, NodeKind::Layer(_) | NodeKind::Group(_)) && !map.in_cordon(id)) {
            continue;
        }

        let entity = map.owning_entity(id).and_then(|e| map.entity(e));
        let entity_def = entity.and_then(|e| ctx.game.entity(&e.classname));
        let selected = ctx.selected_brush_like.contains(&id);
        let tint = if selected {
            SELECTED_TINT
        } else if map.is_locked(id) {
            LOCKED_TINT
        } else if let Some(def) = entity_def {
            let c = def.color;
            [0.75 + 0.25 * c.r, 0.75 + 0.25 * c.g, 0.75 + 0.25 * c.b, 1.0]
        } else {
            [1.0; 4]
        };
        let edge_2d = entity_def.map(|d| [d.color.r, d.color.g, d.color.b, 0.9]).unwrap_or(EDGE_COLOR_2D);
        let is_trigger = entity.is_some_and(|e| e.classname.starts_with("trigger")) || entity_def.is_some_and(|d| d.node_class == "Area3D");
        builder.volume = is_trigger;
        match &node.kind {
            NodeKind::Brush(brush) => {
                builder.brush(brush, tint, selected, |fi| ctx.selected_faces.contains(&(id, fi)), edge_2d, is_trigger, EDGE_COLOR, ctx.pieces.get(&id))
            }
            NodeKind::Mesh(mesh) => builder.mesh(mesh, tint, selected, |fi| ctx.selected_faces.contains(&(id, fi)), edge_2d, is_trigger, ctx.pieces.get(&id)),
            NodeKind::Terrain(_) => builder.stats.terrains += 1,
            NodeKind::Entity(e) if node.children.is_empty() => {
                builder.point_entity(Some(id), e, is_selected(id), None, ctx.entity_models.get(&id).map(|m| m.as_ref()));
            }
            _ => {}
        }
    }

    builder
}

fn upload_bucket(renderer: &Renderer, builder: Builder) -> Bucket {
    let xray: Vec<LineVertex> = builder.sel_edges.iter().map(|v| LineVertex { color: SELECTED_XRAY, ..*v }).collect();
    Bucket {
        opaque: renderer.upload_mesh(&builder.opaque),
        double: renderer.upload_mesh(&builder.double),
        transparent: renderer.upload_mesh(&builder.transparent),
        volumes: renderer.upload_mesh(&builder.volumes),
        overlay: renderer.upload_mesh(&builder.face_overlay),
        edges: renderer.upload_lines(&builder.edges),
        edges_2d: renderer.upload_lines(&builder.edges_2d),
        sel_edges: renderer.upload_lines(&builder.sel_edges),
        xray: renderer.upload_lines(&xray),
        stats: builder.stats,
        instance_bounds: builder.instance_bounds,
        model_bounds: builder.model_bounds,
    }
}

impl SceneCache {
    pub fn invalidate(&mut self) {
        self.prev_map = None;
        self.revision = 0;
    }

    fn bucket_of(id: NodeId) -> usize {
        (id.0 % BUCKETS) as usize
    }

    pub fn update(&mut self, renderer: &mut Renderer, state: &mut EditorState, project_generation: u64) {
        let prefab_generation = state.prefabs.generation;
        let model_generation = state.models.generation;
        let lit = state.prefs.shade == crate::state::Shade::Lit;
        let wireframe = state.prefs.shade == crate::state::Shade::Wireframe;
        if self.revision == state.doc.revision
            && self.project_generation == project_generation
            && self.prefab_generation == prefab_generation
            && self.model_generation == model_generation
            && self.lit == lit
            && self.wireframe == wireframe
            && self.prev_map.is_some()
        {
            return;
        }

        if self.project_generation != project_generation {
            renderer.clear_materials();
        }

        if self.model_reloads != state.models.reloads {
            // A changed model file keeps the same "model:{path}#N" texture keys, so without this a
            // reload picks up the new geometry but leaves the old texture bound under that key.
            renderer.clear_materials_with_prefix("model:");
            self.model_reloads = state.models.reloads;
        }

        let map = state.doc.map.clone();
        let selection = state.doc.selection.clone();
        // Only heavy moves use the drag layer; ordinary brush drags keep the live rebuild-and-cull path.
        let drag_req =
            state.drag_preview.clone().filter(|d| d.nodes.iter().filter_map(|id| map.get(*id)).map(node_face_count).sum::<usize>() > CULL_DEFER_FACES);
        let drag_base = drag_req.as_ref().and_then(|_| state.doc.transaction_base().cloned());
        let drag_nodes: BTreeSet<NodeId> = drag_req.as_ref().map(|d| d.nodes.clone()).unwrap_or_default();
        let full = match &self.prev_map {
            None => true,
            Some(prev) => {
                self.project_generation != project_generation
                    || self.prefab_generation != prefab_generation
                    || self.model_generation != model_generation
                    || self.wireframe != wireframe
                    || prev.editor.cordon != map.editor.cordon
                    || prev.editor.cordon_enabled != map.editor.cordon_enabled
            }
        };
        self.wireframe = wireframe;
        let lit_changed = self.lit != lit;
        self.revision = state.doc.revision;
        self.project_generation = project_generation;
        self.lit = lit;
        if self.buckets.len() != BUCKETS as usize {
            self.buckets = (0..BUCKETS).map(|_| Bucket::default()).collect();
        }

        let mut dirty: BTreeSet<NodeId> = BTreeSet::new();
        if full {
            dirty.extend(map.nodes.keys().copied());
            if let Some(prev) = &self.prev_map {
                dirty.extend(prev.nodes.keys().copied());
            }

            if self.prev_map.is_none() {
                self.terrains.clear();
                self.scatters.clear();
                for b in &mut self.buckets {
                    *b = Bucket::default();
                }
            }
        } else if let Some(prev) = &self.prev_map {
            for item in prev.nodes.diff(&map.nodes) {
                let id = match item {
                    imbl::ordmap::DiffItem::Add(k, _) | imbl::ordmap::DiffItem::Remove(k, _) => *k,
                    imbl::ordmap::DiffItem::Update { new: (k, _), .. } => *k,
                };
                dirty.insert(id);
                dirty.extend(map.descendants(id));
            }

            let sel_changed: BTreeSet<NodeId> = self.prev_selection.nodes.symmetric_difference(&selection.nodes).copied().collect();
            for id in sel_changed {
                dirty.insert(id);
                dirty.extend(map.descendants(id));
            }

            let faces_changed: BTreeSet<NodeId> = self.prev_selection.faces.symmetric_difference(&selection.faces).map(|(id, _)| *id).collect();
            dirty.extend(faces_changed);
        }

        // Nodes entering or leaving a live drag need their buckets rebuilt, to drop them into the drag layer
        // or fold them back in.
        let prev_drag_nodes: BTreeSet<NodeId> = self.drag.as_ref().map(|d| d.nodes.clone()).unwrap_or_default();
        if drag_nodes != prev_drag_nodes {
            dirty.extend(drag_nodes.iter().copied());
            dirty.extend(prev_drag_nodes.iter().copied());
        }

        let entities_changed = full || dirty.iter().any(|id| map.entity(*id).is_some() || self.prev_map.as_ref().is_some_and(|p| p.entity(*id).is_some()));

        // Textures needed by the changed nodes and the prefabs they place.
        let mut needed: BTreeSet<String> = BTreeSet::new();
        let mut pending_prefabs: Vec<(std::path::PathBuf, usize)> = Vec::new();
        let mut blend_pairs: BTreeSet<(String, String)> = BTreeSet::new();
        for id in &dirty {
            let faces: Vec<&gt_geom::FaceData> = match map.get(*id).map(|n| &n.kind) {
                Some(NodeKind::Brush(b)) => b.faces.iter().map(|f| &f.data).collect(),
                Some(NodeKind::Mesh(m)) => m.faces.iter().map(|f| &f.data).collect(),
                _ => Vec::new(),
            };
            for data in faces {
                if let Some(blend) = data.props.get(gt_doc::blend::BLEND_MATERIAL) {
                    blend_pairs.insert((data.material.clone(), blend.clone()));
                }
            }
        }

        needed.extend(blend_pairs.iter().map(|(_, b)| b.clone()));
        let collect = |node: &gt_doc::Node, needed: &mut BTreeSet<String>| match &node.kind {
            NodeKind::Brush(b) => needed.extend(b.faces.iter().map(|f| f.data.material.clone())),
            NodeKind::Mesh(m) => {
                needed.extend(m.faces.iter().map(|f| f.data.material.clone()));
                if m.decal {
                    needed.extend(m.faces.iter().map(|f| decal_key(&f.data.material)));
                }
            }
            NodeKind::Terrain(t) => needed.extend(t.layers.iter().map(|l| l.material.clone())),
            NodeKind::Entity(e) => needed.extend(e.property("texture").filter(|t| t.starts_with("res://")).map(str::to_string)),
            _ => {}
        };
        for id in &dirty {
            let Some(node) = map.get(*id) else { continue };
            collect(node, &mut needed);
            if let NodeKind::Instance(i) = &node.kind
                && let Some(p) = prefabs::resolve(&i.path, state.doc.path.as_deref(), state.game.project_root.as_deref())
            {
                pending_prefabs.push((p, 0));
            }
        }

        let mut visited = BTreeSet::new();
        let mut prefab_models: Vec<std::sync::Arc<crate::models::Model>> = Vec::new();
        state.prefabs.project_root = state.game.project_root.clone();
        while let Some((path, depth)) = pending_prefabs.pop() {
            if depth > prefabs::MAX_DEPTH || !visited.insert(path.clone()) {
                continue;
            }

            if let Some(prefab) = state.prefabs.get(&path).map.clone() {
                for (_, n) in prefabs::exported_nodes(&prefab) {
                    collect(n, &mut needed);
                    match &n.kind {
                        NodeKind::Instance(i) => {
                            if let Some(p) = prefabs::resolve(&i.path, Some(&path), state.game.project_root.as_deref()) {
                                pending_prefabs.push((p, depth + 1));
                            }
                        }
                        NodeKind::Scatter(set) => prefab_models.extend(scatter_item_models(&state.game, &mut state.models, set).into_iter().flatten()),
                        NodeKind::Entity(e) => {
                            if let Some(p) = crate::models::entity_model_path(&state.game, e)
                                && let Some(m) = state.models.get(&p, state.game.units_per_meter)
                            {
                                prefab_models.push(m);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        for m in &prefab_models {
            register_model_textures(renderer, m);
        }

        for name in needed {
            if name.is_empty() || renderer.has_material(&name) {
                continue;
            }

            if let Some(base) = name.strip_prefix(DECAL_PREFIX) {
                // A decal variant of a material: same textures, but alpha cut out and double sided.
                if let Some(loaded) = state.materials.load_material(base) {
                    let mut desc = material_desc(&loaded, state.prefs.texture_filter);
                    desc.alpha = gt_render::AlphaMode::Scissor(0.5);
                    desc.double_sided = true;
                    renderer.set_material_desc(&name, &desc);
                }

                continue;
            }

            if let Some(loaded) = state.materials.load_material(&name) {
                state.materials.remember_size(&name, [loaded.albedo.width(), loaded.albedo.height()]);
                renderer.set_material_desc(&name, &material_desc(&loaded, state.prefs.texture_filter));
            } else if let Some(img) = tool_texture(&name) {
                renderer.set_material_desc(&name, &gt_render::MaterialDesc::plain(&img, true));
            }
        }

        for (base, blend) in &blend_pairs {
            if renderer.has_material(base) && renderer.has_material(blend) {
                renderer.prepare_blend_material(base, blend);
            }
        }

        // Models of changed point entities.
        let game = state.game.clone();
        for id in &dirty {
            if let Some(e) = map.entity(*id)
                && let Some(path) = crate::models::entity_model_path(&game, e)
                && let Some(model) = state.models.get(&path, game.units_per_meter)
            {
                register_model_textures(renderer, &model);
            }
        }

        let opaque = |m: &str| {
            let flags = renderer.material_flags(m);
            !flags.transparent && !flags.double_sided
        };
        // While a heavy selection is being dragged, re-culling it every frame dominates the frame time and
        // its result cannot be seen until the drag ends, so defer it: draw the moving nodes un-culled now
        // and let the commit (which ends the transaction) run the real cull once.
        let heavy_drag =
            !full && state.doc.in_transaction() && dirty.iter().filter_map(|id| map.get(*id)).map(node_face_count).sum::<usize>() > CULL_DEFER_FACES;
        let recull = if heavy_drag { self.face_cull.clear_pieces(&dirty) } else { self.face_cull.update(&map, &game, &opaque, &dirty, full) };
        dirty.extend(recull);

        let selected_brush_like: BTreeSet<NodeId> = selection.geometry(&map).into_iter().collect();
        let is_selected = |id: NodeId| selection.nodes.contains(&id) || map.ancestors(id).iter().any(|a| selection.nodes.contains(a));

        // Terrains keep their own chunked GPU meshes.
        for id in dirty.iter().copied() {
            match map.terrain(id) {
                Some(t) if !map.is_hidden(id) && map.in_cordon(id) && t.is_valid() => {
                    let selected = selected_brush_like.contains(&id);
                    let previous = self.terrains.remove(&id);
                    let prev_terrain = self.prev_map.as_ref().and_then(|p| p.terrain(id));
                    let prev_selected = self.prev_selection.nodes.contains(&id)
                        || self.prev_map.as_ref().is_some_and(|p| p.ancestors(id).iter().any(|a| self.prev_selection.nodes.contains(a)));
                    let chunks = match (prev_terrain, &previous, full || selected != prev_selected) {
                        (Some(old), Some(_), false) => changed_chunks(old, t),
                        _ => None,
                    };
                    let gpu = build_terrain(renderer, t, selected, chunks.as_ref(), previous, wireframe);
                    self.terrains.insert(id, gpu);
                }
                _ => {
                    self.terrains.remove(&id);
                }
            }
        }

        for id in dirty.iter().copied() {
            match map.scatter(id) {
                Some(set) if !map.is_hidden(id) && map.in_cordon(id) => {
                    let selected = is_selected(id);
                    let previous = self.scatters.remove(&id);
                    let gpu = build_scatter(renderer, &game, &mut state.models, set, selected, previous);
                    self.scatters.insert(id, gpu);
                }
                _ => {
                    self.scatters.remove(&id);
                }
            }
        }

        let dirty_buckets: BTreeSet<usize> = dirty.iter().map(|id| Self::bucket_of(*id)).collect();
        if !dirty_buckets.is_empty() {
            let mut per_bucket: BTreeMap<usize, Vec<NodeId>> = BTreeMap::new();
            for id in map.nodes.keys() {
                let b = Self::bucket_of(*id);
                if dirty_buckets.contains(&b) {
                    per_bucket.entry(b).or_default().push(*id);
                }
            }

            let selected_faces = selection.faces.clone();

            // Point-entity models load on demand from the main-thread cache, so resolve them here and hand
            // the builders a read-only snapshot rather than the mutable cache.
            let mut entity_models: HashMap<NodeId, std::sync::Arc<crate::models::Model>> = HashMap::new();
            for ids in per_bucket.values() {
                for &id in ids {
                    if let Some(node) = map.get(id)
                        && node.children.is_empty()
                        && let Some(e) = node.entity()
                        && let Some(p) = crate::models::entity_model_path(&game, e)
                        && let Some(m) = state.models.get(&p, game.units_per_meter)
                    {
                        entity_models.insert(id, m);
                    }
                }
            }

            let per_bucket_ref = &per_bucket;
            let results: Vec<(usize, Bucket)> = {
                let ctx = BucketCtx {
                    map: &map,
                    game: &game,
                    renderer,
                    selected_brush_like: &selected_brush_like,
                    selection: &selection,
                    selected_faces: &selected_faces,
                    pieces: &self.face_cull.pieces,
                    entity_models: &entity_models,
                    drag_nodes: &drag_nodes,
                };
                let has_instance = |b: usize| {
                    per_bucket_ref.get(&b).is_some_and(|ids| ids.iter().any(|id| matches!(map.get(*id).map(|n| &n.kind), Some(NodeKind::Instance(_)))))
                };
                let parallel: Vec<usize> = dirty_buckets.iter().copied().filter(|b| !has_instance(*b)).collect();
                let serial: Vec<usize> = dirty_buckets.iter().copied().filter(|b| has_instance(*b)).collect();

                // Buckets without prefab instances build across threads; each face's tessellation is independent.
                let ctx_ref = &ctx;
                let threads = std::thread::available_parallelism().map(|t| t.get()).unwrap_or(1);
                let mut results: Vec<(usize, Bucket)> = if threads <= 1 || parallel.len() <= 1 {
                    parallel
                        .iter()
                        .map(|&b| (b, upload_bucket(ctx.renderer, build_bucket(&ctx, per_bucket_ref.get(&b).map(|v| v.as_slice()).unwrap_or(&[])))))
                        .collect()
                } else {
                    let chunk = parallel.len().div_ceil(threads);
                    std::thread::scope(|s| {
                        parallel
                            .chunks(chunk)
                            .map(|c| {
                                s.spawn(move || {
                                    c.iter()
                                        .map(|&b| {
                                            let ids = per_bucket_ref.get(&b).map(|v| v.as_slice()).unwrap_or(&[]);
                                            (b, upload_bucket(ctx_ref.renderer, build_bucket(ctx_ref, ids)))
                                        })
                                        .collect::<Vec<_>>()
                                })
                            })
                            .collect::<Vec<_>>()
                            .into_iter()
                            .flat_map(|h| h.join().unwrap())
                            .collect()
                    })
                };

                // Prefab instances need the mutable caches, so their buckets build on this thread.
                if !serial.is_empty() {
                    let mut cx = BuildCx { prefabs: &mut state.prefabs, models: &mut state.models, project_root: game.project_root.as_deref() };
                    for b in serial {
                        let ids = per_bucket_ref.get(&b).map(|v| v.as_slice()).unwrap_or(&[]);
                        let mut builder = build_bucket(&ctx, ids);
                        for &id in ids {
                            let Some(NodeKind::Instance(inst)) = map.get(id).map(|n| &n.kind) else { continue };
                            if map.is_hidden(id) || !map.in_cordon(id) {
                                continue;
                            }

                            let selected = ctx.selection.nodes.contains(&id) || map.ancestors(id).iter().any(|a| ctx.selection.nodes.contains(a));
                            let path = prefabs::resolve(&inst.path, state.doc.path.as_deref(), game.project_root.as_deref());
                            let bounds = match &path {
                                Some(p) => builder.prefab(&mut cx, p, &prefabs::instance_transform(inst), selected, 0),
                                None => Aabb::EMPTY,
                            };
                            let bounds = if bounds.is_empty() { Aabb::from_center_size(inst.origin, DVec3::splat(16.0)) } else { bounds };
                            let corners = bounds.corners();
                            let color = if selected { SELECTED_COLOR } else { INSTANCE_EDGE };
                            for (i, j) in Aabb::EDGES {
                                push_line(if selected { &mut builder.sel_edges } else { &mut builder.edges_2d }, corners[i], corners[j], color);
                            }

                            builder.instance_bounds.insert(id, bounds);
                        }

                        results.push((b, upload_bucket(ctx.renderer, builder)));
                    }
                }

                results
            };
            for (b, bucket) in results {
                self.buckets[b] = bucket;
            }
        }

        // Live drag layer: rebuild its base batches when the dragged set changes, then re-upload a
        // translated copy for the current offset. No re-tessellation while the move runs.
        match (&drag_req, &drag_base) {
            (Some(req), Some(base)) => {
                if self.drag.as_ref().map(|d| d.nodes != req.nodes).unwrap_or(true) {
                    self.drag = Some(build_drag_batches(renderer, &game, base, &req.nodes));
                }

                if let Some(dl) = &mut self.drag {
                    let off = v3(req.offset);
                    dl.gpu_opaque = renderer.upload_mesh(&dl.opaque.translated(off));
                    dl.gpu_double = renderer.upload_mesh(&dl.double.translated(off));
                    dl.gpu_transparent = renderer.upload_mesh(&dl.transparent.translated(off));
                    dl.gpu_edges = renderer.upload_lines(&translate_lines(&dl.edges, req.offset));
                }
            }
            _ => self.drag = None,
        }

        // Target and I/O link lines between entities.
        let mut names: BTreeMap<&str, Vec<NodeId>> = BTreeMap::new();
        for (id, e) in map.entities() {
            if let Some(n) = e.targetname() {
                names.entry(n).or_default().push(id);
            }
        }

        let mut links = Vec::new();
        for (id, e) in map.entities() {
            if map.is_hidden(id) {
                continue;
            }

            let Some(from) = entity_center(&map, &game, id) else { continue };
            let emphasis = if is_selected(id) { 1.0 } else { 0.5 };
            let mut link = |target: &str, color: [f32; 4]| {
                let matches: Vec<NodeId> = if let Some(prefix) = target.strip_suffix('*') {
                    names.iter().filter(|(n, _)| n.starts_with(prefix)).flat_map(|(_, ids)| ids.iter().copied()).collect()
                } else {
                    names.get(target).cloned().unwrap_or_default()
                };
                for t in matches {
                    if let Some(to) = entity_center(&map, &game, t) {
                        push_line(&mut links, from, to, [color[0], color[1], color[2], color[3] * emphasis * 1.6]);
                    }
                }
            };
            if let Some(t) = e.property("target").filter(|t| !t.is_empty()) {
                link(t, TARGET_LINK);
            }

            for o in &e.outputs {
                link(&o.target, IO_LINK);
            }
        }

        self.links = renderer.upload_lines(&links);
        self.cordon = match map.editor.cordon {
            Some(c) => {
                let color = if map.editor.cordon_enabled { [1.0, 0.85, 0.2, 0.9] } else { [1.0, 0.85, 0.2, 0.3] };
                let corners = c.corners();
                let mut lines = Vec::new();
                for (i, j) in Aabb::EDGES {
                    push_line(&mut lines, corners[i], corners[j], color);
                }

                renderer.upload_lines(&lines)
            }
            None => None,
        };

        let mut stats = SceneStats::default();
        let mut instance_bounds = HashMap::new();
        let mut model_bounds = HashMap::new();
        for b in &self.buckets {
            stats += b.stats;
            instance_bounds.extend(b.instance_bounds.iter().map(|(k, v)| (*k, *v)));
            model_bounds.extend(b.model_bounds.iter().map(|(k, v)| (*k, *v)));
        }

        for t in self.terrains.values() {
            stats.triangles += t.triangles;
        }

        for s in self.scatters.values() {
            stats.triangles += s.triangles;
        }

        self.stats = stats;
        state.instance_bounds = instance_bounds;
        state.model_bounds = model_bounds;

        let properties_changed = self.prev_map.as_ref().is_none_or(|p| p.properties != map.properties);
        if entities_changed || full || lit_changed || properties_changed {
            self.lighting = compute_lighting(&map, &game);
        }

        if lit && (!dirty.is_empty() || lit_changed || full || properties_changed) {
            self.shadow_dirty = true;
        }

        self.scene_bounds = map.bounds_of(map.layers.iter().copied());

        self.prev_map = Some(map);
        self.prev_selection = selection;
        // Prefabs and models loaded during this rebuild bump their generations, record them afterwards to avoid a rebuild loop.
        self.prefab_generation = state.prefabs.generation;
        self.model_generation = state.models.generation;
    }

    /// Renders the sun shadow map around `focus` when the scene changed or the camera moved away from the last fit.
    pub fn update_shadows(&mut self, renderer: &mut Renderer, focus: DVec3, lit: bool) {
        const HALF: f64 = 3200.0;
        if !lit {
            self.shadow_center = None;
            return;
        }

        let moved = self.shadow_center.is_none_or(|c| (c - focus).length() > HALF * 0.35);
        if !self.shadow_dirty && !moved {
            return;
        }

        let bounds = self.scene_bounds;
        let vp = if bounds.is_empty() {
            None
        } else {
            let center = focus.clamp(bounds.min, bounds.max);
            let half = DVec3::new(HALF, bounds.size().y * 0.5 + 64.0, HALF).min(bounds.size() * 0.5 + DVec3::splat(64.0));
            let lo = DVec3::new(center.x - half.x, bounds.min.y - 64.0, center.z - half.z);
            let hi = DVec3::new(center.x + half.x, bounds.max.y + 64.0, center.z + half.z);
            let vp = Renderer::sun_view_proj(self.lighting.sun_direction, lo.as_vec3(), hi.as_vec3());
            let mut meshes: Vec<&GpuMesh> = self.buckets.iter().flat_map(|b| [b.opaque.as_ref(), b.double.as_ref()]).flatten().collect();
            meshes.extend(self.terrains.values().flat_map(|t| t.chunks.iter().flatten()));
            meshes.extend(self.scatters.values().flat_map(|s| s.chunks.values().filter_map(|(_, m)| m.as_ref())));
            renderer.render_shadow(&meshes, vp);
            Some(vp)
        };
        renderer.set_lighting(&self.lighting, vp);
        self.shadow_center = Some(focus);
        self.shadow_dirty = false;
    }

    /// Adds the scene to a frame for a 3D or 2D view.
    /// Unselected edges are left out of lit views, which preview the game look.
    pub fn fill_frame<'a>(&'a self, frame: &mut Frame<'a>, is_2d: bool, lit: bool) {
        for t in self.terrains.values() {
            if is_2d {
                frame.overlay_lines.extend(t.lines_2d.as_ref());
            } else {
                frame.terrain.extend(t.chunks.iter().flatten());
                frame.overlay_lines.extend(t.sel_lines.as_ref());
                frame.wire_lines.extend(t.wire.as_ref());
            }
        }

        for s in self.scatters.values() {
            if is_2d {
                frame.overlay_lines.extend(s.lines_2d.as_ref());
            } else {
                frame.opaque.extend(s.chunks.values().filter_map(|(_, m)| m.as_ref()));
                frame.overlay_lines.extend(s.sel_lines.as_ref());
            }
        }

        for b in &self.buckets {
            if is_2d {
                frame.overlay_lines.extend(b.edges_2d.as_ref());
                frame.overlay_lines.extend(b.sel_edges.as_ref());
            } else {
                frame.wire_lines.extend(b.edges_2d.as_ref());
                frame.opaque.extend(b.opaque.as_ref());
                frame.double_sided.extend(b.double.as_ref());
                if !lit {
                    frame.lines.extend(b.edges.as_ref());
                }

                frame.lines.extend(b.sel_edges.as_ref());
                frame.transparent.extend(b.transparent.as_ref());
                if !lit {
                    frame.transparent.extend(b.volumes.as_ref());
                }

                frame.overlay_meshes.extend(b.overlay.as_ref());
                frame.overlay_lines.extend(b.xray.as_ref());
            }
        }

        if let Some(dl) = &self.drag {
            if is_2d {
                frame.overlay_lines.extend(dl.gpu_edges.as_ref());
            } else {
                frame.opaque.extend(dl.gpu_opaque.as_ref());
                frame.double_sided.extend(dl.gpu_double.as_ref());
                frame.transparent.extend(dl.gpu_transparent.as_ref());
                frame.lines.extend(dl.gpu_edges.as_ref());
            }
        }

        frame.overlay_lines.extend(self.links.as_ref());
        frame.lines.extend(self.cordon.as_ref());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn night_worldspawn_keys_dim_the_lit_preview() {
        let game = GameConfig::builtin();
        let mut map = Map::new();
        let day = compute_lighting(&map, &game);
        for (k, v) in [("ambient_color", "40 48 72"), ("ambient_energy", "0.5"), ("sky_energy", "0.25"), ("sun_energy", "0.1")] {
            map.properties.insert(k.into(), v.into());
        }

        let layer = map.default_layer();
        let mut lamp = gt_doc::entity::Entity::new("light");
        lamp.origin = DVec3::new(0.0, 96.0, 0.0);
        lamp.properties.insert("omni_range".into(), "8".into());
        map.insert(layer, NodeKind::Entity(lamp));
        let mut off = gt_doc::entity::Entity::new("light");
        off.properties.insert("start_on".into(), "0".into());
        map.insert(layer, NodeKind::Entity(off));

        let night = compute_lighting(&map, &game);
        assert!((night.ambient - parse_color("40 48 72").unwrap() * 0.5).length() < 1e-5, "ambient scaled by ambient_energy");
        assert!((night.sky_top - day.sky_top * 0.25).length() < 1e-5, "sky scaled by sky_energy");
        assert_eq!(night.sun_energy, 0.1);
        assert_eq!(night.lights.len(), 1, "lights that start off are left out of the preview");
        assert_eq!(night.lights[0].range, 8.0 * game.units_per_meter as f32);
    }
}
