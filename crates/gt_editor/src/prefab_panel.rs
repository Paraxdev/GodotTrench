//! The Prefabs panel: the prefab library as cards with a top-down preview. Click a card to paste its pieces at the
//! pointer, like Ctrl+V with something copied. The selection can be added to the project's library from here.

use egui::{Color32, Pos2, RichText, ScrollArea, Sense, Stroke, Ui, Vec2};

use crate::commands::Action;
use crate::prefab_library::{Entry, Source};
use crate::state::EditorState;

pub struct PrefabPanel {
    filter: String,
    category: Option<String>,
    size: f32,
    new_name: String,
    new_category: String,
    keep_position: bool,
}

impl Default for PrefabPanel {
    fn default() -> Self {
        Self { filter: String::new(), category: None, size: 96.0, new_name: String::new(), new_category: String::new(), keep_position: false }
    }
}

pub fn show(ui: &mut Ui, state: &mut EditorState, ps: &mut PrefabPanel, actions: &mut Vec<Action>) {
    ui.horizontal_wrapped(|ui| {
        ui.add(egui::TextEdit::singleline(&mut ps.filter).hint_text("Search prefabs").desired_width(140.0));
        egui::ComboBox::from_id_salt("prefab_category").selected_text(ps.category.clone().unwrap_or_else(|| "All categories".into())).show_ui(ui, |ui| {
            ui.selectable_value(&mut ps.category, None, "All categories");
            for c in state.prefab_library.categories() {
                ui.selectable_value(&mut ps.category, Some(c.clone()), c);
            }
        });
        ui.add(egui::Slider::new(&mut ps.size, 56.0..=180.0).show_value(false));
        if ui.small_button("⟳").on_hover_text("Rescan res://prefab_library").clicked() {
            let root = state.game.project_root.clone();
            state.prefab_library.rescan(root.as_deref());
        }
    });
    add_row(ui, state, ps);
    ui.separator();

    let filter = ps.filter.to_lowercase();
    let entries: Vec<Entry> = state
        .prefab_library
        .entries
        .iter()
        .filter(|e| filter.is_empty() || e.key().to_lowercase().contains(&filter))
        .filter(|e| ps.category.as_ref().is_none_or(|c| &e.category == c))
        .cloned()
        .collect();
    if entries.is_empty() {
        ui.label(RichText::new("No prefabs match the search").weak());
    }

    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            for entry in &entries {
                let resp = card(ui, entry, ps.size).on_hover_text(hover_text(entry));
                if resp.clicked() {
                    paste(state, entry, actions);
                }

                resp.context_menu(|ui| {
                    if ui.button("Paste").clicked() {
                        paste(state, entry, actions);
                        ui.close();
                    }

                    if ui.button("Copy").on_hover_text("Put the pieces on the clipboard to paste anywhere").clicked() {
                        match state.prefab_library.text(entry) {
                            Ok(text) => ui.ctx().copy_text(text),
                            Err(e) => state.set_status(format!("Could not read {}: {e}", entry.name)),
                        }

                        ui.close();
                    }

                    if let Some(path) = &entry.path {
                        if ui.button("Open to edit").clicked() {
                            actions.push(Action::OpenMapFile(path.clone()));
                            ui.close();
                        }

                        if ui.button("Show in File Manager").clicked() {
                            crate::panels::show_in_file_manager(path);
                            ui.close();
                        }
                    }
                });
            }
        });
    });
}

fn paste(state: &mut EditorState, entry: &Entry, actions: &mut Vec<Action>) {
    match state.prefab_library.text(entry) {
        Ok(text) => actions.push(Action::Paste(text)),
        Err(e) => state.set_status(format!("Could not read {}: {e}", entry.name)),
    }
}

fn add_row(ui: &mut Ui, state: &mut EditorState, ps: &mut PrefabPanel) {
    egui::CollapsingHeader::new("Add the selection").id_salt("prefab_add").show(ui, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::TextEdit::singleline(&mut ps.new_name).hint_text("Name").desired_width(120.0));
            ui.add(egui::TextEdit::singleline(&mut ps.new_category).hint_text("Category").desired_width(100.0));
            ui.checkbox(&mut ps.keep_position, "Keep position")
                .on_hover_text("Keeps the map origin as the piece's origin, for pieces built around it, like a kit for Random Fill");
            let roots = gt_doc::ops::selection_roots(&state.doc.map, &state.doc.selection);
            let root = state.game.project_root.clone();
            let ready = !roots.is_empty() && root.is_some() && !ps.new_name.trim().is_empty();
            let why = if root.is_none() {
                "Open a Godot project first, the entry is saved in its prefab_library folder"
            } else if roots.is_empty() {
                "Select the objects to add"
            } else {
                "Saves the selection as a .gtm in res://prefab_library, its bottom center on the origin"
            };
            if ui.add_enabled(ready, egui::Button::new("Add")).on_hover_text(why).on_disabled_hover_text(why).clicked()
                && let Some(root) = root
            {
                match state.prefab_library.add(&state.doc.map, &roots, &ps.new_category, &ps.new_name, &root, ps.keep_position) {
                    Ok(path) => {
                        state.set_status(format!("Added {} to the prefab library", path.display()));
                        ps.new_name.clear();
                    }
                    Err(e) => state.set_status(format!("Could not add the prefab: {e}")),
                }
            }
        });
    });
}

fn hover_text(entry: &Entry) -> String {
    let size = entry.bounds.size();
    let source = match entry.source {
        Source::BuiltIn => "Built in",
        Source::Project => "Project",
    };
    format!(
        "{}\n{} × {} × {} units, {source}\nClick to paste at the pointer, right click for more",
        entry.key(),
        size.x.round(),
        size.y.round(),
        size.z.round()
    )
}

/// A card: the entry's preview, faces shaded lighter the nearer they sit, point entities as dots.
fn card(ui: &mut Ui, entry: &Entry, size: f32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(size + 8.0, size + 24.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &entry.name));
    let painter = ui.painter_at(rect);
    let visuals = ui.visuals();
    let bg = if resp.hovered() { visuals.widgets.hovered.bg_fill } else { visuals.extreme_bg_color };
    painter.rect_filled(rect, 4.0, bg);
    let square = egui::Rect::from_min_size(rect.min + Vec2::splat(4.0), Vec2::splat(size));
    let preview = &entry.preview;
    let extent = (preview.max[0] - preview.min[0]).max(preview.max[1] - preview.min[1]).max(1.0);
    let scale = (size - 8.0) / extent;
    let center = Pos2::new((preview.min[0] + preview.max[0]) * 0.5, (preview.min[1] + preview.max[1]) * 0.5);
    let to_screen = |p: [f32; 2]| square.center() + Vec2::new(p[0] - center.x, p[1] - center.y) * scale;
    let (low, high) = (preview.depth[0], preview.depth[1].max(preview.depth[0] + 1.0));
    let accent = if entry.source == Source::Project { visuals.selection.bg_fill } else { Color32::from_gray(90) };
    for (points, height) in &entry.preview.faces {
        let t = ((height - low) / (high - low)).clamp(0.0, 1.0);
        let shade = (70.0 + 150.0 * t) as u8;
        let poly: Vec<Pos2> = points.iter().map(|p| to_screen(*p)).collect();
        painter.add(egui::Shape::convex_polygon(poly, Color32::from_gray(shade), Stroke::new(1.0, Color32::from_gray(shade / 2))));
    }

    for p in &entry.preview.points {
        painter.circle_filled(to_screen(*p), 2.5, Color32::from_rgb(230, 140, 40));
    }

    painter.rect_stroke(square, 2.0, Stroke::new(1.0, accent), egui::StrokeKind::Inside);
    painter.text(
        Pos2::new(rect.center().x, rect.max.y - 10.0),
        egui::Align2::CENTER_CENTER,
        &entry.name,
        egui::FontId::proportional(11.0),
        visuals.text_color(),
    );
    resp
}
