use gt_core::{Aabb, DVec3, NodeId, Ray};
use gt_doc::NodeKind;

use crate::scene::entity_box;
use crate::state::EditorState;

#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub node: NodeId,
    /// Face index for brush hits.
    pub face: Option<usize>,
    pub distance: f64,
    pub point: DVec3,
    pub normal: DVec3,
}

/// All hits along the ray, nearest first. Hidden and locked objects are skipped.
pub fn pick_all(state: &EditorState, ray: &Ray) -> Vec<Hit> {
    let map = &state.doc.map;
    let mut hits = Vec::new();
    for (id, node) in map.nodes.iter() {
        let bounds_hit = |b: &Aabb| -> Option<Hit> {
            let t = ray.intersect_aabb(b)?;
            let p = ray.at(t);
            Some(Hit { node: *id, face: None, distance: t, point: p, normal: box_normal(b, p) })
        };
        match &node.kind {
            NodeKind::Brush(b) => {
                let bb = b.bounds().expanded(0.01);
                if (!bb.contains_point(ray.origin) && ray.intersect_aabb(&bb).is_none()) || !map.is_editable(*id) || !map.in_cordon(*id) {
                    continue;
                }

                if let Some(h) = b.ray_cast(ray) {
                    hits.push(Hit { node: *id, face: Some(h.face), distance: h.distance, point: h.point, normal: b.faces[h.face].plane.normal });
                }
            }
            NodeKind::Mesh(m) if map.is_editable(*id) && map.in_cordon(*id) => {
                if let Some((t, face)) = m.ray_cast(ray) {
                    let mut normal = m.face_normal(face);
                    if normal.dot(ray.dir) > 0.0 {
                        normal = -normal;
                    }

                    hits.push(Hit { node: *id, face: Some(face), distance: t, point: ray.at(t), normal });
                }
            }
            NodeKind::Terrain(t) if map.is_editable(*id) && map.in_cordon(*id) => {
                if let Some((dist, p)) = t.ray_cast(ray) {
                    let local = (p - t.origin) / t.cell_size;
                    let i = (local.x.round().max(0.0) as u32).min(t.resolution[0] - 1);
                    let j = (local.z.round().max(0.0) as u32).min(t.resolution[1] - 1);
                    hits.push(Hit { node: *id, face: None, distance: dist, point: p, normal: t.normal(i, j) });
                }
            }
            NodeKind::Entity(e) if node.children.is_empty() => {
                if map.is_editable(*id) {
                    let b = state.model_bounds.get(id).copied().unwrap_or_else(|| entity_box(&state.game, e));
                    hits.extend(bounds_hit(&b));
                }
            }

            // Foliage is painted densely and drawn as one MultiMesh, so a single blade is not a thing to
            // select. Letting it pick would put a carpet of spheres in front of the ground and walls under
            // it, so it never takes a click. Foliage sets are selected in the outliner or the scatter toolbar.
            NodeKind::Scatter(s) if s.kind != gt_doc::ScatterKind::Foliage && map.is_editable(*id) && map.in_cordon(*id) => {
                if ray.intersect_aabb(&s.bounds()).is_none() {
                    continue;
                }

                // Instances pick as spheres around their lower half, closest first.
                let best = s
                    .instances
                    .iter()
                    .filter_map(|inst| {
                        let r = 20.0 * inst.scale.max(0.2);
                        let (d, t) = ray.distance_to_point(inst.position + DVec3::Y * r);
                        (d <= r && t > 0.0).then_some(t)
                    })
                    .min_by(|a, b| a.total_cmp(b));
                if let Some(t) = best {
                    hits.push(Hit { node: *id, face: None, distance: t, point: ray.at(t), normal: -ray.dir });
                }
            }
            NodeKind::Instance(i) if map.is_editable(*id) => {
                let b = state.instance_bounds.get(id).copied().unwrap_or(Aabb::from_center_size(i.origin, DVec3::splat(16.0)));
                hits.extend(bounds_hit(&b));
            }
            _ => {}
        }
    }

    hits.sort_by(|a, b| a.distance.total_cmp(&b.distance));
    hits
}

pub fn pick(state: &EditorState, ray: &Ray) -> Option<Hit> {
    pick_all(state, ray).into_iter().next()
}

enum Surface<'a> {
    Brush(&'a gt_geom::Brush),
    Mesh(&'a gt_geom::Mesh),
    Terrain(&'a gt_geom::Terrain),
    /// Instances of another set, so foliage can be painted onto scattered rocks and props. Each instance
    /// stands in as a sphere sized from its palette entry, which is enough to drop grass on top of a rock
    /// without loading and ray casting every instanced model.
    Scatter(&'a gt_doc::Scatter),
}

/// Radius an instance of `item` stands in as, in map units. Spacing is the room the entry keeps around
/// itself, so half of it is close to the model's own footprint.
fn instance_radius(set: &gt_doc::Scatter, inst: &gt_doc::scatter::ScatterInstance) -> f64 {
    let spacing = set.items.get(inst.item as usize).map(|i| i.spacing).unwrap_or(32.0);
    (spacing * 0.5).clamp(4.0, 512.0) * inst.scale.max(0.05)
}

/// Nearest hit of `ray` on the instances of `set`: the distance, the point's normal on the standin sphere.
fn cast_scatter(set: &gt_doc::Scatter, ray: &Ray) -> Option<(f64, DVec3)> {
    let mut best: Option<(f64, DVec3)> = None;
    for inst in &set.instances {
        let r = instance_radius(set, inst);
        let center = inst.position + DVec3::Y * r;
        let (d, t) = ray.distance_to_point(center);
        if d > r || t <= 0.0 {
            continue;
        }

        // Step back from the closest approach onto the sphere itself, so grass lands on the surface rather
        // than at the middle of the rock.
        let t = (t - (r * r - d * d).max(0.0).sqrt()).max(0.0);
        if best.is_none_or(|(bt, _)| t < bt) {
            best = Some((t, (ray.at(t) - center).normalize_or(DVec3::Y)));
        }
    }

    best
}

/// Ray casts against the solid surfaces (brushes, meshes, terrains) overlapping a region, collected once so that
/// thousands of scatter samples do not walk the whole map each time.
pub struct SurfaceCaster<'a> {
    surfaces: Vec<(NodeId, Aabb, Surface<'a>)>,
}

impl<'a> SurfaceCaster<'a> {
    pub fn new(state: &'a EditorState, region: &Aabb) -> Self {
        Self::with_targets(state, region, &[])
    }

    /// As `new`, but the scatter sets in `targets` also count as ground, so a set can be painted onto the
    /// instances of another one.
    pub fn with_targets(state: &'a EditorState, region: &Aabb, targets: &[NodeId]) -> Self {
        let map = &state.doc.map;
        let mut surfaces = Vec::new();
        for (id, node) in map.nodes.iter() {
            let surface = match &node.kind {
                NodeKind::Brush(b) if !b.faces.iter().all(|f| state.game.is_tool_texture(&f.data.material)) => Surface::Brush(b),
                NodeKind::Mesh(m) => Surface::Mesh(m),
                NodeKind::Terrain(t) => Surface::Terrain(t),
                // Only sets that were picked as targets: otherwise every set would catch the samples meant
                // for the ground it stands on.
                NodeKind::Scatter(s) if !s.instances.is_empty() && targets.contains(id) => Surface::Scatter(s),
                _ => continue,
            };
            if map.is_hidden(*id) || !map.in_cordon(*id) {
                continue;
            }

            // Trigger volumes and other entity brushes that do not render are not ground.
            if let Some(e) = map.owning_entity(*id).and_then(|e| map.entity(e))
                && (e.classname.starts_with("trigger") || e.classname.starts_with("func_illusionary") || e.classname.contains("area"))
            {
                continue;
            }

            let b = map.bounds(*id).expanded(1.0);
            if b.intersects(region) {
                surfaces.push((*id, b, surface));
            }
        }

        Self { surfaces }
    }

    pub fn is_empty(&self) -> bool {
        self.surfaces.is_empty()
    }

    pub fn cast(&self, origin: DVec3, dir: DVec3) -> Option<gt_doc::scatter::SurfaceHit> {
        let ray = Ray::new(origin, dir);
        let mut best: Option<(f64, gt_doc::scatter::SurfaceHit)> = None;
        for (id, bounds, surface) in &self.surfaces {
            if !bounds.contains_point(origin) && ray.intersect_aabb(bounds).is_none_or(|t| best.as_ref().is_some_and(|(bt, _)| t > *bt)) {
                continue;
            }

            let hit = match surface {
                Surface::Brush(b) => b.ray_cast(&ray).map(|h| (h.distance, b.faces[h.face].plane.normal)),
                Surface::Mesh(m) => m.ray_cast(&ray).map(|(t, f)| {
                    let n = m.face_normal(f);
                    (t, if n.dot(dir) > 0.0 { -n } else { n })
                }),
                Surface::Terrain(t) => t.ray_cast(&ray).map(|(dist, p)| {
                    let local = (p - t.origin) / t.cell_size;
                    let i = (local.x.round().max(0.0) as u32).min(t.resolution[0] - 1);
                    let j = (local.z.round().max(0.0) as u32).min(t.resolution[1] - 1);
                    (dist, t.normal(i, j))
                }),
                Surface::Scatter(s) => cast_scatter(s, &ray),
            };
            if let Some((t, normal)) = hit
                && t >= 0.0
                && best.as_ref().is_none_or(|(bt, _)| t < *bt)
            {
                best = Some((t, gt_doc::scatter::SurfaceHit { point: ray.at(t), normal, node: *id }));
            }
        }

        best.map(|(_, h)| h)
    }
}

fn box_normal(b: &Aabb, p: DVec3) -> DVec3 {
    let c = b.center();
    let half = b.size() * 0.5;
    let d = (p - c) / half.max(DVec3::splat(1e-6));
    let a = d.abs();
    if a.x >= a.y && a.x >= a.z {
        DVec3::new(d.x.signum(), 0.0, 0.0)
    } else if a.y >= a.z {
        DVec3::new(0.0, d.y.signum(), 0.0)
    } else {
        DVec3::new(0.0, 0.0, d.z.signum())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::EditorState;
    use gt_doc::{Scatter, ScatterItem, ScatterKind};
    use gt_geom::Brush;

    /// A floor with one scatter instance standing on it at the origin.
    fn floor_with_scatter(kind: ScatterKind) -> (EditorState, NodeId, NodeId) {
        let mut state = EditorState::new(Default::default());
        let layer = state.doc.map.default_layer();
        let bounds = Aabb::new(DVec3::new(-256.0, -16.0, -256.0), DVec3::new(256.0, 0.0, 256.0));
        let floor = state.doc.edit("floor", |m, _| m.insert(layer, NodeKind::Brush(Brush::from_aabb(&bounds, "dev/grey").unwrap())));
        let mut set = Scatter::new("s", kind, vec![ScatterItem { spacing: 40.0, ..ScatterItem::new("res://a.glb") }]);
        set.instances.push(gt_doc::scatter::ScatterInstance { item: 0, position: DVec3::ZERO, angles: DVec3::ZERO, scale: 1.0 });
        let scatter = state.doc.edit("scatter", |m, _| m.insert(layer, NodeKind::Scatter(set)));
        (state, floor, scatter)
    }

    #[test]
    fn foliage_never_takes_a_click_from_the_surface_under_it() {
        // Straight down onto the instance: props answer, foliage leaves the floor selectable.
        let ray = Ray::new(DVec3::new(0.0, 512.0, 0.0), DVec3::NEG_Y);
        let (state, _, scatter) = floor_with_scatter(ScatterKind::Props);
        assert_eq!(pick(&state, &ray).map(|h| h.node), Some(scatter), "a prop is pickable, so its set can be selected");

        let (state, floor, _) = floor_with_scatter(ScatterKind::Foliage);
        assert_eq!(pick(&state, &ray).map(|h| h.node), Some(floor), "grass does not stand in front of the floor");
    }

    #[test]
    fn a_scatter_set_is_only_ground_once_it_is_a_target() {
        let (state, _, scatter) = floor_with_scatter(ScatterKind::Props);
        let region = Aabb::from_center_size(DVec3::ZERO, DVec3::splat(4096.0));
        let origin = DVec3::new(0.0, 512.0, 0.0);

        let plain = SurfaceCaster::new(&state, &region);
        assert_eq!(plain.cast(origin, DVec3::NEG_Y).map(|h| h.point.y), Some(0.0), "without a target the sample lands on the floor");

        let onto = SurfaceCaster::with_targets(&state, &region, &[scatter]);
        let hit = onto.cast(origin, DVec3::NEG_Y).expect("the instance is hit");
        assert_eq!(hit.node, scatter);
        assert!(hit.point.y > 0.0, "the sample sits on top of the instance, got {}", hit.point.y);
        assert!(hit.normal.y > 0.0, "the standin sphere faces up where the ray comes down");
    }
}
