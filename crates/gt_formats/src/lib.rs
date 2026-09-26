pub mod bbmodel;
pub mod game;
pub mod godot_api;
pub mod godot_material;
pub mod idmodel;
pub mod nature;
pub mod quake_map;
pub mod quake_texture;
pub mod texture_import;
pub mod vmf;
pub mod vmt;
pub mod vtf;

pub use game::{EntityDef, EntityKind, GameConfig, GizmoDef, IoDef, PropertyDef, PropertyType};

/// Writes `contents` to `path` through a temporary file next to it that is renamed into place, so an interrupted write
/// never leaves a partial file under the real name, which a later install would take for a finished one. Returns the
/// bytes written.
pub fn write_atomic(path: &std::path::Path, contents: &mut dyn std::io::Read) -> std::io::Result<u64> {
    let mut name = path.file_name().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "no file name"))?.to_os_string();
    name.push(".gtpart");
    let part = path.with_file_name(name);
    let _ = std::fs::remove_file(&part);
    let written = std::fs::OpenOptions::new().write(true).create_new(true).open(&part).and_then(|mut file| {
        let n = std::io::copy(contents, &mut file)?;
        file.sync_all()?;
        Ok(n)
    });
    match written.and_then(|n| std::fs::rename(&part, path).map(|_| n)) {
        Ok(n) => Ok(n),
        Err(e) => {
            let _ = std::fs::remove_file(&part);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn an_interrupted_write_leaves_no_file() {
        struct Broken;
        impl std::io::Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("disk gone"))
            }
        }

        let dir = std::env::temp_dir().join(format!("gt_atomic_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("tree.glb");
        assert!(super::write_atomic(&path, &mut Broken).is_err());
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0, "neither the file nor its temporary twin");
        assert_eq!(super::write_atomic(&path, &mut &b"glTF"[..]).unwrap(), 4);
        assert_eq!(std::fs::read(&path).unwrap(), b"glTF");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
