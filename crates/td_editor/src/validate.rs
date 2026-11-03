//! The checks behind validate_map and the Issues panel, for the open map or any map file of the project.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use td_core::NodeId;
use td_doc::issues::{self, Issue, Severity};
use td_doc::{Map, NodeKind};
use td_formats::GameConfig;

use crate::materials::MaterialLibrary;
use crate::prefabs::PrefabCache;
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
                path => game.resolve_res(path).and_then(|p| std::fs::read_to_string(p).ok()).map(|src| td_formats::godot_api::script_functions(&src)),
            };
            if script.is_none() && list.is_empty() {
                return (d.classname.clone(), list);
            }

            if let Some(members) = td_formats::godot_api::class_members(&d.node_class) {
                list.extend(members.into_iter().map(str::to_string));
                list.extend(script.unwrap_or_default());
            }

            (d.classname.clone(), list)
        })
        .collect()
}

/// Every issue of `map`. `external` holds the targetnames its overlays add in Godot.
pub fn map_issues(
    map: &Map,
    game: &GameConfig,
    materials: &mut MaterialLibrary,
    prefabs: &mut PrefabCache,
    map_path: Option<&Path>,
    external: &BTreeSet<String>,
) -> Vec<Issue> {
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
                message: if crate::entity_pack::in_pack(&e.classname) {
                    format!(
                        "No entity definition for '{}', it comes with the Gameplay entities pack. Gameplay > Install Gameplay Entities adds it",
                        e.classname
                    )
                } else {
                    format!("No entity definition for '{}'", e.classname)
                },
            });
        }
    }

    for clash in &game.clashes {
        if map.entities().any(|(_, e)| e.classname == clash.classname) {
            list.push(Issue { node: None, severity: Severity::Warning, code: "entity_clash", message: clash.describe() });
        }
    }

    list.extend(crate::texture_convert::missing_material_issues(map, game, materials));
    list.extend(missing_model_issues(map, game));
    list.extend(map_script_issues(map, prefabs, map_path));
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
    map_issues(&state.doc.map, &state.game, &mut state.materials, &mut state.prefabs, path.as_deref(), &external)
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
        None => td_doc::format::load(path).map_err(|e| format!("{}: {e}", path.display()))?.map,
    };
    let unknown: Vec<String> = crate::texture_convert::missing_materials_in(&map, &state.game, &state.materials).into_keys().collect();
    state.find_new_materials(unknown.iter().map(String::as_str));
    let sidecar = std::fs::read_to_string(crate::overlays::sidecar_path(path)).ok().and_then(|t| crate::overlays::parse(&t).ok());
    let external = sidecar.unwrap_or_default().into_iter().flat_map(|i| i.targetnames).collect();
    let found = map_issues(&map, &state.game, &mut state.materials, &mut state.prefabs, Some(path), &external);
    Ok((map, found))
}

fn same_file(a: &Path, b: &Path) -> bool {
    a == b || matches!((a.canonicalize(), b.canonicalize()), (Ok(x), Ok(y)) if x == y)
}

/// Every `.tdm` map in the project folder, skipping hidden folders such as `.godot`.
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
            } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("tdm")) {
                out.push(path);
            }
        }
    }

    let mut out = Vec::new();
    walk(root, &mut out);
    out.sort();
    out
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

        // Godot reads a relative path from the project root, like a res:// one.
        let Some(file) = game.resolve_res(model) else { continue };
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

/// Inputs through which I/O can give a logic_script code or have it compile: its source and expression, the Object
/// methods that set any property or call any method, and the methods starting with `_`, such as `_compile`.
const CODE_INPUTS: &[&str] =
    &["source", "expression", "set", "set_deferred", "set_indexed", "call", "call_deferred", "callv", "propagate_call", "notification"];

fn reaches_code(input: &str) -> bool {
    let input = input.trim();
    input.starts_with('_') || CODE_INPUTS.contains(&input.to_lowercase().as_str())
}

const PLAYS: &str = "when the scene plays, read it before you play a map someone else made";

/// A warning for every part of the map that runs GDScript of its own when the scene plays: a logic_script whose source
/// or expression holds code, I/O or a logic_call that can give one code, and a prefab instance whose prefab has either.
/// Instance paths resolve against `map_path`.
pub fn map_script_issues(map: &Map, prefabs: &mut PrefabCache, map_path: Option<&Path>) -> Vec<Issue> {
    let mut found = script_holders(map);
    let root = prefabs.project_root.clone();
    for (id, node) in map.nodes.iter() {
        let NodeKind::Instance(inst) = &node.kind else { continue };
        let Some(path) = crate::prefabs::resolve(&inst.path, map_path, root.as_deref()) else { continue };
        if prefab_holds_script(prefabs, &path, 1) {
            found.push((*id, format!("The prefab {} runs GDScript of its own {PLAYS}", inst.path)));
        }
    }

    found.into_iter().map(|(id, message)| Issue { node: Some(id), severity: Severity::Warning, code: "map_script", message }).collect()
}

/// Whether `map` itself runs GDScript, or hands a logic_script code, when the scene plays.
pub fn holds_script(map: &Map) -> bool {
    !script_holders(map).is_empty()
}

/// The nodes of `map` itself that run GDScript, with the warning for each.
fn script_holders(map: &Map) -> Vec<(NodeId, String)> {
    let has_code = |e: &td_doc::Entity| ["source", "expression"].iter().any(|k| e.property(k).is_some_and(|v| !v.trim().is_empty()));
    let mut named: BTreeMap<&str, Vec<&td_doc::Entity>> = BTreeMap::new();
    for (_, e) in map.entities() {
        if let Some(name) = e.targetname() {
            named.entry(name).or_default().push(e);
        }
    }

    // A name that only entities of this map answer to, none of them a logic_script, cannot reach one. Anything else, a
    // group, a wildcard, !activator or a name in a prefab or a Godot scene, might.
    let may_be_script = |target: &str| named.get(target.trim()).is_none_or(|list| list.iter().any(|e| e.classname == "logic_script"));
    let can_give = format!("which can make a logic_script run GDScript from the map {PLAYS}");
    let mut found = Vec::new();
    for (id, e) in map.entities() {
        let name = map.get(id).map(|n| n.name()).unwrap_or_default();
        if e.classname == "logic_script" && has_code(e) {
            found.push((id, format!("{name} runs GDScript from the map {PLAYS}")));
        }

        if let Some(o) = e.outputs.iter().find(|o| reaches_code(&o.input) && may_be_script(&o.target)) {
            found.push((id, format!("{name} sends {} to {}, {can_give}", o.input, o.target)));
        }

        let target = e.property("call_target").unwrap_or_default();
        if e.classname == "logic_call"
            && let Some(method) = e.property("method").filter(|m| reaches_code(m))
            && may_be_script(target)
        {
            found.push((id, format!("{name} calls {method} on {target}, {can_give}")));
        }
    }

    found
}

fn prefab_holds_script(prefabs: &mut PrefabCache, path: &Path, depth: usize) -> bool {
    if depth > crate::prefabs::MAX_DEPTH {
        return false;
    }

    let Some(map) = prefabs.get(path).map.clone() else { return false };
    if !script_holders(&map).is_empty() {
        return true;
    }

    let root = prefabs.project_root.clone();
    map.nodes.values().any(|node| match &node.kind {
        NodeKind::Instance(inst) => {
            crate::prefabs::resolve(&inst.path, Some(path), root.as_deref()).is_some_and(|p| prefab_holds_script(prefabs, &p, depth + 1))
        }
        _ => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use td_doc::Entity;

    #[test]
    fn logic_scripts_with_code_are_named() {
        let mut map = Map::new();
        let layer = map.default_layer();
        let mut add = |class: &str, props: &[(&str, &str)], outputs: &[(&str, &str)]| {
            let mut e = Entity::new(class);
            e.properties.extend(props.iter().map(|(k, v)| (k.to_string(), v.to_string())));
            for (target, input) in outputs {
                let (target, input) = (target.to_string(), input.to_string());
                e.outputs.push(td_doc::IoConnection { output: "on_trigger".into(), target, input, parameter: "print(1)".into(), delay: 0.0, times: -1 });
            }

            map.insert(layer, NodeKind::Entity(e))
        };
        let heal = add("logic_script", &[("targetname", "heal"), ("source", "activator.health += 10")], &[]);
        let half = add("logic_script", &[("targetname", "half"), ("expression", "parameter / 2")], &[]);
        add(
            "logic_script",
            &[
                ("targetname", "blank"),
                (
                    "source", "  
",
                ),
            ],
            &[],
        );
        add("logic_relay", &[("targetname", "relay"), ("source", "print(1)")], &[]);
        add("func_door", &[("targetname", "door")], &[]);
        let start = add("logic_auto", &[("targetname", "start")], &[("blank", "run"), ("blank", "source"), ("blank", "_compile")]);
        add("logic_relay", &[("targetname", "opener")], &[("door", "set"), ("door", "open")]);
        let group = add("logic_relay", &[("targetname", "caller")], &[("@scripts", "call")]);
        let setter = add("logic_call", &[("targetname", "setter"), ("method", "set"), ("call_target", "blank")], &[]);
        add("logic_call", &[("targetname", "door_setter"), ("method", "set"), ("call_target", "door")], &[]);
        add("logic_call", &[("targetname", "runner"), ("method", "run"), ("call_target", "blank")], &[]);

        let found = map_script_issues(&map, &mut PrefabCache::default(), None);
        assert_eq!(found.iter().map(|i| i.node).collect::<Vec<_>>(), [Some(heal), Some(half), Some(start), Some(group), Some(setter)]);
        assert_eq!(found[0].severity, Severity::Warning);
        assert!(found[0].message.starts_with("logic_script (heal) runs GDScript"), "{}", found[0].message);
        assert!(found[2].message.starts_with("logic_auto (start) sends source to blank, which can make a logic_script run GDScript"), "{}", found[2].message);
        assert!(found[4].message.starts_with("logic_call (setter) calls set on blank"), "{}", found[4].message);
        let game = GameConfig::with_gameplay_pack();
        let all = map_issues(&map, &game, &mut MaterialLibrary::new(&game), &mut PrefabCache::default(), None, &BTreeSet::new());
        assert_eq!(all.iter().filter(|i| i.code == "map_script").count(), 5, "validate_map and the Issues panel list them");
    }

    #[test]
    fn a_prefab_with_scripts_is_named_where_it_is_placed() {
        let dir = std::env::temp_dir().join(format!("td_prefab_scripts_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let save = |name: &str, kind: Option<NodeKind>| {
            let mut map = Map::new();
            let layer = map.default_layer();
            if let Some(kind) = kind {
                map.insert(layer, kind);
            }

            td_doc::format::save(&map, &dir.join(name)).unwrap();
            map
        };
        let mut script = Entity::new("logic_script");
        script.properties.insert("source".into(), "print(1)".into());
        let instance = |path: &str| {
            NodeKind::Instance(td_doc::map::Instance { path: path.into(), origin: Default::default(), angles: Default::default(), fixup: String::new() })
        };
        save("piece.tdm", Some(NodeKind::Entity(script)));
        save("nest.tdm", Some(instance("piece.tdm")));
        save("clean.tdm", None);
        let mut host = save("host.tdm", Some(instance("nest.tdm")));
        let layer = host.default_layer();
        host.insert(layer, instance("clean.tdm"));
        host.insert(layer, instance("missing.tdm"));

        let found = map_script_issues(&host, &mut PrefabCache::default(), Some(&dir.join("host.tdm")));
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].message, format!("The prefab nest.tdm runs GDScript of its own {PLAYS}"));
        assert!(matches!(host.get(found[0].node.unwrap()).map(|n| &n.kind), Some(NodeKind::Instance(i)) if i.path == "nest.tdm"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn missing_models_are_reported_once_per_path() {
        let dir = std::env::temp_dir().join(format!("td_missing_models_{}", std::process::id()));
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
        // Paths out of the project are never looked up, Godot cannot load them either.
        add("prop_model", "//host/share/chair.glb", "");
        add("prop_model", "res://../chair.glb", "");
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
    fn an_unknown_class_of_the_pack_points_at_the_install() {
        let game = GameConfig::builtin();
        let mut map = Map::new();
        let layer = map.default_layer();
        map.insert(layer, NodeKind::Entity(Entity::new("func_door")));
        map.insert(layer, NodeKind::Entity(Entity::new("my_thing")));
        let found = map_issues(&map, &game, &mut MaterialLibrary::new(&game), &mut PrefabCache::default(), None, &BTreeSet::new());
        let mut unknown: Vec<&str> = found.iter().filter(|i| i.code == "unknown_class").map(|i| i.message.as_str()).collect();
        unknown.sort();
        assert_eq!(
            unknown,
            [
                "No entity definition for 'func_door', it comes with the Gameplay entities pack. Gameplay > Install Gameplay Entities adds it",
                "No entity definition for 'my_thing'"
            ]
        );
    }

    #[test]
    fn a_class_defined_twice_is_a_warning_where_a_map_uses_it() {
        let mut game = GameConfig::with_gameplay_pack();
        let definitions = vec!["res://trenchdot/entities/definitions/func_door.tres".to_string(), "res://doors/func_door.tres".to_string()];
        game.clashes.push(td_formats::Clash { classname: "func_door".into(), definitions });
        game.clashes.push(td_formats::Clash { classname: "logic_relay".into(), definitions: vec!["a.tres".into(), "b.tres".into()] });
        let mut map = Map::new();
        let layer = map.default_layer();
        map.insert(layer, NodeKind::Entity(Entity::new("func_door")));
        let found = map_issues(&map, &game, &mut MaterialLibrary::new(&game), &mut PrefabCache::default(), None, &BTreeSet::new());
        let clashes: Vec<&Issue> = found.iter().filter(|i| i.code == "entity_clash").collect();
        assert_eq!(clashes.len(), 1, "only the class the map uses: {found:?}");
        assert_eq!(clashes[0].severity, Severity::Warning);
        assert!(clashes[0].message.contains("res://doors/func_door.tres wins over res://trenchdot/entities/definitions/func_door.tres"));
    }

    #[test]
    fn project_maps_skip_hidden_folders() {
        let dir = std::env::temp_dir().join(format!("td_project_maps_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for sub in ["maps/blocks", ".godot/imported"] {
            std::fs::create_dir_all(dir.join(sub)).unwrap();
        }

        for file in ["maps/a.tdm", "maps/blocks/b.TDM", ".godot/imported/c.tdm", "maps/a.tdm.bak", "maps/a.json"] {
            std::fs::write(dir.join(file), "").unwrap();
        }

        assert_eq!(project_maps(&dir), [dir.join("maps/a.tdm"), dir.join("maps/blocks/b.TDM")]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
