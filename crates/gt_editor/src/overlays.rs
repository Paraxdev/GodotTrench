//! Godot overlays of the open map. The Godot addon writes `<map>.overlay.json` next to the map for every
//! `GodotTrenchOverlay` built on top of it: Godot-only props, decor and interactive scenes that survive map
//! rebuilds. The editor reads it to draw that content as read only ghost boxes and to know the targetnames overlay
//! nodes answer to, so outputs aimed at them are not reported as missing.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime};

use gt_core::{Aabb, DVec3};
use gt_render::LineVertex;
use serde::Deserialize;

pub const SIDECAR_FORMAT: &str = "godottrench-overlay";
/// Ghost box edges, a light Godot blue that stays apart from entity class colors.
pub const GHOST_COLOR: [f32; 4] = [0.45, 0.72, 1.0, 0.85];
const POLL: Duration = Duration::from_secs(1);
/// Items without size (lights, empty nodes) are drawn as a box this many map units across.
const POINT_SIZE: f64 = 8.0;
const DASH: f64 = 6.0;

#[derive(Clone, Debug, Default, Deserialize)]
struct SidecarFile {
    #[serde(default)]
    format: String,
    #[serde(default)]
    overlays: Vec<SidecarOverlay>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct SidecarOverlay {
    #[serde(default)]
    name: String,
    #[serde(default)]
    items: Vec<SidecarItem>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct SidecarItem {
    #[serde(default)]
    name: String,
    #[serde(default)]
    class: String,
    min: [f64; 3],
    max: [f64; 3],
    #[serde(default)]
    targetnames: Vec<String>,
    #[serde(default)]
    anchor: Option<String>,
}

/// One direct child of an overlay: its bounds in map units, what it is and the targetnames inside it.
#[derive(Clone, Debug, PartialEq)]
pub struct OverlayItem {
    pub overlay: String,
    pub name: String,
    pub class: String,
    pub bounds: Aabb,
    pub targetnames: Vec<String>,
    pub anchor: Option<String>,
}

impl OverlayItem {
    /// Bounds as drawn, points grown to a small box.
    pub fn drawn_bounds(&self) -> Aabb {
        let size = self.bounds.size();
        let pad = DVec3::new(
            if size.x < 1.0 { POINT_SIZE * 0.5 } else { 0.0 },
            if size.y < 1.0 { POINT_SIZE * 0.5 } else { 0.0 },
            if size.z < 1.0 { POINT_SIZE * 0.5 } else { 0.0 },
        );
        Aabb::new(self.bounds.min - pad, self.bounds.max + pad)
    }

    /// Label shown next to the ghost box.
    pub fn label(&self) -> String {
        match (self.targetnames.first(), &self.anchor) {
            (Some(t), _) => format!("{} ({t})", self.name),
            (None, Some(a)) => format!("{} on {a}", self.name),
            _ => self.name.clone(),
        }
    }
}

/// `night_district.gtm` has its overlays in `night_district.overlay.json`.
pub fn sidecar_path(map_path: &Path) -> PathBuf {
    map_path.with_extension("overlay.json")
}

/// Items of every overlay in a sidecar file. Anything that is not a sidecar gives an error.
pub fn parse(text: &str) -> Result<Vec<OverlayItem>, String> {
    let file: SidecarFile = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if file.format != SIDECAR_FORMAT {
        return Err(format!("not a {SIDECAR_FORMAT} file"));
    }

    Ok(file
        .overlays
        .into_iter()
        .flat_map(|o| {
            let overlay = o.name;
            o.items.into_iter().map(move |i| OverlayItem {
                overlay: overlay.clone(),
                name: i.name,
                class: i.class,
                bounds: Aabb::from_points([DVec3::from_array(i.min), DVec3::from_array(i.max)]),
                targetnames: i.targetnames,
                anchor: i.anchor.filter(|a| !a.is_empty()),
            })
        })
        .collect())
}

/// Dashed box edges for every item, so ghosts read apart from the solid outlines of map content.
pub fn ghost_lines(items: &[OverlayItem]) -> Vec<LineVertex> {
    let mut out = Vec::new();
    for item in items {
        let corners = item.drawn_bounds().corners();
        for (a, b) in Aabb::EDGES {
            dashed(&mut out, corners[a], corners[b], GHOST_COLOR);
        }
    }

    out
}

fn dashed(out: &mut Vec<LineVertex>, a: DVec3, b: DVec3, color: [f32; 4]) {
    let length = a.distance(b);
    let steps = ((length / (DASH * 2.0)).floor() as usize).max(1);
    let step = length / steps as f64;
    let dir = (b - a) / length.max(1e-9);
    for i in 0..steps {
        let start = a + dir * (step * i as f64);
        let end = start + dir * (step * 0.6);
        out.push(LineVertex { pos: crate::scene::v3(start), color });
        out.push(LineVertex { pos: crate::scene::v3(end), color });
    }
}

/// The sidecar of the open map, reloaded when the map changes or the file is rewritten.
#[derive(Default)]
pub struct OverlayGhosts {
    path: Option<PathBuf>,
    modified: Option<SystemTime>,
    checked: Option<Instant>,
    pub items: Vec<OverlayItem>,
    /// Goes up whenever `items` change.
    pub generation: u64,
}

impl OverlayGhosts {
    /// Rereads the sidecar of `map_path` when it is a different map, or at most once a second when its file changed.
    pub fn refresh(&mut self, map_path: Option<&Path>) {
        let path = map_path.map(sidecar_path);
        let switched = path != self.path;
        if !switched && self.checked.is_some_and(|t| t.elapsed() < POLL) {
            return;
        }

        self.checked = Some(Instant::now());
        let modified = path.as_ref().and_then(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
        if !switched && modified == self.modified {
            return;
        }

        self.path = path;
        self.modified = modified;
        let items = match (&self.path, modified) {
            (Some(p), Some(_)) => std::fs::read_to_string(p).ok().and_then(|t| parse(&t).ok()).unwrap_or_default(),
            _ => Vec::new(),
        };
        if items != self.items || switched {
            self.items = items;
            self.generation += 1;
        }
    }

    /// Targetnames of overlay nodes, targets that exist in Godot but not in the map.
    pub fn targetnames(&self) -> BTreeSet<String> {
        self.items.iter().flat_map(|i| i.targetnames.iter().cloned()).collect()
    }

    pub fn has_target(&self, name: &str) -> bool {
        self.items.iter().any(|i| i.targetnames.iter().any(|t| t == name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
      "format": "godottrench-overlay", "version": 1, "map": "night_district.gtm", "units_per_meter": 32.0,
      "overlays": [{ "source": "res://demo/overlays/night_district_overlay.tscn", "name": "NightDistrictOverlay", "items": [
        { "name": "Breaker", "class": "StaticBody3D", "min": [-592.0, 8.0, -648.3], "max": [-577.0, 44.0, -631.7], "targetnames": ["court_breaker"] },
        { "name": "CourtSignGlow", "class": "OmniLight3D", "min": [-704.0, 137.6, -1017.6], "max": [-704.0, 137.6, -1017.6] },
        { "name": "GarageSign", "class": "Node3D", "min": [150.4, 91.2, -276.8], "max": [233.6, 113.6, -275.8], "anchor": "garage_door" }
      ] }]
    }"#;

    #[test]
    fn sidecar_sits_next_to_the_map() {
        assert_eq!(sidecar_path(Path::new("maps/night_district.gtm")), PathBuf::from("maps/night_district.overlay.json"));
    }

    #[test]
    fn parses_items_with_bounds_targetnames_and_anchors() {
        let items = parse(SAMPLE).unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].overlay, "NightDistrictOverlay");
        assert_eq!(items[0].bounds.min, DVec3::new(-592.0, 8.0, -648.3));
        assert_eq!(items[0].label(), "Breaker (court_breaker)");
        assert_eq!(items[2].anchor.as_deref(), Some("garage_door"));
        assert_eq!(items[2].label(), "GarageSign on garage_door");
        assert!(parse(r#"{"format": "godottrench-map", "layers": []}"#).is_err(), "a map is not a sidecar");
    }

    #[test]
    fn points_draw_as_small_boxes_with_dashed_edges() {
        let items = parse(SAMPLE).unwrap();
        let light = items[1].drawn_bounds();
        assert_eq!(light.size(), DVec3::splat(POINT_SIZE));
        let lines = ghost_lines(&items[1..2]);
        assert_eq!(lines.len(), 12 * 2, "one dash per short edge");
        assert!(lines.iter().all(|l| l.color == GHOST_COLOR));
        let long = ghost_lines(&items[0..1]);
        assert!(long.len() > 12 * 2, "long edges are split into dashes");
    }

    #[test]
    fn reloads_when_the_file_changes() {
        let dir = std::env::temp_dir().join(format!("gt_overlay_ghosts_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let map = dir.join("street.gtm");
        let sidecar = sidecar_path(&map);
        let _ = std::fs::remove_file(&sidecar);
        let mut ghosts = OverlayGhosts::default();
        ghosts.refresh(Some(&map));
        assert!(ghosts.items.is_empty());
        let first = ghosts.generation;

        std::fs::write(&sidecar, SAMPLE).unwrap();
        ghosts.checked = None;
        ghosts.refresh(Some(&map));
        assert_eq!(ghosts.items.len(), 3);
        assert!(ghosts.generation > first);
        assert!(ghosts.has_target("court_breaker") && !ghosts.has_target("garage_door"));
        assert_eq!(ghosts.targetnames().into_iter().collect::<Vec<_>>(), vec!["court_breaker".to_string()]);

        ghosts.refresh(None);
        assert!(ghosts.items.is_empty(), "closing the map drops its ghosts");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
