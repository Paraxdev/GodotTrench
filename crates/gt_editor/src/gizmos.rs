//! Entity gizmos: draggable hinge, travel, radius, cone, box and point handles bound to entity properties, plus I/O
//! connection labels for the selected entities.

use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, Ui, Vec2};
use gt_core::{Aabb, DQuat, DVec3, NodeId, Plane};
use gt_formats::GizmoDef;
use gt_render::LineVertex;

use crate::camera::{Camera, ViewKind};
use crate::scene::v3;
use crate::state::EditorState;

const HANDLE_PICK: f32 = 9.0;
const MAX_GIZMO_ENTITIES: usize = 24;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GizmoHandle {
    pub entity: NodeId,
    pub gizmo: usize,
    /// Which handle of the gizmo: 0 is the main one, hinge angle, cone rim and box axes use 1 and up.
    pub part: u8,
}

pub fn parse_vec3(s: &str) -> Option<DVec3> {
    let v: Vec<f64> = s.split_whitespace().filter_map(|p| p.parse().ok()).collect();
    (v.len() >= 3).then(|| DVec3::new(v[0], v[1], v[2]))
}

fn fmt_num(v: f64) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    if r == r.round() { format!("{}", r as i64) } else { format!("{r}") }
}

pub fn fmt_vec3(v: DVec3) -> String {
    format!("{} {} {}", fmt_num(v.x), fmt_num(v.y), fmt_num(v.z))
}

fn line(out: &mut Vec<LineVertex>, a: DVec3, b: DVec3, color: [f32; 4]) {
    out.push(LineVertex { pos: v3(a), color });
    out.push(LineVertex { pos: v3(b), color });
}

fn axis_vector(name: &str) -> DVec3 {
    match name.trim().to_ascii_lowercase().as_str() {
        "x" => DVec3::X,
        "z" => DVec3::Z,
        _ => DVec3::Y,
    }
}

/// Everything a gizmo needs to know about its entity.
struct Target {
    id: NodeId,
    entity: gt_doc::Entity,
    center: DVec3,
    bounds: Aabb,
    gizmos: Vec<GizmoDef>,
}

impl Target {
    fn prop(&self, name: &str) -> Option<String> {
        self.entity.property(name).map(str::to_string)
    }

    /// Property value, falling back to the definition default.
    fn value(&self, state: &EditorState, name: &str) -> String {
        self.prop(name)
            .unwrap_or_else(|| state.game.entity(&self.entity.classname).and_then(|d| d.property(name)).map(|p| p.default.clone()).unwrap_or_default())
    }

    fn number(&self, state: &EditorState, name: &str) -> f64 {
        self.value(state, name).trim().parse().unwrap_or(0.0)
    }

    fn vector(&self, state: &EditorState, name: &str) -> DVec3 {
        parse_vec3(&self.value(state, name)).unwrap_or(DVec3::ZERO)
    }

    fn forward(&self) -> DVec3 {
        self.entity.rotation() * DVec3::NEG_Z
    }
}

fn targets(state: &EditorState) -> Vec<Target> {
    let map = &state.doc.map;
    let mut out = Vec::new();
    for id in state.doc.selection.nodes.iter().copied() {
        if out.len() >= MAX_GIZMO_ENTITIES {
            break;
        }
        let Some(entity) = map.entity(id) else { continue };
        let Some(def) = state.game.entity(&entity.classname) else { continue };
        let gizmos = def.gizmos(state.game.units_per_meter);
        if gizmos.is_empty() {
            continue;
        }
        let (center, bounds) = if map.is_point_entity(id) {
            let b = state.model_bounds.get(&id).copied().unwrap_or_else(|| crate::scene::entity_box(&state.game, entity));
            (entity.origin, b)
        } else {
            let b = map.bounds(id);
            if b.is_empty() {
                continue;
            }
            (b.center(), b)
        };
        out.push(Target { id, entity: entity.clone(), center, bounds, gizmos });
    }
    out
}

/// Point of the leaf farthest from the hinge, rotated to the open angle around the hinge axis.
fn hinge_geometry(t: &Target, state: &EditorState, gizmo: &GizmoDef) -> Option<(DVec3, DVec3, DVec3, f64)> {
    let GizmoDef::Hinge { property, angle, axis } = gizmo else { return None };
    let hinge = t.center + t.vector(state, property);
    let axis = if axis.is_empty() { DVec3::Y } else { axis_vector(&t.value(state, axis)) };
    let open = if angle.is_empty() { 90.0 } else { t.number(state, angle) };
    let far = t
        .bounds
        .corners()
        .into_iter()
        .map(|c| {
            let d = c - hinge;
            let flat = d - axis * d.dot(axis);
            (flat.length(), hinge + flat)
        })
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, p)| p)?;
    let far = if (far - hinge).length() < 1e-6 { hinge + DVec3::X * 16.0 } else { far };
    Some((hinge, axis, far, open))
}

fn rotated_bounds(bounds: &Aabb, pivot: DVec3, rot: DQuat) -> [DVec3; 8] {
    bounds.corners().map(|c| pivot + rot * (c - pivot))
}

fn box_lines(out: &mut Vec<LineVertex>, corners: &[DVec3; 8], color: [f32; 4]) {
    for (i, j) in Aabb::EDGES {
        line(out, corners[i], corners[j], color);
    }
}

fn circle(out: &mut Vec<LineVertex>, center: DVec3, a: DVec3, b: DVec3, radius: f64, color: [f32; 4]) {
    let pts: Vec<DVec3> = (0..=48)
        .map(|k| {
            let t = std::f64::consts::TAU * k as f64 / 48.0;
            center + (a * t.cos() + b * t.sin()) * radius
        })
        .collect();
    for w in pts.windows(2) {
        line(out, w[0], w[1], color);
    }
}

/// Every handle of the selected entities with its world position and a short label.
pub fn handles(state: &EditorState) -> Vec<(GizmoHandle, DVec3, String)> {
    let mut out = Vec::new();
    for t in targets(state) {
        for (gi, g) in t.gizmos.iter().enumerate() {
            let h = |part: u8| GizmoHandle { entity: t.id, gizmo: gi, part };
            match g {
                GizmoDef::Hinge { property, angle, .. } => {
                    if let Some((hinge, axis, far, open)) = hinge_geometry(&t, state, g) {
                        out.push((h(0), hinge, property.clone()));
                        if !angle.is_empty() {
                            let tip = hinge + DQuat::from_axis_angle(axis, open.to_radians()) * (far - hinge);
                            out.push((h(1), tip, format!("{}°", fmt_num(open))));
                        }
                    }
                }
                GizmoDef::Travel { property } => {
                    let v = t.vector(state, property);
                    out.push((h(0), t.center + v, format!("{property} {}", fmt_vec3(v))));
                }
                GizmoDef::Radius { property, scale } => {
                    let r = t.number(state, property) * scale;
                    out.push((h(0), t.center + DVec3::X * r, format!("{property} {}", fmt_num(t.number(state, property)))));
                }
                GizmoDef::Cone { angle, range, scale } => {
                    let fwd = t.forward();
                    let len = t.number(state, range) * scale;
                    let half = t.number(state, angle).clamp(0.0, 89.0).to_radians();
                    let side = (t.entity.rotation() * DVec3::X).normalize();
                    out.push((h(0), t.center + fwd * len, format!("{range} {}", fmt_num(t.number(state, range)))));
                    out.push((h(1), t.center + fwd * len + side * len * half.tan(), format!("{}°", fmt_num(t.number(state, angle)))));
                }
                GizmoDef::Box { property } => {
                    let s = t.vector(state, property);
                    for (k, axis) in [DVec3::X, DVec3::Y, DVec3::Z].into_iter().enumerate() {
                        out.push((h(k as u8), t.center + axis * s[k] * 0.5, fmt_num(s[k])));
                    }
                }
                GizmoDef::Point { property, world } => {
                    let v = t.vector(state, property);
                    out.push((h(0), if *world { v } else { t.center + v }, property.clone()));
                }
            }
        }
    }
    out
}

/// Gizmo lines for the selected entities.
pub fn lines(state: &EditorState, out: &mut Vec<LineVertex>) {
    const HINGE: [f32; 4] = [1.0, 0.75, 0.3, 0.95];
    const GHOST: [f32; 4] = [0.4, 0.9, 1.0, 0.55];
    const RANGE: [f32; 4] = [1.0, 0.95, 0.4, 0.5];
    for t in targets(state) {
        for g in &t.gizmos {
            match g {
                GizmoDef::Hinge { .. } => {
                    let Some((hinge, axis, far, open)) = hinge_geometry(&t, state, g) else { continue };
                    let extent = t.bounds.size().dot(axis.abs()) * 0.5 + 8.0;
                    let mid = hinge + axis * (t.bounds.center() - hinge).dot(axis);
                    line(out, mid - axis * extent, mid + axis * extent, HINGE);
                    let steps = 24;
                    let pts: Vec<DVec3> =
                        (0..=steps).map(|k| hinge + DQuat::from_axis_angle(axis, (open * k as f64 / steps as f64).to_radians()) * (far - hinge)).collect();
                    for w in pts.windows(2) {
                        line(out, w[0], w[1], [HINGE[0], HINGE[1], HINGE[2], 0.6]);
                    }
                    line(out, hinge, far, [HINGE[0], HINGE[1], HINGE[2], 0.5]);
                    line(out, hinge, *pts.last().unwrap_or(&far), HINGE);
                    box_lines(out, &rotated_bounds(&t.bounds, hinge, DQuat::from_axis_angle(axis, open.to_radians())), GHOST);
                }
                GizmoDef::Travel { property } => {
                    let v = t.vector(state, property);
                    if v.length() < 1e-6 {
                        continue;
                    }
                    let end = t.center + v;
                    line(out, t.center, end, GHOST);
                    let dir = v.normalize();
                    let side = if dir.y.abs() > 0.9 { DVec3::X } else { DVec3::Y.cross(dir).normalize() };
                    let head = (v.length() * 0.15).clamp(4.0, 24.0);
                    line(out, end, end - dir * head + side * head * 0.5, GHOST);
                    line(out, end, end - dir * head - side * head * 0.5, GHOST);
                    let moved = t.bounds.translated(v);
                    box_lines(out, &moved.corners(), GHOST);
                }
                GizmoDef::Radius { property, scale } => {
                    let r = t.number(state, property) * scale;
                    if r <= 0.0 {
                        continue;
                    }
                    circle(out, t.center, DVec3::X, DVec3::Z, r, RANGE);
                    circle(out, t.center, DVec3::X, DVec3::Y, r, [RANGE[0], RANGE[1], RANGE[2], 0.25]);
                    circle(out, t.center, DVec3::Z, DVec3::Y, r, [RANGE[0], RANGE[1], RANGE[2], 0.25]);
                }
                GizmoDef::Cone { angle, range, scale } => {
                    let fwd = t.forward();
                    let len = t.number(state, range) * scale;
                    let half = t.number(state, angle).clamp(0.0, 89.0).to_radians();
                    let rot = t.entity.rotation();
                    let (a, b) = ((rot * DVec3::X).normalize(), (rot * DVec3::Y).normalize());
                    let end = t.center + fwd * len;
                    let rim = len * half.tan();
                    circle(out, end, a, b, rim, RANGE);
                    for k in 0..8 {
                        let th = std::f64::consts::TAU * k as f64 / 8.0;
                        line(out, t.center, end + (a * th.cos() + b * th.sin()) * rim, [RANGE[0], RANGE[1], RANGE[2], 0.35]);
                    }
                }
                GizmoDef::Box { property } => {
                    let s = t.vector(state, property);
                    box_lines(out, &Aabb::from_center_size(t.center, s.abs()).corners(), GHOST);
                }
                GizmoDef::Point { property, world } => {
                    let v = t.vector(state, property);
                    let p = if *world { v } else { t.center + v };
                    line(out, t.center, p, GHOST);
                    for axis in [DVec3::X, DVec3::Y, DVec3::Z] {
                        line(out, p - axis * 6.0, p + axis * 6.0, GHOST);
                    }
                }
            }
        }
    }
}

/// Handle under the cursor, if any.
pub fn handle_at(state: &EditorState, cam: &Camera, rect: Rect, pos: Pos2) -> Option<GizmoHandle> {
    handles(state)
        .into_iter()
        .filter_map(|(h, p, _)| cam.project(rect, p).map(|sp| ((sp - pos).length(), h)))
        .filter(|(d, _)| *d <= HANDLE_PICK)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, h)| h)
}

fn set_property(state: &mut EditorState, id: NodeId, key: &str, value: String) {
    state.doc.reset_transaction();
    let key = key.to_string();
    state.doc.edit(&format!("Set {key}"), |m, _| {
        if let Some(e) = m.entity_mut(id) {
            e.properties.insert(key, value);
        }
    });
}

/// Moves a handle to the pointer. Returns a status line describing the new value.
pub fn drag(state: &mut EditorState, handle: GizmoHandle, start: DVec3, cam: &Camera, rect: Rect, pos: Pos2, modifiers: egui::Modifiers) -> Option<String> {
    let t = targets(state).into_iter().find(|t| t.id == handle.entity)?;
    let gizmo = t.gizmos.get(handle.gizmo)?.clone();
    let ray = cam.ray(rect, pos);
    let plane = match cam.kind {
        ViewKind::Perspective if modifiers.alt => {
            let f = cam.forward();
            Plane::from_point_normal(start, DVec3::new(f.x, 0.0, f.z).normalize_or(DVec3::Z))
        }
        ViewKind::Perspective if matches!(gizmo, GizmoDef::Hinge { .. } | GizmoDef::Travel { .. } | GizmoDef::Point { .. }) => {
            Plane::from_point_normal(start, DVec3::Y)
        }
        ViewKind::Perspective => Plane::from_point_normal(start, -cam.forward()),
        k => Plane::from_point_normal(start, -k.axes().2),
    };
    let world = ray.intersect_plane(&plane).map(|d| ray.at(d))?;
    let angle_step = if modifiers.shift { 1.0 } else { 15.0 };
    let (key, value, status) = match (&gizmo, handle.part) {
        (GizmoDef::Hinge { property, .. }, 0) => {
            let v = state.snap(world) - t.center;
            (property.clone(), fmt_vec3(v), format!("hinge {}", fmt_vec3(v)))
        }
        (GizmoDef::Hinge { angle, .. }, _) => {
            let (hinge, axis, far, _) = hinge_geometry(&t, state, &gizmo)?;
            let (a, b) = (far - hinge, world - hinge);
            let (a, b) = (a - axis * a.dot(axis), b - axis * b.dot(axis));
            let deg = a.cross(b).dot(axis).atan2(a.dot(b)).to_degrees();
            let deg = (deg / angle_step).round() * angle_step;
            (angle.clone(), fmt_num(deg), format!("open angle {}°", fmt_num(deg)))
        }
        (GizmoDef::Travel { property }, _) => {
            let v = state.snap(world - t.center);
            (property.clone(), fmt_vec3(v), format!("{property} {} ({} units)", fmt_vec3(v), fmt_num(v.length())))
        }
        (GizmoDef::Radius { property, scale }, _) => {
            let r = state.snap_scalar((world - t.center).length()).max(0.0) / scale.max(1e-9);
            (property.clone(), fmt_num(r), format!("{property} {}", fmt_num(r)))
        }
        (GizmoDef::Cone { range, scale, .. }, 0) => {
            let len = state.snap_scalar((world - t.center).dot(t.forward()).max(0.0)) / scale.max(1e-9);
            (range.clone(), fmt_num(len), format!("{range} {}", fmt_num(len)))
        }
        (GizmoDef::Cone { angle, .. }, _) => {
            let d = world - t.center;
            let along = d.dot(t.forward()).max(1e-6);
            let side = (d - t.forward() * d.dot(t.forward())).length();
            let deg = ((side / along).atan().to_degrees() / angle_step).round() * angle_step;
            (angle.clone(), fmt_num(deg.clamp(1.0, 89.0)), format!("{angle} {}°", fmt_num(deg)))
        }
        (GizmoDef::Box { property }, k) => {
            let mut s = t.vector(state, property);
            let axis = (k as usize).min(2);
            s[axis] = state.snap_scalar(((world - t.center)[axis]).abs() * 2.0).max(1.0);
            (property.clone(), fmt_vec3(s), format!("{property} {}", fmt_vec3(s)))
        }
        (GizmoDef::Point { property, world: absolute }, _) => {
            let p = state.snap(world);
            let v = if *absolute { p } else { p - t.center };
            (property.clone(), fmt_vec3(v), format!("{property} {}", fmt_vec3(v)))
        }
    };
    set_property(state, t.id, &key, value);
    Some(status)
}

/// World position of a handle, used as the drag anchor.
pub fn handle_position(state: &EditorState, handle: GizmoHandle) -> Option<DVec3> {
    handles(state).into_iter().find(|(h, _, _)| *h == handle).map(|(_, p, _)| p)
}

/// Handle squares, labels and I/O connection labels.
pub fn paint_overlay(ui: &Ui, cam: &Camera, rect: Rect, state: &EditorState, active: Option<GizmoHandle>) {
    let painter = ui.painter_at(rect);
    let hover = ui.input(|i| i.pointer.hover_pos());
    for (h, p, label) in handles(state) {
        let Some(sp) = cam.project(rect, p) else { continue };
        let hot = active == Some(h) || hover.is_some_and(|m| (m - sp).length() <= HANDLE_PICK);
        let fill = if hot { Color32::from_rgb(255, 255, 120) } else { Color32::from_rgb(255, 170, 60) };
        let r = if hot { 6.0 } else { 4.5 };
        painter.circle(sp, r, fill, Stroke::new(1.0, Color32::BLACK));
        if hot || active.is_none() {
            painter.text(sp + Vec2::new(8.0, -4.0), Align2::LEFT_BOTTOM, label, FontId::proportional(11.0), Color32::from_rgb(255, 225, 170));
        }
    }
    io_labels(&painter, cam, rect, state);
}

/// "output > input" labels on the connection lines of selected entities.
fn io_labels(painter: &egui::Painter, cam: &Camera, rect: Rect, state: &EditorState) {
    let map = &state.doc.map;
    let center = |id: NodeId| -> Option<DVec3> {
        let e = map.entity(id)?;
        Some(if map.is_point_entity(id) { e.origin } else { map.bounds(id).center() })
    };
    let mut shown = 0;
    for id in state.doc.selection.nodes.iter().copied() {
        let Some(e) = map.entity(id) else { continue };
        let Some(from) = center(id) else { continue };
        for o in &e.outputs {
            if shown > 40 {
                return;
            }
            let delay = if o.delay > 0.0 { format!(" +{}s", fmt_num(o.delay)) } else { String::new() };
            let param = if o.parameter.is_empty() { String::new() } else { format!("({})", o.parameter) };
            let text = format!("{} > {}.{}{param}{delay}", o.output, o.target, o.input);
            let targets = map.find_by_targetname(&o.target);
            let anchor = match targets.first().and_then(|t| center(*t)) {
                Some(to) => from.lerp(to, 0.5),
                None => from + DVec3::Y * (24.0 + 14.0 * shown as f64),
            };
            if let Some(sp) = cam.project(rect, anchor) {
                let color = if targets.is_empty() && !gt_doc::issues::is_dynamic_target(&o.target) {
                    Color32::from_rgb(255, 110, 90)
                } else {
                    Color32::from_rgb(255, 190, 110)
                };
                painter.text(sp + Vec2::new(0.0, 12.0 * shown as f32 % 36.0), Align2::CENTER_CENTER, text, FontId::proportional(11.0), color);
                shown += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_doc::NodeKind;
    use gt_geom::Brush;

    fn door_state() -> (EditorState, NodeId) {
        let mut state = EditorState::new(Default::default());
        let layer = state.doc.map.default_layer();
        let id = state.doc.edit("door", |m, s| {
            let mut e = gt_doc::Entity::new("func_door_rotating");
            e.properties.insert("hinge".into(), "-32 0 0".into());
            e.properties.insert("open_angle".into(), "90".into());
            let id = m.insert(layer, NodeKind::Entity(e));
            m.insert(id, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::new(-32.0, 0.0, -2.0), DVec3::new(32.0, 96.0, 2.0)), "wood").unwrap()));
            s.select_node(id);
            id
        });
        (state, id)
    }

    #[test]
    fn hinge_handles_follow_properties() {
        let (state, id) = door_state();
        let hs = handles(&state);
        let hinge = hs.iter().find(|(h, _, _)| h.entity == id && h.part == 0).unwrap();
        assert!((hinge.1 - DVec3::new(-32.0, 48.0, 0.0)).length() < 1e-9, "hinge at the leaf edge: {:?}", hinge.1);
        let tip = hs.iter().find(|(h, _, _)| h.part == 1).unwrap();
        // The far edge at x = 32 swings 90 degrees around the hinge.
        assert!((DVec3::new(tip.1.x, 0.0, tip.1.z) - DVec3::new(-32.0, 0.0, -64.0)).length() < 3.0, "open tip {:?}", tip.1);
        let mut lines_out = Vec::new();
        lines(&state, &mut lines_out);
        assert!(lines_out.len() > 40);
    }

    #[test]
    fn value_formatting() {
        assert_eq!(fmt_vec3(DVec3::new(1.0, -0.5, 16.25)), "1 -0.5 16.25");
        assert_eq!(parse_vec3("0 128 -4"), Some(DVec3::new(0.0, 128.0, -4.0)));
    }
}
