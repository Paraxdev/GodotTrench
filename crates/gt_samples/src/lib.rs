//! Demo Blockbench models for the Godot project. The showcase maps themselves are MCP scripts in examples/mcp,
//! replayed inside the editor, and their photo textures come from tools/fetch_demo_textures.py.

pub mod models;
pub mod noise;

use std::path::Path;

/// Writes the demo models into a Godot project folder.
pub fn write_all(godot: &Path) -> std::io::Result<Vec<String>> {
    let mut written = Vec::new();
    let models = godot.join("demo/models");
    std::fs::create_dir_all(&models)?;
    for (file, text) in models::all() {
        std::fs::write(models.join(file), text)?;
        written.push(models.join(file).display().to_string());
    }

    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_core::DVec3;

    #[test]
    fn models_parse_with_the_editor_reader() {
        for (file, text) in models::all() {
            let m = gt_formats::bbmodel::parse(&text).unwrap_or_else(|e| panic!("{file}: {e}"));
            assert!(!m.cubes.is_empty() && !m.textures[0].png.is_empty(), "{file}");
            let mesh = m.to_mesh(2.0, DVec3::ZERO, |_| "t".into());
            mesh.validate().unwrap();
        }
    }

    #[test]
    fn every_showcase_material_has_a_texture_with_a_world_size() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let textures = root.join("godot/demo/textures");
        for entry in std::fs::read_dir(root.join("examples/mcp")).unwrap() {
            let path = entry.unwrap().path();
            let text = std::fs::read_to_string(&path).unwrap();
            for part in text.split('"').filter(|s| s.starts_with("showcase/") || s.starts_with("withered/")) {
                let tres = std::fs::read_to_string(textures.join(format!("{part}.tres")))
                    .unwrap_or_else(|_| panic!("{}: {part} has no material, run tools/fetch_demo_textures.py", path.display()));
                let material = gt_formats::godot_material::parse(&tres).unwrap_or_else(|| panic!("{part}.tres is not a material"));
                let albedo = material.albedo_texture.unwrap_or_else(|| panic!("{part}.tres has no albedo"));
                assert!(root.join("godot").join(albedo.trim_start_matches("res://")).is_file(), "{part}: {albedo} is missing");
                assert!(material.texture_size.is_some(), "{part}.tres needs metadata/texture_size, its photo would tile by pixels");
            }
        }
    }
}
