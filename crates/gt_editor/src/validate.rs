//! The checks behind validate_map and the Issues panel, for the open map or any map file of the project.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use gt_core::NodeId;
use gt_doc::Map;
use gt_doc::issues::{self, Issue, Severity};
use gt_formats::GameConfig;

use crate::materials::MaterialLibrary;
use crate::state::EditorState;

/// Inputs validate_map accepts per entity class: the declared ones, plus every method and property of the Godot class
/// the entity is built as and the functions of its script, since the runtime calls any of them. A class whose Godot
/// class the table does not know keeps only its declared inputs, and one without declared inputs whose script cannot
/// be read (no project open) stays unchecked, an empty list.
pub fn class_inputs(game: &GameConfig) -> BTreeMap<String, Vec<String>> {
    game.entities
        .iter()
        .map(|d| {
            let mut list: Vec<String> = d.inputs.iter().map(|i| i.name.clone()).collect();
            let script = match d.script.as_str() {
                "" => Some(Vec::new()),
                path => game.resolve_res(path).and_then(|p| std::fs::read_to_string(p).ok()).map(|src| gt_formats::godot_api::script_functions(&src)),
            };
            if script.is_none() && list.is_empty() {
                return (d.classname.clone(), list);
            }

            if let Some(members) = gt_formats::godot_api::class_members(&d.node_class) {
                list.extend(members.into_iter().map(str::to_string));
                list.extend(script.unwrap_or_default());
            }

            (d.classname.clone(), list)
        })
        .collect()
}

/// Every issue of `map`. `external` holds the targetnames its overlays add in Godot.
pub fn map_issues(map: &Map, game: &GameConfig, materials: &mut MaterialLibrary, external: &BTreeSet<String>) -> Vec<Issue> {
    let inputs = class_inputs(game);
    let mut list = issues::check_with_external(
        map,
        |class| {
            game.entity(class).map(|d| issues::ClassIo {
                outputs: d.outputs.iter().map(|o| o.name.as_str()).collect(),
                inputs: inputs.get(class).map(|list| list.iter().map(String::as_str).collect()).unwrap_or_default(),
            })
        },
        external,
    );
    for (id, e) in map.entities() {
        if game.entity(&e.classname).is_none() && !e.classname.is_empty() {
            list.push(Issue {
                node: Some(id),
                severity: Severity::Warning,
                code: "unknown_class",
                message: format!("No entity definition for '{}'", e.classname),
            });
        }
    }

    list.extend(crate::texture_convert::missing_material_issues(map, game, materials));
    list.extend(missing_model_issues(map, game));
    list.extend(crate::zfight::coplanar_issues(map, game, materials));
    list.sort_by_key(|i| Reverse(i.severity));
    list
}

/// Issues of the open map, after looking for materials and overlay changes written since the last check.
pub fn open_map_issues(state: &mut EditorState) -> Vec<Issue> {
    let unknown: Vec<String> = crate::texture_convert::missing_materials(state).into_keys().collect();
    state.find_new_materials(unknown.iter().map(String::as_str));
    let path = state.doc.path.clone();
    state.overlay_ghosts.refresh(path.as_deref());
    let external = state.overlay_ghosts.targetnames();
    map_issues(&state.doc.map, &state.game, &mut state.materials, &external)
}

/// Issues of the map saved at `path`, which does not have to be open. A map open in a tab is checked as it is in the
/// editor, unsaved edits included. Also returns that map, for issue bounds.
pub fn file_issues(state: &mut EditorState, path: &Path) -> Result<(Map, Vec<Issue>), String> {
    let same = |p: &Option<PathBuf>| p.as_deref().is_some_and(|p| same_file(p, path));
    if same(&state.doc.path) {
        return Ok((state.doc.map.clone(), open_map_issues(state)));
    }

    let map = match state.tabs.iter().find(|t| same(&t.doc.path)) {
        Some(tab) => tab.doc.map.clone(),
        None => gt_doc::format::load(path).map_err(|e| format!("{}: {e}", path.display()))?.map,
    };
    let unknown: Vec<String> = crate::texture_convert::missing_materials_in(&map, &state.game, &state.materials).into_keys().collect();
    state.find_new_materials(unknown.iter().map(String::as_str));
    let sidecar = std::fs::read_to_string(crate::overlays::sidecar_path(path)).ok().and_then(|t| crate::overlays::parse(&t).ok());
    let external = sidecar.unwrap_or_default().into_iter().flat_map(|i| i.targetnames).collect();
    let found = map_issues(&map, &state.game, &mut state.materials, &external);
    Ok((map, found))
}

fn same_file(a: &Path, b: &Path) -> bool {
    a == b || matches!((a.canonicalize(), b.canonicalize()), (Ok(x), Ok(y)) if x == y)
}

/// Every `.gtm` map in the project folder, skipping hidden folders such as `.godot`.
pub fn project_maps(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            if entry.file_name().to_string_lossy().starts_with('.') {
                continue;
            }

            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("gtm")) {
                out.push(path);
            }
        }
    }

    let mut out = Vec::new();
    walk(root, &mut out);
    out.sort();
    out
}

/// The file a `model` property names. Godot reads a relative path from the project root, like a `res://` one.
fn model_file(game: &GameConfig, model: &str) -> Option<PathBuf> {
    if Path::new(model).is_absolute() {
        return Some(PathBuf::from(model));
    }

    if model.contains("://") && !model.starts_with("res://") {
        return None;
    }

    Some(game.project_root.as_ref()?.join(model.trim_start_matches("res://")))
}

/// One `missing_model` warning per model path that no file answers to, on the first entity using it. A file of the
/// same name with another model extension, the usual `.glb` against `.gltf` mix-up, is named in the message.
pub fn missing_model_issues(map: &Map, game: &GameConfig) -> Vec<Issue> {
    let mut missing: BTreeMap<&str, (PathBuf, Vec<NodeId>)> = BTreeMap::new();
    for (id, e) in map.entities() {
        let Some(model) = e.property("model").map(str::trim) else { continue };
        // Quake brush entities keep their brush model as "*1".
        if model.starts_with('*') || Path::new(model).extension().is_none() {
            continue;
        }

        let Some(file) = model_file(game, model) else { continue };
        if !file.is_file() {
            missing.entry(model).or_insert_with(|| (file, Vec::new())).1.push(id);
        }
    }

    missing
        .into_iter()
        .map(|(model, (file, ids))| {
            let first = map.entity(ids[0]).map(|e| match e.targetname() {
                Some(name) => format!("{} '{name}'", e.classname),
                None => format!("{} {}", e.classname, ids[0]),
            });
            let more = if ids.len() > 1 { format!(" and {} more", ids.len() - 1) } else { String::new() };
            let stem = model.rsplit_once('.').map_or(model, |(stem, _)| stem);
            let other = crate::models::MODEL_EXTS.iter().chain(&["tscn", "scn"]).find(|ext| file.with_extension(ext).is_file());
            let hint = other.map(|ext| format!(", but {stem}.{ext} does")).unwrap_or_default();
            Issue {
                node: Some(ids[0]),
                severity: Severity::Warning,
                code: "missing_model",
                message: format!("Model '{model}' does not exist{hint}, used by {}{more}", first.unwrap_or_default()),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_doc::{Entity, NodeKind};

    #[test]
    fn missing_models_are_reported_once_per_path() {
        let dir = std::env::temp_dir().join(format!("gt_missing_models_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("props")).unwrap();
        std::fs::write(dir.join("props/chair.gltf"), "{}").unwrap();
        std::fs::write(dir.join("props/desk.glb"), "").unwrap();
        let mut game = GameConfig::builtin();
        game.project_root = Some(dir.clone());
        let mut map = Map::new();
        let layer = map.default_layer();
        let mut add = |class: &str, model: &str, name: &str| {
            let mut e = Entity::new(class);
            e.properties.insert("model".into(), model.into());
            if !name.is_empty() {
                e.properties.insert("targetname".into(), name.into());
            }

            map.insert(layer, NodeKind::Entity(e))
        };
        let chair = add("prop_model", "res://props/chair.glb", "clue_chair");
        add("prop_physics", "res://props/chair.glb", "");
        add("prop_model", "props/desk.glb", "");
        add("func_door", "*1", "");
        let lamp = add("prop_physics", "res://props/lamp.tscn", "");

        let found = missing_model_issues(&map, &game);
        assert_eq!(found.len(), 2, "{found:?}");
        assert_eq!((found[0].node, found[0].code), (Some(chair), "missing_model"));
        assert_eq!(
            found[0].message,
            "Model 'res://props/chair.glb' does not exist, but res://props/chair.gltf does, used by prop_model 'clue_chair' and 1 more"
        );
        assert_eq!(found[1].node, Some(lamp));
        assert!(found[1].message.ends_with(&format!("used by prop_physics {lamp}")), "{}", found[1].message);

        game.project_root = None;
        assert!(missing_model_issues(&map, &game).is_empty(), "project paths are not checked without a project");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn project_maps_skip_hidden_folders() {
        let dir = std::env::temp_dir().join(format!("gt_project_maps_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for sub in ["maps/blocks", ".godot/imported"] {
            std::fs::create_dir_all(dir.join(sub)).unwrap();
        }

        for file in ["maps/a.gtm", "maps/blocks/b.GTM", ".godot/imported/c.gtm", "maps/a.gtm.bak", "maps/a.json"] {
            std::fs::write(dir.join(file), "").unwrap();
        }

        assert_eq!(project_maps(&dir), [dir.join("maps/a.gtm"), dir.join("maps/blocks/b.GTM")]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
