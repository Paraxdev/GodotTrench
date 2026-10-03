//! Trim: strips of brush laid along the edges of faces, standing out from them. Along the bottom edges of walls they
//! are baseboards, along the top edges crown molding, around a floor a border, around a pool its coping. It works on
//! the selected faces, or on the upright faces of the selected brushes.
//!
//! Where two trimmed faces meet, only one strip reaches into the corner: at an outer corner it runs on past the corner
//! by its depth, at an inner corner the other one stops short by it. So the strips meet in one piece without two of
//! them overlapping, which would make their faces fight for the same pixels.

use gt_core::{DVec3, NodeId};
use gt_doc::{Map, NodeKind, Selection};
use gt_geom::Brush;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrimEdges {
    Bottom,
    Top,
    Sides,
    All,
}

impl TrimEdges {
    pub const ALL: [TrimEdges; 4] = [TrimEdges::Bottom, TrimEdges::Top, TrimEdges::Sides, TrimEdges::All];

    pub fn name(self) -> &'static str {
        match self {
            TrimEdges::Bottom => "bottom",
            TrimEdges::Top => "top",
            TrimEdges::Sides => "sides",
            TrimEdges::All => "all",
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.name() == name)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrimOptions {
    pub edges: TrimEdges,
    /// How far the strip runs over the face from its edge, map units.
    pub height: f64,
    /// How far it stands out from the face.
    pub depth: f64,
    /// Gap between the edge and the strip.
    pub inset: f64,
    pub material: String,
    /// Bottom edges only where a floor lies under them, so the underside of a lintel gets no baseboard.
    pub on_floor: bool,
    /// Keeps collision. Off wraps the strips in func_detail_illusionary where the game has it.
    pub collision: bool,
}

impl Default for TrimOptions {
    fn default() -> Self {
        Self { edges: TrimEdges::Bottom, height: 6.0, depth: 2.0, inset: 0.0, material: "dev/dark".into(), on_floor: true, collision: false }
    }
}

/// The faces a trim applies to: the selected faces, or else the upright faces of the selected brushes.
pub fn faces_for(map: &Map, selection: &Selection) -> Vec<(NodeId, usize)> {
    if !selection.faces.is_empty() {
        return selection.faces.iter().copied().collect();
    }

    let mut out = Vec::new();
    for id in selection.brushes(map) {
        if let Some(NodeKind::Brush(b)) = map.get(id).map(|n| &n.kind) {
            out.extend(b.faces.iter().enumerate().filter(|(_, f)| f.plane.normal.y.abs() < 0.1).map(|(i, _)| (id, i)));
        }
    }

    out
}

/// One face edge a strip is laid along.
struct Edge {
    a: DVec3,
    b: DVec3,
    /// Outward normal of the face.
    normal: DVec3,
    /// Along the face, away from the edge.
    inward: DVec3,
}

fn edges(map: &Map, faces: &[(NodeId, usize)], opts: &TrimOptions) -> Vec<Edge> {
    let brushes: Vec<(NodeId, &Brush)> = map
        .walk()
        .into_iter()
        .filter_map(|id| match map.get(id).map(|n| &n.kind) {
            Some(NodeKind::Brush(b)) => Some((id, b)),
            _ => None,
        })
        .collect();
    let inside_other = |own: NodeId, p: DVec3| brushes.iter().any(|(id, b)| *id != own && b.bounds().contains_point(p) && b.contains_point(p));
    let mut out = Vec::new();
    for &(id, fi) in faces {
        let Some(NodeKind::Brush(b)) = map.get(id).map(|n| &n.kind) else { continue };
        let Some(face) = b.faces.get(fi) else { continue };
        let n = face.plane.normal;
        let pts = b.face_points(fi);
        for k in 0..pts.len() {
            let (a, c) = (pts[k], pts[(k + 1) % pts.len()]);
            let along = c - a;
            if along.length() < 0.5 {
                continue;
            }

            let inward = n.cross(along).normalize();
            let wanted = match opts.edges {
                TrimEdges::All => true,
                TrimEdges::Bottom => inward.y > 0.7,
                TrimEdges::Top => inward.y < -0.7,
                TrimEdges::Sides => inward.y.abs() <= 0.7,
            };
            if !wanted {
                continue;
            }

            if opts.on_floor && opts.edges == TrimEdges::Bottom && !inside_other(id, (a + c) * 0.5 + n * 1.0 - DVec3::Y) {
                continue;
            }

            // A strip that would sit inside another brush is on a buried face, like the end of a lintel between two
            // wall pieces. Left out before the corners are worked out, so it does not count as a corner either.
            if inside_other(id, (a + c) * 0.5 + inward * (opts.inset + opts.height * 0.5) + n * (opts.depth * 0.5)) {
                continue;
            }

            out.push(Edge { a, b: c, normal: n, inward });
        }
    }

    out
}

/// How far the strip along edge `i` moves its end at `end`: past the corner at an outer one, short of it at an inner
/// one or where it runs into another trimmed face, 0 on a free end. Of two strips meeting at a corner, only the one
/// with the lower index moves.
fn end_shift(all: &[Edge], i: usize, end: DVec3, from: DVec3, depth: f64) -> f64 {
    let toward = (end - from).normalize();
    for (j, other) in all.iter().enumerate() {
        if j == i || other.normal.dot(all[i].normal) > 0.999 {
            continue;
        }

        let seg = other.b - other.a;
        let t = ((end - other.a).dot(seg) / seg.length_squared()).clamp(0.0, 1.0);
        if (other.a + seg * t).distance(end) > 0.5 {
            continue;
        }

        let at_corner = t < 1e-3 || t > 1.0 - 1e-3;
        let outer = other.normal.dot(toward) > 1e-3;
        return match (at_corner, outer) {
            (true, true) if i < j => depth,
            (true, false) if i < j => -depth,
            (true, _) => 0.0,
            // Runs into the middle of another trimmed face, whose strip carries on in front of it.
            (false, _) => -depth,
        };
    }

    0.0
}

/// The strips for `faces`, one brush per edge.
pub fn build(map: &Map, faces: &[(NodeId, usize)], opts: &TrimOptions) -> Vec<Brush> {
    let all = edges(map, faces, opts);
    let mut out = Vec::new();
    for (i, e) in all.iter().enumerate() {
        let dir = (e.b - e.a).normalize();
        let a = e.a - dir * end_shift(&all, i, e.a, e.b, opts.depth);
        let b = e.b + dir * end_shift(&all, i, e.b, e.a, opts.depth);
        if (b - a).dot(dir) < 0.5 {
            continue;
        }

        let (near, far) = (e.inward * opts.inset, e.inward * (opts.inset + opts.height));
        let out_by = e.normal * opts.depth;
        let points = [a + near, b + near, b + far, a + far, a + near + out_by, b + near + out_by, b + far + out_by, a + far + out_by];
        if let Ok(brush) = Brush::from_points(&points, &[], &opts.material) {
            out.push(brush);
        }
    }

    out
}

/// Adds the strips for `faces` in one undo step, grouped, or as func_detail_illusionary without collision, and
/// selects them. Returns the new node and how many strips it holds.
pub fn apply(state: &mut crate::state::EditorState, faces: &[(NodeId, usize)], opts: &TrimOptions) -> Result<(NodeId, usize), String> {
    if faces.is_empty() {
        return Err("Select faces, or brushes whose upright faces get the trim".into());
    }

    let strips = build(&state.doc.map, faces, opts);
    if strips.is_empty() {
        return Err(format!("No {} edges to trim on the selection", opts.edges.name()));
    }

    let count = strips.len();
    let detail = !opts.collision && state.game.entity("func_detail_illusionary").is_some();
    let parent = state.insert_parent();
    let id = state.doc.edit("Add Trim", |m, s| {
        let holder = if detail {
            m.insert(parent, NodeKind::Entity(gt_doc::Entity::new("func_detail_illusionary")))
        } else {
            m.insert(parent, NodeKind::Group(gt_doc::map::Group::new("Trim")))
        };
        for brush in strips {
            m.insert(holder, NodeKind::Brush(brush));
        }

        s.clear();
        s.select_node(holder);
        holder
    });
    state.set_status(format!("Added {count} trim strips"));
    Ok((id, count))
}

/// Brush > Add Trim: the options of the last trim, kept while the editor runs.
#[derive(Default)]
pub struct TrimDialog {
    pub open: bool,
    opts: Option<TrimOptions>,
}

impl TrimDialog {
    pub fn show(&mut self, ctx: &egui::Context, state: &mut crate::state::EditorState) {
        if !self.open {
            return;
        }

        let opts = self.opts.get_or_insert_with(|| TrimOptions { material: state.current_material.clone(), ..Default::default() });
        let mut open = self.open;
        let selected = faces_for(&state.doc.map, &state.doc.selection);
        let faces = selected.len();
        egui::Window::new("Add Trim").open(&mut open).resizable(false).show(ctx, |ui| {
            ui.label("Strips along face edges: baseboards on the bottom, crown molding on top, borders around floors.");
            egui::Grid::new("trim_options").num_columns(2).show(ui, |ui| {
                ui.label("Edges");
                ui.horizontal(|ui| {
                    for e in TrimEdges::ALL {
                        ui.selectable_value(&mut opts.edges, e, e.name());
                    }
                });
                ui.end_row();
                ui.label("Height");
                ui.add(egui::DragValue::new(&mut opts.height).range(0.25..=4096.0).suffix(" u")).on_hover_text("How far the strip runs over the face");
                ui.end_row();
                ui.label("Depth");
                ui.add(egui::DragValue::new(&mut opts.depth).range(0.25..=512.0).suffix(" u")).on_hover_text("How far it stands out from the face");
                ui.end_row();
                ui.label("Inset");
                ui.add(egui::DragValue::new(&mut opts.inset).range(0.0..=4096.0).suffix(" u")).on_hover_text("Gap between the edge and the strip");
                ui.end_row();
                ui.label("Material");
                ui.horizontal(|ui| {
                    ui.label(&opts.material);
                    if ui.small_button("Use Current").on_hover_text(format!("Use {}", state.current_material)).clicked() {
                        opts.material = state.current_material.clone();
                    }
                });
                ui.end_row();
            });
            if opts.edges == TrimEdges::Bottom {
                ui.checkbox(&mut opts.on_floor, "Only where a floor lies under the edge").on_hover_text("Leaves out the underside of lintels over doorways");
            }

            ui.checkbox(&mut opts.collision, "Collision").on_hover_text("Off wraps the strips in func_detail_illusionary, so players do not snag on them");
            ui.label(if faces == 0 { "Select faces, or brushes whose upright faces get the trim".to_string() } else { format!("{faces} faces selected") });
            if ui.add_enabled(faces > 0, egui::Button::new("Add Trim")).clicked()
                && let Err(e) = apply(state, &selected, opts)
            {
                state.set_status(e);
            }
        });
        self.open = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_core::Aabb;

    fn map_with(boxes: &[(DVec3, DVec3)]) -> (Map, Vec<NodeId>) {
        let mut m = Map::new();
        let layer = m.default_layer();
        let ids = boxes.iter().map(|(a, b)| m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(*a, *b), "wall").unwrap()))).collect();
        (m, ids)
    }

    fn select(ids: &[NodeId]) -> Selection {
        let mut s = Selection::default();
        s.nodes.extend(ids.iter().copied());
        s
    }

    fn total_volume(brushes: &[Brush]) -> f64 {
        brushes.iter().map(|b| b.volume()).sum()
    }

    #[test]
    fn a_free_standing_wall_gets_a_baseboard_wrapped_around_it() {
        // A floor and a 64 by 8 wall standing on it.
        let (m, ids) = map_with(&[(DVec3::new(-128.0, -8.0, -128.0), DVec3::new(128.0, 0.0, 128.0)), (DVec3::new(0.0, 0.0, 0.0), DVec3::new(64.0, 96.0, 8.0))]);
        let faces = faces_for(&m, &select(&ids[1..]));
        assert_eq!(faces.len(), 4, "the four upright faces");
        let opts = TrimOptions { depth: 2.0, height: 6.0, ..Default::default() };
        let strips = build(&m, &faces, &opts);
        assert_eq!(strips.len(), 4);
        // The strips cover a 68 by 12 ring around the 64 by 8 wall, 6 tall, with no overlaps: exactly its area.
        let ring = (68.0 * 12.0 - 64.0 * 8.0) * 6.0;
        assert!((total_volume(&strips) - ring).abs() < 1e-6, "{} against {ring}", total_volume(&strips));
        for s in &strips {
            assert!(s.bounds().min.y.abs() < 1e-9 && (s.bounds().max.y - 6.0).abs() < 1e-9, "sits on the floor");
        }
    }

    #[test]
    fn an_inner_corner_meets_without_overlap_and_a_lintel_gets_nothing() {
        // Two walls meeting in an L, seen from inside the L the corner is an inner one, and a lintel over a gap.
        let (m, ids) = map_with(&[
            (DVec3::new(-128.0, -8.0, -128.0), DVec3::new(256.0, 0.0, 256.0)),
            (DVec3::new(0.0, 0.0, 0.0), DVec3::new(128.0, 96.0, 8.0)),
            (DVec3::new(0.0, 0.0, 8.0), DVec3::new(8.0, 96.0, 128.0)),
            (DVec3::new(160.0, 64.0, 0.0), DVec3::new(224.0, 96.0, 8.0)),
        ]);
        let mut sel = Selection::default();
        let face = |id: NodeId, normal: DVec3| {
            let NodeKind::Brush(b) = &m.get(id).unwrap().kind else { unreachable!() };
            (id, b.faces.iter().position(|f| f.plane.normal.distance(normal) < 1e-6).unwrap())
        };
        sel.faces.extend([face(ids[1], DVec3::Z), face(ids[2], DVec3::X), face(ids[3], DVec3::Z)]);
        let faces = faces_for(&m, &sel);
        let strips = build(&m, &faces, &TrimOptions { depth: 2.0, height: 6.0, ..Default::default() });
        assert_eq!(strips.len(), 2, "the lintel's underside has no floor under it");
        // The first wall's strip runs its whole face, behind the second wall too, the second's stops 2 short of it.
        let expected = (128.0 * 2.0 + 118.0 * 2.0) * 6.0;
        assert!((total_volume(&strips) - expected).abs() < 1e-6, "{}", total_volume(&strips));

        let crowns = build(&m, &faces, &TrimOptions { edges: TrimEdges::Top, on_floor: false, ..Default::default() });
        assert_eq!(crowns.len(), 3, "top edges need no floor");
    }

    #[test]
    fn a_doorway_leaves_the_buried_lintel_ends_out() {
        // A wall cut by a doorway: two pieces beside it and a lintel over it, on a floor.
        let (m, ids) = map_with(&[
            (DVec3::new(-128.0, -8.0, -64.0), DVec3::new(128.0, 0.0, 64.0)),
            (DVec3::new(-96.0, 0.0, 0.0), DVec3::new(-24.0, 96.0, 8.0)),
            (DVec3::new(24.0, 0.0, 0.0), DVec3::new(96.0, 96.0, 8.0)),
            (DVec3::new(-24.0, 72.0, 0.0), DVec3::new(24.0, 96.0, 8.0)),
        ]);
        let faces = faces_for(&m, &select(&ids[1..]));
        let base = build(&m, &faces, &TrimOptions { depth: 2.0, height: 6.0, ..Default::default() });
        assert!(base.iter().all(|s| s.bounds().max.y < 6.5), "no baseboard up on the lintel");
        // Each piece wraps its three open sides (front, back, reveal) and the outer end: 4 strips each.
        assert_eq!(base.len(), 8);

        let crown = build(&m, &faces, &TrimOptions { edges: TrimEdges::Top, depth: 2.0, height: 4.0, on_floor: false, ..Default::default() });
        // Along the top: the front and back run from x -96 to 96 in three pieces each, plus the two outer ends.
        assert_eq!(crown.len(), 8, "the reveals' top edges run under the lintel and get nothing");
        let ring = ((196.0 * 12.0) - (192.0 * 8.0)) * 4.0;
        assert!((total_volume(&crown) - ring).abs() < 1e-6, "one ring with no overlaps, {} against {ring}", total_volume(&crown));
    }
}
