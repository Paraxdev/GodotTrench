//! Exports a map as a 3D model for Blender and other 3D tools, as glTF binary (.glb) or Wavefront OBJ. Only what the
//! Godot build draws goes out: brushes, meshes, terrains and models with their materials. Entity logic, I/O wiring,
//! triggers and scripts stay behind, and the file cannot be opened as a map again.

mod collect;
pub mod dialog;
mod glb;
mod obj;
mod textures;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use gt_core::NodeId;
use gt_doc::{Map, NodeKind};
use gt_formats::GameConfig;

use crate::materials::MaterialLibrary;
use crate::models::ModelCache;
use crate::prefabs::PrefabCache;

/// OBJ cannot share a mesh between copies, so every scatter instance is written out in full. Past this many triangles
/// the file runs into gigabytes few tools open, and an OBJ export is refused.
pub const OBJ_TRIANGLE_LIMIT: usize = 10_000_000;

/// The error of an export stopped with [`Progress::cancel`].
pub const CANCELLED: &str = "export cancelled";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Glb,
    Obj,
}

impl Format {
    pub fn extension(self) -> &'static str {
        match self {
            Format::Glb => "glb",
            Format::Obj => "obj",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Format::Glb => "glTF Binary (.glb)",
            Format::Obj => "Wavefront OBJ (.obj)",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// Only the selected objects, still inside their layers and groups.
    pub selection_only: bool,
    /// Hidden layers and objects too.
    pub hidden: bool,
    /// Every instance of the scatter sets, one object each.
    pub scatter: bool,
    /// Models shown by prop and other point entities.
    pub models: bool,
    /// Point entities as empty objects, for lining things up in the 3D tool.
    pub markers: bool,
    /// Brushes without a name of their own join into one object per layer, group or entity.
    pub merge_brushes: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self { selection_only: false, hidden: false, scatter: false, models: true, markers: false, merge_brushes: false }
    }
}

/// What an export wrote.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub objects: usize,
    pub meshes: usize,
    pub materials: usize,
    pub images: usize,
    pub triangles: usize,
    pub bytes: u64,
    /// Materials the project has no image for, exported untextured.
    pub missing: Vec<String>,
}

impl Report {
    /// One line for the status bar.
    pub fn summary(&self, path: &Path) -> String {
        let mut s = format!(
            "Exported {} objects, {} materials and {} triangles to {} ({}). Entity logic, triggers and scripts are not included",
            self.objects,
            self.materials,
            self.triangles,
            path.display(),
            size_text(self.bytes)
        );
        if !self.missing.is_empty() {
            s += &format!(". No image found for {}", self.missing.join(", "));
        }

        s
    }
}

/// The map and the caches an export reads models, prefabs and materials through.
pub struct Sources<'a> {
    pub map: &'a Map,
    pub map_path: Option<&'a Path>,
    pub game: &'a GameConfig,
    pub materials: &'a mut MaterialLibrary,
    pub models: &'a mut ModelCache,
    pub prefabs: &'a mut PrefabCache,
    pub selection: &'a BTreeSet<NodeId>,
}

impl<'a> Sources<'a> {
    pub fn of(state: &'a mut crate::state::EditorState) -> Self {
        Self {
            map: &state.doc.map,
            map_path: state.doc.path.as_deref(),
            game: &state.game,
            materials: &mut state.materials,
            models: &mut state.models,
            prefabs: &mut state.prefabs,
            selection: &state.doc.selection.nodes,
        }
    }
}

/// A file size for people, in MB or GB.
pub fn size_text(bytes: u64) -> String {
    if bytes >= 1 << 30 { format!("{:.1} GB", bytes as f64 / (1u64 << 30) as f64) } else { format!("{:.1} MB", bytes as f64 / (1u64 << 20) as f64) }
}

/// Roughly what an OBJ file of this many vertices and triangles takes, as the writer formats them.
pub fn obj_bytes(vertices: usize, triangles: usize) -> u64 {
    vertices as u64 * 80 + triangles as u64 * 75
}

/// How far an export got, shared by the thread running it and the window showing it, and the flag that stops it.
#[derive(Default)]
pub struct Progress {
    writing: AtomicBool,
    done: AtomicU64,
    total: AtomicU64,
    cancel: AtomicBool,
}

impl Progress {
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    /// Stops the export with [`CANCELLED`] once it is cancelled.
    fn check(&self) -> Result<(), String> {
        if self.is_cancelled() { Err(CANCELLED.into()) } else { Ok(()) }
    }

    fn begin(&self, writing: bool, total: usize) {
        self.writing.store(writing, Ordering::Relaxed);
        self.done.store(0, Ordering::Relaxed);
        self.total.store(total.max(1) as u64, Ordering::Relaxed);
    }

    fn advance(&self, n: usize) {
        self.done.fetch_add(n as u64, Ordering::Relaxed);
    }

    /// Whether the file is being written yet, and how much of that step is done, from 0 to 1.
    pub fn get(&self) -> (bool, f32) {
        let total = self.total.load(Ordering::Relaxed).max(1);
        (self.writing.load(Ordering::Relaxed), (self.done.load(Ordering::Relaxed) as f64 / total as f64).min(1.0) as f32)
    }
}

/// Writes the export to `path`. OBJ also writes `<name>.mtl` and the textures into a `<name>_textures` folder next to it.
pub fn export(src: Sources, path: &Path, format: Format, options: &Options) -> Result<Report, String> {
    export_with(src, path, format, options, &Progress::default())
}

/// [`export`] reporting how far it got to `progress`. A cancelled export fails with [`CANCELLED`] and writes nothing.
pub fn export_with(src: Sources, path: &Path, format: Format, options: &Options, progress: &Progress) -> Result<Report, String> {
    let name = src.map_path.and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "map".into());
    let scene = collect::collect(src, options, name, progress);
    progress.check()?;
    if scene.roots.is_empty() {
        return Err(if options.selection_only { "nothing to export in the selection" } else { "the map has nothing to export" }.into());
    }

    let triangles = scene.triangles();
    if format == Format::Obj && triangles > OBJ_TRIANGLE_LIMIT {
        return Err(format!(
            "the OBJ would hold {:.1} million triangles, about {}, more than the {} million an OBJ export takes. OBJ writes every \
             scatter instance in full, glTF shares one mesh between them. Export as glTF, leave out the scatter instances, or \
             export part of the map with Selection only",
            triangles as f64 / 1e6,
            size_text(obj_bytes(scene.vertices(), triangles)),
            OBJ_TRIANGLE_LIMIT / 1_000_000
        ));
    }

    let bytes = match format {
        Format::Glb => glb::write(&scene, path, progress)?,
        Format::Obj => obj::write(&scene, path, progress)?,
    };
    Ok(Report {
        objects: scene.nodes.len(),
        meshes: scene.meshes.len(),
        materials: scene.materials.list.len(),
        images: scene.materials.images.len(),
        triangles,
        bytes,
        missing: scene.materials.missing.iter().cloned().collect(),
    })
}

/// `--export-glb` and `--export-obj`: loads a map with the Godot project it sits in and exports it with the default
/// options, without a window.
pub fn export_file(map_path: &Path, out: &Path, format: Format) -> Result<Report, String> {
    let loaded = gt_doc::format::load(map_path).map_err(|e| format!("{}: {e}", map_path.display()))?;
    for p in &loaded.problems {
        eprintln!("{}: {p}", map_path.display());
    }

    let map_path = std::path::absolute(map_path).unwrap_or_else(|_| map_path.to_path_buf());
    let root = gt_formats::game::find_project_root(&map_path);
    let mut game = root.as_deref().and_then(GameConfig::discover).and_then(|p| GameConfig::load(&p).ok()).unwrap_or_else(GameConfig::builtin);
    game.project_root = root.clone();
    let mut materials = MaterialLibrary::new(&game);
    let mut models = ModelCache::default();
    let mut prefabs = PrefabCache::default();
    prefabs.project_root = root;
    let selection = BTreeSet::new();
    let src = Sources {
        map: &loaded.map,
        map_path: Some(&map_path),
        game: &game,
        materials: &mut materials,
        models: &mut models,
        prefabs: &mut prefabs,
        selection: &selection,
    };
    export(src, out, format, &Options::default())
}

/// Copies of what an export reads, so it can run on its own thread while the editor goes on.
struct Snapshot {
    map: Map,
    map_path: Option<PathBuf>,
    game: GameConfig,
    materials: MaterialLibrary,
    models: ModelCache,
    prefabs: PrefabCache,
    selection: BTreeSet<NodeId>,
}

/// An export running on its own thread, so the editor keeps drawing and a cancel can stop it.
pub struct Job {
    pub path: PathBuf,
    pub progress: Arc<Progress>,
    thread: Option<std::thread::JoinHandle<Result<Report, String>>>,
}

impl Job {
    /// Exports the map as it is now to `path`. Edits made while it runs do not reach the file.
    pub fn start(state: &crate::state::EditorState, path: PathBuf, format: Format, options: Options) -> Result<Job, String> {
        let mut snapshot = Snapshot {
            map: state.doc.map.clone(),
            map_path: state.doc.path.clone(),
            game: state.game.clone(),
            materials: state.materials.detached(),
            models: state.models.clone(),
            prefabs: PrefabCache::default(),
            selection: state.doc.selection.nodes.clone(),
        };
        snapshot.prefabs.project_root = state.prefabs.project_root.clone();
        let progress = Arc::new(Progress::default());
        let (shared, out) = (progress.clone(), path.clone());
        let thread = std::thread::Builder::new()
            .name("export".into())
            .spawn(move || {
                let src = Sources {
                    map: &snapshot.map,
                    map_path: snapshot.map_path.as_deref(),
                    game: &snapshot.game,
                    materials: &mut snapshot.materials,
                    models: &mut snapshot.models,
                    prefabs: &mut snapshot.prefabs,
                    selection: &snapshot.selection,
                };
                export_with(src, &out, format, &options, &shared)
            })
            .map_err(|e| format!("cannot start the export: {e}"))?;
        Ok(Job { path, progress, thread: Some(thread) })
    }

    /// The outcome once the export is done, None while it runs.
    pub fn finished(&mut self) -> Option<Result<Report, String>> {
        if !self.thread.as_ref().is_some_and(|t| t.is_finished()) {
            return None;
        }

        let outcome = self.thread.take()?.join();
        Some(outcome.unwrap_or_else(|_| Err("the export stopped on an internal error".into())))
    }
}

/// Whether the node `id` goes into an export with these options: not in a layer left out of the Godot build, visible
/// unless hidden objects are wanted, and selected or inside something selected for a selection export.
pub fn in_scope(map: &Map, id: NodeId, selection: &BTreeSet<NodeId>, options: &Options) -> bool {
    let omitted = matches!(map.get(map.layer_of(id)).map(|n| &n.kind), Some(NodeKind::Layer(l)) if l.omit_from_export);
    let chosen = !options.selection_only || selection.contains(&id) || map.ancestors(id).iter().any(|a| selection.contains(a));
    !omitted && chosen && (options.hidden || !map.is_hidden(id))
}

/// What the options dialog lists before exporting.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub brushes: usize,
    pub meshes: usize,
    pub terrains: usize,
    pub models: usize,
    pub point_entities: usize,
    pub scatter: usize,
    /// Vertices and triangles of the scatter instances when they are exported, which OBJ writes out one by one.
    pub scatter_vertices: usize,
    pub scatter_triangles: usize,
}

/// What the export would hold with these options. The scatter sizes load the models the sets show.
pub fn counts(map: &Map, game: &GameConfig, models: &mut ModelCache, selection: &BTreeSet<NodeId>, options: &Options) -> Counts {
    let mut c = Counts::default();
    for (id, node) in map.nodes.iter() {
        if !in_scope(map, *id, selection, options) {
            continue;
        }

        match &node.kind {
            NodeKind::Brush(_) | NodeKind::Mesh(_) if map.owning_entity(*id).and_then(|e| map.entity(e)).is_some_and(|e| crate::scene::is_volume(game, e)) => {}
            NodeKind::Brush(_) => c.brushes += 1,
            NodeKind::Mesh(_) => c.meshes += 1,
            NodeKind::Terrain(_) => c.terrains += 1,
            NodeKind::Entity(e) if node.children.is_empty() => {
                c.point_entities += 1;
                if crate::models::entity_model_path(game, e).is_some() {
                    c.models += 1;
                }
            }
            NodeKind::Scatter(s) => {
                c.scatter += s.instances.len();
                if !options.scatter {
                    continue;
                }

                let sizes: Vec<(usize, usize)> = s
                    .items
                    .iter()
                    .map(|item| {
                        let model = collect::scatter_model(game, models, &item.source);
                        model.map_or((0, 0), |(_, m)| m.parts.iter().fold((0, 0), |(v, t), p| (v + p.vertices.len(), t + p.indices.len() / 3)))
                    })
                    .collect();
                for (v, t) in s.instances.iter().filter_map(|i| sizes.get(i.item as usize)) {
                    c.scatter_vertices += v;
                    c.scatter_triangles += t;
                }
            }
            _ => {}
        }
    }

    c
}

#[cfg(test)]
mod tests;
