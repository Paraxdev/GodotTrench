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
    /// A state that knows the Gameplay entities pack, like a project that installed it.
    fn new() -> Self {
        let mut state = EditorState::new(Prefs::default());
        state.game = gt_formats::GameConfig::with_gameplay_pack();
        Self { state, panels: PanelState::default(), actions: Vec::new() }
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
fn the_app_takes_the_links_to_open_and_only_opens_web_links() {
    let ctx = egui::Context::default();
    let mut state = EditorState::new(Prefs::default());
    ctx.begin_pass(Default::default());
    ctx.open_url(egui::OpenUrl::new_tab("file:///etc/passwd"));
    let urls = panels::take_open_urls(&ctx);
    assert_eq!(urls, ["file:///etc/passwd"]);
    panels::open_link(&mut state, &ctx, &urls[0]);
    let out = ctx.end_pass();
    let commands = out.platform_output.commands.clone();
    out.drop_without_applying_deltas();
    assert!(state.status.starts_with("Only web links open in the browser, file:///etc/passwd is copied"), "{}", state.status);
    assert_eq!(commands, [egui::OutputCommand::CopyText("file:///etc/passwd".into())], "nothing is left for eframe, which cannot open links");
}

#[test]
fn a_link_no_browser_opened_is_copied() {
    let ctx = egui::Context::default();
    let mut state = EditorState::new(Prefs::default());
    // What the app does each frame, returning what it copied.
    let frame = |state: &mut EditorState| {
        ctx.begin_pass(Default::default());
        panels::copy_unopened_links(state, &ctx);
        let out = ctx.end_pass();
        let commands = out.platform_output.commands.clone();
        out.drop_without_applying_deltas();
        commands
    };
    let (tx, opened) = std::sync::mpsc::channel();
    panels::open_link_with(&mut state, &ctx, "https://example.com/?a=1&b=2", move |url| {
        tx.send(url.to_string()).unwrap();
        Ok(())
    });
    assert_eq!(opened.recv_timeout(std::time::Duration::from_secs(5)).unwrap(), "https://example.com/?a=1&b=2", "the opener gets the whole link");
    assert_eq!(state.status, "Opening https://example.com/?a=1&b=2 in your browser");
    assert!(frame(&mut state).is_empty(), "an opened link is not copied");

    // Like xdg-open exiting with 3 when it finds no browser, after it started fine.
    panels::open_link_with(&mut state, &ctx, "https://example.com/guide", |_| Err(std::io::Error::other("xdg-open exit status: 3")));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    let copied = loop {
        let commands = frame(&mut state);
        if !commands.is_empty() || std::time::Instant::now() > deadline {
            break commands;
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    assert_eq!(copied, [egui::OutputCommand::CopyText("https://example.com/guide".into())]);
    assert_eq!(state.status, "No browser opened https://example.com/guide, the link is copied to the clipboard");
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
fn help_lines_are_short_and_the_preference_hides_them() {
    let line = "Nothing selected, these settings apply to the whole map";
    let mut harness = Harness::new_ui_state(|ui, f: &mut Fixture| panels::inspector(ui, &mut f.state, &mut f.panels, &mut f.actions), Fixture::new());
    harness.run();
    assert!(harness.query_by_label(line).is_some());
    harness.state_mut().state.prefs.help_text = false;
    harness.run();
    assert!(harness.query_by_label(line).is_none(), "hidden for people who know their way around");
    harness.get_by_label("Map (worldspawn)");

    let line = "Drag a card into a view, or double click it";
    let mut harness = Harness::builder()
        .with_size(egui::vec2(700.0, 900.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::entity_browser(ui, &mut f.state, &mut f.panels, &mut f.actions), Fixture::new());
    harness.run();
    assert!(harness.query_by_label(line).is_some());
    harness.state_mut().state.prefs.help_text = false;
    harness.run();
    assert!(harness.query_by_label(line).is_none());
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
fn face_inspector_texture_settings_apply_to_the_whole_texture() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let brush = Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(64.0)), "dev/grey").unwrap();
    f.state.doc.edit("add", |m, s| {
        let id = m.insert(layer, NodeKind::Brush(brush));
        s.select_face(id, 0);
    });
    let mut harness = Harness::builder()
        .with_size(egui::vec2(460.0, 900.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::inspector(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    assert!(harness.query_by_label("World").is_none(), "closed while the texture has no settings");
    harness.get_by_label("Texture Settings").click();
    harness.run();
    harness.get_by_label("World").click();
    harness.run();
    harness.get_by_label("Bake light onto it").click();
    harness.run();
    let set = harness.state().state.doc.map.texture("dev/grey");
    assert_eq!((set.projection, set.bake), (gt_doc::textures::Projection::World, false));
    assert_eq!(harness.state().state.doc.history.undo_labels().next(), Some("Texture Settings"));
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
fn reference_panel_opens_only_scripts() {
    let dir = std::env::temp_dir().join(format!("gt_reference_open_{}", std::process::id()));
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::write(dir.join("project.godot"), "config_version=5\n").unwrap();
    std::fs::write(dir.join("scripts/door.gd"), "extends Node3D\n").unwrap();
    std::fs::write(dir.join("scripts/setup.bat"), "echo hi\n").unwrap();
    let mut f = Fixture::new();
    f.state.game.project_root = Some(dir.clone());
    for (class, script) in [("door_script", "res://scripts/door.gd"), ("door_program", "res://scripts/setup.bat")] {
        let def = serde_json::json!({ "classname": class, "type": "point", "script": script });
        f.state.game.entities.push(serde_json::from_value(def).unwrap());
    }

    f.panels.reference_class = "door_script".into();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(900.0, 800.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::reference(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    assert_eq!(harness.query_all_by_label("Open").count(), 1, "a GDScript opens in the script editor");

    harness.state_mut().panels.reference_class = "door_program".into();
    harness.run();
    assert_eq!(harness.query_all_by_label("Open").count(), 0, "a project's game config cannot make the button run a program");
    let _ = std::fs::remove_dir_all(dir);
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
    assert!(harness.query_by_label("Door, hinge right").is_none(), "the Gameplay section starts collapsed");
    harness.get_by_label("Gameplay").click();
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
    // Digit by digit like a person types, nothing may change until Enter.
    let type_digits = |harness: &mut Harness<Fixture>, field: usize, text: &str| {
        harness.get_all_by_role(egui::accesskit::Role::SpinButton).nth(field).unwrap().click();
        harness.run();
        harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
        harness.run();
        let before = harness.state().state.doc.map.bounds(id);
        for c in text.chars() {
            harness.get_all_by_role(egui::accesskit::Role::SpinButton).nth(field).unwrap().type_text(&c.to_string());
            harness.run();
            assert_eq!(harness.state().state.doc.map.bounds(id), before, "typing {text} applied early");
        }
    };
    let undo_steps = |harness: &Harness<Fixture>| harness.state().state.doc.history.undo_labels().count();
    let steps = undo_steps(&harness);
    type_digits(&mut harness, 4, "128");
    harness.key_press(egui::Key::Enter);
    harness.run();
    type_digits(&mut harness, 0, "32");
    harness.key_press(egui::Key::Enter);
    harness.run();
    let doc = &harness.state().state.doc;
    assert_eq!(doc.map.bounds(id), Aabb::new(DVec3::new(32.0, 0.0, 0.0), DVec3::new(96.0, 128.0, 16.0)), "the size grows from the lowest corner");
    assert_eq!(doc.history.undo_labels().take(2).collect::<Vec<_>>(), ["Set Position", "Set Size"]);
    assert_eq!(undo_steps(&harness), steps + 2, "one undo step per typed value");

    type_digits(&mut harness, 3, "0");
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert_eq!(harness.state().state.doc.map.bounds(id).size(), DVec3::new(64.0, 128.0, 16.0), "Escape keeps the old size");
    assert_eq!(undo_steps(&harness), steps + 2);
}

#[test]
fn outliner_rows_tell_brushes_apart_and_outline_the_hovered_one() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let (left, right) = f.state.doc.edit("add", |m, _| {
        let left = m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::new(16.0, 128.0, 256.0)), "wall").unwrap()));
        (left, m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::new(240.0, 0.0, 0.0), DVec3::new(256.0, 128.0, 256.0)), "wall").unwrap())))
    });
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 400.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::outliner(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    harness.get_by_label("Expand").click();
    harness.run();
    harness.get_by_label(&format!("brush{}", left.0));
    harness.get_by_label(&format!("brush{}", right.0)).hover();
    harness.run();
    assert_eq!(harness.state().state.outliner_hover, Some(right), "the views outline the hovered row's brush");
    harness.run_steps(60);
    harness.get_by_label("Size 16 x 128 x 256");
    harness.get_by_label("From 240 0 0 to 256 128 256");

    harness.get_by_role(egui::accesskit::Role::TextInput).click();
    harness.run();
    harness.get_by_role(egui::accesskit::Role::TextInput).type_text(&format!("brush{}", right.0));
    harness.run();
    harness.get_by_label(&format!("brush{}", right.0));
    assert!(harness.query_by_label(&format!("brush{}", left.0)).is_none(), "the filter matches what the rows show");
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
        h.get_by_label(&format!("brush{}", ids[i].0)).click_modifiers(modifiers);
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
fn outliner_ctrl_shift_range_leaves_out_the_children_of_a_selected_group() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let cube = |m: &mut gt_doc::Map, parent, x: f64, size: f64| {
        let min = DVec3::new(x, 0.0, 0.0);
        m.insert(parent, NodeKind::Brush(Brush::from_aabb(&Aabb::new(min, min + DVec3::splat(size)), "dev/grey").unwrap()))
    };
    let (group, first, a, b) = f.state.doc.edit("add", |m, _| {
        let group = m.insert(layer, NodeKind::Group(gt_doc::Group::new("crates")));
        let first = cube(m, group, 0.0, 32.0);
        cube(m, group, 64.0, 32.0);
        (group, first, cube(m, layer, 256.0, 64.0), cube(m, layer, 512.0, 64.0))
    });
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 400.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::outliner(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    harness.get_by_label("Expand").click();
    harness.run();
    harness.get_by_label("Expand").click();
    harness.run();
    harness.get_by_label("crates").click();
    harness.run();
    harness.get_by_label(&format!("brush{}", b.0)).click_modifiers(egui::Modifiers::COMMAND);
    harness.run();
    harness.get_by_label(&format!("brush{}", first.0)).click_modifiers(egui::Modifiers::COMMAND | egui::Modifiers::SHIFT);
    harness.run();
    let selected: Vec<_> = harness.state().state.doc.selection.nodes.iter().copied().collect();
    assert_eq!(selected, [group, a, b], "the group's children would move twice");
}

#[test]
fn outliner_renames_the_selected_object_in_its_row() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let brush = f.state.doc.edit("add", |m, s| {
        let id = m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(64.0)), "dev/grey").unwrap()));
        s.select_node(id);
        id
    });
    gt_editor::commands::execute(&mut f.state, Action::Rename, &egui::Context::default());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 400.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::outliner(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    let steps = harness.state().state.doc.history.undo_labels().count();
    let field = harness.get_all_by_role(egui::accesskit::Role::TextInput).last().unwrap().value();
    assert_eq!(field, Some(format!("brush{}", brush.0)), "F2 opens the row's name field after the filter");
    for c in ["n", "orth wall"] {
        harness.get_all_by_role(egui::accesskit::Role::TextInput).last().unwrap().type_text(c);
        harness.run();
    }

    harness.key_press(egui::Key::Enter);
    harness.run();
    let state = &harness.state().state;
    assert_eq!(state.doc.map.get(brush).unwrap().name(), "north wall", "typing replaces the whole default name");
    assert_eq!(state.doc.history.undo_labels().count(), steps + 1, "one undo step");
    assert_eq!(state.doc.history.undo_labels().next(), Some("Rename"));
    assert_eq!(state.renaming, None);
    harness.get_by_label("north wall");

    // The row menu opens the field too, and Escape leaves the name alone.
    harness.get_by_label("north wall").click_secondary();
    harness.run();
    harness.get_by_label("Rename").click();
    harness.run();
    harness.get_all_by_role(egui::accesskit::Role::TextInput).last().unwrap().type_text("cellar");
    harness.run();
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert_eq!(harness.state().state.doc.map.get(brush).unwrap().name(), "north wall");
    assert_eq!(harness.state().state.doc.history.undo_labels().count(), steps + 1);
    assert_eq!(harness.state().state.renaming, None);
}

#[test]
fn renaming_a_linked_group_keeps_its_name_whole() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    f.state.doc.edit("add", |m, s| {
        let id = m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(64.0)), "dev/grey").unwrap()));
        s.select_node(id);
    });
    f.state.doc.edit("group", |m, s| ops::group_selection(m, s, "room", layer));
    let copy = f.state.doc.edit("link", |m, s| ops::duplicate_linked(m, s, DVec3::new(128.0, 0.0, 0.0), Default::default()))[0];
    f.state.doc.select(|_, s| {
        s.clear();
        s.select_node(copy);
    });
    gt_editor::commands::execute(&mut f.state, Action::Rename, &egui::Context::default());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 400.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::outliner(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    let field = harness.get_all_by_role(egui::accesskit::Role::TextInput).last().unwrap().value();
    assert_eq!(field.as_deref(), Some("room"), "the field holds the group's own name, not the (linked) note");
    harness.get_all_by_role(egui::accesskit::Role::TextInput).last().unwrap().type_text("hall");
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(harness.state().state.doc.map.get(copy).unwrap().name(), "hall (linked)");
}

#[test]
fn a_pending_rename_stays_with_its_map() {
    let mut f = Fixture::new();
    let add = |doc: &mut gt_doc::Document| {
        let layer = doc.map.default_layer();
        doc.edit("add", |m, s| {
            let id = m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(64.0)), "dev/grey").unwrap()));
            s.select_node(id);
            id
        })
    };
    let brush = add(&mut f.state.doc);
    gt_editor::commands::execute(&mut f.state, Action::Rename, &egui::Context::default());
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 400.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::outliner(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    harness.get_all_by_role(egui::accesskit::Role::TextInput).last().unwrap().type_text("cellar");
    harness.run();

    let mut other = gt_doc::Document::new();
    assert_eq!(add(&mut other), brush, "both maps number their first brush alike");
    harness.state_mut().state.open_tab(other);
    harness.run();
    assert_eq!(harness.state().state.renaming, None);
    assert_eq!(harness.get_all_by_role(egui::accesskit::Role::TextInput).count(), 1, "only the filter is left");
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(harness.state().state.doc.map.get(brush).unwrap().label, None, "the other map's brush kept its name");
    assert_eq!(harness.state().state.tab_doc(0).map.get(brush).unwrap().label, None);
}

#[test]
fn a_rename_field_that_disappears_is_dropped() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let brush = f.state.doc.edit("add", |m, s| {
        let id = m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(64.0)), "dev/grey").unwrap()));
        s.select_node(id);
        id
    });
    gt_editor::commands::execute(&mut f.state, Action::Rename, &egui::Context::default());
    let mut harness = Harness::builder().with_size(egui::vec2(360.0, 400.0)).build_ui_state(
        |ui, (f, shown): &mut (Fixture, bool)| {
            if *shown {
                panels::outliner(ui, &mut f.state, &mut f.panels, &mut f.actions);
            }
        },
        (f, true),
    );
    harness.run();
    harness.get_all_by_role(egui::accesskit::Role::TextInput).last().unwrap().type_text("cellar");
    harness.run();

    // Like switching to the History tab and back.
    harness.state_mut().1 = false;
    harness.run();
    harness.state_mut().1 = true;
    harness.run();
    assert_eq!(harness.state().0.state.renaming, None);
    assert_eq!(harness.get_all_by_role(egui::accesskit::Role::TextInput).count(), 1, "only the filter is left");
    assert_eq!(harness.state().0.state.doc.map.get(brush).unwrap().label, None);
}

#[test]
fn a_click_in_another_field_commits_the_rename() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let brushes: Vec<_> = [0.0, 128.0]
        .into_iter()
        .map(|x| {
            let b = Brush::from_aabb(&Aabb::new(DVec3::new(x, 0.0, 0.0), DVec3::new(x + 64.0, 64.0, 64.0)), "dev/grey").unwrap();
            f.state.doc.edit("add", |m, _| m.insert(layer, NodeKind::Brush(b)))
        })
        .collect();
    // The toolbar and the tool options bar are drawn before the Outliner, like this field.
    let mut harness = Harness::builder().with_size(egui::vec2(360.0, 400.0)).build_ui_state(
        |ui, (f, toolbar): &mut (Fixture, String)| {
            ui.text_edit_singleline(toolbar);
            panels::outliner(ui, &mut f.state, &mut f.panels, &mut f.actions);
        },
        (f, String::new()),
    );
    // The Outliner's Filter field, then the toolbar field.
    for (brush, name, field) in [(brushes[0], "cellar", 1), (brushes[1], "vault", 0)] {
        let state = &mut harness.state_mut().0.state;
        state.doc.select(|_, s| {
            s.clear();
            s.select_node(brush);
        });
        gt_editor::commands::execute(state, Action::Rename, &egui::Context::default());
        harness.run();
        harness.get_all_by_role(egui::accesskit::Role::TextInput).last().unwrap().type_text(name);
        harness.run();
        harness.get_all_by_role(egui::accesskit::Role::TextInput).nth(field).unwrap().click();
        harness.run();
        assert_eq!(harness.state().0.state.doc.map.get(brush).unwrap().label.as_deref(), Some(name), "the typed name is kept");
        assert_eq!(harness.state().0.state.renaming, None);
    }
}

#[test]
fn the_layer_menu_renames_in_the_row_and_keeps_a_name() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.add_layer("Upper");
    let mut harness = Harness::builder()
        .with_size(egui::vec2(360.0, 400.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::outliner(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    // A blank name leaves the layer's name alone.
    for (typed, name) in [("   ", "Upper"), ("Upper floor", "Upper floor")] {
        harness.get_by_label("Upper").click_secondary();
        harness.run();
        harness.get_by_label("Rename").click();
        harness.run();
        let field = harness.get_all_by_role(egui::accesskit::Role::TextInput).last().unwrap();
        assert_eq!(field.value().as_deref(), Some("Upper"), "the row's own name field opens");
        field.type_text(typed);
        harness.run();
        harness.key_press(egui::Key::Enter);
        harness.run();
        assert_eq!(harness.state().state.doc.map.get(layer).unwrap().name(), name);
    }

    harness.get_by_label("Upper floor");
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
    harness.get_by_label(&format!("brush{}", brush.0)).click_secondary();
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
fn the_material_grid_stays_put_when_the_first_material_is_applied() {
    let mut f = Fixture::new();
    f.state.materials.entries = vec![gt_editor::materials::MaterialEntry {
        name: "bricks/red".into(),
        folder: "bricks".into(),
        path: None,
        material_file: None,
        has_normal: false,
        missing_albedo: false,
        is_pbr: false,
        is_emissive: false,
    }];
    let mut harness = Harness::builder()
        .with_size(egui::vec2(700.0, 400.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::material_browser(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    // The grid's cells, 8 wider and 22 taller than the thumbnail size.
    let cell = |h: &Harness<'_, Fixture>| {
        let size = egui::vec2(72.0 + 8.0, 72.0 + 22.0);
        let is_cell =
            move |n: &egui_kittest::kittest::AccessKitNode<'_>| n.bounding_box().is_some_and(|b| (b.width() as f32, b.height() as f32) == (size.x, size.y));
        h.query_all(egui_kittest::kittest::By::new().predicate(is_cell)).map(|n| n.rect()).next().expect("a grid cell")
    };
    harness.run_steps(2);
    let before = cell(&harness);
    harness.get_by_label("the materials you apply show up here");
    harness.state_mut().state.prefs.recent_materials = vec!["bricks/red".into()];
    harness.run_steps(2);
    assert_eq!(cell(&harness), before, "the recent row was there already");
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

fn logic_graph(f: Fixture) -> Harness<'static, Fixture> {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1000.0, 640.0))
        .build_ui_state(|ui, f: &mut Fixture| gt_editor::logic_graph::show(ui, &mut f.state, &mut f.panels.logic, &mut f.actions), f);
    harness.run();
    harness
}

fn named(f: &mut Fixture, classname: &str, name: &str, at: DVec3) -> gt_core::NodeId {
    let layer = f.state.doc.map.default_layer();
    f.state.doc.edit("add", |m, _| {
        let id = ops::create_point_entity(m, layer, classname, at);
        if !name.is_empty() {
            m.entity_mut(id).unwrap().properties.insert("targetname".into(), name.into());
        }

        id
    })
}

/// Adds `output -> target.input` to an entity.
fn wire(f: &mut Fixture, from: gt_core::NodeId, output: &str, target: &str, input: &str) {
    f.state.doc.edit("wire", |m, _| {
        m.entity_mut(from).unwrap().outputs.push(gt_doc::IoConnection {
            output: output.into(),
            target: target.into(),
            input: input.into(),
            parameter: String::new(),
            delay: 0.0,
            times: -1,
        });
    });
}

/// A right click with the pointer at `pos`.
fn right_click_at(harness: &mut Harness<'_, Fixture>, pos: egui::Pos2) {
    harness.hover_at(pos);
    harness.run();
    for pressed in [true, false] {
        harness.event(egui::Event::PointerButton { pos, button: egui::PointerButton::Secondary, pressed, modifiers: egui::Modifiers::NONE });
    }

    harness.run();
}

/// Drags with the primary button through the given points, a frame for each.
fn drag_through(harness: &mut Harness<'_, Fixture>, points: &[egui::Pos2]) {
    harness.hover_at(points[0]);
    harness.run();
    harness.drag_at(points[0]);
    harness.run();
    for p in &points[1..] {
        harness.hover_at(*p);
        harness.run();
    }

    harness.drop_at(*points.last().unwrap());
    harness.run();
}

#[test]
fn logic_graph_simulates_and_flags_broken_links() {
    let mut f = Fixture::new();
    let relay = named(&mut f, "logic_relay", "aaa", DVec3::ZERO);
    named(&mut f, "light", "light1", DVec3::new(64.0, 0.0, 0.0));
    f.state.doc.edit("wire", |m, s| {
        m.entity_mut(relay).unwrap().outputs = vec![
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
        s.select_node(relay);
    });

    let mut harness = logic_graph(f);
    let model = &harness.state().panels.logic.model;
    assert_eq!(model.nodes.len(), 3, "the relay, the light and a marker for the missing target");
    harness.get_by_label("Simulate").click();
    harness.run();
    harness.get_by_label("triggered").click();
    harness.run();
    assert!(harness.query_by_label("2 steps").is_some(), "both wired outputs are listed");
    assert!(harness.query_by_label("1 broken").is_some(), "the missing target is flagged");
}

#[test]
fn logic_graph_simulate_marks_the_steps_that_only_might_happen() {
    let mut f = Fixture::new();
    let button = named(&mut f, "func_button", "btn", DVec3::ZERO);
    let counter = named(&mut f, "logic_counter", "count", DVec3::new(64.0, 0.0, 0.0));
    named(&mut f, "light", "lamp", DVec3::new(128.0, 0.0, 0.0));
    wire(&mut f, button, "pressed", "count", "add");
    wire(&mut f, counter, "hit_max", "lamp", "turn_on");
    f.state.doc.select(|_, s| s.select_node(button));

    let mut harness = logic_graph(f);
    harness.get_by_label("Simulate").click();
    harness.run();
    harness.get_by_label("pressed").click();
    harness.run();
    harness.get_by_label("1. btn.pressed -> count.add");
    harness.get_by_label("2. count.hit_max -> lamp.turn_on  (maybe)");
    assert!(harness.query_by_label_contains("1. btn.pressed -> count.add  (maybe)").is_none(), "the press itself always happens");
}

/// A Logic graph that gets its keys first, the way the app hands them out.
fn logic_graph_with_keys(f: Fixture) -> Harness<'static, Fixture> {
    let mut harness = Harness::builder().with_size(egui::vec2(1000.0, 640.0)).build_ui_state(
        |ui, f: &mut Fixture| {
            f.panels.logic.take_keys(ui.ctx(), &mut f.state);
            gt_editor::logic_graph::show(ui, &mut f.state, &mut f.panels.logic, &mut f.actions);
        },
        f,
    );
    harness.run();
    harness
}

#[test]
fn logic_graph_f_frames_the_selection_and_home_shows_everything() {
    use gt_editor::logic_graph::model::NodeKey;

    let mut f = Fixture::new();
    let near = named(&mut f, "logic_relay", "near", DVec3::ZERO);
    let far = named(&mut f, "logic_relay", "far", DVec3::new(64.0, 0.0, 0.0));
    f.state.doc.edit("place", |m, s| {
        m.get_mut(near).unwrap().set_graph(Some([0.0, 0.0]));
        m.get_mut(far).unwrap().set_graph(Some([4000.0, 2000.0]));
        s.select_node(far);
    });
    let mut harness = logic_graph_with_keys(f);
    let rect = |h: &Harness<'_, Fixture>, id| h.state().panels.logic.node_screen_rect(&NodeKey::Entity(id)).unwrap();
    let canvas = harness.state().panels.logic.canvas_rect();
    harness.hover_at(canvas.center());
    harness.run();

    harness.key_press(egui::Key::F);
    harness.run();
    assert!(canvas.contains_rect(rect(&harness, far)), "the selected node is in view: {:?} in {canvas:?}", rect(&harness, far));
    assert!((rect(&harness, far).center() - canvas.center()).length() < 40.0, "and in the middle");
    assert!(!canvas.contains_rect(rect(&harness, near)), "the other one is out of view");

    harness.key_press(egui::Key::Home);
    harness.run();
    assert!(canvas.contains_rect(rect(&harness, near)) && canvas.contains_rect(rect(&harness, far)), "Home shows every node");
    assert!(harness.state().state.doc.selection.nodes.contains(&far), "the selection is not touched");
}

#[test]
fn logic_graph_hovering_a_wire_tells_its_ends_and_delay() {
    use gt_editor::logic_graph::model::NodeKey;

    let mut f = Fixture::new();
    let relay = named(&mut f, "logic_relay", "r", DVec3::ZERO);
    named(&mut f, "func_door", "gate", DVec3::new(64.0, 0.0, 0.0));
    wire(&mut f, relay, "triggered", "gate", "open");
    f.state.doc.edit("delay", |m, _| m.entity_mut(relay).unwrap().outputs[0].delay = 2.0);
    let mut harness = logic_graph(f);
    let id = (relay, 0);
    let at = harness.state().panels.logic.wire_pos(id, 0.3).unwrap();
    assert!(harness.state().panels.logic.model.node(&NodeKey::Entity(relay)).is_some());
    harness.hover_at(at);
    for _ in 0..8 {
        harness.step();
    }

    for line in ["output: triggered", "target: gate", "input: open", "delay: 2 s"] {
        assert!(harness.query_by_label_contains(line).is_some(), "the tooltip says {line}");
    }
}

#[test]
fn logic_graph_right_click_on_the_wire_line_opens_the_wire_menu() {
    let mut f = Fixture::new();
    let relay = named(&mut f, "logic_relay", "r", DVec3::ZERO);
    named(&mut f, "func_door", "gate", DVec3::new(64.0, 0.0, 0.0));
    wire(&mut f, relay, "triggered", "gate", "open");
    let mut harness = logic_graph(f);
    for t in [0.25, 0.5, 0.75] {
        let at = harness.state().panels.logic.wire_pos((relay, 0), t).unwrap();
        right_click_at(&mut harness, at);
        harness.get_by_label("Edit Connection");
        harness.get_by_label("Delete Connection");
        harness.key_press(egui::Key::Escape);
        harness.run();
        assert!(harness.query_by_label("Edit Connection").is_none(), "the menu closes");
    }
}

#[test]
fn logic_graph_node_menu_renames_duplicates_selects_disconnects_and_deletes() {
    use gt_editor::logic_graph::model::NodeKey;

    let mut f = Fixture::new();
    let button = named(&mut f, "func_button", "btn", DVec3::ZERO);
    let counter = named(&mut f, "logic_counter", "count", DVec3::new(64.0, 0.0, 0.0));
    let lamp = named(&mut f, "light", "lamp", DVec3::new(128.0, 0.0, 0.0));
    let other = named(&mut f, "light", "other", DVec3::new(192.0, 0.0, 0.0));
    wire(&mut f, button, "pressed", "count", "add");
    wire(&mut f, counter, "hit_max", "lamp", "turn_on");
    wire(&mut f, other, "switched", "count", "reset");
    let mut harness = logic_graph(f);
    let node = |h: &Harness<'_, Fixture>| {
        let r = h.state().panels.logic.node_screen_rect(&NodeKey::Entity(counter)).unwrap();
        r.center_top() + egui::vec2(0.0, 10.0)
    };
    let menu = |harness: &mut Harness<'_, Fixture>| {
        let at = node(harness);
        right_click_at(harness, at);
    };

    menu(&mut harness);
    for label in ["Frame in Views", "Rename", "Duplicate", "Select Connected", "Disconnect All", "Delete Entity"] {
        harness.get_by_label(label);
    }

    harness.get_by_label("Select Connected").click();
    harness.run();
    let selected: Vec<_> = harness.state().state.doc.selection.nodes.iter().copied().collect();
    assert_eq!(selected.len(), 4, "the counter and the three nodes wired to it: {selected:?}");
    assert!([button, counter, lamp, other].iter().all(|id| selected.contains(id)));

    menu(&mut harness);
    harness.get_by_label("Delete 4 Entities");
    harness.key_press(egui::Key::Escape);
    harness.run();
    harness.state_mut().state.doc.select(|_, s| s.clear());
    harness.run();

    menu(&mut harness);
    harness.get_by_label("Rename").click();
    harness.run();
    assert!(harness.state().actions.contains(&Action::Rename));
    assert_eq!(harness.state().state.doc.selection.nodes.iter().copied().collect::<Vec<_>>(), [counter]);

    harness.state_mut().actions.clear();
    menu(&mut harness);
    harness.get_by_label("Delete Entity").click();
    harness.run();
    assert!(harness.state().actions.contains(&Action::Delete), "the menu queues the editor's own Delete, which removes the entity");

    harness.state_mut().actions.clear();
    menu(&mut harness);
    harness.get_by_label("Duplicate").click();
    harness.run();
    assert!(harness.state().actions.contains(&Action::Duplicate));

    let steps = harness.state().state.doc.history.undo_labels().count();
    menu(&mut harness);
    harness.get_by_label("Disconnect All").click();
    harness.run();
    let map = &harness.state().state.doc.map;
    assert!(map.entity(button).unwrap().outputs.is_empty() && map.entity(counter).unwrap().outputs.is_empty() && map.entity(other).unwrap().outputs.is_empty());
    assert_eq!(harness.state().state.doc.history.undo_labels().count(), steps + 1, "one undo step for every wire");
    assert_eq!(map.entity_count(), 4, "the entities stay");
}

#[test]
fn logic_graph_drag_from_an_output_pin_to_an_input_pin_connects() {
    use gt_editor::logic_graph::model::NodeKey;

    let mut f = Fixture::new();
    let relay = named(&mut f, "logic_relay", "r", DVec3::ZERO);
    let door = named(&mut f, "func_door", "", DVec3::new(128.0, 0.0, 0.0));
    named(&mut f, "func_door", "door_1", DVec3::new(256.0, 0.0, 0.0));
    f.state.doc.select(|_, s| s.select_node(door));
    let mut harness = logic_graph(f);

    let logic = &harness.state().panels.logic;
    let from = logic.pin_pos(&NodeKey::Entity(relay), true, "triggered").expect("the relay shows, it is a logic entity");
    let to = logic.pin_pos(&NodeKey::Entity(door), false, "open").expect("the selected door shows");
    let rects = |h: &Harness<'_, Fixture>| [relay, door].map(|id| h.state().panels.logic.node_screen_rect(&NodeKey::Entity(id)).unwrap());
    let before = rects(&harness);
    let steps = harness.state().state.doc.history.undo_labels().count();
    drag_through(&mut harness, &[from, from + egui::vec2(30.0, 10.0), (from + to.to_vec2()) / 2.0, to]);

    let state = &harness.state().state;
    let e = state.doc.map.entity(relay).unwrap();
    assert_eq!(e.outputs.len(), 1, "one connection");
    assert_eq!((e.outputs[0].output.as_str(), e.outputs[0].target.as_str(), e.outputs[0].input.as_str()), ("triggered", "door_2", "open"));
    assert_eq!(state.doc.map.entity(door).unwrap().targetname(), Some("door_2"), "the unnamed door got a free name");
    assert_eq!(state.doc.history.undo_labels().count(), steps + 1);
    assert_eq!(harness.state().panels.logic.model.edges.len(), 1, "the graph follows the edit");
    assert_eq!(rects(&harness), before, "the nodes stay where they were instead of being laid out again");

    // A click on the wire opens its editor, whose Delete removes it again.
    let logic = &harness.state().panels.logic;
    let a = logic.pin_pos(&NodeKey::Entity(relay), true, "triggered").unwrap();
    let b = logic.pin_pos(&NodeKey::Entity(door), false, "open").unwrap();
    harness.hover_at((a + b.to_vec2()) / 2.0);
    harness.run();
    harness.drag_at((a + b.to_vec2()) / 2.0);
    harness.run();
    harness.drop_at((a + b.to_vec2()) / 2.0);
    harness.run();
    assert_eq!(harness.state().panels.logic.selected_edge, Some((relay, 0)));
    harness.get_by_label("Delete").click();
    harness.run();
    assert!(harness.state().state.doc.map.entity(relay).unwrap().outputs.is_empty());
}

#[test]
fn logic_graph_nodes_with_settings_keep_their_pins_under_the_pointer() {
    use gt_editor::logic_graph::model::NodeKey;

    let mut f = Fixture::new();
    let counter = named(&mut f, "logic_counter", "count", DVec3::ZERO);
    f.state.doc.edit("set", |m, _| {
        let props = &mut m.entity_mut(counter).unwrap().properties;
        props.insert("max".into(), "5".into());
        props.insert("min".into(), "-2".into());
    });
    let lamp = named(&mut f, "light", "lamp", DVec3::new(128.0, 0.0, 0.0));
    wire(&mut f, counter, "changed", "lamp", "toggle");
    let mut harness = logic_graph(f);

    let logic = &harness.state().panels.logic;
    let node = logic.model.node(&NodeKey::Entity(counter)).unwrap();
    assert_eq!(node.settings, ["min -2 \u{b7} max 5"]);
    let from = logic.pin_pos(&NodeKey::Entity(counter), true, "hit_max").unwrap();
    let to = logic.pin_pos(&NodeKey::Entity(lamp), false, "turn_on").unwrap();
    let top = logic.node_screen_rect(&NodeKey::Entity(counter)).unwrap().min.y;
    assert!(from.y - top > gt_editor::logic_graph::model::HEADER * 0.5, "the pins sit below the title and the settings");
    drag_through(&mut harness, &[from, from + egui::vec2(30.0, 10.0), (from + to.to_vec2()) / 2.0, to]);
    let outputs = &harness.state().state.doc.map.entity(counter).unwrap().outputs;
    assert!(outputs.iter().any(|o| (o.output.as_str(), o.input.as_str()) == ("hit_max", "turn_on")), "the wire starts at the pin that was drawn: {outputs:?}");
}

/// The classes the add search created, besides the ones the test started with.
fn added_classes(harness: &Harness<'_, Fixture>, before: usize) -> Vec<String> {
    let mut all: Vec<(u64, String)> = harness.state().state.doc.map.entities().map(|(id, e)| (id.0, e.classname.clone())).collect();
    all.sort();
    all.into_iter().skip(before).map(|(_, c)| c).collect()
}

#[test]
fn logic_graph_search_ranks_names_first_and_arrows_move_the_choice() {
    let open = || {
        let mut f = Fixture::new();
        named(&mut f, "logic_relay", "r", DVec3::ZERO);
        let mut harness = logic_graph(f);
        right_click_at(&mut harness, egui::pos2(600.0, 420.0));
        harness.get_by_role(egui::accesskit::Role::TextInput).type_text("tim");
        harness.run();
        harness
    };

    let mut harness = open();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(added_classes(&harness, 1), ["logic_timer"], "the class named like the text comes before those that only describe it");

    let mut harness = open();
    harness.key_press(egui::Key::ArrowDown);
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    let added = added_classes(&harness, 1);
    assert_eq!(added.len(), 1);
    assert_ne!(added[0], "logic_timer", "one arrow down moves to the next row, whose description mentions time");

    let mut harness = open();
    harness.key_press(egui::Key::ArrowDown);
    harness.key_press(egui::Key::ArrowUp);
    harness.run();
    harness.key_press(egui::Key::Enter);
    harness.run();
    assert_eq!(added_classes(&harness, 1), ["logic_timer"], "and back up");
}

#[test]
fn logic_graph_right_click_list_is_as_tall_as_the_add_menu_allows() {
    let mut f = Fixture::new();
    named(&mut f, "logic_relay", "r", DVec3::ZERO);
    let mut harness = logic_graph(f);
    let menu_height = |h: &Harness<'_, Fixture>| {
        h.ctx.memory(|m| {
            let rects = m.areas().visible_layer_ids().into_iter().filter(|l| l.order == egui::Order::Foreground).filter_map(|l| m.area_rect(l.id));
            rects.map(|r| r.height()).fold(0.0, f32::max)
        })
    };
    right_click_at(&mut harness, egui::pos2(300.0, 600.0));
    let first = menu_height(&harness);
    assert!(first > 250.0, "many rows show, not four: {first}");

    // A short list, such as after a search, must not size the next one.
    harness.get_by_role(egui::accesskit::Role::TextInput).type_text("relay");
    harness.run();
    assert!(menu_height(&harness) < first / 2.0, "the search narrows the list: {}", menu_height(&harness));
    harness.key_press(egui::Key::Escape);
    harness.run();
    right_click_at(&mut harness, egui::pos2(300.0, 200.0));
    assert!((menu_height(&harness) - first).abs() < 2.0, "the next menu shows the whole list again: {} against {first}", menu_height(&harness));
}

#[test]
fn logic_graph_knows_when_the_pointer_is_over_it() {
    let mut f = Fixture::new();
    named(&mut f, "logic_relay", "r", DVec3::ZERO);
    let mut harness = logic_graph(f);
    harness.hover_at(egui::pos2(600.0, 400.0));
    harness.run();
    assert!(harness.state().panels.logic.hovered(), "Ctrl+V there pastes beside the copy, no view is under the pointer");
    harness.hover_at(egui::pos2(900.0, 5.0));
    harness.run();
    assert!(!harness.state().panels.logic.hovered(), "over the toolbar the keys belong to the editor");
}

#[test]
fn logic_graph_add_puts_a_new_node_in_free_space() {
    use gt_editor::logic_graph::model::NodeKey;

    let mut f = Fixture::new();
    let relay = named(&mut f, "logic_relay", "r", DVec3::ZERO);
    let mut harness = logic_graph(f);
    harness.get_by_label("Add").click();
    harness.run();
    harness.get_by_label("logic_counter").click();
    harness.run();
    let map = &harness.state().state.doc.map;
    let (counter, _) = map.entities().find(|(_, e)| e.classname == "logic_counter").expect("a counter was added");
    let logic = &harness.state().panels.logic;
    let (a, b) = (logic.node_screen_rect(&NodeKey::Entity(relay)).unwrap(), logic.node_screen_rect(&NodeKey::Entity(counter)).unwrap());
    assert!(!a.intersects(b), "{a:?} and {b:?} overlap");
}

#[test]
fn logic_graph_frames_name_and_move_their_nodes() {
    use gt_editor::logic_graph::model::NodeKey;

    let mut f = Fixture::new();
    let relay = named(&mut f, "logic_relay", "r", DVec3::ZERO);
    let outside = named(&mut f, "logic_timer", "t", DVec3::ZERO);
    f.state.doc.select(|_, s| s.select_node(relay));
    let mut harness = logic_graph(f);
    harness.get_by_label("Frame").click();
    harness.run();
    harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    harness.event(egui::Event::Text("Doors".into()));
    harness.key_press(egui::Key::Enter);
    harness.run();
    let frames = &harness.state().state.doc.map.editor.graph_frames;
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].title, "Doors", "the new frame's title field has the keyboard");

    let node = |h: &Harness<'_, Fixture>, id| h.state().panels.logic.node_screen_rect(&NodeKey::Entity(id)).unwrap();
    let (relay_before, timer_before) = (node(&harness, relay), node(&harness, outside));
    let title = harness.state().panels.logic.frame_screen_rect(0).unwrap().left_top() + egui::vec2(60.0, 8.0);
    drag_through(&mut harness, &[title, title + egui::vec2(20.0, 10.0), title + egui::vec2(80.0, 40.0)]);
    assert_eq!(node(&harness, relay).min, relay_before.min + egui::vec2(80.0, 40.0), "the node inside moves with the frame");
    assert_eq!(node(&harness, outside), timer_before, "a node outside stays");
    assert_eq!(harness.state().state.doc.history.undo_labels().next(), Some("Move Frame"));

    let title = harness.state().panels.logic.frame_screen_rect(0).unwrap().left_top() + egui::vec2(60.0, 8.0);
    harness.hover_at(title);
    harness.run();
    for pressed in [true, false] {
        harness.event(egui::Event::PointerButton { pos: title, button: egui::PointerButton::Secondary, pressed, modifiers: Default::default() });
    }

    harness.run();
    harness.get_by_label("Delete Frame").click();
    harness.run();
    assert!(harness.state().state.doc.map.editor.graph_frames.is_empty());
    assert!(harness.state().state.doc.map.entity(relay).is_some(), "deleting a frame keeps its nodes");
}

#[test]
fn logic_graph_drop_on_empty_space_adds_a_wired_logic_entity() {
    use gt_editor::logic_graph::model::NodeKey;

    let mut f = Fixture::new();
    let relay = named(&mut f, "logic_relay", "r", DVec3::new(32.0, 16.0, 0.0));
    let mut harness = logic_graph(f);
    let from = harness.state().panels.logic.pin_pos(&NodeKey::Entity(relay), true, "triggered").unwrap();
    let empty = from + egui::vec2(260.0, 120.0);
    drag_through(&mut harness, &[from, from + egui::vec2(40.0, 20.0), empty]);
    harness.get_by_label("logic_timer").click();
    harness.run();

    let map = &harness.state().state.doc.map;
    let (timer, e) =
        map.entities().find(|(_, e)| e.classname == "logic_timer").unwrap_or_else(|| panic!("a timer was added: {}", harness.state().state.status));
    assert_eq!(e.targetname(), Some("timer_1"));
    assert_eq!(map.entity(relay).unwrap().outputs[0].target, "timer_1");
    assert_eq!(map.entity(relay).unwrap().outputs[0].input, "start");
    assert!(matches!(&map.get(map.layer_of(timer)).unwrap().kind, NodeKind::Layer(l) if l.name == "Logic"));
    assert!(map.get(timer).unwrap().graph.is_some() && map.get(relay).unwrap().graph.is_some(), "the new node and the laid out one keep their places");
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

#[test]
fn model_menu_offers_to_show_the_file() {
    use gt_editor::models::{ModelEntry, PackInfo};

    let mut f = Fixture::new();
    let pack = std::sync::Arc::new(PackInfo { name: "Pack".into(), ..Default::default() });
    f.state.model_library.entries = vec![ModelEntry {
        name: "crate".into(),
        folder: String::new(),
        path: "/models/crate.glb".into(),
        ext: "glb".into(),
        source: pack,
        credit: None,
        mtime: None,
    }];
    let mut harness = Harness::builder()
        .with_size(egui::vec2(700.0, 500.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::model_browser(ui, &mut f.state, &mut f.panels, &mut f.actions, None), f);
    harness.run_steps(2);
    harness.get_by_label("crate").click_secondary();
    harness.run_steps(2);
    harness.get_by_label("Show in File Manager");
}

#[test]
fn file_uris_escape_what_a_uri_or_d_bus_would_misread() {
    assert_eq!(panels::file_uri(std::path::Path::new("/home/a b/tex,1'.png")), "file:///home/a%20b/tex%2C1%27.png");
    assert_eq!(panels::file_uri(std::path::Path::new("C:\\maps\\x.png")), "file:///C:/maps/x.png");
}

#[test]
fn a_file_manager_that_starts_slowly_is_not_opened_twice() {
    assert!(panels::file_manager_shown(true, ""));
    assert!(panels::file_manager_shown(false, "Error org.freedesktop.DBus.Error.NoReply: Did not receive a reply."));
    assert!(panels::file_manager_shown(false, "Error: Timeout was reached"));
    assert!(!panels::file_manager_shown(false, "Error org.freedesktop.DBus.Error.ServiceUnknown: The name is not activatable"));
    assert!(!panels::file_manager_shown(false, "Failed to open connection to \"session\" message bus"));
}

#[test]
fn entity_browser_offers_the_gameplay_pack_while_only_the_core_is_known() {
    let mut core = Fixture::new();
    core.state.game = gt_formats::GameConfig::builtin();
    let mut harness = Harness::builder()
        .with_size(egui::vec2(700.0, 900.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::entity_browser(ui, &mut f.state, &mut f.panels, &mut f.actions), core);
    harness.run();
    assert!(harness.query_by_label("func_door").is_none() && harness.query_by_label("info_player_start").is_some());
    harness.get_by_label("Install Gameplay Entities").click();
    harness.run();
    assert_eq!(harness.state().actions, [Action::InstallGameplayEntities]);

    let mut harness = Harness::builder()
        .with_size(egui::vec2(700.0, 900.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::entity_browser(ui, &mut f.state, &mut f.panels, &mut f.actions), Fixture::new());
    harness.run();
    assert!(harness.query_by_label("Install Gameplay Entities").is_none(), "a project with the pack is not asked");
}

#[test]
fn entity_browser_headings_say_what_their_entities_are_for() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(700.0, 1400.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::entity_browser(ui, &mut f.state, &mut f.panels, &mut f.actions), Fixture::new());
    harness.run();
    harness.get_by_label("trigger").hover();
    harness.run_steps(60);
    harness.get_by_label("Invisible volumes that fire outputs when a body enters or leaves them");
}

#[test]
fn mesh_inspector_names_the_materials_a_placed_model_draws_with() {
    let mut f = Fixture::new();
    let layer = f.state.doc.map.default_layer();
    let mut mesh = gt_geom::mesh_shapes::cuboid(&Aabb::new(DVec3::ZERO, DVec3::splat(64.0)), "models/nature/pine/tex0");
    mesh.faces[0].data.material = "models/nature/pine/tex1".into();
    let id = f.state.doc.edit("add", |m, s| {
        let id = m.insert(layer, NodeKind::Mesh(mesh));
        s.select_node(id);
        id
    });
    let mut harness = Harness::builder()
        .with_size(egui::vec2(520.0, 700.0))
        .build_ui_state(|ui, f: &mut Fixture| panels::inspector(ui, &mut f.state, &mut f.panels, &mut f.actions), f);
    harness.run();
    harness.get_by_label("Materials: models/nature/pine/tex0, models/nature/pine/tex1");
    // A material clicked in the Materials panel while the model is still selected replaces its own, and says so.
    let state = &mut harness.state_mut().state;
    state.doc.edit("Apply Material", |m, s| ops::apply_material(m, s, "dev/grey"));
    harness.run();
    harness.get_by_label("Materials: dev/grey");

    harness.state_mut().state.doc.edit("Delete Faces", |m, _| m.mesh_mut(id).unwrap().faces.clear());
    harness.run();
    assert!(harness.query_by_label_contains("Materials:").is_none(), "a mesh without faces lists none");
}
