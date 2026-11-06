//! File > Export > glTF / OBJ: the options asked for before the save dialog, and the progress of a running export.

use std::collections::BTreeSet;
use std::path::PathBuf;

use td_core::NodeId;

use super::{CANCELLED, Counts, Format, Job, OBJ_TRIANGLE_LIMIT, Options, obj_bytes, size_text};
use crate::i18n::{list, tr, tr_n, trf};
use crate::state::EditorState;
use crate::{en, msg};

pub const TITLE: &str = en!("Export for 3D Tools");
/// The window a running export shows its progress in.
pub const PROGRESS_TITLE: &str = en!("Exporting");

/// Said in the dialog and the docs, so nobody expects a round trip.
pub const WHAT_IS_LEFT_OUT: &str = en!("Only geometry and materials are exported, not entity logic, I/O or scripts. The file cannot be opened as a map again.");

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
    if n >= 1_000_000 {
        let millions = format!("{:.1}", n as f64 / 1e6);
        trf!("{millions} million", millions)
    } else {
        n.to_string()
    }
}

impl ExportDialog {
    pub fn open_for(&mut self, format: Format, state: &mut EditorState) {
        if self.job.is_some() {
            state.set_status(msg!("An export is still running, wait for it or cancel it first"));
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
        crate::widgets::window(TITLE)
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
                state.set_status(msg!("Exporting to {path}…", path = job.path.display()));
                self.job = Some(job);
            }
            Err(e) => state.set_status(msg!("Export failed: {e}", e)),
        }
    }

    /// The progress window of a running export, and its outcome on the status line once it is done.
    fn show_progress(&mut self, ctx: &egui::Context, state: &mut EditorState) {
        let Some(job) = &mut self.job else { return };
        if let Some(outcome) = job.finished() {
            match outcome {
                Ok(report) => state.set_status(report.message(&job.path)),
                Err(e) if e == CANCELLED => state.set_status(msg!("Export cancelled, {path} was not written", path = job.path.display())),
                Err(e) => state.set_status(msg!("Export failed: {e}", e)),
            }

            self.job = None;
            return;
        }

        let (writing, done) = job.progress.get();
        let cancelling = job.progress.is_cancelled();
        let file = job.path.file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
        let window = crate::widgets::window(PROGRESS_TITLE).resizable(false).collapsible(false).pivot(egui::Align2::CENTER_CENTER);
        window.default_pos(ctx.content_rect().center()).show(ctx, |ui| {
            ui.label(if writing { trf!("Writing {file}", file) } else { tr("Collecting geometry and textures").to_string() });
            ui.add(egui::ProgressBar::new(done).show_percentage().desired_width(300.0));
            ui.label(egui::RichText::new(tr("You can keep editing. Changes made now do not go into the file.")).weak());
            let label = if cancelling { tr("Cancelling…") } else { tr("Cancel") };
            if ui.add_enabled(!cancelling, egui::Button::new(label)).on_hover_text(tr("Stops the export, nothing is written")).clicked() {
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
            ui.label(tr("Format"));
            ui.selectable_value(&mut self.format, Format::Glb, Format::Glb.label())
                .on_hover_text(tr("One file with the textures inside. Blender opens it with File > Import > glTF 2.0"));
            ui.selectable_value(&mut self.format, Format::Obj, Format::Obj.label())
                .on_hover_text(tr("For tools without glTF. Writes a .mtl file and a folder of textures next to it"));
        });
        ui.add_space(4.0);
        ui.label(egui::RichText::new(tr(WHAT_IS_LEFT_OUT)).weak());
        ui.add_space(4.0);
        let has_selection = !state.doc.selection.nodes.is_empty();
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.options.selection_only, false, tr("Whole map"));
            ui.add_enabled_ui(has_selection, |ui| ui.radio_value(&mut self.options.selection_only, true, tr("Selection only")))
                .response
                .on_disabled_hover_text(tr("Select the objects to export first"));
        });
        let c = self.counts(state);
        let o = &mut self.options;
        ui.checkbox(&mut o.hidden, tr("Hidden layers and objects"));
        ui.checkbox(&mut o.models, trf!("Prop models ({count})", count = c.models))
            .on_hover_text(tr("The models that props and other point entities show, placed where they stand"));
        ui.checkbox(&mut o.scatter, trf!("Scatter instances ({count})", count = c.scatter))
            .on_hover_text(tr("Exports every scattered prop as its own object. Thousands make a big file"));
        ui.checkbox(&mut o.markers, tr("Point entities as empty markers"))
            .on_hover_text(tr("An empty object where each point entity without a model stands, named like it, for lining things up"));
        ui.checkbox(&mut o.merge_brushes, tr("Merge unnamed brushes"))
            .on_hover_text(tr("Joins brushes you have not renamed into one object per layer, group or entity"));
        ui.add_space(4.0);
        let mut parts = vec![tr_n!(c.brushes, "{n} brush", "{n} brushes"), tr_n!(c.meshes, "{n} mesh", "{n} meshes")];
        if c.terrains > 0 {
            parts.push(tr_n!(c.terrains, "{n} terrain", "{n} terrains"));
        }

        ui.label(egui::RichText::new(trf!("Exports {what}. Tool faces and trigger volumes are skipped.", what = list(&parts))).weak());
        // OBJ has no instancing, so this is the part of an export that grows without bound.
        let obj_scatter = if self.format == Format::Obj && o.scatter { c.scatter_triangles } else { 0 };
        let too_big = obj_scatter > OBJ_TRIANGLE_LIMIT;
        if obj_scatter >= WARN_TRIANGLES {
            let size = size_text(obj_bytes(c.scatter_vertices, c.scatter_triangles));
            let triangles = triangles_text(obj_scatter);
            let mut text = trf!("OBJ writes every scatter instance as a full copy of its model: {triangles} triangles, about {size}.", triangles, size);
            if too_big {
                text += " ";
                text += &trf!("That is past the {limit} million triangles an OBJ export takes.", limit = OBJ_TRIANGLE_LIMIT / 1_000_000);
            }

            let color = if too_big { ui.visuals().error_fg_color } else { ui.visuals().warn_fg_color };
            ui.colored_label(color, text);
        }

        ui.add_space(4.0);
        let mut export = false;
        ui.horizontal(|ui| {
            export = ui
                .add_enabled(!too_big, egui::Button::new(tr("Export…")))
                .on_disabled_hover_text(tr("Switch to glTF, or leave out the scatter instances"))
                .clicked();
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
