//! Finding the Godot executable.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Result of the last executable search, redone when its inputs change.
#[derive(Clone, Debug, Default)]
pub struct GodotStatus {
    pub exe: Option<PathBuf>,
    detected_for: Option<(PathBuf, Option<PathBuf>)>,
}

impl GodotStatus {
    /// Searches again when the preference path or the project changed. Returns true when it searched.
    pub fn refresh(&mut self, pref: &Path, project_root: Option<&Path>) -> bool {
        let key = (pref.to_path_buf(), project_root.map(Path::to_path_buf));
        if self.detected_for.as_ref() == Some(&key) {
            return false;
        }
        self.exe = detect(pref, std::env::var_os("GODOT"), std::env::var_os("PATH"), &install_dirs(), project_root);
        self.detected_for = Some(key);
        true
    }

    pub fn found(&self) -> bool {
        self.exe.is_some()
    }
}

/// The preference path, the GODOT environment variable, PATH, then common install folders.
pub fn detect(pref: &Path, env: Option<OsString>, path_var: Option<OsString>, install_dirs: &[PathBuf], project_root: Option<&Path>) -> Option<PathBuf> {
    if !pref.as_os_str().is_empty() && pref.is_file() {
        return Some(pref.to_path_buf());
    }
    if let Some(p) = env.map(PathBuf::from).filter(|p| p.is_file()) {
        return Some(p);
    }
    let mono = project_root.is_some_and(has_csharp_project);
    let path_dirs: Vec<PathBuf> = path_var.map(|p| std::env::split_paths(&p).collect()).unwrap_or_default();
    path_dirs.iter().chain(install_dirs).find_map(|dir| find_in(dir, mono))
}

fn has_csharp_project(root: &Path) -> bool {
    std::fs::read_dir(root).into_iter().flatten().flatten().any(|e| e.path().extension().is_some_and(|x| x.eq_ignore_ascii_case("csproj")))
}

const EXACT_NAMES: &[&str] = if cfg!(windows) { &["godot.exe", "godot4.exe"] } else { &["godot", "godot4"] };

/// Download names like `Godot_v4.7.2-stable_win64.exe` and the Steam build `godot.windows.opt.tools.64.exe`.
fn is_release_name(lower: &str) -> bool {
    let named = lower.starts_with("godot_v4") || lower.starts_with("godot.windows.") || lower.starts_with("godot.linuxbsd.");
    let runnable = if cfg!(windows) { lower.ends_with(".exe") } else { !lower.ends_with(".zip") && !lower.ends_with(".tpz") };
    named && runnable && !lower.contains("console")
}

fn find_in(dir: &Path, mono: bool) -> Option<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else { return None };
    let mut exact = None;
    let mut releases = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if EXACT_NAMES.contains(&name.as_str()) || is_release_name(&name) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if EXACT_NAMES.contains(&name.as_str()) {
                exact.get_or_insert(path);
            } else {
                releases.push((name, path));
            }
        }
    }
    // Mono builds only when the project uses C#, newest version name first.
    releases.sort_by(|a, b| (a.0.contains("mono") != mono).cmp(&(b.0.contains("mono") != mono)).then(b.0.cmp(&a.0)));
    exact.or_else(|| releases.into_iter().next().map(|(_, p)| p))
}

fn install_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let env_dir = |var: &str, rest: &str| std::env::var_os(var).map(|v| PathBuf::from(v).join(rest));
    if cfg!(windows) {
        dirs.extend(env_dir("LOCALAPPDATA", "Microsoft/WinGet/Links"));
        dirs.extend(env_dir("USERPROFILE", "scoop/shims"));
        dirs.extend(env_dir("LOCALAPPDATA", "Programs/Godot"));
        for base in ["ProgramFiles(x86)", "ProgramFiles"] {
            dirs.extend(env_dir(base, "Steam/steamapps/common/Godot Engine"));
            dirs.extend(env_dir(base, "Godot"));
        }
    } else {
        dirs.extend(env_dir("HOME", ".local/share/Steam/steamapps/common/Godot Engine"));
        dirs.extend(env_dir("HOME", ".local/bin"));
        dirs.push(PathBuf::from("/Applications/Godot.app/Contents/MacOS"));
    }
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gt_godot_detect_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn touch(dir: &Path, name: &str) -> PathBuf {
        let p = dir.join(name);
        std::fs::write(&p, b"").unwrap();
        p
    }

    #[test]
    fn finds_release_builds_and_skips_console_and_editor_binaries() {
        let dir = temp_dir("release");
        let ext = if cfg!(windows) { ".exe" } else { "" };
        touch(&dir, &format!("Godot_v4.7.2-stable_win64_console{ext}"));
        touch(&dir, &format!("godottrench{ext}"));
        assert_eq!(detect(Path::new(""), None, Some(dir.clone().into_os_string()), &[], None), None);
        let release = touch(&dir, &format!("Godot_v4.7.2-stable_win64{ext}"));
        assert_eq!(detect(Path::new(""), None, Some(dir.clone().into_os_string()), &[], None), Some(release.clone()));

        let mono = touch(&dir, &format!("Godot_v4.7.2-stable_mono_win64{ext}"));
        let project = temp_dir("csharp_project");
        assert_eq!(detect(Path::new(""), None, Some(dir.clone().into_os_string()), &[], Some(&project)), Some(release));
        touch(&project, "Game.csproj");
        assert_eq!(detect(Path::new(""), None, Some(dir.clone().into_os_string()), &[], Some(&project)), Some(mono));
        let _ = std::fs::remove_dir_all(dir);
        let _ = std::fs::remove_dir_all(project);
    }

    #[test]
    fn preference_and_env_come_before_path_and_install_dirs() {
        let dir = temp_dir("order");
        let exact = touch(&dir, EXACT_NAMES[0]);
        let pref = touch(&dir, "my_godot.bin");
        let env = touch(&dir, "env_godot.bin");
        let path = Some(OsString::from("/does/not/exist"));
        let dirs = std::slice::from_ref(&dir);
        assert_eq!(detect(&pref, Some(env.clone().into_os_string()), path.clone(), dirs, None), Some(pref));
        assert_eq!(detect(Path::new("missing.exe"), Some(env.clone().into_os_string()), path.clone(), dirs, None), Some(env));
        assert_eq!(detect(Path::new(""), None, path.clone(), dirs, None), Some(exact));
        assert_eq!(detect(Path::new(""), None, path, &[], None), None);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn refresh_only_searches_when_inputs_change() {
        let mut status = GodotStatus::default();
        assert!(status.refresh(Path::new("a"), None));
        assert!(!status.refresh(Path::new("a"), None));
        assert!(status.refresh(Path::new("b"), None));
        assert!(status.refresh(Path::new("b"), Some(Path::new("project"))));
    }
}
