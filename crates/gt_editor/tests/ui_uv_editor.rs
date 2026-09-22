//! Headless tests of the UV editor canvas: corner drags, box select, the move gizmo and undo, driven by real pointer events.

use egui::{Event, Modifiers, PointerButton, Pos2};
use egui_kittest::Harness;
use gt_core::{Aabb, DVec2, DVec3, NodeId};
use gt_editor::commands::Action;
use gt_editor::panels::{self, PanelState};
use gt_editor::state::{EditorState, Prefs};
use gt_editor::texture_ops::{self, MeshUvKind, UvCorner};

struct Fixture {
    state: EditorState,
    panels: PanelState,
    actions: Vec<Action>,
}

/// A flat 2x2 grid mesh projected from above, so its middle vertex is one corner stitched across four faces.
fn grid_fixture() -> (Fixture, NodeId, UvCorner) {
    let mut state = EditorState::new(Prefs::default());
    let layer = state.doc.map.default_layer();
    let grid = gt_geom::mesh_shapes::grid(&Aabb::new(DVec3::ZERO, DVec3::new(128.0, 0.0, 128.0)), 2, 2, "dev/grey");
    let id = state.doc.edit("add", |m, s| {
        let id = m.insert(layer, gt_doc::NodeKind::Mesh(grid));
        for f in 0..4 {
            s.select_face(id, f);
        }

        id
    });
    let faces: Vec<(NodeId, usize)> = (0..4).map(|f| (id, f)).collect();
    texture_ops::mesh_uv(&mut state, &faces, MeshUvKind::PlanarAxis(1), (DVec3::X, DVec3::Y));
    let mesh = state.doc.map.mesh(id).unwrap();
    let middle = mesh.vertices.iter().position(|v| (*v - DVec3::new(64.0, 0.0, 64.0)).length() < 1e-6).unwrap() as u32;
    let corner = (0..4).find_map(|f| mesh.faces[f].indices.iter().position(|v| *v == middle).map(|k| (id, f, k))).unwrap();
    (Fixture { state, panels: PanelState::default(), actions: Vec::new() }, id, corner)
}

fn harness(f: Fixture) -> Harness<'static, Fixture> {
    let mut h = Harness::builder()
        .with_size(egui::vec2(520.0, 900.0))
        // Real frame times, so two clicks count as a double click.
        .with_step_dt(1.0 / 60.0)
        .build_ui_state(|ui, f: &mut Fixture| panels::uv_editor(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    h.run();
    h
}

fn uv(h: &Harness<Fixture>, c: UvCorner) -> DVec2 {
    texture_ops::corner_uv(&h.state().state.doc.map, c).unwrap()
}

fn screen(h: &Harness<Fixture>, uv: DVec2) -> Pos2 {
    h.state().panels.uv.screen_pos(uv).expect("the canvas was drawn")
}

fn press(h: &mut Harness<Fixture>, pos: Pos2, pressed: bool, modifiers: Modifiers) {
    h.event(Event::ModifiersChanged(modifiers));
    h.event(Event::PointerButton { pos, button: PointerButton::Primary, pressed, modifiers });
    h.step();
}

fn drag(h: &mut Harness<Fixture>, from: Pos2, to: Pos2, modifiers: Modifiers) {
    h.event(Event::PointerMoved(from));
    h.step();
    press(h, from, true, modifiers);
    for i in 1..=6 {
        h.event(Event::PointerMoved(from + (to - from) * (i as f32 / 6.0)));
        h.step();
    }

    press(h, to, false, modifiers);
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.run();
}

fn click(h: &mut Harness<Fixture>, pos: Pos2, modifiers: Modifiers) {
    h.event(Event::PointerMoved(pos));
    h.step();
    press(h, pos, true, modifiers);
    press(h, pos, false, modifiers);
    h.event(Event::ModifiersChanged(Modifiers::NONE));
    h.step();
}

#[test]
fn dragging_a_corner_moves_its_stitched_corners_in_one_undo_step() {
    let (f, _, corner) = grid_fixture();
    let mut h = harness(f);
    let stitched = texture_ops::stitched_corners(&h.state().state.doc.map, &(0..4).map(|i| (corner.0, i)).collect::<Vec<_>>(), corner);
    assert_eq!(stitched.len(), 4);
    let start = uv(&h, corner);
    let from = screen(&h, start);
    let undo = h.state().state.doc.history.undo_labels().count();
    let to_uv = start + DVec2::new(0.1, 0.05);
    let to = screen(&h, to_uv);
    drag(&mut h, from, to, Modifiers::NONE);

    let moved = uv(&h, corner);
    assert!((moved - to_uv).length() < 0.01, "the corner follows the pointer: {moved:?} vs {to_uv:?}");
    for c in &stitched {
        assert!((uv(&h, *c) - moved).length() < 1e-6, "stitched corner {c:?} came along");
    }

    let history = &h.state().state.doc.history;
    assert_eq!(history.undo_labels().count(), undo + 1, "a drag is one undo step");
    assert_eq!(history.undo_labels().next(), Some("Move UV Corners"));
    assert_eq!(screen(&h, start), from, "the view holds still while editing");

    h.state_mut().state.doc.undo();
    assert_eq!(uv(&h, corner), start);
}

#[test]
fn pixel_snap_lands_dragged_corners_on_texels() {
    let (f, _, corner) = grid_fixture();
    let mut h = harness(f);
    h.state_mut().panels.uv.pixel_snap = true;
    let start = uv(&h, corner);
    let (from, to) = (screen(&h, start), screen(&h, start + DVec2::new(0.1234, -0.0567)));
    drag(&mut h, from, to, Modifiers::NONE);
    let px = gt_editor::uv_editor::image_pixels(&h.state().state, "dev/grey");
    let moved = uv(&h, corner) * px;
    assert!((moved - moved.round()).length() < 1e-4, "on the pixel grid: {moved:?}");
    assert_ne!(uv(&h, corner), start);
}

#[test]
fn box_select_adds_and_removes_and_the_selection_drags_together() {
    let (f, id, corner) = grid_fixture();
    let mut h = harness(f);
    let a = screen(&h, DVec2::new(-0.1, -0.1));
    let b = screen(&h, uv(&h, corner) + DVec2::new(0.1, 0.1));
    drag(&mut h, a, b, Modifiers::SHIFT);
    let picked = h.state().panels.uv.selected.clone();
    // The box covers the vertices at (0, 0), (64, 0), (0, 64) and the middle: 1 + 2 + 2 + 4 face corners.
    assert_eq!(picked.len(), 9, "{picked:?}");
    assert!(picked.contains(&corner));

    // Ctrl+drag takes the middle out again.
    let m = screen(&h, uv(&h, corner));
    drag(&mut h, m - egui::vec2(6.0, 6.0), m + egui::vec2(6.0, 6.0), Modifiers::COMMAND);
    assert_eq!(h.state().panels.uv.selected.len(), 5);

    // Dragging any selected corner moves the whole selection.
    let first = *h.state().panels.uv.selected.iter().next().unwrap();
    let before: Vec<(UvCorner, DVec2)> = h.state().panels.uv.selected.iter().map(|c| (*c, uv(&h, *c))).collect();
    let from = screen(&h, uv(&h, first));
    drag(&mut h, from, from + egui::vec2(30.0, 0.0), Modifiers::NONE);
    let shift = uv(&h, first) - before.iter().find(|(c, _)| *c == first).unwrap().1;
    assert!(shift.x > 0.0 && shift.y.abs() < 1e-6);
    for (c, u) in before {
        assert!((uv(&h, c) - (u + shift)).length() < 1e-6, "{c:?} moved with the rest");
    }

    assert!(texture_ops::corner_uv(&h.state().state.doc.map, (id, 0, 0)).is_some());
}

#[test]
fn double_click_shows_a_gizmo_whose_arrows_move_along_one_axis() {
    let (f, _, corner) = grid_fixture();
    let mut h = harness(f);
    let p = screen(&h, uv(&h, corner));
    click(&mut h, p, Modifiers::NONE);
    click(&mut h, p, Modifiers::NONE);
    h.run();
    assert!(h.state().panels.uv.gizmo, "double click shows the gizmo");
    assert_eq!(h.state().panels.uv.selected.len(), 4, "with the stitched corners selected");

    let start = uv(&h, corner);
    let arrow = p + egui::vec2(40.0, 0.0);
    drag(&mut h, arrow, arrow + egui::vec2(25.0, 30.0), Modifiers::NONE);
    let moved = uv(&h, corner);
    assert!(moved.x > start.x, "the U arrow moves along u");
    assert_eq!(moved.y, start.y, "and never along v");
    assert!(h.state().panels.uv.gizmo, "the gizmo stays while the selection exists");

    let v_arrow = screen(&h, moved) - egui::vec2(0.0, 40.0);
    drag(&mut h, v_arrow, v_arrow + egui::vec2(30.0, -20.0), Modifiers::NONE);
    let moved_v = uv(&h, corner);
    assert_eq!(moved_v.x, moved.x, "the V arrow keeps u");
    assert!(moved_v.y < moved.y);

    // Escape over the editor hides it and keeps the selection, the app asks the panel before its own shortcuts.
    let ctx = egui::Context::default();
    let mut input = egui::RawInput::default();
    input.events.push(Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: Modifiers::NONE });
    let uv_state = &mut h.state_mut().panels.uv;
    ctx.run_ui(input, |ui| assert!(uv_state.take_escape(ui.ctx()))).textures_delta.clear();
    assert!(!h.state().panels.uv.gizmo);
    assert_eq!(h.state().panels.uv.selected.len(), 4);

    // A click on empty space hides it too, and drops the selection.
    h.state_mut().panels.uv.gizmo = true;
    let empty = screen(&h, DVec2::new(-0.3, -0.3));
    click(&mut h, empty, Modifiers::NONE);
    assert!(!h.state().panels.uv.gizmo);
    assert!(h.state().panels.uv.selected.is_empty());
}

#[test]
fn toolbar_menus_and_chips_reach_every_action() {
    let (f, _, _) = grid_fixture();
    let mut h = harness(f);
    use egui_kittest::kittest::Queryable;
    h.get_by_label("Projection").click();
    h.run();
    h.get_by_label("Cylinder X").click();
    h.run();
    assert_eq!(h.state().actions, vec![Action::MeshUv(MeshUvKind::Cylinder(0))]);
    h.get_by_label("UV").click();
    h.run();
    h.get_by_label("Flip U").click();
    h.run();
    assert_eq!(h.state().state.doc.history.undo_labels().next(), Some("Flip U"));

    // The toolbar chips toggle their settings.
    assert!(!h.state().panels.uv.pixel_snap);
    h.get_by_label("Pixel Snap").click();
    h.run();
    assert!(h.state().panels.uv.pixel_snap);
    h.get_by_label("UV Lock").click();
    h.run();
    assert_eq!(h.state().actions.last(), Some(&Action::ToggleUvLock));
}
