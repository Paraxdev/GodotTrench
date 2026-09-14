use gt_core::DVec3;

use crate::brush::{Brush, BrushError, FaceData};

/// `minuend - subtrahend`, as a set of convex pieces. New faces take the subtrahend's surface data.
pub fn subtract(minuend: &Brush, subtrahend: &Brush) -> Vec<Brush> {
    if !minuend.intersects(subtrahend) {
        return vec![minuend.clone()];
    }
    let mut out = Vec::new();
    let mut remaining = minuend.clone();
    // Cutting with the planes that remove the most volume first leaves fewer, larger pieces.
    let mut faces: Vec<&crate::brush::Face> = subtrahend.faces.iter().collect();
    faces.sort_by(|a, b| {
        let cut = |f: &crate::brush::Face| minuend.split(&f.plane, &f.data).0.map(|p| p.volume()).unwrap_or(0.0);
        cut(b).total_cmp(&cut(a))
    });
    for face in faces {
        let (front, back) = remaining.split(&face.plane, &face.data);
        if let Some(f) = front {
            out.push(f);
        }
        match back {
            Some(b) => remaining = b,
            None => break,
        }
    }
    merge_pieces(out)
}

/// Joins pieces whose union is still convex, and drops slivers.
pub fn merge_pieces(mut pieces: Vec<Brush>) -> Vec<Brush> {
    pieces.retain(|p| p.volume() > 1e-4);
    loop {
        let mut merged = None;
        'search: for i in 0..pieces.len() {
            for j in i + 1..pieces.len() {
                if !pieces[i].bounds().intersects(&pieces[j].bounds().expanded(1e-3)) {
                    continue;
                }
                let Ok(hull) = convex_merge(&[&pieces[i], &pieces[j]], "") else { continue };
                let sum = pieces[i].volume() + pieces[j].volume();
                if (hull.volume() - sum).abs() <= 1e-4 * sum.max(1.0) {
                    merged = Some((i, j, hull));
                    break 'search;
                }
            }
        }
        match merged {
            Some((i, j, hull)) => {
                pieces.remove(j);
                pieces[i] = hull;
            }
            None => return pieces,
        }
    }
}

pub fn intersect(a: &Brush, b: &Brush) -> Result<Brush, BrushError> {
    let mut planes = a.planes();
    planes.extend(b.planes());
    Brush::from_planes(planes)
}

/// Convex hull of all input brushes (TrenchBroom's "convex merge").
pub fn convex_merge(brushes: &[&Brush], default_material: &str) -> Result<Brush, BrushError> {
    let points: Vec<DVec3> = brushes.iter().flat_map(|b| b.vertices.iter().copied()).collect();
    let template: Vec<_> = brushes.iter().flat_map(|b| b.planes()).collect();
    Brush::from_points(&points, &template, default_material)
}

/// Replaces the brush by walls of the given thickness around its former volume.
pub fn hollow(brush: &Brush, thickness: f64) -> Vec<Brush> {
    let inner_planes: Vec<_> = brush.planes().into_iter().map(|(p, d)| (p.offset(-thickness), d)).collect();
    match Brush::from_planes(inner_planes) {
        Ok(inner) if inner.volume() > 1e-6 => subtract(brush, &inner),
        _ => vec![brush.clone()],
    }
}

/// Extrudes a face outward by `distance`, creating a new brush attached to it.
pub fn extrude_face(brush: &Brush, face: usize, distance: f64) -> Result<Brush, BrushError> {
    let f = &brush.faces[face];
    let normal = f.plane.normal;
    let points = brush.face_points(face);
    let mut all = points.clone();
    all.extend(points.iter().map(|p| *p + normal * distance));
    let mut template = brush.planes();
    template.push((f.plane.flipped(), FaceData { ..f.data.clone() }));
    Brush::from_points(&all, &template, &f.data.material)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gt_core::Aabb;

    fn boxb(min: DVec3, max: DVec3) -> Brush {
        Brush::from_aabb(&Aabb::new(min, max), "m").unwrap()
    }

    #[test]
    fn subtract_center_hole() {
        let a = boxb(DVec3::splat(-32.0), DVec3::splat(32.0));
        let b = boxb(DVec3::new(-8.0, -64.0, -8.0), DVec3::new(8.0, 64.0, 8.0));
        let pieces = subtract(&a, &b);
        assert_eq!(pieces.len(), 4);
        let vol: f64 = pieces.iter().map(|p| p.volume()).sum();
        assert!((vol - (a.volume() - 16.0 * 16.0 * 64.0)).abs() < 1e-3);
        for p in &pieces {
            p.validate().unwrap();
        }
    }

    #[test]
    fn subtract_disjoint_is_identity() {
        let a = boxb(DVec3::ZERO, DVec3::splat(8.0));
        let b = boxb(DVec3::splat(16.0), DVec3::splat(32.0));
        assert_eq!(subtract(&a, &b).len(), 1);
    }

    #[test]
    fn hollow_volume() {
        let a = boxb(DVec3::splat(-32.0), DVec3::splat(32.0));
        let walls = hollow(&a, 4.0);
        let vol: f64 = walls.iter().map(|p| p.volume()).sum();
        assert!((vol - (64.0f64.powi(3) - 56.0f64.powi(3))).abs() < 1e-3);
    }

    #[test]
    fn merge_two_boxes() {
        let a = boxb(DVec3::ZERO, DVec3::splat(16.0));
        let b = boxb(DVec3::new(16.0, 0.0, 0.0), DVec3::new(32.0, 16.0, 16.0));
        let m = convex_merge(&[&a, &b], "m").unwrap();
        assert_eq!(m.faces.len(), 6);
        assert!((m.volume() - 32.0 * 16.0 * 16.0).abs() < 1e-6);
    }

    #[test]
    fn extrude_adds_volume() {
        let a = boxb(DVec3::ZERO, DVec3::splat(16.0));
        let top = a.faces.iter().position(|f| f.plane.normal.y > 0.5).unwrap();
        let e = extrude_face(&a, top, 8.0).unwrap();
        assert!((e.volume() - 16.0 * 16.0 * 8.0).abs() < 1e-6);
        assert!(e.bounds().min.y > 15.9);
    }
}
