//! Project trust, the way VS Code has it. Godot runs a project's scripts whenever it opens, plays or builds it, so a
//! project whose authors the person has not trusted opens in Restricted Mode: its maps can be viewed and edited, but
//! TrenchDot does not start Godot on it or have Godot build its maps until the person trusts it. A project that holds
//! no code, such as a new one, gives Godot nothing to run and opens trusted, see [`crate::code_scan`].

use std::path::{Path, PathBuf};

use egui::RichText;

use crate::commands::Action;
use crate::state::{EditorState, TrustRequest};

pub const QUESTION: &str = "Do you trust the authors of the files in this project?";
pub const CHIP: &str = "Restricted Mode";
pub const SCANNING: &str = "Looking for code";
pub const SCANNING_TIP: &str = "Looking through the project for code Godot would run. Godot is not started until it is done";
pub const TRUST: &str = "Trust Project";
pub const RESTRICT: &str = "Open in Restricted Mode";
pub const KEEP: &str = "Keep Current Project";
const BLOCKS: &str =
    "Godot runs a project's scripts. In Restricted Mode you can edit maps, but TrenchDot does not start Godot on the project until you trust it.";
pub const CHIP_TIP: &str = "TrenchDot does not run this project in Godot until you trust it. Click to choose";
/// Why an action that has Godot run the project did nothing. MCP returns it as the error.
pub const BLOCKED: &str = "This project is in Restricted Mode, so Godot is not run on it. Click Restricted Mode in the status bar to trust it";
/// [`BLOCKED`] as the reason in a longer message.
pub const RESTRICTED: &str = "the project is in Restricted Mode, click Restricted Mode in the status bar to trust it";

/// The status bar entry for the open project's trust. A note while its code scan runs, the Restricted Mode chip once
/// the scan ended and the project stays restricted, whose response is returned.
pub fn status_entry(ui: &mut egui::Ui, state: &EditorState) -> Option<egui::Response> {
    if state.scanning_code() {
        ui.separator();
        ui.label(RichText::new(SCANNING).color(crate::theme::colors().fg.gamma_multiply_u8(160))).on_hover_text(SCANNING_TIP);
        None
    } else if state.restricted() {
        ui.separator();
        let chip = egui::Button::new(RichText::new(CHIP).color(crate::theme::colors().yellow)).frame(false);
        Some(ui.add(chip).on_hover_text(CHIP_TIP))
    } else {
        None
    }
}

/// `path` with links resolved, as trusted folders are stored. The part that does not exist yet is kept as written.
pub fn real_path(path: &Path) -> PathBuf {
    let path = crate::state::absolute(path);
    let mut missing = Vec::new();
    let mut base = path.as_path();
    loop {
        if let Ok(real) = base.canonicalize() {
            return missing.into_iter().rev().fold(real, |p, name| p.join(name));
        }

        match (base.parent(), base.file_name()) {
            (Some(parent), Some(name)) => {
                missing.push(name);
                base = parent;
            }
            _ => return path.clone(),
        }
    }
}

/// Whether `path` lies in the folder `root`, as written or with links resolved.
pub fn within(path: &Path, root: &Path) -> bool {
    path.starts_with(root) || real_path(path).starts_with(real_path(root))
}

/// The folder `root` is in, None when that is a drive or filesystem root, which is too much to trust with one click.
pub fn parent_folder(root: &Path) -> Option<&Path> {
    root.parent().filter(|p| p.parent().is_some())
}

pub const TRUSTED_TITLE: &str = "Trusted projects";
const TRUSTED_NOTE: &str = "TrenchDot runs Godot on these projects. Other projects with code open in Restricted Mode.";
pub const REMOVE: &str = "Remove";
const LIST_HEIGHT: f32 = 180.0;
/// Follows a folder in the list.
const FOLDER: &str = "and every project in it";

/// A stored path the way a person writes it, without the `\\?\` that canonical paths have on Windows.
pub fn shown(path: &Path) -> String {
    let text = path.display().to_string();
    match text.strip_prefix(r"\\?\") {
        Some(unc) if unc.starts_with("UNC\\") => format!(r"\\{}", &unc[4..]),
        Some(local) => local.to_string(),
        None => text,
    }
}

/// The Trusted projects part of Preferences: every trusted project and folder, each with a button that takes the
/// trust back.
pub fn trusted_list(ui: &mut egui::Ui, state: &mut EditorState) {
    ui.label(RichText::new(TRUSTED_TITLE).strong());
    ui.label(TRUSTED_NOTE);
    let entries: Vec<(PathBuf, bool)> =
        state.prefs.trusted_projects.iter().map(|p| (p.clone(), false)).chain(state.prefs.trusted_folders.iter().map(|p| (p.clone(), true))).collect();
    if entries.is_empty() {
        ui.label(RichText::new("None yet. Trust Project in the trust prompt adds one.").weak());
        return;
    }

    let mut remove = None;
    // The window around the list only offers the height it had last frame, so without a minimum the list would start
    // at 64 points and never grow. It still shrinks to its rows.
    egui::ScrollArea::vertical().min_scrolled_height(LIST_HEIGHT).max_height(LIST_HEIGHT).show(ui, |ui| {
        egui::Grid::new("trusted_projects").num_columns(2).striped(true).show(ui, |ui| {
            for (path, folder) in &entries {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(shown(path)).monospace());
                    if *folder {
                        ui.label(FOLDER);
                    }
                });
                if ui.button(REMOVE).on_hover_text("TrenchDot asks again before Godot runs the scripts here").clicked() {
                    remove = Some((path.clone(), *folder));
                }

                ui.end_row();
            }
        });
    });
    if let Some((path, folder)) = remove {
        state.untrust(&path, folder);
    }
}

#[derive(Default)]
pub struct TrustPrompt {
    request: Option<TrustRequest>,
    /// Trust every project in the parent folder, not only this one.
    parent: bool,
}

impl TrustPrompt {
    pub fn is_open(&self) -> bool {
        self.request.is_some()
    }

    /// Takes what the editor asked for and shows the prompt while it is open. An action the person trusted the project
    /// for goes to `actions`. Returns where the prompt is.
    pub fn show(&mut self, ctx: &egui::Context, state: &mut EditorState, actions: &mut Vec<Action>) -> Option<egui::Rect> {
        if let Some(request) = state.trust_request.take() {
            self.request = Some(request);
            self.parent = false;
        }

        let request = self.request.clone()?;
        let stale = match &request {
            TrustRequest::Project(root) | TrustRequest::Run(root, _) => state.game.project_root.as_ref() != Some(root) || !state.restricted(),
            TrustRequest::Switch { root, .. } => state.game.project_root.as_ref() == Some(root),
        };
        if stale {
            self.request = None;
            return None;
        }

        let root = request.root().to_path_buf();
        let mut answer = None;
        let modal = egui::Modal::new(egui::Id::new("trust_prompt")).show(ctx, |ui| {
            ui.set_width(540.0);
            ui.heading(QUESTION);
            ui.add_space(4.0);
            ui.label(RichText::new(root.display().to_string()).monospace().color(crate::theme::colors().cyan));
            ui.add_space(4.0);
            match &request {
                TrustRequest::Project(_) => {
                    ui.label(BLOCKS);
                }
                TrustRequest::Run(_, action) => {
                    ui.label(format!("{} has Godot run this project.", action.label()));
                    ui.label(RichText::new("Running it runs its scripts. Trust it and continue?").strong());
                }
                TrustRequest::Switch { map, .. } => {
                    let name = map.file_name().unwrap_or_default().to_string_lossy();
                    ui.label(format!("{name} belongs to this project, not to the one that is open. {BLOCKS}"));
                }
            }

            if let Some(name) = parent_folder(&root).and_then(Path::file_name) {
                ui.add_space(4.0);
                ui.checkbox(&mut self.parent, format!("Trust the authors of all files in the parent folder '{}'", name.to_string_lossy()));
            }

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button(RichText::new(TRUST).strong()).clicked() {
                    answer = Some(Answer::Trust);
                }

                match request {
                    TrustRequest::Run(..) => {
                        if ui.button("Cancel").clicked() {
                            answer = Some(Answer::Keep);
                        }
                    }
                    _ => {
                        if ui.button(RESTRICT).clicked() {
                            answer = Some(Answer::Restrict);
                        }
                    }
                }

                if matches!(request, TrustRequest::Switch { .. }) && ui.button(KEEP).clicked() {
                    answer = Some(Answer::Keep);
                }
            });
        });
        if modal.should_close() && answer.is_none() {
            answer = Some(if matches!(request, TrustRequest::Project(_)) { Answer::Restrict } else { Answer::Keep });
        }

        let Some(answer) = answer else { return Some(modal.response.rect) };
        let restricted = format!("{} is open in Restricted Mode, click Restricted Mode in the status bar to trust it", root.display());
        match (answer, request) {
            (Answer::Trust, request) => {
                state.trust_project(&root, self.parent);
                match &request {
                    TrustRequest::Switch { .. } => state.load_project(&root),
                    TrustRequest::Run(_, action) => actions.push(action.clone()),
                    TrustRequest::Project(_) => {}
                }

                match parent_folder(&root).filter(|_| self.parent) {
                    Some(parent) => state.set_status(format!("Trusted every project in {}", parent.display())),
                    None => state.set_status(format!("Trusted {}", root.display())),
                }
            }
            (Answer::Restrict, TrustRequest::Switch { .. }) => {
                state.load_project(&root);
                state.trust_request = None;
                state.set_status(restricted);
            }
            (Answer::Restrict, _) => state.set_status(restricted),
            (Answer::Keep, TrustRequest::Run(_, action)) => {
                state.set_status(format!("Cancelled {}, {} stays in Restricted Mode", action.label(), root.display()));
            }
            (Answer::Keep, _) => state.set_status(format!("Kept the open project, {} stays closed", root.display())),
        }

        self.request = None;
        Some(modal.response.rect)
    }
}

enum Answer {
    Trust,
    Restrict,
    /// Cancel the action, or keep the project that is open.
    Keep,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Prefs, Trust};
    use egui_kittest::kittest::Queryable;

    /// Two Godot projects with a script each, side by side in a folder of their own.
    fn projects(name: &str) -> (PathBuf, PathBuf, PathBuf) {
        let folder = std::env::temp_dir().join(format!("td_trust_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        let (castle, keep) = (folder.join("castle"), folder.join("keep"));
        for dir in [&castle, &keep] {
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(dir.join("project.godot"), "config_version=5\n").unwrap();
            std::fs::write(dir.join("main.gd"), "extends Node\n").unwrap();
        }

        (folder, crate::state::absolute(&castle), crate::state::absolute(&keep))
    }

    struct Fixture {
        state: EditorState,
        prompt: TrustPrompt,
        actions: Vec<Action>,
    }

    fn harness(state: EditorState) -> egui_kittest::Harness<'static, Fixture> {
        egui_kittest::Harness::builder().with_size(egui::vec2(900.0, 600.0)).build_ui_state(
            |ui, f: &mut Fixture| {
                let ctx = ui.ctx().clone();
                f.prompt.show(&ctx, &mut f.state, &mut f.actions);
            },
            Fixture { state, prompt: TrustPrompt::default(), actions: Vec::new() },
        )
    }

    #[test]
    fn an_untrusted_project_opens_in_restricted_mode_until_it_is_trusted() {
        let (folder, castle, keep) = projects("restricted");
        let mut state = EditorState::new(Prefs::default());
        state.load_project(&castle);
        assert!(state.restricted(), "nothing runs before the scan ends");
        state.apply_code_scans(true);
        assert!(state.restricted() && !state.scanning_code());
        assert_eq!(state.trust_request, Some(TrustRequest::Project(castle.clone())));
        state.trust_request = None;
        state.load_project(&castle);
        assert_eq!(state.trust_request, None, "a reload of the same project does not ask again");

        state.trust_project(&castle, false);
        assert!(!state.restricted());
        assert_eq!(state.prefs.trusted_projects.iter().collect::<Vec<_>>(), [&real_path(&castle)]);
        state.load_project(&keep);
        assert!(state.restricted(), "trusting one project leaves its neighbours alone");

        let pack = castle.join("maps/pack");
        std::fs::create_dir_all(&pack).unwrap();
        assert!(!state.is_trusted(&pack), "a project unpacked inside a trusted one is not trusted with it");

        state.trust_project(&keep, true);
        assert!(!state.restricted());
        assert!(state.is_trusted(&castle) && state.is_trusted(&pack) && state.is_trusted(&folder.join("another")));
        assert!(state.prefs.trusted_projects.is_empty(), "the parent covers the project trusted before");
        assert_eq!(state.prefs.trusted_folders.iter().collect::<Vec<_>>(), [&real_path(&folder)]);

        let mut state = EditorState::new(Prefs::default());
        state.trust_for_session(&castle);
        state.load_project(&castle);
        assert!(!state.restricted() && state.trust_request.is_none());
        assert!(state.prefs.trusted_projects.is_empty(), "a --project start is trusted for the session only");
        assert!(!state.is_trusted(&castle.join("maps/pack")));
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn preferences_from_before_project_trust_trust_their_recent_projects_once() {
        let (folder, castle, keep) = projects("seed");
        let mut prefs: Prefs = serde_json::from_value(serde_json::json!({ "recent_projects": [castle] })).unwrap();
        assert!(!prefs.trust_seeded);
        prefs.seed_trust(Some(&keep));
        assert!(prefs.trust_seeded);
        assert_eq!(prefs.trusted_projects, [real_path(&castle), real_path(&keep)].into());
        prefs.recent_projects.push(folder.join("downloaded"));
        prefs.seed_trust(None);
        assert_eq!(prefs.trusted_projects.len(), 2, "projects opened later are asked about");

        let mut fresh = Prefs::default();
        fresh.recent_projects.push(castle.clone());
        fresh.seed_trust(Some(&keep));
        assert!(fresh.trusted_projects.is_empty(), "new preferences have nothing to carry over");
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn a_project_without_code_opens_trusted_until_it_has_some() {
        let (folder, castle, keep) = projects("no_code");
        std::fs::remove_file(castle.join("main.gd")).unwrap();
        let mut state = EditorState::new(Prefs::default());
        assert!(!state.restricted() && state.trust().is_none(), "no project, nothing to ask");
        state.load_project(&castle);
        state.apply_code_scans(true);
        assert!(!state.restricted() && state.trust_request.is_none(), "no prompt and no chip");
        assert_eq!(state.trust(), Some(Trust::NoCode));
        assert!(state.prefs.trusted_projects.is_empty() && state.prefs.trusted_folders.is_empty(), "nothing is stored");

        // Godot adds imported files while the project is open, so it stays trusted until it closes.
        std::fs::write(castle.join("main.gd"), "extends Node\n").unwrap();
        state.load_project(&castle);
        assert_eq!(state.trust(), Some(Trust::NoCode));
        state.load_project(&keep);
        state.apply_code_scans(true);
        state.trust_request = None;
        state.load_project(&castle);
        state.apply_code_scans(true);
        assert!(state.restricted(), "opened again, the project with a script is asked about");
        assert_eq!(state.trust_request, Some(TrustRequest::Project(castle.clone())));

        // A map of a project without code switches to it as to a trusted one.
        std::fs::remove_file(castle.join("main.gd")).unwrap();
        td_doc::format::save(&td_doc::Map::new(), &castle.join("level.tdm")).unwrap();
        state.trust_for_session(&keep);
        state.load_project(&keep);
        state.trust_request = None;
        state.open_map(&castle.join("level.tdm")).unwrap();
        state.apply_code_scans(true);
        assert_eq!(state.game.project_root.as_ref(), Some(&castle));
        assert!(state.trust_request.is_none() && !state.restricted());
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn saving_a_map_from_the_gui_trusts_a_project_without_code() {
        let (folder, castle, keep) = projects("save");
        std::fs::remove_file(castle.join("main.gd")).unwrap();
        let ctx = egui::Context::default();
        let mut state = EditorState::new(Prefs::default());
        state.load_project(&castle);
        state.apply_code_scans(true);
        state.save_map(&castle.join("agent.tdm")).unwrap();
        assert_eq!(state.trust(), Some(Trust::NoCode), "the save MCP makes stores nothing");
        state.doc.path = Some(folder.join("loose.tdm"));
        crate::commands::execute(&mut state, Action::Save, &ctx);
        assert_eq!(state.trust(), Some(Trust::NoCode), "nor does a map saved outside the project");

        state.doc.path = Some(castle.join("yard.tdm"));
        crate::commands::execute(&mut state, Action::Save, &ctx);
        assert!(castle.join("yard.tdm").is_file());
        assert_eq!(state.trust(), Some(Trust::Stored));
        assert_eq!(state.prefs.trusted_projects.iter().collect::<Vec<_>>(), [&real_path(&castle)]);
        std::fs::write(castle.join("main.gd"), "extends Node\n").unwrap();
        let mut later = EditorState::new(state.prefs.clone());
        later.load_project(&castle);
        assert!(!later.restricted(), "the project keeps opening trusted once it has scripts");

        let mut state = EditorState::new(Prefs::default());
        state.load_project(&keep);
        state.doc.path = Some(keep.join("yard.tdm"));
        crate::commands::execute(&mut state, Action::Save, &ctx);
        assert!(keep.join("yard.tdm").is_file());
        assert!(state.restricted() && state.prefs.trusted_projects.is_empty(), "a save in Restricted Mode trusts nothing");

        let mut state = EditorState::new(Prefs::default());
        state.trust_for_session(&keep);
        state.load_project(&keep);
        state.doc.path = Some(keep.join("yard.tdm"));
        crate::commands::execute(&mut state, Action::Save, &ctx);
        assert_eq!(state.trust(), Some(Trust::Session), "a --project start stays trusted for the session only");
        assert!(state.prefs.trusted_projects.is_empty());
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn removing_the_trust_of_the_open_project_restricts_it_unless_it_holds_no_code() {
        let (folder, castle, keep) = projects("untrust");
        let elsewhere = real_path(&folder.join("elsewhere"));
        let mut state = EditorState::new(Prefs::default());
        state.prefs.trusted_projects.insert(real_path(&castle));
        state.prefs.trusted_folders.insert(elsewhere.clone());
        state.load_project(&castle);
        assert_eq!(state.trust(), Some(Trust::Stored));
        state.untrust(&elsewhere, true);
        assert!(state.prefs.trusted_folders.is_empty() && !state.restricted(), "another entry leaves the open project alone");
        state.untrust(&real_path(&castle), false);
        assert!(state.prefs.trusted_projects.is_empty());
        assert!(state.restricted(), "the open project is in Restricted Mode at once");
        state.apply_code_scans(true);
        assert!(state.status.ends_with("is in Restricted Mode now"), "{}", state.status);

        state.trust_project(&castle, true);
        assert_eq!(state.prefs.trusted_folders.iter().collect::<Vec<_>>(), [&real_path(&folder)]);
        state.untrust(&real_path(&folder), true);
        assert!(state.restricted(), "so does taking back the trust of its folder");

        std::fs::remove_file(keep.join("main.gd")).unwrap();
        state.prefs.trusted_projects.insert(real_path(&keep));
        state.load_project(&keep);
        assert_eq!(state.trust(), Some(Trust::Stored));
        state.untrust(&real_path(&keep), false);
        state.apply_code_scans(true);
        assert_eq!(state.trust(), Some(Trust::NoCode), "a project without code stays trusted");
        assert_eq!(state.status, format!("{} is no longer trusted", shown(&real_path(&keep))));
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn preferences_list_the_trusted_projects_with_a_remove_button_each() {
        let (folder, castle, keep) = projects("list");
        let mut state = EditorState::new(Prefs::default());
        state.load_project(&castle);
        state.trust_project(&castle, false);
        state.trust_project(&keep, true);
        state.prefs.trusted_projects.insert(real_path(&keep.join("nested")));
        let mut harness = egui_kittest::Harness::builder().with_size(egui::vec2(900.0, 400.0)).build_ui_state(trusted_list, state);
        harness.run();
        assert!(harness.query_by_label(TRUSTED_TITLE).is_some());
        assert!(harness.query_by_label(&shown(&real_path(&keep.join("nested")))).is_some());
        assert!(
            harness.query_by_label(&shown(&real_path(&folder))).is_some() && harness.query_by_label(FOLDER).is_some(),
            "the parent folder is listed as one"
        );
        assert!(harness.query_by_label(&shown(&real_path(&castle))).is_none(), "the folder covers the project trusted before");
        assert_eq!(harness.get_all_by_label(REMOVE).count(), 2);

        harness.get_all_by_label(REMOVE).last().unwrap().click();
        harness.run();
        let state = harness.state();
        assert!(state.prefs.trusted_folders.is_empty() && state.restricted(), "the open project lay in the removed folder");
        assert_eq!(state.prefs.trusted_projects.len(), 1);
        let _ = std::fs::remove_dir_all(folder);
    }

    /// Preferences is an auto-sized window, so the list only gets the height the window already has below the
    /// settings above it.
    fn list_in_window(listed: usize, folders: usize) -> Vec<egui::Rect> {
        let mut state = EditorState::new(Prefs::default());
        for i in 0..listed {
            state.prefs.trusted_projects.insert(PathBuf::from(format!("/games/slanted_{i}/x{}", i * 7)));
        }

        for i in 0..folders {
            state.prefs.trusted_folders.insert(PathBuf::from(format!("/sets/{i}")));
        }

        let mut harness = egui_kittest::Harness::builder().with_size(egui::vec2(900.0, 900.0)).build_ui_state(
            |ui, (state, window): &mut (EditorState, egui::Rect)| {
                let shown = egui::Window::new("Preferences").resizable(false).show(ui.ctx(), |ui| {
                    for i in 0..12 {
                        ui.label(format!("setting {i}"));
                    }

                    ui.separator();
                    trusted_list(ui, state);
                });
                *window = shown.unwrap().response.rect;
            },
            (state, egui::Rect::NOTHING),
        );
        harness.run();
        let window = harness.state().1;
        let mut rows: Vec<egui::Rect> = harness.get_all_by_label(REMOVE).map(|b| b.rect()).collect();
        rows.push(window);
        rows
    }

    #[test]
    fn the_trusted_list_keeps_every_row_inside_the_preferences_window() {
        for (listed, folders) in [(1, 0), (0, 1), (3, 1), (6, 0)] {
            let mut rects = list_in_window(listed, folders);
            let window = rects.pop().unwrap();
            assert_eq!(rects.len(), listed + folders);
            for (i, row) in rects.iter().enumerate() {
                assert!(row.max.y <= window.max.y, "row {i} of {} lies below the window, {row:?} against {window:?}", rects.len());
            }
        }
    }

    #[test]
    fn the_prompt_trusts_the_project_or_leaves_it_restricted() {
        let (folder, castle, keep) = projects("prompt");
        let mut state = EditorState::new(Prefs::default());
        state.load_project(&castle);
        state.apply_code_scans(true);
        let mut harness = harness(state);
        harness.run();
        assert!(harness.query_by_label(QUESTION).is_some());
        assert!(harness.query_by_label(&castle.display().to_string()).is_some());
        harness.get_by_label(RESTRICT).click();
        harness.run();
        assert!(!harness.state().prompt.is_open());
        assert!(harness.state().state.restricted());
        assert!(harness.state().state.status.contains("Restricted Mode"), "{}", harness.state().state.status);

        harness.state_mut().state.trust_request = Some(TrustRequest::Run(castle.clone(), Action::RunGodotProject));
        harness.run();
        assert!(harness.query_by_label("Run Project has Godot run this project.").is_some());
        assert!(harness.query_by_label("Running it runs its scripts. Trust it and continue?").is_some());
        harness.get_by_label("Cancel").click();
        harness.run();
        assert!(harness.state().state.restricted() && harness.state().actions.is_empty(), "nothing runs without trust");

        harness.state_mut().state.trust_request = Some(TrustRequest::Run(castle.clone(), Action::RunGodotProject));
        harness.run();
        harness.get_by_label_contains("in the parent folder").click();
        harness.get_by_label(TRUST).click();
        harness.run();
        let f = harness.state();
        assert!(!f.prompt.is_open() && !f.state.restricted());
        assert_eq!(f.actions, [Action::RunGodotProject], "the action goes on once trusted");
        assert!(f.state.is_trusted(&keep), "the parent folder is trusted");
        assert!(f.state.status.starts_with("Trusted every project in"), "{}", f.state.status);
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn a_map_of_another_project_asks_whether_to_switch() {
        let (folder, castle, keep) = projects("switch");
        let map = keep.join("level.tdm");
        td_doc::format::save(&td_doc::Map::new(), &map).unwrap();
        let mut state = EditorState::new(Prefs::default());
        state.trust_for_session(&castle);
        state.load_project(&castle);
        state.open_map(&map).unwrap();
        state.apply_code_scans(true);
        let mut harness = harness(state);
        harness.run();
        assert!(harness.query_by_label(&keep.display().to_string()).is_some());
        assert!(harness.query_by_label_contains("level.tdm belongs to this project").is_some());
        harness.get_by_label(KEEP).click();
        harness.run();
        assert_eq!(harness.state().state.game.project_root.as_ref(), Some(&castle));

        let request = TrustRequest::Switch { root: keep.clone(), map: map.clone() };
        harness.state_mut().state.trust_request = Some(request.clone());
        harness.run();
        harness.get_by_label(RESTRICT).click();
        harness.run();
        let f = harness.state();
        assert_eq!(f.state.game.project_root.as_ref(), Some(&keep));
        assert!(f.state.restricted() && !f.prompt.is_open(), "switched without asking a second time");

        let f = harness.state_mut();
        f.state.load_project(&castle);
        f.state.trust_request = Some(request);
        harness.run();
        harness.get_by_label(TRUST).click();
        harness.run();
        let state = &harness.state().state;
        assert_eq!(state.game.project_root.as_ref(), Some(&keep));
        assert!(!state.restricted() && state.is_trusted(&keep));
        let _ = std::fs::remove_dir_all(folder);
    }

    #[test]
    fn the_status_bar_says_it_looks_for_code_until_the_scan_ends() {
        let (folder, castle, _) = projects("status");
        let hold = crate::code_scan::held::hold(&castle);
        let mut state = EditorState::new(Prefs::default());
        state.load_project(&castle);
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::vec2(400.0, 60.0))
            .build_ui_state(|ui, state: &mut EditorState| drop(ui.horizontal(|ui| status_entry(ui, state))), state);
        harness.run();
        assert!(harness.state().scanning_code());
        assert!(harness.query_by_label(SCANNING).is_some(), "the scan is still running");
        assert!(harness.query_by_label(CHIP).is_none());

        drop(hold);
        harness.state_mut().apply_code_scans(true);
        harness.run();
        assert!(harness.state().restricted() && !harness.state().scanning_code());
        assert!(harness.query_by_label(SCANNING).is_none());
        assert!(harness.query_by_label(CHIP).is_some(), "the scan found code, so the chip takes its place");

        harness.state_mut().trust_project(&castle, false);
        harness.run();
        assert!(harness.query_by_label(SCANNING).is_none() && harness.query_by_label(CHIP).is_none());
        let _ = std::fs::remove_dir_all(folder);
    }
}
