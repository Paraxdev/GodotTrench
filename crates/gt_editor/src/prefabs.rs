//! Loaded prefab maps referenced by instance nodes, reloaded when their file changes.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use gt_core::{Aabb, DMat4, DQuat, EulerRot};
use gt_doc::map::Instance;
use gt_doc::{Map, NodeKind, format};

pub const MAX_DEPTH: usize = 8;

pub struct PrefabEntry {
    pub map: Option<Map>,
    pub error: Option<String>,
    /// Bounds of the prefab in its own space, including nested instances.
    pub bounds: Aabb,
    modified: Option<SystemTime>,
}

#[derive(Default)]
pub struct PrefabCache {
    entries: HashMap<PathBuf, PrefabEntry>,
    /// Bumped whenever a prefab is (re)loaded, so render caches know to rebuild.
    pub generation: u64,
}

pub fn instance_transform(i: &Instance) -> DMat4 {
    let q = DQuat::from_euler(EulerRot::YXZ, i.angles.y.to_radians(), i.angles.x.to_radians(), i.angles.z.to_radians());
    DMat4::from_rotation_translation(q, i.origin)
}

/// Resolves an instance path relative to the referencing map, or through res:// in the Godot project.
pub fn resolve(path: &str, map_path: Option<&Path>, project_root: Option<&Path>) -> Option<PathBuf> {
    if let Some(rel) = path.strip_prefix("res://") {
        return project_root.map(|r| r.join(rel));
    }
    let p = Path::new(path);
    if p.is_absolute() {
        return Some(p.to_path_buf());
    }
    map_path.and_then(|m| m.parent()).map(|dir| dir.join(p))
}

fn transformed_bounds(b: &Aabb, m: &DMat4) -> Aabb {
    if b.is_empty() {
        return *b;
    }
    Aabb::from_points(b.corners().iter().map(|c| m.transform_point3(*c)))
}

impl PrefabCache {
    /// Refreshes the entry for `path` if the file changed, returns it.
    pub fn get(&mut self, path: &Path) -> &PrefabEntry {
        let modified = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        let stale = self.entries.get(path).is_none_or(|e| e.modified != modified);
        if stale {
            let entry = match format::load(path) {
                Ok(map) => PrefabEntry { bounds: Aabb::EMPTY, map: Some(map), error: None, modified },
                Err(e) => PrefabEntry { map: None, error: Some(e.to_string()), bounds: Aabb::EMPTY, modified },
            };
            self.entries.insert(path.to_path_buf(), entry);
            self.generation += 1;
            let bounds = self.compute_bounds(path, 0);
            if let Some(e) = self.entries.get_mut(path) {
                e.bounds = bounds;
            }
        }
        &self.entries[path]
    }

    fn compute_bounds(&mut self, path: &Path, depth: usize) -> Aabb {
        if depth > MAX_DEPTH {
            return Aabb::EMPTY;
        }
        let Some(map) = self.entries.get(path).and_then(|e| e.map.clone()) else { return Aabb::EMPTY };
        let mut b = Aabb::EMPTY;
        for (id, node) in map.nodes.iter() {
            match &node.kind {
                NodeKind::Brush(brush) => b.include(&brush.bounds()),
                NodeKind::Entity(e) if node.children.is_empty() => b.include(&Aabb::from_center_size(e.origin, gt_core::DVec3::splat(16.0))),
                NodeKind::Instance(i) => {
                    if let Some(p) = resolve(&i.path, Some(path), None) {
                        self.get(&p);
                        let inner = self.compute_bounds(&p, depth + 1);
                        b.include(&transformed_bounds(&inner, &instance_transform(i)));
                    }
                }
                _ => {}
            }
            let _ = id;
        }
        b
    }

    /// World bounds of an instance node, or a small box if the prefab cannot be loaded.
    pub fn instance_bounds(&mut self, i: &Instance, map_path: Option<&Path>, project_root: Option<&Path>) -> Aabb {
        let fallback = Aabb::from_center_size(i.origin, gt_core::DVec3::splat(16.0));
        let Some(path) = resolve(&i.path, map_path, project_root) else { return fallback };
        let b = self.get(&path).bounds;
        if b.is_empty() { fallback } else { transformed_bounds(&b, &instance_transform(i)) }
    }

    pub fn loaded(&self, path: &Path) -> Option<&PrefabEntry> {
        self.entries.get(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_core::DVec3;
    use gt_geom::Brush;

    #[test]
    fn bounds_follow_instance_transform() {
        let dir = std::env::temp_dir().join(format!("gt_prefab_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut prefab = Map::new();
        let layer = prefab.default_layer();
        prefab.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::new(64.0, 16.0, 16.0)), "m").unwrap()));
        format::save(&prefab, &dir.join("p.gtm")).unwrap();

        let mut cache = PrefabCache::default();
        let inst = Instance { path: "p.gtm".into(), origin: DVec3::new(100.0, 0.0, 0.0), angles: DVec3::new(0.0, 90.0, 0.0), fixup: String::new() };
        let b = cache.instance_bounds(&inst, Some(&dir.join("main.gtm")), None);
        assert!(gt_core::vec_approx_eq(b.min, DVec3::new(100.0, 0.0, -64.0)), "{b:?}");
        assert!(gt_core::vec_approx_eq(b.max, DVec3::new(116.0, 16.0, 0.0)), "{b:?}");
        let _ = std::fs::remove_dir_all(dir);
    }
}
