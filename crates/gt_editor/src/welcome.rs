//! The first steps an empty map shows in its views, and the maps of the open Godot project.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use egui::{Align2, FontId, Rect, RichText, Stroke, StrokeKind, Ui, Vec2};
use gt_doc::Map;

use crate::camera::ViewKind;
use crate::commands::Action;
use crate::state::EditorState;
use crate::theme;
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
    /// Scans again the next time the maps are shown, for maps that were just added.
    pub fn rescan(&mut self) {
        self.scanned = None;
    }

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

/// Tells a newcomer where to start while the map is empty. A card in the 3D view names the first step and the help
/// around it, and the 2D views say where to drag. `start_view` is the 2D view the card sends people to, the Top view
/// unless it is closed, and it says so most clearly.
pub fn view_hint(
    ui: &mut Ui,
    rect: Rect,
    kind: ViewKind,
    start_view: Option<ViewKind>,
    state: &mut EditorState,
    maps: &mut ProjectMaps,
    actions: &mut Vec<Action>,
) {
    if !state.prefs.start_hints || !map_is_empty(&state.doc.map) {
        return;
    }

    if kind.is_2d() {
        if state.tool == ToolKind::Select && rect.width() > 160.0 && rect.height() > 80.0 {
            let start = start_view == Some(kind);
            let (text, color) =
                if start { ("Drag here to draw your first box", theme::YELLOW) } else { ("Drag here to draw a box", ui.visuals().text_color()) };
            let painter = ui.painter_at(rect);
            let galley = painter.layout_no_wrap(text.into(), FontId::proportional(15.0), color);
            let back = Rect::from_center_size(rect.center(), galley.size() + Vec2::new(20.0, 12.0));
            painter.rect_filled(back, 6.0, ui.visuals().extreme_bg_color.gamma_multiply(0.85));
            if start {
                painter.rect_stroke(back, 6.0, Stroke::new(1.5, theme::YELLOW), StrokeKind::Inside);
            }

            painter.galley(back.center() - galley.size() / 2.0, galley, color);
        }

        return;
    }

    if rect.width() < 280.0 || rect.height() < 200.0 {
        return;
    }

    let width = (rect.width() - 32.0).min(420.0);
    let id = ui.id().with("start_hint");
    let size = ui.data(|d| d.get_temp::<Vec2>(id)).unwrap_or(Vec2::new(width, 180.0));
    let card = Align2::CENTER_CENTER.align_size_within_rect(Vec2::new(width, size.y), rect);
    let root = state.game.project_root.clone();
    let shown = ui.scope_builder(egui::UiBuilder::new().max_rect(card), |ui| {
        egui::Frame::popup(ui.style()).inner_margin(12).show(ui, |ui| {
            // Selectable labels would take the clicks, drags and wheel meant for the camera behind the card.
            ui.style_mut().interaction.selectable_labels = false;
            ui.set_width(ui.available_width());
            ui.label(RichText::new("This is a new, empty map").strong().size(17.0));
            ui.add_space(4.0);
            let place = match start_view {
                Some(ViewKind::Top) => "in the Top view",
                Some(ViewKind::Front) => "in the Front view",
                Some(ViewKind::Side) => "in the Side view",
                _ => "in this view",
            };
            let first = if state.tool == ToolKind::Select {
                format!("To start, drag {place} to draw your first box.")
            } else {
                let keys =
                    crate::commands::shortcut_text(ui.ctx(), &state.prefs, &Action::SetTool(ToolKind::Select)).map(|k| format!(" ({k})")).unwrap_or_default();
                format!("To start, pick the Select tool{keys}, then drag {place} to draw your first box.")
            };
            ui.label(RichText::new(first).size(15.0).color(theme::YELLOW));
            ui.add_space(8.0);
            let hollow = match crate::commands::shortcut_text(ui.ctx(), &state.prefs, &Action::CsgHollow) {
                Some(keys) => format!("Brush > CSG > Hollow ({keys})"),
                None => "Brush > CSG > Hollow".into(),
            };
            ui.label(format!("Then {hollow} turns the box into a room."));
            ui.label(RichText::new(CAMERA_3D_HELP).weak());
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.hyperlink_to("Getting started guide", GETTING_STARTED_URL).on_hover_text(GETTING_STARTED_URL);
                let maps = maps.get(ui.ctx(), root.as_deref());
                if !maps.is_empty() {
                    ui.menu_button("Open a map from this project", |ui| maps_menu(ui, root.as_deref(), maps, actions))
                        .response
                        .on_hover_text("See how a finished map is put together");
                }

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
    fn maps_added_meanwhile_show_up_after_a_rescan() {
        let dir = std::env::temp_dir().join(format!("gt_welcome_rescan_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let ctx = egui::Context::default();
        let mut maps = ProjectMaps::default();
        let wait = |maps: &mut ProjectMaps| {
            for _ in 0..500 {
                if maps.pending.is_none() && maps.scanned.is_some() {
                    break;
                }

                std::thread::sleep(Duration::from_millis(5));
                maps.get(&ctx, Some(&dir));
            }

            maps.get(&ctx, Some(&dir)).to_vec()
        };
        assert!(wait(&mut maps).is_empty());
        std::fs::create_dir_all(dir.join("demo/maps")).unwrap();
        std::fs::write(dir.join("demo/maps/demo.gtm"), "").unwrap();
        assert!(wait(&mut maps).is_empty(), "the list is kept for a few seconds");
        maps.rescan();
        assert_eq!(wait(&mut maps), [dir.join("demo/maps/demo.gtm")], "an install asks for a new scan");
        std::fs::remove_dir_all(&dir).ok();
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

    struct Fixture {
        state: EditorState,
        maps: ProjectMaps,
        actions: Vec<Action>,
        start_view: Option<ViewKind>,
        view_hovered: bool,
        opened: Vec<String>,
    }

    impl Fixture {
        fn new(state: EditorState) -> Self {
            Self { state, maps: ProjectMaps::default(), actions: Vec::new(), start_view: Some(ViewKind::Top), view_hovered: false, opened: Vec::new() }
        }
    }

    /// The 3D view with its card, and the links it opens taken the way the app takes them.
    fn card_harness(fixture: Fixture) -> egui_kittest::Harness<'static, Fixture> {
        egui_kittest::Harness::builder().with_size(egui::vec2(800.0, 600.0)).build_ui_state(
            |ui, f: &mut Fixture| {
                let rect = ui.max_rect();
                f.view_hovered = ui.allocate_rect(rect, egui::Sense::click_and_drag()).hovered();
                view_hint(ui, rect, ViewKind::Perspective, f.start_view, &mut f.state, &mut f.maps, &mut f.actions);
                f.opened.extend(crate::panels::take_open_urls(ui.ctx()));
            },
            fixture,
        )
    }

    #[test]
    fn the_3d_view_card_says_where_to_start_and_lets_the_camera_through() {
        use egui_kittest::kittest::Queryable;

        let dir = std::env::temp_dir().join(format!("gt_welcome_hint_{}", std::process::id()));
        std::fs::create_dir_all(dir.join("maps")).unwrap();
        std::fs::write(dir.join("maps/church.gtm"), "").unwrap();
        let mut state = EditorState::new(Default::default());
        state.game.project_root = Some(dir.clone());
        let mut harness = card_harness(Fixture::new(state));
        harness.run();
        assert!(harness.query_by_label("To start, drag in the Top view to draw your first box.").is_some());
        assert!(harness.query_by_label_contains("Brush > CSG > Hollow (").is_some(), "names the menu and the shortcut");
        let title = harness.get_by_label("This is a new, empty map").rect().center();
        harness.hover_at(title);
        harness.run();
        assert!(harness.state().view_hovered, "the card's text lets the camera behind it take the mouse");

        harness.get_by_label("Getting started guide").click();
        harness.run();
        assert_eq!(harness.state().opened, [GETTING_STARTED_URL], "the link reaches the app, which opens it in the browser");

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
        assert!(harness.query_by_label("This is a new, empty map").is_none());
    }

    #[test]
    fn the_first_step_fits_the_open_views_and_the_tool() {
        use egui_kittest::kittest::Queryable;

        let mut harness = card_harness(Fixture { start_view: Some(ViewKind::Front), ..Fixture::new(EditorState::new(Default::default())) });
        harness.run();
        assert!(harness.query_by_label("To start, drag in the Front view to draw your first box.").is_some(), "the Top view is closed");
        harness.state_mut().start_view = None;
        harness.run();
        assert!(harness.query_by_label("To start, drag in this view to draw your first box.").is_some(), "only the 3D view is showing");

        harness.state_mut().state.tool = ToolKind::Clip;
        harness.run();
        assert!(harness.query_by_label_contains("pick the Select tool (Q), then drag in this view").is_some(), "only the Select tool draws boxes");

        let layer = harness.state().state.doc.map.default_layer();
        let brush = gt_geom::Brush::from_aabb(&gt_core::Aabb::new(gt_core::DVec3::ZERO, gt_core::DVec3::splat(64.0)), "dev/grey").unwrap();
        harness.state_mut().state.doc.map.insert(layer, gt_doc::NodeKind::Brush(brush));
        harness.run();
        assert!(harness.query_by_label("This is a new, empty map").is_none(), "the card goes once the map has something in it");
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
