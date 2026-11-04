//! MCP scripts: JSON lists of tool calls replayed against the editor, with results saved under names and referenced by
//! later steps. `"$door.id"` is replaced by that JSON value, `"${door.id}"` inside a longer string by its text, and `$$` is a
//! literal `$`. `"$a.ids + $b.ids"` joins values into one array. When a step replaces brushes (CSG, clip) the ids saved
//! by earlier steps are rewritten to their replacements, see [`follow_replacements`].

use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

pub const FORMAT: &str = "trenchdot-mcp-script";

/// Tools a step may call. Scripts are shared like maps, so their steps edit and save the map, but cannot open another
/// project, have Godot run or build it, or read and write outside the folders run_script allows.
const TOOLS: &[&str] = &[
    "get_state",
    "list_nodes",
    "summarize_map",
    "changes_since",
    "get_node",
    "get_game_config",
    "validate_map",
    "code_reference",
    "project_content",
    "create_brush",
    "create_entity",
    "update_entity",
    "create_mesh",
    "mesh_edit",
    "texture",
    "create_terrain",
    "terrain_edit",
    "scatter",
    "prefab_library",
    "trim",
    "random_fill",
    "blend",
    "gameplay",
    "hierarchy",
    "set_map_properties",
    "bake_lighting",
    "duplicate",
    "import_model",
    "select",
    "transform",
    "set_face",
    "set_camera",
    "set_editor",
    "map_file",
    "run_action",
];

/// run_action actions a step may run: edits of the map and of how the editor shows it.
const ACTIONS: &[&str] = &[
    "new_map",
    "save",
    "undo",
    "redo",
    "delete",
    "duplicate",
    "select_all",
    "select_none",
    "select_inverse",
    "select_touching",
    "select_inside",
    "select_siblings",
    "select_same_material",
    "group",
    "ungroup",
    "hide_selected",
    "isolate_selected",
    "unhide_all",
    "lock_selected",
    "unlock_all",
    "grid_up",
    "grid_down",
    "toggle_snap",
    "toggle_uv_lock",
    "toggle_textured",
    "csg_subtract",
    "csg_merge",
    "csg_intersect",
    "csg_hollow",
    "rotate",
    "flip",
    "focus_selection",
    "create_brush_entity",
    "place_entities",
    "move_to_world",
    "add_layer",
    "snap_vertices",
    "apply_material",
    "create_decal",
    "paste",
    "nudge",
    "set_tool",
    "explode_instances",
    "create_displacement",
    "remove_displacement",
    "sew_displacements",
    "sculpt",
    "sprinkle",
    "mesh_op",
    "store_camera",
    "recall_camera",
    "set_shade",
    "edit_mesh",
    "convert_to_mesh",
    "convert_to_brushes",
    "join_meshes",
    "duplicate_linked",
    "unlink_groups",
    "set_cordon",
    "toggle_cordon",
    "clear_cordon",
    "new_tab",
    "next_tab",
    "close_tab",
    "hotspot_texture",
    "terrain_auto_paint",
    "reload_models",
    "reload_materials",
    "insert_prefab",
    "move_vertices",
    "clip_apply",
];

/// map_file ops that write the file at their path.
pub const WRITE_OPS: &[&str] = &["save", "export_map", "export_glb", "export_obj"];

pub fn tool_allowed(tool: &str) -> bool {
    TOOLS.contains(&tool)
}

/// Why a step may not pass `args` to `tool`, beyond what [`tool_allowed`] and the paths checked by run_script refuse.
pub fn args_refusal(tool: &str, args: &Value) -> Option<String> {
    match tool {
        "run_action" => {
            let action = args["action"].as_str().unwrap_or_default();
            let flag = |key: &str| args["args"][key].as_bool() == Some(true);
            match action {
                "copy" | "cut" => Some(format!("run_action {action} cannot run inside scripts, it writes the system clipboard")),
                "close_tab" if flag("discard") || flag("close_others") => {
                    Some("run_action close_tab cannot discard changes or close other tabs inside scripts, only close the active tab once it is saved".into())
                }
                _ => (!ACTIONS.contains(&action)).then(|| format!("run_action {action} cannot run inside scripts, only actions that edit the map can")),
            }
        }
        "set_editor" => {
            ["live_mode", "live_link_port", "mcp"].into_iter().find(|k| !args[*k].is_null()).map(|k| format!("set_editor {k} cannot be set inside scripts"))
        }
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Access {
    Read,
    Write,
}

/// The files and folders a step names in `args` and what it does with them, as written. run_script resolves and
/// checks them before the step runs. A save without a path writes the open map's file, which run_script adds itself.
pub fn file_args<'a>(tool: &str, args: &'a Value) -> Vec<(Access, &'a str)> {
    let text = |key: &str| args[key].as_str().filter(|s| !s.is_empty());
    let read = |key: &str| text(key).map(|p| (Access::Read, p));
    match (tool, args["op"].as_str().unwrap_or_default()) {
        ("map_file", "import_map" | "import_vmf") => read("path").into_iter().chain(read("textures").filter(|(_, t)| *t != "auto")).collect(),
        ("map_file", "open" | "open_tab" | "convert_textures") | ("import_model" | "validate_map", _) => read("path").into_iter().collect(),
        ("map_file", op) if WRITE_OPS.contains(&op) => text("path").map(|p| (Access::Write, p)).into_iter().collect(),
        ("changes_since", _) => read("file").into_iter().collect(),
        ("run_action", _) if args["action"] == "insert_prefab" => args["args"]["path"].as_str().map(|p| (Access::Read, p)).into_iter().collect(),
        _ => Vec::new(),
    }
}

/// Whether a step with `args` writes textures into the project's texture and material folders.
pub fn writes_textures(tool: &str, args: &Value) -> bool {
    match (tool, args["op"].as_str().unwrap_or_default()) {
        ("map_file", "convert_textures") => true,
        ("map_file", "import_map" | "import_vmf") => args["textures"].is_string(),
        ("import_model", _) => args["mode"].as_str() != Some("prop"),
        _ => false,
    }
}

/// `path` leads to another machine or a Windows device, told by its text alone. A verbatim path to a local drive, the
/// way canonical Windows paths start, stays local.
pub fn network(path: &Path) -> bool {
    let text = path.to_string_lossy().replace('/', "\\");
    let drive = text.strip_prefix(r"\\?\").is_some_and(|rest| matches!(rest.as_bytes(), [letter, b':', ..] if letter.is_ascii_alphabetic()));
    !drive && td_formats::game::is_network_path(&text)
}

/// A step may read `path`: it is local and inside one of `folders`, see [`inside`].
pub fn may_read(path: &Path, folders: &[PathBuf]) -> bool {
    !network(path) && inside(path, folders)
}

/// The folders the steps of a running run_script may read and write in. The editor asks it before each file a step
/// leads to is read or written, so a link inside one of the folders cannot lead out of them.
#[derive(Clone, Debug)]
pub struct Fence(std::sync::Arc<[PathBuf]>);

impl Fence {
    pub fn new(folders: Vec<PathBuf>) -> Self {
        Self(folders.into())
    }

    pub fn may_read(&self, path: &Path) -> bool {
        may_read(path, &self.0)
    }

    /// A file may be written at `path`, which is local and lies below one of the folders, see [`file_inside`].
    pub fn may_write(&self, path: &Path) -> bool {
        !network(path) && file_inside(path, &self.0)
    }
}

/// The maps a script works on and how far undo and redo may go in each: back to where the map stood when the script
/// came to it, and forward over what the script took back itself, so unsaved work from before the script stays.
#[derive(Default)]
pub struct Rewinds(Vec<(td_doc::document::HistoryMark, usize)>);

impl Rewinds {
    /// Notes `doc`, the active map, before and after each step. A map seen for the first time is marked where its
    /// history stands. Its redo steps from before the script stay the person's until an edit clears them.
    pub fn note(&mut self, doc: &td_doc::Document) {
        let redo = doc.history.redo_labels().count();
        match self.0.iter_mut().find(|(mark, _)| doc.owns(mark)) {
            Some((_, kept)) => *kept = (*kept).min(redo),
            None => self.0.push((doc.mark(), redo)),
        }
    }

    /// Where the script began working on `doc`.
    pub fn mark(&self, doc: &td_doc::Document) -> Option<td_doc::document::HistoryMark> {
        self.0.iter().find(|(mark, _)| doc.owns(mark)).map(|(mark, _)| *mark)
    }

    /// Why `steps` of undo, or of redo, may not run on `doc`.
    pub fn refusal(&self, doc: &td_doc::Document, undo: bool, steps: usize) -> Option<String> {
        let (mark, kept) = self.0.iter().find(|(mark, _)| doc.owns(mark))?;
        if undo {
            let room = doc.steps_since(mark);
            (steps > room).then(|| format!("run_action undo can only take back what the script did in this map, {room} steps"))
        } else {
            let room = doc.history.redo_labels().count().saturating_sub(*kept);
            (steps > room).then(|| format!("run_action redo can only bring back what the script took back in this map, {room} steps"))
        }
    }
}

/// The files a map save to `path` writes: the map, the file it goes through on the way and the backup of the one it
/// replaces.
pub fn save_files(path: &Path) -> [PathBuf; 3] {
    let ext = td_doc::legacy::map_ext(path);
    [path.to_path_buf(), path.with_extension(format!("{ext}.tmp")), path.with_extension(format!("{ext}.bak"))]
}

/// Why a step may not write `path`, for an error.
pub fn write_refused(path: &Path) -> String {
    format!("{} is outside the project, the script's folder and the folders passed in vars, scripts cannot write there", super::tools::path_text(path))
}

/// `path` lies inside one of `folders` or is one of them, also once links are resolved. The text is compared first, so
/// a path anywhere else, such as on a network share, is never looked up.
pub fn inside(path: &Path, folders: &[PathBuf]) -> bool {
    within(path, folders, false)
}

/// `path` lies below one of `folders`, so a file written there stays in it, see [`inside`].
pub fn file_inside(path: &Path, folders: &[PathBuf]) -> bool {
    within(path, folders, true)
}

fn within(path: &Path, folders: &[PathBuf], below: bool) -> bool {
    let within = |path: &Path, folder: &Path| {
        let (path, folder) = (plain(path), plain(folder));
        (!below && path == folder) || path.starts_with(&format!("{folder}/"))
    };
    // Note: only a network path is refused on its text alone, so it is never opened. Local paths are judged by where
    // they really lead, since Windows spells one folder several ways (8.3 short names such as RUNNER~1).
    if network(path) && !folders.iter().any(|f| within(&lexical(path), &lexical(f))) {
        return false;
    }

    let real = crate::trust::real_path(path);
    folders.iter().any(|f| within(&real, &crate::trust::real_path(f)))
}

/// `path` is a drive or filesystem root, too much for a script to write into.
pub fn is_root(path: &Path) -> bool {
    lexical(path).parent().is_none()
}

/// `path` made absolute with `.` and `..` taken out as written.
fn lexical(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in crate::state::absolute(path).components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            c => out.push(c),
        }
    }

    out
}

/// The text of a path in one slash style, without the `\\?\` of a canonical Windows path, which scripts spell `//?/`.
fn plain(path: &Path) -> String {
    let text = path.to_string_lossy().replace('\\', "/");
    let text = text.strip_prefix("//?/").unwrap_or(&text).trim_end_matches('/').to_string();
    if cfg!(windows) { text.to_lowercase() } else { text }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub tool: String,
    pub args: Value,
    /// Name the result is stored under.
    pub save: Option<String>,
}

/// The raw step entries of a script document, tool calls and notes together.
fn entries(doc: &Value) -> Option<&Vec<Value>> {
    match doc {
        Value::Array(a) => Some(a),
        Value::Object(o) => o.get("steps").and_then(|s| s.as_array()),
        _ => None,
    }
}

/// Steps of a script document: `{"format": ..., "steps": [...]}` or a bare array of steps.
pub fn parse(doc: &Value) -> Result<Vec<Step>, String> {
    if let Value::Object(o) = doc
        && let Some(f) = o.get("format").and_then(|f| f.as_str())
        && f != FORMAT
    {
        return Err(format!("not a TrenchDot MCP script (format {f})"));
    }

    let steps = match doc {
        Value::Array(_) | Value::Object(_) => entries(doc).ok_or("script needs a steps array")?,
        _ => return Err("script must be an object or an array".into()),
    };
    steps
        .iter()
        .enumerate()
        .filter(|(_, s)| s.get("tool").is_some())
        .map(|(i, s)| {
            let tool = s["tool"].as_str().ok_or_else(|| format!("step {i}: tool must be a string"))?.to_string();
            Ok(Step {
                tool,
                args: s.get("args").cloned().unwrap_or(Value::Object(Default::default())),
                save: s.get("save").and_then(|v| v.as_str()).map(str::to_string),
            })
        })
        .collect()
}

/// Entries with no `tool`, dropped from `parse`'s result. `run_script` reports these separately so a script with
/// comment steps does not look like it silently lost some of its work.
pub fn count_notes(doc: &Value) -> usize {
    entries(doc).map_or(0, |a| a.iter().filter(|s| s.get("tool").is_none()).count())
}

fn lookup<'a>(vars: &'a BTreeMap<String, Value>, path: &str) -> Option<&'a Value> {
    let mut parts = path.split('.');
    let mut cur = vars.get(parts.next()?)?;
    for p in parts {
        cur = match p.parse::<usize>() {
            Ok(i) if cur.is_array() => cur.get(i)?,
            _ => cur.get(p)?,
        };
    }

    Some(cur)
}

fn text_of(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// The path of a whole-string reference: `$name` or `$name.path`, the name starting with a letter or underscore.
fn whole_reference(s: &str) -> Option<&str> {
    let path = s.strip_prefix('$')?;
    let first = path.chars().next()?;
    let ident = (first.is_ascii_alphabetic() || first == '_') && path.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.');
    ident.then_some(path)
}

fn missing(vars: &BTreeMap<String, Value>, path: &str) -> String {
    let name = path.split('.').next().unwrap_or(path);
    if vars.contains_key(name) {
        return format!("${path}: {name} has no value at that path");
    }

    if name == "project" {
        return "$project is not set because no Godot project is open, call open_project first".into();
    }

    format!("unknown variable ${path}, write $$ for a literal $")
}

/// The known variable name right before a `/` in `s`, such as `project` in `"project/maps"`. Catches a script
/// author's `$name/rest` where `${name}/rest` was meant, since unbraced it never expands and passes through as a
/// literal `$name/rest` string, usually breaking whatever the file path was supposed to be.
fn known_name_before_slash<'a>(s: &'a str, vars: &BTreeMap<String, Value>) -> Option<&'a str> {
    let first = s.chars().next()?;
    (first.is_ascii_alphabetic() || first == '_').then_some(())?;
    let end = s.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).unwrap_or(s.len());
    let (name, rest) = s.split_at(end);
    (rest.starts_with('/') && (vars.contains_key(name) || name == "project")).then_some(name)
}

/// `"$a.ids + $b.ids"`: the reference paths of a string made only of whole references joined by `+`.
fn joined_references(s: &str) -> Option<Vec<&str>> {
    if !s.contains('+') {
        return None;
    }

    s.split('+').map(|part| whole_reference(part.trim())).collect()
}

/// Replaces variable references in `args`. Unknown names are an error so typos do not silently pass, `$$` is a literal `$`,
/// and a `$` that starts no reference (such as `$5`) stays as it is.
pub fn resolve(args: &Value, vars: &BTreeMap<String, Value>) -> Result<Value, String> {
    Ok(match args {
        Value::String(s) => {
            if let Some(path) = whole_reference(s) {
                return lookup(vars, path).cloned().ok_or_else(|| missing(vars, path));
            }

            if let Some(paths) = joined_references(s) {
                let mut out = Vec::new();
                for path in paths {
                    match lookup(vars, path).ok_or_else(|| missing(vars, path))? {
                        Value::Array(a) => out.extend(a.iter().cloned()),
                        Value::Null => {}
                        other => out.push(other.clone()),
                    }
                }

                return Ok(Value::Array(out));
            }

            if !s.contains('$') {
                return Ok(args.clone());
            }

            let mut out = String::new();
            let mut rest = s.as_str();
            while let Some(i) = rest.find('$') {
                out.push_str(&rest[..i]);
                let after = &rest[i + 1..];
                if let Some(r) = after.strip_prefix('$') {
                    out.push('$');
                    rest = r;
                } else if let Some(r) = after.strip_prefix('{') {
                    let end = r.find('}').ok_or_else(|| format!("unclosed ${{ in {s}"))?;
                    let path = &r[..end];
                    out.push_str(&text_of(lookup(vars, path).ok_or_else(|| missing(vars, path))?));
                    rest = &r[end + 1..];
                } else {
                    if let Some(name) = known_name_before_slash(after, vars) {
                        return Err(format!("${name}/... in {s} is not expanded, write ${{{name}}}/... instead"));
                    }

                    out.push('$');
                    rest = after;
                }
            }

            out.push_str(rest);
            Value::String(out)
        }
        Value::Array(a) => Value::Array(a.iter().map(|v| resolve(v, vars)).collect::<Result<_, _>>()?),
        Value::Object(o) => Value::Object(o.iter().map(|(k, v)| Ok((k.clone(), resolve(v, vars)?))).collect::<Result<_, String>>()?),
        other => other.clone(),
    })
}

/// Keys of tool results whose values are node ids, alone or in (nested) arrays. Counts such as hierarchy's `moved` are
/// not ids.
fn is_id_key(key: &str) -> bool {
    matches!(
        key,
        "id" | "ids"
            | "selection"
            | "selected"
            | "nodes"
            | "letters"
            | "copies"
            | "entity"
            | "group"
            | "parent"
            | "insert_parent"
            | "current_layer"
            | "open_groups"
            | "targets"
    ) || key.ends_with("_id")
        || key.ends_with("_ids")
}

/// The `replaced` map of a tool result, `{"old id": [new ids]}`.
pub fn replacements(result: &Value) -> BTreeMap<u64, Vec<u64>> {
    let Some(map) = result.get("replaced").and_then(Value::as_object) else { return BTreeMap::new() };
    map.iter().filter_map(|(old, new)| Some((old.parse().ok()?, new.as_array()?.iter().filter_map(Value::as_u64).collect()))).collect()
}

fn rewrite_ids(v: &mut Value, replaced: &BTreeMap<u64, Vec<u64>>) {
    match v {
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for mut item in std::mem::take(items) {
                match item.as_u64().and_then(|id| replaced.get(&id)) {
                    Some(new) => out.extend(new.iter().map(|n| Value::from(*n))),
                    None => {
                        rewrite_ids(&mut item, replaced);
                        out.push(item);
                    }
                }
            }

            *items = out;
        }
        Value::Number(n) => {
            if let Some(new) = n.as_u64().and_then(|id| replaced.get(&id)) {
                *v = match new.as_slice() {
                    [] => Value::Null,
                    [one] => Value::from(*one),
                    many => Value::from(many.to_vec()),
                };
            }
        }

        // list_nodes keeps whole node records under `nodes`.
        Value::Object(_) => follow_replacements(v, replaced),
        _ => {}
    }
}

/// Rewrites the node ids inside a saved result after a later step replaced some of them. Ids are the values under the
/// keys [`is_id_key`] lists and keys ending in `_id` or `_ids`, at any depth. In a list
/// a replaced id gives way to all of its replacements in place (none when it was removed, like a CSG cutter), a
/// single id becomes its one replacement, the list of them when there are several, and null when it was removed.
pub fn follow_replacements(saved: &mut Value, replaced: &BTreeMap<u64, Vec<u64>>) {
    if replaced.is_empty() {
        return;
    }

    match saved {
        Value::Object(o) => {
            for (k, v) in o.iter_mut() {
                if is_id_key(k) && !v.is_object() {
                    rewrite_ids(v, replaced);
                } else if k != "replaced" {
                    follow_replacements(v, replaced);
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(|v| follow_replacements(v, replaced)),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn steps_only_edit_the_map() {
        for tool in ["open_project", "run_script", "screenshot", "simulate_input", "walkability"] {
            assert!(!tool_allowed(tool), "{tool}");
        }

        assert!(tool_allowed("create_brush") && tool_allowed("map_file"));
        for action in ["run_godot_project", "open_godot_editor", "focus_godot", "build_in_godot", "toggle_live_mode", "toggle_walkable", "reload_game_config"] {
            assert!(args_refusal("run_action", &json!({ "action": action })).is_some(), "{action}");
        }

        assert_eq!(args_refusal("run_action", &json!({ "action": "csg_subtract" })), None);
        assert!(args_refusal("set_editor", &json!({ "grid": 8, "live_mode": true })).unwrap().contains("live_mode"));
        assert!(args_refusal("set_editor", &json!({ "mcp": false })).unwrap().contains("mcp"));
        assert_eq!(args_refusal("set_editor", &json!({ "grid": 8 })), None);
        for action in ["copy", "cut"] {
            assert!(args_refusal("run_action", &json!({ "action": action })).unwrap().contains("clipboard"), "{action}");
        }

        for args in [json!({ "discard": true }), json!({ "close_others": true }), json!({ "close_others": true, "discard": true })] {
            assert!(args_refusal("run_action", &json!({ "action": "close_tab", "args": args })).is_some(), "{args}");
        }

        assert_eq!(args_refusal("run_action", &json!({ "action": "close_tab", "args": { "discard": false } })), None);
    }

    #[test]
    fn steps_name_the_files_they_read_and_write() {
        let files = |tool: &str, args: Value| file_args(tool, &args).into_iter().map(|(a, p)| (a, p.to_string())).collect::<Vec<_>>();
        let read = |p: &str| (Access::Read, p.to_string());
        assert_eq!(files("map_file", json!({ "op": "import_map", "path": "a.map", "textures": "b.wad" })), [read("a.map"), read("b.wad")]);
        assert_eq!(files("map_file", json!({ "op": "import_vmf", "path": "a.vmf", "textures": "auto" })), [read("a.vmf")]);
        assert_eq!(files("map_file", json!({ "op": "convert_textures", "path": "evil.wad" })), [read("evil.wad")]);
        assert_eq!(files("map_file", json!({ "op": "export_obj", "path": "a.obj" })), [(Access::Write, "a.obj".to_string())]);
        assert_eq!(files("map_file", json!({ "op": "save" })), [], "run_script adds the open map's file");
        assert_eq!(files("import_model", json!({ "path": "m.glb" })), [read("m.glb")]);
        assert_eq!(files("validate_map", json!({ "path": "m.tdm" })), [read("m.tdm")]);
        assert_eq!(files("changes_since", json!({ "file": "old.tdm" })), [read("old.tdm")]);
        assert_eq!(files("run_action", json!({ "action": "insert_prefab", "args": { "path": "p.tdm" } })), [read("p.tdm")]);
        assert_eq!(files("create_brush", json!({ "path": "x" })), []);

        assert!(writes_textures("map_file", &json!({ "op": "convert_textures" })));
        assert!(writes_textures("map_file", &json!({ "op": "import_map", "textures": "auto" })));
        assert!(!writes_textures("map_file", &json!({ "op": "import_map" })), "without textures it only counts them");
        assert!(writes_textures("import_model", &json!({ "mode": "mesh" })) && !writes_textures("import_model", &json!({ "mode": "prop" })));
    }

    #[test]
    fn network_paths_are_told_by_their_text() {
        for remote in [r"\\host\share\x.tdm", "//host/share/x.tdm", r"\\?\UNC\host\share\x.tdm", r"\\.\pipe\x"] {
            assert!(network(Path::new(remote)), "{remote}");
        }

        for local in [r"\\?\D:\game\x.tdm", "//?/D:/game/x.tdm", "D:/game/x.tdm", "/home/me/x.tdm", "x.tdm"] {
            assert!(!network(Path::new(local)), "{local}");
        }
    }

    #[test]
    fn writes_stay_inside_the_allowed_folders() {
        let dir = std::env::temp_dir().join(format!("td_script_inside_{}", std::process::id()));
        let project = dir.join("project");
        std::fs::create_dir_all(project.join("maps")).unwrap();
        let folders = [project.clone()];
        assert!(inside(&project.join("maps/level.tdm"), &folders));
        assert!(inside(&project.join("new/level.tdm"), &folders), "a folder that does not exist yet");
        assert!(inside(&project, &folders));
        assert!(!inside(&project.join("../level.tdm"), &folders));
        assert!(!inside(&project.join("maps/../../level.tdm"), &folders));
        assert!(!inside(&dir.join("project_old/level.tdm"), &folders), "a sibling sharing the name's start");
        assert!(!inside(Path::new("//server/share/level.tdm"), &folders));
        let spelled = project.join("maps/level.tdm").to_string_lossy().replace('\\', "/");
        assert!(inside(Path::new(&spelled), &folders), "{spelled}");
        let canonical = project.canonicalize().unwrap();
        assert!(inside(&project.join("maps/level.tdm"), std::slice::from_ref(&canonical)));
        let scripted = format!("{}/maps/level.tdm", canonical.to_string_lossy().replace('\\', "/"));
        assert!(inside(Path::new(&scripted), &folders), "{scripted}");
        assert!(!is_root(&project) && is_root(&project.join("../".repeat(project.components().count()))));
        assert!(file_inside(&project.join("maps/level.tdm"), &folders) && !file_inside(&project, &folders), "a file goes below a folder, not in its place");
        assert!(!file_inside(&project.join("maps/.."), &folders));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn a_link_out_of_the_folder_is_followed() {
        let dir = std::env::temp_dir().join(format!("td_script_link_{}", std::process::id()));
        let (project, outside) = (dir.join("project"), dir.join("outside"));
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(outside.join("deep")).unwrap();
        std::os::unix::fs::symlink(outside.join("deep"), project.join("link")).unwrap();
        let folders = [project.clone()];
        assert!(!inside(&project.join("link/level.tdm"), &folders));
        assert!(!inside(&project.join("link/../level.tdm"), &folders));
        let fence = Fence::new(folders.to_vec());
        assert!(!fence.may_read(&project.join("link/secret.png")) && !fence.may_write(&project.join("link/sub/x.png")));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_fence_reads_inside_its_folders_and_writes_below_them() {
        let dir = std::env::temp_dir().join(format!("td_script_fence_{}", std::process::id()));
        let project = dir.join("project");
        std::fs::create_dir_all(&project).unwrap();
        let fence = Fence::new(vec![project.clone()]);
        assert!(fence.may_read(&project) && fence.may_read(&project.join("maps/a.tdm")));
        assert!(fence.may_write(&project.join("maps/a.tdm")) && !fence.may_write(&project), "a file goes below a folder");
        assert!(!fence.may_read(&dir.join("secret.png")) && !fence.may_write(&dir.join("secret.png")));
        assert!(!fence.may_read(Path::new("//host/share/project/a.tdm")) && !fence.may_write(Path::new(r"\\host\share\a.tdm")));
        assert_eq!(save_files(&project.join("a.tdm")).map(|p| p.file_name().unwrap().to_string_lossy().into_owned()), ["a.tdm", "a.tdm.tmp", "a.tdm.bak"]);
        let old = td_doc::legacy::MAP_EXTS[1];
        let names = save_files(&project.join(format!("a.{old}"))).map(|p| p.file_name().unwrap().to_string_lossy().into_owned());
        assert_eq!(names, [format!("a.{old}"), format!("a.{old}.tmp"), format!("a.{old}.bak")]);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn undo_and_redo_keep_to_what_the_script_did() {
        let mut doc = td_doc::Document::new();
        let add = |doc: &mut td_doc::Document| {
            let layer = doc.map.default_layer();
            doc.edit("add", |m, _| m.insert(layer, td_doc::NodeKind::Entity(td_doc::Entity::new("light"))))
        };
        add(&mut doc);
        add(&mut doc);
        doc.undo();
        let mut rewinds = Rewinds::default();
        rewinds.note(&doc);
        assert!(rewinds.refusal(&doc, true, 1).is_some(), "the person's unsaved work stays");
        assert!(rewinds.refusal(&doc, false, 1).is_some(), "and so does what they took back");
        assert_eq!(rewinds.refusal(&doc, true, 0), None);

        add(&mut doc);
        add(&mut doc);
        rewinds.note(&doc);
        assert_eq!(rewinds.refusal(&doc, true, 2), None);
        assert!(rewinds.refusal(&doc, true, 3).is_some());
        doc.undo();
        rewinds.note(&doc);
        assert_eq!(rewinds.refusal(&doc, false, 1), None, "what the script took back it may bring back");
        assert!(rewinds.refusal(&doc, false, 2).is_some());

        let mut other = td_doc::Document::new();
        add(&mut other);
        rewinds.note(&other);
        assert!(rewinds.refusal(&other, true, 1).is_some(), "another tab is marked where the script finds it");
        assert!(rewinds.mark(&doc).is_some_and(|m| doc.owns(&m)) && rewinds.mark(&other).is_some_and(|m| other.owns(&m)));
    }

    #[test]
    fn parses_and_resolves_references() {
        let doc = json!({ "format": FORMAT, "steps": [
            { "note": "comments are steps without a tool" },
            { "tool": "create_brush", "args": { "min": [0, 0, 0], "max": [8, 8, 8] }, "save": "wall" },
            { "tool": "select", "args": { "ids": ["$wall.ids.0"], "label": "wall ${wall.ids.0} done" } }
        ]});
        let steps = parse(&doc).unwrap();
        assert_eq!(steps.len(), 2);
        let mut vars = BTreeMap::new();
        vars.insert("wall".to_string(), json!({ "ids": [42] }));
        let args = resolve(&steps[1].args, &vars).unwrap();
        assert_eq!(args, json!({ "ids": [42], "label": "wall 42 done" }));
        assert!(resolve(&json!("$missing"), &vars).is_err());
        assert_eq!(resolve(&json!("costs $5 each"), &vars).unwrap(), json!("costs $5 each"));
    }

    #[test]
    fn dollar_escapes_and_clear_errors() {
        let mut vars = BTreeMap::new();
        vars.insert("wall".to_string(), json!({ "ids": [42] }));
        assert_eq!(resolve(&json!("$5"), &vars).unwrap(), json!("$5"));
        assert_eq!(resolve(&json!("$$Node"), &vars).unwrap(), json!("$Node"));
        assert_eq!(resolve(&json!("get_node($$Door) ${wall.ids.0} $$${wall.ids.0}"), &vars).unwrap(), json!("get_node($Door) 42 $42"));
        assert!(resolve(&json!("$Node"), &vars).unwrap_err().contains("$$"));
        assert!(resolve(&json!("$wall.nope"), &vars).unwrap_err().contains("no value"));
        assert!(resolve(&json!("${project}/maps"), &vars).unwrap_err().contains("open_project"));
        assert_eq!(resolve(&json!("a $ b"), &vars).unwrap(), json!("a $ b"));
    }

    #[test]
    fn unbraced_known_name_before_a_slash_is_an_error() {
        let mut vars = BTreeMap::new();
        vars.insert("wall".to_string(), json!({ "ids": [42] }));
        let err = resolve(&json!("$project/maps/room.tdm"), &vars).unwrap_err();
        assert!(err.contains("${project}"), "{err}");
        let err = resolve(&json!("$wall/x"), &vars).unwrap_err();
        assert!(err.contains("${wall}"), "{err}");
        // Not a known name, or no slash right after it: passes through as plain text.
        assert_eq!(resolve(&json!("$nope/x"), &vars).unwrap(), json!("$nope/x"));
        assert_eq!(resolve(&json!("$wall-x"), &vars).unwrap(), json!("$wall-x"));
    }

    #[test]
    fn plus_joins_saved_values_into_one_list() {
        let mut vars = BTreeMap::new();
        vars.insert("a".to_string(), json!({ "ids": [1, 2], "id": 9 }));
        vars.insert("b".to_string(), json!({ "ids": [3] }));
        assert_eq!(resolve(&json!({ "ids": "$a.ids + $b.ids" }), &vars).unwrap(), json!({ "ids": [1, 2, 3] }));
        assert_eq!(resolve(&json!("$a.ids+$a.id+$b.ids"), &vars).unwrap(), json!([1, 2, 9, 3]));
        assert!(resolve(&json!("$a.ids + $nope.ids"), &vars).unwrap_err().contains("unknown variable"));
        assert_eq!(resolve(&json!("1 + 2"), &vars).unwrap(), json!("1 + 2"), "plain text with a plus stays text");
        assert_eq!(resolve(&json!("$5 + $6"), &vars).unwrap(), json!("$5 + $6"));
        assert_eq!(resolve(&json!("${a.id} + ${b.ids.0}"), &vars).unwrap(), json!("9 + 3"));
    }

    #[test]
    fn saved_ids_follow_replaced_brushes() {
        let result = json!({ "ok": true, "replaced": { "10": [20, 21], "11": [], "12": [22] } });
        let replaced = replacements(&result);
        assert_eq!(replaced[&10], vec![20, 21]);
        let mut wall = json!({ "ids": [10, 11, 12, 13], "entity": null, "letters": [[10], [13]], "bounds": { "min": [10, 11, 12] } });
        follow_replacements(&mut wall, &replaced);
        assert_eq!(wall["ids"], json!([20, 21, 22, 13]), "pieces take the place of the cut brush, the cutter drops out");
        assert_eq!(wall["letters"], json!([[20, 21], [13]]));
        assert_eq!(wall["bounds"]["min"], json!([10, 11, 12]), "numbers that are not ids stay");
        let mut single = json!({ "id": 12, "other_id": 10, "gone_id": 11, "count": 10, "faces": [[10, 3]] });
        follow_replacements(&mut single, &replaced);
        assert_eq!(single, json!({ "id": 22, "other_id": [20, 21], "gone_id": null, "count": 10, "faces": [[10, 3]] }));
    }

    #[test]
    fn every_key_that_carries_node_ids_follows_replacements() {
        let replaced = replacements(&json!({ "replaced": { "10": [20, 21], "12": [22] } }));
        let mut saved = json!({
            "selected": [10, 13], "group": 12, "insert_parent": 12, "targets": [10], "parent": 12, "open_groups": [12],
            "nodes": [{ "id": 10, "parent": 12, "selected": true, "children": 10 }], "moved": 10,
            "set": { "targets": [12], "layer": 10 },
        });
        follow_replacements(&mut saved, &replaced);
        assert_eq!(saved["selected"], json!([20, 21, 13]));
        assert_eq!(saved["group"], 22);
        assert_eq!(saved["insert_parent"], 22);
        assert_eq!(saved["targets"], json!([20, 21]));
        assert_eq!(saved["parent"], 22);
        assert_eq!(saved["open_groups"], json!([22]));
        assert_eq!(saved["nodes"], json!([{ "id": [20, 21], "parent": 22, "selected": true, "children": 10 }]), "records keep their counts");
        assert_eq!(saved["moved"], 10, "hierarchy reparent's moved is a count");
        assert_eq!(saved["set"], json!({ "targets": [22], "layer": 10 }));
    }
}
