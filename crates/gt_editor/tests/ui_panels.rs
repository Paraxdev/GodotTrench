//! Headless widget tests for the editor panels using egui_kittest (AccessKit based queries).

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use gt_core::{Aabb, DVec3};
use gt_doc::{NodeKind, ops};
use gt_editor::commands::Action;
use gt_editor::panels::{self, PanelState};
use gt_editor::state::{EditorState, Prefs};
use gt_geom::Brush;

struct Fixture {
    state: EditorState,
    panels: PanelState,
    actions: Vec<Action>,
}

impl Fixture {
    fn new() -> Self {
        Self { state: EditorState::new(Prefs::default()), panels: PanelState::default(), actions: Vec::new() }
    }

    fn with_light() -> (Self, gt_core::NodeId) {
        let mut f = Self::new();
        let layer = f.state.doc.map.default_layer();
        let id = f.state.doc.edit("add", |m, s| {
            let id = ops::create_point_entity(m, layer, "light", DVec3::new(8.0, 16.0, 32.0));
            s.select_node(id);
            id
        });
        (f, id)
    }
}

#[test]
fn entity_browser_creates_brush_entities() {
    let mut harness = Harness::new_ui_state(|ui, f: &mut Fixture| panels::entity_browser(ui, &mut f.state, &mut f.panels, &mut f.actions), Fixture::new());
    harness.run();
    harness.get_by_label("func_door  ").click();
    harness.run();
    assert_eq!(harness.state().actions, vec![Action::CreateBrushEntity("func_door".into())]);
    // Point entities are listed too.
    harness.get_by_label_contains("info_player_start");
}

#[test]
fn inspector_adds_io_output() {
    let (fixture, id) = Fixture::with_light();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(420.0, 900.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::inspector(ui, &mut f.state, &mut f.panels, &mut f.actions), fixture);
    harness.run();
    harness.get_by_label("Properties");
    harness.get_by_label("+ Add Output").click();
    harness.run();
    let entity = harness.state().state.doc.map.entity(id).unwrap().clone();
    assert_eq!(entity.outputs.len(), 1);
    harness.get_by_label("Remove").click();
    harness.run();
    assert!(harness.state().state.doc.map.entity(id).unwrap().outputs.is_empty());
    assert!(harness.state().state.doc.history.can_undo());
}

#[test]
fn inspector_shows_worldspawn_without_selection() {
    let mut harness = Harness::new_ui_state(|ui, f: &mut Fixture| panels::inspector(ui, &mut f.state, &mut f.panels, &mut f.actions), Fixture::new());
    harness.run();
    harness.get_by_label("Map (worldspawn)");
}

#[test]
fn face_inspector_reset_is_undoable() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let brush = Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(64.0)), "dev/grey").unwrap();
    let id = f.state.doc.edit("add", |m, s| {
        let id = m.insert(layer, NodeKind::Brush(brush));
        s.select_face(id, 0);
        id
    });
    f.state.doc.edit("scale", |m, _| m.brush_mut(id).unwrap().faces[0].data.uv.scale = gt_core::DVec2::new(4.0, 4.0));
    let mut harness = Harness::builder()
        .with_size(egui::vec2(420.0, 700.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::inspector(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    harness.get_by_label("Reset").click();
    harness.run();
    let face = &harness.state().state.doc.map.brush(id).unwrap().faces[0];
    assert_eq!(face.data.uv.scale, gt_core::DVec2::ONE);
    assert_eq!(harness.state().state.doc.history.undo_labels().next(), Some("Reset UV"));
}

#[test]
fn face_inspector_texture_tools() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let mesh = gt_geom::mesh_shapes::cylinder(&Aabb::new(DVec3::ZERO, DVec3::splat(64.0)), 8, "dev/grey");
    let brush = Brush::from_aabb(&Aabb::new(DVec3::new(100.0, 0.0, 0.0), DVec3::splat(164.0)), "dev/grey").unwrap();
    let (mesh_id, brush_id) = f.state.doc.edit("add", |m, s| {
        let b = m.insert(layer, NodeKind::Brush(brush));
        let me = m.insert(layer, NodeKind::Mesh(mesh));
        s.select_face(b, 0);
        (me, b)
    });
    let mut harness = Harness::builder()
        .with_size(egui::vec2(460.0, 800.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::inspector(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    harness.get_by_label("× 2").click();
    harness.run();
    assert_eq!(harness.state().state.doc.map.brush(brush_id).unwrap().faces[0].data.uv.scale, gt_core::DVec2::splat(2.0));
    harness.get_by_label("Left").click();
    harness.get_by_label("0.5").click();
    harness.run();
    assert_eq!(harness.state().actions, vec![Action::Justify(gt_geom::Justify::Left), Action::TexelDensity(0.5)]);

    // Mesh faces offer the UV projections.
    harness.state_mut().actions.clear();
    harness.state_mut().state.doc.select(|_, s| {
        s.clear();
        s.select_face(mesh_id, 2);
    });
    harness.run();
    harness.get_by_label("Cylinder").click();
    harness.run();
    assert_eq!(harness.state().actions, vec![Action::MeshUv(gt_editor::texture_ops::MeshUvKind::Cylinder)]);
}

#[test]
fn reference_panel_shows_code_for_the_selected_entity() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    f.state.doc.edit("add", |m, s| {
        let id = ops::create_point_entity(m, layer, "info_spawner", DVec3::ZERO);
        s.select_node(id);
    });
    let mut harness = Harness::builder()
        .with_size(egui::vec2(900.0, 800.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::reference(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    harness.get_by_label("Use from GDScript");
    harness.get_by_label("C# class").click();
    harness.run();
    harness.get_by_label("Place").click();
    harness.run();
    assert_eq!(harness.state().actions, vec![Action::CreatePointEntity { classname: "info_spawner".into(), at: None }]);
}

#[test]
fn scatter_inspector_edits_palette_and_activates() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let (kind, items) = gt_doc::scatter::preset("rocks").unwrap();
    let id = f.state.doc.edit("add", |m, s| {
        let id = m.insert(layer, NodeKind::Scatter(gt_doc::Scatter::new("rocks", kind, items)));
        s.select_node(id);
        id
    });
    let mut harness = Harness::builder()
        .with_size(egui::vec2(520.0, 700.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::inspector(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    harness.get_by_label("foliage").click();
    harness.run();
    assert_eq!(harness.state().state.doc.map.scatter(id).unwrap().kind, gt_doc::ScatterKind::Foliage);
    harness.get_by_label("Paint Into This Set").click();
    harness.run();
    assert_eq!(harness.state().actions, vec![Action::ActivateScatter(id)]);
}

#[test]
fn selection_summary_offers_door_wizards() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    f.state.doc.edit("add", |m, s| {
        let a = m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::new(48.0, 96.0, 8.0)), "wood").unwrap()));
        let b = m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::new(0.0, 96.0, 0.0), DVec3::new(48.0, 104.0, 8.0)), "wood").unwrap()));
        s.select_node(a);
        s.select_node(b);
    });
    let mut harness = Harness::builder()
        .with_size(egui::vec2(520.0, 700.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::inspector(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    harness.get_by_label("Door, hinge right").click();
    harness.run();
    assert!(matches!(
        harness.state().actions.as_slice(),
        [Action::MakeDoor { kind: gt_editor::entity_wizards::DoorKind::Hinged { side: gt_editor::entity_wizards::HingeSide::Right, .. }, trigger: true }]
    ));
}

#[test]
fn outliner_toggles_visibility_and_adds_layers() {
    let (fixture, id) = Fixture::with_light();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 400.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::outliner(ui, &mut f.state, &mut f.panels, &mut f.actions), fixture);
    harness.run();
    harness.get_by_label("+ Layer").click();
    harness.run();
    assert_eq!(harness.state().actions, vec![Action::AddLayer]);

    // Expand the default layer, then hide the light through its eye button.
    harness.get_by_label("+").click();
    harness.run();
    harness.get_by_label_contains("light");
    let eyes: Vec<_> = harness.get_all_by_label("👁").collect();
    assert_eq!(eyes.len(), 2, "layer and light rows");
    eyes[1].click();
    harness.run();
    assert!(harness.state().state.doc.map.get(id).unwrap().hidden);
}

#[test]
fn history_panel_undoes_to_clicked_step() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    for i in 0..3 {
        f.state.doc.edit(&format!("step {i}"), |m, _| ops::create_point_entity(m, layer, "light", DVec3::ZERO));
    }
    let mut harness = Harness::new_ui_state(|ui, f: &mut Fixture| panels::history(ui, &mut f.state), f);
    harness.run();
    harness.get_by_label("step 1").click();
    harness.run();
    assert_eq!(harness.state().state.doc.map.entity_count(), 1);
    assert_eq!(harness.state().state.doc.history.redo_labels().count(), 2);
}
