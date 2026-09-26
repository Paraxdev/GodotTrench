//! The nature models. The low poly Blockbench trees, bushes, rocks and foliage in `assets/nature` are small and embedded
//! in the editor, so the scatter presets built from them work in any project, offline. The procedural glTF trees and
//! bushes with the bark and leaf textures they share, the rest of the demo project's `godot/godottrench/nature`, are too
//! big to embed and ship as a release download that the editor's content wizard installs.

macro_rules! models {
    ($($name:literal),* $(,)?) => {
        [$(($name, include_str!(concat!("../assets/nature/", $name, ".bbmodel")))),*]
    };
}

/// The first nine are the original pack names that saved maps and presets refer to.
static MODELS: [(&str, &str); 18] = models!(
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

/// The folders of the downloadable pack inside the nature folder, next to the embedded Blockbench models.
pub const PACK_FOLDERS: [&str; 4] = ["trees", "trees_detailed", "bushes", "textures"];

/// Every embedded model as (name, bbmodel text).
pub fn all() -> Vec<(&'static str, String)> {
    MODELS.iter().map(|(name, text)| (*name, text.to_string())).collect()
}

pub fn names() -> Vec<&'static str> {
    MODELS.iter().map(|(name, _)| *name).collect()
}

/// The embedded model a path inside the nature folder names, such as `rock.bbmodel`.
pub fn embedded(rel: &str) -> Option<&'static str> {
    let name = rel.strip_suffix(".bbmodel")?;
    MODELS.iter().find(|(n, _)| *n == name).map(|(_, text)| *text)
}

/// Whether the downloadable part of the pack is in the nature folder `dir`, judged by the credits file of every folder
/// that has models.
pub fn pack_installed(dir: &std::path::Path) -> bool {
    PACK_FOLDERS.iter().filter(|f| **f != "textures").all(|f| dir.join(f).join("pack.json").is_file())
}

/// Writes the embedded models `names` (all of them when empty) into `dir`, skipping the ones already there unless
/// `overwrite`. Returns the files written.
pub fn install(dir: &std::path::Path, names: &[&str], overwrite: bool) -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    for (name, text) in MODELS.iter().filter(|(n, _)| names.is_empty() || names.contains(n)) {
        let path = dir.join(format!("{name}.bbmodel"));
        if overwrite || !path.exists() {
            std::fs::create_dir_all(dir)?;
            crate::write_atomic(&path, &mut text.as_bytes())?;
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

    /// The demo project's nature folder, which CI zips as the downloadable pack.
    fn repo_pack() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot/godottrench/nature")
    }

    /// A preset uses either embedded models only, or glTF models of the downloadable pack only, so picking one knows
    /// whether it can work offline.
    #[test]
    fn every_preset_item_is_embedded_or_in_the_pack() {
        let prefix = format!("{}/", gt_doc::scatter::NATURE_DIR);
        for preset in gt_doc::scatter::PRESETS {
            let (_, items) = gt_doc::scatter::preset(preset).unwrap();
            let rels: Vec<&str> = items.iter().map(|i| i.source.strip_prefix(&prefix).unwrap_or(&i.source)).collect();
            let embedded_count = rels.iter().filter(|r| embedded(r).is_some()).count();
            assert!(embedded_count == 0 || embedded_count == rels.len(), "{preset} mixes embedded and downloaded models");
            for rel in rels {
                let folder = rel.split('/').next().unwrap();
                assert!(embedded(rel).is_some() || PACK_FOLDERS.contains(&folder), "{preset}: {rel} is neither embedded nor in a pack folder");
                assert!(repo_pack().join(rel).is_file(), "{preset}: {rel} is not in godot/godottrench/nature");
            }
        }
    }

    /// The glTF models load their textures by relative URI, so the pack must carry them.
    #[test]
    fn every_gltf_texture_is_in_the_pack() {
        let mut used = std::collections::BTreeSet::new();
        for folder in PACK_FOLDERS {
            for entry in std::fs::read_dir(repo_pack().join(folder)).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().is_none_or(|e| e != "glb") {
                    continue;
                }

                let bytes = std::fs::read(&path).unwrap();
                assert_eq!(&bytes[..4], b"glTF", "{}", path.display());
                let len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
                let json: serde_json::Value = serde_json::from_slice(&bytes[20..20 + len]).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                let images = json["images"].as_array().cloned().unwrap_or_default();
                assert!(!images.is_empty(), "{} has textures", path.display());
                for uri in images.iter().filter_map(|i| i["uri"].as_str()) {
                    let texture = path.parent().unwrap().join(uri);
                    assert!(texture.is_file(), "{} loads {uri}, which the pack does not have", path.display());
                    used.insert(texture.canonicalize().unwrap());
                }
            }
        }

        for texture in used {
            let img = image::open(&texture).unwrap_or_else(|e| panic!("{}: {e}", texture.display()));
            assert!(img.width() >= 64 && img.height() >= 64, "{}", texture.display());
            assert!(texture.starts_with(repo_pack().join("textures").canonicalize().unwrap()));
        }
    }

    /// The demo project ships the embedded models too, so what a map shows there is what a new project gets.
    #[test]
    fn the_embedded_models_match_the_demo_project() {
        let mut loose: Vec<String> = std::fs::read_dir(repo_pack())
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "bbmodel"))
            .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
            .collect();
        loose.sort();
        let mut names: Vec<String> = names().iter().map(|n| n.to_string()).collect();
        names.sort();
        assert_eq!(loose, names, "embed new Blockbench models of godot/godottrench/nature in MODELS");
        for (name, text) in MODELS {
            let demo = std::fs::read_to_string(repo_pack().join(format!("{name}.bbmodel"))).unwrap();
            assert!(demo == text, "godot/godottrench/nature/{name}.bbmodel differs from assets/nature, copy it over");
        }

        assert!(pack_installed(&repo_pack()));
    }

    #[test]
    fn install_writes_only_missing_models_unless_overwriting() {
        let dir = std::env::temp_dir().join(format!("gt_nature_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(install(&dir, &["rock", "grass"], false).unwrap(), [dir.join("rock.bbmodel"), dir.join("grass.bbmodel")]);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2, "only what was asked for, and no partial files left");
        assert_eq!(install(&dir, &[], false).unwrap().len(), MODELS.len() - 2);
        assert!(install(&dir, &[], false).unwrap().is_empty());
        std::fs::write(dir.join("oak.bbmodel"), "mine").unwrap();
        assert!(install(&dir, &["oak"], false).unwrap().is_empty());
        assert_eq!(std::fs::read_to_string(dir.join("oak.bbmodel")).unwrap(), "mine", "a file already there is kept");
        assert_eq!(install(&dir, &[], true).unwrap().len(), MODELS.len());
        assert!(!pack_installed(&dir), "the glTF part is a separate download");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
