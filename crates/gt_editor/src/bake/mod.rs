//! Godot > Bake Lighting: traces the light, shadow and ambient occlusion maps of the static geometry on a worker
//! thread and stores them in the map, where the Baked view mode shows them and the Godot addon builds them into a
//! LightmapGI.

pub mod collect;
pub mod dialog;

use std::sync::Arc;
use std::time::Instant;

use gt_bake::{Backend, Progress, Settings};
use gt_doc::lightmap::{self, BakeOptions, Lightmap};
use gt_doc::{Map, NodeId};

use crate::state::EditorState;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Quality {
    /// Quick look while laying out lights.
    Preview,
    #[default]
    Medium,
    High,
    /// For shipping, slow.
    Final,
}

impl Quality {
    pub const ALL: [Quality; 4] = [Quality::Preview, Quality::Medium, Quality::High, Quality::Final];

    pub fn label(self) -> &'static str {
        match self {
            Quality::Preview => "Preview",
            Quality::Medium => "Medium",
            Quality::High => "High",
            Quality::Final => "Final",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Quality::Preview => "preview",
            Quality::Medium => "medium",
            Quality::High => "high",
            Quality::Final => "final",
        }
    }

    pub fn from_name(name: &str) -> Option<Quality> {
        Self::ALL.into_iter().find(|q| q.name().eq_ignore_ascii_case(name.trim()))
    }

    pub fn hint(self) -> &'static str {
        match self {
            Quality::Preview => "Few rays and one bounce: noisy, but done in moments while you place lights",
            Quality::Medium => "Two bounces with enough rays for smooth indirect light",
            Quality::High => "Three bounces and more rays, for a clean result",
            Quality::Final => "Four bounces with many rays, for the build you ship. Slow on big maps",
        }
    }

    /// Rays per texel, bounces and shadow rays per light.
    fn tracing(self) -> (u32, u32, u32) {
        match self {
            Quality::Preview => (32, 1, 4),
            Quality::Medium => (96, 2, 8),
            Quality::High => (192, 3, 12),
            Quality::Final => (384, 4, 16),
        }
    }
}

pub fn backend_name(b: Backend) -> &'static str {
    match b {
        Backend::Cpu => "cpu",
        Backend::Gpu => "gpu",
        Backend::Hybrid => "hybrid",
    }
}

pub fn backend_from_name(name: &str) -> Backend {
    Backend::ALL.into_iter().find(|b| backend_name(*b) == name.trim()).unwrap_or_default()
}

/// What the dialog asks for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Options {
    pub quality: Quality,
    /// Map units per light map texel.
    pub texel_size: f32,
    /// 0 for hard shadows, 1 for the softest.
    pub softness: f32,
    pub backend: Backend,
}

pub const DEFAULT_TEXEL: f32 = 16.0;
pub const TEXEL_RANGE: std::ops::RangeInclusive<f32> = 2.0..=128.0;

impl Default for Options {
    fn default() -> Self {
        Self { quality: Quality::Medium, texel_size: DEFAULT_TEXEL, softness: lightmap::DEFAULT_SOFTNESS as f32, backend: Backend::Cpu }
    }
}

impl Options {
    /// The options a map was last baked with, or the defaults.
    pub fn of(map: &Map) -> Options {
        let Some(o) = &map.editor.bake else { return Options::default() };
        let d = Options::default();
        Options {
            quality: Quality::from_name(&o.quality).unwrap_or(d.quality),
            texel_size: if o.texel_size > 0.0 { (o.texel_size as f32).clamp(*TEXEL_RANGE.start(), *TEXEL_RANGE.end()) } else { d.texel_size },
            softness: (o.softness as f32).clamp(0.0, 1.0),
            backend: backend_from_name(&o.backend),
        }
    }

    pub fn stored(&self) -> BakeOptions {
        BakeOptions {
            quality: self.quality.name().into(),
            texel_size: self.texel_size as f64,
            softness: self.softness as f64,
            backend: backend_name(self.backend).into(),
        }
    }

    pub fn settings(&self) -> Settings {
        let (rays, bounces, shadow_samples) = self.quality.tracing();
        // Every core: the tracer threads only share out work, the editor's own thread still gets its turn.
        Settings { texel_size: self.texel_size, rays, bounces, shadow_samples, backend: self.backend, threads: 0, ..Settings::default() }
    }
}

/// A finished bake, ready to store in the map.
pub struct Baked {
    pub lightmap: Lightmap,
    pub counts: collect::Counts,
    pub seconds: f32,
    /// The texel size asked for, when the map had to be baked coarser to fit the largest atlas.
    pub coarsened_from: Option<f32>,
    /// The GPU that took part.
    pub gpu: Option<String>,
    /// Why the GPU asked for was not used.
    pub gpu_error: Option<String>,
}

impl Baked {
    pub fn summary(&self) -> String {
        let lm = &self.lightmap;
        let mut text = format!("Baked lighting in {:.1} s: {} surfaces in a {}×{} light map", self.seconds, self.counts.surfaces, lm.width, lm.height);
        if let Some(gpu) = &self.gpu {
            text += &format!(" on {gpu}");
        }

        if !lm.probes.is_empty() {
            text += &format!(" and {} light probes", lm.probes.len());
        }

        if let Some(asked) = self.coarsened_from {
            text += &format!(", at {:.0} units per texel instead of {asked:.0} so it fits", lm.texel_size);
        }

        if let Some(e) = &self.gpu_error {
            text += &format!(". The GPU was not used, {e}, so the CPU baked it");
        }

        text
    }
}

pub struct Job {
    pub progress: Arc<Progress>,
    pub options: Options,
    pub started: Instant,
    thread: Option<std::thread::JoinHandle<Result<Baked, String>>>,
}

impl Job {
    /// Bakes the map as it is now. Edits made while it runs do not go into the bake.
    pub fn start(state: &EditorState, options: Options) -> Job {
        let collected = collect::collect(&state.doc.map, &state.game, options.softness);
        let mut materials = state.materials.detached();
        let progress = Arc::new(Progress::default());
        let shared = progress.clone();
        let settings = options.settings();
        let thread = std::thread::Builder::new()
            .name("light bake".into())
            .spawn(move || {
                let started = Instant::now();
                let mut input = collected.input;
                input.materials = collected.material_names.iter().map(|name| collect::material(&mut materials, name)).collect();
                let map = gt_bake::bake(&input, &settings, &shared).map_err(|e| e.to_string())?;
                let coarsened_from = (map.texel_size > settings.texel_size * 1.01).then_some(settings.texel_size);
                let (gpu, gpu_error) = (map.gpu.clone(), map.gpu_error.clone());
                Ok(Baked {
                    lightmap: stored(map, collected.nodes, collected.scene),
                    counts: collected.counts,
                    seconds: started.elapsed().as_secs_f32(),
                    coarsened_from,
                    gpu,
                    gpu_error,
                })
            })
            .expect("the bake thread starts");
        Job { progress, options, started: Instant::now(), thread: Some(thread) }
    }

    pub fn finished(&mut self) -> Option<Result<Baked, String>> {
        if !self.thread.as_ref().is_some_and(|t| t.is_finished()) {
            return None;
        }

        let outcome = self.thread.take()?.join();
        Some(outcome.unwrap_or_else(|_| Err("the bake stopped on an internal error".into())))
    }

    /// Bakes on the calling thread, for scripts and tests.
    pub fn run_blocking(state: &EditorState, options: Options) -> Result<Baked, String> {
        let mut job = Job::start(state, options);
        let thread = job.thread.take().expect("a started job has its thread");
        thread.join().unwrap_or_else(|_| Err("the bake stopped on an internal error".into()))
    }
}

/// The bake in the form the map stores.
fn stored(map: gt_bake::Lightmap, nodes: std::collections::BTreeMap<u64, u64>, scene: u64) -> Lightmap {
    let unit = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    Lightmap {
        width: map.width,
        height: map.height,
        texel_size: map.texel_size,
        light: map.light.iter().flat_map(|c| c.map(|v| lightmap::f32_to_f16(v.clamp(0.0, 60000.0)))).collect(),
        shadow: map.shadow.iter().map(|v| unit(*v)).collect(),
        ao: map.ao.iter().map(|v| unit(*v)).collect(),
        charts: map.charts.iter().map(|c| lightmap::Chart { node: c.key.node, face: c.key.face, rows: c.rows }).collect(),
        nodes,
        scene,
        probes: lightmap::Probes {
            points: map.probes.points.iter().flat_map(|p| p.to_array()).collect(),
            sh: map.probes.sh.iter().flat_map(|c| c.iter().flat_map(|v| v.to_array())).collect(),
            tetrahedra: map.probes.tetrahedra.iter().flatten().map(|i| *i as i32).collect(),
            bsp_planes: map.probes.bsp.iter().flat_map(|n| n.plane).collect(),
            bsp_children: map.probes.bsp.iter().flat_map(|n| [n.over, n.under]).collect(),
        },
    }
}

/// Stores a finished bake in the map as one undo step, with the options it was made with.
pub fn apply(state: &mut EditorState, baked: Baked, options: Options) {
    let summary = baked.summary();
    let lightmap = Arc::new(baked.lightmap);
    state.doc.edit("Bake Lighting", |m, _| {
        m.lightmap = Some(lightmap);
        m.editor.bake = Some(options.stored());
    });
    state.set_status(summary);
}

/// Whether the map changed in a way the bake depends on since it was made.
pub fn is_out_of_date(map: &Map, game: &gt_formats::GameConfig) -> bool {
    let Some(lm) = &map.lightmap else { return false };
    let options = Options::of(map);
    collect::collect(map, game, options.softness).scene != lm.scene
}

/// Which map the Baked view shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum View {
    /// The textures lit by the light map.
    #[default]
    Lit,
    /// The light map alone.
    Light,
    Shadow,
    Occlusion,
}

impl View {
    pub const ALL: [View; 4] = [View::Lit, View::Light, View::Shadow, View::Occlusion];

    pub fn name(self) -> &'static str {
        match self {
            View::Lit => "lit",
            View::Light => "light",
            View::Shadow => "shadow",
            View::Occlusion => "occlusion",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            View::Lit => "Textures with Baked Light",
            View::Light => "Light Map Only",
            View::Shadow => "Sun Shadow Mask",
            View::Occlusion => "Ambient Occlusion",
        }
    }
}

/// Texels for the renderer: red, green, blue and alpha half floats of the chosen map.
pub fn preview_texels(lm: &Lightmap, view: View) -> Vec<u16> {
    let one = lightmap::f32_to_f16(1.0);
    let gray = |v: u8| {
        let h = lightmap::f32_to_f16(v as f32 / 255.0);
        [h, h, h, one]
    };
    match view {
        View::Lit | View::Light => lm.light.as_chunks::<3>().0.iter().flat_map(|c| [c[0], c[1], c[2], one]).collect(),
        View::Shadow => lm.shadow.iter().flat_map(|v| gray(*v)).collect(),
        View::Occlusion => lm.ao.iter().flat_map(|v| gray(*v)).collect(),
    }
}

/// Light map coordinates of a node's faces, when it is baked and unchanged since.
pub fn charts_of(map: &Map, id: NodeId) -> std::collections::HashMap<u32, [[f32; 4]; 2]> {
    let Some(lm) = &map.lightmap else { return Default::default() };
    let Some(node) = map.get(id) else { return Default::default() };
    let Some(print) = collect::fingerprint(&node.kind) else { return Default::default() };
    lm.charts_of(id.0, print).iter().map(|c| (c.face, c.rows)).collect()
}

/// A position's light map coordinate under `rows`.
pub fn uv2(rows: &[[f32; 4]; 2], p: gt_core::DVec3) -> [f32; 2] {
    let p = p.as_vec3();
    let row = |r: &[f32; 4]| r[0] * p.x + r[1] * p.y + r[2] * p.z + r[3];
    [row(&rows[0]), row(&rows[1])]
}
