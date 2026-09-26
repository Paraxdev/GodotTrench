//! The setup wizard: sets up the GodotTrench addon and asks what ready made content a Godot project should get. It opens
//! by itself on the project's first start in GodotTrench, on the addon alone when a known project's addon is missing or
//! does not match the editor, and whenever Godot > Add Content to Project or Install or Update Addon opens it.

use std::path::{Path, PathBuf};

use egui::{RichText, Ui};

use crate::content::{Choice, Job, Outcome, Progress};
use crate::state::{ContentRequest, EditorState};
use crate::theme;

pub const TITLE: &str = "Set up this project";

/// MCP tools a person at the screen uses through the test harness: looking and clicking, which leave the wizard open.
const WATCHING_TOOLS: [&str; 3] = ["get_state", "screenshot", "simulate_input"];

#[derive(Default)]
pub struct ContentWizard {
    pub open: bool,
    /// Opened by itself on the project's first start.
    pub automatic: bool,
    /// An MCP client edits or opens things, so the wizard no longer opens by itself.
    pub driven: bool,
    pub choice: Choice,
    note: Option<String>,
    project: Option<PathBuf>,
    job: Option<Job>,
    outcome: Option<Outcome>,
    /// Where the releases are looked up, [`crate::content::api_url`] unless a test serves them.
    pub api: Option<String>,
    /// A preset asked for the nature pack this session, later ones only say so in the status bar.
    pack_offered: bool,
    /// The content choices show below the addon, else the wizard is only about the addon.
    pub content: bool,
    pub addon: crate::addon_wizard::AddonWizard,
}

impl ContentWizard {
    pub fn open_for(&mut self, root: PathBuf, choice: Choice, note: Option<String>, automatic: bool) {
        if self.job.is_none() {
            self.outcome = None;
            self.choice = choice;
        }

        self.note = note;
        self.project = Some(root);
        self.automatic = automatic;
        self.content = true;
        self.addon.reset();
        self.open = true;
    }

    /// Opens on the addon alone, without the content choices.
    pub fn open_addon(&mut self, root: PathBuf, automatic: bool) {
        if !self.open {
            self.content = false;
        }

        self.project = Some(root);
        self.automatic = automatic;
        self.addon.reset();
        self.open = true;
    }

    pub fn open_addon_from_menu(&mut self, state: &mut EditorState) {
        match state.game.project_root.clone() {
            Some(root) => self.open_addon(root, false),
            None => state.set_status("Open a Godot project first, the addon goes into it"),
        }
    }

    /// Opens on the choice that adds something the project lacks.
    pub fn open_from_menu(&mut self, state: &mut EditorState, choice: Option<Choice>) {
        let Some(root) = state.game.project_root.clone() else {
            state.set_status("Open a Godot project first, content is added to it");
            return;
        };
        let choice = choice.unwrap_or(if crate::content::has_nature_pack(&root) { Choice::Demo } else { Choice::Nature });
        self.open_for(root, choice, None, false);
    }

    /// Opens for what the editor asked: a project's first start or an addon that is missing or does not match, unless
    /// an agent drives the editor, or a preset that needs the nature pack.
    pub fn take_request(&mut self, state: &mut EditorState) {
        match state.content_request.take() {
            Some(ContentRequest::FirstStart(root)) if !self.driven && !self.open && state.game.project_root.as_ref() == Some(&root) => {
                self.open_for(root, Choice::Nothing, None, true);
            }
            Some(ContentRequest::Addon(root)) if !self.driven && !self.open && state.game.project_root.as_ref() == Some(&root) => {
                self.open_addon(root, true);
            }

            // An agent learns it from the status and the tool's error, a wizard would only be in its way.
            Some(ContentRequest::NaturePack(why)) if !self.driven && !self.pack_offered => {
                if let Some(root) = state.game.project_root.clone() {
                    self.pack_offered = true;
                    self.open_for(root, Choice::Nature, Some(why), false);
                }
            }
            _ => {}
        }
    }

    /// Every MCP tool call passes through here. One that edits or opens something means an agent is at work, which the
    /// wizard would only stand in the way of, so a wizard that opened by itself closes and stays closed.
    pub fn mcp_call(&mut self, tool: &str) {
        if WATCHING_TOOLS.contains(&tool) {
            return;
        }

        self.driven = true;
        if self.open && self.automatic && self.job.is_none() && !self.addon.running() {
            self.open = false;
        }
    }

    pub fn running(&self) -> bool {
        self.job.is_some()
    }

    pub fn progress(&self) -> Option<Progress> {
        self.job.as_ref().map(Job::progress)
    }

    /// Starts installing into the open project.
    pub fn start(&mut self, state: &EditorState, choice: Choice, repaint: Option<egui::Context>) -> Result<(), String> {
        if self.job.is_some() {
            return Err("content is already being added, wait for it or cancel it".into());
        }

        let root = state.game.project_root.clone().ok_or("open a Godot project first")?;
        self.choice = choice;
        self.outcome = None;
        self.project = Some(root.clone());
        let api = self.api.clone().unwrap_or_else(crate::content::api_url);
        self.job = Some(Job::start(root, choice, api, repaint));
        Ok(())
    }

    /// Checks on a running install, refreshing the model and material lists once it is done. Returns the outcome then.
    pub fn poll(&mut self, state: &mut EditorState) -> Option<Outcome> {
        let outcome = self.job.as_ref()?.poll()?;
        self.job = None;
        if !outcome.added.written.is_empty() {
            state.models.clear();
            let game = state.game.clone();
            state.model_library.rescan(&game);
            state.material_reload = true;
        }

        let summary = outcome.summary();
        if !summary.is_empty() {
            state.set_status(summary);
        }

        self.outcome = Some(outcome.clone());
        Some(outcome)
    }

    fn close(&mut self, state: &mut EditorState) {
        self.open = false;
        if let Some(root) = &self.project {
            if self.content {
                state.prefs.content_asked.insert(root.clone());
            }

            self.addon.dismiss(state, root);
        }
    }

    /// Draws the wizard while it is open and polls a running install either way. Returns where the wizard is and the
    /// outcome of an install that finished this frame.
    pub fn show(&mut self, ctx: &egui::Context, state: &mut EditorState) -> (Option<egui::Rect>, Option<Outcome>) {
        let finished = self.poll(state);
        self.addon.poll(state);
        if !self.open {
            return (None, finished);
        }

        let modal = egui::Modal::new(egui::Id::new("content_wizard")).show(ctx, |ui| {
            ui.set_width(540.0);
            ui.heading(TITLE);
            ui.add_space(4.0);
            if let Some(root) = self.project.clone() {
                self.addon.ui(ui, state, &root);
                ui.add_space(6.0);
                if !self.content {
                    if ui.button("Close").clicked() {
                        self.close(state);
                    }

                    return;
                }

                ui.separator();
                ui.label(RichText::new("Ready made content").strong());
            }

            match (&self.job, &self.project) {
                (Some(job), _) => self.running_ui(ui, job.progress()),
                (None, None) => {
                    ui.label("Open a Godot project first, the content is added to it.");
                    if ui.button("Close").clicked() {
                        self.open = false;
                    }
                }
                (None, Some(root)) => {
                    let root = root.clone();
                    if self.outcome.is_some() {
                        self.outcome_ui(ui, state);
                    } else {
                        self.choice_ui(ui, state, &root);
                    }
                }
            }
        });
        if modal.should_close() && self.job.is_none() && !self.addon.running() {
            self.close(state);
        }

        (Some(modal.response.rect), finished)
    }

    fn choice_ui(&mut self, ui: &mut Ui, state: &mut EditorState, root: &Path) {
        let name = crate::content::project_name(root);
        if self.automatic {
            ui.label(format!(
                "{name} is new to GodotTrench. Should it get ready made models, or the demo to learn from? You can add them later with Godot > Add Content to Project."
            ));
        } else {
            ui.label(format!("Add ready made models, or the demo to learn from, to {name}."));
        }

        if let Some(note) = &self.note {
            ui.label(RichText::new(note).color(theme::YELLOW));
        }

        ui.add_space(6.0);
        let has_pack = crate::content::has_nature_pack(root);
        let has_demo = crate::content::has_demo(root);
        for choice in Choice::ALL {
            let (title, text) = match choice {
                Choice::Nothing => ("Nothing extra".to_string(), "Only the built-in entities and materials. Nothing is written into the project."),
                Choice::Nature => (
                    format!("Nature models, about {} MB to download", choice.approx_mb()),
                    "Trees, bushes, rocks, grass and flowers with their textures, for the Scatter tool and the Models panel. They go into res://godottrench/nature.",
                ),
                Choice::Demo => (
                    format!("Nature models and the demo, about {} MB to download", choice.approx_mb()),
                    "Also the showcase maps, the demo models and textures, overlays and demo scenes, into res://demo and res://models. In Godot, res://demo/demo.tscn plays the showcase maps.",
                ),
            };
            let installed = match choice {
                Choice::Nothing => false,
                Choice::Nature => has_pack,
                Choice::Demo => has_pack && has_demo,
            };
            let title = if installed { format!("{title} (already in this project)") } else { title };
            ui.radio_value(&mut self.choice, choice, RichText::new(title).strong());
            ui.indent(choice.name(), |ui| ui.label(RichText::new(text).weak()));
            ui.add_space(2.0);
        }

        if self.choice == Choice::Demo && state.game.addon_version.is_none() {
            ui.label(RichText::new("The demo scenes need the GodotTrench addon, install it above.").color(theme::YELLOW));
        }

        ui.add_space(4.0);
        ui.label(RichText::new("Files the project already has are never replaced. project.godot, the addon and your own files stay as they are.").weak());
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            let go = match self.choice {
                Choice::Nothing => "Continue".to_string(),
                c => format!("Download and Add ({} MB)", c.approx_mb()),
            };
            if ui.button(RichText::new(go).strong()).clicked() {
                if self.choice == Choice::Nothing {
                    self.close(state);
                } else {
                    state.prefs.content_asked.insert(root.to_path_buf());
                    let _ = self.start(state, self.choice, Some(ui.ctx().clone()));
                }
            }

            if ui.button("Cancel").clicked() {
                self.close(state);
            }
        });
    }

    fn running_ui(&mut self, ui: &mut Ui, progress: Progress) {
        let cancelling = self.job.as_ref().is_some_and(Job::cancelling);
        let stage = match progress.stage.as_str() {
            _ if cancelling => "Cancelling…",
            "" => "Starting",
            stage => stage,
        };
        ui.label(stage);
        ui.add(egui::ProgressBar::new(progress.fraction()).text(progress.amount()).animate(progress.total == 0));
        ui.add_space(6.0);
        let cancel = ui.add_enabled(!cancelling, egui::Button::new("Cancel"));
        if cancel.on_hover_text("Stops after the file being added, the files added so far stay").clicked()
            && let Some(job) = &self.job
        {
            job.cancel();
        }
    }

    fn outcome_ui(&mut self, ui: &mut Ui, state: &mut EditorState) {
        let Some(outcome) = self.outcome.clone() else { return };
        let color = if outcome.error.is_some() { theme::WARNING } else { theme::SUCCESS };
        let summary = outcome.summary();
        ui.label(RichText::new(if summary.is_empty() { "Nothing to add.".to_string() } else { summary }).color(color));
        if outcome.error.as_ref().is_some_and(crate::content::Error::is_download) {
            let assets: Vec<&str> = outcome.choice.downloads().iter().map(|d| d.asset).collect();
            crate::addon_wizard::download_links(ui, &assets);
        }

        if outcome.error.is_none() && outcome.choice == Choice::Demo {
            ui.label("In Godot, open res://demo/demo.tscn and press F6 to walk through the showcase maps. File > Maps in Project opens them here.");
        }

        if outcome.error.is_none() && outcome.choice != Choice::Nothing {
            ui.label("The Scatter presets and the Models panel now find the nature models.");
        } else if outcome.tags.is_empty() && !outcome.added.written.is_empty() {
            ui.label("The Blockbench trees, rocks and grass built into the editor were added, so the rocks, boulders, grass, undergrowth and low-poly trees presets work. Try again for the rest.");
        }

        ui.add_space(6.0);
        ui.horizontal(|ui| {
            if outcome.error.is_some() && ui.button("Try Again").clicked() {
                let _ = self.start(state, outcome.choice, Some(ui.ctx().clone()));
            }

            if ui.button("Close").clicked() {
                self.close(state);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use egui_kittest::kittest::Queryable;

    use super::*;
    use crate::content::test_server::{Server, zip};
    use crate::state::Prefs;

    struct Fixture {
        state: EditorState,
        wizard: ContentWizard,
    }

    fn fresh(name: &str) -> (Fixture, PathBuf) {
        let dir = std::env::temp_dir().join(format!("gt_wizard_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("project.godot"), "config_version=5\n").unwrap();
        let mut state = EditorState::new(Prefs::default());
        state.load_project(&dir);
        let root = state.game.project_root.clone().unwrap();
        (Fixture { state, wizard: ContentWizard::default() }, root)
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

    fn listing(dir: &Path) -> Vec<String> {
        let mut out: Vec<String> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
        out.sort();
        out
    }

    #[test]
    fn a_new_project_is_asked_once_and_nothing_extra_writes_nothing() {
        let (fixture, root) = fresh("nothing");
        let mut harness = harness(fixture);
        harness.run();
        assert!(harness.state().wizard.open && harness.state().wizard.automatic);
        assert!(harness.query_by_label(TITLE).is_some());
        assert!(harness.query_by_label_contains("is new to GodotTrench").is_some());
        assert!(
            harness.query_by_label(crate::addon_wizard::HEADING).is_some() && harness.query_by_label("Install the addon").is_some(),
            "the addon comes first"
        );
        for label in ["Nothing extra", "Nature models, about 23 MB to download", "Nature models and the demo, about 130 MB to download"] {
            assert!(harness.query_by_label(label).is_some(), "{label}");
        }

        harness.get_by_label("Continue").click();
        harness.run();
        assert!(!harness.state().wizard.open);
        assert!(harness.state().state.prefs.content_asked.contains(&root), "the answer is remembered for this project");
        assert!(harness.state().state.prefs.addon_dismissed.contains_key(&root), "and so is letting the addon be");
        assert_eq!(listing(&root), ["project.godot"]);

        harness.state_mut().state.load_project(&root);
        harness.run();
        assert!(!harness.state().wizard.open, "only on the first start");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn an_agent_at_work_closes_a_wizard_that_opened_by_itself() {
        let (mut f, root) = fresh("mcp");
        f.wizard.take_request(&mut f.state);
        assert!(f.wizard.open);
        for watching in WATCHING_TOOLS {
            f.wizard.mcp_call(watching);
        }

        assert!(f.wizard.open, "a tester looking and clicking through MCP sees it like a person");
        f.wizard.mcp_call("create_brush");
        assert!(!f.wizard.open && f.wizard.driven);
        assert!(f.state.prefs.content_asked.is_empty(), "nobody answered it, so a later start without the agent asks again");

        f.state.content_request = Some(ContentRequest::FirstStart(root.clone()));
        f.wizard.take_request(&mut f.state);
        assert!(!f.wizard.open, "a project opened by the agent is not asked about");

        f.wizard.open_from_menu(&mut f.state, None);
        assert!(f.wizard.open && f.wizard.choice == Choice::Nature);
        f.wizard.mcp_call("create_brush");
        assert!(f.wizard.open, "a wizard a person opened stays");
        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_nature_choice_downloads_the_pack_into_the_project() {
        let server = Server::start();
        let pack = zip(&[
            ("godottrench/nature/trees/pack.json", b"{}"),
            ("godottrench/nature/trees_detailed/pack.json", b"{}"),
            ("godottrench/nature/bushes/pack.json", b"{}"),
            ("godottrench/nature/trees/oak.glb", include_bytes!("../../../godot/godottrench/nature/trees/oak.glb")),
            ("godottrench/nature/textures/leaf_oak.png", include_bytes!("../../../godot/godottrench/nature/textures/leaf_oak.png")),
            ("godottrench/nature/textures/bark_oak_albedo.jpg", include_bytes!("../../../godot/godottrench/nature/textures/bark_oak_albedo.jpg")),
            ("godottrench/nature/textures/bark_oak_normal.jpg", include_bytes!("../../../godot/godottrench/nature/textures/bark_oak_normal.jpg")),
        ]);
        server.release("beta", &[(crate::content::NATURE_PACK.asset, &pack)], true);
        let (mut fixture, root) = fresh("nature");
        fixture.wizard.api = Some(server.url.clone());
        let mut harness = harness(fixture);
        harness.run();
        harness.get_by_label("Nature models, about 23 MB to download").click();
        harness.run();
        harness.get_by_label("Download and Add (23 MB)").click();
        harness.step();
        assert!(harness.state().wizard.running());
        for _ in 0..500 {
            if !harness.state().wizard.running() {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(10));
            harness.step();
        }

        harness.run();
        assert!(harness.query_by_label_contains("Added 7 files to the project from the rolling beta.").is_some());
        assert!(crate::content::has_nature_pack(&root));
        let names: Vec<String> = harness.state().state.model_library.entries.iter().map(|e| e.name.clone()).collect();
        assert!(names.contains(&"nature/trees/oak".to_string()), "{names:?}");
        assert!(harness.state().state.status.starts_with("Added 7 files"), "{}", harness.state().state.status);
        harness.get_by_label("Close").click();
        harness.run();
        assert!(!harness.state().wizard.open);

        // A preset of the pack now works, and a reopened wizard says what the project has.
        assert!(crate::scatter_tool::new_set_from_preset(&mut harness.state_mut().state, "bushes").is_none(), "only some bushes are in this small pack");
        assert!(matches!(harness.state().state.content_request, Some(ContentRequest::NaturePack(_))));
        harness.run();
        assert!(harness.query_by_label("Nature models, about 23 MB to download (already in this project)").is_some());
        assert!(harness.query_by_label_contains("The bushes preset uses the glTF trees and bushes").is_some());
        harness.get_by_label("Cancel").click();
        harness.run();
        assert!(crate::scatter_tool::new_set_from_preset(&mut harness.state_mut().state, "bushes").is_none());
        harness.run();
        assert!(!harness.state().wizard.open, "the pack is offered once a session, after that the status bar says it");
        assert!(harness.state().state.status.contains("Add Content to Project"));
        std::fs::remove_dir_all(&root).unwrap();
    }
}
