use std::collections::HashSet;

use egui::{Color32, RichText, ScrollArea, Sense, Ui, Vec2};
use gt_core::{DVec2, DVec3, NodeId};
use gt_doc::{IoConnection, NodeKind};
use gt_formats::{EntityDef, PropertyType};
use gt_geom::FaceUv;
use gt_render::Renderer;

use crate::commands::Action;
use crate::icons;
use crate::state::EditorState;
pub use crate::uv_editor::uv_editor;
use crate::{theme, widgets};

#[derive(Clone, Debug)]
pub enum DndPayload {
    Material(String),
    Entities(Vec<String>),
    /// A model file dragged from the Models panel, placed as an editable mesh on drop.
    Model(std::path::PathBuf),
}

pub struct PanelState {
    expanded: HashSet<NodeId>,
    material_filter: String,
    material_folder: Option<String>,
    material_used_only: bool,
    material_favorites_only: bool,
    model_filter: String,
    model_folder: Option<String>,
    model_source_filter: Option<String>,
    model_sort: ModelSort,
    model_show_slugs: bool,
    thumb_size: f32,
    entity_filter: String,
    pub entity_selection: Vec<String>,
    entity_anchor: Option<String>,
    entity_card_size: f32,
    new_key: String,
    new_value: String,
    outliner_filter: String,
    outliner_offset: f32,
    issues: Vec<gt_doc::issues::Issue>,
    issues_revision: u64,
    pub uv: crate::uv_editor::UvEditorState,
    pub reference_class: String,
    reference_filter: String,
    reference_kind: usize,
    reference_list_width: Option<f32>,
    logic_start: Option<NodeId>,
    logic_output: String,
    logic_result: Option<crate::logic_sim::SimResult>,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            expanded: HashSet::new(),
            material_filter: String::new(),
            material_folder: None,
            material_used_only: false,
            material_favorites_only: false,
            model_filter: String::new(),
            model_folder: None,
            model_source_filter: None,
            model_sort: ModelSort::Folder,
            model_show_slugs: true,
            thumb_size: 72.0,
            entity_filter: String::new(),
            entity_selection: Vec::new(),
            entity_anchor: None,
            entity_card_size: 64.0,
            new_key: String::new(),
            new_value: String::new(),
            outliner_filter: String::new(),
            outliner_offset: 0.0,
            issues: Vec::new(),
            issues_revision: 0,
            uv: Default::default(),
            reference_class: String::new(),
            reference_filter: String::new(),
            reference_kind: 0,
            reference_list_width: None,
            logic_start: None,
            logic_output: String::new(),
            logic_result: None,
        }
    }
}

// ------------------------------------------------------------------ outliner

pub fn outliner(ui: &mut Ui, state: &mut EditorState, ps: &mut PanelState, actions: &mut Vec<Action>) {
    ui.horizontal(|ui| {
        let add_layer = egui::Button::image_and_text(icons::LAYER.image(icons::SMALL), "Add Layer").image_tint_follows_text_color(true);
        if ui.add(add_layer).on_hover_text("New layer, it becomes the current layer new objects go into").clicked() {
            actions.push(Action::AddLayer);
        }

        ui.add(egui::TextEdit::singleline(&mut ps.outliner_filter).hint_text("Filter").desired_width(f32::INFINITY));
    });
    ui.separator();

    let map = &state.doc.map;
    let reveal = state.outliner_reveal.take().filter(|id| map.contains(*id));
    if let Some(id) = reveal {
        ps.expanded.extend(map.ancestors(id));
    }

    let filter = ps.outliner_filter.to_lowercase();
    let mut rows: Vec<(usize, NodeId)> = Vec::new();
    for layer in &map.layers {
        rows.push((0, *layer));
        if ps.expanded.contains(layer) || !filter.is_empty() {
            push_rows(map, *layer, 1, &ps.expanded, &filter, &mut rows);
        }
    }

    if !filter.is_empty() && rows.iter().all(|(depth, _)| *depth == 0) {
        ui.label(RichText::new("No objects match the filter").weak());
    } else if map.layers.iter().all(|l| map.get(*l).is_none_or(|n| n.children.is_empty())) {
        ui.label(RichText::new("The map is empty. Drag in a 2D view to draw a brush, or drag an entity in from the Entities panel.").weak());
    }

    let row_h = 20.0;
    let mut toggles: Vec<(NodeId, u8)> = Vec::new();
    let mut clicked: Option<(NodeId, bool)> = None;
    let mut set_layer: Option<NodeId> = None;
    let mut rename: Option<(NodeId, String)> = None;
    let mut scroll = ScrollArea::vertical().auto_shrink([false, false]);
    if let Some(index) = reveal.and_then(|id| rows.iter().position(|(_, r)| *r == id)) {
        let step = row_h + ui.spacing().item_spacing.y;
        let (top, view) = (index as f32 * step, ui.available_height());
        if top < ps.outliner_offset || top + row_h > ps.outliner_offset + view {
            scroll = scroll.vertical_scroll_offset((top - (view - row_h) / 2.0).max(0.0));
        }
    }

    let output = scroll.show_rows(ui, row_h, rows.len(), |ui, range| {
        for (depth, id) in &rows[range] {
            let Some(node) = map.get(*id) else { continue };
            ui.horizontal(|ui| {
                ui.add_space(*depth as f32 * 14.0);
                let has_children = !node.children.is_empty();
                let expanded = ps.expanded.contains(id);
                if has_children {
                    let (icon, label) = if expanded { (icons::COLLAPSE, "Collapse") } else { (icons::EXPAND, "Expand") };
                    if icons::small(ui, icon, label, None).clicked() {
                        toggles.push((*id, 0));
                    }
                } else {
                    ui.add_space(icons::SMALL + ui.spacing().item_spacing.x);
                }

                let weak = ui.visuals().weak_text_color();
                let (eye, eye_tint) = if node.hidden { (icons::EYE_OFF, Some(weak)) } else { (icons::EYE, None) };
                if icons::small(ui, eye, "Visibility", eye_tint)
                    .on_hover_text(if node.hidden { "Hidden, click to show" } else { "Visible, click to hide" })
                    .clicked()
                {
                    toggles.push((*id, 1));
                }

                let (lock, lock_tint) = if node.locked { (icons::LOCK, theme::YELLOW) } else { (icons::UNLOCK, weak) };
                if icons::small(ui, lock, "Lock", Some(lock_tint))
                    .on_hover_text(if node.locked { "Locked, click to unlock" } else { "Unlocked, click to lock" })
                    .clicked()
                {
                    toggles.push((*id, 2));
                }

                let selected = state.doc.selection.nodes.contains(id);
                let (icon, color) = match &node.kind {
                    NodeKind::Layer(l) => (icons::LAYER, Color32::from_rgb((l.color.r * 255.0) as u8, (l.color.g * 255.0) as u8, (l.color.b * 255.0) as u8)),
                    NodeKind::Group(_) => (icons::GROUP, theme::CYAN),
                    NodeKind::Entity(e) => (
                        icons::ENTITY,
                        state
                            .game
                            .entity(&e.classname)
                            .map(|d| Color32::from_rgb((d.color.r * 255.0) as u8, (d.color.g * 255.0) as u8, (d.color.b * 255.0) as u8))
                            .unwrap_or(theme::GRAY_6),
                    ),
                    NodeKind::Brush(_) => (icons::BRUSH, theme::GRAY_5),
                    NodeKind::Instance(_) => (icons::INSTANCE, theme::YELLOW),
                    NodeKind::Mesh(_) => (icons::MESH, theme::PINK),
                    NodeKind::Terrain(_) => (icons::TERRAIN, theme::GREEN),
                    NodeKind::Scatter(_) => (icons::SCATTER, theme::TEAL),
                };
                ui.add(icon.image(icons::SMALL).tint(color));
                let mut label = node.name();
                if let NodeKind::Layer(_) = node.kind {
                    if state.current_layer == *id {
                        label = format!("{label}  (current)");
                    }

                    if state.doc.map.get(*id).is_some_and(|n| matches!(&n.kind, NodeKind::Layer(l) if l.omit_from_export)) {
                        label = format!("{label}  [omitted]");
                    }
                }

                let resp = ui.selectable_label(selected, label);
                if resp.clicked() {
                    // A layer click also makes it current; selecting it as well lets Delete and the other node ops act on it.
                    if matches!(node.kind, NodeKind::Layer(_)) {
                        set_layer = Some(*id);
                    }

                    clicked = Some((*id, ui.input(|i| i.modifiers.command)));
                }

                if resp.double_clicked() {
                    toggles.push((*id, 0));
                }

                if resp.secondary_clicked() && !selected && !matches!(node.kind, NodeKind::Layer(_)) {
                    clicked = Some((*id, false));
                }

                resp.context_menu(|ui| {
                    if let NodeKind::Layer(l) = &node.kind {
                        let mut name = l.name.clone();
                        if ui.text_edit_singleline(&mut name).changed() {
                            rename = Some((*id, name));
                        }

                        if ui.button("Set Current Layer").clicked() {
                            set_layer = Some(*id);
                            ui.close();
                        }

                        if ui.button("Toggle Omit From Export").clicked() {
                            toggles.push((*id, 3));
                            ui.close();
                        }

                        if ui.button("Move Selection Here").clicked() {
                            actions.push(Action::MoveToLayer(*id));
                            ui.close();
                        }

                        if ui.button("Delete Layer").clicked() {
                            toggles.push((*id, 4));
                            ui.close();
                        }
                    } else {
                        let editable_name = match &node.kind {
                            NodeKind::Group(g) => Some(g.name.clone()),
                            NodeKind::Scatter(s) => Some(s.name.clone()),
                            _ => None,
                        };
                        if let Some(mut name) = editable_name {
                            if ui.text_edit_singleline(&mut name).changed() {
                                rename = Some((*id, name));
                            }

                            ui.separator();
                        }

                        let mut item = |ui: &mut Ui, label: &str, action: Action| {
                            if ui.button(label).clicked() {
                                actions.push(action);
                                ui.close();
                            }
                        };
                        match &node.kind {
                            NodeKind::Group(_) => {
                                item(ui, "Open Group", Action::OpenGroup);
                                item(ui, "Ungroup", Action::Ungroup);
                                ui.separator();
                            }
                            NodeKind::Scatter(_) => {
                                item(ui, "Paint Into This Set", Action::ActivateScatter(*id));
                                ui.separator();
                            }
                            NodeKind::Entity(_) => {
                                item(ui, "Code Reference", Action::ShowReference);
                                ui.separator();
                            }
                            _ => {}
                        }

                        item(ui, "Focus", Action::FocusSelection);
                        if ui.button(if node.hidden { "Show" } else { "Hide" }).clicked() {
                            toggles.push((*id, 1));
                            ui.close();
                        }

                        if ui.button(if node.locked { "Unlock" } else { "Lock" }).clicked() {
                            toggles.push((*id, 2));
                            ui.close();
                        }

                        ui.separator();
                        item(ui, "Duplicate", Action::Duplicate);
                        ui.menu_button("Move to Layer", |ui| {
                            for layer in &map.layers {
                                if let Some(l) = map.get(*layer) {
                                    item(ui, &l.name(), Action::MoveToLayer(*layer));
                                }
                            }
                        });
                        item(ui, "Delete", Action::Delete);
                    }
                });
            });
        }
    });
    ps.outliner_offset = output.state.offset.y;

    for (id, kind) in toggles {
        match kind {
            0 => {
                if !ps.expanded.remove(&id) {
                    ps.expanded.insert(id);
                }
            }
            1 => state.doc.edit("Toggle Visibility", |m, _| {
                if let Some(n) = m.get_mut(id) {
                    n.hidden = !n.hidden;
                }
            }),
            2 => state.doc.edit("Toggle Lock", |m, _| {
                if let Some(n) = m.get_mut(id) {
                    n.locked = !n.locked;
                }
            }),
            3 => state.doc.edit("Toggle Omit Layer", |m, _| {
                if let Some(NodeKind::Layer(l)) = m.get_mut(id).map(|n| &mut n.kind) {
                    l.omit_from_export = !l.omit_from_export;
                }
            }),
            _ => {
                state.doc.edit("Delete Layer", |m, _| m.remove(id));
                if !state.doc.map.contains(state.current_layer) {
                    state.current_layer = state.doc.map.default_layer();
                }
            }
        }
    }

    if let Some((id, name)) = rename {
        state.doc.edit_coalesced("Rename", |m, _| match m.get_mut(id).map(|n| &mut n.kind) {
            Some(NodeKind::Layer(l)) => l.name = name,
            Some(NodeKind::Group(g)) => g.name = name,
            Some(NodeKind::Scatter(s)) => s.name = name,
            _ => {}
        });
    }

    if let Some(l) = set_layer {
        state.current_layer = l;
    }

    if let Some((id, toggle)) = clicked {
        state.last_bounds = state.doc.map.bounds(id);
        state.doc.select(|_, s| {
            if toggle {
                s.toggle_node(id);
            } else {
                s.clear();
                s.select_node(id);
            }
        });
    }
}

fn push_rows(map: &gt_doc::Map, id: NodeId, depth: usize, expanded: &HashSet<NodeId>, filter: &str, rows: &mut Vec<(usize, NodeId)>) {
    let Some(node) = map.get(id) else { return };
    for c in &node.children {
        let Some(child) = map.get(*c) else { continue };
        let matches = filter.is_empty() || child.name().to_lowercase().contains(filter);
        if matches {
            rows.push((depth, *c));
        }

        if expanded.contains(c) || (!filter.is_empty() && !child.children.is_empty()) {
            push_rows(map, *c, depth + 1, expanded, filter, rows);
        }
    }
}

// ----------------------------------------------------------------- inspector

pub fn inspector(ui: &mut Ui, state: &mut EditorState, ps: &mut PanelState, actions: &mut Vec<Action>) {
    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        if state.doc.selection.has_faces() {
            face_inspector(ui, state, actions);
            return;
        }

        let entities: Vec<NodeId> = state.doc.selection.nodes.iter().copied().filter(|id| state.doc.map.entity(*id).is_some()).collect();
        let single = (state.doc.selection.nodes.len() == 1).then(|| state.doc.selection.nodes.iter().next().copied()).flatten();
        if let Some(first) = entities.first().copied() {
            entity_inspector(ui, state, ps, first, actions);
        } else if let Some(id) = single.filter(|id| state.doc.map.terrain(*id).is_some()) {
            terrain_inspector(ui, state, id, actions);
        } else if let Some(id) = single.filter(|id| state.doc.map.mesh(*id).is_some()) {
            mesh_inspector(ui, state, id, actions);
        } else if let Some(id) = single.filter(|id| state.doc.map.scatter(*id).is_some()) {
            scatter_inspector(ui, state, id, actions);
        } else if !state.doc.selection.nodes.is_empty() {
            selection_summary(ui, state, actions);
        } else {
            worldspawn_inspector(ui, state, ps);
        }
    });
}

fn mesh_inspector(ui: &mut Ui, state: &mut EditorState, id: NodeId, actions: &mut Vec<Action>) {
    let Some(mesh) = state.doc.map.mesh(id).cloned() else { return };
    ui.heading(if mesh.decal { "Decal" } else { "Mesh" });
    ui.label(format!("{} vertices, {} faces, {} triangles", mesh.vertices.len(), mesh.faces.len(), mesh.triangle_count()));
    ui.label(if mesh.is_closed() { "closed" } else { "open surface" });
    let mut decal = mesh.decal;
    if ui
        .checkbox(&mut decal, "Decal (alpha cutout, double sided)")
        .on_hover_text("Draw this thin mesh with its texture's alpha cut out, laid over the surface behind it")
        .changed()
    {
        state.doc.edit("Toggle Decal", |m, _| {
            if let Some(mesh) = m.mesh_mut(id) {
                mesh.decal = decal;
            }
        });
    }

    if mesh.is_convex() {
        ui.label(RichText::new("convex, exports to .map as a brush").weak());
    }

    let bounds = mesh.bounds();
    let s = bounds.size();
    ui.label(format!("Size {:.1} x {:.1} x {:.1}", s.x, s.y, s.z));
    let mut angle = mesh.smooth_angle;
    ui.horizontal(|ui| {
        ui.label("Smoothing angle");
        if ui.add(egui::Slider::new(&mut angle, 0.0..=180.0).suffix("°")).changed() {
            state.doc.edit_coalesced("Smoothing Angle", |m, _| {
                if let Some(mesh) = m.mesh_mut(id) {
                    mesh.smooth_angle = angle;
                }
            });
        }
    });
    section(ui, "Mesh Operations", true, |ui| {
        ui.horizontal_wrapped(|ui| {
            for (label, action) in [
                ("Edit (Tab)", Action::EditMesh),
                ("To Brushes", Action::ConvertToBrushes),
                ("Subdivide", Action::MeshOp(crate::mesh_tool::MeshOp::Subdivide)),
                ("Solidify", Action::MeshOp(crate::mesh_tool::MeshOp::Solidify)),
                ("Weld", Action::MeshOp(crate::mesh_tool::MeshOp::MergeByDistance)),
                ("Flip Normals", Action::MeshOp(crate::mesh_tool::MeshOp::Flip)),
            ] {
                if ui.small_button(label).clicked() {
                    actions.push(action);
                }
            }
        });
    });
    brush_entity_section(ui, state, actions);
}

/// Collapsible inspector section with a strong title.
fn section<R>(ui: &mut Ui, title: &str, open: bool, add_contents: impl FnOnce(&mut Ui) -> R) {
    egui::CollapsingHeader::new(RichText::new(title).strong()).id_salt(title).default_open(open).show(ui, add_contents);
}

fn sub_heading(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).weak());
}

/// Brush entity classes grouped like the entity browser, collapsed by default since the list is long.
fn brush_entity_section(ui: &mut Ui, state: &EditorState, actions: &mut Vec<Action>) {
    let mut groups: Vec<(String, Vec<&EntityDef>)> = Vec::new();
    for def in state.game.solid_entities() {
        let group = if def.group.is_empty() { "other".to_string() } else { def.group.clone() };
        match groups.iter_mut().find(|(g, _)| *g == group) {
            Some((_, list)) => list.push(def),
            None => groups.push((group, vec![def])),
        }
    }

    let title = format!("Make Brush Entity ({})", groups.iter().map(|(_, d)| d.len()).sum::<usize>());
    egui::CollapsingHeader::new(RichText::new(title).strong()).id_salt("brush_entity_section").default_open(false).show(ui, |ui| {
        if groups.is_empty() {
            sub_heading(ui, "No brush entities, open a Godot project with a game config");
        }

        for (group, defs) in groups {
            sub_heading(ui, &group);
            ui.horizontal_wrapped(|ui| {
                for def in defs {
                    if ui.small_button(&def.classname).on_hover_text(&def.description).clicked() {
                        actions.push(Action::CreateBrushEntity(def.classname.clone()));
                    }
                }
            });
        }
    });
}

fn scatter_inspector(ui: &mut Ui, state: &mut EditorState, id: NodeId, actions: &mut Vec<Action>) {
    use gt_doc::scatter::ScatterCollision;
    let Some(set) = state.doc.map.scatter(id).cloned() else { return };
    ui.heading(format!("Scatter set '{}'", set.name));
    let active = state.active_scatter == Some(id);
    let enabled = set.items.iter().filter(|i| i.enabled).count();
    ui.label(format!(
        "{} instances of {} models ({enabled} painted), {} targets{}",
        set.instances.len(),
        set.items.len(),
        set.targets.len(),
        if active { ", painting here" } else { "" }
    ));
    let mut edited = set.clone();
    let mut changed = false;
    egui::Grid::new("scatter_props").num_columns(2).show(ui, |ui| {
        ui.label("Name");
        changed |= ui.text_edit_singleline(&mut edited.name).changed();
        ui.end_row();
        ui.label("Kind");
        ui.horizontal(|ui| {
            for k in gt_doc::ScatterKind::ALL {
                changed |=
                    ui.selectable_value(&mut edited.kind, k, k.label()).on_hover_text("props keep scenes and collision, foliage uses MultiMesh").changed();
            }
        });
        ui.end_row();
        ui.label("Collision");
        ui.horizontal(|ui| {
            for c in ScatterCollision::ALL {
                changed |= ui.selectable_value(&mut edited.collision, c, c.label()).changed();
            }
        });
        ui.end_row();
        ui.label("Cast shadows");
        changed |= ui.checkbox(&mut edited.cast_shadows, "").changed();
        ui.end_row();
        ui.label("Visibility range");
        changed |= ui
            .add(egui::DragValue::new(&mut edited.visibility_range).range(0.0..=100_000.0).suffix(" u"))
            .on_hover_text("0 shows instances at any distance")
            .changed();
        ui.end_row();
        ui.label("Chunk size");
        ui.horizontal(|ui| {
            let mut chunked = edited.chunk_size > 0.0;
            if ui.checkbox(&mut chunked, "").changed() {
                edited.chunk_size = if chunked { gt_doc::scatter::DEFAULT_CHUNK_SIZE } else { 0.0 };
                changed = true;
            }

            if edited.chunk_size > 0.0 {
                changed |= ui.add(egui::DragValue::new(&mut edited.chunk_size).range(64.0..=65536.0).suffix(" u")).changed();
            }
        })
        .response
        .on_hover_text(format!("{} chunks, each culled on its own in Godot", set.chunks().len()));
        ui.end_row();
        ui.label("Props as MultiMesh");
        changed |=
            ui.checkbox(&mut edited.static_props_multimesh, "").on_hover_text("Cheaper for many props, but scripts on those prop scenes are dropped").changed();
        ui.end_row();
        ui.label("Material");
        ui.horizontal(|ui| {
            let mut material = edited.material.clone().unwrap_or_default();
            if ui
                .add(egui::TextEdit::singleline(&mut material).hint_text("model's own").desired_width(120.0))
                .on_hover_text("Drawn instead of the materials the models ship with. A palette entry's own material wins over it")
                .changed()
            {
                edited.material = (!material.trim().is_empty()).then(|| material.trim().to_string());
                changed = true;
            }

            if ui.small_button("use current").on_hover_text(state.current_material.clone()).clicked() {
                edited.material = Some(state.current_material.clone());
                changed = true;
            }

            if edited.material.is_some() && ui.small_button("×").on_hover_text("Back to the models' own materials").clicked() {
                edited.material = None;
                changed = true;
            }
        });
        ui.end_row();
    });
    if changed {
        state.doc.edit_coalesced("Edit Scatter Set", |m, _| {
            if let Some(slot) = m.scatter_mut(id) {
                *slot = edited;
            }
        });
    }

    ui.horizontal_wrapped(|ui| {
        if ui.button("Edit in Scatter Panel").on_hover_text("Models, targets and brush of this set, ready to paint").clicked() {
            actions.push(Action::ActivateScatter(id));
            actions.push(Action::ShowScatterPanel);
        }

        for (label, action) in [("Fill Targets", Action::ScatterFill), ("To Entities", Action::ScatterToEntities)] {
            if ui.small_button(label).clicked() {
                if matches!(action, Action::ScatterFill) {
                    actions.push(Action::ActivateScatter(id));
                }

                actions.push(action);
            }
        }

        if ui.small_button("Clear Instances").clicked() {
            state.doc.edit("Clear Scatter", |m, _| {
                if let Some(s) = m.scatter_mut(id) {
                    s.instances.clear();
                }
            });
        }
    });
    if !set.targets.is_empty() {
        let names: Vec<String> = set.targets.iter().map(|t| crate::scatter_tool::target_label(state, *t)).collect();
        ui.label(RichText::new(format!("Targets: {}", names.join(", "))).weak());
    }
}

fn terrain_inspector(ui: &mut Ui, state: &mut EditorState, id: NodeId, actions: &mut Vec<Action>) {
    let Some(t) = state.doc.map.terrain(id).cloned() else { return };
    ui.heading("Terrain");
    let size = t.size();
    let upm = state.game.units_per_meter;
    ui.label(format!("{} x {} vertices, cell {} units", t.resolution[0], t.resolution[1], t.cell_size));
    ui.label(format!("{} x {} units ({:.0} x {:.0} m), {} chunks", size.x, size.y, size.x / upm, size.y / upm, t.chunks().len()));
    let b = t.bounds();
    ui.label(format!("Heights {:.1} to {:.1}", b.min.y, b.max.y));
    let mut edited = t.clone();
    let mut changed = false;
    let mut origin = edited.origin.to_array();
    widgets::row(ui, 0, "Origin", "", widgets::label_width(ui), |ui| {
        if widgets::vector_input(ui, &mut origin, ui.available_width() - widgets::TRAILING, 1.0) {
            edited.origin = DVec3::from_array(origin);
            changed = true;
        }
    });
    egui::Grid::new("terrain_props").num_columns(2).show(ui, |ui| {
        ui.label("Chunk cells");
        changed |= ui.add(egui::DragValue::new(&mut edited.chunk_cells).range(4..=256)).changed();
        ui.end_row();
        for (i, layer) in edited.layers.iter_mut().enumerate() {
            ui.label(format!("Layer {i}"));
            ui.horizontal(|ui| {
                changed |= ui.add(egui::TextEdit::singleline(&mut layer.material).desired_width(130.0)).changed();
                changed |= ui.add(egui::DragValue::new(&mut layer.tile).range(8.0..=16384.0).prefix("tile ")).changed();
                changed |= ui
                    .add(egui::DragValue::new(&mut layer.detile).range(0.0..=1.0).speed(0.02).prefix("detile "))
                    .on_hover_text("Turns and shifts every tile by a fixed random amount and blends the joins, so the repeat stops showing. Good for paths, costs four texture reads per projection")
                    .changed();
                if layer.detile > 0.0 {
                    changed |= ui
                        .add(egui::DragValue::new(&mut layer.detile_sharpen).range(0.0..=1.0).speed(0.02).prefix("sharpen "))
                        .on_hover_text("Keeps the de-tiled texture crisp: 0 mixes the tiles evenly and looks soft, 1 mixes only where they join")
                        .changed();
                }
            });
            ui.end_row();
        }
    });
    ui.horizontal_wrapped(|ui| {
        if edited.layers.len() < gt_geom::heightfield::MAX_LAYERS && ui.small_button("+ layer (current material)").clicked() {
            let last = edited.layers.last();
            let (tile, detile) = last.map(|l| (l.tile, l.detile)).unwrap_or((256.0, 0.0));
            edited.layers.push(gt_geom::TerrainLayer { detile, ..gt_geom::TerrainLayer::new(state.current_material.clone(), tile) });
            changed = true;
        }

        if edited.layers.len() > 1 && ui.small_button("- last layer").clicked() {
            edited.layers.pop();
            changed = true;
        }

        let longest = t.resolution[0].max(t.resolution[1]);
        for res in [65u32, 129, 257, 513] {
            let tip = format!("At most {res} vertices along the longer side, cells stay square so the other side keeps its extent");
            if res != longest && ui.small_button(format!("resample {res}")).on_hover_text(tip).clicked() {
                edited = t.resample([res, res]);
                changed = true;
            }
        }
    });
    if changed {
        state.doc.edit_coalesced("Edit Terrain", |m, _| {
            if let Some(slot) = m.terrain_mut(id) {
                *slot = edited;
            }
        });
    }

    ui.separator();
    ui.horizontal_wrapped(|ui| {
        for (label, action) in [
            ("Sculpt", Action::SetTool(crate::tools::ToolKind::Sculpt)),
            ("Auto Paint", Action::TerrainAutoPaint),
            ("Flatten", Action::TerrainFlatten),
            ("Scatter", Action::SetTool(crate::tools::ToolKind::Scatter)),
            ("Blend", Action::SetTool(crate::tools::ToolKind::Blend)),
        ] {
            if ui.small_button(label).clicked() {
                actions.push(action);
            }
        }
    });
    crate::dialogs::auto_paint_base_ui(ui, &mut state.auto_paint_base);
    let hint = if state.tool == crate::tools::ToolKind::Blend {
        format!("Drop a material from the browser on the terrain to put it in layer {}, the layer the Blend tool paints.", state.blend.layer)
    } else {
        format!(
            "Drop a material from the browser on the terrain to put it in layer {}, the layer the Sculpt tool's PaintLayer mode paints. With the Blend tool active it goes to the Blend layer.",
            state.sculpt.layer
        )
    };
    ui.label(RichText::new(hint).weak());
}

fn selection_summary(ui: &mut Ui, state: &mut EditorState, actions: &mut Vec<Action>) {
    use crate::entity_wizards::{DoorKind, HingeSide, SlideDirection};
    let map = &state.doc.map;
    let brushes = state.doc.selection.brushes(map);
    let meshes = state.doc.selection.meshes(map);
    let entity_count = state.doc.selection.nodes.iter().filter(|id| map.entity(**id).is_some()).count();
    let bounds = map.bounds_of(state.doc.selection.nodes.iter().copied());
    ui.heading(format!("{} objects", state.doc.selection.nodes.len()));
    ui.label(format!("{} brushes, {} meshes", brushes.len(), meshes.len()));
    if !bounds.is_empty() {
        let (s, c) = (bounds.size(), bounds.center());
        ui.label(RichText::new(format!("Size {} x {} x {}, center {:.2} {:.2} {:.2}", s.x, s.y, s.z, c.x, c.y, c.z)).weak());
    }

    let buttons = |ui: &mut Ui, actions: &mut Vec<Action>, list: Vec<(&str, &str, Action)>| {
        ui.horizontal_wrapped(|ui| {
            for (label, tip, action) in list {
                if ui.small_button(label).on_hover_text(tip).clicked() {
                    actions.push(action);
                }
            }
        });
    };
    section(ui, "Selection", true, |ui| {
        buttons(
            ui,
            actions,
            vec![
                ("Edit Mesh", "Convert brushes to a mesh and edit it Blender style (Tab)", Action::EditMesh),
                ("To Mesh", "Convert the selected brushes to meshes", Action::ConvertToMesh),
                ("Join Meshes", "Join the selected meshes and brushes into one mesh", Action::JoinMeshes),
                ("Duplicate Linked", "Linked copies update together when one is edited", Action::DuplicateLinked),
                ("Cordon", "Limit the views and export to the selection bounds", Action::SetCordonFromSelection),
                ("Blend material", "Current material blends into these faces", Action::SetBlendMaterial),
            ],
        );
    });
    section(ui, "Transform", true, |ui| {
        buttons(
            ui,
            actions,
            vec![
                ("Rotate Y 90", "Rotate around the vertical axis", Action::Rotate { axis: 1, degrees: 90.0 }),
                ("Rotate Y -90", "Rotate around the vertical axis", Action::Rotate { axis: 1, degrees: -90.0 }),
                ("Flip X", "Mirror along X", Action::Flip { axis: 0 }),
                ("Flip Y", "Mirror along Y", Action::Flip { axis: 1 }),
                ("Flip Z", "Mirror along Z", Action::Flip { axis: 2 }),
                ("Snap Vertices", "Snap every vertex to the grid", Action::SnapVertices),
            ],
        );
    });
    section(ui, "Gameplay", true, |ui| {
        sub_heading(ui, "Doors and movers");
        buttons(
            ui,
            actions,
            vec![
                (
                    "Door, hinge left",
                    "func_door_rotating hinged on the left end, with a trigger that opens it",
                    Action::MakeDoor { kind: DoorKind::Hinged { side: HingeSide::Left, angle: 95.0 }, trigger: true },
                ),
                (
                    "Door, hinge right",
                    "func_door_rotating hinged on the right end, with a trigger that opens it",
                    Action::MakeDoor { kind: DoorKind::Hinged { side: HingeSide::Right, angle: 95.0 }, trigger: true },
                ),
                (
                    "Sliding door up",
                    "func_door that slides up by its height",
                    Action::MakeDoor { kind: DoorKind::Sliding { direction: SlideDirection::Up, lip: 4.0 }, trigger: true },
                ),
                (
                    "Sliding door sideways",
                    "func_door that slides along its width",
                    Action::MakeDoor { kind: DoorKind::Sliding { direction: SlideDirection::Left, lip: 4.0 }, trigger: true },
                ),
                ("Lift", "func_platform that travels up, drag its travel handle", Action::MakePlatform),
            ],
        );
        sub_heading(ui, "Volumes around the selection");
        buttons(
            ui,
            actions,
            vec![
                ("Trigger around", "trigger_multiple around the selection", Action::VolumeAroundSelection("trigger_multiple".into())),
                ("Spawn area around", "trigger_spawn_area around the selection", Action::VolumeAroundSelection("trigger_spawn_area".into())),
            ],
        );
        if entity_count == 2 {
            sub_heading(ui, "Logic");
            buttons(ui, actions, vec![("Link…", "Connect an output of one selected entity to an input of the other", Action::ShowLinkDialog)]);
        }
    });
    brush_entity_section(ui, state, actions);
}

fn worldspawn_inspector(ui: &mut Ui, state: &mut EditorState, ps: &mut PanelState) {
    ui.heading("Map (worldspawn)");
    ui.label(RichText::new("Nothing selected. Click an object in a view or the outliner to inspect it. These properties apply to the whole map.").weak());
    let props: Vec<(String, String)> = state.doc.map.properties.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let def = state.game.entity("worldspawn").cloned();
    let label_w = widgets::label_width(ui);
    let mut index = 0;
    let mut rows: Vec<(String, String, Option<gt_formats::PropertyDef>)> = Vec::new();
    if let Some(def) = &def {
        for p in &def.properties {
            let value = state.doc.map.properties.get(&p.name).cloned().unwrap_or_else(|| p.default.clone());
            rows.push((p.name.clone(), value, Some(p.clone())));
        }
    }

    for (k, v) in props {
        if k != "classname" && !def.as_ref().is_some_and(|d| d.property(&k).is_some()) {
            rows.push((k, v, None));
        }
    }

    for (key, mut value, p) in rows {
        let ty = widgets::effective_type(p.as_ref().map(|p| p.ty), &key, &value);
        let hover = p.as_ref().map(|p| p.description.clone()).unwrap_or_default();
        let options = p.as_ref().map(|p| p.options.clone()).unwrap_or_default();
        let custom = p.is_none();
        let (edited, remove) = widgets::row(ui, index, key.as_str(), &hover, label_w, |ui| {
            let edited = property_editor(ui, ty, &options, &mut value, state, ui.available_width() - widgets::TRAILING);
            (edited, custom && ui.small_button("×").on_hover_text("Remove property").clicked())
        });
        index += 1;
        if edited {
            state.doc.edit_coalesced("Set Map Property", |m, _| {
                m.properties.insert(key.clone(), value);
            });
        }

        if remove {
            state.doc.edit("Remove Map Property", |m, _| {
                m.properties.remove(&key);
            });
        }
    }

    add_property_row(ui, ps, |k, v| {
        state.doc.edit("Add Map Property", |m, _| {
            m.properties.insert(k, v);
        });
    });
}

fn add_property_row(ui: &mut Ui, ps: &mut PanelState, mut add: impl FnMut(String, String)) {
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut ps.new_key).hint_text("key").desired_width(90.0));
        ui.add(egui::TextEdit::singleline(&mut ps.new_value).hint_text("value").desired_width(90.0));
        if ui.button("Add").clicked() && !ps.new_key.trim().is_empty() {
            add(ps.new_key.trim().to_string(), ps.new_value.clone());
            ps.new_key.clear();
            ps.new_value.clear();
        }
    });
}

fn entity_inspector(ui: &mut Ui, state: &mut EditorState, ps: &mut PanelState, id: NodeId, actions: &mut Vec<Action>) {
    let Some(entity) = state.doc.map.entity(id).cloned() else { return };
    let is_point = state.doc.map.is_point_entity(id);
    let def: Option<EntityDef> = state.game.entity(&entity.classname).cloned();

    ui.horizontal(|ui| {
        ui.heading(&entity.classname);
        if def.is_none() {
            ui.label(RichText::new("no definition").color(theme::WARNING));
        }
    });
    if let Some(d) = &def
        && !d.description.is_empty()
    {
        ui.label(RichText::new(&d.description).weak());
    }

    ui.horizontal_wrapped(|ui| {
        if ui.small_button("Code Reference").on_hover_text("GDScript and C# for this entity").clicked() {
            ps.reference_class = entity.classname.clone();
            actions.push(Action::ShowReference);
        }

        let entity_count = state.doc.selection.nodes.iter().filter(|n| state.doc.map.entity(**n).is_some()).count();
        if entity_count == 2 && ui.small_button("Link…").on_hover_text("Connect an output of one selected entity to an input of the other").clicked() {
            actions.push(Action::ShowLinkDialog);
        }

        if let Some(d) = &def {
            let gizmos = d.gizmos(state.game.units_per_meter);
            if !gizmos.is_empty() {
                let names: Vec<String> = gizmos.iter().map(|g| g.label()).collect();
                ui.label(RichText::new(format!("drag the {} handles in the views", names.join(", "))).weak());
            }
        }
    });

    let label_w = widgets::label_width(ui);
    let mut classname = entity.classname.clone();
    widgets::row(ui, 0, "classname", "", label_w, |ui| {
        let kind_filter = if is_point { gt_formats::EntityKind::Point } else { gt_formats::EntityKind::Solid };
        egui::ComboBox::from_id_salt("classname").selected_text(&classname).width(ui.available_width() - widgets::TRAILING).show_ui(ui, |ui| {
            for d in state.game.entities.iter().filter(|d| d.kind == kind_filter) {
                ui.selectable_value(&mut classname, d.classname.clone(), &d.classname);
            }
        });
    });
    if classname != entity.classname {
        state.doc.edit("Change Class", |m, _| {
            if let Some(e) = m.entity_mut(id) {
                e.classname = classname.clone();
            }
        });
    }

    if is_point {
        let mut origin = [entity.origin.x, entity.origin.y, entity.origin.z];
        let mut angles = [entity.angles.x, entity.angles.y, entity.angles.z];
        let mut changed = false;
        widgets::row(ui, 1, "origin", "Position in map units", label_w, |ui| {
            changed |= widgets::vector_input(ui, &mut origin, ui.available_width() - widgets::TRAILING, 1.0);
        });
        widgets::row(ui, 2, "angles", "Pitch, yaw and roll in degrees", label_w, |ui| {
            changed |= widgets::vector_input(ui, &mut angles, ui.available_width() - widgets::TRAILING, 1.0);
        });
        if changed {
            state.doc.edit_coalesced("Set Transform", |m, _| {
                if let Some(e) = m.entity_mut(id) {
                    e.origin = DVec3::from_array(origin);
                    e.angles = DVec3::from_array(angles);
                }
            });
        }
    }

    let value_x = ui.cursor().left() + label_w;
    egui::CollapsingHeader::new(RichText::new("Properties").strong()).id_salt("entity_properties").default_open(true).show(ui, |ui| {
        let label_w = value_x - ui.cursor().left();
        let mut index = 0;
        if let Some(d) = &def {
            for p in &d.properties {
                let set = entity.properties.contains_key(&p.name);
                let label = if set { RichText::new(&p.name).strong() } else { RichText::new(&p.name).color(ui.visuals().weak_text_color()) };
                let hover = if p.description.is_empty() { p.name.clone() } else { format!("{}\n{}", p.name, p.description) };
                let mut value = entity.properties.get(&p.name).cloned().unwrap_or_else(|| p.default.clone());
                let ty = widgets::effective_type(Some(p.ty), &p.name, &value);
                let (edited, reset) = widgets::row(ui, index, label, &hover, label_w, |ui| {
                    let edited = property_editor(ui, ty, &p.options, &mut value, state, ui.available_width() - widgets::TRAILING);
                    (edited, set && ui.small_button("×").on_hover_text("Reset to default").clicked())
                });
                index += 1;
                if edited {
                    let key = p.name.clone();
                    state.doc.edit_coalesced(&format!("Set {key}"), |m, s| {
                        for sel in s.nodes.clone() {
                            if let Some(e) = m.entity_mut(sel) {
                                e.properties.insert(key.clone(), value.clone());
                            }
                        }
                    });
                }

                if reset {
                    let key = p.name.clone();
                    state.doc.edit("Reset Property", |m, _| {
                        if let Some(e) = m.entity_mut(id) {
                            e.properties.remove(&key);
                        }
                    });
                }
            }
        }

        for (k, v) in &entity.properties {
            if def.as_ref().is_some_and(|d| d.property(k).is_some()) {
                continue;
            }

            let mut value = v.clone();
            let ty = widgets::effective_type(None, k, v);
            let (edited, remove) = widgets::row(ui, index, RichText::new(k).italics(), "Not in the entity definition", label_w, |ui| {
                let edited = property_editor(ui, ty, &[], &mut value, state, ui.available_width() - widgets::TRAILING);
                (edited, ui.small_button("×").on_hover_text("Remove property").clicked())
            });
            index += 1;
            if edited {
                let key = k.clone();
                state.doc.edit_coalesced(&format!("Set {key}"), |m, _| {
                    if let Some(e) = m.entity_mut(id) {
                        e.properties.insert(key, value);
                    }
                });
            }

            if remove {
                let key = k.clone();
                state.doc.edit("Remove Property", |m, _| {
                    if let Some(e) = m.entity_mut(id) {
                        e.properties.remove(&key);
                    }
                });
            }
        }

        add_property_row(ui, ps, |k, v| {
            state.doc.edit("Add Property", |m, _| {
                if let Some(e) = m.entity_mut(id) {
                    e.properties.insert(k, v);
                }
            });
        });
    });

    let title = format!("Outputs (I/O, {})", entity.outputs.len());
    egui::CollapsingHeader::new(RichText::new(title).strong()).id_salt("entity_outputs").default_open(true).show(ui, |ui| {
        io_editor(ui, state, id, &entity, def.as_ref());
    });
}

fn io_editor(ui: &mut Ui, state: &mut EditorState, id: NodeId, entity: &gt_doc::Entity, def: Option<&EntityDef>) {
    if entity.outputs.is_empty() {
        ui.label(RichText::new("No outputs. Outputs fire inputs on other entities, for example a trigger opening a door.").weak());
    }

    let targetnames: Vec<(String, String)> =
        state.doc.map.entities().filter_map(|(_, e)| e.targetname().map(|n| (n.to_string(), e.classname.clone()))).collect();
    let mut outputs = entity.outputs.clone();
    let mut changed = false;
    let mut remove = None;
    for (i, conn) in outputs.iter_mut().enumerate() {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            egui::Grid::new(("io", i)).num_columns(2).show(ui, |ui| {
                ui.label("output");
                changed |= combo_or_text(
                    ui,
                    egui::Id::new(("out", i)),
                    &mut conn.output,
                    def.map(|d| d.outputs.iter().map(|o| o.name.clone()).collect()).unwrap_or_default(),
                    110.0,
                );
                ui.end_row();
                ui.label("target");
                changed |= combo_or_text(ui, egui::Id::new(("target", i)), &mut conn.target, targetnames.iter().map(|(n, _)| n.clone()).collect(), 110.0);
                ui.end_row();
                let target_inputs: Vec<String> = targetnames
                    .iter()
                    .filter(|(n, _)| *n == conn.target)
                    .filter_map(|(_, c)| state.game.entity(c))
                    .flat_map(|d| d.inputs.iter().map(|i| i.name.clone()))
                    .collect();
                ui.label("input");
                changed |= combo_or_text(ui, egui::Id::new(("input", i)), &mut conn.input, target_inputs, 110.0);
                ui.end_row();
                ui.label("parameter");
                changed |= ui.text_edit_singleline(&mut conn.parameter).changed();
                ui.end_row();
                ui.label("delay");
                changed |= ui.add(egui::DragValue::new(&mut conn.delay).speed(0.05).range(0.0..=3600.0).suffix(" s")).changed();
                ui.end_row();
                ui.label("fire once");
                let mut once = conn.times == 1;
                if ui.checkbox(&mut once, "").changed() {
                    conn.times = if once { 1 } else { -1 };
                    changed = true;
                }

                ui.end_row();
            });
            let valid_target = conn.target.is_empty() || targetnames.iter().any(|(n, _)| *n == conn.target) || gt_doc::issues::is_dynamic_target(&conn.target);
            ui.horizontal(|ui| {
                if !valid_target {
                    ui.label(RichText::new("target not found").color(theme::ERROR));
                }

                if ui.small_button("Remove").clicked() {
                    remove = Some(i);
                }
            });
        });
    }

    if let Some(i) = remove {
        outputs.remove(i);
        changed = true;
    }

    if ui.button("+ Add Output").clicked() {
        let output = def.and_then(|d| d.outputs.first()).map(|o| o.name.clone()).unwrap_or_default();
        outputs.push(IoConnection { output, target: String::new(), input: String::new(), parameter: String::new(), delay: 0.0, times: -1 });
        changed = true;
    }

    if changed {
        state.doc.edit_coalesced("Edit Outputs", |m, _| {
            if let Some(e) = m.entity_mut(id) {
                e.outputs = outputs;
            }
        });
    }
}

fn combo_or_text(ui: &mut Ui, salt: egui::Id, value: &mut String, options: Vec<String>, width: f32) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        changed |= ui.add(egui::TextEdit::singleline(value).desired_width(width)).changed();
        if !options.is_empty() {
            egui::ComboBox::from_id_salt(salt).selected_text("▾").width(24.0).show_ui(ui, |ui| {
                for o in options {
                    if ui.selectable_label(*value == o, &o).clicked() {
                        *value = o;
                        changed = true;
                    }
                }
            });
        }
    });
    changed
}

/// Typed editor for a string property value, `width` wide. Returns true when changed.
fn property_editor(ui: &mut Ui, ty: PropertyType, options: &[(String, String)], value: &mut String, state: &EditorState, width: f32) -> bool {
    match ty {
        PropertyType::Bool => {
            let mut b = matches!(value.trim(), "1" | "true" | "True");
            if ui.checkbox(&mut b, "").changed() {
                *value = if b { "1".into() } else { "0".into() };
                return true;
            }

            false
        }
        PropertyType::Int | PropertyType::Float => {
            let integer = ty == PropertyType::Int;
            let mut f: f64 = value.trim().parse().unwrap_or(0.0);
            let speed = widgets::drag_speed(&[f]);
            if widgets::drag_field(ui, &mut f, width, speed, integer) {
                *value = if integer { (f.round() as i64).to_string() } else { widgets::format_number(f) };
                return true;
            }

            false
        }
        PropertyType::Vector2 | PropertyType::Vector3 => {
            let mut nums = widgets::parse_numbers(value).unwrap_or_default();
            nums.resize(if ty == PropertyType::Vector2 { 2 } else { 3 }, 0.0);
            let speed = widgets::drag_speed(&nums).max(0.1);
            if widgets::vector_input(ui, &mut nums, width, speed) {
                *value = nums.iter().map(|n| widgets::format_number(*n)).collect::<Vec<_>>().join(" ");
                return true;
            }

            false
        }
        PropertyType::Color => widgets::color_input(ui, value, width),
        PropertyType::Choices => {
            let mut changed = false;
            let current = options.iter().find(|(_, v)| v == value).map(|(l, _)| l.clone()).unwrap_or_else(|| value.clone());
            egui::ComboBox::from_id_salt(ui.next_auto_id()).selected_text(current).width(width - 8.0).show_ui(ui, |ui| {
                for (label, v) in options {
                    if ui.selectable_label(value == v, label).clicked() {
                        *value = v.clone();
                        changed = true;
                    }
                }
            });
            changed
        }
        PropertyType::Flags => {
            let mut bits: i64 = value.trim().parse().unwrap_or(0);
            let mut changed = false;
            ui.vertical(|ui| {
                for (label, v) in options {
                    let bit: i64 = v.parse().unwrap_or(0);
                    let mut on = bits & bit != 0;
                    if ui.checkbox(&mut on, label).changed() {
                        bits = if on { bits | bit } else { bits & !bit };
                        changed = true;
                    }
                }
            });
            if changed {
                *value = bits.to_string();
            }

            changed
        }
        PropertyType::TargetDestination => {
            let names: Vec<String> = state.doc.map.entities().filter_map(|(_, e)| e.targetname().map(str::to_string)).collect();
            combo_or_text(ui, ui.next_auto_id(), value, names, width - 36.0)
        }
        PropertyType::Resource => {
            let browse = state.game.project_root.is_some();
            let mut changed = widgets::text_field(ui, value, if browse { width - 28.0 } else { width });
            if let Some(root) = &state.game.project_root
                && ui.small_button("…").clicked()
                && let Some(p) = rfd::FileDialog::new().set_directory(root).pick_file()
                && let Some(res) = gt_formats::game::to_res_path(root, &p)
            {
                *value = res;
                changed = true;
            }

            changed
        }
        _ => widgets::text_field(ui, value, width),
    }
}

// --------------------------------------------------------------------- faces

/// Face UV operation: (uv, face normal, face points, texture size).
type UvOp = Box<dyn Fn(&mut FaceUv, DVec3, &[DVec3], DVec2)>;

/// Sets planar UV values on brush faces and on mesh faces without explicit UVs.
fn edit_planar(state: &mut EditorState, faces: &[(NodeId, usize)], label: &str, coalesce: bool, f: impl Fn(&mut FaceUv, DVec3, &[DVec3], DVec2)) {
    let plans: Vec<(NodeId, usize, DVec3, Vec<DVec3>, DVec2)> = faces
        .iter()
        .filter_map(|(id, fi)| {
            let info = crate::texture_ops::face_info(&state.doc.map, *id, *fi).filter(|i| i.explicit.is_none())?;
            let size = crate::texture_ops::tex_size(state, &info.material);
            Some((*id, *fi, info.plane.normal, info.points, size))
        })
        .collect();
    if plans.is_empty() {
        return;
    }

    let apply = |m: &mut gt_doc::Map| {
        for (id, fi, n, pts, size) in &plans {
            if let Some(face) = m.brush_mut(*id).and_then(|b| b.faces.get_mut(*fi)) {
                f(&mut face.data.uv, *n, pts, *size);
            } else if let Some(face) = m.mesh_mut(*id).and_then(|mesh| mesh.faces.get_mut(*fi)) {
                f(&mut face.data.uv, *n, pts, *size);
            }
        }
    };
    if coalesce {
        state.doc.edit_coalesced(label, |m, _| apply(m));
    } else {
        state.doc.edit(label, |m, _| apply(m));
    }
}

fn face_inspector(ui: &mut Ui, state: &mut EditorState, actions: &mut Vec<Action>) {
    let faces: Vec<(NodeId, usize)> = state.doc.selection.faces.iter().copied().collect();
    let Some((bid, fi)) = faces.first().copied() else { return };
    let Some(info) = crate::texture_ops::face_info(&state.doc.map, bid, fi) else { return };
    let is_mesh = state.doc.map.mesh(bid).is_some();
    ui.heading(format!("{} {}faces", faces.len(), if is_mesh { "mesh " } else { "" }));
    let mut material = info.material.clone();
    ui.horizontal(|ui| {
        ui.label("material");
        if ui.text_edit_singleline(&mut material).lost_focus() && material != info.material {
            actions.push(Action::ApplyMaterial(material.clone()));
        }
    });
    if let Some(size) = state.materials.size_label(&info.material) {
        let details = state.materials.info(&info.material).map(|m| material_summary(&m)).unwrap_or_default();
        ui.label(RichText::new(format!("{size}{details}")).weak());
    }

    section(ui, "Alignment", true, |ui| {
        if info.explicit.is_none() {
            let mut uv = info.uv.clone();
            let mut changed = false;
            egui::Grid::new("uv").num_columns(3).show(ui, |ui| {
                ui.label("offset");
                changed |= ui.add(egui::DragValue::new(&mut uv.offset.x).speed(1.0)).changed();
                changed |= ui.add(egui::DragValue::new(&mut uv.offset.y).speed(1.0)).changed();
                ui.end_row();
                ui.label("scale");
                changed |= ui.add(egui::DragValue::new(&mut uv.scale.x).speed(0.01)).changed();
                changed |= ui.add(egui::DragValue::new(&mut uv.scale.y).speed(0.01)).changed();
                ui.end_row();
                ui.label("rotation");
                let mut rot = uv.rotation;
                if ui.add(egui::DragValue::new(&mut rot).speed(1.0).suffix("°")).changed() {
                    let delta = rot - uv.rotation;
                    uv.rotate(delta);
                    changed = true;
                }

                ui.end_row();
            });
            if changed {
                let (offset, scale, rotation_delta) = (uv.offset, uv.scale, uv.rotation - info.uv.rotation);
                edit_planar(state, &faces, "Set UV", true, |uv, _, _, _| {
                    uv.offset = offset;
                    uv.scale = scale;
                    if rotation_delta.abs() > 1e-9 {
                        uv.rotate(rotation_delta);
                    }
                });
            }
        } else {
            ui.label(RichText::new("explicit UVs (edit them in the UV editor)").weak());
        }

        ui.horizontal_wrapped(|ui| {
            ui.label("Justify");
            for j in gt_geom::Justify::ALL {
                if ui.small_button(j.label()).clicked() {
                    actions.push(Action::Justify(j));
                }
            }

            ui.checkbox(&mut state.treat_as_one, "Treat as one");
        });
        ui.horizontal_wrapped(|ui| {
            let mut op: Option<(&str, UvOp)> = None;
            if ui.button("Reset").clicked() {
                op = Some(("Reset UV", Box::new(|uv, n, _, _| *uv = FaceUv::paraxial(n, DVec2::ONE))));
            }

            if ui.button("Align to Face").clicked() {
                op = Some(("Align UV", Box::new(|uv, n, _, _| *uv = FaceUv::face_aligned(n, uv.scale))));
            }

            if ui.button("Align to View").clicked() {
                actions.push(Action::AlignTextureToView);
            }

            if ui.button("Flip H").clicked() {
                op = Some(("Flip UV", Box::new(|uv, _, _, _| uv.scale.x = -uv.scale.x)));
            }

            if ui.button("Flip V").clicked() {
                op = Some(("Flip UV", Box::new(|uv, _, _, _| uv.scale.y = -uv.scale.y)));
            }

            if ui.button("⟲ 90").clicked() {
                crate::texture_ops::rotate(state, &faces, -90.0);
            }

            if ui.button("⟳ 90").clicked() {
                crate::texture_ops::rotate(state, &faces, 90.0);
            }

            if ui.button("× 2").on_hover_text("Double the texture size").clicked() {
                crate::texture_ops::scale(state, &faces, DVec2::splat(2.0));
            }

            if ui.button("÷ 2").on_hover_text("Halve the texture size").clicked() {
                crate::texture_ops::scale(state, &faces, DVec2::splat(0.5));
            }

            if let Some((label, f)) = op {
                edit_planar(state, &faces, label, false, f);
            }
        });
    });
    section(ui, "Texture Tools", true, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label("Texel density");
            for d in [0.25, 0.5, 1.0, 2.0, 4.0] {
                if ui.small_button(format!("{d}")).on_hover_text(format!("{d} world units per texture pixel")).clicked() {
                    actions.push(Action::TexelDensity(d));
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.button("Copy").on_hover_text("Copy material and alignment").clicked() {
                actions.push(Action::CopyAlignment);
            }

            let can_paste = state.uv_clipboard.is_some();
            if ui.add_enabled(can_paste, egui::Button::new("Paste Alignment")).clicked() {
                actions.push(Action::PasteAlignment);
            }

            if ui.button("Hotspot").on_hover_text("Fit to the best rectangle of <texture>.hotspots.json").clicked() {
                actions.push(Action::HotspotTexture);
            }

            if ui.button("Hotspot Editor").clicked() {
                actions.push(Action::ShowHotspotEditor);
            }

            if ui.button("UV Editor").clicked() {
                actions.push(Action::ShowUvEditor);
            }
        });
        if is_mesh {
            ui.horizontal_wrapped(|ui| {
                ui.label("Mesh UVs");
                for k in crate::texture_ops::MeshUvKind::COMMON {
                    if ui.small_button(k.label()).on_hover_text(k.hint()).clicked() {
                        actions.push(Action::MeshUv(k));
                    }
                }
            });
        }
    });

    if is_mesh {
        return;
    }

    section(ui, "Surface Properties", true, |ui| {
        let face = state.doc.map.brush(bid).and_then(|b| b.faces.get(fi)).cloned();
        let Some(face) = face else { return };
        if face.data.props.is_empty() {
            ui.label(RichText::new("No surface properties. Add one below, Godot reads them per face.").weak());
        }

        let mut props = face.data.props.clone();
        let mut prop_changed = false;
        let mut remove = None;
        for (k, v) in props.iter_mut() {
            ui.horizontal(|ui| {
                ui.label(k.as_str());
                prop_changed |= ui.text_edit_singleline(v).changed();
                if ui.small_button("×").clicked() {
                    remove = Some(k.clone());
                }
            });
        }

        if let Some(k) = remove {
            props.remove(&k);
            prop_changed = true;
        }

        ui.horizontal_wrapped(|ui| {
            for preset in ["collision_layer", "smoothing_group", "no_collision", "no_shadow"] {
                if !props.contains_key(preset) && ui.small_button(format!("+ {preset}")).clicked() {
                    props.insert(preset.into(), "1".into());
                    prop_changed = true;
                }
            }
        });
        if prop_changed {
            state.doc.edit_coalesced("Set Face Properties", |m, _| {
                for (id, f) in &faces {
                    if let Some(face) = m.brush_mut(*id).and_then(|b| b.faces.get_mut(*f)) {
                        face.data.props = props.clone();
                    }
                }
            });
        }
    });
}

/// Short description of a Godot material's preview relevant settings.
pub fn material_summary(m: &gt_formats::godot_material::GodotMaterial) -> String {
    use gt_formats::godot_material::Transparency;
    let mut parts = Vec::new();
    match m.transparency {
        Transparency::Alpha => parts.push("transparent".to_string()),
        Transparency::Scissor(t) => parts.push(format!("alpha scissor {t}")),
        Transparency::Hash => parts.push("alpha hash".into()),
        Transparency::Opaque => {}
    }

    if m.emission.is_some() {
        parts.push(format!("emissive x{}", m.emission_energy));
    }

    if m.normal_texture.is_some() {
        parts.push("normal map".into());
    }

    if m.double_sided {
        parts.push("double sided".into());
    }

    if m.unshaded {
        parts.push("unshaded".into());
    }

    if m.nearest == Some(true) {
        parts.push("pixel filter".into());
    }

    if parts.is_empty() { String::new() } else { format!(", {}", parts.join(", ")) }
}

// ----------------------------------------------------------------- materials

/// Every material used by brushes, meshes and terrain layers of the map.
fn used_materials(map: &gt_doc::Map) -> HashSet<String> {
    let mut used: HashSet<String> = map.brushes().flat_map(|(_, b)| b.faces.iter().map(|f| f.data.material.clone())).collect();
    used.extend(map.meshes().flat_map(|(_, m)| m.faces.iter().map(|f| f.data.material.clone())));
    used.extend(map.terrains().flat_map(|(_, t)| t.layers.iter().map(|l| l.material.clone())));
    used
}

/// Selects every brush and mesh face using `material`.
pub fn select_faces_with_material(state: &mut EditorState, material: &str) -> usize {
    let map = &state.doc.map;
    let mut faces: Vec<(NodeId, usize)> = Vec::new();
    for (id, b) in map.brushes() {
        if map.is_editable(id) {
            faces.extend(b.faces.iter().enumerate().filter(|(_, f)| f.data.material == material).map(|(i, _)| (id, i)));
        }
    }

    for (id, m) in map.meshes() {
        if map.is_editable(id) {
            faces.extend(m.faces.iter().enumerate().filter(|(_, f)| f.data.material == material).map(|(i, _)| (id, i)));
        }
    }

    let n = faces.len();
    state.doc.select(|_, s| {
        s.clear();
        for (id, f) in &faces {
            s.select_face(*id, *f);
        }
    });
    n
}

/// Replaces `from` with `to` on every face and terrain layer of the map.
pub fn replace_material(state: &mut EditorState, from: &str, to: &str) -> usize {
    state.doc.edit("Replace Material", |m, _| {
        let mut n = 0;
        let ids: Vec<NodeId> = m.nodes.keys().copied().collect();
        for id in ids {
            match m.get_mut(id).map(|node| &mut node.kind) {
                Some(NodeKind::Brush(b)) => b.faces.iter_mut().filter(|f| f.data.material == from).for_each(|f| {
                    f.data.material = to.to_string();
                    n += 1;
                }),
                Some(NodeKind::Mesh(mesh)) => mesh.faces.iter_mut().filter(|f| f.data.material == from).for_each(|f| {
                    f.data.material = to.to_string();
                    n += 1;
                }),
                Some(NodeKind::Terrain(t)) => t.layers.iter_mut().filter(|l| l.material == from).for_each(|l| {
                    l.material = to.to_string();
                    n += 1;
                }),
                _ => {}
            }
        }

        n
    })
}

fn badge(painter: &egui::Painter, pos: egui::Pos2, text: &str, color: Color32) {
    let font = egui::FontId::proportional(9.0);
    let galley = painter.layout_no_wrap(text.to_string(), font, Color32::BLACK);
    let pad = Vec2::new(3.0, 1.0);
    let rect = egui::Rect::from_min_size(pos, galley.size() + pad * 2.0);
    painter.rect_filled(rect, 2.0, color);
    painter.galley(rect.min + pad, galley, Color32::BLACK);
}

fn material_cell(ui: &mut Ui, state: &mut EditorState, name: &str, size: f32, label: bool, budget: &mut u32) -> egui::Response {
    let cell = if label { Vec2::new(size + 8.0, size + 22.0) } else { Vec2::splat(size + 4.0) };
    let (rect, resp) = ui.allocate_exact_size(cell, Sense::click_and_drag());
    let selected = name == state.current_material;
    let (has_normal, missing_albedo, is_pbr, is_emissive) =
        state.materials.find(name).map(|e| (e.has_normal, e.missing_albedo, e.is_pbr, e.is_emissive)).unwrap_or((false, false, false, false));
    if selected {
        ui.painter().rect_filled(rect, 3.0, theme::selected_fill());
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, 3.0, theme::GRAY_2);
    }

    let img_rect = egui::Rect::from_min_size(rect.min + Vec2::splat(if label { 4.0 } else { 2.0 }), Vec2::splat(size));
    let ctx = ui.ctx().clone();
    match state.materials.thumbnail(&ctx, name, budget) {
        Some(tex) => {
            ui.painter().image(tex.id(), img_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
        }
        None => {
            ui.painter().rect_filled(img_rect, 2.0, theme::GRAY_2);
            ui.ctx().request_repaint();
        }
    }

    if state.prefs.favorite_materials.iter().any(|f| f == name) {
        ui.painter().text(img_rect.right_top() + Vec2::new(-3.0, 1.0), egui::Align2::RIGHT_TOP, "★", egui::FontId::proportional(13.0), theme::YELLOW);
    }

    if label {
        let corner = img_rect.left_top() + Vec2::new(2.0, 2.0);
        if missing_albedo {
            badge(ui.painter(), corner, "no albedo", theme::WARNING);
        } else if is_pbr {
            badge(ui.painter(), corner, "PBR", theme::GREEN);
        } else if has_normal {
            badge(ui.painter(), corner, "normal", theme::INFO);
        }

        if is_emissive {
            badge(ui.painter(), img_rect.left_bottom() + Vec2::new(2.0, -15.0), "glow", theme::YELLOW);
        }
    }

    if label {
        let short = name.rsplit('/').next().unwrap_or(name);
        ui.painter().text(egui::pos2(rect.center().x, rect.max.y - 9.0), egui::Align2::CENTER_CENTER, short, egui::FontId::proportional(11.0), theme::GRAY_6);
    }

    resp
}

fn material_interactions(resp: egui::Response, state: &mut EditorState, name: &str, actions: &mut Vec<Action>) {
    let tooltip = {
        let size = state.materials.size_label(name).map(|s| format!("\n{s}")).unwrap_or_default();
        let info = state.materials.info(name).map(|m| material_summary(&m).trim_start_matches(", ").to_string()).filter(|s| !s.is_empty());
        let note = state
            .materials
            .find(name)
            .map(|e| {
                if e.missing_albedo {
                    "\nnormal map with no matching albedo/diffuse texture"
                } else if e.is_pbr {
                    "\nPBR set: companion maps found and auto mapped, dropping it applies them"
                } else if e.has_normal {
                    "\nhas a normal map, dropping it applies valid normals"
                } else {
                    ""
                }
            })
            .unwrap_or("");
        let glow = if state.materials.find(name).is_some_and(|e| e.is_emissive && e.material_file.is_none()) {
            "\nemissive: its _emission map glows in Godot and in the lit view"
        } else {
            ""
        };
        format!("{name}{size}{}{note}{glow}", info.map(|i| format!("\n{i}")).unwrap_or_default())
    };
    let resp = resp.on_hover_text(tooltip);
    if resp.clicked() {
        actions.push(Action::ApplyMaterial(name.to_string()));
    }

    if resp.drag_started() {
        resp.dnd_set_drag_payload(DndPayload::Material(name.to_string()));
    }

    resp.context_menu(|ui| {
        if ui.button("Apply to selection").clicked() {
            actions.push(Action::ApplyMaterial(name.to_string()));
            ui.close();
        }

        let favorite = state.prefs.favorite_materials.iter().any(|f| f == name);
        if ui.button(if favorite { "Remove from favourites" } else { "Add to favourites" }).clicked() {
            if favorite {
                state.prefs.favorite_materials.retain(|f| f != name);
            } else {
                state.prefs.favorite_materials.push(name.to_string());
            }

            ui.close();
        }

        if ui.button("Select faces using it").clicked() {
            let n = select_faces_with_material(state, name);
            state.set_status(format!("Selected {n} faces using {name}"));
            ui.close();
        }

        let current = state.current_material.clone();
        if current != name && ui.button(format!("Replace {current} with it in the map")).clicked() {
            let n = replace_material(state, &current, name);
            state.set_status(format!("Replaced {current} on {n} faces"));
            ui.close();
        }

        if ui.button("Hotspot editor").clicked() {
            actions.push(Action::EditHotspots(name.to_string()));
            ui.close();
        }

        if ui.button("Copy name").clicked() {
            ui.ctx().copy_text(name.to_string());
            ui.close();
        }
    });
}

pub fn material_browser(ui: &mut Ui, state: &mut EditorState, ps: &mut PanelState, actions: &mut Vec<Action>) {
    ui.horizontal_wrapped(|ui| {
        ui.add(egui::TextEdit::singleline(&mut ps.material_filter).hint_text("Search materials").desired_width(160.0));
        let folders = state.materials.folders();
        egui::ComboBox::from_id_salt("mat_folder").selected_text(ps.material_folder.clone().unwrap_or_else(|| "All folders".into())).show_ui(ui, |ui| {
            ui.selectable_value(&mut ps.material_folder, None, "All folders");
            for f in folders {
                ui.selectable_value(&mut ps.material_folder, Some(f.clone()), if f.is_empty() { "(root)".to_string() } else { f });
            }
        });
        ui.checkbox(&mut ps.material_used_only, "Used in map");
        ui.checkbox(&mut ps.material_favorites_only, "★ Favourites");
        ui.add(egui::Slider::new(&mut ps.thumb_size, 40.0..=160.0).show_value(false));
        if ui.small_button("⟳").on_hover_text("Reload materials and Godot material settings").clicked() {
            actions.push(Action::ReloadMaterials);
        }

        ui.label(RichText::new(format!("current: {}", state.current_material)).weak());
    });
    let mut budget = 6;
    let recent: Vec<String> = state.prefs.recent_materials.iter().take(16).cloned().collect();
    if !recent.is_empty() {
        ui.horizontal(|ui| {
            ui.label(RichText::new("recent").weak());
            for name in recent {
                let resp = material_cell(ui, state, &name, 28.0, false, &mut budget);
                material_interactions(resp, state, &name, actions);
            }
        });
    }

    ui.separator();

    let used = if ps.material_used_only { used_materials(&state.doc.map) } else { HashSet::new() };
    let filter = ps.material_filter.to_lowercase();
    let favorites = state.prefs.favorite_materials.clone();
    let names: Vec<String> = state
        .materials
        .entries
        .iter()
        .filter(|e| filter.is_empty() || e.name.to_lowercase().contains(&filter))
        .filter(|e| ps.material_folder.as_ref().is_none_or(|f| &e.folder == f))
        .filter(|e| !ps.material_used_only || used.contains(&e.name))
        .filter(|e| !ps.material_favorites_only || favorites.contains(&e.name))
        .map(|e| e.name.clone())
        .collect();

    if state.materials.entries.is_empty() {
        ui.label(RichText::new("No materials found. Open a Godot project with textures (Godot > Open Godot Project…).").weak());
    } else if names.is_empty() {
        ui.label(RichText::new("No materials match the search and filters").weak());
    }

    let size = ps.thumb_size;
    let cell = Vec2::new(size + 8.0, size + 22.0);
    let cols = ((ui.available_width() / cell.x).floor() as usize).max(1);
    let rows = names.len().div_ceil(cols);
    ScrollArea::vertical().auto_shrink([false, false]).show_rows(ui, cell.y, rows, |ui, range| {
        for row in range {
            ui.horizontal(|ui| {
                for name in names.iter().skip(row * cols).take(cols) {
                    let resp = material_cell(ui, state, name, size, true, &mut budget);
                    material_interactions(resp, state, name, actions);
                }
            });
        }
    });
}

// -------------------------------------------------------------------- models

/// A tint per model format so the cards read apart at a glance.
pub(crate) fn model_ext_color(ext: &str) -> Color32 {
    match ext {
        "glb" | "gltf" => theme::TEAL,
        "obj" => theme::CYAN,
        "bbmodel" => theme::YELLOW,
        "stl" => theme::GRAY_6,
        "md2" | "md3" | "mdl" => theme::PINK,
        "map" | "vmf" => theme::GREEN,
        _ => theme::GRAY_5,
    }
}

/// The CVD-safe accent palette (see theme.rs), picked stably per pack name so the same pack always
/// gets the same slug color without a color table to maintain.
const PACK_ACCENTS: [Color32; 8] = [theme::MAGENTA, theme::GREEN, theme::YELLOW, theme::BLUE, theme::CYAN, theme::PINK, theme::RED, theme::TEAL];

pub(crate) fn pack_color(name: &str) -> Color32 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in name.bytes() {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }

    PACK_ACCENTS[(hash % PACK_ACCENTS.len() as u64) as usize]
}

pub(crate) fn truncate_slug(s: &str, max_chars: usize) -> std::borrow::Cow<'_, str> {
    if s.chars().count() <= max_chars {
        std::borrow::Cow::Borrowed(s)
    } else {
        std::borrow::Cow::Owned(format!("{}…", s.chars().take(max_chars.saturating_sub(1)).collect::<String>()))
    }
}

/// Fits a pack label to `max_width` by measuring it with the panel's own font rather than guessing a
/// character count: the label as given, then with a leading shared word dropped (packs sharing a
/// prefix like "GodotTrench low poly" otherwise all say the same thing on every card), then
/// character-truncated with an ellipsis.
fn fit_slug(painter: &egui::Painter, text: &str, max_width: f32) -> String {
    let font = egui::FontId::proportional(9.0);
    let width = |s: &str| painter.layout_no_wrap(s.to_string(), font.clone(), Color32::WHITE).size().x;
    if width(text) <= max_width {
        return text.to_string();
    }

    let candidate = text.split_once(' ').map_or(text, |(_, rest)| rest);
    if width(candidate) <= max_width {
        return candidate.to_string();
    }

    let mut s = candidate.to_string();
    while !s.is_empty() && width(&format!("{s}…")) > max_width {
        s.pop();
    }

    if s.is_empty() { "…".to_string() } else { format!("{s}…") }
}

fn model_cell(ui: &mut Ui, selected: bool, entry: &crate::models::ModelEntry, size: f32, show_slug: bool, thumb: Option<egui::TextureId>) -> egui::Response {
    let cell = Vec2::new(size + 8.0, size + 22.0);
    let (rect, resp) = ui.allocate_exact_size(cell, Sense::click_and_drag());
    if selected {
        ui.painter().rect_filled(rect, 3.0, theme::selected_fill());
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, 3.0, theme::GRAY_2);
    }

    let img_rect = egui::Rect::from_min_size(rect.min + Vec2::splat(4.0), Vec2::splat(size));
    ui.painter().rect_filled(img_rect, 2.0, theme::GRAY_1);
    match thumb {
        Some(id) => {
            ui.painter().image(id, img_rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
        }
        None => {
            ui.put(img_rect, icons::MESH.image((size * 0.55).min(48.0)).tint(model_ext_color(&entry.ext)));
        }
    }

    badge(ui.painter(), img_rect.left_top() + Vec2::new(2.0, 2.0), &entry.ext, model_ext_color(&entry.ext));
    if show_slug {
        let text = entry.source.short.as_deref().unwrap_or(entry.source.name.as_str());
        let fitted = fit_slug(ui.painter(), text, (size - 10.0).max(10.0));
        let slug_pos = egui::pos2(img_rect.min.x + 2.0, img_rect.max.y - 15.0);
        badge(ui.painter(), slug_pos, &fitted, pack_color(&entry.source.name));
    }

    let short = entry.name.rsplit('/').next().unwrap_or(&entry.name);
    ui.painter().text(egui::pos2(rect.center().x, rect.max.y - 9.0), egui::Align2::CENTER_CENTER, short, egui::FontId::proportional(11.0), theme::GRAY_6);
    resp
}

fn model_hover_text(entry: &crate::models::ModelEntry) -> String {
    let mut text = format!("{}\n{}", entry.name, entry.path.display());
    let src = &entry.source;
    text.push_str(&format!("\n\nPack: {}", src.name));
    if let Some(author) = &src.author {
        text.push_str(&format!("\nAuthor: {author}"));
    }

    if let Some(license) = &src.license {
        text.push_str(&format!("\nLicense: {license}"));
    }

    if let Some(url) = &src.url {
        text.push_str(&format!("\n{url}"));
    }

    if let Some(credit) = &entry.credit {
        text.push_str("\n\nModel credit:");
        if let Some(author) = &credit.author {
            text.push_str(&format!(" {author}"));
        }

        if let Some(url) = &credit.url {
            text.push_str(&format!(" ({url})"));
        }
    }

    text.push_str("\n\nDrag into a view to place it as an editable mesh");
    text
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum ModelSort {
    Name,
    Source,
    #[default]
    Folder,
    Recent,
}

impl ModelSort {
    const ALL: [ModelSort; 4] = [ModelSort::Name, ModelSort::Source, ModelSort::Folder, ModelSort::Recent];

    fn label(self) -> &'static str {
        match self {
            ModelSort::Name => "Name",
            ModelSort::Source => "Source",
            ModelSort::Folder => "Folder",
            ModelSort::Recent => "Recent",
        }
    }
}

fn sort_entries(entries: &mut [crate::models::ModelEntry], sort: ModelSort) {
    match sort {
        ModelSort::Name => entries.sort_by(|a, b| a.name.cmp(&b.name)),
        ModelSort::Folder => entries.sort_by(|a, b| (a.folder.as_str(), a.name.as_str()).cmp(&(b.folder.as_str(), b.name.as_str()))),
        ModelSort::Source => entries.sort_by(|a, b| (a.source.name.as_str(), a.name.as_str()).cmp(&(b.source.name.as_str(), b.name.as_str()))),
        ModelSort::Recent => entries.sort_by_key(|e| std::cmp::Reverse(e.mtime)),
    }
}

/// A row of the virtualised model grid: either a grid's worth of cells, or, in a grouped view, a header
/// naming the group. Headers are shorter than a row of thumbnails, so the grid lays itself out with
/// `show_viewport` and manually computed row offsets rather than egui's fixed-row-height `show_rows`.
enum ModelRow {
    /// `compact` headers (the folder view) are just the path, no dot, license or count: a folder is its
    /// own explanation, unlike a pack, which needs the extra context to mean anything.
    Header {
        name: String,
        license: Option<String>,
        count: usize,
        compact: bool,
    },
    Cells(std::ops::Range<usize>),
}

/// A header only needs one line of text, unlike a cell row which is as tall as the current thumbnail size.
const MODEL_HEADER_H: f32 = 22.0;

fn model_row_height(row: &ModelRow, cell_h: f32) -> f32 {
    match row {
        ModelRow::Header { .. } => MODEL_HEADER_H,
        ModelRow::Cells(_) => cell_h,
    }
}

/// Splits sorted `entries` into grid rows, grouping by pack or folder with a header row per group when the
/// sort calls for it. `entries` must already be sorted by `sort_entries` with the same `sort`.
fn model_rows(entries: &[crate::models::ModelEntry], sort: ModelSort, cols: usize) -> Vec<ModelRow> {
    let mut rows = Vec::new();
    let grouped = matches!(sort, ModelSort::Source | ModelSort::Folder);
    if !grouped {
        let mut i = 0;
        while i < entries.len() {
            let end = (i + cols).min(entries.len());
            rows.push(ModelRow::Cells(i..end));
            i = end;
        }

        return rows;
    }

    fn group_key(e: &crate::models::ModelEntry, sort: ModelSort) -> &str {
        if sort == ModelSort::Source { &e.source.name } else { &e.folder }
    }

    let mut start = 0;
    while start < entries.len() {
        let group = group_key(&entries[start], sort);
        let mut end = start;
        while end < entries.len() && group_key(&entries[end], sort) == group {
            end += 1;
        }

        let compact = sort == ModelSort::Folder;
        let name = if group.is_empty() {
            "(root)".to_string()
        } else if compact {
            format!("{group}/")
        } else {
            group.to_string()
        };
        let license = if sort == ModelSort::Source { entries[start].source.license.clone() } else { None };
        rows.push(ModelRow::Header { name, license, count: end - start, compact });
        let mut i = start;
        while i < end {
            let chunk_end = (i + cols).min(end);
            rows.push(ModelRow::Cells(i..chunk_end));
            i = chunk_end;
        }

        start = end;
    }

    rows
}

pub fn model_browser(ui: &mut Ui, state: &mut EditorState, ps: &mut PanelState, actions: &mut Vec<Action>, mut renderer: Option<&mut Renderer>) {
    let ctx = ui.ctx().clone();
    ui.horizontal_wrapped(|ui| {
        ui.add(egui::TextEdit::singleline(&mut ps.model_filter).hint_text("Search models").desired_width(160.0));
        let folders = state.model_library.folders();
        egui::ComboBox::from_id_salt("model_folder").selected_text(ps.model_folder.clone().unwrap_or_else(|| "All folders".into())).show_ui(ui, |ui| {
            ui.selectable_value(&mut ps.model_folder, None, "All folders");
            for f in folders {
                ui.selectable_value(&mut ps.model_folder, Some(f.clone()), if f.is_empty() { "(root)".to_string() } else { f });
            }
        });
        let sources = state.model_library.sources();
        egui::ComboBox::from_id_salt("model_source").selected_text(ps.model_source_filter.clone().unwrap_or_else(|| "All sources".into())).show_ui(ui, |ui| {
            ui.selectable_value(&mut ps.model_source_filter, None, "All sources");
            for s in sources {
                ui.selectable_value(&mut ps.model_source_filter, Some(s.clone()), s);
            }
        });
        ui.separator();
        ui.label("Sort");
        egui::ComboBox::from_id_salt("model_sort").selected_text(ps.model_sort.label()).show_ui(ui, |ui| {
            for s in ModelSort::ALL {
                ui.selectable_value(&mut ps.model_sort, s, s.label());
            }
        });
        ui.checkbox(&mut ps.model_show_slugs, "Slugs").on_hover_text("Show a small pack label on each thumbnail");
        ui.separator();
        ui.add(egui::Slider::new(&mut ps.thumb_size, 40.0..=160.0).show_value(false));
        ui.checkbox(&mut state.prefs.model_import.autofit, "Fit")
            .on_hover_text("Scale tiny or huge models to a usable size when placed. Real-world-scale assets are otherwise sub-grid specks.");
        ui.add(egui::DragValue::new(&mut state.prefs.model_import.scale).speed(0.05).range(0.01..=1000.0).prefix("×").fixed_decimals(2))
            .on_hover_text("Size multiplier applied on top of Fit (or on the model's real size when Fit is off)");
        if ui.small_button("⟳").on_hover_text("Rescan res://models for model files").clicked() {
            actions.push(Action::ReloadModels);
        }

        if ui.small_button("Import…").on_hover_text("Pick a model file to place, from anywhere on disk").clicked() {
            actions.push(Action::ImportModel(crate::commands::ModelImport::Mesh));
        }
    });
    ui.separator();

    let filter = ps.model_filter.to_lowercase();
    let mut entries: Vec<crate::models::ModelEntry> = state
        .model_library
        .entries
        .iter()
        .filter(|e| filter.is_empty() || e.name.to_lowercase().contains(&filter))
        .filter(|e| ps.model_folder.as_ref().is_none_or(|f| &e.folder == f))
        .filter(|e| ps.model_source_filter.as_ref().is_none_or(|s| &e.source.name == s))
        .cloned()
        .collect();
    sort_entries(&mut entries, ps.model_sort);

    if state.model_library.entries.is_empty() {
        ui.label(
            RichText::new(
                "No models found. Put .glb, .gltf, .obj, .bbmodel, .stl, .md2 or .md3 files under res://models, or use Import… to place one from anywhere.",
            )
            .weak(),
        );
    } else if entries.is_empty() {
        ui.label(RichText::new("No models match the search and filters").weak());
    }

    let size = ps.thumb_size;
    let cell = Vec2::new(size + 8.0, size + 22.0);
    let cols = ((ui.available_width() / cell.x).floor() as usize).max(1);
    let rows = model_rows(&entries, ps.model_sort, cols);
    let heights: Vec<f32> = rows.iter().map(|r| model_row_height(r, cell.y)).collect();
    let gap = ui.spacing().item_spacing.y;
    let mut offsets = Vec::with_capacity(heights.len());
    let mut y = 0.0;
    for h in &heights {
        offsets.push(y);
        y += h + gap;
    }

    let total_h = (y - gap).max(0.0);
    let cursor = state.cursor_world;
    let show_slugs = ps.model_show_slugs;
    ScrollArea::vertical().auto_shrink([false, false]).show_viewport(ui, |ui, viewport| {
        ui.set_height(total_h);
        ui.set_width(ui.available_width());

        // Only the rows whose span overlaps the visible viewport are laid out, so a grouped view with
        // many packs stays as cheap to scroll as the flat grid.
        let mut first = rows.len();
        for (i, (&o, &h)) in offsets.iter().zip(&heights).enumerate() {
            if o + h > viewport.min.y {
                first = i;
                break;
            }
        }

        let mut last = first;
        for (i, &o) in offsets.iter().enumerate().skip(first) {
            if o >= viewport.max.y {
                break;
            }

            last = i + 1;
        }

        let top = ui.max_rect().top();
        for (i, (&o, &h)) in offsets.iter().zip(&heights).enumerate().take(last).skip(first) {
            let row_rect = egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), (top + o)..=(top + o + h));
            ui.scope_builder(egui::UiBuilder::new().max_rect(row_rect).id_salt(i), |ui| match &rows[i] {
                ModelRow::Header { name, compact: true, .. } => {
                    ui.horizontal(|ui| {
                        ui.add_space(4.0);
                        ui.label(RichText::new(name).weak().small());
                    });
                }
                ModelRow::Header { name, license, count, .. } => {
                    ui.horizontal(|ui| {
                        ui.add_space(4.0);
                        let dot = ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover()).0;
                        ui.painter().circle_filled(dot.center(), 4.0, pack_color(name));
                        ui.label(RichText::new(name).strong());
                        if let Some(license) = license {
                            ui.label(RichText::new(license).weak());
                        }

                        ui.label(RichText::new(format!("{count} model{}", if *count == 1 { "" } else { "s" })).weak());
                    });
                }
                ModelRow::Cells(range) => {
                    ui.horizontal(|ui| {
                        for entry in &entries[range.clone()] {
                            let units = state.game.units_per_meter;
                            let thumb = state.model_thumbs.thumbnail(&ctx, renderer.as_deref_mut(), &mut state.models, units, &entry.path);
                            let resp = model_cell(ui, false, entry, size, show_slugs, thumb);
                            let resp = resp.on_hover_text(model_hover_text(entry));
                            if resp.drag_started() {
                                resp.dnd_set_drag_payload(DndPayload::Model(entry.path.clone()));
                            }

                            resp.context_menu(|ui| {
                                if ui.button("Place at cursor").clicked() {
                                    let at = state.snap(cursor.unwrap_or(DVec3::ZERO));
                                    actions.push(Action::PlaceModel { path: entry.path.clone(), at });
                                    ui.close();
                                }

                                if ui.button("Copy path").clicked() {
                                    ui.ctx().copy_text(entry.path.display().to_string());
                                    ui.close();
                                }

                                if let Some(url) = &entry.source.url
                                    && ui.button("Open pack page").clicked()
                                {
                                    ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                                    ui.close();
                                }
                            });
                        }
                    });
                }
            });
        }
    });
}

/// Follows the pointer while a material or entity is dragged. Views set the copy cursor where a drop works.
pub fn dnd_preview(ctx: &egui::Context, state: &mut EditorState) {
    let Some(payload) = egui::DragAndDrop::payload::<DndPayload>(ctx) else { return };
    let Some(pointer) = ctx.pointer_latest_pos() else { return };
    if ctx.output(|o| o.cursor_icon) == egui::CursorIcon::Default {
        ctx.set_cursor_icon(egui::CursorIcon::NoDrop);
    }

    egui::Area::new(egui::Id::new("dnd_preview")).order(egui::Order::Tooltip).interactable(false).fixed_pos(pointer + Vec2::new(18.0, 14.0)).show(ctx, |ui| {
        egui::Frame::popup(ui.style()).show(ui, |ui| {
            ui.horizontal(|ui| match payload.as_ref() {
                DndPayload::Material(name) => {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(40.0), Sense::hover());
                    match state.materials.thumbnail(ctx, name, &mut 1) {
                        Some(tex) => {
                            ui.painter().image(tex.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::WHITE);
                        }
                        None => {
                            ui.painter().rect_filled(rect, 2.0, theme::GRAY_2);
                        }
                    }

                    ui.vertical(|ui| {
                        ui.label(RichText::new(name).strong());
                        ui.label(RichText::new("Drop on a face, Shift covers the whole brush, Alt drops a decal").weak().small());
                    });
                }
                DndPayload::Entities(classnames) => {
                    for classname in classnames.iter().take(4) {
                        let def = state.game.entity(classname);
                        let (rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::hover());
                        paint_entity_tile(ui, rect, def);
                    }

                    ui.vertical(|ui| match classnames.as_slice() {
                        [one] => {
                            ui.label(RichText::new(one).strong());
                            ui.label(RichText::new("Drop into a view to place it").weak().small());
                        }
                        many => {
                            ui.label(RichText::new(format!("{} entities", many.len())).strong());
                            ui.label(RichText::new("Drop into a view to place them in a row").weak().small());
                        }
                    });
                }
                DndPayload::Model(path) => {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::hover());
                    ui.painter().rect_filled(rect, 3.0, theme::GRAY_2);
                    ui.put(rect, icons::MESH.image(18.0).tint(theme::TEAL));
                    let name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
                    ui.vertical(|ui| {
                        ui.label(RichText::new(name).strong());
                        ui.label(RichText::new("Drop into a view to place it as an editable mesh").weak().small());
                    });
                }
            });
        });
    });
}

// ------------------------------------------------------------------ entities

fn entity_color(def: &EntityDef) -> Color32 {
    let c = def.color;
    Color32::from_rgb((c.r * 255.0) as u8, (c.g * 255.0) as u8, (c.b * 255.0) as u8)
}

fn entity_icon(def: &EntityDef) -> icons::Icon {
    let name = def.classname.as_str();
    if crate::scene::is_decal(Some(def)) {
        icons::PAINT
    } else if name.contains("light") {
        icons::SHADE_LIT
    } else if name.contains("door") || name.contains("gate") {
        icons::DOOR
    } else if name.starts_with("trigger") || name.ends_with("_area") {
        icons::VOLUME
    } else if name.contains("path") {
        icons::PATH
    } else if name.contains("prop") || name.contains("model") || !def.model.is_empty() || !def.scene.is_empty() {
        icons::INSTANCE
    } else if def.kind == gt_formats::EntityKind::Solid {
        icons::BRUSH
    } else {
        icons::ENTITY
    }
}

fn paint_entity_tile(ui: &Ui, rect: egui::Rect, def: Option<&EntityDef>) {
    let color = def.map(entity_color).unwrap_or(theme::GRAY_6);
    ui.painter().rect(rect, 3.0, color.gamma_multiply(0.22), egui::Stroke::new(1.0, color.gamma_multiply(0.7)), egui::StrokeKind::Inside);
    let icon = def.map(entity_icon).unwrap_or(icons::ENTITY);
    icon.image(rect.width() * 0.5).tint(color).paint_at(ui, egui::Rect::from_center_size(rect.center(), Vec2::splat(rect.width() * 0.5)));
}

fn entity_card(ui: &mut Ui, def: &EntityDef, size: f32, selected: bool) -> egui::Response {
    let cell = Vec2::new(size + 8.0, size + 22.0);
    let (rect, resp) = ui.allocate_exact_size(cell, Sense::click_and_drag());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::SelectableLabel, true, selected, &def.classname));
    if selected {
        ui.painter().rect_filled(rect, 3.0, theme::selected_fill());
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, 3.0, theme::GRAY_2);
    }

    let tile = egui::Rect::from_min_size(rect.min + Vec2::splat(4.0), Vec2::splat(size));
    paint_entity_tile(ui, tile, Some(def));
    if def.kind == gt_formats::EntityKind::Solid {
        ui.painter().text(tile.right_top() + Vec2::new(-3.0, 2.0), egui::Align2::RIGHT_TOP, "brush", egui::FontId::proportional(10.0), theme::GRAY_5);
    }

    let mut job = egui::text::LayoutJob::simple_singleline(def.classname.clone(), egui::FontId::proportional(11.0), theme::GRAY_6);
    job.wrap = egui::text::TextWrapping::truncate_at_width(cell.x - 4.0);
    let label = ui.painter().layout_job(job);
    ui.painter().galley(egui::pos2(rect.center().x, rect.max.y - 9.0) - label.size() / 2.0, label, theme::GRAY_6);
    resp
}

pub fn entity_browser(ui: &mut Ui, state: &mut EditorState, ps: &mut PanelState, actions: &mut Vec<Action>) {
    ui.horizontal_wrapped(|ui| {
        ui.add(egui::TextEdit::singleline(&mut ps.entity_filter).hint_text("Search entities").desired_width(160.0));
        ui.add(egui::Slider::new(&mut ps.entity_card_size, 32.0..=128.0).show_value(false)).on_hover_text("Card size");
        if !ps.entity_selection.is_empty() {
            let n = ps.entity_selection.len();
            if ui
                .button(if n == 1 { "Place".to_string() } else { format!("Place {n}") })
                .on_hover_text("Place the selected entities at the 3D cursor")
                .clicked()
            {
                actions.push(place_action(state, ps.entity_selection.clone()));
            }

            if ui.small_button("Clear").clicked() {
                ps.entity_selection.clear();
            }
        }
    });
    ui.label(
        RichText::new("Drag into a view to place, Ctrl or Shift click selects several and drags them together. Double click places at the cursor.").weak(),
    );
    ui.separator();
    let filter = ps.entity_filter.to_lowercase();
    let mut groups: Vec<(String, Vec<&EntityDef>)> = Vec::new();
    for def in state.game.entities.iter().filter(|d| d.classname != "worldspawn") {
        if !filter.is_empty() && !def.classname.to_lowercase().contains(&filter) && !def.description.to_lowercase().contains(&filter) {
            continue;
        }

        let group = if def.group.is_empty() { "other".to_string() } else { def.group.clone() };
        match groups.iter_mut().find(|(g, _)| *g == group) {
            Some((_, list)) => list.push(def),
            None => groups.push((group, vec![def])),
        }
    }

    if state.game.entities.is_empty() {
        ui.label(RichText::new("No entity definitions. Open a Godot project with a game config (Godot > Open Godot Project…).").weak());
    } else if groups.is_empty() {
        ui.label(RichText::new("No entities match the search").weak());
    }

    ps.entity_selection.retain(|c| state.game.entity(c).is_some());
    let order: Vec<String> = groups.iter().flat_map(|(_, defs)| defs.iter().map(|d| d.classname.clone())).collect();
    let size = ps.entity_card_size;
    let cell_width = size + 8.0 + ui.spacing().item_spacing.x;
    let mut clicked: Option<(String, egui::Modifiers)> = None;
    let mut placed: Option<String> = None;
    let mut dragged: Option<(egui::Response, String)> = None;
    let mut to_reference: Option<String> = None;
    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        for (group, defs) in &groups {
            egui::CollapsingHeader::new(group).default_open(true).show(ui, |ui| {
                let cols = ((ui.available_width() / cell_width).floor() as usize).max(1);
                for chunk in defs.chunks(cols) {
                    ui.horizontal(|ui| {
                        for def in chunk {
                            let selected = ps.entity_selection.contains(&def.classname);
                            let kind = if def.kind == gt_formats::EntityKind::Solid { "brush" } else { "point" };
                            let resp = entity_card(ui, def, size, selected);
                            let resp = resp.on_hover_text(format!("{}\n{kind} entity\n{}", def.classname, def.description));
                            if resp.double_clicked() {
                                placed = Some(def.classname.clone());
                            } else if resp.clicked() {
                                clicked = Some((def.classname.clone(), ui.input(|i| i.modifiers)));
                            }

                            if resp.drag_started() {
                                dragged = Some((resp.clone(), def.classname.clone()));
                            }

                            resp.context_menu(|ui| {
                                if ui.button("Place at cursor").clicked() {
                                    actions.push(place_action(state, vec![def.classname.clone()]));
                                    ui.close();
                                }

                                if def.kind == gt_formats::EntityKind::Solid && ui.button("Turn selected brushes into it").clicked() {
                                    actions.push(Action::CreateBrushEntity(def.classname.clone()));
                                    ui.close();
                                }

                                if ui.button("Show in Reference").clicked() {
                                    to_reference = Some(def.classname.clone());
                                    ui.close();
                                }

                                if ui.button("Copy classname").clicked() {
                                    ui.ctx().copy_text(def.classname.clone());
                                    ui.close();
                                }
                            });
                        }
                    });
                }
            });
        }
    });
    if let Some((classname, modifiers)) = clicked {
        select_entity_card(ps, &order, classname, modifiers);
    }

    if let Some(classname) = placed {
        let classnames = if ps.entity_selection.contains(&classname) { ps.entity_selection.clone() } else { vec![classname] };
        actions.push(place_action(state, classnames));
    }

    if let Some((resp, classname)) = dragged {
        if !ps.entity_selection.contains(&classname) {
            ps.entity_selection = vec![classname.clone()];
            ps.entity_anchor = Some(classname);
        }

        resp.dnd_set_drag_payload(DndPayload::Entities(ps.entity_selection.clone()));
    }

    if let Some(classname) = to_reference {
        ps.reference_class = classname;
        actions.push(Action::ShowReference);
    }
}

/// Plain click picks one card, Ctrl toggles it and Shift extends from the last clicked card in display order.
pub fn select_entity_card(ps: &mut PanelState, order: &[String], classname: String, modifiers: egui::Modifiers) {
    let anchor = ps.entity_anchor.as_ref().and_then(|a| order.iter().position(|c| c == a));
    match (modifiers.shift, anchor, order.iter().position(|c| *c == classname)) {
        (true, Some(from), Some(to)) => {
            let range = &order[from.min(to)..=from.max(to)];
            if !modifiers.command {
                ps.entity_selection.clear();
            }

            for c in range {
                if !ps.entity_selection.contains(c) {
                    ps.entity_selection.push(c.clone());
                }
            }

            return;
        }
        _ if modifiers.command => {
            if let Some(i) = ps.entity_selection.iter().position(|c| *c == classname) {
                ps.entity_selection.remove(i);
            } else {
                ps.entity_selection.push(classname.clone());
            }
        }
        _ => ps.entity_selection = vec![classname.clone()],
    }

    ps.entity_anchor = Some(classname);
}

/// A lone brush entity wraps the selected brushes when there are any, everything else is placed at the cursor.
fn place_action(state: &EditorState, classnames: Vec<String>) -> Action {
    let has_brushes = state.doc.selection.geometry(&state.doc.map).iter().any(|id| state.doc.map.terrain(*id).is_none());
    match classnames.as_slice() {
        [one] if has_brushes && state.game.entity(one).is_some_and(|d| d.kind == gt_formats::EntityKind::Solid) => Action::CreateBrushEntity(one.clone()),
        _ => Action::PlaceEntities { classnames, at: None, normal: None, row: DVec3::X },
    }
}

// -------------------------------------------------------------------- issues

fn collect_issues(state: &EditorState) -> Vec<gt_doc::issues::Issue> {
    use gt_doc::issues::{Issue, Severity};
    let mut list = gt_doc::issues::check_with(&state.doc.map, |class| {
        state.game.entity(class).map(|d| gt_doc::issues::ClassIo {
            outputs: d.outputs.iter().map(|o| o.name.as_str()).collect(),
            inputs: d.inputs.iter().map(|i| i.name.as_str()).collect(),
        })
    });
    for (id, e) in state.doc.map.entities() {
        if state.game.entity(&e.classname).is_none() && !e.classname.is_empty() {
            list.push(Issue {
                node: Some(id),
                severity: Severity::Warning,
                code: "unknown_class",
                message: format!("No entity definition for '{}'", e.classname),
            });
        }
    }

    list.sort_by_key(|i| std::cmp::Reverse(i.severity));
    list
}

pub fn issues(ui: &mut Ui, state: &mut EditorState, ps: &mut PanelState, actions: &mut Vec<Action>) {
    use gt_doc::issues::{self, Severity};
    if ps.issues_revision != state.doc.revision {
        ps.issues = collect_issues(state);
        ps.issues_revision = state.doc.revision;
    }

    let errors = ps.issues.iter().filter(|i| i.severity == Severity::Error).count();
    let warnings = ps.issues.iter().filter(|i| i.severity == Severity::Warning).count();
    let fixable = ps.issues.iter().filter(|i| issues::fixable(i.code)).count();
    let mut fix: Vec<gt_doc::issues::Issue> = Vec::new();
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{errors} errors")).color(theme::ERROR));
        ui.label(RichText::new(format!("{warnings} warnings")).color(theme::WARNING));
        ui.label(format!("{} total", ps.issues.len()));
        if ui.add_enabled(fixable > 0, egui::Button::new(format!("Fix all ({fixable})"))).clicked() {
            fix = ps.issues.iter().filter(|i| issues::fixable(i.code)).cloned().collect();
        }
    });
    ui.separator();
    if ps.issues.is_empty() {
        ui.label(RichText::new("No problems found").color(theme::SUCCESS));
    }

    let mut select: Option<NodeId> = None;
    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        for issue in &ps.issues {
            ui.horizontal(|ui| {
                let (icon, color) = match issue.severity {
                    Severity::Error => ("⛔", theme::ERROR),
                    Severity::Warning => ("⚠", theme::WARNING),
                    Severity::Info => ("ℹ", theme::INFO),
                };
                ui.label(RichText::new(icon).color(color));
                let name = issue.node.and_then(|n| state.doc.map.get(n)).map(|n| n.name()).unwrap_or_default();
                if ui.selectable_label(false, format!("{}  {}", issue.message, RichText::new(name).weak().text())).clicked() {
                    select = issue.node;
                }

                if issues::fixable(issue.code) && ui.small_button("Fix").clicked() {
                    fix.push(issue.clone());
                }
            });
        }
    });
    if let Some(id) = select.filter(|id| state.doc.map.contains(*id)) {
        let target = state.doc.map.selection_target(id, &state.open_groups);
        state.doc.select(|_, s| {
            s.clear();
            s.select_node(target);
        });
        actions.push(Action::FocusSelection);
    }

    if !fix.is_empty() {
        let material = state.current_material.clone();
        state.doc.edit("Fix Issues", |m, _| {
            for issue in &fix {
                issues::fix(m, issue, &material);
            }
        });
    }
}

// ----------------------------------------------------------------- reference

/// Short usage notes for every tool, shown in the reference panel.
pub const TOOL_HELP: [(&str, &str); 14] = [
    (
        "Select",
        "Click selects the object, double click its whole group. Drag empty space draws a brush, drag the selection moves it (Alt vertical, Ctrl duplicates). In 3D the gizmo arrows, squares, rings and boxes move, rotate and scale. Entity gizmo handles (hinges, travel, radius) drag here.",
    ),
    ("Clip", "Click two or three points, Tab picks the side to keep, Enter clips."),
    ("Vertex", "Drag brush vertices, edge and face midpoints split. Del removes vertices."),
    ("Rotate", "Drag a ring, snaps to 15 degrees, Shift for 1 degree."),
    ("Scale", "Drag bounds handles, Alt scales symmetrically."),
    ("Mesh", "Blender style: 1/2/3 component modes, G/R/S, E extrude, I inset, Ctrl+R loop cut, K knife, double click a vertex for the gizmo."),
    ("Sculpt", "Raise, lower, smooth, flatten, noise and terrace terrains and displacements. Shift inverts, Ctrl smooths, Ctrl+wheel radius."),
    ("Blend", "Paint, erase, smooth, sharpen, noise, slope and height blends on terrain layers, displacement alpha and faces with a blend material."),
    ("Paint", "Vertex colors on brush faces."),
    (
        "Scatter",
        "Paints the enabled models of the active scatter set, listed in the Scatter panel. Shift erases, Alt+click or the eyedropper toggles a target, Ctrl+wheel radius.",
    ),
    ("Volume", "Drag out trigger, spawn, hurt, teleport and push volumes as brush entities."),
    ("Path", "Click to chain path_corner entities, Enter finishes."),
    ("Measure", "Drag or click two points."),
    ("Texture", "Right click applies, Alt+right wraps, Alt+click picks, drag slides, Ctrl+wheel scales, Alt+wheel rotates."),
];

pub fn tool_help(tool: crate::tools::ToolKind) -> &'static str {
    TOOL_HELP.iter().find(|(name, _)| *name == tool.label()).map(|(_, help)| *help).unwrap_or_default()
}

pub fn reference(ui: &mut Ui, state: &mut EditorState, ps: &mut PanelState, actions: &mut Vec<Action>) {
    use crate::code_refs;
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut ps.reference_filter).hint_text("Filter entities").desired_width(140.0));
        if let Some(root) = state.game.project_root.clone()
            && ui.small_button("Add C# helper to project").on_hover_text("Writes GodotTrench.cs, the bridge C# entities use").clicked()
        {
            let path = root.join("addons/func_godot/csharp/GodotTrench.cs");
            let result = std::fs::create_dir_all(path.parent().unwrap_or(&root)).and_then(|_| std::fs::write(&path, code_refs::CSHARP_HELPER));
            state.set_status(match result {
                Ok(()) => format!("Wrote {}", path.display()),
                Err(e) => format!("Could not write the C# helper: {e}"),
            });
        }
    });
    let filter = ps.reference_filter.to_lowercase();
    let defs: Vec<gt_formats::EntityDef> = state
        .game
        .entities
        .iter()
        .filter(|d| filter.is_empty() || d.classname.contains(&filter) || d.description.to_lowercase().contains(&filter))
        .cloned()
        .collect();
    if ps.reference_class.is_empty()
        && let Some(e) = state.doc.selection.nodes.iter().find_map(|id| state.doc.map.entity(*id))
    {
        ps.reference_class = e.classname.clone();
    }

    let area = ui.available_rect_before_wrap();
    let min_width = 80.0_f32.min(area.width() / 2.0);
    let max_width = (area.width() - 200.0).max(min_width);
    let width = ps
        .reference_list_width
        .unwrap_or_else(|| {
            let font = egui::TextStyle::Body.resolve(ui.style());
            let longest = defs.iter().map(|d| ui.painter().layout_no_wrap(d.classname.clone(), font.clone(), Color32::WHITE).size().x).fold(0.0, f32::max);
            (longest + ui.spacing().button_padding.x * 2.0 + ui.spacing().scroll.bar_width + 16.0).clamp(120.0, area.width() * 0.45)
        })
        .clamp(min_width, max_width);
    let split = area.left() + width;
    let handle = ui
        .interact(egui::Rect::from_x_y_ranges(split - 4.0..=split + 4.0, area.y_range()), ui.id().with("reference_split"), Sense::click_and_drag())
        .on_hover_cursor(egui::CursorIcon::ResizeHorizontal)
        .on_hover_text("Drag to resize, double click to fit the names");
    if handle.dragged()
        && let Some(pointer) = handle.interact_pointer_pos()
    {
        ps.reference_list_width = Some((pointer.x - area.left()).clamp(min_width, max_width));
    }

    if handle.double_clicked() {
        ps.reference_list_width = None;
    }

    let line = if handle.hovered() || handle.dragged() { ui.visuals().widgets.active.bg_stroke } else { ui.visuals().widgets.noninteractive.bg_stroke };
    ui.painter().vline(split, area.y_range(), line);

    let list_rect = egui::Rect::from_min_max(area.min, egui::pos2(split - 4.0, area.max.y));
    ui.scope_builder(egui::UiBuilder::new().max_rect(list_rect), |ui| {
        ui.set_clip_rect(list_rect.intersect(ui.clip_rect()));
        ScrollArea::vertical().id_salt("reference_list").auto_shrink([false, false]).show(ui, |ui| {
            let mut group = String::new();
            for d in &defs {
                if d.group != group {
                    group = d.group.clone();
                    ui.label(RichText::new(&group).strong());
                }

                if ui.selectable_label(ps.reference_class == d.classname, &d.classname).on_hover_text(&d.description).clicked() {
                    ps.reference_class = d.classname.clone();
                }
            }

            ui.separator();
            ui.label(RichText::new("Tools").strong());
            for (tool, help) in TOOL_HELP {
                ui.label(RichText::new(tool).color(theme::CYAN)).on_hover_text(help);
            }
        });
    });
    let detail_rect = egui::Rect::from_min_max(egui::pos2(split + 8.0, area.min.y), area.max);
    ui.scope_builder(egui::UiBuilder::new().max_rect(detail_rect), |ui| {
        ui.set_clip_rect(detail_rect.intersect(ui.clip_rect()));
        ScrollArea::vertical().id_salt("reference_detail").auto_shrink([false, false]).show(ui, |ui| reference_detail(ui, state, ps, actions));
    });
    ui.advance_cursor_after_rect(area);
}

fn reference_detail(ui: &mut Ui, state: &mut EditorState, ps: &mut PanelState, actions: &mut Vec<Action>) {
    use crate::code_refs::{self, CodeKind};
    let Some(def) = state.game.entity(&ps.reference_class).cloned() else {
        ui.label("Pick an entity to see how to use it from GDScript or C#.");
        for (tool, help) in TOOL_HELP {
            ui.label(RichText::new(tool).strong());
            ui.label(RichText::new(help).weak());
        }

        return;
    };
    ui.heading(&def.classname);
    ui.label(RichText::new(&def.description).weak());
    ui.label(format!(
        "{} entity, node {}",
        if def.kind == gt_formats::EntityKind::Solid { "brush" } else { "point" },
        if def.node_class.is_empty() { "-" } else { &def.node_class }
    ));
    if !def.script.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.label("script");
            ui.code(&def.script);
            if let Some(path) = state.game.resolve_res(&def.script).filter(|p| p.is_file())
                && ui.small_button("Open").clicked()
            {
                open_in_system(&path);
            }
        });
    }

    egui::CollapsingHeader::new(format!("Properties ({})", def.properties.len())).default_open(true).show(ui, |ui| {
        for p in &def.properties {
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(&p.name).strong());
                ui.label(format!("{:?} = {}", p.ty, p.default));
                ui.label(RichText::new(&p.description).weak());
            });
        }
    });
    egui::CollapsingHeader::new(format!("Inputs ({}) and outputs ({})", def.inputs.len(), def.outputs.len())).default_open(true).show(ui, |ui| {
        for i in &def.inputs {
            ui.label(format!("input  {}({})", i.name, i.parameter));
        }

        for o in &def.outputs {
            ui.label(format!("output {}({})", o.name, o.parameter));
        }
    });
    ui.horizontal_wrapped(|ui| {
        for (k, kind) in CodeKind::ALL.iter().enumerate() {
            ui.selectable_value(&mut ps.reference_kind, k, kind.label());
        }
    });
    let kind = CodeKind::ALL[ps.reference_kind.min(CodeKind::ALL.len() - 1)];
    let code = code_refs::generate(&def, kind);
    ui.horizontal(|ui| {
        if ui.button("Copy").clicked() {
            ui.ctx().copy_text(code.clone());
            state.set_status("Copied to the clipboard");
        }

        if matches!(kind, CodeKind::GdscriptClass | CodeKind::CsharpClass | CodeKind::FgdResource)
            && let Some(root) = state.game.project_root.clone()
            && ui.button("Create in project").on_hover_text("Writes the file under res://entities/ unless it exists").clicked()
        {
            let stem = if kind == CodeKind::CsharpClass { code_refs::pascal(&def.classname) } else { def.classname.clone() };
            let path = root.join("entities").join(format!("{stem}.{}", kind.extension()));
            let result = if path.exists() {
                Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "file exists"))
            } else {
                std::fs::create_dir_all(root.join("entities")).and_then(|_| std::fs::write(&path, &code))
            };
            state.set_status(match result {
                Ok(()) => format!("Wrote {}", path.display()),
                Err(e) => format!("{}: {e}", path.display()),
            });
        }

        if ui.button("Place").on_hover_text("Point entities at the cursor, brush entities from the selection").clicked() {
            actions.push(if def.kind == gt_formats::EntityKind::Point {
                Action::CreatePointEntity { classname: def.classname.clone(), at: None }
            } else {
                Action::CreateBrushEntity(def.classname.clone())
            });
        }
    });
    let mut text = code.as_str();
    ui.add(egui::TextEdit::multiline(&mut text).code_editor().desired_width(f32::INFINITY));
}

/// Opens a file with the operating system's default application.
pub fn open_in_system(path: &std::path::Path) {
    let mut cmd = if cfg!(windows) {
        let mut cmd = std::process::Command::new("cmd");
        cmd.args(["/C", "start", ""]);
        cmd
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open")
    } else {
        std::process::Command::new("xdg-open")
    };
    // stdout carries the JSON-RPC stream when MCP runs over stdio.
    use std::process::Stdio;
    let _ = cmd.arg(path).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
}

// ------------------------------------------------------------------- history

pub fn history(ui: &mut Ui, state: &mut EditorState) {
    let undo: Vec<String> = state.doc.history.undo_labels().map(str::to_string).collect();
    let redo: Vec<String> = state.doc.history.redo_labels().map(str::to_string).collect();
    let mut undo_steps = 0;
    let mut redo_steps = 0;
    if undo.is_empty() && redo.is_empty() {
        ui.label(RichText::new("No edits yet. Every change shows up here, click one to undo back to it.").weak());
        return;
    }

    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        for (i, label) in redo.iter().enumerate().rev() {
            if ui.selectable_label(false, RichText::new(label).weak()).clicked() {
                redo_steps = i + 1;
            }
        }

        ui.label(RichText::new("▶ current").strong());
        for (i, label) in undo.iter().enumerate() {
            if ui.selectable_label(false, label).clicked() {
                undo_steps = i + 1;
            }
        }
    });
    for _ in 0..undo_steps {
        state.undo();
    }

    for _ in 0..redo_steps {
        state.redo();
    }
}

// ------------------------------------------------------------------ logic preview

/// Fires an entity output and shows how the I/O cascades through the map, so a scene's logic can be checked in the
/// editor without launching Godot. See [crate::logic_sim].
pub fn logic_panel(ui: &mut Ui, state: &mut EditorState, ps: &mut PanelState) {
    use crate::logic_sim;

    let mut named: Vec<(NodeId, String, String)> = Vec::new();
    for (id, e) in state.doc.map.entities() {
        if let Some(name) = e.targetname() {
            named.push((id, name.to_string(), e.classname.clone()));
        }
    }

    named.sort_by(|a, b| a.1.cmp(&b.1));
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Logic preview").strong());
        ui.label(RichText::new("fire an output and watch the wiring cascade").weak());
    });
    ui.separator();
    if named.is_empty() {
        ui.label(RichText::new("No named entities yet. Give an entity a targetname and wire some outputs.").weak());
        return;
    }

    if !named.iter().any(|(id, ..)| Some(*id) == ps.logic_start) {
        ps.logic_start = Some(named[0].0);
        ps.logic_output.clear();
        ps.logic_result = None;
    }

    let start_id = ps.logic_start.expect("selection kept valid above");
    let start_label = named.iter().find(|(id, ..)| *id == start_id).map(|(_, n, c)| format!("{n} ({c})")).unwrap_or_default();
    ui.horizontal_wrapped(|ui| {
        ui.label("Entity");
        egui::ComboBox::from_id_salt("logic_entity").selected_text(start_label).show_ui(ui, |ui| {
            for (id, name, classname) in &named {
                if ui.selectable_label(Some(*id) == ps.logic_start, format!("{name} ({classname})")).clicked() {
                    ps.logic_start = Some(*id);
                    ps.logic_output.clear();
                    ps.logic_result = None;
                }
            }
        });
    });

    let outputs = state.doc.map.entity(start_id).map(|e| logic_sim::start_outputs(state.game.entity(&e.classname), e)).unwrap_or_default();
    if ps.logic_output.is_empty()
        && let Some(first) = outputs.first()
    {
        ps.logic_output = first.clone();
    }

    ui.horizontal_wrapped(|ui| {
        ui.label("Output");
        egui::ComboBox::from_id_salt("logic_output").selected_text(ps.logic_output.clone()).show_ui(ui, |ui| {
            for o in &outputs {
                if ui.selectable_label(&ps.logic_output == o, o).clicked() {
                    ps.logic_output = o.clone();
                    ps.logic_result = None;
                }
            }
        });
        if ui.add_enabled(!ps.logic_output.is_empty(), egui::Button::new("Fire")).clicked() {
            ps.logic_result = Some(logic_sim::simulate(&state.doc.map, start_id, &ps.logic_output));
        }

        if ps.logic_result.is_some() && ui.button("Clear").clicked() {
            ps.logic_result = None;
        }
    });

    ui.separator();
    let Some(result) = &ps.logic_result else {
        ui.label(RichText::new("Pick an entity and output, then Fire to preview the cascade.").weak());
        return;
    };

    if result.events.is_empty() {
        ui.label(RichText::new("That output is not wired to anything.").weak());
        return;
    }

    let broken = result.broken();
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("{} steps", result.events.len()));
        if broken > 0 {
            ui.label(RichText::new(format!("{broken} broken")).color(Color32::from_rgb(230, 90, 90)));
        }

        if result.truncated {
            ui.label(RichText::new("truncated, feedback loop").weak());
        }
    });

    ScrollArea::vertical().show(ui, |ui| {
        for ev in &result.events {
            ui.horizontal(|ui| {
                ui.add_space(ev.depth as f32 * 14.0);
                let mut text = format!("{}.{}  \u{2192}  {}.{}", ev.source_name, ev.output, ev.target, ev.input);
                if ev.delay > 0.0 {
                    text.push_str(&format!("  ({}s)", ev.delay));
                }

                let color = if ev.broken() {
                    Color32::from_rgb(230, 90, 90)
                } else if ev.dynamic {
                    Color32::from_rgb(150, 170, 210)
                } else {
                    ui.visuals().text_color()
                };
                ui.label(RichText::new(text).color(color)).on_hover_text(if ev.dynamic {
                    "runtime target, resolved when the map plays"
                } else if ev.broken() {
                    "no entity has this targetname"
                } else {
                    ""
                });
            });
        }
    });
}
