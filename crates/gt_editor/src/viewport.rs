use egui::{Align2, Color32, CursorIcon, FontId, Key, PointerButton, Pos2, Rect, Response, Sense, Ui, Vec2};
use gt_core::{Aabb, DVec3, NodeId, Plane, Ray};
use gt_doc::ops;
use gt_geom::Brush;
use gt_render::{Frame, FrameParams, LineVertex, Renderer, ViewTarget};

use crate::camera::{Camera, ViewKind};
use crate::commands::Action;
use crate::panels::DndPayload;
use crate::picking::{self, Hit};
use crate::scene::{SceneCache, v3};
use crate::state::EditorState;
use crate::tools::{Drag, ToolKind, line_ray_param};
use crate::toolset::ToolSet;

pub struct ViewCtx<'a> {
    pub state: &'a mut EditorState,
    pub renderer: &'a mut Renderer,
    pub scene: &'a SceneCache,
    pub actions: &'a mut Vec<Action>,
    pub tools: &'a mut ToolSet,
}

pub struct Viewport {
    pub camera: Camera,
    target: Option<ViewTarget>,
    drag: Option<Drag>,
    pub rect: Rect,
    pub hovered: bool,
    press_modifiers: egui::Modifiers,
    pub(crate) pixels_per_point: f32,
}

// Linear values, the targets are sRGB.
const BG_LIT: [f64; 4] = [0.18, 0.28, 0.45, 1.0];
const DROP_COLOR: Color32 = crate::theme::CYAN;

impl Viewport {
    pub fn new(kind: ViewKind) -> Self {
        Self {
            camera: Camera::new(kind),
            target: None,
            drag: None,
            rect: Rect::NOTHING,
            hovered: false,
            press_modifiers: Default::default(),
            pixels_per_point: 1.0,
        }
    }

    pub fn kind(&self) -> ViewKind {
        self.camera.kind
    }

    pub fn ui(&mut self, ui: &mut Ui, cx: &mut ViewCtx) {
        let rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        self.rect = rect;
        self.hovered = response.hovered();
        if rect.width() < 2.0 || rect.height() < 2.0 {
            return;
        }

        let ppp = ui.ctx().pixels_per_point();
        self.pixels_per_point = ppp;
        cx.renderer.ensure_target(&mut self.target, [(rect.width() * ppp).round() as u32, (rect.height() * ppp).round() as u32]);

        cx.tools.settle(cx.state);
        if self.drag.is_some() && !self.is_camera_drag() {
            // A tool picked mid drag (a shortcut, the toolbar) stops the select handling below, so close its edit here.
            // A drag whose transaction was closed elsewhere (undo, a tab switch) must not go on editing outside it.
            let switched = cx.state.tool != ToolKind::Select;
            if switched || !cx.state.doc.in_transaction() {
                self.drag = None;
                cx.state.drag_preview = None;
                if switched {
                    cx.state.doc.commit();
                }
            }
        }

        self.handle_camera(ui, &response, cx);
        self.handle_keys(ui, cx);
        if cx.state.tool == ToolKind::Select || self.is_camera_drag() {
            self.handle_select_tool(ui, &response, cx);
        } else {
            let hover = response.hover_pos();
            cx.tools.viewport_input(ui, &response, &self.camera, rect, hover, cx.state);
        }

        self.handle_drop(ui, &response, cx);
        self.update_cursor_world(&response, cx);

        let tool_lines = cx.tools.lines(&self.camera, rect, cx.state);
        self.render(cx, &tool_lines);
        if let Some(target) = &self.target {
            ui.painter().image(target.texture_id, rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        }

        self.paint_overlay(ui, cx);
        cx.tools.paint_overlay(ui, &self.camera, rect, cx.state);

        // Right click applies materials in the texture tool.
        if cx.state.tool != ToolKind::Texture {
            response.context_menu(|ui| context_menu(ui, cx));
        }
    }

    fn is_camera_drag(&self) -> bool {
        matches!(self.drag, Some(Drag::Look | Drag::Pan | Drag::Orbit { .. }))
    }

    /// The right mouse look in the 3D view, while it runs WASD, Q and E fly the camera.
    pub fn is_flying(&self) -> bool {
        matches!(self.drag, Some(Drag::Look))
    }

    fn handle_camera(&mut self, ui: &Ui, response: &Response, cx: &mut ViewCtx) {
        let rect = self.rect;
        let (delta, scroll, modifiers, dt, press_origin) =
            ui.input(|i| (i.pointer.delta(), i.smooth_scroll_delta.y, i.modifiers, i.stable_dt as f64, i.pointer.press_origin()));
        let scroll = if cx.tools.mesh.takes_wheel() && cx.state.tool == ToolKind::Mesh { 0.0 } else { scroll };
        let (invert_y, look_sensitivity, fly_speed) = (cx.state.prefs.invert_y, cx.state.prefs.look_sensitivity, cx.state.prefs.fly_speed);
        match self.camera.kind {
            ViewKind::Perspective => {
                if response.drag_started_by(PointerButton::Secondary) {
                    self.drag = Some(Drag::Look);
                } else if response.drag_started_by(PointerButton::Middle) {
                    self.drag = Some(Drag::Pan);
                } else if response.drag_started_by(PointerButton::Primary) && modifiers.alt && cx.state.doc.selection.is_empty() {
                    let pivot = press_origin
                        .and_then(|p| picking::pick(cx.state, &self.camera.ray(rect, p)))
                        .map(|h| h.point)
                        .unwrap_or(self.camera.position + self.camera.forward() * 256.0);
                    self.drag = Some(Drag::Orbit { pivot });
                }

                let invert = if invert_y { -1.0 } else { 1.0 };
                match self.drag {
                    Some(Drag::Look) => self.camera.rotate(Vec2::new(delta.x, delta.y * invert), look_sensitivity),
                    Some(Drag::Pan) => self.camera.pan(delta * 1.5, rect),
                    Some(Drag::Orbit { pivot }) => self.camera.orbit(pivot, delta, look_sensitivity),
                    _ => {}
                }

                let brush_tool = matches!(cx.state.tool, ToolKind::Scatter | ToolKind::Blend | ToolKind::Sculpt | ToolKind::Paint);
                // egui turns Ctrl+wheel into zoom and leaves the smoothed scroll at zero, so read the raw notches.
                let resize = if self.hovered && brush_tool { ui.input(|i| wheel_notches(i, |m| m.command)) } else { 0.0 };
                if resize != 0.0 {
                    let factor = 1.08f64.powf(resize as f64);
                    match cx.state.tool {
                        ToolKind::Scatter => cx.state.prefs.scatter.radius = (cx.state.prefs.scatter.radius * factor).clamp(8.0, 16384.0),
                        ToolKind::Blend => cx.state.blend.radius = (cx.state.blend.radius * factor).clamp(2.0, 16384.0),
                        _ => cx.state.sculpt.radius = (cx.state.sculpt.radius * factor).clamp(2.0, 16384.0),
                    }
                } else if self.hovered && scroll != 0.0 && !(brush_tool && modifiers.command) && !(cx.state.tool == ToolKind::Texture && modifiers.alt) {
                    self.camera.position += self.camera.forward() * (scroll as f64) * fly_speed * 0.004;
                }

                // Like TrenchBroom, the fly keys only steer while the right mouse look is held, otherwise they are tool shortcuts.
                if self.is_flying() && !ui.ctx().egui_wants_keyboard_input() && !modifiers.command {
                    let mut dir = DVec3::ZERO;
                    ui.input(|i| {
                        if i.key_down(Key::W) {
                            dir += self.camera.forward();
                        }

                        if i.key_down(Key::S) {
                            dir -= self.camera.forward();
                        }

                        if i.key_down(Key::D) {
                            dir += self.camera.right();
                        }

                        if i.key_down(Key::A) {
                            dir -= self.camera.right();
                        }

                        if i.key_down(Key::E) {
                            dir += DVec3::Y;
                        }

                        if i.key_down(Key::Q) {
                            dir -= DVec3::Y;
                        }
                    });
                    if dir != DVec3::ZERO {
                        let boost = if modifiers.shift { 3.0 } else { 1.0 };
                        self.camera.position += dir.normalize() * fly_speed * boost * dt.min(0.1);
                        ui.ctx().request_repaint();
                    }
                }
            }
            _ => {
                if response.drag_started_by(PointerButton::Secondary) || response.drag_started_by(PointerButton::Middle) {
                    self.drag = Some(Drag::Pan);
                }

                if let Some(Drag::Pan) = self.drag {
                    self.camera.pan(delta, rect);
                }

                if self.hovered
                    && scroll != 0.0
                    && let Some(pos) = response.hover_pos()
                {
                    self.camera.zoom_at(rect, pos, (1.0015f64).powf(scroll as f64));
                }
            }
        }

        if self.is_camera_drag() && (response.drag_stopped() || !ui.input(|i| i.pointer.any_down())) {
            self.drag = None;
        }
    }

    fn handle_keys(&mut self, ui: &Ui, cx: &mut ViewCtx) {
        if !self.hovered || ui.ctx().egui_wants_keyboard_input() || cx.state.tool != ToolKind::Select {
            return;
        }

        let grid = cx.state.grid;
        let (right, up) = match self.camera.kind {
            ViewKind::Perspective => {
                let f = self.camera.forward();
                let flat = DVec3::new(f.x, 0.0, f.z);
                let major = |v: DVec3| {
                    let mut out = DVec3::ZERO;
                    let i = if v.x.abs() > v.z.abs() { 0 } else { 2 };
                    out[i] = v[i].signum();
                    out
                };
                (major(self.camera.right()), major(flat))
            }
            k => {
                let (r, u, _) = k.axes();
                (r, u)
            }
        };
        ui.input(|i| {
            let mut offset = DVec3::ZERO;
            if i.key_pressed(Key::ArrowLeft) {
                offset -= right;
            }

            if i.key_pressed(Key::ArrowRight) {
                offset += right;
            }

            if i.key_pressed(Key::ArrowUp) {
                offset += up;
            }

            if i.key_pressed(Key::ArrowDown) {
                offset -= up;
            }

            if i.key_pressed(Key::PageUp) {
                offset += if self.camera.kind == ViewKind::Perspective { DVec3::Y } else { -self.camera.forward() };
            }

            if i.key_pressed(Key::PageDown) {
                offset -= if self.camera.kind == ViewKind::Perspective { DVec3::Y } else { -self.camera.forward() };
            }

            if offset != DVec3::ZERO {
                cx.actions.push(Action::Nudge(offset * grid));
            }
        });
    }

    fn pick(&self, cx: &ViewCtx, pos: Pos2) -> Option<Hit> {
        picking::pick(cx.state, &self.camera.ray(self.rect, pos))
    }

    fn update_cursor_world(&self, response: &Response, cx: &mut ViewCtx) {
        let Some(pos) = response.hover_pos() else { return };
        let world = match self.camera.kind {
            ViewKind::Perspective => {
                let ray = self.camera.ray(self.rect, pos);
                match picking::pick(cx.state, &ray) {
                    Some(h) => h.point + h.normal * 0.01,
                    None => ray.intersect_plane(&Plane::new(DVec3::Y, 0.0)).filter(|t| *t > 0.0 && *t < 8192.0).map(|t| ray.at(t)).unwrap_or(ray.at(256.0)),
                }
            }
            _ => {
                let mut p = self.camera.screen_to_plane(self.rect, pos);
                let axis = self.camera.kind.depth_axis();
                let lb = cx.state.last_bounds;
                p[axis] = if lb.is_empty() { 0.0 } else { lb.min[axis] };
                p
            }
        };
        cx.state.cursor_world = Some(world);
    }

    fn handle_select_tool(&mut self, ui: &Ui, response: &Response, cx: &mut ViewCtx) {
        let rect = self.rect;
        let (modifiers, press_origin, pointer) = ui.input(|i| (i.modifiers, i.pointer.press_origin(), i.pointer.interact_pos()));
        let open_groups = cx.state.open_groups.clone();

        if response.clicked()
            && let Some(pos) = response.interact_pointer_pos()
            && crate::transform_gizmo::hit(cx.state, &self.camera, rect, pos).is_none()
        {
            let hit = self.pick(cx, pos);
            let map = &cx.state.doc.map;
            match hit {
                Some(h) if modifiers.shift && h.face.is_some() => {
                    let (id, face) = (h.node, h.face.unwrap());
                    cx.state.doc.select(|_, s| {
                        if modifiers.command {
                            s.toggle_face(id, face);
                        } else {
                            s.clear();
                            s.select_face(id, face);
                        }
                    });
                }

                // With the UV editor open, ctrl+click grabs every face of the object so all sides are
                // edited at once. Repeating on another object adds its faces.
                Some(h) if cx.state.uv_panel_open && modifiers.command && h.face.is_some() => {
                    let id = h.node;
                    let count = map.brush(id).map(|b| b.faces.len()).or_else(|| map.mesh(id).map(|m| m.faces.len())).unwrap_or(0);
                    cx.state.outliner_reveal = Some(id);
                    cx.state.doc.select(|_, s| {
                        for f in 0..count {
                            s.select_face(id, f);
                        }
                    });
                }
                Some(h) => {
                    let target = map.click_target(h.node, &open_groups);
                    cx.state.last_bounds = map.bounds(target);
                    cx.state.outliner_reveal = Some(target);
                    cx.state.doc.select(|_, s| {
                        if modifiers.command {
                            s.toggle_node(target);
                        } else {
                            s.clear();
                            s.select_node(target);
                        }
                    });
                }
                None => {
                    if !modifiers.command {
                        cx.state.doc.select(|_, s| s.clear());
                    }
                }
            }
        }

        if response.double_clicked()
            && let Some(h) = response.interact_pointer_pos().and_then(|p| self.pick(cx, p))
        {
            // A click picks the object, a double click widens that to the whole group around it.
            let map = &cx.state.doc.map;
            let group = map.selection_target(h.node, &open_groups);
            if group != map.click_target(h.node, &open_groups) {
                cx.state.last_bounds = map.bounds(group);
                cx.state.outliner_reveal = Some(group);
                cx.state.doc.select(|_, s| {
                    s.clear();
                    s.select_node(group);
                });
                let name = cx.state.doc.map.get(group).map(|n| n.name()).unwrap_or_default();
                cx.state.set_status(format!("Selected group {name}"));
            } else if let Some(e) = map.owning_entity(h.node) {
                let brushes = map.get(e).map(|n| n.children.clone()).unwrap_or_default();
                cx.state.outliner_reveal = Some(h.node);
                cx.state.doc.select(|_, s| {
                    s.clear();
                    s.nodes.extend(brushes);
                });
            }
        }

        if response.drag_started_by(PointerButton::Primary)
            && !(modifiers.alt && self.camera.kind == ViewKind::Perspective && cx.state.doc.selection.is_empty())
        {
            self.press_modifiers = modifiers;
            if let Some(origin) = press_origin {
                self.drag = self.begin_primary_drag(origin, modifiers, cx);
            }
        }

        if let (Some(drag), Some(pos)) = (self.drag.clone(), pointer)
            && response.dragged_by(PointerButton::Primary)
        {
            self.update_primary_drag(&drag, pos, modifiers, cx);
        }

        if (response.drag_stopped_by(PointerButton::Primary) || (self.drag.is_some() && !self.is_camera_drag() && !ui.input(|i| i.pointer.primary_down())))
            && let Some(drag) = self.drag.take()
        {
            if matches!(drag, Drag::CreateBrush { .. } | Drag::CreateBrush2d { .. }) {
                let b = cx.state.doc.map.bounds_of(cx.state.doc.selection.nodes.iter().copied());
                if !b.is_empty() {
                    cx.state.last_bounds = b;
                    cx.tools.last_brush = Some(b);
                }
            }

            cx.state.drag_preview = None;
            cx.state.doc.commit();
        }

        if self.drag.is_none()
            && self.camera.kind.is_2d()
            && let Some(pos) = response.hover_pos()
            && let Some((normal, _)) = self.edge_under_cursor(pos, cx)
        {
            let icon = if normal.dot(self.camera.kind.axes().0).abs() > 0.5 { CursorIcon::ResizeHorizontal } else { CursorIcon::ResizeVertical };
            ui.ctx().set_cursor_icon(icon);
        }

        let _ = rect;
    }

    fn begin_primary_drag(&mut self, origin: Pos2, modifiers: egui::Modifiers, cx: &mut ViewCtx) -> Option<Drag> {
        if let Some(handle) = crate::gizmos::handle_at(cx.state, &self.camera, self.rect, origin)
            && let Some(start) = crate::gizmos::handle_position(cx.state, handle)
        {
            cx.state.doc.begin("Edit Gizmo");
            return Some(Drag::Gizmo { handle, start });
        }

        if let Some(drag) = crate::transform_gizmo::begin(cx.state, &self.camera, self.rect, origin) {
            return Some(Drag::Transform(drag));
        }

        if self.camera.kind.is_2d()
            && !cx.state.doc.selection.nodes.is_empty()
            && let Some((normal, faces)) = self.edge_under_cursor(origin, cx)
        {
            let origin_world = self.camera.screen_to_plane(self.rect, origin);
            cx.state.doc.begin("Resize Brushes");
            return Some(Drag::FaceResize { faces, origin: origin_world, normal });
        }

        let state = &mut *cx.state;
        let map = &state.doc.map;
        let ray = self.camera.ray(self.rect, origin);
        let open_groups = state.open_groups.clone();
        let is_selected = |id: NodeId| {
            let target = map.click_target(id, &open_groups);
            state.doc.selection.nodes.contains(&target) || map.ancestors(id).iter().any(|a| state.doc.selection.nodes.contains(a))
        };
        let hit = if self.camera.kind.is_2d() {
            // A 2D view sees through everything, so a selection under other objects (a floor under its ceiling) still drags.
            let hits = picking::pick_all(state, &ray);
            hits.iter().copied().find(|h| is_selected(h.node)).or(hits.first().copied())
        } else {
            picking::pick(state, &ray)
        };

        if let Some(h) = hit {
            if self.camera.kind == ViewKind::Perspective
                && modifiers.shift
                && is_selected(h.node)
                && let (Some(face), Some(brush)) = (h.face, map.brush(h.node))
            {
                let plane = brush.faces[face].plane;
                let faces = coplanar_selected_faces(state, &plane);
                state.doc.begin(if modifiers.command { "Extrude" } else { "Resize Brushes" });
                return Some(Drag::FaceResize { faces, origin: h.point, normal: plane.normal });
            }

            // Shift on a selected brush face in 3D resizes (above), anywhere else it moves locked to the main axis.
            if is_selected(h.node) {
                let plane = match self.camera.kind {
                    ViewKind::Perspective => {
                        if modifiers.alt {
                            let f = self.camera.forward();
                            Plane::from_point_normal(h.point, DVec3::new(f.x, 0.0, f.z).normalize_or(DVec3::Z))
                        } else {
                            Plane::from_point_normal(h.point, DVec3::Y)
                        }
                    }
                    k => Plane::from_point_normal(h.point, -k.axes().2),
                };
                let axis = (self.camera.kind == ViewKind::Perspective && modifiers.alt).then_some(DVec3::Y);
                state.doc.begin(if modifiers.command { "Duplicate Objects" } else { "Move Objects" });
                return Some(Drag::Move { start: h.point, plane, axis, duplicate: modifiers.command });
            }
        }

        match self.camera.kind {
            ViewKind::Perspective => {
                let (start, normal) = match hit {
                    Some(h) => (h.point, h.normal),
                    None => {
                        let t = ray.intersect_plane(&Plane::new(DVec3::Y, 0.0)).filter(|t| *t > 0.0)?;
                        (ray.at(t), DVec3::Y)
                    }
                };
                let axis = gt_core::major_axis(normal);
                let mut n = DVec3::ZERO;
                n[axis] = normal[axis].signum();
                state.doc.begin("Create Brush");
                Some(Drag::CreateBrush { start, normal: n, anchor: state.snap_scalar(start[axis]) })
            }
            _ => {
                let start = self.camera.screen_to_plane(self.rect, origin);
                state.doc.begin("Create Brush");
                Some(Drag::CreateBrush2d { start })
            }
        }
    }

    fn update_primary_drag(&mut self, drag: &Drag, pos: Pos2, modifiers: egui::Modifiers, cx: &mut ViewCtx) {
        let rect = self.rect;
        let ray = self.camera.ray(rect, pos);
        let state = &mut *cx.state;
        let opts = state.opts();
        match drag {
            Drag::Gizmo { handle, start } => {
                if let Some(status) = crate::gizmos::drag(state, *handle, *start, &self.camera, rect, pos, modifiers) {
                    state.set_status(status);
                }
            }
            Drag::Transform(drag) => {
                if let Some(status) = crate::transform_gizmo::drag(state, drag, &self.camera, rect, pos, modifiers) {
                    state.set_status(status);
                }
            }
            Drag::Move { start, plane, axis, duplicate } => {
                let Some(t) = ray.intersect_plane(plane) else { return };
                let mut delta = ray.at(t) - *start;
                if let Some(a) = axis {
                    delta = *a * delta.dot(*a);
                }

                if modifiers.shift {
                    let i = gt_core::major_axis(delta);
                    let keep = delta[i];
                    delta = DVec3::ZERO;
                    delta[i] = keep;
                }

                let delta = state.snap(delta);
                state.doc.reset_transaction();
                if *duplicate {
                    state.drag_preview = None;
                    if delta == DVec3::ZERO {
                        return;
                    }

                    state.doc.edit("Duplicate", |m, s| ops::duplicate_selection(m, s, delta, opts));
                } else {
                    // Render the moved geometry from its pre-drag shape translated on the GPU, so heavy
                    // meshes are not re-tessellated every frame.
                    let nodes = state.doc.selection.geometry(&state.doc.map).into_iter().collect();
                    state.drag_preview = Some(crate::state::DragPreview { nodes, offset: delta });
                    if delta == DVec3::ZERO {
                        return;
                    }

                    state.doc.edit("Move", |m, s| ops::translate_selection(m, s, delta, opts));
                }

                state.set_status(format!("Move {:.3} {:.3} {:.3}", delta.x, delta.y, delta.z));
            }
            Drag::CreateBrush { start, normal, anchor } => {
                let axis = gt_core::major_axis(*normal);
                let mut plane_n = DVec3::ZERO;
                plane_n[axis] = 1.0;
                let Some(t) = ray.intersect_plane(&Plane::new(plane_n, *anchor)) else { return };
                if t < 0.0 {
                    return;
                }

                let current = ray.at(t);
                let grid = state.grid;
                let mut a = state.snap(*start);
                let mut b = state.snap(current);
                for i in 0..3 {
                    if i == axis {
                        a[i] = *anchor;
                        b[i] = *anchor + normal[axis] * grid;
                    } else {
                        // Include the grid cell under the cursor, like TrenchBroom.
                        (a[i], b[i]) = cell_span(start[i], current[i], grid, state.snap);
                    }
                }

                self.replace_created_brush(Aabb::new(a, b), state);
            }
            Drag::CreateBrush2d { start } => {
                let current = self.camera.screen_to_plane(rect, pos);
                let depth_axis = self.camera.kind.depth_axis();
                let grid = state.grid;
                let mut a = DVec3::ZERO;
                let mut b = DVec3::ZERO;
                for i in 0..3 {
                    if i == depth_axis {
                        // The depth of the last brush drawn, not of whatever was clicked last.
                        let lb = cx.tools.last_brush.unwrap_or(Aabb::new(DVec3::ZERO, DVec3::splat(64.0)));
                        (a[i], b[i]) = if lb.is_empty() || lb.size()[i] < 1e-6 { (0.0, grid) } else { (lb.min[i], lb.max[i]) };
                    } else {
                        (a[i], b[i]) = cell_span(start[i], current[i], grid, state.snap);
                    }
                }

                self.replace_created_brush(Aabb::new(a, b), state);
            }
            Drag::FaceResize { faces, origin, normal } => {
                let dist = match self.camera.kind {
                    ViewKind::Perspective => line_ray_param(*origin, *normal, &ray),
                    _ => Some((self.camera.screen_to_plane(rect, pos) - *origin).dot(*normal)),
                };
                let Some(dist) = dist else { return };
                let dist = state.snap_scalar(dist);
                state.doc.reset_transaction();
                if dist.abs() < 1e-9 {
                    return;
                }

                let faces = faces.clone();
                let normal = *normal;
                let extrude = self.press_modifiers.command && self.camera.kind == ViewKind::Perspective;
                let uv_lock = opts.uv_lock;
                state.doc.edit("Resize", |m, s| {
                    for (id, face) in &faces {
                        let Some(brush) = m.brush(*id).cloned() else { continue };
                        if extrude && dist > 0.0 {
                            if let Ok(nb) = gt_geom::csg::extrude_face(&brush, *face, dist) {
                                let parent = m.get(*id).and_then(|n| n.parent).unwrap_or(m.default_layer());
                                let new_id = m.insert(parent, gt_doc::NodeKind::Brush(nb));
                                s.nodes.insert(new_id);
                            }
                        } else if let Ok(nb) = brush.move_face(*face, normal * dist, uv_lock)
                            && let Some(slot) = m.brush_mut(*id)
                        {
                            *slot = nb;
                        }
                    }
                });
                state.set_status(format!("Resize {dist:.3}"));
            }
            _ => {}
        }
    }

    fn replace_created_brush(&self, bounds: Aabb, state: &mut EditorState) {
        let mat = state.current_material.clone();
        let parent = state.insert_parent();
        state.doc.reset_transaction();
        if let Ok(brush) = Brush::from_aabb(&bounds, &mat) {
            state.doc.edit("Create Brush", |m, s| {
                let id = ops::create_brush(m, parent, brush);
                s.clear();
                s.select_node(id);
            });
            let size = bounds.size();
            state.set_status(format!("Brush {} x {} x {}", size.x, size.y, size.z));
        }
    }

    /// 2D views: the side of the selection bounds under the cursor, with the faces lying on it.
    fn edge_under_cursor(&self, pos: Pos2, cx: &ViewCtx) -> Option<(DVec3, Vec<(NodeId, usize)>)> {
        let state = &cx.state;
        let brushes = state.doc.selection.brushes(&state.doc.map);
        if brushes.is_empty() {
            return None;
        }

        let bounds = state.doc.map.bounds_of(brushes.iter().copied());
        let (r, u, _) = self.camera.kind.axes();
        let min = self.camera.project(self.rect, bounds.min)?;
        let max = self.camera.project(self.rect, bounds.max)?;
        let screen = Rect::from_two_pos(min, max);
        const TOL: f32 = 5.0;
        if !screen.expand(TOL).contains(pos) {
            return None;
        }

        // View axes can point along negative world axes, so the side of the screen the bounds minimum lands on picks the face.
        let (r, u) = (r.abs(), u.abs());
        let candidates = [
            ((pos.x - screen.min.x).abs(), if min.x <= max.x { -r } else { r }),
            ((pos.x - screen.max.x).abs(), if min.x <= max.x { r } else { -r }),
            ((pos.y - screen.min.y).abs(), if min.y <= max.y { -u } else { u }),
            ((pos.y - screen.max.y).abs(), if min.y <= max.y { u } else { -u }),
        ];
        let (dist, normal) = candidates.into_iter().min_by(|a, b| a.0.total_cmp(&b.0))?;
        if dist > TOL {
            return None;
        }

        let extreme = if normal.element_sum() > 0.0 { bounds.max.dot(normal) } else { bounds.min.dot(normal) };
        let mut faces = Vec::new();
        for id in brushes {
            let Some(b) = state.doc.map.brush(id) else { continue };
            for (fi, f) in b.faces.iter().enumerate() {
                if f.plane.normal.dot(normal) > 0.9999 && (f.plane.dist - extreme).abs() < 0.01 {
                    faces.push((id, fi));
                }
            }
        }

        (!faces.is_empty()).then_some((normal, faces))
    }

    /// Shows where a dragged payload lands: the faces a material goes on, or the surface point an entity rests on.
    fn paint_drop_target(&self, ui: &Ui, cx: &ViewCtx, payload: &DndPayload, pos: egui::Pos2) {
        let Some(hit) = picking::pick(cx.state, &self.camera.ray(self.rect, pos)) else { return };
        let painter = ui.painter().with_clip_rect(self.rect);
        let stroke = egui::Stroke::new(2.0, DROP_COLOR);
        match payload {
            DndPayload::Material(_) => {
                let map = &cx.state.doc.map;
                let whole = ui.input(|i| i.modifiers.shift);
                let polygons: Vec<Vec<DVec3>> = match (map.brush(hit.node), map.mesh(hit.node), hit.face) {
                    (Some(b), _, Some(face)) if !whole => vec![b.face_points(face)],
                    (Some(b), _, Some(_)) => (0..b.faces.len()).map(|f| b.face_points(f)).collect(),
                    (_, Some(m), Some(face)) => {
                        let faces: Vec<usize> = if whole { (0..m.faces.len()).collect() } else { vec![face] };
                        faces.iter().filter_map(|f| m.faces.get(*f)).map(|f| f.indices.iter().map(|i| m.vertices[*i as usize]).collect()).collect()
                    }
                    _ => Vec::new(),
                };
                for polygon in polygons {
                    let Some(points) = polygon.iter().map(|p| self.camera.project(self.rect, *p)).collect::<Option<Vec<_>>>() else { continue };
                    painter.add(egui::Shape::closed_line(points.clone(), stroke));
                    if map.brush(hit.node).is_some() {
                        painter.add(egui::Shape::convex_polygon(points, DROP_COLOR.gamma_multiply(0.25), egui::Stroke::NONE));
                    }
                }
            }
            DndPayload::Entities(_) | DndPayload::Model(_) => {
                let at = crate::commands::drop_target(cx.state, &self.camera.ray(self.rect, pos), self.camera.kind).map_or(hit.point, |(p, _)| p);
                if let Some(point) = self.camera.project(self.rect, at) {
                    painter.circle(point, 5.0, DROP_COLOR.gamma_multiply(0.4), stroke);
                }
            }
        }
    }

    fn handle_drop(&mut self, ui: &Ui, response: &Response, cx: &mut ViewCtx) {
        let Some(payload) = response.dnd_release_payload::<DndPayload>() else {
            if let Some(payload) = response.dnd_hover_payload::<DndPayload>() {
                ui.painter().rect_stroke(self.rect.shrink(1.0), 0.0, egui::Stroke::new(2.0, DROP_COLOR), egui::StrokeKind::Inside);
                ui.ctx().set_cursor_icon(CursorIcon::Copy);
                if let Some(pos) = response.hover_pos() {
                    self.paint_drop_target(ui, cx, &payload, pos);
                }
            }

            return;
        };
        let Some(pos) = response.hover_pos() else { return };
        let ray = self.camera.ray(self.rect, pos);
        let hit = picking::pick(cx.state, &ray);
        match payload.as_ref() {
            DndPayload::Material(name) => {
                let name = name.clone();
                cx.state.current_material = name.clone();
                // Alt while dropping on a surface lays the material as a decal sheet instead of painting it.
                if ui.input(|i| i.modifiers.alt) {
                    if let Some(h) = hit {
                        cx.actions.push(Action::CreateDecal { material: name, at: h.point, normal: h.normal, size: None });
                    }

                    return;
                }

                let apply_whole = ui.input(|i| i.modifiers.shift);
                match hit {
                    Some(Hit { node, face: Some(face), .. }) => {
                        cx.state.doc.edit("Apply Material", |m, _| {
                            if let Some(b) = m.brush_mut(node) {
                                if apply_whole {
                                    b.set_material(&name);
                                } else {
                                    b.faces[face].data.material = name.clone();
                                }
                            } else if let Some(mesh) = m.mesh_mut(node) {
                                if apply_whole {
                                    mesh.faces.iter_mut().for_each(|f| f.data.material = name.clone());
                                } else if let Some(f) = mesh.faces.get_mut(face) {
                                    f.data.material = name.clone();
                                }
                            }
                        });
                    }
                    Some(Hit { node, face: None, .. }) if cx.state.doc.map.terrain(node).is_some() => {
                        // Dropping on a terrain assigns the material to the layer being painted.
                        let blend = cx.state.tool == ToolKind::Blend;
                        let layer = if blend { cx.state.blend.layer } else { cx.state.sculpt.layer as usize };
                        let slot = cx.state.doc.edit("Set Terrain Layer", |m, _| m.terrain_mut(node).map(|t| set_terrain_layer(t, layer, &name)));
                        if let Some(slot) = slot.filter(|s| *s != layer) {
                            if blend {
                                cx.state.blend.layer = slot;
                            } else {
                                cx.state.sculpt.layer = slot as u8;
                            }

                            cx.state.set_status(format!("The terrain had no layer {layer}, {name} was added as layer {slot}"));
                        }
                    }
                    _ => {}
                }
            }
            DndPayload::Entities(classnames) => {
                let (at, normal) = match (self.camera.kind, crate::commands::drop_target(cx.state, &ray, self.camera.kind)) {
                    (_, Some(target)) => target,
                    (ViewKind::Perspective, None) => (cx.state.cursor_world.unwrap_or(ray.at(256.0)), None),
                    (_, None) => (self.camera.screen_to_plane(self.rect, pos), None),
                };
                let right = self.camera.right();
                let axis = right.abs().max_position();
                let row = DVec3::AXES[axis] * right[axis].signum();
                cx.actions.push(Action::PlaceEntities { classnames: classnames.clone(), at: Some(at), normal, row });
            }
            DndPayload::Model(path) => {
                let at = match (self.camera.kind, crate::commands::drop_target(cx.state, &ray, self.camera.kind)) {
                    (_, Some((p, _))) => p,
                    (ViewKind::Perspective, None) => cx.state.cursor_world.unwrap_or(ray.at(256.0)),
                    (_, None) => self.camera.screen_to_plane(self.rect, pos),
                };
                cx.actions.push(Action::PlaceModel { path: path.clone(), at: cx.state.snap(at) });
            }
        }
    }

    /// Renders this view's camera into a throwaway target of `size` pixels, for screenshots larger than the docked view.
    /// Without `overlays` it is a beauty shot: no grid, entity boxes, volumes, edges, selection outlines or links.
    pub fn render_offscreen(
        &self,
        renderer: &mut Renderer,
        scene: &SceneCache,
        state: &EditorState,
        size: [u32; 2],
        overlays: bool,
    ) -> Option<image::RgbaImage> {
        let mut target = None;
        renderer.ensure_target(&mut target, size);
        let target = target?;
        let size_points = Vec2::new(size[0] as f32, size[1] as f32);
        let mut params = self.frame_params(size_points, state, 1.0);
        let is_2d = self.camera.kind.is_2d();
        let grid =
            if overlays && is_2d { renderer.upload_lines(&grid_lines(&self.camera, Rect::from_min_size(Pos2::ZERO, size_points), state.grid)) } else { None };
        if !overlays {
            params.grid_size = 0.0;
        }

        let lit = state.prefs.shade == crate::state::Shade::Lit;
        let mut frame = Frame::default();
        frame.overlay_lines.extend(grid.as_ref());
        scene.fill_frame(&mut frame, is_2d, lit, overlays);
        frame.sky = !is_2d && lit;
        renderer.render(&target, &params, &frame);
        let image = renderer.read_target(&target);
        renderer.free_target(target);
        image
    }

    fn frame_params(&self, size: Vec2, state: &EditorState, line_width: f32) -> FrameParams {
        let is_2d = self.camera.kind.is_2d();
        FrameParams {
            view_proj: self.camera.view_proj(size).as_mat4(),
            eye: self.camera.eye().as_vec3(),
            grid_size: if is_2d { 0.0 } else { state.grid as f32 },
            grid_alpha: state.prefs.grid_alpha,
            shade: state.shade_mode(),
            orthographic: is_2d,
            clear: if is_2d {
                crate::theme::linear(crate::theme::GRAY_0)
            } else if state.prefs.shade == crate::state::Shade::Lit {
                BG_LIT
            } else {
                crate::theme::linear(crate::theme::GRAY_1)
            },
            line_width,
        }
    }

    fn render(&mut self, cx: &mut ViewCtx, tool_lines: &[LineVertex]) {
        let Some(target) = &self.target else { return };
        let rect = self.rect;
        let state = &cx.state;
        let is_2d = self.camera.kind.is_2d();
        let params = self.frame_params(rect.size(), state, self.pixels_per_point);
        let grid = if is_2d { cx.renderer.upload_lines(&grid_lines(&self.camera, rect, state.grid)) } else { None };
        let tools = cx.renderer.upload_lines(tool_lines);
        let scene = cx.scene;
        let mut frame = Frame::default();
        if is_2d {
            frame.overlay_lines.extend(grid.as_ref());
        }

        let lit = state.prefs.shade == crate::state::Shade::Lit;
        scene.fill_frame(&mut frame, is_2d, lit, true);
        frame.sky = !is_2d && lit;
        frame.overlay_lines.extend(tools.as_ref());
        cx.renderer.render(target, &params, &frame);
    }

    fn paint_overlay(&self, ui: &Ui, cx: &ViewCtx) {
        let painter = ui.painter_at(self.rect);
        let label_color = Color32::from_rgb(200, 200, 210);
        let text = match self.camera.kind {
            ViewKind::Perspective => format!("{}  {}", self.camera.kind.label(), cx.state.prefs.shade.label()),
            k => format!("{}  zoom {:.2}", k.label(), self.camera.zoom),
        };
        painter.text(self.rect.min + Vec2::new(8.0, 6.0), Align2::LEFT_TOP, text, FontId::proportional(12.0), label_color);

        let state = &cx.state;
        let active = match &self.drag {
            Some(Drag::Gizmo { handle, .. }) => Some(*handle),
            _ => None,
        };
        crate::gizmos::paint_overlay(ui, &self.camera, self.rect, state, active);
        let transforming = match &self.drag {
            Some(Drag::Transform(drag)) => Some(drag.part),
            _ => None,
        };
        crate::transform_gizmo::paint(ui, &self.camera, self.rect, state, transforming);
        if self.camera.kind.is_2d() && !state.doc.selection.nodes.is_empty() {
            let b = state.doc.map.bounds_of(state.doc.selection.nodes.iter().copied());
            if !b.is_empty() {
                let (r, u, _) = self.camera.kind.axes();
                let size = b.size();
                let w = (size * r.abs()).element_sum();
                let h = (size * u.abs()).element_sum();
                if let (Some(p0), Some(p1)) = (self.camera.project(self.rect, b.min), self.camera.project(self.rect, b.max)) {
                    let sr = Rect::from_two_pos(p0, p1);
                    let color = Color32::from_rgb(255, 120, 100);
                    painter.text(Pos2::new(sr.center().x, sr.min.y - 4.0), Align2::CENTER_BOTTOM, fmt_num(w), FontId::monospace(11.0), color);
                    painter.text(Pos2::new(sr.max.x + 6.0, sr.center().y), Align2::LEFT_CENTER, fmt_num(h), FontId::monospace(11.0), color);
                }
            }
        }

        if state.prefs.godot_overlays {
            let ghost = Color32::from_rgba_unmultiplied(115, 184, 255, 230);
            for item in &state.overlay_ghosts.items {
                let points: Vec<Pos2> = item.drawn_bounds().corners().iter().filter_map(|c| self.camera.project(self.rect, *c)).collect();
                if points.len() < 8 {
                    continue;
                }

                // Only boxes big enough on screen get a name, an overview would drown in labels.
                let screen = Rect::from_points(&points);
                if screen.width().max(screen.height()) < 28.0 || !self.rect.intersects(screen) {
                    continue;
                }

                painter.text(Pos2::new(screen.center().x, screen.min.y - 2.0), Align2::CENTER_BOTTOM, item.label(), FontId::proportional(11.0), ghost);
            }
        }

        if self.camera.kind == ViewKind::Perspective {
            for (id, e) in state.doc.map.entities() {
                let selected = state.doc.selection.nodes.contains(&id);
                if !selected || state.doc.map.is_hidden(id) {
                    continue;
                }

                let pos = if state.doc.map.is_point_entity(id) { e.origin } else { state.doc.map.bounds(id).center() };
                if let Some(p) = self.camera.project(self.rect, pos) {
                    painter.text(p + Vec2::new(0.0, -18.0), Align2::CENTER_BOTTOM, e.classname.as_str(), FontId::proportional(12.0), Color32::WHITE);
                }
            }
        }
    }

    pub fn focus(&mut self, bounds: &Aabb) {
        if self.rect.width() > 1.0 {
            self.camera.focus(bounds, self.rect);
        } else {
            self.camera.focus(bounds, Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0)));
        }
    }

    pub fn ray_at(&self, pos: Pos2) -> Ray {
        self.camera.ray(self.rect, pos)
    }

    pub fn target(&self) -> Option<&ViewTarget> {
        self.target.as_ref()
    }
}

/// Wheel movement this frame in notches, from the raw events whose modifiers pass `keep`. egui spreads one notch
/// over several frames of `smooth_scroll_delta` and turns Ctrl+wheel into zoom, so tools that step or resize per
/// notch read the events instead. Positive is away from the user.
pub fn wheel_notches(input: &egui::InputState, keep: impl Fn(egui::Modifiers) -> bool) -> f32 {
    input
        .raw
        .events
        .iter()
        .map(|e| match e {
            egui::Event::MouseWheel { unit, delta, modifiers, .. } if keep(*modifiers) => match unit {
                egui::MouseWheelUnit::Line => delta.y,
                egui::MouseWheelUnit::Point => delta.y / 50.0,
                egui::MouseWheelUnit::Page => delta.y * 3.0,
            },
            _ => 0.0,
        })
        .sum()
}

/// A grid aligned span covering `a` to `b` and the cells under both ends, at least one cell wide. An end a hair
/// past a grid line (pointer to world rounding) counts as on it.
fn cell_span(a: f64, b: f64, grid: f64, snap: bool) -> (f64, f64) {
    let (lo, hi) = (a.min(b), a.max(b));
    let eps = grid * 1e-3;
    let (lo, hi) = if snap { (((lo + eps) / grid).floor() * grid, ((hi - eps) / grid).ceil() * grid) } else { (lo, hi) };
    if (hi - lo).abs() < 1e-6 { (lo, lo + grid) } else { (lo, hi) }
}

fn fmt_num(v: f64) -> String {
    if (v - v.round()).abs() < 1e-6 { format!("{}", v.round() as i64) } else { format!("{v:.3}") }
}

fn coplanar_selected_faces(state: &EditorState, plane: &Plane) -> Vec<(NodeId, usize)> {
    let mut out = Vec::new();
    for id in state.doc.selection.brushes(&state.doc.map) {
        if let Some(b) = state.doc.map.brush(id)
            && let Some(f) = b.find_face_by_plane(plane)
        {
            out.push((id, f));
        }
    }

    out
}

fn grid_lines(cam: &Camera, rect: Rect, grid: f64) -> Vec<LineVertex> {
    let (r, u, _) = cam.kind.axes();
    let (r0, r1, u0, u1) = cam.visible_range(rect);
    let mut spacing = grid.max(0.125);
    while spacing * cam.zoom < 10.0 {
        spacing *= 2.0;
    }

    let depth = cam.kind.depth_axis();
    let mut base = DVec3::ZERO;
    base[depth] = 0.0;
    let mut out = Vec::new();
    let axis_color = |a: DVec3| -> [f32; 4] {
        match gt_core::major_axis(a) {
            0 => [0.45, 0.08, 0.08, 0.8],
            1 => [0.08, 0.4, 0.08, 0.8],
            _ => [0.08, 0.12, 0.5, 0.8],
        }
    };
    let mut push = |along: DVec3, across: DVec3, value: f64, from: f64, to: f64, index: i64| {
        let color = if index == 0 {
            axis_color(across)
        } else if index % 8 == 0 {
            [0.16, 0.16, 0.19, 0.6]
        } else {
            [0.09, 0.09, 0.11, 0.45]
        };
        let p0 = base + along * value + across * from;
        let p1 = base + along * value + across * to;
        out.push(LineVertex { pos: v3(p0), color });
        out.push(LineVertex { pos: v3(p1), color });
    };
    let (i0, i1) = ((r0 / spacing).floor() as i64, (r1 / spacing).ceil() as i64);
    for i in i0..=i1 {
        push(r, u, i as f64 * spacing, u0, u1, (i as f64 * spacing / grid).round() as i64);
    }

    let (j0, j1) = ((u0 / spacing).floor() as i64, (u1 / spacing).ceil() as i64);
    for j in j0..=j1 {
        push(u, r, j as f64 * spacing, r0, r1, (j as f64 * spacing / grid).round() as i64);
    }

    out
}

fn context_menu(ui: &mut Ui, cx: &mut ViewCtx) {
    let has_sel = !cx.state.doc.selection.nodes.is_empty();
    let cursor = cx.state.cursor_world;
    let point_classes: Vec<String> = cx.state.game.point_entities().map(|e| e.classname.clone()).collect();
    let solid_classes: Vec<String> = cx.state.game.solid_entities().map(|e| e.classname.clone()).collect();
    let layers: Vec<(NodeId, String)> = cx.state.doc.map.layers.iter().filter_map(|l| cx.state.doc.map.get(*l).map(|n| (*l, n.name()))).collect();
    let mut out: Vec<Action> = Vec::new();

    ui.menu_button("Create Point Entity", |ui| {
        for c in point_classes {
            if ui.button(&c).clicked() {
                out.push(Action::CreatePointEntity { classname: c, at: cursor });
                ui.close();
            }
        }
    });
    ui.add_enabled_ui(has_sel, |ui| {
        ui.menu_button("Create Brush Entity", |ui| {
            for c in solid_classes {
                if ui.button(&c).clicked() {
                    out.push(Action::CreateBrushEntity(c));
                    ui.close();
                }
            }
        });
    });
    let entries: [Option<(&str, Action)>; 15] = [
        Some(("Move Brushes to World", Action::MoveToWorld)),
        None,
        Some(("Create Prefab from Selection…", Action::CreatePrefab)),
        Some(("Explode Instance", Action::ExplodeInstances)),
        Some(("Open Prefab", Action::OpenPrefab)),
        Some(("Group", Action::Group)),
        Some(("Ungroup", Action::Ungroup)),
        Some(("Hide", Action::HideSelected)),
        Some(("Isolate", Action::IsolateSelected)),
        None,
        Some(("CSG Subtract", Action::CsgSubtract)),
        Some(("CSG Merge", Action::CsgMerge)),
        Some(("CSG Intersect", Action::CsgIntersect)),
        Some(("Hollow", Action::CsgHollow)),
        None,
    ];
    for entry in entries {
        match entry {
            Some((label, action)) => {
                if ui.add_enabled(has_sel, egui::Button::new(label)).clicked() {
                    out.push(action);
                    ui.close();
                }
            }
            None => {
                ui.separator();
            }
        }
    }

    ui.add_enabled_ui(has_sel, |ui| {
        ui.menu_button("Move to Layer", |ui| {
            for (id, name) in layers {
                if ui.button(name).clicked() {
                    out.push(Action::MoveToLayer(id));
                    ui.close();
                }
            }
        });
    });
    cx.actions.extend(out);
}

/// Puts `material` into terrain layer `layer`. A layer past the last one adds a single new layer instead of filling
/// the gap. Returns the slot that was set.
pub fn set_terrain_layer(t: &mut gt_geom::Terrain, layer: usize, material: &str) -> usize {
    if layer < t.layers.len() {
        t.layers[layer].material = material.to_string();
        return layer;
    }

    if t.layers.len() >= gt_geom::heightfield::MAX_LAYERS {
        let last = t.layers.len() - 1;
        t.layers[last].material = material.to_string();
        return last;
    }

    let tile = t.layers.last().map(|l| l.tile).unwrap_or(256.0);
    t.layers.push(gt_geom::TerrainLayer::new(material, tile));
    t.layers.len() - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropping_past_the_last_terrain_layer_adds_one_layer() {
        let mut t = gt_geom::Terrain::new(DVec3::ZERO, [3, 3], 32.0, "grass");
        assert_eq!(set_terrain_layer(&mut t, 3, "sand"), 1);
        assert_eq!(t.layers.iter().map(|l| l.material.as_str()).collect::<Vec<_>>(), ["grass", "sand"]);
        assert_eq!(set_terrain_layer(&mut t, 0, "moss"), 0);
        assert_eq!(t.layers[0].material, "moss");
        for m in ["a", "b", "c"] {
            set_terrain_layer(&mut t, 9, m);
        }

        assert_eq!(t.layers.len(), 4, "never more than four layers");
        assert_eq!(t.layers[3].material, "c", "a drop past four replaces the last layer");
    }

    #[test]
    fn a_drawn_brush_covers_the_cells_under_both_ends() {
        assert_eq!(cell_span(10.0, 20.0, 16.0, true), (0.0, 32.0));
        assert_eq!(cell_span(20.0, 10.0, 16.0, true), (0.0, 32.0), "dragging backwards gives the same span");
        assert_eq!(cell_span(16.0, 48.0, 16.0, true), (16.0, 48.0), "ends on grid lines stay put");
        assert_eq!(cell_span(15.9999, 48.0001, 16.0, true), (16.0, 48.0), "so do ends a rounding error off them");
        assert_eq!(cell_span(3.0, 3.0, 16.0, true), (0.0, 16.0), "a click still makes one cell");
        assert_eq!(cell_span(3.0, 3.0, 16.0, false), (3.0, 19.0));
    }
}
