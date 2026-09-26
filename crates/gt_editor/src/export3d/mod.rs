//! Exports a map as a 3D model for Blender and other 3D tools, as glTF binary (.glb) or Wavefront OBJ. Only what the
//! Godot build draws goes out: brushes, meshes, terrains and models with their materials. Entity logic, I/O wiring,
//! triggers and scripts stay behind, and the file cannot be opened as a map again.

mod collect;
pub mod dialog;
mod glb;
mod obj;
mod textures;

use std::collections::BTreeSet;
use std::path::Path;

use gt_core::NodeId;
use gt_doc::{Map, NodeKind};
use gt_formats::GameConfig;

use crate::materials::MaterialLibrary;
use crate::models::ModelCache;
use crate::prefabs::PrefabCache;

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
            "Exported {} objects, {} materials and {} triangles to {} ({:.1} MB). Entity logic, triggers and scripts are not included",
            self.objects,
            self.materials,
            self.triangles,
            path.display(),
            self.bytes as f64 / 1_048_576.0
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

/// Writes the export to `path`. OBJ also writes `<name>.mtl` and the textures into a `<name>_textures` folder next to it.
pub fn export(src: Sources, path: &Path, format: Format, options: &Options) -> Result<Report, String> {
    let name = src.map_path.and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "map".into());
    let scene = collect::collect(src, options, name);
    if scene.roots.is_empty() {
        return Err(if options.selection_only { "nothing to export in the selection" } else { "the map has nothing to export" }.into());
    }

    let bytes = match format {
        Format::Glb => {
            let data = glb::write(&scene);
            std::fs::write(path, &data).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
            data.len() as u64
        }
        Format::Obj => obj::write(&scene, path)?,
    };
    Ok(Report {
        objects: scene.nodes.len(),
        meshes: scene.meshes.len(),
        materials: scene.materials.list.len(),
        images: scene.materials.images.len(),
        triangles: scene.triangles(),
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
}

pub fn counts(map: &Map, game: &GameConfig, selection: &BTreeSet<NodeId>, options: &Options) -> Counts {
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
            NodeKind::Scatter(s) => c.scatter += s.instances.len(),
            _ => {}
        }
    }

    c
}

#[cfg(test)]
mod tests;
