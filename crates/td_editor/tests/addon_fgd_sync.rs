//! The addon's FGD resources are generated from the editor's entity libraries, and the demo project has the Gameplay
//! entities pack installed as the addon ships it. This fails when either side changed alone; regenerate with
//! `cargo run -p td_editor --example export_addon_fgd`.

use std::path::{Path, PathBuf};

fn godot() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../godot")
}

fn files(root: &Path, dir: &Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            files(root, &path, out);
        } else {
            out.push(path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
        }
    }
}

#[test]
fn addon_fgd_matches_the_entity_libraries() {
    let addon = godot().join("addons/func_godot");
    let generated = td_editor::code_refs::addon_fgd_files();
    for (name, text) in &generated {
        let on_disk = std::fs::read_to_string(addon.join(name)).unwrap_or_default().replace("\r\n", "\n");
        assert_eq!(on_disk, *text, "{name} is out of date, run the export_addon_fgd example");
    }

    for dir in td_editor::code_refs::GENERATED_DIRS {
        let mut found = Vec::new();
        files(&addon, &addon.join(dir), &mut found);
        for name in found.iter().filter(|n| n.ends_with(".tres")) {
            assert!(generated.iter().any(|(n, _)| n == name), "{name} is left over, run the export_addon_fgd example");
        }
    }

    let template = td_editor::entity_pack::template_dir(&godot());
    assert!(template.join(".gdignore").is_file(), "Godot must not register the template's class_names");
    for def in td_formats::GameConfig::with_gameplay_pack().entities.iter().filter(|d| !d.script.is_empty()) {
        let path = match def.script.strip_prefix(td_editor::entity_pack::RES_DIR) {
            Some(rel) => template.join(rel.trim_start_matches('/')),
            None => godot().join(def.script.trim_start_matches("res://")),
        };
        assert!(path.is_file(), "{} script {} exists", def.classname, def.script);
    }
}

/// The demo project ships with the pack installed, byte for byte what the addon's template holds.
#[test]
fn the_demo_project_has_the_pack_installed() {
    let template = td_editor::entity_pack::template_dir(&godot());
    let installed = td_editor::entity_pack::install_dir(&godot());
    let (mut want, mut have) = (Vec::new(), Vec::new());
    files(&template, &template, &mut want);
    files(&installed, &installed, &mut have);
    want.retain(|n| n != ".gdignore" && n != td_editor::entity_pack::MANIFEST);
    have.retain(|n| !n.ends_with(".uid") && n != td_editor::entity_pack::MANIFEST);
    want.sort();
    have.sort();
    assert_eq!(have, want, "run the export_addon_fgd example");
    for name in &want {
        let a = std::fs::read(template.join(name)).unwrap();
        let b = std::fs::read(installed.join(name)).unwrap();
        assert!(a == b, "{name} differs from the template, change the template and run the export_addon_fgd example");
    }

    assert_eq!(td_editor::entity_pack::Status::check(&godot()).changes, 0, "the demo's record is current");
}

/// A project's worldspawn offers the keys the editor's own worldspawn has, so the Inspector lists the sky, fog and sun
/// inside a project too. The demo's trenchdot_game.json is exported by the addon, run its export_game_config_cli.gd.
#[test]
fn a_project_worldspawn_lists_the_builtin_keys() {
    let project = td_formats::GameConfig::load(&godot().join("trenchdot_game.json")).unwrap();
    let have = project.entity("worldspawn").unwrap();
    let builtin = td_formats::GameConfig::builtin();
    for want in &builtin.entity("worldspawn").unwrap().properties {
        let Some(p) = have.properties.iter().find(|p| p.name == want.name) else { panic!("worldspawn lacks {}", want.name) };
        assert_eq!(p.ty, want.ty, "{} type", want.name);
        assert_eq!(p.options, want.options, "{} options", want.name);
        let same = match (p.default.parse::<f64>(), want.default.parse::<f64>()) {
            (Ok(a), Ok(b)) => a == b,
            _ => p.default == want.default,
        };
        assert!(same, "{} default is {:?}, the editor's is {:?}", want.name, p.default, want.default);
    }
}
