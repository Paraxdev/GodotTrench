use td_core::{DVec2, DVec3, EPSILON, Plane};

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

/// Ear clipping work a face may take per corner and in all, counted in corners and grid rows looked at. Past either
/// the rest of the face is fanned, so a face made to block its own ears cannot stall a build. Ordinary faces of a few
/// thousand corners stay well within it. The Godot addon's `TrenchDotMesh` counts the same work.
const CLIP_WORK_PER_CORNER: usize = 256;
const MAX_CLIP_WORK: usize = 1 << 22;

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

    // The polygon left to clip as a ring of corners starting at `head`, so clipping an ear takes constant time.
    let mut next: Vec<usize> = (1..=n).map(|i| i % n).collect();
    let mut prev: Vec<usize> = (0..n).map(|i| (i + n - 1) % n).collect();
    let (mut head, mut m) = (0, n);
    let mut out = Vec::with_capacity(n - 2);
    let inside = |p: DVec2, a: DVec2, b: DVec2, c: DVec2| cross(a, b, p) >= -1e-9 && cross(b, c, p) >= -1e-9 && cross(c, a, p) >= -1e-9;
    let is_reflex = |prev: &[usize], next: &[usize], j: usize| cross(p2[prev[j]], p2[j], p2[next[j]]) <= 1e-12;
    // An ear of a simple polygon holds another corner only if it holds a reflex one, so only those are tested. Corners
    // that turn reflex after the grid is made, which takes a face that crosses itself, are kept in `late`.
    let mut reflex: Vec<bool> = (0..n).map(|j| is_reflex(&prev, &next, j)).collect();
    let grid = ReflexGrid::new(&p2, &reflex);
    let mut listed = reflex.clone();
    let mut late = Vec::new();
    let budget = (n * CLIP_WORK_PER_CORNER).min(MAX_CLIP_WORK);
    let mut work = 0;
    while m > 3 {
        let mut clipped = false;
        let mut ib = head;
        for _ in 0..m {
            work += 1;
            if work > budget {
                break;
            }

            let (ia, ic) = (prev[ib], next[ib]);
            let (a, b, c) = (p2[ia], p2[ib], p2[ic]);
            let area = cross(a, b, c);
            let blocks = |j: usize| reflex[j] && j != ia && j != ib && j != ic && p2[j] != a && p2[j] != b && p2[j] != c && inside(p2[j], a, b, c);
            if area <= 1e-12 || grid.any([a, b, c], area, &mut work, blocks) || ReflexGrid::scan(&late, &mut work, blocks) {
                ib = ic;
                continue;
            }

            out.push([ia, ib, ic]);
            (next[ia], prev[ic]) = (ic, ia);
            if ib == head {
                head = ic;
            }

            for j in [ia, ic] {
                reflex[j] = is_reflex(&prev, &next, j);
                if reflex[j] && !listed[j] {
                    listed[j] = true;
                    late.push(j);
                }
            }

            clipped = true;
            break;
        }

        if !clipped {
            break;
        }

        m -= 1;
    }

    let (first, mut at) = (head, next[head]);
    for _ in 2..m {
        out.push([first, at, next[at]]);
        at = next[at];
    }

    out
}

/// The corners of a face that start out reflex, sorted into a square grid over their bounds, row after row.
struct ReflexGrid {
    min: DVec2,
    scale: DVec2,
    side: usize,
    /// Where the corners of each cell start in `items`, with one more entry for the end of the last cell.
    start: Vec<usize>,
    items: Vec<usize>,
}

impl ReflexGrid {
    fn new(p2: &[DVec2], reflex: &[bool]) -> Self {
        let corners: Vec<usize> = (0..p2.len()).filter(|&j| reflex[j]).collect();
        let side = ((corners.len() as f64).sqrt() as usize).max(1);
        let (mut min, mut max) = (DVec2::INFINITY, DVec2::NEG_INFINITY);
        for &j in &corners {
            (min, max) = (min.min(p2[j]), max.max(p2[j]));
        }

        let per = |extent: f64| if extent > 0.0 { side as f64 / extent } else { 0.0 };
        let scale = DVec2::new(per(max.x - min.x), per(max.y - min.y));
        let mut grid = Self { min, scale, side, start: vec![0; side * side + 1], items: vec![0; corners.len()] };
        let cells: Vec<usize> = corners.iter().map(|&j| grid.row(p2[j].y) * side + grid.column(p2[j].x)).collect();
        cells.iter().for_each(|&cell| grid.start[cell + 1] += 1);
        for cell in 0..side * side {
            grid.start[cell + 1] += grid.start[cell];
        }

        let mut fill = grid.start.clone();
        for (&j, &cell) in corners.iter().zip(&cells) {
            grid.items[fill[cell]] = j;
            fill[cell] += 1;
        }

        grid
    }

    fn cell(&self, f: f64) -> usize {
        if f >= self.side as f64 {
            self.side - 1
        } else if f >= 0.0 {
            f as usize
        } else {
            0
        }
    }

    fn column(&self, x: f64) -> usize {
        self.cell((x - self.min.x) * self.scale.x)
    }

    fn row(&self, y: f64) -> usize {
        self.cell((y - self.min.y) * self.scale.y)
    }

    fn scan(items: &[usize], work: &mut usize, blocks: impl Fn(usize) -> bool) -> bool {
        *work += 1;
        items.iter().any(|&j| {
            *work += 1;
            blocks(j)
        })
    }

    /// Whether `blocks` holds for a corner of the grid that may count as inside the ear `a b c`, adding the corners and
    /// rows looked at to `work`.
    fn any(&self, [a, b, c]: [DVec2; 3], area: f64, work: &mut usize, blocks: impl Fn(usize) -> bool) -> bool {
        let lengths = [(b - a).length(), (c - b).length(), (a - c).length()];
        let (longest, shortest) = (lengths[0].max(lengths[1]).max(lengths[2]), lengths[0].min(lengths[1]).min(lengths[2]));
        // A corner counts as inside up to 1e-9 / edge length past an edge, and the addon's single precision floats
        // round by about 1e-7 of the edge. Toward the sharpest corner of the ear that grows by up to longest² / area.
        let margin = 4.0 * (1e-9 / shortest + 1e-6 * longest) * longest * longest / area;
        let (lo, hi) = (a.min(b).min(c) - margin, a.max(b).max(c) + margin);
        let (x0, x1, y0, y1) = (self.column(lo.x), self.column(hi.x), self.row(lo.y), self.row(hi.y));
        let narrow = margin <= longest && (x1 - x0 + 1) * (y1 - y0 + 1) <= self.items.len();
        if !narrow {
            return Self::scan(&self.items, work, blocks);
        }

        (y0..=y1).any(|y| Self::scan(&self.items[self.start[y * self.side + x0]..self.start[y * self.side + x1 + 1]], work, &blocks))
    }
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

    fn circle(n: usize, radius: f64) -> Vec<DVec3> {
        (0..n).map(|i| i as f64 / n as f64 * std::f64::consts::TAU).map(|a| DVec3::new(a.cos() * radius, 0.0, -a.sin() * radius)).collect()
    }

    fn gear(n: usize) -> Vec<DVec3> {
        let radius = |i: usize| if i.is_multiple_of(2) { 512.0 } else { 480.0 };
        circle(n, 1.0).into_iter().enumerate().map(|(i, p)| p * radius(i)).collect()
    }

    /// A random simple polygon, every corner on its own ray from the middle.
    fn star(n: usize, seed: u64) -> Vec<DVec3> {
        let mut state = seed;
        let mut radius = || {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            200.0 + (state >> 40) as f64 / (1u64 << 24) as f64 * 312.0
        };
        circle(n, 1.0).into_iter().map(|p| p * radius()).collect()
    }

    /// A ring cut open along one edge, so two pairs of corners sit at the same place.
    fn keyhole(n: usize) -> Vec<DVec3> {
        let (outer, inner) = (circle(n, 512.0), circle(n, 256.0));
        let back = (0..=n).map(|i| inner[(n - i) % n]);
        outer.iter().copied().chain([outer[0]]).chain(back).collect()
    }

    /// An outer arc and an inner arc so close that every outer corner's ear holds an inner corner.
    fn thin_band(n: usize) -> Vec<DVec3> {
        let (r, step) = (4096.0, 1.0 / n as f64);
        let w = r * (1.0 - step.cos()) * 0.25;
        let arc = |radius: f64| move |i: usize| DVec3::new((i as f64 * step).cos() * radius, 0.0, -(i as f64 * step).sin() * radius);
        (0..n / 2).map(arc(r)).chain((0..n / 2).rev().map(arc(r - w))).collect()
    }

    /// Ear clipping that tests every corner against every ear, as `triangulate` did before it kept to reflex corners.
    fn clip_testing_every_corner(points: &[DVec3]) -> Vec<[usize; 3]> {
        let normal = newell(points).normalize();
        let helper = if normal.y.abs() < 0.9 { DVec3::Y } else { DVec3::X };
        let u = helper.cross(normal).normalize();
        let v = normal.cross(u);
        let p2: Vec<DVec2> = points.iter().map(|p| DVec2::new(p.dot(u), p.dot(v))).collect();
        let cross = |a: DVec2, b: DVec2, c: DVec2| (b - a).perp_dot(c - a);
        let inside = |p: DVec2, a: DVec2, b: DVec2, c: DVec2| cross(a, b, p) >= -1e-9 && cross(b, c, p) >= -1e-9 && cross(c, a, p) >= -1e-9;
        let mut idx: Vec<usize> = (0..points.len()).collect();
        let mut out = Vec::new();
        while idx.len() > 3 {
            let m = idx.len();
            let ear = (0..m).find(|&k| {
                let (ia, ib, ic) = (idx[(k + m - 1) % m], idx[k], idx[(k + 1) % m]);
                let (a, b, c) = (p2[ia], p2[ib], p2[ic]);
                cross(a, b, c) > 1e-12
                    && !idx.iter().any(|&j| j != ia && j != ib && j != ic && p2[j] != a && p2[j] != b && p2[j] != c && inside(p2[j], a, b, c))
            });
            let Some(k) = ear else { break };
            out.push([idx[(k + m - 1) % m], idx[k], idx[(k + 1) % m]]);
            idx.remove(k);
        }

        out.extend((1..idx.len() - 1).map(|k| [idx[0], idx[k], idx[k + 1]]));
        out
    }

    #[test]
    fn testing_only_reflex_corners_finds_the_same_ears() {
        let mut faces: Vec<Vec<DVec3>> = vec![gear(1024), keyhole(64)];
        faces.extend([5, 6, 7, 8, 12, 24, 100, 400].map(|n| circle(n, 64.0)));
        faces.extend([6, 8, 10, 16, 50, 128, 512].map(gear));
        faces.extend((0..60).map(|seed| star(5 + seed as usize * 7, seed)));
        let rect = [[0.0, 0.0], [640.0, 0.0], [640.0, 640.0], [0.0, 640.0]].map(|[x, z]| DVec3::new(x, 0.0, z));
        faces.push((0..4).flat_map(|side| (0..10).map(move |i| rect[side].lerp(rect[(side + 1) % 4], i as f64 / 10.0))).collect());
        for (i, points) in faces.iter().enumerate() {
            assert_eq!(triangulate(points, newell(points)), clip_testing_every_corner(points), "face {i} with {} corners", points.len());
        }
    }

    #[test]
    fn big_ordinary_faces_are_clipped_to_the_end() {
        for points in [gear(4096), star(4000, 7), keyhole(1024)] {
            let normal = newell(&points);
            let tris = triangulate(&points, normal);
            assert_eq!(tris.len(), points.len() - 2);
            let doubled: Vec<f64> = tris.iter().map(|t| (points[t[1]] - points[t[0]]).cross(points[t[2]] - points[t[0]]).dot(normal.normalize())).collect();
            assert!(doubled.iter().all(|a| *a > -1e-6), "{} corners give a flipped triangle", points.len());
            let total = doubled.iter().map(|a| a.abs()).sum::<f64>() * 0.5;
            assert!((total - area(&points)).abs() < area(&points) * 1e-9, "{} corners cover {total} of {}", points.len(), area(&points));
        }

        // godot/tests/td_editor_parity_test.gd expects the addon to clip the same triangles.
        let tris = triangulate(&gear(1024), newell(&gear(1024)));
        assert_eq!([tris[0], tris[1], tris[600], tris[1020], tris[1021]], [[1023, 0, 1], [1, 2, 3], [1023, 599, 601], [1023, 1019, 1021], [1021, 1022, 1023]]);
    }

    #[test]
    fn faces_made_to_block_their_ears_finish_quickly() {
        let octagon = circle(8, 64.0);
        let repeated: Vec<DVec3> = octagon.iter().copied().cycle().take(2_000_000).collect();
        for points in [thin_band(2048), thin_band(8192), repeated] {
            let started = std::time::Instant::now();
            assert_eq!(triangulate(&points, newell(&points)).len(), points.len() - 2);
            assert!(started.elapsed() < std::time::Duration::from_secs(1), "{} corners took {:?}", points.len(), started.elapsed());
        }
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
