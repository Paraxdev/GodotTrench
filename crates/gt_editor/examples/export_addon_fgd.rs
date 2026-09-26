//! Writes the Godot addon's entity definitions from the editor's entity libraries, so the editor and Godot agree on
//! classnames, properties, inputs, outputs and gizmos: the core ones in addons/func_godot/fgd/godottrench/ and the
//! Gameplay entities pack template in addons/func_godot/gameplay_pack/. Then updates the demo project's installed copy
//! of the pack. cargo run -p gt_editor --example export_addon_fgd

use std::path::PathBuf;

fn main() {
    let godot = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../godot");
    let addon = godot.join("addons/func_godot");
    let files = gt_editor::code_refs::addon_fgd_files();
    for (name, text) in &files {
        let path = addon.join(name);
        std::fs::create_dir_all(path.parent().unwrap()).expect("create fgd folder");
        std::fs::write(&path, text).expect("write fgd resource");
    }

    for dir in gt_editor::code_refs::GENERATED_DIRS {
        for entry in std::fs::read_dir(addon.join(dir)).expect("read fgd folder").flatten() {
            let name = format!("{dir}/{}", entry.file_name().to_string_lossy());
            if name.ends_with(".tres") && !files.iter().any(|(n, _)| *n == name) {
                std::fs::remove_file(entry.path()).expect("remove stale fgd resource");
                println!("removed {name}");
            }
        }
    }

    println!("wrote {} files to {}", files.len(), addon.display());
    match gt_editor::entity_pack::install(&godot) {
        Ok(report) => println!("demo project: {}", report.summary()),
        Err(e) => panic!("demo project: {e}"),
    }
}
