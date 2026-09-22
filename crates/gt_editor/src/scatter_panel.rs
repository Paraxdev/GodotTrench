//! Scatter panel: the active scatter set as a palette of model cards, the surfaces it paints onto and the brush.

use egui::{Align, Color32, Layout, RichText, ScrollArea, Sense, Ui, Vec2};
use gt_core::NodeId;
use gt_doc::{ScatterItem, ScatterKind};
use gt_render::Renderer;

use crate::commands::Action;
use crate::panels::DndPayload;
use crate::scatter_tool as tool;
use crate::state::{EditorState, ScatterOutput};
use crate::tools::ToolKind;
use crate::{icons, theme};

const THUMB: f32 = 44.0;

/// `renderer` draws model thumbnails that are not cached yet, without one the cards show type icons until the
/// Models panel has rendered them.
pub fn scatter_panel(ui: &mut Ui, state: &mut EditorState, actions: &mut Vec<Action>, renderer: Option<&mut Renderer>) {
    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        set_row(ui, state, actions);
        ui.add_space(4.0);
        let active = tool::active_set(state);
        models_section(ui, state, active, renderer);
        ui.separator();
        targets_section(ui, state, actions, active);
        ui.separator();
        brush_section(ui, state, active);
        ui.separator();
        action_row(ui, state, actions, active);
    });
}

fn set_row(ui: &mut Ui, state: &mut EditorState, actions: &mut Vec<Action>) {
    let active = tool::active_set(state);
    let rename_id = ui.id().with("scatter_rename");
    let mut renaming: Option<String> = ui.data(|d| d.get_temp(rename_id));
    ui.horizontal_wrapped(|ui| {
        ui.label("Set");
        match (&mut renaming, active) {
            (Some(name), Some(id)) => {
                let edit = ui.add(egui::TextEdit::singleline(name).desired_width(150.0));
                edit.request_focus();
                if edit.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if !name.trim().is_empty() {
                        tool::rename_set(state, id, name.trim());
                    }

                    renaming = None;
                }
            }
            _ => {
                renaming = None;
                let sets: Vec<(NodeId, String)> = state.doc.map.scatters().map(|(id, s)| (id, format!("{} ({})", s.name, s.instances.len()))).collect();
                let current = active.and_then(|a| sets.iter().find(|(id, _)| *id == a)).map(|(_, n)| n.clone()).unwrap_or_else(|| "none".into());
                let width = (ui.available_width() - 240.0).clamp(80.0, 220.0);
                egui::ComboBox::from_id_salt("scatter_panel_set").selected_text(current).width(width).show_ui(ui, |ui| {
                    if sets.is_empty() {
                        ui.label(RichText::new("No sets in this map yet").weak());
                    }

                    for (id, name) in &sets {
                        if ui.selectable_label(active == Some(*id), name).clicked() {
                            state.active_scatter = Some(*id);
                        }
                    }
                });
            }
        }

        if icons::button(ui, icons::PLUS, 16.0, "New set", "New empty set on its own layer, drop models into it").clicked() {
            actions.push(Action::NewScatterSet);
        }

        ui.menu_button("Preset", |ui| {
            ui.label(RichText::new("New set from a built-in preset").weak());
            for preset in gt_doc::scatter::PRESETS {
                if ui.button(preset).clicked() {
                    actions.push(Action::ScatterPreset(preset.to_string()));
                    ui.close();
                }
            }
        })
        .response
        .on_hover_text("Start a new set holding one of the built-in nature presets");

        if let Some(id) = active {
            if renaming.is_none() && ui.small_button("Rename").clicked() {
                renaming = state.doc.map.scatter(id).map(|s| s.name.clone());
            }

            if icons::button(ui, icons::DELETE, 16.0, "Delete set", "Delete this set, its instances and its empty layer").clicked() {
                tool::delete_set(state, id);
            }
        }
    });
    ui.data_mut(|d| match renaming {
        Some(name) => {
            d.insert_temp(rename_id, name);
        }
        None => d.remove::<String>(rename_id),
    });
}

/// The file extension of a scatter source, lower case.
fn extension(source: &str) -> String {
    source.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default()
}

fn thumbnail(ui: &mut Ui, item: &ScatterItem, image: Option<egui::TextureId>, missing: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(THUMB), Sense::hover());
    ui.painter().rect_filled(rect, 3.0, theme::GRAY_2);
    match image {
        Some(id) => {
            let tint = if item.enabled { Color32::WHITE } else { theme::GRAY_4 };
            ui.painter().image(id, rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), tint);
        }
        None => {
            let (icon, tint) = if tool::is_classname(&item.source) {
                (icons::ENTITY, theme::MAGENTA)
            } else {
                (icons::MESH, crate::panels::model_ext_color(&extension(&item.source)))
            };
            let tint = if item.enabled { tint } else { theme::GRAY_4 };
            ui.put(rect.shrink(THUMB * 0.2), icon.image(THUMB * 0.6).tint(tint));
        }
    }

    if missing {
        ui.painter().rect_stroke(rect, 3.0, egui::Stroke::new(1.5, theme::ERROR), egui::StrokeKind::Inside);
    }

    resp
}

fn models_section(ui: &mut Ui, state: &mut EditorState, active: Option<NodeId>, renderer: Option<&mut Renderer>) {
    ui.horizontal(|ui| {
        ui.strong("Models");
        if let Some(set) = active.and_then(|id| state.doc.map.scatter(id)) {
            let on = set.items.iter().filter(|i| i.enabled).count();
            ui.label(RichText::new(format!("{on} of {} painted, {} placed", set.items.len(), set.instances.len())).weak());
        }
    });

    let frame = egui::Frame::new().inner_margin(4.0).corner_radius(4.0);
    let (_, dropped) = ui.dnd_drop_zone::<DndPayload, _>(frame, |ui| {
        ui.set_width(ui.available_width());
        match active.and_then(|id| state.doc.map.scatter(id).cloned().map(|s| (id, s))) {
            Some((id, set)) => item_cards(ui, state, id, set, renderer),
            None => {
                ui.add_space(12.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("No scatter set is active").strong());
                    ui.label(
                        RichText::new("Drop models here from the Models panel to start one, or use + and Preset above. Painting without a set starts one from the last preset.")
                            .weak(),
                    );
                });
                ui.add_space(12.0);
            }
        }

        ui.vertical_centered(|ui| ui.label(RichText::new("Drop models here from the Models panel").small().color(theme::GRAY_5)));
    });
    if let Some(payload) = dropped {
        drop_payload(state, &payload);
    }

    ui.horizontal_wrapped(|ui| {
        if ui.small_button("Add models…").on_hover_text("Pick .bbmodel, .glb, .gltf or .tscn files").clicked() {
            let mut dialog = rfd::FileDialog::new().add_filter("Models and scenes", &["bbmodel", "glb", "gltf", "tscn", "scn"]);
            if let Some(root) = &state.game.project_root {
                dialog = dialog.set_directory(root);
            }

            if let Some(paths) = dialog.pick_files() {
                let (_, added) = tool::add_model_files(state, &paths);
                state.set_status(format!("Added {added} models to the scatter set"));
            }
        }

        if ui.small_button("Add selected props").on_hover_text("The models (or classnames) of the selected entities").clicked() {
            let items: Vec<ScatterItem> = state
                .doc
                .selection
                .nodes
                .iter()
                .filter_map(|id| state.doc.map.entity(*id))
                .map(|e| ScatterItem::new(e.property("model").map(str::to_string).unwrap_or_else(|| e.classname.clone())))
                .collect();
            if items.is_empty() {
                state.set_status("Select prop entities to add their models");
            } else {
                let id = tool::active_set(state).unwrap_or_else(|| tool::new_empty_set(state));
                tool::add_items(state, id, items);
            }
        }
    });
}

/// Adds what was dropped onto the panel to the active set, starting an empty set when none is active.
pub fn drop_payload(state: &mut EditorState, payload: &DndPayload) {
    match payload {
        DndPayload::Model(path) => {
            let (id, added) = tool::add_model_files(state, std::slice::from_ref(path));
            let name = state.doc.map.scatter(id).map(|s| s.name.clone()).unwrap_or_default();
            let file = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            state.set_status(if added > 0 { format!("{file} added to scatter set '{name}'") } else { format!("{file} is painted by '{name}' again") });
        }
        DndPayload::Entities(classnames) => {
            let id = tool::active_set(state).unwrap_or_else(|| tool::new_empty_set(state));
            tool::add_items(state, id, classnames.iter().map(ScatterItem::new).collect());
        }
        DndPayload::Material(_) => state.set_status("Drop models here, materials go onto a model's card in its details"),
    }
}

fn item_cards(ui: &mut Ui, state: &mut EditorState, id: NodeId, set: gt_doc::Scatter, mut renderer: Option<&mut Renderer>) {
    if set.items.is_empty() {
        ui.add_space(8.0);
        ui.vertical_centered(|ui| ui.label(RichText::new(format!("'{}' has no models yet", set.name)).weak()));
        ui.add_space(8.0);
        return;
    }

    let counts = set.counts();
    let mut edited = set.clone();
    let mut changed = false;
    let mut remove = None;
    let current_material = state.current_material.clone();
    for (k, item) in edited.items.iter_mut().enumerate() {
        let resolved = crate::picking::item_model_path(&state.game, &item.source);
        let entry = resolved.as_ref().and_then(|p| state.model_library.entries.iter().find(|e| &e.path == p));
        let missing = !tool::is_classname(&item.source) && state.game.project_root.is_some() && resolved.as_ref().is_none_or(|p| !p.is_file());
        let upm = state.game.units_per_meter;
        let image =
            resolved.as_ref().filter(|_| !missing).and_then(|p| state.model_thumbs.thumbnail(ui.ctx(), renderer.as_deref_mut(), &mut state.models, upm, p));
        let open_id = ui.id().with(("scatter_item_open", id, &item.source));
        let mut open: bool = ui.data(|d| d.get_temp(open_id).unwrap_or(false));
        egui::Frame::new().fill(theme::GRAY_1).corner_radius(4.0).inner_margin(6.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let hover = if missing { format!("{} is missing, install the nature models or fix the path", item.source) } else { item.source.clone() };
                thumbnail(ui, item, image, missing).on_hover_text(hover);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let name = RichText::new(item.label());
                        ui.label(if item.enabled { name.strong() } else { name.color(theme::GRAY_5) });
                        if let Some(e) = entry {
                            let slug = e.source.short.as_deref().unwrap_or(e.source.name.as_str());
                            ui.label(RichText::new(crate::panels::truncate_slug(slug, 12).as_ref()).small().color(crate::panels::pack_color(&e.source.name)));
                        }

                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let arrow = if open { icons::COLLAPSE } else { icons::EXPAND };
                            if icons::small(ui, arrow, "Details", None).on_hover_text("Scale, spacing, alignment and material").clicked() {
                                open = !open;
                            }

                            changed |= ui
                                .checkbox(&mut item.enabled, "paint")
                                .on_hover_text("Off keeps the placed instances but the brush skips this model")
                                .changed();
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.spacing_mut().slider_width = 80.0;
                        ui.label(RichText::new("weight").color(theme::GRAY_5));
                        changed |=
                            ui.add(egui::Slider::new(&mut item.weight, 0.0..=10.0).max_decimals(1)).on_hover_text("Relative chance of this model").changed();
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(RichText::new(format!("{} placed", counts.get(k).copied().unwrap_or(0))).weak());
                        });
                    });
                });
            });
            if open {
                ui.add_space(4.0);
                egui::Grid::new(("scatter_item_details", id, k)).num_columns(2).spacing([10.0, 4.0]).show(ui, |ui| {
                    ui.label("Scale");
                    ui.horizontal(|ui| {
                        changed |= ui.add(egui::DragValue::new(&mut item.scale[0]).range(0.01..=50.0).speed(0.01)).changed();
                        ui.label("to");
                        changed |= ui.add(egui::DragValue::new(&mut item.scale[1]).range(0.01..=50.0).speed(0.01)).changed();
                    });
                    ui.end_row();
                    ui.label("Spacing");
                    changed |= ui
                        .add(egui::DragValue::new(&mut item.spacing).range(0.0..=4096.0).suffix(" u"))
                        .on_hover_text("Minimum distance to other instances")
                        .changed();
                    ui.end_row();
                    ui.label("Align");
                    changed |= ui.add(egui::Slider::new(&mut item.align, 0.0..=1.0)).on_hover_text("0 stands upright, 1 follows the surface").changed();
                    ui.end_row();
                    ui.label("Tilt");
                    changed |= ui.add(egui::DragValue::new(&mut item.tilt).range(0.0..=90.0).suffix("°")).on_hover_text("Largest random lean").changed();
                    ui.end_row();
                    ui.label("Sink");
                    changed |=
                        ui.add(egui::DragValue::new(&mut item.sink).range(-256.0..=256.0).suffix(" u")).on_hover_text("Pushed into the ground").changed();
                    ui.end_row();
                    ui.label("Random yaw");
                    changed |= ui.checkbox(&mut item.random_yaw, "").changed();
                    ui.end_row();
                    ui.label("Material");
                    ui.horizontal(|ui| {
                        let mut material = item.material.clone().unwrap_or_default();
                        if ui
                            .add(egui::TextEdit::singleline(&mut material).hint_text("set's or model's").desired_width(110.0))
                            .on_hover_text("Drawn instead of the model's own materials, empty falls back to the set's material")
                            .changed()
                        {
                            item.material = (!material.trim().is_empty()).then(|| material.trim().to_string());
                            changed = true;
                        }

                        if ui.small_button("use current").on_hover_text(current_material.clone()).clicked() {
                            item.material = Some(current_material.clone());
                            changed = true;
                        }
                    });
                    ui.end_row();
                });
                let n = counts.get(k).copied().unwrap_or(0);
                if ui.small_button(format!("Remove with {n} instances")).on_hover_text("Takes the model and everything it placed out of the set").clicked() {
                    remove = Some(k);
                }
            }
        });
        ui.data_mut(|d| d.insert_temp(open_id, open));
        ui.add_space(3.0);
    }

    if let Some(k) = remove {
        state.doc.edit("Remove Scatter Model", |m, _| {
            if let Some(s) = m.scatter_mut(id) {
                s.remove_item(k);
            }
        });
    } else if changed {
        state.doc.edit_coalesced("Edit Scatter Models", |m, _| {
            if let Some(slot) = m.scatter_mut(id) {
                slot.items = edited.items;
            }
        });
    }
}

fn targets_section(ui: &mut Ui, state: &mut EditorState, actions: &mut Vec<Action>, active: Option<NodeId>) {
    ui.horizontal(|ui| {
        ui.strong("Targets");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let picking = state.scatter_eyedropper;
            let resp = ui.add_enabled(active.is_some(), |ui: &mut Ui| {
                icons::toggle(
                    ui,
                    icons::EYEDROPPER,
                    16.0,
                    picking,
                    "Pick target",
                    "Click a surface or a scattered object in a view to add it as a target, or a target to remove it. Alt+click does the same.",
                )
            });
            if resp.clicked() {
                state.scatter_eyedropper = !picking;
                if state.scatter_eyedropper && state.tool != ToolKind::Scatter {
                    actions.push(Action::SetTool(ToolKind::Scatter));
                }
            }
        });
    });
    let Some(set) = active.and_then(|id| state.doc.map.scatter(id)).cloned() else {
        ui.label(RichText::new("Targets belong to a set, start or pick one first").weak());
        return;
    };
    let follow = !state.prefs.scatter.rules.only_targets;
    if set.targets.is_empty() {
        let hint =
            if follow { "None, the brush paints whatever is under it" } else { "None yet, the first stroke targets the surface or scattered set it starts on" };
        ui.label(RichText::new(hint).weak());
    }

    let mut drop = None;
    for t in &set.targets {
        ui.horizontal(|ui| {
            let icon = match state.doc.map.get(*t).map(|n| &n.kind) {
                Some(gt_doc::NodeKind::Scatter(_)) => icons::SCATTER,
                Some(gt_doc::NodeKind::Terrain(_)) => icons::TERRAIN,
                Some(gt_doc::NodeKind::Mesh(_)) => icons::MESH,
                _ => icons::BRUSH,
            };
            ui.add(icon.image(14.0).tint(theme::GRAY_6));
            ui.label(tool::target_label(state, *t));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icons::small(ui, icons::DELETE, "Remove target", None).on_hover_text("Stop painting on this").clicked() {
                    drop = Some(*t);
                }
            });
        });
    }

    if let Some(t) = drop {
        tool::toggle_target(state, t);
    }

    let mut follow_edit = follow;
    ui.checkbox(&mut follow_edit, "Follow cursor").on_hover_text(
        "Paint on any surface or scattered prop under the brush. Off keeps each stroke on the set's targets, and a set without targets takes the surface its first stroke starts on",
    );
    state.prefs.scatter.rules.only_targets = !follow_edit;
    if follow && !set.targets.is_empty() {
        ui.label(RichText::new("The targets above are ignored while following the cursor").small().color(theme::WARNING));
    }
}

fn brush_section(ui: &mut Ui, state: &mut EditorState, active: Option<NodeId>) {
    ui.strong("Brush");
    let s = &mut state.prefs.scatter;
    egui::Grid::new("scatter_panel_brush").num_columns(2).spacing([12.0, 5.0]).show(ui, |ui| {
        ui.label("Radius");
        ui.add(egui::DragValue::new(&mut s.radius).range(8.0..=16384.0).suffix(" u")).on_hover_text("Ctrl+wheel in a view");
        ui.end_row();
        ui.label("Density");
        ui.add(egui::DragValue::new(&mut s.rules.density).range(0.01..=64.0).speed(0.05))
            .on_hover_text("Attempts per 64 x 64 units, spacing limits how many fit");
        ui.end_row();
        ui.label("Slope");
        ui.horizontal(|ui| {
            ui.add(egui::DragValue::new(&mut s.rules.slope[0]).range(0.0..=90.0).suffix("°"));
            ui.label("to");
            ui.add(egui::DragValue::new(&mut s.rules.slope[1]).range(0.0..=90.0).suffix("°"));
        });
        ui.end_row();
        ui.label("Height");
        ui.horizontal(|ui| {
            let mut limited = s.rules.height.is_some();
            if ui.checkbox(&mut limited, "").on_hover_text("Only place between two world heights").changed() {
                s.rules.height = limited.then_some([-1024.0, 4096.0]);
            }

            match &mut s.rules.height {
                Some(h) => {
                    ui.add(egui::DragValue::new(&mut h[0]));
                    ui.label("to");
                    ui.add(egui::DragValue::new(&mut h[1]));
                }
                None => {
                    ui.label(RichText::new("any").weak());
                }
            }
        });
        ui.end_row();
        ui.label("Falloff");
        ui.add(egui::Slider::new(&mut s.rules.falloff, 0.0..=1.0)).on_hover_text("Thins instances out towards the rim");
        ui.end_row();
        ui.label("Erase amount");
        ui.add(egui::Slider::new(&mut s.erase_amount, 0.05..=1.0)).on_hover_text("Share of the instances under the brush one erase dab removes");
        ui.end_row();
        ui.label("Seed");
        ui.add(egui::DragValue::new(&mut s.seed).range(0..=u32::MAX as u64)).on_hover_text("0 gives a new pattern every stroke, any other value repeats it");
        ui.end_row();
        ui.label("Output");
        ui.horizontal(|ui| {
            ui.selectable_value(&mut s.output, ScatterOutput::Set, "scatter set");
            ui.selectable_value(&mut s.output, ScatterOutput::Entities, "entities");
        });
        ui.end_row();
        if s.output == ScatterOutput::Entities {
            ui.label("Prop class");
            ui.add(egui::TextEdit::singleline(&mut s.prop_class).desired_width(120.0))
                .on_hover_text("Entity class models are placed as, classnames stay themselves");
            ui.end_row();
        }
    });
    ui.checkbox(&mut s.avoid_other_sets, "Keep spacing to other sets").on_hover_text("Sets this one is painted onto do not count");
    ui.checkbox(&mut s.erase_palette_only, "Erase only the enabled models");

    if let Some(id) = active
        && let Some(set) = state.doc.map.scatter(id)
    {
        let mut kind = set.kind;
        ui.horizontal(|ui| {
            ui.label("Kind");
            for k in ScatterKind::ALL {
                ui.selectable_value(&mut kind, k, k.label())
                    .on_hover_text("Props keep their scenes and collision, foliage is drawn as MultiMesh without collision");
            }
        });
        if kind != set.kind {
            state.doc.edit("Scatter Kind", |m, _| {
                if let Some(s) = m.scatter_mut(id) {
                    s.kind = kind;
                    s.collision =
                        if kind == ScatterKind::Foliage { gt_doc::scatter::ScatterCollision::None } else { gt_doc::scatter::ScatterCollision::Convex };
                }
            });
        }
    }

    egui::CollapsingHeader::new("New set defaults").id_salt("scatter_new_defaults").show(ui, |ui| {
        let s = &mut state.prefs.scatter;
        egui::Grid::new("scatter_panel_defaults").num_columns(2).show(ui, |ui| {
            ui.label("Chunk size");
            ui.horizontal(|ui| {
                let mut chunked = s.chunk_size > 0.0;
                if ui.checkbox(&mut chunked, "").changed() {
                    s.chunk_size = if chunked { gt_doc::scatter::DEFAULT_CHUNK_SIZE } else { 0.0 };
                }

                if s.chunk_size > 0.0 {
                    ui.add(egui::DragValue::new(&mut s.chunk_size).range(64.0..=65536.0).suffix(" u"));
                }
            })
            .response
            .on_hover_text("Splits a set into cells, each its own MultiMesh so Godot culls the ones off screen");
            ui.end_row();
            ui.label("Visibility range");
            ui.horizontal(|ui| {
                let mut limited = s.visibility_range.is_some();
                if ui.checkbox(&mut limited, "").changed() {
                    s.visibility_range = limited.then_some(2400.0);
                }

                match &mut s.visibility_range {
                    Some(range) => {
                        ui.add(egui::DragValue::new(range).range(0.0..=100_000.0).suffix(" u"));
                    }
                    None => {
                        ui.label(RichText::new("from the kind").weak());
                    }
                }
            });
            ui.end_row();
        });
        ui.checkbox(&mut s.static_props_multimesh, "Draw prop scenes as MultiMesh")
            .on_hover_text("Much cheaper for many props, but scripts on those scenes are dropped");
        ui.label(RichText::new("Each set keeps its own values, edit them in the Inspector").weak());
    });
}

fn action_row(ui: &mut Ui, state: &mut EditorState, actions: &mut Vec<Action>, active: Option<NodeId>) {
    ui.horizontal_wrapped(|ui| {
        let tool_on = state.tool == ToolKind::Scatter;
        if ui.selectable_label(tool_on && !state.scatter_erase, "Paint").on_hover_text("Drag in a view paints, Shift erases").clicked() {
            state.scatter_erase = false;
            actions.push(Action::SetTool(ToolKind::Scatter));
        }

        if ui.selectable_label(tool_on && state.scatter_erase, "Erase").on_hover_text("Drag in a view erases, Shift paints").clicked() {
            state.scatter_erase = true;
            actions.push(Action::SetTool(ToolKind::Scatter));
        }

        ui.separator();
        if ui.button("Fill targets").on_hover_text("Cover the set's targets, or the selected surfaces, in one go").clicked() {
            actions.push(Action::ScatterFill);
        }

        ui.add_enabled_ui(active.is_some(), |ui| {
            if ui.button("Clear").on_hover_text("Remove every placed instance, keep the models").clicked()
                && let Some(id) = active
            {
                state.doc.edit("Clear Scatter", |m, _| {
                    if let Some(s) = m.scatter_mut(id) {
                        s.instances.clear();
                    }
                });
            }

            if ui.button("Bake to entities").on_hover_text("Replace the set with one prop entity per instance").clicked()
                && let Some(id) = active
            {
                let n = tool::bake_to_entities(state, id);
                state.set_status(format!("Baked {n} scatter instances into prop entities"));
            }
        });
    });
    if ui.small_button("Install nature models").on_hover_text(gt_doc::scatter::NATURE_DIR).clicked() {
        actions.push(Action::InstallNatureModels);
    }
}
