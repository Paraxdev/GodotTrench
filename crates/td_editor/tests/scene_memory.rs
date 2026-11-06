//! GPU memory the 3D scene takes per node. Needs a GPU backend with an allocator report (Vulkan or DX12), so the
//! tests are ignored by default: `cargo test -p td_editor --test scene_memory -- --ignored`.

use egui_wgpu::wgpu;
use td_core::DVec3;
use td_doc::scatter::ScatterInstance;
use td_doc::{Map, NodeKind, Scatter, ScatterItem, ScatterKind};
use td_editor::scene::SceneCache;
use td_editor::state::{EditorState, Prefs};

const TREE: &str = "res://demo/models/pine.bbmodel";

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    let mut f = std::pin::pin!(f);
    let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
    loop {
        if let std::task::Poll::Ready(v) = f.as_mut().poll(&mut cx) {
            return v;
        }
    }
}

struct Gpu {
    rs: egui_wgpu::RenderState,
    renderer: td_render::Renderer,
    dir: std::path::PathBuf,
}

impl Gpu {
    fn new(name: &str) -> Self {
        let config = egui_wgpu::WgpuConfiguration::default();
        let instance = block_on(config.wgpu_setup.new_instance());
        let rs = block_on(egui_wgpu::RenderState::create(&config, &instance, None, egui_wgpu::RendererOptions::default())).unwrap();
        let renderer = td_render::Renderer::new(&rs);
        let dir = std::env::temp_dir().join(format!("td_scene_memory_{name}_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        Gpu { rs, renderer, dir }
    }

    fn allocated(&self) -> i64 {
        self.rs.device.generate_allocator_report().expect("this backend gives no allocator report, use Vulkan or DX12").total_allocated_bytes as i64
    }

    /// GPU bytes the scene of `map` holds once its upload buffers are freed, and the triangles it draws.
    fn scene_cost(&mut self, map: &Map) -> (i64, usize) {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../godot");
        let path = self.dir.join("map.tdm");
        td_doc::format::save(map, &path).unwrap();
        let mut state = EditorState::new(Prefs::default());
        state.load_project(&root);
        state.open_map(&path).unwrap();
        self.settle();
        let before = self.allocated();
        let mut scene = SceneCache::default();
        scene.update(&mut self.renderer, &mut state, 0);
        self.settle();
        let cost = (self.allocated() - before, scene.stats.triangles);
        drop(scene);
        self.settle();
        cost
    }

    fn settle(&self) {
        self.rs.queue.submit([]);
        self.rs.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
}

impl Drop for Gpu {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn forest(sets: usize, per_set: usize) -> Map {
    let mut m = Map::new();
    let layer = m.default_layer();
    for s in 0..sets {
        let mut set = Scatter::new(format!("forest {s}"), ScatterKind::Props, vec![ScatterItem::new(TREE), ScatterItem::new("res://missing.glb")]);
        set.instances = (0..per_set)
            .map(|i| {
                let (x, z) = ((i % 97) as f64, (i / 97) as f64);
                ScatterInstance {
                    item: if i % 7 == 3 { 1 } else { 0 },
                    position: DVec3::new(3000.5 + x * 70.0 + z * 13.0, 40.0 + x * 1.5 - z * 0.75, -2000.25 + z * 55.0 + s as f64 * 9.0),
                    angles: DVec3::new((i % 5) as f64 * 3.0, (i * 37 % 360) as f64, -((i % 3) as f64) * 2.0),
                    scale: 0.6 + (i % 11) as f64 * 0.09,
                }
            })
            .collect();
        m.insert(layer, NodeKind::Scatter(set));
    }

    m
}

#[test]
#[ignore = "needs a GPU"]
fn scatter_instances_upload_their_model_once() {
    let mut gpu = Gpu::new("instances");
    let (_, empty_tris) = gpu.scene_cost(&forest(1, 0));
    let (one, one_tris) = gpu.scene_cost(&forest(1, 1));
    let (sets, sets_tris) = gpu.scene_cost(&forest(101, 1));
    let (few, few_tris) = gpu.scene_cost(&forest(1, 1_000));
    let (many, many_tris) = gpu.scene_cost(&forest(1, 11_000));
    assert_eq!(empty_tris, 0);
    assert!(one_tris > 0, "one tree draws its model");
    let trees = |n: usize| (0..n).filter(|i| i % 7 != 3).count();
    assert_eq!(sets_tris, one_tris * 101);
    assert_eq!(few_tris, one_tris * trees(1_000));
    assert_eq!(many_tris, one_tris * trees(11_000));
    // A record per instance plus its cross in the 2D views, the tree itself is uploaded once.
    let per_instance = (many - few) / 10_000;
    assert!(per_instance <= 192, "a scatter instance takes {per_instance} B of GPU memory");
    let per_set = (sets - one) / 100;
    assert!(per_set <= 2048, "a scatter set of one tree takes {per_set} B of GPU memory");
}
