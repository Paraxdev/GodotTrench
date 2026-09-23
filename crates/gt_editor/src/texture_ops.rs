//! Texture alignment operations shared by the texture tool, the face inspector and the MCP server.
//! Faces are brush faces or mesh faces. Mesh faces with explicit UVs are edited in normalized UV space.

use gt_core::{DVec2, DVec3, NodeId, Plane};
use gt_doc::Map;
use gt_geom::{FaceUv, Justify, UvProjection};

use crate::state::EditorState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshUvKind {
    /// Along the dominant axis of the faces' average normal, all faces as one island.
    Planar,
    /// Along a world axis (0 X, 1 Y, 2 Z), all faces as one island.
    PlanarAxis(usize),
    Box,
    /// Around a world axis (0 X, 1 Y, 2 Z) through the selection center.
    Cylinder(usize),
    Sphere,
    View,
    Unfold,
    /// Each face in its own plane, like a brush face after a texture reset.
    World,
    /// The planar projection stored on each face written as explicit UVs, so corners can be edited.
    Bake,
    /// Drops explicit UVs, back to the planar projection stored on the face.
    Clear,
    Normalize,
    /// Packs the UV islands into 0..1 with a uniform texel density, stacking identical islands.
    Pack,
}

impl MeshUvKind {
    pub const ALL: [MeshUvKind; 16] = [
        MeshUvKind::Planar,
        MeshUvKind::PlanarAxis(0),
        MeshUvKind::PlanarAxis(1),
        MeshUvKind::PlanarAxis(2),
        MeshUvKind::Box,
        MeshUvKind::Cylinder(0),
        MeshUvKind::Cylinder(1),
        MeshUvKind::Cylinder(2),
        MeshUvKind::Sphere,
        MeshUvKind::View,
        MeshUvKind::Unfold,
        MeshUvKind::World,
        MeshUvKind::Bake,
        MeshUvKind::Clear,
        MeshUvKind::Normalize,
        MeshUvKind::Pack,
    ];
    /// The everyday layouts, for places with little room such as the face inspector.
    pub const COMMON: [MeshUvKind; 8] = [
        MeshUvKind::Planar,
        MeshUvKind::Box,
        MeshUvKind::Cylinder(1),
        MeshUvKind::Sphere,
        MeshUvKind::Unfold,
        MeshUvKind::World,
        MeshUvKind::Normalize,
        MeshUvKind::Pack,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            MeshUvKind::Planar => "Planar",
            MeshUvKind::PlanarAxis(0) => "Planar X",
            MeshUvKind::PlanarAxis(1) => "Planar Y",
            MeshUvKind::PlanarAxis(_) => "Planar Z",
            MeshUvKind::Box => "Box",
            MeshUvKind::Cylinder(0) => "Cylinder X",
            MeshUvKind::Cylinder(1) => "Cylinder Y",
            MeshUvKind::Cylinder(_) => "Cylinder Z",
            MeshUvKind::Sphere => "Sphere",
            MeshUvKind::View => "View",
            MeshUvKind::Unfold => "Unfold",
            MeshUvKind::World => "Reset to World",
            MeshUvKind::Bake => "Bake Planar",
            MeshUvKind::Clear => "Clear UVs",
            MeshUvKind::Normalize => "Normalize",
            MeshUvKind::Pack => "Pack",
        }
    }

    pub fn hint(&self) -> &'static str {
        match self {
            MeshUvKind::Planar => "One projection along the main axis of the faces' average normal, the faces stay one piece",
            MeshUvKind::PlanarAxis(_) => "One projection straight along this world axis, the faces stay one piece",
            MeshUvKind::Box => "Each face along its closest world axis, like a cube map",
            MeshUvKind::Cylinder(_) => "Wraps around this world axis through the center of the faces, u runs around it",
            MeshUvKind::Sphere => "Longitude and latitude around the center of the faces",
            MeshUvKind::View => "Straight along the 3D view camera",
            MeshUvKind::Unfold => "Lays the faces out flat edge by edge, so strips and grids stay continuous",
            MeshUvKind::World => "Each face in its own plane at the texel density of a reset brush face",
            MeshUvKind::Bake => "Writes the current planar look as explicit UVs, so the corners can be edited here",
            MeshUvKind::Clear => "Drops the explicit UVs, back to the planar projection stored on each face",
            MeshUvKind::Normalize => "Scales the UVs to fit the texture once, keeping their aspect",
            MeshUvKind::Pack => "Packs the UV islands into the texture at one texel density, stacking identical islands",
        }
    }

    /// Accepts labels and snake case names case insensitively ("planar_x", "cylinder_y", "reset_to_world"). A bare
    /// "cylinder" wraps around Y.
    pub fn from_name(name: &str) -> Option<Self> {
        let key = name.trim().to_ascii_lowercase().replace(['_', '-'], " ");
        match key.as_str() {
            "cylinder" => return Some(MeshUvKind::Cylinder(1)),
            "world" | "reset world" => return Some(MeshUvKind::World),
            "bake" => return Some(MeshUvKind::Bake),
            "clear" => return Some(MeshUvKind::Clear),
            _ => {}
        }

        Self::ALL.into_iter().find(|k| k.label().eq_ignore_ascii_case(&key))
    }
}

fn axis_vec(i: usize) -> DVec3 {
    [DVec3::X, DVec3::Y, DVec3::Z][i.min(2)]
}

/// Runs a mesh UV layout on faces. `view` is the camera's (right, up) for the view projection.
pub fn mesh_uv(state: &mut EditorState, faces: &[(NodeId, usize)], kind: MeshUvKind, view: (DVec3, DVec3)) -> usize {
    let projection = match kind {
        MeshUvKind::Clear => {
            let targets: Vec<FaceInfo> = infos(state, faces).into_iter().filter(|i| i.explicit.is_some()).collect();
            let n = targets.len();
            state.doc.edit("Clear UVs", |m, _| {
                for t in &targets {
                    set_uv(m, t.id, t.face, t.uv.clone());
                }
            });
            return n;
        }
        MeshUvKind::Bake => return bake_mesh_uvs(state, faces),
        MeshUvKind::Normalize => return normalize_mesh_uvs(state, faces, true),
        MeshUvKind::Pack => return pack_mesh_uvs(state, faces, true),
        MeshUvKind::Planar => None,
        MeshUvKind::PlanarAxis(i) => Some(UvProjection::Planar { normal: axis_vec(i) }),
        MeshUvKind::Box => Some(UvProjection::Box),
        MeshUvKind::World => Some(UvProjection::World),
        MeshUvKind::Cylinder(i) => Some(UvProjection::Cylinder { axis: axis_vec(i) }),
        MeshUvKind::Sphere => Some(UvProjection::Sphere),
        MeshUvKind::View => Some(UvProjection::View { right: view.0, up: view.1 }),
        MeshUvKind::Unfold => Some(UvProjection::Unfold),
    };
    match projection {
        Some(p) => project_mesh(state, faces, p, None),
        None => planar_best_axis(state, faces),
    }
}

/// Planar projection per mesh along the main axis of the area weighted normal of its listed faces.
fn planar_best_axis(state: &mut EditorState, faces: &[(NodeId, usize)]) -> usize {
    let jobs: Vec<(NodeId, Vec<usize>, DVec2, DVec3)> = mesh_jobs(state, faces, None)
        .into_iter()
        .filter_map(|(id, list, repeat)| {
            let mesh = state.doc.map.mesh(id)?;
            let sum: DVec3 = list.iter().map(|f| mesh.face_normal(*f) * mesh.face_area(*f).max(1e-9)).sum();
            let axis = gt_core::major_axis(sum);
            let mut normal = DVec3::ZERO;
            normal[axis] = if sum[axis] < 0.0 { -1.0 } else { 1.0 };
            Some((id, list, repeat, normal))
        })
        .collect();
    let n = jobs.iter().map(|j| j.1.len()).sum();
    if n > 0 {
        state.doc.edit("Planar UV Projection", |m, _| {
            for (id, list, repeat, normal) in &jobs {
                if let Some(mesh) = m.mesh_mut(*id) {
                    mesh.project_uvs(list, UvProjection::Planar { normal: *normal }, *repeat);
                }
            }
        });
    }

    n
}

/// Writes the planar projection of mesh faces as explicit UVs, each face at its own texture size.
pub fn bake_mesh_uvs(state: &mut EditorState, faces: &[(NodeId, usize)]) -> usize {
    let jobs: Vec<(NodeId, usize, DVec2)> =
        infos(state, faces).into_iter().filter(|i| state.doc.map.mesh(i.id).is_some()).map(|i| (i.id, i.face, tex_size(state, &i.material))).collect();
    let n = jobs.len();
    if n > 0 {
        state.doc.edit("Bake Planar UVs", |m, _| {
            for (id, f, tex) in &jobs {
                if let Some(mesh) = m.mesh_mut(*id) {
                    mesh.bake_uvs(&[*f], *tex);
                }
            }
        });
    }

    n
}

/// Material and alignment copied with the eyedropper or "Copy alignment".
#[derive(Clone, Debug)]
pub struct UvClipboard {
    pub material: String,
    pub uv: FaceUv,
    pub plane: Plane,
}

#[derive(Clone, Debug)]
pub struct FaceInfo {
    pub id: NodeId,
    pub face: usize,
    pub points: Vec<DVec3>,
    pub plane: Plane,
    pub material: String,
    pub uv: FaceUv,
    /// Explicit normalized UVs of a mesh face, one per corner.
    pub explicit: Option<Vec<[f32; 2]>>,
}

impl FaceInfo {
    pub fn center(&self) -> DVec3 {
        self.points.iter().copied().sum::<DVec3>() / self.points.len().max(1) as f64
    }
}

pub fn face_info(map: &Map, id: NodeId, face: usize) -> Option<FaceInfo> {
    if let Some(b) = map.brush(id) {
        let f = b.faces.get(face)?;
        return Some(FaceInfo {
            id,
            face,
            points: b.face_points(face),
            plane: f.plane,
            material: f.data.material.clone(),
            uv: f.data.uv.clone(),
            explicit: None,
        });
    }

    let m = map.mesh(id)?;
    let f = m.faces.get(face)?;
    Some(FaceInfo {
        id,
        face,
        points: m.face_points(face),
        plane: m.face_plane(face),
        material: f.data.material.clone(),
        uv: f.data.uv.clone(),
        explicit: (f.uvs.len() == f.indices.len()).then(|| f.uvs.clone()),
    })
}

pub fn tex_size(state: &EditorState, material: &str) -> DVec2 {
    state.materials.size(material).map(DVec2::from_array).unwrap_or(DVec2::splat(state.game.textures.fallback_size as f64))
}

/// Selected faces, or every face of the selected brushes and meshes when no faces are selected.
pub fn target_faces(state: &EditorState) -> Vec<(NodeId, usize)> {
    let sel = &state.doc.selection;
    if sel.has_faces() {
        return sel.faces.iter().copied().collect();
    }

    let map = &state.doc.map;
    sel.geometry(map)
        .into_iter()
        .flat_map(|id| {
            let n = map.brush(id).map(|b| b.faces.len()).or_else(|| map.mesh(id).map(|m| m.faces.len())).unwrap_or(0);
            (0..n).map(move |f| (id, f))
        })
        .collect()
}

/// Writes a planar projection, dropping explicit UVs of mesh faces.
fn set_uv(map: &mut Map, id: NodeId, face: usize, uv: FaceUv) {
    if let Some(f) = map.brush_mut(id).and_then(|b| b.faces.get_mut(face)) {
        f.data.uv = uv;
    } else if let Some(f) = map.mesh_mut(id).and_then(|m| m.faces.get_mut(face)) {
        f.data.uv = uv;
        f.uvs.clear();
    }
}

fn set_material(map: &mut Map, id: NodeId, face: usize, material: &str) {
    if let Some(f) = map.brush_mut(id).and_then(|b| b.faces.get_mut(face)) {
        f.data.material = material.to_string();
    } else if let Some(f) = map.mesh_mut(id).and_then(|m| m.faces.get_mut(face)) {
        f.data.material = material.to_string();
    }
}

fn set_explicit(map: &mut Map, id: NodeId, face: usize, uvs: Vec<[f32; 2]>) {
    if let Some(f) = map.mesh_mut(id).and_then(|m| m.faces.get_mut(face))
        && uvs.len() == f.indices.len()
    {
        f.uvs = uvs;
    }
}

fn infos(state: &EditorState, faces: &[(NodeId, usize)]) -> Vec<FaceInfo> {
    faces.iter().filter_map(|(id, f)| face_info(&state.doc.map, *id, *f)).collect()
}

enum Plan {
    Planar(FaceUv),
    Explicit(Vec<[f32; 2]>),
}

fn commit(state: &mut EditorState, label: &str, coalesce: bool, plans: Vec<(NodeId, usize, Plan)>) -> usize {
    let n = plans.len();
    if n == 0 {
        return 0;
    }

    let apply = move |m: &mut Map| {
        for (id, f, plan) in plans {
            match plan {
                Plan::Planar(uv) => set_uv(m, id, f, uv),
                Plan::Explicit(uvs) => set_explicit(m, id, f, uvs),
            }
        }
    };
    if coalesce {
        state.doc.edit_coalesced(label, |m, _| apply(m));
    } else {
        state.doc.edit(label, |m, _| apply(m));
    }

    n
}

fn centroid(uvs: &[[f32; 2]]) -> [f32; 2] {
    let n = uvs.len().max(1) as f32;
    let s = uvs.iter().fold([0.0f32; 2], |a, u| [a[0] + u[0], a[1] + u[1]]);
    [s[0] / n, s[1] / n]
}

/// Justifies faces. With `treat_as_one` all faces are aligned against their combined extent.
pub fn justify(state: &mut EditorState, faces: &[(NodeId, usize)], mode: Justify, treat_as_one: bool) -> usize {
    let list = infos(state, faces);
    state.find_new_materials(list.iter().map(|i| i.material.as_str()));
    let all_points: Vec<DVec3> = list.iter().flat_map(|i| i.points.clone()).collect();
    let mut plans = Vec::new();
    for info in &list {
        let tex = tex_size(state, &info.material);
        if let Some(uvs) = &info.explicit {
            let (lo, hi) =
                uvs.iter().fold(([f32::MAX; 2], [f32::MIN; 2]), |(lo, hi), u| ([lo[0].min(u[0]), lo[1].min(u[1])], [hi[0].max(u[0]), hi[1].max(u[1])]));
            let size = [(hi[0] - lo[0]).max(1e-6), (hi[1] - lo[1]).max(1e-6)];
            let map = |u: &[f32; 2]| -> [f32; 2] {
                match mode {
                    Justify::Left => [u[0] - lo[0], u[1]],
                    Justify::Top => [u[0], u[1] - lo[1]],
                    Justify::Right => [u[0] + (1.0 - hi[0]), u[1]],
                    Justify::Bottom => [u[0], u[1] + (1.0 - hi[1])],
                    Justify::Center => [u[0] + 0.5 - (lo[0] + hi[0]) * 0.5, u[1] + 0.5 - (lo[1] + hi[1]) * 0.5],
                    Justify::Fit => [(u[0] - lo[0]) / size[0], (u[1] - lo[1]) / size[1]],
                    Justify::FitWidth => [(u[0] - lo[0]) / size[0], (u[1] - lo[1]) / size[0]],
                    Justify::FitHeight => [(u[0] - lo[0]) / size[1], (u[1] - lo[1]) / size[1]],
                }
            };
            plans.push((info.id, info.face, Plan::Explicit(uvs.iter().map(map).collect())));
            continue;
        }

        let mut uv = info.uv.clone();
        uv.justify(if treat_as_one { &all_points } else { &info.points }, tex, mode);
        plans.push((info.id, info.face, Plan::Planar(uv)));
    }

    commit(state, &format!("Justify {}", mode.label()), false, plans)
}

/// Projects faces straight along the camera, keeping their scale.
pub fn align_to_view(state: &mut EditorState, faces: &[(NodeId, usize)], right: DVec3, up: DVec3) -> usize {
    let plans = infos(state, faces)
        .into_iter()
        .map(|i| {
            let uv = FaceUv::from_view(right, up, i.uv.scale.abs());
            (i.id, i.face, Plan::Planar(if uv.is_degenerate_for(i.plane.normal) { FaceUv::face_aligned(i.plane.normal, i.uv.scale) } else { uv }))
        })
        .collect();
    commit(state, "Align to View", false, plans)
}

/// Resets faces to a face aligned projection with the given scale.
pub fn reset(state: &mut EditorState, faces: &[(NodeId, usize)], scale: DVec2) -> usize {
    let plans = infos(state, faces).into_iter().map(|i| (i.id, i.face, Plan::Planar(FaceUv::face_aligned(i.plane.normal, scale)))).collect();
    commit(state, "Reset Texture", false, plans)
}

/// Adds `texels` (texture pixels) to the texture coordinates of faces. Consecutive calls merge into one undo step.
pub fn shift(state: &mut EditorState, faces: &[(NodeId, usize)], texels: DVec2) -> usize {
    let plans = infos(state, faces)
        .into_iter()
        .map(|i| match &i.explicit {
            Some(uvs) => {
                let tex = tex_size(state, &i.material);
                let d = [(texels.x / tex.x) as f32, (texels.y / tex.y) as f32];
                (i.id, i.face, Plan::Explicit(uvs.iter().map(|u| [u[0] + d[0], u[1] + d[1]]).collect()))
            }
            None => {
                let mut uv = i.uv.clone();
                uv.offset += texels;
                (i.id, i.face, Plan::Planar(uv))
            }
        })
        .collect();
    commit(state, "Shift Texture", true, plans)
}

/// Multiplies the texture scale (world units per texel), keeping the texel at each face center in place.
pub fn scale(state: &mut EditorState, faces: &[(NodeId, usize)], factor: DVec2) -> usize {
    let plans = infos(state, faces)
        .into_iter()
        .map(|i| match &i.explicit {
            Some(uvs) => {
                let c = centroid(uvs);
                let f = [factor.x.max(1e-6) as f32, factor.y.max(1e-6) as f32];
                (i.id, i.face, Plan::Explicit(uvs.iter().map(|u| [c[0] + (u[0] - c[0]) / f[0], c[1] + (u[1] - c[1]) / f[1]]).collect()))
            }
            None => {
                let center = i.center();
                let mut uv = i.uv.clone();
                let before = uv.texel(center);
                uv.scale *= factor;
                uv.offset += before - uv.texel(center);
                (i.id, i.face, Plan::Planar(uv))
            }
        })
        .collect();
    commit(state, "Scale Texture", true, plans)
}

/// Rotates textures around each face center.
pub fn rotate(state: &mut EditorState, faces: &[(NodeId, usize)], degrees: f64) -> usize {
    let plans = infos(state, faces)
        .into_iter()
        .map(|i| match &i.explicit {
            Some(uvs) => {
                let c = centroid(uvs);
                let (s, co) = (degrees as f32).to_radians().sin_cos();
                (
                    i.id,
                    i.face,
                    Plan::Explicit(
                        uvs.iter().map(|u| [c[0] + (u[0] - c[0]) * co - (u[1] - c[1]) * s, c[1] + (u[0] - c[0]) * s + (u[1] - c[1]) * co]).collect(),
                    ),
                )
            }
            None => {
                let center = i.center();
                let mut uv = i.uv.clone();
                let before = uv.texel(center);
                // FaceUv rotates around the texture normal; flip for faces whose projection looks at them from behind.
                let sign = if uv.texture_normal().dot(i.plane.normal) < 0.0 { -1.0 } else { 1.0 };
                uv.rotate(degrees * sign);
                uv.offset += before - uv.texel(center);
                (i.id, i.face, Plan::Planar(uv))
            }
        })
        .collect();
    commit(state, "Rotate Texture", true, plans)
}

/// Sets the texel density: world units covered by one texture pixel.
pub fn set_density(state: &mut EditorState, faces: &[(NodeId, usize)], units_per_texel: f64) -> usize {
    let plans = infos(state, faces)
        .into_iter()
        .filter(|i| i.explicit.is_none())
        .map(|i| {
            let center = i.center();
            let mut uv = i.uv.clone();
            let before = uv.texel(center);
            uv.scale = DVec2::new(units_per_texel * uv.scale.x.signum(), units_per_texel * uv.scale.y.signum());
            uv.offset += before - uv.texel(center);
            (i.id, i.face, Plan::Planar(uv))
        })
        .collect();
    commit(state, "Texel Density", false, plans)
}

/// Applies `material` to faces, continuing the alignment of `source` across edges (Hammer's apply with wrap).
pub fn wrap_from(state: &mut EditorState, source: (NodeId, usize), faces: &[(NodeId, usize)], material: Option<&str>) -> usize {
    let Some(src) = face_info(&state.doc.map, source.0, source.1) else { return 0 };
    let material = material.map(str::to_string).unwrap_or(src.material.clone());
    let targets: Vec<FaceInfo> = infos(state, faces).into_iter().filter(|i| (i.id, i.face) != source).collect();
    let n = targets.len();
    state.doc.edit("Wrap Texture", |m, _| {
        for t in &targets {
            set_uv(m, t.id, t.face, FaceUv::wrapped(&src.uv, &src.plane, &t.plane));
            set_material(m, t.id, t.face, &material);
        }
    });
    n
}

/// Picks a face's material and alignment: sets the current material and fills the clipboard.
pub fn eyedropper(state: &mut EditorState, face: (NodeId, usize)) -> bool {
    let Some(info) = face_info(&state.doc.map, face.0, face.1) else { return false };
    state.current_material = info.material.clone();
    state.note_material(&info.material);
    state.set_status(format!("Picked {}", info.material));
    state.uv_clipboard = Some(UvClipboard { material: info.material, uv: info.uv, plane: info.plane });
    true
}

/// Pastes the clipboard alignment (wrapped onto each face's plane) and, when `with_material`, its material.
pub fn paste_alignment(state: &mut EditorState, faces: &[(NodeId, usize)], with_material: bool) -> usize {
    let Some(clip) = state.uv_clipboard.clone() else { return 0 };
    let targets = infos(state, faces);
    let n = targets.len();
    state.doc.edit("Paste Alignment", |m, _| {
        for t in &targets {
            let uv = FaceUv::wrapped(&clip.uv, &clip.plane, &t.plane);
            set_uv(m, t.id, t.face, if uv.is_degenerate_for(t.plane.normal) { FaceUv::face_aligned(t.plane.normal, clip.uv.scale) } else { uv });
            if with_material {
                set_material(m, t.id, t.face, &clip.material);
            }
        }
    });
    n
}

/// Applies a material to faces, resetting their projection to face aligned at the current scale when `reset`.
pub fn apply_material(state: &mut EditorState, faces: &[(NodeId, usize)], material: &str, reset_alignment: bool) -> usize {
    let targets = infos(state, faces);
    let n = targets.len();
    state.note_material(material);
    state.doc.edit("Apply Material", |m, _| {
        for t in &targets {
            set_material(m, t.id, t.face, material);
            if reset_alignment && t.explicit.is_none() {
                set_uv(m, t.id, t.face, FaceUv::face_aligned(t.plane.normal, t.uv.scale.abs()));
            }
        }
    });
    n
}

/// Mesh faces grouped per mesh with the world size of one texture repeat, from the first face's texture and scale
/// unless `repeat` is given.
fn mesh_jobs(state: &EditorState, faces: &[(NodeId, usize)], repeat: Option<DVec2>) -> Vec<(NodeId, Vec<usize>, DVec2)> {
    let mut per_mesh: std::collections::BTreeMap<NodeId, Vec<usize>> = Default::default();
    for (id, f) in faces {
        if state.doc.map.mesh(*id).is_some_and(|m| *f < m.faces.len()) {
            per_mesh.entry(*id).or_default().push(*f);
        }
    }

    let mut jobs = Vec::new();
    for (id, list) in per_mesh {
        let Some(info) = list.first().and_then(|f| face_info(&state.doc.map, id, *f)) else { continue };
        let r = repeat.unwrap_or_else(|| tex_size(state, &info.material) * info.uv.scale.abs().max(DVec2::splat(1e-3)));
        jobs.push((id, list, r));
    }

    jobs
}

/// Writes explicit UVs on mesh faces. `repeat` is world units per texture repeat, taken from each face's texture and scale when None.
pub fn project_mesh(state: &mut EditorState, faces: &[(NodeId, usize)], projection: UvProjection, repeat: Option<DVec2>) -> usize {
    let jobs = mesh_jobs(state, faces, repeat);
    let n: usize = jobs.iter().map(|(_, l, _)| l.len()).sum();
    if n > 0 {
        state.doc.edit(&format!("{} UV Projection", projection.label()), |m, _| {
            for (id, list, r) in &jobs {
                if let Some(mesh) = m.mesh_mut(*id) {
                    mesh.project_uvs(list, projection, *r);
                }
            }
        });
    }

    n
}

/// Scales explicit UVs of mesh faces to span the texture once.
pub fn normalize_mesh_uvs(state: &mut EditorState, faces: &[(NodeId, usize)], keep_aspect: bool) -> usize {
    let mut per_mesh: std::collections::BTreeMap<NodeId, Vec<usize>> = Default::default();
    for (id, f) in faces {
        if state.doc.map.mesh(*id).is_some() {
            per_mesh.entry(*id).or_default().push(*f);
        }
    }

    let n = per_mesh.values().map(|v| v.len()).sum();
    state.doc.edit("Normalize UVs", |m, _| {
        for (id, list) in &per_mesh {
            if let Some(mesh) = m.mesh_mut(*id) {
                mesh.normalize_uvs(list, keep_aspect);
            }
        }
    });
    n
}

/// Packs the explicit UV islands of the selected mesh faces into 0..1, stacking identical islands.
pub fn pack_mesh_uvs(state: &mut EditorState, faces: &[(NodeId, usize)], stack: bool) -> usize {
    let mut per_mesh: std::collections::BTreeMap<NodeId, Vec<usize>> = Default::default();
    for (id, f) in faces {
        if state.doc.map.mesh(*id).is_some() {
            per_mesh.entry(*id).or_default().push(*f);
        }
    }

    let n = per_mesh.values().map(|v| v.len()).sum();
    if n > 0 {
        state.doc.edit("Pack UVs", |m, _| {
            for (id, list) in &per_mesh {
                if let Some(mesh) = m.mesh_mut(*id) {
                    mesh.pack_uv_islands(list, stack);
                }
            }
        });
    }

    n
}

/// A corner of a mesh face with explicit UVs: (mesh, face, corner).
pub type UvCorner = (NodeId, usize, usize);

/// Edits of explicit UV corners in normalized UV space, where one texture repeat is 1 and v points down the image.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UvAdjust {
    /// Mirrors u across the center of the corners.
    FlipU,
    FlipV,
    /// Degrees around the center, positive turns clockwise on screen.
    Rotate(f64),
    /// Multiplies the layout around the center.
    Scale(DVec2),
    Move(DVec2),
    /// Moves the corners so their center lands here.
    Center(DVec2),
    /// One shared v, the corners line up horizontally.
    AlignHorizontal,
    /// One shared u, the corners line up vertically.
    AlignVertical,
    /// Onto the line between the two corners farthest apart.
    Straighten,
    /// Stretches the bounds of the corners onto [u, v, width, height].
    Fit([f64; 4]),
    /// Rounds to the pixel grid of a texture of this size.
    Snap(DVec2),
}

impl UvAdjust {
    pub fn label(&self) -> &'static str {
        match self {
            UvAdjust::FlipU => "Flip U",
            UvAdjust::FlipV => "Flip V",
            UvAdjust::Rotate(_) => "Rotate UVs",
            UvAdjust::Scale(_) => "Scale UVs",
            UvAdjust::Move(_) | UvAdjust::Center(_) => "Move UV Corners",
            UvAdjust::AlignHorizontal => "Align UVs Horizontally",
            UvAdjust::AlignVertical => "Align UVs Vertically",
            UvAdjust::Straighten => "Straighten UVs",
            UvAdjust::Fit(_) => "Fit UVs",
            UvAdjust::Snap(_) => "Snap UVs to Pixels",
        }
    }
}

/// Center of the bounds of `points`.
pub fn uv_center(points: &[DVec2]) -> DVec2 {
    let (lo, hi) = points.iter().fold((DVec2::MAX, DVec2::MIN), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
    if points.is_empty() { DVec2::ZERO } else { (lo + hi) * 0.5 }
}

pub fn adjust_points(points: &mut [DVec2], op: UvAdjust) {
    if points.is_empty() {
        return;
    }

    let c = uv_center(points);
    let (lo, hi) = points.iter().fold((DVec2::MAX, DVec2::MIN), |(lo, hi), p| (lo.min(*p), hi.max(*p)));
    match op {
        UvAdjust::FlipU => points.iter_mut().for_each(|p| p.x = 2.0 * c.x - p.x),
        UvAdjust::FlipV => points.iter_mut().for_each(|p| p.y = 2.0 * c.y - p.y),
        UvAdjust::Rotate(degrees) => {
            let (s, co) = degrees.to_radians().sin_cos();
            // Quarter turns land exactly, so pixel aligned layouts stay aligned.
            let (s, co) = (snap_whole(s), snap_whole(co));
            for p in points.iter_mut() {
                let d = *p - c;
                *p = c + DVec2::new(d.x * co - d.y * s, d.x * s + d.y * co);
            }
        }
        UvAdjust::Scale(f) => points.iter_mut().for_each(|p| *p = c + (*p - c) * f),
        UvAdjust::Move(d) => points.iter_mut().for_each(|p| *p += d),
        UvAdjust::Center(to) => points.iter_mut().for_each(|p| *p += to - c),
        UvAdjust::AlignHorizontal => {
            let v = points.iter().map(|p| p.y).sum::<f64>() / points.len() as f64;
            points.iter_mut().for_each(|p| p.y = v);
        }
        UvAdjust::AlignVertical => {
            let u = points.iter().map(|p| p.x).sum::<f64>() / points.len() as f64;
            points.iter_mut().for_each(|p| p.x = u);
        }
        UvAdjust::Straighten => {
            let mean = points.iter().copied().sum::<DVec2>() / points.len() as f64;
            let far = |from: DVec2| points.iter().copied().max_by(|a, b| (*a - from).length_squared().total_cmp(&(*b - from).length_squared())).unwrap_or(from);
            let a = far(mean);
            let b = far(a);
            let Some(dir) = (b - a).try_normalize() else { return };
            points.iter_mut().for_each(|p| *p = a + dir * (*p - a).dot(dir));
        }
        UvAdjust::Fit(r) => {
            let size = hi - lo;
            for p in points.iter_mut() {
                let t = DVec2::new(if size.x > 1e-9 { (p.x - lo.x) / size.x } else { 0.5 }, if size.y > 1e-9 { (p.y - lo.y) / size.y } else { 0.5 });
                *p = DVec2::new(r[0] + t.x * r[2], r[1] + t.y * r[3]);
            }
        }
        UvAdjust::Snap(tex) => {
            let tex = tex.max(DVec2::ONE);
            points.iter_mut().for_each(|p| *p = (*p * tex).round() / tex);
        }
    }
}

fn snap_whole(x: f64) -> f64 {
    if (x - x.round()).abs() < 1e-12 { x.round() } else { x }
}

/// Every explicit UV corner of the listed mesh faces.
pub fn all_corners(map: &Map, faces: &[(NodeId, usize)]) -> Vec<UvCorner> {
    faces
        .iter()
        .filter_map(|(id, f)| {
            let face = map.mesh(*id)?.faces.get(*f)?;
            (face.uvs.len() == face.indices.len() && !face.uvs.is_empty()).then(|| (0..face.uvs.len()).map(move |k| (*id, *f, k)))
        })
        .flatten()
        .collect()
}

pub fn corner_uv(map: &Map, (id, f, k): UvCorner) -> Option<DVec2> {
    let face = map.mesh(id)?.faces.get(f)?;
    (face.uvs.len() == face.indices.len()).then(|| face.uvs.get(k).map(|u| DVec2::new(u[0] as f64, u[1] as f64))).flatten()
}

/// `corner` and the corners stitched to it: the same mesh vertex with the same UV on the other listed faces.
pub fn stitched_corners(map: &Map, faces: &[(NodeId, usize)], corner: UvCorner) -> Vec<UvCorner> {
    let (id, face, k) = corner;
    let Some(mesh) = map.mesh(id) else { return vec![corner] };
    let Some(f) = mesh.faces.get(face) else { return vec![corner] };
    let (Some(vertex), Some(uv)) = (f.indices.get(k).copied(), f.uvs.get(k).copied()) else { return vec![corner] };
    let mut out = vec![corner];
    for (_, other_face) in faces.iter().filter(|(i, f)| *i == id && *f != face) {
        let Some(other) = mesh.faces.get(*other_face) else { continue };
        for (j, v) in other.indices.iter().enumerate() {
            if *v == vertex && other.uvs.get(j).is_some_and(|u| (u[0] - uv[0]).abs() < 1e-5 && (u[1] - uv[1]).abs() < 1e-5) {
                out.push((id, *other_face, j));
            }
        }
    }

    out
}

/// Adds the stitched partners of every corner.
pub fn with_stitched(map: &Map, faces: &[(NodeId, usize)], corners: &[UvCorner]) -> Vec<UvCorner> {
    let mut set: std::collections::BTreeSet<UvCorner> = Default::default();
    for c in corners {
        set.extend(stitched_corners(map, faces, *c));
    }

    set.into_iter().collect()
}

/// Writes new UVs for corners of explicit mesh faces. Inside an open transaction this edits in place.
pub fn set_corner_uvs(state: &mut EditorState, label: &str, coalesce: bool, uvs: &[(UvCorner, DVec2)]) {
    let apply = |m: &mut Map| {
        for ((id, f, k), uv) in uvs {
            if let Some(face) = m.mesh_mut(*id).and_then(|mesh| mesh.faces.get_mut(*f))
                && face.uvs.len() == face.indices.len()
                && let Some(u) = face.uvs.get_mut(*k)
            {
                *u = [uv.x as f32, uv.y as f32];
            }
        }
    };
    if coalesce {
        state.doc.edit_coalesced(label, |m, _| apply(m));
    } else {
        state.doc.edit(label, |m, _| apply(m));
    }
}

/// Applies `op` to the corners together, as one undo step. Returns how many corners changed.
pub fn adjust_corners(state: &mut EditorState, corners: &[UvCorner], op: UvAdjust, coalesce: bool) -> usize {
    let list: Vec<UvCorner> = corners.iter().copied().filter(|c| corner_uv(&state.doc.map, *c).is_some()).collect();
    let mut points: Vec<DVec2> = list.iter().filter_map(|c| corner_uv(&state.doc.map, *c)).collect();
    if points.is_empty() {
        return 0;
    }

    adjust_points(&mut points, op);
    let uvs: Vec<(UvCorner, DVec2)> = list.into_iter().zip(points).collect();
    set_corner_uvs(state, op.label(), coalesce, &uvs);
    uvs.len()
}

/// World space u and v axis directions and the world size of one texture repeat on a face, for overlays.
pub fn face_axes(state: &EditorState, info: &FaceInfo) -> (DVec3, DVec3, DVec2) {
    let tex = tex_size(state, &info.material);
    let n = info.plane.normal;
    let flat = |a: DVec3| (a - n * a.dot(n)).normalize_or_zero();
    (flat(info.uv.u_axis), flat(info.uv.v_axis), tex * info.uv.scale.abs())
}
