//! Finds the surface point behind every atlas texel a chart covers. Texels whose center falls outside the surface
//! but whose square still touches it take the nearest point on the surface, so the light reaches right up to the
//! edges instead of fading into the gutter.

use glam::{Vec2, Vec3};

use crate::Surface;
use crate::atlas::{Chart, Layout};

/// Texels farther than this from their surface, in texels, stay empty. A little over half the diagonal.
const REACH: f32 = 0.75;

#[derive(Clone, Copy, Debug)]
pub struct Sample {
    pub pos: Vec3,
    /// Interpolated shading normal.
    pub normal: Vec3,
    /// Normal of the triangle the point lies on, which rays are offset along.
    pub face_normal: Vec3,
    pub chart: u32,
    /// Atlas pixel index.
    pub pixel: u32,
}

pub struct Samples {
    pub list: Vec<Sample>,
    /// Per atlas pixel, the sample it holds or `u32::MAX`.
    pub at: Vec<u32>,
}

pub fn rasterize(surfaces: &[Surface], layout: &Layout) -> Samples {
    let mut at = vec![u32::MAX; (layout.width * layout.height) as usize];
    let mut list = Vec::new();
    for (ci, chart) in layout.charts.iter().enumerate() {
        chart_samples(&surfaces[chart.surface], chart, ci as u32, layout.width, &mut list, &mut at);
    }

    Samples { list, at }
}

fn chart_samples(s: &Surface, chart: &Chart, ci: u32, width: u32, list: &mut Vec<Sample>, at: &mut [u32]) {
    let (w, h) = (chart.w as usize, chart.h as usize);
    // Distance of the best candidate so far per chart pixel, and its sample.
    let mut best: Vec<(f32, Option<Sample>)> = vec![(f32::MAX, None); w * h];
    let origin = Vec2::new(chart.x as f32, chart.y as f32);
    for tri in &s.triangles {
        let [ia, ib, ic] = tri.map(|i| i as usize);
        let (Some(pa), Some(pb), Some(pc)) = (s.positions.get(ia), s.positions.get(ib), s.positions.get(ic)) else { continue };
        let face_normal = (*pb - *pa).cross(*pc - *pa);
        if face_normal.length_squared() < 1e-12 {
            continue;
        }

        let face_normal = face_normal.normalize();
        let normal_of = |k: usize| s.normals.get(k).copied().filter(|n| n.length_squared() > 1e-8).unwrap_or(face_normal);
        let (na, nb, nc) = (normal_of(ia), normal_of(ib), normal_of(ic));
        let (a, b, c) = (chart.pixel(*pa) - origin, chart.pixel(*pb) - origin, chart.pixel(*pc) - origin);
        let lo = a.min(b).min(c) - Vec2::splat(1.0);
        let hi = a.max(b).max(c) + Vec2::splat(1.0);
        let x0 = (lo.x.floor().max(0.0)) as usize;
        let y0 = (lo.y.floor().max(0.0)) as usize;
        let x1 = (hi.x.ceil() as usize).min(w);
        let y1 = (hi.y.ceil() as usize).min(h);
        for y in y0..y1 {
            for x in x0..x1 {
                if !chart.content(chart.x + x as u32, chart.y + y as u32) {
                    continue;
                }

                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let (d, bary) = closest_on_triangle(p, a, b, c);
                let slot = &mut best[y * w + x];
                if d > REACH || d >= slot.0 {
                    continue;
                }

                let pos = *pa * bary.x + *pb * bary.y + *pc * bary.z;
                let n = (na * bary.x + nb * bary.y + nc * bary.z).normalize_or(face_normal);
                let pixel = (chart.y + y as u32) * width + chart.x + x as u32;
                *slot = (d, Some(Sample { pos, normal: n, face_normal, chart: ci, pixel }));
            }
        }
    }

    for (_, sample) in best {
        if let Some(sample) = sample {
            at[sample.pixel as usize] = list.len() as u32;
            list.push(sample);
        }
    }
}

/// Distance from `p` to the triangle and the barycentric coordinates of the nearest point on it.
fn closest_on_triangle(p: Vec2, a: Vec2, b: Vec2, c: Vec2) -> (f32, Vec3) {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    let den = v0.x * v1.y - v1.x * v0.y;
    if den.abs() > 1e-12 {
        let v = (v2.x * v1.y - v1.x * v2.y) / den;
        let w = (v0.x * v2.y - v2.x * v0.y) / den;
        let u = 1.0 - v - w;
        if u >= -1e-5 && v >= -1e-5 && w >= -1e-5 {
            return (0.0, Vec3::new(u, v, w));
        }
    }

    let mut best = (f32::MAX, Vec3::X);
    for (s, e, pick) in [(a, b, 0), (b, c, 1), (c, a, 2)] {
        let d = e - s;
        let t = if d.length_squared() > 1e-12 { ((p - s).dot(d) / d.length_squared()).clamp(0.0, 1.0) } else { 0.0 };
        let dist = (s + d * t - p).length();
        if dist < best.0 {
            let bary = match pick {
                0 => Vec3::new(1.0 - t, t, 0.0),
                1 => Vec3::new(0.0, 1.0 - t, t),
                _ => Vec3::new(t, 0.0, 1.0 - t),
            };
            best = (dist, bary);
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Settings;
    use crate::atlas::layout;

    #[test]
    fn every_texel_touching_a_quad_gets_a_point_on_it() {
        let size = 100.0;
        let s = Surface {
            positions: vec![Vec3::ZERO, Vec3::new(size, 0.0, 0.0), Vec3::new(size, 0.0, -size), Vec3::new(0.0, 0.0, -size)],
            normals: vec![Vec3::Y; 4],
            triangles: vec![[0, 1, 2], [0, 2, 3]],
            axes: [Vec3::X, Vec3::NEG_Z],
            receives: true,
            ..Surface::default()
        };
        let l = layout(std::slice::from_ref(&s), &Settings { texel_size: 10.0, ..Settings::default() }).unwrap();
        let samples = rasterize(std::slice::from_ref(&s), &l);
        let c = &l.charts[0];
        // 100 units at 10 per texel spans 11 texel centers per side, all on the quad.
        assert_eq!(samples.list.len(), 11 * 11);
        for sample in &samples.list {
            assert!(sample.pos.x >= -1e-3 && sample.pos.x <= size + 1e-3 && sample.pos.z <= 1e-3 && sample.pos.z >= -size - 1e-3);
            assert!((sample.normal - Vec3::Y).length() < 1e-5);
            let (x, y) = (sample.pixel % l.width, sample.pixel / l.width);
            assert!(c.content(x, y));
            let px = c.pixel(sample.pos);
            assert!((px - Vec2::new(x as f32 + 0.5, y as f32 + 0.5)).length() < 1e-3, "the sample sits at its texel center");
        }
    }
}
