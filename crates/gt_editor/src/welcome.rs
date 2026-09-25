//! The first steps an empty map shows in its views, and the maps of the open Godot project.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use egui::{Align2, FontId, Rect, RichText, Ui, Vec2};
use gt_doc::Map;

use crate::camera::ViewKind;
use crate::commands::Action;
use crate::state::EditorState;
use crate::tools::ToolKind;

pub const GETTING_STARTED_URL: &str = "https://paraxdev.github.io/GodotTrench/getting-started.html";

pub const CAMERA_3D_HELP: &str =
    "Hold the right mouse button and move the mouse to look around, WASD flies while it is held. Middle drag pans, the wheel moves forward and back.";
pub const CAMERA_2D_HELP: &str = "Right or middle drag pans, the wheel zooms. Drag on empty space to draw a box brush.";

const RESCAN: Duration = Duration::from_secs(3);
const MAX_MAPS: usize = 200;

pub fn map_is_empty(map: &Map) -> bool {
    map.layers.iter().all(|l| map.get(*l).is_none_or(|n| n.children.is_empty()))
}

/// The `.gtm` files of the open Godot project. They are scanned on a thread when the project changes and again now and
/// then while something shows them, so a map saved meanwhile turns up without the UI waiting on the disk.
#[derive(Default)]
pub struct ProjectMaps {
    root: Option<PathBuf>,
    scanned: Option<Instant>,
    maps: Vec<PathBuf>,
    pending: Option<mpsc::Receiver<Vec<PathBuf>>>,
}

impl ProjectMaps {
    pub fn get(&mut self, ctx: &egui::Context, root: Option<&Path>) -> &[PathBuf] {
        if self.root.as_deref() != root {
            *self = Self { root: root.map(Path::to_path_buf), ..Default::default() };
        }

        if let Some(rx) = &self.pending {
            match rx.try_recv() {
                Ok(maps) => {
                    self.maps = maps;
                    self.pending = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => self.pending = None,
                Err(mpsc::TryRecvError::Empty) => {}
            }
        }

        if let Some(root) = self.root.clone()
            && self.pending.is_none()
            && self.scanned.is_none_or(|t| t.elapsed() > RESCAN)
        {
            self.scanned = Some(Instant::now());
            let (tx, rx) = mpsc::channel();
            let ctx = ctx.clone();
            std::thread::spawn(move || {
                if tx.send(find_maps(&root)).is_ok() {
                    ctx.request_repaint();
                }
            });
            self.pending = Some(rx);
        }

        &self.maps
    }
}

/// The first maps under `root` by path, leaving out the addons' and Godot's own folders.
pub fn find_maps(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                if depth < 8 && !name.starts_with('.') && name != "addons" {
                    walk(&path, depth + 1, out);
                }
            } else if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("gtm")) {
                out.push(path);
            }
        }
    }

    let mut out = Vec::new();
    walk(root, 0, &mut out);
    out.sort_by(|a, b| a.parent().cmp(&b.parent()).then(a.cmp(b)));
    out.truncate(MAX_MAPS);
    out
}

/// Menu entries for `maps`, under a heading per folder.
pub fn maps_menu(ui: &mut Ui, root: Option<&Path>, maps: &[PathBuf], actions: &mut Vec<Action>) {
    if root.is_none() {
        ui.label(RichText::new("Open a Godot project to list its maps").weak().italics());
        return;
    }

    if maps.is_empty() {
        ui.label(RichText::new("No maps in this project yet").weak().italics());
        return;
    }

    let mut folder: Option<&Path> = None;
    for path in maps {
        let parent = path.parent();
        if parent != folder {
            folder = parent;
            let shown = parent.zip(root).and_then(|(p, r)| p.strip_prefix(r).ok()).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_default();
            ui.label(RichText::new(if shown.is_empty() { "res://".to_string() } else { format!("res://{shown}") }).weak());
        }

        let name = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        if ui.button(name).on_hover_text(path.display().to_string()).clicked() {
            actions.push(Action::OpenMapFile(path.clone()));
            ui.close();
        }
    }
}

/// Tells a newcomer where to start while the map is empty: how to draw in the 2D views, and in the 3D view how to
/// look around, where the guide is and which maps of the project show a finished level.
pub fn view_hint(ui: &mut Ui, rect: Rect, kind: ViewKind, state: &mut EditorState, maps: &mut ProjectMaps, actions: &mut Vec<Action>) {
    if !state.prefs.start_hints || !map_is_empty(&state.doc.map) {
        return;
    }

    if kind.is_2d() {
        if state.tool == ToolKind::Select && rect.width() > 160.0 && rect.height() > 80.0 {
            let painter = ui.painter_at(rect);
            let galley = painter.layout_no_wrap("Drag here to draw a box".into(), FontId::proportional(15.0), ui.visuals().text_color());
            let back = Rect::from_center_size(rect.center(), galley.size() + Vec2::new(20.0, 12.0));
            painter.rect_filled(back, 6.0, ui.visuals().extreme_bg_color.gamma_multiply(0.85));
            painter.galley(back.center() - galley.size() / 2.0, galley, ui.visuals().text_color());
        }

        return;
    }

    if rect.width() < 280.0 || rect.height() < 200.0 {
        return;
    }

    let width = (rect.width() - 32.0).min(440.0);
    let id = ui.id().with("start_hint");
    let size = ui.data(|d| d.get_temp::<Vec2>(id)).unwrap_or(Vec2::new(width, 180.0));
    let card = Align2::CENTER_CENTER.align_size_within_rect(Vec2::new(width, size.y), rect);
    let root = state.game.project_root.clone();
    let shown = ui.scope_builder(egui::UiBuilder::new().max_rect(card), |ui| {
        egui::Frame::popup(ui.style()).show(ui, |ui| {
            // Selectable labels would take the clicks, drags and wheel meant for the camera behind the card.
            ui.style_mut().interaction.selectable_labels = false;
            ui.set_width(ui.available_width());
            ui.label(RichText::new("This map is empty").strong().size(16.0));
            let hollow = match crate::commands::shortcut_text(ui.ctx(), &state.prefs, &Action::CsgHollow) {
                Some(keys) => format!("Brush > CSG > Hollow ({keys})"),
                None => "Brush > CSG > Hollow".into(),
            };
            ui.label(format!(
                "Levels are built from brushes, solid blocks you draw and then shape. Drag in the Top, Front or Side view to draw your \
                 first one, then hollow it into a room with {hollow}."
            ));
            ui.label(RichText::new(CAMERA_3D_HELP).weak());
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                let maps = maps.get(ui.ctx(), root.as_deref());
                if !maps.is_empty() {
                    ui.menu_button("Open a map from this project", |ui| maps_menu(ui, root.as_deref(), maps, actions))
                        .response
                        .on_hover_text("See how a finished map is put together");
                }

                ui.hyperlink_to("Getting started guide", GETTING_STARTED_URL);
                if ui.button("Hide tips").on_hover_text("File > Preferences brings them back").clicked() {
                    state.prefs.start_hints = false;
                }
            });
        })
    });
    ui.data_mut(|d| d.insert_temp(id, shown.response.rect.size()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_project_maps_but_not_the_addons_or_godot_folders() {
        let dir = std::env::temp_dir().join(format!("gt_welcome_maps_{}", std::process::id()));
        for sub in ["maps/showcase", "addons/func_godot", ".godot"] {
            std::fs::create_dir_all(dir.join(sub)).unwrap();
        }

        for file in ["maps/b.gtm", "maps/z.gtm", "maps/showcase/a.gtm", "addons/func_godot/test.gtm", ".godot/cache.gtm", "maps/notes.txt"] {
            std::fs::write(dir.join(file), "").unwrap();
        }

        let found: Vec<String> = find_maps(&dir).iter().map(|p| p.strip_prefix(&dir).unwrap().to_string_lossy().replace('\\', "/")).collect();
        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(found, ["maps/b.gtm", "maps/z.gtm", "maps/showcase/a.gtm"], "a folder's maps stay together");
    }

    #[test]
    fn a_long_map_list_keeps_the_first_maps_by_path() {
        let dir = std::env::temp_dir().join(format!("gt_welcome_many_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("a")).unwrap();
        std::fs::create_dir_all(dir.join("b")).unwrap();
        std::fs::write(dir.join("a/first.gtm"), "").unwrap();
        for i in 0..MAX_MAPS + 20 {
            std::fs::write(dir.join(format!("b/{i:03}.gtm")), "").unwrap();
        }

        let found = find_maps(&dir);
        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(found.len(), MAX_MAPS);
        assert_eq!(found[0], dir.join("a/first.gtm"));
        assert_eq!(found[MAX_MAPS - 1], dir.join(format!("b/{:03}.gtm", MAX_MAPS - 2)));
    }

    #[test]
    fn the_3d_view_offers_the_project_maps_until_the_tips_are_hidden() {
        use egui_kittest::Harness;
        use egui_kittest::kittest::Queryable;

        struct Fixture {
            state: EditorState,
            maps: ProjectMaps,
            actions: Vec<Action>,
            view_hovered: bool,
        }

        let dir = std::env::temp_dir().join(format!("gt_welcome_hint_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("maps")).unwrap();
        std::fs::write(dir.join("maps/church.gtm"), "").unwrap();
        let mut state = EditorState::new(Default::default());
        state.game.project_root = Some(dir.clone());
        let fixture = Fixture { state, maps: ProjectMaps::default(), actions: Vec::new(), view_hovered: false };
        let mut harness = Harness::builder().with_size(egui::vec2(800.0, 600.0)).build_ui_state(
            |ui, f: &mut Fixture| {
                let rect = ui.max_rect();
                f.view_hovered = ui.allocate_rect(rect, egui::Sense::click_and_drag()).hovered();
                view_hint(ui, rect, ViewKind::Perspective, &mut f.state, &mut f.maps, &mut f.actions);
            },
            fixture,
        );
        harness.run();
        assert!(harness.query_by_label_contains("with Brush > CSG > Hollow (").is_some(), "names the menu and the shortcut");
        let title = harness.get_by_label("This map is empty").rect().center();
        harness.hover_at(title);
        harness.run();
        assert!(harness.state().view_hovered, "the card's text lets the camera behind it take the mouse");

        // The scan runs on a thread, the menu shows up once it is done.
        for _ in 0..500 {
            if harness.query_by_label("Open a map from this project").is_some() {
                break;
            }

            std::thread::sleep(Duration::from_millis(10));
            harness.step();
        }

        harness.get_by_label("Open a map from this project").click();
        harness.run();
        assert!(harness.query_by_label("res://maps").is_some());
        harness.get_by_label("church").click();
        harness.run();
        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(harness.state().actions, vec![Action::OpenMapFile(dir.join("maps/church.gtm"))]);

        harness.get_by_label("Hide tips").click();
        harness.run();
        assert!(!harness.state().state.prefs.start_hints);
        assert!(harness.query_by_label("This map is empty").is_none());
    }

    #[test]
    fn a_map_with_only_its_layers_is_empty() {
        let mut map = Map::new();
        assert!(map_is_empty(&map));
        let layer = map.default_layer();
        let brush = gt_geom::Brush::from_aabb(&gt_core::Aabb::new(gt_core::DVec3::ZERO, gt_core::DVec3::splat(64.0)), "dev/grey").unwrap();
        map.insert(layer, gt_doc::NodeKind::Brush(brush));
        assert!(!map_is_empty(&map));
    }
}
