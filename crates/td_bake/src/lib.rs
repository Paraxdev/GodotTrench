//! Light baking: ray traced light, shadow and ambient occlusion maps for the static geometry of a map. Every
//! receiving surface gets a chart in one atlas, the light arriving at each of its texels is traced against the whole
//! scene, and the result is ready for Godot's `LightmapGI`.
//!
//! Light values are in Godot's units: the number a surface's albedo is multiplied by, so a white floor under a sun of
//! energy 1 shining straight down reads 1.

mod atlas;
mod bvh;
mod filter;
mod raster;
mod sampling;
mod trace;

use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};

pub use atlas::{Chart, Layout, PAD};
pub use bvh::{Bvh, Hit, Node, Tri};
use glam::Vec3;
pub use raster::{Sample, Samples};

/// Identifies a baked surface across saves: the node it belongs to and the face within it (0 for a terrain).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SurfaceKey {
    pub node: u64,
    pub face: u32,
}

/// One piece of geometry: a brush face, a mesh face, a displacement or a terrain.
#[derive(Clone, Debug)]
pub struct Surface {
    pub key: SurfaceKey,
    pub positions: Vec<Vec3>,
    /// Shading normal per position.
    pub normals: Vec<Vec3>,
    /// Counter-clockwise seen from the side the surface faces.
    pub triangles: Vec<[u32; 3]>,
    /// Orthonormal axes in the surface's plane its chart is laid out along.
    pub axes: [Vec3; 2],
    pub material: u32,
    /// Gets a chart in the light map.
    pub receives: bool,
    /// Blocks light.
    pub casts: bool,
    /// Multiplies the texel size, from the texture's bake settings.
    pub texel_scale: f32,
}

impl Default for Surface {
    fn default() -> Self {
        Self {
            key: SurfaceKey::default(),
            positions: Vec::new(),
            normals: Vec::new(),
            triangles: Vec::new(),
            axes: [Vec3::X, Vec3::Y],
            material: 0,
            receives: true,
            casts: true,
            texel_scale: 1.0,
        }
    }
}

/// How a surface's material takes part in the bounce.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Material {
    /// Average linear albedo, times the tint.
    pub albedo: Vec3,
    /// Linear emission times its energy, which lights the scene like an area light.
    pub emission: Vec3,
    /// Both sides are surfaces. Rays meeting the back of a one sided surface are inside a solid.
    pub double_sided: bool,
}

impl Default for Material {
    fn default() -> Self {
        Self { albedo: Vec3::splat(0.5), emission: Vec3::ZERO, double_sided: false }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightKind {
    Sun,
    Omni,
    Spot,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Light {
    pub kind: LightKind,
    pub position: Vec3,
    /// Direction the light travels, for suns and spots.
    pub direction: Vec3,
    /// Linear color times energy.
    pub color: Vec3,
    /// Map units, suns have no range.
    pub range: f32,
    /// Exponent of the distance falloff, Godot's `omni_attenuation` and `spot_attenuation`.
    pub attenuation: f32,
    /// Cosine of a spot's cone half angle.
    pub spot_cos: f32,
    /// Godot's `spot_angle_attenuation`.
    pub spot_attenuation: f32,
    /// Radius in map units the light is spread over for soft shadows, the angular radius in radians for a sun.
    pub size: f32,
    /// The light's direct light goes into the light map, Godot's static bake mode. Otherwise Godot draws it in real
    /// time and only its bounce is baked.
    pub bake_direct: bool,
    /// Multiplies the light's share of the bounce, 0 leaves the light out of the bake.
    pub indirect_energy: f32,
    /// This sun's visibility is the shadow mask.
    pub shadow_mask: bool,
}

impl Light {
    pub fn sun(direction: Vec3, color: Vec3) -> Self {
        Self {
            kind: LightKind::Sun,
            position: Vec3::ZERO,
            direction: direction.normalize_or(Vec3::NEG_Y),
            color,
            range: 0.0,
            attenuation: 1.0,
            spot_cos: -1.0,
            spot_attenuation: 1.0,
            size: 0.0,
            bake_direct: true,
            indirect_energy: 1.0,
            shadow_mask: true,
        }
    }

    pub fn omni(position: Vec3, color: Vec3, range: f32) -> Self {
        Self { kind: LightKind::Omni, position, range, shadow_mask: false, ..Self::sun(Vec3::NEG_Y, color) }
    }
}

/// The environment rays that leave the map see: Godot's procedural sky blended with the ambient color.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sky {
    /// Linear sky colors, times the sky energy.
    pub top: Vec3,
    pub horizon: Vec3,
    pub ground: Vec3,
    /// Linear ambient color.
    pub ambient: Vec3,
    /// Share of the sky in the ambient light, Godot's `ambient_light_sky_contribution`.
    pub sky_share: f32,
    /// Godot's `ambient_light_energy`.
    pub energy: f32,
}

impl Default for Sky {
    fn default() -> Self {
        Self {
            top: Vec3::new(0.08, 0.2, 0.57),
            horizon: Vec3::new(0.48, 0.6, 0.74),
            ground: Vec3::new(0.15, 0.16, 0.17),
            ambient: Vec3::ZERO,
            sky_share: 1.0,
            energy: 1.0,
        }
    }
}

impl Sky {
    /// Black, for sealed interiors and tests.
    pub const NONE: Sky = Sky { top: Vec3::ZERO, horizon: Vec3::ZERO, ground: Vec3::ZERO, ambient: Vec3::ZERO, sky_share: 1.0, energy: 0.0 };

    /// Light arriving from `dir`, following `ProceduralSkyMaterial` with its default curves.
    pub fn radiance(&self, dir: Vec3) -> Vec3 {
        let angle = dir.y.clamp(-1.0, 1.0).acos();
        let half = std::f32::consts::FRAC_PI_2;
        let gradient = if angle <= half {
            let c = 1.0 - angle / half;
            self.horizon.lerp(self.top, (1.0 - (1.0 - c).powf(1.0 / 0.15)).clamp(0.0, 1.0))
        } else {
            let c = (angle - half) / half;
            self.horizon.lerp(self.ground, (1.0 - (1.0 - c).powf(1.0 / 0.02)).clamp(0.0, 1.0))
        };
        (gradient * self.sky_share + self.ambient * (1.0 - self.sky_share)) * self.energy
    }
}

#[derive(Clone, Debug, Default)]
pub struct BakeInput {
    pub surfaces: Vec<Surface>,
    pub materials: Vec<Material>,
    pub lights: Vec<Light>,
    pub sky: Sky,
    pub units_per_meter: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Backend {
    #[default]
    Cpu,
}

impl Backend {
    pub const ALL: [Backend; 1] = [Backend::Cpu];

    pub fn label(self) -> &'static str {
        match self {
            Backend::Cpu => "CPU",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// Map units per light map texel.
    pub texel_size: f32,
    /// Largest atlas side in texels. The texel size is coarsened when the map does not fit.
    pub max_size: u32,
    /// Rays per texel for each bounce.
    pub rays: u32,
    /// Times light bounces off surfaces, 0 for direct light and sky only.
    pub bounces: u32,
    /// Rays per light for soft shadows. Lights without a size take one.
    pub shadow_samples: u32,
    /// Occluders past this many map units no longer darken the ambient occlusion.
    pub ao_distance: f32,
    /// Smooths the noise of the bounce light within each chart.
    pub denoise: bool,
    pub backend: Backend,
    /// Worker threads for the CPU, 0 for one per core.
    pub threads: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self { texel_size: 16.0, max_size: 4096, rays: 128, bounces: 2, shadow_samples: 8, ao_distance: 64.0, denoise: true, backend: Backend::Cpu, threads: 0 }
    }
}

/// Where a running bake is, shared with the thread that shows it. Also carries the request to cancel.
#[derive(Default)]
pub struct Progress {
    stage: AtomicU8,
    /// Work done and total in the current stage, packed as done << 32 | total.
    work: AtomicU64,
    cancel: AtomicBool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Preparing,
    Direct,
    Bounce(u8),
    Finishing,
}

impl Stage {
    pub fn label(self) -> String {
        match self {
            Stage::Preparing => "Laying out the light map".into(),
            Stage::Direct => "Tracing direct light and shadows".into(),
            Stage::Bounce(0) => "Tracing sky light and ambient occlusion".into(),
            Stage::Bounce(k) => format!("Tracing bounce {k}"),
            Stage::Finishing => "Smoothing and filling the gutters".into(),
        }
    }
}

impl Progress {
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    pub fn stage(&self) -> Stage {
        match self.stage.load(Ordering::Relaxed) {
            0 => Stage::Preparing,
            1 => Stage::Direct,
            255 => Stage::Finishing,
            k => Stage::Bounce(k - 2),
        }
    }

    /// Done share of the current stage, 0 to 1.
    pub fn fraction(&self) -> f32 {
        let w = self.work.load(Ordering::Relaxed);
        let (done, total) = ((w >> 32) as f32, (w & 0xffff_ffff) as f32);
        if total > 0.0 { (done / total).min(1.0) } else { 0.0 }
    }

    fn start(&self, stage: Stage, total: usize) {
        let code = match stage {
            Stage::Preparing => 0,
            Stage::Direct => 1,
            Stage::Bounce(k) => k.saturating_add(2).min(254),
            Stage::Finishing => 255,
        };
        self.stage.store(code, Ordering::Relaxed);
        self.work.store(total.min(u32::MAX as usize) as u64, Ordering::Relaxed);
    }

    fn advance(&self, n: usize) {
        self.work.fetch_add((n as u64) << 32, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BakeError {
    Cancelled,
    /// Nothing in the map takes a light map.
    NothingToBake,
}

impl std::fmt::Display for BakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BakeError::Cancelled => f.write_str("the bake was cancelled"),
            BakeError::NothingToBake => f.write_str("nothing in the map receives baked light"),
        }
    }
}

/// Where one surface landed in the atlas.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChartEntry {
    pub key: SurfaceKey,
    /// Maps a position in map units to its 0..1 atlas coordinate, see [`Chart::rows`].
    pub rows: [[f32; 4]; 2],
}

/// The finished bake: three atlases of the same layout, one value per texel, row by row.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lightmap {
    pub width: u32,
    pub height: u32,
    /// Map units per texel the atlas was laid out at, coarser than asked when the map did not fit.
    pub texel_size: f32,
    /// Linear light in Godot's units: direct light of the baked lights, the sky and the bounce.
    pub light: Vec<[f32; 3]>,
    /// Share of the sun that reaches each texel, 1 in full sun.
    pub shadow: Vec<f32>,
    /// Ambient occlusion, 1 where nothing is near.
    pub ao: Vec<f32>,
    pub charts: Vec<ChartEntry>,
}

impl Lightmap {
    /// The chart of a surface.
    pub fn chart(&self, key: SurfaceKey) -> Option<&ChartEntry> {
        self.charts.binary_search_by_key(&key, |c| c.key).ok().map(|i| &self.charts[i])
    }
}

/// Bakes `input`. Runs on the calling thread and the workers it starts, check `progress` from another thread.
pub fn bake(input: &BakeInput, settings: &Settings, progress: &Progress) -> Result<Lightmap, BakeError> {
    progress.start(Stage::Preparing, 1);
    let layout = atlas::layout(&input.surfaces, settings).ok_or(BakeError::NothingToBake)?;
    let samples = raster::rasterize(&input.surfaces, &layout);
    let scene = trace::Scene::new(input, &layout, settings);
    let result = trace::run(&scene, &samples, settings, progress)?;
    progress.start(Stage::Finishing, 1);
    let (width, height) = (layout.width, layout.height);
    let mut charts: Vec<ChartEntry> = layout.charts.iter().map(|c| ChartEntry { key: input.surfaces[c.surface].key, rows: c.rows(width, height) }).collect();
    charts.sort_by_key(|c| c.key);
    let (light, shadow, ao) = filter::finish(&layout, &samples, &result, settings.denoise);
    progress.advance(1);
    Ok(Lightmap { width, height, texel_size: layout.texel, light, shadow, ao, charts })
}
