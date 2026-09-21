//! Graphical UV editor. Every selected face with the first face's material is drawn over its tiled texture.
//! Planar faces: drag moves the texture, wheel scales, right drag rotates.
//! Mesh faces with explicit UVs: click or Shift+click corner handles, drag them (stitched corners follow), drag empty space to move whole faces.

use std::collections::BTreeSet;

use egui::{Color32, Pos2, Rect, RichText, Sense, Shape, Stroke, StrokeKind, Ui, Vec2};
use gt_core::{DVec2, NodeId};

use crate::commands::Action;
use crate::panels::PanelState;
use crate::state::EditorState;
use crate::texture_ops::{self, FaceInfo};

/// A corner of a mesh face with explicit UVs: (mesh, face, corner).
pub type UvCorner = (NodeId, usize, usize);

#[derive(Default)]
pub struct UvEditorState {
    pub selected: BTreeSet<UvCorner>,
    pub pixel_snap: bool,
    pub show_hotspots: bool,
    dragging_corners: bool,
}

fn corner_uvs(info: &FaceInfo, tex: DVec2) -> Vec<DVec2> {
    match &info.explicit {
        Some(uvs) => uvs.iter().map(|u| DVec2::new(u[0] as f64, u[1] as f64)).collect(),
        None => info.points.iter().map(|p| info.uv.uv(*p, tex)).collect(),
    }
}

/// Corners stitched to `corner`: same mesh vertex and same UV on other listed faces.
fn stitched(state: &EditorState, faces: &[FaceInfo], corner: UvCorner) -> Vec<UvCorner> {
    let (id, face, k) = corner;
    let Some(mesh) = state.doc.map.mesh(id) else { return vec![corner] };
    let Some(f) = mesh.faces.get(face) else { return vec![corner] };
    let (Some(vertex), Some(uv)) = (f.indices.get(k).copied(), f.uvs.get(k).copied()) else { return vec![corner] };
    let mut out = vec![corner];
    for info in faces.iter().filter(|i| i.id == id && i.face != face) {
        let other = &mesh.faces[info.face];
        for (j, v) in other.indices.iter().enumerate() {
            if *v == vertex && other.uvs.get(j).is_some_and(|u| (u[0] - uv[0]).abs() < 1e-5 && (u[1] - uv[1]).abs() < 1e-5) {
                out.push((id, info.face, j));
            }
        }
    }

    out
}

pub fn uv_editor(ui: &mut Ui, state: &mut EditorState, ps: &mut PanelState, actions: &mut Vec<Action>) {
    let selected_faces: Vec<(NodeId, usize)> = state.doc.selection.faces.iter().copied().collect();
    let Some(first) = selected_faces.first().and_then(|(id, f)| texture_ops::face_info(&state.doc.map, *id, *f)) else {
        ui.label(
            RichText::new("Select faces to edit their UVs: Shift+click a face, or Ctrl+click an object to grab all its sides, or use the Texture tool.").weak(),
        );
        return;
    };
    let material = first.material.clone();
    let tex = texture_ops::tex_size(state, &material);
    let faces: Vec<FaceInfo> =
        selected_faces.iter().filter_map(|(id, f)| texture_ops::face_info(&state.doc.map, *id, *f)).filter(|i| i.material == material).collect();
    let uv_state = &mut ps.uv;
    uv_state.selected.retain(|(id, f, _)| faces.iter().any(|i| i.id == *id && i.face == *f && i.explicit.is_some()));

    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(&material).strong());
        ui.label(RichText::new(format!("{} x {} px, {} faces", tex.x, tex.y, faces.len())).weak());
        ui.checkbox(&mut uv_state.pixel_snap, "pixel snap");
        ui.checkbox(&mut uv_state.show_hotspots, "hotspots");
        if ui.small_button("Reset UVs").on_hover_text("Drop any custom UVs on the selected faces back to a clean face-aligned projection").clicked() {
            actions.push(Action::ResetTexture);
        }

        for (label, action) in [("Hotspot", Action::HotspotTexture), ("Hotspot Editor", Action::ShowHotspotEditor), ("UV Lock", Action::ToggleUvLock)] {
            if ui.small_button(label).clicked() {
                actions.push(action);
            }
        }
    });
    if faces.iter().any(|f| f.explicit.is_some()) {
        ui.horizontal_wrapped(|ui| {
            for k in texture_ops::MeshUvKind::ALL {
                if ui.small_button(k.label()).clicked() {
                    actions.push(Action::MeshUv(k));
                }
            }
        });
    }

    ui.label(RichText::new("Drag moves, wheel scales, right drag rotates. Mesh UV corners: click, Shift+click, drag.").weak());

    let face_uvs: Vec<Vec<DVec2>> = faces.iter().map(|f| corner_uvs(f, tex)).collect();
    let avail = ui.available_size();
    let side = avail.x.min(avail.y).max(120.0);
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::click_and_drag());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::from_gray(18));

    let (lo, hi) = face_uvs.iter().flatten().fold((DVec2::MAX, DVec2::MIN), |(lo, hi), u| (lo.min(*u), hi.max(*u)));
    let center = (lo + hi) * 0.5;
    let extent = (hi - lo).max_element().max(1.0) * 0.75;
    let origin = center - DVec2::splat(extent);
    let to_screen = |uv: DVec2| -> Pos2 {
        let t = (uv - origin) / (extent * 2.0);
        Pos2::new(rect.min.x + t.x as f32 * rect.width(), rect.min.y + t.y as f32 * rect.height())
    };
    let to_uv = |p: Pos2| origin + DVec2::new(((p.x - rect.min.x) / rect.width()) as f64, ((p.y - rect.min.y) / rect.height()) as f64) * extent * 2.0;
    let px_per_uv = rect.width() as f64 / (extent * 2.0);

    let ctx = ui.ctx().clone();
    if let Some(texture) = state.materials.full_texture(&ctx, &material) {
        let start = origin.floor();
        let end = (center + DVec2::splat(extent)).ceil();
        if (end.x - start.x) * (end.y - start.y) <= 400.0 {
            let mut y = start.y;
            while y < end.y {
                let mut x = start.x;
                while x < end.x {
                    let r = Rect::from_min_max(to_screen(DVec2::new(x, y)), to_screen(DVec2::new(x + 1.0, y + 1.0)));
                    painter.image(texture.id(), r, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::from_gray(150));
                    painter.rect_stroke(r, 0.0, Stroke::new(1.0, Color32::from_black_alpha(90)), StrokeKind::Inside);
                    x += 1.0;
                }

                y += 1.0;
            }
        }
    }

    if uv_state.show_hotspots {
        for r in crate::commands::hotspot_rects(state, &material) {
            let a = to_screen(DVec2::new(r[0], r[1]) / tex);
            let b = to_screen(DVec2::new(r[0] + r[2], r[1] + r[3]) / tex);
            painter.rect_stroke(Rect::from_two_pos(a, b), 0.0, Stroke::new(1.0, Color32::from_rgb(90, 220, 255)), StrokeKind::Inside);
        }
    }

    for (info, uvs) in faces.iter().zip(&face_uvs) {
        let pts: Vec<Pos2> = uvs.iter().map(|u| to_screen(*u)).collect();
        let color = if info.explicit.is_some() { Color32::from_rgb(120, 200, 255) } else { Color32::from_rgb(255, 150, 60) };
        painter.add(Shape::closed_line(pts.clone(), Stroke::new(1.5, color)));
        if info.explicit.is_none() {
            painter.add(Shape::convex_polygon(pts.clone(), Color32::from_rgba_unmultiplied(255, 120, 40, 30), Stroke::NONE));
        }

        for (k, p) in pts.iter().enumerate() {
            let sel = uv_state.selected.contains(&(info.id, info.face, k));
            painter.circle_filled(*p, if sel { 4.5 } else { 3.0 }, if sel { Color32::from_rgb(255, 230, 90) } else { Color32::from_rgb(230, 230, 230) });
        }
    }

    let pointer = response.interact_pointer_pos();
    let corner_at = |p: Pos2| -> Option<UvCorner> {
        faces
            .iter()
            .zip(&face_uvs)
            .filter(|(i, _)| i.explicit.is_some())
            .flat_map(|(i, uvs)| uvs.iter().enumerate().map(move |(k, u)| ((i.id, i.face, k), (to_screen(*u) - p).length())))
            .filter(|(_, d)| *d < 8.0)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(c, _)| c)
    };
    let shift = ui.input(|i| i.modifiers.shift);
    if response.clicked()
        && let Some(p) = pointer
    {
        match corner_at(p) {
            Some(c) => {
                if !shift {
                    uv_state.selected.clear();
                }

                for s in stitched(state, &faces, c) {
                    uv_state.selected.insert(s);
                }
            }
            None if !shift => uv_state.selected.clear(),
            None => {}
        }
    }

    if response.drag_started_by(egui::PointerButton::Primary)
        && let Some(origin_pos) = ui.input(|i| i.pointer.press_origin())
    {
        uv_state.dragging_corners = match corner_at(origin_pos) {
            Some(c) => {
                if !uv_state.selected.contains(&c) {
                    if !shift {
                        uv_state.selected.clear();
                    }

                    for s in stitched(state, &faces, c) {
                        uv_state.selected.insert(s);
                    }
                }

                true
            }
            None => false,
        };
    }

    let delta = response.drag_delta();
    let scroll = if response.hovered() { ui.input(|i| i.smooth_scroll_delta.y) } else { 0.0 };
    let all: Vec<(NodeId, usize)> = faces.iter().map(|i| (i.id, i.face)).collect();
    if response.dragged_by(egui::PointerButton::Primary) && delta != Vec2::ZERO {
        let duv = DVec2::new(delta.x as f64, delta.y as f64) / px_per_uv;
        if uv_state.dragging_corners && !uv_state.selected.is_empty() {
            let selected = uv_state.selected.clone();
            let snap = uv_state.pixel_snap;
            // Snapping follows the pointer so small drags still move a whole texel at a time.
            let target = pointer.map(to_uv);
            state.doc.edit_coalesced("Move UV Corners", |m, _| {
                for (id, f, k) in &selected {
                    if let Some(face) = m.mesh_mut(*id).and_then(|mesh| mesh.faces.get_mut(*f))
                        && let Some(u) = face.uvs.get_mut(*k)
                    {
                        let mut next = DVec2::new(u[0] as f64, u[1] as f64) + duv;
                        if snap && selected.len() == 1 {
                            next = (target.unwrap_or(next) * tex).round() / tex;
                        }

                        *u = [next.x as f32, next.y as f32];
                    }
                }
            });
        } else {
            // The face outline follows the pointer over the texture.
            texture_ops::shift(state, &all, duv * tex);
        }
    }

    if response.dragged_by(egui::PointerButton::Secondary) && delta.x != 0.0 {
        texture_ops::rotate(state, &all, delta.x as f64 * 0.5);
    }

    if scroll != 0.0 {
        let factor = (1.0 + scroll as f64 * 0.002).clamp(0.5, 2.0);
        texture_ops::scale(state, &all, DVec2::splat(factor));
    }

    if response.drag_stopped() {
        uv_state.dragging_corners = false;
        if uv_state.pixel_snap {
            let selected = uv_state.selected.clone();
            state.doc.edit_coalesced("Move UV Corners", |m, _| {
                for (id, f, k) in &selected {
                    if let Some(u) = m.mesh_mut(*id).and_then(|mesh| mesh.faces.get_mut(*f)).and_then(|face| face.uvs.get_mut(*k)) {
                        *u = [((u[0] as f64 * tex.x).round() / tex.x) as f32, ((u[1] as f64 * tex.y).round() / tex.y) as f32];
                    }
                }
            });
        }
    }
}
