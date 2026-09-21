//! The addon's FGD resources are generated from the built-in entity library. This fails when either side changed alone;
//! regenerate with `cargo run -p gt_editor --example export_addon_fgd`.

#[test]
fn addon_fgd_matches_builtin_entities() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../godot/addons/func_godot/fgd/godottrench");
    for (name, text) in gt_editor::code_refs::addon_fgd_files() {
        let on_disk = std::fs::read_to_string(dir.join(&name)).unwrap_or_default().replace("\r\n", "\n");
        assert_eq!(on_disk, text, "{name} is out of date, run the export_addon_fgd example");
    }

    let cfg = gt_formats::GameConfig::builtin();
    for class in gt_editor::code_refs::ADDON_ENTITIES {
        let def = cfg.entity(class).unwrap_or_else(|| panic!("{class} missing from builtin_entities.json"));
        if !def.script.is_empty() {
            let script = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../godot").join(def.script.trim_start_matches("res://"));
            assert!(script.is_file(), "{class} script {} exists", script.display());
        }
    }
}
