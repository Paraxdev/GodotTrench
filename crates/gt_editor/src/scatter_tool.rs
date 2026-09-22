//! Scatter tool: radial painting and erasing of trees, rocks and foliage from a weighted palette. Instances go into a
//! scatter set on its own layer, restricted to the set's target surfaces, or become point entities.

use egui::{Align2, Color32, FontId, PointerButton, Pos2, Rect, Response, Ui, Vec2};
use gt_core::{Aabb, DVec3, NodeId};
use gt_doc::scatter::{Rng, ScatterRules, SurfaceHit};
use gt_doc::{Entity, NodeKind, Scatter, ScatterItem};
use gt_render::LineVertex;

use crate::camera::Camera;
use crate::picking::SurfaceCaster;
use crate::scene::v3;
use crate::state::{EditorState, ScatterOutput};

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

/// Creates a scatter set on a new layer from the current palette and makes it active.
pub fn new_set(state: &mut EditorState, name: Option<&str>) -> NodeId {
    let s = &state.prefs.scatter;
    let label = name.map(str::to_string).unwrap_or_else(|| {
        if s.preset.is_empty() { s.palette.first().map(|i| i.label().to_string()).unwrap_or_else(|| "scatter".into()) } else { s.preset.clone() }
    });
    let mut set = Scatter::new(label.clone(), s.kind, s.palette.clone());
    if s.kind == gt_doc::ScatterKind::Foliage {
        set.collision = gt_doc::scatter::ScatterCollision::None;
    }

    set.chunk_size = s.chunk_size;
    set.static_props_multimesh = s.static_props_multimesh;
    // None keeps the range Scatter::new picked from the kind: foliage fades out, props stay visible.
    if let Some(range) = s.visibility_range {
        set.visibility_range = range;
    }

    let id = state.doc.edit("New Scatter Set", |m, _| {
        let layer = m.add_layer(&format!("Scatter: {label}"));
        m.insert(layer, NodeKind::Scatter(set))
    });
    state.active_scatter = Some(id);
    id
}

/// Instances of other sets near `region`, so sets keep their spacing against each other.
fn other_footprints(state: &EditorState, skip: Option<NodeId>, region: &Aabb) -> Vec<(DVec3, f64)> {
    if !state.prefs.scatter.avoid_other_sets {
        return Vec::new();
    }

    state
        .doc
        .map
        .scatters()
        .filter(|(id, _)| Some(*id) != skip && !state.doc.map.is_hidden(*id))
        .flat_map(|(_, s)| s.footprints())
        .filter(|(p, _)| region.contains_point(*p))
        .collect()
}

/// One paint dab. Returns the number of instances or entities placed.
pub fn paint(state: &mut EditorState, center: DVec3, normal: DVec3, rng: &mut Rng) -> usize {
    let settings = state.prefs.scatter.clone();
    if settings.palette.is_empty() {
        state.set_status("The scatter palette is empty, add models in the palette window");
        return 0;
    }

    if settings.output == ScatterOutput::Entities {
        return paint_entities(state, center, normal, settings.radius, &settings.rules, rng).len();
    }

    let id = active_set(state).unwrap_or_else(|| new_set(state, None));
    let region = dab_region(center, settings.radius);
    let others = other_footprints(state, Some(id), &region);
    let Some(mut set) = state.doc.map.scatter(id).cloned() else { return 0 };
    let indices = set.merge_items(&settings.palette);
    let placed = {
        let caster = SurfaceCaster::with_targets(state, &region, &set.targets);
        if settings.rules.only_targets
            && set.targets.is_empty()
            && let Some(hit) = caster.cast(center + normal * settings.radius, -normal)
        {
            set.targets.push(hit.node);
        }

        let rules = ScatterRules { items: Some(indices), ..settings.rules.clone() };
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
        return erase_entities(state, center, s.radius);
    }

    let Some(id) = active_set(state) else {
        state.set_status("No active scatter set to erase from, pick one in the toolbar");
        return 0;
    };
    let Some(mut set) = state.doc.map.scatter(id).cloned() else { return 0 };
    let only: Option<Vec<usize>> = s.erase_palette_only.then(|| s.palette.iter().filter_map(|p| set.items.iter().position(|i| i.source == p.source)).collect());
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

/// Fills the whole area of the active set's targets (or the selected geometry) with the palette.
pub fn fill(state: &mut EditorState, rng: &mut Rng) -> Result<usize, String> {
    let settings = state.prefs.scatter.clone();
    let id = active_set(state).unwrap_or_else(|| new_set(state, None));
    let Some(mut set) = state.doc.map.scatter(id).cloned() else { return Err("scatter set vanished".into()) };
    if set.targets.is_empty() {
        set.targets = state.doc.selection.geometry(&state.doc.map);
    }

    if set.targets.is_empty() {
        return Err("Select the surfaces to fill, or paint a stroke first so the set has a target".into());
    }

    let region = state.doc.map.bounds_of(set.targets.iter().copied());
    let indices = set.merge_items(&settings.palette);
    let others = other_footprints(state, Some(id), &region.expanded(256.0));
    let placed = {
        let caster = SurfaceCaster::with_targets(state, &region.expanded(64.0), &set.targets);
        let rules = ScatterRules { items: Some(indices), only_targets: true, ..settings.rules.clone() };
        set.fill(&region, &rules, rng, &others, |o, d| caster.cast(o, d))
    };
    state.doc.edit("Fill Scatter", |m, _| {
        if let Some(slot) = m.scatter_mut(id) {
            *slot = set;
        }
    });
    Ok(placed)
}

/// Adds or removes the surface under a point from the active set's targets.
pub fn toggle_target(state: &mut EditorState, node: NodeId) -> bool {
    let Some(id) = active_set(state) else { return false };
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

/// Places palette entries as point entities, keeping spacing to existing ones.
pub fn paint_entities(state: &mut EditorState, center: DVec3, normal: DVec3, radius: f64, rules: &ScatterRules, rng: &mut Rng) -> Vec<NodeId> {
    let s = state.prefs.scatter.clone();
    let mut temp = Scatter::new("entities", gt_doc::ScatterKind::Props, s.palette.clone());
    let region = dab_region(center, radius);
    let existing: Vec<(DVec3, f64)> = state
        .doc
        .map
        .entities()
        .filter_map(|(_, e)| s.palette.iter().find(|p| matches_source(e, &p.source, &s.prop_class)).map(|p| (e.origin, p.spacing)))
        .filter(|(p, _)| region.contains_point(*p))
        .collect();
    let rules = ScatterRules { only_targets: false, ..rules.clone() };
    {
        let caster = SurfaceCaster::new(state, &region);
        temp.paint(center, normal, radius, &rules, rng, &existing, |o, d| caster.cast(o, d));
    }

    if temp.instances.is_empty() {
        return Vec::new();
    }

    let parent = state.insert_parent();
    let prop_class = s.prop_class.clone();
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

fn erase_entities(state: &mut EditorState, center: DVec3, radius: f64) -> usize {
    let s = state.prefs.scatter.clone();
    let ids: Vec<NodeId> = state
        .doc
        .map
        .entities()
        .filter(|(id, e)| {
            state.doc.map.is_editable(*id) && (e.origin - center).length() <= radius && s.palette.iter().any(|p| matches_source(e, &p.source, &s.prop_class))
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
    n
}

/// Writes the built-in nature models into the project so the presets resolve. Returns how many files were written.
pub fn install_nature(state: &mut EditorState, overwrite: bool) -> Result<usize, String> {
    let dir = state.game.resolve_res(gt_doc::scatter::NATURE_DIR).ok_or("Open a Godot project first, models are installed into it")?;
    let written = gt_formats::nature::install(&dir, overwrite).map_err(|e| e.to_string())?;
    if !written.is_empty() {
        state.models.clear();
    }

    Ok(written.len())
}

pub fn apply_preset(state: &mut EditorState, name: &str) -> bool {
    let Some((kind, items)) = gt_doc::scatter::preset(name) else { return false };
    let s = &mut state.prefs.scatter;
    s.palette = items;
    s.kind = kind;
    s.preset = name.to_string();
    if kind == gt_doc::ScatterKind::Foliage {
        s.rules.density = s.rules.density.max(4.0);
        s.radius = s.radius.min(256.0);
    }

    true
}

/// Palette entries for model files picked at once, with sensible spacing from their size.
pub fn items_from_paths(state: &EditorState, paths: &[std::path::PathBuf]) -> Vec<ScatterItem> {
    paths
        .iter()
        .map(|p| {
            let source = state
                .game
                .project_root
                .as_ref()
                .and_then(|root| gt_formats::game::to_res_path(root, p))
                .unwrap_or_else(|| p.to_string_lossy().replace('\\', "/"));
            ScatterItem::new(source)
        })
        .collect()
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
        self.hover = pointer.and_then(|p| {
            let ray = cam.ray(rect, p);
            let region = Aabb::from_center_size(ray.origin, DVec3::splat(1.0e6));
            let targets = active_set(state).and_then(|id| state.doc.map.scatter(id)).map(|s| s.targets.clone()).unwrap_or_default();
            let caster = SurfaceCaster::with_targets(state, &region, &targets);
            caster.cast(ray.origin, ray.dir).map(|h: SurfaceHit| (h.point, h.normal, h.node))
        });
        let modifiers = ui.input(|i| i.modifiers);
        if response.clicked_by(PointerButton::Primary) && modifiers.alt {
            // Picked rather than cast, so clicking a scattered rock names that set as the target and foliage
            // can then be painted over it. The caster only knows the sets that are already targets.
            let picked = response
                .interact_pointer_pos()
                .and_then(|p| crate::picking::pick(state, &cam.ray(rect, p)))
                .map(|h| h.node)
                .or(self.hover.map(|(_, _, node)| node));
            if let Some(node) = picked {
                if active_set(state).is_none() {
                    new_set(state, None);
                }

                if Some(node) == active_set(state) {
                    state.set_status("A scatter set cannot be painted onto itself");
                    return;
                }

                let added = toggle_target(state, node);
                state.set_status(if added { format!("{node} is now a target of the scatter set") } else { format!("{node} removed from the scatter targets") });
            }

            return;
        }

        let rng = self
            .rng
            .get_or_insert_with(|| Rng::new(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(7)));
        let down = ui.input(|i| i.pointer.primary_down());
        if (response.drag_started_by(PointerButton::Primary) || response.clicked_by(PointerButton::Primary)) && !self.stroking && !modifiers.alt {
            self.stroking = true;
            self.erasing = modifiers.shift;
            self.last = None;
            state.doc.begin(if self.erasing { "Erase Scatter" } else { "Scatter" });
        }

        if self.stroking {
            if let Some((p, n, _)) = self.hover {
                let spacing = state.prefs.scatter.radius * 0.5;
                if self.last.is_none_or(|l| (l - p).length() >= spacing) {
                    let mut r = *rng;
                    if self.erasing {
                        erase(state, p, &mut r);
                    } else {
                        paint(state, p, n, &mut r);
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
        let erasing = self.stroking && self.erasing;
        if let Some((p, n, _)) = self.hover {
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
        let target = match s.output {
            ScatterOutput::Entities => "entities".to_string(),
            ScatterOutput::Set => match active_set(state).and_then(|id| state.doc.map.scatter(id)) {
                Some(set) => format!("set '{}' ({} instances, {} targets)", set.name, set.instances.len(), set.targets.len()),
                None => "a new scatter layer".to_string(),
            },
        };
        let names: Vec<&str> = s.palette.iter().map(|i| i.label()).collect();
        let text = format!("Scatter {} into {target}: drag paints, Shift+drag erases, Alt+click toggles a target surface", names.join(", "));
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

    #[test]
    fn painting_creates_a_layer_and_only_touches_the_target() {
        let mut state = EditorState::new(Default::default());
        let a = ground(&mut state, [-1024.0, -16.0, -1024.0], [0.0, 0.0, 1024.0]);
        let b = ground(&mut state, [0.0, -16.0, -1024.0], [1024.0, 0.0, 1024.0]);
        state.prefs.scatter.palette = vec![ScatterItem { spacing: 40.0, ..ScatterItem::new("res://trees/pine.glb") }];
        state.prefs.scatter.radius = 400.0;
        state.prefs.scatter.rules.density = 3.0;
        let layers = state.doc.map.layers.len();
        let mut rng = Rng::new(5);
        // The stroke starts on brush a, so the set targets a and nothing lands on b.
        let placed = paint(&mut state, DVec3::new(-300.0, 0.0, 0.0), DVec3::Y, &mut rng);
        assert!(placed > 10, "placed {placed}");
        assert_eq!(state.doc.map.layers.len(), layers + 1, "the set lives on a new layer");
        let id = state.active_scatter.unwrap();
        assert_eq!(state.doc.map.scatter(id).unwrap().targets, vec![a]);
        paint(&mut state, DVec3::new(0.0, 0.0, 0.0), DVec3::Y, &mut rng);
        let set = state.doc.map.scatter(id).unwrap();
        assert!(set.instances.iter().all(|i| i.position.x <= 1e-6), "no instance on the neighbouring brush");

        assert!(toggle_target(&mut state, b));
        paint(&mut state, DVec3::new(300.0, 0.0, 0.0), DVec3::Y, &mut rng);
        assert!(state.doc.map.scatter(id).unwrap().instances.iter().any(|i| i.position.x > 0.0));

        let before = state.doc.map.scatter(id).unwrap().instances.len();
        state.prefs.scatter.erase_amount = 1.0;
        let removed = erase(&mut state, DVec3::new(300.0, 0.0, 0.0), &mut rng);
        assert!(removed > 0 && state.doc.map.scatter(id).unwrap().instances.len() == before - removed);
    }

    #[test]
    fn a_scatter_set_can_be_the_ground_of_another_one() {
        let mut state = EditorState::new(Default::default());
        let floor = ground(&mut state, [-512.0, -16.0, -512.0], [512.0, 0.0, 512.0]);

        // Rocks on the floor first.
        state.prefs.scatter.palette = vec![ScatterItem { spacing: 120.0, ..ScatterItem::new("res://rock.glb") }];
        state.prefs.scatter.radius = 400.0;
        state.prefs.scatter.rules.density = 2.0;
        let mut rng = Rng::new(11);
        assert!(paint(&mut state, DVec3::ZERO, DVec3::Y, &mut rng) > 0);
        let rocks = state.active_scatter.unwrap();
        let top = state.doc.map.scatter(rocks).unwrap().instances.iter().map(|i| i.position.y).fold(f64::MIN, f64::max);
        assert_eq!(top, 0.0, "the rocks sit on the floor");

        // A grass set that targets the rocks lands on them, above the floor.
        state.prefs.scatter.palette = vec![ScatterItem { spacing: 12.0, ..ScatterItem::new("res://grass.glb") }];
        state.prefs.scatter.kind = gt_doc::ScatterKind::Foliage;
        let grass = new_set(&mut state, Some("grass"));
        state.active_scatter = Some(grass);
        assert!(toggle_target(&mut state, rocks), "the rock set becomes a target surface");
        state.prefs.scatter.rules.density = 8.0;
        let placed = paint(&mut state, DVec3::ZERO, DVec3::Y, &mut rng);
        assert!(placed > 0, "grass lands on the rocks, placed {placed}");
        let blades = &state.doc.map.scatter(grass).unwrap().instances;
        assert!(blades.iter().all(|i| i.position.y > 0.0), "every blade sits above the floor, on a rock");
        assert!(state.doc.map.scatter(floor).is_none(), "the floor is a brush, not a set");
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
        let n = paint(&mut state, DVec3::ZERO, DVec3::Y, &mut rng);
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
