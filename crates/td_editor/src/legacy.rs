//! The names TrenchDot went by as GodotTrench, so old installs, settings and projects keep working.

use std::ffi::OsString;
use std::io;
use std::path::Path;

/// The name of the settings folder the old versions used.
const APP_ID: &str = "GodotTrench";

/// Not copied across: it is rebuilt on demand and holds the old version's thumbnails.
const CACHE: &str = "cache";

pub const ENV_PREFIX: &str = "GODOTTRENCH_";

/// The environment variable `name` (a `TRENCHDOT_` one), or the same variable under its old prefix. A blank value counts
/// as unset.
pub fn env_var_os(name: &str) -> Option<OsString> {
    let set = |name: &str| std::env::var_os(name).filter(|v| !v.is_empty());
    set(name).or_else(|| set(&format!("{ENV_PREFIX}{}", name.strip_prefix("TRENCHDOT_")?)))
}

pub fn env_var(name: &str) -> Option<String> {
    env_var_os(name)?.into_string().ok()
}

/// What the status line says about a map that was updated while it was read.
pub fn migrated_status(path: &Path) -> crate::i18n::Msg {
    let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| path.display().to_string());
    crate::i18n::msg!("Updated {name} from the {format} format, save to keep it", name, format = td_doc::legacy::NAME)
}

/// Brings the settings of an install made under the old name into the settings folder of this one, once. The old folder
/// stays where it is, so going back to the old version loses nothing. A run with default preferences never touches
/// either, and neither does one that already has settings of its own.
pub fn migrate_settings(default_prefs: bool) {
    if default_prefs {
        return;
    }

    let (Some(old), Some(new)) = (eframe::storage_dir(APP_ID), eframe::storage_dir(crate::APP_ID)) else { return };
    match copy_settings(&old, &new) {
        Ok(true) => eprintln!("Copied the settings from {} to {}", old.display(), new.display()),
        Ok(false) => {}
        Err(e) => eprintln!("Could not copy the settings from {}: {e}", old.display()),
    }
}

/// Copies what is in `old` to `new` unless `new` holds anything but its cache, and says whether it did.
fn copy_settings(old: &Path, new: &Path) -> io::Result<bool> {
    let entries = |dir: &Path| -> io::Result<Vec<std::fs::DirEntry>> {
        match std::fs::read_dir(dir) {
            Ok(read) => read.collect(),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    };
    let kept = |entry: &std::fs::DirEntry| entry.file_name() != CACHE;
    let (from, held) = (entries(old)?, entries(new)?);
    if !from.iter().any(kept) || held.iter().any(kept) {
        return Ok(false);
    }

    std::fs::create_dir_all(new)?;
    from.iter().filter(|e| kept(e)).try_for_each(|e| copy_tree(&e.path(), &new.join(e.file_name())))?;
    Ok(true)
}

fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    if from.is_dir() {
        std::fs::create_dir_all(to)?;
        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            copy_tree(&entry.path(), &to.join(entry.file_name()))?;
        }
    } else {
        std::fs::copy(from, to)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each test has a variable of its own, they run in parallel.
    #[test]
    fn the_old_variable_answers_when_the_new_one_is_not_set() {
        let (new, old) = ("TRENCHDOT_TEST_LEGACY_A", format!("{ENV_PREFIX}TEST_LEGACY_A"));
        assert_eq!(env_var(new), None);
        unsafe { std::env::set_var(&old, "from the old name") };
        assert_eq!(env_var(new).as_deref(), Some("from the old name"));
        unsafe { std::env::set_var(new, "from the new name") };
        assert_eq!(env_var(new).as_deref(), Some("from the new name"), "the new name wins");
        unsafe { std::env::set_var(new, "") };
        assert_eq!(env_var(new).as_deref(), Some("from the old name"), "a blank one counts as unset");
        unsafe { std::env::remove_var(&old) };
        assert_eq!(env_var_os(new), None);
    }

    #[test]
    fn only_trenchdot_variables_have_an_old_name() {
        unsafe { std::env::set_var("PATH_LEGACY_TEST", "x") };
        assert_eq!(env_var("PATH_LEGACY_TEST").as_deref(), Some("x"));
        assert_eq!(env_var("OTHER_LEGACY_TEST_B"), None);
    }

    #[test]
    fn the_status_names_the_file_and_the_old_format() {
        assert_eq!(migrated_status(Path::new("maps/room.gtm")).en, "Updated room.gtm from the GodotTrench format, save to keep it");
    }

    fn put(dir: &Path, file: &str, text: &str) {
        std::fs::create_dir_all(dir.join(file).parent().unwrap()).unwrap();
        std::fs::write(dir.join(file), text).unwrap();
    }

    #[test]
    fn settings_are_copied_once_without_the_cache() {
        let temp = tempfile::tempdir().unwrap();
        let (old, new) = (temp.path().join("old/data"), temp.path().join("new/data"));
        put(&old, "app.ron", "prefs");
        put(&old, "mcp-token", "secret");
        put(&old, "nested/trusted.json", "projects");
        put(&old, "cache/model_thumbs/a.png", "thumb");
        assert!(copy_settings(&old, &new).unwrap());
        assert_eq!(std::fs::read_to_string(new.join("app.ron")).unwrap(), "prefs");
        assert_eq!(std::fs::read_to_string(new.join("mcp-token")).unwrap(), "secret");
        assert_eq!(std::fs::read_to_string(new.join("nested/trusted.json")).unwrap(), "projects");
        assert!(!new.join("cache").exists(), "the cache stays behind");
        assert!(old.join("app.ron").is_file() && old.join("cache/model_thumbs/a.png").is_file(), "the old folder is left alone");

        put(&new, "app.ron", "changed since");
        std::fs::remove_file(new.join("mcp-token")).unwrap();
        assert!(!copy_settings(&old, &new).unwrap(), "settings that are there already are not copied over again");
        assert_eq!(std::fs::read_to_string(new.join("app.ron")).unwrap(), "changed since");
        assert!(!new.join("mcp-token").exists());
    }

    #[test]
    fn settings_are_copied_into_a_folder_that_holds_only_a_cache() {
        let temp = tempfile::tempdir().unwrap();
        let (old, new) = (temp.path().join("old"), temp.path().join("new"));
        put(&old, "app.ron", "prefs");
        put(&new, "cache/model_thumbs/b.png", "thumb");
        assert!(copy_settings(&old, &new).unwrap());
        assert_eq!(std::fs::read_to_string(new.join("app.ron")).unwrap(), "prefs");
        assert!(new.join("cache/model_thumbs/b.png").is_file());
    }

    #[test]
    fn nothing_is_made_without_old_settings() {
        let temp = tempfile::tempdir().unwrap();
        let new = temp.path().join("new");
        assert!(!copy_settings(&temp.path().join("missing"), &new).unwrap());
        put(temp.path(), "old/cache/a.png", "thumb");
        assert!(!copy_settings(&temp.path().join("old"), &new).unwrap());
        std::fs::create_dir_all(temp.path().join("empty")).unwrap();
        assert!(!copy_settings(&temp.path().join("empty"), &new).unwrap());
        assert!(!new.exists());
    }

    #[test]
    fn an_old_kill_switch_still_turns_mcp_off() {
        let old = format!("{ENV_PREFIX}DISABLE_MCP");
        unsafe { std::env::set_var(&old, "1") };
        let value = env_var_os(crate::mcp::switch::DISABLE_ENV);
        unsafe { std::env::remove_var(&old) };
        assert!(crate::mcp::switch::env_disables(value.as_deref()));
    }
}
