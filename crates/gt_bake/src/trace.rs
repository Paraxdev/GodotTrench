//! The CPU tracer: direct light with shadows from every light, then gather passes that bring in the sky, glowing
//! surfaces and light bounced off the rest of the map.

use std::sync::Mutex;

use glam::{Vec2, Vec3};

use crate::atlas::Layout;
use crate::bvh::Bvh;
use crate::raster::{Sample, Samples};
use crate::sampling::{cone, cosine, hash, r2};
use crate::{BakeError, BakeInput, Light, LightKind, Progress, Settings, Stage, filter};

/// Samples handed to a worker at a time.
const CHUNK: usize = 256;
/// Share of a texel's rays meeting the back of a one sided surface past which the texel counts as inside a solid.
const INSIDE: f32 = 0.15;

pub struct Scene<'a> {
    pub input: &'a BakeInput,
    pub layout: &'a Layout,
    pub bvh: Bvh,
    /// Surface of every traced triangle.
    pub tri_surface: Vec<u32>,
    /// Chart of every surface that receives light.
    pub surface_chart: Vec<Option<u32>>,
    /// Map units rays start off their surface, so they do not hit it again.
    pub bias: f32,
    pub far: f32,
    /// Corners of the box around every traced triangle.
    pub bounds: (Vec3, Vec3),
}

impl<'a> Scene<'a> {
    pub fn new(input: &'a BakeInput, layout: &'a Layout, settings: &Settings) -> Self {
        let mut tris = Vec::new();
        let mut tri_surface = Vec::new();
        let (mut lo, mut hi) = (Vec3::splat(f32::MAX), Vec3::splat(f32::MIN));
        for (si, s) in input.surfaces.iter().enumerate() {
            if !s.casts {
                continue;
            }

            for t in &s.triangles {
                let [Some(a), Some(b), Some(c)] = t.map(|i| s.positions.get(i as usize).copied()) else { continue };
                tris.push([a, b, c]);
                tri_surface.push(si as u32);
                lo = lo.min(a.min(b).min(c));
                hi = hi.max(a.max(b).max(c));
            }
        }

        let mut surface_chart = vec![None; input.surfaces.len()];
        for (ci, c) in layout.charts.iter().enumerate() {
            surface_chart[c.surface] = Some(ci as u32);
        }

        let far = if tris.is_empty() { 1.0 } else { (hi - lo).length() * 2.0 + 1.0 };
        let bias = 0.05 + settings.texel_size.max(0.0) * 0.01;
        Scene { input, layout, bvh: Bvh::build(&tris), tri_surface, surface_chart, bias, far, bounds: (lo, hi) }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Direct {
    /// Direct light of the lights baked in full.
    pub bake: Vec3,
    /// Direct light of every light that bounces, weighted by its indirect energy.
    pub bounce: Vec3,
    pub sun: f32,
}

#[derive(Clone, Copy, Default)]
pub struct Gather {
    pub indirect: Vec3,
    pub ao: f32,
    pub inside: bool,
}

/// Everything traced, one entry per sample.
pub struct Traced {
    pub direct: Vec<Direct>,
    pub indirect: Vec<Vec3>,
    pub ao: Vec<f32>,
    /// False for samples inside solids, their texels are filled from their neighbours.
    pub valid: Vec<bool>,
}

pub fn run(scene: &Scene, samples: &Samples, settings: &Settings, progress: &Progress) -> Result<Traced, BakeError> {
    let n = samples.list.len();
    let threads = match settings.threads {
        0 => std::thread::available_parallelism().map_or(4, |p| p.get()),
        t => t,
    };
    progress.start(Stage::Direct, n);
    let direct = parallel(n, threads, progress, |i| direct(scene, &samples.list[i], i as u32, settings))?;
    let passes = settings.bounces.max(1);
    let mut prev = if settings.bounces > 0 {
        filter::spread(scene.layout, samples, |i| Some(direct[i].bounce))
    } else {
        vec![Vec3::ZERO; (scene.layout.width * scene.layout.height) as usize]
    };
    let mut valid = vec![true; n];
    let mut ao = vec![1.0; n];
    let mut indirect = vec![Vec3::ZERO; n];
    for pass in 0..passes {
        let label = if settings.bounces == 0 { 0 } else { pass + 1 };
        progress.start(Stage::Bounce(label.min(250) as u8), n);
        let g = parallel(n, threads, progress, |i| gather(scene, &samples.list[i], i as u32, pass, &prev, settings))?;
        if pass == 0 {
            for (i, g) in g.iter().enumerate() {
                valid[i] = !g.inside;
                ao[i] = g.ao;
            }
        }

        for (i, g) in g.iter().enumerate() {
            indirect[i] = g.indirect;
        }

        if pass + 1 < passes {
            prev = filter::spread(scene.layout, samples, |i| valid[i].then(|| direct[i].bounce + indirect[i]));
        }
    }

    Ok(Traced { direct, indirect, ao, valid })
}

/// Runs `f` for every index on `threads` workers, in chunks so a cancel stops them quickly.
pub(crate) fn parallel<T: Send + Default + Clone>(n: usize, threads: usize, progress: &Progress, f: impl Fn(usize) -> T + Sync) -> Result<Vec<T>, BakeError> {
    let mut out = vec![T::default(); n];
    {
        let chunks = Mutex::new(out.chunks_mut(CHUNK).enumerate());
        std::thread::scope(|s| {
            for _ in 0..threads.max(1) {
                s.spawn(|| {
                    loop {
                        if progress.is_cancelled() {
                            return;
                        }

                        let Some((ci, chunk)) = chunks.lock().unwrap_or_else(|e| e.into_inner()).next() else { return };
                        for (k, slot) in chunk.iter_mut().enumerate() {
                            *slot = f(ci * CHUNK + k);
                        }

                        progress.advance(chunk.len());
                    }
                });
            }
        });
    }

    if progress.is_cancelled() { Err(BakeError::Cancelled) } else { Ok(out) }
}

/// Godot's light falloff, distances in meters.
fn falloff(distance: f32, range: f32, decay: f32) -> f32 {
    let nd = (distance / range.max(1e-4)).powi(4);
    let window = (1.0 - nd).max(0.0);
    window * window * distance.max(1e-4).powf(-decay)
}

/// Unshadowed light of `l` at `p` with normal `n`, and the direction towards it.
fn light_at(l: &Light, p: Vec3, n: Vec3, upm: f32) -> Option<(Vec3, Vec3)> {
    match l.kind {
        LightKind::Sun => {
            let to = -l.direction;
            let ndl = n.dot(to);
            (ndl > 0.0).then_some((l.color * ndl, to))
        }
        LightKind::Omni | LightKind::Spot => {
            let d = l.position - p;
            let dist = d.length();
            if dist >= l.range || dist <= 1e-4 {
                return None;
            }

            let to = d / dist;
            let ndl = n.dot(to);
            if ndl <= 0.0 {
                return None;
            }

            let mut att = falloff(dist / upm, l.range / upm, l.attenuation);
            if l.kind == LightKind::Spot {
                let cos = l.spot_cos.clamp(-1.0, 0.9999);
                let scos = (-to).dot(l.direction).max(cos);
                let rim = ((1.0 - scos) / (1.0 - cos)).max(1e-4);
                att *= 1.0 - rim.powf(l.spot_attenuation);
            }

            (att > 0.0).then_some((l.color * att * ndl, to))
        }
    }
}

fn direct(scene: &Scene, s: &Sample, index: u32, settings: &Settings) -> Direct {
    let upm = scene.input.units_per_meter.max(1e-3);
    let origin = s.pos + s.face_normal * scene.bias;
    let mut out = Direct { sun: 1.0, ..Direct::default() };
    for (li, l) in scene.input.lights.iter().enumerate() {
        if !l.bake_direct && l.indirect_energy <= 0.0 && !l.shadow_mask {
            continue;
        }

        let Some((light, to)) = light_at(l, s.pos, s.normal, upm) else {
            if l.shadow_mask {
                out.sun = 0.0;
            }

            continue;
        };
        let count = if l.size > 0.0 { settings.shadow_samples.max(1) } else { 1 };
        let seed = hash(index.wrapping_mul(31).wrapping_add(li as u32 * 7919));
        let mut lit = 0;
        for k in 0..count {
            let u = if count > 1 { r2(seed, k) } else { Vec2::splat(0.5) };
            let blocked = match l.kind {
                LightKind::Sun => scene.bvh.occluded(origin, cone(to, l.size, u), 0.0, scene.far),
                _ => {
                    let target = if l.size > 0.0 { l.position + sphere(u, seed ^ k) * l.size } else { l.position };
                    let d = target - origin;
                    let len = d.length();
                    len > 1e-3 && scene.bvh.occluded(origin, d / len, 0.0, len - scene.bias)
                }
            };
            lit += u32::from(!blocked);
        }

        let visible = lit as f32 / count as f32;
        if l.bake_direct {
            out.bake += light * visible;
        }

        out.bounce += light * visible * l.indirect_energy;
        if l.shadow_mask {
            out.sun = visible;
        }
    }

    out
}

/// A point in the unit ball, for spreading a light over its size.
fn sphere(u: Vec2, seed: u32) -> Vec3 {
    let z = u.x * 2.0 - 1.0;
    let r = (1.0 - z * z).max(0.0).sqrt();
    let phi = std::f32::consts::TAU * u.y;
    let radius = ((hash(seed) >> 8) as f32 / (1u32 << 24) as f32).cbrt();
    Vec3::new(r * phi.cos(), r * phi.sin(), z) * radius
}

fn gather(scene: &Scene, s: &Sample, index: u32, pass: u32, prev: &[Vec3], settings: &Settings) -> Gather {
    let rays = settings.rays.max(1);
    let seed = hash(index ^ pass.wrapping_mul(0x68e3_1da4));
    let origin = s.pos + s.face_normal * scene.bias;
    let (mut sum, mut open, mut backs) = (Vec3::ZERO, 0.0f32, 0u32);
    let ao_distance = settings.ao_distance.max(1e-3);
    for k in 0..rays {
        let mut dir = cosine(s.normal, r2(seed, k));
        let below = dir.dot(s.face_normal);
        if below <= 1e-3 {
            dir = (dir + s.face_normal * (2e-3 - below)).normalize();
        }

        let Some(hit) = scene.bvh.closest(origin, dir, 0.0, scene.far) else {
            sum += scene.input.sky.radiance(dir);
            open += 1.0;
            continue;
        };
        let si = scene.tri_surface[hit.tri as usize] as usize;
        let surface = &scene.input.surfaces[si];
        let material = scene.input.materials.get(surface.material as usize).copied().unwrap_or_default();
        open += (hit.t / ao_distance).clamp(0.0, 1.0);
        if !hit.front && !material.double_sided {
            backs += 1;
            continue;
        }

        let p = origin + dir * hit.t;
        let arriving = match scene.surface_chart[si] {
            Some(c) => bilinear(prev, scene.layout.width, scene.layout.height, scene.layout.charts[c as usize].pixel(p)),
            None => Vec3::ZERO,
        };
        sum += material.emission + material.albedo * arriving;
    }

    let n = rays as f32;
    Gather { indirect: sum / n, ao: open / n, inside: backs as f32 > n * INSIDE }
}

pub fn bilinear(image: &[Vec3], width: u32, height: u32, px: Vec2) -> Vec3 {
    let p = px - Vec2::splat(0.5);
    let (x0, y0) = (p.x.floor(), p.y.floor());
    let (fx, fy) = (p.x - x0, p.y - y0);
    let at = |x: f32, y: f32| {
        let x = (x as i64).clamp(0, width as i64 - 1) as usize;
        let y = (y as i64).clamp(0, height as i64 - 1) as usize;
        image[y * width as usize + x]
    };
    let top = at(x0, y0).lerp(at(x0 + 1.0, y0), fx);
    let bottom = at(x0, y0 + 1.0).lerp(at(x0 + 1.0, y0 + 1.0), fx);
    top.lerp(bottom, fy)
}
