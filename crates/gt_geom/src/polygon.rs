use gt_core::{DVec2, DVec3, EPSILON, Plane};

/// Keeps the part of a convex polygon behind (or on) the plane.
pub fn clip_back(poly: &[DVec3], plane: &Plane) -> Vec<DVec3> {
    if poly.is_empty() {
        return Vec::new();
    }
    let dists: Vec<f64> = poly.iter().map(|p| plane.distance(*p)).collect();
    if dists.iter().all(|d| *d <= EPSILON) {
        return poly.to_vec();
    }
    if dists.iter().all(|d| *d >= -EPSILON) {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(poly.len() + 1);
    for i in 0..poly.len() {
        let j = (i + 1) % poly.len();
        let (a, b) = (poly[i], poly[j]);
        let (da, db) = (dists[i], dists[j]);
        if da <= EPSILON {
            out.push(a);
        }
        if (da > EPSILON && db < -EPSILON) || (da < -EPSILON && db > EPSILON) {
            let t = da / (da - db);
            out.push(a + (b - a) * t);
        }
    }
    out
}

/// Huge quad lying on the plane, counter-clockwise around the plane normal.
pub fn base_winding(plane: &Plane, size: f64) -> Vec<DVec3> {
    let (u, v) = plane.basis();
    let c = plane.point();
    let (u, v) = (u * size, v * size);
    vec![c - u - v, c + u - v, c + u + v, c - u + v]
}

pub fn centroid(points: &[DVec3]) -> DVec3 {
    if points.is_empty() {
        return DVec3::ZERO;
    }
    points.iter().copied().sum::<DVec3>() / points.len() as f64
}

pub fn area(points: &[DVec3]) -> f64 {
    let mut n = DVec3::ZERO;
    for i in 1..points.len().saturating_sub(1) {
        n += (points[i] - points[0]).cross(points[i + 1] - points[0]);
    }
    n.length() * 0.5
}

/// Newell normal of a possibly non-planar polygon, scaled by twice its area.
pub fn newell(points: &[DVec3]) -> DVec3 {
    let mut n = DVec3::ZERO;
    for i in 0..points.len() {
        let cur = points[i];
        let next = points[(i + 1) % points.len()];
        n.x += (cur.y - next.y) * (cur.z + next.z);
        n.y += (cur.z - next.z) * (cur.x + next.x);
        n.z += (cur.x - next.x) * (cur.y + next.y);
    }
    n
}

/// Ear clipping triangulation of a simple polygon wound counter-clockwise around `normal`.
/// Returns corner index triples with the polygon's winding. Falls back to a fan for polygons it cannot clip.
pub fn triangulate(points: &[DVec3], normal: DVec3) -> Vec<[usize; 3]> {
    let n = points.len();
    if n < 3 {
        return Vec::new();
    }
    if n == 3 {
        return vec![[0, 1, 2]];
    }
    let normal = normal.normalize_or(DVec3::Y);
    let helper = if normal.y.abs() < 0.9 { DVec3::Y } else { DVec3::X };
    let u = helper.cross(normal).normalize();
    let v = normal.cross(u);
    let p2: Vec<DVec2> = points.iter().map(|p| DVec2::new(p.dot(u), p.dot(v))).collect();
    let cross = |a: DVec2, b: DVec2, c: DVec2| (b - a).perp_dot(c - a);
    let convex_quad = n == 4 && (0..4).all(|i| cross(p2[i], p2[(i + 1) % 4], p2[(i + 2) % 4]) > 0.0);
    if convex_quad {
        // Split along the shorter diagonal, it keeps slivers out of near-planar quads.
        return if (p2[0] - p2[2]).length_squared() <= (p2[1] - p2[3]).length_squared() { vec![[0, 1, 2], [0, 2, 3]] } else { vec![[0, 1, 3], [1, 2, 3]] };
    }
    let mut idx: Vec<usize> = (0..n).collect();
    let mut out = Vec::with_capacity(n - 2);
    let inside = |p: DVec2, a: DVec2, b: DVec2, c: DVec2| cross(a, b, p) >= -1e-9 && cross(b, c, p) >= -1e-9 && cross(c, a, p) >= -1e-9;
    let mut guard = 0;
    while idx.len() > 3 && guard < n * n {
        guard += 1;
        let m = idx.len();
        let mut clipped = false;
        for k in 0..m {
            let (ia, ib, ic) = (idx[(k + m - 1) % m], idx[k], idx[(k + 1) % m]);
            let (a, b, c) = (p2[ia], p2[ib], p2[ic]);
            if cross(a, b, c) <= 1e-12 {
                continue;
            }
            let blocked = idx.iter().any(|&j| j != ia && j != ib && j != ic && p2[j] != a && p2[j] != b && p2[j] != c && inside(p2[j], a, b, c));
            if blocked {
                continue;
            }
            out.push([ia, ib, ic]);
            idx.remove(k);
            clipped = true;
            break;
        }
        if !clipped {
            break;
        }
    }
    if idx.len() == 3 {
        out.push([idx[0], idx[1], idx[2]]);
    } else if idx.len() > 3 {
        for k in 1..idx.len() - 1 {
            out.push([idx[0], idx[k], idx[k + 1]]);
        }
    }
    out
}
