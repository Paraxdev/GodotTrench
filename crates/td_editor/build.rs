//! Hashes the addon in godot/addons/func_godot, which the release zip is made of, so the editor knows the bytes of the
//! addon and the Gameplay pack it installs, see `code_scan.rs`.

use std::path::Path;

fn main() {
    let addon = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot/addons/func_godot");
    println!("cargo::rerun-if-changed={}", addon.display());
    let mut files = Vec::new();
    walk(&addon, &addon, &mut files);
    files.sort();
    let mut out = String::from("&[\n");
    for (rel, hash) in files {
        out += &format!("    ({rel:?}, {hash:?}),\n");
    }

    out += "]\n";
    let path = Path::new(&std::env::var_os("OUT_DIR").expect("cargo sets OUT_DIR")).join("addon_files.rs");
    std::fs::write(&path, out).unwrap_or_else(|e| panic!("could not write {}: {e}", path.display()));
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<(String, [u8; 32])>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().replace('\\', "/");
        // The release zip leaves these out, see ci.yml.
        if dir == root && rel.starts_with(".git") {
            continue;
        }

        match entry.file_type() {
            Ok(kind) if kind.is_dir() => walk(root, &path, out),
            Ok(kind) if kind.is_file() => {
                let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()));
                let digest = ring::digest::digest(&ring::digest::SHA256, &bytes);
                out.push((rel, digest.as_ref().try_into().expect("SHA-256 has 32 bytes")));
            }
            _ => {}
        }
    }
}
