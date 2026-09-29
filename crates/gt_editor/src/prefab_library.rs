//! The prefab library: ready made pieces to paste into a map, like the prefab collections TrenchBroom users copy
//! from. Every entry is an ordinary .gtm. The built-in ones ship inside the editor and use the dev materials every
//! project has, the project's own live in res://prefab_library with one folder per category, and open like any
//! map to edit. Pasting an entry copies its objects, so the result is plain map content with no link back to it.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use gt_core::{Aabb, DVec3, NodeId};
use gt_doc::{Map, NodeKind, Selection, format, ops};

/// Folder in the Godot project that holds the project's own entries.
pub const FOLDER: &str = "prefab_library";

/// Category, name and file of every built-in entry.
const BUILTIN: &[(&str, &str, &[u8])] = &[
    ("Stairs", "Straight stair 8", include_bytes!("../assets/prefab_library/stairs/straight_8.gtm")),
    ("Stairs", "Straight stair 16", include_bytes!("../assets/prefab_library/stairs/straight_16.gtm")),
    ("Stairs", "Switchback stair", include_bytes!("../assets/prefab_library/stairs/switchback.gtm")),
    ("Doorways", "Doorway", include_bytes!("../assets/prefab_library/doorways/doorway.gtm")),
    ("Doorways", "Archway", include_bytes!("../assets/prefab_library/doorways/archway.gtm")),
    ("Doorways", "Window", include_bytes!("../assets/prefab_library/doorways/window.gtm")),
    ("Pillars", "Square pillar", include_bytes!("../assets/prefab_library/pillars/square_pillar.gtm")),
    ("Pillars", "Round pillar", include_bytes!("../assets/prefab_library/pillars/round_pillar.gtm")),
    ("Pillars", "Pillar with trim", include_bytes!("../assets/prefab_library/pillars/pillar_trim.gtm")),
    ("Rooms", "Small room", include_bytes!("../assets/prefab_library/rooms/small_room.gtm")),
    ("Rooms", "Corridor corner", include_bytes!("../assets/prefab_library/rooms/corridor_corner.gtm")),
    ("Details", "Railing", include_bytes!("../assets/prefab_library/details/railing.gtm")),
    ("Details", "Pipe run", include_bytes!("../assets/prefab_library/details/pipe_run.gtm")),
    ("Details", "Crate stack", include_bytes!("../assets/prefab_library/details/crate_stack.gtm")),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    BuiltIn,
    Project,
}

#[derive(Clone)]
pub struct Entry {
    pub name: String,
    pub category: String,
    pub source: Source,
    /// The file of a project entry.
    pub path: Option<PathBuf>,
    pub bounds: Aabb,
    pub preview: Arc<Preview>,
    builtin: Option<&'static [u8]>,
}

impl Entry {
    /// category/name, how scripts and the MCP tool address an entry.
    pub fn key(&self) -> String {
        format!("{}/{}", self.category, self.name)
    }
}

/// A flat drawing of an entry from the side that shows most of it: from above for floor plans, from the front or
/// the side for walls and railings. Every face turned to the viewer as an outline with its depth, nearer is larger,
/// and the spots of point entities. Map units, `min` and `max` bound the drawing.
#[derive(Debug, Default)]
pub struct Preview {
    pub faces: Vec<(Vec<[f32; 2]>, f32)>,
    pub points: Vec<[f32; 2]>,
    pub min: [f32; 2],
    pub max: [f32; 2],
    pub depth: [f32; 2],
}

#[derive(Default)]
pub struct PrefabLibrary {
    pub entries: Vec<Entry>,
    /// res://prefab_library of the open project.
    pub folder: Option<PathBuf>,
}

impl PrefabLibrary {
    pub fn new(project_root: Option<&Path>) -> Self {
        let mut library = Self::default();
        library.rescan(project_root);
        library
    }

    pub fn rescan(&mut self, project_root: Option<&Path>) {
        self.entries.clear();
        for (category, name, bytes) in BUILTIN {
            if let Ok(loaded) = format::from_bytes(bytes) {
                self.entries.push(entry(name, category, Source::BuiltIn, None, Some(bytes), &loaded.map));
            }
        }

        self.folder = project_root.map(|r| r.join(FOLDER));
        if let Some(folder) = self.folder.clone() {
            scan(&folder, &folder, &mut self.entries);
        }

        self.entries.sort_by(|a, b| (a.source == Source::BuiltIn, &a.category, &a.name).cmp(&(b.source == Source::BuiltIn, &b.category, &b.name)));
    }

    pub fn categories(&self) -> Vec<String> {
        let mut out: Vec<String> = self.entries.iter().map(|e| e.category.clone()).collect();
        out.sort();
        out.dedup();
        out
    }

    /// The entry named by `key`, category/name or just the name, ignoring case.
    pub fn find(&self, key: &str) -> Option<&Entry> {
        let key = key.trim().to_lowercase();
        self.entries.iter().find(|e| e.key().to_lowercase() == key).or_else(|| self.entries.iter().find(|e| e.name.to_lowercase() == key))
    }

    /// Clipboard text holding every object of the entry.
    pub fn text(&self, entry: &Entry) -> Result<String, String> {
        let loaded = match (entry.builtin, &entry.path) {
            (Some(bytes), _) => format::from_bytes(bytes),
            (None, Some(path)) => format::load(path),
            (None, None) => return Err("the entry has no file".into()),
        };
        loaded.map(|l| clipboard_of(&l.map)).map_err(|e| e.to_string())
    }

    /// Saves `roots` of `map` as a new project entry in `category` and rescans. It is moved so its bottom center sits
    /// on the origin, or stays where it is with `keep_position`, for pieces built around the map's origin on purpose.
    pub fn add(&mut self, map: &Map, roots: &[NodeId], category: &str, name: &str, project_root: &Path, keep_position: bool) -> Result<PathBuf, String> {
        if roots.is_empty() {
            return Err("nothing selected to add".into());
        }

        let (category, name) = (file_name(category), file_name(name));
        if name.is_empty() {
            return Err("give the entry a name".into());
        }

        let dir = project_root.join(FOLDER).join(if category.is_empty() { "misc" } else { &category });
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join(format!("{name}.gtm"));
        let bounds = map.bounds_of(roots.iter().copied());
        let pivot = if keep_position { DVec3::ZERO } else { DVec3::new(bounds.center().x, bounds.min.y, bounds.center().z) };
        format::save(&extract(map, roots, pivot), &path).map_err(|e| e.to_string())?;
        self.rescan(Some(project_root));
        Ok(path)
    }
}

/// Clipboard text holding every object in the map's exported layers.
pub fn clipboard_of(map: &Map) -> String {
    let layers: Vec<NodeId> =
        map.layers.iter().copied().filter(|l| !matches!(map.get(*l).map(|n| &n.kind), Some(NodeKind::Layer(layer)) if layer.omit_from_export)).collect();
    format::nodes_to_string(map, &layers)
}

/// `roots` of `map` as a map of their own, moved so `pivot` lands on the origin.
pub fn extract(map: &Map, roots: &[NodeId], pivot: DVec3) -> Map {
    let mut out = Map::new();
    out.properties.insert("classname".into(), "worldspawn".into());
    let layer = out.default_layer();
    let text = format::nodes_to_string(map, roots);
    if let Ok(ids) = format::paste_nodes(&mut out, layer, &text) {
        let mut sel = Selection::default();
        sel.nodes.extend(ids);
        ops::translate_selection(&mut out, &sel, -pivot, ops::EditOptions { uv_lock: true, grid: 0.0 });
    }

    out
}

fn file_name(text: &str) -> String {
    text.trim().chars().map(|c| if c.is_alphanumeric() || c == '-' { c.to_ascii_lowercase() } else { '_' }).collect::<String>().trim_matches('_').to_string()
}

fn display_name(stem: &str) -> String {
    let words = stem.replace(['_', '-'], " ");
    let mut chars = words.chars();
    chars.next().map(|c| c.to_uppercase().chain(chars).collect()).unwrap_or_default()
}

fn scan(root: &Path, dir: &Path, out: &mut Vec<Entry>) {
    let Ok(read) = std::fs::read_dir(dir) else { return };
    let mut paths: Vec<PathBuf> = read.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            scan(root, &path, out);
            continue;
        }

        if path.extension().is_none_or(|e| !e.eq_ignore_ascii_case("gtm")) {
            continue;
        }

        let Ok(loaded) = format::load(&path) else { continue };
        let category = path.parent().and_then(|p| p.strip_prefix(root).ok()).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_default();
        let name = path.file_stem().map(|s| display_name(&s.to_string_lossy())).unwrap_or_default();
        let category = if category.is_empty() { "Misc".to_string() } else { category.split('/').map(display_name).collect::<Vec<_>>().join("/") };
        out.push(entry(&name, &category, Source::Project, Some(path.clone()), None, &loaded.map));
    }
}

fn entry(name: &str, category: &str, source: Source, path: Option<PathBuf>, builtin: Option<&'static [u8]>, map: &Map) -> Entry {
    let bounds = map.bounds_of(map.layers.clone());
    let s = bounds.size();
    // Toward the viewer, then the drawing's across and down axes. A piece much thinner along one axis than the other
    // two is drawn looking along that axis, anything else from above.
    let (toward, across, down) = if s.z < 0.35 * s.x.max(s.y) {
        (DVec3::Z, DVec3::X, -DVec3::Y)
    } else if s.x < 0.35 * s.z.max(s.y) {
        (DVec3::X, -DVec3::Z, -DVec3::Y)
    } else {
        (DVec3::Y, DVec3::X, DVec3::Z)
    };
    let flat = |p: DVec3| [p.dot(across) as f32, p.dot(down) as f32];
    let mut preview = Preview::default();
    for id in map.walk() {
        match map.get(id).map(|n| &n.kind) {
            Some(NodeKind::Brush(b)) => {
                for (i, face) in b.faces.iter().enumerate() {
                    if face.plane.normal.dot(toward) < 0.3 {
                        continue;
                    }

                    let points = b.face_points(i);
                    let depth = points.iter().map(|p| p.dot(toward)).sum::<f64>() / points.len().max(1) as f64;
                    preview.faces.push((points.iter().map(|p| flat(*p)).collect(), depth as f32));
                }
            }
            Some(NodeKind::Entity(e)) if map.get(id).is_some_and(|n| n.children.is_empty()) => preview.points.push(flat(e.origin)),
            _ => {}
        }
    }

    // Farther faces first, so nearer ones draw over what lies behind them.
    preview.faces.sort_by(|a, b| a.1.total_cmp(&b.1));
    let corners: Vec<[f32; 2]> = bounds.corners().iter().map(|c| flat(*c)).collect();
    preview.min = [corners.iter().map(|c| c[0]).fold(f32::MAX, f32::min), corners.iter().map(|c| c[1]).fold(f32::MAX, f32::min)];
    preview.max = [corners.iter().map(|c| c[0]).fold(f32::MIN, f32::max), corners.iter().map(|c| c[1]).fold(f32::MIN, f32::max)];
    preview.depth = [bounds.min.dot(toward) as f32, bounds.max.dot(toward) as f32];
    Entry { name: name.to_string(), category: category.to_string(), source, path, bounds, preview: Arc::new(preview), builtin }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_geom::Brush;

    #[test]
    fn every_builtin_entry_loads_with_something_to_paste() {
        let library = PrefabLibrary::new(None);
        assert_eq!(library.entries.len(), BUILTIN.len());
        for e in &library.entries {
            let text = library.text(e).unwrap();
            let mut map = Map::new();
            let layer = map.default_layer();
            let ids = format::paste_nodes(&mut map, layer, &text).unwrap();
            assert!(!ids.is_empty(), "{} pastes nothing", e.key());
            assert!(!e.preview.faces.is_empty(), "{} has nothing to preview", e.key());
            assert!(e.bounds.min.y.abs() < 1e-6, "{} stands on the origin", e.key());
            for id in map.walk() {
                if let Some(NodeKind::Brush(b)) = map.get(id).map(|n| &n.kind) {
                    assert!(b.faces.iter().all(|f| f.data.material.starts_with("dev/")), "{} uses only dev materials", e.key());
                }
            }
        }
    }

    #[test]
    fn added_entries_land_in_their_category_centered_on_the_origin() {
        let root = std::env::temp_dir().join(format!("gt_prefab_library_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let mut map = Map::new();
        let layer = map.default_layer();
        let brush = map
            .insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::new(100.0, 32.0, 200.0), DVec3::new(164.0, 96.0, 232.0)), "dev/grey").unwrap()));
        let mut library = PrefabLibrary::new(Some(&root));
        let path = library.add(&map, &[brush], "Office Rooms", "Desk block", &root, false).unwrap();
        assert!(path.ends_with("prefab_library/office_rooms/desk_block.gtm"));
        let e = library.find("office rooms/desk block").expect("rescanned");
        assert_eq!((e.source, e.name.as_str(), e.category.as_str()), (Source::Project, "Desk block", "Office rooms"));
        assert_eq!(e.bounds, Aabb::new(DVec3::new(-32.0, 0.0, -16.0), DVec3::new(32.0, 64.0, 16.0)));
        assert!(library.find("desk block").is_some());
        assert!(library.add(&map, &[], "x", "y", &root, false).is_err());
        library.add(&map, &[brush], "Office Rooms", "Kept", &root, true).unwrap();
        assert_eq!(library.find("kept").unwrap().bounds, Aabb::new(DVec3::new(100.0, 32.0, 200.0), DVec3::new(164.0, 96.0, 232.0)), "keeps its position");
        std::fs::remove_dir_all(&root).unwrap();
    }
}
