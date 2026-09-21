//! Mesh primitive generators. Faces wind counter-clockwise seen from outside, Y is up.

use std::f64::consts::{PI, TAU};

use gt_core::{Aabb, DVec2, DVec3};

use crate::brush::FaceData;
use crate::mesh::{Mesh, MeshFace};
use crate::polygon;
use crate::uv::FaceUv;

fn data(material: &str, pts: &[DVec3]) -> FaceData {
    FaceData::new(material, FaceUv::face_aligned(polygon::newell(pts).normalize_or(DVec3::Y), DVec2::ONE))
}

fn build(vertices: Vec<DVec3>, faces: Vec<Vec<u32>>, material: &str, smooth_angle: f32) -> Mesh {
    let mut mesh = Mesh { vertices, faces: Vec::with_capacity(faces.len()), smooth_angle, decal: false };
    for idx in faces {
        let pts: Vec<DVec3> = idx.iter().map(|i| mesh.vertices[*i as usize]).collect();
        mesh.faces.push(MeshFace::new(idx, data(material, &pts)));
    }

    mesh
}

pub fn cuboid(bounds: &Aabb, material: &str) -> Mesh {
    let b = crate::Brush::from_aabb(bounds, material).expect("non empty bounds");
    Mesh::from_brush(&b)
}

/// Flat grid on the bottom of the bounds, `nx` by `nz` quads.
pub fn grid(bounds: &Aabb, nx: usize, nz: usize, material: &str) -> Mesh {
    let (nx, nz) = (nx.max(1), nz.max(1));
    let size = bounds.size();
    let mut vertices = Vec::with_capacity((nx + 1) * (nz + 1));
    for j in 0..=nz {
        for i in 0..=nx {
            vertices.push(DVec3::new(bounds.min.x + size.x * i as f64 / nx as f64, bounds.min.y, bounds.min.z + size.z * j as f64 / nz as f64));
        }
    }

    let row = nx as u32 + 1;
    let mut faces = Vec::with_capacity(nx * nz);
    for j in 0..nz as u32 {
        for i in 0..nx as u32 {
            let a = j * row + i;
            faces.push(vec![a, a + row, a + row + 1, a + 1]);
        }
    }

    build(vertices, faces, material, 0.0)
}

/// Surface of revolution around the vertical axis through `center`. `profile` holds (radius, height) pairs from
/// bottom to top. A zero radius end becomes a pole, other ends are capped when `caps` is set.
pub fn lathe(center: DVec3, profile: &[DVec2], sides: usize, caps: bool, material: &str, smooth_angle: f32) -> Mesh {
    let sides = sides.max(3);
    let mut vertices = Vec::new();
    let mut rings: Vec<Vec<u32>> = Vec::new();
    for p in profile {
        if p.x.abs() < 1e-9 {
            vertices.push(center + DVec3::new(0.0, p.y, 0.0));
            rings.push(vec![(vertices.len() - 1) as u32; sides]);
            continue;
        }

        let mut ring = Vec::with_capacity(sides);
        for s in 0..sides {
            let a = TAU * s as f64 / sides as f64;
            ring.push(vertices.len() as u32);
            vertices.push(center + DVec3::new(a.cos() * p.x, p.y, a.sin() * p.x));
        }

        rings.push(ring);
    }

    let mut faces = Vec::new();
    for r in 0..rings.len().saturating_sub(1) {
        for s in 0..sides {
            let t = (s + 1) % sides;
            let quad = [rings[r][s], rings[r + 1][s], rings[r + 1][t], rings[r][t]];
            let mut f: Vec<u32> = Vec::with_capacity(4);
            for v in quad {
                if !f.contains(&v) {
                    f.push(v);
                }
            }

            if f.len() >= 3 {
                faces.push(f);
            }
        }
    }

    if caps {
        if let (Some(first), Some(p)) = (rings.first(), profile.first())
            && p.x.abs() > 1e-9
        {
            faces.push(first.clone());
        }

        if let (Some(last), Some(p)) = (rings.last(), profile.last())
            && p.x.abs() > 1e-9
        {
            faces.push(last.iter().rev().copied().collect());
        }
    }

    let mut mesh = build(vertices, faces, material, smooth_angle);
    // Caps stay flat even on smooth shaded solids: their normals differ from the sides by 90 degrees.
    mesh.cleanup();
    mesh
}

pub fn cylinder(bounds: &Aabb, sides: usize, material: &str) -> Mesh {
    let c = bounds.center();
    let r = bounds.size().x.min(bounds.size().z) * 0.5;
    lathe(DVec3::new(c.x, 0.0, c.z), &[DVec2::new(r, bounds.min.y), DVec2::new(r, bounds.max.y)], sides, true, material, 40.0)
}

pub fn cone(bounds: &Aabb, sides: usize, material: &str) -> Mesh {
    let c = bounds.center();
    let r = bounds.size().x.min(bounds.size().z) * 0.5;
    lathe(DVec3::new(c.x, 0.0, c.z), &[DVec2::new(r, bounds.min.y), DVec2::new(0.0, bounds.max.y)], sides, true, material, 50.0)
}

pub fn sphere(bounds: &Aabb, sides: usize, rings: usize, material: &str) -> Mesh {
    let c = bounds.center();
    let r = bounds.size() * 0.5;
    let rings = rings.max(2);
    let profile: Vec<DVec2> = (0..=rings)
        .map(|i| {
            let lat = PI * i as f64 / rings as f64;
            DVec2::new(lat.sin(), c.y - r.y * lat.cos())
        })
        .collect();
    let mut m = lathe(DVec3::new(c.x, 0.0, c.z), &profile, sides, false, material, 60.0);
    let scale = DVec3::new(r.x, 1.0, r.z);
    for v in &mut m.vertices {
        let local = *v - DVec3::new(c.x, 0.0, c.z);
        *v = DVec3::new(c.x, 0.0, c.z) + local * scale;
    }

    m
}

pub fn torus(center: DVec3, radius: f64, tube: f64, sides: usize, tube_sides: usize, material: &str) -> Mesh {
    let tube_sides = tube_sides.max(3);
    let profile: Vec<DVec2> = (0..=tube_sides)
        .map(|i| {
            let a = TAU * i as f64 / tube_sides as f64 - PI * 0.5;
            DVec2::new(radius + tube * a.cos(), tube * a.sin())
        })
        .collect();
    let mut m = lathe(center, &profile, sides, false, material, 60.0);
    m.weld(1e-6);
    m
}

/// Extrudes a planar polygon (counter-clockwise around its normal) by `offset`. The polygon becomes the back cap.
pub fn extrude_polygon(points: &[DVec3], offset: DVec3, material: &str) -> Mesh {
    let n = points.len();
    if n < 3 {
        return Mesh::default();
    }

    let normal = polygon::newell(points);
    // The back cap faces against the extrusion, so the outline has to wind away from it.
    let outline: Vec<DVec3> = if normal.dot(offset) > 0.0 { points.iter().rev().copied().collect() } else { points.to_vec() };
    let mut vertices = outline.clone();
    vertices.extend(outline.iter().map(|p| *p + offset));
    let n32 = n as u32;
    let mut faces = vec![(0..n32).collect::<Vec<_>>(), (0..n32).rev().map(|i| i + n32).collect()];
    for i in 0..n32 {
        let j = (i + 1) % n32;
        faces.push(vec![j, i, i + n32, j + n32]);
    }

    build(vertices, faces, material, 0.0)
}

/// Vertical prism over a footprint given as (x, z) points.
pub fn prism(footprint: &[DVec2], bottom: f64, top: f64, material: &str) -> Mesh {
    let pts: Vec<DVec3> = footprint.iter().map(|p| DVec3::new(p.x, bottom, p.y)).collect();
    extrude_polygon(&pts, DVec3::new(0.0, top - bottom, 0.0), material)
}

/// Wall with an arched opening, in the XY plane of the bounds and `bounds.size().z` thick.
pub fn arch_wall(bounds: &Aabb, opening_width: f64, opening_height: f64, segments: usize, material: &str) -> Mesh {
    let segments = segments.max(2);
    let cx = bounds.center().x;
    let half = opening_width * 0.5;
    let spring = bounds.min.y + (opening_height - half).max(0.0);
    let mut outline = vec![DVec2::new(bounds.min.x, bounds.min.y), DVec2::new(cx - half, bounds.min.y)];
    for s in 0..=segments {
        let a = PI - PI * s as f64 / segments as f64;
        outline.push(DVec2::new(cx + half * a.cos(), spring + half * a.sin()));
    }

    outline.extend([
        DVec2::new(cx + half, bounds.min.y),
        DVec2::new(bounds.max.x, bounds.min.y),
        DVec2::new(bounds.max.x, bounds.max.y),
        DVec2::new(bounds.min.x, bounds.max.y),
    ]);
    let pts: Vec<DVec3> = outline.iter().map(|p| DVec3::new(p.x, p.y, bounds.max.z)).collect();
    let mut m = extrude_polygon(&pts, DVec3::new(0.0, 0.0, -bounds.size().z), material);
    m.weld(1e-6);
    m
}

/// Gable roof over the bounds with the ridge along X (or Z when `ridge_z`), overhanging by `overhang`.
pub fn gable_roof(bounds: &Aabb, ridge_z: bool, thickness: f64, overhang: f64, material: &str) -> Mesh {
    let (a, b) = (bounds.min, bounds.max);
    let mid = |lo: f64, hi: f64| (lo + hi) * 0.5;
    let t = thickness.max(0.01);
    let profile = |w0: f64, w1: f64| {
        let m = mid(w0, w1);
        vec![
            DVec2::new(w0 - overhang, a.y - t),
            DVec2::new(w0 - overhang, a.y),
            DVec2::new(m, b.y),
            DVec2::new(w1 + overhang, a.y),
            DVec2::new(w1 + overhang, a.y - t),
            DVec2::new(m, b.y - t * 1.4),
        ]
    };
    if ridge_z {
        let pts: Vec<DVec3> = profile(a.x, b.x).iter().map(|p| DVec3::new(p.x, p.y, b.z + overhang)).collect();
        let pts: Vec<DVec3> = if polygon::newell(&pts).z < 0.0 { pts.into_iter().rev().collect() } else { pts };
        extrude_polygon(&pts, DVec3::new(0.0, 0.0, -(b.z - a.z) - overhang * 2.0), material)
    } else {
        let pts: Vec<DVec3> = profile(a.z, b.z).iter().map(|p| DVec3::new(a.x - overhang, p.y, p.x)).collect();
        extrude_polygon(&pts, DVec3::new((b.x - a.x) + overhang * 2.0, 0.0, 0.0), material)
    }
}

/// Four sided pyramid roof (spire) over the bounds.
pub fn spire(bounds: &Aabb, sides: usize, material: &str) -> Mesh {
    let c = bounds.center();
    let r = bounds.size().x.min(bounds.size().z) * 0.5 * std::f64::consts::SQRT_2;
    let mut m = lathe(DVec3::new(c.x, 0.0, c.z), &[DVec2::new(r, bounds.min.y), DVec2::new(0.0, bounds.max.y)], sides, true, material, 0.0);
    let rot = gt_core::DMat4::from_translation(DVec3::new(c.x, 0.0, c.z))
        * gt_core::DMat4::from_rotation_y(PI / sides.max(3) as f64)
        * gt_core::DMat4::from_translation(DVec3::new(-c.x, 0.0, -c.z));
    m = m.transformed(&rot, false);
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b() -> Aabb {
        Aabb::new(DVec3::splat(-32.0), DVec3::splat(32.0))
    }

    #[test]
    fn closed_primitives_have_positive_volume() {
        for (name, m) in [
            ("cuboid", cuboid(&b(), "m")),
            ("cylinder", cylinder(&b(), 16, "m")),
            ("cone", cone(&b(), 12, "m")),
            ("sphere", sphere(&b(), 16, 8, "m")),
            ("torus", torus(DVec3::ZERO, 32.0, 8.0, 16, 8, "m")),
            (
                "prism",
                prism(&[DVec2::new(0.0, 0.0), DVec2::new(64.0, 0.0), DVec2::new(64.0, 32.0), DVec2::new(32.0, 64.0), DVec2::new(0.0, 32.0)], 0.0, 16.0, "m"),
            ),
            ("arch", arch_wall(&b(), 32.0, 48.0, 8, "m")),
            ("roof", gable_roof(&b(), false, 4.0, 4.0, "m")),
            ("roof z", gable_roof(&b(), true, 4.0, 4.0, "m")),
            ("spire", spire(&b(), 4, "m")),
        ] {
            m.validate().unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(m.is_closed(), "{name} open: {:?}", m.boundary_edges());
            assert!(m.volume() > 0.0, "{name} volume {}", m.volume());
        }
    }

    #[test]
    fn prism_winding_is_independent_of_footprint_order() {
        let fp = [DVec2::new(0.0, 0.0), DVec2::new(0.0, 32.0), DVec2::new(32.0, 32.0), DVec2::new(32.0, 0.0)];
        let rev: Vec<DVec2> = fp.iter().rev().copied().collect();
        assert!((prism(&fp, 0.0, 8.0, "m").volume() - prism(&rev, 0.0, 8.0, "m").volume()).abs() < 1e-9);
    }

    #[test]
    fn arch_wall_volume_excludes_opening() {
        let m = arch_wall(&Aabb::new(DVec3::new(-64.0, 0.0, -8.0), DVec3::new(64.0, 128.0, 8.0)), 48.0, 96.0, 16, "m");
        let full = 128.0 * 128.0 * 16.0;
        let opening = (48.0 * 72.0 + PI * 24.0 * 24.0 * 0.5) * 16.0;
        assert!((m.volume() - (full - opening)).abs() < 150.0, "volume {} vs {}", m.volume(), full - opening);
    }
}
