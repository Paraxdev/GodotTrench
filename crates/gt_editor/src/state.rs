use std::path::{Path, PathBuf};
use std::time::Instant;

use gt_core::{Aabb, DVec3, NodeId};
use gt_doc::ops::EditOptions;
use gt_doc::{Document, format};
use gt_formats::GameConfig;
use gt_render::ShadeMode;
use serde::{Deserialize, Serialize};

use crate::materials::MaterialLibrary;
use crate::tools::ToolKind;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shade {
    #[default]
    Textured,
    Flat,
    /// Textured with the map's lights, sun shadows, sky and fog.
    Lit,
    Wireframe,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFilter {
    /// Uses the Godot material's texture_filter, linear when it has none.
    #[default]
    Auto,
    Nearest,
    Linear,
}

impl Shade {
    pub fn label(&self) -> &'static str {
        match self {
            Shade::Textured => "textured",
            Shade::Flat => "flat",
            Shade::Lit => "lit",
            Shade::Wireframe => "wireframe",
        }
    }

    pub const ALL: [Shade; 4] = [Shade::Textured, Shade::Flat, Shade::Lit, Shade::Wireframe];

    pub fn from_name(name: &str) -> Option<Shade> {
        Self::ALL.into_iter().find(|s| s.label().eq_ignore_ascii_case(name))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScatterOutput {
    /// Instances in a scatter set on its own layer, exported as MultiMesh or scene instances.
    #[default]
    Set,
    /// One point entity per instance: model props, or classnames such as enemy spawns.
    Entities,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ScatterSettings {
    /// Weighted palette: models, scenes or (for entity output) classnames.
    pub palette: Vec<gt_doc::ScatterItem>,
    /// Built-in preset the palette came from, empty for custom palettes.
    pub preset: String,
    pub kind: gt_doc::ScatterKind,
    pub radius: f64,
    pub rules: gt_doc::scatter::ScatterRules,
    /// Fraction of the instances under the brush removed per erase dab.
    pub erase_amount: f64,
    /// Erasing only removes instances of the current palette entries.
    pub erase_palette_only: bool,
    /// Keeps spacing against every other scatter set, not only the active one.
    pub avoid_other_sets: bool,
    pub output: ScatterOutput,
    /// Classname used for models placed as entities.
    pub prop_class: String,
}

impl Default for ScatterSettings {
    fn default() -> Self {
        let (kind, palette) = gt_doc::scatter::preset("forest").unwrap_or_default();
        Self {
            palette,
            preset: "forest".into(),
            kind,
            radius: 512.0,
            rules: Default::default(),
            erase_amount: 1.0,
            erase_palette_only: false,
            avoid_other_sets: true,
            output: ScatterOutput::Set,
            prop_class: "prop_model".into(),
        }
    }
}

/// Editing has to pause this long before translated nodes are sent exactly.
const LIVE_IDLE: std::time::Duration = std::time::Duration::from_millis(250);

/// When the live map was last handed to the link.
struct LiveTick {
    revision: u64,
    changed_at: Instant,
    posted: Option<(PathBuf, u64, bool, u64)>,
    /// Maps that received live edits, so dropping their unsaved changes can put Godot back to the saved file.
    sent: std::collections::BTreeSet<PathBuf>,
}

impl Default for LiveTick {
    fn default() -> Self {
        Self { revision: 0, changed_at: Instant::now(), posted: None, sent: Default::default() }
    }
}

/// A map open in a background tab.
pub struct MapTab {
    pub doc: Document,
    pub open_groups: Vec<NodeId>,
    pub current_layer: NodeId,
}

/// How placed models are sized. Real-world-scale assets (many glTF samples are authored in metres) come in
/// as sub-grid specks otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelImportPrefs {
    /// Scale pathologically small or large models to a usable size on placement.
    pub autofit: bool,
    /// Extra size multiplier, applied on top of auto-fit (or on the model's real size when auto-fit is off).
    pub scale: f32,
}

impl Default for ModelImportPrefs {
    fn default() -> Self {
        Self { autofit: true, scale: 1.0 }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub fly_speed: f64,
    pub look_sensitivity: f64,
    pub grid_alpha: f32,
    pub autosave_minutes: f64,
    pub recent_files: Vec<PathBuf>,
    pub recent_projects: Vec<PathBuf>,
    pub shade: Shade,
    pub invert_y: bool,
    pub mcp_http: bool,
    pub mcp_port: u16,
    /// Notify the Godot editor after saving so it rebuilds the map.
    pub live_link: bool,
    pub live_link_port: u16,
    /// Send edits to a Godot editor that has the map open before saving.
    pub live_mode: bool,
    pub godot_path: PathBuf,
    /// Also notify a running game (GodotTrenchHotReload autoload) after saving.
    pub hot_reload: bool,
    pub hot_reload_port: u16,
    /// Base shortcut set: "trenchbroom", "hammer" or "blender".
    pub keymap_preset: String,
    /// Binding id to shortcut text ("Ctrl+Shift+K"), an empty string unbinds.
    pub key_overrides: std::collections::BTreeMap<String, String>,
    pub scatter: ScatterSettings,
    pub model_import: ModelImportPrefs,
    pub texture_filter: TextureFilter,
    /// Materials marked as favourites in the material browser.
    pub favorite_materials: Vec<String>,
    /// Most recently applied materials, newest first.
    pub recent_materials: Vec<String>,
    /// Brush entity class the volume tool creates.
    pub volume_class: String,
    /// Height of volumes drawn in the 3D view.
    pub volume_height: f64,
    /// Size of the whole interface, 1.0 is 100%.
    pub ui_scale: f32,
    /// Multiply `ui_scale` by the monitor's scaling. When off, `ui_scale` is the exact pixels per point.
    pub follow_display_scaling: bool,
    /// The first start welcome with the tour offer was answered.
    pub guide_welcome_seen: bool,
    /// Ids of guide chapters the tour went all the way through.
    pub guide_done: Vec<String>,
    /// Move, rotate and scale handles on the selection in the 3D view.
    pub transform_gizmo: bool,
}

pub const UI_SCALE_MIN: f32 = 0.5;
pub const UI_SCALE_MAX: f32 = 3.0;

impl Prefs {
    pub fn ui_zoom_factor(&self, native_pixels_per_point: f32) -> f32 {
        let scale = self.ui_scale.clamp(UI_SCALE_MIN, UI_SCALE_MAX);
        if self.follow_display_scaling { scale } else { scale / native_pixels_per_point.max(0.25) }
    }

    pub fn step_ui_scale(&mut self, steps: i32) {
        self.ui_scale = (((self.ui_scale * 10.0).round() + steps as f32) / 10.0).clamp(UI_SCALE_MIN, UI_SCALE_MAX);
    }

    /// Switches between following the monitor scaling and fixed pixels, keeping the interface the same size.
    pub fn set_follow_display_scaling(&mut self, follow: bool, native_pixels_per_point: f32) {
        if follow == self.follow_display_scaling {
            return;
        }

        let native = native_pixels_per_point.max(0.25);
        let scale = if follow { self.ui_scale / native } else { self.ui_scale * native };
        self.ui_scale = ((scale * 100.0).round() / 100.0).clamp(UI_SCALE_MIN, UI_SCALE_MAX);
        self.follow_display_scaling = follow;
    }
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            fly_speed: 640.0,
            look_sensitivity: 0.005,
            grid_alpha: 0.5,
            autosave_minutes: 3.0,
            recent_files: Vec::new(),
            recent_projects: Vec::new(),
            shade: Shade::Textured,
            invert_y: false,
            mcp_http: false,
            mcp_port: crate::DEFAULT_MCP_PORT,
            live_link: true,
            live_link_port: crate::live_link::DEFAULT_PORT,
            live_mode: false,
            godot_path: PathBuf::new(),
            hot_reload: true,
            hot_reload_port: crate::live_link::DEFAULT_GAME_PORT,
            keymap_preset: "trenchbroom".into(),
            key_overrides: Default::default(),
            scatter: ScatterSettings::default(),
            model_import: ModelImportPrefs::default(),
            texture_filter: TextureFilter::Auto,
            favorite_materials: Vec::new(),
            recent_materials: Vec::new(),
            volume_class: "trigger_once".into(),
            volume_height: 128.0,
            ui_scale: 1.0,
            follow_display_scaling: true,
            guide_welcome_seen: false,
            guide_done: Vec::new(),
            transform_gizmo: true,
        }
    }
}

pub struct EditorState {
    pub doc: Document,
    pub game: GameConfig,
    pub materials: MaterialLibrary,
    /// The placeable models under `res://models`, shown in the Models panel.
    pub model_library: crate::models::ModelLibrary,
    pub prefs: Prefs,
    pub grid: f64,
    pub snap: bool,
    pub uv_lock: bool,
    pub tool: ToolKind,
    pub current_material: String,
    pub current_layer: NodeId,
    pub open_groups: Vec<NodeId>,
    pub status: String,
    status_time: Instant,
    /// Bounds of the last created or selected brush, used for the depth of brushes drawn in 2D views.
    pub last_bounds: Aabb,
    /// Node picked in a view that the outliner should expand to and scroll into sight.
    pub outliner_reveal: Option<NodeId>,
    /// World position under the mouse in the last hovered viewport.
    pub cursor_world: Option<DVec3>,
    pub focus_request: Option<Aabb>,
    pub hollow_thickness: f64,
    last_autosave: Instant,
    autosave_revision: u64,
    live_link_status: Option<std::sync::mpsc::Receiver<String>>,
    pub godot: crate::godot::GodotStatus,
    /// Started by the app, tests and tools run without it.
    pub link: Option<crate::live_link::LiveLink>,
    /// Link state as of this frame.
    pub link_state: crate::live_link::LinkState,
    live: LiveTick,
    pub sculpt: gt_doc::terrain::SculptBrush,
    pub paint_color: [f32; 4],
    pub prefabs: crate::prefabs::PrefabCache,
    /// World bounds of every instance node, refreshed by the scene cache.
    pub instance_bounds: std::collections::HashMap<NodeId, Aabb>,
    pub models: crate::models::ModelCache,
    /// World bounds of point entities drawn with a model.
    pub model_bounds: std::collections::HashMap<NodeId, Aabb>,
    /// Maps open in other tabs. The active map lives in `doc`.
    pub tabs: Vec<MapTab>,
    pub active_tab: usize,
    /// Set when the renderer should drop cached scene data (tab switch, project change).
    pub scene_reset: bool,
    /// Set when uploaded materials must be dropped and loaded again (filter change, reload).
    pub material_reload: bool,
    /// Material and alignment picked with the texture tool's eyedropper.
    pub uv_clipboard: Option<crate::texture_ops::UvClipboard>,
    /// Justify selected faces against their combined extent.
    pub treat_as_one: bool,
    /// The UV editor panel is open, so a ctrl+click in a view selects all of a brush's faces.
    pub uv_panel_open: bool,
    /// Scatter set the scatter tool paints into. A new one is created on its own layer when unset.
    pub active_scatter: Option<NodeId>,
    pub blend: gt_doc::blend::BlendBrush,
    /// Live move in progress: the dragged nodes and their offset from the drag start. The scene renders
    /// these from their pre-drag geometry translated on the GPU, skipping the per-frame rebuild.
    pub drag_preview: Option<DragPreview>,
}

#[derive(Clone, Default)]
pub struct DragPreview {
    pub nodes: std::collections::BTreeSet<NodeId>,
    pub offset: DVec3,
}

impl EditorState {
    pub fn new(prefs: Prefs) -> Self {
        let game = GameConfig::builtin();
        let materials = MaterialLibrary::new(&game);
        let model_library = crate::models::ModelLibrary::new(&game);
        let doc = Document::new();
        let layer = doc.map.default_layer();
        Self {
            doc,
            game,
            materials,
            model_library,
            prefs,
            grid: 16.0,
            snap: true,
            uv_lock: true,
            tool: ToolKind::Select,
            current_material: "dev/grey".into(),
            current_layer: layer,
            open_groups: Vec::new(),
            status: "Welcome to GodotTrench".into(),
            status_time: Instant::now(),
            last_bounds: Aabb::new(DVec3::ZERO, DVec3::splat(64.0)),
            outliner_reveal: None,
            cursor_world: None,
            focus_request: None,
            hollow_thickness: 16.0,
            last_autosave: Instant::now(),
            autosave_revision: 0,
            live_link_status: None,
            godot: Default::default(),
            link: None,
            link_state: Default::default(),
            live: LiveTick::default(),
            sculpt: gt_doc::terrain::SculptBrush::default(),
            paint_color: [0.55, 0.45, 0.35, 1.0],
            prefabs: Default::default(),
            instance_bounds: Default::default(),
            models: Default::default(),
            model_bounds: Default::default(),
            tabs: Vec::new(),
            active_tab: 0,
            scene_reset: false,
            material_reload: false,
            uv_clipboard: None,
            treat_as_one: false,
            uv_panel_open: false,
            active_scatter: None,
            blend: Default::default(),
            drag_preview: None,
        }
    }

    pub fn opts(&self) -> EditOptions {
        EditOptions { uv_lock: self.uv_lock, grid: self.effective_grid() }
    }

    pub fn effective_grid(&self) -> f64 {
        if self.snap { self.grid } else { 0.0 }
    }

    pub fn snap(&self, v: DVec3) -> DVec3 {
        if self.snap { gt_core::snap_vec_to_grid(v, self.grid) } else { v }
    }

    pub fn snap_scalar(&self, v: f64) -> f64 {
        if self.snap { gt_core::snap_to_grid(v, self.grid) } else { v }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status = msg.into();
        self.status_time = Instant::now();
    }

    pub fn status_age(&self) -> f32 {
        self.status_time.elapsed().as_secs_f32()
    }

    /// Parent for newly created objects: the innermost open group, or the current layer.
    pub fn insert_parent(&self) -> NodeId {
        for g in self.open_groups.iter().rev() {
            if self.doc.map.contains(*g) {
                return *g;
            }
        }

        if self.doc.map.contains(self.current_layer) { self.current_layer } else { self.doc.map.default_layer() }
    }

    pub fn shade_mode(&self) -> ShadeMode {
        match self.prefs.shade {
            Shade::Textured => ShadeMode::Textured,
            Shade::Flat => ShadeMode::Flat,
            Shade::Lit => ShadeMode::Lit,
            Shade::Wireframe => ShadeMode::Wireframe,
        }
    }

    /// Remembers a material as recently used.
    pub fn note_material(&mut self, name: &str) {
        let recent = &mut self.prefs.recent_materials;
        recent.retain(|m| m != name);
        recent.insert(0, name.to_string());
        recent.truncate(16);
    }

    /// Titles of all open maps in tab order, with the index of the active one.
    pub fn tab_titles(&self) -> (Vec<String>, usize) {
        let mut titles: Vec<String> = self.tabs.iter().map(|t| t.doc.title()).collect();
        titles.insert(self.active_tab.min(titles.len()), self.doc.title());
        (titles, self.active_tab.min(self.tabs.len()))
    }

    fn take_active(&mut self) -> MapTab {
        MapTab { doc: std::mem::take(&mut self.doc), open_groups: std::mem::take(&mut self.open_groups), current_layer: self.current_layer }
    }

    fn activate(&mut self, mut all: Vec<MapTab>, index: usize) {
        let index = index.min(all.len() - 1);
        let target = all.remove(index);
        self.doc = target.doc;
        self.open_groups = target.open_groups;
        self.current_layer = target.current_layer;
        self.tabs = all;
        self.active_tab = index;
        self.scene_reset = true;
    }

    /// Makes tab `index` (in tab order) the active map.
    pub fn switch_tab(&mut self, index: usize) {
        if index > self.tabs.len() || index == self.active_tab {
            return;
        }

        let mut all = std::mem::take(&mut self.tabs);
        let current = self.take_active();
        all.insert(self.active_tab.min(all.len()), current);
        self.activate(all, index);
        self.set_status(format!("Switched to {}", self.doc.title()));
    }

    /// Opens a new tab holding `doc` and switches to it.
    pub fn open_tab(&mut self, doc: Document) {
        let mut all = std::mem::take(&mut self.tabs);
        let current = self.take_active();
        all.insert(self.active_tab.min(all.len()), current);
        let layer = doc.map.default_layer();
        all.push(MapTab { doc, open_groups: Vec::new(), current_layer: layer });
        let last = all.len() - 1;
        self.activate(all, last);
        let bounds = self.doc.map.bounds_of(self.doc.map.layers.clone());
        if !bounds.is_empty() {
            self.focus_request = Some(bounds);
        }
    }

    /// Closes the active tab, activating a neighbour. Returns false when it is the last map.
    pub fn close_tab(&mut self) -> bool {
        if self.tabs.is_empty() {
            return false;
        }

        let all = std::mem::take(&mut self.tabs);
        let index = self.active_tab.min(all.len() - 1);
        self.activate(all, index);
        true
    }

    pub fn reset_document(&mut self, doc: Document) {
        if self.doc.is_modified()
            && let Some(path) = self.doc.path.clone()
        {
            self.revert_live(vec![path], false);
        }

        self.current_layer = doc.map.default_layer();
        self.open_groups.clear();
        self.doc = doc;
        self.scene_reset = true;
        let bounds = self.doc.map.bounds_of(self.doc.map.layers.clone());
        if !bounds.is_empty() {
            self.focus_request = Some(bounds);
        }
    }

    pub fn open_map(&mut self, path: &Path) -> Result<(), String> {
        let doc = load_document(path)?;
        self.reset_document(doc);
        self.after_open(path);
        Ok(())
    }

    /// Recent files, project and status for a map just opened from `path`.
    pub fn after_open(&mut self, path: &Path) {
        if let Some(map_path) = self.doc.path.clone() {
            self.add_recent(&map_path);
        }

        if let Some(root) = gt_formats::game::find_project_root(path)
            && self.game.project_root.as_deref() != Some(root.as_path())
        {
            self.load_project(&root);
        }

        match (&self.doc.recovered_from, &self.doc.path) {
            (Some(_), Some(map_path)) => self.set_status(format!("Recovered {} from its autosave, save to keep the changes", map_path.display())),
            (Some(_), None) => self.set_status("Recovered an untitled map from its autosave, save it to keep the changes"),
            _ => self.set_status(format!("Opened {}", path.display())),
        }
    }

    pub fn save_map(&mut self, path: &Path) -> Result<(), String> {
        if path.exists() {
            let _ = std::fs::copy(path, path.with_extension("gtm.bak"));
        }

        format::save(&self.doc.map, path).map_err(|e| e.to_string())?;
        self.doc.path = Some(path.to_path_buf());
        self.doc.mark_saved();
        self.add_recent(path);
        for autosave in [autosave_path(path), legacy_autosave_path(path)] {
            let _ = std::fs::remove_file(autosave);
        }

        if let Some(map_path) = autosave_source(path) {
            self.set_status(format!(
                "Saved into the autosave file {}, Godot uses {}, Save As to write the real map",
                path.display(),
                map_path.file_name().unwrap_or_default().to_string_lossy()
            ));
            return Ok(());
        }

        self.set_status(format!("Saved {}", path.display()));
        self.live.sent.remove(path);
        if self.prefs.live_link {
            let game_port = self.prefs.hot_reload.then_some(self.prefs.hot_reload_port);
            match &self.link {
                Some(link) if self.link_state.connected => {
                    let live = self.live_active().then(|| self.doc.map.clone());
                    link.request(crate::live_link::Request::MapSaved { path: crate::live_link::godot_path(path), game_port, live })
                }
                _ => self.live_link_status = Some(crate::live_link::notify_saved_all(self.prefs.live_link_port, game_port, path)),
            }
        }

        Ok(())
    }

    /// Picks up the asynchronous live link result, if one arrived.
    pub fn poll_live_link(&mut self) {
        if let Some(rx) = &self.live_link_status
            && let Ok(msg) = rx.try_recv()
        {
            self.live_link_status = None;
            self.set_status(msg);
        }
    }

    /// Godot has this project open.
    pub fn godot_has_project(&self) -> bool {
        self.game.project_root.as_deref().is_some_and(|root| self.link_state.has_project(root))
    }

    /// Edits of the current map are being sent to Godot.
    pub fn live_active(&self) -> bool {
        self.prefs.live_link && self.prefs.live_mode && self.godot_has_project() && self.doc.path.as_deref().is_some_and(|p| self.link_state.has_map(p))
    }

    /// Per frame: executable detection, link settings, link status messages and live edits.
    pub fn tick_godot(&mut self, ctx: &egui::Context) {
        self.godot.refresh(&self.prefs.godot_path, self.game.project_root.as_deref());
        let Some(link) = &self.link else { return };
        link.configure(self.prefs.live_link_port, self.prefs.live_link && self.game.project_root.is_some());
        let state = link.state();
        let mut messages = Vec::new();
        while let Some(msg) = link.poll_status() {
            messages.push(msg);
        }

        self.link_state = state;
        for msg in messages {
            self.set_status(msg);
        }

        if self.doc.revision != self.live.revision {
            self.live.revision = self.doc.revision;
            self.live.changed_at = Instant::now();
        }

        let Some(path) = self.doc.path.clone().filter(|_| self.live_active()) else {
            self.live.posted = None;
            return;
        };
        let since_change = self.live.changed_at.elapsed();
        let idle = since_change >= LIVE_IDLE && !self.doc.in_transaction();
        if !idle {
            ctx.request_repaint_after(LIVE_IDLE.saturating_sub(since_change) + std::time::Duration::from_millis(10));
        }

        let key = (path.clone(), self.doc.revision, idle, self.link_state.resync);
        if self.live.posted.as_ref() == Some(&key) {
            return;
        }

        if let Some(link) = &self.link {
            link.sync(&path, crate::live_sync::Frame { map: self.doc.map.clone(), dragging: self.doc.in_transaction(), idle });
        }

        if self.doc.is_modified() {
            self.live.sent.insert(path);
        }

        self.live.posted = Some(key);
    }

    /// Puts Godot back to the saved files before the editor quits without saving. Waits briefly for Godot.
    pub fn revert_all_live_changes(&mut self) {
        let modified = std::iter::once(&self.doc).chain(self.tabs.iter().map(|t| &t.doc)).filter(|d| d.is_modified()).filter_map(|d| d.path.clone()).collect();
        self.revert_live(modified, true);
    }

    /// Godot goes back to the saved files of maps whose live edits are thrown away.
    fn revert_live(&mut self, paths: Vec<PathBuf>, blocking: bool) {
        for path in paths.into_iter().filter(|p| self.live.sent.remove(p)) {
            let path = crate::live_link::godot_path(&path);
            if blocking {
                let message = serde_json::json!({ "event": "live_end", "path": path, "revert": true });
                let _ = crate::live_link::send_timeout(self.prefs.live_link_port, &message, std::time::Duration::from_secs(2));
            } else if let Some(link) = &self.link {
                link.request(crate::live_link::Request::LiveEnd { path, revert: true });
            }
        }
    }

    fn add_recent(&mut self, path: &Path) {
        let p = path.to_path_buf();
        self.prefs.recent_files.retain(|r| r != &p);
        self.prefs.recent_files.insert(0, p);
        self.prefs.recent_files.truncate(12);
    }

    /// Loads the game config of a Godot project, falling back to built-in definitions.
    pub fn load_project(&mut self, root: &Path) {
        let mut game = match GameConfig::discover(root) {
            Some(path) => match GameConfig::load(&path) {
                Ok(g) => {
                    self.set_status(format!("Loaded game config {}", path.display()));
                    g
                }
                Err(e) => {
                    self.set_status(format!("Failed to load {}: {e}", path.display()));
                    GameConfig::builtin()
                }
            },
            None => {
                self.set_status(format!(
                    "No {} in project, using built-in entities. Export it from the FuncGodot dock in Godot.",
                    gt_formats::game::GAME_FILE_NAME
                ));
                GameConfig::builtin()
            }
        };
        game.project_root = Some(root.to_path_buf());
        self.materials.rescan(&game);
        self.model_library.rescan(&game);
        self.game = game;
        let p = root.to_path_buf();
        self.prefs.recent_projects.retain(|r| r != &p);
        self.prefs.recent_projects.insert(0, p);
        self.prefs.recent_projects.truncate(8);
    }

    pub fn tick_autosave(&mut self) {
        if self.prefs.autosave_minutes <= 0.0 || !self.doc.is_modified() || self.doc.in_transaction() {
            return;
        }

        if self.last_autosave.elapsed().as_secs_f64() < self.prefs.autosave_minutes * 60.0 || self.autosave_revision == self.doc.revision {
            return;
        }

        self.last_autosave = Instant::now();
        self.autosave_revision = self.doc.revision;
        let path = match &self.doc.path {
            Some(p) => autosave_path(p),
            None => std::env::temp_dir().join(UNTITLED_AUTOSAVE),
        };
        if format::save(&self.doc.map, &path).is_ok() {
            self.set_status(format!("Autosaved to {}", path.display()));
        }
    }
}

const UNTITLED_AUTOSAVE: &str = "godottrench_untitled.gtm.autosave";

// Autosaves must not end in .gtm: Godot would import them and they would sit next to the real map in file dialogs.
pub fn autosave_path(path: &Path) -> PathBuf {
    let name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "map.gtm".into());
    path.with_file_name(format!("{name}.autosave"))
}

fn legacy_autosave_path(path: &Path) -> PathBuf {
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "map".into());
    path.with_file_name(format!("{stem}.autosave.gtm"))
}

/// The map an autosave belongs to, for both `level.gtm.autosave` and the older `level.autosave.gtm`.
pub fn autosave_source(path: &Path) -> Option<PathBuf> {
    let name = path.file_name()?.to_string_lossy().into_owned();
    let lower = name.to_lowercase();
    let stem_len = if lower.ends_with(".gtm.autosave") {
        name.len() - ".gtm.autosave".len()
    } else if lower.ends_with(".autosave.gtm") {
        name.len() - ".autosave.gtm".len()
    } else {
        return None;
    };
    Some(path.with_file_name(format!("{}.gtm", name.get(..stem_len)?)))
}

/// Loads a map, turning an autosave into an unsaved recovery of the map it belongs to.
pub fn load_document(path: &Path) -> Result<Document, String> {
    let map = format::load(path).map_err(|e| e.to_string())?;
    let Some(source) = autosave_source(path) else {
        return Ok(Document::from_map(map, Some(path.to_path_buf())));
    };
    let untitled = source.file_stem().is_some_and(|s| s == "godottrench_untitled");
    let mut doc = Document::from_map(map, (!untitled).then_some(source));
    doc.recovered_from = Some(path.to_path_buf());
    doc.mark_unsaved();
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn autosaves_recover_into_their_map() {
        let map = Path::new("maps/church.gtm");
        assert_eq!(autosave_path(map), Path::new("maps/church.gtm.autosave"));
        assert_eq!(autosave_source(&autosave_path(map)).as_deref(), Some(map));
        assert_eq!(autosave_source(&legacy_autosave_path(map)).as_deref(), Some(map));
        assert_eq!(autosave_source(map), None);

        let dir = std::env::temp_dir().join(format!("gt_autosave_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let map_path = dir.join("church.gtm");
        let mut state = EditorState::new(Prefs::default());
        state.prefs.live_link = false;
        format::save(&state.doc.map, &autosave_path(&map_path)).unwrap();
        format::save(&state.doc.map, &legacy_autosave_path(&map_path)).unwrap();

        state.open_map(&legacy_autosave_path(&map_path)).unwrap();
        assert_eq!(state.doc.path.as_deref(), Some(map_path.as_path()));
        assert!(state.doc.is_modified());
        assert_eq!(state.doc.title(), "church.gtm* (recovered)");
        assert_eq!(state.prefs.recent_files.first(), Some(&map_path));

        state.save_map(&map_path.clone()).unwrap();
        assert!(map_path.exists());
        assert!(!autosave_path(&map_path).exists() && !legacy_autosave_path(&map_path).exists());
        assert_eq!(state.doc.title(), "church.gtm");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn ui_scale_follows_or_overrides_display_scaling() {
        let mut p = Prefs { ui_scale: 1.25, ..Prefs::default() };
        assert_eq!(p.ui_zoom_factor(2.0), 1.25);
        p.set_follow_display_scaling(false, 2.0);
        assert_eq!(p.ui_scale, 2.5);
        assert_eq!(p.ui_zoom_factor(2.0), 1.25);
        p.set_follow_display_scaling(true, 2.0);
        assert_eq!(p.ui_scale, 1.25);
        p.ui_scale = 2.95;
        p.step_ui_scale(1);
        assert_eq!(p.ui_scale, UI_SCALE_MAX);
        p.ui_scale = 1.0;
        p.step_ui_scale(-1);
        assert_eq!(p.ui_scale, 0.9);
        let old: Prefs = serde_json::from_str(r#"{"fly_speed": 100.0}"#).unwrap();
        assert_eq!((old.ui_scale, old.follow_display_scaling), (1.0, true));
    }
}
