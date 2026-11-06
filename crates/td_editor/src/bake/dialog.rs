//! The Bake Lighting window: what gets baked, the quality and detail, and the progress of a running bake.

use td_bake::{Backend, Stage};

use super::{Job, Options, Quality, TEXEL_RANGE, collect};
use crate::i18n::{list, tr, tr_n, trf};
use crate::state::{EditorState, Shade};
use crate::{en, msg};

pub const TITLE: &str = en!("Bake Lighting");
pub const PROGRESS_TITLE: &str = en!("Baking Lighting");

#[derive(Default)]
pub struct BakeDialog {
    pub open: bool,
    options: Option<Options>,
    /// What the summary was made for and the summary.
    counts: Option<(u64, collect::Counts, bool)>,
    pub job: Option<Job>,
}

impl BakeDialog {
    pub fn open(&mut self, state: &mut EditorState) {
        self.open = true;
        self.options.get_or_insert_with(|| Options::of(&state.doc.map));
    }

    pub fn show(&mut self, ctx: &egui::Context, state: &mut EditorState) {
        self.show_progress(ctx, state);
        let mut open = self.open;
        let mut bake = false;
        crate::widgets::window(TITLE)
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            .show(ctx, |ui| bake = self.ui(ui, state));
        self.open = open && !bake;
        if bake {
            self.start(state);
        }
    }

    pub fn start(&mut self, state: &mut EditorState) {
        if self.job.is_some() {
            state.set_status(msg!("A bake is already running, wait for it or cancel it first"));
            return;
        }

        let options = *self.options.get_or_insert_with(|| Options::of(&state.doc.map));
        self.job = Some(Job::start(state, options));
        state.set_status(msg!("Baking lighting…"));
    }

    fn show_progress(&mut self, ctx: &egui::Context, state: &mut EditorState) {
        let Some(job) = &mut self.job else { return };
        if let Some(outcome) = job.finished() {
            let options = job.options;
            self.job = None;
            match outcome {
                Ok(baked) => {
                    super::apply(state, baked, options);
                    if state.prefs.shade != Shade::Baked {
                        state.prefs.shade = Shade::Baked;
                    }
                }
                Err(e) if e == td_bake::BakeError::Cancelled.to_string() => state.set_status(msg!("Bake cancelled, the map keeps its previous lighting")),
                Err(e) => state.set_status(msg!("Bake failed: {e}", e)),
            }

            return;
        }

        let progress = &job.progress;
        let cancelling = progress.is_cancelled();
        let elapsed = job.started.elapsed().as_secs();
        crate::widgets::window(PROGRESS_TITLE)
            .resizable(false)
            .collapsible(false)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            .show(ctx, |ui| {
                ui.label(stage_text(progress.stage()));
                ui.add(egui::ProgressBar::new(progress.fraction()).show_percentage().desired_width(300.0));
                let time = format!("{}:{:02}", elapsed / 60, elapsed % 60);
                ui.label(egui::RichText::new(trf!("{time} elapsed. You can keep editing, changes made now are not in this bake.", time)).weak());
                let label = if cancelling { tr("Cancelling…") } else { tr("Cancel") };
                if ui.add_enabled(!cancelling, egui::Button::new(label)).on_hover_text(tr("Stops the bake, the map keeps the lighting it has")).clicked() {
                    progress.cancel();
                }
            });
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }

    fn counts(&mut self, state: &EditorState) -> (collect::Counts, bool) {
        match self.counts {
            Some((rev, c, stale)) if rev == state.doc.revision => (c, stale),
            _ => {
                let softness = self.options.map_or(0.3, |o| o.softness);
                let c = collect::collect(&state.doc.map, &state.game, softness);
                let stale = state.doc.map.lightmap.as_ref().is_some_and(|lm| lm.scene != c.scene);
                self.counts = Some((state.doc.revision, c.counts, stale));
                (c.counts, stale)
            }
        }
    }

    /// The window's contents. True when Bake was clicked.
    pub fn ui(&mut self, ui: &mut egui::Ui, state: &mut EditorState) -> bool {
        ui.set_max_width(440.0);
        let (counts, stale) = self.counts(state);
        let o = self.options.get_or_insert_with(|| Options::of(&state.doc.map));
        ui.label(summary(&counts));
        ui.label(
            egui::RichText::new(tr(
                "Moving brush entities, triggers, tool textures and textures with Bake off are left out. A light's bake_mode key sets how it is baked.",
            ))
            .weak(),
        );
        if stale {
            ui.colored_label(ui.visuals().warn_fg_color, tr("The map changed since the last bake. Bake again to update the lighting."));
        }

        ui.add_space(6.0);
        egui::Grid::new("bake options").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
            ui.label(tr("Quality"));
            ui.horizontal(|ui| {
                for q in Quality::ALL {
                    ui.selectable_value(&mut o.quality, q, tr(q.label())).on_hover_text(tr(q.hint()));
                }
            });
            ui.end_row();

            ui.label(tr("Texel size")).on_hover_text(tr("Map units one light map texel covers"));
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut o.texel_size).range(TEXEL_RANGE).speed(0.25).suffix(" u"));
                let meters = o.texel_size / state.game.units_per_meter.max(1.0) as f32;
                let cm = format!("{:.0}", meters * 100.0);
                ui.label(egui::RichText::new(trf!("{cm} cm, smaller is sharper and slower", cm)).weak());
            });
            ui.end_row();

            ui.label(tr("Shadow softness"));
            ui.add(egui::Slider::new(&mut o.softness, 0.0..=1.0).fixed_decimals(2))
                .on_hover_text(tr("Spreads lights without a light_size of their own, so shadows get soft edges. 0 keeps them hard"));
            ui.end_row();

            ui.label(tr("Runs on"));
            ui.horizontal(|ui| {
                for b in Backend::ALL {
                    ui.radio_value(&mut o.backend, b, b.label()).on_hover_text(tr(backend_hint(b)));
                }
            });
            ui.end_row();
        });
        ui.add_space(6.0);
        let mut bake = false;
        ui.horizontal(|ui| {
            let nothing = counts.surfaces == 0;
            bake = ui.add_enabled(!nothing, egui::Button::new(tr("Bake"))).on_disabled_hover_text(tr("The map has no static geometry to bake")).clicked();
            if state.doc.map.lightmap.is_some()
                && ui.button(tr("Remove Bake")).on_hover_text(tr("Drops the baked lighting from the map, Godot then lights it in real time")).clicked()
            {
                state.doc.edit("Remove Baked Lighting", |m, _| m.lightmap = None);
                state.set_status(msg!("Removed the baked lighting"));
            }
        });
        bake
    }
}

fn backend_hint(b: Backend) -> &'static str {
    match b {
        Backend::Cpu => en!("Traces on every core. Works on every machine"),
        Backend::Gpu => en!("Traces on the graphics card, much faster on a dedicated one. Uses the CPU without a usable GPU"),
        Backend::Hybrid => en!("The graphics card and every core share the work. Fastest when both are strong"),
    }
}

fn stage_text(stage: Stage) -> String {
    match stage {
        Stage::Preparing => tr("Laying out the light map").into(),
        Stage::Direct => tr("Tracing direct light and shadows").into(),
        Stage::Bounce(0) => tr("Tracing sky light and ambient occlusion").into(),
        Stage::Bounce(k) => trf!("Tracing bounce {k}", k),
        Stage::Probes => tr("Tracing the light probes").into(),
        Stage::Finishing => tr("Smoothing and filling the gutters").into(),
    }
}

fn summary(c: &collect::Counts) -> String {
    let mut parts = Vec::new();
    match c.sun {
        Some(collect::BakeMode::Realtime) => {}
        Some(collect::BakeMode::Bounce) => parts.push(tr("the sun's bounce").to_string()),
        Some(_) => parts.push(tr("the sun").to_string()),
        None => {}
    }

    if c.baked_lights > 0 {
        parts.push(tr_n!(c.baked_lights, "{n} light", "{n} lights"));
    }

    if c.bounce_lights > 0 {
        parts.push(trf!("the bounce of {lights}", lights = tr_n!(c.bounce_lights, "{n} real time light", "{n} real time lights")));
    }

    parts.push(tr("the sky").to_string());
    parts.push(tr("glowing materials").to_string());
    let mut text = trf!("Bakes {lights} into {surfaces}.", lights = list(&parts), surfaces = tr_n!(c.surfaces, "{n} surface", "{n} surfaces"));
    if c.sun == Some(collect::BakeMode::Realtime) {
        text += " ";
        text += tr("The sun stays real time.");
    }

    if c.realtime_lights > 0 {
        text += " ";
        text += &tr_n!(c.realtime_lights, "{n} light stays real time.", "{n} lights stay real time.");
    }

    text
}
