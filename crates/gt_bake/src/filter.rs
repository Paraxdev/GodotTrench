//! Turns per sample results into atlases: smooths the bounce noise within each chart and fills empty texels and
//! gutters from their neighbours, so filtering at a chart's edge never reads black.

use glam::Vec3;

use crate::atlas::Layout;
use crate::raster::Samples;
use crate::trace::{Traced, par_map};

/// Filled texels spread into their empty neighbours this many times at most.
const DILATE_STEPS: usize = 64;

/// Chart index of every atlas pixel, gutters included, `u32::MAX` between charts.
fn chart_map(layout: &Layout) -> Vec<u32> {
    let mut map = vec![u32::MAX; (layout.width * layout.height) as usize];
    for (ci, c) in layout.charts.iter().enumerate() {
        for y in c.y..c.y + c.h {
            let row = (y * layout.width) as usize;
            map[row + c.x as usize..row + (c.x + c.w) as usize].fill(ci as u32);
        }
    }

    map
}

/// An atlas of `value` per sample, spread over each whole chart. Samples without a value are filled like gutters.
pub fn spread(layout: &Layout, samples: &Samples, value: impl Fn(usize) -> Option<Vec3>) -> Vec<Vec3> {
    let size = (layout.width * layout.height) as usize;
    let mut data = vec![[0.0f32; 3]; size];
    let mut filled = vec![false; size];
    for (i, s) in samples.list.iter().enumerate() {
        if let Some(v) = value(i) {
            data[s.pixel as usize] = v.to_array();
            filled[s.pixel as usize] = true;
        }
    }

    dilate(layout, &chart_map(layout), &mut filled, &mut data);
    data.into_iter().map(Vec3::from).collect()
}

/// Grows every chart's filled texels into its empty ones, each empty texel taking the mean of its filled neighbours.
fn dilate<const N: usize>(layout: &Layout, charts: &[u32], filled: &mut [bool], data: &mut [[f32; N]]) {
    let (w, h) = (layout.width as i64, layout.height as i64);
    let mut frontier: Vec<usize> = (0..filled.len()).filter(|i| !filled[*i] && charts[*i] != u32::MAX).collect();
    let threads = std::thread::available_parallelism().map_or(4, |p| p.get());
    for _ in 0..DILATE_STEPS {
        let (filled_now, data_now) = (&*filled, &*data);
        let step = |part: &[usize]| {
            let mut updates = Vec::new();
            for &i in part {
                let (x, y) = (i as i64 % w, i as i64 / w);
                let mut sum = [0.0f32; N];
                let mut count = 0;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let (nx, ny) = (x + dx, y + dy);
                        if (dx, dy) == (0, 0) || nx < 0 || ny < 0 || nx >= w || ny >= h {
                            continue;
                        }

                        let j = (ny * w + nx) as usize;
                        if filled_now[j] && charts[j] == charts[i] {
                            for (s, v) in sum.iter_mut().zip(data_now[j]) {
                                *s += v;
                            }

                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    updates.push((i, sum.map(|s| s / count as f32)));
                }
            }

            updates
        };
        let updates: Vec<(usize, [f32; N])> = if frontier.len() < 4096 {
            step(&frontier)
        } else {
            let part = frontier.len().div_ceil(threads);
            std::thread::scope(|s| {
                let jobs: Vec<_> = frontier.chunks(part).map(|c| s.spawn(move || step(c))).collect();
                jobs.into_iter().flat_map(|j| j.join().unwrap_or_default()).collect()
            })
        };

        if updates.is_empty() {
            break;
        }

        for (i, v) in &updates {
            data[*i] = *v;
            filled[*i] = true;
        }

        frontier.retain(|i| !filled[*i]);
    }
}

/// Edge aware blur of the bounce light: neighbours on the same chart facing the same way and lying in the same plane
/// count, so the noise evens out without light bleeding across corners.
fn denoise(layout: &Layout, samples: &Samples, traced: &Traced) -> Vec<Vec3> {
    const RADIUS: i64 = 2;
    let (w, h) = (layout.width as i64, layout.height as i64);
    par_map(samples.list.len(), |i| {
        let s = &samples.list[i];
        if !traced.valid[i] {
            return traced.indirect[i];
        }

        let texel = layout.charts[s.chart as usize].texel;
        let (x, y) = (s.pixel as i64 % w, s.pixel as i64 / w);
        let (mut sum, mut weight) = (Vec3::ZERO, 0.0);
        for dy in -RADIUS..=RADIUS {
            for dx in -RADIUS..=RADIUS {
                let (nx, ny) = (x + dx, y + dy);
                if nx < 0 || ny < 0 || nx >= w || ny >= h {
                    continue;
                }

                let j = samples.at[(ny * w + nx) as usize];
                if j == u32::MAX || !traced.valid[j as usize] {
                    continue;
                }

                let o = &samples.list[j as usize];
                if o.chart != s.chart {
                    continue;
                }

                let facing = s.normal.dot(o.normal).max(0.0).powi(8);
                let off_plane = (s.normal.dot(o.pos - s.pos) / texel).abs();
                let wgt = (-((dx * dx + dy * dy) as f32) / 4.5).exp() * facing * (-off_plane * 4.0).exp();
                sum += traced.indirect[j as usize] * wgt;
                weight += wgt;
            }
        }

        if weight > 0.0 { sum / weight } else { traced.indirect[i] }
    })
}

/// The finished light, shadow and ambient occlusion atlases.
pub fn finish(layout: &Layout, samples: &Samples, traced: &Traced, smooth: bool) -> (Vec<[f32; 3]>, Vec<f32>, Vec<f32>) {
    let indirect = if smooth { denoise(layout, samples, traced) } else { traced.indirect.clone() };
    let size = (layout.width * layout.height) as usize;
    let mut light = vec![[0.0f32; 3]; size];
    let mut rest = vec![[1.0f32; 2]; size];
    let mut filled = vec![false; size];
    for (i, s) in samples.list.iter().enumerate() {
        if !traced.valid[i] {
            continue;
        }

        let p = s.pixel as usize;
        light[p] = (traced.direct[i].bake + indirect[i]).to_array();
        rest[p] = [traced.direct[i].sun, traced.ao[i]];
        filled[p] = true;
    }

    let charts = chart_map(layout);
    let mut filled_rest = filled.clone();
    std::thread::scope(|s| {
        s.spawn(|| dilate(layout, &charts, &mut filled_rest, &mut rest));
        dilate(layout, &charts, &mut filled, &mut light);
    });
    (light, rest.iter().map(|r| r[0]).collect(), rest.iter().map(|r| r[1]).collect())
}
