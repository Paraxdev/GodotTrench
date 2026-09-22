//! Blender style mesh editing: vertex, edge and face selection with modal grab, rotate and scale,
//! extrude, inset, bevel, loop cut, knife and the usual topology operations. Double clicking a vertex, edge or face
//! selects it and puts the transform gizmo on it, Escape or a click on empty space hides the gizmo again.

use std::collections::{BTreeMap, BTreeSet};

use egui::{Align2, Color32, FontId, Key, Modifiers, PointerButton, Pos2, Rect, Response, Stroke, Ui, Vec2};
use gt_core::{Aabb, DMat4, DQuat, DVec3, NodeId, Plane};
use gt_geom::Mesh;
use gt_render::LineVertex;

use crate::camera::{Camera, ViewKind};
use crate::scene::v3;
use crate::state::EditorState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Component {
    #[default]
    Vertex,
    Edge,
    Face,
}

impl Component {
    pub fn label(&self) -> &'static str {
        match self {
            Component::Vertex => "vertex",
            Component::Edge => "edge",
            Component::Face => "face",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshSel {
    pub verts: BTreeSet<u32>,
    pub edges: BTreeSet<(u32, u32)>,
    pub faces: BTreeSet<usize>,
}

impl MeshSel {
    pub fn is_empty(&self) -> bool {
        self.verts.is_empty() && self.edges.is_empty() && self.faces.is_empty()
    }

    /// Vertices affected by a transform in the current component mode.
    pub fn moved_vertices(&self, mesh: &Mesh) -> Vec<u32> {
        let mut set = self.verts.clone();
        for (a, b) in &self.edges {
            set.insert(*a);
            set.insert(*b);
        }

        set.extend(mesh.face_vertices(&self.faces.iter().copied().collect::<Vec<_>>()));
        set.into_iter().filter(|v| (*v as usize) < mesh.vertices.len()).collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransformKind {
    Grab,
    Rotate,
    Scale,
}

#[derive(Clone, Copy, Debug)]
pub struct Constraint {
    pub dir: DVec3,
    /// Shift+axis: move within the plane perpendicular to `dir`.
    pub exclude: bool,
    pub label: &'static str,
}

#[derive(Clone, Debug)]
pub enum Modal {
    Transform { kind: TransformKind, constraint: Option<Constraint>, start: Option<Pos2>, center: DVec3, numeric: String, view: Option<ViewKind> },
    Inset { start: Option<Pos2>, center: DVec3, view: Option<ViewKind> },
    Bevel { start: Option<Pos2>, center: DVec3, vertices: bool, view: Option<ViewKind> },
    LoopCut { hover: Option<(NodeId, (u32, u32))>, cuts: usize },
    Knife { first: Option<Pos2>, view: Option<ViewKind> },
}

#[derive(Default)]
pub struct MeshTool {
    pub component: Component,
    pub selection: BTreeMap<NodeId, MeshSel>,
    pub modal: Option<Modal>,
    box_start: Option<Pos2>,
    drag_grab: bool,
    /// The transform gizmo on the selection, shown by double clicking a component.
    pub gizmo: bool,
    /// A gizmo drag and the view it started in.
    gizmo_drag: Option<(crate::transform_gizmo::GizmoDrag, ViewKind)>,
    /// Wheel movement not yet turned into a loop cut step, for wheels and touchpads that report fractions of a notch.
    wheel: f32,
}

const PICK_RADIUS: f32 = 8.0;

fn project_all(cam: &Camera, rect: Rect, mesh: &Mesh) -> Vec<Option<Pos2>> {
    mesh.vertices.iter().map(|v| cam.project(rect, *v)).collect()
}

fn seg_dist(p: Pos2, a: Pos2, b: Pos2) -> (f32, f32) {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_sq().max(1e-6)).clamp(0.0, 1.0);
    ((a + ab * t - p).length(), t)
}

pub fn edit_meshes(state: &EditorState) -> Vec<NodeId> {
    state.doc.selection.meshes(&state.doc.map).into_iter().filter(|id| state.doc.map.is_editable(*id)).collect()
}

impl MeshTool {
    pub fn reset(&mut self) {
        self.modal = None;
        self.box_start = None;
        self.drag_grab = false;
        self.wheel = 0.0;
        self.gizmo = false;
        self.gizmo_drag = None;
    }

    /// A modal or gizmo drag that edits inside an open undo transaction.
    pub fn holds_transaction(&self) -> bool {
        matches!(self.modal, Some(Modal::Transform { .. } | Modal::Inset { .. } | Modal::Bevel { .. })) || self.gizmo_drag.is_some()
    }

    /// Bounds of the vertices the gizmo moves, None while it is hidden.
    pub fn gizmo_bounds(&self, state: &EditorState) -> Option<Aabb> {
        if !self.gizmo || self.modal.is_some() {
            return None;
        }

        let points: Vec<DVec3> = self
            .selection
            .iter()
            .filter_map(|(id, sel)| state.doc.map.mesh(*id).map(|m| sel.moved_vertices(m).into_iter().map(|v| m.vertices[v as usize]).collect::<Vec<_>>()))
            .flatten()
            .collect();
        (!points.is_empty()).then(|| Aabb::from_points(points))
    }

    fn on_gizmo(&self, cam: &Camera, rect: Rect, pos: Pos2, state: &EditorState) -> bool {
        self.gizmo_bounds(state).and_then(|b| crate::transform_gizmo::hit_on(cam, rect, &b, pos)).is_some()
    }

    fn drag_gizmo(&mut self, cam: &Camera, rect: Rect, pos: Pos2, modifiers: Modifiers, state: &mut EditorState) {
        let Some((drag, _)) = &self.gizmo_drag else { return };
        let Some((motion, status)) = crate::transform_gizmo::motion(state, drag, cam, rect, pos, modifiers) else { return };
        state.doc.reset_transaction();
        if let Some(motion) = motion {
            let m = motion.matrix();
            let selection = self.selection.clone();
            state.doc.edit("Transform", |map, _| {
                for (id, sel) in &selection {
                    if let Some(mesh) = map.mesh_mut(*id) {
                        let verts = sel.moved_vertices(mesh);
                        mesh.transform_vertices(&verts, &m);
                    }
                }
            });
        }

        if let Some(status) = status {
            state.set_status(status);
        }
    }

    /// The loop cut modal steps its cut count with the wheel, so the views must not dolly meanwhile.
    pub fn takes_wheel(&self) -> bool {
        matches!(self.modal, Some(Modal::LoopCut { .. }))
    }

    /// Drops selections of meshes that are no longer edited and indices that went out of range.
    pub fn prune(&mut self, state: &EditorState) {
        let meshes: BTreeSet<NodeId> = edit_meshes(state).into_iter().collect();
        self.selection.retain(|id, _| meshes.contains(id));
        for (id, sel) in &mut self.selection {
            let Some(m) = state.doc.map.mesh(*id) else { continue };
            let nv = m.vertices.len() as u32;
            sel.verts.retain(|v| *v < nv);
            sel.edges.retain(|(a, b)| *a < nv && *b < nv);
            sel.faces.retain(|f| *f < m.faces.len());
        }

        if !self.has_selection() {
            self.gizmo = false;
        }
    }

    fn has_selection(&self) -> bool {
        self.selection.values().any(|s| !s.is_empty())
    }

    fn selection_center(&self, state: &EditorState) -> Option<DVec3> {
        let mut sum = DVec3::ZERO;
        let mut n = 0.0;
        for (id, sel) in &self.selection {
            let Some(m) = state.doc.map.mesh(*id) else { continue };
            for v in sel.moved_vertices(m) {
                sum += m.vertices[v as usize];
                n += 1.0;
            }
        }

        (n > 0.0).then(|| sum / n)
    }

    fn average_normal(&self, state: &EditorState) -> DVec3 {
        let mut sum = DVec3::ZERO;
        for (id, sel) in &self.selection {
            let Some(m) = state.doc.map.mesh(*id) else { continue };
            for f in &sel.faces {
                sum += m.face_normal(*f);
            }

            if sel.faces.is_empty() {
                let verts: BTreeSet<u32> = sel.moved_vertices(m).into_iter().collect();
                for (fi, face) in m.faces.iter().enumerate() {
                    if face.indices.iter().any(|v| verts.contains(v)) {
                        sum += m.face_normal(fi);
                    }
                }
            }
        }

        sum.normalize_or(DVec3::Y)
    }

    // ----------------------------------------------------------------- keyboard

    /// Returns true if the key was consumed.
    pub fn keys(&mut self, ctx: &egui::Context, state: &mut EditorState) -> bool {
        self.prune(state);
        let pressed = |ctx: &egui::Context, m: Modifiers, k: Key| ctx.input_mut(|i| i.consume_key(m, k));
        if let Some(modal) = self.modal.clone() {
            if pressed(ctx, Modifiers::NONE, Key::Escape) {
                self.cancel(state);
                return true;
            }

            if pressed(ctx, Modifiers::NONE, Key::Enter) {
                self.confirm(state);
                return true;
            }

            if let Modal::Transform { kind, constraint, start, center, mut numeric, view } = modal {
                let mut constraint = constraint;
                for (key, dir, label) in [(Key::X, DVec3::X, "X"), (Key::Y, DVec3::Y, "Y"), (Key::Z, DVec3::Z, "Z")] {
                    let shift = pressed(ctx, Modifiers::SHIFT, key);
                    if shift || pressed(ctx, Modifiers::NONE, key) {
                        constraint = match constraint {
                            Some(c) if c.label == label && c.exclude == shift => None,
                            _ => Some(Constraint { dir, exclude: shift, label }),
                        };
                    }
                }

                let typed: String = ctx.input(|i| {
                    i.events
                        .iter()
                        .filter_map(|e| match e {
                            egui::Event::Text(t) => Some(t.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-').collect::<String>()),
                            _ => None,
                        })
                        .collect()
                });
                numeric.push_str(&typed);
                if pressed(ctx, Modifiers::NONE, Key::Backspace) {
                    numeric.pop();
                }

                self.modal = Some(Modal::Transform { kind, constraint, start, center, numeric, view });
                // Swallow remaining letters so they do not trigger tool shortcuts mid transform.
                ctx.input_mut(|i| i.events.retain(|e| !matches!(e, egui::Event::Key { pressed: true, .. } | egui::Event::Text(_))));
                return true;
            }

            if let Modal::LoopCut { hover, cuts } = modal {
                // One step per wheel notch. egui's smoothed scroll spreads a notch over several frames.
                self.wheel += ctx.input(|i| crate::viewport::wheel_notches(i, |_| true));
                let mut cuts = cuts;
                if pressed(ctx, Modifiers::NONE, Key::Plus) || pressed(ctx, Modifiers::NONE, Key::Equals) {
                    cuts = (cuts + 1).min(32);
                }

                if pressed(ctx, Modifiers::NONE, Key::Minus) {
                    cuts = cuts.saturating_sub(1).max(1);
                }

                while self.wheel >= 1.0 {
                    self.wheel -= 1.0;
                    cuts = (cuts + 1).min(32);
                }

                while self.wheel <= -1.0 {
                    self.wheel += 1.0;
                    cuts = cuts.saturating_sub(1).max(1);
                }

                self.modal = Some(Modal::LoopCut { hover, cuts });
            }

            return false;
        }

        let none = Modifiers::NONE;
        let ctrl = Modifiers::COMMAND;
        let shift = Modifiers::SHIFT;
        let alt = Modifiers::ALT;
        let ctrl_shift = Modifiers { shift: true, ..Modifiers::COMMAND };
        if self.gizmo && self.gizmo_drag.is_none() && pressed(ctx, none, Key::Escape) {
            self.gizmo = false;
            return true;
        }

        if pressed(ctx, none, Key::Num1) {
            self.component = Component::Vertex;
            return true;
        }

        if pressed(ctx, none, Key::Num2) {
            self.component = Component::Edge;
            return true;
        }

        if pressed(ctx, none, Key::Num3) {
            self.component = Component::Face;
            return true;
        }

        if pressed(ctx, alt, Key::A) {
            self.selection.values_mut().for_each(|s| *s = MeshSel::default());
            return true;
        }

        if pressed(ctx, none, Key::A) || pressed(ctx, ctrl, Key::A) {
            self.select_all(state);
            return true;
        }

        if pressed(ctx, ctrl, Key::R) {
            self.modal = Some(Modal::LoopCut { hover: None, cuts: 1 });
            self.wheel = 0.0;
            return true;
        }

        if pressed(ctx, none, Key::K) {
            self.modal = Some(Modal::Knife { first: None, view: None });
            state.set_status("Knife: click two points to cut, Esc cancels");
            return true;
        }

        if !self.has_selection() {
            return false;
        }

        if pressed(ctx, none, Key::G) {
            self.start_transform(TransformKind::Grab, None, state);
            return true;
        }

        if pressed(ctx, none, Key::R) {
            self.start_transform(TransformKind::Rotate, None, state);
            return true;
        }

        if pressed(ctx, none, Key::S) {
            self.start_transform(TransformKind::Scale, None, state);
            return true;
        }

        if pressed(ctx, none, Key::E) {
            self.extrude(state);
            return true;
        }

        if pressed(ctx, none, Key::I) {
            if let Some(center) = self.selection_center(state) {
                state.doc.begin("Inset Faces");
                self.modal = Some(Modal::Inset { start: None, center, view: None });
            }

            return true;
        }

        if pressed(ctx, ctrl_shift, Key::B) {
            if let Some(center) = self.selection_center(state) {
                state.doc.begin("Bevel Vertices");
                self.modal = Some(Modal::Bevel { start: None, center, vertices: true, view: None });
            }

            return true;
        }

        if pressed(ctx, ctrl, Key::B) {
            if let Some(center) = self.selection_center(state) {
                state.doc.begin("Bevel Edges");
                self.modal = Some(Modal::Bevel { start: None, center, vertices: false, view: None });
            }

            return true;
        }

        if pressed(ctx, none, Key::M) {
            self.run(state, MeshOp::MergeCenter);
            return true;
        }

        if pressed(ctx, none, Key::F) {
            self.run(state, MeshOp::Fill);
            return true;
        }

        if pressed(ctx, none, Key::X) || pressed(ctx, none, Key::Delete) {
            self.run(state, MeshOp::Delete);
            return true;
        }

        if pressed(ctx, shift, Key::D) {
            self.run(state, MeshOp::Duplicate);
            self.start_transform(TransformKind::Grab, None, state);
            return true;
        }

        if pressed(ctx, none, Key::P) {
            self.run(state, MeshOp::Separate);
            return true;
        }

        if pressed(ctx, alt, Key::F) {
            self.run(state, MeshOp::Flip);
            return true;
        }

        if pressed(ctx, ctrl, Key::L) {
            self.run(state, MeshOp::SelectLinked);
            return true;
        }

        false
    }

    pub fn select_all(&mut self, state: &EditorState) {
        for id in edit_meshes(state) {
            let Some(m) = state.doc.map.mesh(id) else { continue };
            let sel = self.selection.entry(id).or_default();
            *sel = MeshSel::default();
            match self.component {
                Component::Vertex => sel.verts = (0..m.vertices.len() as u32).collect(),
                Component::Edge => sel.edges = m.edges().into_iter().collect(),
                Component::Face => sel.faces = (0..m.faces.len()).collect(),
            }
        }
    }

    pub fn start_transform(&mut self, kind: TransformKind, constraint: Option<Constraint>, state: &mut EditorState) {
        let Some(center) = self.selection_center(state) else { return };
        state.doc.begin(match kind {
            TransformKind::Grab => "Grab",
            TransformKind::Rotate => "Rotate Vertices",
            TransformKind::Scale => "Scale Vertices",
        });
        self.modal = Some(Modal::Transform { kind, constraint, start: None, center, numeric: String::new(), view: None });
    }

    fn extrude(&mut self, state: &mut EditorState) {
        let normal = self.average_normal(state);
        let component = self.component;
        let selection = self.selection.clone();
        let mut new_sel = BTreeMap::new();
        state.doc.edit("Extrude", |m, _| {
            for (id, sel) in &selection {
                let Some(mesh) = m.mesh_mut(*id) else { continue };
                let mut out = MeshSel::default();
                match component {
                    Component::Face if !sel.faces.is_empty() => {
                        let faces: Vec<usize> = sel.faces.iter().copied().collect();
                        out.verts = mesh.extrude_faces(&faces).into_iter().collect();
                        out.faces = sel.faces.clone();
                    }
                    _ => {
                        let edges: Vec<(u32, u32)> = if sel.edges.is_empty() {
                            let verts: BTreeSet<u32> = sel.verts.clone();
                            mesh.boundary_edges().into_iter().filter(|(a, b)| verts.contains(a) && verts.contains(b)).collect()
                        } else {
                            sel.edges.iter().copied().collect()
                        };
                        let map: BTreeMap<u32, u32> = mesh.extrude_edges(&edges).into_iter().collect();
                        out.edges = edges.iter().filter_map(|(a, b)| Some(gt_geom::mesh::edge_key(*map.get(a)?, *map.get(b)?))).collect();
                    }
                }

                new_sel.insert(*id, out);
            }
        });
        self.selection = new_sel;
        // Extrusion is its own undo step, so cancelling the following grab keeps the new geometry like Blender does.
        let constraint = (component == Component::Face).then_some(Constraint { dir: normal, exclude: false, label: "normal" });
        self.start_transform(TransformKind::Grab, constraint, state);
    }

    fn cancel(&mut self, state: &mut EditorState) {
        if matches!(self.modal, Some(Modal::Transform { .. } | Modal::Inset { .. } | Modal::Bevel { .. })) {
            state.doc.cancel();
        }

        self.modal = None;
        self.prune(state);
    }

    fn confirm(&mut self, state: &mut EditorState) {
        if matches!(self.modal, Some(Modal::Transform { .. } | Modal::Inset { .. } | Modal::Bevel { .. })) {
            state.doc.commit();
        }

        self.modal = None;
        self.prune(state);
    }

    // -------------------------------------------------------------------- operations

    pub fn run(&mut self, state: &mut EditorState, op: MeshOp) {
        let selection = self.selection.clone();
        let component = self.component;
        let material = state.current_material.clone();
        let parent = state.insert_parent();
        let grid = state.grid.max(1.0);
        let mut new_selection: Option<BTreeMap<NodeId, MeshSel>> = None;
        let mut new_nodes: Vec<NodeId> = Vec::new();
        let label = op.label();
        state.doc.edit(label, |m, s| {
            let mut next = BTreeMap::new();
            for (id, sel) in &selection {
                let Some(mesh) = m.mesh_mut(*id) else { continue };
                let faces: Vec<usize> = sel.faces.iter().copied().collect();
                let verts = sel.moved_vertices(mesh);
                let edges: Vec<(u32, u32)> = if sel.edges.is_empty() {
                    let vs: BTreeSet<u32> = verts.iter().copied().collect();
                    mesh.edges_within(&vs)
                } else {
                    sel.edges.iter().copied().collect()
                };
                let mut out = MeshSel::default();
                match op {
                    MeshOp::MergeCenter => {
                        if verts.len() >= 2 {
                            let center = verts.iter().map(|v| mesh.vertices[*v as usize]).sum::<DVec3>() / verts.len() as f64;
                            mesh.merge_vertices(&verts, center);
                        }
                    }
                    MeshOp::MergeByDistance => {
                        mesh.merge_by_distance(&verts, 0.5);
                    }
                    MeshOp::Fill => {
                        let data = mesh
                            .faces
                            .first()
                            .map(|f| gt_geom::FaceData { material: f.data.material.clone(), ..Default::default() })
                            .unwrap_or_else(|| gt_geom::FaceData::new(&material, Default::default()));
                        if let Some(f) = mesh.fill(&verts, &data) {
                            out.faces.insert(f);
                        }
                    }
                    MeshOp::Delete => match component {
                        Component::Vertex => mesh.delete_vertices(&verts),
                        Component::Edge => mesh.delete_edges(&edges),
                        Component::Face => mesh.delete_faces(&faces),
                    },
                    MeshOp::Dissolve => mesh.dissolve_vertices(&verts),
                    MeshOp::Subdivide => {
                        let faces = if faces.is_empty() { mesh.faces_within(&verts.iter().copied().collect()) } else { faces.clone() };
                        out.faces = mesh.subdivide_faces(&faces).into_iter().collect();
                    }
                    MeshOp::Triangulate => mesh.triangulate_faces(&faces),
                    MeshOp::Flip => {
                        let faces = if faces.is_empty() { (0..mesh.faces.len()).collect() } else { faces.clone() };
                        mesh.flip_faces(&faces);
                        out.faces = sel.faces.clone();
                    }
                    MeshOp::Smooth => {
                        mesh.smooth_vertices(&verts, 0.5, 2);
                        out = sel.clone();
                    }
                    MeshOp::Solidify => mesh.solidify(grid),
                    MeshOp::Duplicate => {
                        let faces = if faces.is_empty() { mesh.faces_within(&verts.iter().copied().collect()) } else { faces.clone() };
                        out.faces = mesh.duplicate_faces(&faces).into_iter().collect();
                    }
                    MeshOp::Separate => {
                        let faces = if faces.is_empty() { mesh.faces_within(&verts.iter().copied().collect()) } else { faces.clone() };
                        if !faces.is_empty() && faces.len() < mesh.faces.len() {
                            let part = mesh.separate(&faces);
                            let parent_id = m.get(*id).and_then(|n| n.parent).unwrap_or(parent);
                            new_nodes.push(m.insert(parent_id, gt_doc::NodeKind::Mesh(part)));
                        }
                    }
                    MeshOp::SelectLinked => {
                        let seeds = if faces.is_empty() { mesh.faces_within(&verts.iter().copied().collect()) } else { faces.clone() };
                        let seeds = if seeds.is_empty() {
                            let vs: BTreeSet<u32> = verts.iter().copied().collect();
                            (0..mesh.faces.len()).filter(|f| mesh.faces[*f].indices.iter().any(|v| vs.contains(v))).collect()
                        } else {
                            seeds
                        };
                        let linked = mesh.linked_faces(&seeds);
                        match component {
                            Component::Face => out.faces = linked.into_iter().collect(),
                            _ => {
                                let vs = mesh.face_vertices(&linked);
                                out.verts = vs.iter().copied().collect();
                                if component == Component::Edge {
                                    out.edges = mesh.edges_within(&out.verts).into_iter().collect();
                                    out.verts.clear();
                                }
                            }
                        }
                    }
                    MeshOp::ShadeSmooth => mesh.smooth_angle = 60.0,
                    MeshOp::ShadeFlat => mesh.smooth_angle = 0.0,
                    MeshOp::Mirror(axis) => {
                        if !verts.is_empty() {
                            let center = verts.iter().map(|v| mesh.vertices[*v as usize]).sum::<DVec3>() / verts.len() as f64;
                            let mut scale = DVec3::ONE;
                            scale[axis] = -1.0;
                            let mx = DMat4::from_translation(center) * DMat4::from_scale(scale) * DMat4::from_translation(-center);
                            if faces.len() == mesh.faces.len() || verts.len() == mesh.vertices.len() {
                                *mesh = mesh.transformed(&mx, false);
                            } else {
                                mesh.transform_vertices(&verts, &mx);
                                let moved: BTreeSet<u32> = verts.iter().copied().collect();
                                let mirrored = mesh.faces_within(&moved);
                                mesh.flip_faces(&mirrored);
                            }
                        }

                        out = sel.clone();
                    }
                    MeshOp::SnapToGrid => {
                        for v in &verts {
                            mesh.vertices[*v as usize] = gt_core::snap_vec_to_grid(mesh.vertices[*v as usize], grid);
                        }

                        out = sel.clone();
                    }
                }

                next.insert(*id, out);
            }

            if !new_nodes.is_empty() {
                s.nodes.extend(new_nodes.iter().copied());
            }

            new_selection = Some(next);
        });
        if let Some(sel) = new_selection {
            self.selection = sel;
        }

        self.prune(state);
        state.set_status(label);
    }

    // ------------------------------------------------------------------------ mouse

    pub fn viewport_input(&mut self, ui: &Ui, response: &Response, cam: &Camera, rect: Rect, hover: Option<Pos2>, state: &mut EditorState) {
        self.prune(state);
        let modifiers = ui.input(|i| i.modifiers);
        let pointer = ui.input(|i| i.pointer.hover_pos()).or(hover);

        if let Some(modal) = self.modal.clone() {
            self.update_modal(ui, response, cam, rect, pointer, modal, state);
            return;
        }

        if let Some((_, view)) = &self.gizmo_drag {
            if *view == cam.kind {
                if ui.input(|i| i.pointer.primary_down()) {
                    if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                        self.drag_gizmo(cam, rect, pos, modifiers, state);
                    }
                } else {
                    state.doc.commit();
                    self.gizmo_drag = None;
                }
            }

            return;
        }

        if response.clicked_by(PointerButton::Primary)
            && let Some(pos) = response.interact_pointer_pos()
            && !self.on_gizmo(cam, rect, pos, state)
        {
            let extend = modifiers.shift || modifiers.command;
            if modifiers.alt {
                self.select_loop_at(cam, rect, pos, modifiers.command, extend, state);
            } else {
                self.click_select(cam, rect, pos, extend, state);
            }
        }

        if response.double_clicked_by(PointerButton::Primary)
            && !modifiers.alt
            && let Some(pos) = response.interact_pointer_pos()
            && !self.on_gizmo(cam, rect, pos, state)
            && let Some((id, c)) = self.component_at(cam, rect, pos, state)
        {
            if !(modifiers.shift || modifiers.command) {
                self.selection.values_mut().for_each(|s| *s = MeshSel::default());
            }

            let sel = self.selection.entry(id).or_default();
            match c {
                Picked::Vertex(v) => _ = sel.verts.insert(v),
                Picked::Edge((a, b)) => _ = sel.edges.insert(gt_geom::mesh::edge_key(a, b)),
                Picked::Face(f) => _ = sel.faces.insert(f),
            }

            self.gizmo = true;
            state.set_status("Gizmo: drag an arrow to move along one axis, a square for a plane, a ring rotates, a box scales. Esc hides it");
        }

        if response.drag_started_by(PointerButton::Primary)
            && let Some(origin) = ui.input(|i| i.pointer.press_origin())
            && let Some(bounds) = self.gizmo_bounds(state)
            && let Some(drag) = crate::transform_gizmo::begin_on(cam, rect, bounds, origin)
        {
            state.doc.begin(match drag.part {
                crate::transform_gizmo::Part::Rotate(_) => "Rotate Vertices",
                crate::transform_gizmo::Part::Scale(_) => "Scale Vertices",
                _ => "Move Vertices",
            });
            self.gizmo_drag = Some((drag, cam.kind));
            return;
        }

        if response.drag_started_by(PointerButton::Primary)
            && let Some(origin) = ui.input(|i| i.pointer.press_origin())
        {
            let on_selected = self.component_at(cam, rect, origin, state).is_some_and(|(id, c)| self.is_selected(id, &c));
            if on_selected && !modifiers.shift {
                self.drag_grab = true;
                self.start_transform(TransformKind::Grab, None, state);
                if let Some(Modal::Transform { start, view, .. }) = &mut self.modal {
                    *start = Some(origin);
                    *view = Some(cam.kind);
                }
            } else {
                self.box_start = Some(origin);
            }
        }

        if let Some(start) = self.box_start {
            if !ui.input(|i| i.pointer.primary_down()) {
                if let Some(end) = pointer {
                    self.box_select(cam, rect, Rect::from_two_pos(start, end), modifiers.shift || modifiers.command, state);
                }

                self.box_start = None;
            } else {
                ui.ctx().request_repaint();
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn update_modal(&mut self, ui: &Ui, response: &Response, cam: &Camera, rect: Rect, pointer: Option<Pos2>, modal: Modal, state: &mut EditorState) {
        let hovered = response.hovered() || self.drag_grab;
        let confirm_click = response.clicked_by(PointerButton::Primary) || (self.drag_grab && !ui.input(|i| i.pointer.primary_down()));
        let cancel_click = response.clicked_by(PointerButton::Secondary);
        match modal {
            Modal::Transform { kind, constraint, start, center, numeric, view } => {
                if view.is_some_and(|v| v != cam.kind) || !hovered {
                    return;
                }

                let Some(pos) = pointer else { return };
                let start = start.unwrap_or(pos);
                self.modal = Some(Modal::Transform { kind, constraint, start: Some(start), center, numeric: numeric.clone(), view: Some(cam.kind) });
                if cancel_click {
                    self.cancel(state);
                    self.drag_grab = false;
                    return;
                }

                let snap = state.snap != ui.input(|i| i.modifiers.command);
                let m = transform_matrix(kind, constraint, start, pos, center, &numeric, cam, rect, snap, state.grid);
                state.doc.reset_transaction();
                let selection = self.selection.clone();
                state.doc.edit("Transform", |map, _| {
                    for (id, sel) in &selection {
                        if let Some(mesh) = map.mesh_mut(*id) {
                            let verts = sel.moved_vertices(mesh);
                            mesh.transform_vertices(&verts, &m);
                            // A negative scale mirrors, so the faces it moved whole need their winding flipped back.
                            if m.determinant() < 0.0 {
                                let mirrored = mesh.faces_within(&verts.iter().copied().collect());
                                mesh.flip_faces(&mirrored);
                            }
                        }
                    }
                });
                let (_, _, t) = m.to_scale_rotation_translation();
                state.set_status(format!(
                    "{kind:?} {}  {}  (X/Y/Z constrain, Shift+axis plane, type a value, Enter or click confirms, Esc cancels)",
                    constraint.map(|c| if c.exclude { format!("not {}", c.label) } else { c.label.to_string() }).unwrap_or_else(|| "free".into()),
                    if numeric.is_empty() { format!("{:.2} {:.2} {:.2}", t.x, t.y, t.z) } else { numeric.clone() }
                ));
                if confirm_click {
                    self.confirm(state);
                    self.drag_grab = false;
                }

                ui.ctx().request_repaint();
            }
            Modal::Inset { start, center, view } | Modal::Bevel { start, center, view, .. } => {
                if view.is_some_and(|v| v != cam.kind) || !response.hovered() {
                    return;
                }

                let Some(pos) = pointer else { return };
                let start = start.unwrap_or(pos);
                let vertices = matches!(self.modal, Some(Modal::Bevel { vertices: true, .. }));
                let is_bevel = matches!(self.modal, Some(Modal::Bevel { .. }));
                self.modal = Some(if is_bevel {
                    Modal::Bevel { start: Some(start), center, vertices, view: Some(cam.kind) }
                } else {
                    Modal::Inset { start: Some(start), center, view: Some(cam.kind) }
                });
                if cancel_click {
                    self.cancel(state);
                    return;
                }

                let px_per_unit = cam
                    .project(rect, center)
                    .zip(cam.project(rect, center + cam.right() * 16.0))
                    .map(|(a, b)| ((b - a).length() / 16.0).max(1e-3))
                    .unwrap_or(1.0);
                let amount = state.snap_scalar(((pos - start).length() / px_per_unit) as f64 * 0.5).max(0.0);
                state.doc.reset_transaction();
                let selection = self.selection.clone();
                let component = self.component;
                state.doc.edit("Modal", |map, _| {
                    for (id, sel) in &selection {
                        let Some(mesh) = map.mesh_mut(*id) else { continue };
                        if amount <= 0.0 {
                            continue;
                        }

                        if !is_bevel {
                            mesh.inset_faces(&sel.faces.iter().copied().collect::<Vec<_>>(), amount);
                        } else if vertices || component == Component::Vertex {
                            mesh.bevel_vertices(&sel.moved_vertices(mesh), amount);
                        } else {
                            mesh.bevel_edges(&sel.edges.iter().copied().collect::<Vec<_>>(), amount);
                        }
                    }
                });
                state.set_status(format!("{} {amount:.2} (move the mouse, click to confirm)", if is_bevel { "Bevel" } else { "Inset" }));
                if confirm_click {
                    self.confirm(state);
                    if is_bevel {
                        self.selection.values_mut().for_each(|s| *s = MeshSel::default());
                    }
                }

                ui.ctx().request_repaint();
            }
            Modal::LoopCut { cuts, .. } => {
                let hover = pointer.filter(|_| response.hovered()).and_then(|pos| self.edge_at(cam, rect, pos, state));
                self.modal = Some(Modal::LoopCut { hover, cuts });
                if cancel_click {
                    self.modal = None;
                    return;
                }

                if response.clicked_by(PointerButton::Primary)
                    && let Some((id, edge)) = hover
                {
                    let mut created = Vec::new();
                    state.doc.edit("Loop Cut", |m, _| {
                        if let Some(mesh) = m.mesh_mut(id) {
                            created = mesh.loop_cut(edge, cuts);
                        }
                    });
                    self.selection.clear();
                    self.selection.insert(id, MeshSel { verts: created.into_iter().collect(), ..Default::default() });
                    self.component = Component::Vertex;
                    self.modal = None;
                    state.set_status(format!("Loop cut with {cuts} cut(s), G slides the new loop"));
                }
            }
            Modal::Knife { first, view } => {
                if cancel_click {
                    self.modal = None;
                    return;
                }

                if response.clicked_by(PointerButton::Primary)
                    && let Some(pos) = response.interact_pointer_pos()
                {
                    match first {
                        None => self.modal = Some(Modal::Knife { first: Some(pos), view: Some(cam.kind) }),
                        Some(a) if view == Some(cam.kind) => {
                            self.knife(cam, rect, a, pos, state);
                            self.modal = None;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// Cuts faces crossed by the screen segment a-b with the plane through it.
    fn knife(&mut self, cam: &Camera, rect: Rect, a: Pos2, b: Pos2, state: &mut EditorState) {
        let (ra, rb) = (cam.ray(rect, a), cam.ray(rect, b));
        let plane = match cam.kind {
            ViewKind::Perspective => Plane::from_points(cam.position, ra.at(256.0), rb.at(256.0)),
            _ => {
                let (pa, pb) = (cam.screen_to_plane(rect, a), cam.screen_to_plane(rect, b));
                Plane::from_points(pa, pb, pa + cam.forward() * 64.0)
            }
        };
        let Some(plane) = plane else { return };
        let ab = b - a;
        let mut changed = 0;
        let meshes = edit_meshes(state);
        let selection = self.selection.clone();
        let component = self.component;
        state.doc.edit("Knife", |m, _| {
            for id in &meshes {
                let Some(mesh) = m.mesh_mut(*id) else { continue };
                let screen = project_all(cam, rect, mesh);
                let restrict: Option<BTreeSet<usize>> =
                    selection.get(id).filter(|s| component == Component::Face && !s.faces.is_empty()).map(|s| s.faces.clone());
                let faces: Vec<usize> = (0..mesh.faces.len())
                    .filter(|f| restrict.as_ref().is_none_or(|r| r.contains(f)))
                    .filter(|f| {
                        let pts: Vec<Pos2> = mesh.faces[*f].indices.iter().filter_map(|v| screen[*v as usize]).collect();
                        if pts.len() != mesh.faces[*f].indices.len() {
                            return false;
                        }

                        let ts: Vec<f32> = pts.iter().map(|p| (*p - a).dot(ab) / ab.length_sq().max(1e-6)).collect();
                        let lo = ts.iter().cloned().fold(f32::MAX, f32::min);
                        let hi = ts.iter().cloned().fold(f32::MIN, f32::max);
                        let sides: Vec<f32> = pts.iter().map(|p| ab.x * (p.y - a.y) - ab.y * (p.x - a.x)).collect();
                        hi >= -0.05 && lo <= 1.05 && sides.iter().any(|s| *s > 0.0) && sides.iter().any(|s| *s < 0.0)
                    })
                    .collect();
                if faces.is_empty() {
                    continue;
                }

                mesh.bisect(&plane, Some(&faces));
                changed += faces.len();
            }
        });
        state.set_status(format!("Knife cut {changed} face(s)"));
    }

    fn is_selected(&self, id: NodeId, c: &Picked) -> bool {
        let Some(sel) = self.selection.get(&id) else { return false };
        match c {
            Picked::Vertex(v) => sel.verts.contains(v),
            Picked::Edge(e) => sel.edges.contains(&gt_geom::mesh::edge_key(e.0, e.1)),
            Picked::Face(f) => sel.faces.contains(f),
        }
    }

    fn edge_at(&self, cam: &Camera, rect: Rect, pos: Pos2, state: &EditorState) -> Option<(NodeId, (u32, u32))> {
        let mut best: Option<(f32, NodeId, (u32, u32))> = None;
        for id in edit_meshes(state) {
            let Some(mesh) = state.doc.map.mesh(id) else { continue };
            let screen = project_all(cam, rect, mesh);
            for (a, b) in mesh.edges() {
                let (Some(pa), Some(pb)) = (screen[a as usize], screen[b as usize]) else { continue };
                let (d, _) = seg_dist(pos, pa, pb);
                if d < PICK_RADIUS && best.is_none_or(|(bd, ..)| d < bd) {
                    best = Some((d, id, (a, b)));
                }
            }
        }

        best.map(|(_, id, e)| (id, e))
    }

    fn component_at(&self, cam: &Camera, rect: Rect, pos: Pos2, state: &EditorState) -> Option<(NodeId, Picked)> {
        match self.component {
            Component::Vertex => {
                let mut best: Option<(f32, NodeId, u32)> = None;
                for id in edit_meshes(state) {
                    let Some(mesh) = state.doc.map.mesh(id) else { continue };
                    for (i, p) in project_all(cam, rect, mesh).into_iter().enumerate() {
                        let Some(p) = p else { continue };
                        let d = (p - pos).length();
                        if d < PICK_RADIUS && best.is_none_or(|(bd, ..)| d < bd) {
                            best = Some((d, id, i as u32));
                        }
                    }
                }

                best.map(|(_, id, v)| (id, Picked::Vertex(v)))
            }
            Component::Edge => self.edge_at(cam, rect, pos, state).map(|(id, e)| (id, Picked::Edge(e))),
            Component::Face => {
                let ray = cam.ray(rect, pos);
                edit_meshes(state)
                    .into_iter()
                    .filter_map(|id| state.doc.map.mesh(id).and_then(|m| m.ray_cast(&ray)).map(|(t, f)| (t, id, f)))
                    .min_by(|a, b| a.0.total_cmp(&b.0))
                    .map(|(_, id, f)| (id, Picked::Face(f)))
            }
        }
    }

    fn click_select(&mut self, cam: &Camera, rect: Rect, pos: Pos2, extend: bool, state: &mut EditorState) {
        let picked = self.component_at(cam, rect, pos, state);
        if !extend {
            self.selection.values_mut().for_each(|s| *s = MeshSel::default());
        }

        let Some((id, c)) = picked else {
            if !extend {
                self.gizmo = false;
            }

            // Clicking another object switches the edited mesh, like Blender's multi object edit.
            if !extend
                && let Some(h) = crate::picking::pick(state, &cam.ray(rect, pos))
                && state.doc.map.mesh(h.node).is_some()
            {
                state.doc.select(|_, s| {
                    s.clear();
                    s.select_node(h.node);
                });
            }

            return;
        };
        let sel = self.selection.entry(id).or_default();
        match c {
            Picked::Vertex(v) => {
                if !sel.verts.remove(&v) {
                    sel.verts.insert(v);
                }
            }
            Picked::Edge((a, b)) => {
                let key = gt_geom::mesh::edge_key(a, b);
                if !sel.edges.remove(&key) {
                    sel.edges.insert(key);
                }
            }
            Picked::Face(f) => {
                if !sel.faces.remove(&f) {
                    sel.faces.insert(f);
                }
            }
        }
    }

    fn select_loop_at(&mut self, cam: &Camera, rect: Rect, pos: Pos2, ring: bool, extend: bool, state: &EditorState) {
        let Some((id, edge)) = self.edge_at(cam, rect, pos, state) else { return };
        let Some(mesh) = state.doc.map.mesh(id) else { return };
        if !extend {
            self.selection.values_mut().for_each(|s| *s = MeshSel::default());
        }

        let edges = if ring { mesh.edge_ring(edge) } else { mesh.edge_loop(edge) };
        let sel = self.selection.entry(id).or_default();
        match self.component {
            Component::Vertex => sel.verts.extend(edges.iter().flat_map(|(a, b)| [*a, *b])),
            Component::Edge => sel.edges.extend(edges.iter().map(|(a, b)| gt_geom::mesh::edge_key(*a, *b))),
            Component::Face => {
                let verts: BTreeSet<u32> = edges.iter().flat_map(|(a, b)| [*a, *b]).collect();
                sel.faces.extend(mesh.faces_within(&verts));
                if ring {
                    let keys: BTreeSet<(u32, u32)> = edges.iter().map(|(a, b)| gt_geom::mesh::edge_key(*a, *b)).collect();
                    sel.faces.extend((0..mesh.faces.len()).filter(|f| {
                        let idx = &mesh.faces[*f].indices;
                        (0..idx.len()).filter(|k| keys.contains(&gt_geom::mesh::edge_key(idx[*k], idx[(k + 1) % idx.len()]))).count() >= 2
                    }));
                }
            }
        }
    }

    fn box_select(&mut self, cam: &Camera, rect: Rect, area: Rect, extend: bool, state: &EditorState) {
        if area.width() < 3.0 && area.height() < 3.0 {
            return;
        }

        if !extend {
            self.selection.values_mut().for_each(|s| *s = MeshSel::default());
        }

        for id in edit_meshes(state) {
            let Some(mesh) = state.doc.map.mesh(id) else { continue };
            let screen = project_all(cam, rect, mesh);
            let inside = |v: u32| screen[v as usize].is_some_and(|p| area.contains(p));
            let sel = self.selection.entry(id).or_default();
            match self.component {
                Component::Vertex => sel.verts.extend((0..mesh.vertices.len() as u32).filter(|v| inside(*v))),
                Component::Edge => sel.edges.extend(mesh.edges().into_iter().filter(|(a, b)| inside(*a) && inside(*b))),
                Component::Face => {
                    sel.faces.extend((0..mesh.faces.len()).filter(|f| cam.project(rect, mesh.face_center(*f)).is_some_and(|p| area.contains(p))))
                }
            }
        }
    }

    // -------------------------------------------------------------------- drawing

    pub fn lines(&self, cam: &Camera, rect: Rect, state: &EditorState) -> Vec<LineVertex> {
        let mut out = Vec::new();
        let mut line = |a: DVec3, b: DVec3, color: [f32; 4]| {
            out.push(LineVertex { pos: v3(a), color });
            out.push(LineVertex { pos: v3(b), color });
        };
        for id in edit_meshes(state) {
            let Some(mesh) = state.doc.map.mesh(id) else { continue };
            let sel = self.selection.get(&id);
            let mut face_edges: BTreeSet<(u32, u32)> = BTreeSet::new();
            for f in sel.map(|s| s.faces.iter()).into_iter().flatten() {
                if let Some(face) = mesh.faces.get(*f) {
                    let idx = &face.indices;
                    face_edges.extend((0..idx.len()).map(|k| gt_geom::mesh::edge_key(idx[k], idx[(k + 1) % idx.len()])));
                }
            }

            for (a, b) in mesh.edges() {
                let selected = face_edges.contains(&(a, b)) || sel.is_some_and(|s| s.edges.contains(&(a, b)) || (s.verts.contains(&a) && s.verts.contains(&b)));
                let color = if selected { [1.0, 0.6, 0.1, 1.0] } else { [0.1, 0.1, 0.12, 0.85] };
                line(mesh.vertices[a as usize], mesh.vertices[b as usize], color);
            }

            if let Some(sel) = sel {
                for f in &sel.faces {
                    if let Some(face) = mesh.faces.get(*f) {
                        let c = mesh.face_center(*f);
                        let n = mesh.face_normal(*f);
                        line(c, c + n * 12.0, [0.3, 0.8, 1.0, 0.9]);
                        for v in &face.indices {
                            line(mesh.vertices[*v as usize], c, [1.0, 0.55, 0.1, 0.25]);
                        }
                    }
                }
            }
        }

        match &self.modal {
            Some(Modal::LoopCut { hover: Some((id, edge)), cuts }) => {
                if let Some(mesh) = state.doc.map.mesh(*id) {
                    let ring = mesh.edge_ring(*edge);
                    for c in 1..=*cuts {
                        let t = c as f64 / (*cuts + 1) as f64;
                        let pts: Vec<DVec3> = ring.iter().map(|(a, b)| mesh.vertices[*a as usize].lerp(mesh.vertices[*b as usize], t)).collect();
                        for w in pts.windows(2) {
                            line(w[0], w[1], [1.0, 0.9, 0.2, 1.0]);
                        }

                        if ring.len() > 2
                            && let (Some(first), Some(last)) = (pts.first(), pts.last())
                        {
                            let closes = mesh.faces.iter().any(|f| {
                                f.indices.len() == 4 && {
                                    let (fa, fb) = (ring[0], ring[ring.len() - 1]);
                                    [fa.0, fa.1, fb.0, fb.1].iter().all(|v| f.indices.contains(v))
                                }
                            });
                            if closes {
                                line(*last, *first, [1.0, 0.9, 0.2, 1.0]);
                            }
                        }
                    }
                }
            }
            Some(Modal::Transform { constraint: Some(c), center, .. }) if !c.exclude => {
                let len = 100_000.0;
                let color = match c.label {
                    "X" => [1.0, 0.3, 0.3, 0.9],
                    "Y" => [0.3, 1.0, 0.3, 0.9],
                    "Z" => [0.4, 0.5, 1.0, 0.9],
                    _ => [0.3, 0.9, 1.0, 0.9],
                };
                line(*center - c.dir * len, *center + c.dir * len, color);
            }
            _ => {}
        }

        let _ = (cam, rect);
        out
    }

    pub fn paint_overlay(&self, ui: &Ui, cam: &Camera, rect: Rect, state: &EditorState) {
        let painter = ui.painter_at(rect);
        let meshes = edit_meshes(state);
        if self.component == Component::Vertex {
            for id in &meshes {
                let Some(mesh) = state.doc.map.mesh(*id) else { continue };
                let sel = self.selection.get(id);
                // Very dense meshes would flood the painter, only the selection is drawn then.
                let dense = mesh.vertices.len() > 20_000;
                for (i, p) in project_all(cam, rect, mesh).into_iter().enumerate() {
                    let Some(p) = p.filter(|p| rect.contains(*p)) else { continue };
                    let selected = sel.is_some_and(|s| s.verts.contains(&(i as u32)));
                    if dense && !selected {
                        continue;
                    }

                    let (size, color) = if selected { (6.0, Color32::from_rgb(255, 150, 30)) } else { (4.0, Color32::from_rgb(20, 20, 24)) };
                    painter.rect_filled(Rect::from_center_size(p, Vec2::splat(size)), 0.0, color);
                }
            }
        }

        if self.component == Component::Face {
            for id in &meshes {
                let Some(mesh) = state.doc.map.mesh(*id) else { continue };
                let sel = self.selection.get(id);
                for f in 0..mesh.faces.len().min(20_000) {
                    if let Some(p) = cam.project(rect, mesh.face_center(f)).filter(|p| rect.contains(*p)) {
                        let selected = sel.is_some_and(|s| s.faces.contains(&f));
                        painter.circle_filled(
                            p,
                            if selected { 3.5 } else { 2.0 },
                            if selected { Color32::from_rgb(255, 150, 30) } else { Color32::from_gray(30) },
                        );
                    }
                }
            }
        }

        if let Some(bounds) = self.gizmo_bounds(state) {
            let active = self.gizmo_drag.as_ref().filter(|(_, view)| *view == cam.kind).map(|(d, _)| d.part);
            crate::transform_gizmo::paint_on(ui, cam, rect, &bounds, active);
        }

        if let (Some(start), Some(end)) = (self.box_start, ui.input(|i| i.pointer.hover_pos())) {
            let r = Rect::from_two_pos(start, end);
            painter.rect_filled(r, 0.0, Color32::from_rgba_unmultiplied(255, 160, 60, 20));
            painter.rect_stroke(r, 0.0, Stroke::new(1.0, Color32::from_rgb(255, 160, 60)), egui::StrokeKind::Inside);
        }

        if let Some(Modal::Knife { first: Some(a), view }) = &self.modal
            && *view == Some(cam.kind)
            && let Some(b) = ui.input(|i| i.pointer.hover_pos())
        {
            painter.line_segment([*a, b], Stroke::new(2.0, Color32::from_rgb(120, 255, 120)));
        }

        let hint = match &self.modal {
            Some(Modal::Transform { .. }) => "Transform: X/Y/Z axis, Shift+X/Y/Z plane, type a value, click or Enter confirms, Esc cancels".to_string(),
            Some(Modal::Inset { .. }) => "Inset: move the mouse away from the selection, click to confirm".to_string(),
            Some(Modal::Bevel { .. }) => "Bevel: move the mouse to set the width, click to confirm".to_string(),
            Some(Modal::LoopCut { cuts, .. }) => format!("Loop cut: hover an edge, wheel or +/- changes cuts ({cuts}), click to cut"),
            Some(Modal::Knife { .. }) => "Knife: click the start and end of the cut".to_string(),
            None if meshes.is_empty() => "Mesh edit: select a mesh (or brushes and press Tab to convert them)".to_string(),
            None => format!(
                "Mesh edit ({}): 1/2/3 mode, click/box select, double click for a gizmo, Alt+click loop, G/R/S transform, E extrude, I inset, Ctrl+B bevel, Ctrl+R loop cut, K knife, M merge, F fill, X delete, Shift+D duplicate, P separate",
                self.component.label()
            ),
        };
        painter.text(rect.left_bottom() + Vec2::new(8.0, -8.0), Align2::LEFT_BOTTOM, hint, FontId::proportional(12.0), Color32::from_rgb(255, 200, 120));
    }
}

#[derive(Clone, Copy, Debug)]
enum Picked {
    Vertex(u32),
    Edge((u32, u32)),
    Face(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshOp {
    MergeCenter,
    MergeByDistance,
    Fill,
    Delete,
    Dissolve,
    Subdivide,
    Triangulate,
    Flip,
    Smooth,
    Solidify,
    Duplicate,
    Separate,
    SelectLinked,
    ShadeSmooth,
    ShadeFlat,
    Mirror(usize),
    SnapToGrid,
}

impl MeshOp {
    pub fn label(&self) -> &'static str {
        match self {
            MeshOp::MergeCenter => "Merge at Center",
            MeshOp::MergeByDistance => "Merge by Distance",
            MeshOp::Fill => "Fill",
            MeshOp::Delete => "Delete",
            MeshOp::Dissolve => "Dissolve Vertices",
            MeshOp::Subdivide => "Subdivide",
            MeshOp::Triangulate => "Triangulate",
            MeshOp::Flip => "Flip Normals",
            MeshOp::Smooth => "Smooth Vertices",
            MeshOp::Solidify => "Solidify",
            MeshOp::Duplicate => "Duplicate",
            MeshOp::Separate => "Separate",
            MeshOp::SelectLinked => "Select Linked",
            MeshOp::ShadeSmooth => "Shade Smooth",
            MeshOp::ShadeFlat => "Shade Flat",
            MeshOp::Mirror(0) => "Mirror X",
            MeshOp::Mirror(1) => "Mirror Y",
            MeshOp::Mirror(_) => "Mirror Z",
            MeshOp::SnapToGrid => "Snap Vertices to Grid",
        }
    }

    pub const ALL: [MeshOp; 19] = [
        MeshOp::MergeCenter,
        MeshOp::MergeByDistance,
        MeshOp::Fill,
        MeshOp::Delete,
        MeshOp::Dissolve,
        MeshOp::Subdivide,
        MeshOp::Triangulate,
        MeshOp::Flip,
        MeshOp::Smooth,
        MeshOp::Solidify,
        MeshOp::Duplicate,
        MeshOp::Separate,
        MeshOp::SelectLinked,
        MeshOp::ShadeSmooth,
        MeshOp::ShadeFlat,
        MeshOp::Mirror(0),
        MeshOp::Mirror(1),
        MeshOp::Mirror(2),
        MeshOp::SnapToGrid,
    ];
}

/// Matrix for a modal transform from the mouse movement since `start`.
#[allow(clippy::too_many_arguments)]
fn transform_matrix(
    kind: TransformKind,
    constraint: Option<Constraint>,
    start: Pos2,
    pos: Pos2,
    center: DVec3,
    numeric: &str,
    cam: &Camera,
    rect: Rect,
    snap: bool,
    grid: f64,
) -> DMat4 {
    let value: Option<f64> = numeric.parse().ok();
    match kind {
        TransformKind::Grab => {
            let delta = if let Some(v) = value {
                constraint.map(|c| c.dir * v).unwrap_or(DVec3::X * v)
            } else {
                let view_plane = Plane::from_point_normal(center, cam.forward());
                let world = |p: Pos2| {
                    let ray = cam.ray(rect, p);
                    ray.intersect_plane(&view_plane).map(|t| ray.at(t)).unwrap_or(center)
                };
                let raw = match constraint {
                    Some(c) if !c.exclude => {
                        let (r0, r1) = (cam.ray(rect, start), cam.ray(rect, pos));
                        match (crate::tools::line_ray_param(center, c.dir, &r0), crate::tools::line_ray_param(center, c.dir, &r1)) {
                            (Some(a), Some(b)) => c.dir * (b - a),
                            _ => {
                                let d = world(pos) - world(start);
                                c.dir * d.dot(c.dir)
                            }
                        }
                    }
                    Some(c) => {
                        let plane = Plane::from_point_normal(center, c.dir);
                        let hit = |p: Pos2| {
                            let ray = cam.ray(rect, p);
                            ray.intersect_plane(&plane).map(|t| ray.at(t))
                        };
                        match (hit(start), hit(pos)) {
                            (Some(a), Some(b)) => b - a,
                            _ => DVec3::ZERO,
                        }
                    }
                    None => world(pos) - world(start),
                };
                if snap {
                    match constraint {
                        Some(c) if !c.exclude => c.dir * gt_core::snap_to_grid(raw.dot(c.dir), grid),
                        _ => gt_core::snap_vec_to_grid(raw, grid),
                    }
                } else {
                    raw
                }
            };
            DMat4::from_translation(delta)
        }
        TransformKind::Rotate => {
            let axis = constraint.map(|c| c.dir).unwrap_or(-cam.forward());
            let degrees = if let Some(v) = value {
                v
            } else {
                let c = cam.project(rect, center).unwrap_or(rect.center());
                let a0 = (start - c).angle();
                let a1 = (pos - c).angle();
                // Screen y points down, so a positive screen angle is clockwise.
                let mut d = -((a1 - a0) as f64).to_degrees();
                if axis.dot(-cam.forward()) < 0.0 {
                    d = -d;
                }

                if snap { (d / 15.0).round() * 15.0 } else { d }
            };
            DMat4::from_translation(center)
                * DMat4::from_quat(DQuat::from_axis_angle(axis.normalize(), degrees.to_radians()))
                * DMat4::from_translation(-center)
        }
        TransformKind::Scale => {
            let factor = if let Some(v) = value {
                v
            } else {
                let c = cam.project(rect, center).unwrap_or(rect.center());
                let d0 = (start - c).length().max(1.0) as f64;
                let d1 = (pos - c).length() as f64;
                let f = d1 / d0;
                if snap { (f * 10.0).round() / 10.0 } else { f }
            };
            if let Some(c) = constraint.filter(|c| c.label.len() != 1) {
                // Scaling along an arbitrary direction: rotate it onto X, scale, rotate back.
                let q = DQuat::from_rotation_arc(c.dir.normalize(), DVec3::X);
                let s = if c.exclude { DVec3::new(1.0, factor, factor) } else { DVec3::new(factor, 1.0, 1.0) };
                return DMat4::from_translation(center)
                    * DMat4::from_quat(q.inverse())
                    * DMat4::from_scale(s)
                    * DMat4::from_quat(q)
                    * DMat4::from_translation(-center);
            }

            let scale = match constraint {
                Some(c) if !c.exclude => DVec3::ONE + c.dir.abs() * (factor - 1.0),
                Some(c) => DVec3::splat(factor) - c.dir.abs() * (factor - 1.0),
                None => DVec3::splat(factor),
            };
            DMat4::from_translation(center) * DMat4::from_scale(scale) * DMat4::from_translation(-center)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_numeric_transforms() {
        let cam = Camera::new(ViewKind::Perspective);
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
        let y = Some(Constraint { dir: DVec3::Y, exclude: false, label: "Y" });
        let m = transform_matrix(TransformKind::Grab, y, Pos2::new(1.0, 1.0), Pos2::new(5.0, 5.0), DVec3::ZERO, "32", &cam, rect, true, 16.0);
        assert!((m.transform_point3(DVec3::ZERO) - DVec3::new(0.0, 32.0, 0.0)).length() < 1e-9);
        let r = transform_matrix(TransformKind::Rotate, y, Pos2::ZERO, Pos2::ZERO, DVec3::ZERO, "90", &cam, rect, false, 16.0);
        assert!((r.transform_point3(DVec3::X) - DVec3::new(0.0, 0.0, -1.0)).length() < 1e-9);
        let x = Some(Constraint { dir: DVec3::X, exclude: false, label: "X" });
        let s = transform_matrix(TransformKind::Scale, x, Pos2::ZERO, Pos2::ZERO, DVec3::ZERO, "2", &cam, rect, false, 16.0);
        assert!((s.transform_point3(DVec3::ONE) - DVec3::new(2.0, 1.0, 1.0)).length() < 1e-9);
    }

    #[test]
    fn mirroring_part_of_a_mesh_keeps_its_faces_outward() {
        let a = gt_core::Aabb::new(DVec3::ZERO, DVec3::new(32.0, 16.0, 16.0));
        let mut mesh = gt_geom::mesh_shapes::cuboid(&a, "m");
        let other = gt_geom::mesh_shapes::cuboid(&gt_core::Aabb::new(DVec3::splat(100.0), DVec3::splat(116.0)), "m");
        let offset = mesh.vertices.len() as u32;
        let first: Vec<usize> = (0..mesh.faces.len()).collect();
        mesh.vertices.extend(other.vertices.iter().copied());
        mesh.faces.extend(other.faces.iter().cloned().map(|mut f| {
            f.indices.iter_mut().for_each(|i| *i += offset);
            f
        }));

        let mut state = EditorState::new(Default::default());
        let layer = state.doc.map.default_layer();
        let id = state.doc.edit("mesh", |m, s| {
            let id = m.insert(layer, gt_doc::NodeKind::Mesh(mesh));
            s.select_node(id);
            id
        });
        let mut tool = MeshTool::default();
        tool.selection.insert(id, MeshSel { verts: (0..offset).collect(), ..Default::default() });
        tool.run(&mut state, MeshOp::Mirror(0));
        let m = state.doc.map.mesh(id).unwrap();
        for f in first {
            let out = m.face_center(f) - a.center();
            assert!(m.face_normal(f).dot(out) > 0.0, "face {f} points into the mirrored box");
        }
    }

    struct GizmoFixture {
        state: EditorState,
        tool: MeshTool,
        cam: Camera,
        rect: Rect,
    }

    fn step_pointer(h: &mut egui_kittest::Harness<GizmoFixture>, pos: Pos2, pressed: Option<bool>) {
        h.event(egui::Event::PointerMoved(pos));
        if let Some(pressed) = pressed {
            h.event(egui::Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers: Modifiers::NONE });
        }

        h.step();
    }

    #[test]
    fn double_clicking_a_vertex_shows_the_transform_gizmo_in_a_2d_view() {
        let mut state = EditorState::new(Default::default());
        let layer = state.doc.map.default_layer();
        let cube = gt_geom::mesh_shapes::cuboid(&gt_core::Aabb::new(DVec3::ZERO, DVec3::splat(64.0)), "m");
        let id = state.doc.edit("mesh", |m, s| {
            let id = m.insert(layer, gt_doc::NodeKind::Mesh(cube));
            s.select_node(id);
            id
        });
        let mut cam = Camera::new(ViewKind::Front);
        cam.center = DVec3::new(32.0, 32.0, 0.0);
        cam.zoom = 2.0;
        let fixture = GizmoFixture { state, tool: MeshTool::default(), cam, rect: Rect::NOTHING };
        let mut h = egui_kittest::Harness::builder().with_size(Vec2::new(800.0, 600.0)).with_step_dt(1.0 / 60.0).build_ui_state(
            |ui, f: &mut GizmoFixture| {
                let rect = ui.available_rect_before_wrap();
                let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                f.rect = rect;
                f.tool.viewport_input(ui, &response, &f.cam, rect, response.hover_pos(), &mut f.state);
                f.tool.paint_overlay(ui, &f.cam, rect, &f.state);
            },
            fixture,
        );
        h.run();

        // The top right corner, where a front and a back vertex overlap in this view.
        let at = h.state().cam.project(h.state().rect, DVec3::new(64.0, 64.0, 64.0)).unwrap();
        for _ in 0..2 {
            step_pointer(&mut h, at, Some(true));
            step_pointer(&mut h, at, Some(false));
        }

        h.run();
        let f = h.state();
        assert!(f.tool.gizmo, "a double click shows the gizmo");
        assert_eq!(f.tool.selection[&id].verts.len(), 1, "on just that vertex");
        let v = *f.tool.selection[&id].verts.first().unwrap();
        let start = f.state.doc.map.mesh(id).unwrap().vertices[v as usize];
        assert_eq!((start.x, start.y), (64.0, 64.0));
        let bounds = f.tool.gizmo_bounds(&f.state).unwrap();
        assert_eq!(bounds.center(), start);

        // The Z arrow points at the camera in the front view, so only X and Y offer handles.
        let hit = |p: Pos2| crate::transform_gizmo::hit_on(&f.cam, f.rect, &bounds, p);
        let x_arrow = (10..200).map(|d| at + Vec2::new(d as f32, 0.0)).find(|p| hit(*p) == Some(crate::transform_gizmo::Part::Move(0))).unwrap();
        assert!((0..200).all(|d| hit(at + Vec2::new(-(d as f32), 0.0)) != Some(crate::transform_gizmo::Part::Move(2))));

        let undo = f.state.doc.history.undo_labels().count();
        step_pointer(&mut h, x_arrow, Some(true));
        for i in 1..=6 {
            step_pointer(&mut h, x_arrow + Vec2::new(10.0 * i as f32, -8.0 * i as f32), None);
        }

        step_pointer(&mut h, x_arrow + Vec2::new(60.0, -48.0), Some(false));
        h.run();
        let f = h.state();
        let moved = f.state.doc.map.mesh(id).unwrap().vertices[v as usize];
        assert!(moved.x > 64.0, "moved along X: {moved:?}");
        assert_eq!((moved.y, moved.z), (start.y, start.z), "and only along X");
        assert_eq!(f.state.doc.history.undo_labels().count(), undo + 1, "one undo step");
        assert_eq!(f.state.doc.history.undo_labels().next(), Some("Move Vertices"));
        assert!(f.tool.gizmo, "the gizmo stays for the next drag");

        // Escape hides the gizmo and keeps the selection.
        let ctx = egui::Context::default();
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Key { key: Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE });
        let f = h.state_mut();
        ctx.run_ui(input, |ui| assert!(f.tool.keys(ui.ctx(), &mut f.state))).textures_delta.clear();
        assert!(!f.tool.gizmo);
        assert_eq!(f.tool.selection[&id].verts.len(), 1);
    }
}
