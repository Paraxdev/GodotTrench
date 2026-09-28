//! Charts: each receiving surface is flattened onto its plane at the texel size, and the charts are packed into one
//! atlas. A point's atlas coordinate is an affine function of its position, so the Godot addon can compute the UV2 of
//! any vertex it builds from the chart's two rows.

use glam::{Vec2, Vec3};

use crate::{Settings, Surface};

/// Texels of gutter around every chart, so bilinear filtering and dilation never read a neighbour.
pub const PAD: u32 = 1;
/// Longest side of one chart, in texels. Bigger surfaces are sampled coarser rather than crowding the atlas.
const MAX_CHART: u32 = 1024;
/// Smallest atlas side.
const MIN_SIZE: u32 = 16;

#[derive(Clone, Debug)]
pub struct Chart {
    /// Index of the surface in the bake input.
    pub surface: usize,
    pub x: u32,
    pub y: u32,
    /// Size including the gutter.
    pub w: u32,
    pub h: u32,
    /// Map units per texel of this chart.
    pub texel: f32,
    /// The surface's projection onto its chart axes starts here, in map units.
    pub min: Vec2,
    pub axes: [Vec3; 2],
}

impl Chart {
    /// Atlas pixel coordinate of a point, texel centers at half integers.
    pub fn pixel(&self, p: Vec3) -> Vec2 {
        let a = Vec2::new(p.dot(self.axes[0]), p.dot(self.axes[1]));
        Vec2::new(self.x as f32, self.y as f32) + Vec2::splat(PAD as f32 + 0.5) + (a - self.min) / self.texel
    }

    /// The two rows that map a position to its 0..1 atlas coordinate: `uv = (s.xyz . p + s.w, t.xyz . p + t.w)`.
    pub fn rows(&self, width: u32, height: u32) -> [[f32; 4]; 2] {
        let row = |axis: Vec3, min: f32, start: u32, size: u32| {
            let k = 1.0 / (self.texel * size as f32);
            let s = axis * k;
            [s.x, s.y, s.z, (start as f32 + PAD as f32 + 0.5) / size as f32 - min * k]
        };
        [row(self.axes[0], self.min.x, self.x, width), row(self.axes[1], self.min.y, self.y, height)]
    }

    /// Whether an atlas pixel lies in this chart's content, not its gutter.
    pub fn content(&self, x: u32, y: u32) -> bool {
        x >= self.x + PAD && y >= self.y + PAD && x < self.x + self.w - PAD && y < self.y + self.h - PAD
    }
}

pub struct Layout {
    pub width: u32,
    pub height: u32,
    pub charts: Vec<Chart>,
    /// Map units per texel after any coarsening to fit `max_size`.
    pub texel: f32,
}

/// Charts every receiving surface and packs them into an atlas no larger than `settings.max_size`, coarsening the
/// texel size until they fit. None when nothing receives light.
pub fn layout(surfaces: &[Surface], settings: &Settings) -> Option<Layout> {
    let receivers: Vec<(usize, [Vec3; 2], Vec2, Vec2)> = surfaces
        .iter()
        .enumerate()
        .filter(|(_, s)| s.receives && !s.triangles.is_empty())
        .map(|(i, s)| {
            let (mut lo, mut hi) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
            for p in &s.positions {
                let a = Vec2::new(p.dot(s.axes[0]), p.dot(s.axes[1]));
                lo = lo.min(a);
                hi = hi.max(a);
            }

            (i, s.axes, lo, hi)
        })
        .collect();
    if receivers.is_empty() {
        return None;
    }

    let max = settings.max_size.max(MIN_SIZE);
    let mut texel = settings.texel_size.max(0.01);
    for _ in 0..64 {
        let mut charts: Vec<Chart> = receivers
            .iter()
            .map(|(i, axes, lo, hi)| {
                let mut t = texel * surfaces[*i].texel_scale.max(0.01);
                let extent = *hi - *lo;
                let longest = extent.max_element();
                let limit = MAX_CHART.min(max / 2).max(4) as f32 - 2.0 * PAD as f32 - 1.0;
                if longest / t > limit {
                    t = longest / limit;
                }

                let content = |e: f32| (e / t).ceil() as u32 + 1;
                Chart { surface: *i, x: 0, y: 0, w: content(extent.x) + 2 * PAD, h: content(extent.y) + 2 * PAD, texel: t, min: *lo, axes: *axes }
            })
            .collect();
        if let Some((width, height)) = pack(&mut charts, max) {
            return Some(Layout { width, height, charts, texel });
        }

        texel *= 1.25;
    }

    None
}

/// Shelf packs the charts, tallest first. Returns the atlas size, or None when they do not fit in `max` square.
fn pack(charts: &mut [Chart], max: u32) -> Option<(u32, u32)> {
    let area: u64 = charts.iter().map(|c| c.w as u64 * c.h as u64).sum();
    let widest = charts.iter().map(|c| c.w).max().unwrap_or(1);
    let mut width = (((area as f64 * 1.15).sqrt().ceil() as u32).max(widest).max(MIN_SIZE) + 3) & !3;
    if width > max {
        width = max;
    }

    let mut order: Vec<usize> = (0..charts.len()).collect();
    order.sort_by_key(|i| (std::cmp::Reverse(charts[*i].h), std::cmp::Reverse(charts[*i].w)));
    loop {
        let (mut x, mut y, mut shelf) = (0u32, 0u32, 0u32);
        let mut fits = true;
        for i in &order {
            let c = &mut charts[*i];
            if c.w > width {
                fits = false;
                break;
            }

            if x + c.w > width {
                x = 0;
                y += shelf;
                shelf = 0;
            }

            c.x = x;
            c.y = y;
            x += c.w;
            shelf = shelf.max(c.h);
        }

        let height = ((y + shelf).max(MIN_SIZE) + 3) & !3;
        if fits && height <= max {
            // A square-ish atlas wastes less when it is sampled, so widen while the height runs away.
            if height > width * 2 && width < max {
                width = (width * 5 / 4 + 3).min(max) & !3;
                continue;
            }

            return Some((width, height));
        }

        if width >= max {
            return None;
        }

        width = (width * 5 / 4 + 3).min(max) & !3;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quad(size: f32, offset: Vec3) -> Surface {
        Surface {
            positions: vec![offset, offset + Vec3::X * size, offset + Vec3::new(size, size, 0.0), offset + Vec3::Y * size],
            normals: vec![Vec3::Z; 4],
            triangles: vec![[0, 1, 2], [0, 2, 3]],
            axes: [Vec3::X, Vec3::Y],
            ..Surface::default()
        }
    }

    #[test]
    fn charts_do_not_overlap_and_map_corners_inside_their_content() {
        let surfaces: Vec<Surface> = (0..40).map(|k| quad(16.0 + (k % 7) as f32 * 30.0, Vec3::new(k as f32 * 500.0, 0.0, 0.0))).collect();
        let settings = Settings { texel_size: 8.0, ..Settings::default() };
        let l = layout(&surfaces, &settings).unwrap();
        for (i, a) in l.charts.iter().enumerate() {
            assert!(a.x + a.w <= l.width && a.y + a.h <= l.height);
            for b in &l.charts[i + 1..] {
                let apart = a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y;
                assert!(apart, "charts overlap");
            }

            let rows = a.rows(l.width, l.height);
            for p in &surfaces[a.surface].positions {
                let px = a.pixel(*p);
                let uv = Vec2::new(Vec3::from_slice(&rows[0]).dot(*p) + rows[0][3], Vec3::from_slice(&rows[1]).dot(*p) + rows[1][3]);
                assert!((uv * Vec2::new(l.width as f32, l.height as f32) - px).length() < 1e-2, "rows agree with pixel()");
                assert!(px.x >= (a.x + PAD) as f32 + 0.49 && px.x <= (a.x + a.w - PAD) as f32 - 0.49, "corner inside the content");
                assert!(px.y >= (a.y + PAD) as f32 + 0.49 && px.y <= (a.y + a.h - PAD) as f32 - 0.49);
            }
        }
    }

    #[test]
    fn coarsens_the_texel_until_the_charts_fit() {
        let surfaces: Vec<Surface> = (0..30).map(|k| quad(1000.0, Vec3::new(0.0, 0.0, k as f32 * 10.0))).collect();
        let settings = Settings { texel_size: 4.0, max_size: 512, ..Settings::default() };
        let l = layout(&surfaces, &settings).unwrap();
        assert!(l.width <= 512 && l.height <= 512);
        assert!(l.texel > 4.0);
    }
}
