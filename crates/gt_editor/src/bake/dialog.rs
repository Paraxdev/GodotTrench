//! The Bake Lighting window: what gets baked, the quality and detail, and the progress of a running bake.

use gt_bake::Backend;

use super::{Job, Options, Quality, TEXEL_RANGE, collect};
use crate::state::{EditorState, Shade};

pub const TITLE: &str = "Bake Lighting";
pub const PROGRESS_TITLE: &str = "Baking Lighting";

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
        egui::Window::new(TITLE)
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
            state.set_status("A bake is already running, wait for it or cancel it first");
            return;
        }

        let options = *self.options.get_or_insert_with(|| Options::of(&state.doc.map));
        self.job = Some(Job::start(state, options));
        state.set_status("Baking lighting…");
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
                Err(e) if e == gt_bake::BakeError::Cancelled.to_string() => state.set_status("Bake cancelled, the map keeps its previous lighting"),
                Err(e) => state.set_status(format!("Bake failed: {e}")),
            }

            return;
        }

        let progress = &job.progress;
        let cancelling = progress.is_cancelled();
        let elapsed = job.started.elapsed().as_secs();
        egui::Window::new(PROGRESS_TITLE).resizable(false).collapsible(false).pivot(egui::Align2::CENTER_CENTER).default_pos(ctx.content_rect().center()).show(
            ctx,
            |ui| {
                ui.label(progress.stage().label());
                ui.add(egui::ProgressBar::new(progress.fraction()).show_percentage().desired_width(300.0));
                ui.label(
                    egui::RichText::new(format!("{}:{:02} elapsed. You can keep editing, changes made now are not in this bake.", elapsed / 60, elapsed % 60))
                        .weak(),
                );
                let label = if cancelling { "Cancelling…" } else { "Cancel" };
                if ui.add_enabled(!cancelling, egui::Button::new(label)).on_hover_text("Stops the bake, the map keeps the lighting it has").clicked() {
                    progress.cancel();
                }
            },
        );
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
            egui::RichText::new(
                "Moving brush entities, triggers and tool textures are left out, and textures with Bake off in their settings. A light's \
                 bake_mode key picks how it is baked, auto bakes it unless a targetname or start_on 0 lets I/O switch it.",
            )
            .weak(),
        );
        if stale {
            ui.colored_label(ui.visuals().warn_fg_color, "The map changed since the last bake. Bake again to update the lighting.");
        }

        ui.add_space(6.0);
        egui::Grid::new("bake options").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
            ui.label("Quality");
            ui.horizontal(|ui| {
                for q in Quality::ALL {
                    ui.selectable_value(&mut o.quality, q, q.label()).on_hover_text(q.hint());
                }
            });
            ui.end_row();

            ui.label("Texel size").on_hover_text("Map units one light map texel covers");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut o.texel_size).range(TEXEL_RANGE).speed(0.25).suffix(" units"));
                let meters = o.texel_size / state.game.units_per_meter.max(1.0) as f32;
                ui.label(egui::RichText::new(format!("{:.0} cm, smaller is sharper and slower", meters * 100.0)).weak());
            });
            ui.end_row();

            ui.label("Shadow softness");
            ui.add(egui::Slider::new(&mut o.softness, 0.0..=1.0).fixed_decimals(2))
                .on_hover_text("Spreads lights without a light_size of their own, so shadows get soft edges. 0 keeps them hard");
            ui.end_row();

            ui.label("Runs on");
            ui.horizontal(|ui| {
                for b in Backend::ALL {
                    ui.radio_value(&mut o.backend, b, b.label()).on_hover_text(backend_hint(b));
                }
            });
            ui.end_row();
        });
        ui.add_space(6.0);
        let mut bake = false;
        ui.horizontal(|ui| {
            let nothing = counts.surfaces == 0;
            bake = ui.add_enabled(!nothing, egui::Button::new("Bake")).on_disabled_hover_text("The map has no static geometry to bake").clicked();
            if state.doc.map.lightmap.is_some()
                && ui.button("Remove Bake").on_hover_text("Drops the baked lighting from the map, Godot then lights it in real time").clicked()
            {
                state.doc.edit("Remove Baked Lighting", |m, _| m.lightmap = None);
                state.set_status("Removed the baked lighting");
            }
        });
        bake
    }
}

fn backend_hint(b: Backend) -> &'static str {
    match b {
        Backend::Cpu => "Traces on every core. Works on every machine",
        Backend::Gpu => "Traces on the graphics card, much faster on a dedicated one. Falls back to the CPU when there is no usable GPU",
        Backend::Hybrid => "The graphics card and every core share the work, each taking more while it keeps up. The fastest choice when both are strong",
    }
}

fn summary(c: &collect::Counts) -> String {
    let mut parts = Vec::new();
    match c.sun {
        Some(collect::BakeMode::Realtime) => {}
        Some(collect::BakeMode::Bounce) => parts.push("the sun's bounce".to_string()),
        Some(_) => parts.push("the sun".to_string()),
        None => {}
    }

    let plural = |n: usize, one: &str| if n == 1 { format!("1 {one}") } else { format!("{n} {one}s") };
    if c.baked_lights > 0 {
        parts.push(plural(c.baked_lights, "light"));
    }

    if c.bounce_lights > 0 {
        parts.push(format!("the bounce of {}", plural(c.bounce_lights, "real time light")));
    }

    let lights = if parts.is_empty() { "the sky and glowing materials".to_string() } else { format!("{}, the sky and glowing materials", parts.join(", ")) };
    let mut text = format!("Bakes {} into {}.", lights, plural(c.surfaces, "surface"));
    if c.sun == Some(collect::BakeMode::Realtime) {
        text += " The sun stays real time.";
    }

    match c.realtime_lights {
        0 => {}
        1 => text += " 1 light stays real time.",
        n => text += &format!(" {n} lights stay real time."),
    }

    text
}
