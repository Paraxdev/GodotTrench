//! Primitive generators. All shapes fit inside `bounds` and use Y as the height axis.

use std::f64::consts::TAU;

use gt_core::{Aabb, DVec2, DVec3};

use crate::brush::{Brush, FaceData};

fn hull(points: &[DVec3], material: &str) -> Option<Brush> {
    Brush::from_points(points, &[], material).ok()
}

fn ring(center: DVec2, radius: DVec2, sides: usize, phase: f64) -> Vec<DVec2> {
    (0..sides)
        .map(|i| {
            let a = phase + TAU * i as f64 / sides as f64;
            center + DVec2::new(a.cos() * radius.x, a.sin() * radius.y)
        })
        .collect()
}

fn xz(bounds: &Aabb) -> (DVec2, DVec2) {
    let c = bounds.center();
    let s = bounds.size() * 0.5;
    (DVec2::new(c.x, c.z), DVec2::new(s.x, s.z))
}

pub fn cylinder(bounds: &Aabb, sides: usize, material: &str) -> Option<Brush> {
    let (c, r) = xz(bounds);
    let pts: Vec<DVec3> =
        ring(c, r, sides.max(3), 0.0).into_iter().flat_map(|p| [DVec3::new(p.x, bounds.min.y, p.y), DVec3::new(p.x, bounds.max.y, p.y)]).collect();
    hull(&pts, material)
}

pub fn cone(bounds: &Aabb, sides: usize, material: &str) -> Option<Brush> {
    let (c, r) = xz(bounds);
    let mut pts: Vec<DVec3> = ring(c, r, sides.max(3), 0.0).into_iter().map(|p| DVec3::new(p.x, bounds.min.y, p.y)).collect();
    pts.push(DVec3::new(c.x, bounds.max.y, c.y));
    hull(&pts, material)
}

pub fn spike(bounds: &Aabb, material: &str) -> Option<Brush> {
    cone(bounds, 4, material)
}

/// Wedge sloping down towards +Z.
pub fn wedge(bounds: &Aabb, material: &str) -> Option<Brush> {
    let (a, b) = (bounds.min, bounds.max);
    let pts = [
        DVec3::new(a.x, a.y, a.z),
        DVec3::new(b.x, a.y, a.z),
        DVec3::new(a.x, a.y, b.z),
        DVec3::new(b.x, a.y, b.z),
        DVec3::new(a.x, b.y, a.z),
        DVec3::new(b.x, b.y, a.z),
    ];
    hull(&pts, material)
}

/// UV sphere made of one brush per latitude band.
pub fn sphere(bounds: &Aabb, sides: usize, rings: usize, material: &str) -> Vec<Brush> {
    let c = bounds.center();
    let r = bounds.size() * 0.5;
    let rings = rings.max(2);
    let sides = sides.max(3);
    let band = |lat: f64| -> Vec<DVec3> {
        let y = c.y - r.y * lat.cos();
        let rad = lat.sin();
        ring(DVec2::new(c.x, c.z), DVec2::new(r.x * rad, r.z * rad), sides, 0.0).into_iter().map(|p| DVec3::new(p.x, y, p.y)).collect()
    };
    (0..rings)
        .filter_map(|i| {
            let lat0 = std::f64::consts::PI * i as f64 / rings as f64;
            let lat1 = std::f64::consts::PI * (i + 1) as f64 / rings as f64;
            let mut pts = band(lat0);
            pts.extend(band(lat1));
            hull(&pts, material)
        })
        .collect()
}

/// Ring of segments around Y with a square cross section. `thickness` is the radial wall width.
pub fn pipe(bounds: &Aabb, sides: usize, thickness: f64, material: &str) -> Vec<Brush> {
    arc_segments(bounds, sides, thickness, 0.0, 360.0, material)
}

/// Arch in the XY plane: half ring spanning the bounds width, depth along Z.
pub fn arch(bounds: &Aabb, segments: usize, thickness: f64, material: &str) -> Vec<Brush> {
    let segments = segments.max(1);
    let size = bounds.size();
    let rx = size.x * 0.5;
    let ry = size.y;
    let center = DVec2::new(bounds.center().x, bounds.min.y);
    (0..segments)
        .filter_map(|i| {
            let a0 = std::f64::consts::PI * i as f64 / segments as f64;
            let a1 = std::f64::consts::PI * (i + 1) as f64 / segments as f64;
            let outer = |a: f64| center + DVec2::new(a.cos() * rx, a.sin() * ry);
            let inner = |a: f64| center + DVec2::new(a.cos() * (rx - thickness).max(0.0), a.sin() * (ry - thickness).max(0.0));
            let quad = [outer(a0), outer(a1), inner(a0), inner(a1)];
            let pts: Vec<DVec3> = quad.iter().flat_map(|p| [DVec3::new(p.x, p.y, bounds.min.z), DVec3::new(p.x, p.y, bounds.max.z)]).collect();
            hull(&pts, material)
        })
        .collect()
}

pub fn torus(bounds: &Aabb, sides: usize, thickness: f64, material: &str) -> Vec<Brush> {
    pipe(bounds, sides, thickness, material)
}

fn arc_segments(bounds: &Aabb, sides: usize, thickness: f64, start_deg: f64, end_deg: f64, material: &str) -> Vec<Brush> {
    let (c, r) = xz(bounds);
    let sides = sides.max(3);
    let span = (end_deg - start_deg).to_radians();
    let inner_r = (r - DVec2::splat(thickness)).max(DVec2::ZERO);
    (0..sides)
        .filter_map(|i| {
            let a0 = start_deg.to_radians() + span * i as f64 / sides as f64;
            let a1 = start_deg.to_radians() + span * (i + 1) as f64 / sides as f64;
            let at = |a: f64, rad: DVec2| c + DVec2::new(a.cos() * rad.x, a.sin() * rad.y);
            let quad = [at(a0, r), at(a1, r), at(a0, inner_r), at(a1, inner_r)];
            let pts: Vec<DVec3> = quad.iter().flat_map(|p| [DVec3::new(p.x, bounds.min.y, p.y), DVec3::new(p.x, bounds.max.y, p.y)]).collect();
            hull(&pts, material)
        })
        .collect()
}

/// Staircase rising towards -Z (Godot forward).
pub fn stairs(bounds: &Aabb, steps: usize, material: &str) -> Vec<Brush> {
    let steps = steps.max(1);
    let size = bounds.size();
    let step_h = size.y / steps as f64;
    let step_d = size.z / steps as f64;
    (0..steps)
        .filter_map(|i| {
            let min = DVec3::new(bounds.min.x, bounds.min.y, bounds.max.z - step_d * (i + 1) as f64);
            let max = DVec3::new(bounds.max.x, bounds.min.y + step_h * (i + 1) as f64, bounds.max.z - step_d * i as f64);
            Brush::from_aabb(&Aabb::new(min, max), material).ok()
        })
        .collect()
}

/// Spiral staircase of wedge shaped steps around the bounds' vertical axis, `turns` full turns high.
pub fn spiral_stairs(bounds: &Aabb, steps: usize, inner_radius: f64, turns: f64, material: &str) -> Vec<Brush> {
    let steps = steps.max(2);
    let c = bounds.center();
    let outer = bounds.size().x.min(bounds.size().z) * 0.5;
    let step_h = bounds.size().y / steps as f64;
    let arc = TAU * turns / steps as f64;
    (0..steps)
        .filter_map(|i| {
            let a0 = arc * i as f64;
            let a1 = a0 + arc * 1.05;
            let y0 = bounds.min.y + step_h * i as f64;
            let y1 = y0 + step_h;
            let at = |a: f64, r: f64, y: f64| DVec3::new(c.x + a.cos() * r, y, c.z + a.sin() * r);
            let pts = [
                at(a0, inner_radius, y0),
                at(a1, inner_radius, y0),
                at(a0, outer, y0),
                at(a1, outer, y0),
                at(a0, inner_radius, y1),
                at(a1, inner_radius, y1),
                at(a0, outer, y1),
                at(a1, outer, y1),
            ];
            Brush::from_points(&pts, &[], material).ok()
        })
        .collect()
}

pub fn default_face(material: &str, normal: DVec3) -> FaceData {
    FaceData::with_normal(material, normal)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b() -> Aabb {
        Aabb::new(DVec3::splat(-32.0), DVec3::splat(32.0))
    }

    #[test]
    fn cylinder_faces() {
        let c = cylinder(&b(), 8, "m").unwrap();
        assert_eq!(c.faces.len(), 10);
        c.validate().unwrap();
    }

    #[test]
    fn generators_produce_valid_brushes() {
        cone(&b(), 12, "m").unwrap().validate().unwrap();
        wedge(&b(), "m").unwrap().validate().unwrap();
        for s in sphere(&b(), 12, 6, "m") {
            s.validate().unwrap();
        }

        assert_eq!(pipe(&b(), 12, 8.0, "m").len(), 12);
        assert_eq!(arch(&b(), 8, 8.0, "m").len(), 8);
        assert_eq!(stairs(&b(), 4, "m").len(), 4);
    }
}
