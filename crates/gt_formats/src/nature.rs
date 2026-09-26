//! Built-in nature pack: low poly Blockbench trees, bushes, rocks and foliage with embedded pixel textures, plus the
//! procedural glTF trees and bushes with the bark and leaf textures they share. The Blockbench models in `assets/nature`
//! are authored in Blockbench, the glTF part is the one in the demo project's `godot/godottrench/nature`. The editor
//! installs the whole pack into a project so every scatter preset works without any assets.

macro_rules! models {
    ($($name:literal),* $(,)?) => {
        [$(($name, include_str!(concat!("../assets/nature/", $name, ".bbmodel")))),*]
    };
}

macro_rules! files {
    ($($path:literal),* $(,)?) => {
        [$(($path, include_bytes!(concat!("../../../godot/godottrench/nature/", $path)) as &[u8])),*]
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

/// Everything else in the pack by its path inside the pack folder: the glTF models, the textures they load by relative
/// URI, and the `pack.json` files that credit the scanned textures.
const FILES: [(&str, &[u8]); 64] = files!(
    "pack.json",
    "bushes/pack.json",
    "bushes/bush_flowering.glb",
    "bushes/bush_hedge.glb",
    "bushes/bush_red.glb",
    "bushes/bush_round.glb",
    "bushes/bush_round_b.glb",
    "bushes/fern_clump.glb",
    "bushes/fern_clump_b.glb",
    "trees/pack.json",
    "trees/beech.glb",
    "trees/beech_b.glb",
    "trees/beech_young.glb",
    "trees/birch.glb",
    "trees/birch_b.glb",
    "trees/birch_young.glb",
    "trees/dead_oak.glb",
    "trees/dead_oak_b.glb",
    "trees/maple.glb",
    "trees/maple_autumn.glb",
    "trees/maple_b.glb",
    "trees/oak.glb",
    "trees/oak_b.glb",
    "trees/oak_young.glb",
    "trees/pine.glb",
    "trees/pine_b.glb",
    "trees/pine_young.glb",
    "trees/poplar.glb",
    "trees/poplar_b.glb",
    "trees/willow.glb",
    "trees/willow_b.glb",
    "trees_detailed/pack.json",
    "trees_detailed/beech.glb",
    "trees_detailed/birch.glb",
    "trees_detailed/maple.glb",
    "trees_detailed/maple_autumn.glb",
    "trees_detailed/oak.glb",
    "trees_detailed/pine.glb",
    "trees_detailed/willow.glb",
    "textures/bark_beech_albedo.jpg",
    "textures/bark_beech_normal.jpg",
    "textures/bark_birch_albedo.jpg",
    "textures/bark_birch_normal.jpg",
    "textures/bark_dead_albedo.jpg",
    "textures/bark_dead_normal.jpg",
    "textures/bark_oak_albedo.jpg",
    "textures/bark_oak_normal.jpg",
    "textures/bark_pine_albedo.jpg",
    "textures/bark_pine_normal.jpg",
    "textures/bark_willow_albedo.jpg",
    "textures/bark_willow_normal.jpg",
    "textures/leaf_beech.png",
    "textures/leaf_birch.png",
    "textures/leaf_box.png",
    "textures/leaf_fern.png",
    "textures/leaf_flowering.png",
    "textures/leaf_maple.png",
    "textures/leaf_maple_autumn.png",
    "textures/leaf_oak.png",
    "textures/leaf_pine.png",
    "textures/leaf_poplar.png",
    "textures/leaf_red.png",
    "textures/leaf_shrub.png",
    "textures/leaf_willow.png",
);

/// Every model as (name, bbmodel text).
pub fn all() -> Vec<(&'static str, String)> {
    MODELS.iter().map(|(name, text)| (*name, text.to_string())).collect()
}

pub fn names() -> Vec<&'static str> {
    MODELS.iter().map(|(name, _)| *name).collect()
}

/// Every model of the pack by its path inside the pack folder, the Blockbench ones first.
pub fn models() -> Vec<String> {
    let glb = FILES.iter().map(|(path, _)| *path).filter(|p| p.ends_with(".glb")).map(str::to_string);
    MODELS.iter().map(|(name, _)| format!("{name}.bbmodel")).chain(glb).collect()
}

/// Every file of the pack as (path inside the pack folder, contents).
fn files() -> impl Iterator<Item = (String, &'static [u8])> {
    MODELS.iter().map(|(name, text)| (format!("{name}.bbmodel"), text.as_bytes())).chain(FILES.iter().map(|(path, bytes)| (path.to_string(), *bytes)))
}

/// Writes missing (or all, with `overwrite`) files of the pack into `dir`, models and textures alike. Returns the files
/// written.
pub fn install(dir: &std::path::Path, overwrite: bool) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    for (rel, bytes) in files() {
        let path = dir.join(&rel);
        if overwrite || !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&path, bytes)?;
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
    fn every_preset_item_is_installed() {
        let prefix = format!("{}/", gt_doc::scatter::NATURE_DIR);
        let models = models();
        for preset in gt_doc::scatter::PRESETS {
            let (_, items) = gt_doc::scatter::preset(preset).unwrap();
            for item in items {
                let rel = item.source.strip_prefix(&prefix).unwrap_or(&item.source);
                assert!(models.iter().any(|m| m == rel), "{preset}: {} is not part of the installed pack", item.source);
            }
        }
    }

    /// The glTF models load their textures by relative URI, so a model installed without them draws untextured.
    #[test]
    fn every_gltf_texture_is_installed() {
        let installed: Vec<String> = files().map(|(path, _)| path).collect();
        let mut used = std::collections::BTreeSet::new();
        for (path, bytes) in FILES.iter().filter(|(p, _)| p.ends_with(".glb")) {
            assert_eq!(&bytes[..4], b"glTF", "{path}");
            let len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
            let json: serde_json::Value = serde_json::from_slice(&bytes[20..20 + len]).unwrap_or_else(|e| panic!("{path}: {e}"));
            let images = json["images"].as_array().cloned().unwrap_or_default();
            assert!(!images.is_empty(), "{path} has textures");
            for uri in images.iter().filter_map(|i| i["uri"].as_str()) {
                let mut parts: Vec<&str> = path.split('/').collect();
                parts.pop();
                for part in uri.split('/') {
                    match part {
                        ".." => {
                            parts.pop();
                        }
                        "." => {}
                        _ => parts.push(part),
                    }
                }

                let rel = parts.join("/");
                assert!(installed.contains(&rel), "{path} loads {uri}, which the pack does not install");
                used.insert(rel);
            }
        }

        for rel in used {
            let (_, bytes) = FILES.iter().find(|(p, _)| *p == rel).unwrap();
            let img = image::load_from_memory(bytes).unwrap_or_else(|e| panic!("{rel}: {e}"));
            assert!(img.width() >= 64 && img.height() >= 64, "{rel}");
        }
    }

    /// The demo project ships the same pack the editor installs, so what a map shows there is what a new project gets.
    #[test]
    fn the_installed_pack_matches_the_demo_project() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot/godottrench/nature");
        fn walk(root: &std::path::Path, dir: &std::path::Path, out: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(root, &path, out);
                } else if path.extension().is_none_or(|e| e != "import") {
                    out.push(path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
                }
            }
        }

        let mut on_disk = Vec::new();
        walk(&root, &root, &mut on_disk);
        on_disk.sort();
        let mut installed: Vec<String> = files().map(|(path, _)| path).collect();
        installed.sort();
        assert_eq!(installed, on_disk, "list new or removed files of godot/godottrench/nature in FILES");
        for (name, text) in MODELS {
            let demo = std::fs::read_to_string(root.join(format!("{name}.bbmodel"))).unwrap();
            assert!(demo == text, "godot/godottrench/nature/{name}.bbmodel differs from assets/nature, copy it over");
        }
    }

    #[test]
    fn install_writes_only_missing_files_unless_overwriting() {
        let dir = std::env::temp_dir().join(format!("gt_nature_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let all = MODELS.len() + FILES.len();
        assert_eq!(install(&dir, false).unwrap().len(), all);
        for model in models() {
            assert!(dir.join(&model).is_file(), "{model}");
        }

        assert_eq!(std::fs::read_dir(dir.join("textures")).unwrap().count(), 25, "the glTF models find their textures");
        assert!(install(&dir, false).unwrap().is_empty());
        std::fs::remove_file(dir.join("textures/leaf_pine.png")).unwrap();
        assert_eq!(install(&dir, false).unwrap(), [dir.join("textures/leaf_pine.png")], "an older install gets what it lacks");
        assert_eq!(install(&dir, true).unwrap().len(), all);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
