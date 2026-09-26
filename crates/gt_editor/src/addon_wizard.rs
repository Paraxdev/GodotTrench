//! The GodotTrench addon part of the setup wizard: says in plain words what the project has, and offers the one step
//! that comes next, installing, updating or enabling the addon.

use std::path::{Path, PathBuf};
use std::time::Duration;

use egui::{RichText, Ui};

use crate::addon_install::{self, Installed, Job, Outcome, Status};
use crate::content::Error;
use crate::state::EditorState;
use crate::theme;

pub const HEADING: &str = "GodotTrench addon";
/// Where people download by hand what the editor downloads by itself.
pub const RELEASES_PAGE: &str = "https://github.com/Paraxdev/GodotTrench/releases";
/// A storefront with the same files. The editor never downloads from it.
pub const ITCH_PAGE: &str = "https://paraxdev.itch.io/godottrench";

/// Seconds between looks at the project while the section shows, so it notices Godot exporting the game config.
const RECHECK: f64 = 1.0;

#[derive(Default)]
pub struct AddonWizard {
    checked: Option<(PathBuf, Status, f64)>,
    job: Option<(PathBuf, Job)>,
    outcome: Option<Outcome>,
    /// What enabling the plugin said.
    enabled: Option<Result<bool, String>>,
    /// Where the release is looked up, [`crate::content::api_url`] unless a test serves it.
    pub api: Option<String>,
}

impl AddonWizard {
    /// Forgets what the last install and enable said, unless an install is still running.
    pub fn reset(&mut self) {
        if self.job.is_none() {
            self.outcome = None;
            self.enabled = None;
        }

        self.checked = None;
    }

    pub fn running(&self) -> bool {
        self.job.is_some()
    }

    fn status(&mut self, root: &Path, now: f64) -> Status {
        match &self.checked {
            Some((checked, status, at)) if checked == root && (*at..*at + RECHECK).contains(&now) => status.clone(),
            _ => {
                let status = Status::check(root);
                self.checked = Some((root.to_path_buf(), status.clone(), now));
                status
            }
        }
    }

    pub fn start(&mut self, root: PathBuf, repaint: Option<egui::Context>) {
        if self.job.is_some() {
            return;
        }

        self.outcome = None;
        self.enabled = None;
        let api = self.api.clone().unwrap_or_else(crate::content::api_url);
        self.job = Some((root.clone(), Job::start(root, api, repaint)));
    }

    /// Checks on a running install. Once it is done, the editor learns the new version and the status bar says what
    /// happened.
    pub fn poll(&mut self, state: &mut EditorState) -> Option<Outcome> {
        let outcome = self.job.as_ref()?.1.poll()?;
        let (root, _) = self.job.take()?;
        self.checked = None;
        if state.game.project_root.as_ref() == Some(&root) {
            state.game.addon_version = gt_formats::game::addon_version(&root);
        }

        state.set_status(outcome.summary(&root));
        self.outcome = Some(outcome.clone());
        Some(outcome)
    }

    /// Remembers that a person let a missing or mismatched addon be, so opening the project does not ask again until
    /// the editor or the addon changes.
    pub fn dismiss(&self, state: &mut EditorState, root: &Path) {
        let status = Status::check(root);
        if status.mismatched() {
            state.prefs.addon_dismissed.insert(root.to_path_buf(), status.dismiss_key());
        } else {
            state.prefs.addon_dismissed.remove(root);
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, state: &mut EditorState, root: &Path) {
        ui.label(RichText::new(HEADING).strong());
        if let Some((_, job)) = &self.job {
            let progress = job.progress();
            let stage = match progress.stage.as_str() {
                _ if job.cancelling() => "Cancelling…",
                "" => "Starting",
                stage => stage,
            };
            ui.label(stage);
            ui.add(egui::ProgressBar::new(progress.fraction()).text(progress.amount()).animate(progress.total == 0));
            let cancel = ui.add_enabled(!job.cancelling(), egui::Button::new("Cancel"));
            if cancel.on_hover_text("Stops the install, the project stays as it was").clicked() {
                job.cancel();
            }

            return;
        }

        let status = self.status(root, ui.input(|i| i.time));
        if let Some(outcome) = &self.outcome {
            let color = if outcome.error.is_some() { theme::WARNING } else { theme::SUCCESS };
            ui.label(RichText::new(outcome.summary(root)).color(color));
            if outcome.error.as_ref().is_some_and(Error::is_download) {
                download_links(ui, &[addon_install::ADDON.asset]);
            }

            if let Some(backup) = &outcome.backup
                && ui.button("Show in File Manager").on_hover_text("Shows the folder with the old addon").clicked()
            {
                crate::panels::show_in_file_manager(backup);
            }
        }

        let godot_open = state.godot_has_project();
        let editor = crate::VERSION;
        let (text, button) = match &status.installed {
            Installed::Missing => {
                let name = crate::content::project_name(root);
                (
                    format!(
                        "Godot needs the GodotTrench addon to turn maps into scenes, and {name} does not have it yet. It goes into res://addons/func_godot."
                    ),
                    "Install the addon".to_string(),
                )
            }
            Installed::Older(v) => (
                format!("The addon in this project is {v}, older than this editor, {editor}. Update it, so Godot builds maps the way the editor shows them."),
                format!("Update the addon to {editor}"),
            ),
            Installed::Newer(v) => (
                format!(
                    "The addon in this project is {v}, newer than this editor, {editor}. Get the editor that matches it from the releases, or go back to the addon {editor}."
                ),
                format!("Replace the addon with {editor}"),
            ),
            Installed::Unknown => (
                "res://addons/func_godot has no version in its plugin.cfg, so it is damaged or not the GodotTrench addon.".to_string(),
                format!("Replace the addon with {editor}"),
            ),
            Installed::Current => {
                self.current_ui(ui, state, root, &status, godot_open);
                return;
            }
        };

        ui.label(text);
        ui.ctx().request_repaint_after(Duration::from_secs_f64(RECHECK));
        if status.git {
            ui.label(
                RichText::new("It is a git checkout, so the editor leaves it alone. Update it with git, for example with git submodule update --remote.")
                    .color(theme::YELLOW),
            );
            return;
        }

        let replacing = status.installed != Installed::Missing;
        if replacing && godot_open {
            ui.label(
                RichText::new("Godot has this project open. Close it first, it keeps running the old addon and holds on to its files.").color(theme::YELLOW),
            );
        }

        let go = ui.add_enabled(!(replacing && godot_open), egui::Button::new(RichText::new(button).strong()));
        if go.on_disabled_hover_text("Close Godot first").clicked() {
            self.start(root.to_path_buf(), Some(ui.ctx().clone()));
        }

        if replacing && !godot_open {
            ui.label(RichText::new("The old addon is kept in a backup folder that Godot ignores. If Godot has this project open, close it first.").weak());
        }
    }

    fn current_ui(&mut self, ui: &mut Ui, state: &mut EditorState, root: &Path, status: &Status, godot_open: bool) {
        if let Some(result) = &self.enabled {
            let (text, color) = match result {
                Ok(_) => ("Enabled the plugin in project.godot.", theme::SUCCESS),
                Err(e) => (e.as_str(), theme::WARNING),
            };
            ui.label(RichText::new(text).color(color));
        }

        if !status.enabled {
            ui.label(format!("The addon {} is installed but not enabled, so Godot does not use it yet.", crate::VERSION));
            if godot_open {
                ui.label("Godot has this project open, enable it there under Project > Project Settings > Plugins.");
            } else {
                if ui.button(RichText::new("Enable the plugin").strong()).clicked() {
                    self.enabled = Some(addon_install::enable(root));
                    self.checked = None;
                }

                ui.label(
                    RichText::new("Adds it to the plugins in project.godot and leaves the rest of the file as it is. Close Godot first if it has this project open, or it writes its own copy back.")
                        .weak(),
                );
            }
        } else if !status.exported {
            ui.label("Next, open the project in Godot once. The addon then writes godottrench_game.json with the project's entities and textures, and the editor loads it by itself.");
            if !godot_open && state.godot.found() && ui.button("Open in Godot").clicked() {
                crate::commands::execute(state, crate::commands::Action::OpenGodotEditor, ui.ctx());
            }
        } else {
            ui.label(RichText::new(format!("The addon {} is installed and enabled.", crate::VERSION)).color(theme::SUCCESS));
            return;
        }

        ui.ctx().request_repaint_after(Duration::from_secs_f64(RECHECK));
    }
}

/// Links to the pages that carry `assets` for people who download by hand, shown next to a failed download.
pub fn download_links(ui: &mut Ui, assets: &[&str]) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        ui.label("Download it yourself from");
        ui.hyperlink_to("the GitHub releases", RELEASES_PAGE).on_hover_text(RELEASES_PAGE);
        ui.label("or");
        ui.hyperlink_to("itch.io", ITCH_PAGE).on_hover_text(ITCH_PAGE);
        ui.label(format!("and unpack {} into the project folder.", assets.join(" and ")));
    });
}

/// A short warning for the status bar while the project's addon is missing or does not match the editor, with a tooltip.
/// Clicking it opens the addon part of the wizard.
pub fn status_warning(state: &EditorState) -> Option<(String, String)> {
    state.game.project_root.as_ref()?;
    let editor = crate::VERSION;
    match state.game.addon_version.as_deref() {
        None => Some(("No GodotTrench addon".into(), "Godot needs the GodotTrench addon to build maps and this project lacks it. Click to install it.".into())),
        Some(v) if addon_install::compare(v, editor) != Some(std::cmp::Ordering::Equal) => {
            Some((format!("Addon {v}, editor {editor}"), format!("The project's GodotTrench addon is {v} and this editor is {editor}. Click to update it.")))
        }
        Some(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use egui_kittest::kittest::{NodeT, Queryable};

    use super::*;
    use crate::content::test_server::{Server, zip};
    use crate::content_wizard::ContentWizard;
    use crate::state::{ContentRequest, Prefs};

    struct Fixture {
        state: EditorState,
        wizard: ContentWizard,
    }

    /// A project the editor has seen before, so only the addon brings up the wizard.
    fn known(name: &str, godot: &str, server: &Server) -> (Fixture, PathBuf) {
        let dir = std::env::temp_dir().join(format!("gt_addon_wizard_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("project.godot"), godot).unwrap();
        let root = crate::state::absolute(&dir);
        let mut prefs = Prefs::default();
        prefs.content_asked.insert(root.clone());
        let mut state = EditorState::new(prefs);
        state.load_project(&dir);
        let mut wizard = ContentWizard::default();
        wizard.addon.api = Some(server.url.clone());
        (Fixture { state, wizard }, root)
    }

    fn harness(fixture: Fixture) -> egui_kittest::Harness<'static, Fixture> {
        egui_kittest::Harness::builder().with_size(egui::vec2(900.0, 700.0)).build_ui_state(
            |ui, f: &mut Fixture| {
                f.wizard.take_request(&mut f.state);
                let ctx = ui.ctx().clone();
                f.wizard.show(&ctx, &mut f.state);
            },
            fixture,
        )
    }

    fn wait(harness: &mut egui_kittest::Harness<'static, Fixture>) {
        harness.step();
        for _ in 0..500 {
            if !harness.state().wizard.addon.running() {
                break;
            }

            std::thread::sleep(Duration::from_millis(10));
            harness.step();
        }

        harness.run();
    }

    fn release(server: &Server, version: &str) {
        let cfg = format!("[plugin]\nname=\"GodotTrench\"\nversion=\"{version}\"\n");
        server.release("beta", &[(addon_install::ADDON.asset, &zip(&[("addons/func_godot/plugin.cfg", cfg.as_bytes())]))], true);
    }

    #[test]
    fn a_missing_addon_is_installed_and_enabled_from_the_wizard() {
        let server = Server::start();
        release(&server, crate::VERSION);
        let godot = "config_version=5\r\n\r\n[application]\r\n\r\nconfig/name=\"Addon Test\"\r\n";
        let (fixture, root) = known("install", godot, &server);
        assert_eq!(fixture.state.content_request, Some(ContentRequest::Addon(root.clone())));
        let mut harness = harness(fixture);
        harness.run();
        assert!(harness.state().wizard.open && !harness.state().wizard.content, "only the addon part shows");
        assert!(harness.query_by_label_contains("Addon Test does not have it yet").is_some());
        assert!(harness.query_by_label("Ready made content").is_none());

        harness.get_by_label("Install the addon").click();
        wait(&mut harness);
        assert!(harness.query_by_label_contains("Installed the GodotTrench addon").is_some());
        assert_eq!(harness.state().state.game.addon_version.as_deref(), Some(crate::VERSION));
        assert!(harness.query_by_label_contains("installed but not enabled").is_some());

        harness.get_by_label("Enable the plugin").click();
        harness.run();
        assert!(harness.query_by_label("Enabled the plugin in project.godot.").is_some());
        assert!(harness.query_by_label_contains("Next, open the project in Godot once").is_some());
        let written = std::fs::read_to_string(root.join("project.godot")).unwrap();
        assert_eq!(written, format!("{godot}\r\n[editor_plugins]\r\n\r\nenabled=PackedStringArray(\"{}\")\r\n", addon_install::PLUGIN));

        harness.get_by_label("Close").click();
        harness.run();
        assert!(!harness.state().wizard.open);
        assert!(harness.state().state.prefs.addon_dismissed.is_empty(), "nothing is left to dismiss");
        assert_eq!(status_warning(&harness.state().state), None);
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn an_old_addon_waits_for_godot_to_close_and_a_failed_download_links_to_the_pages() {
        let server = Server::start();
        let (mut fixture, root) = known("update", "config_version=5\n", &server);
        std::fs::create_dir_all(root.join(addon_install::DIR)).unwrap();
        std::fs::write(root.join(addon_install::DIR).join("plugin.cfg"), "[plugin]\nversion=\"0.0.1\"\n").unwrap();
        fixture.state.load_project(&root);
        assert!(status_warning(&fixture.state).is_some_and(|(text, _)| text == format!("Addon 0.0.1, editor {}", crate::VERSION)));
        fixture.state.link_state.connected = true;
        fixture.state.link_state.project = Some(crate::live_link::path_key(&crate::live_link::godot_path(&root)));
        let update = format!("Update the addon to {}", crate::VERSION);
        let mut harness = harness(fixture);
        harness.run();
        assert!(harness.query_by_label_contains("older than this editor").is_some());
        assert!(harness.query_by_label_contains("Godot has this project open").is_some());
        assert!(harness.get_by_label(&update).accesskit_node().is_disabled(), "Godot would keep the old addon loaded");

        harness.state_mut().state.link_state.connected = false;
        server.release("beta", &[], true);
        harness.run();
        assert!(!harness.get_by_label(&update).accesskit_node().is_disabled());
        harness.get_by_label(&update).click();
        wait(&mut harness);
        assert!(harness.query_by_label_contains("is not published yet").is_some());
        assert!(harness.query_by_label("the GitHub releases").is_some() && harness.query_by_label("itch.io").is_some());
        assert_eq!(addon_install::Status::check(&root).installed, Installed::Older("0.0.1".into()));

        release(&server, crate::VERSION);
        harness.get_by_label(&update).click();
        wait(&mut harness);
        assert!(harness.query_by_label_contains("Updated the addon from 0.0.1").is_some());
        assert!(harness.query_by_label("Show in File Manager").is_some());
        assert!(harness.query_by_label("the GitHub releases").is_none());
        assert!(root.join(addon_install::BACKUPS).join("func_godot-0.0.1/plugin.cfg").is_file());
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_dismissed_addon_is_not_asked_about_again_and_git_checkouts_are_left_to_git() {
        let server = Server::start();
        let (fixture, root) = known("dismiss", "config_version=5\n", &server);
        let mut harness = harness(fixture);
        harness.run();
        harness.get_by_label("Close").click();
        harness.run();
        let key = addon_install::Status::check(&root).dismiss_key();
        assert_eq!(harness.state().state.prefs.addon_dismissed.get(&root), Some(&key));
        harness.state_mut().state.load_project(&root);
        harness.run();
        assert!(!harness.state().wizard.open, "a calm prompt, not a nag");

        let f = harness.state_mut();
        f.wizard.open_addon_from_menu(&mut f.state);
        harness.run();
        assert!(harness.state().wizard.open, "the Godot menu still brings it up");
        std::fs::create_dir_all(root.join(addon_install::DIR).join(".git")).unwrap();
        std::fs::write(root.join(addon_install::DIR).join("plugin.cfg"), "[plugin]\nversion=\"0.0.1\"\n").unwrap();
        harness.state_mut().wizard.addon.reset();
        harness.run();
        assert!(harness.query_by_label_contains("It is a git checkout").is_some());
        assert!(harness.query_by_label_contains("Update the addon").is_none(), "no button replaces a checkout");
        std::fs::remove_dir_all(&root).unwrap();
    }
}
