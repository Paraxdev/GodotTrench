//! Texture alignment operations shared by the texture tool, the face inspector and the MCP server.
//! Faces are brush faces or mesh faces. Mesh faces with explicit UVs are edited in normalized UV space.

use gt_core::{DVec2, DVec3, NodeId, Plane};
use gt_doc::Map;
use gt_geom::{FaceUv, Justify, UvProjection};

use crate::state::EditorState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshUvKind {
    /// Back to the planar projection stored on the face.
    Planar,
    Box,
    Cylinder,
    Sphere,
    View,
    Unfold,
    Normalize,
    /// Packs the UV islands into 0..1 with a uniform texel density, stacking identical islands.
    Pack,
}

impl MeshUvKind {
    pub const ALL: [MeshUvKind; 8] = [
        MeshUvKind::Planar,
        MeshUvKind::Box,
        MeshUvKind::Cylinder,
        MeshUvKind::Sphere,
        MeshUvKind::View,
        MeshUvKind::Unfold,
        MeshUvKind::Normalize,
        MeshUvKind::Pack,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            MeshUvKind::Planar => "Planar",
            MeshUvKind::Box => "Box",
            MeshUvKind::Cylinder => "Cylinder",
            MeshUvKind::Sphere => "Sphere",
            MeshUvKind::View => "View",
            MeshUvKind::Unfold => "Unfold",
            MeshUvKind::Normalize => "Normalize",
            MeshUvKind::Pack => "Pack",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.label().eq_ignore_ascii_case(name))
    }
}

/// Runs a mesh UV layout on faces. `view` is the camera's (right, up) for the view projection.
pub fn mesh_uv(state: &mut EditorState, faces: &[(NodeId, usize)], kind: MeshUvKind, view: (DVec3, DVec3)) -> usize {
    let projection = match kind {
        MeshUvKind::Planar => {
            let targets: Vec<FaceInfo> = infos(state, faces).into_iter().filter(|i| i.explicit.is_some()).collect();
            let n = targets.len();
            state.doc.edit("Planar UVs", |m, _| {
                for t in &targets {
                    set_uv(m, t.id, t.face, t.uv.clone());
                }
            });
            return n;
        }
        MeshUvKind::Normalize => return normalize_mesh_uvs(state, faces, true),
        MeshUvKind::Pack => return pack_mesh_uvs(state, faces, true),
        MeshUvKind::Box => UvProjection::Box,
        MeshUvKind::Cylinder => UvProjection::Cylinder { axis: DVec3::Y },
        MeshUvKind::Sphere => UvProjection::Sphere,
        MeshUvKind::View => UvProjection::View { right: view.0, up: view.1 },
        MeshUvKind::Unfold => UvProjection::Unfold,
    };
    project_mesh(state, faces, projection, None)
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
    state.materials.size(material).map(|s| DVec2::new(s[0] as f64, s[1] as f64)).unwrap_or(DVec2::splat(state.game.textures.fallback_size as f64))
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

/// Writes explicit UVs on mesh faces. `repeat` is world units per texture repeat, taken from each face's texture and scale when None.
pub fn project_mesh(state: &mut EditorState, faces: &[(NodeId, usize)], projection: UvProjection, repeat: Option<DVec2>) -> usize {
    let mut per_mesh: std::collections::BTreeMap<NodeId, Vec<usize>> = Default::default();
    for (id, f) in faces {
        if state.doc.map.mesh(*id).is_some() {
            per_mesh.entry(*id).or_default().push(*f);
        }
    }

    let mut jobs = Vec::new();
    for (id, list) in per_mesh {
        let Some(info) = list.first().and_then(|f| face_info(&state.doc.map, id, *f)) else { continue };
        let r = repeat.unwrap_or_else(|| tex_size(state, &info.material) * info.uv.scale.abs().max(DVec2::splat(1e-3)));
        jobs.push((id, list, r));
    }

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

/// World space u and v axis directions and the world size of one texture repeat on a face, for overlays.
pub fn face_axes(state: &EditorState, info: &FaceInfo) -> (DVec3, DVec3, DVec2) {
    let tex = tex_size(state, &info.material);
    let n = info.plane.normal;
    let flat = |a: DVec3| (a - n * a.dot(n)).normalize_or_zero();
    (flat(info.uv.u_axis), flat(info.uv.v_axis), tex * info.uv.scale.abs())
}
