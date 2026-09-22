//! Blend tool: paints material blends on terrain layers, displacement alpha and faces that have a blend material.

use egui::{Align2, Color32, FontId, PointerButton, Pos2, Rect, Response, Ui, Vec2};
use gt_core::{DVec3, NodeId, Plane};
use gt_doc::blend::{self, BlendMode};
use gt_render::LineVertex;

use crate::camera::Camera;
use crate::picking;
use crate::scene::v3;
use crate::state::EditorState;

/// What one dab edits: terrains, displacement faces and blend faces, from the selection or the whole map.
pub struct BlendTargets {
    pub terrains: Vec<NodeId>,
    pub displacements: Vec<(NodeId, usize)>,
    pub faces: Vec<(NodeId, usize)>,
}

impl BlendTargets {
    pub fn is_empty(&self) -> bool {
        self.terrains.is_empty() && self.displacements.is_empty() && self.faces.is_empty()
    }
}

/// Brushes and terrains a brush tool (sculpt, paint, blend) works on: the selected ones, or every one when nothing
/// is selected. Locked and hidden ones are always left alone.
pub struct StrokeScope {
    pub brushes: Vec<NodeId>,
    pub terrains: Vec<NodeId>,
}

impl StrokeScope {
    pub fn of(state: &EditorState) -> Self {
        let map = &state.doc.map;
        let sel = &state.doc.selection;
        let (brushes, terrains): (Vec<NodeId>, Vec<NodeId>) = if sel.is_empty() {
            (map.brushes().map(|(id, _)| id).collect(), map.terrains().map(|(id, _)| id).collect())
        } else {
            let mut brushes = sel.brushes(map);
            brushes.extend(sel.faces.iter().map(|(id, _)| *id).filter(|id| map.brush(*id).is_some()));
            brushes.sort();
            brushes.dedup();
            (brushes, sel.terrains(map))
        };
        Self {
            brushes: brushes.into_iter().filter(|id| map.is_editable(*id)).collect(),
            terrains: terrains.into_iter().filter(|id| map.is_editable(*id)).collect(),
        }
    }

    /// Displacement faces of the scope's brushes. `displacement_faces` reads an empty list as the whole map.
    pub fn displacements(&self, state: &EditorState) -> Vec<(NodeId, usize)> {
        if self.brushes.is_empty() { Vec::new() } else { gt_doc::terrain::displacement_faces(&state.doc.map, &self.brushes) }
    }
}

pub fn targets(state: &EditorState) -> BlendTargets {
    let map = &state.doc.map;
    let sel = &state.doc.selection;
    let scope = StrokeScope::of(state);
    let displacements = scope.displacements(state);
    let faces = if sel.has_faces() {
        sel.faces.iter().copied().filter(|(id, f)| map.is_editable(*id) && blend::blend_faces(map, &[*id]).contains(&(*id, *f))).collect()
    } else {
        let geometry: Vec<NodeId> = sel.geometry(map).into_iter().filter(|id| map.is_editable(*id)).collect();
        // `blend_faces` reads an empty list as the whole map, which is only meant when nothing is selected.
        if geometry.is_empty() && !sel.is_empty() {
            Vec::new()
        } else {
            blend::blend_faces(map, &geometry).into_iter().filter(|(id, _)| map.is_editable(*id)).collect()
        }
    };
    BlendTargets { terrains: scope.terrains, displacements, faces }
}

/// One dab. Returns true when anything changed.
pub fn dab(state: &mut EditorState, center: DVec3, brush: &blend::BlendBrush) -> bool {
    let t = targets(state);
    state.doc.edit("Blend", |m, _| {
        let mut changed = false;
        for id in &t.terrains {
            if let Some(terrain) = m.terrain_mut(*id) {
                changed |= blend::blend_terrain(terrain, center, brush);
            }
        }

        changed |= blend::blend_displacements(m, &t.displacements, center, brush);
        changed |= blend::blend_face_corners(m, &t.faces, center, brush);
        changed
    })
}

/// The selected faces, or every face of the selected geometry when no face is picked.
fn blend_target_faces(state: &EditorState) -> Vec<(NodeId, usize)> {
    let map = &state.doc.map;
    if state.doc.selection.has_faces() {
        return state.doc.selection.faces.iter().copied().collect();
    }

    state
        .doc
        .selection
        .geometry(map)
        .into_iter()
        .flat_map(|id| {
            let n = map.brush(id).map(|b| b.faces.len()).or_else(|| map.mesh(id).map(|m| m.faces.len())).unwrap_or(0);
            (0..n).map(move |f| (id, f))
        })
        .collect()
}

/// Sets the blend material on the selected faces, or on every face of the selected brushes and meshes.
pub fn set_blend_material(state: &mut EditorState, material: Option<&str>) -> usize {
    let faces = blend_target_faces(state);
    let material = material.map(str::to_string);
    state.doc.edit("Blend Material", |m, _| blend::set_blend_material(m, &faces, material.as_deref()))
}

/// Sets the blend material's repeat and de-tiling on the same faces `set_blend_material` would touch.
pub fn set_blend_tiling(state: &mut EditorState, detile: f64, uv_scale: f64, sharpen: f64) -> usize {
    let faces = blend_target_faces(state);
    state.doc.edit("Blend Tiling", |m, _| blend::set_blend_options(m, &faces, detile, uv_scale, sharpen))
}

#[derive(Default)]
pub struct BlendTool {
    pub hover: Option<(DVec3, DVec3)>,
    stroking: bool,
    last: Option<DVec3>,
    dabs: u32,
}

impl BlendTool {
    pub fn stroking(&self) -> bool {
        self.stroking
    }

    /// Ends a stroke without touching the document, the caller owns its transaction.
    pub fn reset(&mut self) {
        self.hover = None;
        self.last = None;
        self.stroking = false;
    }

    pub fn input(&mut self, ui: &Ui, response: &Response, cam: &Camera, rect: Rect, hover: Option<Pos2>, state: &mut EditorState) {
        let pointer = if self.stroking { ui.input(|i| i.pointer.interact_pos()) } else { hover };
        self.hover = pointer
            .and_then(|p| {
                picking::pick_all(state, &cam.ray(rect, p))
                    .into_iter()
                    .find(|h| state.doc.map.entity(h.node).is_none() && state.doc.map.scatter(h.node).is_none())
            })
            .map(|h| (h.point, h.normal));
        let modifiers = ui.input(|i| i.modifiers);
        if (response.drag_started_by(PointerButton::Primary) || response.clicked_by(PointerButton::Primary)) && !self.stroking && self.hover.is_some() {
            if targets(state).is_empty() {
                state
                    .set_status("Nothing to blend: add terrain layers, displacements, or set a blend material on faces (Terrain > Blend > Set Blend Material)");
                return;
            }

            let layer = state.blend.layer;
            let short = targets(state).terrains.iter().filter_map(|id| state.doc.map.terrain(*id)).map(|t| t.layer_slot(layer)).find(|slot| *slot != layer);
            if let Some(slot) = short.filter(|_| !matches!(state.blend.mode, BlendMode::Erase | BlendMode::Smooth | BlendMode::Sharpen)) {
                state.set_status(format!("The terrain has no layer {layer}, painting its last layer, {slot}. Add a layer in the Inspector first"));
            }

            self.stroking = true;
            self.last = None;
            state.doc.begin("Blend");
        }

        if self.stroking {
            if let Some((p, _)) = self.hover {
                let spacing = (state.blend.radius * 0.2).max(1.0);
                if self.last.is_none_or(|l| (l - p).length() >= spacing) {
                    let mut brush = state.blend;
                    if modifiers.shift {
                        brush.mode = match brush.mode {
                            BlendMode::Paint | BlendMode::Noise | BlendMode::Slope | BlendMode::Height => BlendMode::Erase,
                            BlendMode::Erase => BlendMode::Paint,
                            BlendMode::Sharpen => BlendMode::Smooth,
                            BlendMode::Smooth => BlendMode::Sharpen,
                        };
                    }

                    if modifiers.command {
                        brush.mode = BlendMode::Smooth;
                    }

                    self.dabs = self.dabs.wrapping_add(1);
                    brush.seed = state.blend.seed.wrapping_add(if brush.falloff == blend::Falloff::Spray { self.dabs } else { 0 });
                    dab(state, p, &brush);
                    self.last = Some(p);
                }
            }

            if !ui.input(|i| i.pointer.primary_down()) {
                self.stroking = false;
                state.doc.commit();
            }
        }
    }

    pub fn lines(&self, state: &EditorState, out: &mut Vec<LineVertex>) {
        let Some((p, n)) = self.hover else { return };
        let r = state.blend.radius;
        let basis = Plane::from_point_normal(p, n).basis();
        let color = [0.95, 0.6, 1.0, 0.95];
        let ring = |scale: f64| -> Vec<DVec3> {
            (0..=48)
                .map(|i| {
                    let a = std::f64::consts::TAU * i as f64 / 48.0;
                    p + (basis.0 * a.cos() + basis.1 * a.sin()) * r * scale + n * 0.5
                })
                .collect()
        };
        for w in ring(1.0).windows(2) {
            out.push(LineVertex { pos: v3(w[0]), color });
            out.push(LineVertex { pos: v3(w[1]), color });
        }

        let inner = [color[0], color[1], color[2], 0.4];
        for w in ring(state.blend.strength.clamp(0.05, 1.0)).windows(2) {
            out.push(LineVertex { pos: v3(w[0]), color: inner });
            out.push(LineVertex { pos: v3(w[1]), color: inner });
        }
    }

    pub fn paint_overlay(&self, ui: &Ui, rect: Rect, state: &EditorState) {
        let b = &state.blend;
        let text = format!(
            "Blend {} layer {} ({} falloff): drag paints, Shift inverts, Ctrl smooths. Terrains, displacements and faces with a blend material",
            b.mode.label(),
            b.layer,
            b.falloff.label()
        );
        ui.painter_at(rect).text(
            rect.left_bottom() + Vec2::new(8.0, -8.0),
            Align2::LEFT_BOTTOM,
            text,
            FontId::proportional(12.0),
            Color32::from_rgb(240, 170, 255),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_core::Aabb;
    use gt_doc::NodeKind;
    use gt_geom::{Brush, Terrain};

    /// A displacement brush beside a terrain, both editable.
    fn map_with_both() -> (EditorState, NodeId, NodeId) {
        let mut state = EditorState::new(Default::default());
        let layer = state.doc.map.default_layer();
        let mut brush = Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::new(128.0, 16.0, 128.0)), "dev/grey").unwrap();
        let top = brush.faces.iter().position(|f| f.plane.normal.y > 0.9).unwrap();
        brush.faces[top].data.disp = Some(gt_geom::displacement::Displacement::new(2));
        let terrain = Terrain::new(DVec3::new(128.0, 0.0, 0.0), [5, 5], 32.0, "dev/grey");
        let (disp, ter) = state.doc.edit("setup", |m, _| (m.insert(layer, NodeKind::Brush(brush)), m.insert(layer, NodeKind::Terrain(terrain))));
        (state, disp, ter)
    }

    #[test]
    fn a_selection_limits_the_stroke_to_what_is_selected() {
        let (mut state, disp, ter) = map_with_both();
        let all = targets(&state);
        assert_eq!(all.terrains, vec![ter], "nothing selected reaches everything");
        assert_eq!(all.displacements.len(), 1);

        state.doc.select(|_, s| s.select_node(disp));
        let t = targets(&state);
        assert!(t.terrains.is_empty(), "a selected displacement leaves the terrain alone");
        assert_eq!(t.displacements, vec![(disp, all.displacements[0].1)]);

        state.doc.select(|_, s| {
            s.clear();
            s.select_node(ter);
        });
        let t = targets(&state);
        assert_eq!(t.terrains, vec![ter]);
        assert!(t.displacements.is_empty(), "a selected terrain leaves the displacement alone");

        state.doc.select(|_, s| s.clear());
        state.doc.map.get_mut(ter).unwrap().locked = true;
        assert!(targets(&state).terrains.is_empty(), "a locked terrain is never painted");
    }
}
