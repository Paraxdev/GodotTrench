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
    let (part, n) = write_part(path, contents)?;
    std::fs::rename(&part, path).map(|_| n).inspect_err(|_| {
        let _ = std::fs::remove_file(&part);
    })
}

/// Like [`write_atomic`], but never replaces a file. When `path` exists, even one that appeared while writing, nothing
/// changes and the error is `AlreadyExists`: the finished file is hard linked into place, which fails rather than
/// replaces. Where the file system has no hard links it is renamed after one more look.
pub fn write_new(path: &std::path::Path, contents: &mut dyn std::io::Read) -> std::io::Result<u64> {
    let exists = || std::fs::symlink_metadata(path).is_ok();
    if exists() {
        return Err(std::io::ErrorKind::AlreadyExists.into());
    }

    let (part, n) = write_part(path, contents)?;
    let placed = match std::fs::hard_link(&part, path) {
        Err(e) if e.kind() != std::io::ErrorKind::AlreadyExists && !exists() => std::fs::rename(&part, path),
        linked => linked,
    };
    let _ = std::fs::remove_file(&part);
    placed.map(|_| n)
}

/// Writes `contents` to `<path>.gtpart`, flushed to disk. Removes it again when that fails.
fn write_part(path: &std::path::Path, contents: &mut dyn std::io::Read) -> std::io::Result<(std::path::PathBuf, u64)> {
    let mut name = path.file_name().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "no file name"))?.to_os_string();
    name.push(PART_SUFFIX);
    let part = path.with_file_name(name);
    let _ = std::fs::remove_file(&part);
    let written = std::fs::OpenOptions::new().write(true).create_new(true).open(&part).and_then(|mut file| {
        let n = std::io::copy(contents, &mut file)?;
        file.sync_all()?;
        Ok(n)
    });
    match written {
        Ok(n) => Ok((part, n)),
        Err(e) => {
            let _ = std::fs::remove_file(&part);
            Err(e)
        }
    }
}

/// Ends the name of a file [`write_atomic`] or [`write_new`] is still writing. One that outlived a crash can go.
pub const PART_SUFFIX: &str = ".gtpart";

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

    #[test]
    fn write_new_never_replaces_a_file() {
        let dir = std::env::temp_dir().join(format!("gt_write_new_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("oak.glb");
        assert_eq!(super::write_new(&path, &mut &b"glTF"[..]).unwrap(), 4);
        let again = super::write_new(&path, &mut &b"other"[..]);
        assert_eq!(again.unwrap_err().kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(std::fs::read(&path).unwrap(), b"glTF");

        // A file that appears while the new one is written stays too.
        struct Racing<'a>(&'a std::path::Path, bool);
        impl std::io::Read for Racing<'_> {
            fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
                if std::mem::replace(&mut self.1, true) {
                    return Ok(0);
                }

                std::fs::write(self.0, "theirs")?;
                buf[0] = b'x';
                Ok(1)
            }
        }

        let racing = dir.join("pine.glb");
        let result = super::write_new(&racing, &mut Racing(&racing, false));
        assert_eq!(result.unwrap_err().kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(std::fs::read_to_string(&racing).unwrap(), "theirs");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 2, "no temporary file stays");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
