//! Path and measure tools.

use egui::{Align2, Color32, FontId, PointerButton, Pos2, Rect, Response, Ui, Vec2};
use gt_core::{DVec2, DVec3, NodeId};
use gt_doc::{Entity, NodeKind};
use gt_render::LineVertex;

use crate::camera::Camera;
use crate::picking;
use crate::scene::v3;
use crate::state::EditorState;

fn line(out: &mut Vec<LineVertex>, a: DVec3, b: DVec3, color: [f32; 4]) {
    out.push(LineVertex { pos: v3(a), color });
    out.push(LineVertex { pos: v3(b), color });
}

#[derive(Default)]
pub struct PathTool {
    /// Last corner of the chain being drawn.
    pub last: Option<NodeId>,
    pub classname: String,
}

impl PathTool {
    pub fn input(&mut self, response: &Response, cam: &Camera, rect: Rect, state: &mut EditorState) {
        if !response.clicked_by(PointerButton::Primary) {
            return;
        }

        let Some(pos) = response.interact_pointer_pos() else { return };
        let ray = cam.ray(rect, pos);
        let point = match picking::pick(state, &ray) {
            Some(h) => h.point + h.normal * state.grid.min(16.0),
            None if cam.kind.is_2d() => cam.screen_to_plane(rect, pos),
            None => ray.intersect_plane(&gt_core::Plane::new(DVec3::Y, 0.0)).filter(|t| *t > 0.0).map(|t| ray.at(t)).unwrap_or(ray.at(256.0)),
        };
        let point = state.snap(point);
        let classname = if self.classname.is_empty() { "path_corner".to_string() } else { self.classname.clone() };
        let taken: std::collections::BTreeSet<String> = state.doc.map.entities().filter_map(|(_, e)| e.targetname().map(str::to_string)).collect();
        let name = (1..).map(|i| format!("path{i}")).find(|n| !taken.contains(n)).unwrap_or_default();
        let parent = state.insert_parent();
        let previous = self.last.filter(|id| state.doc.map.entity(*id).is_some());
        let id = state.doc.edit("Place Path Corner", |m, s| {
            let mut e = Entity::new(classname);
            e.origin = point;
            e.properties.insert("targetname".into(), name.clone());
            let id = m.insert(parent, NodeKind::Entity(e));
            if let Some(prev) = previous
                && let Some(pe) = m.entity_mut(prev)
            {
                pe.properties.insert("target".into(), name.clone());
            }

            s.clear();
            s.select_node(id);
            id
        });
        self.last = Some(id);
        state.set_status(format!("Placed {name}. Click to continue the path, Enter or Esc to finish"));
    }

    pub fn finish(&mut self) {
        self.last = None;
    }
}

#[derive(Default)]
pub struct MeasureTool {
    pub start: Option<DVec3>,
    pub end: Option<DVec3>,
    dragging: bool,
}

impl MeasureTool {
    fn point(cam: &Camera, rect: Rect, pos: Pos2, state: &EditorState) -> DVec3 {
        let ray = cam.ray(rect, pos);
        let p = match picking::pick(state, &ray) {
            Some(h) => h.point,
            None if cam.kind.is_2d() => cam.screen_to_plane(rect, pos),
            None => ray.intersect_plane(&gt_core::Plane::new(DVec3::Y, 0.0)).filter(|t| *t > 0.0).map(|t| ray.at(t)).unwrap_or(ray.at(256.0)),
        };
        state.snap(p)
    }

    pub fn input(&mut self, ui: &Ui, response: &Response, cam: &Camera, rect: Rect, state: &EditorState) {
        if response.drag_started_by(PointerButton::Primary)
            && let Some(origin) = ui.input(|i| i.pointer.press_origin())
        {
            self.start = Some(Self::point(cam, rect, origin, state));
            self.dragging = true;
        }

        if self.dragging
            && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
        {
            self.end = Some(Self::point(cam, rect, pos, state));
            if !ui.input(|i| i.pointer.primary_down()) {
                self.dragging = false;
            }
        }

        if response.clicked_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
        {
            let p = Self::point(cam, rect, pos, state);
            match (self.start, self.end) {
                (Some(_), None) => self.end = Some(p),
                _ => {
                    self.start = Some(p);
                    self.end = None;
                }
            }
        }
    }

    pub fn lines(&self, out: &mut Vec<LineVertex>) {
        if let (Some(a), Some(b)) = (self.start, self.end) {
            line(out, a, b, [1.0, 1.0, 0.3, 1.0]);
            let corner = DVec3::new(b.x, a.y, b.z);
            line(out, a, corner, [1.0, 1.0, 0.3, 0.35]);
            line(out, corner, b, [1.0, 1.0, 0.3, 0.35]);
        }
    }

    pub fn paint(&self, ui: &Ui, cam: &Camera, rect: Rect, state: &EditorState) {
        let painter = ui.painter_at(rect);
        let upm = state.game.units_per_meter;
        let text = match (self.start, self.end) {
            (Some(a), Some(b)) => {
                let d = b - a;
                if let Some(mid) = cam.project(rect, (a + b) * 0.5) {
                    painter.text(
                        mid + Vec2::new(8.0, -8.0),
                        Align2::LEFT_BOTTOM,
                        format!("{:.2} u ({:.2} m)", d.length(), d.length() / upm),
                        FontId::monospace(13.0),
                        Color32::from_rgb(255, 255, 120),
                    );
                }

                format!(
                    "Measure: {:.2} units, {:.3} m   dx {:.2}  dy {:.2}  dz {:.2}   horizontal {:.2}",
                    d.length(),
                    d.length() / upm,
                    d.x,
                    d.y,
                    d.z,
                    DVec2::new(d.x, d.z).length()
                )
            }
            (Some(_), None) => "Measure: click the second point".into(),
            _ => "Measure: drag or click two points".into(),
        };
        painter.text(rect.left_bottom() + Vec2::new(8.0, -8.0), Align2::LEFT_BOTTOM, text, FontId::proportional(12.0), Color32::from_rgb(255, 255, 140));
    }
}
