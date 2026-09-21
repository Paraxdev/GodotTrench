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

/// Parts of the same plane within this distance of each other count as touching.
const OVERLAP_EPS: f64 = 1e-3;
/// Pieces smaller than this are slivers from touching edges, not real overlap.
const MIN_PIECE_AREA: f64 = 1e-2;

/// Splits a convex polygon into the parts where `normal·p - offset` is positive and negative. Points within `eps` go to both.
pub fn split(poly: &[DVec3], normal: DVec3, offset: f64, eps: f64) -> (Vec<DVec3>, Vec<DVec3>) {
    let (mut front, mut back) = (Vec::with_capacity(poly.len() + 1), Vec::with_capacity(poly.len() + 1));
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        let (da, db) = (normal.dot(a) - offset, normal.dot(b) - offset);
        if da >= -eps {
            front.push(a);
        }

        if da <= eps {
            back.push(a);
        }

        if (da > eps && db < -eps) || (da < -eps && db > eps) {
            let x = a + (b - a) * (da / (da - db));
            front.push(x);
            back.push(x);
        }
    }

    (front, back)
}

pub fn is_convex(poly: &[DVec3], normal: DVec3) -> bool {
    let n = poly.len();
    n >= 3 && (0..n).all(|i| (poly[(i + 1) % n] - poly[i]).cross(poly[(i + 2) % n] - poly[(i + 1) % n]).dot(normal) >= -1e-9)
}

/// Convex polygon `a` minus convex polygon `b`, both lying on one plane with `normal`.
/// Returns None when they only touch, otherwise the convex pieces of `a` outside `b` with `a`'s winding.
pub fn subtract_convex(a: &[DVec3], b: &[DVec3], normal: DVec3) -> Option<Vec<Vec<DVec3>>> {
    let center = centroid(b);
    let mut rest = a.to_vec();
    let mut out = Vec::new();
    for i in 0..b.len() {
        let (p, q) = (b[i], b[(i + 1) % b.len()]);
        let mut inward = normal.cross(q - p).normalize_or_zero();
        if inward == DVec3::ZERO {
            continue;
        }

        if inward.dot(center - p) < 0.0 {
            inward = -inward;
        }

        let (inside, outside) = split(&rest, inward, inward.dot(p), OVERLAP_EPS);
        if inside.len() < 3 || area(&inside) < MIN_PIECE_AREA {
            return None;
        }

        if outside.len() >= 3 && area(&outside) >= MIN_PIECE_AREA {
            out.push(outside);
        }

        rest = inside;
    }

    Some(out)
}

/// What stays visible of a face once the parts covered by `occluders` on the same plane are removed.
/// None when nothing overlaps, so the face can be drawn as it is. Every returned piece is convex.
pub fn visible_pieces(face: &[DVec3], normal: DVec3, occluders: &[&[DVec3]]) -> Option<Vec<Vec<DVec3>>> {
    let mut pieces = convex_parts(face, normal);
    let mut changed = false;
    for occluder in occluders {
        let facing = if newell(occluder).dot(normal) < 0.0 { -normal } else { normal };
        for part in convex_parts(occluder, facing) {
            let mut next = Vec::with_capacity(pieces.len());
            for piece in pieces {
                match subtract_convex(&piece, &part, normal) {
                    Some(rest) => {
                        changed = true;
                        next.extend(rest);
                    }
                    None => next.push(piece),
                }
            }

            pieces = next;
            if pieces.is_empty() {
                return Some(pieces);
            }
        }
    }

    changed.then_some(pieces)
}

fn convex_parts(poly: &[DVec3], normal: DVec3) -> Vec<Vec<DVec3>> {
    if is_convex(poly, normal) { vec![poly.to_vec()] } else { triangulate(poly, normal).iter().map(|t| t.iter().map(|i| poly[*i]).collect()).collect() }
}

/// Blend weights of a polygon's corners for a point on it, taken from whichever of `triangles` holds the point best.
pub fn corner_weights(corners: &[DVec3], triangles: &[[usize; 3]], p: DVec3) -> [(usize, f64); 3] {
    let mut best = (f64::NEG_INFINITY, [(0, 1.0), (0, 0.0), (0, 0.0)]);
    for &[ia, ib, ic] in triangles {
        let (a, b, c) = (corners[ia], corners[ib], corners[ic]);
        let normal = (b - a).cross(c - a);
        let len2 = normal.length_squared();
        if len2 < 1e-18 {
            continue;
        }

        let wa = (b - p).cross(c - p).dot(normal) / len2;
        let wb = (c - p).cross(a - p).dot(normal) / len2;
        let wc = 1.0 - wa - wb;
        let worst = wa.min(wb).min(wc);
        if worst > best.0 {
            best = (worst, [(ia, wa), (ib, wb), (ic, wc)]);
        }
    }

    best.1
}

pub fn fan(n: usize) -> Vec<[usize; 3]> {
    (1..n.saturating_sub(1)).map(|k| [0, k, k + 1]).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(x0: f64, z0: f64, x1: f64, z1: f64) -> Vec<DVec3> {
        vec![DVec3::new(x0, 0.0, z0), DVec3::new(x0, 0.0, z1), DVec3::new(x1, 0.0, z1), DVec3::new(x1, 0.0, z0)]
    }

    #[test]
    fn subtracting_keeps_the_uncovered_area() {
        let a = square(0.0, 0.0, 64.0, 64.0);
        let b = square(32.0, -16.0, 96.0, 80.0);
        let pieces = subtract_convex(&a, &b, DVec3::Y).unwrap();
        let total: f64 = pieces.iter().map(|p| area(p)).sum();
        assert!((total - 32.0 * 64.0).abs() < 1e-6, "got {total}");
        assert!(pieces.iter().all(|p| p.iter().all(|v| v.x <= 32.0 + 1e-9)));
        assert!(pieces.iter().all(|p| newell(p).dot(DVec3::Y) > 0.0), "winding is kept");
    }

    #[test]
    fn faces_that_only_share_an_edge_are_left_alone() {
        let a = square(0.0, 0.0, 64.0, 64.0);
        assert!(subtract_convex(&a, &square(64.0, 0.0, 128.0, 64.0), DVec3::Y).is_none());
        assert!(visible_pieces(&a, DVec3::Y, &[&square(64.0, 0.0, 128.0, 64.0)]).is_none());
        let covered = visible_pieces(&a, DVec3::Y, &[&square(-8.0, -8.0, 72.0, 72.0)]).unwrap();
        assert!(covered.is_empty());
    }

    #[test]
    fn a_hole_in_the_middle_leaves_a_ring() {
        let a = square(0.0, 0.0, 64.0, 64.0);
        let pieces = visible_pieces(&a, DVec3::Y, &[&square(16.0, 16.0, 48.0, 48.0)]).unwrap();
        let total: f64 = pieces.iter().map(|p| area(p)).sum();
        assert!((total - (64.0 * 64.0 - 32.0 * 32.0)).abs() < 1e-6);
        assert!(pieces.iter().all(|p| is_convex(p, newell(p))));
    }

    #[test]
    fn corner_weights_reproduce_the_point() {
        let corners = square(0.0, 0.0, 64.0, 32.0);
        let p = DVec3::new(20.0, 0.0, 10.0);
        let w = corner_weights(&corners, &fan(4), p);
        let back: DVec3 = w.iter().map(|(i, w)| corners[*i] * *w).sum();
        assert!(back.distance(p) < 1e-9);
    }
}
