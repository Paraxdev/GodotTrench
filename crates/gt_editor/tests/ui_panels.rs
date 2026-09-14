//! Headless widget tests for the editor panels using egui_kittest (AccessKit based queries).

use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use gt_core::{Aabb, DVec3};
use gt_doc::{NodeKind, ops};
use gt_editor::commands::Action;
use gt_editor::guide::Guide;
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
    harness.get_by_label("Add Layer").click();
    harness.run();
    assert_eq!(harness.state().actions, vec![Action::AddLayer]);

    // Expand the default layer, then hide the light through its eye button.
    harness.get_by_label("Expand").click();
    harness.run();
    harness.get_by_label_contains("light");
    let eyes: Vec<_> = harness.get_all_by_label("Visibility").collect();
    assert_eq!(eyes.len(), 2, "layer and light rows");
    eyes[1].click();
    harness.run();
    assert!(harness.state().state.doc.map.get(id).unwrap().hidden);
}

#[test]
fn outliner_context_menu_selects_and_acts_on_objects() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let brush =
        f.state.doc.edit("add", |m, _| m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(64.0)), "dev/grey").unwrap())));
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 400.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::outliner(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    harness.get_by_label("Expand").click();
    harness.run();
    harness.get_by_label_contains("brush").click_secondary();
    harness.run();
    assert!(harness.state().state.doc.selection.nodes.contains(&brush), "right click selects the row");
    for entry in ["Focus", "Hide", "Duplicate"] {
        harness.get_by_label(entry);
    }
    harness.get_by_label_contains("Move to Layer");
    assert_eq!(harness.get_all_by_label("Lock").count(), 3, "the two row lock icons and the menu entry");
    harness.get_by_label("Delete").click();
    harness.run();
    assert_eq!(harness.state().actions, vec![Action::Delete]);
}

#[test]
fn dragged_entities_and_materials_show_a_preview_at_the_pointer() {
    let mut harness = Harness::builder().with_size(egui::vec2(600.0, 400.0)).build_ui_state(
        |ui, f: &mut Fixture| {
            let ctx = ui.ctx().clone();
            panels::dnd_preview(&ctx, &mut f.state);
        },
        Fixture::new(),
    );
    harness.run();
    assert!(harness.query_by_label("info_player_start").is_none(), "nothing without a drag");

    egui::DragAndDrop::set_payload(&harness.ctx, panels::DndPayload::Entity("info_player_start".into()));
    harness.hover_at(egui::pos2(200.0, 150.0));
    harness.run();
    let card = harness.get_by_label("info_player_start").rect();
    assert!(card.min.x > 200.0 && card.min.y > 150.0, "the card sits next to the pointer, got {card:?}");
    harness.get_by_label("Drop into a view to place it");

    egui::DragAndDrop::set_payload(&harness.ctx, panels::DndPayload::Material("dev/grey".into()));
    harness.run();
    harness.get_by_label("dev/grey");
}

struct GuideFixture {
    guide: Guide,
    state: EditorState,
    actions: Vec<Action>,
}

fn guide_harness(guide: Guide) -> Harness<'static, GuideFixture> {
    let fixture = GuideFixture { guide, state: EditorState::new(Prefs::default()), actions: Vec::new() };
    Harness::builder().with_size(egui::vec2(1200.0, 900.0)).build_ui_state(
        |ui, f: &mut GuideFixture| {
            let ctx = ui.ctx().clone();
            f.guide.show(&ctx, &mut f.state, &mut f.actions);
        },
        fixture,
    )
}

#[test]
fn guide_window_opens_chapters_searches_and_runs_step_buttons() {
    let mut guide = Guide::new(false);
    guide.window_open = true;
    let mut harness = guide_harness(guide);
    harness.run();
    harness.get_by_label("3. Set up a Godot project").click();
    harness.run();
    harness.get_by_label("Open Godot Project…").scroll_to_me();
    harness.run();
    harness.get_by_label("Open Godot Project…").click();
    harness.run();
    assert_eq!(harness.state().actions, vec![Action::OpenProject]);

    let search = harness.get_by_role(egui::accesskit::Role::TextInput);
    search.click();
    harness.run();
    harness.get_by_role(egui::accesskit::Role::TextInput).type_text("hollow");
    harness.run();
    harness.get_by_label("Build a first room: Hollow it into a room").click();
    harness.run();
    harness.get_by_label("Hollow selection");

    // The result scrolls to its step, whose Show me button starts the tour right there.
    harness.get_all_by_label("Show me").nth(5).unwrap().click();
    harness.run();
    assert_eq!(harness.state().guide.tour(), Some((4, 5)));
}

#[test]
fn tour_steps_through_chapters_and_remembers_finished_ones() {
    let mut guide = Guide::new(false);
    guide.start_tour(0, 0);
    let mut harness = guide_harness(guide);
    harness.run();
    harness.get_by_label("Welcome to GodotTrench");
    harness.get_by_label("Next").click();
    harness.run();
    harness.get_by_label("Next").click();
    harness.run();
    assert_eq!(harness.state().guide.tour(), Some((0, 2)));
    harness.get_by_label("Next chapter").click();
    harness.run();
    assert_eq!(harness.state().guide.tour(), Some((1, 0)));
    assert_eq!(harness.state().state.prefs.guide_done, vec!["welcome".to_string()]);
    harness.get_by_label("Back").click();
    harness.run();
    assert_eq!(harness.state().guide.tour(), Some((0, 2)));
    harness.get_by_label("Close").click();
    harness.run();
    assert_eq!(harness.state().guide.tour(), None);

    // Resuming skips the finished chapter.
    let prefs = harness.state().state.prefs.clone();
    harness.state_mut().guide.resume_tour(&prefs);
    assert_eq!(harness.state().guide.tour(), Some((1, 0)));
}

#[test]
fn welcome_offers_the_tour_once() {
    let mut harness = guide_harness(Guide::new(true));
    harness.run();
    harness.get_by_label("Take the guided tour").click();
    harness.run();
    assert_eq!(harness.state().guide.tour(), Some((0, 0)));
    assert!(harness.state().state.prefs.guide_welcome_seen);
    assert!(!harness.state().guide.welcome_open);
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
