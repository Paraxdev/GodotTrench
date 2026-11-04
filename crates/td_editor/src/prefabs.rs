//! Loaded prefab maps referenced by instance nodes, reloaded when their file changes.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;

use td_core::{Aabb, DMat4, NodeId};
use td_doc::map::Instance;
use td_doc::{Map, Node, NodeKind, format};

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
    /// Godot project root that res:// instance paths inside prefabs resolve against.
    pub project_root: Option<PathBuf>,
    /// What loading damaged prefab files lost, until the editor shows it.
    pub problems: Vec<String>,
    extents_seen: Option<ExtentsSeen>,
    /// Prefabs a running run_script may load, see [`Self::set_fence`].
    fence: Option<crate::mcp::script::Fence>,
    /// What [`Self::get`] hands out for a prefab outside the fence.
    refused: Option<PrefabEntry>,
}

/// What [`PrefabCache::refresh_extents`] last filled in, so it only walks the map again when that could change.
struct ExtentsSeen {
    nodes: imbl::OrdMap<NodeId, Node>,
    paths: BTreeSet<String>,
    generation: u64,
    map_path: Option<PathBuf>,
    extents: Arc<BTreeMap<String, Aabb>>,
}

pub fn instance_transform(i: &Instance) -> DMat4 {
    i.transform()
}

/// Resolves an instance path relative to the referencing map, or through res:// in the Godot project. An absolute path
/// may name any local file, but never a network share.
pub fn resolve(path: &str, map_path: Option<&Path>, project_root: Option<&Path>) -> Option<PathBuf> {
    if path.starts_with("res://") {
        return td_formats::game::project_file(project_root?, path);
    }

    if td_formats::game::is_network_path(path) {
        return None;
    }

    let p = Path::new(path);
    if p.is_absolute() {
        return Some(p.to_path_buf());
    }

    map_path.and_then(|m| m.parent()).map(|dir| dir.join(p))
}

/// The nodes of a prefab that Godot builds: everything under a layer that is not omitted from export.
/// Top level nodes outside any layer are ignored, like the addon does.
pub fn exported_nodes(map: &Map) -> Vec<(NodeId, &Node)> {
    let kept: BTreeSet<NodeId> =
        map.layers.iter().copied().filter(|l| matches!(map.get(*l).map(|n| &n.kind), Some(NodeKind::Layer(layer)) if !layer.omit_from_export)).collect();
    map.nodes.iter().filter(|(id, _)| kept.contains(&map.layer_of(**id))).map(|(id, n)| (*id, n)).collect()
}

fn transformed_bounds(b: &Aabb, m: &DMat4) -> Aabb {
    if b.is_empty() {
        return *b;
    }

    Aabb::from_points(b.corners().iter().map(|c| m.transform_point3(*c)))
}

impl PrefabCache {
    /// Whether `map`, saved at `map_path`, instances the prefab at `target`, directly or through other prefabs.
    pub fn instances(&mut self, map: &Map, map_path: Option<&Path>, target: &Path) -> bool {
        self.instances_below(map, map_path, &crate::live_link::path_key(&crate::live_link::godot_path(target)), 0)
    }

    fn instances_below(&mut self, map: &Map, map_path: Option<&Path>, target: &str, depth: usize) -> bool {
        let root = self.project_root.clone();
        let paths: BTreeSet<PathBuf> = map
            .nodes
            .values()
            .filter_map(|n| match &n.kind {
                NodeKind::Instance(i) => resolve(&i.path, map_path, root.as_deref()),
                _ => None,
            })
            .collect();
        if paths.iter().any(|p| crate::live_link::path_key(&crate::live_link::godot_path(p)) == target) {
            return true;
        }

        depth < MAX_DEPTH
            && paths.iter().any(|p| match self.get(p).map.clone() {
                Some(prefab) => self.instances_below(&prefab, Some(p), target, depth + 1),
                None => false,
            })
    }

    /// Refreshes the entry for `path` if the file changed, returns it. A prefab outside the fence is not loaded, nor
    /// handed out when it was loaded before.
    pub fn get(&mut self, path: &Path) -> &PrefabEntry {
        if self.fence.as_ref().is_some_and(|f| !f.may_read(path)) {
            let error = format!("{} is outside the folders the running script may read", path.display());
            return self.refused.insert(PrefabEntry { map: None, error: Some(error), bounds: Aabb::EMPTY, modified: None });
        }

        let modified = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        let stale = self.entries.get(path).is_none_or(|e| e.modified != modified);
        if stale {
            let entry = match format::load(path) {
                Ok(format::Loaded { map, problems, .. }) => {
                    for p in problems {
                        eprintln!("prefab {}: {p}", path.display());
                        self.problems.push(format!("Prefab {} is damaged: {p}", path.display()));
                    }

                    PrefabEntry { bounds: Aabb::EMPTY, map: Some(map), error: None, modified }
                }
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
        let root = self.project_root.clone();
        let mut b = Aabb::EMPTY;
        for (_, node) in exported_nodes(&map) {
            match &node.kind {
                NodeKind::Brush(brush) => b.include(&brush.bounds()),
                NodeKind::Mesh(m) => b.include(&m.bounds()),
                NodeKind::Terrain(t) => b.include(&t.bounds()),
                NodeKind::Scatter(s) if !s.instances.is_empty() => b.include(&s.bounds()),
                NodeKind::Entity(e) if node.children.is_empty() => b.include(&Aabb::from_center_size(e.origin, td_core::DVec3::splat(16.0))),
                NodeKind::Instance(i) => {
                    if let Some(p) = resolve(&i.path, Some(path), root.as_deref()) {
                        self.get(&p);
                        let inner = self.compute_bounds(&p, depth + 1);
                        b.include(&transformed_bounds(&inner, &instance_transform(i)));
                    }
                }
                _ => {}
            }
        }

        b
    }

    /// World bounds of an instance node, or a small box if the prefab cannot be loaded.
    pub fn instance_bounds(&mut self, i: &Instance, map_path: Option<&Path>, project_root: Option<&Path>) -> Aabb {
        let fallback = Aabb::from_center_size(i.origin, td_core::DVec3::splat(16.0));
        let Some(path) = resolve(&i.path, map_path, project_root) else { return fallback };
        let b = self.get(&path).bounds;
        if b.is_empty() { fallback } else { transformed_bounds(&b, &instance_transform(i)) }
    }

    /// Fills `map.instance_extents` with the bounds of every prefab the map instances, so bounds queries on an
    /// instance (the Inspector, framing, selection boxes) cover its content rather than a placeholder box.
    pub fn refresh_extents(&mut self, map: &mut Map, map_path: Option<&Path>) {
        let generation = self.generation;
        let current =
            |seen: &ExtentsSeen| seen.generation == generation && seen.map_path.as_deref() == map_path && Arc::ptr_eq(&seen.extents, &map.instance_extents);
        if self.extents_seen.as_ref().is_some_and(|seen| current(seen) && seen.nodes.ptr_eq(&map.nodes)) {
            return;
        }

        // Extents only depend on which prefab files are instanced. Collecting those is cheap, whereas diffing the nodes
        // would compare every changed node in full, a sculpted terrain's heights included.
        let paths: BTreeSet<String> = map
            .nodes
            .values()
            .filter_map(|n| match &n.kind {
                NodeKind::Instance(i) => Some(i.path.clone()),
                _ => None,
            })
            .collect();
        if let Some(seen) = &mut self.extents_seen
            && current(seen)
            && seen.paths == paths
        {
            seen.nodes = map.nodes.clone();
            return;
        }

        let root = self.project_root.clone();
        let mut extents = BTreeMap::new();
        for path in &paths {
            if let Some(file) = resolve(path, map_path, root.as_deref()) {
                extents.insert(path.clone(), self.get(&file).bounds);
            }
        }

        if *map.instance_extents != extents {
            map.instance_extents = Arc::new(extents);
        }

        self.extents_seen = Some(ExtentsSeen {
            nodes: map.nodes.clone(),
            paths,
            generation: self.generation,
            map_path: map_path.map(Path::to_path_buf),
            extents: map.instance_extents.clone(),
        });
    }

    pub fn loaded(&self, path: &Path) -> Option<&PrefabEntry> {
        self.entries.get(path)
    }

    /// Only hands out prefabs `fence` allows while a run_script runs, None allows all again. Prefabs loaded so far are
    /// read again, since their bounds take in nested prefabs that only one of the two allows.
    pub fn set_fence(&mut self, fence: Option<crate::mcp::script::Fence>) {
        self.fence = fence;
        self.entries.clear();
        self.generation += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use td_core::DVec3;
    use td_geom::Brush;

    #[test]
    fn bounds_follow_instance_transform() {
        let dir = std::env::temp_dir().join(format!("td_prefab_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut prefab = Map::new();
        let layer = prefab.default_layer();
        prefab.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::new(64.0, 16.0, 16.0)), "m").unwrap()));
        format::save(&prefab, &dir.join("p.tdm")).unwrap();

        let mut cache = PrefabCache::default();
        let inst = Instance { path: "p.tdm".into(), origin: DVec3::new(100.0, 0.0, 0.0), angles: DVec3::new(0.0, 90.0, 0.0), fixup: String::new() };
        let b = cache.instance_bounds(&inst, Some(&dir.join("main.tdm")), None);
        assert!(td_core::vec_approx_eq(b.min, DVec3::new(100.0, 0.0, -64.0)), "{b:?}");
        assert!(td_core::vec_approx_eq(b.max, DVec3::new(116.0, 16.0, 0.0)), "{b:?}");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn map_bounds_of_an_instance_cover_the_prefab() {
        let dir = std::env::temp_dir().join(format!("td_prefab_extents_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut prefab = Map::new();
        let layer = prefab.default_layer();
        prefab.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::new(-144.0, 0.0, -128.0), DVec3::new(144.0, 64.0, 128.0)), "m").unwrap()));
        format::save(&prefab, &dir.join("p.tdm")).unwrap();

        let mut map = Map::new();
        let layer = map.default_layer();
        let inst = Instance { path: "p.tdm".into(), origin: DVec3::new(0.0, 32.0, 0.0), angles: DVec3::ZERO, fixup: String::new() };
        let id = map.insert(layer, NodeKind::Instance(inst.clone()));
        assert_eq!(map.bounds(id).size(), DVec3::splat(16.0), "unknown prefab content falls back to a small box");

        let mut cache = PrefabCache::default();
        let main = dir.join("main.tdm");
        cache.refresh_extents(&mut map, Some(&main));
        assert_eq!(map.bounds(id), Aabb::new(DVec3::new(-144.0, 32.0, -128.0), DVec3::new(144.0, 96.0, 128.0)));

        // An undo brings back a map whose extents predate the instance, the next refresh fills them in again.
        let mut older = map.clone();
        older.instance_extents = Default::default();
        cache.refresh_extents(&mut older, Some(&main));
        assert_eq!(older.bounds(id).size(), DVec3::new(288.0, 64.0, 256.0));

        let extents = map.instance_extents.clone();
        map.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(16.0)), "m").unwrap()));
        cache.refresh_extents(&mut map, Some(&main));
        assert!(Arc::ptr_eq(&extents, &map.instance_extents), "other edits keep the extents");
        let mut small = Map::new();
        let small_layer = small.default_layer();
        small.insert(small_layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(8.0)), "m").unwrap()));
        format::save(&small, &dir.join("q.tdm")).unwrap();
        let other = map.insert(layer, NodeKind::Instance(Instance { path: "q.tdm".into(), ..inst }));
        cache.refresh_extents(&mut map, Some(&main));
        assert_eq!(map.bounds(other).size(), DVec3::splat(8.0), "a newly instanced prefab gets its extents");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_fence_keeps_prefabs_outside_it_unread() {
        let dir = std::env::temp_dir().join(format!("td_prefab_fence_{}", std::process::id()));
        let (inside, outside) = (dir.join("pack"), dir.join("elsewhere"));
        std::fs::create_dir_all(&inside).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        let mut prefab = Map::new();
        let layer = prefab.default_layer();
        prefab.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(64.0)), "m").unwrap()));
        format::save(&prefab, &inside.join("door.tdm")).unwrap();
        format::save(&prefab, &outside.join("secret.tdm")).unwrap();

        let mut cache = PrefabCache::default();
        assert!(cache.get(&outside.join("secret.tdm")).map.is_some());
        cache.set_fence(Some(crate::mcp::script::Fence::new(vec![inside.clone()])));
        assert!(cache.get(&inside.join("door.tdm")).map.is_some());
        let refused = cache.get(&outside.join("secret.tdm"));
        assert!(refused.map.is_none() && refused.error.as_deref().is_some_and(|e| e.contains("outside")), "also once it was loaded before");
        let inst = Instance { path: outside.join("secret.tdm").to_string_lossy().into_owned(), origin: DVec3::ZERO, angles: DVec3::ZERO, fixup: String::new() };
        assert_eq!(cache.instance_bounds(&inst, Some(&inside.join("main.tdm")), None).size(), DVec3::splat(16.0), "only the placeholder box");
        cache.set_fence(None);
        assert!(cache.get(&outside.join("secret.tdm")).map.is_some());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn instance_paths_never_lead_to_a_network_share() {
        let (map, root) = (Path::new("/game/maps/main.tdm"), Path::new("/game"));
        let resolve = |path: &str| resolve(path, Some(map), Some(root));
        assert_eq!(resolve("door.tdm"), Some(Path::new("/game/maps").join("door.tdm")));
        assert_eq!(resolve("res://prefabs/door.tdm"), Some(root.join("prefabs").join("door.tdm")));
        assert!(resolve("../shared/door.tdm").is_some(), "a prefab may sit next to the map's folder");
        assert!(resolve("/elsewhere/door.tdm").is_some(), "or anywhere on a local drive");
        for remote in ["//host/share/door.tdm", r"\\host\share\door.tdm", r"\\?\UNC\host\share\door.tdm", "res:////host/share/door.tdm", "res://../door.tdm"] {
            assert_eq!(resolve(remote), None, "{remote}");
        }
    }

    #[test]
    fn layers_omitted_from_export_are_not_part_of_the_prefab() {
        let mut prefab = Map::new();
        let layer = prefab.default_layer();
        let kept = prefab.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::splat(16.0)), "m").unwrap()));
        let notes = prefab.add_layer("notes");
        if let Some(NodeKind::Layer(l)) = prefab.get_mut(notes).map(|n| &mut n.kind) {
            l.omit_from_export = true;
        }

        let omitted = prefab.insert(notes, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::splat(512.0), DVec3::splat(528.0)), "m").unwrap()));
        let ids: Vec<NodeId> = exported_nodes(&prefab).into_iter().map(|(id, _)| id).collect();
        assert!(ids.contains(&kept));
        assert!(!ids.contains(&omitted) && !ids.contains(&notes), "Godot and Explode skip the omitted layer");
    }
}
