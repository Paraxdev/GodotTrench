use egui::{Align2, Color32, FontId, Key, PointerButton, Pos2, Rect, Response, Stroke, Ui, Vec2};
use gt_core::{Aabb, DMat4, DVec3, NodeId, Plane};
use gt_doc::ops;
use gt_geom::Brush;
use gt_render::LineVertex;

use crate::camera::{Camera, ViewKind};
use crate::picking;
use crate::scene::v3;
use crate::state::EditorState;
use crate::tools::ToolKind;

const HANDLE_RADIUS: f32 = 7.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipSide {
    Front,
    Back,
    Both,
}

/// Splits the selected brushes by `plane` as one undo step, keeping the part in front of it (where the normal points),
/// behind it or both, and selects what is left.
pub fn clip_selection(state: &mut EditorState, plane: &Plane, side: ClipSide) {
    let brushes = state.doc.selection.brushes(&state.doc.map);
    state.replaced = state.doc.edit("Clip", |m, s| {
        let mut replaced = gt_doc::ops::Replaced::new();
        s.clear();
        for id in brushes {
            let Some(b) = m.brush(id).cloned() else { continue };
            let template = b.planes();
            let cap = gt_geom::brush::best_face_data(&plane.flipped(), &template).cloned().unwrap_or_default();
            let (front, back) = b.split(plane, &cap);
            let parent = m.get(id).and_then(|n| n.parent).unwrap_or(m.default_layer());
            let label = m.get(id).and_then(|n| n.label.clone());
            m.remove(id);
            let keep: Vec<Brush> = match side {
                ClipSide::Front => front.into_iter().collect(),
                ClipSide::Back => back.into_iter().collect(),
                ClipSide::Both => front.into_iter().chain(back).collect(),
            };
            let new_ids: Vec<_> = keep.into_iter().map(|k| m.insert_labeled(parent, gt_doc::NodeKind::Brush(k), label.clone())).collect();
            s.nodes.extend(new_ids.iter().copied());
            replaced.insert(id, new_ids);
        }

        replaced
    });
}

#[derive(Default)]
pub struct ClipTool {
    pub points: Vec<DVec3>,
    side: Option<ClipSide>,
    dragging: Option<usize>,
    /// Extrusion direction used to make a plane from only two points.
    second_axis: Option<DVec3>,
}

#[derive(Default)]
pub struct VertexTool {
    pub selected: Vec<DVec3>,
    drag: Option<VertexDrag>,
    /// Vertex (or edge/face point) that has a precise-move gizmo, set by double clicking it.
    gizmo: Option<DVec3>,
    gizmo_drag: Option<VGizmoDrag>,
}

struct VertexDrag {
    start: DVec3,
    plane: Plane,
    base: Vec<DVec3>,
}

struct VGizmoDrag {
    axis: usize,
    start: DVec3,
    plane: Plane,
    base: DVec3,
}

#[derive(Default)]
pub struct RotateTool {
    drag: Option<RotateDrag>,
    hover_axis: Option<usize>,
    pub snap_degrees: f64,
}

struct RotateDrag {
    axis: DVec3,
    center: DVec3,
    start_vec: DVec3,
    screen_start: Pos2,
}

#[derive(Default)]
pub struct ScaleTool {
    drag: Option<ScaleDrag>,
}

struct ScaleDrag {
    base: Aabb,
    handle: DVec3,
    plane: Plane,
    start: DVec3,
}

#[derive(Default)]
pub struct BrushStrokeTool {
    /// Surface point and normal under the cursor.
    pub hover: Option<(DVec3, DVec3)>,
    stroking: bool,
    last_dab: Option<DVec3>,
    dabs: u32,
    /// Seconds the current stroke has been held.
    held: f64,
}

/// A click or a quick flick is topped up to this much hold time on release, so a single dab always shows.
const MIN_STROKE_SECONDS: f64 = 0.25;

/// Units a raise or lower stroke moves the brush centre per second of holding, per point of strength. It does not
/// depend on the radius, so resizing mid stroke keeps the speed, like the MCP `sculpt` op whose strength is one dab's
/// height whatever the radius.
const SCULPT_UNITS_PER_SECOND: f64 = 8.0;

/// The brush one sculpt dab applies for `seconds` of holding.
pub fn timed_sculpt_brush(mut brush: gt_doc::terrain::SculptBrush, seconds: f64) -> gt_doc::terrain::SculptBrush {
    brush.strength = if brush.mode.is_paint() { (brush.strength * seconds * 2.0).min(1.0) } else { brush.strength * seconds * SCULPT_UNITS_PER_SECOND };
    brush
}

#[derive(Default)]
pub struct ToolSet {
    pub clip: ClipTool,
    pub vertex: VertexTool,
    pub rotate: RotateTool,
    pub scale: ScaleTool,
    pub stroke: BrushStrokeTool,
    pub mesh: crate::mesh_tool::MeshTool,
    pub scatter: crate::scatter_tool::ScatterTool,
    pub blend: crate::blend_tool::BlendTool,
    pub volume: crate::volume_tool::VolumeTool,
    pub path: crate::extra_tools::PathTool,
    pub measure: crate::extra_tools::MeasureTool,
    pub texture: crate::texture_tool::TextureTool,
    active: Option<ToolKind>,
    /// A tool was switched away from mid drag with its undo transaction open. `sync` cannot reach the document,
    /// so `settle` commits it on the next call that can.
    orphaned: bool,
    /// Set by the app while a viewport's right mouse look runs, read and cleared by `keys`.
    pub flying: bool,
    /// Bounds of the last brush drawn in a view, 2D views give a new brush its depth.
    pub last_brush: Option<Aabb>,
}

fn screen_dist(cam: &Camera, rect: Rect, world: DVec3, pos: Pos2) -> f32 {
    cam.project(rect, world).map(|p| (p - pos).length()).unwrap_or(f32::MAX)
}

/// Outline of the object under the pointer in the outliner, so a row can be matched to what it is in the views.
fn outliner_hover_lines(state: &EditorState, id: NodeId, out: &mut Vec<LineVertex>) {
    const COLOR: [f32; 4] = [0.39, 0.76, 0.93, 1.0];
    let map = &state.doc.map;
    if map.layers.contains(&id) {
        return;
    }

    let mut drew = false;
    for n in std::iter::once(id).chain(map.descendants(id)) {
        if let Some(b) = map.brush(n) {
            for (a, c) in b.edges() {
                line(out, b.vertices[a as usize], b.vertices[c as usize], COLOR);
            }

            drew = true;
        }
    }

    let bounds = map.bounds(id);
    if !drew && !bounds.is_empty() {
        let c = bounds.corners();
        for (i, j) in Aabb::EDGES {
            line(out, c[i], c[j], COLOR);
        }
    }
}

fn line(out: &mut Vec<LineVertex>, a: DVec3, b: DVec3, color: [f32; 4]) {
    out.push(LineVertex { pos: v3(a), color });
    out.push(LineVertex { pos: v3(b), color });
}

impl ToolSet {
    /// Resets per-tool state when the active tool changes.
    pub fn sync(&mut self, state: &EditorState) {
        if self.active != Some(state.tool) {
            self.orphaned |= self.holds_transaction();
            self.clip = ClipTool::default();
            self.end_drags();
            self.vertex.gizmo = None;
            if self.rotate.snap_degrees == 0.0 {
                self.rotate.snap_degrees = 15.0;
            }

            self.mesh.reset();
            self.path.finish();
            self.texture.reset();
            self.volume.reset();
            self.active = Some(state.tool);
        }
    }

    /// A drag, stroke or mesh modal that edits inside an open undo transaction.
    fn holds_transaction(&self) -> bool {
        self.holds_drag() || self.mesh.holds_transaction()
    }

    fn holds_drag(&self) -> bool {
        self.vertex.drag.is_some()
            || self.vertex.gizmo_drag.is_some()
            || self.rotate.drag.is_some()
            || self.scale.drag.is_some()
            || self.stroke.stroking
            || self.scatter.stroking()
            || self.blend.stroking()
    }

    /// Drops drags and strokes without touching the document.
    fn end_drags(&mut self) {
        self.vertex.drag = None;
        self.vertex.gizmo_drag = None;
        self.rotate.drag = None;
        self.scale.drag = None;
        self.stroke.stroking = false;
        self.stroke.last_dab = None;
        self.scatter.reset();
        self.blend.reset();
    }

    /// Closes what a tool switch left open, and forgets the old map after a tab switch, New or Open. Runs from
    /// `keys` and the viewports, which can reach the document where `sync` cannot.
    pub fn settle(&mut self, state: &mut EditorState) {
        if state.scene_reset {
            self.reset();
        } else if std::mem::take(&mut self.orphaned) {
            state.doc.commit();
        } else if self.holds_transaction() && !state.doc.in_transaction() {
            // Something else closed the transaction (an undo mid drag), the drag must not go on editing outside it.
            self.end_drags();
            if self.mesh.holds_transaction() {
                self.mesh.reset();
            }
        }
    }

    /// Forgets everything that refers to the current map, node ids restart per map. For a tab switch, New or Open,
    /// after the document was swapped: an open drag's transaction belongs to the map that was left, so the document
    /// is not touched.
    pub fn reset(&mut self) {
        self.orphaned = false;
        self.clip = ClipTool::default();
        self.end_drags();
        self.vertex.selected.clear();
        self.vertex.gizmo = None;
        self.stroke.hover = None;
        self.mesh.reset();
        self.mesh.selection.clear();
        self.path.finish();
        self.measure = Default::default();
        self.texture.reset();
        self.volume.reset();
    }

    /// Enter, Tab and similar keys for the active tool. Returns true if the key was consumed.
    pub fn keys(&mut self, ctx: &egui::Context, state: &mut EditorState) -> bool {
        self.settle(state);
        let flying = std::mem::take(&mut self.flying);
        if ctx.egui_wants_keyboard_input() {
            return false;
        }

        if flying {
            // WASD, Q and E steer the camera during the right mouse look, so letters and other plain keys must not
            // reach tool shortcuts. Only Escape and Ctrl or Alt combinations get through.
            ctx.input_mut(|i| {
                i.events.retain(|e| match e {
                    egui::Event::Key { key, pressed: true, modifiers, .. } => *key == Key::Escape || modifiers.command || modifiers.alt,
                    egui::Event::Text(_) => false,
                    _ => true,
                })
            });
            return false;
        }

        if self.holds_drag() && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Escape)) {
            state.doc.cancel();
            self.end_drags();
            state.set_status("Cancelled");
            return true;
        }

        match state.tool {
            ToolKind::Clip => {
                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Tab)) {
                    self.clip.side = Some(match self.clip.side.unwrap_or(ClipSide::Front) {
                        ClipSide::Front => ClipSide::Back,
                        ClipSide::Back => ClipSide::Both,
                        ClipSide::Both => ClipSide::Front,
                    });
                    return true;
                }

                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter)) {
                    self.apply_clip(state);
                    return true;
                }

                if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Backspace)) && !self.clip.points.is_empty() {
                    self.clip.points.pop();
                    return true;
                }
            }
            ToolKind::Vertex if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Delete)) && !self.vertex.selected.is_empty() => {
                self.delete_vertices(state);
                return true;
            }
            ToolKind::Mesh => return self.mesh.keys(ctx, state),
            ToolKind::Texture => return self.texture.keys(ctx, state),
            ToolKind::Path
                if self.path.last.is_some()
                    && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, Key::Enter) || i.consume_key(egui::Modifiers::NONE, Key::Escape)) =>
            {
                self.path.finish();
                state.set_status("Path finished");
                return true;
            }
            _ => {}
        }

        false
    }

    pub fn viewport_input(&mut self, ui: &Ui, response: &Response, cam: &Camera, rect: Rect, hover: Option<Pos2>, state: &mut EditorState) {
        match state.tool {
            ToolKind::Clip => self.clip_input(ui, response, cam, rect, state),
            ToolKind::Vertex => self.vertex_input(ui, response, cam, rect, state),
            ToolKind::Rotate => self.rotate_input(ui, response, cam, rect, hover, state),
            ToolKind::Scale => self.scale_input(ui, response, cam, rect, hover, state),
            ToolKind::Sculpt | ToolKind::Paint => self.stroke_input(ui, response, cam, rect, hover, state),
            ToolKind::Mesh => self.mesh.viewport_input(ui, response, cam, rect, hover, state),
            ToolKind::Scatter => self.scatter.input(ui, response, cam, rect, hover, state),
            ToolKind::Blend => self.blend.input(ui, response, cam, rect, hover, state),
            ToolKind::Volume => self.volume.input(ui, response, cam, rect, state),
            ToolKind::Path => self.path.input(response, cam, rect, state),
            ToolKind::Measure => self.measure.input(ui, response, cam, rect, state),
            ToolKind::Texture => self.texture.input(ui, response, cam, rect, hover, state),
            ToolKind::Select => {}
        }
    }

    // ------------------------------------------------------- sculpt / paint

    fn stroke_input(&mut self, ui: &Ui, response: &Response, cam: &Camera, rect: Rect, hover: Option<Pos2>, state: &mut EditorState) {
        let scope = crate::blend_tool::StrokeScope::of(state);
        let sculpting = state.tool == ToolKind::Sculpt;
        let faces = if sculpting { scope.displacements(state) } else { Vec::new() };
        let (targets, terrains) = (scope.brushes, scope.terrains);
        let pointer = if self.stroke.stroking { ui.input(|i| i.pointer.interact_pos()) } else { hover };
        self.stroke.hover = pointer.and_then(|pos| {
            let ray = cam.ray(rect, pos);
            if sculpting {
                let disp = gt_doc::terrain::ray_cast(&state.doc.map, &faces, &ray)
                    .map(|(d, p, id, face)| (d, p, state.doc.map.brush(id).map(|b| b.faces[face].plane.normal).unwrap_or(DVec3::Y)));
                let terrain = gt_doc::terrain::terrain_ray_cast(&state.doc.map, &terrains, &ray).map(|(d, p, _)| (d, p, DVec3::Y));
                [disp, terrain].into_iter().flatten().min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, p, n)| (p, n))
            } else {
                // Vertex paint only reaches brush faces, a mesh or anything else in front shows no brush ring.
                picking::pick(state, &ray).filter(|h| h.face.is_some() && targets.contains(&h.node)).map(|h| (h.point, h.normal))
            }
        });

        let modifiers = ui.input(|i| i.modifiers);
        // A quick click can press and release within one frame, which only shows up as a click.
        if (response.drag_started_by(PointerButton::Primary)
            || response.clicked_by(PointerButton::Primary)
            || (response.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_pressed())))
            && !self.stroke.stroking
            && let Some((p, n)) = self.stroke.hover
        {
            self.stroke.stroking = true;
            self.stroke.last_dab = None;
            self.stroke.held = 0.0;
            state.sculpt.flatten_height = p.dot(n);
            state.doc.begin(if sculpting { "Sculpt" } else { "Vertex Paint" });
            let cell = terrains.iter().filter_map(|id| state.doc.map.terrain(*id)).map(|t| t.cell_size).fold(0.0, f64::max);
            if sculpting && state.sculpt.radius < cell {
                state.set_status(format!(
                    "The brush radius {:.0} is smaller than the terrain's {cell:.0} unit cells, so it barely moves a vertex. Ctrl+wheel makes it bigger",
                    state.sculpt.radius
                ));
            }
        }

        if self.stroke.stroking {
            let dt = ui.input(|i| i.stable_dt as f64).min(0.05);
            let released = !ui.input(|i| i.pointer.primary_down());
            let seconds = if released { dt.max(MIN_STROKE_SECONDS - self.stroke.held) } else { dt };
            self.stroke.held += dt;
            if let Some((p, _)) = self.stroke.hover {
                let spacing = state.sculpt.radius * 0.15;
                let moved_enough = self.stroke.last_dab.is_none_or(|last| (last - p).length() >= spacing);
                if moved_enough || sculpting || seconds > dt {
                    let mut brush = state.sculpt;
                    if modifiers.shift {
                        brush.mode = match brush.mode {
                            gt_doc::terrain::SculptMode::Raise => gt_doc::terrain::SculptMode::Lower,
                            gt_doc::terrain::SculptMode::Lower => gt_doc::terrain::SculptMode::Raise,
                            gt_doc::terrain::SculptMode::PaintAlpha => gt_doc::terrain::SculptMode::EraseAlpha,
                            other => other,
                        };
                    }

                    if modifiers.command {
                        brush.mode = gt_doc::terrain::SculptMode::Smooth;
                    }

                    // Continuous brushes scale with the time held so the result does not depend on frame rate.
                    self.stroke.dabs = self.stroke.dabs.wrapping_add(1);
                    let brush = gt_doc::terrain::SculptBrush { seed: self.stroke.dabs, ..timed_sculpt_brush(brush, seconds) };
                    let color = state.paint_color;
                    let radius = state.sculpt.radius;
                    let strength = state.sculpt.strength;
                    state.doc.edit("Stroke", |m, _| {
                        if sculpting {
                            gt_doc::terrain::sculpt(m, &faces, p, &brush);
                            gt_doc::terrain::sculpt_terrains(m, &terrains, p, &brush);
                        } else {
                            gt_doc::terrain::paint_vertices(m, &targets, p, radius, color, (strength * seconds).min(1.0));
                        }
                    });
                    self.stroke.last_dab = Some(p);
                    ui.ctx().request_repaint();
                }
            }

            if released {
                self.stroke.stroking = false;
                state.doc.commit();
            }
        }
    }

    // ---------------------------------------------------------------- clip

    fn clip_point_at(&self, cam: &Camera, rect: Rect, pos: Pos2, state: &EditorState) -> Option<(DVec3, Option<DVec3>)> {
        match cam.kind {
            ViewKind::Perspective => {
                let hit = picking::pick(state, &cam.ray(rect, pos))?;
                Some((state.snap(hit.point), Some(hit.normal)))
            }
            k => {
                let mut p = cam.screen_to_plane(rect, pos);
                let b = state.doc.map.bounds_of(state.doc.selection.nodes.iter().copied());
                let axis = k.depth_axis();
                p[axis] = if b.is_empty() { 0.0 } else { b.center()[axis] };
                Some((state.snap(p), Some(k.axes().2)))
            }
        }
    }

    fn clip_input(&mut self, ui: &Ui, response: &Response, cam: &Camera, rect: Rect, state: &mut EditorState) {
        let pointer = ui.input(|i| i.pointer.interact_pos());
        if response.drag_started_by(PointerButton::Primary)
            && let Some(origin) = ui.input(|i| i.pointer.press_origin())
        {
            self.clip.dragging = self.clip.points.iter().position(|p| screen_dist(cam, rect, *p, origin) < HANDLE_RADIUS);
        }

        if let (Some(i), Some(pos)) = (self.clip.dragging, pointer) {
            if response.dragged_by(PointerButton::Primary)
                && let Some((p, _)) = self.clip_point_at(cam, rect, pos, state)
            {
                self.clip.points[i] = p;
            }

            if response.drag_stopped() {
                self.clip.dragging = None;
            }

            return;
        }

        if response.clicked()
            && self.clip.points.len() < 3
            && let Some(pos) = response.interact_pointer_pos()
            && let Some((p, n)) = self.clip_point_at(cam, rect, pos, state)
        {
            if self.clip.points.is_empty() {
                self.clip.second_axis = n;
            }

            if !self.clip.points.iter().any(|q| (*q - p).length() < 1e-6) {
                self.clip.points.push(p);
            }
        }
    }

    pub fn clip_plane(&self) -> Option<Plane> {
        let pts = &self.clip.points;
        match pts.len() {
            2 => {
                let extra = pts[0] + self.clip.second_axis? * 64.0;
                Plane::from_points(pts[0], pts[1], extra)
            }
            3 => Plane::from_points(pts[0], pts[1], pts[2]),
            _ => None,
        }
    }

    pub fn apply_clip_public(&mut self, state: &mut EditorState) {
        self.apply_clip(state);
    }

    fn apply_clip(&mut self, state: &mut EditorState) {
        let Some(plane) = self.clip_plane() else {
            state.set_status("Place two or three clip points first");
            return;
        };
        clip_selection(state, &plane, self.clip.side.unwrap_or(ClipSide::Front));
        self.clip.points.clear();
        state.set_status("Clipped");
    }

    // -------------------------------------------------------------- vertex

    fn vertex_handles(state: &EditorState) -> (Vec<DVec3>, Vec<DVec3>) {
        let mut verts: Vec<DVec3> = Vec::new();
        let mut extra: Vec<DVec3> = Vec::new();
        for id in state.doc.selection.brushes(&state.doc.map) {
            let Some(b) = state.doc.map.brush(id) else { continue };
            for v in &b.vertices {
                if !verts.iter().any(|q| (*q - *v).length() < 1e-6) {
                    verts.push(*v);
                }
            }

            for (a, c) in b.edges() {
                extra.push((b.vertices[a as usize] + b.vertices[c as usize]) * 0.5);
            }

            for f in 0..b.faces.len() {
                extra.push(b.face_center(f));
            }
        }

        (verts, extra)
    }

    fn vertex_input(&mut self, ui: &Ui, response: &Response, cam: &Camera, rect: Rect, state: &mut EditorState) {
        let modifiers = ui.input(|i| i.modifiers);
        let (verts, extra) = Self::vertex_handles(state);
        let nearest = |pos: Pos2, list: &[DVec3]| {
            list.iter()
                .copied()
                .map(|v| (screen_dist(cam, rect, v, pos), v))
                .filter(|(d, _)| *d < HANDLE_RADIUS)
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, v)| v)
        };

        if response.clicked() {
            let Some(pos) = response.interact_pointer_pos() else { return };
            match nearest(pos, &verts) {
                Some(v) => {
                    if modifiers.command {
                        if let Some(i) = self.vertex.selected.iter().position(|q| (*q - v).length() < 1e-6) {
                            self.vertex.selected.remove(i);
                        } else {
                            self.vertex.selected.push(v);
                        }
                    } else {
                        self.vertex.selected = vec![v];
                    }
                }
                None => {
                    if let Some(h) = picking::pick(state, &cam.ray(rect, pos)) {
                        let target = state.doc.map.click_target(h.node, &state.open_groups);
                        state.doc.select(|_, s| {
                            if !modifiers.command {
                                s.clear();
                            }

                            s.toggle_node(target);
                        });
                    }

                    self.vertex.selected.clear();
                    self.vertex.gizmo = None;
                }
            }
        }

        // Double clicking a vertex (or an edge/face point, which creates a vertex) puts a precise-move
        // gizmo on it. Double clicking elsewhere clears it.
        if response.double_clicked() {
            let hit = response.interact_pointer_pos().and_then(|pos| nearest(pos, &verts).or_else(|| nearest(pos, &extra)));
            match hit {
                Some(v) => {
                    self.vertex.selected = vec![v];
                    self.vertex.gizmo = Some(v);
                }
                None => self.vertex.gizmo = None,
            }
        }

        if response.drag_started_by(PointerButton::Primary) {
            let Some(origin) = ui.input(|i| i.pointer.press_origin()) else { return };
            // A drag that starts on a gizmo arrow moves the vertex along just that axis.
            if let Some(g) = self.vertex.gizmo.filter(|_| cam.kind == ViewKind::Perspective)
                && let Some(i) = vgizmo_axis_at(cam, rect, g, origin)
            {
                let plane = axis_drag_plane(cam, i, g);
                let ray = cam.ray(rect, origin);
                if let Some(t) = ray.intersect_plane(&plane) {
                    self.vertex.selected = vec![g];
                    state.doc.begin("Move Vertex");
                    self.vertex.gizmo_drag = Some(VGizmoDrag { axis: i, start: ray.at(t), plane, base: g });
                }

                return;
            }

            let grabbed = nearest(origin, &verts).map(|v| (v, false)).or_else(|| nearest(origin, &extra).map(|v| (v, true)));
            if let Some((v, is_new)) = grabbed {
                // Grabbing an unselected handle (or an edge/face point) drags just that one.
                if is_new || !self.vertex.selected.iter().any(|q| (*q - v).length() < 1e-6) {
                    self.vertex.selected = vec![v];
                }

                let plane = match cam.kind {
                    ViewKind::Perspective if modifiers.alt => {
                        let f = cam.forward();
                        Plane::from_point_normal(v, DVec3::new(f.x, 0.0, f.z).normalize_or(DVec3::Z))
                    }
                    ViewKind::Perspective => Plane::from_point_normal(v, DVec3::Y),
                    k => Plane::from_point_normal(v, -k.axes().2),
                };
                state.doc.begin("Move Vertices");
                self.vertex.drag = Some(VertexDrag { start: v, plane, base: self.vertex.selected.clone() });
            }
        }

        if let Some(drag) = &self.vertex.drag {
            if response.dragged_by(PointerButton::Primary)
                && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
            {
                let ray = cam.ray(rect, pos);
                if let Some(t) = ray.intersect_plane(&drag.plane) {
                    let mut delta = ray.at(t) - drag.start;
                    if cam.kind == ViewKind::Perspective && modifiers.alt {
                        delta = DVec3::new(0.0, delta.y, 0.0);
                    }

                    // Snaps the offset like the Move tool, so an off grid vertex keeps its position on the other axes.
                    let delta = state.snap(delta);
                    state.doc.reset_transaction();
                    let base = drag.base.clone();
                    let moved = move_vertices(state, &base, delta);
                    if moved {
                        self.vertex.selected = base.iter().map(|b| *b + delta).collect();
                    }
                }
            }

            if response.drag_stopped() || !ui.input(|i| i.pointer.primary_down()) {
                self.vertex.drag = None;
                state.doc.commit();
            }
        }

        if let Some(gd) = &self.vertex.gizmo_drag {
            let (axis, start, plane, base) = (gd.axis, gd.start, gd.plane, gd.base);
            if response.dragged_by(PointerButton::Primary)
                && let Some(pos) = ui.input(|i| i.pointer.interact_pos())
            {
                let ray = cam.ray(rect, pos);
                if let Some(t) = ray.intersect_plane(&plane) {
                    let offset = ray.at(t) - start;
                    let delta = unit_axis(axis) * state.snap_scalar(offset[axis]);
                    state.doc.reset_transaction();
                    if move_vertices(state, &[base], delta) {
                        self.vertex.selected = vec![base + delta];
                        self.vertex.gizmo = Some(base + delta);
                        state.set_status(format!("Vertex {:.3} {:.3} {:.3}", (base + delta).x, (base + delta).y, (base + delta).z));
                    }
                }
            }

            if response.drag_stopped() || !ui.input(|i| i.pointer.primary_down()) {
                self.vertex.gizmo_drag = None;
                state.doc.commit();
            }
        }
    }

    fn delete_vertices(&mut self, state: &mut EditorState) {
        self.vertex.gizmo = None;
        let selected = std::mem::take(&mut self.vertex.selected);
        let brushes = state.doc.selection.brushes(&state.doc.map);
        state.doc.edit("Remove Vertices", |m, _| {
            for id in brushes {
                let Some(b) = m.brush(id).cloned() else { continue };
                let pts: Vec<DVec3> = b.vertices.iter().copied().filter(|v| !selected.iter().any(|s| (*s - *v).length() < 1e-6)).collect();
                if pts.len() == b.vertices.len() {
                    continue;
                }

                if let Ok(nb) = Brush::from_points(&pts, &b.planes(), "")
                    && let Some(slot) = m.brush_mut(id)
                {
                    *slot = nb;
                }
            }
        });
    }

    // -------------------------------------------------------------- rotate

    fn rotate_center(state: &EditorState) -> Option<DVec3> {
        let b = state.doc.map.bounds_of(state.doc.selection.nodes.iter().copied());
        (!b.is_empty()).then(|| state.snap(b.center()))
    }

    fn ring_radius(cam: &Camera, center: DVec3) -> f64 {
        match cam.kind {
            ViewKind::Perspective => (cam.position - center).length() * 0.18,
            _ => 90.0 / cam.zoom,
        }
    }

    fn ring_points(center: DVec3, axis: usize, radius: f64) -> Vec<DVec3> {
        let (u, v) = match axis {
            0 => (DVec3::Y, DVec3::Z),
            1 => (DVec3::Z, DVec3::X),
            _ => (DVec3::X, DVec3::Y),
        };
        (0..=48)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / 48.0;
                center + (u * a.cos() + v * a.sin()) * radius
            })
            .collect()
    }

    fn rotate_input(&mut self, ui: &Ui, response: &Response, cam: &Camera, rect: Rect, hover: Option<Pos2>, state: &mut EditorState) {
        let Some(center) = Self::rotate_center(state) else { return };
        let radius = Self::ring_radius(cam, center);
        let axes: Vec<usize> = if cam.kind.is_2d() { vec![cam.kind.depth_axis()] } else { vec![0, 1, 2] };
        if self.rotate.drag.is_none() {
            self.rotate.hover_axis = hover.and_then(|pos| {
                axes.iter()
                    .map(|a| {
                        let pts = Self::ring_points(center, *a, radius);
                        let d = pts.windows(2).map(|w| seg_screen_dist(cam, rect, w[0], w[1], pos)).fold(f32::MAX, f32::min);
                        (d, *a)
                    })
                    .filter(|(d, _)| *d < 8.0 || cam.kind.is_2d())
                    .min_by(|a, b| a.0.total_cmp(&b.0))
                    .map(|(_, a)| a)
            });
        }

        if response.drag_started_by(PointerButton::Primary)
            && let (Some(axis), Some(origin)) = (self.rotate.hover_axis, ui.input(|i| i.pointer.press_origin()))
        {
            let mut n = DVec3::ZERO;
            n[axis] = 1.0;
            if let Some(p) = ring_plane_point(cam, rect, origin, center, n) {
                state.doc.begin("Rotate");
                self.rotate.drag = Some(RotateDrag { axis: n, center, start_vec: (p - center).normalize_or_zero(), screen_start: origin });
            }
        }

        if let Some(drag) = &self.rotate.drag {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                if let Some(p) = ring_plane_point(cam, rect, pos, drag.center, drag.axis) {
                    let cur = (p - drag.center).normalize_or_zero();
                    let mut angle = drag.start_vec.cross(cur).dot(drag.axis).atan2(drag.start_vec.dot(cur)).to_degrees();
                    let snap = if ui.input(|i| i.modifiers.shift) { 1.0 } else { self.rotate.snap_degrees.max(1.0) };
                    angle = (angle / snap).round() * snap;
                    state.doc.reset_transaction();
                    if angle.abs() > 1e-9 {
                        let m = ops::rotation_about(drag.center, drag.axis, angle);
                        let opts = state.opts();
                        state.doc.edit("Rotate", |map, s| ops::transform_selection(map, s, &m, opts));
                    }

                    state.set_status(format!("Rotate {angle:.1}°"));
                }

                let _ = drag.screen_start;
            }

            if response.drag_stopped() || !ui.input(|i| i.pointer.primary_down()) {
                self.rotate.drag = None;
                state.doc.commit();
            }
        }
    }

    // --------------------------------------------------------------- scale

    fn scale_handles(bounds: &Aabb, cam: &Camera) -> Vec<DVec3> {
        let mut out = Vec::new();
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let h = DVec3::new(x as f64, y as f64, z as f64);
                    if h == DVec3::ZERO {
                        continue;
                    }

                    if cam.kind.is_2d() && h[cam.kind.depth_axis()] != 0.0 {
                        continue;
                    }

                    // 3D shows face handles only to keep the view readable.
                    if !cam.kind.is_2d() && h.abs().element_sum() != 1.0 {
                        continue;
                    }

                    out.push(h);
                }
            }
        }

        let _ = bounds;
        out
    }

    fn handle_pos(bounds: &Aabb, h: DVec3) -> DVec3 {
        bounds.center() + bounds.size() * 0.5 * h
    }

    fn scale_input(&mut self, ui: &Ui, response: &Response, cam: &Camera, rect: Rect, _hover: Option<Pos2>, state: &mut EditorState) {
        let bounds = state.doc.map.bounds_of(state.doc.selection.nodes.iter().copied());
        if bounds.is_empty() {
            return;
        }

        if response.drag_started_by(PointerButton::Primary) {
            let Some(origin) = ui.input(|i| i.pointer.press_origin()) else { return };
            let best = Self::scale_handles(&bounds, cam)
                .into_iter()
                .map(|h| (screen_dist(cam, rect, Self::handle_pos(&bounds, h), origin), h))
                .filter(|(d, _)| *d < HANDLE_RADIUS * 1.5)
                .min_by(|a, b| a.0.total_cmp(&b.0));
            if let Some((_, h)) = best {
                let p = Self::handle_pos(&bounds, h);
                let plane = match cam.kind {
                    ViewKind::Perspective => {
                        let f = cam.forward();
                        let n = if h.y != 0.0 { DVec3::new(f.x, 0.0, f.z).normalize_or(DVec3::Z) } else { DVec3::Y };
                        Plane::from_point_normal(p, n)
                    }
                    k => Plane::from_point_normal(p, -k.axes().2),
                };
                state.doc.begin("Scale");
                self.scale.drag = Some(ScaleDrag { base: bounds, handle: h, plane, start: p });
            }
        }

        if let Some(drag) = &self.scale.drag {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                let ray = cam.ray(rect, pos);
                if let Some(t) = ray.intersect_plane(&drag.plane) {
                    let delta = ray.at(t) - drag.start;
                    let symmetric = ui.input(|i| i.modifiers.alt);
                    let mut new = drag.base;
                    for i in 0..3 {
                        if drag.handle[i] > 0.0 {
                            new.max[i] = state.snap_scalar(drag.base.max[i] + delta[i]);
                            if symmetric {
                                new.min[i] = drag.base.min[i] - (new.max[i] - drag.base.max[i]);
                            }
                        } else if drag.handle[i] < 0.0 {
                            new.min[i] = state.snap_scalar(drag.base.min[i] + delta[i]);
                            if symmetric {
                                new.max[i] = drag.base.max[i] - (new.min[i] - drag.base.min[i]);
                            }
                        }
                    }

                    let valid = (0..3).all(|i| new.max[i] - new.min[i] > 1e-3 || drag.base.size()[i] < 1e-6);
                    state.doc.reset_transaction();
                    if valid && new != drag.base {
                        let m: DMat4 = ops::scale_bounds(&drag.base, &new);
                        let opts = state.opts();
                        state.doc.edit("Scale", |map, s| ops::transform_selection(map, s, &m, opts));
                        let size = new.size();
                        state.set_status(format!("Scale to {} x {} x {}", size.x, size.y, size.z));
                    }
                }
            }

            if response.drag_stopped() || !ui.input(|i| i.pointer.primary_down()) {
                self.scale.drag = None;
                state.doc.commit();
            }
        }
    }

    // ------------------------------------------------------------ drawing

    pub fn lines(&self, cam: &Camera, rect: Rect, state: &EditorState) -> Vec<LineVertex> {
        let mut out = Vec::new();
        match state.tool {
            ToolKind::Clip => {
                let pts = &self.clip.points;
                for w in pts.windows(2) {
                    line(&mut out, w[0], w[1], [0.3, 1.0, 1.0, 1.0]);
                }

                if pts.len() == 3 {
                    line(&mut out, pts[2], pts[0], [0.3, 1.0, 1.0, 1.0]);
                }

                if let Some(plane) = self.clip_plane() {
                    let side = self.clip.side.unwrap_or(ClipSide::Front);
                    for id in state.doc.selection.brushes(&state.doc.map) {
                        let Some(b) = state.doc.map.brush(id) else { continue };
                        let (front, back) = b.split(&plane, &Default::default());
                        let kept = [0.3, 1.0, 0.4, 1.0];
                        let removed = [0.6, 0.6, 0.6, 0.35];
                        for (piece, keep) in [(front, side != ClipSide::Back), (back, side != ClipSide::Front)] {
                            if let Some(p) = piece {
                                for (a, c) in p.edges() {
                                    line(&mut out, p.vertices[a as usize], p.vertices[c as usize], if keep { kept } else { removed });
                                }
                            }
                        }
                    }
                }
            }
            ToolKind::Rotate => {
                if let Some(center) = Self::rotate_center(state) {
                    let radius = Self::ring_radius(cam, center);
                    let axes: Vec<usize> = if cam.kind.is_2d() { vec![cam.kind.depth_axis()] } else { vec![0, 1, 2] };
                    for a in axes {
                        let active = self.rotate.hover_axis == Some(a) || self.rotate.drag.as_ref().is_some_and(|d| d.axis[a] == 1.0);
                        let mut color = match a {
                            0 => [1.0, 0.3, 0.3, 0.9],
                            1 => [0.3, 1.0, 0.3, 0.9],
                            _ => [0.35, 0.5, 1.0, 0.9],
                        };
                        if active {
                            color = [1.0, 1.0, 0.3, 1.0];
                        }

                        let pts = Self::ring_points(center, a, radius);
                        for w in pts.windows(2) {
                            line(&mut out, w[0], w[1], color);
                        }
                    }
                }
            }
            ToolKind::Scale => {
                let b = state.doc.map.bounds_of(state.doc.selection.nodes.iter().copied());
                if !b.is_empty() {
                    let c = b.corners();
                    for (i, j) in Aabb::EDGES {
                        line(&mut out, c[i], c[j], [1.0, 0.8, 0.2, 0.9]);
                    }
                }
            }
            ToolKind::Mesh => out.extend(self.mesh.lines(cam, rect, state)),
            ToolKind::Scatter => self.scatter.lines(state, &mut out),
            ToolKind::Blend => self.blend.lines(state, &mut out),
            ToolKind::Volume => self.volume.lines(state, cam, &mut out),
            ToolKind::Measure => self.measure.lines(&mut out),
            ToolKind::Texture => out.extend(self.texture.lines(state)),
            ToolKind::Sculpt | ToolKind::Paint => {
                if let Some((p, n)) = self.stroke.hover {
                    let basis = Plane::from_point_normal(p, n).basis();
                    let r = state.sculpt.radius;
                    let color = if state.tool == ToolKind::Sculpt { [0.4, 1.0, 0.6, 0.9] } else { [1.0, 0.6, 0.9, 0.9] };
                    let ring: Vec<DVec3> = (0..=40)
                        .map(|i| {
                            let a = std::f64::consts::TAU * i as f64 / 40.0;
                            p + (basis.0 * a.cos() + basis.1 * a.sin()) * r + n * 0.5
                        })
                        .collect();
                    for w in ring.windows(2) {
                        line(&mut out, w[0], w[1], color);
                    }

                    line(&mut out, p, p + n * r * 0.4, color);
                }
            }
            _ => {}
        }

        crate::gizmos::lines(state, &mut out);
        if let Some(id) = state.outliner_hover {
            outliner_hover_lines(state, id, &mut out);
        }

        let _ = rect;
        out
    }

    pub fn paint_overlay(&self, ui: &Ui, cam: &Camera, rect: Rect, state: &EditorState) {
        let painter = ui.painter_at(rect);
        let hover = ui.input(|i| i.pointer.hover_pos());
        let handle = |p: Pos2, color: Color32, hot: bool| {
            let r = if hot { 5.0 } else { 4.0 };
            painter.rect_filled(Rect::from_center_size(p, Vec2::splat(r * 2.0)), 1.0, color);
            painter.rect_stroke(Rect::from_center_size(p, Vec2::splat(r * 2.0)), 1.0, Stroke::new(1.0, Color32::BLACK), egui::StrokeKind::Outside);
        };
        match state.tool {
            ToolKind::Clip => {
                for (i, p) in self.clip.points.iter().enumerate() {
                    if let Some(sp) = cam.project(rect, *p) {
                        let hot = hover.is_some_and(|h| (h - sp).length() < HANDLE_RADIUS);
                        handle(sp, Color32::from_rgb(80, 255, 255), hot);
                        painter.text(sp + Vec2::new(8.0, -8.0), Align2::LEFT_BOTTOM, format!("{}", i + 1), FontId::monospace(11.0), Color32::WHITE);
                    }
                }

                let side = match self.clip.side.unwrap_or(ClipSide::Front) {
                    ClipSide::Front => "front",
                    ClipSide::Back => "back",
                    ClipSide::Both => "both",
                };
                painter.text(
                    rect.left_bottom() + Vec2::new(8.0, -8.0),
                    Align2::LEFT_BOTTOM,
                    format!("Clip: click to place points, keep {side} (Tab), Enter to apply"),
                    FontId::proportional(12.0),
                    Color32::from_rgb(120, 240, 240),
                );
            }
            ToolKind::Vertex => {
                let (verts, extra) = Self::vertex_handles(state);
                for v in extra {
                    if let Some(sp) = cam.project(rect, v) {
                        painter.circle_filled(sp, 2.5, Color32::from_rgb(120, 200, 255));
                    }
                }

                for v in verts {
                    if let Some(sp) = cam.project(rect, v) {
                        let selected = self.vertex.selected.iter().any(|s| (*s - v).length() < 1e-6);
                        let hot = hover.is_some_and(|h| (h - sp).length() < HANDLE_RADIUS);
                        handle(sp, if selected { Color32::from_rgb(255, 60, 40) } else { Color32::from_rgb(255, 220, 80) }, hot);
                    }
                }

                if let Some(g) = self.vertex.gizmo.filter(|_| cam.kind == ViewKind::Perspective) {
                    let len = vgizmo_len(cam, rect, g);
                    let hot_axis = self.vertex.gizmo_drag.as_ref().map(|d| d.axis).or_else(|| hover.and_then(|p| vgizmo_axis_at(cam, rect, g, p)));
                    for i in 0..3 {
                        let a = unit_axis(i);
                        let (Some(from), Some(head), Some(tip)) =
                            (cam.project(rect, g + a * len * 0.15), cam.project(rect, g + a * len * 0.78), cam.project(rect, g + a * len))
                        else {
                            continue;
                        };
                        let c = if hot_axis == Some(i) { crate::theme::YELLOW } else { crate::theme::AXIS[i] };
                        painter.line_segment([from, head], Stroke::new(if hot_axis == Some(i) { 3.5 } else { 2.5 }, c));
                        let dir = tip - head;
                        if dir.length() > 1.0 {
                            let side = Vec2::new(-dir.y, dir.x).normalized() * 6.0;
                            painter.add(egui::Shape::convex_polygon(vec![tip, head + side, head - side], c, Stroke::NONE));
                        }
                    }

                    if let Some(sp) = cam.project(rect, g) {
                        painter.circle_filled(sp, 3.0, Color32::WHITE);
                    }
                }

                painter.text(
                    rect.left_bottom() + Vec2::new(8.0, -8.0),
                    Align2::LEFT_BOTTOM,
                    "Vertex: drag handles, double click a vertex for a precise gizmo, edge/face dots add vertices, Del removes",
                    FontId::proportional(12.0),
                    Color32::from_rgb(255, 220, 120),
                );
            }
            ToolKind::Scale => {
                let b = state.doc.map.bounds_of(state.doc.selection.nodes.iter().copied());
                if !b.is_empty() {
                    for h in Self::scale_handles(&b, cam) {
                        if let Some(sp) = cam.project(rect, Self::handle_pos(&b, h)) {
                            let hot = hover.is_some_and(|p| (p - sp).length() < HANDLE_RADIUS * 1.5);
                            handle(sp, Color32::from_rgb(255, 200, 60), hot);
                        }
                    }
                }

                painter.text(
                    rect.left_bottom() + Vec2::new(8.0, -8.0),
                    Align2::LEFT_BOTTOM,
                    "Scale: drag handles, Alt scales symmetrically",
                    FontId::proportional(12.0),
                    Color32::from_rgb(255, 210, 120),
                );
            }
            ToolKind::Rotate => {
                painter.text(
                    rect.left_bottom() + Vec2::new(8.0, -8.0),
                    Align2::LEFT_BOTTOM,
                    format!("Rotate: drag a ring, snaps {}° (Shift for 1°)", self.rotate.snap_degrees),
                    FontId::proportional(12.0),
                    Color32::from_rgb(200, 255, 160),
                );
            }
            ToolKind::Mesh => self.mesh.paint_overlay(ui, cam, rect, state),
            ToolKind::Measure => self.measure.paint(ui, cam, rect, state),
            ToolKind::Texture => self.texture.paint(ui, rect, state),
            ToolKind::Scatter => self.scatter.paint_overlay(ui, rect, state),
            ToolKind::Blend => self.blend.paint_overlay(ui, rect, state),
            ToolKind::Volume => self.volume.paint_overlay(ui, rect, state),
            ToolKind::Path => {
                painter.text(
                    rect.left_bottom() + Vec2::new(8.0, -8.0),
                    Align2::LEFT_BOTTOM,
                    "Path: click to place path corners linked by target, Enter finishes the chain",
                    FontId::proportional(12.0),
                    Color32::from_rgb(160, 220, 255),
                );
            }
            ToolKind::Sculpt => {
                let has_terrain = state.doc.map.terrains().next().is_some();
                // An empty list is the whole map here: the hint is about the map having nothing to sculpt at all.
                let hint = if !has_terrain && gt_doc::terrain::displacement_faces(&state.doc.map, &[]).is_empty() {
                    "Sculpt: no terrain or displacements. Terrain > Create Terrain, or select quad faces and use Brush > Displacement".to_string()
                } else {
                    format!("Sculpt {:?}: drag to apply, Shift inverts, Ctrl smooths", state.sculpt.mode)
                };
                painter.text(
                    rect.left_bottom() + Vec2::new(8.0, -8.0),
                    Align2::LEFT_BOTTOM,
                    hint,
                    FontId::proportional(12.0),
                    Color32::from_rgb(140, 255, 170),
                );
            }
            ToolKind::Paint => {
                painter.text(
                    rect.left_bottom() + Vec2::new(8.0, -8.0),
                    Align2::LEFT_BOTTOM,
                    "Vertex paint: drag over brush faces (the selected brushes, or every brush when nothing is selected)",
                    FontId::proportional(12.0),
                    Color32::from_rgb(255, 170, 230),
                );
            }
            ToolKind::Select => {}
        }
    }
}

/// Applies a vertex move to every selected brush containing one of the base positions.
/// Brushes where the move would swallow a moved vertex are left unchanged. Returns true if anything moved.
fn unit_axis(i: usize) -> DVec3 {
    let mut a = DVec3::ZERO;
    a[i] = 1.0;
    a
}

fn seg_dist(a: Pos2, b: Pos2, p: Pos2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_sq().max(1e-6)).clamp(0.0, 1.0);
    (a + ab * t).distance(p)
}

/// World length at `center` that spans about 64 screen pixels, so the vertex gizmo keeps its size.
fn vgizmo_len(cam: &Camera, rect: Rect, center: DVec3) -> f64 {
    let fallback = (cam.eye() - center).length().max(1.0) * 0.15;
    let (Some(c), Some(p)) = (cam.project(rect, center), cam.project(rect, center + cam.right())) else {
        return fallback;
    };
    let per = (p - c).length() as f64;
    if per > 1e-6 { 64.0 / per } else { fallback }
}

/// The gizmo arrow under the pointer, if any.
fn vgizmo_axis_at(cam: &Camera, rect: Rect, center: DVec3, pos: Pos2) -> Option<usize> {
    let len = vgizmo_len(cam, rect, center);
    (0..3)
        .filter_map(|i| {
            let a = unit_axis(i);
            let from = cam.project(rect, center + a * len * 0.15)?;
            let to = cam.project(rect, center + a * len)?;
            Some((seg_dist(from, to, pos), i))
        })
        .filter(|(d, _)| *d < 8.0)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, i)| i)
}

/// Plane holding the gizmo axis and facing the camera, so pointer motion maps cleanly to the axis.
fn axis_drag_plane(cam: &Camera, i: usize, center: DVec3) -> Plane {
    let f = cam.forward();
    let a = unit_axis(i);
    let n = a.cross(f).cross(a);
    Plane::from_point_normal(center, if n.length_squared() < 1e-9 { f } else { n.normalize() })
}

/// Moves the selected brushes' vertices at `base` (or new ones on an edge or face) by `delta`, like the vertex tool.
/// Brushes that would turn concave or lose a moved vertex are left alone.
pub(crate) fn move_vertices(state: &mut EditorState, base: &[DVec3], delta: DVec3) -> bool {
    let brushes: Vec<NodeId> = state.doc.selection.brushes(&state.doc.map);
    let mut any = false;
    state.doc.edit("Move Vertices", |m, _| {
        for id in brushes {
            let Some(b) = m.brush(id).cloned() else { continue };
            let mut pts = b.vertices.clone();
            let mut touched = false;
            for bp in base {
                match pts.iter().position(|v| (*v - *bp).length() < 1e-6) {
                    Some(i) => {
                        pts[i] += delta;
                        touched = true;
                    }
                    None => {
                        // Edge midpoint or face center: only valid if it lies on this brush's surface.
                        if b.faces.iter().any(|f| f.plane.distance(*bp).abs() < 1e-4)
                            && b.contains_point(*bp)
                            && !b.vertices.iter().any(|v| (*v - *bp).length() < 1e-6)
                        {
                            pts.push(*bp + delta);
                            touched = true;
                        }
                    }
                }
            }

            if !touched {
                continue;
            }

            if let Ok(nb) = Brush::from_points(&pts, &b.planes(), "") {
                let kept = base.iter().all(|bp| {
                    let target = *bp + delta;
                    !b.contains_point(*bp) || nb.vertices.iter().any(|v| (*v - target).length() < 1e-3)
                });
                if kept && let Some(slot) = m.brush_mut(id) {
                    *slot = nb;
                    any = true;
                }
            }
        }
    });
    any
}

fn seg_screen_dist(cam: &Camera, rect: Rect, a: DVec3, b: DVec3, pos: Pos2) -> f32 {
    let (Some(pa), Some(pb)) = (cam.project(rect, a), cam.project(rect, b)) else { return f32::MAX };
    let ab = pb - pa;
    let t = ((pos - pa).dot(ab) / ab.length_sq().max(1e-6)).clamp(0.0, 1.0);
    (pa + ab * t - pos).length()
}

fn ring_plane_point(cam: &Camera, rect: Rect, pos: Pos2, center: DVec3, axis: DVec3) -> Option<DVec3> {
    let ray = cam.ray(rect, pos);
    let plane = Plane::from_point_normal(center, axis);
    match ray.intersect_plane(&plane) {
        Some(t) if t > 0.0 || cam.kind.is_2d() => Some(ray.at(t)),
        _ => {
            // Ring seen edge-on: fall back to projecting the closest point on the view ray.
            let (_, t) = ray.distance_to_point(center);
            Some(plane.project_point(ray.at(t)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with_brush() -> (EditorState, NodeId) {
        let mut state = EditorState::new(Default::default());
        let layer = state.doc.map.default_layer();
        let brush = Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(64.0)), "dev/grey").unwrap();
        let id = state.doc.edit("brush", |m, s| {
            let id = m.insert(layer, gt_doc::NodeKind::Brush(brush));
            s.select_node(id);
            id
        });
        (state, id)
    }

    /// Starts a rotate drag the way `rotate_input` does and applies a quarter turn inside it.
    fn rotate_mid_drag(tools: &mut ToolSet, state: &mut EditorState) {
        state.tool = ToolKind::Rotate;
        tools.sync(state);
        state.doc.begin("Rotate");
        tools.rotate.drag = Some(RotateDrag { axis: DVec3::Y, center: DVec3::splat(32.0), start_vec: DVec3::X, screen_start: Pos2::ZERO });
        let m = ops::rotation_about(DVec3::splat(32.0), DVec3::Y, 90.0);
        let opts = state.opts();
        state.doc.edit("Rotate", |map, s| ops::transform_selection(map, s, &m, opts));
    }

    #[test]
    fn clip_pieces_keep_the_brush_name() {
        let (mut state, id) = state_with_brush();
        state.doc.edit("name", |m, _| m.rename(id, "ramp"));
        clip_selection(&mut state, &Plane::from_point_normal(DVec3::splat(32.0), DVec3::Z), ClipSide::Both);
        let pieces = &state.replaced[&id];
        assert_eq!(pieces.len(), 2);
        assert!(pieces.iter().all(|p| state.doc.map.get(*p).unwrap().name() == "ramp"));
    }

    #[test]
    fn clip_selection_keeps_the_side_the_normal_points_to() {
        let (mut state, _) = state_with_brush();
        let slanted = Plane::from_point_normal(DVec3::new(32.0, 32.0, 32.0), DVec3::new(1.0, 1.0, 0.0).normalize());
        let undo_before = state.doc.history.undo_labels().count();
        clip_selection(&mut state, &slanted, ClipSide::Front);
        let kept = state.doc.selection.brushes(&state.doc.map);
        assert_eq!(kept.len(), 1);
        let b = state.doc.map.brush(kept[0]).unwrap().bounds();
        assert_eq!((b.min.x, b.min.y, b.max.x, b.max.y), (0.0, 0.0, 64.0, 64.0), "a diagonal cut keeps the corner's full reach");
        assert!(state.doc.map.brush(kept[0]).unwrap().vertices.iter().all(|v| v.x + v.y >= 64.0 - 1e-6), "only the +x+y half is left");
        assert_eq!(state.doc.history.undo_labels().count(), undo_before + 1);

        clip_selection(&mut state, &Plane::from_point_normal(DVec3::splat(48.0), DVec3::Z), ClipSide::Both);
        assert_eq!(state.doc.selection.brushes(&state.doc.map).len(), 2, "both halves stay and are selected");
    }

    #[test]
    fn move_vertices_sags_a_corner_and_refuses_concave_results() {
        let (mut state, id) = state_with_brush();
        assert!(move_vertices(&mut state, &[DVec3::new(64.0, 64.0, 64.0)], DVec3::new(0.0, -24.0, 0.0)));
        let b = state.doc.map.brush(id).unwrap();
        assert!(b.vertices.iter().any(|v| (*v - DVec3::new(64.0, 40.0, 64.0)).length() < 1e-6));
        assert!(!move_vertices(&mut state, &[DVec3::new(1.0, 2.0, 3.0)], DVec3::X), "a point off the brush moves nothing");
    }

    #[test]
    fn switching_tools_mid_drag_closes_the_undo_step() {
        let (mut state, _) = state_with_brush();
        let mut tools = ToolSet::default();
        rotate_mid_drag(&mut tools, &mut state);
        let undo_before = state.doc.history.undo_labels().count();

        state.tool = ToolKind::Select;
        tools.sync(&state);
        assert!(tools.rotate.drag.is_none());
        tools.settle(&mut state);
        assert!(!state.doc.in_transaction(), "the rotate transaction is not left open");
        assert_eq!(state.doc.history.undo_labels().count(), undo_before + 1, "the rotation so far is kept as its own undo step");
    }

    #[test]
    fn escape_mid_drag_cancels_it() {
        let (mut state, id) = state_with_brush();
        let before = state.doc.map.brush(id).unwrap().clone();
        let mut tools = ToolSet::default();
        rotate_mid_drag(&mut tools, &mut state);
        let ctx = egui::Context::default();
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Key { key: Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE });
        ctx.begin_pass(input);
        assert!(tools.keys(&ctx, &mut state), "Escape is taken by the drag");
        assert!(!state.doc.in_transaction());
        assert_eq!(state.doc.map.brush(id), Some(&before), "the brush is back where it was");
        assert_eq!(state.tool, ToolKind::Rotate, "the tool stays, only the drag ends");
    }

    #[test]
    fn a_document_swap_forgets_node_ids() {
        let (mut state, id) = state_with_brush();
        let mut tools = ToolSet::default();
        tools.path.last = Some(id);
        tools.mesh.selection.insert(id, Default::default());
        state.scene_reset = true;
        tools.settle(&mut state);
        assert!(tools.path.last.is_none(), "the next path corner does not link into another map's node");
        assert!(tools.mesh.selection.is_empty());
    }

    /// A flat 4096 unit terrain around the origin and a camera looking straight down at it.
    fn terrain_under_camera() -> (EditorState, NodeId, Camera) {
        let mut state = EditorState::new(Default::default());
        let layer = state.doc.map.default_layer();
        let terrain = gt_geom::Terrain::new(DVec3::new(-2048.0, 0.0, -2048.0), [65, 65], 64.0, "dev/grey");
        let id = state.doc.edit("terrain", |m, _| m.insert(layer, gt_doc::NodeKind::Terrain(terrain)));
        state.tool = ToolKind::Sculpt;
        let mut cam = Camera::new(ViewKind::Perspective);
        cam.position = DVec3::new(0.0, 3000.0, 0.0);
        cam.pitch = (-89.0f64).to_radians();
        (state, id, cam)
    }

    /// Runs one frame of the viewport tools over a 400 by 400 view with the pointer at its center.
    fn view_frame(ctx: &egui::Context, tools: &mut ToolSet, state: &mut EditorState, cam: &Camera, pressed: Option<bool>) {
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::splat(400.0));
        let mut events = vec![egui::Event::PointerMoved(rect.center())];
        if let Some(pressed) = pressed {
            events.push(egui::Event::PointerButton { pos: rect.center(), button: PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE });
        }

        let raw = egui::RawInput { screen_rect: Some(rect), events, ..Default::default() };
        let mut output = ctx.run_ui(raw, |ui| {
            let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
            let hover = response.hover_pos();
            tools.viewport_input(ui, &response, cam, rect, hover, state);
        });
        output.textures_delta.clear();
    }

    fn click(tools: &mut ToolSet, state: &mut EditorState, cam: &Camera) {
        let ctx = egui::Context::default();
        for pressed in [None, Some(true), Some(false), None] {
            view_frame(&ctx, tools, state, cam, pressed);
        }
    }

    #[test]
    fn a_sculpt_click_raises_a_visible_bump_and_a_no_op_stroke_leaves_no_undo_step() {
        let (mut state, id, cam) = terrain_under_camera();
        let mut tools = ToolSet::default();
        state.sculpt.radius = 256.0;
        click(&mut tools, &mut state, &cam);
        let top = state.doc.map.terrain(id).unwrap().heights.iter().copied().fold(0.0f32, f32::max);
        let expected = timed_sculpt_brush(state.sculpt, MIN_STROKE_SECONDS).strength as f32;
        assert!(top >= expected * 0.9, "a click raises the ground by a quarter second of holding, {top} of {expected}");
        assert!(expected >= 8.0, "{expected}");
        assert_eq!(state.doc.history.undo_labels().next(), Some("Sculpt"));

        let steps = state.doc.history.undo_labels().count();
        state.sculpt.mode = gt_doc::terrain::SculptMode::Flatten;
        state.doc.edit("level", |m, _| m.terrain_mut(id).unwrap().heights.fill(0.0));
        let steps = steps + 1;
        click(&mut tools, &mut state, &cam);
        assert_eq!(state.doc.history.undo_labels().count(), steps, "flattening flat ground changes nothing and records nothing");
    }

    #[test]
    fn sculpt_speed_keeps_the_default_brush_fast_and_ignores_the_radius() {
        let default = gt_doc::terrain::SculptBrush::default();
        assert!(timed_sculpt_brush(default, 1.0).strength >= 32.0, "the default brush raises at least 32 units a second");
        let big = gt_doc::terrain::SculptBrush { radius: 2800.0, ..default };
        assert_eq!(timed_sculpt_brush(big, 0.5).strength, timed_sculpt_brush(default, 0.5).strength, "a big brush is not faster");
    }
}
