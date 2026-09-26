//! File > Export > glTF / OBJ: the options asked for before the save dialog, and the progress of a running export.

use std::collections::BTreeSet;
use std::path::PathBuf;

use gt_core::NodeId;

use super::{CANCELLED, Counts, Format, Job, OBJ_TRIANGLE_LIMIT, Options, obj_bytes, size_text};
use crate::state::EditorState;

pub const TITLE: &str = "Export for 3D Tools";
/// The window a running export shows its progress in.
pub const PROGRESS_TITLE: &str = "Exporting";

/// Said in the dialog and the docs, so nobody expects a round trip.
pub const WHAT_IS_LEFT_OUT: &str = "Only geometry and materials are exported. Entity logic, I/O wiring, triggers, scripts and gameplay behaviour are left \
     out, and the file cannot be opened in GodotTrench as a map again.";

/// Scatter instances past this many triangles make an OBJ big enough to say so before exporting.
const WARN_TRIANGLES: usize = 1_000_000;

/// What the dialog's counts were made for.
#[derive(Clone, PartialEq)]
struct CountsKey {
    revision: u64,
    /// The selection, for a selection export only.
    selection: Option<BTreeSet<NodeId>>,
    options: Options,
    models: u64,
}

pub struct ExportDialog {
    pub open: bool,
    pub format: Format,
    pub options: Options,
    counts: Option<(CountsKey, Counts)>,
    /// The export running now.
    pub job: Option<Job>,
}

impl Default for ExportDialog {
    fn default() -> Self {
        Self { open: false, format: Format::Glb, options: Options::default(), counts: None, job: None }
    }
}

fn triangles_text(n: usize) -> String {
    if n >= 1_000_000 { format!("{:.1} million", n as f64 / 1e6) } else { n.to_string() }
}

impl ExportDialog {
    pub fn open_for(&mut self, format: Format, state: &mut EditorState) {
        if self.job.is_some() {
            state.set_status("An export is still running, wait for it or cancel it first");
            return;
        }

        self.open = true;
        self.format = format;
        // A selection export only makes sense while something is selected.
        self.options.selection_only &= !state.doc.selection.nodes.is_empty();
    }

    pub fn show(&mut self, ctx: &egui::Context, state: &mut EditorState) {
        self.show_progress(ctx, state);
        let mut open = self.open;
        let mut export = false;
        egui::Window::new(TITLE)
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            .show(ctx, |ui| export = self.ui(ui, state));
        self.open = open && !export;
        if export && let Some(path) = save_path(state, self.format) {
            self.start(state, path);
        }
    }

    /// Starts exporting to `path` on its own thread. Unsaved work is autosaved first, in case the export runs the
    /// computer out of memory, which ends the whole program.
    pub fn start(&mut self, state: &mut EditorState, path: PathBuf) {
        state.autosave_now();
        match Job::start(state, path, self.format, self.options) {
            Ok(job) => {
                state.set_status(format!("Exporting to {}…", job.path.display()));
                self.job = Some(job);
            }
            Err(e) => state.set_status(format!("Export failed: {e}")),
        }
    }

    /// The progress window of a running export, and its outcome on the status line once it is done.
    fn show_progress(&mut self, ctx: &egui::Context, state: &mut EditorState) {
        let Some(job) = &mut self.job else { return };
        if let Some(outcome) = job.finished() {
            match outcome {
                Ok(report) => state.set_status(report.summary(&job.path)),
                Err(e) if e == CANCELLED => state.set_status(format!("Export cancelled, {} was not written", job.path.display())),
                Err(e) => state.set_status(format!("Export failed: {e}")),
            }

            self.job = None;
            return;
        }

        let (writing, done) = job.progress.get();
        let cancelling = job.progress.is_cancelled();
        let file = job.path.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
        let window = egui::Window::new(PROGRESS_TITLE).resizable(false).collapsible(false).pivot(egui::Align2::CENTER_CENTER);
        window.default_pos(ctx.content_rect().center()).show(ctx, |ui| {
            ui.label(if writing { format!("Writing {file}") } else { "Collecting geometry and textures".to_string() });
            ui.add(egui::ProgressBar::new(done).show_percentage().desired_width(300.0));
            ui.label(egui::RichText::new("You can keep editing. Changes made now do not go into the file.").weak());
            let label = if cancelling { "Cancelling…" } else { "Cancel" };
            if ui.add_enabled(!cancelling, egui::Button::new(label)).on_hover_text("Stops the export, nothing is written").clicked() {
                job.progress.cancel();
            }
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

    fn counts(&mut self, state: &mut EditorState) -> Counts {
        let key = CountsKey {
            revision: state.doc.revision,
            selection: self.options.selection_only.then(|| state.doc.selection.nodes.clone()),
            options: self.options,
            models: state.models.generation,
        };
        match &self.counts {
            Some((k, c)) if *k == key => *c,
            _ => {
                let c = super::counts(&state.doc.map, &state.game, &mut state.models, &state.doc.selection.nodes, &self.options);
                // Counting loads the scatter models, which bumps the generation once.
                self.counts = Some((CountsKey { models: state.models.generation, ..key }, c));
                c
            }
        }
    }

    /// The dialog's contents. True when Export was clicked.
    pub fn ui(&mut self, ui: &mut egui::Ui, state: &mut EditorState) -> bool {
        ui.set_max_width(420.0);
        ui.horizontal(|ui| {
            ui.label("Format");
            ui.selectable_value(&mut self.format, Format::Glb, Format::Glb.label())
                .on_hover_text("One file with the textures inside. Blender opens it with File > Import > glTF 2.0");
            ui.selectable_value(&mut self.format, Format::Obj, Format::Obj.label())
                .on_hover_text("For tools without glTF. Writes a .mtl file and a folder of textures next to it");
        });
        ui.add_space(4.0);
        ui.label(egui::RichText::new(WHAT_IS_LEFT_OUT).weak());
        ui.add_space(4.0);
        let has_selection = !state.doc.selection.nodes.is_empty();
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.options.selection_only, false, "Whole map");
            ui.add_enabled_ui(has_selection, |ui| ui.radio_value(&mut self.options.selection_only, true, "Selection only"))
                .response
                .on_disabled_hover_text("Select the objects to export first");
        });
        let c = self.counts(state);
        let o = &mut self.options;
        ui.checkbox(&mut o.hidden, "Hidden layers and objects");
        ui.checkbox(&mut o.models, format!("Prop models ({})", c.models))
            .on_hover_text("The models that props and other point entities show, placed where they stand");
        ui.checkbox(&mut o.scatter, format!("Scatter instances ({})", c.scatter))
            .on_hover_text("Every tree, rock and tuft of the scatter sets as its own object. Thousands of them make a big file that is slow to open");
        ui.checkbox(&mut o.markers, "Point entities as empty markers")
            .on_hover_text("An empty object where each point entity without a model stands, named like it, for lining things up");
        ui.checkbox(&mut o.merge_brushes, "Merge unnamed brushes")
            .on_hover_text("Brushes you have not renamed join into one object per layer, group or entity, which keeps the object list short");
        ui.add_space(4.0);
        let mut parts = vec![format!("{} brushes", c.brushes), format!("{} meshes", c.meshes)];
        if c.terrains > 0 {
            parts.push(format!("{} terrains", c.terrains));
        }

        ui.label(egui::RichText::new(format!("Exports {}. Tool faces and trigger volumes are skipped.", parts.join(", "))).weak());
        // OBJ has no instancing, so this is the part of an export that grows without bound.
        let obj_scatter = if self.format == Format::Obj && o.scatter { c.scatter_triangles } else { 0 };
        let too_big = obj_scatter > OBJ_TRIANGLE_LIMIT;
        if obj_scatter >= WARN_TRIANGLES {
            let size = size_text(obj_bytes(c.scatter_vertices, c.scatter_triangles));
            let mut text = format!(
                "OBJ writes every scatter instance as a full copy of its model: {} triangles, about {size}. glTF shares one mesh per model.",
                triangles_text(obj_scatter)
            );
            if too_big {
                text += &format!(" That is past the {} million triangles an OBJ export takes.", OBJ_TRIANGLE_LIMIT / 1_000_000);
            }

            let color = if too_big { ui.visuals().error_fg_color } else { ui.visuals().warn_fg_color };
            ui.colored_label(color, text);
        }

        ui.add_space(4.0);
        let mut export = false;
        ui.horizontal(|ui| {
            export =
                ui.add_enabled(!too_big, egui::Button::new("Export…")).on_disabled_hover_text("Switch to glTF, or leave out the scatter instances").clicked();
        });
        export
    }
}

/// Asks where to save, with the format's extension.
fn save_path(state: &EditorState, format: Format) -> Option<PathBuf> {
    let ext = format.extension();
    let stem = state.doc.path.as_ref().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "map".into());
    let mut path = crate::commands::file_dialog(state, |d| d.add_filter(format.label(), &[ext]).set_file_name(format!("{stem}.{ext}")).save_file())?;
    if !path.extension().is_some_and(|e| e.eq_ignore_ascii_case(ext)) {
        path.set_extension(ext);
    }

    Some(path)
}
