//! Bounding volume hierarchy over the scene's triangles, built with binned SAH. The node and triangle layouts are
//! plain arrays of 16 byte rows, so the GPU tracer can upload them as they are.

use glam::Vec3;

/// A triangle as its first corner and two edges, ready for the Möller-Trumbore test.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Tri {
    pub a: [f32; 3],
    /// Index of the triangle in the scene's triangle list.
    pub id: u32,
    pub e1: [f32; 3],
    pub _pad1: u32,
    pub e2: [f32; 3],
    pub _pad2: u32,
}

/// A node's box. Inner nodes keep their left child right after themselves and `start` names the right one, leaves
/// hold `count` triangles from `start`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Node {
    pub min: [f32; 3],
    pub start: u32,
    pub max: [f32; 3],
    pub count: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub t: f32,
    /// Index into the scene's triangle list.
    pub tri: u32,
    pub u: f32,
    pub v: f32,
    /// The ray met the side the triangle's normal points to.
    pub front: bool,
}

pub struct Bvh {
    pub nodes: Vec<Node>,
    pub tris: Vec<Tri>,
}

const BINS: usize = 12;
const LEAF: usize = 4;
const STACK: usize = 64;

#[derive(Clone, Copy)]
struct Bounds {
    min: Vec3,
    max: Vec3,
}

impl Bounds {
    const EMPTY: Bounds = Bounds { min: Vec3::splat(f32::MAX), max: Vec3::splat(f32::MIN) };

    fn grow(&mut self, p: Vec3) {
        self.min = self.min.min(p);
        self.max = self.max.max(p);
    }

    fn join(&mut self, o: &Bounds) {
        self.min = self.min.min(o.min);
        self.max = self.max.max(o.max);
    }

    fn area(&self) -> f32 {
        let d = (self.max - self.min).max(Vec3::ZERO);
        d.x * d.y + d.y * d.z + d.z * d.x
    }
}

impl Bvh {
    /// Builds the tree over `triangles`, each given by its three corners. Degenerate triangles are left out.
    pub fn build(triangles: &[[Vec3; 3]]) -> Bvh {
        let mut tris: Vec<Tri> = Vec::with_capacity(triangles.len());
        let mut boxes = Vec::with_capacity(triangles.len());
        let mut centers = Vec::with_capacity(triangles.len());
        for (i, [a, b, c]) in triangles.iter().enumerate() {
            let (e1, e2) = (*b - *a, *c - *a);
            if e1.cross(e2).length_squared() <= 1e-12 || !(a.is_finite() && b.is_finite() && c.is_finite()) {
                continue;
            }

            tris.push(Tri { a: a.to_array(), id: i as u32, e1: e1.to_array(), _pad1: 0, e2: e2.to_array(), _pad2: 0 });
            let mut bb = Bounds::EMPTY;
            bb.grow(*a);
            bb.grow(*b);
            bb.grow(*c);
            centers.push((bb.min + bb.max) * 0.5);
            boxes.push(bb);
        }

        let mut order: Vec<u32> = (0..tris.len() as u32).collect();
        let mut nodes = Vec::with_capacity(tris.len().max(1) * 2);
        nodes.push(Node::default());
        if !tris.is_empty() {
            Self::split(&mut nodes, 0, &mut order, 0, &boxes, &centers);
        }

        let tris = order.iter().map(|i| tris[*i as usize]).collect();
        Bvh { nodes, tris }
    }

    fn split(nodes: &mut Vec<Node>, node: usize, order: &mut [u32], offset: usize, boxes: &[Bounds], centers: &[Vec3]) {
        let mut bb = Bounds::EMPTY;
        let mut cb = Bounds::EMPTY;
        for i in order.iter() {
            bb.join(&boxes[*i as usize]);
            cb.grow(centers[*i as usize]);
        }

        nodes[node].min = bb.min.to_array();
        nodes[node].max = bb.max.to_array();
        let n = order.len();
        let make_leaf = |nodes: &mut Vec<Node>| {
            nodes[node].start = offset as u32;
            nodes[node].count = n as u32;
        };
        if n <= LEAF {
            return make_leaf(nodes);
        }

        let extent = cb.max - cb.min;
        let mut best = (f32::MAX, 0usize, 0usize);
        for axis in 0..3 {
            if extent[axis] <= 1e-6 {
                continue;
            }

            let mut bins = [(Bounds::EMPTY, 0usize); BINS];
            let scale = BINS as f32 / extent[axis];
            let bin_of = |c: Vec3| (((c[axis] - cb.min[axis]) * scale) as usize).min(BINS - 1);
            for i in order.iter() {
                let b = &mut bins[bin_of(centers[*i as usize])];
                b.0.join(&boxes[*i as usize]);
                b.1 += 1;
            }

            let mut right = [0.0f32; BINS];
            let (mut acc, mut count) = (Bounds::EMPTY, 0);
            for k in (1..BINS).rev() {
                acc.join(&bins[k].0);
                count += bins[k].1;
                right[k] = if count > 0 { acc.area() * count as f32 } else { 0.0 };
            }

            let (mut acc, mut count) = (Bounds::EMPTY, 0);
            for k in 0..BINS - 1 {
                acc.join(&bins[k].0);
                count += bins[k].1;
                if count == 0 || count == n {
                    continue;
                }

                let cost = acc.area() * count as f32 + right[k + 1];
                if cost < best.0 {
                    best = (cost, axis, k);
                }
            }
        }

        let leaf_cost = bb.area() * n as f32;
        if best.0 == f32::MAX || (best.0 >= leaf_cost && n <= 16) {
            if best.0 == f32::MAX && n > LEAF {
                // All centers coincide, halve the list so the leaves stay small.
                return Self::children(nodes, node, order, offset, n / 2, boxes, centers);
            }

            return make_leaf(nodes);
        }

        let (_, axis, k) = best;
        let scale = BINS as f32 / extent[axis];
        let mut mid = 0;
        for j in 0..n {
            let c = centers[order[j] as usize];
            if ((((c[axis] - cb.min[axis]) * scale) as usize).min(BINS - 1)) <= k {
                order.swap(j, mid);
                mid += 1;
            }
        }

        Self::children(nodes, node, order, offset, mid, boxes, centers);
    }

    fn children(nodes: &mut Vec<Node>, node: usize, order: &mut [u32], offset: usize, mid: usize, boxes: &[Bounds], centers: &[Vec3]) {
        let (left, right) = order.split_at_mut(mid);
        let l = nodes.len();
        nodes.push(Node::default());
        Self::split(nodes, l, left, offset, boxes, centers);
        let r = nodes.len();
        nodes.push(Node::default());
        Self::split(nodes, r, right, offset + mid, boxes, centers);
        nodes[node].start = r as u32;
        nodes[node].count = 0;
    }

    /// The nearest triangle along the ray between `t_min` and `t_max`.
    pub fn closest(&self, origin: Vec3, dir: Vec3, t_min: f32, t_max: f32) -> Option<Hit> {
        let mut best: Option<Hit> = None;
        let mut t_far = t_max;
        self.walk(origin, dir, t_min, &mut t_far, |hit| {
            best = Some(hit);
            false
        });
        best
    }

    /// Whether anything blocks the ray between `t_min` and `t_max`.
    pub fn occluded(&self, origin: Vec3, dir: Vec3, t_min: f32, t_max: f32) -> bool {
        let mut hit = false;
        let mut t_far = t_max;
        self.walk(origin, dir, t_min, &mut t_far, |_| {
            hit = true;
            true
        });
        hit
    }

    /// Visits the triangle hits nearer than `t_far`, which each hit shrinks. `on_hit` returns true to stop.
    fn walk(&self, origin: Vec3, dir: Vec3, t_min: f32, t_far: &mut f32, mut on_hit: impl FnMut(Hit) -> bool) {
        if self.tris.is_empty() {
            return;
        }

        let inv = Vec3::new(safe_inv(dir.x), safe_inv(dir.y), safe_inv(dir.z));
        let mut stack = [0u32; STACK];
        let mut top = 0;
        let mut node = 0usize;
        loop {
            let n = &self.nodes[node];
            if n.count > 0 {
                for k in n.start..n.start + n.count {
                    let tri = &self.tris[k as usize];
                    if let Some(hit) = intersect(tri, origin, dir, t_min, *t_far) {
                        *t_far = hit.t;
                        if on_hit(hit) {
                            return;
                        }
                    }
                }
            } else {
                let (l, r) = (node + 1, n.start as usize);
                let tl = slab(&self.nodes[l], origin, inv, t_min, *t_far);
                let tr = slab(&self.nodes[r], origin, inv, t_min, *t_far);
                match (tl, tr) {
                    (Some(a), Some(b)) => {
                        let (near, far) = if a <= b { (l, r) } else { (r, l) };
                        if top < STACK {
                            stack[top] = far as u32;
                            top += 1;
                        }

                        node = near;
                        continue;
                    }
                    (Some(_), None) => {
                        node = l;
                        continue;
                    }
                    (None, Some(_)) => {
                        node = r;
                        continue;
                    }
                    (None, None) => {}
                }
            }

            if top == 0 {
                return;
            }

            top -= 1;
            node = stack[top] as usize;
        }
    }
}

fn safe_inv(x: f32) -> f32 {
    if x.abs() < 1e-20 { 1e20f32.copysign(x) } else { 1.0 / x }
}

fn slab(n: &Node, o: Vec3, inv: Vec3, t_min: f32, t_max: f32) -> Option<f32> {
    let t0 = (Vec3::from(n.min) - o) * inv;
    let t1 = (Vec3::from(n.max) - o) * inv;
    let near = t0.min(t1).max_element().max(t_min);
    let far = t0.max(t1).min_element().min(t_max);
    (near <= far).then_some(near)
}

fn intersect(tri: &Tri, o: Vec3, d: Vec3, t_min: f32, t_max: f32) -> Option<Hit> {
    let (e1, e2) = (Vec3::from(tri.e1), Vec3::from(tri.e2));
    let p = d.cross(e2);
    let det = e1.dot(p);
    if det.abs() < 1e-12 {
        return None;
    }

    let inv = 1.0 / det;
    let s = o - Vec3::from(tri.a);
    let u = s.dot(p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = s.cross(e1);
    let v = d.dot(q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = e2.dot(q) * inv;
    (t > t_min && t < t_max).then_some(Hit { t, tri: tri.id, u, v, front: det > 0.0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad(z: f32, size: f32) -> [[Vec3; 3]; 2] {
        let (a, b, c, d) = (Vec3::new(-size, -size, z), Vec3::new(size, -size, z), Vec3::new(size, size, z), Vec3::new(-size, size, z));
        [[a, b, c], [a, c, d]]
    }

    #[test]
    fn finds_the_nearest_of_stacked_quads() {
        let mut tris = Vec::new();
        for k in 0..50 {
            tris.extend(quad(k as f32 * 2.0, 10.0));
        }

        let bvh = Bvh::build(&tris);
        let hit = bvh.closest(Vec3::new(0.3, 0.2, 31.0), Vec3::NEG_Z, 0.0, 1000.0).unwrap();
        assert!((hit.t - 1.0).abs() < 1e-4, "{}", hit.t);
        assert!(hit.tri / 2 == 15);
        assert!(bvh.occluded(Vec3::new(0.3, 0.2, 31.0), Vec3::NEG_Z, 0.0, 1000.0));
        assert!(!bvh.occluded(Vec3::new(0.3, 0.2, 31.0), Vec3::NEG_Z, 0.0, 0.5), "t_max stops before the quad");
        assert!(bvh.closest(Vec3::new(20.0, 0.0, 31.0), Vec3::NEG_Z, 0.0, 1000.0).is_none(), "misses beside the quads");
    }

    #[test]
    fn tells_front_from_back() {
        // Counter clockwise seen from +z, so the normal points up.
        let bvh = Bvh::build(&quad(0.0, 1.0));
        assert!(bvh.closest(Vec3::new(0.1, 0.1, 5.0), Vec3::NEG_Z, 0.0, 10.0).unwrap().front);
        assert!(!bvh.closest(Vec3::new(0.1, 0.1, -5.0), Vec3::Z, 0.0, 10.0).unwrap().front);
    }

    #[test]
    fn matches_brute_force_on_random_triangles() {
        let mut seed = 7u32;
        let mut rnd = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as f32 / u32::MAX as f32 * 100.0 - 50.0
        };
        let tris: Vec<[Vec3; 3]> = (0..400)
            .map(|_| {
                let c = Vec3::new(rnd(), rnd(), rnd());
                [c, c + Vec3::new(rnd(), rnd(), rnd()) * 0.1, c + Vec3::new(rnd(), rnd(), rnd()) * 0.1]
            })
            .collect();
        let bvh = Bvh::build(&tris);
        let flat = Bvh { nodes: vec![Node { min: [-1e9; 3], start: 0, max: [1e9; 3], count: bvh.tris.len() as u32 }], tris: bvh.tris.clone() };
        for _ in 0..500 {
            let o = Vec3::new(rnd(), rnd(), rnd());
            let d = Vec3::new(rnd(), rnd(), rnd()).normalize();
            let a = bvh.closest(o, d, 0.0, 1e6).map(|h| h.tri);
            let b = flat.closest(o, d, 0.0, 1e6).map(|h| h.tri);
            assert_eq!(a, b);
        }
    }
}
