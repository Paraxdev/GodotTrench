//! Bringing Valve (`.vmt`/`.vtf`) and Quake (`.wad`, `.wal`) textures into the project, and finding the face
//! materials a map uses that the project does not have.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use gt_core::NodeId;
use gt_formats::texture_import::{self, Library, Report, Target};
use serde_json::{Value, json};

use crate::state::EditorState;

/// Where converted textures go in the open project.
pub fn target(state: &EditorState) -> Result<Target, String> {
    let game = &state.game;
    let project_root = game.project_root.clone().ok_or("Open a Godot project first, converted textures go into its texture folder")?;
    let texture_dir = game.texture_root().ok_or("The project has no texture folder")?;
    let material_dir = if game.textures.material_dir.is_empty() { None } else { game.resolve_res(&game.textures.material_dir) };
    Ok(Target {
        project_root,
        material_dir: material_dir.unwrap_or_else(|| texture_dir.clone()),
        texture_dir,
        material_ext: game.textures.material_extension.trim_start_matches('.').to_string(),
    })
}

/// Face materials of the map that neither the project, the built-in placeholders nor the tool textures provide, with
/// how many faces use each and the first brush or mesh using it.
pub fn missing_materials(state: &EditorState) -> BTreeMap<String, (usize, NodeId)> {
    let known: HashSet<String> = state.materials.entries.iter().map(|e| e.name.to_ascii_lowercase()).collect();
    let map = &state.doc.map;
    let faces = map
        .brushes()
        .flat_map(|(id, b)| b.faces.iter().map(move |f| (id, f.data.material.as_str())))
        .chain(map.meshes().flat_map(|(id, m)| m.faces.iter().map(move |f| (id, f.data.material.as_str()))));
    let mut out: BTreeMap<String, (usize, NodeId)> = BTreeMap::new();
    for (id, material) in faces {
        if material.is_empty() {
            continue;
        }

        let lower = material.to_ascii_lowercase();
        if known.contains(&lower) || known.contains(&texture_import::file_stem(&lower)) || state.game.is_tool_texture(&lower) {
            continue;
        }

        out.entry(lower).or_insert((0, id)).0 += 1;
    }

    out
}

pub fn missing_material_issues(state: &EditorState) -> Vec<gt_doc::issues::Issue> {
    missing_materials(state)
        .into_iter()
        .map(|(name, (faces, node))| gt_doc::issues::Issue {
            node: Some(node),
            severity: gt_doc::issues::Severity::Warning,
            code: "missing_material",
            message: format!("Material '{name}' is not in the project ({faces} faces), convert or add its texture"),
        })
        .collect()
}

/// Converts `only` (face material names) or every texture the sources hold, then reloads the material browser.
/// The map's sky colors come from its sky texture, unless the map has them already.
pub fn convert(state: &mut EditorState, lib: &Library, only: Option<&BTreeSet<String>>, overwrite: bool) -> Result<Report, String> {
    let target = target(state)?;
    let mut report = lib.convert(only, &target, overwrite);
    if !report.converted.is_empty() {
        state.material_reload = true;
        match_file_case(state, &report.converted);
    }

    if overwrite || !state.doc.map.properties.contains_key("sky_top_color") {
        report.sky = lib.sky_keys(&state.doc.map.properties, &target)?;
    }

    if !report.sky.is_empty() {
        let keys = report.sky.clone();
        state.doc.edit("Sky From Texture", |m, _| m.properties.extend(keys));
    }

    Ok(report)
}

/// Converted files are lowercase, while Quake and Half-Life maps often spell the same texture `WALL1`. The engines
/// never cared, but Godot's exported games do, so faces take the file's spelling.
fn match_file_case(state: &mut EditorState, converted: &[String]) {
    let converted: HashSet<&str> = converted.iter().map(String::as_str).collect();
    let wrong = |m: &str| m != m.to_ascii_lowercase() && converted.contains(m.to_ascii_lowercase().as_str());
    let map = &state.doc.map;
    let brushes: Vec<NodeId> = map.brushes().filter(|(_, b)| b.faces.iter().any(|f| wrong(&f.data.material))).map(|(id, _)| id).collect();
    let meshes: Vec<NodeId> = map.meshes().filter(|(_, m)| m.faces.iter().any(|f| wrong(&f.data.material))).map(|(id, _)| id).collect();
    if brushes.is_empty() && meshes.is_empty() {
        return;
    }

    state.doc.edit("Match Texture File Names", |m, _| {
        for id in &brushes {
            for f in m.brush_mut(*id).map(|b| b.faces.iter_mut()).into_iter().flatten() {
                f.data.material = f.data.material.to_ascii_lowercase();
            }
        }

        for id in &meshes {
            for f in m.mesh_mut(*id).map(|b| b.faces.iter_mut()).into_iter().flatten() {
                f.data.material = f.data.material.to_ascii_lowercase();
            }
        }
    });
}

pub fn report_json(report: &Report) -> Value {
    json!({
        "converted": report.converted.len(),
        "existing": report.existing.len(),
        "failed": report.failed.iter().map(|(n, e)| json!({ "material": n, "error": e })).collect::<Vec<_>>(),
        "sky": report.sky.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<serde_json::Map<String, Value>>(),
    })
}

/// Texture sources near an imported map file: `materials` and `textures` folders, and for a `.map` the WADs its
/// worldspawn lists.
pub fn sources_for_map(state: &EditorState, map_file: &Path) -> Vec<PathBuf> {
    texture_import::sources_near(map_file, state.doc.map.properties.get("wad").map(String::as_str))
}

/// The missing materials of the open map that `lib` can provide.
pub fn convertible(state: &EditorState, lib: &Library) -> BTreeSet<String> {
    missing_materials(state).into_keys().filter(|m| lib.contains(m)).collect()
}

/// After importing `map_file`, offers to convert the textures the map uses that were found next to it.
pub fn offer_for_map(state: &mut EditorState, map_file: &Path) {
    let sources = sources_for_map(state, map_file);
    if sources.is_empty() || state.game.project_root.is_none() {
        return;
    }

    let lib = texture_import::scan(&sources);
    let wanted = convertible(state, &lib);
    if wanted.is_empty() {
        return;
    }

    let missing = missing_materials(state).len();
    let places: Vec<String> = sources.iter().take(3).map(|p| p.display().to_string()).collect();
    let answer = rfd::MessageDialog::new()
        .set_title("Convert the map's textures?")
        .set_description(format!(
            "{} of the {missing} materials this map uses but the project lacks were found in {}. Convert them into the project's texture folder?",
            wanted.len(),
            places.join(", ")
        ))
        .set_buttons(rfd::MessageButtons::YesNo)
        .show();
    if answer != rfd::MessageDialogResult::Yes {
        return;
    }

    match convert(state, &lib, Some(&wanted), false) {
        Ok(report) => state.set_status(format!("Textures: {}", report.summary())),
        Err(e) => state.set_status(e),
    }
}

/// File > Import > Convert textures: everything in a folder, or only what the open map is missing.
pub fn convert_folder_dialog(state: &mut EditorState) {
    if let Err(e) = target(state) {
        state.set_status(e);
        return;
    }

    let Some(folder) = rfd::FileDialog::new().set_title("Folder with .vmt/.vtf, .wad or .wal textures").pick_folder() else { return };
    let lib = texture_import::scan(std::slice::from_ref(&folder));
    if lib.is_empty() {
        state.set_status(format!("No .vmt, .wad or .wal textures in {}", folder.display()));
        return;
    }

    let wanted = convertible(state, &lib);
    let only = if wanted.is_empty() {
        None
    } else {
        let answer = rfd::MessageDialog::new()
            .set_title("Convert textures")
            .set_description(format!(
                "{} textures found. The open map is missing {} of them. Convert only those, or all {}?",
                lib.len(),
                wanted.len(),
                lib.len()
            ))
            .set_buttons(rfd::MessageButtons::YesNoCancelCustom("The map's".into(), "All".into(), "Cancel".into()))
            .show();
        match answer {
            rfd::MessageDialogResult::Custom(c) if c == "The map's" => Some(wanted),
            rfd::MessageDialogResult::Custom(c) if c == "All" => None,
            _ => return,
        }
    };
    match convert(state, &lib, only.as_ref(), false) {
        Ok(report) => state.set_status(format!("Textures: {}", report.summary())),
        Err(e) => state.set_status(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_doc::NodeKind;
    use gt_geom::Brush;

    #[test]
    fn missing_materials_skip_project_placeholder_and_tool_textures() {
        let mut state = EditorState::new(Default::default());
        let layer = state.doc.map.default_layer();
        let aabb = gt_core::Aabb::new(gt_core::DVec3::ZERO, gt_core::DVec3::splat(64.0));
        for m in ["brick/wall", "dev/grey", "special/skip", "BRICK/WALL", "*water1"] {
            state.doc.map.insert(layer, NodeKind::Brush(Brush::from_aabb(&aabb, m).unwrap()));
        }

        let missing = missing_materials(&state);
        assert_eq!(missing.keys().collect::<Vec<_>>(), ["*water1", "brick/wall"]);
        assert_eq!(missing["brick/wall"].0, 12, "two brushes of six faces, names compared without case");
        let issues = missing_material_issues(&state);
        assert!(issues.iter().all(|i| i.code == "missing_material") && issues.len() == 2);
    }
}
