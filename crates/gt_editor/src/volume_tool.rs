//! Volume tool: drag out gameplay volumes (triggers, spawn areas, hurt, teleport and push zones) as brush entities.

use egui::{Align2, Color32, FontId, PointerButton, Rect, Response, Ui, Vec2};
use gt_core::{Aabb, DVec3, Plane};
use gt_render::LineVertex;

use crate::camera::Camera;
use crate::picking;
use crate::scene::v3;
use crate::state::EditorState;

/// Brush entity classes offered by the tool, in menu order.
pub const VOLUME_CLASSES: [&str; 8] =
    ["trigger_once", "trigger_multiple", "trigger_call", "trigger_spawn_area", "trigger_hurt", "trigger_teleport", "trigger_push", "trigger_area"];

#[derive(Default)]
pub struct VolumeTool {
    start: Option<(DVec3, Plane)>,
    current: Option<DVec3>,
}

impl VolumeTool {
    pub fn reset(&mut self) {
        self.start = None;
        self.current = None;
    }

    fn preview(&self, state: &EditorState, cam: &Camera) -> Option<Aabb> {
        let (a, _) = self.start?;
        let b = self.current?;
        let grid = state.grid.max(1.0);
        let mut min = state.snap(a.min(b));
        let mut max = state.snap(a.max(b));
        if cam.kind.is_2d() {
            let depth = cam.kind.depth_axis();
            let lb = state.last_bounds;
            let (lo, hi) = if lb.is_empty() || lb.size()[depth] < 1e-6 { (0.0, state.prefs.volume_height) } else { (lb.min[depth], lb.max[depth]) };
            min[depth] = lo;
            max[depth] = hi;
        } else {
            max.y = min.y + state.prefs.volume_height;
        }

        for i in 0..3 {
            if max[i] - min[i] < grid {
                max[i] = min[i] + grid;
            }
        }

        Some(Aabb::new(min, max))
    }

    pub fn input(&mut self, ui: &Ui, response: &Response, cam: &Camera, rect: Rect, state: &mut EditorState) {
        let pointer = ui.input(|i| i.pointer.interact_pos());
        let point_on = |pos: egui::Pos2, plane: &Plane| {
            let ray = cam.ray(rect, pos);
            ray.intersect_plane(plane).map(|t| ray.at(t))
        };
        if response.drag_started_by(PointerButton::Primary)
            && let Some(origin) = ui.input(|i| i.pointer.press_origin())
        {
            let ray = cam.ray(rect, origin);
            let plane = if cam.kind.is_2d() {
                Plane::from_point_normal(cam.screen_to_plane(rect, origin), -cam.kind.axes().2)
            } else {
                let y = picking::pick(state, &ray).map(|h| h.point.y).unwrap_or(0.0);
                Plane::from_point_normal(DVec3::new(0.0, state.snap_scalar(y), 0.0), DVec3::Y)
            };
            if let Some(p) = point_on(origin, &plane) {
                self.start = Some((p, plane));
                self.current = Some(p);
            }
        }

        if let (Some((_, plane)), Some(pos)) = (self.start, pointer) {
            if let Some(p) = point_on(pos, &plane) {
                self.current = Some(p);
            }

            if !ui.input(|i| i.pointer.primary_down()) {
                if let Some(bounds) = self.preview(state, cam) {
                    let class = state.prefs.volume_class.clone();
                    let props = default_props(&class);
                    match crate::entity_wizards::make_volume(state, &class, &bounds, &props, Vec::new()) {
                        Ok(_) => {
                            state.set_status(format!("{class} created. Set its outputs in the inspector, or use Gameplay > Logic > Link Two Selected Entities"))
                        }
                        Err(e) => state.set_status(e),
                    }
                }

                self.reset();
            }
        }
    }

    pub fn lines(&self, state: &EditorState, cam: &Camera, out: &mut Vec<LineVertex>) {
        if let Some(b) = self.preview(state, cam) {
            let c = b.corners();
            for (i, j) in Aabb::EDGES {
                out.push(LineVertex { pos: v3(c[i]), color: [1.0, 0.65, 0.2, 1.0] });
                out.push(LineVertex { pos: v3(c[j]), color: [1.0, 0.65, 0.2, 1.0] });
            }
        }
    }

    pub fn paint_overlay(&self, ui: &Ui, rect: Rect, state: &EditorState) {
        let text = format!(
            "Volume: drag to draw a {} volume ({} units tall in 3D). Pick the class in the toolbar",
            state.prefs.volume_class, state.prefs.volume_height
        );
        ui.painter_at(rect).text(
            rect.left_bottom() + Vec2::new(8.0, -8.0),
            Align2::LEFT_BOTTOM,
            text,
            FontId::proportional(12.0),
            Color32::from_rgb(255, 190, 110),
        );
    }
}

/// Useful starting properties per volume class.
pub fn default_props(classname: &str) -> Vec<(String, String)> {
    let p = |k: &str, v: &str| (k.to_string(), v.to_string());
    match classname {
        "trigger_call" => vec![p("call_target", "/root/Game"), p("method", "on_area_entered"), p("arguments", "[\"$activator\"]")],
        "trigger_spawn_area" => vec![p("count", "3"), p("max_alive", "6")],
        "trigger_teleport" => vec![p("destination", "")],
        _ => Vec::new(),
    }
}
