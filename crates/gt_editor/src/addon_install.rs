//! The GodotTrench addon in a Godot project: whether it is there, matches the editor and is enabled, installing or
//! updating it from the release the content downloads come from, and enabling it in `project.godot`.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::mpsc;

use crate::content::{Added, Download, Error, Progress, Reporter, io_error};

pub const ADDON: Download = Download { asset: "func_godot-godottrench-addon.zip", folders: &["addons/func_godot"], approx_mb: 1 };
/// Where the addon lives in a project.
pub const DIR: &str = "addons/func_godot";
/// How `project.godot` names the addon among its enabled plugins.
pub const PLUGIN: &str = "res://addons/func_godot/plugin.cfg";
/// Holds the addon while it is unpacked, and the old addons an update replaced. Godot neither imports nor lists folders
/// whose names start with a dot. A backup under `addons` would still show as a second GodotTrench in Godot's plugin
/// list, even with a `.gdignore` in it.
pub const WORK_DIR: &str = ".godottrench";
pub const BACKUPS: &str = ".godottrench/addon-backups";
/// Starts the name of the folder in [`WORK_DIR`] the addon is unpacked into. One a crash left behind goes on the next
/// install.
const UNPACKING: &str = "addon-new-";

const GIT: &str = "res://addons/func_godot is a git checkout, so the editor leaves it alone. Update it with git instead.";

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Installed {
    #[default]
    Missing,
    /// The folder is there, but its plugin.cfg names no version.
    Unknown,
    Older(String),
    Newer(String),
    Current,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub installed: Installed,
    /// The addon folder is a git checkout or submodule, which git updates rather than the editor.
    pub git: bool,
    /// Listed among the enabled plugins in project.godot.
    pub enabled: bool,
    /// The project has a godottrench_game.json, so the addon ran in Godot at least once.
    pub exported: bool,
}

impl Status {
    pub fn check(root: &Path) -> Status {
        let dir = root.join(DIR);
        let installed = match gt_formats::game::addon_version(root) {
            Some(version) => match compare(&version, crate::VERSION) {
                Some(Ordering::Equal) => Installed::Current,
                Some(Ordering::Greater) => Installed::Newer(version),
                _ => Installed::Older(version),
            },
            None if dir.exists() => Installed::Unknown,
            None => Installed::Missing,
        };
        let project = std::fs::read_to_string(root.join("project.godot")).unwrap_or_default();
        Status { installed, git: dir.join(".git").exists(), enabled: plugin_enabled(&project), exported: gt_formats::GameConfig::discover(root).is_some() }
    }

    /// The addon is missing or does not match the editor.
    pub fn mismatched(&self) -> bool {
        self.installed != Installed::Current
    }

    /// Names what a person dismissed: this editor's version and what the project had.
    pub fn dismiss_key(&self) -> String {
        let had = match &self.installed {
            Installed::Missing => "missing",
            Installed::Unknown => "unknown",
            Installed::Older(v) | Installed::Newer(v) => v,
            Installed::Current => "current",
        };
        format!("{} {had}", crate::VERSION)
    }
}

/// Whether opening the project should bring up the addon part of the setup wizard: the addon is missing or does not
/// match the editor, and nobody dismissed that for this project and version.
pub fn should_ask(root: &Path, dismissed: &BTreeMap<PathBuf, String>) -> bool {
    let status = Status::check(root);
    status.mismatched() && dismissed.get(root) != Some(&status.dismiss_key())
}

/// Compares versions like `0.1.0` or `0.2.0-beta.1` part by part as numbers, a pre-release before its release. None
/// when either is not such a version.
pub fn compare(a: &str, b: &str) -> Option<Ordering> {
    fn parse(v: &str) -> Option<(Vec<u64>, Option<&str>)> {
        let v = v.trim().trim_start_matches('v').split('+').next().unwrap_or_default();
        let (core, pre) = v.split_once('-').map_or((v, None), |(c, p)| (c, Some(p)));
        Some((core.split('.').map(|p| p.parse().ok()).collect::<Option<Vec<u64>>>()?, pre))
    }

    let ((a, a_pre), (b, b_pre)) = (parse(a)?, parse(b)?);
    let part = |v: &[u64], i: usize| v.get(i).copied().unwrap_or(0);
    let core = (0..a.len().max(b.len())).map(|i| part(&a, i).cmp(&part(&b, i))).find(|o| o.is_ne()).unwrap_or(Ordering::Equal);
    let pre = match (a_pre, b_pre) {
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(x), Some(y)) => x.cmp(y),
    };
    Some(core.then(pre))
}

/// Each line of `text` with the byte offset it starts at, without its line ending.
fn lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut at = 0;
    text.split_inclusive('\n').map(move |line| {
        let start = at;
        at += line.len();
        (start, line.trim_end_matches(['\n', '\r']))
    })
}

/// Where project.godot enables plugins.
#[derive(Debug, PartialEq)]
enum Plugins {
    NoSection,
    /// `[editor_plugins]` without an `enabled` key. Where its header line ends, before the line break.
    NoKey(usize),
    /// Between the parentheses of `enabled=PackedStringArray(...)`.
    List(Range<usize>),
}

const UNREADABLE: &str =
    "project.godot lists its enabled plugins in a way the editor does not change. Enable GodotTrench in Godot under Project > Project Settings > Plugins.";

fn find_plugins(text: &str) -> Result<Plugins, String> {
    let mut header = None;
    let mut end = text.len();
    for (start, line) in lines(text) {
        let Some(name) = line.trim().strip_prefix('[').and_then(|l| l.split_once(']')).map(|(n, _)| n.trim()) else { continue };
        if header.is_some() {
            end = start;
            break;
        }

        if name == "editor_plugins" {
            header = Some(start + line.len());
        }
    }

    let Some(body) = header else { return Ok(Plugins::NoSection) };
    for (start, line) in lines(&text[..end]).filter(|(start, _)| *start > body) {
        let Some((key, _)) = line.split_once('=') else { continue };
        if key.trim() != "enabled" {
            continue;
        }

        let value = start + key.len() + 1;
        let lead = text[value..end].len() - text[value..end].trim_start().len();
        let open = value + lead + "PackedStringArray(".len();
        if !text[value + lead..end].starts_with("PackedStringArray(") {
            return Err(UNREADABLE.into());
        }

        let (mut quoted, mut escaped) = (false, false);
        for (i, c) in text[open..end].char_indices() {
            match c {
                _ if escaped => escaped = false,
                '\\' if quoted => escaped = true,
                '"' => quoted = !quoted,
                ')' if !quoted => return Ok(Plugins::List(open..open + i)),
                _ => {}
            }
        }

        return Err(UNREADABLE.into());
    }

    Ok(Plugins::NoKey(body))
}

/// Whether the text of a `project.godot` enables the addon.
pub fn plugin_enabled(project: &str) -> bool {
    matches!(find_plugins(project), Ok(Plugins::List(list)) if project[list.clone()].contains(&format!("\"{PLUGIN}\"")))
}

/// `project`, the text of a `project.godot`, with the addon added to its enabled plugins and every other byte as it
/// was. None when the addon is enabled already.
pub fn with_plugin_enabled(project: &str) -> Result<Option<String>, String> {
    let nl = if project.contains("\r\n") { "\r\n" } else { "\n" };
    let entry = format!("\"{PLUGIN}\"");
    let (at, insert) = match find_plugins(project)? {
        Plugins::List(list) if project[list.clone()].contains(&entry) => return Ok(None),
        Plugins::List(list) if project[list.clone()].trim().is_empty() => (list.end, entry),
        Plugins::List(list) => (list.end, format!(", {entry}")),
        Plugins::NoKey(header) => (header, format!("{nl}enabled=PackedStringArray({entry})")),
        Plugins::NoSection => {
            let gap = match project {
                "" => String::new(),
                p if p.ends_with('\n') => nl.to_string(),
                _ => format!("{nl}{nl}"),
            };
            (project.len(), format!("{gap}[editor_plugins]{nl}{nl}enabled=PackedStringArray({entry}){nl}"))
        }
    };

    Ok(Some(format!("{}{insert}{}", &project[..at], &project[at..])))
}

/// Enables the addon in the project's `project.godot`. Returns false when it was enabled already.
pub fn enable(root: &Path) -> Result<bool, String> {
    let path = root.join("project.godot");
    let text = std::fs::read(&path).map_err(|e| format!("Could not read {}: {e}", path.display()))?;
    let text = String::from_utf8(text).map_err(|_| UNREADABLE.to_string())?;
    let Some(changed) = with_plugin_enabled(&text)? else { return Ok(false) };
    gt_formats::write_atomic(&path, &mut changed.as_bytes()).map_err(|e| format!("Could not write {}: {e}", path.display()))?;
    Ok(true)
}

/// What an install or update did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outcome {
    /// What the project had before.
    pub before: Installed,
    /// The version the project has now.
    pub version: Option<String>,
    /// The release the addon came from.
    pub tag: Option<String>,
    /// Where the replaced addon went.
    pub backup: Option<PathBuf>,
    pub error: Option<Error>,
}

impl Outcome {
    pub fn summary(&self, root: &Path) -> String {
        match &self.error {
            // The addon is unpacked beside the project and only moved in once complete, so nothing half done stays.
            Some(Error::Cancelled) => return "Cancelled, the project is as it was.".into(),
            Some(Error::DiskFull) => return "The disk is full. Free some space and try again, the project is as it was.".into(),
            Some(e) => return e.to_string(),
            None => {}
        }

        let version = self.version.as_deref().unwrap_or_default();
        let from = if self.tag.as_deref() == Some("beta") { " from the rolling beta" } else { "" };
        let Some(backup) = &self.backup else { return format!("Installed the GodotTrench addon {version}{from} into res://addons/func_godot.") };
        let old = match &self.before {
            Installed::Older(v) | Installed::Newer(v) => format!(" from {v}"),
            _ => String::new(),
        };
        let shown = backup.strip_prefix(root).unwrap_or(backup).to_string_lossy().replace('\\', "/");
        format!(
            "Updated the addon{old} to {version}{from}. The old one is in the project's {shown} folder, which Godot ignores. Delete it once the new one works."
        )
    }
}

/// Downloads the addon from the release matching the editor, or the rolling beta, into the project at `root`. An addon
/// already there is replaced as a whole and kept in [`BACKUPS`], unless it is a git checkout.
pub fn install(root: &Path, api: &str, report: &Reporter) -> Outcome {
    let before = Status::check(root);
    let mut outcome = Outcome { before: before.installed.clone(), ..Default::default() };
    if before.git {
        outcome.error = Some(Error::Io(GIT.into()));
        return outcome;
    }

    let work = root.join(WORK_DIR);
    for left in std::fs::read_dir(&work).into_iter().flatten().flatten() {
        if left.file_name().to_string_lossy().starts_with(UNPACKING) {
            let _ = std::fs::remove_dir_all(left.path());
        }
    }

    static NEXT: AtomicU64 = AtomicU64::new(0);
    // Inside the project, so the addon moves into place with a rename rather than a copy.
    let unpacked = work.join(format!("{UNPACKING}{}-{}", std::process::id(), NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
    let result = (|| {
        report.set(|p| *p = Progress { stage: format!("Looking up {} in the releases", ADDON.asset), ..Default::default() });
        let agent = crate::content::agent();
        let asset = crate::content::find_asset(&agent, api, ADDON.asset)?;
        let archive = crate::content::download(&agent, &asset, report)?;
        crate::content::extract(archive, ADDON.asset, &unpacked, ADDON.folders, report, &mut Added::default())?;
        let version = gt_formats::game::addon_version(&unpacked).ok_or_else(|| Error::Damaged(format!("it has no {DIR}/plugin.cfg with a version")))?;
        report.check()?;
        let backup = swap(root, &unpacked.join(DIR), &before.installed)?;
        Ok((asset.tag, version, backup))
    })();
    let _ = std::fs::remove_dir_all(&unpacked);
    // Only goes when empty, that is when there are no backups.
    let _ = std::fs::remove_dir(&work);
    match result {
        Ok((tag, version, backup)) => {
            outcome.tag = Some(tag);
            outcome.version = Some(version);
            outcome.backup = backup;
        }
        Err(e) => outcome.error = Some(e),
    }

    outcome
}

/// Moves the unpacked addon `new` into place. An addon already there moves to a backup folder first, and back when the
/// new one cannot take its place. Returns where the old one went.
fn swap(root: &Path, new: &Path, before: &Installed) -> Result<Option<PathBuf>, Error> {
    let target = root.join(DIR);
    if std::fs::symlink_metadata(&target).is_err() {
        let addons = root.join("addons");
        std::fs::create_dir_all(&addons).map_err(|e| io_error(e, &addons))?;
        std::fs::rename(new, &target).map_err(|e| io_error(e, &target))?;
        return Ok(None);
    }

    if target.join(".git").exists() {
        return Err(Error::Io(GIT.into()));
    }

    let backups = root.join(BACKUPS);
    std::fs::create_dir_all(&backups).map_err(|e| io_error(e, &backups))?;
    // In case Godot ever looks into hidden folders, and so a project in git does not pick the backups up.
    for (name, text) in [(".gdignore", ""), (".gitignore", "*\n")] {
        let path = backups.join(name);
        if !path.exists() {
            std::fs::write(&path, text).map_err(|e| io_error(e, &path))?;
        }
    }

    let version = match before {
        Installed::Older(v) | Installed::Newer(v) => v.chars().map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' { c } else { '_' }).collect(),
        Installed::Current => crate::VERSION.to_string(),
        _ => "unknown".to_string(),
    };
    let backup = (1..)
        .map(|n| backups.join(if n == 1 { format!("func_godot-{version}") } else { format!("func_godot-{version}-{n}") }))
        .find(|p| std::fs::symlink_metadata(p).is_err())
        .expect("a free backup name");
    std::fs::rename(&target, &backup)
        .map_err(|e| Error::Io(format!("Could not move the old addon out of the way ({e}). Close Godot and anything else using its files, then try again.")))?;
    if let Err(e) = std::fs::rename(new, &target) {
        let back = std::fs::rename(&backup, &target);
        return Err(Error::Io(match back {
            Ok(()) => format!("Could not put the new addon in place ({e}), the old one is back where it was."),
            Err(_) => format!("Could not put the new addon in place ({e}). The old one is in {}, move it back to {}.", backup.display(), target.display()),
        }));
    }

    Ok(Some(backup))
}

/// An install running on its own thread.
pub struct Job {
    report: Reporter,
    rx: mpsc::Receiver<Outcome>,
}

impl Job {
    pub fn start(root: PathBuf, api: String, repaint: Option<egui::Context>) -> Job {
        let report = Reporter::new(repaint.clone());
        let thread_report = report.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(install(&root, &api, &thread_report));
            if let Some(ctx) = repaint {
                ctx.request_repaint();
            }
        });
        Job { report, rx }
    }

    pub fn progress(&self) -> Progress {
        self.report.progress()
    }

    pub fn cancel(&self) {
        self.report.cancel();
    }

    pub fn cancelling(&self) -> bool {
        self.report.cancelled()
    }

    pub fn poll(&self) -> Option<Outcome> {
        match self.rx.try_recv() {
            Ok(outcome) => Some(outcome),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => Some(Outcome { error: Some(Error::Io("The install stopped unexpectedly.".into())), ..Default::default() }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::test_server::{Server, zip};

    fn project(name: &str, godot: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gt_addon_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("project.godot"), godot).unwrap();
        dir
    }

    fn addon(root: &Path, version: &str) {
        std::fs::create_dir_all(root.join(DIR)).unwrap();
        std::fs::write(root.join(DIR).join("plugin.cfg"), format!("[plugin]\n\nname=\"GodotTrench\"\nversion=\"{version}\"\n")).unwrap();
    }

    fn release_zip(version: &str) -> Vec<u8> {
        zip(&[
            ("addons/func_godot/plugin.cfg", format!("[plugin]\nname=\"GodotTrench\"\nversion=\"{version}\"\n").as_bytes()),
            ("addons/func_godot/src/func_godot_plugin.gd", b"@tool\nextends EditorPlugin\n"),
        ])
    }

    #[test]
    fn versions_compare_as_numbers() {
        assert_eq!(compare("0.1.0", "0.1.0"), Some(Ordering::Equal));
        assert_eq!(compare("0.1", "0.1.0"), Some(Ordering::Equal));
        assert_eq!(compare("0.10.0", "0.9.0"), Some(Ordering::Greater));
        assert_eq!(compare("0.1.0-beta", "0.1.0"), Some(Ordering::Less));
        assert_eq!(compare("v1.0.0", "0.9.9"), Some(Ordering::Greater));
        assert_eq!(compare("dev", "0.1.0"), None);
    }

    #[test]
    fn detects_what_the_project_has() {
        let root = project("detect", "config_version=5\n");
        let status = Status::check(&root);
        assert_eq!(status, Status { installed: Installed::Missing, git: false, enabled: false, exported: false });
        assert!(should_ask(&root, &BTreeMap::new()));
        assert!(!should_ask(&root, &BTreeMap::from([(root.clone(), status.dismiss_key())])), "dismissed for this version");
        assert!(should_ask(&root, &BTreeMap::from([(root.clone(), "0.0.1 missing".to_string())])), "a new editor asks again");

        addon(&root, "0.0.1");
        assert_eq!(Status::check(&root).installed, Installed::Older("0.0.1".into()));
        assert!(should_ask(&root, &BTreeMap::from([(root.clone(), status.dismiss_key())])), "another addon asks again");
        addon(&root, "999.0.0");
        assert_eq!(Status::check(&root).installed, Installed::Newer("999.0.0".into()));
        std::fs::write(root.join(DIR).join("plugin.cfg"), "[plugin]\nname=\"Something\"\n").unwrap();
        assert_eq!(Status::check(&root).installed, Installed::Unknown);

        addon(&root, crate::VERSION);
        std::fs::write(root.join(DIR).join(".git"), "gitdir: ../../.git/modules/func_godot\n").unwrap();
        std::fs::write(root.join("project.godot"), format!("[editor_plugins]\n\nenabled=PackedStringArray(\"{PLUGIN}\")\n")).unwrap();
        std::fs::write(root.join(gt_formats::game::GAME_FILE_NAME), "{}").unwrap();
        let status = Status::check(&root);
        assert_eq!(status, Status { installed: Installed::Current, git: true, enabled: true, exported: true });
        assert!(!status.mismatched() && !should_ask(&root, &BTreeMap::new()));
        std::fs::remove_dir_all(&root).unwrap();
    }

    fn enabled(text: &str) -> String {
        with_plugin_enabled(text).unwrap().expect("a change")
    }

    #[test]
    fn enabling_changes_only_the_plugin_list() {
        let godot = "; Engine configuration file.\n\nconfig_version=5\n\n[application]\n\nconfig/name=\"My Game\"\n";
        let added = format!("{godot}\n[editor_plugins]\n\nenabled=PackedStringArray(\"{PLUGIN}\")\n");
        assert_eq!(enabled(godot), added);
        assert!(plugin_enabled(&added) && !plugin_enabled(godot));
        assert_eq!(with_plugin_enabled(&added), Ok(None), "enabled already");
        assert_eq!(enabled("config_version=5"), format!("config_version=5\n\n[editor_plugins]\n\nenabled=PackedStringArray(\"{PLUGIN}\")\n"));

        let crlf = godot.replace('\n', "\r\n");
        assert_eq!(enabled(&crlf), added.replace('\n', "\r\n"), "CRLF files stay CRLF");

        let other = "[editor_plugins]\r\n\r\nenabled=PackedStringArray(\"res://addons/other/plugin.cfg\")\r\n\r\n[rendering]\r\n\r\nx=1\r\n";
        assert_eq!(
            enabled(other),
            format!("[editor_plugins]\r\n\r\nenabled=PackedStringArray(\"res://addons/other/plugin.cfg\", \"{PLUGIN}\")\r\n\r\n[rendering]\r\n\r\nx=1\r\n")
        );
        let empty = "[editor_plugins]\n\nenabled=PackedStringArray()\n";
        assert_eq!(enabled(empty), format!("[editor_plugins]\n\nenabled=PackedStringArray(\"{PLUGIN}\")\n"));

        // A tricky name, a list over two lines and the key of another section do not throw it off.
        let tricky = "[editor_plugins]\n\nenabled=PackedStringArray(\"res://addons/a)b/plugin.cfg\",\n\"res://addons/c/plugin.cfg\")\n[z]\nenabled=1\n";
        assert_eq!(
            enabled(tricky),
            format!(
                "[editor_plugins]\n\nenabled=PackedStringArray(\"res://addons/a)b/plugin.cfg\",\n\"res://addons/c/plugin.cfg\", \"{PLUGIN}\")\n[z]\nenabled=1\n"
            )
        );
        let no_key = "[editor_plugins]\n\nsomething=1\n\n[z]\n";
        assert_eq!(enabled(no_key), format!("[editor_plugins]\nenabled=PackedStringArray(\"{PLUGIN}\")\n\nsomething=1\n\n[z]\n"));
        assert!(!plugin_enabled("[other]\nenabled=PackedStringArray(\"res://addons/func_godot/plugin.cfg\")\n"), "only in [editor_plugins]");

        assert_eq!(with_plugin_enabled("[editor_plugins]\nenabled=PoolStringArray(\"x\")\n"), Err(UNREADABLE.to_string()));
        assert_eq!(with_plugin_enabled("[editor_plugins]\nenabled=PackedStringArray(\"x\"\n"), Err(UNREADABLE.to_string()));
    }

    #[test]
    fn enable_writes_project_godot_byte_for_byte() {
        let godot = "; Engine configuration file.\r\n\r\nconfig_version=5\r\n\r\n[application]\r\n\r\nconfig/name=\"Über Spiel\"\r\n";
        let root = project("enable", godot);
        assert_eq!(enable(&root), Ok(true));
        let written = std::fs::read(root.join("project.godot")).unwrap();
        let expected = format!("{godot}\r\n[editor_plugins]\r\n\r\nenabled=PackedStringArray(\"{PLUGIN}\")\r\n");
        assert_eq!(written, expected.as_bytes());
        assert_eq!(enable(&root), Ok(false));
        assert_eq!(std::fs::read(root.join("project.godot")).unwrap(), expected.as_bytes());
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1, "no temporary file stays");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn installs_a_missing_addon_from_the_release() {
        let server = Server::start();
        server.release(&format!("v{}", crate::VERSION), &[(ADDON.asset, &release_zip(crate::VERSION))], true);
        let root = project("install", "config_version=5\n");
        std::fs::create_dir_all(root.join(WORK_DIR).join("addon-new-1-0/addons")).unwrap();
        let outcome = install(&root, &server.url, &Reporter::default());
        assert_eq!(outcome.error, None);
        assert_eq!((outcome.version.as_deref(), outcome.backup.as_ref()), (Some(crate::VERSION), None));
        assert_eq!(Status::check(&root).installed, Installed::Current);
        assert!(root.join(DIR).join("src/func_godot_plugin.gd").is_file());
        assert!(!root.join(WORK_DIR).exists(), "nothing is left beside the addon, not even what a crash left");
        assert!(outcome.summary(&root).starts_with("Installed the GodotTrench addon"), "{}", outcome.summary(&root));
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn an_update_replaces_the_folder_and_keeps_the_old_one() {
        let server = Server::start();
        server.release("beta", &[(ADDON.asset, &release_zip(crate::VERSION))], true);
        let root = project("update", "config_version=5\n");
        addon(&root, "0.0.1");
        std::fs::write(root.join(DIR).join("removed_since.gd"), "old").unwrap();
        let outcome = install(&root, &server.url, &Reporter::default());
        assert_eq!(outcome.error, None);
        let backup = root.join(BACKUPS).join("func_godot-0.0.1");
        assert_eq!(outcome.backup.as_ref(), Some(&backup));
        assert!(!root.join(DIR).join("removed_since.gd").exists(), "the folder is replaced as a whole");
        assert_eq!(std::fs::read_to_string(backup.join("removed_since.gd")).unwrap(), "old");
        assert!(root.join(BACKUPS).join(".gdignore").is_file() && root.join(BACKUPS).join(".gitignore").is_file());
        assert_eq!(Status::check(&root).installed, Installed::Current);
        let summary = outcome.summary(&root);
        assert!(summary.starts_with("Updated the addon from 0.0.1 to") && summary.contains(".godottrench/addon-backups/func_godot-0.0.1"), "{summary}");
        assert_eq!(std::fs::read_dir(root.join(WORK_DIR)).unwrap().count(), 1, "only the backups stay");

        addon(&root, "0.0.1");
        assert_eq!(install(&root, &server.url, &Reporter::default()).backup, Some(root.join(BACKUPS).join("func_godot-0.0.1-2")), "an older backup stays");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_git_checkout_is_never_replaced() {
        let server = Server::start();
        server.release("beta", &[(ADDON.asset, &release_zip(crate::VERSION))], true);
        let root = project("git", "config_version=5\n");
        addon(&root, "0.0.1");
        std::fs::create_dir_all(root.join(DIR).join(".git")).unwrap();
        let outcome = install(&root, &server.url, &Reporter::default());
        assert!(matches!(&outcome.error, Some(Error::Io(why)) if why.contains("git")), "{outcome:?}");
        assert!(server.requests.lock().unwrap().is_empty(), "nothing is downloaded");
        assert_eq!(Status::check(&root).installed, Installed::Older("0.0.1".into()));
        assert!(!root.join(WORK_DIR).exists());

        std::fs::remove_dir_all(root.join(DIR).join(".git")).unwrap();
        let unpacked = root.join("new");
        std::fs::create_dir_all(unpacked.join(DIR)).unwrap();
        std::fs::create_dir_all(root.join(DIR).join(".git")).unwrap();
        assert!(matches!(swap(&root, &unpacked.join(DIR), &Installed::Older("0.0.1".into())), Err(Error::Io(_))), "checked again right before the swap");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn failed_downloads_leave_the_project_alone() {
        let server = Server::start();
        let root = project("failed", "config_version=5\n");
        addon(&root, "0.0.1");
        assert_eq!(install(&root, &server.url, &Reporter::default()).error, Some(Error::NoRelease));
        server.release("beta", &[], true);
        assert_eq!(install(&root, &server.url, &Reporter::default()).error, Some(Error::NotPublished(ADDON.asset)));

        server.release("beta", &[(ADDON.asset, &zip(&[("addons/func_godot/readme.txt", b"no plugin.cfg")]))], true);
        assert!(matches!(install(&root, &server.url, &Reporter::default()).error, Some(Error::Damaged(_))));
        server.release("beta", &[(ADDON.asset, &zip(&[("addons/func_godot/plugin.cfg", b"x"), ("project.godot", b"x")]))], true);
        assert!(matches!(install(&root, &server.url, &Reporter::default()).error, Some(Error::Unsafe(_))));

        let cancelled = Reporter::default();
        cancelled.cancel();
        server.release("beta", &[(ADDON.asset, &release_zip(crate::VERSION))], true);
        let outcome = install(&root, &server.url, &cancelled);
        assert_eq!(outcome.error, Some(Error::Cancelled));
        assert_eq!(outcome.summary(&root), "Cancelled, the project is as it was.");

        assert_eq!(Status::check(&root).installed, Installed::Older("0.0.1".into()));
        assert_eq!(std::fs::read(root.join("project.godot")).unwrap(), b"config_version=5\n");
        assert!(!root.join(WORK_DIR).exists());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_release_zip_holds_the_addon_folder() {
        let ci = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/ci.yml")).unwrap();
        let line = ci.lines().find(|l| l.contains(&format!("{}\"", ADDON.asset)) && l.contains("zip -qr")).expect("ci.yml zips the addon");
        assert!(line.contains(&format!("cd godot && zip -qr \"../dist/{}\" {DIR} ", ADDON.asset)), "{line}");
        assert_eq!(ADDON.folders, [DIR]);
    }

    #[test]
    fn a_job_installs_on_its_own_thread() {
        let server = Server::start();
        server.release("beta", &[(ADDON.asset, &release_zip(crate::VERSION))], false);
        let root = project("job", "config_version=5\n");
        let job = Job::start(root.clone(), server.url.clone(), None);
        let outcome = loop {
            if let Some(o) = job.poll() {
                break o;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        assert_eq!((outcome.error, outcome.tag.as_deref()), (None, Some("beta")));
        assert_eq!(job.progress().done, 2, "both files were counted");
        std::fs::remove_dir_all(&root).unwrap();
    }
}
