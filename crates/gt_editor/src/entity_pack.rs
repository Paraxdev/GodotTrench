//! The Gameplay entities pack: doors, buttons, lifts, triggers, logic, spawners, props and effects. The addon carries it
//! as a template in `addons/func_godot/gameplay_pack`, which a `.gdignore` hides from Godot, and installing it copies
//! the definitions and scripts into `res://godottrench/entities`. The project owns those copies, so nothing the addon
//! ships shares a classname or a class_name with them, and an update only replaces the files nobody edited.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use gt_formats::{EntityDef, GameConfig};
use serde::{Deserialize, Serialize};

/// The template's folder inside the addon.
pub const TEMPLATE_IN_ADDON: &str = "gameplay_pack";
/// Where the pack is installed, relative to the project.
pub const DIR: &str = "godottrench/entities";
pub const RES_DIR: &str = "res://godottrench/entities";
/// Describes the pack, and in a project also what was installed.
pub const MANIFEST: &str = "pack.json";
/// The pack's FuncGodotFGDFile, which the addon's core FGD reads when it is there.
pub const FGD: &str = "gameplay_fgd.tres";
pub const FORMAT: &str = "godottrench-entity-pack";
pub const NAME: &str = "Gameplay entities";

/// Template files that are not copied into the project.
const NOT_COPIED: [&str; 2] = [".gdignore", MANIFEST];

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Manifest {
    pub format: String,
    pub name: String,
    /// The pack's definitions as the editor reads them, so it knows them before Godot exports the game config again.
    #[serde(default)]
    pub entities: Vec<EntityDef>,
    /// In a project: each installed file with the SHA-256 it had when installed, which tells edited files apart.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub files: BTreeMap<String, String>,
}

impl Manifest {
    pub fn template(entities: Vec<EntityDef>) -> Manifest {
        Manifest { format: FORMAT.into(), name: NAME.into(), entities, files: BTreeMap::new() }
    }

    pub fn read(path: &Path) -> Option<Manifest> {
        let manifest: Manifest = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
        (manifest.format == FORMAT).then_some(manifest)
    }

    /// Only the installed files of a project's manifest. Godot's JSON writes every number as a float, so a manifest the
    /// addon wrote may not read as a whole.
    pub fn read_files(path: &Path) -> Option<BTreeMap<String, String>> {
        let value: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
        if value["format"] != FORMAT {
            return None;
        }

        serde_json::from_value(value.get("files").cloned().unwrap_or_default()).ok().or(Some(BTreeMap::new()))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("manifest serializes") + "\n"
    }
}

pub fn template_dir(root: &Path) -> PathBuf {
    root.join(crate::addon_install::DIR).join(TEMPLATE_IN_ADDON)
}

pub fn install_dir(root: &Path) -> PathBuf {
    root.join(DIR)
}

/// The project's addon carries the pack.
pub fn available(root: &Path) -> bool {
    template_dir(root).join(MANIFEST).is_file()
}

pub fn installed(root: &Path) -> bool {
    Manifest::read_files(&install_dir(root).join(MANIFEST)).is_some()
}

fn sha256(bytes: &[u8]) -> String {
    ring::digest::digest(&ring::digest::SHA256, bytes).as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

/// Every file of the template that is copied, as (path relative to the template with forward slashes, contents).
fn template_files(template: &Path) -> std::io::Result<Vec<(String, Vec<u8>)>> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            let rel = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
            if path.is_dir() {
                walk(root, &path, out)?;
            } else if !NOT_COPIED.contains(&rel.as_str()) {
                out.push((rel, std::fs::read(&path)?));
            }
        }

        Ok(())
    }

    let mut out = Vec::new();
    walk(template, template, &mut out)?;
    out.sort();
    Ok(out)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Step {
    Add,
    Update,
    /// Already what the template has.
    Same,
    /// Edited since it was installed, or made by the person, so it stays.
    Keep,
    /// Like [`Step::Keep`], and the template changed since the person last installed or updated.
    KeepChanged,
    /// Deleted after it was installed, so it stays deleted.
    Deleted,
}

struct Plan {
    manifest: Manifest,
    /// Relative path, contents and what happens to the file.
    files: Vec<(String, Vec<u8>, Step)>,
    /// The hash each file is recorded with afterwards.
    record: BTreeMap<String, String>,
}

fn plan(root: &Path) -> Result<Plan, String> {
    let template = template_dir(root);
    let Some(mut manifest) = Manifest::read(&template.join(MANIFEST)) else {
        return Err("The addon in this project has no Gameplay entities pack. Update the addon first.".into());
    };
    let dir = install_dir(root);
    let before = Manifest::read_files(&dir.join(MANIFEST)).unwrap_or_default();
    let files = template_files(&template).map_err(|e| format!("Could not read {}: {e}", template.display()))?;
    let mut record = BTreeMap::new();
    let mut out = Vec::new();
    for (rel, bytes) in files {
        let new = sha256(&bytes);
        let current = std::fs::read(dir.join(&rel)).ok().map(|b| sha256(&b));
        let step = match (before.get(&rel), current) {
            (None, None) => Step::Add,
            (Some(_), None) => Step::Deleted,
            (_, Some(current)) if current == new => Step::Same,
            (Some(old), Some(current)) if current == *old => Step::Update,
            (old, Some(_)) if old == Some(&new) => Step::Keep,
            (_, Some(_)) => Step::KeepChanged,
        };
        // A kept file is recorded with the template it was kept against, so its news is told once.
        let hash = match step {
            Step::Deleted => before.get(&rel).cloned(),
            _ => Some(new.clone()),
        };
        if let Some(hash) = hash {
            record.insert(rel.clone(), hash);
        }

        out.push((rel, bytes, step));
    }

    manifest.files = BTreeMap::new();
    Ok(Plan { manifest, files: out, record })
}

/// The pack in a project, for the setup window and the Gameplay menu.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Status {
    pub available: bool,
    pub installed: bool,
    /// Files an install or update would add or replace. Edited and deleted ones are left out, they stay as they are.
    pub changes: usize,
}

impl Status {
    pub fn check(root: &Path) -> Status {
        let changes = plan(root).map(|p| p.files.iter().filter(|(_, _, s)| matches!(s, Step::Add | Step::Update)).count()).unwrap_or_default();
        Status { available: available(root), installed: installed(root), changes }
    }

    /// The addon has a newer pack than the one installed.
    pub fn update(&self) -> bool {
        self.installed && self.changes > 0
    }
}

/// What an install or update did, by path relative to the pack folder.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub added: Vec<String>,
    pub updated: Vec<String>,
    /// Edited files the template changed, which stayed as they are.
    pub kept: Vec<String>,
}

impl Report {
    pub fn summary(&self) -> String {
        let mut text = match (self.added.len(), self.updated.len()) {
            (0, 0) => format!("The {NAME} pack in {RES_DIR} is up to date."),
            (added, 0) => format!("Added {added} files of the {NAME} pack to {RES_DIR}."),
            (0, updated) => format!("Updated {updated} files of the {NAME} pack in {RES_DIR}."),
            (added, updated) => format!("Added {added} and updated {updated} files of the {NAME} pack in {RES_DIR}."),
        };
        if !self.kept.is_empty() {
            text += &format!(" Kept your edited {}, the addon has a newer version of it.", self.kept.join(", "));
        }

        text
    }
}

/// Installs the pack, or updates it: adds missing files, replaces the ones nobody edited, and leaves edited and deleted
/// files as they are. Never writes outside the pack folder.
pub fn install(root: &Path) -> Result<Report, String> {
    let plan = plan(root)?;
    let dir = install_dir(root);
    let mut report = Report::default();
    for (rel, bytes, step) in &plan.files {
        let list = match step {
            Step::Add => &mut report.added,
            Step::Update => &mut report.updated,
            Step::KeepChanged => {
                report.kept.push(rel.clone());
                continue;
            }
            _ => continue,
        };
        let path = dir.join(rel);
        let write = || -> std::io::Result<()> {
            std::fs::create_dir_all(path.parent().unwrap_or(&dir))?;
            gt_formats::write_atomic(&path, &mut bytes.as_slice()).map(|_| ())
        };
        write().map_err(|e| format!("Could not write {}: {e}", path.display()))?;
        list.push(rel.clone());
    }

    let manifest = Manifest { files: plan.record, ..plan.manifest };
    let path = dir.join(MANIFEST);
    std::fs::create_dir_all(&dir)
        .and_then(|_| gt_formats::write_atomic(&path, &mut manifest.to_json().as_bytes()).map(|_| ()))
        .map_err(|e| format!("Could not write {}: {e}", path.display()))?;
    Ok(report)
}

/// Installs or updates the pack in the open project, and lets the editor know its entities right away.
pub fn install_in_project(state: &mut crate::state::EditorState) -> Result<Report, String> {
    let root = state.game.project_root.clone().ok_or("Open a Godot project first, the Gameplay entities pack goes into it.")?;
    let report = install(&root);
    state.gameplay_pack = Status::check(&root);
    let report = report?;
    add_unexported(&mut state.game, &root);
    Ok(report)
}

/// The pack's classnames that the project's maps use, for a project that has not installed the pack.
pub fn used_by_maps(root: &Path) -> BTreeSet<String> {
    let pack = GameConfig::gameplay_pack();
    let mut used = BTreeSet::new();
    for map in crate::validate::project_maps(root) {
        let Ok(text) = std::fs::read_to_string(&map) else { continue };
        for def in &pack.entities {
            if !used.contains(&def.classname) && text.contains(&format!("\"{}\"", def.classname)) {
                used.insert(def.classname.clone());
            }
        }
    }

    used
}

/// Whether `classname` belongs to the pack.
pub fn in_pack(classname: &str) -> bool {
    static CLASSES: std::sync::OnceLock<BTreeSet<String>> = std::sync::OnceLock::new();
    CLASSES.get_or_init(|| GameConfig::gameplay_pack().entities.into_iter().map(|e| e.classname).collect()).contains(classname)
}

/// Adds the installed pack's definitions that `game` lacks when Godot has not exported the game config since the pack
/// was installed or updated, so the editor offers them right away. Returns how many it added.
pub fn add_unexported(game: &mut GameConfig, root: &Path) -> usize {
    let path = install_dir(root).join(MANIFEST);
    let Some(manifest) = Manifest::read(&path) else { return 0 };
    let modified = |p: &Path| std::fs::metadata(p).and_then(|m| m.modified()).ok();
    let exported = game.source.as_deref().and_then(modified);
    if exported.is_some_and(|e| modified(&path).is_some_and(|installed| e > installed)) {
        return 0;
    }

    let mut added = 0;
    for mut def in manifest.entities {
        def.fill_group();
        match game.entities.iter().position(|e| e.classname == def.classname) {
            None => game.entities.push(def),
            Some(i) if !defined(game, &def.classname) => game.entities[i] = def,
            Some(_) => continue,
        }

        added += 1;
    }

    added
}

/// Whether `game` defines `classname` with a script that is there. The config an older addon exported names scripts
/// that moved into the pack.
fn defined(game: &GameConfig, classname: &str) -> bool {
    game.entity(classname).is_some_and(|d| d.script.is_empty() || game.resolve_res(&d.script).is_none_or(|p| p.is_file()))
}

/// Installs the pack into the open project when its maps use entities of the pack that nothing defines, which is what
/// a project made while those entities were part of the addon looks like after an addon update. Returns a status line
/// when it installed or tried to.
pub fn install_for_maps(state: &mut crate::state::EditorState) -> Option<String> {
    let root = state.game.project_root.clone()?;
    if state.gameplay_pack.installed || !state.gameplay_pack.available {
        return None;
    }

    let need: Vec<String> = used_by_maps(&root).into_iter().filter(|c| !defined(&state.game, c)).collect();
    let named = match need.len() {
        0 => return None,
        1..=3 => need.join(", "),
        n => format!("{} and {} more", need[..3].join(", "), n - 3),
    };
    Some(match install_in_project(state) {
        Ok(report) => format!("The maps use {named}, which now come with the {NAME} pack. {}", report.summary()),
        Err(e) => format!("The maps use {named} from the {NAME} pack, which could not be installed: {e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    /// A project whose addon carries a small pack template.
    fn project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("project.godot"), "config_version=5\n").unwrap();
        let template = template_dir(dir.path());
        write(&template.join(".gdignore"), "");
        write(&template.join(MANIFEST), &Manifest::template(GameConfig::gameplay_pack().entities).to_json());
        write(&template.join(FGD), "fgd 1");
        write(&template.join("scripts/gt_door.gd"), "door 1");
        write(&template.join("scripts/gt_relay.gd"), "relay 1");
        write(&template.join("definitions/func_door.tres"), "func_door 1");
        dir
    }

    fn read(root: &Path, rel: &str) -> String {
        std::fs::read_to_string(install_dir(root).join(rel)).unwrap_or_default()
    }

    #[test]
    fn installing_copies_the_template_and_never_overwrites_edits() {
        let dir = project();
        let root = dir.path();
        assert_eq!(Status::check(root), Status { available: true, installed: false, changes: 4 });
        let report = install(root).unwrap();
        assert_eq!(report.added, ["definitions/func_door.tres", "gameplay_fgd.tres", "scripts/gt_door.gd", "scripts/gt_relay.gd"]);
        assert!(!install_dir(root).join(".gdignore").exists(), "Godot has to see the copies");
        assert_eq!(read(root, "scripts/gt_door.gd"), "door 1");
        assert_eq!(Status::check(root), Status { available: true, installed: true, changes: 0 });
        assert_eq!(install(root).unwrap(), Report::default(), "installing again changes nothing");

        // The person edits the door and deletes the relay, then an addon update changes all three and adds a file.
        write(&install_dir(root).join("scripts/gt_door.gd"), "my door");
        std::fs::remove_file(install_dir(root).join("scripts/gt_relay.gd")).unwrap();
        let template = template_dir(root);
        for (rel, text) in [("scripts/gt_door.gd", "door 2"), ("scripts/gt_relay.gd", "relay 2"), (FGD, "fgd 2"), ("scripts/gt_timer.gd", "timer 1")] {
            write(&template.join(rel), text);
        }

        let status = Status::check(root);
        assert!(status.update() && status.changes == 2, "only the untouched FGD and the new timer: {status:?}");
        let report = install(root).unwrap();
        assert_eq!(report.updated, [FGD]);
        assert_eq!(report.added, ["scripts/gt_timer.gd"]);
        assert_eq!(report.kept, ["scripts/gt_door.gd"]);
        assert!(report.summary().contains("Kept your edited scripts/gt_door.gd"), "{}", report.summary());
        assert_eq!(read(root, "scripts/gt_door.gd"), "my door", "an edit is never overwritten");
        assert!(!install_dir(root).join("scripts/gt_relay.gd").exists(), "a deleted file stays deleted");
        assert_eq!(read(root, FGD), "fgd 2");
        assert_eq!(Status::check(root).changes, 0, "no update is offered for what the person changed");
        assert!(install(root).unwrap().kept.is_empty(), "an edit is reported once per template change");

        // A file the person made before installing is kept too.
        let fresh = project();
        write(&install_dir(fresh.path()).join("scripts/gt_door.gd"), "mine");
        assert_eq!(install(fresh.path()).unwrap().kept, ["scripts/gt_door.gd"]);
        assert_eq!(read(fresh.path(), "scripts/gt_door.gd"), "mine");
    }

    #[test]
    fn without_the_template_nothing_is_installed() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(Status::check(dir.path()), Status::default());
        assert!(install(dir.path()).unwrap_err().contains("Update the addon"));
        assert!(!install_dir(dir.path()).exists());
    }

    #[test]
    fn the_editor_knows_an_installed_pack_until_godot_exports() {
        let dir = project();
        let root = dir.path();
        let mut game = GameConfig::builtin();
        assert_eq!(add_unexported(&mut game, root), 0, "not installed");
        install(root).unwrap();
        assert!(add_unexported(&mut game, root) > 30);
        assert_eq!(game.entity("func_door").unwrap().group, "func");
        assert_eq!(add_unexported(&mut game, root), 0, "known ones are not added twice");

        // A config Godot exported after the install is complete, the person may have removed entities from it.
        let exported = root.join(gt_formats::game::GAME_FILE_NAME);
        std::fs::write(&exported, "{}").unwrap();
        let later = std::fs::metadata(install_dir(root).join(MANIFEST)).unwrap().modified().unwrap() + std::time::Duration::from_secs(5);
        std::fs::File::options().write(true).open(&exported).unwrap().set_modified(later).unwrap();
        let mut game = GameConfig::builtin();
        game.source = Some(exported);
        assert_eq!(add_unexported(&mut game, root), 0);
    }

    #[test]
    fn a_project_whose_maps_use_the_pack_gets_it_when_the_addon_no_longer_has_the_entities() {
        let dir = project();
        let root = dir.path();
        let mut state = crate::state::EditorState::new(Default::default());
        state.load_project(root);
        assert!(install_for_maps(&mut state).is_none(), "no map uses the pack");
        write(&root.join("maps/old.gtm"), r#"{"classname": "func_door"}, {"classname": "my_door"}"#);

        // The config the old addon exported still names func_door, with a script the update removed.
        let mut game = GameConfig::with_gameplay_pack();
        game.entities.iter_mut().find(|e| e.classname == "func_door").unwrap().script = "res://addons/func_godot/src/godottrench/entities/gt_door.gd".into();
        game.project_root = Some(root.to_path_buf());
        state.game = game;
        let status = install_for_maps(&mut state).expect("installed");
        assert!(status.starts_with("The maps use func_door, which now come with the Gameplay entities pack. Added 4 files"), "{status}");
        assert!(state.gameplay_pack.installed && install_dir(root).join("scripts/gt_door.gd").is_file());
        assert_eq!(state.game.entity("func_door").unwrap().script, "res://godottrench/entities/scripts/gt_door.gd", "the stale definition is replaced");
        assert!(install_for_maps(&mut state).is_none(), "once");
    }

    #[test]
    fn a_project_that_defines_the_classes_itself_is_left_alone() {
        let dir = project();
        let root = dir.path();
        write(&root.join("maps/a.gtm"), r#"{"classname": "func_door"}"#);
        let config = r#"{"format": "godottrench-game", "version": 1, "entities": [{"classname": "func_door", "type": "solid"}]}"#;
        write(&root.join(gt_formats::game::GAME_FILE_NAME), config);
        let mut state = crate::state::EditorState::new(Default::default());
        state.load_project(root);
        assert!(install_for_maps(&mut state).is_none());
        assert!(!install_dir(root).exists());

        std::fs::remove_file(root.join(gt_formats::game::GAME_FILE_NAME)).unwrap();
        state.load_project(root);
        assert!(state.gameplay_pack.installed, "opening the project installs it once nothing defines func_door");
        assert!(state.status.contains("The maps use func_door"), "{}", state.status);
    }

    #[test]
    fn finds_the_pack_classes_maps_use() {
        let dir = project();
        write(&dir.path().join("maps/a.gtm"), r#"{"classname": "func_door", "x": {"classname":"light"}}"#);
        write(&dir.path().join(".godot/b.gtm"), r#""trigger_once""#);
        assert_eq!(used_by_maps(dir.path()).into_iter().collect::<Vec<_>>(), ["func_door"]);
        assert!(in_pack("func_door") && !in_pack("light") && !in_pack("worldspawn"));
    }
}
