//! Graphical UV editor. Every selected face with the first face's material is drawn over its tiled texture.
//! Planar faces: drag moves the texture, wheel scales, right drag rotates.
//! Mesh faces with explicit UVs: click a corner handle to select it with its stitched corners, Shift+click adds, Ctrl+click
//! removes and Alt takes a single face's corner so a stitch can be split. Dragging a corner moves the whole selection, dragging
//! empty space moves whole faces, Shift or Ctrl+drag on empty space (or a plain drag in box select mode) selects with a box.
//! With corners selected the wheel scales and right drag rotates them around their center. Double click a corner for a move
//! gizmo, its arrows move along U or V only. Escape or a click on empty space hides it. Pixel snap applies while dragging.
//! Middle drag pans and Ctrl+wheel zooms, the view only refits for a new set of faces or with Frame, so it holds still while
//! editing. Every drag is one undo step.

use std::collections::BTreeSet;

use egui::{Color32, CursorIcon, PointerButton, Pos2, Rect, RichText, Sense, Shape, Stroke, StrokeKind, Ui, Vec2};
use gt_core::{DVec2, NodeId};

use crate::commands::Action;
use crate::icons;
use crate::panels::PanelState;
use crate::state::EditorState;
pub use crate::texture_ops::UvCorner;
use crate::texture_ops::{self, FaceInfo, MeshUvKind, UvAdjust};

/// Pointer distance in points that still grabs a corner or a gizmo handle.
const GRAB: f32 = 8.0;
const GIZMO_START: f32 = 12.0;
const GIZMO_LEN: f32 = 64.0;
const GIZMO_BOX: f32 = 7.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GizmoPart {
    U,
    V,
    Free,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BoxMode {
    Replace,
    Add,
    Remove,
}

#[derive(Clone, Debug)]
enum UvDrag {
    /// Corners moved from their start positions. With pixel snap `anchor` lands on the pixel grid, without one the
    /// offset moves in whole pixels.
    Corners {
        start: Vec<(UvCorner, DVec2)>,
        origin: DVec2,
        anchor: Option<DVec2>,
        axis: Option<GizmoPart>,
    },
    Faces {
        last: DVec2,
    },
    Box {
        start: Pos2,
        mode: BoxMode,
    },
    RotateCorners {
        start: Vec<(UvCorner, DVec2)>,
        center: DVec2,
        origin_x: f32,
    },
    RotateFaces,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct View {
    center: DVec2,
    half: f64,
}

pub struct UvEditorState {
    pub selected: BTreeSet<UvCorner>,
    pub pixel_snap: bool,
    pub show_hotspots: bool,
    /// Plain drags on empty space draw a selection box instead of moving whole faces.
    pub box_select: bool,
    /// The move gizmo on the selected corners, shown by double clicking a corner.
    pub gizmo: bool,
    pub rotate_step: f64,
    pub scale_step: f64,
    view: Option<View>,
    view_faces: Vec<(NodeId, usize)>,
    refit: u8,
    drag: Option<UvDrag>,
    hovered: bool,
    canvas: Rect,
}

impl Default for UvEditorState {
    fn default() -> Self {
        Self {
            selected: BTreeSet::new(),
            pixel_snap: false,
            show_hotspots: false,
            box_select: false,
            gizmo: false,
            rotate_step: 15.0,
            scale_step: 2.0,
            view: None,
            view_faces: Vec::new(),
            refit: 0,
            drag: None,
            hovered: false,
            canvas: Rect::NOTHING,
        }
    }
}

impl UvEditorState {
    /// Screen position of a UV coordinate in the last drawn canvas.
    pub fn screen_pos(&self, uv: DVec2) -> Option<Pos2> {
        let view = self.view?;
        Some(to_screen(self.canvas, view, uv))
    }

    /// Escape over the editor hides the gizmo instead of reaching the global shortcuts. Returns true if it was used.
    pub fn take_escape(&mut self, ctx: &egui::Context) -> bool {
        let used = self.hovered && self.gizmo && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if used {
            self.gizmo = false;
        }

        used
    }
}

fn to_screen(rect: Rect, view: View, uv: DVec2) -> Pos2 {
    let ppu = rect.width().min(rect.height()) as f64 / (view.half * 2.0);
    let d = (uv - view.center) * ppu;
    rect.center() + Vec2::new(d.x as f32, d.y as f32)
}

fn to_uv(rect: Rect, view: View, p: Pos2) -> DVec2 {
    let ppu = rect.width().min(rect.height()) as f64 / (view.half * 2.0);
    let d = p - rect.center();
    view.center + DVec2::new(d.x as f64, d.y as f64) / ppu
}

fn fit_view(points: impl Iterator<Item = DVec2>) -> View {
    let (lo, hi) = points.fold((DVec2::MAX, DVec2::MIN), |(lo, hi), u| (lo.min(u), hi.max(u)));
    if lo.x > hi.x {
        return View { center: DVec2::splat(0.5), half: 0.75 };
    }

    View { center: (lo + hi) * 0.5, half: (hi - lo).max_element().max(1.0) * 0.75 }
}

fn corner_uvs(info: &FaceInfo, tex: DVec2) -> Vec<DVec2> {
    match &info.explicit {
        Some(uvs) => uvs.iter().map(|u| DVec2::new(u[0] as f64, u[1] as f64)).collect(),
        None => info.points.iter().map(|p| info.uv.uv(*p, tex)).collect(),
    }
}

fn segment_distance(a: Pos2, b: Pos2, p: Pos2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_sq().max(1e-6)).clamp(0.0, 1.0);
    (a + ab * t).distance(p)
}

pub fn gizmo_part_at(center: Pos2, p: Pos2) -> Option<GizmoPart> {
    let d = p - center;
    if d.x.abs() <= GIZMO_BOX + 2.0 && d.y.abs() <= GIZMO_BOX + 2.0 {
        return Some(GizmoPart::Free);
    }

    let u = segment_distance(center + Vec2::new(GIZMO_START, 0.0), center + Vec2::new(GIZMO_LEN, 0.0), p);
    let v = segment_distance(center - Vec2::new(0.0, GIZMO_START), center - Vec2::new(0.0, GIZMO_LEN), p);
    match (u < GRAB, v < GRAB) {
        (true, true) => Some(if u <= v { GizmoPart::U } else { GizmoPart::V }),
        (true, false) => Some(GizmoPart::U),
        (false, true) => Some(GizmoPart::V),
        _ => None,
    }
}

fn paint_gizmo(painter: &egui::Painter, center: Pos2, hot: Option<GizmoPart>) {
    let color = |part: GizmoPart, base: Color32| if hot == Some(part) { crate::theme::YELLOW } else { base };
    for (part, dir, base) in [(GizmoPart::U, Vec2::X, crate::theme::AXIS[0]), (GizmoPart::V, -Vec2::Y, crate::theme::AXIS[1])] {
        let c = color(part, base);
        let (from, head, tip) = (center + dir * GIZMO_START, center + dir * (GIZMO_LEN - 12.0), center + dir * GIZMO_LEN);
        painter.line_segment([from, head], Stroke::new(if hot == Some(part) { 3.5 } else { 2.5 }, c));
        let side = Vec2::new(-dir.y, dir.x) * 6.0;
        painter.add(Shape::convex_polygon(vec![tip, head + side, head - side], c, Stroke::NONE));
    }

    let free = hot == Some(GizmoPart::Free);
    let r = Rect::from_center_size(center, Vec2::splat(GIZMO_BOX * 2.0));
    painter.rect_filled(r, 1.0, if free { crate::theme::YELLOW.gamma_multiply(0.6) } else { Color32::from_black_alpha(120) });
    painter.rect_stroke(r, 1.0, Stroke::new(1.5, if free { crate::theme::YELLOW } else { Color32::WHITE }), StrokeKind::Inside);
}

/// Pixel size of a material's image, its UV size when the image is unknown.
pub fn image_pixels(state: &EditorState, material: &str) -> DVec2 {
    state.materials.pixel_size(material).map(|[w, h]| DVec2::new(w as f64, h as f64)).unwrap_or_else(|| texture_ops::tex_size(state, material))
}

fn corner_positions(state: &EditorState, corners: impl IntoIterator<Item = UvCorner>) -> Vec<(UvCorner, DVec2)> {
    corners.into_iter().filter_map(|c| texture_ops::corner_uv(&state.doc.map, c).map(|u| (c, u))).collect()
}

/// The hotspot rectangle (pixels) under the center of `uvs`, else the closest one.
pub fn hotspot_for(rects: &[[f64; 4]], uvs: &[DVec2], tex: DVec2) -> Option<[f64; 4]> {
    let c = texture_ops::uv_center(uvs) * tex;
    let inside = |r: &[f64; 4]| c.x >= r[0] && c.x <= r[0] + r[2] && c.y >= r[1] && c.y <= r[1] + r[3];
    let dist = |r: &[f64; 4]| (DVec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5) - c).length();
    rects.iter().find(|r| inside(r)).or_else(|| rects.iter().min_by(|a, b| dist(a).total_cmp(&dist(b)))).copied()
}

fn select_corner(uv_state: &mut UvEditorState, state: &EditorState, faces: &[(NodeId, usize)], c: UvCorner, mods: egui::Modifiers) {
    let group = if mods.alt { vec![c] } else { texture_ops::stitched_corners(&state.doc.map, faces, c) };
    if mods.command {
        for s in group {
            uv_state.selected.remove(&s);
        }

        return;
    }

    if !mods.shift {
        uv_state.selected.clear();
    }

    uv_state.selected.extend(group);
}

/// Toolbar icon size.
const CHIP: f32 = 16.0;
const FOOTER_HEIGHT: f32 = 24.0;
const CONTROLS: &str = "Faces: drag empty space moves the texture, wheel scales, right drag rotates.\n\
Corners: click selects one with its stitched corners, Shift+click adds, Ctrl+click removes, Alt+click splits a stitch.\n\
Shift or Ctrl+drag on empty space box selects, adding or removing.\n\
Dragging a selected corner moves the whole selection, wheel and right drag scale and rotate it.\n\
Double click a corner for a move gizmo, its arrows move along U or V only, Esc hides it.\n\
Middle drag pans, Ctrl+wheel zooms.";

fn projection_menu(ui: &mut Ui, state: &EditorState, faces: &[FaceInfo], uv_state: &mut UvEditorState, actions: &mut Vec<Action>) {
    let mut item = |ui: &mut Ui, k: MeshUvKind| {
        if ui.button(k.label()).on_hover_text(k.hint()).clicked() {
            actions.push(Action::MeshUv(k));
            uv_state.refit = 2;
            ui.close();
        }
    };
    for k in MeshUvKind::ALL {
        if matches!(k, MeshUvKind::World | MeshUvKind::Normalize) {
            ui.separator();
        }

        if k == MeshUvKind::Bake && !faces.iter().any(|i| i.explicit.is_none() && state.doc.map.mesh(i.id).is_some()) {
            continue;
        }

        item(ui, k);
    }
}

#[allow(clippy::too_many_arguments)]
fn adjust_menu(
    ui: &mut Ui,
    state: &mut EditorState,
    uv_state: &mut UvEditorState,
    targets: &[UvCorner],
    has_selection: bool,
    hotspots: &[[f64; 4]],
    tex: DVec2,
    px: DVec2,
) {
    let uvs: Vec<DVec2> = corner_positions(state, targets.iter().copied()).into_iter().map(|(_, u)| u).collect();
    let fit = match hotspot_for(hotspots, &uvs, tex) {
        Some(r) => ([r[0] / tex.x, r[1] / tex.y, r[2] / tex.x, r[3] / tex.y], "Stretch onto the hotspot under the corners"),
        None => ([0.0, 0.0, 1.0, 1.0], "Stretch onto the whole texture, 0 to 1"),
    };
    let mut run = |ui: &mut Ui, enabled: bool, icon: Option<icons::Icon>, label: &str, hint: &str, op: UvAdjust| {
        let button = egui::Button::new((icons::atom(icon, icons::SMALL), label)).image_tint_follows_text_color(true);
        if ui.add_enabled(enabled, button).on_hover_text(hint).clicked() {
            texture_ops::adjust_corners(state, targets, op, false);
        }
    };
    run(ui, true, Some(icons::FLIP_U), "Flip U", "Mirror left to right around the center", UvAdjust::FlipU);
    run(ui, true, Some(icons::FLIP_V), "Flip V", "Mirror top to bottom around the center", UvAdjust::FlipV);
    run(ui, true, Some(icons::ROTATE_CCW), "Rotate -90", "Quarter turn counterclockwise", UvAdjust::Rotate(-90.0));
    run(ui, true, Some(icons::ROTATE_CW), "Rotate +90", "Quarter turn clockwise", UvAdjust::Rotate(90.0));
    ui.separator();
    run(ui, true, None, "Fit", fit.1, UvAdjust::Fit(fit.0));
    run(ui, has_selection, None, "Align Horizontally", "Line the selected corners up on one v", UvAdjust::AlignHorizontal);
    run(ui, has_selection, None, "Align Vertically", "Line the selected corners up on one u", UvAdjust::AlignVertical);
    run(ui, has_selection, None, "Straighten", "Put the selected corners on the line between the two farthest apart", UvAdjust::Straighten);
    run(ui, true, None, "Snap to Pixels", "Round to whole texture pixels", UvAdjust::Snap(px));
    ui.separator();
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut uv_state.rotate_step).speed(1.0).range(0.0..=360.0).prefix("Rotate ").suffix("°"));
        for (icon, label, sign) in [(icons::ROTATE_CCW, "Rotate Step Counterclockwise", -1.0), (icons::ROTATE_CW, "Rotate Step Clockwise", 1.0)] {
            if icons::button(ui, icon, icons::SMALL, label, label).clicked() {
                texture_ops::adjust_corners(state, targets, UvAdjust::Rotate(uv_state.rotate_step * sign), false);
            }
        }
    });
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut uv_state.scale_step).speed(0.01).range(0.01..=100.0).prefix("Scale "));
        let step = uv_state.scale_step.max(0.01);
        for (label, hint, f) in [("÷", "Shrink by the step", 1.0 / step), ("×", "Grow by the step", step)] {
            if ui.small_button(label).on_hover_text(hint).clicked() {
                texture_ops::adjust_corners(state, targets, UvAdjust::Scale(DVec2::splat(f)), false);
            }
        }
    });
}

/// The pixel position of the selected corners, in a strip of fixed height under the canvas so the canvas never jumps.
fn footer(ui: &mut Ui, state: &mut EditorState, selection: &[UvCorner], px: DVec2) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), FOOTER_HEIGHT), Sense::hover());
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
    let ui = &mut child;
    if selection.is_empty() {
        ui.label(RichText::new("Click a corner to select it, double click for a gizmo").weak());
        return;
    }

    let uvs: Vec<DVec2> = corner_positions(state, selection.iter().copied()).into_iter().map(|(_, u)| u).collect();
    let center = texture_ops::uv_center(&uvs) * px;
    ui.label(RichText::new(if selection.len() == 1 { "1 corner".to_string() } else { format!("{} corners", selection.len()) }).weak());
    let (mut u, mut v) = (center.x, center.y);
    let du = ui
        .add_sized([88.0, 18.0], egui::DragValue::new(&mut u).speed(0.25).max_decimals(2).prefix("U ").suffix(" px"))
        .on_hover_text("Center of the selected corners in texture pixels");
    let dv = ui.add_sized([88.0, 18.0], egui::DragValue::new(&mut v).speed(0.25).max_decimals(2).prefix("V ").suffix(" px"));
    if (du.changed() || dv.changed()) && (u != center.x || v != center.y) {
        texture_ops::adjust_corners(state, selection, UvAdjust::Center(DVec2::new(u, v) / px), true);
    }
}

pub fn uv_editor(ui: &mut Ui, state: &mut EditorState, ps: &mut PanelState, actions: &mut Vec<Action>) {
    let selected_faces: Vec<(NodeId, usize)> = state.doc.selection.faces.iter().copied().collect();
    let Some(first) = selected_faces.first().and_then(|(id, f)| texture_ops::face_info(&state.doc.map, *id, *f)) else {
        let uv_state = &mut ps.uv;
        if uv_state.drag.take().is_some() && state.doc.in_transaction() {
            state.doc.commit();
        }

        uv_state.gizmo = false;
        uv_state.hovered = false;
        ui.label(
            RichText::new("Select faces to edit their UVs: Shift+click a face, or Ctrl+click an object to grab all its sides, or use the Texture tool.").weak(),
        );
        return;
    };
    let material = first.material.clone();
    let tex = texture_ops::tex_size(state, &material);
    // UVs count in texture repeats of `tex` map units, pixel snap and the pixel fields use the image's own pixels.
    let px = image_pixels(state, &material);
    let faces: Vec<FaceInfo> =
        selected_faces.iter().filter_map(|(id, f)| texture_ops::face_info(&state.doc.map, *id, *f)).filter(|i| i.material == material).collect();
    let face_keys: Vec<(NodeId, usize)> = faces.iter().map(|i| (i.id, i.face)).collect();
    let has_mesh = faces.iter().any(|i| state.doc.map.mesh(i.id).is_some());
    let has_explicit = faces.iter().any(|i| i.explicit.is_some());
    let uv_state = &mut ps.uv;
    uv_state.selected.retain(|(id, f, _)| faces.iter().any(|i| i.id == *id && i.face == *f && i.explicit.is_some()));
    if uv_state.selected.is_empty() {
        uv_state.gizmo = false;
    }

    // An undo or a tab switch mid drag closes the edit, the drag must not go on outside it.
    if matches!(uv_state.drag, Some(UvDrag::Corners { .. } | UvDrag::Faces { .. } | UvDrag::RotateCorners { .. } | UvDrag::RotateFaces))
        && !state.doc.in_transaction()
    {
        uv_state.drag = None;
    }

    let hotspots = if uv_state.show_hotspots { crate::commands::hotspot_rects_uv(state, &material) } else { Vec::new() };
    let size = state.materials.size_label(&material).unwrap_or_else(|| format!("{} x {} px", tex.x, tex.y));
    ui.horizontal(|ui| {
        let info = format!("{material}  ·  {size}  ·  {} faces", faces.len());
        let width = (ui.available_width() - CHIP - 16.0).max(40.0);
        ui.allocate_ui_with_layout(Vec2::new(width, ui.spacing().interact_size.y), egui::Layout::left_to_right(egui::Align::Center), |ui| {
            ui.add(egui::Label::new(RichText::new(&info).weak()).truncate()).on_hover_text(&info);
        });
        icons::button(ui, icons::HELP, CHIP, "Controls", CONTROLS);
    });

    let selection: Vec<UvCorner> = uv_state.selected.iter().copied().collect();
    let has_selection = !selection.is_empty();
    // Adjustments work on the selected corners, or on every corner of the faces when none are selected.
    let targets = if has_selection { selection.clone() } else { texture_ops::all_corners(&state.doc.map, &face_keys) };
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        if has_mesh {
            ui.menu_button("Projection", |ui| projection_menu(ui, state, &faces, uv_state, actions))
                .response
                .on_hover_text("Lay out explicit UVs on the selected mesh faces, or every face of the selected meshes");
        }

        ui.menu_button("UV", |ui| {
            if has_explicit {
                adjust_menu(ui, state, uv_state, &targets, has_selection, &hotspots, tex, px);
                ui.separator();
            }

            if ui.button("Reset UVs").on_hover_text("Drop any custom UVs on the selected faces back to a clean face-aligned projection").clicked() {
                actions.push(Action::ResetTexture);
                ui.close();
            }
        })
        .response
        .on_hover_text("Flip, turn, fit, align, straighten and snap the selected corners, or all of them when none are selected");
        ui.menu_button("Hotspot", |ui| {
            if ui.button("Hotspot Fit").on_hover_text("Fit the faces to the best rectangle of <texture>.hotspots.json").clicked() {
                actions.push(Action::HotspotTexture);
                ui.close();
            }

            if has_explicit {
                let rects = crate::commands::hotspot_rects_uv(state, &material);
                let uvs: Vec<DVec2> = corner_positions(state, targets.iter().copied()).into_iter().map(|(_, u)| u).collect();
                let rect = hotspot_for(&rects, &uvs, tex);
                if ui
                    .add_enabled(rect.is_some(), egui::Button::new("Fit Corners to Hotspot"))
                    .on_hover_text("Stretch the corners onto the hotspot under them")
                    .clicked()
                    && let Some(r) = rect
                {
                    texture_ops::adjust_corners(state, &targets, UvAdjust::Fit([r[0] / tex.x, r[1] / tex.y, r[2] / tex.x, r[3] / tex.y]), false);
                    ui.close();
                }
            }

            ui.checkbox(&mut uv_state.show_hotspots, "Show Hotspots");
            if ui.button("Hotspot Editor").clicked() {
                actions.push(Action::ShowHotspotEditor);
                ui.close();
            }
        });

        ui.add_space(4.0);
        let chip = |ui: &mut Ui, icon: icons::Icon, on: bool, label: &str, tip: &str| icons::toggle(ui, icon, CHIP, on, label, tip).clicked();
        if chip(ui, icons::SNAP, uv_state.pixel_snap, "Pixel Snap", "Pixel snap: dragged corners land on whole texture pixels") {
            uv_state.pixel_snap = !uv_state.pixel_snap;
        }

        if chip(ui, icons::GRID, uv_state.show_hotspots, "Show Hotspots", "Show the hotspot rectangles of the texture") {
            uv_state.show_hotspots = !uv_state.show_hotspots;
        }

        if chip(ui, icons::VOLUME, uv_state.box_select, "Box Select", "Box select: a plain drag on empty space selects corners instead of moving the faces") {
            uv_state.box_select = !uv_state.box_select;
        }

        if has_explicit
            && ui
                .add_enabled_ui(has_selection, |ui| {
                    chip(ui, icons::MOVE, uv_state.gizmo, "Gizmo", "Move gizmo on the selected corners, a double click on a corner shows it too")
                })
                .inner
        {
            uv_state.gizmo = !uv_state.gizmo;
        }

        if chip(ui, icons::UV_LOCK, state.uv_lock, "UV Lock", "UV lock: textures stay attached while objects move (Ctrl+Shift+U)") {
            actions.push(Action::ToggleUvLock);
        }

        if icons::button(ui, icons::FOCUS, CHIP, "Frame", "Frame: fit the view to the faces again").clicked() {
            uv_state.refit = 1;
        }
    });

    let face_uvs: Vec<Vec<DVec2>> = faces.iter().map(|f| corner_uvs(f, tex)).collect();
    let avail = ui.available_size();
    let footer_height = if has_explicit { FOOTER_HEIGHT + ui.spacing().item_spacing.y } else { 0.0 };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(avail.x.max(120.0), (avail.y - footer_height).max(120.0)), Sense::click_and_drag());
    uv_state.canvas = rect;
    uv_state.hovered = response.hovered() || uv_state.drag.is_some();
    let dragging = uv_state.drag.is_some();
    let out_of_sight = uv_state.view.is_some_and(|v| {
        let r = face_uvs.iter().flatten().fold(Rect::NOTHING, |r, u| r.union(Rect::from_center_size(to_screen(rect, v, *u), Vec2::ZERO)));
        !r.intersects(rect)
    });
    // A layout button refits once its action has run, which is after this frame.
    if uv_state.refit > 1 {
        uv_state.refit -= 1;
        ui.ctx().request_repaint();
    } else if !dragging && (uv_state.view.is_none() || uv_state.view_faces != face_keys || out_of_sight || uv_state.refit == 1) {
        uv_state.view = Some(fit_view(face_uvs.iter().flatten().copied()));
        uv_state.view_faces = face_keys.clone();
        uv_state.refit = 0;
    }

    let mut view = uv_state.view.unwrap_or(View { center: DVec2::splat(0.5), half: 0.75 });
    if response.hovered() {
        let zoom = ui.input(|i| i.zoom_delta());
        if zoom != 1.0
            && let Some(p) = response.hover_pos()
        {
            let anchor = to_uv(rect, view, p);
            let half = (view.half / zoom as f64).clamp(1e-3, 1e4);
            view = View { center: anchor - (anchor - view.center) * (half / view.half), half };
        }
    }

    if response.dragged_by(PointerButton::Middle) {
        let d = response.drag_delta();
        let ppu = rect.width().min(rect.height()) as f64 / (view.half * 2.0);
        view.center -= DVec2::new(d.x as f64, d.y as f64) / ppu;
    }

    uv_state.view = Some(view);
    let view = view;
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, Color32::from_gray(18));

    let ctx = ui.ctx().clone();
    if let Some(texture) = state.materials.full_texture(&ctx, &material) {
        let start = to_uv(rect, view, rect.min).floor();
        let end = to_uv(rect, view, rect.max).ceil();
        if (end.x - start.x) * (end.y - start.y) <= 400.0 {
            let mut y = start.y;
            while y < end.y {
                let mut x = start.x;
                while x < end.x {
                    let r = Rect::from_min_max(to_screen(rect, view, DVec2::new(x, y)), to_screen(rect, view, DVec2::new(x + 1.0, y + 1.0)));
                    painter.image(texture.id(), r, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::from_gray(150));
                    painter.rect_stroke(r, 0.0, Stroke::new(1.0, Color32::from_black_alpha(90)), StrokeKind::Inside);
                    x += 1.0;
                }

                y += 1.0;
            }
        }
    }

    for r in &hotspots {
        let a = to_screen(rect, view, DVec2::new(r[0], r[1]) / tex);
        let b = to_screen(rect, view, DVec2::new(r[0] + r[2], r[1] + r[3]) / tex);
        painter.rect_stroke(Rect::from_two_pos(a, b), 0.0, Stroke::new(1.0, crate::theme::CYAN), StrokeKind::Inside);
    }

    let mods = ui.input(|i| i.modifiers);
    let hover = response.hover_pos();
    let corner_at = |p: Pos2| -> Option<UvCorner> {
        faces
            .iter()
            .zip(&face_uvs)
            .filter(|(i, _)| i.explicit.is_some())
            .flat_map(|(i, uvs)| uvs.iter().enumerate().map(move |(k, u)| ((i.id, i.face, k), (to_screen(rect, view, *u) - p).length())))
            .filter(|(_, d)| *d < GRAB)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(c, _)| c)
    };
    let selected_uvs = |uv_state: &UvEditorState, state: &EditorState| -> Vec<DVec2> {
        corner_positions(state, uv_state.selected.iter().copied()).into_iter().map(|(_, u)| u).collect()
    };
    let gizmo_center = |uv_state: &UvEditorState, state: &EditorState| -> Option<Pos2> {
        (uv_state.gizmo && !uv_state.selected.is_empty()).then(|| to_screen(rect, view, texture_ops::uv_center(&selected_uvs(uv_state, state))))
    };
    let hovered_corner = hover.filter(|_| uv_state.drag.is_none()).and_then(corner_at);

    for (info, uvs) in faces.iter().zip(&face_uvs) {
        let pts: Vec<Pos2> = uvs.iter().map(|u| to_screen(rect, view, *u)).collect();
        let color = if info.explicit.is_some() { Color32::from_rgb(120, 200, 255) } else { Color32::from_rgb(255, 150, 60) };
        painter.add(Shape::closed_line(pts.clone(), Stroke::new(1.5, color)));
        if info.explicit.is_none() {
            painter.add(Shape::convex_polygon(pts.clone(), Color32::from_rgba_unmultiplied(255, 120, 40, 30), Stroke::NONE));
        }

        for (k, p) in pts.iter().enumerate() {
            let key = (info.id, info.face, k);
            let sel = uv_state.selected.contains(&key);
            painter.circle_filled(*p, if sel { 4.5 } else { 3.0 }, if sel { crate::theme::YELLOW } else { Color32::from_rgb(230, 230, 230) });
            if hovered_corner == Some(key) {
                painter.circle_stroke(*p, 6.5, Stroke::new(1.5, Color32::WHITE));
            }
        }
    }

    // ------------------------------------------------------------------ input

    let gizmo_hit = |uv_state: &UvEditorState, state: &EditorState, p: Pos2| gizmo_center(uv_state, state).and_then(|c| gizmo_part_at(c, p));

    if response.clicked()
        && let Some(p) = response.interact_pointer_pos()
        && gizmo_hit(uv_state, state, p).is_none()
    {
        match corner_at(p) {
            Some(c) => select_corner(uv_state, state, &face_keys, c, mods),
            None if !mods.shift && !mods.command => {
                uv_state.selected.clear();
                uv_state.gizmo = false;
            }
            None => {}
        }
    }

    if response.double_clicked()
        && let Some(p) = response.interact_pointer_pos()
        && let Some(c) = corner_at(p)
    {
        if !uv_state.selected.contains(&c) {
            select_corner(uv_state, state, &face_keys, c, egui::Modifiers { alt: mods.alt, ..Default::default() });
        }

        uv_state.gizmo = true;
    }

    let pointer_uv = ui.input(|i| i.pointer.interact_pos()).map(|p| to_uv(rect, view, p));
    if response.drag_started_by(PointerButton::Primary)
        && let Some(origin) = ui.input(|i| i.pointer.press_origin())
    {
        let origin_uv = to_uv(rect, view, origin);
        if let Some(part) = gizmo_hit(uv_state, state, origin) {
            state.doc.begin("Move UV Corners");
            let start = corner_positions(state, uv_state.selected.iter().copied());
            uv_state.drag = Some(UvDrag::Corners { start, origin: origin_uv, anchor: None, axis: (part != GizmoPart::Free).then_some(part) });
        } else if let Some(c) = corner_at(origin).filter(|_| !mods.command) {
            if !uv_state.selected.contains(&c) {
                select_corner(uv_state, state, &face_keys, c, mods);
            }

            state.doc.begin("Move UV Corners");
            let start = corner_positions(state, uv_state.selected.iter().copied());
            let anchor = texture_ops::corner_uv(&state.doc.map, c);
            uv_state.drag = Some(UvDrag::Corners { start, origin: origin_uv, anchor, axis: None });
        } else if mods.shift || mods.command || uv_state.box_select {
            let mode = if mods.command {
                BoxMode::Remove
            } else if mods.shift {
                BoxMode::Add
            } else {
                BoxMode::Replace
            };
            uv_state.drag = Some(UvDrag::Box { start: origin, mode });
        } else {
            state.doc.begin("Shift Texture");
            uv_state.drag = Some(UvDrag::Faces { last: origin_uv });
        }
    }

    if response.drag_started_by(PointerButton::Secondary) {
        if uv_state.selected.is_empty() {
            state.doc.begin("Rotate Texture");
            uv_state.drag = Some(UvDrag::RotateFaces);
        } else if let Some(origin) = ui.input(|i| i.pointer.press_origin()) {
            state.doc.begin("Rotate UVs");
            let start = corner_positions(state, uv_state.selected.iter().copied());
            let center = texture_ops::uv_center(&start.iter().map(|(_, u)| *u).collect::<Vec<_>>());
            uv_state.drag = Some(UvDrag::RotateCorners { start, center, origin_x: origin.x });
        }
    }

    let pointer_pos = ui.input(|i| i.pointer.interact_pos());
    match uv_state.drag.clone() {
        Some(UvDrag::Corners { start, origin, anchor, axis }) => {
            if let Some(p) = pointer_uv {
                let mut offset = p - origin;
                match axis {
                    Some(GizmoPart::U) => offset.y = 0.0,
                    Some(GizmoPart::V) => offset.x = 0.0,
                    _ => {}
                }

                if uv_state.pixel_snap {
                    offset = match anchor {
                        Some(a) => {
                            let snapped = ((a + offset) * px).round() / px - a;
                            DVec2::new(if axis == Some(GizmoPart::V) { 0.0 } else { snapped.x }, if axis == Some(GizmoPart::U) { 0.0 } else { snapped.y })
                        }
                        None => (offset * px).round() / px,
                    };
                }

                let moved: Vec<(UvCorner, DVec2)> = start.iter().map(|(c, u)| (*c, *u + offset)).collect();
                texture_ops::set_corner_uvs(state, "Move UV Corners", false, &moved);
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            }
        }
        Some(UvDrag::Faces { last }) => {
            if let Some(p) = pointer_uv
                && p != last
            {
                // The face outline follows the pointer over the texture.
                texture_ops::shift(state, &face_keys, (p - last) * tex);
                uv_state.drag = Some(UvDrag::Faces { last: p });
            }
        }
        Some(UvDrag::RotateCorners { start, center, origin_x }) => {
            if let Some(p) = pointer_pos {
                let mut degrees = ((p.x - origin_x) * 0.5) as f64;
                if uv_state.pixel_snap || mods.shift {
                    degrees = (degrees / 15.0).round() * 15.0;
                }

                let (s, c) = degrees.to_radians().sin_cos();
                let moved: Vec<(UvCorner, DVec2)> = start
                    .iter()
                    .map(|(k, u)| {
                        let d = *u - center;
                        (*k, center + DVec2::new(d.x * c - d.y * s, d.x * s + d.y * c))
                    })
                    .collect();
                texture_ops::set_corner_uvs(state, "Rotate UVs", false, &moved);
            }
        }
        Some(UvDrag::RotateFaces) => {
            let delta = response.drag_delta();
            if delta.x != 0.0 {
                texture_ops::rotate(state, &face_keys, delta.x as f64 * 0.5);
            }
        }
        Some(UvDrag::Box { start, .. }) => {
            if let Some(p) = pointer_pos {
                let r = Rect::from_two_pos(start, p);
                painter.rect_filled(r, 0.0, crate::theme::YELLOW.gamma_multiply(0.08));
                painter.rect_stroke(r, 0.0, Stroke::new(1.0, crate::theme::YELLOW), StrokeKind::Inside);
            }
        }
        None => {}
    }

    let released = !ui.input(|i| i.pointer.any_down());
    if released && let Some(drag) = uv_state.drag.take() {
        match drag {
            UvDrag::Box { start, mode } => {
                let end = pointer_pos.unwrap_or(start);
                let area = Rect::from_two_pos(start, end);
                if mode == BoxMode::Replace {
                    uv_state.selected.clear();
                }

                let inside: Vec<UvCorner> = faces
                    .iter()
                    .zip(&face_uvs)
                    .filter(|(i, _)| i.explicit.is_some())
                    .flat_map(|(i, uvs)| uvs.iter().enumerate().filter(|(_, u)| area.contains(to_screen(rect, view, **u))).map(move |(k, _)| (i.id, i.face, k)))
                    .collect();
                for c in inside {
                    if mode == BoxMode::Remove {
                        uv_state.selected.remove(&c);
                    } else {
                        uv_state.selected.insert(c);
                    }
                }
            }
            _ => state.doc.commit(),
        }
    }

    let scroll = if response.hovered() && !mods.command { ui.input(|i| i.smooth_scroll_delta.y) } else { 0.0 };
    if scroll != 0.0 && uv_state.drag.is_none() {
        let factor = (1.0 + scroll as f64 * 0.002).clamp(0.5, 2.0);
        if uv_state.selected.is_empty() {
            texture_ops::scale(state, &face_keys, DVec2::splat(factor));
        } else {
            let selection: Vec<UvCorner> = uv_state.selected.iter().copied().collect();
            texture_ops::adjust_corners(state, &selection, UvAdjust::Scale(DVec2::splat(1.0 / factor)), true);
        }
    }

    if let Some(c) = gizmo_center(uv_state, state) {
        let active = match &uv_state.drag {
            Some(UvDrag::Corners { anchor: None, axis, .. }) => Some(axis.unwrap_or(GizmoPart::Free)),
            _ => None,
        };
        let hot = active.or_else(|| hover.and_then(|p| gizmo_part_at(c, p)));
        if hot.is_some() && active.is_none() {
            ui.ctx().set_cursor_icon(CursorIcon::Grab);
        }

        paint_gizmo(&painter, c, hot);
    }

    if has_explicit {
        let selection: Vec<UvCorner> = uv_state.selected.iter().copied().collect();
        footer(ui, state, &selection, px);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_core::{Aabb, DVec3};
    use gt_geom::{Mesh, mesh_shapes};

    fn close(a: DVec2, b: DVec2) -> bool {
        (a - b).length() < 1e-9
    }

    fn scene(mesh: Mesh) -> (EditorState, NodeId, Vec<(NodeId, usize)>) {
        let mut state = EditorState::new(Default::default());
        let layer = state.doc.map.default_layer();
        let n = mesh.faces.len();
        let id = state.doc.edit("add", |m, s| {
            let id = m.insert(layer, gt_doc::NodeKind::Mesh(mesh));
            for f in 0..n {
                s.select_face(id, f);
            }

            id
        });
        (state, id, (0..n).map(|f| (id, f)).collect())
    }

    const VIEW: (DVec3, DVec3) = (DVec3::X, DVec3::Y);

    #[test]
    fn adjust_points_flip_rotate_scale_align_fit_and_snap() {
        let square = [DVec2::new(0.0, 0.0), DVec2::new(2.0, 0.0), DVec2::new(2.0, 1.0), DVec2::new(0.0, 1.0)];
        let mut p = square;
        texture_ops::adjust_points(&mut p, UvAdjust::FlipU);
        assert!(close(p[0], DVec2::new(2.0, 0.0)) && close(p[1], DVec2::ZERO));
        texture_ops::adjust_points(&mut p, UvAdjust::FlipV);
        assert!(close(p[0], DVec2::new(2.0, 1.0)));

        let mut p = square;
        texture_ops::adjust_points(&mut p, UvAdjust::Rotate(90.0));
        assert_eq!(texture_ops::uv_center(&p), DVec2::new(1.0, 0.5), "turns around the center");
        assert_eq!(p[1] - p[0], DVec2::new(0.0, 2.0), "a quarter turn is exact, u runs down the screen now");
        for _ in 0..3 {
            texture_ops::adjust_points(&mut p, UvAdjust::Rotate(90.0));
        }

        assert_eq!(p, square, "four quarter turns are the identity");

        let mut p = square;
        texture_ops::adjust_points(&mut p, UvAdjust::Scale(DVec2::new(2.0, 0.5)));
        assert!(close(p[0], DVec2::new(-1.0, 0.25)) && close(p[2], DVec2::new(3.0, 0.75)));

        let mut p = [DVec2::new(0.0, 0.1), DVec2::new(1.0, 0.3), DVec2::new(2.0, 0.2)];
        texture_ops::adjust_points(&mut p, UvAdjust::AlignHorizontal);
        assert!(p.iter().all(|q| (q.y - 0.2).abs() < 1e-12) && p[2].x == 2.0);
        let mut p = [DVec2::new(0.1, 0.0), DVec2::new(0.3, 5.0)];
        texture_ops::adjust_points(&mut p, UvAdjust::AlignVertical);
        assert!(p.iter().all(|q| (q.x - 0.2).abs() < 1e-12));

        let mut p = [DVec2::new(0.0, 0.0), DVec2::new(1.0, 0.2), DVec2::new(2.0, -0.1), DVec2::new(4.0, 0.0)];
        texture_ops::adjust_points(&mut p, UvAdjust::Straighten);
        assert!(close(p[0], DVec2::ZERO) && close(p[3], DVec2::new(4.0, 0.0)), "the ends stay");
        assert!(p.iter().all(|q| q.y.abs() < 1e-12), "the middle corners land on the line: {p:?}");

        let mut p = square;
        texture_ops::adjust_points(&mut p, UvAdjust::Fit([0.25, 0.5, 0.5, 0.25]));
        assert!(close(p[0], DVec2::new(0.25, 0.5)) && close(p[2], DVec2::new(0.75, 0.75)));

        let mut p = [DVec2::new(0.1234, 0.9876)];
        texture_ops::adjust_points(&mut p, UvAdjust::Snap(DVec2::new(64.0, 32.0)));
        assert_eq!(p[0], DVec2::new(8.0 / 64.0, 32.0 / 32.0));
        texture_ops::adjust_points(&mut p, UvAdjust::Center(DVec2::new(0.5, 0.5)));
        assert_eq!(p[0], DVec2::new(0.5, 0.5));
    }

    #[test]
    fn projections_write_explicit_uvs_at_brush_texel_density() {
        let (mut state, id, faces) = scene(mesh_shapes::cuboid(&Aabb::new(DVec3::ZERO, DVec3::splat(128.0)), "dev/grey"));
        let tex = texture_ops::tex_size(&state, "dev/grey");
        assert_eq!(texture_ops::mesh_uv(&mut state, &faces, MeshUvKind::World, VIEW), 6);
        assert_eq!(state.doc.history.undo_labels().next(), Some("World UV Projection"));
        let mesh = state.doc.map.mesh(id).unwrap();
        for f in 0..6 {
            let brush = gt_geom::FaceUv::face_aligned(mesh.face_normal(f), DVec2::ONE);
            for (k, p) in mesh.face_points(f).iter().enumerate() {
                assert!((mesh.corner_uv(f, k, tex) - brush.uv(*p, tex)).length() < 1e-5, "face {f} matches a reset brush face");
            }
        }

        // Planar picks the main axis of the normals, here the top face alone.
        let top = (0..6).find(|f| mesh.face_normal(*f).y > 0.5).unwrap();
        texture_ops::mesh_uv(&mut state, &[(id, top)], MeshUvKind::Planar, VIEW);
        let mesh = state.doc.map.mesh(id).unwrap();
        for (k, p) in mesh.face_points(top).iter().enumerate() {
            assert!((mesh.corner_uv(top, k, tex) - DVec2::new(p.x, p.z) / tex).length() < 1e-5);
        }

        // Texel density follows the face scale.
        state.doc.edit("scale", |m, _| m.mesh_mut(id).unwrap().faces.iter_mut().for_each(|f| f.data.uv.scale = DVec2::splat(2.0)));
        texture_ops::mesh_uv(&mut state, &faces, MeshUvKind::PlanarAxis(1), VIEW);
        let mesh = state.doc.map.mesh(id).unwrap();
        let p = mesh.face_points(top)[0];
        assert!((mesh.corner_uv(top, 0, tex) - DVec2::new(p.x, p.z) / (tex * 2.0)).length() < 1e-5);

        assert_eq!(texture_ops::mesh_uv(&mut state, &faces, MeshUvKind::Clear, VIEW), 6);
        assert!(state.doc.map.mesh(id).unwrap().faces.iter().all(|f| f.uvs.is_empty()));
        let before: Vec<DVec2> = (0..4).map(|k| state.doc.map.mesh(id).unwrap().corner_uv(top, k, tex)).collect();
        texture_ops::mesh_uv(&mut state, &faces, MeshUvKind::Bake, VIEW);
        let mesh = state.doc.map.mesh(id).unwrap();
        assert!(mesh.faces.iter().all(|f| f.uvs.len() == f.indices.len()));
        assert!((0..4).all(|k| (mesh.corner_uv(top, k, tex) - before[k]).length() < 1e-5), "baking keeps the look");
    }

    #[test]
    fn projection_names_round_trip() {
        for k in MeshUvKind::ALL {
            assert_eq!(MeshUvKind::from_name(k.label()), Some(k));
        }

        assert_eq!(MeshUvKind::from_name("cylinder"), Some(MeshUvKind::Cylinder(1)));
        assert_eq!(MeshUvKind::from_name("planar_x"), Some(MeshUvKind::PlanarAxis(0)));
        assert_eq!(MeshUvKind::from_name("reset_to_world"), Some(MeshUvKind::World));
        assert_eq!(MeshUvKind::from_name("bake"), Some(MeshUvKind::Bake));
    }

    #[test]
    fn adjusting_stitched_corners_keeps_them_together() {
        let grid = mesh_shapes::grid(&Aabb::new(DVec3::ZERO, DVec3::new(128.0, 0.0, 128.0)), 2, 2, "dev/grey");
        let (mut state, id, faces) = scene(grid);
        texture_ops::mesh_uv(&mut state, &faces, MeshUvKind::PlanarAxis(1), VIEW);
        let mesh = state.doc.map.mesh(id).unwrap();
        let middle = mesh.vertices.iter().position(|v| (*v - DVec3::new(64.0, 0.0, 64.0)).length() < 1e-6).unwrap() as u32;
        let (f, k) = (0..4).find_map(|f| mesh.faces[f].indices.iter().position(|v| *v == middle).map(|k| (f, k))).unwrap();
        let stitched = texture_ops::stitched_corners(&state.doc.map, &faces, (id, f, k));
        assert_eq!(stitched.len(), 4, "the middle vertex is shared by four faces with one UV");

        let undo = state.doc.history.undo_labels().count();
        texture_ops::adjust_corners(&mut state, &stitched, UvAdjust::Move(DVec2::new(0.25, 0.0)), false);
        assert_eq!(state.doc.history.undo_labels().count(), undo + 1, "one undo step");
        let uvs: Vec<DVec2> = stitched.iter().map(|c| texture_ops::corner_uv(&state.doc.map, *c).unwrap()).collect();
        assert!(uvs.iter().all(|u| close(*u, uvs[0])), "{uvs:?}");
        let before = 64.0 / texture_ops::tex_size(&state, "dev/grey").x;
        assert!((uvs[0].x - before - 0.25).abs() < 1e-6);

        // All corners of the faces flip together, shared corners stay shared.
        let all = texture_ops::all_corners(&state.doc.map, &faces);
        assert_eq!(all.len(), 16);
        texture_ops::adjust_corners(&mut state, &all, UvAdjust::FlipU, false);
        let flipped: Vec<DVec2> = stitched.iter().map(|c| texture_ops::corner_uv(&state.doc.map, *c).unwrap()).collect();
        assert!(flipped.iter().all(|u| close(*u, flipped[0])));
    }

    #[test]
    fn gizmo_parts_under_the_pointer() {
        let c = Pos2::new(100.0, 100.0);
        assert_eq!(gizmo_part_at(c, c), Some(GizmoPart::Free));
        assert_eq!(gizmo_part_at(c, c + Vec2::new(40.0, 2.0)), Some(GizmoPart::U));
        assert_eq!(gizmo_part_at(c, c + Vec2::new(-1.0, -40.0)), Some(GizmoPart::V));
        assert_eq!(gizmo_part_at(c, c + Vec2::new(40.0, 40.0)), None);
    }
}
