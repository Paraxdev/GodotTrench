use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use td_core::{Aabb, DVec3, NodeId};
use td_doc::ops::EditOptions;
use td_doc::{Document, format};
use td_formats::GameConfig;
use td_render::ShadeMode;

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
    /// Like Lit, with the baked light map on the surfaces it covers.
    Baked,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextureFilter {
    /// Uses the Godot material's texture_filter, linear when it has none.
    #[default]
    Auto,
    Nearest,
    Linear,
}

impl TextureFilter {
    pub fn label(&self) -> &'static str {
        match self {
            TextureFilter::Auto => "Auto (Godot material)",
            TextureFilter::Nearest => "Nearest (pixel art)",
            TextureFilter::Linear => "Linear",
        }
    }
}

impl Shade {
    pub fn label(&self) -> &'static str {
        match self {
            Shade::Textured => "textured",
            Shade::Flat => "flat",
            Shade::Lit => "lit",
            Shade::Wireframe => "wireframe",
            Shade::Baked => "baked",
        }
    }

    /// The name the View > Shading menu, the toolbar, the palette and the keymap show.
    pub fn title(&self) -> &'static str {
        match self {
            Shade::Textured => "Textured",
            Shade::Flat => "Flat",
            Shade::Lit => "Lit Preview",
            Shade::Wireframe => "Wireframe",
            Shade::Baked => "Baked Lighting",
        }
    }

    pub const ALL: [Shade; 5] = [Shade::Textured, Shade::Flat, Shade::Lit, Shade::Baked, Shade::Wireframe];

    /// Lit with the scene's lights, sky and fog, baked or not.
    pub fn is_lit(&self) -> bool {
        matches!(self, Shade::Lit | Shade::Baked)
    }

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
    /// Models a set starts with when a stroke has no set to paint into, or when a script makes one: models,
    /// scenes or (for entity output) classnames. Sets carry their own models once created.
    pub palette: Vec<td_doc::ScatterItem>,
    /// Built-in preset the template came from, empty for custom ones.
    pub preset: String,
    /// Kind of the sets made from the template.
    pub kind: td_doc::ScatterKind,
    pub radius: f64,
    pub rules: td_doc::scatter::ScatterRules,
    /// Fraction of the instances under the brush removed per erase dab.
    pub erase_amount: f64,
    /// Erasing only removes instances of the active set's enabled models.
    pub erase_palette_only: bool,
    /// Keeps spacing against every other scatter set, not only the active one.
    pub avoid_other_sets: bool,
    pub output: ScatterOutput,
    /// Classname used for models placed as entities.
    pub prop_class: String,
    /// Seeds `Scatter::chunk_size` on new sets: cell in map units each set is split into for culling, 0 for one MultiMesh.
    pub chunk_size: f64,
    /// Seeds `Scatter::visibility_range` on new sets, 0 keeps the set visible at any distance.
    pub visibility_range: Option<f64>,
    /// Seeds `Scatter::static_props_multimesh` on new sets.
    pub static_props_multimesh: bool,
    /// Random seed every stroke and fill starts from, 0 for a new pattern each time.
    pub seed: u64,
}

impl Default for ScatterSettings {
    fn default() -> Self {
        let (kind, palette) = td_doc::scatter::preset("forest").unwrap_or_default();
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
            chunk_size: td_doc::scatter::DEFAULT_CHUNK_SIZE,
            visibility_range: None,
            static_props_multimesh: false,
            seed: 0,
        }
    }
}

/// Editing has to pause this long before translated nodes are sent exactly.
const LIVE_IDLE: std::time::Duration = std::time::Duration::from_millis(250);
/// Added to the status when a map from outside the open, trusted project holds code, see [`crate::validate::map_script_issues`].
pub const MAP_SCRIPTS: &str = "The map runs GDScript of its own when the scene plays, the Issues panel shows where";

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
    autosave: Autosave,
}

/// Autosave bookkeeping of one open map.
#[derive(Clone, Debug, Default)]
struct Autosave {
    revision: u64,
    /// Temp file of an untitled map, named once so every untitled tab keeps its own.
    untitled: Option<PathBuf>,
}

impl Autosave {
    fn for_doc(doc: &Document) -> Self {
        Self { revision: 0, untitled: doc.recovered_from.clone().filter(|_| doc.path.is_none()) }
    }
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

/// Why the content wizard should open.
#[derive(Clone, Debug, PartialEq)]
pub enum ContentRequest {
    /// The project at this path opened in TrenchDot for the first time.
    FirstStart(PathBuf),
    /// Something needs the nature pack, which the project lacks. Says what.
    NaturePack(String),
    /// The project at this path opened with the TrenchDot addon missing or not matching the editor.
    Addon(PathBuf),
}

/// Why the trust prompt should open, see [`crate::trust`].
#[derive(Clone, Debug, PartialEq)]
pub enum TrustRequest {
    /// The project at this path opened in Restricted Mode.
    Project(PathBuf),
    /// An action that has Godot run the project at this path waits for the person to trust it.
    Run(PathBuf, crate::commands::Action),
    /// A map of the project at `root`, which is not trusted, opened while another project is open. That one stays open
    /// until the person answers.
    Switch { root: PathBuf, map: PathBuf },
}

/// Why the open project is trusted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trust {
    /// The person trusts it, or a folder it lies in, from the trust prompt or by saving a map into it.
    Stored,
    /// Until the editor closes: the project given with --project.
    Session,
    /// Until it closes: it held no code when it opened, see [`crate::code_scan`].
    NoCode,
}

impl TrustRequest {
    /// The project the person is asked about.
    pub fn root(&self) -> &Path {
        match self {
            TrustRequest::Project(root) | TrustRequest::Run(root, _) | TrustRequest::Switch { root, .. } => root,
        }
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
    /// Which baked map the Baked shading shows.
    pub bake_view: crate::bake::View,
    pub invert_y: bool,
    /// Agents and scripts may drive the editor over MCP. Off, no MCP server runs. Saved as
    /// [`crate::mcp::switch::OFF_FILE`].
    #[serde(skip)]
    pub mcp_enabled: bool,
    pub mcp_http: bool,
    pub mcp_port: u16,
    /// MCP HTTP clients have to send the token kept next to the settings. Saved in a file of its own, see
    /// [`crate::mcp::Saved`].
    #[serde(skip)]
    pub mcp_require_token: bool,
    /// Notify the Godot editor after saving so it rebuilds the map.
    pub live_link: bool,
    pub live_link_port: u16,
    /// Send edits to a Godot editor that has the map open before saving.
    pub live_mode: bool,
    pub godot_path: PathBuf,
    /// Also notify a running game (TrenchDotHotReload autoload) after saving.
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
    /// How much more often a blend material repeats across a face, applied by the Blend tool.
    pub blend_uv_scale: f64,
    /// De-tiling applied to a blend material by the Blend tool, 0 leaves it repeating as authored.
    pub blend_detile: f64,
    /// How crisp the de-tiled blend material stays, 0 softest and 1 crispest.
    pub blend_detile_sharpen: f64,
    /// Size of the whole interface, 1.0 is 100%.
    pub ui_scale: f32,
    /// Multiply `ui_scale` by the monitor's scaling. When off, `ui_scale` is the exact pixels per point.
    pub follow_display_scaling: bool,
    /// Short guidance lines in the tool options bar and the panels. Tooltips keep the details either way.
    pub help_text: bool,
    /// Move, rotate and scale handles on the selection in the 3D view.
    pub transform_gizmo: bool,
    /// Ghost boxes of the Godot overlays built on top of the map, read from its overlay sidecar.
    pub godot_overlays: bool,
    /// Agent the walkable area overlay is baked for.
    pub walk_agent: crate::walkable::Agent,
    /// Show first steps in the views while the map is empty.
    pub start_hints: bool,
    /// Projects, by absolute path, whose content wizard a person has answered or closed. It opens by itself only for
    /// a project that is neither here nor among the recent projects.
    pub content_asked: std::collections::BTreeSet<PathBuf>,
    /// Projects whose missing or mismatched addon a person let be, with the editor version and the addon they had then
    /// ([`crate::addon_install::Status::dismiss_key`]). The addon part of the wizard opens again once either changes.
    pub addon_dismissed: std::collections::BTreeMap<PathBuf, String>,
    /// Roots of the Godot projects the person trusts, as canonical paths. A project unpacked inside one of them has its
    /// own project.godot and is asked about, Godot treats it as a project of its own.
    pub trusted_projects: std::collections::BTreeSet<PathBuf>,
    /// Folders trusted with the parent folder checkbox, as canonical paths. Every project inside them counts as trusted.
    pub trusted_folders: std::collections::BTreeSet<PathBuf>,
    /// The trusted projects were seeded from the recent ones. Preferences saved before project trust existed lack it.
    #[serde(default)]
    pub trust_seeded: bool,
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

    /// Trusts the recent projects and `current` once, when preferences from before project trust load, so the projects
    /// someone already worked in keep opening the way they did.
    pub fn seed_trust(&mut self, current: Option<&Path>) {
        if std::mem::replace(&mut self.trust_seeded, true) {
            return;
        }

        for root in self.recent_projects.iter().map(PathBuf::as_path).chain(current) {
            self.trusted_projects.insert(crate::trust::real_path(root));
        }
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
            bake_view: crate::bake::View::Lit,
            invert_y: false,
            mcp_enabled: true,
            mcp_http: false,
            mcp_port: crate::DEFAULT_MCP_PORT,
            mcp_require_token: false,
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
            blend_uv_scale: 1.0,
            blend_detile: 0.6,
            blend_detile_sharpen: 0.5,
            ui_scale: 1.0,
            follow_display_scaling: true,
            help_text: true,
            transform_gizmo: true,
            godot_overlays: true,
            walk_agent: Default::default(),
            start_hints: true,
            content_asked: Default::default(),
            addon_dismissed: Default::default(),
            trusted_projects: Default::default(),
            trusted_folders: Default::default(),
            trust_seeded: true,
        }
    }
}

pub struct EditorState {
    pub doc: Document,
    pub game: GameConfig,
    pub materials: MaterialLibrary,
    /// The placeable models under `res://models`, shown in the Models panel.
    pub model_library: crate::models::ModelLibrary,
    pub prefab_library: crate::prefab_library::PrefabLibrary,
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
    /// The status line reports that the last command failed or had nothing to do, set by [`Self::fail_status`].
    status_failed: bool,
    /// Bounds of the last created or selected brush, used for the depth of brushes drawn in 2D views.
    pub last_bounds: Aabb,
    /// Node picked in a view that the outliner should expand to and scroll into sight.
    pub outliner_reveal: Option<NodeId>,
    /// Node whose Outliner row shows a name field, started by Rename.
    pub renaming: Option<NodeId>,
    /// Object under the pointer in the outliner, outlined in the views.
    pub outliner_hover: Option<NodeId>,
    /// World position under the mouse in the last hovered viewport.
    pub cursor_world: Option<DVec3>,
    /// A point a little in front of the 3D view's camera, for things placed from a panel with no better spot.
    pub view_focus: Option<DVec3>,
    pub focus_request: Option<Aabb>,
    pub hollow_thickness: f64,
    /// Whether faces CSG Subtract carves take the cutter's or the target's material.
    pub carve_material: td_geom::csg::CarveMaterial,
    /// Brushes the last CSG or clip command replaced, with their replacements, for MCP results.
    pub replaced: td_doc::ops::Replaced,
    last_autosave: Instant,
    autosave: Autosave,
    /// MCP runs over stdin and stdout, so launched programs must not write to them.
    pub stdio_mcp: bool,
    /// Asks the app to open the content wizard.
    pub content_request: Option<ContentRequest>,
    /// Asks the app to show the trust prompt.
    pub trust_request: Option<TrustRequest>,
    /// Projects trusted until the editor closes, canonical: the one given with --project.
    session_trusted: Vec<PathBuf>,
    /// The open project's root while it is trusted, see [`Self::restricted`].
    trusted_root: Option<PathBuf>,
    /// The open project's root when it held no code as it opened.
    no_code_root: Option<PathBuf>,
    /// The code scan the open project's trust waits for.
    open_scan: Option<OpenScan>,
    /// The project of a map just opened and the map, switched to once the scan of the project finds no code.
    switch_scan: Option<(PathBuf, PathBuf)>,
    pub code_scans: crate::code_scan::Scans,
    /// The Gameplay entities pack in the open project, checked when it loads and after an install.
    pub gameplay_pack: crate::entity_pack::Status,
    live_link_status: Option<std::sync::mpsc::Receiver<String>>,
    /// The scan Godot runs for an installed Gameplay pack, until it is done, see [`crate::live_link::scan`].
    pub godot_scan: Option<u64>,
    /// Godot's answer to Run Project, see [`crate::commands::poll_play`].
    pub play_reply: Option<std::sync::mpsc::Receiver<Result<serde_json::Value, String>>>,
    /// Links the system opener found no browser for, sent back from its thread to be copied instead.
    pub(crate) unopened_links: (std::sync::mpsc::Sender<String>, std::sync::mpsc::Receiver<String>),
    pub godot: crate::godot::GodotStatus,
    /// Started by the app, tests and tools run without it.
    pub link: Option<crate::live_link::LiveLink>,
    /// Link state as of this frame.
    pub link_state: crate::live_link::LinkState,
    live: LiveTick,
    pub sculpt: td_doc::terrain::SculptBrush,
    /// World height Auto Paint measures its bands from, such as sea level. None measures from the lowest point.
    pub auto_paint_base: Option<f64>,
    pub paint_color: [f32; 4],
    pub prefabs: crate::prefabs::PrefabCache,
    /// World bounds of every instance node, refreshed by the scene cache.
    pub instance_bounds: std::collections::HashMap<NodeId, Aabb>,
    /// Godot overlay content of the open map, from its `.overlay.json` sidecar.
    pub overlay_ghosts: crate::overlays::OverlayGhosts,
    pub walkable: crate::walkable::WalkableView,
    pub models: crate::models::ModelCache,
    pub model_thumbs: crate::model_thumbs::ModelThumbnails,
    /// World bounds of point entities drawn with a model.
    pub model_bounds: std::collections::HashMap<NodeId, Aabb>,
    /// Maps open in other tabs. The active map lives in `doc`.
    pub tabs: Vec<MapTab>,
    pub active_tab: usize,
    /// Set when the renderer should drop cached scene data (tab switch, project change).
    pub scene_reset: bool,
    /// Set when uploaded materials must be dropped and loaded again (filter change, reload).
    pub material_reload: bool,
    /// Set when the material library found new files, so the scene loads the materials it lacked.
    pub materials_rescanned: bool,
    /// The game config file as it was when the project loaded, compared against the disk to reload it.
    game_file: Option<FileStamp>,
    game_watch: Option<GameWatch>,
    repaint: Option<egui::Context>,
    /// Material and alignment picked with the texture tool's eyedropper.
    pub uv_clipboard: Option<crate::texture_ops::UvClipboard>,
    /// Justify selected faces against their combined extent.
    pub treat_as_one: bool,
    /// The UV editor is the visible tab of its dock group, so a ctrl+click in a view selects all of a brush's faces.
    pub uv_panel_open: bool,
    /// Last rotate, flip, nudge or duplicate, run again by Repeat Last.
    pub last_repeatable: Option<crate::commands::Action>,
    /// Scatter set the scatter tool paints into. A new one is created on its own layer when unset.
    pub active_scatter: Option<NodeId>,
    /// The next click in a view with the scatter tool adds or removes the surface under it as a target.
    pub scatter_eyedropper: bool,
    /// A plain scatter stroke erases, Shift paints.
    pub scatter_erase: bool,
    pub blend: td_doc::blend::BlendBrush,
    /// Live move in progress: the dragged nodes and their offset from the drag start. The scene renders
    /// these from their pre-drag geometry translated on the GPU, skipping the per-frame rebuild.
    pub drag_preview: Option<DragPreview>,
    /// Where the steps of the running run_script may read and write, None outside scripts. Set with
    /// [`Self::fence_script`].
    pub script_fence: Option<crate::mcp::script::Fence>,
}

/// A file with its modification time and size.
type FileStamp = (PathBuf, Option<std::time::SystemTime>, u64);

/// The game config file of a project, None when it has none.
fn game_file_stamp(root: &Path) -> Option<FileStamp> {
    let path = GameConfig::discover(root)?;
    let meta = std::fs::metadata(&path).ok()?;
    Some((path, meta.modified().ok(), meta.len()))
}

/// Godot writes the game config in place, so a read right after it started can find the file cut off. A cut off file
/// only counts as broken once it has stayed that way for a moment.
fn still_written((path, modified, _): &FileStamp) -> bool {
    modified.and_then(|m| m.elapsed().ok()).is_some_and(|age| age.as_secs() < 2)
        && matches!(GameConfig::load(path), Err(td_formats::game::GameError::Json(e)) if e.is_eof())
}

struct OpenScan {
    root: PathBuf,
    /// Asks the person to trust the project when the scan finds code.
    ask: bool,
    /// The trust taken back before the scan, named in the status when the project ends up restricted.
    untrusted: Option<String>,
}

/// Looks at a project's game config once a second on a thread and wakes the UI once the file differs from the one
/// loaded, so a config exported from Godot is picked up without a click.
struct GameWatch {
    stop: Arc<AtomicBool>,
    changed: Arc<AtomicBool>,
}

impl GameWatch {
    fn start(root: PathBuf, loaded: Option<FileStamp>, repaint: egui::Context) -> Self {
        let (stop, changed) = (Arc::new(AtomicBool::new(false)), Arc::new(AtomicBool::new(false)));
        let (stop_flag, changed_flag) = (stop.clone(), changed.clone());
        let _ = std::thread::Builder::new().name("game-config-watch".into()).spawn(move || {
            while !changed_flag.load(Ordering::Relaxed) {
                for _ in 0..10 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    if stop_flag.load(Ordering::Relaxed) {
                        return;
                    }
                }

                if game_file_stamp(&root) != loaded {
                    changed_flag.store(true, Ordering::Relaxed);
                    repaint.request_repaint();
                }
            }
        });
        Self { stop, changed }
    }
}

impl Drop for GameWatch {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
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
            prefab_library: crate::prefab_library::PrefabLibrary::new(None),
            prefs,
            grid: 16.0,
            snap: true,
            uv_lock: true,
            tool: ToolKind::Select,
            current_material: "dev/grey".into(),
            current_layer: layer,
            open_groups: Vec::new(),
            status: "Welcome to TrenchDot".into(),
            status_time: Instant::now(),
            status_failed: false,
            last_bounds: Aabb::new(DVec3::ZERO, DVec3::splat(64.0)),
            outliner_reveal: None,
            renaming: None,
            outliner_hover: None,
            cursor_world: None,
            view_focus: None,
            focus_request: None,
            hollow_thickness: 16.0,
            carve_material: Default::default(),
            replaced: Default::default(),
            last_autosave: Instant::now(),
            autosave: Autosave::default(),
            stdio_mcp: false,
            content_request: None,
            trust_request: None,
            session_trusted: Vec::new(),
            trusted_root: None,
            no_code_root: None,
            open_scan: None,
            switch_scan: None,
            code_scans: crate::code_scan::Scans::default(),
            gameplay_pack: Default::default(),
            live_link_status: None,
            godot_scan: None,
            play_reply: None,
            unopened_links: std::sync::mpsc::channel(),
            godot: Default::default(),
            link: None,
            link_state: Default::default(),
            live: LiveTick::default(),
            sculpt: td_doc::terrain::SculptBrush::default(),
            auto_paint_base: None,
            paint_color: [0.55, 0.45, 0.35, 1.0],
            prefabs: Default::default(),
            instance_bounds: Default::default(),
            overlay_ghosts: Default::default(),
            walkable: Default::default(),
            models: Default::default(),
            model_thumbs: Default::default(),
            model_bounds: Default::default(),
            tabs: Vec::new(),
            active_tab: 0,
            scene_reset: false,
            material_reload: false,
            materials_rescanned: false,
            game_file: None,
            game_watch: None,
            repaint: None,
            uv_clipboard: None,
            treat_as_one: false,
            uv_panel_open: false,
            last_repeatable: None,
            active_scatter: None,
            scatter_eyedropper: false,
            scatter_erase: false,
            blend: Default::default(),
            drag_preview: None,
            script_fence: None,
        }
    }

    pub fn opts(&self) -> EditOptions {
        EditOptions { uv_lock: self.uv_lock, grid: self.effective_grid() }
    }

    pub fn effective_grid(&self) -> f64 {
        if self.snap { self.grid } else { 0.0 }
    }

    pub fn snap(&self, v: DVec3) -> DVec3 {
        if self.snap { td_core::snap_vec_to_grid(v, self.grid) } else { v }
    }

    pub fn snap_scalar(&self, v: f64) -> f64 {
        if self.snap { td_core::snap_to_grid(v, self.grid) } else { v }
    }

    /// Picks up materials written into the project since the last scan when one of `names` is unknown, see
    /// [`MaterialLibrary::find_new`]. Returns true when the library changed.
    pub fn find_new_materials<'a>(&mut self, names: impl IntoIterator<Item = &'a str>) -> bool {
        let found = self.materials.find_new(&self.game, names);
        self.materials_rescanned |= found;
        found
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status = msg.into();
        self.status_time = Instant::now();
        self.status_failed = false;
    }

    /// Shows why a command failed or did nothing. MCP's run_action turns it into an error.
    pub fn fail_status(&mut self, msg: impl Into<String>) {
        self.set_status(msg);
        self.status_failed = true;
    }

    pub fn status_failed(&self) -> bool {
        self.status_failed
    }

    pub fn status_age(&self) -> f32 {
        self.status_time.elapsed().as_secs_f32()
    }

    /// Parent for newly created objects: the innermost open group, or the current layer.
    pub fn insert_parent(&self) -> NodeId {
        let map = &self.doc.map;
        if let Some(g) = self.open_groups.iter().rev().find(|g| is_group(map, **g)) {
            return *g;
        }

        self.valid_layer()
    }

    /// The current layer, or the default layer when the current one is gone.
    pub fn valid_layer(&self) -> NodeId {
        if self.doc.map.layers.contains(&self.current_layer) { self.current_layer } else { self.doc.map.default_layer() }
    }

    pub fn refresh_instance_extents(&mut self) {
        self.prefabs.project_root = self.game.project_root.clone();
        self.prefabs.refresh_extents(&mut self.doc.map, self.doc.path.as_deref());
    }

    /// Keeps the files the editor reads and writes inside `fence` while a run_script runs, prefabs, models and
    /// materials included. None lifts it.
    pub fn fence_script(&mut self, fence: Option<crate::mcp::script::Fence>) {
        self.prefabs.set_fence(fence.clone());
        self.prefab_library.set_fence(fence.clone());
        self.models.set_fence(fence.clone());
        self.materials.set_fence(fence.clone());
        self.script_fence = fence;
    }

    /// Whether the file at `path` may be read, always outside scripts.
    pub fn may_read(&self, path: &Path) -> bool {
        self.script_fence.as_ref().is_none_or(|f| f.may_read(path))
    }

    /// Fails with the reason when a running script may not write the file at `path`.
    pub fn may_write(&self, path: &Path) -> Result<(), String> {
        match &self.script_fence {
            Some(fence) if !fence.may_write(path) => Err(crate::mcp::script::write_refused(path)),
            _ => Ok(()),
        }
    }

    /// Drops open groups and a current layer that no longer exist. Undo restores `next_id`, so a removed layer's id can
    /// come back as a brush and new objects would nest under it.
    pub fn validate_insert_context(&mut self) {
        let map = &self.doc.map;
        self.open_groups.retain(|g| is_group(map, *g));
        self.current_layer = self.valid_layer();
    }

    /// Undo that keeps camera bookmarks, they are view state stored in the map, not edits.
    pub fn undo(&mut self) -> Option<String> {
        let cameras = self.doc.map.editor.cameras.clone();
        let label = self.doc.undo();
        self.doc.map.editor.cameras = cameras;
        self.validate_insert_context();
        label
    }

    pub fn redo(&mut self) -> Option<String> {
        let cameras = self.doc.map.editor.cameras.clone();
        let label = self.doc.redo();
        self.doc.map.editor.cameras = cameras;
        self.validate_insert_context();
        label
    }

    pub fn shade_mode(&self) -> ShadeMode {
        match self.prefs.shade {
            Shade::Textured => ShadeMode::Textured,
            Shade::Flat => ShadeMode::Flat,
            Shade::Lit => ShadeMode::Lit,
            Shade::Wireframe => ShadeMode::Wireframe,
            Shade::Baked => ShadeMode::Baked,
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

    /// The map at `index` in tab order.
    pub fn tab_doc(&self, index: usize) -> &Document {
        let active = self.active_tab.min(self.tabs.len());
        match index.cmp(&active) {
            std::cmp::Ordering::Equal => &self.doc,
            std::cmp::Ordering::Less => &self.tabs[index].doc,
            std::cmp::Ordering::Greater => &self.tabs[index - 1].doc,
        }
    }

    /// Tab order indices of the maps with unsaved changes.
    pub fn modified_tabs(&self) -> Vec<usize> {
        (0..=self.tabs.len()).filter(|i| self.tab_doc(*i).is_modified()).collect()
    }

    fn take_active(&mut self) -> MapTab {
        // A drag still running on the map being left keeps its work as an undo step instead of staying open.
        self.doc.commit();
        MapTab {
            doc: std::mem::take(&mut self.doc),
            open_groups: std::mem::take(&mut self.open_groups),
            current_layer: self.current_layer,
            autosave: std::mem::take(&mut self.autosave),
        }
    }

    fn activate(&mut self, mut all: Vec<MapTab>, index: usize) {
        let index = index.min(all.len() - 1);
        let target = all.remove(index);
        self.doc = target.doc;
        self.open_groups = target.open_groups;
        self.current_layer = target.current_layer;
        self.autosave = target.autosave;
        self.tabs = all;
        self.active_tab = index;
        self.scene_reset = true;
        self.forget_nodes();
        self.validate_insert_context();
    }

    /// Node ids restart per map, so state naming a node of the map being left would point at another map's node.
    fn forget_nodes(&mut self) {
        self.active_scatter = None;
        self.renaming = None;
        self.outliner_reveal = None;
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
        let autosave = Autosave::for_doc(&doc);
        all.push(MapTab { doc, open_groups: Vec::new(), current_layer: layer, autosave });
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

    /// Closes every tab but the active one. Refuses when one has unsaved changes, unless `discard` is true.
    /// Returns how many tabs closed.
    pub fn close_other_tabs(&mut self, discard: bool) -> Result<usize, String> {
        if !discard && let Some(dirty) = self.tabs.iter().find(|t| t.doc.is_modified()) {
            return Err(format!("{} has unsaved changes, save it or pass args.discard true to drop them", dirty.doc.title()));
        }

        let closed = self.tabs.len();
        for tab in std::mem::take(&mut self.tabs) {
            if tab.doc.is_modified()
                && let Some(path) = tab.doc.path
            {
                self.revert_live(vec![path], false);
            }
        }

        self.active_tab = 0;
        Ok(closed)
    }

    /// Closes the active tab and throws away its unsaved changes. The last open map is replaced by a new one.
    pub fn discard_tab(&mut self) {
        if self.tabs.is_empty() {
            self.reset_document(Document::new());
            return;
        }

        if self.doc.is_modified()
            && let Some(path) = self.doc.path.clone()
        {
            self.revert_live(vec![path], false);
        }

        self.close_tab();
    }

    pub fn reset_document(&mut self, doc: Document) {
        if self.doc.is_modified()
            && let Some(path) = self.doc.path.clone()
        {
            self.revert_live(vec![path], false);
        }

        self.current_layer = doc.map.default_layer();
        self.open_groups.clear();
        self.autosave = Autosave::for_doc(&doc);
        self.doc = doc;
        self.scene_reset = true;
        self.forget_nodes();
        let bounds = self.doc.map.bounds_of(self.doc.map.layers.clone());
        if !bounds.is_empty() {
            self.focus_request = Some(bounds);
        }
    }

    pub fn open_map(&mut self, path: &Path) -> Result<(), String> {
        let path = &absolute(path);
        let doc = load_document(path)?;
        self.reset_document(doc);
        self.after_open(path);
        Ok(())
    }

    /// The project the map at `path` belongs to when that is not the open one. A project nested inside the open one does
    /// not count, Godot ignores those too.
    pub fn other_project(&self, path: &Path) -> Option<PathBuf> {
        let root = td_formats::game::find_project_root(&absolute(path))?;
        let inside = |open: &PathBuf| root.starts_with(open) || crate::trust::real_path(&root).starts_with(crate::trust::real_path(open));
        (!self.game.project_root.as_ref().is_some_and(inside)).then_some(root)
    }

    /// Recent files, project and status for a map just opened from `path`. A map of another project switches to it when
    /// that one is trusted, holds no code or none is open, else the open project stays and the person is asked.
    pub fn after_open(&mut self, path: &Path) {
        let path = &absolute(path);
        if let Some(map_path) = self.doc.path.clone() {
            self.add_recent(&map_path);
        }

        let mut kept = None;
        let mut switching = false;
        if let Some(root) = self.other_project(path) {
            let found = (self.game.project_root.is_some() && !self.is_trusted(&root)).then(|| {
                self.code_scans.check(&root);
                self.code_scans.get(&root)
            });
            match found {
                None | Some(Some(None)) => self.load_project(&root),
                Some(Some(Some(_))) => {
                    self.trust_request = Some(TrustRequest::Switch { root: root.clone(), map: path.clone() });
                    kept = Some(root);
                }
                Some(None) => {
                    self.switch_scan = Some((root, path.clone()));
                    switching = true;
                }
            }
        }

        match (&self.doc.recovered_from, &self.doc.path) {
            _ if !self.doc.load_problems.is_empty() => self.set_status(format!(
                "{} could not be read in full, opened what could be read: {}. Saving keeps only that, the old file is copied to .tdm.bak",
                path.display(),
                self.doc.load_problems.join("; ")
            )),
            (Some(_), Some(map_path)) => self.set_status(format!("Recovered {} from its autosave, save to keep the changes", map_path.display())),
            (Some(_), None) => self.set_status("Recovered an untitled map from its autosave, save it to keep the changes"),
            _ if self.doc.migrated => self.set_status(crate::legacy::migrated_status(path)),
            _ => self.set_status(format!("Opened {}", path.display())),
        }

        // Maps of the open project, while it is trusted, are the person's own. Any other map may come from anyone.
        let inside = |root: &PathBuf| path.starts_with(root) || crate::trust::real_path(path).starts_with(crate::trust::real_path(root));
        let own = !self.restricted() && self.game.project_root.as_ref().is_some_and(inside);
        if let Some(root) = kept {
            let status = format!("{}. It belongs to {}, a project you have not trusted, so the open project stays", self.status, root.display());
            self.set_status(status);
        }

        // Whether the map is the project's own is not known until the scan of the project it belongs to ends.
        if !own && !switching {
            self.note_map_scripts();
        }
    }

    fn note_map_scripts(&mut self) {
        if !crate::validate::map_script_issues(&self.doc.map, &mut self.prefabs, self.doc.path.as_deref()).is_empty() {
            let status = format!("{}. {MAP_SCRIPTS}", self.status);
            self.set_status(status);
        }
    }

    pub fn save_map(&mut self, path: &Path) -> Result<(), String> {
        let path = &absolute(path);
        if path.exists() {
            let _ = std::fs::copy(path, path.with_extension(format!("{}.bak", td_doc::legacy::map_ext(path))));
        }

        format::save(&self.doc.map, path).map_err(|e| e.to_string())?;
        self.doc.path = Some(path.to_path_buf());
        self.doc.mark_saved();
        self.add_recent(path);
        for autosave in [autosave_path(path), legacy_autosave_path(path)] {
            let _ = std::fs::remove_file(autosave);
        }

        if let Some(untitled) = self.autosave.untitled.take().filter(|u| u != path) {
            let _ = std::fs::remove_file(untitled);
        }

        if let Some(folder) = self.prefab_library.folder.clone().filter(|f| path.starts_with(absolute(f))) {
            self.prefab_library.rescan(folder.parent());
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
            let users = self.maps_using(path);
            match &self.link {
                Some(link) if self.link_state.connected => {
                    let live = self.live_active().then(|| self.doc.map.clone());
                    link.request(crate::live_link::Request::MapSaved { path: crate::live_link::godot_path(path), game_port, live, users })
                }
                _ => self.live_link_status = Some(crate::live_link::notify_saved_all(self.prefs.live_link_port, game_port, path)),
            }
        }

        Ok(())
    }

    /// The maps in other tabs that Godot shows and that instance the file at `path`, directly or through other prefabs,
    /// by Godot path and as shown here. None in Restricted Mode, which has Godot build saved files only.
    fn maps_using(&mut self, path: &Path) -> Vec<(String, td_doc::Map)> {
        let mut out = Vec::new();
        if self.restricted() {
            return out;
        }

        for tab in &self.tabs {
            let Some(tab_path) = tab.doc.path.as_deref().filter(|p| self.link_state.has_map(p)) else { continue };
            if self.prefabs.instances(&tab.doc.map, Some(tab_path), path) {
                out.push((crate::live_link::godot_path(tab_path), tab.doc.map.clone()));
            }
        }

        out
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

    /// What to tell while Godot shows entities of the current map without their definition.
    pub fn godot_unresolved(&self) -> Option<String> {
        let report = self.link_state.report(self.doc.path.as_deref()?)?;
        let used: Vec<&String> = report.missing.iter().filter(|c| self.doc.map.entities().any(|(_, e)| e.classname == **c)).collect();
        (!used.is_empty()).then(|| crate::live_link::unresolved_text(&used, report))
    }

    fn note_scan_done(&mut self) {
        if self.godot_scan.is_some_and(|scan| !self.link_state.connected || self.link_state.scanned >= scan) {
            self.godot_scan = None;
        }
    }

    /// Shows a live link message. Until Godot is done scanning for an installed pack it keeps saying so, the next live
    /// reply would hide it otherwise.
    fn link_status(&mut self, msg: String) {
        match self.godot_scan {
            Some(_) => self.set_status(format!("{}. {}", msg.trim_end_matches('.'), crate::entity_pack::SCAN_NOTE)),
            None => self.set_status(msg),
        }
    }

    /// Edits of the current map are being sent to Godot.
    pub fn live_active(&self) -> bool {
        self.prefs.live_link
            && self.prefs.live_mode
            && !self.restricted()
            && self.godot_has_project()
            && self.doc.path.as_deref().is_some_and(|p| self.link_state.has_map(p))
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
        self.note_scan_done();
        for msg in messages {
            self.link_status(msg);
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

    /// The person trusts the authors of the project at `root`, or of a folder it lies in.
    pub fn is_trusted(&self, root: &Path) -> bool {
        let real = crate::trust::real_path(root);
        self.prefs.trusted_projects.iter().chain(&self.session_trusted).any(|t| *t == real) || self.prefs.trusted_folders.iter().any(|t| real.starts_with(t))
    }

    /// The open project is not trusted, so TrenchDot does not start Godot on it or have Godot build its maps.
    pub fn restricted(&self) -> bool {
        self.game.project_root.as_ref().is_some_and(|root| self.trusted_root.as_ref() != Some(root))
    }

    /// Trusts the project at `root` from now on, or with `parent` every project in the folder it is in.
    pub fn trust_project(&mut self, root: &Path, parent: bool) {
        let real = crate::trust::real_path(root);
        match crate::trust::parent_folder(&real).filter(|_| parent).map(Path::to_path_buf) {
            Some(folder) => {
                self.prefs.trusted_projects.retain(|t| !t.starts_with(&folder));
                self.prefs.trusted_folders.retain(|t| !t.starts_with(&folder));
                self.prefs.trusted_folders.insert(folder);
            }
            None => {
                self.prefs.trusted_projects.insert(real);
            }
        }

        self.refresh_trust();
    }

    /// Takes back the stored trust of a project, or with `folder` of a folder and the projects in it. The open project
    /// goes into Restricted Mode at once when nothing else trusts it, unless it holds no code.
    pub fn untrust(&mut self, path: &Path, folder: bool) {
        let was_trusted = !self.restricted();
        let stored = if folder { &mut self.prefs.trusted_folders } else { &mut self.prefs.trusted_projects };
        if !stored.remove(path) {
            return;
        }

        let shown = crate::trust::shown(path);
        self.set_status(format!("{shown} is no longer trusted"));
        if let Some(root) = self.game.project_root.clone().filter(|root| !self.is_trusted(root) && self.no_code_root.as_ref() != Some(root)) {
            self.wait_for_scan(&root, false, was_trusted.then_some(shown));
        }

        self.refresh_trust();
        self.apply_code_scans(false);
    }

    /// Trusts the project at `root` until the editor closes.
    pub fn trust_for_session(&mut self, root: &Path) {
        self.session_trusted.push(crate::trust::real_path(root));
        self.refresh_trust();
    }

    /// Why the open project is trusted, None in Restricted Mode and without a project.
    pub fn trust(&self) -> Option<Trust> {
        let root = self.game.project_root.as_ref().filter(|root| self.trusted_root.as_ref() == Some(*root))?;
        let real = crate::trust::real_path(root);
        Some(if self.prefs.trusted_projects.contains(&real) || self.prefs.trusted_folders.iter().any(|t| real.starts_with(t)) {
            Trust::Stored
        } else if self.session_trusted.contains(&real) {
            Trust::Session
        } else {
            Trust::NoCode
        })
    }

    /// A map saved from the GUI into a project trusted only for holding no code trusts the project from now on, so it
    /// keeps opening trusted once its owner adds scripts in Godot. Saves by an agent never do.
    pub fn trust_after_save(&mut self, path: &Path) {
        if let Some(root) = self.game.project_root.clone().filter(|root| self.trust() == Some(Trust::NoCode) && crate::trust::within(path, root)) {
            self.trust_project(&root, false);
        }
    }

    /// Holds the trust of the open project at `root` until its code scan ends.
    fn wait_for_scan(&mut self, root: &Path, ask: bool, untrusted: Option<String>) {
        let ask = ask || self.open_scan.as_ref().is_some_and(|s| s.root == root && s.ask);
        self.code_scans.check(root);
        self.open_scan = Some(OpenScan { root: root.to_path_buf(), ask, untrusted });
    }

    /// The open project's trust waits for its code scan, so it counts as restricted until then.
    pub fn scanning_code(&self) -> bool {
        self.restricted() && self.open_scan.as_ref().is_some_and(|s| self.game.project_root.as_ref() == Some(&s.root))
    }

    /// Settles what waits for code scans that ended. With `wait` it waits for those still running.
    pub fn apply_code_scans(&mut self, wait: bool) {
        // First, since switching projects starts a scan of the one switched to.
        if let Some((root, map)) = self.switch_scan.clone()
            && let Some(found) = self.scanned(&root, wait)
        {
            self.switch_scan = None;
            // Another map opened since leaves the project as it is.
            let current = self.doc.path.as_deref().map(absolute) == Some(map.clone()) && self.other_project(&map).as_ref() == Some(&root);
            if current && (found.is_none() || self.is_trusted(&root)) {
                self.load_project(&root);
            } else if current {
                self.trust_request = Some(TrustRequest::Switch { root: root.clone(), map: map.clone() });
                self.set_status(format!("{} belongs to {}, a project you have not trusted, so the open project stays", map.display(), root.display()));
                self.note_map_scripts();
            }
        }

        if let Some(root) = self.open_scan.as_ref().map(|s| s.root.clone())
            && let Some(found) = self.scanned(&root, wait)
            && let Some(scan) = self.open_scan.take()
            && self.game.project_root.as_ref() == Some(&root)
        {
            self.no_code_root = (!self.is_trusted(&root) && found.is_none()).then(|| root.clone());
            self.refresh_trust();
            if scan.ask && self.restricted() {
                self.trust_request = Some(TrustRequest::Project(root.clone()));
            }

            if let Some(shown) = scan.untrusted.filter(|_| self.restricted()) {
                self.set_status(format!("{shown} is no longer trusted, so {} is in Restricted Mode now", root.display()));
            }
        }
    }

    fn scanned(&mut self, root: &Path, wait: bool) -> Option<Option<crate::code_scan::Code>> {
        if wait { Some(self.code_scans.wait(root)) } else { self.code_scans.get(root) }
    }

    fn refresh_trust(&mut self) {
        self.trusted_root = self.game.project_root.clone().filter(|root| self.is_trusted(root) || self.no_code_root.as_ref() == Some(root));
    }

    /// Loads the game config of a Godot project, falling back to built-in definitions.
    pub fn load_project(&mut self, root: &Path) {
        self.load_project_with(root, true);
    }

    /// Like [`EditorState::load_project`]. Without `announce` only a failure or an install changes the status.
    fn load_project_with(&mut self, root: &Path, announce: bool) {
        let root = &absolute(root);
        let switched = self.game.project_root.as_ref() != Some(root);
        // Taken before reading, so a file still being written when it was read counts as changed afterwards.
        self.game_file = game_file_stamp(root);
        let mut game = match GameConfig::discover(root) {
            Some(path) => match GameConfig::load(&path) {
                Ok(g) => {
                    if announce {
                        self.set_status(format!("Loaded game config {}", path.display()));
                    }

                    g
                }
                Err(e) => {
                    self.set_status(format!("Failed to load {}: {e}", path.display()));
                    GameConfig::builtin()
                }
            },
            None => {
                self.set_status(format!(
                    "No {} in project, using built-in entities. Export it in Godot with Project > Tools > TrenchDot: Export Game Config.",
                    td_formats::game::GAME_FILE_NAME
                ));
                GameConfig::builtin()
            }
        };
        game.project_root = Some(root.to_path_buf());
        game.addon_version = td_formats::game::addon_version(root);
        crate::entity_pack::add_unexported(&mut game, root);
        self.gameplay_pack = crate::entity_pack::Status::check(root);
        if let Some(addon) = &game.addon_version
            && crate::addon_install::mismatched(addon)
        {
            eprintln!("TrenchDot: info: the addon's plugin.cfg is v{addon}, the editor is v{}; update the addon submodule to match", crate::VERSION);
        }

        self.materials.rescan(&game);
        self.model_library.rescan(&game);
        self.prefab_library.rescan(Some(root));
        let before = self.defined_classes();
        self.game = game;
        // Checked again each time a project opens, but not while it stays open: Godot adds imported files to it then.
        if switched || self.no_code_root.as_ref() != Some(root) {
            self.no_code_root = None;
            if self.is_trusted(root) {
                self.open_scan = None;
            } else {
                self.wait_for_scan(root, switched, None);
            }
        }

        self.forget_godot_missing(&before);
        self.refresh_trust();
        self.apply_code_scans(false);

        self.restart_game_watch();
        let p = root.to_path_buf();
        let known = self.prefs.recent_projects.contains(&p) || self.prefs.content_asked.contains(&p);
        if !known && !crate::content::has_content(&p) {
            self.content_request = Some(ContentRequest::FirstStart(p.clone()));
        } else if crate::addon_install::should_ask(&p, &self.prefs.addon_dismissed) {
            self.content_request = Some(ContentRequest::Addon(p.clone()));
        }

        self.prefs.recent_projects.retain(|r| r != &p);
        self.prefs.recent_projects.insert(0, p);
        self.prefs.recent_projects.truncate(8);
        if let Some(status) = crate::entity_pack::install_for_maps(self) {
            self.set_status(status);
        } else if let Some(clash) = self.game.clashes.first().filter(|_| announce) {
            let more = match self.game.clashes.len() {
                1 => String::new(),
                n => format!(" {} more classnames are defined twice, the Issues panel lists the ones a map uses.", n - 1),
            };
            self.set_status(format!("{}.{more}", clash.describe()));
        } else if announce && self.gameplay_pack.update() {
            self.set_status("The addon has a newer Gameplay entities pack. Gameplay > Update Gameplay Entities installs it and keeps the files you edited.");
        }
    }

    /// Watches the project's game config from now on, waking `repaint` when it changes on disk.
    pub fn watch_game_config(&mut self, repaint: egui::Context) {
        self.repaint = Some(repaint);
        self.restart_game_watch();
    }

    fn restart_game_watch(&mut self) {
        self.game_watch = match (&self.repaint, &self.game.project_root) {
            (Some(repaint), Some(root)) => Some(GameWatch::start(root.clone(), self.game_file.clone(), repaint.clone())),
            _ => None,
        };
    }

    /// The watcher saw the game config file appear, change or go away since the project loaded.
    pub fn game_config_changed(&self) -> bool {
        self.game_watch.as_ref().is_some_and(|w| w.changed.load(Ordering::Relaxed))
    }

    /// Reloads the project when its game config file differs from the one loaded. Returns true when it reloaded.
    pub fn reload_changed_game_config(&mut self) -> bool {
        let Some(root) = self.game.project_root.clone() else { return false };
        let stamp = game_file_stamp(&root);
        if stamp == self.game_file || stamp.as_ref().is_some_and(still_written) {
            if self.game_config_changed() {
                self.restart_game_watch();
            }

            return false;
        }

        // Godot exports the config again after each build, which must not hide what the build reported.
        self.load_project_with(&root, false);
        true
    }

    /// The classes the game config defines, and those its export says the FGD Godot builds with defines.
    pub fn defined_classes(&self) -> [std::collections::BTreeSet<String>; 2] {
        let built = self.game.build.iter().flat_map(|b| b.classes.iter().cloned()).collect();
        [self.game.entities.iter().map(|e| e.classname.clone()).collect(), built]
    }

    /// What Godot reported missing predates the definitions added since `before`, in the game config or in the FGD Godot
    /// builds with.
    pub fn forget_godot_missing(&mut self, before: &[std::collections::BTreeSet<String>; 2]) {
        let now = self.defined_classes();
        let added: std::collections::BTreeSet<String> = now.iter().zip(before).flat_map(|(now, before)| now.difference(before).cloned()).collect();
        if added.is_empty() {
            return;
        }

        self.link_state.forget_missing(&added);
        if let Some(link) = &self.link {
            link.forget_missing(&added);
        }
    }

    /// Autosaves every open map with unsaved changes once the autosave interval has passed.
    pub fn tick_autosave(&mut self) {
        if self.doc.in_transaction() || self.last_autosave.elapsed().as_secs_f64() < self.prefs.autosave_minutes * 60.0 {
            return;
        }

        self.autosave_now();
    }

    /// Autosaves every open map with unsaved changes right away, unless autosave is turned off.
    pub fn autosave_now(&mut self) {
        if self.prefs.autosave_minutes <= 0.0 {
            return;
        }

        self.last_autosave = Instant::now();
        let tabs = self.tabs.iter_mut().map(|t| (&t.doc, &mut t.autosave));
        let (saved, failed): (Vec<_>, Vec<_>) =
            std::iter::once((&self.doc, &mut self.autosave)).chain(tabs).filter_map(|(doc, autosave)| autosave_doc(doc, autosave)).partition(Result::is_ok);
        if let Some(Err(e)) = failed.into_iter().next() {
            self.fail_status(e);
            return;
        }

        match saved.as_slice() {
            [] => {}
            [Ok(one)] => self.set_status(format!("Autosaved to {}", one.display())),
            many => self.set_status(format!("Autosaved {} maps", many.len())),
        }
    }

    /// Untitled map autosaves in the temp folder that no open tab is writing, newest first.
    pub fn recoverable_autosaves(&self) -> Vec<PathBuf> {
        let open: Vec<&PathBuf> = std::iter::once(&self.autosave)
            .chain(self.tabs.iter().map(|t| &t.autosave))
            .filter_map(|a| a.untitled.as_ref())
            .chain(std::iter::once(&self.doc).chain(self.tabs.iter().map(|t| &t.doc)).filter_map(|d| d.recovered_from.as_ref()))
            .collect();
        untitled_autosaves(&std::env::temp_dir()).into_iter().filter(|p| !open.contains(&p)).collect()
    }
}

/// The autosave file written, or why it could not be. A map that fails is not tried again until it changes.
fn autosave_doc(doc: &Document, autosave: &mut Autosave) -> Option<Result<PathBuf, String>> {
    if !doc.is_modified() || doc.in_transaction() || autosave.revision == doc.revision {
        return None;
    }

    let path = match &doc.path {
        Some(p) => autosave_path(p),
        None => autosave.untitled.get_or_insert_with(new_untitled_autosave).clone(),
    };
    autosave.revision = doc.revision;
    Some(format::save(&doc.map, &path).map(|()| path.clone()).map_err(|e| format!("Autosave to {} failed: {e}", path.display())))
}

const UNTITLED_PREFIX: &str = "trenchdot_untitled";

fn new_untitled_autosave() -> PathBuf {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    let n = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!("{UNTITLED_PREFIX}_{}_{n}.tdm.autosave", std::process::id()))
}

/// Autosaves of untitled maps in `dir`, newest first.
pub fn untitled_autosaves(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut found: Vec<(std::time::SystemTime, PathBuf)> = entries
        .flatten()
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            name.starts_with(UNTITLED_PREFIX) && name.ends_with(".tdm.autosave")
        })
        .map(|e| (e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH), e.path()))
        .collect();
    found.sort_by_key(|e| std::cmp::Reverse(e.0));
    found.into_iter().map(|(_, p)| p).collect()
}

fn is_group(map: &td_doc::Map, id: NodeId) -> bool {
    matches!(map.get(id).map(|n| &n.kind), Some(td_doc::NodeKind::Group(_)))
}

// Autosaves must not end in .tdm: Godot would import them and they would sit next to the real map in file dialogs.
pub fn autosave_path(path: &Path) -> PathBuf {
    let name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "map.tdm".into());
    path.with_file_name(format!("{name}.autosave"))
}

fn legacy_autosave_path(path: &Path) -> PathBuf {
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "map".into());
    path.with_file_name(format!("{stem}.autosave.{}", td_doc::legacy::map_ext(path)))
}

/// The map an autosave belongs to, for both `level.tdm.autosave` and the older `level.autosave.tdm`.
pub fn autosave_source(path: &Path) -> Option<PathBuf> {
    let name = path.file_name()?.to_string_lossy().into_owned();
    let lower = name.to_lowercase();
    td_doc::legacy::MAP_EXTS.iter().find_map(|ext| {
        let suffix = [format!(".{ext}.autosave"), format!(".autosave.{ext}")].into_iter().find(|suffix| lower.ends_with(suffix.as_str()))?;
        Some(path.with_file_name(format!("{}.{ext}", name.get(..name.len() - suffix.len())?)))
    })
}

/// Resolves `path` against the working directory once, so the watcher threads never depend on it and a file dialog may
/// change it.
pub fn absolute(path: &Path) -> PathBuf {
    std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Loads a map, turning an autosave into an unsaved recovery of the map it belongs to.
pub fn load_document(path: &Path) -> Result<Document, String> {
    let path = &absolute(path);
    let format::Loaded { map, problems, old_format } = format::load(path).map_err(|e| e.to_string())?;
    for p in &problems {
        eprintln!("{}: {p}", path.display());
    }

    let mut doc = match autosave_source(path) {
        None => Document::from_map(map, Some(path.to_path_buf())),
        Some(source) => {
            let untitled = source.file_stem().is_some_and(|s| s.to_string_lossy().to_lowercase().starts_with(UNTITLED_PREFIX));
            let mut doc = Document::from_map(map, (!untitled).then_some(source));
            doc.recovered_from = Some(path.to_path_buf());
            doc.mark_unsaved();
            doc
        }
    };
    if !problems.is_empty() {
        doc.load_problems = problems;
        doc.mark_unsaved();
    }

    if old_format {
        doc.migrated = true;
        doc.mark_unsaved();
    }

    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_godot_lacks_are_told_while_the_map_uses_them() {
        let mut state = EditorState::new(Prefs::default());
        let path = std::env::temp_dir().join("td_unresolved").join("hall.tdm");
        state.doc.path = Some(path.clone());
        let layer = state.doc.map.default_layer();
        state.doc.edit("Door", |m, _| m.insert(layer, td_doc::NodeKind::Entity(td_doc::Entity::new("func_door"))));
        assert_eq!(state.godot_unresolved(), None);
        let report = crate::live_link::GodotReport { missing: ["func_door", "my_lamp"].map(String::from).into(), ..Default::default() };
        let key = crate::live_link::path_key(&crate::live_link::godot_path(&path));
        state.link_state.reports.insert(key, report);
        assert_eq!(
            state.godot_unresolved().as_deref(),
            Some("Godot has no definition for func_door, it shows a plain node. Install the Gameplay entities pack or add the class to the map settings FGD"),
            "my_lamp is no longer in the map"
        );
    }

    #[test]
    fn saving_a_prefab_has_godot_build_the_open_maps_that_use_it() {
        let dir = std::env::temp_dir().join(format!("td_prefab_users_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let instance = |path: &str| {
            td_doc::NodeKind::Instance(td_doc::map::Instance {
                path: path.into(),
                origin: Default::default(),
                angles: Default::default(),
                fixup: String::new(),
            })
        };
        let mut nest = td_doc::Map::new();
        let layer = nest.default_layer();
        nest.insert(layer, instance("crate.tdm"));
        format::save(&nest, &dir.join("nest.tdm")).unwrap();

        let mut state = EditorState::new(Prefs::default());
        state.link = Some(crate::live_link::LiveLink::idle());
        let key = |name: &str| crate::live_link::path_key(&crate::live_link::godot_path(&dir.join(name)));
        state.link_state = crate::live_link::LinkState {
            connected: true,
            maps: [(key("direct.tdm"), 1), (key("nested.tdm"), 1), (key("other.tdm"), 1)].into(),
            ..Default::default()
        };
        for (name, uses) in [("direct.tdm", "crate.tdm"), ("nested.tdm", "nest.tdm"), ("other.tdm", "barrel.tdm"), ("hidden.tdm", "crate.tdm")] {
            let mut doc = Document::new();
            let layer = doc.map.default_layer();
            doc.map.insert(layer, instance(uses));
            doc.path = Some(dir.join(name));
            state.open_tab(doc);
        }

        state.open_tab(Document::new());
        state.save_map(&dir.join("crate.tdm")).unwrap();
        let requests = state.link.as_ref().unwrap().take_requests();
        let [crate::live_link::Request::MapSaved { users, .. }] = &requests[..] else { panic!("one save request") };
        let names: Vec<&str> = users.iter().map(|(p, _)| p.rsplit('/').next().unwrap()).collect();
        assert_eq!(names, ["direct.tdm", "nested.tdm"], "not a map using another prefab, nor one Godot does not show");

        state.game.project_root = Some(dir.clone());
        assert!(state.restricted());
        state.save_map(&dir.join("crate.tdm")).unwrap();
        let requests = state.link.as_ref().unwrap().take_requests();
        let [crate::live_link::Request::MapSaved { users, .. }] = &requests[..] else { panic!("one save request") };
        assert!(users.is_empty(), "Restricted Mode does not have Godot build maps from the editor");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn autosaves_recover_into_their_map() {
        let map = Path::new("maps/church.tdm");
        assert_eq!(autosave_path(map), Path::new("maps/church.tdm.autosave"));
        assert_eq!(autosave_source(&autosave_path(map)).as_deref(), Some(map));
        assert_eq!(autosave_source(&legacy_autosave_path(map)).as_deref(), Some(map));
        assert_eq!(autosave_source(map), None);

        let dir = std::env::temp_dir().join(format!("td_autosave_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let map_path = dir.join("church.tdm");
        let mut state = EditorState::new(Prefs::default());
        state.prefs.live_link = false;
        format::save(&state.doc.map, &autosave_path(&map_path)).unwrap();
        format::save(&state.doc.map, &legacy_autosave_path(&map_path)).unwrap();

        state.open_map(&legacy_autosave_path(&map_path)).unwrap();
        assert_eq!(state.doc.path.as_deref(), Some(map_path.as_path()));
        assert!(state.doc.is_modified());
        assert_eq!(state.doc.title(), "church.tdm* (recovered)");
        assert_eq!(state.prefs.recent_files.first(), Some(&map_path));

        state.save_map(&map_path.clone()).unwrap();
        assert!(map_path.exists());
        assert!(!autosave_path(&map_path).exists() && !legacy_autosave_path(&map_path).exists());
        assert_eq!(state.doc.title(), "church.tdm");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_map_of_the_old_format_opens_modified_and_saves_in_the_current_one() {
        let dir = std::env::temp_dir().join(format!("td_old_format_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("room.{}", td_doc::legacy::MAP_EXTS[1]));
        let mut map = td_doc::Map::new();
        let layer = map.default_layer();
        td_doc::ops::create_point_entity(&mut map, layer, "light", DVec3::ZERO);
        let mut value: serde_json::Value = serde_json::from_str(&format::to_string(&map)).unwrap();
        value["format"] = td_doc::legacy::MAP_FORMAT.into();
        std::fs::write(&path, value.to_string()).unwrap();

        let mut state = EditorState::new(Prefs::default());
        state.prefs.live_link = false;
        state.open_map(&path).unwrap();
        assert!(state.doc.is_modified() && state.doc.migrated);
        assert_eq!(state.status, format!("Updated room.{} from the {} format, save to keep it", td_doc::legacy::MAP_EXTS[1], td_doc::legacy::NAME));
        assert_eq!(state.doc.map.entities().count(), 1);

        state.save_map(&path).unwrap();
        assert!(!state.doc.is_modified() && !state.doc.migrated);
        let (saved, _) = format::file_json_value(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved["format"], format::FORMAT_NAME);
        state.open_map(&path).unwrap();
        assert!(!state.doc.is_modified());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn an_old_map_keeps_its_extension_through_a_save_and_its_autosave() {
        let old = td_doc::legacy::MAP_EXTS[1];
        let map = Path::new("maps").join(format!("church.{old}"));
        assert_eq!(autosave_path(&map), Path::new("maps").join(format!("church.{old}.autosave")));
        assert_eq!(autosave_source(&autosave_path(&map)).as_deref(), Some(map.as_path()));
        assert_eq!(autosave_source(&legacy_autosave_path(&map)).as_deref(), Some(map.as_path()));
        assert_eq!(legacy_autosave_path(&map), Path::new("maps").join(format!("church.autosave.{old}")));

        let dir = std::env::temp_dir().join(format!("td_old_ext_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("hall.{old}"));
        let mut state = EditorState::new(Prefs::default());
        state.prefs.live_link = false;
        state.save_map(&path).unwrap();
        state.save_map(&path).unwrap();
        assert!(dir.join(format!("hall.{old}.bak")).is_file() && !dir.join("hall.tdm.bak").exists());
        assert!(!dir.join(format!("hall.{old}.tmp")).exists());
        assert_eq!(state.doc.path.as_deref(), Some(path.as_path()));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn every_modified_tab_autosaves_and_untitled_ones_get_their_own_file() {
        let mut state = EditorState::new(Prefs::default());
        state.prefs.autosave_minutes = 1e-9;
        let layer = state.doc.map.default_layer();
        state.doc.edit("add", |m, _| td_doc::ops::create_point_entity(m, layer, "light", DVec3::ZERO));
        state.open_tab(Document::new());
        let layer = state.doc.map.default_layer();
        state.doc.edit("add", |m, _| td_doc::ops::create_point_entity(m, layer, "light", DVec3::X));
        state.open_tab(Document::new());
        std::thread::sleep(std::time::Duration::from_millis(5));
        state.tick_autosave();

        let autosaves = std::iter::once(&state.autosave).chain(state.tabs.iter().map(|t| &t.autosave));
        let files: Vec<PathBuf> = autosaves.filter_map(|a| a.untitled.clone()).collect();
        assert_eq!(files.len(), 2, "the unmodified tab is not autosaved");
        assert_ne!(files[0], files[1]);
        let found = untitled_autosaves(&std::env::temp_dir());
        assert!(files.iter().all(|f| found.contains(f)), "recovery lists them");
        assert!(state.recoverable_autosaves().iter().all(|f| !files.contains(f)), "open maps are not offered for recovery");

        let recovered = load_document(&files[0]).unwrap();
        assert!(recovered.path.is_none() && recovered.is_modified());
        assert_eq!(recovered.recovered_from.as_deref(), Some(files[0].as_path()));
        for f in files {
            let _ = std::fs::remove_file(f);
        }
    }

    #[test]
    fn an_autosave_that_fails_says_so_once() {
        let mut state = EditorState::new(Prefs::default());
        state.prefs.autosave_minutes = 1e-9;
        let probes = td_doc::lightmap::Probes { sh: vec![0.0; 16 << 20], ..Default::default() };
        let lightmap = td_doc::lightmap::Lightmap { width: 1, height: 1, light: vec![0; 3], shadow: vec![0], ao: vec![0], probes, ..Default::default() };
        state.doc.edit("bake", |m, _| m.lightmap = Some(std::sync::Arc::new(lightmap)));
        state.autosave_now();
        assert!(state.status_failed(), "{}", state.status);
        assert!(state.status.starts_with("Autosave to ") && state.status.contains("the map is too big to save"), "{}", state.status);
        let file = state.autosave.untitled.clone().unwrap();
        assert!(!file.exists());

        state.set_status("later");
        state.autosave_now();
        assert_eq!(state.status, "later", "the same map is not tried again");
        let layer = state.doc.map.default_layer();
        state.doc.edit("add", |m, _| td_doc::ops::create_point_entity(m, layer, "light", DVec3::ZERO));
        state.autosave_now();
        assert!(state.status.starts_with("Autosave to "), "a change tries again, {}", state.status);
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

    #[test]
    fn load_project_reads_the_addon_version() {
        let dir = std::env::temp_dir().join(format!("td_addon_version_test_{}", std::process::id()));
        let addon = dir.join("addons/func_godot");
        std::fs::create_dir_all(&addon).unwrap();
        std::fs::write(dir.join("project.godot"), "").unwrap();
        std::fs::write(addon.join("plugin.cfg"), "[plugin]\nversion=\"0.0.1-not-the-editor\"\n").unwrap();

        let mut state = EditorState::new(Prefs::default());
        state.load_project(&dir);
        assert_eq!(state.game.addon_version.as_deref(), Some("0.0.1-not-the-editor"));
        assert_ne!(state.game.addon_version.as_deref(), Some(crate::VERSION), "picked an addon version that differs from the editor's");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[cfg(unix)]
    #[test]
    fn project_and_map_paths_are_kept_absolute() {
        let dir = std::env::temp_dir().join(format!("td_absolute_paths_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("project.godot"), "").unwrap();
        let map = dir.join("level.tdm");
        format::save(&td_doc::Map::new(), &map).unwrap();
        let up: PathBuf = std::env::current_dir().unwrap().components().skip(1).map(|_| "..").collect();
        let relative = up.join(map.strip_prefix("/").unwrap());
        assert!(relative.is_relative());

        let mut state = EditorState::new(Prefs::default());
        state.open_map(&relative).unwrap();
        assert!(state.doc.path.as_deref().is_some_and(Path::is_absolute), "{:?}", state.doc.path);
        assert!(state.game.project_root.as_deref().is_some_and(Path::is_absolute), "{:?}", state.game.project_root);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_project_is_offered_content_on_its_first_start_only() {
        let dir = std::env::temp_dir().join(format!("td_first_start_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("project.godot"), "config_version=5\n").unwrap();
        let root = absolute(&dir);
        let mut state = EditorState::new(Prefs::default());
        state.load_project(&dir);
        assert_eq!(state.content_request, Some(ContentRequest::FirstStart(root.clone())), "the first start also covers the addon");
        state.content_request = None;
        state.load_project(&dir);
        assert_eq!(state.content_request, Some(ContentRequest::Addon(root.clone())), "a missing addon is brought up again");
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1, "asking writes nothing into the project");
        let status = crate::addon_install::Status::check(&root);
        state.prefs.addon_dismissed.insert(root.clone(), status.dismiss_key());
        state.content_request = None;
        state.load_project(&dir);
        assert_eq!(state.content_request, None, "a recent project has had its first start, and the addon was let be");
        std::fs::create_dir_all(dir.join("addons/func_godot")).unwrap();
        std::fs::write(dir.join("addons/func_godot/plugin.cfg"), format!("[plugin]\nversion=\"{}\"\n", crate::VERSION)).unwrap();
        state.load_project(&dir);
        assert_eq!(state.content_request, None);

        // Remembered per project in the preferences, so it survives the project leaving the recent list.
        let mut prefs = Prefs::default();
        prefs.content_asked.insert(root.clone());
        let mut state = EditorState::new(prefs);
        state.load_project(&dir);
        assert_eq!(state.content_request, None);

        std::fs::create_dir_all(dir.join("trenchdot/nature")).unwrap();
        let mut state = EditorState::new(Prefs::default());
        state.load_project(&dir);
        assert_eq!(state.content_request, None, "a project with the nature models has its content, like the demo project");
        let demo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot");
        state.load_project(&demo);
        assert_eq!(state.content_request, None);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_map_of_another_project_switches_only_to_a_trusted_one() {
        let dir = std::env::temp_dir().join(format!("td_map_project_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (mine, pack, other) = (dir.join("mine"), dir.join("mine/maps/pack"), dir.join("other"));
        for root in [&mine, &pack, &other] {
            std::fs::create_dir_all(root).unwrap();
            std::fs::write(root.join("project.godot"), "config_version=5\n").unwrap();
            std::fs::write(root.join("main.gd"), "extends Node\n").unwrap();
            format::save(&td_doc::Map::new(), &root.join("level.tdm")).unwrap();
        }

        let mut state = EditorState::new(Prefs::default());
        state.trust_for_session(&mine);
        state.load_project(&mine);
        state.open_map(&pack.join("level.tdm")).unwrap();
        assert_eq!(state.game.project_root.as_deref(), Some(mine.as_path()), "a project nested in the open one is ignored, as Godot does");
        assert_eq!(state.other_project(&pack.join("level.tdm")), None);

        state.open_map(&other.join("level.tdm")).unwrap();
        state.apply_code_scans(true);
        assert_eq!(state.game.project_root.as_deref(), Some(mine.as_path()), "an untrusted project is not switched to");
        assert_eq!(state.trust_request, Some(TrustRequest::Switch { root: other.clone(), map: other.join("level.tdm") }));
        assert!(state.status.contains("a project you have not trusted"), "{}", state.status);
        assert_eq!(state.doc.path.as_deref(), Some(other.join("level.tdm").as_path()), "the map still opens");

        state.trust_project(&other, false);
        state.trust_request = None;
        state.open_map(&other.join("level.tdm")).unwrap();
        assert_eq!(state.game.project_root.as_deref(), Some(other.as_path()), "a trusted one is switched to");
        assert_eq!(state.trust_request, None);

        let mut state = EditorState::new(Prefs::default());
        state.open_map(&pack.join("level.tdm")).unwrap();
        assert_eq!(state.game.project_root.as_deref(), Some(pack.as_path()), "with no project open the map's own loads");
        state.apply_code_scans(true);
        assert!(state.restricted() && state.trust_request == Some(TrustRequest::Project(pack.clone())));

        // A project unpacked inside a trusted one is not trusted with it.
        for trust in [EditorState::trust_for_session as fn(&mut EditorState, &Path), |s, root| s.trust_project(root, false)] {
            let mut state = EditorState::new(Prefs::default());
            trust(&mut state, &mine);
            trust(&mut state, &other);
            state.load_project(&other);
            state.open_map(&pack.join("level.tdm")).unwrap();
            state.apply_code_scans(true);
            assert_eq!(state.game.project_root.as_deref(), Some(other.as_path()), "the nested project is asked about");
            assert_eq!(state.trust_request, Some(TrustRequest::Switch { root: pack.clone(), map: pack.join("level.tdm") }));
        }

        let mut state = EditorState::new(Prefs::default());
        state.trust_project(&mine, false);
        state.open_map(&pack.join("level.tdm")).unwrap();
        assert_eq!(state.game.project_root.as_deref(), Some(pack.as_path()));
        assert!(state.restricted(), "with no project open the nested one opens restricted");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_map_with_scripts_says_so_when_it_comes_from_outside() {
        let dir = std::env::temp_dir().join(format!("td_map_scripts_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (project, loose) = (dir.join("project"), dir.join("downloads"));
        std::fs::create_dir_all(&project).unwrap();
        std::fs::create_dir_all(&loose).unwrap();
        std::fs::write(
            project.join("project.godot"),
            "config_version=5
",
        )
        .unwrap();
        let mut map = td_doc::Map::new();
        let layer = map.default_layer();
        let mut script = td_doc::Entity::new("logic_script");
        script.properties.insert("source".into(), "print(activator)".into());
        map.insert(layer, td_doc::NodeKind::Entity(script));
        let (path, outside) = (project.join("level.tdm"), loose.join("level.tdm"));
        format::save(&map, &path).unwrap();
        format::save(&map, &outside).unwrap();

        let mut state = EditorState::new(Prefs::default());
        state.open_map(&path).unwrap();
        assert!(state.restricted());
        assert!(state.status.ends_with(MAP_SCRIPTS), "{}", state.status);
        state.trust_project(&project, false);
        state.open_map(&path).unwrap();
        assert_eq!(state.status, format!("Opened {}", absolute(&path).display()), "a trusted project's maps open quietly");
        state.open_map(&outside).unwrap();
        assert_eq!(state.game.project_root.as_deref(), Some(absolute(&project).as_path()));
        assert_eq!(state.status, format!("Opened {}. {MAP_SCRIPTS}", absolute(&outside).display()), "a loose map is not the project's own");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_map_waits_for_the_scan_of_its_project_before_it_says_it_has_scripts() {
        let dir = std::env::temp_dir().join(format!("td_map_scripts_scan_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (mine, quiet, coded) = (dir.join("mine"), dir.join("quiet"), dir.join("coded"));
        for root in [&mine, &quiet, &coded] {
            std::fs::create_dir_all(root).unwrap();
            std::fs::write(
                root.join("project.godot"),
                "config_version=5
",
            )
            .unwrap();
        }

        std::fs::write(
            coded.join("main.gd"),
            "extends Node
",
        )
        .unwrap();
        for i in 0..2000 {
            let art = quiet.join(format!("art/{i}.png.import"));
            std::fs::create_dir_all(art.parent().unwrap()).unwrap();
            std::fs::write(
                art,
                "[remap]

importer=\"texture\"
",
            )
            .unwrap();
        }

        let with_script = || {
            let mut map = td_doc::Map::new();
            let layer = map.default_layer();
            let mut script = td_doc::Entity::new("logic_script");
            script.properties.insert("source".into(), "print(activator)".into());
            map.insert(layer, td_doc::NodeKind::Entity(script));
            map
        };
        // The same prefab path, a script in the open project's and none in the other's.
        std::fs::create_dir_all(mine.join("prefabs")).unwrap();
        std::fs::create_dir_all(quiet.join("prefabs")).unwrap();
        format::save(&with_script(), &mine.join("prefabs/door.tdm")).unwrap();
        format::save(&td_doc::Map::new(), &quiet.join("prefabs/door.tdm")).unwrap();
        let mut placed = td_doc::Map::new();
        let layer = placed.default_layer();
        let instance =
            td_doc::map::Instance { path: "res://prefabs/door.tdm".into(), origin: DVec3::new(96.0, -32.0, 8.0), angles: DVec3::ZERO, fixup: String::new() };
        placed.insert(layer, td_doc::NodeKind::Instance(instance));
        format::save(&placed, &quiet.join("level.tdm")).unwrap();
        format::save(&with_script(), &coded.join("level.tdm")).unwrap();

        let mut state = EditorState::new(Prefs::default());
        state.trust_for_session(&mine);
        state.load_project(&mine);
        state.refresh_instance_extents();
        state.open_map(&quiet.join("level.tdm")).unwrap();
        assert!(state.switch_scan.is_some(), "the scan of the other project still runs");
        assert!(!state.status.contains(MAP_SCRIPTS), "{}", state.status);
        state.apply_code_scans(true);
        assert_eq!(state.game.project_root.as_deref(), Some(quiet.as_path()), "a project without code is switched to");
        assert!(!state.restricted());
        assert!(!state.status.contains(MAP_SCRIPTS), "{}", state.status);

        state.load_project(&mine);
        state.refresh_instance_extents();
        state.open_map(&coded.join("level.tdm")).unwrap();
        state.apply_code_scans(true);
        assert_eq!(state.game.project_root.as_deref(), Some(mine.as_path()), "a project with code that is not trusted is not switched to");
        assert!(state.status.contains("a project you have not trusted"), "{}", state.status);
        assert!(state.status.ends_with(MAP_SCRIPTS), "{}", state.status);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn close_other_tabs_keeps_the_active_one() {
        let mut state = EditorState::new(Prefs::default());
        let layer = state.doc.map.default_layer();
        state.doc.edit("add", |m, _| td_doc::ops::create_point_entity(m, layer, "light", DVec3::ZERO));
        assert!(state.doc.is_modified());
        // Opening tabs pushes the modified map into `tabs`, each new active one starts blank.
        state.open_tab(Document::new());
        state.open_tab(Document::new());
        assert_eq!(state.tabs.len(), 2);
        assert!(!state.doc.is_modified(), "the now active tab is untouched");
        assert!(state.close_other_tabs(false).is_err(), "a modified tab among the others refuses without discard");
        assert_eq!(state.tabs.len(), 2);
        assert_eq!(state.close_other_tabs(true), Ok(2));
        assert!(state.tabs.is_empty());
        assert!(!state.doc.is_modified(), "closing others left the active tab alone");
    }

    #[test]
    fn live_link_messages_say_godot_scans_until_it_is_done() {
        let mut state = EditorState::new(Prefs::default());
        state.godot_scan = Some(3);
        state.link_state = crate::live_link::LinkState { connected: true, scanned: 2, ..Default::default() };
        state.note_scan_done();
        state.link_status("Godot has no definition for func_door, it shows a plain node".into());
        assert_eq!(state.status, format!("Godot has no definition for func_door, it shows a plain node. {}", crate::entity_pack::SCAN_NOTE));
        state.link_state.scanned = 3;
        state.note_scan_done();
        state.link_status("Live link: Godot rebuilt 1 map node(s)".into());
        assert_eq!((state.status.as_str(), state.godot_scan), ("Live link: Godot rebuilt 1 map node(s)", None));
    }

    #[test]
    fn a_changed_game_config_is_reloaded() {
        let dir = std::env::temp_dir().join(format!("td_game_reload_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("project.godot"), "").unwrap();
        let file = dir.join(td_formats::game::GAME_FILE_NAME);
        let write = |classes: &[&str]| {
            let entities: Vec<_> = classes.iter().map(|c| serde_json::json!({ "classname": c, "type": "point" })).collect();
            std::fs::write(&file, serde_json::json!({ "format": "trenchdot-game", "entities": entities }).to_string()).unwrap();
        };
        write(&["game_locker"]);
        let mut state = EditorState::new(Prefs::default());
        state.load_project(&dir);
        state.watch_game_config(egui::Context::default());
        assert!(state.game.entity("game_locker").is_some());
        assert!(!state.reload_changed_game_config() && !state.game_config_changed());

        write(&["game_locker", "clue_spot"]);
        let start = Instant::now();
        while !state.game_config_changed() && start.elapsed().as_secs() < 10 {
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        assert!(state.game_config_changed(), "the watcher notices the export");
        let map = dir.join("hall.tdm");
        let report = crate::live_link::GodotReport { missing: ["clue_spot".to_string(), "my_lamp".to_string()].into(), ..Default::default() };
        state.link_state.reports.insert(crate::live_link::path_key(&crate::live_link::godot_path(&map)), report);
        state.set_status("Live link: Godot rebuilt 1 map node(s)");
        assert!(state.reload_changed_game_config() && state.game.entity("clue_spot").is_some());
        assert!(!state.game_config_changed(), "the reload watches the new file");
        assert_eq!(state.status, "Live link: Godot rebuilt 1 map node(s)", "Godot exports after each build, the build status stays");
        let missing: Vec<&String> = state.link_state.report(&map).unwrap().missing.iter().collect();
        assert_eq!(missing, ["my_lamp"], "Godot reported clue_spot before its definition appeared");

        for report in state.link_state.reports.values_mut() {
            report.missing.insert("clue_spot".into());
        }

        let entities = serde_json::json!([{ "classname": "game_locker", "type": "point" }, { "classname": "clue_spot", "type": "point" }]);
        let build = serde_json::json!({ "map_settings": "res://settings.tres", "classes": ["clue_spot", "game_locker"] });
        std::fs::write(&file, serde_json::json!({ "format": "trenchdot-game", "entities": entities, "build": build }).to_string()).unwrap();
        assert!(state.reload_changed_game_config());
        let missing: Vec<&String> = state.link_state.report(&map).unwrap().missing.iter().collect();
        assert_eq!(missing, ["my_lamp"], "the FGD Godot builds with has clue_spot now, though the game config had it before");

        std::fs::remove_file(&file).unwrap();
        assert!(state.reload_changed_game_config() && state.game.entity("clue_spot").is_none(), "a removed config falls back to the built-in one");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_game_config_caught_half_written_keeps_the_build_status() {
        let dir = std::env::temp_dir().join(format!("td_game_half_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("project.godot"), "").unwrap();
        let file = dir.join(td_formats::game::GAME_FILE_NAME);
        let full = serde_json::json!({ "format": "trenchdot-game", "entities": [{ "classname": "game_locker", "type": "point" }] }).to_string();
        std::fs::write(&file, &full).unwrap();
        let mut state = EditorState::new(Prefs::default());
        state.load_project(&dir);
        state.set_status("Live link: Godot rebuilt 1 map node(s)");

        std::fs::write(&file, &full[..full.len() / 2]).unwrap();
        assert!(!state.reload_changed_game_config(), "Godot is still writing the file");
        assert_eq!(state.status, "Live link: Godot rebuilt 1 map node(s)");
        assert!(state.game.entity("game_locker").is_some(), "the loaded config stays until the file is whole");

        std::fs::write(&file, full.replace("game_locker", "clue_spot")).unwrap();
        assert!(state.reload_changed_game_config() && state.game.entity("clue_spot").is_some());
        assert_eq!(state.status, "Live link: Godot rebuilt 1 map node(s)");

        std::fs::write(&file, &full[..full.len() / 2]).unwrap();
        let old = std::time::SystemTime::now() - std::time::Duration::from_secs(10);
        std::fs::File::options().write(true).open(&file).unwrap().set_modified(old).unwrap();
        assert!(state.reload_changed_game_config());
        assert!(state.status.starts_with("Failed to load"), "a file that stays cut off is broken: {}", state.status);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
