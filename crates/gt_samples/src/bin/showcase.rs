//! Regenerates the Blockbench models in the Godot demo project. The maps come from examples/mcp, the textures from
//! tools/fetch_demo_textures.py.
//! cargo run -p gt_samples --bin showcase --release

fn main() {
    let godot = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../godot");
    match gt_samples::write_all(&godot) {
        Ok(files) => println!("wrote {} files under {}", files.len(), godot.display()),
        Err(e) => {
            eprintln!("showcase generation failed: {e}");
            std::process::exit(1);
        }
    }
}
