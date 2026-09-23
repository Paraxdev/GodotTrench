//! Game configuration exported by the GodotTrench Godot addon (`godottrench_game.json`).
//! Describes textures, tool textures, scale and entity definitions of one Godot project.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use gt_core::{Aabb, Color, DVec3};
use serde::{Deserialize, Serialize};

pub const GAME_FILE_NAME: &str = "godottrench_game.json";
pub const GAME_FORMAT: &str = "godottrench-game";
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
    /// Absolute path of the Godot project folder, filled in when loading.
    #[serde(skip)]
    pub project_root: Option<PathBuf>,
    #[serde(skip)]
    pub source: Option<PathBuf>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolTextures {
    pub clip: String,
    pub skip: String,
    pub origin: String,
    /// Faces that show the sky in Quake and Source. They collide but build no visual mesh.
    #[serde(default = "default_sky")]
    pub sky: String,
}

fn default_sky() -> String {
    "special/sky".into()
}

impl Default for ToolTextures {
    fn default() -> Self {
        Self { clip: "special/clip".into(), skip: "special/skip".into(), origin: "special/origin".into(), sky: default_sky() }
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
}

fn one() -> f64 {
    1.0
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
    /// Box of size "x y z" centered on the entity.
    Box { property: String },
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
            | GizmoDef::Box { property }
            | GizmoDef::Point { property, .. } => property.clone(),
            GizmoDef::Cone { range, .. } => range.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IoDef {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub parameter: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
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

        if has("size") && self.kind == EntityKind::Point {
            out.push(GizmoDef::Box { property: "size".into() });
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
    #[error("not a GodotTrench game config (format = {0:?})")]
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
        if !self.format.is_empty() && self.format != GAME_FORMAT {
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

    /// Finds a game config inside a Godot project, looking at the project root and one level of subfolders.
    pub fn discover(project_root: &Path) -> Option<PathBuf> {
        let direct = project_root.join(GAME_FILE_NAME);
        if direct.is_file() {
            return Some(direct);
        }

        let entries = std::fs::read_dir(project_root).ok()?;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() && !p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.')) {
                let candidate = p.join(GAME_FILE_NAME);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        None
    }

    pub fn entity(&self, classname: &str) -> Option<&EntityDef> {
        self.entities.iter().find(|e| e.classname == classname)
    }

    pub fn point_entities(&self) -> impl Iterator<Item = &EntityDef> {
        self.entities.iter().filter(|e| e.kind == EntityKind::Point && e.classname != "worldspawn")
    }

    pub fn solid_entities(&self) -> impl Iterator<Item = &EntityDef> {
        self.entities.iter().filter(|e| e.kind == EntityKind::Solid && e.classname != "worldspawn")
    }

    /// Resolves a res:// path to a file system path inside the project.
    pub fn resolve_res(&self, res_path: &str) -> Option<PathBuf> {
        let root = self.project_root.as_ref()?;
        let rel = res_path.strip_prefix("res://").unwrap_or(res_path);
        Some(root.join(rel))
    }

    pub fn texture_root(&self) -> Option<PathBuf> {
        self.resolve_res(&self.textures.base_dir)
    }

    pub fn is_tool_texture(&self, material: &str) -> bool {
        let m = material.to_ascii_lowercase();
        m == self.tool_textures.clip || m == self.tool_textures.skip || m == self.tool_textures.origin || m == self.tool_textures.sky
    }

    /// Used when no project is open, so the editor is usable out of the box. Lists the GodotTrench gameplay entities
    /// that ship with the Godot addon (doors, lifts, triggers, spawners, logic and debug helpers).
    pub fn builtin() -> Self {
        let mut cfg: GameConfig = serde_json::from_str(BUILTIN_ENTITIES).expect("built-in entity definitions parse");
        for e in &mut cfg.entities {
            if e.group.is_empty() {
                e.group = e.classname.split('_').next().unwrap_or_default().into();
            }
        }

        cfg
    }
}

/// The entity library of the Godot addon, mirrored by `addons/func_godot/fgd/godottrench/` in Godot.
pub const BUILTIN_ENTITIES: &str = include_str!("builtin_entities.json");
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

pub type PropertyValues = BTreeMap<String, String>;

#[cfg(test)]
mod tests {
    use super::*;

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
    fn accepts_current_format_and_a_config_missing_the_fields() {
        let current: GameConfig = serde_json::from_str(r#"{"format":"godottrench-game","version":1}"#).unwrap();
        assert!(current.check_version().is_ok());
        let legacy: GameConfig = serde_json::from_str(r#"{"name":"Test"}"#).unwrap();
        assert!(legacy.check_version().is_ok());
    }

    #[test]
    fn rejects_wrong_format_and_a_newer_version() {
        let wrong: GameConfig = serde_json::from_str(r#"{"format":"something-else","version":1}"#).unwrap();
        assert!(matches!(wrong.check_version(), Err(GameError::WrongFormat(f)) if f == "something-else"));
        let newer: GameConfig = serde_json::from_str(r#"{"format":"godottrench-game","version":99}"#).unwrap();
        assert!(matches!(newer.check_version(), Err(GameError::TooNew(99))));
    }

    #[test]
    fn builtin_round_trips_through_json() {
        let cfg = GameConfig::builtin();
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
        assert!(cfg.entity("light").unwrap().script.ends_with("gt_light.gd"), "light has a switch script");
    }

    #[test]
    fn gizmos_are_declared_or_inferred() {
        let cfg = GameConfig::builtin();
        let door = cfg.entity("func_door_rotating").unwrap();
        assert!(matches!(&door.gizmos(32.0)[..], [GizmoDef::Hinge { angle, axis, .. }] if angle == "open_angle" && axis == "axis"));
        let light = cfg.entity("light").unwrap();
        assert!(matches!(&light.gizmos(32.0)[..], [GizmoDef::Radius { scale, .. }] if *scale == 32.0), "omni_range is meters");
        assert!(cfg.entity("info_spawner").unwrap().gizmos(32.0).iter().any(|g| matches!(g, GizmoDef::Radius { property, .. } if property == "radius")));
        assert!(cfg.entity("func_detail").unwrap().gizmos(32.0).is_empty());
    }
}
