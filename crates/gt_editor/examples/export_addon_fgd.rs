//! Writes the FuncGodot fork's gameplay entity definitions (addons/func_godot/fgd/godottrench/) from the editor's built-in
//! entity library, so the editor and Godot agree on classnames, properties, inputs, outputs and gizmos.
//! cargo run -p gt_editor --example export_addon_fgd

fn main() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../godot/addons/func_godot/fgd/godottrench");
    std::fs::create_dir_all(&dir).expect("create fgd folder");
    let files = gt_editor::code_refs::addon_fgd_files();
    for (name, text) in &files {
        std::fs::write(dir.join(name), text).expect("write fgd resource");
    }
    println!("wrote {} files to {}", files.len(), dir.display());
}
