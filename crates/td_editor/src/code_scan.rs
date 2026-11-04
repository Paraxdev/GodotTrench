//! Whether a Godot project holds code that Godot runs when it opens, plays or builds the project. A project that holds
//! none opens trusted, see [`crate::trust`]. TrenchDot's own addon and the Gameplay pack it installs do not count, as
//! long as every file in them has the bytes this editor version installs.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, Instant};

use base64::Engine;

/// SHA-256 of each file of the addon this editor was built with, by path inside the addon, sorted. build.rs makes it.
const ADDON_FILES: &[(&str, [u8; 32])] = include!(concat!(env!("OUT_DIR"), "/addon_files.rs"));

const ADDON: &str = "addons/func_godot/";
const PACK: &str = "trenchdot/entities/";
const OWN: [&str; 2] = ["res://addons/func_godot/", "res://trenchdot/entities/"];

/// How much one scan looks through before it gives up and counts the project as holding code.
pub struct Limits {
    pub entries: usize,
    pub time: Duration,
}

/// A project made with TrenchDot and its content packs stays well below these.
pub const LIMITS: Limits = Limits { entries: 20_000, time: Duration::from_secs(2) };

/// Files read whole, to hash or parse them, stay below this.
const MAX_READ: u64 = 64 << 20;

pub const TOO_BIG: &str = "holds more files than TrenchDot looks through";

/// What makes a project hold code: a file, relative to the project with forward slashes, and why it counts.
#[derive(Clone, Debug, PartialEq)]
pub struct Code {
    pub path: String,
    pub why: String,
}

/// The first thing in the project at `root` that Godot could run, None when there is nothing.
pub fn find(root: &Path) -> Option<Code> {
    find_within(root, &LIMITS)
}

pub fn find_within(root: &Path, limits: &Limits) -> Option<Code> {
    Scan { root, limits, started: Instant::now(), entries: 0 }.run().err()
}

fn code(path: &str, why: impl Into<String>) -> Code {
    Code { path: path.to_string(), why: why.into() }
}

struct Scan<'a> {
    root: &'a Path,
    limits: &'a Limits,
    started: Instant,
    entries: usize,
}

impl Scan<'_> {
    fn run(&mut self) -> Result<(), Code> {
        let mut dirs = vec![String::new()];
        while let Some(dir) = dirs.pop() {
            let entries = std::fs::read_dir(self.root.join(&dir)).map_err(|e| code(&dir, format!("could not be read: {e}")))?;
            for entry in entries {
                let entry = entry.map_err(|e| code(&dir, format!("could not be read: {e}")))?;
                self.entries += 1;
                if self.entries > self.limits.entries {
                    return Err(code("", TOO_BIG));
                }

                let name = entry.file_name().to_string_lossy().into_owned();
                let rel = if dir.is_empty() { name.clone() } else { format!("{dir}/{name}") };
                if name == ".git" {
                    continue;
                }

                // A link may lead out of the project, where Godot follows it but this scan does not.
                let kind = entry.file_type().map_err(|e| code(&rel, format!("could not be read: {e}")))?;
                if kind.is_symlink() {
                    return Err(code(&rel, "is a link"));
                } else if kind.is_dir() {
                    let lower = name.to_ascii_lowercase();
                    if lower.ends_with(".framework") || lower.ends_with(".xcframework") {
                        return Err(code(&rel, "is a native library"));
                    }

                    dirs.push(rel);
                } else {
                    file(&rel, &name, &entry.path())?;
                }

                if self.started.elapsed() > self.limits.time {
                    return Err(code("", TOO_BIG));
                }
            }
        }

        Ok(())
    }
}

/// The checks for one file, by the folder it is in, its name and its extension.
fn file(rel: &str, name: &str, path: &Path) -> Result<(), Code> {
    let lower = name.to_ascii_lowercase();
    let ext = lower.rsplit_once('.').map_or("", |(_, e)| e);
    if let Some(inside) = rel.strip_prefix(ADDON) {
        // The release zip leaves these out.
        if inside.starts_with(".git") && !inside.contains('/') {
            return Ok(());
        }

        return own(rel, ext, path, expected(inside), "the TrenchDot addon");
    }

    if let Some(inside) = rel.strip_prefix(PACK) {
        // What the pack installed, read by the editor and the addon as JSON. Its hashes are not trusted here.
        if inside == crate::entity_pack::MANIFEST {
            return Ok(());
        }

        return own(rel, ext, path, expected(&format!("{}/{inside}", crate::entity_pack::TEMPLATE_IN_ADDON)), "the Gameplay entities pack");
    }

    // Godot finds these by name, and on Windows the case does not matter.
    match rel.to_ascii_lowercase().as_str() {
        "project.godot" | "override.cfg" => return settings(rel, &text(rel, path)?),
        "project.binary" => return Err(code(rel, "holds project settings TrenchDot cannot read")),
        "export_presets.cfg" => return export_presets(rel, &text(rel, path)?),
        ".godot/extension_list.cfg" if !text(rel, path)?.trim().is_empty() => return Err(code(rel, "lists native extensions for Godot to load")),
        ".godot/uid_cache.bin" => return uid_cache(rel, &read(rel, path)?),
        _ => {}
    }

    match ext {
        "gd" | "gdc" | "cs" => Err(code(rel, "is a script")),
        "csproj" | "sln" | "slnx" | "props" | "targets" | "rsp" => Err(code(rel, "is read by MSBuild, which runs tasks when Godot builds C#")),
        "gdextension" | "dll" | "so" | "dylib" => Err(code(rel, "is a native library")),
        "gradle" | "kts" => Err(code(rel, "is a Gradle build script, which runs when Godot exports for Android")),
        _ if lower == "gradlew" || lower == "gradlew.bat" => Err(code(rel, "is a Gradle build script, which runs when Godot exports for Android")),
        "blend" => Err(code(rel, "is opened in Blender, which can run scripts in it, when Godot imports it")),
        "remap" => Err(code(rel, "sends Godot to load another file in place of one")),
        "res" | "scn" => Err(code(rel, "is a binary resource, which can hold scripts")),
        "tscn" | "tres" | "escn" => text_resource(rel, &read(rel, path)?),
        "import" => import(rel, &text(rel, path)?),
        "tdm" | "gtm" => map(rel, &read(rel, path)?),
        "map" | "vmf" if text(rel, path)?.to_ascii_lowercase().contains("logic_script") => {
            Err(code(rel, "has a logic_script, which runs GDScript from the map"))
        }
        "uid" | "map" | "vmf" => Ok(()),
        _ => binary_resource(rel, path),
    }
}

fn expected(inside: &str) -> Option<[u8; 32]> {
    ADDON_FILES.binary_search_by(|(p, _)| (*p).cmp(inside)).ok().map(|i| ADDON_FILES[i].1)
}

/// A file of TrenchDot's own: fine when its bytes are the ones this editor installs.
fn own(rel: &str, ext: &str, path: &Path, expected: Option<[u8; 32]>, what: &str) -> Result<(), Code> {
    let bytes = read(rel, path)?;
    if expected.is_some_and(|hash| ring::digest::digest(&ring::digest::SHA256, &bytes).as_ref() == hash) {
        return Ok(());
    }

    match ext {
        // Godot writes these beside the files it imports or loads.
        "uid" => Ok(()),
        "import" => import(rel, &String::from_utf8_lossy(&bytes)),
        _ if expected.is_some() => Err(code(rel, format!("differs from {what} this TrenchDot installs"))),
        _ => Err(code(rel, format!("is not part of {what} this TrenchDot installs"))),
    }
}

fn read(rel: &str, path: &Path) -> Result<Vec<u8>, Code> {
    let size = std::fs::metadata(path).map_err(|e| code(rel, format!("could not be read: {e}")))?.len();
    if size > MAX_READ {
        return Err(code(rel, "is too big to look through"));
    }

    std::fs::read(path).map_err(|e| code(rel, format!("could not be read: {e}")))
}

fn text(rel: &str, path: &Path) -> Result<String, Code> {
    read(rel, path).map(|b| String::from_utf8_lossy(&b).into_owned())
}

/// Godot's binary resources start with RSRC, or RSCC when compressed, whatever their extension.
fn binary_resource(rel: &str, path: &Path) -> Result<(), Code> {
    use std::io::Read;
    let mut magic = [0; 4];
    let mut file = std::fs::File::open(path).map_err(|e| code(rel, format!("could not be read: {e}")))?;
    let n = file.read(&mut magic).map_err(|e| code(rel, format!("could not be read: {e}")))?;
    match &magic[..n] {
        b"RSRC" | b"RSCC" => Err(code(rel, "is a binary resource, which can hold scripts")),
        _ => Ok(()),
    }
}

/// project.godot and override.cfg: autoloads, editor plugins, a command the game runs through and paths out of the
/// project. The addon's own plugin and scripts are fine.
fn settings(rel: &str, text: &str) -> Result<(), Code> {
    let mut section = "";
    for entry in entries(text) {
        let (key, value) = match entry {
            Entry::Header(header) => {
                section = header.trim();
                continue;
            }
            Entry::Key(key, value) => (key, value),
        };
        let full = if section.is_empty() { key.to_string() } else { format!("{section}/{key}") };
        let strings = strings(value);
        if let Some(path) = strings.iter().find(|s| leaves_project(s)) {
            return Err(code(rel, format!("names {path}, outside the project")));
        }

        if names_script(value) {
            return Err(code(rel, format!("gives {full} a script")));
        }

        let set = strings.iter().any(|s| !s.trim().is_empty());
        let why = match full.as_str() {
            _ if full.starts_with("autoload/") && !strings.iter().all(|s| own_path(s.trim_start_matches('*'))) => "adds an autoload",
            "editor_plugins/enabled" if !strings.iter().all(|s| s == crate::addon_install::PLUGIN) => "enables an editor plugin",
            "editor/run/main_run_args" if set => "runs the game through a command of its own",
            "application/run/main_loop_type" if set && strings.iter().any(|s| s != "SceneTree") => "runs the game from a main loop of its own",
            "application/config/project_settings_override" if set => "loads project settings from another file",
            _ => continue,
        };
        return Err(code(rel, why));
    }

    Ok(())
}

/// export_presets.cfg: what runs a program or loads a library when Godot exports or deploys the game.
fn export_presets(rel: &str, text: &str) -> Result<(), Code> {
    for entry in entries(text) {
        let Entry::Key(key, value) = entry else { continue };
        let on = value.trim() == "true";
        let set = strings(value).iter().any(|s| !s.trim().is_empty());
        let runs = (key.ends_with("ssh_remote_deploy/enabled") && on)
            || (key.ends_with("gradle_build/use_gradle_build") && on)
            || ((key.ends_with("custom_template/debug") || key.ends_with("custom_template/release")) && set)
            || (key.ends_with("codesign/custom_options") && set)
            || names_script(value);
        if runs {
            return Err(code(rel, format!("sets {key}, which runs a program when Godot exports the game")));
        }
    }

    Ok(())
}

/// Godot's `.import` files: an import script, or an imported file read from outside the project.
fn import(rel: &str, text: &str) -> Result<(), Code> {
    for entry in entries(text) {
        let Entry::Key(key, value) = entry else { continue };
        let strings = strings(value);
        if let Some(path) = strings.iter().find(|s| leaves_project(s)) {
            return Err(code(rel, format!("names {path}, outside the project")));
        }

        if (key.contains("script") && strings.iter().any(|s| !s.trim().is_empty())) || names_script(value) {
            return Err(code(rel, "imports with a script"));
        }
    }

    Ok(())
}

/// A text scene or resource: scripts in it, scripts it uses other than TrenchDot's own, and paths out of the project.
/// An imported map holds the map's bytes, which can hold GDScript too.
fn text_resource(rel: &str, bytes: &[u8]) -> Result<(), Code> {
    if bytes.starts_with(b"RSRC") || bytes.starts_with(b"RSCC") {
        return Err(code(rel, "is a binary resource, which can hold scripts"));
    }

    let text = String::from_utf8_lossy(bytes);
    let mut own_ids = BTreeMap::new();
    for entry in entries(&text) {
        match entry {
            Entry::Header(header) => {
                let attrs = attrs(header);
                let attr = |name: &str| attrs.iter().find(|(k, _)| *k == name).map(|(_, v)| v.as_str()).unwrap_or_default();
                let tag = header.split_whitespace().next().unwrap_or_default();
                for name in ["path", "instance_placeholder"] {
                    if leaves_project(attr(name)) {
                        return Err(code(rel, format!("names {}, outside the project", attr(name))));
                    }
                }

                let script = attr("type").contains("Script");
                match tag {
                    "gd_resource" | "gd_scene" | "sub_resource" if script => return Err(code(rel, "holds a script")),
                    "ext_resource" if script && !own_path(attr("path")) => return Err(code(rel, format!("uses the script {}", attr("path")))),
                    "ext_resource" => {
                        own_ids.insert(attr("id").to_string(), own_path(attr("path")));
                    }
                    _ => {}
                }
            }
            Entry::Key(key, value) => {
                if let Some(path) = strings(value).iter().find(|s| s.starts_with("res://") && leaves_project(s)) {
                    return Err(code(rel, format!("names {path}, outside the project")));
                }

                for path in resource_calls(value) {
                    if leaves_project(&path) {
                        return Err(code(rel, format!("names {path}, outside the project")));
                    }
                }

                let value = value.trim();
                let own_script = match value.strip_prefix("ExtResource(").and_then(|v| v.strip_suffix(')')) {
                    Some(id) => own_ids.get(id.trim().trim_matches('"')).copied().unwrap_or(false),
                    None => value == "null",
                };
                if key == "script/source" || (key == "script" && !own_script) || names_script(value) {
                    return Err(code(rel, "holds a script"));
                }

                if key == "map_bytes" {
                    let map_bytes = packed_bytes(value).ok_or_else(|| code(rel, "holds a map TrenchDot cannot read"))?;
                    map(rel, &map_bytes)?;
                }
            }
        }
    }

    Ok(())
}

/// A TrenchDot map: a logic_script with code, or I/O that can hand one code, runs GDScript when the scene plays.
fn map(rel: &str, bytes: &[u8]) -> Result<(), Code> {
    match td_doc::format::from_bytes(bytes) {
        Ok(loaded) if loaded.problems.is_empty() && !crate::validate::holds_script(&loaded.map) => Ok(()),
        Ok(loaded) if loaded.problems.is_empty() => Err(code(rel, "has a logic_script, which runs GDScript from the map")),
        _ => Err(code(rel, "is a map TrenchDot cannot read whole")),
    }
}

/// Godot's cache of resource UIDs: a u32 count, then per entry the u64 UID and the path as a u32 length and UTF-8.
fn uid_cache(rel: &str, bytes: &[u8]) -> Result<(), Code> {
    let damaged = || code(rel, "could not be read");
    let u32_at = |at: usize| bytes.get(at..at + 4).map(|b| u32::from_le_bytes(b.try_into().expect("four bytes")) as usize);
    let count = u32_at(0).ok_or_else(damaged)?;
    let mut at = 4;
    for _ in 0..count {
        let len = u32_at(at + 8).ok_or_else(damaged)?;
        let path = bytes.get(at + 12..at + 12 + len).and_then(|b| std::str::from_utf8(b).ok()).ok_or_else(damaged)?;
        if !path.starts_with("res://") || leaves_project(path) {
            return Err(code(rel, format!("sends a resource to {path}, outside the project")));
        }

        at += 12 + len;
    }

    if at == bytes.len() { Ok(()) } else { Err(damaged()) }
}

/// Whether Godot reads `path` from outside what the scan looked through: outside the project, or in the .git folder it
/// skips. Paths without res:// are relative to the project.
fn leaves_project(path: &str) -> bool {
    let path = path.trim_start_matches('*');
    if path.starts_with("user://") || path.starts_with("uid://") {
        return false;
    }

    let inner = path.strip_prefix("res://");
    let absolute = path.starts_with(['/', '\\']) || path.as_bytes().get(1..3).is_some_and(|b| b[0] == b':' && (b[1] == b'/' || b[1] == b'\\'));
    (inner.is_none() && absolute) || inner.unwrap_or(path).split(['/', '\\']).any(|part| part == ".." || part.eq_ignore_ascii_case(".git"))
}

/// A path into TrenchDot's addon or Gameplay pack folder.
fn own_path(path: &str) -> bool {
    OWN.iter().any(|own| path.starts_with(own)) && !leaves_project(path) && !path.contains('\\')
}

/// Whether a value sets the script of an object written inline, as project.godot does for input events.
fn names_script(value: &str) -> bool {
    value.match_indices("\"script\"").any(|(at, m)| {
        let rest = value[at + m.len()..].trim_start();
        rest.strip_prefix(':').is_some_and(|r| !r.trim_start().starts_with("null"))
    })
}

/// The paths of `Resource("...")` in a value, which Godot loads as it reads the file.
fn resource_calls(value: &str) -> Vec<String> {
    value
        .match_indices("Resource(")
        .filter(|(at, _)| !value[..*at].ends_with("Ext") && !value[..*at].ends_with("Sub"))
        .filter_map(|(at, m)| strings(&value[at + m.len()..]).into_iter().next())
        .collect()
}

/// The bytes of `PackedByteArray("base64")`, or of `PackedByteArray(1, 2, 3)` as older files write it.
fn packed_bytes(value: &str) -> Option<Vec<u8>> {
    let inner = value.strip_prefix("PackedByteArray(")?.strip_suffix(')')?.trim();
    match inner.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        Some(b64) => base64::engine::general_purpose::STANDARD.decode(b64).ok(),
        None if inner.is_empty() => Some(Vec::new()),
        None => inner.split(',').map(|n| n.trim().parse().ok()).collect(),
    }
}

/// A part of Godot's text formats, project.godot, `.import`, `.tscn` and `.tres` alike.
enum Entry<'a> {
    /// The text between the brackets of `[section]` or `[tag key="value"]`.
    Header(&'a str),
    /// `key=value`, the value possibly over several lines.
    Key(&'a str, &'a str),
}

/// The headers and keys of a file in Godot's text format. Brackets and strings may run over several lines.
fn entries(text: &str) -> Vec<Entry<'_>> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at].is_ascii_whitespace() {
            at += 1;
            continue;
        }

        let end = entry_end(text, at);
        let part = &text[at..end];
        match bytes[at] {
            b';' | b'#' => {}
            b'[' => out.push(Entry::Header(part.trim_end().strip_prefix('[').and_then(|h| h.strip_suffix(']')).unwrap_or(part))),
            _ => {
                if let Some(eq) = outside_strings(part).find(|&(_, c)| c == b'=').map(|(i, _)| i) {
                    out.push(Entry::Key(part[..eq].trim().trim_matches('"'), part[eq + 1..].trim()));
                }
            }
        }

        at = end;
    }

    out
}

/// Where the entry starting at `start` ends: the first line break outside strings, comments and brackets.
fn entry_end(text: &str, start: usize) -> usize {
    let mut depth = 0i64;
    for (i, c) in outside_strings(&text[start..]) {
        match c {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b'\n' if depth <= 0 => return start + i,
            _ => {}
        }
    }

    text.len()
}

/// The bytes of `text` outside strings and comments, with their offsets. A quote in a comment opens no string, the
/// way Godot reads it, so nothing after it hides from the scan.
fn outside_strings(text: &str) -> impl Iterator<Item = (usize, u8)> + '_ {
    let (mut quoted, mut escaped, mut comment) = (false, false, false);
    text.bytes().enumerate().filter(move |&(_, c)| {
        if comment {
            comment = c != b'\n';
            return !comment;
        }

        if quoted {
            match c {
                _ if escaped => escaped = false,
                b'\\' => escaped = true,
                b'"' => quoted = false,
                _ => {}
            }

            return false;
        }

        match c {
            b'"' => quoted = true,
            b';' | b'#' => comment = true,
            _ => return true,
        }

        false
    })
}

/// The `key=value` attributes of a header, values unquoted.
fn attrs(header: &str) -> Vec<(&str, String)> {
    let mut out = Vec::new();
    let mut rest = header.split_once(char::is_whitespace).map_or("", |(_, r)| r);
    while let Some(eq) = rest.find('=') {
        let key = rest[..eq].trim();
        let value = rest[eq + 1..].trim_start();
        let end = match value.strip_prefix('"') {
            Some(quoted) => 1 + string_end(quoted),
            None => value.find(char::is_whitespace).unwrap_or(value.len()),
        };
        let raw = &value[..end];
        out.push((key, strings(raw).into_iter().next().unwrap_or_else(|| raw.to_string())));
        rest = &value[end..];
    }

    out
}

/// The length of a string's text and its closing quote, `text` starting right after the opening one.
fn string_end(text: &str) -> usize {
    let mut escaped = false;
    for (i, c) in text.bytes().enumerate() {
        match c {
            _ if escaped => escaped = false,
            b'\\' => escaped = true,
            b'"' => return i + 1,
            _ => {}
        }
    }

    text.len()
}

/// The strings in a value, with Godot's escapes undone, so `\u002e\u002e` reads as the `..` it is.
fn strings(value: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = value.chars();
    while chars.any(|c| c == '"') {
        let mut s = String::new();
        while let Some(c) = chars.next() {
            match c {
                '"' => break,
                '\\' => match chars.next() {
                    Some('n') => s.push('\n'),
                    Some('t') => s.push('\t'),
                    Some('r') => s.push('\r'),
                    Some(u @ ('u' | 'U')) => {
                        let hex: String = chars.by_ref().take(if u == 'u' { 4 } else { 6 }).collect();
                        s.extend(u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32));
                    }
                    Some(c) => s.push(c),
                    None => {}
                },
                c => s.push(c),
            }
        }

        out.push(s);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, bytes: impl AsRef<[u8]>) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, bytes).unwrap();
    }

    /// A project the way Godot 4.7 makes a new one.
    fn fresh() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(
            root,
            "project.godot",
            "; Engine configuration file.\n\nconfig_version=5\n\n[application]\n\nconfig/name=\"Castle\"\nconfig/features=PackedStringArray(\"4.7\", \"Forward Plus\")\nconfig/icon=\"res://icon.svg\"\n\n[input]\n\njump={\n\"deadzone\": 0.2,\n\"events\": [Object(InputEventKey,\"resource_local_to_scene\":false,\"keycode\":32,\"script\":null)\n]\n}\n",
        );
        write(root, "icon.svg", "<svg xmlns=\"http://www.w3.org/2000/svg\"/>");
        write(
            root,
            "icon.svg.import",
            "[remap]\n\nimporter=\"texture\"\ntype=\"CompressedTexture2D\"\nuid=\"uid://b1\"\npath=\"res://.godot/imported/icon.svg-1.ctex\"\n\n[deps]\n\nsource_file=\"res://icon.svg\"\ndest_files=[\"res://.godot/imported/icon.svg-1.ctex\"]\n\n[params]\n\ncompress/mode=0\n",
        );
        write(root, ".gitignore", "# Godot 4+ specific ignores\n.godot/\n");
        write(root, ".editorconfig", "root = true\n");
        dir
    }

    /// The addon this editor was built with, copied the way the release zip has it.
    fn copy_addon(root: &Path) {
        let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot/addons/func_godot");
        for (rel, _) in ADDON_FILES {
            write(root, &format!("{ADDON}{rel}"), std::fs::read(from.join(rel)).unwrap());
        }
    }

    /// A project with the addon enabled, the Gameplay pack installed and what Godot leaves after opening it.
    fn own_project() -> tempfile::TempDir {
        let dir = fresh();
        let root = dir.path();
        copy_addon(root);
        crate::addon_install::enable(root).unwrap();
        crate::entity_pack::install(root).unwrap();
        write(root, "trenchdot/entities/scripts/td_door.gd.uid", "uid://c2\n");
        let mut map = td_doc::Map::new();
        let layer = map.default_layer();
        map.insert(layer, td_doc::NodeKind::Entity(td_doc::Entity::new("func_door")));
        std::fs::create_dir_all(root.join("maps")).unwrap();
        td_doc::format::save(&map, &root.join("maps/yard.tdm")).unwrap();
        write(root, ".godot/imported/yard.tdm-2.tres", imported_map(&td_doc::format::to_bytes(&map).unwrap()));
        write(root, ".godot/imported/icon.svg-1.ctex", b"GST2....");
        write(root, ".godot/editor/project_metadata.cfg", "[editor_metadata]\n\nexecutable_path=\"C:/Godot/godot.exe\"\n");
        write(root, ".godot/uid_cache.bin", uid_cache_bytes(&["res://icon.svg", "res://addons/func_godot/plugin.cfg"]));
        write(root, ".godot/global_script_class_cache.cfg", "list=[]\n");
        dir
    }

    fn imported_map(bytes: &[u8]) -> String {
        let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
        format!(
            "[gd_resource type=\"Resource\" script_class=\"TrenchDotImportedMap\" format=4]\n\n[ext_resource type=\"Script\" uid=\"uid://c3\" path=\"res://addons/func_godot/src/import/tdm_imported_map.gd\" id=\"1_ab\"]\n\n[resource]\nscript = ExtResource(\"1_ab\")\nrevision = 1\nmap_bytes = PackedByteArray(\"{b64}\")\n"
        )
    }

    fn uid_cache_bytes(paths: &[&str]) -> Vec<u8> {
        let mut out = (paths.len() as u32).to_le_bytes().to_vec();
        for (i, path) in paths.iter().enumerate() {
            out.extend((i as u64).to_le_bytes());
            out.extend((path.len() as u32).to_le_bytes());
            out.extend(path.as_bytes());
        }

        out
    }

    fn found(root: &Path) -> Option<String> {
        find(root).map(|c| c.path)
    }

    #[test]
    fn empty_and_fresh_projects_hold_no_code() {
        let empty = tempfile::tempdir().unwrap();
        assert_eq!(find(empty.path()), None);
        let fresh = fresh();
        assert_eq!(find(fresh.path()), None);
    }

    #[test]
    fn trenchdot_s_own_addon_and_pack_do_not_count_while_unchanged() {
        assert!(ADDON_FILES.len() > 100 && expected("plugin.cfg").is_some(), "build.rs hashed the addon");
        let dir = own_project();
        let root = dir.path();
        assert_eq!(find(root), None);

        let plugin = root.join("addons/func_godot/plugin.cfg");
        let original = std::fs::read(&plugin).unwrap();
        std::fs::write(&plugin, [original.as_slice(), b"\n"].concat()).unwrap();
        let changed = find(root).unwrap();
        assert_eq!(changed.path, "addons/func_godot/plugin.cfg");
        assert!(changed.why.contains("differs from the TrenchDot addon"), "{}", changed.why);
        std::fs::write(&plugin, original).unwrap();

        for extra in ["addons/func_godot/notes.txt", "trenchdot/entities/scripts/td_mine.gd", "trenchdot/entities/definitions/mine.tres"] {
            write(root, extra, "x");
            assert_eq!(found(root).as_deref(), Some(extra), "an extra file in TrenchDot's folders counts");
            std::fs::remove_file(root.join(extra)).unwrap();
        }

        let door = root.join("trenchdot/entities/scripts/td_door.gd");
        std::fs::write(&door, "extends Node\nfunc _ready(): OS.execute(\"calc\", [])\n").unwrap();
        assert_eq!(found(root).as_deref(), Some("trenchdot/entities/scripts/td_door.gd"), "an edited pack script counts");
    }

    #[test]
    fn scripts_scenes_resources_and_libraries_count() {
        let cases: &[(&str, &[u8])] = &[
            ("player.gd", b"extends Node\n"),
            ("src/Player.CS", b"class Player {}"),
            ("Game.csproj", b"<Project Sdk=\"Godot.NET.Sdk/4.7.2\"/>"),
            ("Directory.Build.props", b"<Project/>"),
            ("bin/ext.gdextension", b"[configuration]\n"),
            ("bin/ext.dll", b"MZ"),
            ("level.scn", b"RSCC...."),
            (".godot/imported/model.glb-3.scn", b"RSCC...."),
            (".godot/imported/step.wav-4.sample", b"RSRC...."),
            ("art/hidden.png", b"RSRC...."),
            ("models/house.blend", b"BLENDER"),
            ("level.tscn.remap", b"[remap]\n\npath=\"res://level.scn\"\n"),
            ("android/build/build.gradle", b"plugins {}"),
            (".godot/extension_list.cfg", b"res://bin/ext.gdextension\n"),
            ("maps/old.map", b"{\n\"classname\" \"logic_script\"\n\"source\" \"print(1)\"\n}\n"),
            ("project.binary", b"ECFG"),
        ];
        for (rel, bytes) in cases {
            let dir = fresh();
            write(dir.path(), rel, bytes);
            assert_eq!(found(dir.path()).as_deref(), Some(*rel));
        }
    }

    #[test]
    fn text_scenes_count_when_they_hold_or_use_a_script() {
        let scene = |body: &str| format!("[gd_scene format=3 uid=\"uid://a1\"]\n\n{body}\n");
        let cases = [
            scene(
                "[sub_resource type=\"GDScript\" id=\"G\"]\nscript/source = \"extends Node\nfunc _ready():\n\tOS.execute(\\\"calc\\\", [])\n\"\n\n[node name=\"Main\" type=\"Node\"]\nscript = SubResource(\"G\")",
            ),
            scene("[ext_resource type=\"Script\" path=\"res://player.gd\" id=\"1\"]\n\n[node name=\"Main\" type=\"Node\"]\nscript = ExtResource(\"1\")"),
            scene("[ext_resource type=\"Script\" path=\"res://addons/func_godot/\\u002e\\u002e/evil.gd\" id=\"1\"]"),
            scene("[ext_resource type=\"PackedScene\" path=\"res://../outside/level.tscn\" id=\"1\"]"),
            scene("[ext_resource type=\"PackedScene\" path=\"C:/Users/someone/level.tscn\" id=\"1\"]"),
            scene("[node name=\"Main\" type=\"Node\"]\nscript = Resource(\"res://player.gd\")"),
            scene("[node name=\"Main\" type=\"Node\"]\nmetadata/x = Resource(\"//host/share/x.tres\")"),
            "[gd_resource type=\"GDScript\" format=3]\n\n[resource]\nscript/source = \"\"\n".to_string(),
            "[gd_resource type=\"Resource\" format=3]\n\n[resource]\nitems = [Object(RefCounted,\"script\":ExtResource(\"1\"))]\n".to_string(),
        ];
        for text in cases {
            let dir = fresh();
            write(dir.path(), "main.tscn", &text);
            assert_eq!(found(dir.path()).as_deref(), Some("main.tscn"), "{text}");
        }

        let dir = fresh();
        let root = dir.path();
        write(
            root,
            "main.tscn",
            scene(
                "[ext_resource type=\"Script\" path=\"res://addons/func_godot/src/map/func_godot_map.gd\" id=\"1\"]\n[ext_resource type=\"Texture2D\" path=\"res://icon.svg\" id=\"2\"]\n\n[node name=\"Map\" type=\"Node3D\"]\nscript = ExtResource(\"1\")\nlabel = \"see ../notes and /help\"\n\n[connection signal=\"ready\" from=\".\" to=\".\" method=\"show\"]",
            ),
        );
        write(root, "wall.tres", "[gd_resource type=\"StandardMaterial3D\" format=3]\n\n[resource]\nalbedo_color = Color(1, 0, 0, 1)\n");
        assert_eq!(find(root), None, "TrenchDot's own scripts, textures and plain resources hold no code");
    }

    #[test]
    fn maps_count_when_they_hold_gdscript() {
        let dir = own_project();
        let root = dir.path();
        let mut map = td_doc::Map::new();
        let layer = map.default_layer();
        let mut script = td_doc::Entity::new("logic_script");
        script.properties.insert("source".into(), "OS.execute(\"calc\", [])".into());
        map.insert(layer, td_doc::NodeKind::Entity(script));
        td_doc::format::save(&map, &root.join("maps/trap.tdm")).unwrap();
        assert_eq!(found(root).as_deref(), Some("maps/trap.tdm"));
        std::fs::remove_file(root.join("maps/trap.tdm")).unwrap();

        let old = format!("maps/trap.{}", td_doc::legacy::MAP_EXTS[1]);
        td_doc::format::save(&map, &root.join(&old)).unwrap();
        assert_eq!(found(root).as_deref(), Some(old.as_str()), "an old map counts like a new one");
        std::fs::remove_file(root.join(&old)).unwrap();

        write(root, ".godot/imported/yard.tdm-2.tres", imported_map(&td_doc::format::to_bytes(&map).unwrap()));
        assert_eq!(found(root).as_deref(), Some(".godot/imported/yard.tdm-2.tres"), "the map Godot imported counts too");
        write(root, ".godot/imported/yard.tdm-2.tres", imported_map(b"not a map"));
        assert_eq!(found(root).as_deref(), Some(".godot/imported/yard.tdm-2.tres"));
    }

    #[test]
    fn project_settings_count_when_they_run_code() {
        let base = "config_version=5\n\n[application]\n\nconfig/name=\"Castle\"\n";
        let cases = [
            "[autoload]\n\nGame=\"*res://game.tscn\"\n",
            "[autoload]\n\nGame=\"*uid://c4\"\n",
            "[editor_plugins]\n\nenabled=PackedStringArray(\"res://addons/func_godot/plugin.cfg\", \"res://addons/other/plugin.cfg\")\n",
            "[editor]\n\nrun/main_run_args=\"calc.exe %command%\"\n",
            "[editor/run]\n\nmain_run_args=\"calc.exe %command%\"\n",
            "[application]\n\nrun/main_loop_type=\"MyLoop\"\n",
            "[application]\n\nconfig/project_settings_override=\"res://more.cfg\"\n",
            "[dotnet]\n\nproject/solution_directory=\"../elsewhere\"\n",
            "[input]\n\njump={\n\"events\": [Object(InputEventKey,\"keycode\":32,\"script\":Resource(\"res://key.gd\"))]\n}\n",
        ];
        for extra in cases {
            for file in ["project.godot", "override.cfg"] {
                let dir = fresh();
                let text = if file == "project.godot" { format!("{base}\n{extra}") } else { extra.to_string() };
                write(dir.path(), file, text);
                assert_eq!(found(dir.path()).as_deref(), Some(file), "{extra}");
            }
        }

        let dir = fresh();
        write(
            dir.path(),
            "project.godot",
            format!(
                "{base}\n[autoload]\n\nReload=\"*res://addons/func_godot/src/trenchdot/runtime/trenchdot_hot_reload.gd\"\n\n[editor_plugins]\n\nenabled=PackedStringArray(\"res://addons/func_godot/plugin.cfg\")\n"
            ),
        );
        assert_eq!(find(dir.path()), None, "TrenchDot's own plugin and scripts are fine");
    }

    #[test]
    fn imports_caches_and_export_presets_count_when_they_run_code() {
        let cases = [
            (
                "ship.glb.import",
                "[remap]\n\nimporter=\"scene\"\npath=\"res://.godot/imported/ship.glb-5.scn\"\n\n[params]\n\nimport_script/path=\"res://post.gd\"\n",
            ),
            ("wall.png.import", "[remap]\n\nimporter=\"texture\"\npath=\"res://../outside/wall.ctex\"\n"),
            ("export_presets.cfg", "[preset.0]\n\nname=\"Windows\"\n\n[preset.0.options]\n\nssh_remote_deploy/enabled=true\n"),
            ("export_presets.cfg", "[preset.0.options]\n\ncustom_template/release=\"res://my_template.exe\"\n"),
            ("export_presets.cfg", "[preset.0.options]\n\ngradle_build/use_gradle_build=true\n"),
        ];
        for (rel, text) in cases {
            let dir = fresh();
            write(dir.path(), rel, text);
            assert_eq!(found(dir.path()).as_deref(), Some(rel), "{text}");
        }

        let dir = fresh();
        let presets = "[preset.0]\n\nname=\"Windows\"\nexport_path=\"../build/castle.exe\"\n\n[preset.0.options]\n\nssh_remote_deploy/enabled=false\nssh_remote_deploy/run_script=\"#!/usr/bin/env bash\nunzip -o -q \\\"{temp_dir}/{archive_name}\\\"\"\ncustom_template/debug=\"\"\n";
        write(dir.path(), "export_presets.cfg", presets);
        assert_eq!(find(dir.path()), None, "plain export presets hold no code");

        for paths in [&["res://../outside/level.tscn"][..], &["C:/outside/level.tscn"]] {
            write(dir.path(), ".godot/uid_cache.bin", uid_cache_bytes(paths));
            assert_eq!(found(dir.path()).as_deref(), Some(".godot/uid_cache.bin"));
        }

        let mut damaged = uid_cache_bytes(&["res://icon.svg"]);
        damaged.push(0);
        write(dir.path(), ".godot/uid_cache.bin", damaged);
        assert_eq!(found(dir.path()).as_deref(), Some(".godot/uid_cache.bin"), "a cache that does not read whole counts");
    }

    #[test]
    fn a_scan_stops_at_its_limit_and_skips_git() {
        let dir = fresh();
        let root = dir.path();
        write(root, ".git/hooks/pre-commit", "#!/bin/sh\n");
        write(root, ".git/objects/a.gd", "extends Node\n");
        assert_eq!(find(root), None, ".git is not Godot's");

        let few = Limits { entries: 3, time: Duration::from_secs(60) };
        assert_eq!(find_within(root, &few), Some(code("", TOO_BIG)));
        let none = Limits { entries: 1000, time: Duration::ZERO };
        assert_eq!(find_within(root, &none), Some(code("", TOO_BIG)));
    }

    #[test]
    fn links_count() {
        let dir = fresh();
        let outside = tempfile::tempdir().unwrap();
        let link = dir.path().join("addons");
        // Making a symbolic link needs Developer Mode on Windows, a junction does not.
        #[cfg(windows)]
        let made = std::os::windows::fs::symlink_dir(outside.path(), &link).or_else(|_| {
            let status = std::process::Command::new("cmd").arg("/C").arg("mklink").arg("/J").arg(&link).arg(outside.path()).output()?.status;
            if status.success() { Ok(()) } else { Err(std::io::Error::other("mklink failed")) }
        });
        #[cfg(unix)]
        let made = std::os::unix::fs::symlink(outside.path(), &link);
        if made.is_err() {
            eprintln!("skipped, this account cannot make links");
            return;
        }

        assert_eq!(found(dir.path()).as_deref(), Some("addons"));
    }

    #[test]
    fn godot_text_reads_like_godot_reads_it() {
        let text =
            "; comment = \"x\"\na=1\n[section]\nb=\"line\none\"\nc={\n\"k\": [1,\n2]\n}\n[ext_resource type=\"Script\" path=\"res://a b.gd\" id=\"1_x\"]\n";
        let parts: Vec<String> = entries(text)
            .into_iter()
            .map(|e| match e {
                Entry::Header(h) => format!("[{h}]"),
                Entry::Key(k, v) => format!("{k}={v}"),
            })
            .collect();
        assert_eq!(parts, ["a=1", "[section]", "b=\"line\none\"", "c={\n\"k\": [1,\n2]\n}", "[ext_resource type=\"Script\" path=\"res://a b.gd\" id=\"1_x\"]"]);
        let header = "ext_resource type=\"Script\" path=\"res://a b.gd\" id=\"1_x\"";
        assert_eq!(attrs(header), [("type", "Script".to_string()), ("path", "res://a b.gd".to_string()), ("id", "1_x".to_string())]);
        assert_eq!(strings(r#"["a\"b", "\u002e\u002e"]"#), ["a\"b", ".."]);
        assert!(leaves_project("res://a/../../b") && leaves_project("..\\b") && leaves_project("/etc/x") && leaves_project("D:\\x"));
        assert!(!leaves_project("res://a/b..c") && !leaves_project("user://save") && !leaves_project("uid://x") && !leaves_project("Wait.. what"));
        assert_eq!(packed_bytes("PackedByteArray(1, 2, 255)"), Some(vec![1, 2, 255]));
        assert_eq!(packed_bytes("PackedByteArray(\"AQL/\")"), Some(vec![1, 2, 255]));
    }

    #[test]
    fn a_comment_does_not_hide_what_follows() {
        let dir = fresh();
        write(dir.path(), "project.godot", "config_version=5\nx=[1, ; \"\n]\n[autoload]\nGame=\"*res://game.tscn\"\n; \"\n");
        assert_eq!(found(dir.path()).as_deref(), Some("project.godot"));
    }
}
