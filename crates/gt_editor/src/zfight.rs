//! Finds faces of different brushes that lie on one plane and overlap, which z-fight once the map is built in Godot.
//! The build (godottrench_face_cull.gd) only hides a face that other faces cover completely or that is buried in a
//! solid, so a partly covered face still draws where it overlaps its neighbour.

use std::collections::{BTreeMap, HashMap, HashSet};

use gt_core::{Aabb, DVec2, DVec3, NodeId};
use gt_doc::Map;
use gt_doc::issues::{Issue, Severity};
use gt_formats::GameConfig;
use gt_geom::{FaceUv, Terrain, polygon};

use crate::face_cull::{self, COPLANAR_DIST, CullFace, SAME_NORMAL};
use crate::materials::MaterialLibrary;

/// Faces closer than this to each other's plane fight over depth at any distance.
const PLANE_DIST: f64 = 0.05;
/// Overlaps narrower than this many map units, like the edge of a thin decal or pane against a wall, are too thin
/// to notice.
const MIN_WIDTH: f64 = 3.0;
/// A face looking down at the terrain from less than this many units above it, like the foot of a post, is only
/// seen from underground.
const GROUND_CLEARANCE: f64 = 4.0;
/// A one sided face that looks at an opaque face closer than this, like the back of a decal on a wall, is never seen.
const BACKING_GAP: f64 = 1.0;

/// How a face material draws in Godot.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Look {
    /// Blended, cut out or dithered, the build never culls against it.
    pub see_through: bool,
    /// Draws its back too, so it fights a face it lies back to back with.
    pub double_sided: bool,
    /// Map units one repeat of the texture covers, when known. Faces whose textures line up draw the same pixels.
    pub repeat: Option<DVec2>,
}

struct Face {
    id: NodeId,
    cull: CullFace,
    /// Takes part in the build's face culling, so it can be hidden there.
    culls: bool,
    double_sided: bool,
    material: String,
    uv: FaceUv,
    repeat: Option<DVec2>,
}

type NormalKey = (i64, i64, i64);

fn normal_key(n: DVec3) -> NormalKey {
    ((n.x * 1000.0).round() as i64, (n.y * 1000.0).round() as i64, (n.z * 1000.0).round() as i64)
}

/// Planes are bucketed one map unit deep, so faces within [`PLANE_DIST`] share a bucket or sit in neighbouring ones.
fn depth_key(dist: f64) -> i64 {
    dist.floor() as i64
}

/// Tool textures, hints and nodraw faces build no visual mesh.
fn draws(game: &GameConfig, material: &str) -> bool {
    let lower = material.to_ascii_lowercase();
    let name = lower.rsplit('/').next().unwrap_or_default();
    !lower.is_empty()
        && !game.is_tool_texture(&lower)
        && !lower.starts_with("special/")
        && !lower.starts_with("tools/")
        && !matches!(name, "clip" | "skip" | "origin" | "sky" | "trigger" | "nodraw" | "hint" | "hintskip" | "caulk" | "null" | "areaportal")
}

fn gather(map: &Map, game: &GameConfig, look: &dyn Fn(&str) -> Look) -> Vec<Face> {
    let mut out = Vec::new();
    for (id, b) in map.brushes() {
        let owner = map.owning_entity(id).and_then(|e| map.entity(e));
        if owner.is_some_and(|e| e.classname.starts_with("trigger")) || b.faces.iter().any(|f| f.data.disp.is_some()) {
            continue;
        }

        let fixed = !face_cull::skipped_entity(map, game, id);
        for (fi, f) in b.faces.iter().enumerate() {
            if !draws(game, &f.data.material) || f.indices.len() < 3 {
                continue;
            }

            let polygon: Vec<DVec3> = f.indices.iter().map(|i| b.vertices[*i as usize]).collect();
            let normal = f.plane.normal;
            let dist = normal.dot(polygon::centroid(&polygon));
            let l = look(&f.data.material);
            let cull = CullFace {
                face: fi,
                key: face_cull::plane_key(normal, dist),
                normal,
                dist,
                bounds: Aabb::from_points(polygon.iter().copied()),
                polygon,
                closed: true,
                is_mesh: false,
            };
            out.push(Face {
                id,
                cull,
                culls: fixed && !l.see_through && !l.double_sided,
                double_sided: l.double_sided,
                material: f.data.material.to_ascii_lowercase(),
                uv: f.data.uv.clone(),
                repeat: l.repeat,
            });
        }
    }

    out
}

/// The part of convex polygon `a` inside convex polygon `b`, both on (nearly) one plane with `normal`.
fn overlap(a: &[DVec3], b: &[DVec3], normal: DVec3) -> Vec<DVec3> {
    let center = polygon::centroid(b);
    let mut rest = a.to_vec();
    for i in 0..b.len() {
        let (p, q) = (b[i], b[(i + 1) % b.len()]);
        let mut inward = normal.cross(q - p).normalize_or_zero();
        if inward == DVec3::ZERO {
            continue;
        }

        if inward.dot(center - p) < 0.0 {
            inward = -inward;
        }

        rest = polygon::split(&rest, inward, inward.dot(p), 1e-6).0;
        if rest.len() < 3 {
            return Vec::new();
        }
    }

    rest
}

/// Height of the terrain surface under x, z, where there is terrain and no hole in it.
fn ground_height(terrains: &[&Terrain], x: f64, z: f64) -> Option<f64> {
    terrains
        .iter()
        .filter_map(|t| {
            let h = t.height_at(x, z)?;
            let [cw, cd] = t.cells();
            let cell = |v: f64, n: u32| ((v / t.cell_size).floor().max(0.0) as u32).min(n.saturating_sub(1));
            (!t.is_hole(cell(x - t.origin.x, cw), cell(z - t.origin.z, cd))).then_some(h)
        })
        .reduce(f64::max)
}

/// The smaller side of the polygon's extent within its plane.
fn width(points: &[DVec3], normal: DVec3) -> f64 {
    let (u, v) = gt_core::Plane::from_point_normal(DVec3::ZERO, normal).basis();
    let span = |axis: DVec3| {
        let (lo, hi) = points.iter().fold((f64::MAX, f64::MIN), |(lo, hi), p| (lo.min(p.dot(axis)), hi.max(p.dot(axis))));
        hi - lo
    };
    span(u).min(span(v))
}

/// Same material mapped the same way over `points`, so both faces draw the same pixels and the fight never shows.
fn same_pixels(f: &Face, g: &Face, points: &[DVec3]) -> bool {
    f.material == g.material
        && points.iter().all(|p| {
            let d = f.uv.texel(*p) - g.uv.texel(*p);
            let d = f.repeat.map_or(d, |r| d - (d / r).round() * r);
            d.abs().max_element() < 0.01
        })
}

struct Found {
    face: usize,
    other_face: usize,
    at: DVec3,
    area: f64,
    back_to_back: bool,
}

/// Faces of different brushes on one plane that overlap and would both draw in the Godot build: facing the same
/// way, or back to back where one of them draws both sides. Left out are tool textures, faces nobody sees (covered by
/// faces that win over them, looking at an opaque face close by, buried in a solid or standing on terrain), thin
/// overlaps and matching textures. One warning per brush pair.
pub fn coplanar_overlaps(map: &Map, game: &GameConfig, look: &dyn Fn(&str) -> Look) -> Vec<Issue> {
    let faces = gather(map, game, look);
    let mut buckets: HashMap<(NormalKey, i64), Vec<usize>> = HashMap::new();
    for (i, f) in faces.iter().enumerate() {
        buckets.entry((normal_key(f.cull.normal), depth_key(f.cull.dist))).or_default().push(i);
    }

    let buckets = &buckets;
    let near = |n: DVec3, dist: f64| {
        let (nk, d) = (normal_key(n), depth_key(dist));
        (d - 1..=d + 1).flat_map(move |k| buckets.get(&(nk, k)).into_iter().flatten().copied())
    };

    let mut candidates: Vec<(usize, usize, bool)> = Vec::new();
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    for (i, f) in faces.iter().enumerate() {
        let reach = f.cull.bounds.expanded(PLANE_DIST);
        let mut consider = |j: usize, back_to_back: bool| {
            let g = &faces[j];
            let dot = f.cull.normal.dot(g.cull.normal);
            let fits = if back_to_back {
                dot < -SAME_NORMAL && (f.cull.dist + g.cull.dist).abs() < PLANE_DIST
            } else {
                dot > SAME_NORMAL && (f.cull.dist - g.cull.dist).abs() < PLANE_DIST
            };
            if g.id != f.id && fits && reach.intersects(&g.cull.bounds) && seen.insert((i.min(j), i.max(j))) {
                candidates.push((i, j, back_to_back));
            }
        };
        for j in near(f.cull.normal, f.cull.dist) {
            consider(j, false);
        }

        if f.double_sided {
            for j in near(-f.cull.normal, -f.cull.dist) {
                consider(j, true);
            }
        }
    }

    let solids: Vec<(NodeId, Aabb)> = map
        .brushes()
        .filter(|(id, b)| {
            !face_cull::skipped_entity(map, game, *id)
                && b.faces.iter().all(|f| {
                    f.data.disp.is_none() && draws(game, &f.data.material) && {
                        let l = look(&f.data.material);
                        !l.see_through && !l.double_sided
                    }
                })
        })
        .map(|(id, b)| (id, b.bounds()))
        .collect();
    let terrains: Vec<&Terrain> = map.terrains().map(|(_, t)| t).collect();
    let mut hidden: HashMap<usize, bool> = HashMap::new();
    let mut mesh_tris = HashMap::new();
    let mut is_hidden = |i: usize| -> bool {
        if let Some(h) = hidden.get(&i) {
            return *h;
        }

        // Beyond what the build hides, faces only visible from inside a solid or from underground never show.
        let f = &faces[i];
        let on_ground = f.cull.normal.y < -0.5
            && f.cull
                .polygon
                .iter()
                .chain([&polygon::centroid(&f.cull.polygon)])
                .all(|p| ground_height(&terrains, p.x, p.z).is_some_and(|h| p.y <= h + GROUND_CLEARANCE));
        let h = on_ground || {
            let mut covers: Vec<&[DVec3]> = Vec::new();
            let same = near(f.cull.normal, f.cull.dist).map(|j| (j, false));
            let back = near(-f.cull.normal, -f.cull.dist).map(|j| (j, true));
            for (j, back_to_back) in same.chain(back) {
                let g = &faces[j];
                if g.id == f.id || !g.culls || !g.cull.bounds.expanded(BACKING_GAP).intersects(&f.cull.bounds) {
                    continue;
                }

                let dot = f.cull.normal.dot(g.cull.normal);
                let covering = if back_to_back {
                    let gap = -g.cull.dist - f.cull.dist;
                    !f.double_sided && dot < -SAME_NORMAL && gap > -COPLANAR_DIST && gap < BACKING_GAP
                } else {
                    f.culls
                        && dot > SAME_NORMAL
                        && (f.cull.dist - g.cull.dist).abs() < COPLANAR_DIST
                        && face_cull::priority(&g.cull, g.id) < face_cull::priority(&f.cull, f.id)
                };
                if covering {
                    covers.push(&g.cull.polygon);
                }
            }

            let covered = !covers.is_empty() && polygon::visible_pieces(&f.cull.polygon, f.cull.normal, &covers).is_some_and(|p| p.is_empty());
            covered || solids.iter().any(|(s, b)| *s != f.id && b.contains(&f.cull.bounds) && face_cull::buried_in(map, *s, &f.cull, &mut mesh_tris))
        };
        hidden.insert(i, h);
        h
    };

    let mut pairs: BTreeMap<(NodeId, NodeId), Vec<Found>> = BTreeMap::new();
    for (i, j, back_to_back) in candidates {
        let (f, g) = (&faces[i], &faces[j]);
        let shared = overlap(&f.cull.polygon, &g.cull.polygon, f.cull.normal);
        if shared.len() < 3 || width(&shared, f.cull.normal) < MIN_WIDTH || (!back_to_back && same_pixels(f, g, &shared)) || is_hidden(i) || is_hidden(j) {
            continue;
        }

        let area = polygon::area(&shared);
        let (later, earlier) = if f.id > g.id { (f, g) } else { (g, f) };
        pairs.entry((later.id, earlier.id)).or_default().push(Found {
            face: later.cull.face,
            other_face: earlier.cull.face,
            at: polygon::centroid(&shared),
            area,
            back_to_back,
        });
    }

    pairs
        .into_iter()
        .filter_map(|((id, other), found)| {
            let n = found.len();
            let top = found.into_iter().max_by(|a, b| a.area.total_cmp(&b.area))?;
            let how = if top.back_to_back { "lies back to back with" } else { "overlaps" };
            let also = if n > 1 { format!(", as do {} more of their faces", n - 1) } else { String::new() };
            let (x, y, z) = (top.at.x, top.at.y, top.at.z);
            Some(Issue {
                node: Some(id),
                severity: Severity::Warning,
                code: "coplanar_faces",
                message: format!(
                    "Face {} {how} face {} of {other} on the same plane around ({x:.0}, {y:.0}, {z:.0}), they z-fight in Godot{also}",
                    top.face, top.other_face
                ),
            })
        })
        .collect()
}

/// [`coplanar_overlaps`] with materials as the project's material files describe them.
pub fn coplanar_issues(map: &Map, game: &GameConfig, materials: &mut MaterialLibrary) -> Vec<Issue> {
    let mut looks: HashMap<String, Look> = HashMap::new();
    for (_, b) in map.brushes() {
        for f in &b.faces {
            if !looks.contains_key(&f.data.material) {
                let info = materials.info(&f.data.material);
                let look = Look {
                    see_through: info.as_ref().is_some_and(|m| m.transparency != gt_formats::godot_material::Transparency::Opaque),
                    double_sided: info.as_ref().is_some_and(|m| m.double_sided),
                    repeat: materials.size(&f.data.material).map(DVec2::from_array),
                };
                looks.insert(f.data.material.clone(), look);
            }
        }
    }

    coplanar_overlaps(map, game, &|m| looks.get(m).copied().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_doc::NodeKind;
    use gt_geom::Brush;

    fn add(map: &mut Map, min: DVec3, max: DVec3, material: &str) -> NodeId {
        let layer = map.default_layer();
        map.insert(layer, NodeKind::Brush(Brush::from_aabb(&Aabb::new(min, max), material).unwrap()))
    }

    fn look(m: &str) -> Look {
        Look { see_through: m.starts_with("decals/"), double_sided: m == "decals/both_sides", repeat: None }
    }

    fn check(map: &Map) -> Vec<Issue> {
        coplanar_overlaps(map, &GameConfig::default(), &look)
    }

    #[test]
    fn partly_overlapping_faces_on_one_plane_are_reported() {
        let mut map = Map::new();
        let a = add(&mut map, DVec3::ZERO, DVec3::new(64.0, 64.0, 16.0), "dev/grey");
        let b = add(&mut map, DVec3::new(48.0, 16.0, 0.0), DVec3::new(112.0, 64.0, 16.0), "dev/dark");
        let issues = check(&map);
        assert_eq!(issues.len(), 1, "{issues:?}");
        assert_eq!((issues[0].node, issues[0].code), (Some(b), "coplanar_faces"));
        assert!(issues[0].message.contains(&a.to_string()), "{}", issues[0].message);
    }

    #[test]
    fn a_face_the_build_hides_is_not_reported() {
        let mut map = Map::new();
        add(&mut map, DVec3::ZERO, DVec3::new(128.0, 64.0, 128.0), "dev/grey");
        add(&mut map, DVec3::new(32.0, 0.0, 32.0), DVec3::new(64.0, 64.0, 64.0), "dev/dark");
        assert!(check(&map).is_empty(), "the older block covers the newer one's top and bottom and buries its sides: {:?}", check(&map));

        // The other way round the small block wins the shared top, the big top is only partly covered and still draws.
        let mut map = Map::new();
        let small = add(&mut map, DVec3::new(32.0, 0.0, 32.0), DVec3::new(64.0, 64.0, 64.0), "dev/dark");
        let big = add(&mut map, DVec3::ZERO, DVec3::new(128.0, 64.0, 128.0), "dev/grey");
        let issues = check(&map);
        assert_eq!(issues.len(), 1, "{issues:?}");
        assert_eq!(issues[0].node, Some(big));
        assert!(issues[0].message.contains(&small.to_string()) && issues[0].message.contains("as do 1 more"), "{}", issues[0].message);
    }

    #[test]
    fn see_through_layers_at_one_depth_are_reported_and_tool_faces_skipped() {
        let wall = |map: &mut Map| add(map, DVec3::new(-64.0, -64.0, -16.0), DVec3::new(256.0, 256.0, 0.0), "dev/grey");
        let mut map = Map::new();
        wall(&mut map);
        add(&mut map, DVec3::new(0.0, 0.0, 0.3), DVec3::new(64.0, 64.0, 0.6), "decals/grime");
        let stain = add(&mut map, DVec3::new(32.0, 16.0, 0.3), DVec3::new(96.0, 48.0, 0.6), "decals/stain");
        add(&mut map, DVec3::new(0.0, 128.0, 0.0), DVec3::new(64.0, 192.0, 16.0), "special/clip");
        add(&mut map, DVec3::new(32.0, 128.0, 0.0), DVec3::new(96.0, 192.0, 16.0), "special/clip");
        let issues = check(&map);
        assert_eq!(issues.len(), 1, "only the decal fronts fight, their backs face the wall from a hair away: {issues:?}");
        assert_eq!(issues[0].node, Some(stain));
        assert!(!issues[0].message.contains("more of their faces"), "{}", issues[0].message);

        let mut map = Map::new();
        wall(&mut map);
        add(&mut map, DVec3::new(0.0, 0.0, 0.0), DVec3::new(64.0, 64.0, 0.6), "decals/grime");
        add(&mut map, DVec3::new(32.0, 16.0, 0.0), DVec3::new(96.0, 48.0, 0.75), "decals/stain");
        assert!(check(&map).is_empty(), "stepped out by 0.15 units the fronts no longer share a plane: {:?}", check(&map));
    }

    #[test]
    fn a_double_sided_face_on_a_wall_is_reported() {
        let mut map = Map::new();
        add(&mut map, DVec3::new(0.0, 0.0, -16.0), DVec3::new(128.0, 128.0, 0.0), "dev/grey");
        add(&mut map, DVec3::new(32.0, 32.0, 0.0), DVec3::new(64.0, 64.0, 0.6), "decals/grime");
        assert!(check(&map).is_empty(), "a one sided decal's back faces into the wall and never draws");

        let mut map = Map::new();
        add(&mut map, DVec3::new(0.0, 0.0, -16.0), DVec3::new(128.0, 128.0, 0.0), "dev/grey");
        let decal = add(&mut map, DVec3::new(32.0, 32.0, 0.0), DVec3::new(64.0, 64.0, 0.6), "decals/both_sides");
        let issues = check(&map);
        assert_eq!(issues.len(), 1, "{issues:?}");
        assert_eq!(issues[0].node, Some(decal));
        assert!(issues[0].message.contains("back to back"), "{}", issues[0].message);
    }

    #[test]
    fn matching_textures_and_thin_slivers_are_not_reported() {
        let mut map = Map::new();
        add(&mut map, DVec3::ZERO, DVec3::new(64.0, 64.0, 16.0), "dev/grey");
        let b = add(&mut map, DVec3::new(48.0, 16.0, 0.0), DVec3::new(112.0, 64.0, 16.0), "dev/grey");
        assert!(check(&map).is_empty(), "one material mapped the same way draws the same pixels: {:?}", check(&map));
        for f in &mut map.brush_mut(b).unwrap().faces {
            f.data.uv.offset = DVec2::new(8.0, 0.0);
        }

        assert_eq!(check(&map).len(), 1, "shifted, the texture flickers between the two");

        let pane = |thickness: f64| {
            let mut map = Map::new();
            add(&mut map, DVec3::new(0.0, 0.0, -8.0), DVec3::new(64.0, 64.0, 8.0), "dev/grey");
            add(&mut map, DVec3::new(64.0, 16.0, -thickness), DVec3::new(112.0, 48.0, 0.0), "decals/both_sides");
            check(&map)
        };
        assert!(pane(2.0).is_empty(), "the two unit wide edge of a two sided pane against its frame: {:?}", pane(2.0));
        assert_eq!(pane(4.0).len(), 1, "a thick one shows");
    }

    #[test]
    fn feet_standing_on_the_terrain_are_not_reported() {
        let mut map = Map::new();
        let layer = map.default_layer();
        let mut ground = Terrain::new(DVec3::new(-512.0, 0.0, -512.0), [9, 9], 128.0, "dev/green");
        ground.heights.iter_mut().for_each(|h| *h = 2.0);
        map.insert(layer, NodeKind::Terrain(ground));
        add(&mut map, DVec3::new(0.0, 0.0, 0.0), DVec3::new(16.0, 64.0, 16.0), "dev/grey");
        add(&mut map, DVec3::new(8.0, 0.0, 4.0), DVec3::new(128.0, 48.0, 12.0), "dev/dark");
        assert!(check(&map).is_empty(), "the post and the panel stand on the ground: {:?}", check(&map));

        let mut map = Map::new();
        add(&mut map, DVec3::new(0.0, 0.0, 0.0), DVec3::new(16.0, 64.0, 16.0), "dev/grey");
        add(&mut map, DVec3::new(8.0, 0.0, 4.0), DVec3::new(128.0, 48.0, 12.0), "dev/dark");
        assert_eq!(check(&map).len(), 1, "floating, their undersides show");
    }

    #[test]
    fn trigger_volumes_draw_nothing() {
        let mut map = Map::new();
        let layer = map.default_layer();
        add(&mut map, DVec3::ZERO, DVec3::new(64.0, 64.0, 16.0), "dev/grey");
        let trigger = map.insert(layer, NodeKind::Entity(gt_doc::Entity::new("trigger_once")));
        map.insert(trigger, NodeKind::Brush(Brush::from_aabb(&Aabb::new(DVec3::new(48.0, 16.0, 0.0), DVec3::new(112.0, 64.0, 16.0)), "dev/orange").unwrap()));
        assert!(check(&map).is_empty(), "{:?}", check(&map));
    }
}
