use gt_core::{DVec3, Plane};

const HULL_EPSILON: f64 = 1e-7;

struct Tri {
    v: [usize; 3],
    plane: Plane,
    alive: bool,
}

/// Incremental 3D convex hull. Returns the unique planes of the hull, or None if the points are degenerate.
pub fn convex_hull_planes(input: &[DVec3]) -> Option<Vec<Plane>> {
    let mut pts: Vec<DVec3> = Vec::with_capacity(input.len());
    for p in input {
        if !pts.iter().any(|q| (*q - *p).length_squared() < 1e-10) {
            pts.push(*p);
        }
    }
    if pts.len() < 4 {
        return None;
    }

    let i0 = 0;
    let i1 = (0..pts.len()).max_by(|a, b| (pts[*a] - pts[i0]).length_squared().total_cmp(&(pts[*b] - pts[i0]).length_squared()))?;
    let line = (pts[i1] - pts[i0]).normalize_or_zero();
    let dist_line = |p: DVec3| {
        let d = p - pts[i0];
        (d - line * d.dot(line)).length_squared()
    };
    let i2 = (0..pts.len()).max_by(|a, b| dist_line(pts[*a]).total_cmp(&dist_line(pts[*b])))?;
    if dist_line(pts[i2]) < 1e-10 {
        return None;
    }
    let base = Plane::from_points(pts[i0], pts[i1], pts[i2])?;
    let i3 = (0..pts.len()).max_by(|a, b| base.distance(pts[*a]).abs().total_cmp(&base.distance(pts[*b]).abs()))?;
    if base.distance(pts[i3]).abs() < 1e-6 {
        return None;
    }

    let centroid = (pts[i0] + pts[i1] + pts[i2] + pts[i3]) / 4.0;
    let mut tris: Vec<Tri> = Vec::new();
    let make = |a: usize, b: usize, c: usize, pts: &[DVec3]| -> Option<Tri> {
        let mut v = [a, b, c];
        let mut plane = Plane::from_points(pts[a], pts[b], pts[c])?;
        if plane.distance(centroid) > 0.0 {
            v.swap(1, 2);
            plane = plane.flipped();
        }
        Some(Tri { v, plane, alive: true })
    };
    for (a, b, c) in [(i0, i1, i2), (i0, i1, i3), (i0, i2, i3), (i1, i2, i3)] {
        tris.push(make(a, b, c, &pts)?);
    }

    for (pi, p) in pts.iter().enumerate() {
        if [i0, i1, i2, i3].contains(&pi) {
            continue;
        }
        let visible: Vec<usize> = (0..tris.len()).filter(|t| tris[*t].alive && tris[*t].plane.distance(*p) > HULL_EPSILON).collect();
        if visible.is_empty() {
            continue;
        }
        let mut edges: Vec<(usize, usize)> = Vec::new();
        for t in &visible {
            let v = tris[*t].v;
            for k in 0..3 {
                edges.push((v[k], v[(k + 1) % 3]));
            }
        }
        let horizon: Vec<(usize, usize)> = edges.iter().copied().filter(|(a, b)| !edges.contains(&(*b, *a))).collect();
        for t in visible {
            tris[t].alive = false;
        }
        for (a, b) in horizon {
            if let Some(plane) = Plane::from_points(pts[a], pts[b], *p) {
                tris.push(Tri { v: [a, b, pi], plane, alive: true });
            }
        }
    }

    let mut planes: Vec<Plane> = Vec::new();
    for t in tris.iter().filter(|t| t.alive) {
        if !planes.iter().any(|p| p.approx_eq(&t.plane, 1e-5, 1e-3)) {
            planes.push(t.plane);
        }
    }
    (planes.len() >= 4).then_some(planes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_hull_has_six_planes() {
        let mut pts = Vec::new();
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                for z in [-1.0, 1.0] {
                    pts.push(DVec3::new(x, y, z) * 16.0);
                }
            }
        }
        pts.push(DVec3::ZERO);
        pts.push(DVec3::new(16.0, 0.0, 0.0));
        assert_eq!(convex_hull_planes(&pts).unwrap().len(), 6);
    }

    #[test]
    fn coplanar_points_fail() {
        let pts = [DVec3::ZERO, DVec3::X, DVec3::Y, DVec3::ONE.with_z(0.0)];
        assert!(convex_hull_planes(&pts).is_none());
    }
}
