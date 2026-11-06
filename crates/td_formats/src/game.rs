//! Game configuration exported by the TrenchDot Godot addon (`trenchdot_game.json`).
//! Describes textures, tool textures, scale and entity definitions of one Godot project.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use td_core::{Aabb, Color, DVec3};

pub const GAME_FILE_NAME: &str = "trenchdot_game.json";
pub const GAME_FORMAT: &str = "trenchdot-game";
pub const GAME_FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameConfig {
    /// Empty on files written before the field existed, which are treated as compatible.
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub version: u32,
    #[serde(default = "default_name")]
    pub name: String,
    /// Map units per Godot meter (FuncGodot's inverse scale factor).
    #[serde(default = "default_scale")]
    pub units_per_meter: f64,
    #[serde(default)]
    pub textures: TextureConfig,
    #[serde(default)]
    pub tool_textures: ToolTextures,
    #[serde(default)]
    pub entities: Vec<EntityDef>,
    /// Classnames that more than one FGD resource defines, as the addon's export found them.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub clashes: Vec<Clash>,
    /// What Godot builds maps with when a map node names no map settings. None in files from older addons.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildConfig>,
    /// Absolute path of the Godot project folder, filled in when loading.
    #[serde(skip)]
    pub project_root: Option<PathBuf>,
    #[serde(skip)]
    pub source: Option<PathBuf>,
    /// The addon's `plugin.cfg` version, read from the project alongside it. None when the addon is missing.
    #[serde(skip)]
    pub addon_version: Option<String>,
}

fn default_name() -> String {
    "Godot".into()
}

fn default_scale() -> f64 {
    32.0
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextureConfig {
    /// res:// directory holding textures. Face material names are relative to it, without extension.
    pub base_dir: String,
    pub extensions: Vec<String>,
    #[serde(default)]
    pub material_dir: String,
    #[serde(default = "default_material_ext")]
    pub material_extension: String,
    /// Texture size in pixels used when an image cannot be read.
    #[serde(default = "default_texture_size")]
    pub fallback_size: u32,
}

fn default_material_ext() -> String {
    "tres".into()
}

fn default_texture_size() -> u32 {
    64
}

impl Default for TextureConfig {
    fn default() -> Self {
        Self {
            base_dir: "res://textures".into(),
            extensions: vec!["png".into(), "jpg".into(), "jpeg".into(), "webp".into(), "tga".into(), "bmp".into()],
            material_dir: String::new(),
            material_extension: default_material_ext(),
            fallback_size: default_texture_size(),
        }
    }
}

/// Folders whose textures all build no visual mesh, like the project's tool textures. FuncGodotUtil in the Godot addon
/// keeps the same list, `nodraw_lists_match_the_addon` compares them.
pub const NODRAW_FOLDERS: [&str; 3] = ["special/", "tools/", "td/"];

/// Texture names that build no visual mesh in any folder. In Godot most still collide like clip, `skip`, `origin`,
/// `hint`, `hintskip`, `null` and `areaportal` build nothing at all, and `occluder` faces feed the occluder.
pub const NODRAW_NAMES: [&str; 13] =
    ["clip", "skip", "origin", "sky", "trigger", "nodraw", "hint", "hintskip", "caulk", "null", "areaportal", "playerclip", "occluder"];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolTextures {
    #[serde(default = "default_clip")]
    pub clip: String,
    #[serde(default = "default_skip")]
    pub skip: String,
    #[serde(default = "default_origin")]
    pub origin: String,
    /// Faces that show the sky in Quake and Source. They collide but build no visual mesh.
    #[serde(default = "default_sky")]
    pub sky: String,
}

fn default_clip() -> String {
    "special/clip".into()
}

fn default_skip() -> String {
    "special/skip".into()
}

fn default_origin() -> String {
    "special/origin".into()
}

fn default_sky() -> String {
    "special/sky".into()
}

impl Default for ToolTextures {
    fn default() -> Self {
        Self { clip: default_clip(), skip: default_skip(), origin: default_origin(), sky: default_sky() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityKind {
    Point,
    Solid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    String,
    Int,
    Float,
    Bool,
    Vector2,
    Vector3,
    Color,
    Choices,
    Flags,
    /// Name other entities use to reference this one.
    TargetSource,
    /// Reference to another entity's target source.
    TargetDestination,
    /// res:// path to a resource.
    Resource,
    NodePath,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyDef {
    pub name: String,
    #[serde(rename = "type")]
    pub ty: PropertyType,
    #[serde(default)]
    pub default: String,
    #[serde(default)]
    pub description: String,
    /// For choices: label -> value. For flags: label -> bit value.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<(String, String)>,
    /// Flags that are enabled by default (bit values).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_flags: Vec<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityDef {
    pub classname: String,
    #[serde(rename = "type")]
    pub kind: EntityKind,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_entity_color")]
    pub color: Color,
    /// Point entity bounds in map units, Y-up.
    #[serde(default = "default_size")]
    pub size: [DVec3; 2],
    /// res:// path of a glTF model shown in the editor.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub model: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub node_class: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub scene: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub script: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub group: String,
    #[serde(default)]
    pub properties: Vec<PropertyDef>,
    /// Signals usable as I/O outputs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<IoDef>,
    /// Methods usable as I/O inputs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<IoDef>,
    /// Viewport handles for properties. Inferred from property names when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gizmos: Vec<GizmoDef>,
    /// Godot node groups the built node joins, so an `@group` target names it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub node_groups: Vec<String>,
}

fn one() -> f64 {
    1.0
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Editable viewport helper bound to entity properties.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GizmoDef {
    /// Hinge point ("x y z" relative to the entity center) with the swing of `angle` degrees previewed.
    Hinge {
        property: String,
        #[serde(default)]
        angle: String,
        /// Property holding the axis ("x", "y" or "z"), vertical when empty or unset.
        #[serde(default)]
        axis: String,
    },
    /// Offset ("x y z" map units) the entity moves by, with a ghost at the end position.
    Travel { property: String },
    /// Sphere of `property` times `scale` map units.
    Radius {
        property: String,
        #[serde(default = "one")]
        scale: f64,
    },
    /// Spot cone along the entity's forward axis.
    Cone {
        angle: String,
        range: String,
        #[serde(default = "one")]
        scale: f64,
    },
    /// Box of size "x y z" centered on the entity, or of half extents when `half` is set.
    Box {
        property: String,
        #[serde(default, skip_serializing_if = "is_false")]
        half: bool,
    },
    /// Point ("x y z"), relative to the entity center unless `world` is set.
    Point {
        property: String,
        #[serde(default)]
        world: bool,
    },
}

impl GizmoDef {
    pub fn label(&self) -> String {
        match self {
            GizmoDef::Hinge { property, .. }
            | GizmoDef::Travel { property }
            | GizmoDef::Radius { property, .. }
            | GizmoDef::Box { property, .. }
            | GizmoDef::Point { property, .. } => property.clone(),
            GizmoDef::Cone { range, .. } => range.clone(),
        }
    }
}

/// One classname defined more than once. Godot merges FGD files in order, so the last definition is the one maps get.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Clash {
    pub classname: String,
    /// res:// paths of the definitions in the order they were read, the last one wins.
    pub definitions: Vec<String>,
}

impl Clash {
    pub fn winner(&self) -> &str {
        self.definitions.last().map(String::as_str).unwrap_or_default()
    }

    /// One sentence for the status bar and the Issues panel.
    pub fn describe(&self) -> String {
        let hidden = &self.definitions[..self.definitions.len().saturating_sub(1)];
        format!("{} is defined more than once. {} wins over {}", self.classname, self.winner(), hidden.join(" and "))
    }
}

/// The map settings of the project setting `func_godot/default_map_settings` and the classes their Entity Fgd defines,
/// which can differ from the entities the game config lists.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BuildConfig {
    #[serde(default)]
    pub map_settings: String,
    #[serde(default)]
    pub entity_fgd: String,
    /// Sorted, C# entities included.
    #[serde(default)]
    pub classes: Vec<String>,
}

impl BuildConfig {
    pub fn defines(&self, classname: &str) -> bool {
        self.classes.iter().any(|c| c == classname)
    }
}

/// The types an I/O parameter can have, as [`IoDef::ty`] names them.
pub const IO_TYPES: [&str; 8] = ["bool", "int", "float", "string", "vector3", "color", "node", "variant"];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IoDef {
    pub name: String,
    /// Names of the values the signal passes or the method takes, comma separated.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub parameter: String,
    /// One of [`IO_TYPES`] for each name in `parameter`, comma separated. Empty when unknown, which reads as variant.
    #[serde(default, rename = "type", skip_serializing_if = "String::is_empty")]
    pub ty: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
}

impl IoDef {
    /// The type of each parameter, variant where none is declared.
    pub fn parameter_types(&self) -> Vec<&str> {
        let declared: Vec<&str> = self.ty.split(',').map(str::trim).filter(|t| !t.is_empty()).collect();
        let count = self.parameter.split(',').filter(|p| !p.trim().is_empty()).count();
        (0..count).map(|i| declared.get(i).copied().unwrap_or("variant")).collect()
    }
}

fn default_entity_color() -> Color {
    Color::rgb(0.8, 0.5, 1.0)
}

fn default_size() -> [DVec3; 2] {
    [DVec3::splat(-8.0), DVec3::splat(8.0)]
}

impl EntityDef {
    pub fn bounds(&self) -> Aabb {
        Aabb::new(self.size[0], self.size[1])
    }

    pub fn property(&self, name: &str) -> Option<&PropertyDef> {
        self.properties.iter().find(|p| p.name == name)
    }

    /// Declared gizmos, or ones inferred from well known property names.
    pub fn gizmos(&self, units_per_meter: f64) -> Vec<GizmoDef> {
        if !self.gizmos.is_empty() {
            return self.gizmos.clone();
        }

        let has = |n: &str| self.property(n).is_some();
        let mut out = Vec::new();
        if has("hinge") {
            out.push(GizmoDef::Hinge {
                property: "hinge".into(),
                angle: if has("open_angle") { "open_angle".into() } else { String::new() },
                axis: if has("axis") { "axis".into() } else { String::new() },
            });
        }

        for travel in ["travel", "move_offset"] {
            if has(travel) {
                out.push(GizmoDef::Travel { property: travel.into() });
            }
        }

        for radius in ["radius", "spawn_radius", "trigger_radius"] {
            if has(radius) {
                out.push(GizmoDef::Radius { property: radius.into(), scale: 1.0 });
            }
        }

        if has("omni_range") {
            out.push(GizmoDef::Radius { property: "omni_range".into(), scale: units_per_meter });
        }

        if has("spot_range") && has("spot_angle") {
            out.push(GizmoDef::Cone { angle: "spot_angle".into(), range: "spot_range".into(), scale: units_per_meter });
        }

        if self.kind == EntityKind::Point && self.property("size").is_some_and(|p| p.ty == PropertyType::Vector3) {
            out.push(GizmoDef::Box { property: "size".into(), half: false });
        }

        for point in ["target_offset", "look_at", "exit_offset"] {
            if has(point) {
                out.push(GizmoDef::Point { property: point.into(), world: false });
            }
        }

        out
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GameError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("not a TrenchDot game config (format = {0:?})")]
    WrongFormat(String),
    #[error("game config version {0} is newer than this editor supports ({GAME_FORMAT_VERSION})")]
    TooNew(u32),
}

impl Default for GameConfig {
    fn default() -> Self {
        Self::builtin()
    }
}

impl GameConfig {
    /// Missing format/version fields mean a file written before they existed, and are accepted.
    fn check_version(&self) -> Result<(), GameError> {
        if !self.format.is_empty() && self.format != GAME_FORMAT && self.format != td_doc::legacy::GAME_FORMAT {
            return Err(GameError::WrongFormat(self.format.clone()));
        }

        if self.version > GAME_FORMAT_VERSION {
            return Err(GameError::TooNew(self.version));
        }

        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, GameError> {
        let mut cfg: GameConfig = serde_json::from_str(&std::fs::read_to_string(path)?)?;
        cfg.check_version()?;
        cfg.source = Some(path.to_path_buf());
        cfg.project_root = find_project_root(path);
        Ok(cfg)
    }

    /// Finds a game config inside a Godot project, looking at the project root and one level of subfolders. A config
    /// the addon wrote under its old name counts when there is no other.
    pub fn discover(project_root: &Path) -> Option<PathBuf> {
        let find = |dir: &Path| [GAME_FILE_NAME, td_doc::legacy::GAME_FILE].iter().map(|name| dir.join(name)).find(|p| p.is_file());
        if let Some(direct) = find(project_root) {
            return Some(direct);
        }

        let entries = std::fs::read_dir(project_root).ok()?;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir()
                && !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'))
                && let Some(candidate) = find(&p)
            {
                return Some(candidate);
            }
        }

        None
    }

    pub fn entity(&self, classname: &str) -> Option<&EntityDef> {
        self.entities.iter().find(|e| e.classname == classname)
    }

    /// Whether Godot puts the node built for `classname` in the node group `group`.
    pub fn in_node_group(&self, classname: &str, group: &str) -> bool {
        self.entity(classname).is_some_and(|d| d.node_groups.iter().any(|g| g == group))
    }

    pub fn point_entities(&self) -> impl Iterator<Item = &EntityDef> {
        self.entities.iter().filter(|e| e.kind == EntityKind::Point && e.classname != "worldspawn")
    }

    pub fn solid_entities(&self) -> impl Iterator<Item = &EntityDef> {
        self.entities.iter().filter(|e| e.kind == EntityKind::Solid && e.classname != "worldspawn")
    }

    /// Resolves a res:// path to a file system path inside the project, see [`project_file`].
    pub fn resolve_res(&self, res_path: &str) -> Option<PathBuf> {
        project_file(self.project_root.as_ref()?, res_path)
    }

    pub fn texture_root(&self) -> Option<PathBuf> {
        self.resolve_res(&self.textures.base_dir)
    }

    /// Whether faces with `material` build no visual mesh in Godot: the project's clip, skip, origin and sky textures,
    /// anything in a [`NODRAW_FOLDERS`] folder and anything named one of [`NODRAW_NAMES`].
    pub fn is_tool_texture(&self, material: &str) -> bool {
        let t = &self.tool_textures;
        if [&t.clip, &t.skip, &t.origin, &t.sky].iter().any(|tool| tool.eq_ignore_ascii_case(material)) {
            return true;
        }

        let lower = material.to_ascii_lowercase();
        let name = lower.rsplit('/').next().unwrap_or_default();
        NODRAW_FOLDERS.iter().any(|f| lower.starts_with(f)) || NODRAW_NAMES.contains(&name)
    }

    /// Used when no project is open, and for a project that has not exported its config yet. Lists the core entities
    /// every project has with the Godot addon: worldspawn, the func_ helpers, lights and the player start.
    pub fn builtin() -> Self {
        Self::parse_library(BUILTIN_ENTITIES)
    }

    /// The Gameplay entities pack: doors, buttons, lifts, triggers, logic, spawners, props and effects. A project gets
    /// them by installing the pack, which copies them into the project.
    pub fn gameplay_pack() -> Self {
        Self::parse_library(GAMEPLAY_ENTITIES)
    }

    /// The core entities and the Gameplay entities pack, what a project with the pack offers.
    pub fn with_gameplay_pack() -> Self {
        let mut cfg = Self::builtin();
        cfg.entities.extend(Self::gameplay_pack().entities);
        cfg
    }

    fn parse_library(json: &str) -> Self {
        let mut cfg: GameConfig = serde_json::from_str(json).expect("built-in entity definitions parse");
        for e in &mut cfg.entities {
            e.fill_group();
        }

        cfg
    }
}

impl EntityDef {
    /// Groups an entity without one by the first word of its classname, as the addon's export does.
    pub fn fill_group(&mut self) {
        if self.group.is_empty() {
            self.group = self.classname.split('_').next().unwrap_or_default().into();
        }
    }
}

/// The core entities of the Godot addon, mirrored by `addons/func_godot/fgd/trenchdot/` and FuncGodot's own FGD.
pub const BUILTIN_ENTITIES: &str = include_str!("builtin_entities.json");
/// The Gameplay entities pack, mirrored by the pack template in `addons/func_godot/gameplay_pack/`.
pub const GAMEPLAY_ENTITIES: &str = include_str!("gameplay_entities.json");
/// Walks up from a file until a folder containing project.godot is found.
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    let mut dir = if start.is_dir() { Some(start) } else { start.parent() };
    while let Some(d) = dir {
        if d.join("project.godot").is_file() {
            return Some(d.to_path_buf());
        }

        dir = d.parent();
    }

    None
}

/// The value of `key` in a Godot `plugin.cfg`'s `[plugin]` section.
fn parse_plugin_value(text: &str, key: &str) -> Option<String> {
    let mut in_plugin = false;
    for raw in text.lines() {
        let line = raw.trim();
        if let Some(section) = line.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            in_plugin = section == "plugin";
            continue;
        }

        if in_plugin
            && let Some((k, v)) = line.split_once('=')
            && k.trim() == key
        {
            return Some(v.trim().trim_matches('"').to_string());
        }
    }

    None
}

/// The version the TrenchDot addon declares in its `plugin.cfg`, for the editor to compare against its own. The addon of
/// an install from before the rename can carry the very version of this editor without its content, so it gets the old name behind
/// the version, which no editor version equals.
pub fn addon_version(project_root: &Path) -> Option<String> {
    let text = std::fs::read_to_string(project_root.join("addons/func_godot/plugin.cfg")).ok()?;
    let version = parse_plugin_value(&text, "version")?;
    let name = parse_plugin_value(&text, "name");
    let old = name.as_deref().and_then(|n| td_doc::legacy::ADDON_NAMES.into_iter().find(|old| n.eq_ignore_ascii_case(old)));
    Some(match old {
        Some(old) => format!("{version} ({old})"),
        None => version,
    })
}

/// Converts a file system path inside the project into a res:// path.
pub fn to_res_path(project_root: &Path, path: &Path) -> Option<String> {
    if let Ok(rel) = path.strip_prefix(project_root) {
        return Some(format!("res://{}", rel.to_string_lossy().replace('\\', "/")));
    }

    // A canonicalized root is a verbatim `\\?\D:\` path and scripts spell it `//?/D:/`, which Path parses as a
    // different prefix. Compare the plain text of both, in one slash style and without that prefix.
    let plain = |p: &Path| {
        let text = p.to_string_lossy().replace('\\', "/");
        let text = text.strip_prefix("//?/").unwrap_or(&text).trim_end_matches('/').to_string();
        if cfg!(windows) { text.to_lowercase() } else { text }
    };
    let rest = plain(path).strip_prefix(&plain(project_root))?.strip_prefix('/')?.to_string();
    let full = path.to_string_lossy().replace('\\', "/");
    Some(format!("res://{}", &full[full.len() - rest.len()..]))
}

/// The file `reference` names in the Godot project at `project_root`: a res:// path, or a relative one, which Godot
/// reads from the project root too. None when it would lead out of the project, see [`file_below`].
pub fn project_file(project_root: &Path, reference: &str) -> Option<PathBuf> {
    file_below(project_root, project_root, reference.strip_prefix("res://").unwrap_or(reference))
}

/// The file `rel` names relative to `dir`, a folder inside `root`, as long as it stays inside `root`. Only plain names,
/// `.` and `..` that do not climb above `root` are taken, so absolute, drive and UNC paths, URIs and NTFS streams are
/// refused. The check reads the text alone: on Windows even looking up a `\\host\share` path connects to that host.
pub fn file_below(root: &Path, dir: &Path, rel: &str) -> Option<PathBuf> {
    if rel.starts_with(['/', '\\']) {
        return None;
    }

    let mut parts = Vec::new();
    for c in dir.strip_prefix(root).ok()?.components() {
        match c {
            std::path::Component::Normal(name) => parts.push(name),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                parts.pop()?;
            }
            _ => return None,
        }
    }

    for part in rel.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            name if name.contains(':') => return None,
            name => parts.push(std::ffi::OsStr::new(name)),
        }
    }

    Some(parts.into_iter().fold(root.to_path_buf(), |path, part| path.join(part)))
}

/// `name`, read from inside a file, as one file name that stays in the folder it is written to on any system. Path
/// separators, drive and stream colons, the other characters Windows refuses and control characters turn into `_`,
/// trailing dots and spaces go, a Windows device name such as `con` or `lpt1` gets a leading `_`, a long name is cut
/// and an empty one becomes `unnamed`.
pub fn safe_file_name(name: &str) -> String {
    let refused = |c: char| c.is_control() || matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|');
    let name: String = name.chars().take(120).map(|c| if refused(c) { '_' } else { c }).collect();
    let name = name.trim_end_matches(['.', ' ']);
    if name.is_empty() {
        return "unnamed".into();
    }

    // Windows reads these as devices whatever the extension, and with trailing spaces before it.
    let base = name.split('.').next().unwrap_or_default().trim_end().to_ascii_lowercase();
    let numbered = ["com", "lpt"].iter().filter_map(|p| base.strip_prefix(p)).any(|n| {
        let mut digits = n.chars();
        digits.next().is_some_and(|d| d.is_ascii_digit() || matches!(d, '¹' | '²' | '³')) && digits.next().is_none()
    });
    if numbered || matches!(base.as_str(), "con" | "prn" | "aux" | "nul" | "conin$" | "conout$") { format!("_{name}") } else { name.to_string() }
}

/// Whether `path` is a network share or a Windows device or verbatim path: `\\host\share`, `//host/share`, `\\?\`,
/// `\\.\` or `\??\`. Paths from shared files may lead to local drives, never to these.
pub fn is_network_path(path: &str) -> bool {
    let path = path.replace('/', "\\");
    path.starts_with(r"\\") || path.starts_with(r"\??\")
}

pub type PropertyValues = BTreeMap<String, String>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_version_reads_only_the_plugin_section() {
        let cfg = "[plugin]\nname=\"TrenchDot\"\nversion=\"0.1.0\"\n\n[other]\nversion=\"9.9.9\"\n";
        assert_eq!(parse_plugin_value(cfg, "version").as_deref(), Some("0.1.0"));
        assert_eq!(parse_plugin_value("[plugin]\nname=\"x\"\n", "version"), None, "no version line");
        assert_eq!(parse_plugin_value("version=\"9.9.9\"\n", "version"), None, "not inside [plugin]");
    }

    #[test]
    fn res_paths_ignore_the_verbatim_prefix_and_slash_style() {
        let res = |root: &str, path: &str| to_res_path(Path::new(root), Path::new(path));
        assert_eq!(res(r"\\?\D:\game\godot", "//?/D:/game/godot/models/Crate.gltf").as_deref(), Some("res://models/Crate.gltf"));
        assert_eq!(res("D:/game/godot", r"\\?\D:\game\godot\a\b.glb").as_deref(), Some("res://a/b.glb"));
        assert_eq!(res("/game/godot", "/game/godot/x.glb").as_deref(), Some("res://x.glb"));
        assert_eq!(res("D:/game/godot", "D:/game/godot2/x.glb"), None, "a sibling folder is outside");
        assert_eq!(res("D:/game/godot", "D:/elsewhere/x.glb"), None);
    }

    #[test]
    fn project_files_stay_inside_the_project() {
        let root = Path::new("/game/godot");
        let file = |r: &str| project_file(root, r);
        assert_eq!(file("res://textures/brick.png"), Some(root.join("textures").join("brick.png")));
        assert_eq!(file("models/crate.glb"), Some(root.join("models").join("crate.glb")), "Godot reads a relative path from the root");
        assert_eq!(file("res://models/../textures/./a.png"), Some(root.join("textures").join("a.png")));
        assert_eq!(file("res://"), Some(root.to_path_buf()));
        assert_eq!(file(""), Some(root.to_path_buf()));
        assert_eq!(file("res://%2F%2Fhost/x.png"), Some(root.join("%2F%2Fhost").join("x.png")), "Godot does not decode res:// paths");
        for outside in [
            "res://../secret.png",
            "res://models/../../secret.png",
            r"res://..\secret.png",
            "res://C:/Users/me/secret.png",
            "C:/Users/me/secret.png",
            r"C:secret.png",
            "res:////host/share/x.png",
            "//host/share/x.png",
            r"\\host\share\x.png",
            r"\\?\C:\secret.png",
            r"\\?\UNC\host\share\x.png",
            r"\\.\pipe\x",
            "/etc/passwd",
            "file:///C:/secret.png",
            "file://host/share/x.png",
            "uid://b2x4kfh6",
            "textures/brick.png:hidden",
        ] {
            assert_eq!(file(outside), None, "{outside}");
        }

        let models = root.join("models");
        assert_eq!(file_below(root, &models, "../textures/bark.png"), Some(root.join("textures").join("bark.png")));
        assert_eq!(file_below(root, &models, "../../bark.png"), None, "never above the root");
        assert_eq!(file_below(root, Path::new("/elsewhere"), "bark.png"), None, "the folder has to be inside the root");
    }

    #[test]
    fn network_paths_are_told_apart_from_local_ones() {
        for remote in [r"\\host\share\x.wad", "//host/share/x.wad", r"\/host/x", r"\\?\C:\x.wad", r"\\.\pipe\x", r"\??\UNC\host\share", "/??/C:/x"] {
            assert!(is_network_path(remote), "{remote}");
        }

        for local in ["C:/quake/id1/gfx.wad", r"C:\quake\gfx.wad", "/home/me/gfx.wad", "../gfx.wad", "gfx.wad"] {
            assert!(!is_network_path(local), "{local}");
        }
    }

    #[test]
    fn names_from_files_become_one_safe_file_name() {
        for (name, file) in [
            ("wall1", "wall1"),
            ("{fence", "{fence"),
            ("+0button", "+0button"),
            (".hidden", ".hidden"),
            ("../../wadout", ".._.._wadout"),
            (r"..\..\x", r".._.._x"),
            ("c:x", "c_x"),
            ("/x", "_x"),
            (r"\\host\share\x", "__host_share_x"),
            ("a:stream", "a_stream"),
            ("a\0b\nc", "a_b_c"),
            ("<>|?\"*", "______"),
            ("..", "unnamed"),
            (". .", "unnamed"),
            ("", "unnamed"),
            ("wall. ", "wall"),
            ("con", "_con"),
            ("NUL.png", "_NUL.png"),
            ("aux .x", "_aux .x"),
            ("lpt1", "_lpt1"),
            ("com²", "_com²"),
            ("com10", "com10"),
            ("console", "console"),
        ] {
            assert_eq!(safe_file_name(name), file, "{name:?}");
        }

        assert_eq!(safe_file_name(&"x".repeat(500)).len(), 120);
        assert_eq!(safe_file_name("é".repeat(200).as_str()).chars().count(), 120, "cut by characters, not bytes");
    }

    #[test]
    fn accepts_current_format_and_a_config_missing_the_fields() {
        let current: GameConfig = serde_json::from_str(r#"{"format":"trenchdot-game","version":1}"#).unwrap();
        assert!(current.check_version().is_ok());
        let legacy: GameConfig = serde_json::from_str(r#"{"name":"Test"}"#).unwrap();
        assert!(legacy.check_version().is_ok());
    }

    #[test]
    fn a_config_under_the_old_name_is_found_when_there_is_no_other() {
        let root = &std::env::temp_dir().join(format!("td_game_old_name_{}", std::process::id()));
        std::fs::create_dir_all(root).unwrap();
        let old = root.join(td_doc::legacy::GAME_FILE);
        std::fs::write(&old, format!(r#"{{"format":"{}","version":1}}"#, td_doc::legacy::GAME_FORMAT)).unwrap();
        assert_eq!(GameConfig::discover(root), Some(old.clone()));
        let loaded = GameConfig::load(&old).unwrap();
        assert_eq!(loaded.source.as_deref(), Some(old.as_path()));

        let new = root.join(GAME_FILE_NAME);
        std::fs::write(&new, "{}").unwrap();
        assert_eq!(GameConfig::discover(root), Some(new), "the current name wins");
        std::fs::remove_file(root.join(GAME_FILE_NAME)).unwrap();
        std::fs::remove_file(&old).unwrap();
        std::fs::create_dir(root.join("game")).unwrap();
        std::fs::write(root.join("game").join(td_doc::legacy::GAME_FILE), "{}").unwrap();
        assert_eq!(GameConfig::discover(root), Some(root.join("game").join(td_doc::legacy::GAME_FILE)));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_wrong_format_and_a_newer_version() {
        let wrong: GameConfig = serde_json::from_str(r#"{"format":"something-else","version":1}"#).unwrap();
        assert!(matches!(wrong.check_version(), Err(GameError::WrongFormat(f)) if f == "something-else"));
        let newer: GameConfig = serde_json::from_str(r#"{"format":"trenchdot-game","version":99}"#).unwrap();
        assert!(matches!(newer.check_version(), Err(GameError::TooNew(99))));
    }

    #[test]
    fn pack_brush_and_model_entities_offer_collision_keys() {
        let pack = GameConfig::gameplay_pack();
        let models = ["prop_model", "prop_physics", "npc_walker"];
        let classes: Vec<&EntityDef> = pack.solid_entities().chain(pack.point_entities().filter(|e| models.contains(&e.classname.as_str()))).collect();
        assert_eq!(classes.len(), pack.solid_entities().count() + models.len());
        for def in classes {
            let shape = def.property("collision").unwrap_or_else(|| panic!("{} has no collision key", def.classname));
            assert_eq!((shape.ty, shape.default.as_str()), (PropertyType::Choices, "default"), "{}", def.classname);
            let values: Vec<&str> = shape.options.iter().map(|(_, value)| value.as_str()).collect();
            assert_eq!(values, ["default", "none", "box", "convex", "decomposition", "trimesh"], "{}", def.classname);
            for key in ["collision_layer", "collision_mask"] {
                let p = def.property(key).unwrap_or_else(|| panic!("{} has no {key}", def.classname));
                assert_eq!((p.ty, p.default.as_str()), (PropertyType::String, ""), "{} {key}", def.classname);
            }
        }
    }

    #[test]
    fn builtin_holds_only_the_core_and_the_pack_the_rest() {
        let mut core: Vec<String> = GameConfig::builtin().entities.into_iter().map(|e| e.classname).collect();
        core.sort();
        assert_eq!(
            core,
            [
                "env_reflection_probe",
                "env_voxel_gi",
                "func_detail",
                "func_detail_illusionary",
                "func_geo",
                "func_illusionary",
                "info_player_start",
                "light",
                "light_probe",
                "light_spot",
                "worldspawn"
            ]
        );
        let pack = GameConfig::gameplay_pack();
        assert!(pack.entities.iter().all(|e| !core.contains(&e.classname)), "no classname is in both");
        for def in &pack.entities {
            assert!(
                !def.script.contains("/entities/td_") || def.script.starts_with("res://trenchdot/entities/scripts/"),
                "{} runs its own copy",
                def.classname
            );
        }
    }

    #[test]
    fn reads_the_clashes_the_export_found() {
        let text = r#"{"format": "trenchdot-game", "version": 1, "clashes": [{"classname": "func_door", "definitions": ["res://trenchdot/entities/definitions/func_door.tres", "res://my/door.tres"]}]}"#;
        let cfg: GameConfig = serde_json::from_str(text).unwrap();
        assert_eq!(cfg.clashes[0].winner(), "res://my/door.tres");
        assert_eq!(
            cfg.clashes[0].describe(),
            "func_door is defined more than once. res://my/door.tres wins over res://trenchdot/entities/definitions/func_door.tres"
        );
        assert!(GameConfig::builtin().clashes.is_empty());
    }

    #[test]
    fn reads_what_godot_builds_with() {
        let text = r#"{"format": "trenchdot-game", "version": 1, "build": {"map_settings": "res://my_settings.tres",
            "entity_fgd": "res://my_fgd.tres", "classes": ["func_door", "worldspawn"]}}"#;
        let cfg: GameConfig = serde_json::from_str(text).unwrap();
        let build = cfg.build.as_ref().unwrap();
        assert_eq!((build.map_settings.as_str(), build.entity_fgd.as_str()), ("res://my_settings.tres", "res://my_fgd.tres"));
        assert!(build.defines("func_door") && !build.defines("light"));
        let back: GameConfig = serde_json::from_str(&serde_json::to_string(&cfg).unwrap()).unwrap();
        assert_eq!(back.build, cfg.build);
        let old: GameConfig = serde_json::from_str(r#"{"format": "trenchdot-game", "version": 1}"#).unwrap();
        assert!(old.build.is_none() && !serde_json::to_string(&old).unwrap().contains("\"build\""), "older exports have none");
    }

    #[test]
    fn builtin_round_trips_through_json() {
        let cfg = GameConfig::with_gameplay_pack();
        let text = serde_json::to_string_pretty(&cfg).unwrap();
        let back: GameConfig = serde_json::from_str(&text).unwrap();
        assert_eq!(back.entities.len(), cfg.entities.len());
        assert!(back.entity("func_door").unwrap().inputs.iter().any(|i| i.name == "toggle"));
        for classname in ["func_door_rotating", "func_platform", "trigger_call", "trigger_spawn_area", "info_spawner", "logic_debug", "logic_counter"] {
            assert!(cfg.entity(classname).is_some(), "{classname} is built in");
        }

        for classname in [
            "logic_script",
            "logic_sequence",
            "logic_animate",
            "game_text",
            "env_explosion",
            "prop_physics",
            "npc_walker",
            "light_spot",
            "env_sound",
            "env_particles",
            "logic_branch",
        ] {
            let def = cfg.entity(classname).unwrap_or_else(|| panic!("{classname} is built in"));
            assert!(!def.script.is_empty(), "{classname} points at a runtime script");
        }

        assert!(cfg.entity("logic_branch").unwrap().outputs.iter().any(|o| o.name == "on_true"));

        assert!(cfg.entity("logic_script").unwrap().inputs.iter().any(|i| i.name == "run"));
        assert!(cfg.entity("light").unwrap().script.ends_with("td_light.gd"), "light has a switch script");
    }

    #[test]
    fn the_pack_carries_the_projected_decal() {
        let pack = GameConfig::gameplay_pack();
        let decal = pack.entity("infodecal").expect("the pack defines infodecal");
        assert_eq!((decal.kind, decal.node_class.as_str()), (EntityKind::Point, "Decal"));
        assert_eq!(decal.script, "res://addons/func_godot/src/trenchdot/runtime/trenchdot_decal.gd");
        let keys: Vec<(&str, &str)> = decal.properties.iter().map(|p| (p.name.as_str(), p.default.as_str())).collect();
        assert_eq!(keys, [("material", ""), ("texture", ""), ("normal_texture", ""), ("size", "64 32 64"), ("modulate", "255 255 255")]);
        assert!(decal.properties.iter().all(|p| !p.description.is_empty()), "every key is described");
    }

    #[test]
    fn every_parameter_of_the_built_in_entities_has_a_type() {
        for cfg in [GameConfig::builtin(), GameConfig::gameplay_pack()] {
            for def in &cfg.entities {
                for io in def.inputs.iter().chain(&def.outputs) {
                    let types: Vec<&str> = io.ty.split(',').map(str::trim).filter(|t| !t.is_empty()).collect();
                    let names = io.parameter.split(',').filter(|p| !p.trim().is_empty()).count();
                    assert_eq!(types.len(), names, "{}.{} has a type for each of its parameters", def.classname, io.name);
                    assert!(types.iter().all(|t| IO_TYPES.contains(t)), "{}.{} uses known types: {}", def.classname, io.name, io.ty);
                }
            }
        }

        let cfg = GameConfig::gameplay_pack();
        let add = cfg.entity("logic_counter").unwrap().inputs.iter().find(|i| i.name == "add").unwrap();
        assert_eq!((add.parameter.as_str(), add.ty.as_str()), ("amount", "int"));
    }

    #[test]
    fn an_io_type_is_optional_and_left_out_when_empty() {
        let untyped: IoDef = serde_json::from_str(r#"{"name": "add", "parameter": "amount"}"#).unwrap();
        assert!(untyped.ty.is_empty());
        assert_eq!(untyped.parameter_types(), ["variant"], "an untyped parameter reads as variant");
        assert_eq!(serde_json::to_string(&untyped).unwrap(), r#"{"name":"add","parameter":"amount"}"#);

        let typed: IoDef = serde_json::from_str(r#"{"name": "hit", "parameter": "who, hp", "type": "node, float"}"#).unwrap();
        assert_eq!(typed.parameter_types(), ["node", "float"]);
        assert!(serde_json::to_string(&typed).unwrap().contains(r#""type":"node, float""#));

        let none: IoDef = serde_json::from_str(r#"{"name": "reset"}"#).unwrap();
        assert!(none.parameter_types().is_empty());
    }

    #[test]
    fn tool_textures_fill_missing_keys_and_match_any_case() {
        let tools: ToolTextures = serde_json::from_str(r#"{ "clip": "Tools/Clip" }"#).unwrap();
        assert_eq!((tools.skip.as_str(), tools.origin.as_str(), tools.sky.as_str()), ("special/skip", "special/origin", "special/sky"));
        let cfg = GameConfig { tool_textures: tools, ..GameConfig::builtin() };
        assert!(cfg.is_tool_texture("tools/clip") && cfg.is_tool_texture("TOOLS/CLIP") && cfg.is_tool_texture("Special/Skip"));
        assert!(!cfg.is_tool_texture("brick/clipper") && !cfg.is_tool_texture("special_brick") && !cfg.is_tool_texture(""));
        for nodraw in ["special/trigger", "Special/NoDraw", "special/anything", "tools/toolsnodraw", "td/occluder", "common/caulk", "e1u1/hint", "playerclip"] {
            assert!(cfg.is_tool_texture(nodraw), "{nodraw}");
        }
    }

    /// The Godot addon hides the same faces, see FuncGodotUtil.filter_face.
    #[test]
    fn nodraw_lists_match_the_addon() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot/addons/func_godot/src/util/func_godot_util.gd");
        let src = std::fs::read_to_string(&path).expect("the addon submodule is checked out");
        let list = |name: &str| -> Vec<String> {
            let line = src.lines().find(|l| l.starts_with(&format!("const {name}"))).unwrap_or_else(|| panic!("{name} in func_godot_util.gd"));
            let inner = &line[line.find('[').unwrap() + 1..line.rfind(']').unwrap()];
            inner.split(',').map(|s| s.trim().trim_matches('"').to_string()).filter(|s| !s.is_empty()).collect()
        };
        assert_eq!(list("NODRAW_FOLDERS"), NODRAW_FOLDERS);
        assert_eq!(list("NODRAW_NAMES"), NODRAW_NAMES);
    }

    #[test]
    fn gizmos_are_declared_or_inferred() {
        let cfg = GameConfig::with_gameplay_pack();
        let door = cfg.entity("func_door_rotating").unwrap();
        assert!(matches!(&door.gizmos(32.0)[..], [GizmoDef::Hinge { angle, axis, .. }] if angle == "open_angle" && axis == "axis"));
        let light = cfg.entity("light").unwrap();
        assert!(matches!(&light.gizmos(32.0)[..], [GizmoDef::Radius { scale, .. }] if *scale == 32.0), "omni_range is meters");
        assert!(cfg.entity("info_spawner").unwrap().gizmos(32.0).iter().any(|g| matches!(g, GizmoDef::Radius { property, .. } if property == "radius")));
        assert!(cfg.entity("func_detail").unwrap().gizmos(32.0).is_empty());
        let prop = cfg.entity("prop_physics").unwrap();
        assert_eq!(prop.gizmos(32.0), [GizmoDef::Box { property: "size".into(), half: true }], "size is half extents");
        assert!(!cfg.entity("env_particles").unwrap().gizmos(32.0).iter().any(|g| matches!(g, GizmoDef::Box { .. })), "a float size is no box");
        let mut boxed = prop.clone();
        boxed.gizmos.clear();
        assert_eq!(boxed.gizmos(32.0), [GizmoDef::Box { property: "size".into(), half: false }], "an undeclared vector3 size is full size");
    }
}
