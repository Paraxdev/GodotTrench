//! Light probes for moving objects, in the form Godot's LightmapGI keeps them: points with the light arriving there as
//! spherical harmonics, the Delaunay tetrahedra between them and a BSP tree that finds the tetrahedron holding a
//! position. The tree and the harmonics follow scene/3d/lightmap_gi.cpp and modules/lightmapper_rd/lm_compute.glsl.

use std::collections::HashMap;

use glam::{DVec3, Vec2, Vec3};

use crate::sampling::{hash, r2};
use crate::trace::{Scene, bilinear};

/// Rays per probe.
const RAYS: u32 = 256;
/// Share of a probe's rays meeting the back of a one sided surface past which it counts as inside a solid.
const INSIDE: f32 = 0.2;
/// Most probes a map gets, the grid spacing grows past it. The BSP build is quadratic in the worst case.
pub const MAX_PROBES: usize = 1000;
/// Godot's tolerance for a point lying on a plane, in meters.
const ON_PLANE: f64 = 1.0 / (1 << 13) as f64;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Probes {
    /// Positions in map units.
    pub points: Vec<Vec3>,
    /// Nine RGB coefficients per point, scaled like Godot's: the projection of the arriving radiance divided by pi.
    pub sh: Vec<[Vec3; 9]>,
    pub tetrahedra: Vec<[u32; 4]>,
    pub bsp: Vec<BspNode>,
}

/// A node of the BSP tree. A child is another node when positive, the tetrahedron `-child - 1` when negative, or
/// nothing at [`BspNode::EMPTY`]. Positions over the plane (`normal . p > d`) go to `over`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BspNode {
    /// Normal and distance, the distance in map units.
    pub plane: [f32; 4],
    pub over: i32,
    pub under: i32,
}

impl BspNode {
    pub const EMPTY: i32 = i32::MIN;
}

/// Probe positions: a grid over the baked geometry `spacing` map units apart, coarser when it would hold more than
/// [`MAX_PROBES`], plus the `extra` points placed by hand. Points inside solids are dropped.
pub fn place(scene: &Scene, spacing: f32, extra: &[Vec3]) -> Vec<Vec3> {
    let (lo, hi) = scene.bounds;
    if spacing <= 0.0 || lo.cmpgt(hi).any() {
        return extra.to_vec();
    }

    let size = hi - lo;
    let mut step = spacing;
    // Cells of the grid, a probe in the middle of each keeps them off the outer walls.
    let cells = |step: f32| (size / step).floor().max(Vec3::ONE).as_uvec3();
    let count = |step: f32| {
        let n = cells(step);
        (n.x * n.y * n.z) as usize
    };
    while count(step) + extra.len() > MAX_PROBES {
        step *= 1.25;
    }

    let n = cells(step);
    // A little off the grid so the tetrahedra are not all degenerate.
    let start = lo + (size - (n - 1).as_vec3() * step) * 0.5;
    let mut out = extra.to_vec();
    for z in 0..n.z {
        for y in 0..n.y {
            for x in 0..n.x {
                let k = hash(x ^ y.rotate_left(10) ^ z.rotate_left(20));
                let jitter = (Vec3::new((k & 255) as f32, ((k >> 8) & 255) as f32, ((k >> 16) & 255) as f32) / 255.0 - 0.5) * (step * 0.05);
                let p = start + Vec3::new(x as f32, y as f32, z as f32) * step + jitter;
                if !inside(scene, p) {
                    out.push(p);
                }
            }
        }
    }

    out
}

fn inside(scene: &Scene, p: Vec3) -> bool {
    let rays = 32u32;
    let seed = hash(p.x.to_bits() ^ p.y.to_bits().rotate_left(11) ^ p.z.to_bits().rotate_left(22));
    let mut backs = 0;
    for k in 0..rays {
        let dir = sphere_dir(r2(seed, k));
        if let Some(hit) = scene.bvh.closest(p, dir, 0.0, scene.far)
            && !hit.front
        {
            let si = scene.tri_surface[hit.tri as usize] as usize;
            let material = scene.input.materials.get(scene.input.surfaces[si].material as usize).copied().unwrap_or_default();
            if !material.double_sided {
                backs += 1;
            }
        }
    }

    backs as f32 > rays as f32 * INSIDE
}

fn sphere_dir(u: Vec2) -> Vec3 {
    let z = u.x * 2.0 - 1.0;
    let r = (1.0 - z * z).max(0.0).sqrt();
    let phi = std::f32::consts::TAU * u.y;
    Vec3::new(r * phi.cos(), r * phi.sin(), z)
}

/// Light arriving at `p` from every direction as spherical harmonics. `source` is the light each atlas texel sends
/// on, before its surface's albedo.
pub fn capture(scene: &Scene, source: &[Vec3], p: Vec3, index: u32) -> [Vec3; 9] {
    let mut sh = [Vec3::ZERO; 9];
    let seed = hash(index.wrapping_mul(0x2c1b_3c6d) ^ 0x49502741);
    for k in 0..RAYS {
        let d = sphere_dir(r2(seed, k));
        let radiance = match scene.bvh.closest(p, d, 0.0, scene.far) {
            None => scene.input.sky.radiance(d),
            Some(hit) => {
                let si = scene.tri_surface[hit.tri as usize] as usize;
                let surface = &scene.input.surfaces[si];
                let material = scene.input.materials.get(surface.material as usize).copied().unwrap_or_default();
                if !hit.front && !material.double_sided {
                    Vec3::ZERO
                } else {
                    let at = p + d * hit.t;
                    let arriving = match scene.surface_chart[si] {
                        Some(c) => bilinear(source, scene.layout.width, scene.layout.height, scene.layout.charts[c as usize].pixel(at)),
                        None => Vec3::ZERO,
                    };
                    material.emission + material.albedo * arriving
                }
            }
        };
        for (c, basis) in sh.iter_mut().zip(basis(d)) {
            *c += radiance * basis;
        }
    }

    sh.map(|c| c * (4.0 / RAYS as f32))
}

/// Godot's order of the nine real spherical harmonics.
fn basis(d: Vec3) -> [f32; 9] {
    [
        0.282095,
        0.488603 * d.y,
        0.488603 * d.z,
        0.488603 * d.x,
        1.092548 * d.x * d.y,
        1.092548 * d.y * d.z,
        0.315392 * (3.0 * d.z * d.z - 1.0),
        1.092548 * d.x * d.z,
        0.546274 * (d.x * d.x - d.y * d.y),
    ]
}

/// Delaunay tetrahedra of `points` by Bowyer-Watson, leaving out near flat ones.
pub fn tetrahedralize(points: &[DVec3]) -> Vec<[u32; 4]> {
    let n = points.len();
    if n < 4 {
        return Vec::new();
    }

    let (mut lo, mut hi) = (DVec3::splat(f64::MAX), DVec3::splat(f64::MIN));
    for p in points {
        lo = lo.min(*p);
        hi = hi.max(*p);
    }

    let center = (lo + hi) * 0.5;
    let r = (hi - lo).length().max(1.0) * 20.0;
    let mut all = points.to_vec();
    all.extend([
        center + DVec3::new(0.0, 0.0, 3.0 * r),
        center + DVec3::new(-2.0 * r, -r, -r),
        center + DVec3::new(2.0 * r, -r, -r),
        center + DVec3::new(0.0, 2.0 * r, -r),
    ]);
    let n32 = n as u32;
    let mut tets = vec![Tet::new(&all, [n32, n32 + 1, n32 + 2, n32 + 3])];
    for (pi, p) in points.iter().enumerate() {
        let mut faces: HashMap<[u32; 3], u32> = HashMap::new();
        tets.retain(|t| {
            if (*p - t.center).length_squared() >= t.r2 * (1.0 + 1e-9) {
                return true;
            }

            for f in t.faces() {
                *faces.entry(f).or_default() += 1;
            }

            false
        });
        for (f, count) in faces {
            if count == 1 {
                let t = Tet::new(&all, [f[0], f[1], f[2], pi as u32]);
                if t.r2.is_finite() {
                    tets.push(t);
                }
            }
        }
    }

    let scale = (hi - lo).length().max(1e-9);
    tets.into_iter()
        .filter(|t| t.v.iter().all(|v| *v < n32))
        .filter(|t| {
            let [a, b, c, d] = t.v.map(|i| all[i as usize]);
            (b - a).cross(c - a).dot(d - a).abs() > scale * scale * scale * 1e-12
        })
        .map(|t| t.v)
        .collect()
}

struct Tet {
    v: [u32; 4],
    center: DVec3,
    r2: f64,
}

impl Tet {
    fn new(points: &[DVec3], v: [u32; 4]) -> Tet {
        let [a, b, c, d] = v.map(|i| points[i as usize]);
        let (ab, ac, ad) = (b - a, c - a, d - a);
        let det = 2.0 * ab.dot(ac.cross(ad));
        let center = if det.abs() < 1e-18 {
            DVec3::splat(f64::NAN)
        } else {
            a + (ac.cross(ad) * ab.length_squared() + ad.cross(ab) * ac.length_squared() + ab.cross(ac) * ad.length_squared()) / det
        };
        Tet { v, center, r2: (center - a).length_squared() }
    }

    fn faces(&self) -> [[u32; 3]; 4] {
        let [a, b, c, d] = self.v;
        [[a, b, c], [a, b, d], [a, c, d], [b, c, d]].map(|mut f| {
            f.sort_unstable();
            f
        })
    }
}

#[derive(Clone, Copy)]
struct Plane {
    normal: DVec3,
    d: f64,
}

impl Plane {
    /// Godot's `Plane(a, b, c)`.
    fn new(a: DVec3, b: DVec3, c: DVec3) -> Option<Plane> {
        let normal = (a - c).cross(a - b).try_normalize()?;
        Some(Plane { normal, d: normal.dot(a) })
    }

    fn same(&self, o: &Plane) -> bool {
        let close = |n: DVec3, d: f64| (self.normal - n).length() < 1e-5 && (self.d - d).abs() < 1e-5;
        close(o.normal, o.d) || close(-o.normal, -o.d)
    }
}

struct Simplex {
    vertices: [u32; 4],
    planes: [usize; 4],
}

const FACE_ORDER: [[usize; 3]; 4] = [[0, 1, 2], [0, 2, 3], [0, 1, 3], [1, 2, 3]];

/// The BSP tree Godot walks to find the tetrahedron holding a point, a port of LightmapGI::_compute_bsp_tree.
/// `points` are in meters, where Godot's tolerances apply.
pub fn bsp(points: &[DVec3], tetrahedra: &[[u32; 4]]) -> Vec<BspNode> {
    if tetrahedra.is_empty() {
        return Vec::new();
    }

    let mut planes: Vec<Plane> = Vec::new();
    let mut simplices = Vec::with_capacity(tetrahedra.len());
    for t in tetrahedra {
        let mut ids = [0; 4];
        for (j, f) in FACE_ORDER.iter().enumerate() {
            let [a, b, c] = f.map(|k| points[t[k] as usize]);
            let Some(p) = Plane::new(a, b, c) else { continue };
            ids[j] = planes.iter().position(|q| q.same(&p)).unwrap_or_else(|| {
                planes.push(p);
                planes.len() - 1
            });
        }

        simplices.push(Simplex { vertices: *t, planes: ids });
    }

    if simplices.len() == 1 {
        // Godot needs a node to start from, one plane with the tetrahedron on both sides does.
        return vec![BspNode { plane: [0.0, 1.0, 0.0, 0.0], over: -1, under: -1 }];
    }

    let indices: Vec<usize> = (0..simplices.len()).collect();
    build(points, &simplices, &indices, planes.len(), 0)
}

/// 1 when the simplex lies over the plane, -1 under, 0 across and None when it is flat on it.
fn side(points: &[DVec3], s: &Simplex, plane: &Plane) -> Option<i32> {
    let (mut over, mut under) = (0, 0);
    for v in s.vertices {
        let dist = plane.normal.dot(points[v as usize]) - plane.d;
        if dist.abs() <= ON_PLANE {
            continue;
        } else if dist > 0.0 {
            over += 1;
        } else {
            under += 1;
        }
    }

    match (under, over) {
        (0, 0) => None,
        (0, _) => Some(1),
        (_, 0) => Some(-1),
        _ => Some(0),
    }
}

/// Subtrees with more simplices than this on both sides build on threads of their own, down to [`PARALLEL_DEPTH`].
const PARALLEL_MIN: usize = 256;
const PARALLEL_DEPTH: u32 = 4;

/// The subtree over `indices`, its root first. Its child indices count from that root, a parent shifts them by where
/// it places the subtree. Godot only needs children after their parent.
fn build(points: &[DVec3], simplices: &[Simplex], indices: &[usize], plane_count: usize, depth: u32) -> Vec<BspNode> {
    let mut tested = vec![false; plane_count];
    let mut best: Option<Plane> = None;
    let mut best_score = -1.0f64;
    // Godot tries the faces of every simplex, which is quadratic. Faces of a spread out sample choose nearly as well.
    let stride = (indices.len() / 48).max(1);
    for &idx in indices.iter().step_by(stride) {
        let s = &simplices[idx];
        for j in 0..4 {
            if tested[s.planes[j]] {
                continue;
            }

            tested[s.planes[j]] = true;
            let [a, b, c] = FACE_ORDER[j].map(|k| points[s.vertices[k] as usize]);
            let Some(plane) = Plane::new(a, b, c) else { continue };
            let (mut over, mut under) = (0usize, 0usize);
            for &other in indices {
                match side(points, &simplices[other], &plane) {
                    Some(v) if v < 0 => under += 1,
                    Some(v) if v > 0 => over += 1,
                    _ => {}
                }
            }

            if under == 0 && over == 0 {
                continue;
            }

            if under > over {
                std::mem::swap(&mut under, &mut over);
            }

            let score = if over > 0 {
                let balance = under as f64 / over as f64;
                let separation = (over + under) as f64 / indices.len() as f64;
                balance * separation * separation
            } else {
                0.0
            };
            if score > best_score {
                best = Some(plane);
                best_score = score;
            }
        }
    }

    // Two simplices that share no separating face: try planes through one vertex of the first and two of the second.
    if best_score == 0.0 {
        let (s0, s1) = (&simplices[indices[0]], &simplices[indices[1]]);
        'search: for i in 0..4 {
            let v0 = points[s0.vertices[i] as usize];
            for j in 0..3 {
                if s0.vertices[i] == s1.vertices[j] {
                    break;
                }

                for k in j + 1..4 {
                    if s0.vertices[i] == s1.vertices[k] {
                        break;
                    }

                    let Some(plane) = Plane::new(v0, points[s1.vertices[j] as usize], points[s1.vertices[k] as usize]) else { continue };
                    let a = side(points, s0, &plane);
                    let b = side(points, s1, &plane);
                    if matches!((a, b), (Some(1), Some(-1)) | (Some(-1), Some(1))) {
                        best = Some(plane);
                        best_score = 1.0;
                        break 'search;
                    }
                }
            }
        }
    }

    let (mut over, mut under) = (Vec::new(), Vec::new());
    if let Some(plane) = &best {
        for &idx in indices {
            match side(points, &simplices[idx], plane) {
                None => {}
                Some(v) => {
                    if v <= 0 {
                        under.push(idx);
                    }

                    if v >= 0 {
                        over.push(idx);
                    }
                }
            }
        }
    }

    if best_score < 0.0 || best.is_none() || over.len() == indices.len() || under.len() == indices.len() {
        // No plane separates them, Delaunay broke down somewhere. Split off the lowest simplex along the longest
        // axis like Godot does, it interpolates a little worse there but always finds a tetrahedron.
        let (mut lo, mut hi) = (DVec3::splat(f64::MAX), DVec3::splat(f64::MIN));
        let centers: Vec<DVec3> = indices
            .iter()
            .map(|&idx| {
                let (mut a, mut b) = (DVec3::splat(f64::MAX), DVec3::splat(f64::MIN));
                for v in simplices[idx].vertices {
                    a = a.min(points[v as usize]);
                    b = b.max(points[v as usize]);
                }

                lo = lo.min(a);
                hi = hi.max(b);
                (a + b) * 0.5
            })
            .collect();
        let size = hi - lo;
        let axis = if size.x >= size.y && size.x >= size.z {
            0
        } else if size.y >= size.z {
            1
        } else {
            2
        };
        let lowest = (0..centers.len()).min_by(|a, b| centers[*a][axis].total_cmp(&centers[*b][axis])).unwrap_or(0);
        let mut normal = DVec3::ZERO;
        normal[axis] = 1.0;
        best = Some(Plane { normal, d: centers[lowest][axis] });
        under = vec![indices[lowest]];
        over = indices.iter().enumerate().filter(|(i, _)| *i != lowest).map(|(_, idx)| *idx).collect();
    }

    let subtree = |list: &[usize]| if list.len() > 1 { build(points, simplices, list, plane_count, depth + 1) } else { Vec::new() };
    let (under_nodes, over_nodes) = if depth < PARALLEL_DEPTH && under.len().min(over.len()) > PARALLEL_MIN {
        std::thread::scope(|s| {
            let job = s.spawn(|| subtree(&under));
            let over_nodes = subtree(&over);
            (job.join().unwrap_or_default(), over_nodes)
        })
    } else {
        (subtree(&under), subtree(&over))
    };

    let child = |list: &[usize], at: usize| match list.len() {
        0 => BspNode::EMPTY,
        1 => -(list[0] as i32 + 1),
        _ => at as i32,
    };
    let plane = best.expect("a plane was chosen above");
    let over_at = 1 + under_nodes.len();
    let mut nodes = Vec::with_capacity(over_at + over_nodes.len());
    nodes.push(BspNode {
        plane: [plane.normal.x as f32, plane.normal.y as f32, plane.normal.z as f32, plane.d as f32],
        over: child(&over, over_at),
        under: child(&under, 1),
    });
    for (offset, part) in [(1, under_nodes), (over_at, over_nodes)] {
        nodes.extend(part.into_iter().map(|mut n| {
            for c in [&mut n.over, &mut n.under] {
                if *c >= 0 {
                    *c += offset as i32;
                }
            }

            n
        }));
    }

    nodes
}

/// The tetrahedron holding `p` by walking the tree like Godot does, for tests.
pub fn find(nodes: &[BspNode], p: DVec3) -> Option<usize> {
    let mut node = 0i32;
    while node >= 0 {
        let n = nodes.get(node as usize)?;
        let over = DVec3::new(n.plane[0] as f64, n.plane[1] as f64, n.plane[2] as f64).dot(p) > n.plane[3] as f64;
        node = if over { n.over } else { n.under };
    }

    (node != BspNode::EMPTY).then(|| (-node - 1) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid(n: u32, jitter: f64) -> Vec<DVec3> {
        let mut out = Vec::new();
        for z in 0..n {
            for y in 0..n {
                for x in 0..n {
                    let k = hash(x * 7919 + y * 104729 + z * 1299709);
                    let j = DVec3::new((k & 255) as f64, ((k >> 8) & 255) as f64, ((k >> 16) & 255) as f64) / 255.0 - 0.5;
                    out.push(DVec3::new(x as f64, y as f64, z as f64) * 2.0 + j * jitter);
                }
            }
        }

        out
    }

    fn volume(points: &[DVec3], t: &[u32; 4]) -> f64 {
        let [a, b, c, d] = t.map(|i| points[i as usize]);
        (b - a).cross(c - a).dot(d - a).abs() / 6.0
    }

    #[test]
    fn tetrahedra_fill_the_hull_without_overlap() {
        for jitter in [0.0, 0.2] {
            let points = grid(4, jitter);
            let tets = tetrahedralize(&points);
            let total: f64 = tets.iter().map(|t| volume(&points, t)).sum();
            // Hull of a regular grid is the 6 m cube, jitter changes it a little.
            assert!((total - 216.0).abs() < 216.0 * 0.08, "jitter {jitter}: volume {total}");
        }
    }

    #[test]
    fn the_tree_finds_the_tetrahedron_holding_a_point() {
        // Big enough that subtrees build on threads of their own.
        let points = grid(8, 0.2);
        let tets = tetrahedralize(&points);
        let nodes = bsp(&points, &tets);
        for (i, n) in nodes.iter().enumerate() {
            for child in [n.over, n.under] {
                assert!(child < 0 || child as usize > i, "children come after their parent, as Godot checks");
            }
        }

        let mut found = 0;
        for k in 0..200u32 {
            let u = r2(7, k);
            let p = DVec3::new(0.5 + u.x as f64 * 13.0, 0.5 + u.y as f64 * 13.0, 0.5 + ((k * 37) % 100) as f64 / 100.0 * 13.0);
            let Some(t) = find(&nodes, p) else { continue };
            let [a, b, c, d] = tets[t].map(|i| points[i as usize]);
            let parts =
                [(p, b, c, d), (a, p, c, d), (a, b, p, d), (a, b, c, p)].map(|(a, b, c, d)| (b - a).cross(c - a).dot(d - a).abs() / 6.0).iter().sum::<f64>();
            assert!((parts - volume(&points, &tets[t])).abs() < 1e-6, "point {p} is inside the tetrahedron the tree found");
            found += 1;
        }

        assert!(found > 190, "points inside the hull find a tetrahedron, {found} of 200");
    }

    #[test]
    fn a_uniform_sky_gives_godots_ambient() {
        // Godot evaluates c4 * sh[0] for the ambient light, which has to come out as the sky radiance.
        let mut sh0 = 0.0;
        for k in 0..RAYS {
            sh0 += basis(sphere_dir(r2(3, k)))[0];
        }

        let ambient = 0.886227 * sh0 * 4.0 / RAYS as f32;
        assert!((ambient - 1.0).abs() < 0.01, "{ambient}");
    }
}
