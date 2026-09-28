//! The GPU tracer: trace.wgsl run through wgpu compute on whatever adapter the system offers. It traces ranges of
//! samples on request, so the scheduler in trace.rs can hand the GPU big chunks while CPU threads take small ones.

use std::future::Future;
use std::ops::Range;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::atlas::PAD;
use crate::bvh::Tri;
use crate::raster::Samples;
use crate::trace::{Direct, Gather, INSIDE, Scene};
use crate::{LightKind, Settings};

/// Rays one dispatch may trace. Keeps each dispatch short, since a desktop driver resets a GPU that stays busy for a
/// couple of seconds.
const RAYS_PER_DISPATCH: u64 = 1 << 23;
const GROUP: u32 = 64;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Params {
    sky_top: [f32; 3],
    sky_energy: f32,
    sky_horizon: [f32; 3],
    sky_share: f32,
    sky_ground: [f32; 3],
    bias: f32,
    sky_ambient: [f32; 3],
    far: f32,
    offset: u32,
    count: u32,
    lights: u32,
    rays: u32,
    shadow_samples: u32,
    pass_index: u32,
    width: u32,
    height: u32,
    upm: f32,
    ao_distance: f32,
    inside: f32,
    tris: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuSurface {
    albedo: [f32; 3],
    chart: u32,
    emission: [f32; 3],
    double_sided: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuChart {
    axis0: [f32; 3],
    texel: f32,
    axis1: [f32; 3],
    pad: f32,
    min: [f32; 2],
    corner: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuSample {
    pos: [f32; 3],
    pad0: f32,
    normal: [f32; 3],
    pad1: f32,
    face_normal: [f32; 3],
    pad2: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GpuLight {
    position: [f32; 3],
    kind: u32,
    direction: [f32; 3],
    range: f32,
    color: [f32; 3],
    attenuation: f32,
    spot_cos: f32,
    spot_attenuation: f32,
    size: f32,
    indirect_energy: f32,
    flags: u32,
    pad: [u32; 3],
}

pub struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    direct: wgpu::ComputePipeline,
    gather: wgpu::ComputePipeline,
    bind: wgpu::BindGroup,
    params: wgpu::Buffer,
    prev: wgpu::Buffer,
    out: wgpu::Buffer,
    readback: wgpu::Buffer,
    base: Params,
    /// Samples one dispatch may hold at most, from the output buffer's size.
    capacity: usize,
    error: Arc<Mutex<Option<String>>>,
    /// The adapter's name, for the bake summary.
    pub name: String,
}

/// Runs a wgpu future to completion. Native wgpu resolves them without an event loop.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut cx = Context::from_waker(Waker::noop());
    let mut future = std::pin::pin!(future);
    loop {
        if let Poll::Ready(v) = future.as_mut().poll(&mut cx) {
            return v;
        }

        std::thread::yield_now();
    }
}

impl Gpu {
    /// Uploads the scene. Fails when there is no usable adapter or the driver refuses the tracer.
    pub fn new(scene: &Scene, samples: &Samples, settings: &Settings) -> Result<Gpu, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle_from_env());
        let adapter =
            block_on(instance.request_adapter(&wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, ..Default::default() }))
                .map_err(|e| format!("no GPU found ({e})"))?;
        let info = adapter.get_info();
        if info.device_type == wgpu::DeviceType::Cpu && !settings.software_gpu {
            return Err(format!("{} is a software renderer, slower than the CPU tracer", info.name));
        }

        let limits = adapter.limits();
        if limits.max_storage_buffers_per_shader_stage < 8 {
            return Err(format!("{} cannot bind the eight buffers the tracer needs", info.name));
        }

        let (device, queue) =
            block_on(adapter.request_device(&wgpu::DeviceDescriptor { label: Some("light bake"), required_limits: limits.clone(), ..Default::default() }))
                .map_err(|e| format!("{} refused a device ({e})", info.name))?;
        let error = Arc::new(Mutex::new(None::<String>));
        let slot = error.clone();
        device.on_uncaptured_error(Arc::new(move |e: wgpu::Error| {
            slot.lock().unwrap_or_else(|p| p.into_inner()).get_or_insert_with(|| e.to_string());
        }));

        let tris: Vec<Tri> = scene.bvh.tris.iter().map(|t| Tri { _pad1: scene.tri_surface[t.id as usize], ..*t }).collect();
        let surfaces: Vec<GpuSurface> = scene
            .input
            .surfaces
            .iter()
            .zip(&scene.surface_chart)
            .map(|(s, chart)| {
                let m = scene.input.materials.get(s.material as usize).copied().unwrap_or_default();
                GpuSurface {
                    albedo: m.albedo.to_array(),
                    chart: chart.unwrap_or(u32::MAX),
                    emission: m.emission.to_array(),
                    double_sided: u32::from(m.double_sided),
                }
            })
            .collect();
        let charts: Vec<GpuChart> = scene
            .layout
            .charts
            .iter()
            .map(|c| GpuChart {
                axis0: c.axes[0].to_array(),
                texel: c.texel,
                axis1: c.axes[1].to_array(),
                pad: 0.0,
                min: c.min.to_array(),
                corner: [c.x as f32 + PAD as f32 + 0.5, c.y as f32 + PAD as f32 + 0.5],
            })
            .collect();
        let gpu_samples: Vec<GpuSample> = samples
            .list
            .iter()
            .map(|s| GpuSample { pos: s.pos.to_array(), pad0: 0.0, normal: s.normal.to_array(), pad1: 0.0, face_normal: s.face_normal.to_array(), pad2: 0.0 })
            .collect();
        let lights: Vec<GpuLight> = scene
            .input
            .lights
            .iter()
            .map(|l| GpuLight {
                position: l.position.to_array(),
                kind: match l.kind {
                    LightKind::Sun => 0,
                    LightKind::Omni => 1,
                    LightKind::Spot => 2,
                },
                direction: l.direction.to_array(),
                range: l.range,
                color: l.color.to_array(),
                attenuation: l.attenuation,
                spot_cos: l.spot_cos,
                spot_attenuation: l.spot_attenuation,
                size: l.size,
                indirect_energy: l.indirect_energy,
                flags: u32::from(l.bake_direct) | u32::from(l.shadow_mask) << 1,
                pad: [0; 3],
            })
            .collect();

        let texels = (scene.layout.width * scene.layout.height) as u64;
        let max_binding = limits.max_storage_buffer_binding_size;
        if texels * 8 > max_binding || gpu_samples.len() as u64 * 48 > max_binding || scene.bvh.nodes.len() as u64 * 32 > max_binding {
            return Err(format!("the map is too big for the buffers {} allows", info.name));
        }

        let storage = |label: &str, bytes: &[u8]| {
            // wgpu refuses empty bindings, a zero row stands in for an empty list.
            let contents: &[u8] = if bytes.is_empty() { &[0u8; 64] } else { bytes };
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some(label), contents, usage: wgpu::BufferUsages::STORAGE })
        };
        let nodes = storage("bake nodes", bytemuck::cast_slice(&scene.bvh.nodes));
        let tris = storage("bake triangles", bytemuck::cast_slice(&tris));
        let surfaces = storage("bake surfaces", bytemuck::cast_slice(&surfaces));
        let charts = storage("bake charts", bytemuck::cast_slice(&charts));
        let gpu_samples_buf = storage("bake samples", bytemuck::cast_slice(&gpu_samples));
        let lights_buf = storage("bake lights", bytemuck::cast_slice(&lights));
        let prev = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bake bounce source"),
            size: (texels * 8).max(64),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let capacity = ((max_binding / 32).min(1 << 20) as usize).min(samples.list.len().max(1));
        let out_size = (capacity * 32) as u64;
        let out = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bake results"),
            size: out_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bake readback"),
            size: out_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bake params"),
            size: std::mem::size_of::<Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let entry = |binding: u32, ty: wgpu::BufferBindingType| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { ty, has_dynamic_offset: false, min_binding_size: None },
            count: None,
        };
        let read = wgpu::BufferBindingType::Storage { read_only: true };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bake layout"),
            entries: &[
                entry(0, wgpu::BufferBindingType::Uniform),
                entry(1, read),
                entry(2, read),
                entry(3, read),
                entry(4, read),
                entry(5, read),
                entry(6, read),
                entry(7, read),
                entry(8, wgpu::BufferBindingType::Storage { read_only: false }),
            ],
        });
        let buffers = [&params, &nodes, &tris, &surfaces, &charts, &gpu_samples_buf, &lights_buf, &prev, &out];
        let entries: Vec<wgpu::BindGroupEntry> =
            buffers.iter().enumerate().map(|(i, b)| wgpu::BindGroupEntry { binding: i as u32, resource: b.as_entire_binding() }).collect();
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("bake bindings"), layout: &layout, entries: &entries });
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bake tracer"),
            source: wgpu::ShaderSource::Wgsl(include_str!("trace.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bake pipeline"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = |entry: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(&pipeline_layout),
                module: &module,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        };
        let direct = pipeline("direct");
        let gather = pipeline("gather");
        let sky = &scene.input.sky;
        let base = Params {
            sky_top: sky.top.to_array(),
            sky_energy: sky.energy,
            sky_horizon: sky.horizon.to_array(),
            sky_share: sky.sky_share,
            sky_ground: sky.ground.to_array(),
            bias: scene.bias,
            sky_ambient: sky.ambient.to_array(),
            far: scene.far,
            offset: 0,
            count: 0,
            lights: lights.len() as u32,
            rays: settings.rays.max(1),
            shadow_samples: settings.shadow_samples.max(1),
            pass_index: 0,
            width: scene.layout.width,
            height: scene.layout.height,
            upm: scene.input.units_per_meter,
            ao_distance: settings.ao_distance,
            inside: INSIDE,
            tris: scene.bvh.tris.len() as u32,
        };
        let gpu = Gpu { device, queue, direct, gather, bind, params, prev, out, readback, base, capacity, error, name: info.name.clone() };
        gpu.check()?;
        Ok(gpu)
    }

    fn check(&self) -> Result<(), String> {
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        match self.error.lock().unwrap_or_else(|p| p.into_inner()).take() {
            Some(e) => Err(format!("{}: {e}", self.name)),
            None => Ok(()),
        }
    }

    /// Samples worth giving the GPU at once for kernels tracing `rays` rays per sample.
    pub fn chunk(&self, rays: u32) -> usize {
        ((RAYS_PER_DISPATCH / rays.max(1) as u64) as usize).clamp(GROUP as usize, self.capacity)
    }

    /// Runs `kernel` over `range` in dispatches the output buffer holds, `per` rows of four floats per sample.
    fn run(&self, kernel: &wgpu::ComputePipeline, range: Range<usize>, pass: u32, per: usize, rays: u32) -> Result<Vec<[f32; 4]>, String> {
        let mut rows = Vec::with_capacity(range.len() * per);
        let step = self.chunk(rays).min(self.capacity * 2 / per).max(1);
        let mut start = range.start;
        while start < range.end {
            let count = step.min(range.end - start);
            let params = Params { offset: start as u32, count: count as u32, pass_index: pass, ..self.base };
            self.queue.write_buffer(&self.params, 0, bytemuck::bytes_of(&params));
            let bytes = (count * per * 16) as u64;
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("bake trace") });
            {
                let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("bake trace"), timestamp_writes: None });
                pass.set_pipeline(kernel);
                pass.set_bind_group(0, &self.bind, &[]);
                pass.dispatch_workgroups((count as u32).div_ceil(GROUP), 1, 1);
            }

            encoder.copy_buffer_to_buffer(&self.out, 0, &self.readback, 0, bytes);
            self.queue.submit([encoder.finish()]);
            let slice = self.readback.slice(..bytes);
            slice.map_async(wgpu::MapMode::Read, |_| {});
            self.check()?;
            {
                let data = slice.get_mapped_range().map_err(|e| format!("{}: {e}", self.name))?;
                rows.extend_from_slice(bytemuck::cast_slice::<u8, [f32; 4]>(&data));
            }

            self.readback.unmap();
            start += count;
        }

        Ok(rows)
    }

    pub fn direct(&self, range: Range<usize>) -> Result<Vec<Direct>, String> {
        let rays = self.base.lights.max(1) * self.base.shadow_samples;
        let rows = self.run(&self.direct, range, 0, 2, rays)?;
        Ok(rows
            .as_chunks::<2>()
            .0
            .iter()
            .map(|r| Direct { bake: Vec3::new(r[0][0], r[0][1], r[0][2]), sun: r[0][3], bounce: Vec3::new(r[1][0], r[1][1], r[1][2]) })
            .collect())
    }

    /// Sets the light each texel sends on for the next gathers.
    pub fn set_source(&self, source: &[Vec3]) {
        let packed: Vec<[u32; 2]> = source
            .iter()
            .map(|c| {
                let h = c.to_array().map(|v| half(v.clamp(0.0, 65000.0)) as u32);
                [h[0] | h[1] << 16, h[2]]
            })
            .collect();
        if !packed.is_empty() {
            self.queue.write_buffer(&self.prev, 0, bytemuck::cast_slice(&packed));
        }
    }

    pub fn gather(&self, range: Range<usize>, pass: u32) -> Result<Vec<Gather>, String> {
        let rows = self.run(&self.gather, range, pass, 1, self.base.rays)?;
        Ok(rows
            .iter()
            .map(|r| {
                let inside = r[3] < 0.0;
                Gather { indirect: Vec3::new(r[0], r[1], r[2]), ao: if inside { -r[3] - 1.0 } else { r[3] }, inside }
            })
            .collect())
    }
}

/// Half float bits of `f`, rounded to nearest, for values in the half range.
fn half(f: f32) -> u16 {
    let x = f.to_bits();
    let sign = ((x >> 16) & 0x8000) as u16;
    let exp = ((x >> 23) & 0xff) as i32 - 127 + 15;
    let man = x & 0x7f_ffff;
    if exp <= 0 {
        if exp < -10 {
            return sign;
        }

        let m = (man | 0x80_0000) >> (1 - exp);
        return sign | ((m >> 13) + ((m >> 12) & 1)) as u16;
    }

    if exp >= 0x1f {
        return sign | 0x7bff;
    }

    let h = (sign as u32 | (exp as u32) << 10 | man >> 13) + ((man >> 12) & 1);
    h as u16
}
