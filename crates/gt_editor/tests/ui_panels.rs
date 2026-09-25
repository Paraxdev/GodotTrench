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
fn entity_browser_selects_several_cards_and_places_them() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(700.0, 1400.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::entity_browser(ui, &mut f.state, &mut f.panels, &mut f.actions), Fixture::new());
    harness.run();
    harness.get_by_label("func_door").click();
    harness.run();
    assert_eq!(harness.state().panels.entity_selection, vec!["func_door".to_string()]);
    assert!(harness.state().actions.is_empty(), "a click only selects");

    harness.get_by_label("info_player_start").click_modifiers(egui::Modifiers::COMMAND);
    harness.run();
    assert_eq!(harness.state().panels.entity_selection, vec!["func_door".to_string(), "info_player_start".to_string()]);
    harness.get_by_label("func_door").click_modifiers(egui::Modifiers::COMMAND);
    harness.run();
    assert_eq!(harness.state().panels.entity_selection, vec!["info_player_start".to_string()]);
    harness.get_by_label("func_door").click_modifiers(egui::Modifiers::COMMAND);
    harness.run();

    harness.get_by_label("Place 2").click();
    harness.run();
    assert_eq!(
        harness.state().actions,
        vec![Action::PlaceEntities { classnames: vec!["info_player_start".into(), "func_door".into()], at: None, normal: None, row: DVec3::X }]
    );
}

#[test]
fn entity_browser_shift_click_selects_a_range() {
    let mut ps = PanelState::default();
    let order: Vec<String> = ["a", "b", "c", "d"].map(String::from).to_vec();
    panels::select_entity_card(&mut ps, &order, "b".into(), egui::Modifiers::NONE);
    panels::select_entity_card(&mut ps, &order, "d".into(), egui::Modifiers::SHIFT);
    assert_eq!(ps.entity_selection, ["b", "c", "d"].map(String::from).to_vec());
    panels::select_entity_card(&mut ps, &order, "a".into(), egui::Modifiers::NONE);
    assert_eq!(ps.entity_selection, vec!["a".to_string()]);
}

#[test]
fn placing_entities_lines_them_up_and_gives_brush_entities_a_box() {
    let mut f = Fixture::new();
    f.state.snap = true;
    f.state.grid = 16.0;
    let classnames = vec!["info_player_start".to_string(), "func_door".to_string(), "light".to_string()];
    let action = Action::PlaceEntities { classnames, at: Some(DVec3::ZERO), normal: Some(DVec3::Y), row: DVec3::X };
    gt_editor::commands::execute(&mut f.state, action, &egui::Context::default());
    let map = &f.state.doc.map;
    let placed: Vec<gt_core::NodeId> = f.state.doc.selection.nodes.iter().copied().collect();
    assert_eq!(placed.len(), 3);
    let mut boxes: Vec<(String, Aabb)> = placed.iter().map(|id| (map.entity(*id).unwrap().classname.clone(), map.bounds(*id))).collect();
    boxes.sort_by(|a, b| a.1.center().x.total_cmp(&b.1.center().x));
    let names: Vec<&str> = boxes.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["info_player_start", "func_door", "light"]);
    let door = placed.iter().find(|id| map.entity(**id).unwrap().classname == "func_door").unwrap();
    let brush = map.get(*door).unwrap().children[0];
    let bounds = map.brush(brush).unwrap().bounds();
    assert_eq!(bounds.min.y, 0.0, "the box rests on the floor");
    assert_eq!(bounds.size(), DVec3::splat(64.0));
    assert!(boxes.windows(2).all(|w| w[0].1.max.x <= w[1].1.min.x), "no overlaps: {boxes:?}");
    assert_eq!(f.state.doc.history.undo_labels().next(), Some("Place 3 Entities"));
}

#[test]
fn outliner_reveals_an_object_picked_in_a_view() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let inner = f.state.doc.edit("add", |m, _| {
        for i in 0..300 {
            let min = DVec3::new(i as f64 * 64.0, 0.0, 0.0);
            m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(min, min + DVec3::splat(32.0)), "dev/grey").unwrap()));
        }

        let outer = m.insert(layer, NodeKind::Group(gt_doc::Group::new("outer")));
        let inner = m.insert(outer, NodeKind::Group(gt_doc::Group::new("inner")));
        m.insert(inner, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(8.0)), "dev/grey").unwrap()));
        inner
    });
    let mut harness = Harness::builder()
        .with_size(egui::vec2(400.0, 500.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::outliner(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    assert!(harness.query_by_label_contains("inner").is_none());

    harness.state_mut().state.outliner_reveal = Some(inner);
    harness.run();
    let row = harness.get_by_label_contains("inner").rect();
    assert!(row.min.y > 0.0 && row.max.y < 500.0, "the row is scrolled into sight, got {row:?}");
    harness.get_by_label_contains("outer");
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
fn output_target_drop_down_lists_overlay_targets_sorted_and_filtered() {
    let (mut f, id) = Fixture::with_light();
    let layer = f.state.doc.map.default_layer();
    f.state.doc.edit("wire", |m, _| {
        for i in 0..30 {
            let lamp = ops::create_point_entity(m, layer, "light", DVec3::new(i as f64 * 32.0, 0.0, 0.0));
            m.entity_mut(lamp).unwrap().properties.insert("targetname".into(), format!("lamp_{:02}", i % 15));
        }

        let conn = gt_doc::IoConnection { output: "on".into(), target: "COURT".into(), input: String::new(), parameter: String::new(), delay: 0.0, times: -1 };
        m.entity_mut(id).unwrap().outputs = vec![conn];
    });
    let sidecar = r#"{ "format": "godottrench-overlay", "overlays": [{ "name": "Court", "items": [
        { "name": "Breaker", "min": [0, 0, 0], "max": [8, 8, 8], "targetnames": ["court_breaker"] },
        { "name": "Fireflies", "min": [0, 0, 0], "max": [8, 8, 8], "targetnames": ["court_fireflies"] }
    ] }] }"#;
    f.state.overlay_ghosts.items = gt_editor::overlays::parse(sidecar).unwrap();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(420.0, 1200.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::inspector(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();

    let drop_down = |harness: &Harness<'_, Fixture>, label: &str| {
        let row = harness.get_by_label(label).rect().center().y;
        harness.get_all_by_value("▾").min_by(|a, b| (a.rect().center().y - row).abs().total_cmp(&(b.rect().center().y - row).abs())).unwrap().click();
    };
    let target_drop_down = |harness: &Harness<'_, Fixture>| drop_down(harness, "target");
    target_drop_down(&harness);
    harness.run();
    assert!(harness.query_by_label_contains("lamp_").is_none(), "what was typed filters the list");
    assert!(harness.query_by_label("court_fireflies   (Godot overlay)").is_some());
    harness.get_by_label("court_breaker   (Godot overlay)").click();
    harness.run();
    assert_eq!(harness.state().state.doc.map.entity(id).unwrap().outputs[0].target, "court_breaker");

    target_drop_down(&harness);
    harness.run();
    assert_eq!(harness.get_all_by_label("lamp_03").count(), 1, "a targetname shared by several entities is listed once");
    let overlay = harness.get_by_label("court_breaker   (Godot overlay)").rect();
    let first_lamp = harness.get_by_label("lamp_00").rect();
    assert!(overlay.min.y < first_lamp.min.y, "the list is sorted by name, overlay targets are not left at the end");
    harness.key_press(egui::Key::Escape);
    harness.run();

    drop_down(&harness, "fixture");
    harness.run();
    assert_eq!(harness.get_all_by_label("lamp_03").count(), 1, "target keys list the same targets");
    harness.get_by_label("court_fireflies   (Godot overlay)").click();
    harness.run();
    assert_eq!(harness.state().state.doc.map.entity(id).unwrap().properties.get("fixture").map(String::as_str), Some("court_fireflies"));
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
    harness.get_by_label("Cylinder Y").click();
    harness.run();
    assert_eq!(harness.state().actions, vec![Action::MeshUv(gt_editor::texture_ops::MeshUvKind::Cylinder(1))]);
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
fn reference_list_fits_the_names_and_the_divider_drags() {
    let mut f = Fixture::new();
    f.panels.reference_class = "info_spawner".into();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(900.0, 700.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::reference(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    let detail_x = |h: &Harness<Fixture>| h.get_by_label("Use from GDScript").rect().min.x;
    let auto = detail_x(&harness);
    let list_item = harness.get_all_by_label("info_spawner").map(|n| n.rect()).min_by(|a, b| a.min.x.total_cmp(&b.min.x)).unwrap();
    assert!(auto > list_item.max.x && auto < 450.0, "auto width fits the names without taking half, detail at {auto}");

    let split = egui::pos2(auto - 8.0 - 4.0, 400.0);
    harness.hover_at(split);
    harness.run();
    harness.event(egui::Event::PointerButton { pos: split, button: egui::PointerButton::Primary, pressed: true, modifiers: Default::default() });
    harness.run();
    for step in 1..=5 {
        harness.event(egui::Event::PointerMoved(split + egui::vec2(30.0 * step as f32, 0.0)));
        harness.run();
    }

    harness.event(egui::Event::PointerButton {
        pos: split + egui::vec2(150.0, 0.0),
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    });
    harness.run();
    let dragged = detail_x(&harness);
    assert!((dragged - auto - 150.0).abs() < 12.0, "the divider follows the pointer, {auto} to {dragged}");
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
    harness.get_by_label("Edit in Scatter Panel").click();
    harness.run();
    assert_eq!(harness.state().actions, vec![Action::ActivateScatter(id), Action::ShowScatterPanel]);
}

#[test]
fn scatter_panel_switches_models_picks_targets_and_takes_dropped_models() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let (kind, items) = gt_doc::scatter::preset("rocks").unwrap();
    let first = items[0].label().to_string();
    let count = items.len();
    let id = f.state.doc.edit("add", |m, _| m.insert(layer, NodeKind::Scatter(gt_doc::Scatter::new("stones", kind, items))));
    f.state.active_scatter = Some(id);
    let mut harness = Harness::builder()
        .with_size(egui::vec2(420.0, 1400.0))
        .build_ui_state(|ui, f: &mut Fixture| gt_editor::scatter_panel::scatter_panel(ui, &mut f.state, &mut f.actions, None), f);
    harness.run();
    assert!(harness.query_by_label("Rename").is_some() && harness.query_by_label("Delete set").is_some(), "the active set can be renamed and deleted");
    assert!(harness.query_by_label(&first).is_some(), "each model has a card");
    assert_eq!(harness.get_all_by_label("paint").count(), count, "and a switch per card");

    harness.get_all_by_label("paint").next().unwrap().click();
    harness.run();
    let set = harness.state().state.doc.map.scatter(id).unwrap().clone();
    assert!(!set.items[0].enabled && set.items[1..].iter().all(|i| i.enabled), "the first card is switched off");

    harness.get_by_label("Pick target").click();
    harness.run();
    assert!(harness.state().state.scatter_eyedropper, "the eyedropper waits for a click in a view");
    assert_eq!(harness.state().actions, vec![Action::SetTool(gt_editor::tools::ToolKind::Scatter)]);

    harness.get_by_label("Preset").click();
    harness.run();
    harness.get_by_label("grass").click();
    harness.run();
    assert_eq!(harness.state().actions.last(), Some(&Action::ScatterPreset("grass".into())));

    let f = harness.state_mut();
    gt_editor::scatter_panel::drop_payload(&mut f.state, &panels::DndPayload::Model("C:/models/fir.glb".into()));
    let set = f.state.doc.map.scatter(id).unwrap();
    assert_eq!(set.items.len(), count + 1, "a model dropped on the panel joins the active set");
    assert_eq!(set.items.last().unwrap().label(), "fir");
    harness.run();
    assert!(harness.query_by_label("fir").is_some(), "and gets its own card");
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
fn inspector_types_the_position_and_size_of_the_selection() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let id = f.state.doc.edit("add", |m, s| {
        let id = m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::new(64.0, 32.0, 16.0)), "wood").unwrap()));
        s.select_node(id);
        id
    });
    let mut harness = Harness::builder()
        .with_size(egui::vec2(520.0, 700.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::inspector(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    let mut type_into = |field: usize, text: &str| {
        harness.get_all_by_role(egui::accesskit::Role::SpinButton).nth(field).unwrap().click();
        harness.run();
        harness.get_all_by_role(egui::accesskit::Role::SpinButton).nth(field).unwrap().type_text(text);
        harness.run();
        harness.key_press(egui::Key::Enter);
        harness.run();
    };
    type_into(4, "128");
    type_into(0, "32");
    let doc = &harness.state().state.doc;
    assert_eq!(doc.map.bounds(id), Aabb::new(DVec3::new(32.0, 0.0, 0.0), DVec3::new(96.0, 128.0, 16.0)), "the size grows from the lowest corner");
    assert_eq!(doc.history.undo_labels().take(2).collect::<Vec<_>>(), ["Set Position", "Set Size"]);
}

#[test]
fn outliner_rows_tell_brushes_apart_and_outline_the_hovered_one() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let right = f.state.doc.edit("add", |m, _| {
        m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::new(16.0, 128.0, 256.0)), "wall").unwrap()));
        m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::new(240.0, 0.0, 0.0), DVec3::new(256.0, 128.0, 256.0)), "wall").unwrap()))
    });
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 400.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::outliner(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    harness.get_by_label("Expand").click();
    harness.run();
    harness.get_by_label_contains("16x128x256 at 0 0 0");
    harness.get_by_label_contains("16x128x256 at 240 0 0").hover();
    harness.run();
    assert_eq!(harness.state().state.outliner_hover, Some(right), "the views outline the hovered row's brush");
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
fn outliner_shift_click_selects_a_range_and_ctrl_click_toggles() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let ids: Vec<gt_core::NodeId> = f.state.doc.edit("add", |m, _| {
        (0..4)
            .map(|i| {
                let min = DVec3::new(i as f64 * 128.0, 0.0, 0.0);
                m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(min, min + DVec3::splat(64.0)), "dev/grey").unwrap()))
            })
            .collect()
    });
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 400.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::outliner(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    harness.get_by_label("Expand").click();
    harness.run();
    let selected = |h: &Harness<Fixture>| {
        let sel = &h.state().state.doc.selection.nodes;
        ids.iter().map(|id| sel.contains(id)).collect::<Vec<_>>()
    };
    let click = |h: &mut Harness<Fixture>, i: usize, modifiers: egui::Modifiers| {
        h.get_all_by_label_contains("64x64x64 at").nth(i).unwrap().click_modifiers(modifiers);
        h.run();
    };

    click(&mut harness, 0, egui::Modifiers::NONE);
    click(&mut harness, 2, egui::Modifiers::SHIFT);
    assert_eq!(selected(&harness), [true, true, true, false], "Shift selects the rows from the last clicked one");
    click(&mut harness, 1, egui::Modifiers::COMMAND);
    assert_eq!(selected(&harness), [true, false, true, false], "Ctrl takes one out");
    click(&mut harness, 3, egui::Modifiers::COMMAND | egui::Modifiers::SHIFT);
    assert_eq!(selected(&harness), [true, true, true, true], "Ctrl+Shift adds the range from the row Ctrl clicked");
    click(&mut harness, 3, egui::Modifiers::NONE);
    assert_eq!(selected(&harness), [false, false, false, true]);
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
    harness.get_by_label_contains("64x64x64 at 0 0 0").click_secondary();
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

    egui::DragAndDrop::set_payload(&harness.ctx, panels::DndPayload::Entities(vec!["info_player_start".into()]));
    harness.hover_at(egui::pos2(200.0, 150.0));
    harness.run();
    let card = harness.get_by_label("info_player_start").rect();
    assert!(card.min.x > 200.0 && card.min.y > 150.0, "the card sits next to the pointer, got {card:?}");
    harness.get_by_label("Drop into a view to place it");

    egui::DragAndDrop::set_payload(&harness.ctx, panels::DndPayload::Entities(vec!["info_player_start".into(), "light".into()]));
    harness.run();
    harness.get_by_label("2 entities");

    egui::DragAndDrop::set_payload(&harness.ctx, panels::DndPayload::Material("dev/grey".into()));
    harness.run();
    harness.get_by_label("dev/grey");
}

#[test]
fn material_menu_opens_the_hotspot_editor_on_the_clicked_material() {
    let mut f = Fixture::new();
    f.state.prefs.recent_materials = vec!["bricks/red".into()];
    let mut harness = Harness::builder()
        .with_size(egui::vec2(700.0, 400.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::material_browser(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    // Missing thumbnails keep requesting repaints, so step a fixed number of frames.
    harness.run_steps(2);
    let label = harness.get_by_label("recent").rect();
    let cell = egui::pos2(label.max.x + 22.0, label.center().y);
    harness.hover_at(cell);
    harness.run_steps(1);
    for pressed in [true, false] {
        harness.event(egui::Event::PointerButton { pos: cell, button: egui::PointerButton::Secondary, pressed, modifiers: Default::default() });
        harness.run_steps(1);
    }

    harness.run_steps(2);
    harness.get_by_label("Hotspot editor").click();
    harness.run_steps(2);
    assert_eq!(harness.state().actions, vec![Action::EditHotspots("bricks/red".into())]);
    assert_eq!(harness.state().state.current_material, "dev/grey", "opening the editor does not change the current material");
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

#[test]
fn logic_panel_fires_cascade_and_flags_broken_links() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    f.state.doc.edit("wire", |m, _| {
        let relay = ops::create_point_entity(m, layer, "logic_relay", DVec3::ZERO);
        let e = m.entity_mut(relay).unwrap();
        e.properties.insert("targetname".into(), "aaa".into());
        e.outputs = vec![
            gt_doc::IoConnection {
                output: "triggered".into(),
                target: "light1".into(),
                input: "turn_on".into(),
                parameter: String::new(),
                delay: 0.0,
                times: -1,
            },
            gt_doc::IoConnection { output: "triggered".into(), target: "ghost".into(), input: "kill".into(), parameter: String::new(), delay: 0.0, times: -1 },
        ];
        let light = ops::create_point_entity(m, layer, "light", DVec3::new(64.0, 0.0, 0.0));
        m.entity_mut(light).unwrap().properties.insert("targetname".into(), "light1".into());
    });

    let mut harness =
        Harness::builder().with_size(egui::vec2(520.0, 800.0)).build_ui_state(|ui, f: &mut Fixture| panels::logic_panel(ui, &mut f.state, &mut f.panels), f);
    harness.run();
    harness.get_by_label("Fire").click();
    harness.run();
    assert!(harness.query_by_label("2 steps").is_some(), "both wired outputs are listed");
    assert!(harness.query_by_label("1 broken").is_some(), "the missing target is flagged");
}

#[test]
fn model_browser_groups_by_source_with_a_header_per_pack() {
    use gt_editor::models::{ModelEntry, PackInfo};
    use std::sync::Arc;

    let mut f = Fixture::new();
    let pack_a = Arc::new(PackInfo { name: "Pack A".into(), license: Some("CC0".into()), ..Default::default() });
    let pack_b = Arc::new(PackInfo { name: "Pack B".into(), license: Some("CC-BY".into()), ..Default::default() });
    f.state.model_library.entries = vec![
        ModelEntry { name: "one".into(), folder: String::new(), path: "one.glb".into(), ext: "glb".into(), source: pack_a, credit: None, mtime: None },
        ModelEntry { name: "two".into(), folder: String::new(), path: "two.glb".into(), ext: "glb".into(), source: pack_b, credit: None, mtime: None },
    ];

    let mut harness = Harness::builder()
        .with_size(egui::vec2(700.0, 500.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::model_browser(ui, &mut f.state, &mut f.panels, &mut f.actions, None), f);
    harness.run();

    // Sort defaults to Name, no group headers yet.
    assert!(harness.query_by_label("Pack A").is_none(), "grouped headers only show once Source sort is picked");

    harness.get_by_value("Folder").click();
    harness.run();
    harness.get_by_label("Source").click();
    harness.run();

    assert!(harness.query_by_label("Pack A").is_some(), "a header names each pack");
    assert!(harness.query_by_label("Pack B").is_some());
    assert!(harness.query_by_label("CC0").is_some(), "the header shows the pack's license");
    assert_eq!(harness.query_all_by_label("1 model").count(), 2, "each header shows its model count");
}
