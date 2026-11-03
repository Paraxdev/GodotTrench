//! Project trust, the way VS Code has it. Godot runs a project's scripts whenever it opens, plays or builds it, so a
//! project whose authors the person has not trusted opens in Restricted Mode: its maps can be viewed and edited, but
//! TrenchDot does not start Godot on it or have Godot build its maps until the person trusts it.

use std::path::{Path, PathBuf};

use egui::RichText;

use crate::commands::Action;
use crate::state::{EditorState, TrustRequest};

pub const QUESTION: &str = "Do you trust the authors of the files in this project?";
pub const CHIP: &str = "Restricted Mode";
pub const TRUST: &str = "Trust Project";
pub const RESTRICT: &str = "Open in Restricted Mode";
pub const KEEP: &str = "Keep Current Project";
const BLOCKS: &str = "Godot runs a project's scripts when it opens, plays or builds it. In Restricted Mode you can view and edit the maps here, but TrenchDot does not start Godot on the project, build in Godot or send live edits until you trust it.";
pub const CHIP_TIP: &str = "TrenchDot does not run this project in Godot until you trust it. Click to choose";
/// Why an action that has Godot run the project did nothing. MCP returns it as the error.
pub const BLOCKED: &str = "This project is in Restricted Mode, so Godot does not run or build it. Click Restricted Mode in the status bar to trust it";
/// [`BLOCKED`] as the reason in a longer message.
pub const RESTRICTED: &str = "the project is in Restricted Mode, click Restricted Mode in the status bar to trust it";

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

/// The folder `root` is in, None when that is a drive or filesystem root, which is too much to trust with one click.
pub fn parent_folder(root: &Path) -> Option<&Path> {
    root.parent().filter(|p| p.parent().is_some())
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
            ui.label(RichText::new(root.display().to_string()).monospace().color(crate::theme::CYAN));
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
    use crate::state::Prefs;
    use egui_kittest::kittest::Queryable;

    /// Two Godot projects side by side in a folder of their own.
    fn projects(name: &str) -> (PathBuf, PathBuf, PathBuf) {
        let folder = std::env::temp_dir().join(format!("td_trust_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        let (castle, keep) = (folder.join("castle"), folder.join("keep"));
        for dir in [&castle, &keep] {
            std::fs::create_dir_all(dir).unwrap();
            std::fs::write(dir.join("project.godot"), "config_version=5\n").unwrap();
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
        assert!(state.restricted());
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
    fn the_prompt_trusts_the_project_or_leaves_it_restricted() {
        let (folder, castle, keep) = projects("prompt");
        let mut state = EditorState::new(Prefs::default());
        state.load_project(&castle);
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
}
