//! File > Export > glTF / OBJ: the options asked for before the save dialog.

use super::{Counts, Format, Options, Sources};
use crate::state::EditorState;

pub const TITLE: &str = "Export for 3D Tools";

/// Said in the dialog and the docs, so nobody expects a round trip.
pub const WHAT_IS_LEFT_OUT: &str = "Only geometry and materials are exported. Entity logic, I/O wiring, triggers, scripts and gameplay behaviour are left \
     out, and the file cannot be opened in GodotTrench as a map again.";

pub struct ExportDialog {
    pub open: bool,
    pub format: Format,
    pub options: Options,
    /// Counts for the current options, and the map revision, selection and options they were made for.
    counts: Option<(u64, usize, Options, Counts)>,
}

impl Default for ExportDialog {
    fn default() -> Self {
        Self { open: false, format: Format::Glb, options: Options::default(), counts: None }
    }
}

impl ExportDialog {
    pub fn open_for(&mut self, format: Format, state: &EditorState) {
        self.open = true;
        self.format = format;
        // A selection export only makes sense while something is selected.
        self.options.selection_only &= !state.doc.selection.nodes.is_empty();
    }

    pub fn show(&mut self, ctx: &egui::Context, state: &mut EditorState) {
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
        if export {
            run(state, self.format, &self.options);
        }
    }

    fn counts(&mut self, state: &EditorState) -> Counts {
        let key = (state.doc.revision, state.doc.selection.nodes.len(), self.options);
        match &self.counts {
            Some((r, s, o, c)) if (*r, *s, *o) == key => *c,
            _ => {
                let c = super::counts(&state.doc.map, &state.game, &state.doc.selection.nodes, &self.options);
                self.counts = Some((key.0, key.1, key.2, c));
                c
            }
        }
    }

    /// The dialog's contents. True when Export was clicked.
    pub fn ui(&mut self, ui: &mut egui::Ui, state: &EditorState) -> bool {
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
        ui.add_space(4.0);
        let mut export = false;
        ui.horizontal(|ui| {
            export = ui.button("Export…").clicked();
        });
        export
    }
}

/// Asks where to save, then exports and reports on the status line.
pub fn run(state: &mut EditorState, format: Format, options: &Options) {
    let ext = format.extension();
    let stem = state.doc.path.as_ref().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "map".into());
    let picked = crate::commands::file_dialog(state, |d| d.add_filter(format.label(), &[ext]).set_file_name(format!("{stem}.{ext}")).save_file());
    let Some(mut path) = picked else { return };
    if !path.extension().is_some_and(|e| e.eq_ignore_ascii_case(ext)) {
        path.set_extension(ext);
    }

    let result = super::export(Sources::of(state), &path, format, options);
    match result {
        Ok(report) => state.set_status(report.summary(&path)),
        Err(e) => state.set_status(format!("Export failed: {e}")),
    }
}
