//! Move, rotate and scale handles on the selection in the 3D view while the select tool is active.
//!
//! Every drag frame undoes the previous frame and applies the transform from the drag start again, so snapping never
//! accumulates rounding.

use egui::{Color32, CursorIcon, Pos2, Rect, Shape, Stroke, Ui, Vec2};
use gt_core::{Aabb, DVec3, Plane};
use gt_doc::ops;

use crate::camera::{Camera, ViewKind};
use crate::commands::axis_name;
use crate::state::EditorState;
use crate::tools::ToolKind;

const AXIS_COLORS: [Color32; 3] = crate::theme::AXIS;
const HOT: Color32 = crate::theme::YELLOW;
/// Pointer distance in points that still grabs a handle.
const GRAB: f32 = 8.0;
const ROTATE_SNAP_DEGREES: f64 = 15.0;
/// Arrow length as a fraction of the camera distance.
const GIZMO_SCALE: f64 = 0.22;
/// Handle positions as fractions of the arrow length.
const SHAFT_START: f64 = 0.14;
const HEAD_START: f64 = 0.78;
const SCALE_AT: f64 = 1.18;
const RING_RADIUS: f64 = 0.62;
const PLANE_NEAR: f64 = 0.2;
const PLANE_FAR: f64 = 0.4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// Along one axis.
    Move(usize),
    /// In the plane with this axis as its normal.
    Plane(usize),
    /// In the view plane.
    Free,
    Rotate(usize),
    /// Along one axis, symmetric around the center.
    Scale(usize),
}

impl Part {
    fn label(self) -> &'static str {
        match self {
            Part::Move(_) | Part::Plane(_) | Part::Free => "Move",
            Part::Rotate(_) => "Rotate",
            Part::Scale(_) => "Scale",
        }
    }
}

#[derive(Clone, Debug)]
pub struct GizmoDrag {
    pub part: Part,
    center: DVec3,
    base: Aabb,
    plane: Plane,
    start: DVec3,
}

fn axis(i: usize) -> DVec3 {
    let mut v = DVec3::ZERO;
    v[i] = 1.0;
    v
}

/// Selection bounds the gizmo is drawn on, None when it is hidden.
fn target(state: &EditorState, cam: &Camera) -> Option<Aabb> {
    let sel = &state.doc.selection;
    if cam.kind != ViewKind::Perspective || state.tool != ToolKind::Select || !state.prefs.transform_gizmo || sel.has_faces() || sel.nodes.is_empty() {
        return None;
    }
    let bounds = state.doc.map.bounds_of(sel.nodes.iter().copied());
    (!bounds.is_empty()).then_some(bounds)
}

/// World space placement of the handles. The arrow length follows the camera distance so the gizmo keeps its screen size.
struct Layout {
    center: DVec3,
    len: f64,
    /// Plane squares sit on the side of each axis that faces the camera.
    facing: DVec3,
}

impl Layout {
    fn new(cam: &Camera, bounds: &Aabb) -> Self {
        let center = bounds.center();
        let to_cam = cam.position - center;
        let facing = DVec3::new(if to_cam.x < 0.0 { -1.0 } else { 1.0 }, if to_cam.y < 0.0 { -1.0 } else { 1.0 }, if to_cam.z < 0.0 { -1.0 } else { 1.0 });
        Self { center, len: to_cam.length().max(1.0) * GIZMO_SCALE, facing }
    }

    fn along(&self, i: usize, fraction: f64) -> DVec3 {
        self.center + axis(i) * self.len * fraction
    }

    fn plane_quad(&self, normal: usize) -> [DVec3; 4] {
        let (u, v) = ((normal + 1) % 3, (normal + 2) % 3);
        let du = axis(u) * self.facing[u] * self.len;
        let dv = axis(v) * self.facing[v] * self.len;
        let c = self.center;
        [c + du * PLANE_NEAR + dv * PLANE_NEAR, c + du * PLANE_FAR + dv * PLANE_NEAR, c + du * PLANE_FAR + dv * PLANE_FAR, c + du * PLANE_NEAR + dv * PLANE_FAR]
    }

    fn ring(&self, i: usize) -> Vec<DVec3> {
        let (u, v) = (axis((i + 1) % 3), axis((i + 2) % 3));
        (0..=64)
            .map(|k| {
                let a = std::f64::consts::TAU * k as f64 / 64.0;
                self.center + (u * a.cos() + v * a.sin()) * self.len * RING_RADIUS
            })
            .collect()
    }
}

fn project_all(cam: &Camera, rect: Rect, points: &[DVec3]) -> Option<Vec<Pos2>> {
    points.iter().map(|p| cam.project(rect, *p)).collect()
}

fn segment_distance(a: Pos2, b: Pos2, p: Pos2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_sq().max(1e-6)).clamp(0.0, 1.0);
    (a + ab * t).distance(p)
}

fn inside_convex(points: &[Pos2], p: Pos2) -> bool {
    let cross = |a: Pos2, b: Pos2| (b - a).x * (p - a).y - (b - a).y * (p - a).x;
    let signs: Vec<f32> = (0..points.len()).map(|k| cross(points[k], points[(k + 1) % points.len()])).collect();
    signs.iter().all(|s| *s >= 0.0) || signs.iter().all(|s| *s <= 0.0)
}

fn part_at(cam: &Camera, rect: Rect, bounds: &Aabb, pos: Pos2) -> Option<Part> {
    let l = Layout::new(cam, bounds);
    let screen = |p: DVec3| cam.project(rect, p);
    if screen(l.center).is_some_and(|c| c.distance(pos) < GRAB) {
        return Some(Part::Free);
    }
    let closest =
        |candidates: Vec<(f32, Part)>, limit: f32| candidates.into_iter().filter(|(d, _)| *d < limit).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, p)| p);
    let scale = (0..3).filter_map(|i| screen(l.along(i, SCALE_AT)).map(|s| (s.distance(pos), Part::Scale(i)))).collect();
    if let Some(part) = closest(scale, GRAB) {
        return Some(part);
    }
    let arrows = (0..3).filter_map(|i| Some((segment_distance(screen(l.along(i, SHAFT_START))?, screen(l.along(i, 1.0))?, pos), Part::Move(i)))).collect();
    if let Some(part) = closest(arrows, GRAB) {
        return Some(part);
    }
    if let Some(i) = (0..3).find(|i| project_all(cam, rect, &l.plane_quad(*i)).is_some_and(|q| inside_convex(&q, pos))) {
        return Some(Part::Plane(i));
    }
    let rings = (0..3)
        .filter_map(|i| {
            let points = project_all(cam, rect, &l.ring(i))?;
            Some((points.windows(2).map(|w| segment_distance(w[0], w[1], pos)).fold(f32::MAX, f32::min), Part::Rotate(i)))
        })
        .collect();
    closest(rings, GRAB * 0.75)
}

/// The gizmo part under `pos`, if the gizmo is shown.
pub fn hit(state: &EditorState, cam: &Camera, rect: Rect, pos: Pos2) -> Option<Part> {
    part_at(cam, rect, &target(state, cam)?, pos)
}

fn drag_plane(cam: &Camera, part: Part, center: DVec3) -> Plane {
    let forward = cam.forward();
    let normal = match part {
        // The plane through the axis that faces the camera the most.
        Part::Move(i) | Part::Scale(i) => {
            let a = axis(i);
            let n = a.cross(forward).cross(a);
            if n.length_squared() < 1e-9 { forward } else { n.normalize() }
        }
        Part::Plane(i) | Part::Rotate(i) => axis(i),
        Part::Free => forward,
    };
    Plane::from_point_normal(center, normal)
}

pub fn begin(state: &mut EditorState, cam: &Camera, rect: Rect, pos: Pos2) -> Option<GizmoDrag> {
    let base = target(state, cam)?;
    let part = part_at(cam, rect, &base, pos)?;
    let center = base.center();
    let plane = drag_plane(cam, part, center);
    let ray = cam.ray(rect, pos);
    let start = ray.at(ray.intersect_plane(&plane)?);
    state.doc.begin(part.label());
    Some(GizmoDrag { part, center, base, plane, start })
}

/// Applies the drag for the pointer at `pos`. Returns a status line.
pub fn drag(state: &mut EditorState, drag: &GizmoDrag, cam: &Camera, rect: Rect, pos: Pos2, modifiers: egui::Modifiers) -> Option<String> {
    let ray = cam.ray(rect, pos);
    let point = ray.at(ray.intersect_plane(&drag.plane)?);
    let opts = state.opts();
    state.doc.reset_transaction();
    let offset = point - drag.start;
    let moved = match drag.part {
        Part::Move(i) => Some(axis(i) * offset[i]),
        Part::Plane(i) => Some(offset - axis(i) * offset[i]),
        Part::Free => Some(offset),
        _ => None,
    };
    if let Some(delta) = moved {
        let delta = state.snap(delta);
        if delta != DVec3::ZERO {
            state.doc.edit("Move", |m, s| ops::translate_selection(m, s, delta, opts));
        }
        return Some(format!("Move {} {} {}", delta.x, delta.y, delta.z));
    }
    match drag.part {
        Part::Move(_) | Part::Plane(_) | Part::Free => None,
        Part::Rotate(i) => {
            let (from, to) = ((drag.start - drag.center).normalize_or_zero(), (point - drag.center).normalize_or_zero());
            let raw = from.cross(to).dot(axis(i)).atan2(from.dot(to)).to_degrees();
            let step = if modifiers.shift { 1.0 } else { ROTATE_SNAP_DEGREES };
            let angle = (raw / step).round() * step;
            if angle != 0.0 {
                let m = ops::rotation_about(drag.center, axis(i), angle);
                state.doc.edit("Rotate", |map, s| ops::transform_selection(map, s, &m, opts));
            }
            Some(format!("Rotate {} {angle}°", axis_name(i)))
        }
        Part::Scale(i) => {
            let reach = (drag.start - drag.center)[i];
            let size = drag.base.size()[i];
            if reach.abs() < 1e-6 || size < 1e-6 {
                return None;
            }
            let grow = (size * (point - drag.center)[i] / reach - size) * 0.5;
            let mut scaled = drag.base;
            scaled.min[i] = state.snap_scalar(drag.base.min[i] - grow);
            scaled.max[i] = state.snap_scalar(drag.base.max[i] + grow);
            let new_size = scaled.size()[i];
            if new_size < 1e-3 {
                return None;
            }
            if scaled != drag.base {
                let m = ops::scale_bounds(&drag.base, &scaled);
                state.doc.edit("Scale", |map, s| ops::transform_selection(map, s, &m, opts));
            }
            Some(format!("Scale {} to {new_size}", axis_name(i)))
        }
    }
}

pub fn paint(ui: &Ui, cam: &Camera, rect: Rect, state: &EditorState, active: Option<Part>) {
    let Some(bounds) = target(state, cam) else { return };
    let hovered = active.or_else(|| ui.input(|i| i.pointer.hover_pos()).filter(|p| rect.contains(*p)).and_then(|p| part_at(cam, rect, &bounds, p)));
    if hovered.is_some() {
        ui.ctx().set_cursor_icon(if active.is_some() { CursorIcon::Grabbing } else { CursorIcon::Grab });
    }
    let l = Layout::new(cam, &bounds);
    let painter = ui.painter_at(rect);
    let color = |part: Part, base: Color32| if hovered == Some(part) { HOT } else { base };

    for (i, axis_color) in AXIS_COLORS.into_iter().enumerate() {
        if let Some(points) = project_all(cam, rect, &l.ring(i)) {
            let hot = hovered == Some(Part::Rotate(i));
            painter.add(Shape::line(points, Stroke::new(if hot { 3.0 } else { 1.5 }, color(Part::Rotate(i), axis_color.gamma_multiply(0.8)))));
        }
    }
    for (i, axis_color) in AXIS_COLORS.into_iter().enumerate() {
        if let Some(quad) = project_all(cam, rect, &l.plane_quad(i)) {
            let c = color(Part::Plane(i), axis_color);
            painter.add(Shape::convex_polygon(quad, c.gamma_multiply(if hovered == Some(Part::Plane(i)) { 0.55 } else { 0.3 }), Stroke::new(1.0, c)));
        }
    }
    for (i, axis_color) in AXIS_COLORS.into_iter().enumerate() {
        let c = color(Part::Move(i), axis_color);
        let (Some(from), Some(head), Some(tip)) =
            (cam.project(rect, l.along(i, SHAFT_START)), cam.project(rect, l.along(i, HEAD_START)), cam.project(rect, l.along(i, 1.0)))
        else {
            continue;
        };
        painter.line_segment([from, head], Stroke::new(if hovered == Some(Part::Move(i)) { 3.5 } else { 2.5 }, c));
        let dir = tip - head;
        if dir.length() > 1.0 {
            let side = Vec2::new(-dir.y, dir.x).normalized() * 6.0;
            painter.add(Shape::convex_polygon(vec![tip, head + side, head - side], c, Stroke::NONE));
        }
        if let Some(handle) = cam.project(rect, l.along(i, SCALE_AT)) {
            painter.rect_filled(Rect::from_center_size(handle, Vec2::splat(9.0)), 1.0, color(Part::Scale(i), axis_color));
        }
    }
    if let Some(center) = cam.project(rect, l.center) {
        let hot = hovered == Some(Part::Free);
        painter.circle(
            center,
            5.5,
            if hot { HOT.gamma_multiply(0.6) } else { Color32::from_black_alpha(90) },
            Stroke::new(1.5, if hot { HOT } else { Color32::WHITE }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_core::NodeId;
    use gt_doc::NodeKind;
    use gt_geom::Brush;

    fn scene() -> (EditorState, Camera, Rect, NodeId) {
        let mut state = EditorState::new(crate::state::Prefs::default());
        let layer = state.doc.map.default_layer();
        let brush = Brush::from_aabb(&Aabb::new(DVec3::new(-32.0, 0.0, -32.0), DVec3::new(32.0, 64.0, 32.0)), "dev/grey").unwrap();
        let id = state.doc.edit("add", |m, s| {
            let id = m.insert(layer, NodeKind::Brush(brush));
            s.select_node(id);
            id
        });
        let mut cam = Camera::new(ViewKind::Perspective);
        cam.position = DVec3::new(300.0, 250.0, 300.0);
        cam.look_at(DVec3::new(0.0, 32.0, 0.0));
        (state, cam, Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 800.0)), id)
    }

    fn screen(cam: &Camera, rect: Rect, state: &EditorState, i: usize, fraction: f64) -> Pos2 {
        let l = Layout::new(cam, &state.doc.map.bounds(state.doc.selection.nodes.iter().copied().next().unwrap()));
        cam.project(rect, l.along(i, fraction)).unwrap()
    }

    #[test]
    fn handles_are_found_under_the_pointer() {
        let (state, cam, rect, _) = scene();
        let center = cam.project(rect, DVec3::new(0.0, 32.0, 0.0)).unwrap();
        assert_eq!(hit(&state, &cam, rect, center), Some(Part::Free));
        for i in 0..3 {
            assert_eq!(hit(&state, &cam, rect, screen(&cam, rect, &state, i, 0.55)), Some(Part::Move(i)), "arrow {i}");
            assert_eq!(hit(&state, &cam, rect, screen(&cam, rect, &state, i, SCALE_AT)), Some(Part::Scale(i)), "scale box {i}");
        }
        let mut hidden = state;
        hidden.tool = ToolKind::Clip;
        assert_eq!(hit(&hidden, &cam, rect, center), None, "only the select tool shows the gizmo");
    }

    #[test]
    fn dragging_handles_moves_rotates_and_scales_the_selection() {
        let (mut state, cam, rect, id) = scene();
        let start = screen(&cam, rect, &state, 0, 0.55);
        let drag_x = begin(&mut state, &cam, rect, start).expect("grabs the x arrow");
        assert_eq!(drag_x.part, Part::Move(0));
        let target = cam.project(rect, drag_x.start + DVec3::new(35.0, 0.0, 0.0)).unwrap();
        drag(&mut state, &drag_x, &cam, rect, target, egui::Modifiers::NONE);
        state.doc.commit();
        let moved = state.doc.map.bounds(id);
        assert_eq!((moved.min.x, moved.min.y, moved.min.z), (0.0, 0.0, -32.0), "moves along x only, snapped to the 16 grid");

        let before = state.doc.map.bounds(id);
        let scale_box = screen(&cam, rect, &state, 1, SCALE_AT);
        let drag_scale = begin(&mut state, &cam, rect, scale_box).expect("grabs the y scale box");
        let reach = drag_scale.start - drag_scale.center;
        let target = cam.project(rect, drag_scale.center + reach * 2.0).unwrap();
        drag(&mut state, &drag_scale, &cam, rect, target, egui::Modifiers::NONE);
        state.doc.commit();
        let scaled = state.doc.map.bounds(id);
        assert!((scaled.size().y - before.size().y * 2.0).abs() < 16.0 + 1e-6, "roughly doubles the height, got {}", scaled.size().y);
        assert!((scaled.center().y - before.center().y).abs() < 8.0 + 1e-6, "scales around the center");
        assert_eq!(scaled.size().x, before.size().x);

        let mut state2 = scene().0;
        let l = Layout::new(&cam, &state2.doc.map.bounds_of(state2.doc.selection.nodes.iter().copied()));
        let ring = l.ring(1);
        let grab = cam.project(rect, ring[8]).unwrap();
        let drag_rot = begin(&mut state2, &cam, rect, grab).expect("grabs the y ring");
        assert_eq!(drag_rot.part, Part::Rotate(1));
        let status = drag(&mut state2, &drag_rot, &cam, rect, cam.project(rect, ring[24]).unwrap(), egui::Modifiers::NONE);
        assert_eq!(status.as_deref(), Some("Rotate Y 90°"));
    }
}
