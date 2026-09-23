//! The fixture is exported by the Godot addon (see godot/addons/func_godot/src/godottrench/export_game_config_cli.gd),
//! so this test guards the JSON contract between the addon and the editor.

use gt_core::DVec3;
use gt_formats::{EntityKind, GameConfig, PropertyType};

fn fixture() -> GameConfig {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/godottrench_game.json");
    GameConfig::load(&path).expect("fixture parses")
}

#[test]
fn parses_exported_config() {
    let cfg = fixture();
    assert_eq!(cfg.name, "GodotTrench Demo");
    assert_eq!(cfg.units_per_meter, 32.0);
    assert_eq!(cfg.textures.base_dir, "res://demo/textures");
    assert_eq!(cfg.tool_textures.clip, "special/clip");
    assert!(cfg.is_tool_texture("SPECIAL/CLIP"));
    assert!(cfg.is_tool_texture("special/sky"), "sky faces build no mesh, like the other tool textures");
}

#[test]
fn entity_definitions_round_trip() {
    let cfg = fixture();
    let door = cfg.entity("func_door").expect("func_door");
    assert_eq!(door.kind, EntityKind::Solid);
    assert_eq!(door.node_class, "AnimatableBody3D");
    assert!(door.outputs.iter().any(|o| o.name == "opened"));
    assert!(door.inputs.iter().any(|i| i.name == "toggle"));
    assert_eq!(door.property("targetname").unwrap().ty, PropertyType::TargetSource);
    assert_eq!(door.property("move_distance").unwrap().ty, PropertyType::Float);

    let start = cfg.entity("info_player_start").expect("info_player_start");
    assert_eq!(start.kind, EntityKind::Point);
    assert_eq!(start.size, [DVec3::new(-16.0, 0.0, -16.0), DVec3::new(16.0, 56.0, 16.0)]);

    let light = cfg.entity("light").unwrap();
    assert_eq!(light.property("light_color").unwrap().ty, PropertyType::Color);
    assert_eq!(light.property("start_on").unwrap().default, "1");

    let trigger = cfg.entity("trigger_area").unwrap();
    assert!(trigger.outputs.iter().any(|o| o.name == "body_entered"));
    assert!(cfg.point_entities().all(|e| e.kind == EntityKind::Point));
}
