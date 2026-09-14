use std::collections::HashMap;

pub use egui_wgpu::wgpu;
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

pub const MSAA_SAMPLES: u32 = 4;
const COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const SHADOW_SIZE: u32 = 4096;
pub const MAX_LIGHTS: usize = 64;

pub const WHITE_MATERIAL: &str = "__white";
pub const MISSING_MATERIAL: &str = "__missing";

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LineVertex {
    pub pos: [f32; 3],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    eye: [f32; 4],
    params: [f32; 4],
    viewport: [f32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadeMode {
    Textured,
    Flat,
    /// Textured with scene lights, sun shadows, sky and fog.
    Lit,
    /// Edges only.
    Wireframe,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AlphaMode {
    Opaque,
    Blend,
    Scissor(f32),
}

/// Everything needed to preview a material. Colors are linear.
#[derive(Clone, Debug)]
pub struct MaterialDesc<'a> {
    pub albedo: &'a image::RgbaImage,
    pub normal: Option<&'a image::RgbaImage>,
    pub emission_texture: Option<&'a image::RgbaImage>,
    pub tint: [f32; 4],
    pub emission: [f32; 3],
    pub alpha: AlphaMode,
    pub nearest: bool,
    pub unshaded: bool,
    pub double_sided: bool,
    pub normal_scale: f32,
}

impl<'a> MaterialDesc<'a> {
    pub fn plain(albedo: &'a image::RgbaImage, nearest: bool) -> Self {
        Self {
            albedo,
            normal: None,
            emission_texture: None,
            tint: [1.0; 4],
            emission: [0.0; 3],
            alpha: AlphaMode::Opaque,
            nearest,
            unshaded: false,
            double_sided: false,
            normal_scale: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MaterialFlags {
    pub transparent: bool,
    pub double_sided: bool,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MaterialUniform {
    tint: [f32; 4],
    emission: [f32; 4],
    flags: [f32; 4],
    extra: [f32; 4],
}

pub fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
}

#[repr(C)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    pos_range: [f32; 4],
    color_energy: [f32; 4],
    dir_cone: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LightsUniform {
    sun_dir: [f32; 4],
    sun_color: [f32; 4],
    ambient: [f32; 4],
    sky_top: [f32; 4],
    sky_horizon: [f32; 4],
    sky_ground: [f32; 4],
    fog: [f32; 4],
    shadow_view_proj: [[f32; 4]; 4],
    lights: [LightUniform; MAX_LIGHTS],
}

#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub position: Vec3,
    pub range: f32,
    pub color: Vec3,
    pub energy: f32,
    /// Direction and cosine of the cone half angle for spot lights.
    pub spot: Option<(Vec3, f32)>,
}

#[derive(Clone, Debug)]
pub struct Lighting {
    /// Direction the sun light travels.
    pub sun_direction: Vec3,
    pub sun_color: Vec3,
    pub sun_energy: f32,
    pub ambient: Vec3,
    pub lights: Vec<PointLight>,
    /// Linear sky gradient colors.
    pub sky_top: Vec3,
    pub sky_horizon: Vec3,
    pub sky_ground: Vec3,
    pub fog_color: Vec3,
    /// Exponential fog density per world unit, 0 disables fog.
    pub fog_density: f32,
}

impl Default for Lighting {
    fn default() -> Self {
        let lin = |r: f32, g: f32, b: f32| Vec3::new(srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b));
        Self {
            sun_direction: Vec3::new(-0.45, -0.8, -0.35).normalize(),
            sun_color: Vec3::new(1.0, 0.96, 0.88),
            sun_energy: 1.1,
            ambient: Vec3::new(0.26, 0.28, 0.33),
            lights: Vec::new(),
            sky_top: lin(0.32, 0.5, 0.78),
            sky_horizon: lin(0.72, 0.8, 0.88),
            sky_ground: lin(0.42, 0.44, 0.46),
            fog_color: lin(0.7, 0.78, 0.86),
            fog_density: 0.0,
        }
    }
}

/// CPU side mesh grouped into per-material draw ranges.
#[derive(Default)]
pub struct MeshBatch {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
    by_material: HashMap<String, Vec<u32>>,
}

impl MeshBatch {
    /// Adds a convex polygon as a triangle fan.
    pub fn add_polygon(&mut self, material: &str, verts: &[MeshVertex]) {
        if verts.len() < 3 {
            return;
        }
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(verts);
        let idx = self.by_material.entry(material.to_string()).or_default();
        for k in 1..verts.len() as u32 - 1 {
            idx.extend_from_slice(&[base, base + k, base + k + 1]);
        }
    }

    pub fn add_triangles(&mut self, material: &str, verts: &[MeshVertex], indices: &[u32]) {
        let base = self.vertices.len() as u32;
        self.vertices.extend_from_slice(verts);
        self.by_material.entry(material.to_string()).or_default().extend(indices.iter().map(|i| i + base));
    }

    pub fn add_box(&mut self, min: Vec3, max: Vec3, color: [f32; 4]) {
        let faces: [(Vec3, [Vec3; 4]); 6] = [
            (Vec3::X, [Vec3::new(max.x, min.y, max.z), Vec3::new(max.x, min.y, min.z), Vec3::new(max.x, max.y, min.z), Vec3::new(max.x, max.y, max.z)]),
            (Vec3::NEG_X, [Vec3::new(min.x, min.y, min.z), Vec3::new(min.x, min.y, max.z), Vec3::new(min.x, max.y, max.z), Vec3::new(min.x, max.y, min.z)]),
            (Vec3::Y, [Vec3::new(min.x, max.y, max.z), Vec3::new(max.x, max.y, max.z), Vec3::new(max.x, max.y, min.z), Vec3::new(min.x, max.y, min.z)]),
            (Vec3::NEG_Y, [Vec3::new(min.x, min.y, min.z), Vec3::new(max.x, min.y, min.z), Vec3::new(max.x, min.y, max.z), Vec3::new(min.x, min.y, max.z)]),
            (Vec3::Z, [Vec3::new(min.x, min.y, max.z), Vec3::new(max.x, min.y, max.z), Vec3::new(max.x, max.y, max.z), Vec3::new(min.x, max.y, max.z)]),
            (Vec3::NEG_Z, [Vec3::new(max.x, min.y, min.z), Vec3::new(min.x, min.y, min.z), Vec3::new(min.x, max.y, min.z), Vec3::new(max.x, max.y, min.z)]),
        ];
        for (n, quad) in faces {
            let verts: Vec<MeshVertex> = quad.iter().map(|p| MeshVertex { pos: p.to_array(), normal: n.to_array(), uv: [0.0, 0.0], color }).collect();
            self.add_polygon(WHITE_MATERIAL, &verts);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    pub fn materials(&self) -> impl Iterator<Item = &String> {
        self.by_material.keys()
    }
}

pub struct GpuMesh {
    vertex: wgpu::Buffer,
    index: wgpu::Buffer,
    draws: Vec<(String, std::ops::Range<u32>)>,
}

pub struct GpuLines {
    vertex: wgpu::Buffer,
    count: u32,
}

pub struct Material {
    bind_group: wgpu::BindGroup,
    view: wgpu::TextureView,
    normal: Option<wgpu::TextureView>,
    uniform: MaterialUniform,
    nearest: bool,
    pub size: [u32; 2],
    pub flags: MaterialFlags,
}

/// Prefix of composite materials that mix two textures by vertex color alpha.
pub const BLEND_PREFIX: &str = "blend:";

pub fn blend_key(base: &str, blend: &str) -> String {
    format!("{BLEND_PREFIX}{base}|{blend}")
}

/// Offscreen render target for one viewport, displayed by egui as an image.
pub struct ViewTarget {
    pub size: [u32; 2],
    msaa: wgpu::TextureView,
    resolve: wgpu::Texture,
    resolve_view: wgpu::TextureView,
    depth: wgpu::TextureView,
    camera: wgpu::Buffer,
    camera_bg: wgpu::BindGroup,
    pub texture_id: egui::TextureId,
}

pub struct FrameParams {
    pub view_proj: Mat4,
    pub eye: Vec3,
    pub grid_size: f32,
    pub grid_alpha: f32,
    pub shade: ShadeMode,
    pub orthographic: bool,
    pub clear: [f64; 4],
    /// Line thickness in physical pixels, usually the UI pixels per point.
    pub line_width: f32,
}

#[derive(Default)]
pub struct Frame<'a> {
    pub opaque: Vec<&'a GpuMesh>,
    /// Opaque geometry drawn without back face culling (materials with cull disabled).
    pub double_sided: Vec<&'a GpuMesh>,
    /// Sky gradient behind everything, for lit perspective views.
    pub sky: bool,
    /// Edge lines shown in wireframe mode instead of surfaces.
    pub wire_lines: Vec<&'a GpuLines>,
    pub transparent: Vec<&'a GpuMesh>,
    pub lines: Vec<&'a GpuLines>,
    /// Drawn on top of everything, ignoring depth.
    pub overlay_lines: Vec<&'a GpuLines>,
    pub overlay_meshes: Vec<&'a GpuMesh>,
    /// Terrain chunks, drawn with the layer blending pipeline. Draw keys come from `prepare_terrain_material`.
    pub terrain: Vec<&'a GpuMesh>,
}

pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    egui_renderer: std::sync::Arc<egui::mutex::RwLock<egui_wgpu::Renderer>>,
    camera_bgl: wgpu::BindGroupLayout,
    material_bgl: wgpu::BindGroupLayout,
    terrain_bgl: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    sampler_nearest: wgpu::Sampler,
    mesh_opaque: wgpu::RenderPipeline,
    mesh_double: wgpu::RenderPipeline,
    sky_pipeline: wgpu::RenderPipeline,
    flat_normal: wgpu::TextureView,
    white: wgpu::TextureView,
    mesh_transparent: wgpu::RenderPipeline,
    mesh_overlay: wgpu::RenderPipeline,
    line_depth: wgpu::RenderPipeline,
    line_overlay: wgpu::RenderPipeline,
    terrain_pipeline: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    shadow_uniform: wgpu::Buffer,
    shadow_bg: wgpu::BindGroup,
    shadow_view: wgpu::TextureView,
    shadow_sampler: wgpu::Sampler,
    lights_buffer: wgpu::Buffer,
    lights: LightsUniform,
    materials: HashMap<String, Material>,
    terrain_materials: HashMap<String, wgpu::BindGroup>,
}

impl Renderer {
    pub fn new(state: &egui_wgpu::RenderState) -> Self {
        let device = state.device.clone();
        let queue = state.queue.clone();

        let uniform_entry = |binding: u32, visibility: wgpu::ShaderStages| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
            count: None,
        };
        let texture_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let camera_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
                uniform_entry(1, wgpu::ShaderStages::FRAGMENT),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });
        let terrain_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain material"),
            entries: &[
                texture_entry(0),
                texture_entry(1),
                texture_entry(2),
                texture_entry(3),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                uniform_entry(5, wgpu::ShaderStages::FRAGMENT),
            ],
        });
        let shadow_bgl = device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("shadow"), entries: &[uniform_entry(0, wgpu::ShaderStages::VERTEX)] });
        let material_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("material"),
            entries: &[
                texture_entry(0),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                texture_entry(2),
                texture_entry(3),
                uniform_entry(4, wgpu::ShaderStages::FRAGMENT),
            ],
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("material sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 8,
            ..Default::default()
        });
        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("pixel sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow map"),
            size: wgpu::Extent3d { width: SHADOW_SIZE, height: SHADOW_SIZE, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_view = shadow_texture.create_view(&Default::default());
        let shadow_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shadow camera"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shadow_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow"),
            layout: &shadow_bgl,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: shadow_uniform.as_entire_binding() }],
        });
        let lights_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lights"),
            size: std::mem::size_of::<LightsUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let common = include_str!("common.wgsl");
        let mesh_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mesh"),
            source: wgpu::ShaderSource::Wgsl(format!("{common}\n{}", include_str!("mesh.wgsl")).into()),
        });
        let terrain_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("terrain"),
            source: wgpu::ShaderSource::Wgsl(format!("{common}\n{}", include_str!("terrain.wgsl")).into()),
        });
        let shadow_shader = device
            .create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("shadow"), source: wgpu::ShaderSource::Wgsl(include_str!("shadow.wgsl").into()) });
        let line_shader = device
            .create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("line"), source: wgpu::ShaderSource::Wgsl(include_str!("line.wgsl").into()) });
        let sky_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sky"),
            source: wgpu::ShaderSource::Wgsl(format!("{common}\n{}", include_str!("sky.wgsl")).into()),
        });

        let mesh_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesh"),
            bind_group_layouts: &[Some(&camera_bgl), Some(&material_bgl)],
            immediate_size: 0,
        });
        let line_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("line"), bind_group_layouts: &[Some(&camera_bgl)], immediate_size: 0 });
        let terrain_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("terrain"),
            bind_group_layouts: &[Some(&camera_bgl), Some(&terrain_bgl)],
            immediate_size: 0,
        });
        let shadow_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow"),
            bind_group_layouts: &[Some(&shadow_bgl)],
            immediate_size: 0,
        });

        let mesh_attrs = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x2, 3 => Float32x4];
        // One instance per segment, reading both endpoints of the line vertex pair.
        let line_attrs = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x3, 3 => Float32x4];

        let make = |label: &str,
                    shader: &wgpu::ShaderModule,
                    layout: &wgpu::PipelineLayout,
                    attrs: &[wgpu::VertexAttribute],
                    stride: usize,
                    step_mode: wgpu::VertexStepMode,
                    topology: wgpu::PrimitiveTopology,
                    cull: bool,
                    blend: bool,
                    depth_write: bool,
                    depth_compare: wgpu::CompareFunction| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: Some("vs_main"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout { array_stride: stride as u64, step_mode, attributes: attrs })],
                },
                primitive: wgpu::PrimitiveState {
                    topology,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: if cull { Some(wgpu::Face::Back) } else { None },
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(depth_write),
                    depth_compare: Some(depth_compare),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState { count: MSAA_SAMPLES, mask: !0, alpha_to_coverage_enabled: false },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: COLOR_FORMAT,
                        blend: if blend { Some(wgpu::BlendState::ALPHA_BLENDING) } else { None },
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            })
        };

        let mesh_stride = std::mem::size_of::<MeshVertex>();
        let line_stride = std::mem::size_of::<LineVertex>() * 2;
        let tri = wgpu::PrimitiveTopology::TriangleList;
        use wgpu::CompareFunction::{Always, Greater, GreaterEqual};
        let mesh_opaque =
            make("mesh opaque", &mesh_shader, &mesh_layout, &mesh_attrs, mesh_stride, wgpu::VertexStepMode::Vertex, tri, true, false, true, Greater);
        let mesh_double =
            make("mesh double sided", &mesh_shader, &mesh_layout, &mesh_attrs, mesh_stride, wgpu::VertexStepMode::Vertex, tri, false, false, true, Greater);
        let sky_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky"),
            layout: Some(&line_layout),
            vertex: wgpu::VertexState { module: &sky_shader, entry_point: Some("vs_main"), compilation_options: Default::default(), buffers: &[] },
            primitive: wgpu::PrimitiveState { topology: tri, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(Always),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState { count: MSAA_SAMPLES, mask: !0, alpha_to_coverage_enabled: false },
            fragment: Some(wgpu::FragmentState {
                module: &sky_shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState { format: COLOR_FORMAT, blend: None, write_mask: wgpu::ColorWrites::ALL })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let solid_texture = |label: &str, format: wgpu::TextureFormat, pixel: [u8; 4]| {
            let t = device.create_texture_with_data(
                &queue,
                &wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING,
                    view_formats: &[],
                },
                wgpu::util::TextureDataOrder::LayerMajor,
                &pixel,
            );
            t.create_view(&Default::default())
        };
        let flat_normal = solid_texture("flat normal", wgpu::TextureFormat::Rgba8Unorm, [128, 128, 255, 255]);
        let white = solid_texture("white", wgpu::TextureFormat::Rgba8UnormSrgb, [255, 255, 255, 255]);
        let mesh_transparent =
            make("mesh transparent", &mesh_shader, &mesh_layout, &mesh_attrs, mesh_stride, wgpu::VertexStepMode::Vertex, tri, false, true, false, GreaterEqual);
        let mesh_overlay =
            make("mesh overlay", &mesh_shader, &mesh_layout, &mesh_attrs, mesh_stride, wgpu::VertexStepMode::Vertex, tri, false, true, false, Always);
        let line_depth =
            make("line depth", &line_shader, &line_layout, &line_attrs, line_stride, wgpu::VertexStepMode::Instance, tri, false, true, false, GreaterEqual);
        let line_overlay =
            make("line overlay", &line_shader, &line_layout, &line_attrs, line_stride, wgpu::VertexStepMode::Instance, tri, false, true, false, Always);
        let terrain_pipeline =
            make("terrain", &terrain_shader, &terrain_layout, &mesh_attrs, mesh_stride, wgpu::VertexStepMode::Vertex, tri, true, false, true, Greater);
        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow"),
            layout: Some(&shadow_layout),
            vertex: wgpu::VertexState {
                module: &shadow_shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: mesh_stride as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &mesh_attrs,
                })],
            },
            // Both sides cast shadows so open meshes and single sided decals still block the sun.
            primitive: wgpu::PrimitiveState { topology: tri, cull_mode: None, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: wgpu::DepthBiasState { constant: 2, slope_scale: 2.0, clamp: 0.0 },
            }),
            multisample: Default::default(),
            fragment: None,
            multiview_mask: None,
            cache: None,
        });

        let mut r = Self {
            device,
            queue,
            egui_renderer: state.renderer.clone(),
            camera_bgl,
            material_bgl,
            terrain_bgl,
            sampler,
            sampler_nearest,
            mesh_opaque,
            mesh_double,
            sky_pipeline,
            flat_normal,
            white,
            mesh_transparent,
            mesh_overlay,
            line_depth,
            line_overlay,
            terrain_pipeline,
            shadow_pipeline,
            shadow_uniform,
            shadow_bg,
            shadow_view,
            shadow_sampler,
            lights_buffer,
            lights: bytemuck::Zeroable::zeroed(),
            materials: HashMap::new(),
            terrain_materials: HashMap::new(),
        };
        r.set_lighting(&Lighting::default(), None);
        r.set_material(WHITE_MATERIAL, &image::RgbaImage::from_pixel(4, 4, image::Rgba([255, 255, 255, 255])));
        let checker =
            image::RgbaImage::from_fn(
                64,
                64,
                |x, y| {
                    if ((x / 16) + (y / 16)) % 2 == 0 { image::Rgba([96, 96, 104, 255]) } else { image::Rgba([150, 150, 160, 255]) }
                },
            );
        r.set_material(MISSING_MATERIAL, &checker);
        r
    }

    pub fn has_material(&self, name: &str) -> bool {
        self.materials.contains_key(name)
    }

    pub fn material_size(&self, name: &str) -> Option<[u32; 2]> {
        self.materials.get(name).map(|m| m.size)
    }

    pub fn material_flags(&self, name: &str) -> MaterialFlags {
        self.materials.get(name).map(|m| m.flags).unwrap_or_default()
    }

    /// Drops every uploaded material except the built-in ones, so they reload with new settings.
    pub fn clear_materials(&mut self) {
        self.materials.retain(|k, _| k == WHITE_MATERIAL || k == MISSING_MATERIAL);
        self.terrain_materials.clear();
    }

    pub fn set_material(&mut self, name: &str, image: &image::RgbaImage) {
        self.set_material_filtered(name, image, false);
    }

    /// Like `set_material`, with nearest neighbour magnification for pixel art when `pixelated` is set.
    pub fn set_material_filtered(&mut self, name: &str, image: &image::RgbaImage, pixelated: bool) {
        self.set_material_desc(name, &MaterialDesc::plain(image, pixelated));
    }

    fn upload_texture(&self, label: &str, image: &image::RgbaImage, srgb: bool) -> wgpu::TextureView {
        let (w, h) = image.dimensions();
        let mip_count = (w.max(h) as f32).log2().floor() as u32 + 1;
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
            mip_level_count: mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm },
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let mut level = image.clone();
        for mip in 0..mip_count {
            let (lw, lh) = level.dimensions();
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: mip, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                level.as_raw(),
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(4 * lw), rows_per_image: Some(lh) },
                wgpu::Extent3d { width: lw, height: lh, depth_or_array_layers: 1 },
            );
            if mip + 1 < mip_count {
                level = image::imageops::resize(&level, (lw / 2).max(1), (lh / 2).max(1), image::imageops::FilterType::Triangle);
            }
        }
        texture.create_view(&Default::default())
    }

    /// Key of a material mixing `base` and `blend` by vertex color alpha (0 base, 1 blend). Both must be uploaded first,
    /// otherwise the base material key is returned.
    pub fn prepare_blend_material(&mut self, base: &str, blend: &str) -> String {
        let key = blend_key(base, blend);
        if self.materials.contains_key(&key) {
            return key;
        }
        let (Some(a), Some(b)) = (self.materials.get(base), self.materials.get(blend)) else { return base.to_string() };
        let mut uniform = a.uniform;
        uniform.emission = [0.0; 4];
        uniform.extra[1] = 1.0;
        let params = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("blend material params"),
            contents: bytemuck::bytes_of(&uniform),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&key),
            layout: &self.material_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&a.view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(if a.nearest { &self.sampler_nearest } else { &self.sampler }) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(a.normal.as_ref().unwrap_or(&self.flat_normal)) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&b.view) },
                wgpu::BindGroupEntry { binding: 4, resource: params.as_entire_binding() },
            ],
        });
        let material = Material {
            bind_group,
            view: a.view.clone(),
            normal: a.normal.clone(),
            uniform,
            nearest: a.nearest,
            size: a.size,
            flags: MaterialFlags { transparent: false, double_sided: a.flags.double_sided },
        };
        self.materials.insert(key.clone(), material);
        key
    }

    pub fn set_material_desc(&mut self, name: &str, desc: &MaterialDesc) {
        self.terrain_materials.retain(|key, _| !key.split('|').any(|part| part == name));
        self.materials.retain(|key, _| !key.strip_prefix(BLEND_PREFIX).is_some_and(|rest| rest.split('|').any(|part| part == name)));
        let (w, h) = desc.albedo.dimensions();
        let view = self.upload_texture(name, desc.albedo, true);
        let normal = desc.normal.map(|n| self.upload_texture(name, n, false));
        let emission = desc.emission_texture.map(|e| self.upload_texture(name, e, true));
        let (alpha_mode, threshold) = match desc.alpha {
            AlphaMode::Opaque => (0.0, 0.0),
            AlphaMode::Blend => (1.0, 0.0),
            AlphaMode::Scissor(t) => (2.0, t),
        };
        let uniform = MaterialUniform {
            tint: desc.tint,
            emission: [desc.emission[0], desc.emission[1], desc.emission[2], if emission.is_some() { 1.0 } else { 0.0 }],
            flags: [if normal.is_some() { 1.0 } else { 0.0 }, alpha_mode, threshold, if desc.unshaded { 1.0 } else { 0.0 }],
            extra: [desc.normal_scale, 0.0, 0.0, 0.0],
        };
        let params = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("material params"),
            contents: bytemuck::bytes_of(&uniform),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(name),
            layout: &self.material_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(if desc.nearest { &self.sampler_nearest } else { &self.sampler }) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(normal.as_ref().unwrap_or(&self.flat_normal)) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(emission.as_ref().unwrap_or(&self.white)) },
                wgpu::BindGroupEntry { binding: 4, resource: params.as_entire_binding() },
            ],
        });
        let flags = MaterialFlags { transparent: desc.alpha == AlphaMode::Blend || desc.tint[3] < 0.999, double_sided: desc.double_sided };
        self.materials.insert(name.to_string(), Material { bind_group, view, normal, uniform, nearest: desc.nearest, size: [w, h], flags });
    }

    /// Key for a terrain draw with up to four layers `(material, world units per repeat)`. Creates the bind group once.
    pub fn prepare_terrain_material(&mut self, layers: &[(String, f32)]) -> String {
        let mut names: Vec<String> = layers.iter().take(4).map(|(m, _)| m.clone()).collect();
        let mut tiles: Vec<f32> = layers.iter().take(4).map(|(_, t)| *t).collect();
        while names.len() < 4 {
            names.push(names.first().cloned().unwrap_or_else(|| MISSING_MATERIAL.to_string()));
            tiles.push(tiles.first().copied().unwrap_or(256.0));
        }
        let key = format!("{}|{}|{}|{}|{}", names[0], names[1], names[2], names[3], tiles.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(","));
        if self.terrain_materials.contains_key(&key) {
            return key;
        }
        let view = |n: &str| &self.materials.get(n).or_else(|| self.materials.get(MISSING_MATERIAL)).expect("missing material exists").view;
        let tiles_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("terrain tiles"),
            contents: bytemuck::cast_slice(&[tiles[0], tiles[1], tiles[2], tiles[3]]),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("terrain material"),
            layout: &self.terrain_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(view(&names[0])) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(view(&names[1])) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(view(&names[2])) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(view(&names[3])) },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(&self.sampler) },
                wgpu::BindGroupEntry { binding: 5, resource: tiles_buffer.as_entire_binding() },
            ],
        });
        self.terrain_materials.insert(key.clone(), bind_group);
        key
    }

    /// Uploads lights. `shadow` is the sun's view projection when a shadow map was rendered for it.
    pub fn set_lighting(&mut self, lighting: &Lighting, shadow: Option<Mat4>) {
        let mut u: LightsUniform = bytemuck::Zeroable::zeroed();
        let d = lighting.sun_direction.normalize_or(Vec3::NEG_Y);
        u.sun_dir = [d.x, d.y, d.z, if shadow.is_some() { 1.0 } else { 0.0 }];
        u.sun_color = [lighting.sun_color.x, lighting.sun_color.y, lighting.sun_color.z, lighting.sun_energy];
        let count = lighting.lights.len().min(MAX_LIGHTS);
        u.ambient = [lighting.ambient.x, lighting.ambient.y, lighting.ambient.z, count as f32];
        u.sky_top = lighting.sky_top.extend(1.0).to_array();
        u.sky_horizon = lighting.sky_horizon.extend(1.0).to_array();
        u.sky_ground = lighting.sky_ground.extend(1.0).to_array();
        u.fog = lighting.fog_color.extend(lighting.fog_density).to_array();
        u.shadow_view_proj = shadow.unwrap_or(Mat4::IDENTITY).to_cols_array_2d();
        for (slot, l) in u.lights.iter_mut().zip(&lighting.lights) {
            slot.pos_range = [l.position.x, l.position.y, l.position.z, l.range];
            slot.color_energy = [l.color.x, l.color.y, l.color.z, l.energy];
            slot.dir_cone = match l.spot {
                Some((dir, cos)) => [dir.x, dir.y, dir.z, cos],
                None => [0.0, -1.0, 0.0, -2.0],
            };
        }
        self.lights = u;
        self.queue.write_buffer(&self.lights_buffer, 0, bytemuck::bytes_of(&u));
    }

    /// Sun view projection fitting the bounds (min, max), looking along `direction`.
    pub fn sun_view_proj(direction: Vec3, min: Vec3, max: Vec3) -> Mat4 {
        let center = (min + max) * 0.5;
        let radius = ((max - min).length() * 0.5).max(1.0);
        let dir = direction.normalize_or(Vec3::NEG_Y);
        let up = if dir.y.abs() > 0.95 { Vec3::Z } else { Vec3::Y };
        let view = glam::camera::rh::view::look_to_mat4(center - dir * radius * 2.0, dir, up);
        let proj = glam::camera::rh::proj::directx::orthographic(-radius, radius, -radius, radius, 0.0, radius * 4.0);
        proj * view
    }

    /// Renders the opaque and double sided meshes into the sun shadow map.
    pub fn render_shadow(&self, meshes: &[&GpuMesh], view_proj: Mat4) {
        self.queue.write_buffer(&self.shadow_uniform, 0, bytemuck::cast_slice(&view_proj.to_cols_array()));
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("shadow") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.shadow_pipeline);
            pass.set_bind_group(0, &self.shadow_bg, &[]);
            for mesh in meshes {
                pass.set_vertex_buffer(0, mesh.vertex.slice(..));
                pass.set_index_buffer(mesh.index.slice(..), wgpu::IndexFormat::Uint32);
                for (_, range) in &mesh.draws {
                    pass.draw_indexed(range.clone(), 0, 0..1);
                }
            }
        }
        self.queue.submit([encoder.finish()]);
    }

    pub fn upload_mesh(&self, batch: &MeshBatch) -> Option<GpuMesh> {
        if batch.vertices.is_empty() {
            return None;
        }
        let mut indices = Vec::with_capacity(batch.by_material.values().map(|v| v.len()).sum());
        let mut draws = Vec::new();
        let mut keys: Vec<&String> = batch.by_material.keys().collect();
        keys.sort();
        for k in keys {
            let idx = &batch.by_material[k];
            if idx.is_empty() {
                continue;
            }
            let start = indices.len() as u32;
            indices.extend_from_slice(idx);
            draws.push((k.clone(), start..indices.len() as u32));
        }
        if indices.is_empty() {
            return None;
        }
        let vertex = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh vertices"),
            contents: bytemuck::cast_slice(&batch.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesh indices"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        Some(GpuMesh { vertex, index, draws })
    }

    pub fn upload_lines(&self, lines: &[LineVertex]) -> Option<GpuLines> {
        if lines.len() < 2 {
            return None;
        }
        let vertex = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("lines"),
            contents: bytemuck::cast_slice(lines),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Some(GpuLines { vertex, count: (lines.len() / 2 * 2) as u32 })
    }

    /// Creates or resizes the target. Returns true if it was (re)created.
    pub fn ensure_target(&self, target: &mut Option<ViewTarget>, size: [u32; 2]) -> bool {
        let size = [size[0].max(1), size[1].max(1)];
        if target.as_ref().is_some_and(|t| t.size == size) {
            return false;
        }
        let extent = wgpu::Extent3d { width: size[0], height: size[1], depth_or_array_layers: 1 };
        let tex = |label: &str, format, samples, usage, view_formats: &[wgpu::TextureFormat]| {
            self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: extent,
                mip_level_count: 1,
                sample_count: samples,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats,
            })
        };
        let msaa = tex("view msaa", COLOR_FORMAT, MSAA_SAMPLES, wgpu::TextureUsages::RENDER_ATTACHMENT, &[]).create_view(&Default::default());
        let resolve = tex(
            "view resolve",
            COLOR_FORMAT,
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC,
            &[wgpu::TextureFormat::Rgba8Unorm],
        );
        let resolve_view = resolve.create_view(&Default::default());
        // egui expects gamma encoded texels from native textures, so it samples the bytes through a linear view.
        let egui_view = resolve.create_view(&wgpu::TextureViewDescriptor { format: Some(wgpu::TextureFormat::Rgba8Unorm), ..Default::default() });
        let depth = tex("view depth", DEPTH_FORMAT, MSAA_SAMPLES, wgpu::TextureUsages::RENDER_ATTACHMENT, &[]).create_view(&Default::default());

        let mut egui_r = self.egui_renderer.write();
        let texture_id = match target.take() {
            Some(old) => {
                egui_r.update_egui_texture_from_wgpu_texture(&self.device, &egui_view, wgpu::FilterMode::Linear, old.texture_id);
                old.texture_id
            }
            None => egui_r.register_native_texture(&self.device, &egui_view, wgpu::FilterMode::Linear),
        };
        let camera = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("camera"),
            size: std::mem::size_of::<CameraUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera"),
            layout: &self.camera_bgl,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: camera.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: self.lights_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&self.shadow_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::Sampler(&self.shadow_sampler) },
            ],
        });
        *target = Some(ViewTarget { size, msaa, resolve, resolve_view, depth, camera, camera_bg, texture_id });
        true
    }

    /// Copies the last rendered frame of a target back to the CPU (blocking). Pixels are sRGB.
    pub fn read_target(&self, target: &ViewTarget) -> Option<image::RgbaImage> {
        let [w, h] = target.size;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded = (4 * w).div_ceil(align) * align;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (padded * h) as u64,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("readback") });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &target.resolve, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(h) },
            },
            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        );
        self.queue.submit([encoder.finish()]);
        let slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        let _ = self.device.poll(wgpu::PollType::wait_indefinitely());
        rx.recv().ok()?.ok()?;
        let data = slice.get_mapped_range().ok()?;
        let mut out = Vec::with_capacity((w * h * 4) as usize);
        for row in 0..h {
            let start = (row * padded) as usize;
            out.extend_from_slice(&data[start..start + (w * 4) as usize]);
        }
        drop(data);
        buffer.unmap();
        image::RgbaImage::from_raw(w, h, out)
    }

    pub fn free_target(&self, target: ViewTarget) {
        self.egui_renderer.write().free_texture(&target.texture_id);
        target.resolve.destroy();
    }

    pub fn render(&self, target: &ViewTarget, params: &FrameParams, frame: &Frame) {
        let uniform = CameraUniform {
            view_proj: params.view_proj.to_cols_array_2d(),
            inv_view_proj: params.view_proj.inverse().to_cols_array_2d(),
            eye: params.eye.extend(1.0).to_array(),
            params: [
                params.grid_size,
                params.grid_alpha,
                match params.shade {
                    ShadeMode::Textured => 0.0,
                    ShadeMode::Flat | ShadeMode::Wireframe => 1.0,
                    ShadeMode::Lit => 2.0,
                },
                if params.orthographic { 1.0 } else { 0.0 },
            ],
            viewport: [target.size[0] as f32, target.size[1] as f32, params.line_width.max(1.0), 0.0],
        };
        self.queue.write_buffer(&target.camera, 0, bytemuck::bytes_of(&uniform));

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("viewport") });
        {
            let [r, g, b, a] = params.clear;
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("viewport"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target.msaa,
                    depth_slice: None,
                    resolve_target: Some(&target.resolve_view),
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a }), store: wgpu::StoreOp::Discard },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &target.depth,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(0.0), store: wgpu::StoreOp::Discard }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &target.camera_bg, &[]);

            let draw_meshes = |pass: &mut wgpu::RenderPass, pipeline: &wgpu::RenderPipeline, meshes: &[&GpuMesh]| {
                if meshes.is_empty() {
                    return;
                }
                pass.set_pipeline(pipeline);
                for mesh in meshes {
                    pass.set_vertex_buffer(0, mesh.vertex.slice(..));
                    pass.set_index_buffer(mesh.index.slice(..), wgpu::IndexFormat::Uint32);
                    for (material, range) in &mesh.draws {
                        let mat = self.materials.get(material).or_else(|| self.materials.get(MISSING_MATERIAL)).expect("missing material exists");
                        pass.set_bind_group(1, &mat.bind_group, &[]);
                        pass.draw_indexed(range.clone(), 0, 0..1);
                    }
                }
            };
            let draw_lines = |pass: &mut wgpu::RenderPass, pipeline: &wgpu::RenderPipeline, lines: &[&GpuLines]| {
                if lines.is_empty() {
                    return;
                }
                pass.set_pipeline(pipeline);
                for l in lines {
                    pass.set_vertex_buffer(0, l.vertex.slice(..));
                    pass.draw(0..6, 0..l.count / 2);
                }
            };

            if frame.sky {
                pass.set_pipeline(&self.sky_pipeline);
                pass.draw(0..3, 0..1);
            }
            if params.shade == ShadeMode::Wireframe {
                draw_lines(&mut pass, &self.line_depth, &frame.wire_lines);
                draw_lines(&mut pass, &self.line_depth, &frame.lines);
                draw_meshes(&mut pass, &self.mesh_overlay, &frame.overlay_meshes);
                draw_lines(&mut pass, &self.line_overlay, &frame.overlay_lines);
                drop(pass);
                self.queue.submit([encoder.finish()]);
                return;
            }
            draw_meshes(&mut pass, &self.mesh_opaque, &frame.opaque);
            draw_meshes(&mut pass, &self.mesh_double, &frame.double_sided);
            if !frame.terrain.is_empty() {
                pass.set_pipeline(&self.terrain_pipeline);
                for mesh in &frame.terrain {
                    pass.set_vertex_buffer(0, mesh.vertex.slice(..));
                    pass.set_index_buffer(mesh.index.slice(..), wgpu::IndexFormat::Uint32);
                    for (key, range) in &mesh.draws {
                        if let Some(bg) = self.terrain_materials.get(key) {
                            pass.set_bind_group(1, bg, &[]);
                            pass.draw_indexed(range.clone(), 0, 0..1);
                        }
                    }
                }
            }
            draw_lines(&mut pass, &self.line_depth, &frame.lines);
            draw_meshes(&mut pass, &self.mesh_transparent, &frame.transparent);
            draw_meshes(&mut pass, &self.mesh_overlay, &frame.overlay_meshes);
            draw_lines(&mut pass, &self.line_overlay, &frame.overlay_lines);
        }
        self.queue.submit([encoder.finish()]);
    }
}

#[cfg(test)]
mod tests {
    fn validate(label: &str, source: &str) {
        let module = naga::front::wgsl::parse_str(source).unwrap_or_else(|e| panic!("{label}: {}", e.emit_to_string(source)));
        naga::valid::Validator::new(naga::valid::ValidationFlags::all(), naga::valid::Capabilities::empty())
            .validate(&module)
            .unwrap_or_else(|e| panic!("{label}: {e:?}"));
    }

    #[test]
    fn shaders_validate() {
        let common = include_str!("common.wgsl");
        for (label, body) in [("mesh", include_str!("mesh.wgsl")), ("terrain", include_str!("terrain.wgsl")), ("sky", include_str!("sky.wgsl"))] {
            validate(label, &format!("{common}\n{body}"));
        }
        validate("line", include_str!("line.wgsl"));
        validate("shadow", include_str!("shadow.wgsl"));
    }

    #[test]
    fn uniform_sizes_match_shaders() {
        assert_eq!(std::mem::size_of::<super::CameraUniform>(), 176);
        assert_eq!(std::mem::size_of::<super::MaterialUniform>(), 64);
        assert_eq!(std::mem::size_of::<super::LightsUniform>(), 28 * 4 + 64 + super::MAX_LIGHTS * 48);
    }
}
