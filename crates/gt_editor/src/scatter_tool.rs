//! Scatter tool: radial painting and erasing of trees, rocks and foliage. A scatter set is its own palette: the brush
//! paints the set's enabled models onto the set's target surfaces, which can be brushes, meshes, terrains or the
//! instances of other sets. Instances live in the set on its own layer, or become point entities.

use egui::{Align2, Color32, FontId, PointerButton, Pos2, Rect, Response, Ui, Vec2};
use gt_core::{Aabb, DVec3, NodeId, Ray};
use gt_doc::scatter::{Rng, ScatterRules, SurfaceHit};
use gt_doc::{Entity, NodeKind, Scatter, ScatterItem, ScatterKind};
use gt_render::LineVertex;

use crate::camera::Camera;
use crate::picking::SurfaceCaster;
use crate::scene::v3;
use crate::state::{EditorState, ScatterOutput};

fn rgba(c: Color32, a: f32) -> [f32; 4] {
    [c.r() as f32 / 255.0, c.g() as f32 / 255.0, c.b() as f32 / 255.0, a]
}

fn line(out: &mut Vec<LineVertex>, a: DVec3, b: DVec3, color: [f32; 4]) {
    out.push(LineVertex { pos: v3(a), color });
    out.push(LineVertex { pos: v3(b), color });
}

/// Palette sources that are not model or scene files are entity classnames.
pub fn is_classname(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    !(crate::models::is_model_path(source) || lower.ends_with(".tscn") || lower.ends_with(".scn"))
}

/// Region a dab can place into, tall enough for casts from above hills and into valleys.
fn dab_region(center: DVec3, radius: f64) -> Aabb {
    Aabb::from_center_size(center, DVec3::new(radius * 2.0 + 64.0, radius * 4.0 + 4096.0, radius * 2.0 + 64.0))
}

/// The active scatter set if it still exists.
pub fn active_set(state: &EditorState) -> Option<NodeId> {
    state.active_scatter.filter(|id| state.doc.map.scatter(*id).is_some())
}

/// `base`, or `base 2`, `base 3` and so on when another set already has that name.
fn unique_name(state: &EditorState, base: &str) -> String {
    let taken = |n: &str| state.doc.map.scatters().any(|(_, s)| s.name == n);
    if !taken(base) {
        return base.to_string();
    }

    (2..).map(|k| format!("{base} {k}")).find(|n| !taken(n)).unwrap_or_else(|| base.to_string())
}

/// Creates a scatter set holding `items` on a new layer and makes it active.
pub fn create_set(state: &mut EditorState, name: &str, kind: ScatterKind, items: Vec<ScatterItem>) -> NodeId {
    let s = &state.prefs.scatter;
    let mut set = Scatter::new(name, kind, items);
    set.chunk_size = s.chunk_size;
    set.static_props_multimesh = s.static_props_multimesh;
    // None keeps the range Scatter::new picked from the kind: foliage fades out, props stay visible.
    if let Some(range) = s.visibility_range {
        set.visibility_range = range;
    }

    let id = state.doc.edit("New Scatter Set", |m, _| {
        let layer = m.add_layer(&format!("Scatter: {name}"));
        m.insert(layer, NodeKind::Scatter(set))
    });
    state.active_scatter = Some(id);
    id
}

/// Creates a set from the template in the scatter preferences (the last preset or script palette).
pub fn new_set(state: &mut EditorState, name: Option<&str>) -> NodeId {
    let s = &state.prefs.scatter;
    let label = name.map(str::to_string).unwrap_or_else(|| {
        if s.preset.is_empty() { s.palette.first().map(|i| i.label().to_string()).unwrap_or_else(|| "scatter".into()) } else { s.preset.clone() }
    });
    let (kind, items) = (s.kind, s.palette.clone());
    create_set(state, &label, kind, items)
}

/// Where a set starts when the template's preset needs the nature pack the project lacks. Its models are built into the
/// editor.
pub const FALLBACK_PRESET: &str = "low-poly trees";

/// The active set, or a new one from the template. A template that is still a preset gets its models the way picking the
/// preset does. When it needs the nature pack the project lacks, the set gets [`FALLBACK_PRESET`] and the status says
/// why, so painting works without asking each time.
pub fn active_or_new_set(state: &mut EditorState) -> Option<NodeId> {
    if let Some(id) = active_set(state) {
        return Some(id);
    }

    let preset = state.prefs.scatter.preset.clone();
    let unchanged = gt_doc::scatter::preset(&preset).is_some_and(|(_, items)| items == state.prefs.scatter.palette);
    if unchanged {
        match prepare_preset(state, &preset) {
            Ok(_) | Err(PresetProblem::Unknown) => {}
            Err(PresetProblem::NeedsPack) => {
                let id = new_set_from_preset(state, FALLBACK_PRESET)?;
                state.set_status(format!(
                    "The {preset} preset needs the nature pack, which this project does not have, so the set uses the {FALLBACK_PRESET} preset. Godot > Add Content to Project adds the pack"
                ));
                return Some(id);
            }
            Err(PresetProblem::Write(e)) => {
                state.set_status(format!("Could not add the models of the {preset} preset to the project: {e}"));
                return None;
            }
        }
    }

    Some(new_set(state, None))
}

/// An empty set to drop models into.
pub fn new_empty_set(state: &mut EditorState) -> NodeId {
    let name = unique_name(state, "scatter");
    create_set(state, &name, ScatterKind::Props, Vec::new())
}

/// A new set holding a built-in preset. The embedded Blockbench models it uses are written into the project when they
/// are missing. A preset of the downloadable nature pack in a project without it creates nothing, and asks for the
/// content wizard instead.
pub fn new_set_from_preset(state: &mut EditorState, preset: &str) -> Option<NodeId> {
    let installed = match prepare_preset(state, preset) {
        Ok(installed) => installed,
        Err(PresetProblem::NeedsPack) => {
            ask_for_pack(state, preset);
            return None;
        }
        Err(PresetProblem::Unknown) => return None,
        Err(PresetProblem::Write(e)) => {
            state.set_status(format!("Could not add the models of the {preset} preset to the project: {e}"));
            return None;
        }
    };
    apply_preset(state, preset);
    let name = unique_name(state, preset);
    let id = new_set(state, Some(&name));
    let dir = gt_doc::scatter::NATURE_DIR;
    state.set_status(match installed {
        0 => format!("New scatter set from the {preset} preset on its own layer"),
        n => format!("New scatter set from the {preset} preset on its own layer, its {n} missing models were added to {dir}"),
    });
    Some(id)
}

/// Says why a preset cannot be used yet and asks for the content wizard, which offers the nature pack the first time
/// in a session.
fn ask_for_pack(state: &mut EditorState, preset: &str) {
    let why = format!("The {preset} preset uses the glTF trees and bushes of the nature pack, which this project does not have yet.");
    state.set_status(format!("{why} Godot > Add Content to Project adds it, the other presets work without it"));
    state.content_request = Some(crate::state::ContentRequest::NaturePack(why));
}

#[derive(Debug, PartialEq)]
pub enum PresetProblem {
    Unknown,
    /// The preset uses models of the downloadable nature pack that the project lacks.
    NeedsPack,
    Write(String),
}

/// Makes sure the project has the models of a preset: the embedded ones it lacks are written, nothing else is. Returns
/// how many were written. Without a project nothing can be checked, and the preset is used as it is.
pub fn prepare_preset(state: &mut EditorState, preset: &str) -> Result<usize, PresetProblem> {
    let (_, items) = gt_doc::scatter::preset(preset).ok_or(PresetProblem::Unknown)?;
    let Some(root) = state.game.project_root.clone() else { return Ok(0) };
    let nature = crate::content::nature_dir(&root);
    let prefix = format!("{}/", gt_doc::scatter::NATURE_DIR);
    let missing: Vec<&str> = items.iter().filter_map(|i| i.source.strip_prefix(&prefix)).filter(|rel| !nature.join(rel).is_file()).collect();
    if missing.iter().any(|rel| gt_formats::nature::embedded(rel).is_none()) {
        return Err(PresetProblem::NeedsPack);
    }

    let names: Vec<&str> = missing.iter().filter_map(|rel| rel.strip_suffix(".bbmodel")).collect();
    if names.is_empty() {
        return Ok(0);
    }

    let written = gt_formats::nature::install(&nature, &names, false).map_err(|e| PresetProblem::Write(e.to_string()))?;
    refresh_models(state);
    Ok(written.len())
}

fn refresh_models(state: &mut EditorState) {
    state.models.clear();
    let game = state.game.clone();
    state.model_library.rescan(&game);
}

/// Adds models to a set, enabling the ones it already holds. Returns how many were new.
pub fn add_items(state: &mut EditorState, id: NodeId, items: Vec<ScatterItem>) -> usize {
    let mut added = 0;
    state.doc.edit("Add Scatter Models", |m, _| {
        if let Some(s) = m.scatter_mut(id) {
            for item in items {
                match s.items.iter_mut().find(|i| i.source == item.source) {
                    Some(existing) => existing.enabled = true,
                    None => {
                        s.items.push(ScatterItem { enabled: true, ..item });
                        added += 1;
                    }
                }
            }
        }
    });
    added
}

/// Makes `items` the models a set paints: new ones are added, listed ones take the given settings, the rest are
/// disabled so their instances stay.
pub fn set_palette(state: &mut EditorState, id: NodeId, items: &[ScatterItem]) {
    state.doc.edit("Scatter Models", |m, _| {
        if let Some(s) = m.scatter_mut(id) {
            let listed = s.merge_items(items);
            for (k, item) in s.items.iter_mut().enumerate() {
                item.enabled = listed.contains(&k);
            }
        }
    });
}

/// Renames a set together with the layer it was created on.
pub fn rename_set(state: &mut EditorState, id: NodeId, name: &str) {
    let _ = state.doc.try_edit("Rename Scatter Set", |m, _| if m.rename(id, name) { Ok(()) } else { Err(()) });
}

/// Deletes a set, and the layer it was created on once that is empty.
pub fn delete_set(state: &mut EditorState, id: NodeId) {
    let layer = state.doc.map.layer_of(id);
    state.doc.edit("Delete Scatter Set", |m, s| {
        m.remove(id);
        let empty = m.get(layer).is_some_and(|n| n.children.is_empty() && matches!(&n.kind, NodeKind::Layer(l) if l.name.starts_with("Scatter: ")));
        if empty && m.default_layer() != layer {
            m.remove(layer);
        }

        s.clear();
    });
    if state.active_scatter == Some(id) {
        state.active_scatter = None;
    }

    state.validate_insert_context();
}

/// Scatter sets that count as ground besides brushes, meshes and terrains. That is the targets of `set`, and, while
/// it has none or follows the cursor, every visible prop set with instances, so a stroke that starts on a scattered
/// rock paints onto the rocks. Foliage only becomes ground when picked as a target.
pub fn ground_sets(state: &EditorState, set: Option<NodeId>) -> Vec<NodeId> {
    let mut out = set.and_then(|id| state.doc.map.scatter(id)).map(|s| s.targets.clone()).unwrap_or_default();
    if out.is_empty() || !state.prefs.scatter.rules.only_targets {
        for (id, s) in state.doc.map.scatters() {
            if Some(id) != set && s.kind == ScatterKind::Props && !s.instances.is_empty() && !state.doc.map.is_hidden(id) && !out.contains(&id) {
                out.push(id);
            }
        }
    }

    out
}

/// Loads the models of these sets, so casts onto their instances hit the real shapes rather than standins.
fn load_models(state: &mut EditorState, sets: &[NodeId]) {
    let paths: Vec<std::path::PathBuf> = sets
        .iter()
        .filter_map(|id| state.doc.map.scatter(*id))
        .flat_map(|s| s.items.iter().filter_map(|i| crate::picking::item_model_path(&state.game, &i.source)))
        .collect();
    let upm = state.game.units_per_meter;
    for p in paths {
        state.models.get(&p, upm);
    }
}

/// The surface the scatter brush sits on along a ray, the same surfaces a stroke can paint onto.
pub fn cursor_hit(state: &EditorState, ray: &Ray) -> Option<SurfaceHit> {
    let ground = ground_sets(state, active_set(state));
    let region = Aabb::from_center_size(ray.origin, DVec3::splat(1.0e6));
    SurfaceCaster::with_targets(state, &region, &ground).cast(ray.origin, ray.dir)
}

/// What the target eyedropper picks along a ray: any brush, mesh, terrain or other scatter set, foliage included.
pub fn target_hit(state: &EditorState, ray: &Ray) -> Option<SurfaceHit> {
    let active = active_set(state);
    let sets: Vec<NodeId> =
        state.doc.map.scatters().filter(|(id, s)| Some(*id) != active && !s.instances.is_empty() && !state.doc.map.is_hidden(*id)).map(|(id, _)| id).collect();
    let region = Aabb::from_center_size(ray.origin, DVec3::splat(1.0e6));
    SurfaceCaster::with_targets(state, &region, &sets).cast(ray.origin, ray.dir)
}

/// Adds or removes `node` as a target of the active set, creating a set when there is none. Returns whether it was
/// added, or an error for nodes that cannot carry instances.
pub fn eyedrop_target(state: &mut EditorState, node: NodeId) -> Result<bool, String> {
    let is_ground = state.doc.map.get(node).is_some_and(|n| n.kind.is_geometry() || matches!(n.kind, NodeKind::Scatter(_)));
    if !is_ground {
        return Err(format!("{node} is not a surface, targets are brushes, meshes, terrains and scatter sets"));
    }

    if active_or_new_set(state).is_none() {
        return Err(state.status.clone());
    }

    if Some(node) == active_set(state) {
        return Err("A scatter set cannot be painted onto itself".into());
    }

    Ok(toggle_target(state, node))
}

/// Instances of other sets near `region`, so sets keep their spacing against each other. Sets in `ground` are what
/// the active set is painted onto, so they are left out.
fn other_footprints(state: &EditorState, skip: Option<NodeId>, ground: &[NodeId], region: &Aabb) -> Vec<(DVec3, f64)> {
    if !state.prefs.scatter.avoid_other_sets {
        return Vec::new();
    }

    state
        .doc
        .map
        .scatters()
        .filter(|(id, _)| Some(*id) != skip && !ground.contains(id) && !state.doc.map.is_hidden(*id))
        .flat_map(|(_, s)| s.footprints())
        .filter(|(p, _)| region.contains_point(*p))
        .collect()
}

/// Models the brush places: the active set's enabled ones, or the template when no set is active.
pub fn brush_items(state: &EditorState) -> Vec<ScatterItem> {
    match active_set(state).and_then(|id| state.doc.map.scatter(id)) {
        Some(set) => set.items.iter().filter(|i| i.enabled).cloned().collect(),
        None => state.prefs.scatter.palette.clone(),
    }
}

/// One paint dab at `center` on a surface facing `normal`. `under` is the surface the brush sits on: a set without
/// targets takes it as its first target. Returns the number of instances or entities placed.
pub fn paint(state: &mut EditorState, center: DVec3, normal: DVec3, under: Option<NodeId>, rng: &mut Rng) -> usize {
    let settings = state.prefs.scatter.clone();
    if settings.output == ScatterOutput::Entities {
        let items = brush_items(state);
        if items.is_empty() {
            state.set_status("Nothing to scatter, add models to the set in the Scatter panel");
            return 0;
        }

        return paint_entities(state, &items, center, normal, settings.radius, &settings.rules, rng).len();
    }

    let Some(id) = active_or_new_set(state) else { return 0 };
    let Some(mut set) = state.doc.map.scatter(id).cloned() else { return 0 };
    let enabled = set.enabled_items();
    if enabled.is_empty() {
        let why = if set.items.is_empty() { "has no models, drop some into it" } else { "has every model switched off" };
        state.set_status(format!("Scatter set '{}' {why} in the Scatter panel", set.name));
        return 0;
    }

    let region = dab_region(center, settings.radius);
    if settings.rules.only_targets && set.targets.is_empty() {
        let found = under.or_else(|| {
            let ground = ground_sets(state, Some(id));
            SurfaceCaster::with_targets(state, &region, &ground).cast(center + normal * settings.radius, -normal).map(|h| h.node)
        });
        if let Some(node) = found.filter(|n| *n != id) {
            set.targets.push(node);
        }
    }

    let ground = if settings.rules.only_targets { set.targets.clone() } else { ground_sets(state, Some(id)) };
    load_models(state, &ground);
    let others = other_footprints(state, Some(id), &ground, &region);
    let placed = {
        let caster = SurfaceCaster::with_targets(state, &region, &ground);
        let rules = ScatterRules { items: Some(enabled), ..settings.rules.clone() };
        set.paint(center, normal, settings.radius, &rules, rng, &others, |o, d| caster.cast(o, d))
    };
    state.doc.edit("Scatter", |m, _| {
        if let Some(slot) = m.scatter_mut(id) {
            *slot = set;
        }
    });
    placed
}

/// One erase dab on the active set, or on scattered entities in entity mode.
pub fn erase(state: &mut EditorState, center: DVec3, rng: &mut Rng) -> usize {
    let s = state.prefs.scatter.clone();
    if s.output == ScatterOutput::Entities {
        let items = brush_items(state);
        return erase_entities(state, &items, center, s.radius);
    }

    let Some(id) = active_set(state) else {
        state.set_status("No active scatter set to erase from, pick one in the Scatter panel");
        return 0;
    };
    let Some(mut set) = state.doc.map.scatter(id).cloned() else { return 0 };
    let only = s.erase_palette_only.then(|| set.enabled_items());
    let removed = set.erase(center, s.radius, only.as_deref(), s.erase_amount, rng);
    if removed > 0 {
        state.doc.edit("Erase Scatter", |m, _| {
            if let Some(slot) = m.scatter_mut(id) {
                *slot = set;
            }
        });
    }

    removed
}

/// Fills the whole area of the active set's targets (or the selected surfaces and sets) with its enabled models.
pub fn fill(state: &mut EditorState, rng: &mut Rng) -> Result<usize, String> {
    let settings = state.prefs.scatter.clone();
    let Some(id) = active_or_new_set(state) else { return Err(state.status.clone()) };
    let Some(mut set) = state.doc.map.scatter(id).cloned() else { return Err("scatter set vanished".into()) };
    if set.targets.is_empty() {
        let map = &state.doc.map;
        set.targets = state.doc.selection.geometry(map);
        set.targets.extend(state.doc.selection.nodes.iter().copied().filter(|n| *n != id && map.scatter(*n).is_some()));
    }

    if set.targets.is_empty() {
        return Err("Select the surfaces to fill, or pick targets with the eyedropper in the Scatter panel".into());
    }

    let enabled = set.enabled_items();
    if enabled.is_empty() {
        return Err(format!("Scatter set '{}' has no enabled models to fill with", set.name));
    }

    load_models(state, &set.targets);
    let everywhere = Aabb::from_center_size(DVec3::ZERO, DVec3::splat(1.0e7));
    let placed = {
        let caster = SurfaceCaster::with_targets(state, &everywhere, &set.targets);
        let region = caster.bounds_of(&set.targets);
        let others = other_footprints(state, Some(id), &set.targets, &region.expanded(256.0));
        let rules = ScatterRules { items: Some(enabled), only_targets: true, ..settings.rules.clone() };
        set.fill(&region, &rules, rng, &others, |o, d| caster.cast(o, d))
    };
    state.doc.edit("Fill Scatter", |m, _| {
        if let Some(slot) = m.scatter_mut(id) {
            *slot = set;
        }
    });
    Ok(placed)
}

/// Adds or removes a surface or another scatter set from the active set's targets.
pub fn toggle_target(state: &mut EditorState, node: NodeId) -> bool {
    let Some(id) = active_set(state) else { return false };
    if node == id {
        return false;
    }

    let mut added = false;
    state.doc.edit("Scatter Target", |m, _| {
        if let Some(s) = m.scatter_mut(id) {
            match s.targets.iter().position(|t| *t == node) {
                Some(i) => {
                    s.targets.remove(i);
                }
                None => {
                    s.targets.push(node);
                    added = true;
                }
            }
        }
    });
    added
}

/// Readable name of a target: the set's name for scatter sets, else the node's own name.
pub fn target_label(state: &EditorState, node: NodeId) -> String {
    match state.doc.map.get(node) {
        Some(n) => match &n.kind {
            NodeKind::Scatter(s) => format!("{} (set)", s.name),
            other => format!("{} {node}", other.type_name()),
        },
        None => format!("missing {node}"),
    }
}

fn entity_for(source: &str, prop_class: &str) -> Entity {
    if is_classname(source) {
        Entity::new(source)
    } else {
        let mut e = Entity::new(prop_class);
        e.properties.insert("model".into(), source.to_string());
        e
    }
}

fn matches_source(e: &Entity, source: &str, prop_class: &str) -> bool {
    if is_classname(source) { e.classname == source } else { e.classname == prop_class && e.property("model") == Some(source) }
}

/// Places `items` as point entities, keeping spacing to existing ones.
pub fn paint_entities(
    state: &mut EditorState,
    items: &[ScatterItem],
    center: DVec3,
    normal: DVec3,
    radius: f64,
    rules: &ScatterRules,
    rng: &mut Rng,
) -> Vec<NodeId> {
    let prop_class = state.prefs.scatter.prop_class.clone();
    let mut temp = Scatter::new("entities", ScatterKind::Props, items.to_vec());
    let region = dab_region(center, radius);
    let existing: Vec<(DVec3, f64)> = state
        .doc
        .map
        .entities()
        .filter_map(|(_, e)| items.iter().find(|p| matches_source(e, &p.source, &prop_class)).map(|p| (e.origin, p.spacing)))
        .filter(|(p, _)| region.contains_point(*p))
        .collect();
    let rules = ScatterRules { only_targets: false, items: None, ..rules.clone() };
    {
        let caster = SurfaceCaster::new(state, &region);
        temp.paint(center, normal, radius, &rules, rng, &existing, |o, d| caster.cast(o, d));
    }

    if temp.instances.is_empty() {
        return Vec::new();
    }

    let parent = state.insert_parent();
    state.doc.edit("Scatter Entities", |m, _| {
        temp.instances
            .iter()
            .map(|inst| {
                let item = &temp.items[inst.item as usize];
                let mut e = entity_for(&item.source, &prop_class);
                e.origin = inst.position;
                e.angles = inst.angles.map(|a| (a * 10.0).round() / 10.0);
                if (inst.scale - 1.0).abs() > 1e-3 && !is_classname(&item.source) {
                    e.properties.insert("scale".into(), format!("{:.2}", inst.scale));
                }

                m.insert(parent, NodeKind::Entity(e))
            })
            .collect()
    })
}

fn erase_entities(state: &mut EditorState, items: &[ScatterItem], center: DVec3, radius: f64) -> usize {
    let prop_class = state.prefs.scatter.prop_class.clone();
    let ids: Vec<NodeId> = state
        .doc
        .map
        .entities()
        .filter(|(id, e)| {
            state.doc.map.is_editable(*id) && (e.origin - center).length() <= radius && items.iter().any(|p| matches_source(e, &p.source, &prop_class))
        })
        .map(|(id, _)| id)
        .collect();
    if !ids.is_empty() {
        state.doc.edit("Erase Entities", |m, _| ids.iter().for_each(|id| m.remove(*id)));
    }

    ids.len()
}

/// Replaces a scatter set with one prop entity per instance, in a group on the set's layer.
pub fn bake_to_entities(state: &mut EditorState, id: NodeId) -> usize {
    let Some(set) = state.doc.map.scatter(id).cloned() else { return 0 };
    let parent = state.doc.map.get(id).and_then(|n| n.parent).unwrap_or(state.doc.map.default_layer());
    let prop_class = state.prefs.scatter.prop_class.clone();
    let n = set.instances.len();
    state.doc.edit("Scatter to Entities", |m, s| {
        let group = m.insert(parent, NodeKind::Group(gt_doc::Group::new(set.name.clone())));
        for inst in &set.instances {
            let Some(item) = set.items.get(inst.item as usize) else { continue };
            let mut e = entity_for(&item.source, &prop_class);
            e.origin = inst.position;
            e.angles = inst.angles.map(|a| (a * 10.0).round() / 10.0);
            if (inst.scale - 1.0).abs() > 1e-3 {
                e.properties.insert("scale".into(), format!("{:.2}", inst.scale));
            }

            m.insert(group, NodeKind::Entity(e));
        }

        m.remove(id);
        s.clear();
        s.select_node(group);
    });
    if state.active_scatter == Some(id) {
        state.active_scatter = None;
    }

    n
}

/// Writes the embedded Blockbench nature models into the project, the glTF part of the pack is a download. Returns the
/// files written.
pub fn install_nature(state: &mut EditorState, overwrite: bool) -> Result<Vec<std::path::PathBuf>, String> {
    let dir = state.game.resolve_res(gt_doc::scatter::NATURE_DIR).ok_or("Open a Godot project first, models are installed into it")?;
    let written = gt_formats::nature::install(&dir, &[], overwrite).map_err(|e| e.to_string())?;
    if !written.is_empty() {
        refresh_models(state);
    }

    Ok(written)
}

/// Makes a built-in preset the template new sets start from.
pub fn apply_preset(state: &mut EditorState, name: &str) -> bool {
    let Some((kind, items)) = gt_doc::scatter::preset(name) else { return false };
    let s = &mut state.prefs.scatter;
    s.palette = items;
    s.kind = kind;
    s.preset = name.to_string();
    if kind == ScatterKind::Foliage {
        s.rules.density = s.rules.density.max(4.0);
        s.radius = s.radius.min(256.0);
    }

    true
}

/// res:// source of a model file, or its plain path outside the project.
pub fn source_for_path(state: &EditorState, path: &std::path::Path) -> String {
    state.game.project_root.as_ref().and_then(|root| gt_formats::game::to_res_path(root, path)).unwrap_or_else(|| path.to_string_lossy().replace('\\', "/"))
}

/// Set entries for model files picked or dropped at once.
pub fn items_from_paths(state: &EditorState, paths: &[std::path::PathBuf]) -> Vec<ScatterItem> {
    paths.iter().map(|p| ScatterItem::new(source_for_path(state, p))).collect()
}

/// Adds model files to the active set, creating an empty one when none is active. Returns the set and how many
/// models were new to it.
pub fn add_model_files(state: &mut EditorState, paths: &[std::path::PathBuf]) -> (NodeId, usize) {
    let items = items_from_paths(state, paths);
    let id = active_set(state).unwrap_or_else(|| new_empty_set(state));
    let added = add_items(state, id, items);
    (id, added)
}

#[derive(Default)]
pub struct ScatterTool {
    pub hover: Option<(DVec3, DVec3, NodeId)>,
    last: Option<DVec3>,
    rng: Option<Rng>,
    stroking: bool,
    erasing: bool,
}

impl ScatterTool {
    pub fn stroking(&self) -> bool {
        self.stroking
    }

    /// Ends a stroke without touching the document, the caller owns its transaction.
    pub fn reset(&mut self) {
        self.hover = None;
        self.last = None;
        self.stroking = false;
        self.erasing = false;
    }

    pub fn input(&mut self, ui: &Ui, response: &Response, cam: &Camera, rect: Rect, hover: Option<Pos2>, state: &mut EditorState) {
        let pointer = if self.stroking { ui.input(|i| i.pointer.interact_pos()) } else { hover };
        let modifiers = ui.input(|i| i.modifiers);
        let picking = state.scatter_eyedropper || modifiers.alt;
        self.hover = pointer.and_then(|p| {
            let ray = cam.ray(rect, p);
            let hit = if picking && !self.stroking { target_hit(state, &ray) } else { cursor_hit(state, &ray) };
            hit.map(|h| (h.point, h.normal, h.node))
        });
        if response.clicked_by(PointerButton::Primary) && picking {
            state.scatter_eyedropper = false;
            match self.hover.map(|(_, _, node)| node) {
                Some(node) => match eyedrop_target(state, node) {
                    Ok(added) => {
                        let name = target_label(state, node);
                        state.set_status(if added {
                            format!("{name} is now a target of the scatter set")
                        } else {
                            format!("{name} is no longer a scatter target")
                        });
                    }
                    Err(e) => state.set_status(e),
                },
                None => state.set_status("Nothing under the cursor to target"),
            }

            return;
        }

        let rng = self.rng.get_or_insert_with(|| Rng::new(crate::commands::time_seed()));
        let down = ui.input(|i| i.pointer.primary_down());
        if (response.drag_started_by(PointerButton::Primary) || response.clicked_by(PointerButton::Primary)) && !self.stroking && !picking {
            self.stroking = true;
            self.erasing = modifiers.shift != state.scatter_erase;
            self.last = None;
            if state.prefs.scatter.seed != 0 {
                *rng = Rng::new(state.prefs.scatter.seed);
            }

            state.doc.begin(if self.erasing { "Erase Scatter" } else { "Scatter" });
        }

        if self.stroking {
            if let Some((p, n, node)) = self.hover {
                let spacing = state.prefs.scatter.radius * 0.5;
                if self.last.is_none_or(|l| (l - p).length() >= spacing) {
                    let mut r = *rng;
                    if self.erasing {
                        erase(state, p, &mut r);
                    } else {
                        paint(state, p, n, Some(node), &mut r);
                    }

                    *rng = r;
                    self.last = Some(p);
                }
            }

            if !down {
                self.stroking = false;
                state.doc.commit();
            }
        }
    }

    pub fn lines(&self, state: &EditorState, out: &mut Vec<LineVertex>) {
        let erasing = if self.stroking { self.erasing } else { state.scatter_erase };
        let picking = state.scatter_eyedropper;
        if let Some((p, n, node)) = self.hover {
            if picking {
                let b = state.doc.map.bounds(node);
                if !b.is_empty() {
                    let c = b.corners();
                    for (i, j) in Aabb::EDGES {
                        line(out, c[i], c[j], rgba(crate::theme::YELLOW, 0.9));
                    }
                }
            } else {
                let color = if erasing { [1.0, 0.4, 0.3, 0.95] } else { [0.5, 1.0, 0.4, 0.95] };
                let r = state.prefs.scatter.radius;
                let basis = gt_core::Plane::from_point_normal(p, n).basis();
                let ring: Vec<DVec3> = (0..=48)
                    .map(|i| {
                        let a = std::f64::consts::TAU * i as f64 / 48.0;
                        p + (basis.0 * a.cos() + basis.1 * a.sin()) * r + n * 1.0
                    })
                    .collect();
                for w in ring.windows(2) {
                    line(out, w[0], w[1], color);
                }

                let inner = r * (1.0 - state.prefs.scatter.rules.falloff.clamp(0.0, 1.0) * 0.5);
                for k in 0..24 {
                    let a = std::f64::consts::TAU * k as f64 / 24.0;
                    let q = p + (basis.0 * a.cos() + basis.1 * a.sin()) * inner + n;
                    line(out, q, q + n * 6.0, [color[0], color[1], color[2], 0.5]);
                }

                line(out, p, p + n * r * 0.25, color);
            }
        }

        if let Some(id) = active_set(state)
            && let Some(set) = state.doc.map.scatter(id)
        {
            for t in &set.targets {
                let b = state.doc.map.bounds(*t);
                if b.is_empty() {
                    continue;
                }

                let c = b.corners();
                for (i, j) in Aabb::EDGES {
                    line(out, c[i], c[j], [0.45, 1.0, 0.55, 0.35]);
                }
            }
        }
    }

    pub fn paint_overlay(&self, ui: &Ui, rect: Rect, state: &EditorState) {
        let painter = ui.painter_at(rect);
        let s = &state.prefs.scatter;
        let text = if state.scatter_eyedropper {
            "Scatter target: click a surface or a scattered object to add it to the set's targets, click a target again to remove it".to_string()
        } else {
            let target = match s.output {
                ScatterOutput::Entities => "entities".to_string(),
                ScatterOutput::Set => match active_set(state).and_then(|id| state.doc.map.scatter(id)) {
                    Some(set) => format!("'{}' ({} instances, {} targets)", set.name, set.instances.len(), set.targets.len()),
                    None => "a new set".to_string(),
                },
            };
            let (drag, shift) = if state.scatter_erase { ("erases", "paints") } else { ("paints", "erases") };
            format!("Scatter into {target}: drag {drag}, Shift+drag {shift}, Alt+click toggles a target, the Scatter panel holds the models")
        };
        painter.text(rect.left_bottom() + Vec2::new(8.0, -8.0), Align2::LEFT_BOTTOM, text, FontId::proportional(12.0), Color32::from_rgb(160, 255, 140));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_geom::Brush;

    fn ground(state: &mut EditorState, min: [f64; 3], max: [f64; 3]) -> NodeId {
        let layer = state.doc.map.default_layer();
        state.doc.edit("floor", |m, _| m.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::from(min), DVec3::from(max)), "dev/grey").unwrap())))
    }

    fn template(state: &mut EditorState, source: &str, spacing: f64, kind: ScatterKind) {
        state.prefs.scatter.palette = vec![ScatterItem { spacing, ..ScatterItem::new(source) }];
        state.prefs.scatter.kind = kind;
        state.prefs.scatter.preset.clear();
    }

    /// A floor with a set of rocks painted onto it, and nothing active afterwards.
    fn floor_with_rocks() -> (EditorState, NodeId, NodeId) {
        let mut state = EditorState::new(Default::default());
        let floor = ground(&mut state, [-512.0, -16.0, -512.0], [512.0, 0.0, 512.0]);
        template(&mut state, "res://rock.glb", 120.0, ScatterKind::Props);
        state.prefs.scatter.radius = 400.0;
        state.prefs.scatter.rules.density = 2.0;
        let mut rng = Rng::new(11);
        assert!(paint(&mut state, DVec3::ZERO, DVec3::Y, None, &mut rng) > 0);
        let rocks = state.active_scatter.unwrap();
        assert_eq!(state.doc.map.scatter(rocks).unwrap().targets, vec![floor]);
        state.active_scatter = None;
        (state, floor, rocks)
    }

    fn down(x: f64, z: f64) -> Ray {
        Ray::new(DVec3::new(x, 2000.0, z), DVec3::NEG_Y)
    }

    #[test]
    fn painting_creates_a_layer_and_only_touches_the_target() {
        let mut state = EditorState::new(Default::default());
        let a = ground(&mut state, [-1024.0, -16.0, -1024.0], [0.0, 0.0, 1024.0]);
        let b = ground(&mut state, [0.0, -16.0, -1024.0], [1024.0, 0.0, 1024.0]);
        template(&mut state, "res://trees/pine.glb", 40.0, ScatterKind::Props);
        state.prefs.scatter.radius = 400.0;
        state.prefs.scatter.rules.density = 3.0;
        let layers = state.doc.map.layers.len();
        let mut rng = Rng::new(5);
        // The stroke starts on brush a, so the set targets a and nothing lands on b.
        let placed = paint(&mut state, DVec3::new(-300.0, 0.0, 0.0), DVec3::Y, Some(a), &mut rng);
        assert!(placed > 10, "placed {placed}");
        assert_eq!(state.doc.map.layers.len(), layers + 1, "the set lives on a new layer");
        let id = state.active_scatter.unwrap();
        assert_eq!(state.doc.map.scatter(id).unwrap().targets, vec![a]);
        paint(&mut state, DVec3::new(0.0, 0.0, 0.0), DVec3::Y, Some(b), &mut rng);
        let set = state.doc.map.scatter(id).unwrap();
        assert!(set.instances.iter().all(|i| i.position.x <= 1e-6), "no instance on the neighbouring brush");

        assert!(toggle_target(&mut state, b));
        paint(&mut state, DVec3::new(300.0, 0.0, 0.0), DVec3::Y, Some(b), &mut rng);
        assert!(state.doc.map.scatter(id).unwrap().instances.iter().any(|i| i.position.x > 0.0));

        let before = state.doc.map.scatter(id).unwrap().instances.len();
        state.prefs.scatter.erase_amount = 1.0;
        let removed = erase(&mut state, DVec3::new(300.0, 0.0, 0.0), &mut rng);
        assert!(removed > 0 && state.doc.map.scatter(id).unwrap().instances.len() == before - removed);
    }

    #[test]
    fn exposed_only_fill_leaves_the_ground_under_a_slab_bare() {
        let mut state = EditorState::new(Default::default());
        let floor = ground(&mut state, [-512.0, -16.0, -512.0], [512.0, 0.0, 512.0]);
        ground(&mut state, [-512.0, 200.0, -512.0], [0.0, 204.0, 512.0]);
        template(&mut state, "res://weed.glb", 16.0, ScatterKind::Foliage);
        state.prefs.scatter.rules.density = 4.0;
        let under = |state: &EditorState| {
            let set = state.doc.map.scatter(active_set(state).unwrap()).unwrap();
            (set.instances.iter().filter(|i| i.position.x < 0.0).count(), set.instances.len())
        };
        state.doc.select(|_, s| s.select_node(floor));
        fill(&mut state, &mut Rng::new(2)).unwrap();
        let (covered, all) = under(&state);
        assert!(covered > 0 && all > covered, "the slab starts above the fill's rays, so weeds grow under it: {covered} of {all}");

        state.active_scatter = None;
        state.prefs.scatter.rules.exposed_only = true;
        state.doc.select(|_, s| s.select_node(floor));
        fill(&mut state, &mut Rng::new(2)).unwrap();
        let (covered, all) = under(&state);
        assert!(covered == 0 && all > 0, "exposed only keeps them out: {covered} of {all}");
    }

    #[test]
    fn a_scatter_set_can_be_the_ground_of_another_one() {
        let (mut state, floor, rocks) = floor_with_rocks();
        let top = state.doc.map.scatter(rocks).unwrap().instances.iter().map(|i| i.position.y).fold(f64::MIN, f64::max);
        assert_eq!(top, 0.0, "the rocks sit on the floor");

        // A grass set that targets the rocks lands on them, above the floor, although it keeps spread against other
        // sets: the set it is painted onto is ground, not a neighbour.
        assert!(state.prefs.scatter.avoid_other_sets);
        template(&mut state, "res://grass.glb", 12.0, ScatterKind::Foliage);
        let grass = new_set(&mut state, Some("grass"));
        assert!(toggle_target(&mut state, rocks), "the rock set becomes a target surface");
        state.prefs.scatter.rules.density = 8.0;
        let mut rng = Rng::new(3);
        let placed = paint(&mut state, DVec3::ZERO, DVec3::Y, None, &mut rng);
        let rock_count = state.doc.map.scatter(rocks).unwrap().instances.len();
        assert!(placed >= rock_count, "grass lands on the rocks, placed {placed} on {rock_count} rocks");
        let blades = &state.doc.map.scatter(grass).unwrap().instances;
        assert!(blades.iter().all(|i| i.position.y > 0.0), "every blade sits above the floor, on a rock");
        assert!(blades.iter().all(|i| i.position.y <= 32.0 * 1.2 + 1e-6), "blades sit on the rocks, not on a standin far above them");
        assert!(state.doc.map.scatter(floor).is_none(), "the floor is a brush, not a set");
    }

    #[test]
    fn a_stroke_that_starts_on_scattered_rocks_paints_onto_them() {
        // The mouse path: the cursor ray finds the surface under the brush and the dab takes it as the first target.
        let (mut state, floor, rocks) = floor_with_rocks();
        template(&mut state, "res://grass.glb", 12.0, ScatterKind::Foliage);
        state.prefs.scatter.radius = 64.0;
        state.prefs.scatter.rules.density = 8.0;
        let rock = state.doc.map.scatter(rocks).unwrap().instances[0].position;
        let hit = cursor_hit(&state, &down(rock.x, rock.z)).expect("the cursor finds the rock");
        assert_eq!(hit.node, rocks, "the brush sits on the rock rather than on the floor under it");
        assert!(hit.point.y > 0.0);

        let mut rng = Rng::new(9);
        let placed = paint(&mut state, hit.point, hit.normal, Some(hit.node), &mut rng);
        assert!(placed > 0);
        let grass = state.active_scatter.unwrap();
        let set = state.doc.map.scatter(grass).unwrap();
        assert_eq!(set.targets, vec![rocks], "the rock set became the grass set's target");
        assert!(set.instances.iter().all(|i| i.position.y > 0.0), "no blade fell through to the floor");
        assert!(!set.targets.contains(&floor));
    }

    #[test]
    fn grass_lands_on_the_real_shape_of_a_scattered_model() {
        // A block twice as tall as it is wide: much taller than the 32 unit standin an unloaded model gets.
        let dir = std::env::temp_dir().join(format!("gt_scatter_block_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("block.obj");
        let corners = [(-1, 0, -1), (1, 0, -1), (1, 0, 1), (-1, 0, 1), (-1, 4, -1), (1, 4, -1), (1, 4, 1), (-1, 4, 1)];
        let mut obj: String = corners.iter().map(|(x, y, z)| format!("v {} {} {}\n", *x as f64 * 0.5, *y as f64 * 0.5, *z as f64 * 0.5)).collect();
        obj.push_str("f 1 2 3 4\nf 5 8 7 6\nf 1 5 6 2\nf 2 6 7 3\nf 3 7 8 4\nf 4 8 5 1\n");
        std::fs::write(&path, obj).unwrap();

        let mut state = EditorState::new(Default::default());
        let floor = ground(&mut state, [-512.0, -16.0, -512.0], [512.0, 0.0, 512.0]);
        let source = path.to_string_lossy().replace('\\', "/");
        let blocks = create_set(&mut state, "blocks", ScatterKind::Props, vec![ScatterItem::new(source.clone())]);
        state.doc.edit("place", |m, _| {
            let s = m.scatter_mut(blocks).unwrap();
            s.targets = vec![floor];
            s.instances.push(gt_doc::scatter::ScatterInstance { item: 0, position: DVec3::ZERO, angles: DVec3::ZERO, scale: 1.0 });
        });

        template(&mut state, "res://grass.glb", 4.0, ScatterKind::Foliage);
        new_set(&mut state, Some("grass"));
        assert!(toggle_target(&mut state, blocks));
        state.prefs.scatter.radius = 200.0;
        state.prefs.scatter.rules.density = 40.0;
        let placed = paint(&mut state, DVec3::new(0.0, 200.0, 0.0), DVec3::Y, None, &mut Rng::new(1));
        let model = state.models.peek(&path).expect("painting loads the models of the sets it paints onto");
        let top = model.bounds.max.y;
        assert!(top > 40.0, "the block is taller than a standin, top {top}");
        assert!(placed > 0);
        let grass = state.active_scatter.unwrap();
        for blade in &state.doc.map.scatter(grass).unwrap().instances {
            assert!((blade.position.y - top).abs() < 1e-3, "blade at {:?}, the block top is at {top}", blade.position);
            assert!(blade.position.x.abs() <= model.bounds.max.x + 1e-3, "blade off the block at {:?}", blade.position);
        }

        let hit = cursor_hit(&state, &down(0.0, 0.0)).unwrap();
        assert_eq!(hit.node, blocks);
        assert!((hit.point.y - top).abs() < 1e-3 && hit.normal.y > 0.99, "the brush sits on the block top, {hit:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_stroke_on_the_floor_between_rocks_still_targets_the_floor() {
        let (mut state, floor, rocks) = floor_with_rocks();
        template(&mut state, "res://grass.glb", 12.0, ScatterKind::Foliage);
        let set = state.doc.map.scatter(rocks).unwrap();
        let clear = (-500..=500)
            .step_by(25)
            .flat_map(|x| (-500..=500).step_by(25).map(move |z| (x as f64, z as f64)))
            .find(|(x, z)| set.instances.iter().all(|i| (i.position.x - x).hypot(i.position.z - z) > 80.0))
            .expect("some floor without rocks");
        let hit = cursor_hit(&state, &down(clear.0, clear.1)).unwrap();
        assert_eq!(hit.node, floor);
        paint(&mut state, hit.point, hit.normal, Some(hit.node), &mut Rng::new(2));
        assert_eq!(state.doc.map.scatter(state.active_scatter.unwrap()).unwrap().targets, vec![floor]);
    }

    #[test]
    fn disabled_models_are_not_painted_and_keep_their_instances() {
        let mut state = EditorState::new(Default::default());
        let floor = ground(&mut state, [-512.0, -16.0, -512.0], [512.0, 0.0, 512.0]);
        let id = create_set(&mut state, "mix", ScatterKind::Props, vec![ScatterItem { spacing: 20.0, ..ScatterItem::new("res://a.glb") }]);
        state.prefs.scatter.radius = 300.0;
        state.prefs.scatter.rules.density = 3.0;
        let mut rng = Rng::new(4);
        paint(&mut state, DVec3::ZERO, DVec3::Y, Some(floor), &mut rng);
        let first = state.doc.map.scatter(id).unwrap().counts()[0];
        assert!(first > 0);

        add_items(&mut state, id, vec![ScatterItem { spacing: 20.0, ..ScatterItem::new("res://b.glb") }]);
        state.doc.edit("off", |m, _| m.scatter_mut(id).unwrap().items[0].enabled = false);
        paint(&mut state, DVec3::new(200.0, 0.0, 200.0), DVec3::Y, Some(floor), &mut rng);
        let counts = state.doc.map.scatter(id).unwrap().counts();
        assert_eq!(counts[0], first, "the disabled model neither grows nor loses instances");
        assert!(counts[1] > 0, "the enabled one is painted");

        state.doc.edit("all off", |m, _| m.scatter_mut(id).unwrap().items[1].enabled = false);
        assert_eq!(paint(&mut state, DVec3::ZERO, DVec3::Y, Some(floor), &mut rng), 0);
        assert!(state.status.contains("switched off"), "{}", state.status);
    }

    #[test]
    fn dropped_models_join_the_active_set_or_start_one() {
        let mut state = EditorState::new(Default::default());
        let paths = vec![std::path::PathBuf::from("C:/models/fir.glb"), std::path::PathBuf::from("C:/models/stone.bbmodel")];
        let (id, added) = add_model_files(&mut state, &paths);
        assert_eq!(added, 2);
        assert_eq!(state.active_scatter, Some(id), "a drop without a set starts an empty one and fills it");
        let set = state.doc.map.scatter(id).unwrap();
        assert_eq!(set.items.iter().map(|i| i.label()).collect::<Vec<_>>(), ["fir", "stone"]);
        assert!(set.items.iter().all(|i| i.enabled));

        state.doc.edit("off", |m, _| m.scatter_mut(id).unwrap().items[0].enabled = false);
        let (again, added) = add_model_files(&mut state, &paths[..1]);
        assert_eq!((again, added), (id, 0), "dropping a model the set has switches it back on");
        assert!(state.doc.map.scatter(id).unwrap().items[0].enabled);
        assert_eq!(state.doc.map.scatter(id).unwrap().items.len(), 2);
    }

    #[test]
    fn the_eyedropper_targets_another_set_and_refuses_the_set_itself() {
        let (mut state, floor, rocks) = floor_with_rocks();
        template(&mut state, "res://moss.glb", 10.0, ScatterKind::Foliage);
        let moss = new_set(&mut state, Some("moss"));
        let rock = state.doc.map.scatter(rocks).unwrap().instances[0].position;
        let hit = target_hit(&state, &down(rock.x, rock.z)).expect("a rock is under the eyedropper");
        assert_eq!(hit.node, rocks);
        assert_eq!(eyedrop_target(&mut state, hit.node), Ok(true));
        assert_eq!(state.doc.map.scatter(moss).unwrap().targets, vec![rocks]);
        assert_eq!(target_label(&state, rocks), "rock (set)");
        assert_eq!(eyedrop_target(&mut state, rocks), Ok(false), "a second pick removes it");
        assert!(eyedrop_target(&mut state, moss).is_err());
        assert_eq!(eyedrop_target(&mut state, floor), Ok(true));

        let layer = state.doc.map.default_layer();
        let lamp = state.doc.edit("lamp", |m, _| m.insert(layer, NodeKind::Entity(Entity::new("light"))));
        assert!(eyedrop_target(&mut state, lamp).is_err(), "an entity is not a surface");
    }

    #[test]
    fn a_palette_disables_the_models_it_leaves_out() {
        let mut state = EditorState::new(Default::default());
        let id = create_set(&mut state, "s", ScatterKind::Props, vec![ScatterItem::new("res://a.glb"), ScatterItem::new("res://b.glb")]);
        set_palette(&mut state, id, &[ScatterItem { weight: 4.0, ..ScatterItem::new("res://b.glb") }, ScatterItem::new("res://c.glb")]);
        let set = state.doc.map.scatter(id).unwrap();
        assert_eq!(set.items.iter().map(|i| (i.label(), i.enabled)).collect::<Vec<_>>(), [("a", false), ("b", true), ("c", true)]);
        assert_eq!(set.items[1].weight, 4.0);
        assert_eq!(brush_items(&state).len(), 2);
    }

    #[test]
    fn deleting_a_set_drops_its_empty_layer_and_renaming_follows_the_layer() {
        let mut state = EditorState::new(Default::default());
        let layers = state.doc.map.layers.len();
        let a = new_empty_set(&mut state);
        let b = new_empty_set(&mut state);
        assert_eq!(state.doc.map.scatter(b).unwrap().name, "scatter 2");
        rename_set(&mut state, a, "hedge");
        let layer = state.doc.map.layer_of(a);
        assert_eq!(state.doc.map.get(layer).unwrap().name(), "Scatter: hedge");
        delete_set(&mut state, a);
        assert!(state.doc.map.scatter(a).is_none());
        assert_eq!(state.doc.map.layers.len(), layers + 1, "only the second set's layer is left");
        assert_eq!(state.active_scatter, Some(b), "deleting another set keeps the active one");
        delete_set(&mut state, b);
        assert_eq!(state.active_scatter, None);
    }

    #[test]
    fn entity_output_places_classnames_with_spacing() {
        let mut state = EditorState::new(Default::default());
        ground(&mut state, [-512.0, -16.0, -512.0], [512.0, 0.0, 512.0]);
        state.prefs.scatter.output = ScatterOutput::Entities;
        state.prefs.scatter.palette = vec![ScatterItem { spacing: 48.0, ..ScatterItem::new("npc_zombie") }];
        state.prefs.scatter.radius = 256.0;
        state.prefs.scatter.rules.density = 2.0;
        let mut rng = Rng::new(2);
        let n = paint(&mut state, DVec3::ZERO, DVec3::Y, None, &mut rng);
        assert!(n > 3);
        let points: Vec<DVec3> = state.doc.map.entities().filter(|(_, e)| e.classname == "npc_zombie").map(|(_, e)| e.origin).collect();
        for (i, a) in points.iter().enumerate() {
            for b in &points[i + 1..] {
                assert!((*a - *b).length() >= 48.0 - 1e-6);
            }
        }

        assert_eq!(erase(&mut state, DVec3::ZERO, &mut rng), points.len());
    }
}
