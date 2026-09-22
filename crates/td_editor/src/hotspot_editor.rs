//! Window for drawing Hammer++ style hotspot rectangles on a texture, saved as `<texture>.hotspots.json`.

use egui::{Color32, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Vec2};

use crate::commands::{self, Action};
use crate::state::EditorState;

#[derive(Clone, Copy)]
enum HotDrag {
    Create { start: [f64; 2] },
    Move { index: usize, grab: [f64; 2], orig: [f64; 4] },
    Resize { index: usize, orig: [f64; 4] },
}

pub struct HotspotEditor {
    pub open: bool,
    pub material: String,
    pub rects: Vec<[f64; 4]>,
    pub selected: Option<usize>,
    loaded: Option<String>,
    drag: Option<HotDrag>,
    zoom: f32,
    snap: u32,
    split: [u32; 2],
    dirty: bool,
}

impl Default for HotspotEditor {
    fn default() -> Self {
        Self {
            open: false,
            material: String::new(),
            rects: Vec::new(),
            selected: None,
            loaded: None,
            drag: None,
            zoom: 4.0,
            snap: 8,
            split: [4, 4],
            dirty: false,
        }
    }
}

fn snap(v: f64, step: u32) -> f64 {
    let s = step.max(1) as f64;
    (v / s).round() * s
}

/// Rectangle from two corners, snapped and at least one snap step in size.
pub fn rect_from_corners(a: [f64; 2], b: [f64; 2], step: u32, size: [f64; 2]) -> [f64; 4] {
    let s = step.max(1) as f64;
    let x0 = snap(a[0].min(b[0]), step).clamp(0.0, size[0]);
    let y0 = snap(a[1].min(b[1]), step).clamp(0.0, size[1]);
    let x1 = snap(a[0].max(b[0]), step).clamp(0.0, size[0]).max(x0 + s);
    let y1 = snap(a[1].max(b[1]), step).clamp(0.0, size[1]).max(y0 + s);
    [x0, y0, x1 - x0, y1 - y0]
}

/// Splits the texture into an even grid of rectangles.
pub fn grid_rects(size: [f64; 2], nx: u32, ny: u32) -> Vec<[f64; 4]> {
    let (nx, ny) = (nx.max(1), ny.max(1));
    let (w, h) = (size[0] / nx as f64, size[1] / ny as f64);
    (0..ny).flat_map(|j| (0..nx).map(move |i| [i as f64 * w, j as f64 * h, w, h])).collect()
}

impl HotspotEditor {
    pub fn open_for(&mut self, material: &str) {
        self.open = true;
        if self.material != material {
            self.material = material.to_string();
            self.loaded = None;
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, state: &mut EditorState, actions: &mut Vec<Action>) {
        if !self.open {
            return;
        }

        if self.material.is_empty() {
            self.material = state.current_material.clone();
        }

        if self.loaded.as_deref() != Some(self.material.as_str()) {
            self.rects = commands::hotspot_rects(state, &self.material);
            self.loaded = Some(self.material.clone());
            self.selected = None;
            self.dirty = false;
        }

        let mut open = self.open;
        egui::Window::new("Hotspot Editor").open(&mut open).default_size([640.0, 560.0]).resizable(true).show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(&self.material).strong());
                if ui.button("Use current material").clicked() && self.material != state.current_material {
                    self.material = state.current_material.clone();
                    return;
                }

                ui.label("snap");
                egui::ComboBox::from_id_salt("hotspot_snap").selected_text(format!("{} px", self.snap)).width(60.0).show_ui(ui, |ui| {
                    for s in [1, 2, 4, 8, 16, 32, 64] {
                        ui.selectable_value(&mut self.snap, s, format!("{s} px"));
                    }
                });
                ui.add(egui::Slider::new(&mut self.zoom, 0.5..=16.0).logarithmic(true).text("zoom"));
            });
            let Some(tex) = state.materials.full_texture(ctx, &self.material) else {
                ui.label("This material has no texture image.");
                return;
            };
            let size = state.materials.pixel_size(&self.material).map(|s| [s[0] as f64, s[1] as f64]).unwrap_or([64.0, 64.0]);
            ui.horizontal_wrapped(|ui| {
                ui.add(egui::DragValue::new(&mut self.split[0]).range(1..=64).prefix("grid "));
                ui.add(egui::DragValue::new(&mut self.split[1]).range(1..=64).prefix("x "));
                if ui.button("Split into grid").clicked() {
                    self.rects = grid_rects(size, self.split[0], self.split[1]);
                    self.dirty = true;
                }

                if ui.add_enabled(self.selected.is_some(), egui::Button::new("Delete")).clicked()
                    && let Some(i) = self.selected.take()
                {
                    self.rects.remove(i);
                    self.dirty = true;
                }

                if ui.button("Clear").clicked() {
                    self.rects.clear();
                    self.selected = None;
                    self.dirty = true;
                }

                let save = ui.add_enabled(self.dirty, egui::Button::new("Save"));
                if save.clicked() {
                    match commands::write_hotspots(state, &self.material, &self.rects) {
                        Ok(path) => {
                            self.dirty = false;
                            state.set_status(format!("Saved {}", path.display()));
                        }
                        Err(e) => state.set_status(e),
                    }
                }

                if ui.button("Apply to selection").on_hover_text("Saves, then fits the selected faces to their best rectangle").clicked() {
                    if self.dirty && commands::write_hotspots(state, &self.material, &self.rects).is_ok() {
                        self.dirty = false;
                    }

                    actions.push(Action::HotspotTexture);
                }

                ui.label(RichText::new(format!("{} rectangles{}", self.rects.len(), if self.dirty { ", unsaved" } else { "" })).weak());
            });
            ui.label(
                RichText::new("Drag on empty texture to draw, drag a rectangle to move it, drag its corner handle to resize, right click deletes.").weak(),
            );
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                let zoom = self.zoom as f64;
                let (rect, response) = ui.allocate_exact_size(Vec2::new((size[0] * zoom) as f32, (size[1] * zoom) as f32), Sense::click_and_drag());
                let painter = ui.painter_at(rect);
                painter.image(tex.id(), rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
                let to_px = |p: Pos2| [((p.x - rect.min.x) as f64 / zoom).clamp(0.0, size[0]), ((p.y - rect.min.y) as f64 / zoom).clamp(0.0, size[1])];
                let to_screen = |r: &[f64; 4]| {
                    Rect::from_min_size(rect.min + Vec2::new((r[0] * zoom) as f32, (r[1] * zoom) as f32), Vec2::new((r[2] * zoom) as f32, (r[3] * zoom) as f32))
                };
                if self.snap as f64 * zoom >= 6.0 {
                    let step = self.snap as f64;
                    let grid = Stroke::new(1.0, Color32::from_black_alpha(50));
                    let mut x = step;
                    while x < size[0] {
                        painter.vline(rect.min.x + (x * zoom) as f32, rect.y_range(), grid);
                        x += step;
                    }

                    let mut y = step;
                    while y < size[1] {
                        painter.hline(rect.x_range(), rect.min.y + (y * zoom) as f32, grid);
                        y += step;
                    }
                }

                let pointer = response.interact_pointer_pos().or(response.hover_pos());
                let hit = |p: Pos2, rects: &[[f64; 4]]| rects.iter().rposition(|r| to_screen(r).contains(p));
                let handle_of = |r: &[f64; 4]| to_screen(r).max;

                if response.drag_started()
                    && let Some(p) = ui.input(|i| i.pointer.press_origin())
                {
                    let px = to_px(p);
                    self.drag = if let Some(i) = self.rects.iter().position(|r| (handle_of(r) - p).length() < 7.0) {
                        Some(HotDrag::Resize { index: i, orig: self.rects[i] })
                    } else if let Some(i) = hit(p, &self.rects) {
                        Some(HotDrag::Move { index: i, grab: px, orig: self.rects[i] })
                    } else {
                        Some(HotDrag::Create { start: px })
                    };
                }

                if let (Some(drag), Some(p)) = (self.drag, pointer) {
                    let px = to_px(p);
                    match drag {
                        HotDrag::Create { start } => {
                            painter.rect_stroke(
                                to_screen(&rect_from_corners(start, px, self.snap, size)),
                                0.0,
                                Stroke::new(2.0, Color32::YELLOW),
                                StrokeKind::Inside,
                            );
                            if response.drag_stopped() {
                                self.rects.push(rect_from_corners(start, px, self.snap, size));
                                self.selected = Some(self.rects.len() - 1);
                                self.dirty = true;
                                self.drag = None;
                            }
                        }
                        HotDrag::Move { index, grab, orig } => {
                            let dx = snap(px[0] - grab[0], self.snap);
                            let dy = snap(px[1] - grab[1], self.snap);
                            if let Some(r) = self.rects.get_mut(index) {
                                *r = [(orig[0] + dx).clamp(0.0, size[0] - orig[2]), (orig[1] + dy).clamp(0.0, size[1] - orig[3]), orig[2], orig[3]];
                            }

                            self.selected = Some(index);
                            self.dirty = true;
                        }
                        HotDrag::Resize { index, orig } => {
                            if let Some(r) = self.rects.get_mut(index) {
                                *r = rect_from_corners([orig[0], orig[1]], px, self.snap, size);
                            }

                            self.selected = Some(index);
                            self.dirty = true;
                        }
                    }

                    if response.drag_stopped() || !ui.input(|i| i.pointer.any_down()) {
                        self.drag = None;
                    }
                }

                if response.clicked()
                    && let Some(p) = response.interact_pointer_pos()
                {
                    self.selected = hit(p, &self.rects);
                }

                if response.secondary_clicked()
                    && let Some(i) = response.interact_pointer_pos().and_then(|p| hit(p, &self.rects))
                {
                    self.rects.remove(i);
                    self.selected = None;
                    self.dirty = true;
                }

                for (i, r) in self.rects.iter().enumerate() {
                    let sr = to_screen(r);
                    let selected = self.selected == Some(i);
                    if selected {
                        painter.rect_filled(sr, 0.0, Color32::from_rgba_unmultiplied(255, 160, 40, 50));
                    }

                    painter.rect_stroke(
                        sr,
                        0.0,
                        Stroke::new(if selected { 2.5 } else { 1.5 }, if selected { Color32::from_rgb(255, 170, 60) } else { Color32::from_rgb(90, 220, 255) }),
                        StrokeKind::Inside,
                    );
                    painter.rect_filled(Rect::from_center_size(sr.max, Vec2::splat(7.0)), 1.0, Color32::WHITE);
                }
            });
            if let Some(i) = self.selected
                && let Some(r) = self.rects.get_mut(i)
            {
                ui.horizontal(|ui| {
                    ui.label(format!("rect {i}"));
                    for (label, v) in ["x", "y", "w", "h"].iter().zip(r.iter_mut()) {
                        if ui.add(egui::DragValue::new(v).speed(1.0).range(0.0..=size[0].max(size[1])).prefix(format!("{label} "))).changed() {
                            self.dirty = true;
                        }
                    }
                });
            }
        });
        self.open = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangles_snap_and_stay_inside() {
        assert_eq!(rect_from_corners([3.0, 70.0], [29.0, 5.0], 8, [64.0, 64.0]), [0.0, 8.0, 32.0, 56.0]);
        assert_eq!(rect_from_corners([10.0, 10.0], [10.0, 10.0], 16, [64.0, 64.0]), [16.0, 16.0, 16.0, 16.0]);
        let grid = grid_rects([128.0, 64.0], 4, 2);
        assert_eq!(grid.len(), 8);
        assert_eq!(grid[5], [32.0, 32.0, 32.0, 32.0]);
    }
}
