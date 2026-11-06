//! The Gameplay entities pack: doors, buttons, lifts, triggers, logic, spawners, props and effects. The addon carries it
//! as a template in `addons/func_godot/gameplay_pack`, which a `.gdignore` hides from Godot, and installing it copies
//! the definitions and scripts into `res://trenchdot/entities`. The project owns those copies, so nothing the addon
//! ships shares a classname or a class_name with them, and an update only replaces the files nobody edited.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use td_formats::{EntityDef, GameConfig};

use crate::i18n::{Msg, en, msg_n, tr};

/// The template's folder inside the addon.
pub const TEMPLATE_IN_ADDON: &str = "gameplay_pack";
/// Where the pack is installed, relative to the project.
pub const DIR: &str = "trenchdot/entities";
pub const RES_DIR: &str = "res://trenchdot/entities";
/// Describes the pack, and in a project also what was installed.
pub const MANIFEST: &str = "pack.json";
/// The pack's FuncGodotFGDFile, which the addon's core FGD reads when it is there.
pub const FGD: &str = "gameplay_fgd.tres";
/// The addon's FGD with FuncGodot's entities, the core entities and the pack, for an FGD's Base Fgd Files.
pub const DEFAULT_FGD: &str = "res://addons/func_godot/fgd/trenchdot_default_fgd.tres";
pub const FORMAT: &str = "trenchdot-entity-pack";
pub const NAME: &str = en!("Gameplay entities");

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
    /// The Godot editor with the project open scans for the new files.
    pub scanned: bool,
}

/// Pack scripts that appear while Godot runs can be compiled before Godot knows the classes they use, and stay broken
/// until Godot starts again.
pub const RESTART_NOTE: &str = en!("If Godot has the project open, restart it, then press Build Map in each scene that uses your maps.");
pub const SCAN_NOTE: &str = en!("Godot is picking up the new files. Press Build Map in scenes that are not in live mode.");

impl Report {
    pub fn message(&self) -> Msg {
        let name = tr(NAME);
        let words = [("name", NAME, name), ("dir", RES_DIR, RES_DIR)];
        let mut text = match (self.added.len(), self.updated.len()) {
            (0, 0) => Msg::filled("The {name} pack in {dir} is up to date.", &words),
            (added, 0) => Msg::filled_n("Added {n} file of the {name} pack to {dir}.", "Added {n} files of the {name} pack to {dir}.", added, &words),
            (0, updated) => Msg::filled_n("Updated {n} file of the {name} pack in {dir}.", "Updated {n} files of the {name} pack in {dir}.", updated, &words),
            (added, updated) => {
                let added = added.to_string();
                Msg::filled_n(
                    "Added {added} and updated {n} file of the {name} pack in {dir}.",
                    "Added {added} and updated {n} files of the {name} pack in {dir}.",
                    updated,
                    &[words[0], words[1], ("added", &added, &added)],
                )
            }
        };
        if !self.kept.is_empty() {
            let files = self.kept.join(", ");
            text = text.then(
                " ",
                msg_n!(
                    self.kept.len(),
                    "Kept your edited {files}, the addon has a newer version of it.",
                    "Kept your edited {files}, the addon has a newer version of them.",
                    files
                ),
            );
        }

        if !self.added.is_empty() || !self.updated.is_empty() {
            text = text.then(" ", Msg::key(if self.scanned { SCAN_NOTE } else { RESTART_NOTE }));
        }

        text
    }

    /// [`Report::message`] in English, for MCP and tests.
    pub fn summary(&self) -> String {
        self.message().en
    }
}

/// Installs the pack, or updates it: adds missing files, replaces the ones nobody edited, and leaves edited and deleted
/// files as they are. Never writes outside the pack folder.
pub fn install(root: &Path) -> Result<Report, String> {
    install_where(root, &|_| Ok(()))
}

/// [`install`] once `may_write` allows every file it would write.
fn install_where(root: &Path, may_write: &dyn Fn(&Path) -> Result<(), String>) -> Result<Report, String> {
    let plan = plan(root)?;
    let dir = install_dir(root);
    let written = plan.files.iter().filter(|(.., step)| matches!(step, Step::Add | Step::Update)).map(|(rel, ..)| dir.join(rel));
    for path in written.chain([dir.join(MANIFEST)]) {
        may_write(&path)?;
    }

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
            td_formats::write_atomic(&path, &mut bytes.as_slice()).map(|_| ())
        };
        write().map_err(|e| format!("Could not write {}: {e}", path.display()))?;
        list.push(rel.clone());
    }

    let manifest = Manifest { files: plan.record, ..plan.manifest };
    let path = dir.join(MANIFEST);
    std::fs::create_dir_all(&dir)
        .and_then(|_| td_formats::write_atomic(&path, &mut manifest.to_json().as_bytes()).map(|_| ()))
        .map_err(|e| format!("Could not write {}: {e}", path.display()))?;
    Ok(report)
}

/// Installs or updates the pack in the open project, and lets the editor and a Godot editor with the project open know
/// its entities right away.
pub fn install_in_project(state: &mut crate::state::EditorState) -> Result<Report, String> {
    let root = state.game.project_root.clone().ok_or("Open a Godot project first, the Gameplay entities pack goes into it.")?;
    let report = install_where(&root, &|p| state.may_write(p));
    state.gameplay_pack = Status::check(&root);
    let mut report = report?;
    let before = state.defined_classes();
    add_unexported(&mut state.game, &root);
    state.forget_godot_missing(&before);
    if (!report.added.is_empty() || !report.updated.is_empty()) && state.godot_has_project() {
        state.godot_scan = crate::live_link::scan(state.prefs.live_link_port);
        report.scanned = state.godot_scan.is_some();
    }

    Ok(report)
}

/// Installs the pack before a gameplay wizard places `classes` of it that the open project does not define. Returns the
/// install, or why the pack cannot be installed. Without a project nothing is installed.
pub fn install_for_wizard(state: &mut crate::state::EditorState, classes: &[&str]) -> Result<Option<Report>, String> {
    if state.game.project_root.is_none() || state.gameplay_pack.installed {
        return Ok(None);
    }

    let lacking: Vec<&str> = classes.iter().copied().filter(|c| in_pack(c) && !defined(&state.game, c)).collect();
    if lacking.is_empty() {
        return Ok(None);
    }

    let named = format!("{} {}", crate::live_link::name_list(&lacking), if lacking.len() == 1 { "comes" } else { "come" });
    if !state.gameplay_pack.available {
        return Err(format!(
            "{named} with the {NAME} pack, which the addon in this project does not have. Update the addon, then use Gameplay > Install Gameplay Entities"
        ));
    }

    install_in_project(state)
        .map(Some)
        .map_err(|e| format!("{named} with the {NAME} pack, which could not be installed: {e} Gameplay > Install Gameplay Entities tries again"))
}

/// Runs a gameplay wizard that places `classes`, see [`install_for_wizard`]. What the install did follows the status the
/// wizard set.
pub fn with_pack<T>(
    state: &mut crate::state::EditorState,
    classes: &[&str],
    wizard: impl FnOnce(&mut crate::state::EditorState) -> Result<T, String>,
) -> Result<T, String> {
    let installed = install_for_wizard(state, classes)?;
    let result = wizard(state);
    if let (Ok(_), Some(report)) = (&result, installed) {
        let trimmed = |s: &str| s.trim_end_matches(['.', '。']).to_string();
        let status = Msg { en: trimmed(&state.status), shown: trimmed(state.status_shown()) };
        state.set_status(status.then(". ", report.message()));
    }

    result
}

/// The pack's classnames that the project's maps use, for a project that has not installed the pack. Maps are read as
/// maps, the binary files the editor saves included, and a file that does not load is searched as text.
pub fn used_by_maps(root: &Path) -> BTreeSet<String> {
    let pack = GameConfig::gameplay_pack();
    let mut used = BTreeSet::new();
    for map in crate::validate::project_maps(root) {
        if let Ok(loaded) = td_doc::format::load(&map) {
            used.extend(loaded.map.entities().filter(|(_, e)| in_pack(&e.classname)).map(|(_, e)| e.classname.clone()));
            continue;
        }

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

    // What the export says Godot builds with predates the pack, which Godot's default FGD includes once installed.
    if let Some(build) = &mut game.build {
        build.classes.extend(manifest.entities.iter().map(|e| e.classname.clone()));
        build.classes.sort();
        build.classes.dedup();
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
pub fn install_for_maps(state: &mut crate::state::EditorState) -> Option<Msg> {
    let root = state.game.project_root.clone()?;
    if state.gameplay_pack.installed || !state.gameplay_pack.available {
        return None;
    }

    let need: Vec<String> = used_by_maps(&root).into_iter().filter(|c| !defined(&state.game, c)).collect();
    let named = match need.len() {
        0 => return None,
        1..=3 => Msg::from(need.join(", ")),
        n => msg_n!(n - 3, "{first} and {n} more", "{first} and {n} more", first = need[..3].join(", ")),
    };
    Some(match install_in_project(state) {
        Ok(report) => {
            Msg::filled("The maps use {named}, which now come with the {name} pack.", &[("named", &named.en, &named.shown), ("name", NAME, tr(NAME))])
                .then(" ", report.message())
        }
        Err(e) => Msg::filled(
            "The maps use {named} from the {name} pack, which could not be installed: {e}",
            &[("named", &named.en, &named.shown), ("name", NAME, tr(NAME)), ("e", &e, &e)],
        ),
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
        write(&template.join("scripts/td_door.gd"), "door 1");
        write(&template.join("scripts/td_relay.gd"), "relay 1");
        write(&template.join("definitions/func_door.tres"), "func_door 1");
        dir
    }

    fn report(added: &[&str], updated: &[&str], kept: &[&str]) -> Report {
        let owned = |files: &[&str]| files.iter().map(|f| f.to_string()).collect();
        Report { added: owned(added), updated: owned(updated), kept: owned(kept), scanned: false }
    }

    #[test]
    fn a_report_words_added_updated_and_kept_files_per_language() {
        let _lang = crate::i18n::scope(crate::i18n::Lang::De);
        let one = report(&["a"], &[], &[]).message();
        assert!(one.en.starts_with(&format!("Added 1 file of the {NAME} pack to {RES_DIR}. ")), "{one:?}");
        assert!(one.en.ends_with(RESTART_NOTE), "{one:?}");
        assert_eq!(one.shown, format!("1 Datei des Pakets Gameplay-Entitäten nach {RES_DIR} hinzugefügt. {}", tr(RESTART_NOTE)));
        assert_ne!(tr(RESTART_NOTE), RESTART_NOTE);

        let both = report(&["a", "b"], &["c"], &[]).message();
        assert!(both.en.starts_with(&format!("Added 2 and updated 1 file of the {NAME} pack in {RES_DIR}.")), "{both:?}");
        assert!(both.shown.starts_with(&format!("2 hinzugefügt und 1 Datei des Pakets Gameplay-Entitäten in {RES_DIR} aktualisiert.")), "{both:?}");
        let updated = report(&[], &["c", "d", "e"], &[]).message();
        assert!(updated.shown.starts_with(&format!("3 Dateien des Pakets Gameplay-Entitäten in {RES_DIR} aktualisiert.")), "{updated:?}");

        let kept = report(&[], &[], &["a.gd"]).message();
        assert_eq!(kept.en, format!("The {NAME} pack in {RES_DIR} is up to date. Kept your edited a.gd, the addon has a newer version of it."));
        assert!(kept.shown.ends_with("1 bearbeitete Datei behalten (a.gd), das Addon hat eine neuere Version."), "{kept:?}");
        let kept = report(&[], &[], &["a.gd", "b.gd"]).message();
        assert!(kept.en.ends_with("Kept your edited a.gd, b.gd, the addon has a newer version of them."), "{kept:?}");
        assert!(kept.shown.ends_with("2 bearbeitete Dateien behalten (a.gd, b.gd), das Addon hat neuere Versionen."), "{kept:?}");

        let scanned = Report { scanned: true, ..report(&["a"], &[], &[]) }.message();
        assert!(scanned.en.ends_with(SCAN_NOTE) && scanned.shown.ends_with(tr(SCAN_NOTE)), "{scanned:?}");
        assert_eq!(report(&["a"], &[], &[]).summary(), one.en, "summary stays English");
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
        assert_eq!(report.added, ["definitions/func_door.tres", "gameplay_fgd.tres", "scripts/td_door.gd", "scripts/td_relay.gd"]);
        assert!(!install_dir(root).join(".gdignore").exists(), "Godot has to see the copies");
        assert_eq!(read(root, "scripts/td_door.gd"), "door 1");
        assert_eq!(Status::check(root), Status { available: true, installed: true, changes: 0 });
        assert_eq!(install(root).unwrap(), Report::default(), "installing again changes nothing");

        // The person edits the door and deletes the relay, then an addon update changes all three and adds a file.
        write(&install_dir(root).join("scripts/td_door.gd"), "my door");
        std::fs::remove_file(install_dir(root).join("scripts/td_relay.gd")).unwrap();
        let template = template_dir(root);
        for (rel, text) in [("scripts/td_door.gd", "door 2"), ("scripts/td_relay.gd", "relay 2"), (FGD, "fgd 2"), ("scripts/td_timer.gd", "timer 1")] {
            write(&template.join(rel), text);
        }

        let status = Status::check(root);
        assert!(status.update() && status.changes == 2, "only the untouched FGD and the new timer: {status:?}");
        let report = install(root).unwrap();
        assert_eq!(report.updated, [FGD]);
        assert_eq!(report.added, ["scripts/td_timer.gd"]);
        assert_eq!(report.kept, ["scripts/td_door.gd"]);
        assert!(report.summary().contains("Kept your edited scripts/td_door.gd"), "{}", report.summary());
        assert_eq!(read(root, "scripts/td_door.gd"), "my door", "an edit is never overwritten");
        assert!(!install_dir(root).join("scripts/td_relay.gd").exists(), "a deleted file stays deleted");
        assert_eq!(read(root, FGD), "fgd 2");
        assert_eq!(Status::check(root).changes, 0, "no update is offered for what the person changed");
        assert!(install(root).unwrap().kept.is_empty(), "an edit is reported once per template change");

        // A file the person made before installing is kept too.
        let fresh = project();
        write(&install_dir(fresh.path()).join("scripts/td_door.gd"), "mine");
        assert_eq!(install(fresh.path()).unwrap().kept, ["scripts/td_door.gd"]);
        assert_eq!(read(fresh.path(), "scripts/td_door.gd"), "mine");
    }

    #[test]
    fn the_summary_says_to_restart_godot_after_new_files() {
        let added = Report { added: vec!["scripts/td_door.gd".into()], ..Default::default() };
        assert_eq!(added.summary(), format!("Added 1 file of the {NAME} pack to {RES_DIR}. {RESTART_NOTE}"));
        let updated = Report { updated: vec!["a".into()], kept: vec!["b".into()], ..Default::default() };
        assert!(updated.summary().ends_with(&format!("newer version of it. {RESTART_NOTE}")), "{}", updated.summary());
        assert!(!Report::default().summary().contains("restart"), "nothing changed, nothing to pick up");
    }

    #[test]
    fn a_connected_godot_scans_for_the_installed_pack() {
        let dir = project();
        let root = dir.path();
        let mut state = crate::state::EditorState::new(Default::default());
        state.load_project(root);
        state.link_state = crate::live_link::LinkState {
            connected: true,
            project: Some(crate::live_link::path_key(&crate::live_link::godot_path(&state.game.project_root.clone().unwrap()))),
            ..Default::default()
        };
        let (port, scans) = fake_scan(serde_json::json!({ "ok": true }));
        state.prefs.live_link_port = port;
        let report = install_in_project(&mut state).unwrap();
        assert!(report.scanned && report.summary().ends_with(SCAN_NOTE), "{}", report.summary());
        assert_eq!(state.godot_scan, Some(0), "an addon that does not number its scans");
        assert_eq!(scans.recv_timeout(std::time::Duration::from_secs(5)).unwrap()["event"], "scan");
        let again = install_in_project(&mut state).unwrap();
        assert!(!again.scanned && scans.try_recv().is_err(), "nothing new, nothing to scan");

        std::fs::remove_dir_all(install_dir(root)).unwrap();
        let (port, _) = fake_scan(serde_json::json!({ "ok": false, "error": "unknown event" }));
        state.prefs.live_link_port = port;
        let report = install_in_project(&mut state).unwrap();
        assert!(!report.scanned && report.summary().ends_with(RESTART_NOTE), "an addon from before the scan event: {}", report.summary());
    }

    /// Answers one message on a port of its own with `reply`, and passes the message on.
    fn fake_scan(reply: serde_json::Value) -> (u16, std::sync::mpsc::Receiver<serde_json::Value>) {
        use std::io::{BufRead, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut line = String::new();
            std::io::BufReader::new(stream.try_clone().unwrap()).read_line(&mut line).unwrap();
            let _ = tx.send(serde_json::from_str(&line).unwrap());
            writeln!(&stream, "{reply}").unwrap();
        });
        (port, rx)
    }

    #[test]
    fn a_wizard_installs_the_pack_it_needs_first() {
        let dir = project();
        let root = dir.path();
        let mut state = crate::state::EditorState::new(Default::default());
        state.load_project(root);
        assert!(install_for_wizard(&mut state, &["light", "info_player_start"]).unwrap().is_none(), "core classes need no pack");
        let made = with_pack(&mut state, &["func_door"], |state| {
            state.set_status("func_door 'door_1' created");
            Ok(7)
        });
        assert_eq!(made, Ok(7));
        assert!(state.gameplay_pack.installed && state.game.entity("func_door").is_some());
        assert!(state.status.starts_with("func_door 'door_1' created. Added 4 files of the Gameplay entities pack"), "{}", state.status);
        assert!(install_for_wizard(&mut state, &["func_door"]).unwrap().is_none(), "once");

        let bare = tempfile::tempdir().unwrap();
        std::fs::write(bare.path().join("project.godot"), "config_version=5\n").unwrap();
        state.load_project(bare.path());
        let e = with_pack(&mut state, &["func_door", "trigger_multiple"], |_| Ok(())).unwrap_err();
        assert!(e.starts_with("func_door and trigger_multiple come with the Gameplay entities pack, which the addon in this project does not have"), "{e}");
        assert!(e.ends_with("Gameplay > Install Gameplay Entities"), "{e}");
        state.game.project_root = None;
        assert!(install_for_wizard(&mut state, &["func_door"]).unwrap().is_none(), "a map outside a project is placed as it is");
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

        let mut game = GameConfig::builtin();
        game.build = Some(td_formats::BuildConfig { classes: vec!["light".into()], ..Default::default() });
        add_unexported(&mut game, root);
        let build = game.build.unwrap();
        assert!(build.defines("func_door") && build.defines("light"), "an export from before the install does not know the pack");

        // A config Godot exported after the install is complete, the person may have removed entities from it.
        let exported = root.join(td_formats::game::GAME_FILE_NAME);
        std::fs::write(&exported, "{}").unwrap();
        let later = std::fs::metadata(install_dir(root).join(MANIFEST)).unwrap().modified().unwrap() + std::time::Duration::from_secs(5);
        std::fs::File::options().write(true).open(&exported).unwrap().set_modified(later).unwrap();
        let mut game = GameConfig::builtin();
        game.source = Some(exported);
        assert_eq!(add_unexported(&mut game, root), 0);
    }

    #[test]
    fn installing_the_pack_drops_what_godot_reported_missing_before() {
        let dir = project();
        let mut state = crate::state::EditorState::new(Default::default());
        state.load_project(dir.path());
        let report = crate::live_link::GodotReport { missing: ["func_door".to_string(), "my_lamp".to_string()].into(), ..Default::default() };
        state.link_state.reports.insert("hall".into(), report);
        install_in_project(&mut state).unwrap();
        assert_eq!(state.link_state.reports["hall"].missing.iter().collect::<Vec<_>>(), ["my_lamp"]);
    }

    #[test]
    fn a_project_whose_maps_use_the_pack_gets_it_when_the_addon_no_longer_has_the_entities() {
        let dir = project();
        let root = dir.path();
        let mut state = crate::state::EditorState::new(Default::default());
        state.load_project(root);
        assert!(install_for_maps(&mut state).is_none(), "no map uses the pack");
        write(&root.join("maps/old.tdm"), r#"{"classname": "func_door"}, {"classname": "my_door"}"#);

        // The config the old addon exported still names func_door, with a script the update removed.
        let mut game = GameConfig::with_gameplay_pack();
        game.entities.iter_mut().find(|e| e.classname == "func_door").unwrap().script = "res://addons/func_godot/src/trenchdot/entities/td_door.gd".into();
        game.project_root = Some(root.to_path_buf());
        state.game = game;
        let status = install_for_maps(&mut state).expect("installed");
        assert!(status.en.starts_with("The maps use func_door, which now come with the Gameplay entities pack. Added 4 files"), "{}", status.en);
        assert!(state.gameplay_pack.installed && install_dir(root).join("scripts/td_door.gd").is_file());
        assert_eq!(state.game.entity("func_door").unwrap().script, "res://trenchdot/entities/scripts/td_door.gd", "the stale definition is replaced");
        assert!(install_for_maps(&mut state).is_none(), "once");
    }

    #[test]
    fn a_project_that_defines_the_classes_itself_is_left_alone() {
        let dir = project();
        let root = dir.path();
        write(&root.join("maps/a.tdm"), r#"{"classname": "func_door"}"#);
        let config = r#"{"format": "trenchdot-game", "version": 1, "entities": [{"classname": "func_door", "type": "solid"}]}"#;
        write(&root.join(td_formats::game::GAME_FILE_NAME), config);
        let mut state = crate::state::EditorState::new(Default::default());
        state.load_project(root);
        assert!(install_for_maps(&mut state).is_none());
        assert!(!install_dir(root).exists());

        std::fs::remove_file(root.join(td_formats::game::GAME_FILE_NAME)).unwrap();
        state.load_project(root);
        assert!(state.gameplay_pack.installed, "opening the project installs it once nothing defines func_door");
        assert!(state.status.contains("The maps use func_door"), "{}", state.status);
    }

    #[test]
    fn opening_a_project_tells_about_classnames_defined_twice() {
        let dir = project();
        let config = r#"{"format": "trenchdot-game", "version": 1, "entities": [{"classname": "func_door", "type": "solid"}],
            "clashes": [{"classname": "func_door", "definitions": ["res://trenchdot/entities/definitions/func_door.tres", "res://my_door.tres"]}]}"#;
        write(&dir.path().join(td_formats::game::GAME_FILE_NAME), config);
        let mut state = crate::state::EditorState::new(Default::default());
        state.load_project(dir.path());
        assert_eq!(state.status, "func_door is defined more than once. res://my_door.tres wins over res://trenchdot/entities/definitions/func_door.tres.");
    }

    #[test]
    fn finds_the_pack_classes_maps_use() {
        let dir = project();
        write(&dir.path().join("maps/a.tdm"), r#"{"classname": "func_door", "x": {"classname":"light"}}"#);
        write(&dir.path().join(".godot/b.tdm"), r#""trigger_once""#);
        assert_eq!(used_by_maps(dir.path()).into_iter().collect::<Vec<_>>(), ["func_door"]);
        assert!(in_pack("func_door") && !in_pack("light") && !in_pack("worldspawn"));

        let mut map = td_doc::Map::new();
        let layer = map.default_layer();
        for class in ["trigger_once", "light", "my_door"] {
            map.insert(layer, td_doc::NodeKind::Entity(td_doc::Entity::new(class)));
        }

        td_doc::format::save(&map, &dir.path().join("maps/saved.tdm")).unwrap();
        assert!(!std::fs::read(dir.path().join("maps/saved.tdm")).unwrap().starts_with(b"{"), "the editor saves the binary container");
        assert_eq!(used_by_maps(dir.path()).into_iter().collect::<Vec<_>>(), ["func_door", "trigger_once"]);
    }
}
