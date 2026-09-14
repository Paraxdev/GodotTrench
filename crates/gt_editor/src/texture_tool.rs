//! Hammer style texture application tool. Click selects faces, right click applies the current material, Alt+right
//! click applies it wrapped from the selected face, Alt+click picks a material and alignment, dragging a selected
//! face slides its texture, Ctrl+wheel scales and Alt+wheel rotates.

use egui::{Align2, Color32, FontId, Key, PointerButton, Pos2, Rect, Response, Ui, Vec2};
use gt_core::{DMat3, DVec2, DVec3, NodeId, Plane};
use gt_render::LineVertex;

use crate::camera::Camera;
use crate::picking;
use crate::scene::v3;
use crate::state::EditorState;
use crate::texture_ops::{self, FaceInfo};

#[derive(Default)]
pub struct TextureTool {
    pub hover: Option<(NodeId, usize)>,
    drag: Option<SlideDrag>,
}

struct SlideDrag {
    plane: Plane,
    start: DVec3,
    uv: gt_geom::FaceUv,
    applied: DVec2,
}

fn line(out: &mut Vec<LineVertex>, a: DVec3, b: DVec3, color: [f32; 4]) {
    out.push(LineVertex { pos: v3(a), color });
    out.push(LineVertex { pos: v3(b), color });
}

/// World position of a texel coordinate on the face plane.
fn texel_to_world(info: &FaceInfo, texel: DVec2) -> Option<DVec3> {
    let uv = &info.uv;
    let n = info.plane.normal;
    let rows = DMat3::from_cols(uv.u_axis, uv.v_axis, n).transpose();
    if rows.determinant().abs() < 1e-9 {
        return None;
    }
    let rhs = DVec3::new((texel.x - uv.offset.x) * uv.scale.x, (texel.y - uv.offset.y) * uv.scale.y, info.plane.dist);
    Some(rows.inverse() * rhs)
}

impl TextureTool {
    pub fn reset(&mut self) {
        self.drag = None;
        self.hover = None;
    }

    fn primary_face(state: &EditorState) -> Option<(NodeId, usize)> {
        state.doc.selection.faces.iter().next().copied()
    }

    pub fn keys(&mut self, ctx: &egui::Context, state: &mut EditorState) -> bool {
        let faces: Vec<(NodeId, usize)> = state.doc.selection.faces.iter().copied().collect();
        if faces.is_empty() {
            return false;
        }
        let step = if ctx.input(|i| i.modifiers.shift) { 1.0 } else { state.grid.max(1.0) };
        let mut delta = DVec2::ZERO;
        for (key, d) in [
            (Key::ArrowLeft, DVec2::new(step, 0.0)),
            (Key::ArrowRight, DVec2::new(-step, 0.0)),
            (Key::ArrowUp, DVec2::new(0.0, step)),
            (Key::ArrowDown, DVec2::new(0.0, -step)),
        ] {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key) || i.consume_key(egui::Modifiers::SHIFT, key)) {
                delta += d;
            }
        }
        if delta != DVec2::ZERO {
            texture_ops::shift(state, &faces, delta);
            return true;
        }
        false
    }

    pub fn input(&mut self, ui: &Ui, response: &Response, cam: &Camera, rect: Rect, hover: Option<Pos2>, state: &mut EditorState) {
        let modifiers = ui.input(|i| i.modifiers);
        let face_at = |pos: Pos2, state: &EditorState| picking::pick(state, &cam.ray(rect, pos)).and_then(|h| h.face.map(|f| (h.node, f)));
        self.hover = hover.and_then(|p| face_at(p, state));

        if response.clicked_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            match face_at(pos, state) {
                Some(face) if modifiers.alt => {
                    texture_ops::eyedropper(state, face);
                }
                Some((id, f)) => state.doc.select(|_, s| {
                    if modifiers.command {
                        s.toggle_face(id, f);
                    } else {
                        s.clear();
                        s.select_face(id, f);
                    }
                }),
                None if !modifiers.command => state.doc.select(|_, s| s.clear()),
                None => {}
            }
        }

        if response.clicked_by(PointerButton::Secondary)
            && let Some(face) = response.interact_pointer_pos().and_then(|p| face_at(p, state))
        {
            let material = state.current_material.clone();
            if modifiers.alt {
                match Self::primary_face(state).filter(|p| *p != face) {
                    Some(src) => {
                        texture_ops::wrap_from(state, src, &[face], Some(&material));
                        state.set_status(format!("Wrapped {material} from the selected face"));
                    }
                    None => state.set_status("Select a source face first, then Alt+right click the faces to wrap onto"),
                }
            } else if modifiers.shift {
                if texture_ops::paste_alignment(state, &[face], true) == 0 {
                    state.set_status("Pick a face with Alt+click first");
                }
            } else {
                texture_ops::apply_material(state, &[face], &material, modifiers.command);
            }
        }

        if response.drag_started_by(PointerButton::Primary)
            && !modifiers.alt
            && let Some(origin) = ui.input(|i| i.pointer.press_origin())
            && let Some(face) = face_at(origin, state)
            && state.doc.selection.faces.contains(&face)
            && let Some(info) = texture_ops::face_info(&state.doc.map, face.0, face.1)
        {
            let ray = cam.ray(rect, origin);
            if let Some(t) = ray.intersect_plane(&info.plane) {
                self.drag = Some(SlideDrag { plane: info.plane, start: ray.at(t), uv: info.uv, applied: DVec2::ZERO });
            }
        }
        if let Some(drag) = &mut self.drag {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                let ray = cam.ray(rect, pos);
                if let Some(t) = ray.intersect_plane(&drag.plane) {
                    // The texel under the cursor stays under the cursor.
                    let mut want = drag.uv.texel(drag.start) - drag.uv.texel(ray.at(t));
                    if state.snap {
                        want = want.round();
                    }
                    let step = want - drag.applied;
                    if step != DVec2::ZERO {
                        let faces: Vec<(NodeId, usize)> = state.doc.selection.faces.iter().copied().collect();
                        texture_ops::shift(state, &faces, step);
                        drag.applied = want;
                    }
                }
            }
            if !ui.input(|i| i.pointer.primary_down()) {
                self.drag = None;
            }
        }

        if response.hovered() {
            let wheel: Vec<(f32, egui::Modifiers)> = ui.input(|i| {
                i.raw
                    .events
                    .iter()
                    .filter_map(|e| match e {
                        egui::Event::MouseWheel { delta, modifiers, .. } if delta.y != 0.0 => Some((delta.y, *modifiers)),
                        _ => None,
                    })
                    .collect()
            });
            let faces: Vec<(NodeId, usize)> = state.doc.selection.faces.iter().copied().collect();
            for (dy, m) in wheel {
                if faces.is_empty() {
                    break;
                }
                let dir = dy.signum() as f64;
                if m.command {
                    let f = if m.shift { 1.1f64 } else { 2.0 };
                    texture_ops::scale(state, &faces, DVec2::splat(f.powf(dir)));
                } else if m.alt {
                    texture_ops::rotate(state, &faces, if m.shift { 1.0 } else { 15.0 } * dir);
                }
            }
        }
    }

    pub fn lines(&self, state: &EditorState) -> Vec<LineVertex> {
        let mut out = Vec::new();
        let map = &state.doc.map;
        if let Some((id, f)) = self.hover
            && let Some(info) = texture_ops::face_info(map, id, f)
        {
            let n = info.plane.normal * 0.2;
            for k in 0..info.points.len() {
                line(&mut out, info.points[k] + n, info.points[(k + 1) % info.points.len()] + n, [1.0, 0.9, 0.3, 0.9]);
            }
        }
        for (id, f) in state.doc.selection.faces.iter().take(64) {
            let Some(info) = texture_ops::face_info(map, *id, *f) else { continue };
            let lift = info.plane.normal * 0.3;
            let center = info.center() + lift;
            let (u, v, repeat) = texture_ops::face_axes(state, &info);
            let size = info.points.iter().map(|p| (*p - info.center()).length()).fold(0.0, f64::max).max(4.0);
            line(&mut out, center, center + u * repeat.x.min(size), [1.0, 0.3, 0.3, 1.0]);
            line(&mut out, center, center + v * repeat.y.min(size), [0.3, 1.0, 0.4, 1.0]);
            if info.explicit.is_some() {
                continue;
            }
            // Outline of the texture tile under the face center.
            let tex = texture_ops::tex_size(state, &info.material);
            let t = info.uv.texel(info.center());
            let origin = DVec2::new((t.x / tex.x).floor() * tex.x, (t.y / tex.y).floor() * tex.y);
            let corners: Vec<DVec3> = [DVec2::ZERO, DVec2::new(tex.x, 0.0), tex, DVec2::new(0.0, tex.y)]
                .iter()
                .filter_map(|c| texel_to_world(&info, origin + *c))
                .map(|p| p + lift)
                .collect();
            if corners.len() == 4 {
                for k in 0..4 {
                    line(&mut out, corners[k], corners[(k + 1) % 4], [0.4, 0.8, 1.0, 0.8]);
                }
            }
        }
        out
    }

    pub fn paint(&self, ui: &Ui, rect: Rect, state: &EditorState) {
        let painter = ui.painter_at(rect);
        let clip = state.uv_clipboard.as_ref().map(|c| format!("   picked: {}", c.material)).unwrap_or_default();
        let text = format!(
            "Texture: click selects faces, right click applies {}, Alt+right wraps, Shift+right pastes, Alt+click picks, drag slides, Ctrl+wheel scales, Alt+wheel rotates{clip}",
            state.current_material
        );
        painter.text(rect.left_bottom() + Vec2::new(8.0, -8.0), Align2::LEFT_BOTTOM, text, FontId::proportional(12.0), Color32::from_rgb(170, 220, 255));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texel_to_world_inverts_the_projection() {
        let mut uv = gt_geom::FaceUv::paraxial(DVec3::Z, DVec2::new(0.5, 2.0));
        uv.offset = DVec2::new(3.0, -5.0);
        let plane = Plane::from_point_normal(DVec3::new(0.0, 0.0, 16.0), DVec3::Z);
        let info = FaceInfo { id: NodeId(1), face: 0, points: vec![], plane, material: String::new(), uv: uv.clone(), explicit: None };
        let p = texel_to_world(&info, DVec2::new(40.0, 12.0)).unwrap();
        assert!((uv.texel(p) - DVec2::new(40.0, 12.0)).length() < 1e-9);
        assert!((p.z - 16.0).abs() < 1e-9);
    }
}
