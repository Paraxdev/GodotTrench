//! Demo assets for the Godot project: pixel art textures with material overrides and Blockbench models.
//! The showcase maps themselves are MCP scripts in examples/mcp, replayed inside the editor.

pub mod models;
pub mod textures;

use std::path::Path;

/// Writes textures, material overrides and models into a Godot project folder.
pub fn write_all(godot: &Path) -> std::io::Result<Vec<String>> {
    let mut written = Vec::new();
    let textures = godot.join("demo/textures");
    for (name, img) in textures::all() {
        let path = textures.join(format!("{name}.png"));
        std::fs::create_dir_all(path.parent().unwrap())?;
        img.save(&path).map_err(std::io::Error::other)?;
        written.push(path.display().to_string());
        if textures::NORMAL_MAPPED.contains(&name) {
            let normal = textures.join(format!("{name}_normal.png"));
            textures::normal_map(&img, 6.0).save(&normal).map_err(std::io::Error::other)?;
            written.push(normal.display().to_string());
        }
    }

    for (name, text) in textures::material_overrides() {
        let path = textures.join(format!("{name}.tres"));
        std::fs::write(&path, text)?;
        written.push(path.display().to_string());
    }

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
    fn every_showcase_material_has_a_texture() {
        let names: std::collections::HashSet<&str> = textures::all().into_iter().map(|(n, _)| n).collect();
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/mcp");
        for entry in std::fs::read_dir(&root).unwrap() {
            let path = entry.unwrap().path();
            let text = std::fs::read_to_string(&path).unwrap();
            for part in text.split('"').filter(|s| s.starts_with("showcase/")) {
                assert!(names.contains(part), "{}: {part} is not generated", path.display());
            }
        }
    }
}
