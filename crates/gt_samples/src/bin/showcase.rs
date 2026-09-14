//! Regenerates the showcase textures and Blockbench models in the Godot demo project. The maps come from examples/mcp.
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
