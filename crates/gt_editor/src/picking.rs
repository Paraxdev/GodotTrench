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
            NodeKind::Scatter(s) if map.is_editable(*id) && map.in_cordon(*id) => {
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
}

/// Ray casts against the solid surfaces (brushes, meshes, terrains) overlapping a region, collected once so that
/// thousands of scatter samples do not walk the whole map each time.
pub struct SurfaceCaster<'a> {
    surfaces: Vec<(NodeId, Aabb, Surface<'a>)>,
}

impl<'a> SurfaceCaster<'a> {
    pub fn new(state: &'a EditorState, region: &Aabb) -> Self {
        let map = &state.doc.map;
        let mut surfaces = Vec::new();
        for (id, node) in map.nodes.iter() {
            let surface = match &node.kind {
                NodeKind::Brush(b) if !b.faces.iter().all(|f| state.game.is_tool_texture(&f.data.material)) => Surface::Brush(b),
                NodeKind::Mesh(m) => Surface::Mesh(m),
                NodeKind::Terrain(t) => Surface::Terrain(t),
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
