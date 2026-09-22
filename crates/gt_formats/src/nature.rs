//! Built-in nature pack: low poly Blockbench trees, bushes, rocks and foliage with embedded pixel textures.
//! The models in `assets/nature` are authored in Blockbench, the editor installs them into a project so the
//! scatter presets work without any assets.

macro_rules! models {
    ($($name:literal),* $(,)?) => {
        [$(($name, include_str!(concat!("../assets/nature/", $name, ".bbmodel")))),*]
    };
}

/// The first nine are the original pack names that saved maps and presets refer to.
const MODELS: [(&str, &str); 18] = models!(
    "pine",
    "oak",
    "birch",
    "bush",
    "fern",
    "rock",
    "boulder",
    "grass",
    "flowers",
    "bush_berries",
    "bush_round",
    "rock_flat",
    "boulder_mossy",
    "grass_tall",
    "flowers_yellow",
    "stump",
    "log",
    "mushrooms",
);

/// Every model as (name, bbmodel text).
pub fn all() -> Vec<(&'static str, String)> {
    MODELS.iter().map(|(name, text)| (*name, text.to_string())).collect()
}

pub fn names() -> Vec<&'static str> {
    MODELS.iter().map(|(name, _)| *name).collect()
}

/// Writes missing (or all, with `overwrite`) models into `dir`. Returns the files written.
pub fn install(dir: &std::path::Path, overwrite: bool) -> std::io::Result<Vec<std::path::PathBuf>> {
    std::fs::create_dir_all(dir)?;
    let mut out = Vec::new();
    for (name, text) in MODELS {
        let path = dir.join(format!("{name}.bbmodel"));
        if overwrite || !path.exists() {
            std::fs::write(&path, text)?;
            out.push(path);
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_model_parses_with_textures() {
        for (name, text) in all() {
            let model = crate::bbmodel::parse(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
            let png = image::load_from_memory(&model.textures[0].png).unwrap_or_else(|e| panic!("{name} embeds a png: {e}"));
            assert!(png.width() <= 64 && png.height() <= 64, "{name} keeps a small pixel art atlas");
            assert!(model.polygons().count() >= 4, "{name}");
            for p in model.polygons() {
                assert!(p.uvs.iter().all(|uv| (-1e-6..=1.0 + 1e-6).contains(&uv.x) && (-1e-6..=1.0 + 1e-6).contains(&uv.y)), "{name} uv inside the atlas");
            }

            let b = model.bounds();
            assert!(b.size().y > 4.0 && b.min.y >= -8.0 && b.min.y < 1.0, "{name} stands on the ground: {b:?}");
        }
    }

    #[test]
    fn original_names_stay_and_sizes_match_the_presets() {
        let names = names();
        for keep in ["pine", "oak", "birch", "bush", "fern", "rock", "boulder", "grass", "flowers"] {
            assert!(names.contains(&keep), "{keep}");
        }

        // 16 units per meter: trees are several meters tall, ground cover stays below knee height.
        let height = |n: &str| crate::bbmodel::parse(MODELS.iter().find(|(m, _)| *m == n).unwrap().1).unwrap().bounds().size().y / 16.0;
        assert!((9.0..12.5).contains(&height("pine")), "{}", height("pine"));
        assert!((6.0..10.0).contains(&height("oak")));
        assert!(height("boulder") > height("rock") * 1.5);
        assert!(height("grass") < 1.0 && height("mushrooms") < 1.0);
    }

    #[test]
    fn every_bbmodel_preset_item_is_shipped() {
        let prefix = format!("{}/", gt_doc::scatter::NATURE_DIR);
        for preset in gt_doc::scatter::PRESETS {
            let (_, items) = gt_doc::scatter::preset(preset).unwrap();
            for item in items.iter().filter(|i| i.source.ends_with(".bbmodel")) {
                let name = item.source.strip_prefix(&prefix).and_then(|s| s.strip_suffix(".bbmodel")).unwrap_or(&item.source);
                assert!(names().contains(&name), "{preset}: {}", item.source);
            }
        }
    }

    #[test]
    fn install_writes_only_missing_files_unless_overwriting() {
        let dir = std::env::temp_dir().join(format!("gt_nature_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(install(&dir, false).unwrap().len(), MODELS.len());
        assert!(install(&dir, false).unwrap().is_empty());
        assert_eq!(install(&dir, true).unwrap().len(), MODELS.len());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
