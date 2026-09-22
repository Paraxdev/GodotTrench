//! Hammer style displacements: a quad face subdivided into a (2^power + 1)² grid of vertices that
//! can be pushed along the face normal and carry blend weights for two-texture blending.

use gt_core::{DVec2, DVec3};
use serde::{Deserialize, Serialize};

use crate::brush::Brush;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Displacement {
    /// 1 to 4. Each side has 2^power + 1 vertices.
    pub power: u8,
    /// Offset along the face normal per grid vertex, row major (v rows of u columns).
    pub heights: Vec<f32>,
    /// Texture blend weight per grid vertex, 0 = first texture, 1 = second.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub alphas: Vec<f32>,
}

#[derive(Clone, Debug)]
pub struct DisplacementGrid {
    /// Vertices per side.
    pub size: usize,
    /// Flat positions on the face, used for texture coordinates.
    pub base: Vec<DVec3>,
    pub positions: Vec<DVec3>,
    pub normals: Vec<DVec3>,
    pub alphas: Vec<f32>,
}

impl Displacement {
    pub fn check_data(&self) -> Result<(), String> {
        if !(1..=4).contains(&self.power) {
            return Err(format!("displacement power {} is outside 1 to 4", self.power));
        }

        let n = Self::side(self.power);
        if self.heights.len() != n * n {
            return Err(format!("displacement of power {} needs {} heights, has {}", self.power, n * n, self.heights.len()));
        }

        if !self.alphas.is_empty() && self.alphas.len() != n * n {
            return Err(format!("displacement of power {} needs {} alphas, has {}", self.power, n * n, self.alphas.len()));
        }

        Ok(())
    }

    pub fn new(power: u8) -> Self {
        let power = power.clamp(1, 4);
        let n = Self::side(power);
        Self { power, heights: vec![0.0; n * n], alphas: Vec::new() }
    }

    pub fn side(power: u8) -> usize {
        (1usize << power) + 1
    }

    pub fn size(&self) -> usize {
        Self::side(self.power)
    }

    pub fn is_valid(&self) -> bool {
        let n = self.size();
        self.heights.len() == n * n && (self.alphas.is_empty() || self.alphas.len() == n * n)
    }

    pub fn alpha(&self, index: usize) -> f32 {
        self.alphas.get(index).copied().unwrap_or(0.0)
    }

    /// Mirrors rows, used when the owning face winding is reversed.
    pub fn flip_v(&mut self) {
        let n = self.size();
        for data in [&mut self.heights, &mut self.alphas] {
            if data.len() != n * n {
                continue;
            }

            for j in 0..n / 2 {
                for i in 0..n {
                    data.swap(j * n + i, (n - 1 - j) * n + i);
                }
            }
        }
    }

    /// Changes the power, resampling heights and alphas bilinearly.
    pub fn resample(&self, power: u8) -> Displacement {
        let power = power.clamp(1, 4);
        let (old_n, new_n) = (self.size(), Self::side(power));
        let sample = |data: &[f32], u: f64, v: f64| -> f32 {
            if data.len() != old_n * old_n {
                return 0.0;
            }

            let (x, y) = (u * (old_n - 1) as f64, v * (old_n - 1) as f64);
            let (x0, y0) = (x.floor() as usize, y.floor() as usize);
            let (x1, y1) = ((x0 + 1).min(old_n - 1), (y0 + 1).min(old_n - 1));
            let (fx, fy) = ((x - x0 as f64) as f32, (y - y0 as f64) as f32);
            let at = |i: usize, j: usize| data[j * old_n + i];
            let top = at(x0, y0) * (1.0 - fx) + at(x1, y0) * fx;
            let bottom = at(x0, y1) * (1.0 - fx) + at(x1, y1) * fx;
            top * (1.0 - fy) + bottom * fy
        };
        let mut out = Displacement::new(power);
        let has_alpha = !self.alphas.is_empty();
        if has_alpha {
            out.alphas = vec![0.0; new_n * new_n];
        }

        for j in 0..new_n {
            for i in 0..new_n {
                let (u, v) = (i as f64 / (new_n - 1) as f64, j as f64 / (new_n - 1) as f64);
                out.heights[j * new_n + i] = sample(&self.heights, u, v);
                if has_alpha {
                    out.alphas[j * new_n + i] = sample(&self.alphas, u, v);
                }
            }
        }

        out
    }
}

/// Grid for a displacement face, or None if the face is not a quad or the data does not match.
pub fn grid(brush: &Brush, face: usize) -> Option<DisplacementGrid> {
    let f = brush.faces.get(face)?;
    let disp = f.data.disp.as_ref()?;
    if f.indices.len() != 4 || !disp.is_valid() {
        return None;
    }

    let c: Vec<DVec3> = f.indices.iter().map(|i| brush.vertices[*i as usize]).collect();
    let normal = f.plane.normal;
    let n = disp.size();
    let mut base = Vec::with_capacity(n * n);
    let mut positions = Vec::with_capacity(n * n);
    for j in 0..n {
        let v = j as f64 / (n - 1) as f64;
        for i in 0..n {
            let u = i as f64 / (n - 1) as f64;
            // c0 -> c1 is the u direction, c0 -> c3 the v direction.
            let p = c[0] * (1.0 - u) * (1.0 - v) + c[1] * u * (1.0 - v) + c[2] * u * v + c[3] * (1.0 - u) * v;
            base.push(p);
            positions.push(p + normal * disp.heights[j * n + i] as f64);
        }
    }

    let normals = grid_normals(&positions, n, normal);
    let alphas = (0..n * n).map(|k| disp.alpha(k)).collect();
    Some(DisplacementGrid { size: n, base, positions, normals, alphas })
}

fn grid_normals(positions: &[DVec3], n: usize, fallback: DVec3) -> Vec<DVec3> {
    let mut normals = vec![DVec3::ZERO; n * n];
    for (a, b, c) in triangles(n) {
        let fnorm = (positions[b] - positions[a]).cross(positions[c] - positions[a]);
        normals[a] += fnorm;
        normals[b] += fnorm;
        normals[c] += fnorm;
    }

    normals.into_iter().map(|v| v.try_normalize().unwrap_or(fallback)).collect()
}

/// Triangle vertex indices for an n x n grid, counter-clockwise around the face normal.
pub fn triangles(n: usize) -> impl Iterator<Item = (usize, usize, usize)> {
    (0..n - 1).flat_map(move |j| {
        (0..n - 1).flat_map(move |i| {
            let a = j * n + i;
            let b = a + 1;
            let c = a + n + 1;
            let d = a + n;
            // Alternate the split diagonal so terrain folds symmetrically.
            if (i + j) % 2 == 0 { [(a, b, c), (a, c, d)] } else { [(a, b, d), (b, c, d)] }
        })
    })
}

/// Face UV position helper for grids.
pub fn uv(brush: &Brush, face: usize, p: DVec3, tex_size: DVec2) -> DVec2 {
    brush.faces[face].data.uv.uv(p, tex_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brush::Brush;
    use gt_core::{Aabb, DMat4};

    fn disp_box() -> (Brush, usize) {
        let mut b = Brush::from_aabb(&Aabb::new(DVec3::ZERO, DVec3::new(64.0, 16.0, 64.0)), "m").unwrap();
        let top = b.faces.iter().position(|f| f.plane.normal.y > 0.5).unwrap();
        let mut d = Displacement::new(2);
        d.heights[12] = 8.0;
        b.faces[top].data.disp = Some(d);
        (b, top)
    }

    #[test]
    fn grid_raises_center() {
        let (b, top) = disp_box();
        let g = grid(&b, top).unwrap();
        assert_eq!(g.size, 5);
        let center = g.positions[12];
        assert!((center - DVec3::new(32.0, 24.0, 32.0)).length() < 1e-9, "{center}");
        assert!(g.normals[12].y > 0.99);
        assert_eq!(triangles(5).count(), 32);
        for (a, b2, c) in triangles(5) {
            let n = (g.base[b2] - g.base[a]).cross(g.base[c] - g.base[a]);
            assert!(n.y > 0.0, "triangles face along the normal");
        }
    }

    #[test]
    fn resample_keeps_shape() {
        let (b, top) = disp_box();
        let d = b.faces[top].data.disp.as_ref().unwrap();
        let up = d.resample(3);
        assert_eq!(up.heights.len(), 81);
        assert!((up.heights[40] - 8.0).abs() < 1e-6);
        let down = up.resample(2);
        assert!((down.heights[12] - 8.0).abs() < 1e-6);
    }

    #[test]
    fn survives_mirroring() {
        let (b, top) = disp_box();
        let before = grid(&b, top).unwrap();
        let mirrored = b.transformed(&DMat4::from_scale(DVec3::new(-1.0, 1.0, 1.0)), true);
        let top2 = mirrored.faces.iter().position(|f| f.plane.normal.y > 0.5).unwrap();
        let after = grid(&mirrored, top2).unwrap();
        let mut a: Vec<(i64, i64, i64)> = before.positions.iter().map(|p| ((-p.x).round() as i64, p.y.round() as i64, p.z.round() as i64)).collect();
        let mut c: Vec<(i64, i64, i64)> = after.positions.iter().map(|p| (p.x.round() as i64, p.y.round() as i64, p.z.round() as i64)).collect();
        a.sort();
        c.sort();
        assert_eq!(a, c);
    }
}
