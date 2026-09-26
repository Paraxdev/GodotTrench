use egui::{Key, KeyboardShortcut, Modifiers};
use gt_core::{Aabb, DVec2, DVec3, NodeId};
use gt_doc::ops;
use gt_doc::{Document, format};

use crate::mesh_tool::MeshOp;
use crate::state::{EditorState, Prefs, Shade};
use crate::tools::ToolKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelImport {
    /// One editable mesh node with the model's UVs.
    Mesh,
    /// Every cube becomes a brush with matching Valve UVs.
    Brushes,
    /// A prop entity that references the model file.
    Prop,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    NewMap,
    OpenMap,
    /// Opens this map in a tab, without a file dialog.
    OpenMapFile(std::path::PathBuf),
    OpenProject,
    Save,
    SaveAs,
    Undo,
    Redo,
    RepeatLast,
    Delete,
    Duplicate,
    /// Names the selected object in its Outliner row.
    Rename,
    SelectAll,
    SelectNone,
    SelectInverse,
    SelectTouching,
    SelectInside,
    SelectSiblings,
    SelectSameMaterial,
    Group,
    Ungroup,
    HideSelected,
    IsolateSelected,
    UnhideAll,
    LockSelected,
    UnlockAll,
    GridDown,
    GridUp,
    ToggleSnap,
    ToggleUvLock,
    ToggleTransformGizmo,
    ToggleGodotOverlays,
    /// Shows where an agent can walk, baked by the connected Godot editor.
    ToggleWalkable,
    ToggleTextured,
    SetShade(Shade),
    CsgSubtract,
    CsgMerge,
    CsgIntersect,
    CsgHollow,
    Rotate {
        axis: usize,
        degrees: f64,
    },
    Flip {
        axis: usize,
    },
    SetTool(ToolKind),
    FocusSelection,
    CreateBrushEntity(String),
    CreatePointEntity {
        classname: String,
        at: Option<DVec3>,
    },
    /// Lines the entities up along `row`, resting on the surface facing `normal`. Brush entities get a box brush.
    PlaceEntities {
        classnames: Vec<String>,
        at: Option<DVec3>,
        normal: Option<DVec3>,
        row: DVec3,
    },
    MoveToWorld,
    MoveToLayer(NodeId),
    AddLayer,
    SnapVertices,
    ApplyMaterial(String),
    Paste(String),
    Copy,
    Cut,
    OpenGroup,
    CloseGroup,
    ReloadProject,
    CreateBrushFromBounds,
    Nudge(DVec3),
    CreatePrefab,
    InsertPrefab,
    ExplodeInstances,
    OpenPrefab,
    ShowCommandPalette,
    ShowShapeDialog,
    ShowTerrainDialog,
    ShowKeymap,
    ShowUvEditor,
    OpenGodotEditor,
    RunGodotProject,
    /// Brings the Godot editor that has this project open to the front.
    FocusGodot,
    /// Full build in Godot of the map as shown here, saved or not.
    BuildInGodot,
    ToggleLiveMode,
    CreateDisplacement(u8),
    RemoveDisplacement,
    SewDisplacements,
    ImportQuakeMap,
    ExportQuakeMap,
    ExportQuakeMapCordon,
    ImportVmf,
    /// Turns a folder of Valve, Quake or Half-Life textures into project PNGs and materials.
    ConvertTextures,
    ImportModel(ModelImport),
    /// Drops a model from the Models panel into the scene as an editable mesh at `at`.
    PlaceModel {
        path: std::path::PathBuf,
        at: DVec3,
    },
    ReloadModels,
    /// Lays a decal sheet (a quad blended over what is behind it) on a surface, facing along `normal`.
    CreateDecal {
        material: String,
        at: DVec3,
        normal: DVec3,
        /// Width and height in map units, one metre square when unset.
        size: Option<[f64; 2]>,
    },
    ConvertToMesh,
    ConvertToBrushes,
    JoinMeshes,
    /// Enters mesh editing, converting selected brushes first.
    EditMesh,
    /// Tab: edits the selection where it stands, meshes in the Mesh tool and brushes in the Vertex tool, or goes back
    /// to Select. Never converts, a reflex key press must not change what the objects are.
    ToggleEditMode,
    MeshOp(MeshOp),
    DuplicateLinked,
    UnlinkGroups,
    SetCordonFromSelection,
    ToggleCordon,
    ClearCordon,
    StoreCamera(u8),
    RecallCamera(u8),
    NewTab,
    CloseTab,
    NextTab,
    HotspotTexture,
    TerrainFlatten,
    TerrainAutoPaint,
    Justify(gt_geom::Justify),
    ToggleTreatAsOne,
    /// Projects the selected faces straight along the 3D camera.
    AlignTextureToView,
    ResetTexture,
    CopyAlignment,
    PasteAlignment,
    /// World units per texture pixel.
    TexelDensity(f64),
    MeshUv(crate::texture_ops::MeshUvKind),
    ShowHotspotEditor,
    /// Opens the hotspot editor on this material.
    EditHotspots(String),
    ReloadMaterials,
    /// Makes a scatter set the one the scatter tool paints into.
    ActivateScatter(NodeId),
    /// An empty scatter set on a new layer, to drop models into.
    NewScatterSet,
    ScatterFill,
    /// A new scatter set holding a built-in preset.
    ScatterPreset(String),
    ShowScatterPanel,
    InstallNatureModels,
    /// Replaces the selected scatter sets with prop entities.
    ScatterToEntities,
    /// Uses the current material as the blend material of the selected faces.
    SetBlendMaterial,
    ClearBlendMaterial,
    /// Sets how the blend material of the selected faces repeats, for painting paths that do not look tiled.
    SetBlendTiling {
        detile: f64,
        uv_scale: f64,
        sharpen: f64,
    },
    MakeDoor {
        kind: crate::entity_wizards::DoorKind,
        trigger: bool,
    },
    /// Selected brushes become a lift travelling up by their height plus the grid.
    MakePlatform,
    /// Opens the link dialog for the two selected entities.
    ShowLinkDialog,
    /// A trigger volume of this class around the selection bounds.
    VolumeAroundSelection(String),
    ShowReference,
    ShowPreferences,
    /// Fills the view area with the view under the pointer, or brings the other views back.
    ToggleMaximizeView,
    /// Shows 1, 2 or 4 views.
    ViewLayout(u8),
    /// Closes or reopens one of the four views.
    ToggleView(usize),
    UiScaleUp,
    UiScaleDown,
    UiScaleReset,
}

impl Action {
    pub fn label(&self) -> String {
        match self {
            Action::Rotate { axis, degrees } => format!("Rotate {} {degrees}°", axis_name(*axis)),
            Action::Flip { axis } => format!("Flip {}", axis_name(*axis)),
            Action::SetTool(t) => format!("{} Tool", t.label()),
            Action::CreateBrushEntity(c) => format!("Create {c}"),
            Action::CreatePointEntity { classname, .. } => format!("Create {classname}"),
            Action::PlaceEntities { classnames, .. } => match classnames.as_slice() {
                [one] => format!("Create {one}"),
                many => format!("Place {} Entities", many.len()),
            },
            Action::PlaceModel { path, .. } => {
                format!("Place {}", path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "Model".into()))
            }
            Action::MeshOp(op) => format!("Mesh: {}", op.label()),
            Action::StoreCamera(n) => format!("Store Camera {n}"),
            Action::RecallCamera(n) => format!("Recall Camera {n}"),
            Action::SetShade(s) => format!("Shade {}", s.label()),
            Action::Justify(j) => format!("Justify Texture {}", j.label()),
            Action::TexelDensity(d) => format!("Texel Density {d}"),
            Action::MeshUv(k) => format!("Mesh UVs: {}", k.label()),
            Action::ToggleEditMode => "Edit Mode".into(),
            Action::ToggleMaximizeView => "Maximize View".into(),
            Action::ViewLayout(1) => "Single View Layout".into(),
            Action::ViewLayout(2) => "Two View Layout".into(),
            Action::ViewLayout(_) => "Four View Layout".into(),
            Action::ToggleView(i) => format!("Toggle View {}", i + 1),
            Action::UiScaleUp => "Increase UI Scale".into(),
            Action::UiScaleDown => "Decrease UI Scale".into(),
            Action::UiScaleReset => "Reset UI Scale".into(),
            Action::ShowPreferences => "Preferences".into(),
            Action::ToggleTransformGizmo => "Toggle Transform Gizmo".into(),
            Action::ToggleGodotOverlays => "Toggle Godot Overlays".into(),
            Action::ToggleWalkable => "Toggle Walkable Area".into(),
            Action::OpenGodotEditor => "Open Project in Godot".into(),
            Action::RunGodotProject => "Run Godot Project".into(),
            Action::FocusGodot => "Show Godot Editor".into(),
            Action::BuildInGodot => "Build in Godot".into(),
            Action::ToggleLiveMode => "Toggle Godot Live Mode".into(),
            other => format!("{other:?}"),
        }
    }

    /// What a command does in plain words, for the ones whose name assumes level editor jargon.
    pub fn help(&self) -> Option<&'static str> {
        Some(match self {
            Action::CsgSubtract => {
                "Cuts the selected brushes out of every brush they overlap, then deletes them. Subtract a box that pokes through a wall to make a doorway"
            }
            Action::CsgMerge => "Replaces the selected brushes with one brush that wraps around all of them, like shrink wrap",
            Action::CsgIntersect => "Keeps only the space the selected brushes share",
            Action::CsgHollow => "Turns a solid brush into a room, with walls, floor and ceiling of the wall thickness around an empty inside",
            Action::SnapVertices => "Moves every corner of the selected brushes to the nearest grid point",
            Action::MoveToWorld => "Takes the selected brushes out of their door, trigger or other brush entity, back into plain level geometry",
            Action::CreateBrushFromBounds => "Draws a box brush the size of the last selection",
            Action::SelectTouching => "Uses the selected brushes as a selection box, selecting everything they touch and deleting the box",
            Action::SelectInside => "Uses the selected brushes as a selection box, selecting everything fully inside and deleting the box",
            Action::SelectSiblings => "Selects everything in the same group or brush entity as the selection",
            Action::IsolateSelected => "Hides everything except the selection, Show All brings it back",
            Action::DuplicateLinked => "Copies the selection as a linked group, editing one copy changes every copy",
            Action::UnlinkGroups => "Makes the selected linked groups independent copies again",
            Action::OpenGroup => "Lets you click and edit the objects inside the selected group",
            Action::CreatePrefab => "Saves the selection as its own map file, a prefab, and puts an instance in its place that follows changes to that file",
            Action::InsertPrefab => "Places an instance of another map file, a prefab, at the cursor",
            Action::OpenPrefab => "Opens the map file of the selected prefab instance in its own tab, edits there update every instance",
            Action::ExplodeInstances => "Replaces the selected prefab instances with plain copies of their contents",
            Action::SetCordonFromSelection => "Hides everything outside the selection bounds, to work on one part of a big map",
            Action::ToggleCordon => "Switches the cordon on or off without forgetting its box",
            Action::ClearCordon => "Removes the cordon box and shows the whole map again",
            Action::ExportQuakeMapCordon => "Exports only the objects that touch the cordon box",
            Action::ToggleUvLock => {
                "When on, textures stick to brushes as you move or rotate them. When off, they stay put in the world and slide across the faces"
            }
            Action::ToggleTreatAsOne => "Aligns textures across all selected faces as if they were one surface",
            Action::HotspotTexture => "Fits the selected faces to the best matching rectangle of a trim sheet, read from <texture>.hotspots.json",
            Action::TexelDensity(_) => "Sets how many texture pixels cover each map unit on the selected faces",
            Action::SetBlendMaterial => "Uses the current material as a second texture on the selected faces, painted in with the Blend tool",
            Action::CreateDisplacement(_) => "Splits the selected face into a grid of points you can raise and lower, for uneven ground",
            Action::SewDisplacements => "Joins the edges of neighbouring displacements so they leave no gaps",
            Action::RemoveDisplacement => "Turns displaced faces back into flat faces",
            Action::EditMesh => {
                "Converts the selection to a mesh and edits its vertices, edges and faces Blender style. Meshes can take any shape, brushes stay convex"
            }
            Action::ConvertToMesh => "Turns the selected brushes into meshes, which can take any shape but no longer work with CSG",
            Action::ConvertToBrushes => "Turns the selected meshes back into brushes, split into convex pieces",
            Action::MakePlatform => "Turns the selected brushes into a func_platform, a lift that moves up and down",
            Action::ToggleLiveMode => "Sends every edit to the scene open in the Godot editor before you save",
            Action::ToggleWalkable => "Shades the floors a player can walk on, baked by Godot",
            Action::ToggleGodotOverlays => "Shows the nodes added on top of the built map in Godot as ghost boxes",
            Action::RepeatLast => "Runs the last rotate, flip, nudge or duplicate again",
            _ => return None,
        })
    }

    /// Stable identifier used by key binding overrides.
    pub fn binding_id(&self) -> String {
        format!("{self:?}").chars().filter(|c| c.is_alphanumeric() || *c == '_').collect()
    }
}

pub fn axis_name(axis: usize) -> &'static str {
    ["X", "Y", "Z"][axis.min(2)]
}

const fn sc(modifiers: Modifiers, key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(modifiers, key)
}

const CTRL: Modifiers = Modifiers::COMMAND;
const CTRL_SHIFT: Modifiers = Modifiers { alt: false, ctrl: false, shift: true, mac_cmd: false, command: true };
const SHIFT: Modifiers = Modifiers::SHIFT;
const ALT: Modifiers = Modifiers::ALT;
const ALT_SHIFT: Modifiers = Modifiers { alt: true, ctrl: false, shift: true, mac_cmd: false, command: false };
const NONE: Modifiers = Modifiers::NONE;

const DIGITS: [Key; 9] = [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5, Key::Num6, Key::Num7, Key::Num8, Key::Num9];

pub const PRESETS: [&str; 3] = ["trenchbroom", "hammer", "blender"];

/// TrenchBroom defaults plus GodotTrench additions.
fn trenchbroom_bindings() -> Vec<(KeyboardShortcut, Action)> {
    let mut out = vec![
        (sc(CTRL, Key::N), Action::NewMap),
        (sc(CTRL, Key::O), Action::OpenMap),
        (sc(CTRL, Key::S), Action::Save),
        (sc(CTRL_SHIFT, Key::S), Action::SaveAs),
        (sc(CTRL, Key::Z), Action::Undo),
        (sc(CTRL_SHIFT, Key::Z), Action::Redo),
        (sc(CTRL, Key::Y), Action::Redo),
        (sc(CTRL, Key::R), Action::RepeatLast),
        (sc(NONE, Key::Delete), Action::Delete),
        (sc(NONE, Key::Backspace), Action::Delete),
        (sc(CTRL, Key::D), Action::Duplicate),
        (sc(CTRL_SHIFT, Key::D), Action::DuplicateLinked),
        (sc(CTRL, Key::A), Action::SelectAll),
        (sc(NONE, Key::Escape), Action::SelectNone),
        (sc(CTRL, Key::I), Action::SelectInverse),
        (sc(CTRL, Key::T), Action::SelectTouching),
        (sc(CTRL_SHIFT, Key::T), Action::SelectInside),
        (sc(CTRL, Key::G), Action::Group),
        (sc(CTRL_SHIFT, Key::G), Action::Ungroup),
        (sc(CTRL, Key::H), Action::HideSelected),
        (sc(CTRL, Key::J), Action::IsolateSelected),
        (sc(CTRL_SHIFT, Key::H), Action::UnhideAll),
        (sc(NONE, Key::OpenBracket), Action::GridDown),
        (sc(NONE, Key::CloseBracket), Action::GridUp),
        (sc(NONE, Key::Minus), Action::GridDown),
        (sc(NONE, Key::Equals), Action::GridUp),
        (sc(CTRL_SHIFT, Key::U), Action::ToggleUvLock),
        (sc(CTRL, Key::K), Action::CsgSubtract),
        (sc(CTRL, Key::M), Action::CsgMerge),
        (sc(CTRL, Key::L), Action::CsgIntersect),
        (sc(CTRL_SHIFT, Key::K), Action::CsgHollow),
        (sc(NONE, Key::C), Action::SetTool(ToolKind::Clip)),
        (sc(NONE, Key::V), Action::SetTool(ToolKind::Vertex)),
        (sc(NONE, Key::R), Action::SetTool(ToolKind::Rotate)),
        (sc(NONE, Key::T), Action::SetTool(ToolKind::Scale)),
        (sc(NONE, Key::Q), Action::SetTool(ToolKind::Select)),
        (sc(NONE, Key::G), Action::SetTool(ToolKind::Sculpt)),
        (sc(NONE, Key::P), Action::SetTool(ToolKind::Paint)),
        (sc(NONE, Key::Tab), Action::ToggleEditMode),
        (sc(NONE, Key::B), Action::SetTool(ToolKind::Scatter)),
        (sc(SHIFT, Key::G), Action::SetTool(ToolKind::Blend)),
        (sc(SHIFT, Key::E), Action::SetTool(ToolKind::Volume)),
        (sc(NONE, Key::M), Action::SetTool(ToolKind::Measure)),
        (sc(SHIFT, Key::P), Action::SetTool(ToolKind::Path)),
        (sc(SHIFT, Key::T), Action::SetTool(ToolKind::Texture)),
        (sc(CTRL, Key::U), Action::FocusSelection),
        (sc(NONE, Key::F), Action::FocusSelection),
        (sc(NONE, Key::F2), Action::Rename),
        (sc(NONE, Key::F4), Action::ToggleTextured),
        (sc(NONE, Key::F3), Action::SetShade(Shade::Lit)),
        (sc(CTRL_SHIFT, Key::W), Action::MoveToWorld),
        (sc(CTRL_SHIFT, Key::P), Action::ShowCommandPalette),
        (sc(NONE, Key::F1), Action::ShowCommandPalette),
        (sc(CTRL_SHIFT, Key::B), Action::ShowShapeDialog),
        (sc(NONE, Key::F5), Action::RunGodotProject),
        (sc(CTRL_SHIFT, Key::E), Action::ConvertToMesh),
        (sc(CTRL_SHIFT, Key::J), Action::JoinMeshes),
        (sc(ALT, Key::H), Action::HotspotTexture),
        (sc(CTRL, Key::Tab), Action::NextTab),
        (sc(CTRL, Key::W), Action::CloseTab),
        (sc(CTRL_SHIFT, Key::N), Action::NewTab),
        (sc(CTRL, Key::Equals), Action::UiScaleUp),
        (sc(CTRL, Key::Minus), Action::UiScaleDown),
        (sc(CTRL, Key::Num0), Action::UiScaleReset),
        (sc(SHIFT, Key::Space), Action::ToggleMaximizeView),
    ];
    for (i, key) in DIGITS.iter().enumerate() {
        out.push((sc(CTRL, *key), Action::RecallCamera(i as u8 + 1)));
        out.push((sc(CTRL_SHIFT, *key), Action::StoreCamera(i as u8 + 1)));
    }

    out
}

/// Key bindings for a preset. Hammer and Blender presets take over some TrenchBroom chords, actions keep their other
/// bindings, and the last entries of each preset rebind actions whose only chord was taken.
pub fn preset_bindings(preset: &str) -> Vec<(KeyboardShortcut, Action)> {
    let mut base = trenchbroom_bindings();
    let replace: Vec<(KeyboardShortcut, Action)> = match preset {
        "hammer" => vec![
            (sc(SHIFT, Key::S), Action::SetTool(ToolKind::Select)),
            (sc(SHIFT, Key::X), Action::SetTool(ToolKind::Clip)),
            (sc(SHIFT, Key::V), Action::SetTool(ToolKind::Vertex)),
            (sc(CTRL, Key::W), Action::MoveToWorld),
            (sc(NONE, Key::H), Action::HideSelected),
            (sc(CTRL, Key::H), Action::IsolateSelected),
            (sc(NONE, Key::U), Action::UnhideAll),
            (sc(CTRL, Key::L), Action::Rotate { axis: 1, degrees: 90.0 }),
            (sc(CTRL, Key::M), Action::ShowShapeDialog),
            (sc(SHIFT, Key::A), Action::SetTool(ToolKind::Texture)),
            (sc(CTRL, Key::F4), Action::CloseTab),
            (sc(CTRL_SHIFT, Key::M), Action::CsgMerge),
            (sc(CTRL_SHIFT, Key::L), Action::CsgIntersect),
        ],
        "blender" => vec![
            (sc(NONE, Key::A), Action::SelectAll),
            (sc(ALT, Key::A), Action::SelectNone),
            (sc(NONE, Key::X), Action::Delete),
            (sc(SHIFT, Key::D), Action::Duplicate),
            (sc(ALT, Key::D), Action::DuplicateLinked),
            (sc(NONE, Key::H), Action::HideSelected),
            (sc(ALT, Key::H), Action::UnhideAll),
            (sc(CTRL, Key::J), Action::JoinMeshes),
            (sc(NONE, Key::R), Action::SetTool(ToolKind::Rotate)),
            (sc(NONE, Key::S), Action::SetTool(ToolKind::Scale)),
            (sc(NONE, Key::Z), Action::ToggleTextured),
            (sc(SHIFT, Key::A), Action::ShowShapeDialog),
            (sc(NONE, Key::F3), Action::ShowCommandPalette),
            (sc(NONE, Key::Slash), Action::IsolateSelected),
            (sc(ALT_SHIFT, Key::H), Action::HotspotTexture),
            (sc(NONE, Key::F4), Action::SetShade(Shade::Lit)),
        ],
        _ => Vec::new(),
    };
    for (shortcut, action) in replace {
        base.retain(|(s, _)| *s != shortcut);
        base.push((shortcut, action));
    }

    base
}

pub fn parse_shortcut(text: &str) -> Option<KeyboardShortcut> {
    let mut modifiers = Modifiers::NONE;
    let mut key = None;
    for part in text.split('+').map(str::trim).filter(|p| !p.is_empty()) {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "cmd" | "command" => modifiers.command = true,
            "shift" => modifiers.shift = true,
            "alt" | "option" => modifiers.alt = true,
            _ => key = Key::from_name(part).or_else(|| Key::from_name(&part.to_ascii_uppercase())),
        }
    }

    key.map(|k| KeyboardShortcut::new(modifiers, k))
}

pub fn shortcut_to_text(s: &KeyboardShortcut) -> String {
    let mut parts = Vec::new();
    if s.modifiers.command || s.modifiers.ctrl {
        parts.push("Ctrl".to_string());
    }

    if s.modifiers.shift {
        parts.push("Shift".to_string());
    }

    if s.modifiers.alt {
        parts.push("Alt".to_string());
    }

    parts.push(s.logical_key.name().to_string());
    parts.join("+")
}

/// Active shortcuts: the preset with the user's overrides applied.
pub fn shortcuts(prefs: &Prefs) -> Vec<(KeyboardShortcut, Action)> {
    let mut list = preset_bindings(&prefs.keymap_preset);
    for (id, text) in &prefs.key_overrides {
        let Some(action) = bindable_actions().into_iter().find(|a| a.binding_id() == *id) else { continue };
        list.retain(|(_, a)| a.binding_id() != *id);
        if let Some(s) = parse_shortcut(text) {
            list.retain(|(existing, _)| *existing != s);
            list.push((s, action));
        }
    }

    list
}

/// Every action that can carry a key binding, for the keymap editor.
pub fn bindable_actions() -> Vec<Action> {
    let mut out: Vec<Action> = trenchbroom_bindings().into_iter().map(|(_, a)| a).collect();
    for preset in PRESETS {
        out.extend(preset_bindings(preset).into_iter().map(|(_, a)| a));
    }

    out.extend([
        Action::SelectSiblings,
        Action::SelectSameMaterial,
        Action::LockSelected,
        Action::UnlockAll,
        Action::ToggleSnap,
        Action::SetShade(Shade::Textured),
        Action::SetShade(Shade::Flat),
        Action::SnapVertices,
        Action::CreatePrefab,
        Action::InsertPrefab,
        Action::ExplodeInstances,
        Action::ShowTerrainDialog,
        Action::ShowKeymap,
        Action::ShowUvEditor,
        Action::ImportVmf,
        Action::ConvertTextures,
        Action::ImportModel(ModelImport::Mesh),
        Action::ImportModel(ModelImport::Brushes),
        Action::ImportModel(ModelImport::Prop),
        Action::ConvertToBrushes,
        Action::EditMesh,
        Action::UnlinkGroups,
        Action::SetCordonFromSelection,
        Action::ToggleCordon,
        Action::ClearCordon,
        Action::AddLayer,
        Action::CloseGroup,
        Action::SetTool(ToolKind::Mesh),
        Action::ShowUvEditor,
        Action::ToggleTreatAsOne,
        Action::AlignTextureToView,
        Action::ResetTexture,
        Action::CopyAlignment,
        Action::PasteAlignment,
        Action::ShowHotspotEditor,
        Action::ReloadMaterials,
        Action::NewScatterSet,
        Action::ScatterFill,
        Action::ShowScatterPanel,
        Action::SetBlendMaterial,
        Action::MakePlatform,
        Action::ShowLinkDialog,
        Action::ShowReference,
        Action::ShowPreferences,
        Action::ViewLayout(1),
        Action::ViewLayout(2),
        Action::ViewLayout(4),
        Action::ToggleTransformGizmo,
        Action::ToggleGodotOverlays,
        Action::ToggleWalkable,
        Action::OpenGodotEditor,
        Action::BuildInGodot,
        Action::ToggleLiveMode,
    ]);
    for j in gt_geom::Justify::ALL {
        out.push(Action::Justify(j));
    }

    for k in crate::texture_ops::MeshUvKind::ALL {
        out.push(Action::MeshUv(k));
    }

    for op in MeshOp::ALL {
        out.push(Action::MeshOp(op));
    }

    let mut seen = std::collections::BTreeSet::new();
    out.retain(|a| seen.insert(a.binding_id()));
    out
}

pub fn shortcut_text(ctx: &egui::Context, prefs: &Prefs, action: &Action) -> Option<String> {
    shortcuts(prefs).into_iter().find(|(_, a)| a == action).map(|(s, _)| ctx.format_shortcut(&s))
}

/// `InputState::consume_shortcut` that also matches digits by their physical key. With Shift held the logical key is
/// the shifted symbol ("!" for 1 on a US layout), so Ctrl+Shift+1 would never fire otherwise.
pub fn consume_shortcut(input: &mut egui::InputState, shortcut: &KeyboardShortcut) -> bool {
    input.consume_shortcut(shortcut) | consume_physical_digit(&mut input.events, shortcut)
}

fn consume_physical_digit(events: &mut Vec<egui::Event>, shortcut: &KeyboardShortcut) -> bool {
    if !DIGITS.contains(&shortcut.logical_key) && shortcut.logical_key != Key::Num0 {
        return false;
    }

    let before = events.len();
    events.retain(|e| {
        !matches!(e, egui::Event::Key { physical_key: Some(k), pressed: true, modifiers, .. }
            if *k == shortcut.logical_key && modifiers.matches_logically(shortcut.modifiers))
    });
    events.len() != before
}

fn selection_bounds(state: &EditorState) -> Aabb {
    let map = &state.doc.map;
    if state.doc.selection.has_faces() {
        let mut b = Aabb::EMPTY;
        for (id, f) in &state.doc.selection.faces {
            if let Some(brush) = map.brush(*id) {
                for p in brush.face_points(*f) {
                    b.include_point(p);
                }
            } else if let Some(mesh) = map.mesh(*id).filter(|m| *f < m.faces.len()) {
                for p in mesh.face_points(*f) {
                    b.include_point(p);
                }
            }
        }

        return b;
    }

    map.bounds_of(state.doc.selection.nodes.iter().copied())
}

/// The folder file dialogs start in: the current map's, else the Godot project's, since maps have to live inside it.
pub fn dialog_dir(state: &EditorState) -> Option<std::path::PathBuf> {
    let map_dir = state.doc.path.as_deref().and_then(std::path::Path::parent).filter(|d| d.is_dir());
    map_dir.or(state.game.project_root.as_deref()).and_then(|d| std::path::absolute(d).ok())
}

/// Shows a file dialog that starts in `dialog_dir`.
pub fn file_dialog<T>(state: &EditorState, show: impl FnOnce(rfd::FileDialog) -> T) -> T {
    file_dialog_in(dialog_dir(state), show)
}

/// Shows a file dialog that starts in `dir`.
pub fn file_dialog_in<T>(dir: Option<std::path::PathBuf>, show: impl FnOnce(rfd::FileDialog) -> T) -> T {
    let Some(dir) = dir.and_then(|d| std::path::absolute(d).ok()) else { return show(rfd::FileDialog::new()) };
    // Without a desktop portal rfd falls back to zenity, which ignores the start folder and opens the working directory.
    #[cfg(target_os = "linux")]
    let restore = std::env::current_dir().ok().filter(|_| dialogs_use_zenity() && std::env::set_current_dir(&dir).is_ok());
    let result = show(rfd::FileDialog::new().set_directory(&dir));
    #[cfg(target_os = "linux")]
    if let Some(cwd) = restore {
        let _ = std::env::set_current_dir(cwd);
    }

    result
}

/// Whether rfd will show zenity: it is installed and no desktop portal offers a file chooser. Checked once, when the
/// first dialog opens, and an installed zenity counts when there is no `dbus-send` to ask the bus with.
#[cfg(target_os = "linux")]
fn dialogs_use_zenity() -> bool {
    static ZENITY: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ZENITY.get_or_init(|| {
        if !std::env::var_os("PATH").is_some_and(|p| std::env::split_paths(&p).any(|d| d.join("zenity").is_file())) {
            return false;
        }

        // The portal only has the FileChooser interface with a desktop backend behind it. Asking starts a portal that
        // D-Bus activates on demand, which rfd's own call would do a moment later anyway.
        let chooser = session_bus()
            && std::process::Command::new("dbus-send")
                .args(["--session", "--print-reply", "--reply-timeout=1000", "--dest=org.freedesktop.portal.Desktop", "/org/freedesktop/portal/desktop"])
                .arg("org.freedesktop.DBus.Introspectable.Introspect")
                .output()
                .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).contains("\"org.freedesktop.portal.FileChooser\""));
        !chooser
    })
}

/// Whether `dbus-send` and `gdbus` find a session bus: the address in the environment, or the user bus socket both fall
/// back to. Without either they would start a bus daemon of their own, which stays around after the editor quits.
#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) fn session_bus() -> bool {
    use std::os::unix::fs::FileTypeExt;
    std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some_and(|a| !a.is_empty())
        || std::env::var_os("XDG_RUNTIME_DIR")
            .is_some_and(|d| std::fs::symlink_metadata(std::path::Path::new(&d).join("bus")).is_ok_and(|m| m.file_type().is_socket()))
}

pub fn execute(state: &mut EditorState, action: Action, ctx: &egui::Context) {
    state.validate_insert_context();
    run(state, action, ctx);
    state.validate_insert_context();
}

/// Runs an edit that returns `None` when there is nothing to do, recording no undo step then and showing `why`.
fn edit_or<T>(state: &mut EditorState, label: &str, why: &str, f: impl FnOnce(&mut gt_doc::Map, &mut gt_doc::Selection) -> Option<T>) -> Option<T> {
    let result = state.doc.try_edit(label, |m, s| f(m, s).ok_or(())).ok();
    if result.is_none() {
        state.set_status(why);
    }

    result
}

fn run(state: &mut EditorState, action: Action, ctx: &egui::Context) {
    if matches!(action, Action::Rotate { .. } | Action::Flip { .. } | Action::Nudge(_) | Action::Duplicate) {
        state.last_repeatable = Some(action.clone());
    }

    let opts = state.opts();
    let parent = state.insert_parent();
    let open_groups = state.open_groups.clone();
    let grid = state.grid;
    match action {
        Action::NewMap => {
            if state.doc.is_modified() {
                state.open_tab(Document::new());
                state.set_status("New map opened in a tab, the current map has unsaved changes");
            } else {
                state.reset_document(Document::new());
                state.set_status("New map");
            }
        }
        Action::NewTab => {
            state.open_tab(Document::new());
            state.set_status("New map tab");
        }
        Action::CloseTab => {
            if state.doc.is_modified() {
                state.set_status("Save or discard changes before closing the tab (File > Save)");
            } else if !state.close_tab() {
                state.reset_document(Document::new());
            }
        }
        Action::NextTab => {
            let (titles, active) = state.tab_titles();
            state.switch_tab((active + 1) % titles.len());
        }
        Action::OpenMap => {
            let picked = file_dialog(state, |d| {
                d.add_filter("GodotTrench map", &["gtm"]).add_filter("Map as JSON", &["json"]).add_filter("Autosave (recover a map)", &["autosave"]).pick_file()
            });
            if let Some(path) = picked
                && let Err(e) = open_map_in_tab(state, &path)
            {
                state.set_status(format!("Open failed: {e}"));
            }
        }
        Action::OpenMapFile(path) => {
            if let Err(e) = open_map_in_tab(state, &path) {
                state.set_status(format!("Open failed: {e}"));
            }
        }
        Action::OpenProject => {
            if let Some(dir) = rfd::FileDialog::new().set_title("Select Godot project folder").pick_folder() {
                match gt_formats::game::find_project_root(&dir) {
                    Some(root) => state.load_project(&root),
                    None => state.set_status("No project.godot found in that folder or its parents"),
                }
            }
        }
        Action::ReloadProject => {
            if let Some(root) = state.game.project_root.clone() {
                state.load_project(&root);
            }
        }
        Action::Save => match state.doc.path.clone() {
            Some(p) => {
                if let Err(e) = state.save_map(&p) {
                    state.set_status(format!("Save failed: {e}"));
                }
            }
            None => execute(state, Action::SaveAs, ctx),
        },
        Action::SaveAs => {
            let name = state.doc.path.as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "map.gtm".into());
            let picked = file_dialog(state, |d| d.add_filter("GodotTrench map", &["gtm"]).add_filter("Map as JSON", &["json"]).set_file_name(name).save_file());
            if let Some(path) = picked
                && let Err(e) = state.save_map(&path)
            {
                state.set_status(format!("Save failed: {e}"));
            }
        }
        Action::Undo => {
            if let Some(l) = state.undo() {
                state.set_status(format!("Undo {l}"));
            }
        }
        Action::Redo => {
            if let Some(l) = state.redo() {
                state.set_status(format!("Redo {l}"));
            }
        }
        Action::RepeatLast => match state.last_repeatable.clone() {
            Some(last) => execute(state, last, ctx),
            None => state.set_status("Nothing to repeat yet, Repeat Last runs the last rotate, flip, nudge or duplicate again"),
        },
        Action::Delete => {
            if state.doc.selection.nodes.is_empty() {
                return;
            }

            let n = state.doc.edit("Delete", ops::delete_selection);
            state.set_status(format!("Deleted {n} objects"));
        }
        Action::Rename => match state.doc.selection.nodes.first().copied().filter(|_| state.doc.selection.nodes.len() == 1) {
            Some(id) => {
                state.renaming = Some(id);
                state.outliner_reveal = Some(id);
            }
            None => state.set_status("Select one object to rename it"),
        },
        Action::Duplicate => {
            let offset = DVec3::new(grid, 0.0, grid);
            state.doc.edit("Duplicate", |m, s| ops::duplicate_selection(m, s, offset, opts));
        }
        Action::DuplicateLinked => {
            let offset = DVec3::new(grid, 0.0, grid);
            let has_groups = state.doc.selection.nodes.iter().any(|id| matches!(state.doc.map.get(*id).map(|n| &n.kind), Some(gt_doc::NodeKind::Group(_))));
            let linked = edit_or(state, "Duplicate Linked", "Select objects or groups to duplicate linked", |m, s| {
                if !has_groups {
                    ops::group_selection(m, s, "Linked", parent)?;
                }

                Some(ops::duplicate_linked(m, s, offset, opts)).filter(|ids| !ids.is_empty())
            });
            if let Some(ids) = linked {
                state.set_status(format!("Created {} linked group(s). Edits inside one update the others", ids.len()));
            }
        }
        Action::UnlinkGroups => state.doc.edit("Unlink Groups", |m, s| ops::unlink_groups(m, s)),
        Action::SelectAll => state.doc.select(|m, s| ops::select_all(m, s, &open_groups)),
        Action::SelectNone => {
            if state.tool != ToolKind::Select {
                state.tool = ToolKind::Select;
            } else {
                state.doc.select(|_, s| s.clear());
            }
        }
        Action::SelectInverse => state.doc.select(|m, s| ops::select_inverse(m, s, &open_groups)),
        // Not pure selection changes: like TrenchBroom they delete the selecting brushes, so they stay undoable edits.
        Action::SelectTouching | Action::SelectInside => {
            let inside = action == Action::SelectInside;
            let label = if inside { "Select Inside" } else { "Select Touching" };
            edit_or(state, label, "Select the brushes to select with first", |m, s| {
                (!s.brushes(m).is_empty()).then(|| ops::select_touching(m, s, &open_groups, inside))
            });
        }
        Action::SelectSiblings => state.doc.select(ops::select_siblings),
        Action::SelectSameMaterial => {
            let mat = first_selected_material(state).unwrap_or_else(|| state.current_material.clone());
            state.doc.select(|m, s| ops::select_by_material(m, s, &mat, &open_groups));
        }
        Action::Group => {
            edit_or(state, "Group", "Nothing to group", |m, s| ops::group_selection(m, s, "Group", parent));
        }
        Action::Ungroup => state.doc.edit("Ungroup", ops::ungroup_selection),
        Action::HideSelected => state.doc.edit("Hide", ops::hide_selection),
        Action::IsolateSelected => state.doc.edit("Isolate", |m, s| ops::isolate_selection(m, s)),
        Action::UnhideAll => state.doc.edit("Show All", |m, _| ops::unhide_all(m)),
        Action::LockSelected => state.doc.edit("Lock", |m, s| {
            let roots = ops::selection_roots(m, s);
            ops::set_locked(m, &roots, true);
            s.clear();
        }),
        Action::UnlockAll => state.doc.edit("Unlock All", |m, _| {
            let ids: Vec<NodeId> = m.nodes.keys().copied().collect();
            ops::set_locked(m, &ids, false);
        }),
        Action::GridDown => {
            state.grid = (state.grid / 2.0).max(0.125);
            state.set_status(format!("Grid {}", state.grid));
        }
        Action::GridUp => {
            state.grid = (state.grid * 2.0).min(1024.0);
            state.set_status(format!("Grid {}", state.grid));
        }
        Action::ToggleSnap => state.snap = !state.snap,
        Action::UiScaleUp | Action::UiScaleDown | Action::UiScaleReset => {
            match action {
                Action::UiScaleUp => state.prefs.step_ui_scale(1),
                Action::UiScaleDown => state.prefs.step_ui_scale(-1),
                _ => {
                    state.prefs.ui_scale = 1.0;
                    state.prefs.follow_display_scaling = true;
                }
            }

            state.set_status(format!("UI scale {:.0}%", state.prefs.ui_scale * 100.0));
        }
        Action::ToggleUvLock => {
            state.uv_lock = !state.uv_lock;
            state.set_status(if state.uv_lock { "UV lock on" } else { "UV lock off" });
        }
        Action::ToggleTransformGizmo => {
            state.prefs.transform_gizmo = !state.prefs.transform_gizmo;
            state.set_status(if state.prefs.transform_gizmo { "Transform gizmo on" } else { "Transform gizmo off" });
        }
        Action::ToggleGodotOverlays => {
            state.prefs.godot_overlays = !state.prefs.godot_overlays;
            state.set_status(if state.prefs.godot_overlays { "Godot overlays shown" } else { "Godot overlays hidden" });
        }
        Action::ToggleWalkable => toggle_walkable(state),
        Action::ToggleTextured => {
            state.prefs.shade = match state.prefs.shade {
                Shade::Textured => Shade::Flat,
                Shade::Flat => Shade::Lit,
                Shade::Lit => Shade::Wireframe,
                Shade::Wireframe => Shade::Textured,
            };
            state.set_status(format!("Shading: {}", state.prefs.shade.label()));
        }
        Action::SetShade(s) => {
            state.prefs.shade = if state.prefs.shade == s && s == Shade::Lit { Shade::Textured } else { s };
            state.set_status(format!("Shading: {}", state.prefs.shade.label()));
        }
        Action::CsgSubtract => {
            let carve = state.carve_material;
            let (n, replaced) = state.doc.edit("CSG Subtract", |m, s| ops::csg_subtract(m, s, carve));
            state.replaced = replaced;
            state.set_status(format!("Subtracted from {n} brushes"));
        }
        Action::CsgMerge | Action::CsgIntersect => {
            let before = state.doc.selection.brushes(&state.doc.map);
            let result = if action == Action::CsgMerge {
                let mat = state.current_material.clone();
                edit_or(state, "CSG Merge", "Select at least two brushes to merge", |m, s| ops::csg_merge(m, s, &mat))
            } else {
                edit_or(state, "CSG Intersect", "Select at least two brushes that intersect", ops::csg_intersect)
            };
            if let Some(new_id) = result {
                state.replaced = before.into_iter().map(|id| (id, vec![new_id])).collect();
            }
        }
        Action::CsgHollow => {
            let t = state.hollow_thickness.max(state.grid.min(state.hollow_thickness));
            state.replaced = state.doc.edit("Hollow", |m, s| ops::csg_hollow(m, s, t));
        }
        Action::Rotate { axis, degrees } => {
            let center = ops::selection_center(&state.doc.map, &state.doc.selection, grid);
            let mut dir = DVec3::ZERO;
            dir[axis] = 1.0;
            let m = ops::rotation_about(center, dir, degrees);
            state.doc.edit("Rotate", |map, s| ops::transform_selection(map, s, &m, opts));
        }
        Action::Flip { axis } => {
            let center = ops::selection_center(&state.doc.map, &state.doc.selection, grid);
            let m = ops::flip_about(center, axis);
            state.doc.edit("Flip", |map, s| ops::transform_selection(map, s, &m, opts));
        }
        Action::Nudge(offset) => {
            if !state.doc.selection.nodes.is_empty() {
                state.doc.edit("Move", |m, s| ops::translate_selection(m, s, offset, opts));
            }
        }
        Action::SetTool(t) => {
            state.tool = if state.tool == t { ToolKind::Select } else { t };
            state.set_status(format!("{} tool", state.tool.label()));
        }
        Action::EditMesh => {
            if state.tool == ToolKind::Mesh {
                state.tool = ToolKind::Select;
                state.set_status("Object mode");
                return;
            }

            let has_meshes = !state.doc.selection.meshes(&state.doc.map).is_empty();
            if !has_meshes && !state.doc.selection.brushes(&state.doc.map).is_empty() {
                state.doc.edit("Convert to Mesh", |m, s| ops::convert_to_mesh(m, s, false));
            }

            state.tool = ToolKind::Mesh;
            let (verts, tris) = edit_mesh_weight(state);
            if verts > HEAVY_VERTS || tris > HEAVY_TRIS {
                // High-res meshes are heavy to edit vertex by vertex, warn but never block, the tools still work.
                state.set_status(format!(
                    "Mesh edit mode, {verts} vertices, {tris} triangles, that is high res so editing may be slow (Tab returns to object mode)"
                ));
            } else {
                state.set_status("Mesh edit mode (Tab returns to object mode)");
            }
        }
        Action::ToggleEditMode => {
            if matches!(state.tool, ToolKind::Mesh | ToolKind::Vertex) {
                state.tool = ToolKind::Select;
                state.set_status("Object mode");
            } else if !state.doc.selection.meshes(&state.doc.map).is_empty() {
                execute(state, Action::EditMesh, ctx);
            } else if !state.doc.selection.brushes(&state.doc.map).is_empty() {
                state.tool = ToolKind::Vertex;
                state.set_status("Vertex editing the selected brushes (Tab returns). To edit them as a mesh, use Mesh > Edit Mesh");
            } else {
                state.set_status("Select a brush or mesh, then Tab edits its vertices");
            }
        }
        Action::ConvertToMesh => {
            let n = state.doc.edit("Convert to Mesh", |m, s| ops::convert_to_mesh(m, s, false)).len();
            state.set_status(format!("Converted {n} brush(es) to meshes"));
        }
        Action::ConvertToBrushes => {
            let n = state.doc.edit("Convert to Brushes", ops::convert_to_brushes).len();
            state.set_status(format!("Converted {n} mesh(es) to convex brushes"));
        }
        Action::JoinMeshes => {
            edit_or(state, "Join Meshes", "Select two or more meshes or brushes to join", ops::join_meshes);
        }
        Action::FocusSelection => {
            let b = selection_bounds(state);
            state.focus_request = Some(if b.is_empty() { state.doc.map.bounds_of(state.doc.map.layers.clone()) } else { b });
        }
        Action::CreateBrushEntity(classname) => {
            edit_or(state, "Create Brush Entity", "Select brushes first", |m, s| ops::create_brush_entity(m, s, &classname, parent));
        }
        Action::CreatePointEntity { classname, at } => {
            let origin = state.snap(at.or(state.cursor_world).unwrap_or(DVec3::ZERO));
            state.doc.edit("Create Entity", |m, s| {
                let id = ops::create_point_entity(m, parent, &classname, origin);
                s.clear();
                s.select_node(id);
            });
        }
        Action::PlaceEntities { classnames, at, normal, row } => place_entities(state, &classnames, at, normal, row),
        Action::PlaceModel { path, at } => match place_model_mesh(state, &path, at) {
            Ok(msg) => state.set_status(msg),
            Err(e) => state.set_status(format!("Place model failed: {e}")),
        },
        Action::CreateDecal { material, at, normal, size } => create_decal(state, &material, at, normal, size),
        Action::MoveToWorld => {
            let layer = state.valid_layer();
            state.doc.edit("Move to World", |m, s| ops::move_brushes_to_world(m, s, layer));
        }
        Action::MoveToLayer(layer) => state.doc.edit("Move to Layer", |m, s| ops::move_to_layer(m, s, layer)),
        Action::AddLayer => {
            let n = state.doc.map.layers.len() + 1;
            let id = state.doc.edit("Add Layer", |m, _| m.add_layer(&format!("Layer {n}")));
            state.current_layer = id;
        }
        Action::SnapVertices => state.doc.edit("Snap Vertices", |m, s| ops::snap_vertices(m, s, grid)),
        Action::ApplyMaterial(name) => {
            state.current_material = name.clone();
            state.note_material(&name);
            if !state.doc.selection.is_empty() {
                state.doc.edit("Apply Material", |m, s| ops::apply_material(m, s, &name));
            }
        }
        Action::Copy | Action::Cut => {
            let roots = ops::selection_roots(&state.doc.map, &state.doc.selection);
            if roots.is_empty() {
                return;
            }

            ctx.copy_text(format::nodes_to_string(&state.doc.map, &roots));
            if action == Action::Cut {
                state.doc.edit("Cut", ops::delete_selection);
            }

            state.set_status(format!("Copied {} objects", roots.len()));
        }
        Action::Paste(text) => {
            let cursor = state.cursor_world;
            let (snap, grid) = (state.snap, state.grid);
            let result = state.doc.try_edit("Paste", |m, s| {
                let ids = format::paste_nodes(m, parent, &text)?;
                s.clear();
                s.nodes.extend(ids.iter().copied());
                // Paste at cursor: center the pasted objects under the mouse, snapped to grid.
                let b = m.bounds_of(ids.iter().copied());
                if let Some(cursor) = cursor
                    && !b.is_empty()
                {
                    let offset = cursor - b.center();
                    let offset = if snap { gt_core::snap_vec_to_grid(offset, grid) } else { offset };
                    ops::translate_selection(m, s, DVec3::new(offset.x, 0.0, offset.z), opts);
                }

                Ok::<_, format::FormatError>(ids)
            });
            if result.is_err() {
                state.set_status("Clipboard does not contain GodotTrench objects");
            }
        }
        Action::OpenGroup => {
            let groups: Vec<NodeId> = state
                .doc
                .selection
                .nodes
                .iter()
                .copied()
                .filter(|id| matches!(state.doc.map.get(*id).map(|n| &n.kind), Some(gt_doc::NodeKind::Group(_))))
                .collect();
            if let Some(g) = groups.first() {
                state.open_groups.push(*g);
                state.doc.select(|_, s| s.clear());
            }
        }
        Action::CloseGroup => {
            if let Some(g) = state.open_groups.pop() {
                state.doc.select(|_, s| {
                    s.clear();
                    s.nodes.insert(g);
                });
            }
        }

        // Handled by the app, which owns dialogs, viewports and tool state.
        Action::ShowCommandPalette
        | Action::ShowShapeDialog
        | Action::ShowTerrainDialog
        | Action::ShowKeymap
        | Action::ShowUvEditor
        | Action::MeshOp(_)
        | Action::StoreCamera(_)
        | Action::RecallCamera(_) => {}
        Action::RunGodotProject => launch_godot(state, false),
        // A second Godot editor with the project could not take the live link port, so show the one that has it.
        Action::OpenGodotEditor | Action::FocusGodot => match &state.link {
            Some(link) if state.godot_has_project() => {
                link.request(crate::live_link::Request::Focus);
                if action == Action::OpenGodotEditor {
                    state.set_status("The Godot editor already has this project open, showing it");
                }
            }
            _ => launch_godot(state, true),
        },
        Action::BuildInGodot => build_in_godot(state),
        Action::ToggleLiveMode => {
            if !state.prefs.live_mode && !state.prefs.live_link {
                state.set_status(LIVE_LINK_OFF);
                return;
            }

            state.prefs.live_mode = !state.prefs.live_mode;
            state.set_status(match (state.prefs.live_mode, state.godot_has_project()) {
                (true, true) => "Live mode on: edits reach Godot before you save",
                (true, false) => "Live mode on: edits reach Godot once its editor has this project and map open",
                (false, _) => "Live mode off: Godot keeps what it shows until the next save",
            });
        }
        Action::CreateDisplacement(power) => {
            let faces = displacement_candidates(state);
            let n = state.doc.edit("Create Displacement", |m, _| gt_doc::terrain::create_displacements(m, &faces, power));
            state.set_status(if n == 0 {
                "Select quad faces (Shift+click) or brushes with a top face".to_string()
            } else {
                format!("{n} displacement(s) with power {power}")
            });
            if n > 0 {
                state.tool = ToolKind::Sculpt;
            }
        }
        Action::RemoveDisplacement => {
            let faces = displacement_candidates(state);
            state.doc.edit("Remove Displacement", |m, _| gt_doc::terrain::remove_displacements(m, &faces));
        }
        Action::SewDisplacements => {
            let targets = state.doc.selection.brushes(&state.doc.map);
            let faces = gt_doc::terrain::displacement_faces(&state.doc.map, &targets);
            let n = state.doc.edit("Sew Displacements", |m, _| gt_doc::terrain::sew(m, &faces));
            state.set_status(format!("Sewed {n} shared vertices"));
        }
        Action::TerrainFlatten => {
            let ids = state.doc.selection.terrains(&state.doc.map);
            state.doc.edit("Flatten Terrain", |m, _| {
                for id in &ids {
                    if let Some(t) = m.terrain_mut(*id) {
                        t.heights.iter_mut().for_each(|h| *h = 0.0);
                    }
                }
            });
        }
        Action::TerrainAutoPaint => {
            let ids = state.doc.selection.terrains(&state.doc.map);
            let base = state.auto_paint_base;
            state.doc.edit("Auto Paint Terrain", |m, _| {
                for id in &ids {
                    if let Some(t) = m.terrain_mut(*id) {
                        t.auto_paint_from(base);
                    }
                }
            });
        }
        Action::ImportQuakeMap => {
            if let Some(path) = file_dialog(state, |d| d.add_filter("Quake / TrenchBroom map", &["map"]).pick_file()) {
                match import_quake_map(state, &path) {
                    Ok(_) => crate::texture_convert::offer_for_map(state, &path),
                    Err(e) => state.set_status(format!("Import failed: {e}")),
                }
            }
        }
        Action::ImportVmf => {
            if let Some(path) = file_dialog(state, |d| d.add_filter("Hammer map", &["vmf"]).pick_file()) {
                match import_vmf(state, &path) {
                    Ok(_) => crate::texture_convert::offer_for_map(state, &path),
                    Err(e) => state.set_status(format!("Import failed: {e}")),
                }
            }
        }
        Action::ConvertTextures => crate::texture_convert::convert_folder_dialog(state),
        Action::ExportQuakeMap | Action::ExportQuakeMapCordon => {
            let name = state.doc.path.as_ref().and_then(|p| p.file_stem()).map(|s| format!("{}.map", s.to_string_lossy())).unwrap_or_else(|| "map.map".into());
            if let Some(path) = file_dialog(state, |d| d.add_filter("Quake / TrenchBroom map", &["map"]).set_file_name(name).save_file()) {
                let options = gt_formats::quake_map::ExportOptions { cordon: action == Action::ExportQuakeMapCordon, ..Default::default() };
                match std::fs::write(&path, gt_formats::quake_map::export_with(&state.doc.map, options)) {
                    Ok(()) => state.set_status(format!("Exported {}", path.display())),
                    Err(e) => state.set_status(format!("Export failed: {e}")),
                }
            }
        }
        Action::ImportModel(mode) => {
            let Some(picked) = file_dialog(state, |d| d.add_filter("Models", &crate::models::MODEL_EXTS).pick_file()) else { return };
            let path = if mode == ModelImport::Prop { prop_model_in_project(state, &picked) } else { Some(picked) };
            if let Some(path) = path {
                let at = state.snap(state.cursor_world.unwrap_or(DVec3::ZERO));
                match import_model(state, &path, mode, at) {
                    Ok(n) => state.set_status(format!("Imported {} ({n} object(s))", path.display())),
                    Err(e) => state.set_status(format!("Import failed: {e}")),
                }
            }
        }
        Action::ReloadModels => {
            state.models.clear();
            state.model_thumbs.clear();
            let game = state.game.clone();
            state.model_library.rescan(&game);
            state.set_status("Models reloaded");
        }
        Action::SetCordonFromSelection => {
            let b = selection_bounds(state);
            if b.is_empty() {
                state.set_status("Select the objects that define the cordon");
                return;
            }

            state.doc.edit("Set Cordon", |m, _| {
                m.editor.cordon = Some(b);
                m.editor.cordon_enabled = true;
            });
        }
        Action::ToggleCordon => {
            if state.doc.map.editor.cordon.is_none() {
                state.set_status("No cordon set, use View > Cordon > Set Cordon from Selection");
                return;
            }

            state.doc.edit("Toggle Cordon", |m, _| m.editor.cordon_enabled = !m.editor.cordon_enabled);
        }
        Action::ClearCordon => state.doc.edit("Clear Cordon", |m, _| {
            m.editor.cordon = None;
            m.editor.cordon_enabled = false;
        }),
        Action::Justify(mode) => {
            let faces = crate::texture_ops::target_faces(state);
            let n = crate::texture_ops::justify(state, &faces, mode, state.treat_as_one);
            state.set_status(format!("Justified {n} faces {}", mode.label()));
        }
        Action::ToggleTreatAsOne => {
            state.treat_as_one = !state.treat_as_one;
            state.set_status(if state.treat_as_one { "Treat as one: on" } else { "Treat as one: off" });
        }
        Action::ResetTexture => {
            let faces = crate::texture_ops::target_faces(state);
            crate::texture_ops::reset(state, &faces, DVec2::ONE);
        }
        Action::CopyAlignment => match crate::texture_ops::target_faces(state).first() {
            Some(face) => {
                crate::texture_ops::eyedropper(state, *face);
            }
            None => state.set_status("Select a face to copy its alignment"),
        },
        Action::PasteAlignment => {
            let faces = crate::texture_ops::target_faces(state);
            if crate::texture_ops::paste_alignment(state, &faces, false) == 0 {
                state.set_status("Nothing to paste, copy a face alignment first");
            }
        }
        Action::TexelDensity(d) => {
            let faces = crate::texture_ops::target_faces(state);
            let n = crate::texture_ops::set_density(state, &faces, d);
            state.set_status(format!("Texel density {d} on {n} faces"));
        }
        Action::ReloadMaterials => {
            state.material_reload = true;
            state.set_status("Reloading materials");
        }
        Action::AlignTextureToView | Action::MeshUv(_) | Action::ShowHotspotEditor | Action::EditHotspots(_) => {}
        Action::HotspotTexture => {
            let n = hotspot_texture(state);
            state.set_status(if n == 0 {
                "No hotspot rectangles for the selected faces' materials (add <texture>.hotspots.json)".to_string()
            } else {
                format!("Hotspot textured {n} face(s)")
            });
        }
        Action::CreatePrefab => create_prefab(state),
        Action::ExplodeInstances => explode_instances(state),
        Action::InsertPrefab => {
            if let Some(path) = file_dialog(state, |d| d.add_filter("GodotTrench map", &["gtm"]).pick_file()) {
                let reference = prefab_reference(&path, state.doc.path.as_deref(), state.game.project_root.as_deref());
                let origin = state.snap(state.cursor_world.unwrap_or(DVec3::ZERO));
                state.doc.edit("Insert Prefab", |m, s| {
                    let id = m.insert(
                        parent,
                        gt_doc::NodeKind::Instance(gt_doc::map::Instance { path: reference, origin, angles: DVec3::ZERO, fixup: String::new() }),
                    );
                    s.clear();
                    s.select_node(id);
                });
            }
        }
        Action::OpenPrefab => {
            let Some((_, inst)) = selected_instances(state).into_iter().next() else { return };
            match crate::prefabs::resolve(&inst.path, state.doc.path.as_deref(), state.game.project_root.as_deref()) {
                Some(p) => {
                    if let Err(e) = open_map_in_tab(state, &p) {
                        state.set_status(format!("Cannot open prefab: {e}"));
                    }
                }
                None => state.set_status("Cannot resolve prefab path"),
            }
        }
        Action::ActivateScatter(id) => {
            if let Some(set) = state.doc.map.scatter(id) {
                let name = set.name.clone();
                state.active_scatter = Some(id);
                state.tool = ToolKind::Scatter;
                state.set_status(format!("Painting into scatter set '{name}'"));
            }
        }
        Action::NewScatterSet => {
            crate::scatter_tool::new_empty_set(state);
            state.set_status("New scatter set, drag models into it from the Models panel");
        }
        Action::ScatterFill => {
            let seed = state.prefs.scatter.seed;
            let mut rng = gt_doc::scatter::Rng::new(if seed == 0 { time_seed() } else { seed });
            match crate::scatter_tool::fill(state, &mut rng) {
                Ok(n) => state.set_status(format!("Filled the scatter targets with {n} instances")),
                Err(e) => state.set_status(e),
            }
        }
        Action::ScatterPreset(name) => {
            if crate::scatter_tool::new_set_from_preset(state, &name).is_some() {
                state.set_status(format!("New scatter set from the {name} preset on its own layer"));
            }
        }
        Action::InstallNatureModels => match crate::scatter_tool::install_nature(state, false) {
            Ok(0) => state.set_status(format!("The nature models are already installed in {}", gt_doc::scatter::NATURE_DIR)),
            Ok(n) => state.set_status(format!("Installed {n} nature models into {}", gt_doc::scatter::NATURE_DIR)),
            Err(e) => state.set_status(e),
        },
        Action::ScatterToEntities => {
            let sets: Vec<NodeId> = state.doc.selection.nodes.iter().copied().filter(|id| state.doc.map.scatter(*id).is_some()).collect();
            let n: usize = sets.into_iter().map(|id| crate::scatter_tool::bake_to_entities(state, id)).sum();
            state.set_status(format!("Converted {n} scatter instances to entities"));
        }
        Action::SetBlendMaterial => {
            let material = state.current_material.clone();
            let n = crate::blend_tool::set_blend_material(state, Some(&material));
            state.set_status(format!("{material} blends into {n} faces, paint the blend with the Blend tool (Shift+G)"));
        }
        Action::ClearBlendMaterial => {
            let n = crate::blend_tool::set_blend_material(state, None);
            state.set_status(format!("Removed the blend material from {n} faces"));
        }
        Action::SetBlendTiling { detile, uv_scale, sharpen } => {
            let n = crate::blend_tool::set_blend_tiling(state, detile, uv_scale, sharpen);
            state.set_status(format!("{n} faces now repeat the blend material {uv_scale:.2}x with de-tiling {detile:.2}"));
        }
        Action::MakeDoor { kind, trigger } => {
            let ids: Vec<NodeId> = state.doc.selection.nodes.iter().copied().collect();
            if let Err(e) = crate::entity_wizards::make_door(state, &ids, &kind, trigger) {
                state.set_status(e);
            }
        }
        Action::MakePlatform => {
            let ids: Vec<NodeId> = state.doc.selection.nodes.iter().copied().collect();
            let b = state.doc.map.bounds_of(ids.iter().copied());
            let travel = DVec3::new(0.0, (b.size().y * 4.0).max(grid * 8.0), 0.0);
            match crate::entity_wizards::make_platform(state, &ids, travel, 0) {
                Ok(_) => state.set_status("func_platform created, drag the travel handle to set where it goes"),
                Err(e) => state.set_status(e),
            }
        }
        Action::VolumeAroundSelection(classname) => {
            let b = state.doc.map.bounds_of(state.doc.selection.nodes.iter().copied());
            if b.is_empty() {
                state.set_status("Select what the volume should surround");
                return;
            }

            let props = crate::volume_tool::default_props(&classname);
            if let Err(e) = crate::entity_wizards::make_volume(state, &classname, &b.expanded(grid), &props, Vec::new()) {
                state.set_status(e);
            }
        }

        Action::ShowScatterPanel
        | Action::ShowLinkDialog
        | Action::ShowReference
        | Action::ShowPreferences
        | Action::ToggleMaximizeView
        | Action::ViewLayout(_)
        | Action::ToggleView(_) => {}
        Action::CreateBrushFromBounds => {
            let b = state.last_bounds;
            let mat = state.current_material.clone();
            if let Ok(brush) = gt_geom::Brush::from_aabb(&b, &mat) {
                state.doc.edit("Create Brush", |m, s| {
                    let id = ops::create_brush(m, parent, brush);
                    s.clear();
                    s.select_node(id);
                });
            }
        }
    }
}

/// Saves every map with unsaved changes, asking for a file name for untitled ones. Returns false when one was not
/// saved, its tab is then left active.
pub fn save_all(state: &mut EditorState, ctx: &egui::Context) -> bool {
    let start = state.active_tab.min(state.tabs.len());
    for i in state.modified_tabs() {
        state.switch_tab(i);
        execute(state, Action::Save, ctx);
        if state.doc.is_modified() {
            return false;
        }
    }

    state.switch_tab(start);
    true
}

pub fn time_seed() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(7)
}

/// Opens a map in a new tab, or in the current one when it is an untouched empty map.
pub fn open_map_in_tab(state: &mut EditorState, path: &std::path::Path) -> Result<(), String> {
    let path = crate::state::absolute(path);
    let path = path.as_path();
    let map_path = crate::state::autosave_source(path).unwrap_or_else(|| path.to_path_buf());
    let active = state.active_tab.min(state.tabs.len());
    let existing = (0..=state.tabs.len()).position(|i| {
        let doc = if i == active { &state.doc } else { &state.tabs[if i > active { i - 1 } else { i }].doc };
        doc.path.as_deref() == Some(map_path.as_path())
    });
    if let Some(i) = existing {
        state.switch_tab(i);
        if map_path != path {
            state.set_status(format!("{} is already open, close its tab to recover the autosave", map_path.display()));
        }

        return Ok(());
    }

    let untouched = state.doc.path.is_none() && !state.doc.is_modified() && state.doc.map.nodes.len() <= 1;
    if untouched {
        return state.open_map(path);
    }

    state.open_tab(crate::state::load_document(path)?);
    state.after_open(path);
    Ok(())
}

/// How an instance node refers to a prefab file: relative to the map, res:// inside the project, or absolute.
pub fn prefab_reference(prefab: &std::path::Path, map_path: Option<&std::path::Path>, project_root: Option<&std::path::Path>) -> String {
    if let Some(dir) = map_path.and_then(|m| m.parent())
        && let Ok(rel) = prefab.strip_prefix(dir)
    {
        return rel.to_string_lossy().replace('\\', "/");
    }

    if let Some(root) = project_root
        && let Some(res) = gt_formats::game::to_res_path(root, prefab)
    {
        return res;
    }

    prefab.to_string_lossy().replace('\\', "/")
}

fn create_prefab(state: &mut EditorState) {
    let roots = ops::selection_roots(&state.doc.map, &state.doc.selection);
    if roots.is_empty() {
        state.set_status("Select objects to turn into a prefab");
        return;
    }

    let Some(map_path) = state.doc.path.clone() else {
        state.set_status("Save the map first, prefab paths are stored relative to it");
        return;
    };
    let Some(path) = file_dialog(state, |d| d.add_filter("GodotTrench map", &["gtm"]).set_file_name("prefab.gtm").save_file()) else {
        return;
    };

    let bounds = state.doc.map.bounds_of(roots.iter().copied());
    let mut pivot = state.snap(bounds.center());
    pivot.y = bounds.min.y;
    let mut prefab = gt_doc::Map::new();
    prefab.properties.insert("classname".into(), "worldspawn".into());
    let layer = prefab.default_layer();
    let text = format::nodes_to_string(&state.doc.map, &roots);
    let Ok(ids) = format::paste_nodes(&mut prefab, layer, &text) else { return };
    let mut sel = gt_doc::Selection::default();
    sel.nodes.extend(ids);
    ops::translate_selection(&mut prefab, &sel, -pivot, ops::EditOptions { uv_lock: true, grid: 0.0 });
    if let Err(e) = format::save(&prefab, &path) {
        state.set_status(format!("Could not save prefab: {e}"));
        return;
    }

    let reference = prefab_reference(&path, Some(&map_path), state.game.project_root.as_deref());
    let parent = state.insert_parent();
    let layer = state.valid_layer();
    state.doc.edit("Create Prefab", |m, s| {
        ops::delete_selection(m, s);
        // The selection can hold the open group or the current layer itself.
        let parent = [parent, layer].into_iter().find(|p| m.contains(*p)).unwrap_or_else(|| m.default_layer());
        let id = m.insert(
            parent,
            gt_doc::NodeKind::Instance(gt_doc::map::Instance { path: reference.clone(), origin: pivot, angles: DVec3::ZERO, fixup: String::new() }),
        );
        s.select_node(id);
    });
    state.set_status(format!("Prefab saved to {}", path.display()));
}

const BRUSH_ENTITY_BOX: f64 = 64.0;

fn support(b: &Aabb, dir: DVec3) -> f64 {
    (0..3).map(|i| (b.min[i] * dir[i]).max(b.max[i] * dir[i])).sum()
}

pub fn place_entities(state: &mut EditorState, classnames: &[String], at: Option<DVec3>, normal: Option<DVec3>, row: DVec3) {
    enum Placed {
        Point { origin: DVec3, angles: DVec3 },
        Brush(gt_geom::Brush),
    }

    let start = at.or(state.cursor_world).unwrap_or(DVec3::ZERO);
    let gap = state.grid.max(8.0);
    let items: Vec<(&String, Option<&gt_formats::EntityDef>, Aabb)> = classnames
        .iter()
        .map(|c| {
            let def = state.game.entity(c);
            let bounds = match def {
                Some(d) if d.kind == gt_formats::EntityKind::Solid => Aabb::from_center_size(DVec3::ZERO, DVec3::splat(BRUSH_ENTITY_BOX)),
                Some(d) => d.bounds(),
                None => Aabb::new(DVec3::splat(-8.0), DVec3::splat(8.0)),
            };
            (c, def, bounds)
        })
        .collect();
    let total: f64 = items.iter().map(|(_, _, b)| b.size().dot(row).abs()).sum::<f64>() + gap * items.len().saturating_sub(1) as f64;
    let mut offset = -total / 2.0;
    let mut placed = Vec::new();
    for (classname, def, bounds) in &items {
        let width = bounds.size().dot(row).abs();
        let slot = start + row * (offset + width / 2.0 - bounds.center().dot(row));
        offset += width + gap;
        let item = if crate::scene::is_decal(*def) {
            // Decals project along their local -Y, so local +Y is turned to face out of the surface.
            let q = gt_core::DQuat::from_rotation_arc(DVec3::Y, normal.unwrap_or(DVec3::Y).normalize());
            let (y, x, z) = q.to_euler(gt_core::EulerRot::YXZ);
            Placed::Point { origin: slot, angles: DVec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees()).map(|a| (a * 1e4).round() / 1e4) }
        } else {
            let rested = slot + normal.map(|n| n * support(bounds, -n)).unwrap_or_default();
            match def {
                Some(d) if d.kind == gt_formats::EntityKind::Solid => {
                    let min = state.snap(rested + bounds.min);
                    let Ok(brush) = gt_geom::Brush::from_aabb(&Aabb::new(min, min + bounds.size()), &state.current_material) else { continue };
                    Placed::Brush(brush)
                }
                _ => Placed::Point { origin: state.snap(rested), angles: DVec3::ZERO },
            }
        };
        placed.push((classname.to_string(), item));
    }

    if placed.is_empty() {
        return;
    }

    let parent = state.insert_parent();
    let label = Action::PlaceEntities { classnames: classnames.to_vec(), at, normal, row }.label();
    state.doc.edit(&label, |m, s| {
        s.clear();
        for (classname, item) in placed {
            let mut e = gt_doc::Entity::new(classname);
            let brush = match item {
                Placed::Point { origin, angles } => {
                    e.origin = origin;
                    e.angles = angles;
                    None
                }
                Placed::Brush(b) => Some(b),
            };
            let id = m.insert(parent, gt_doc::NodeKind::Entity(e));
            if let Some(b) = brush {
                m.insert(id, gt_doc::NodeKind::Brush(b));
            }

            s.select_node(id);
        }
    });
}

/// Where an entity or model dropped into a view lands, with the normal of the surface it rests on: the first solid
/// surface under the pointer, past trigger volumes and entities. A 2D view shows no depth, so with `through` (Alt held)
/// the drop passes the solid in front, a roof or the near wall, and lands in the space behind it: on its floor in the
/// Top view, halfway across it in the Front and Side views.
pub fn drop_target(state: &EditorState, ray: &gt_core::Ray, kind: crate::camera::ViewKind, through: bool) -> Option<(DVec3, Option<DVec3>)> {
    let hits = crate::picking::pick_all(state, ray);
    let first = hits.first()?;
    let map = &state.doc.map;
    let volume = |id| map.owning_entity(id).and_then(|e| map.entity(e)).is_some_and(|e| crate::scene::is_volume(&state.game, e));
    let solid = |h: &&crate::picking::Hit| match map.get(h.node).map(|n| &n.kind) {
        Some(gt_doc::NodeKind::Brush(b)) => !volume(h.node) && !b.faces.iter().all(|f| state.game.is_tool_texture(&f.data.material)),
        Some(gt_doc::NodeKind::Mesh(_)) => !volume(h.node),
        Some(gt_doc::NodeKind::Terrain(_)) => true,
        _ => false,
    };
    let Some(front) = hits.iter().find(solid) else { return Some((first.point, Some(first.normal))) };
    if through && kind.is_2d() {
        let b = map.bounds(front.node);
        let exit = ((b.min - ray.origin) / ray.dir).max((b.max - ray.origin) / ray.dir).min_element();
        if let Some(back) = hits.iter().filter(solid).find(|h| h.distance > exit + 1.0 && h.normal.dot(ray.dir) < 0.0) {
            return Some(if kind == crate::camera::ViewKind::Top { (back.point, Some(back.normal)) } else { (ray.at((exit + back.distance) / 2.0), None) });
        }
    }

    Some((front.point, Some(front.normal)))
}

/// Opens a Quake `.map` as a new, unsaved GodotTrench document, in a tab when the current map has unsaved changes.
pub fn import_quake_map(state: &mut EditorState, path: &std::path::Path) -> Result<gt_formats::quake_map::ImportReport, String> {
    let text = gt_formats::vmf::read_text(path).map_err(|e| e.to_string())?;
    let options = gt_formats::vmf::ImportOptions { tools: state.game.tool_textures.clone(), ..Default::default() };
    let (map, report) = gt_formats::quake_map::import_with(&text, &options).map_err(|e| e.to_string())?;
    let mut doc = Document::from_map(map, None);
    doc.revision += 1;
    if state.doc.is_modified() {
        state.open_tab(doc);
    } else {
        state.reset_document(doc);
    }

    let mut status = format!("Imported {}", path.display());
    if report.skipped_patches > 0 {
        status += &format!(", {} Quake 3 patches left out", report.skipped_patches);
    }

    state.set_status(status);
    Ok(report)
}

/// Opens a Hammer `.vmf` in a new tab, with its instances inlined and Valve tool materials renamed to the project's.
pub fn import_vmf(state: &mut EditorState, path: &std::path::Path) -> Result<gt_formats::vmf::ImportReport, String> {
    let options = gt_formats::vmf::ImportOptions { tools: state.game.tool_textures.clone(), ..Default::default() };
    let (map, report) = gt_formats::vmf::import_file(path, &options).map_err(|e| e.to_string())?;
    let mut doc = Document::from_map(map, None);
    doc.revision += 1;
    state.open_tab(doc);
    let mut status = format!("Imported {}", path.display());
    if report.instances > 0 {
        status += &format!(", {} instances inlined", report.instances);
    }

    if let Some(first) = report.missing_instances.first() {
        status += &format!(", {} instances not found (first: {first})", report.missing_instances.len());
    }

    state.set_status(status);
    Ok(report)
}

fn sanitize(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c.to_ascii_lowercase() } else { '_' }).collect()
}

/// The file a prop entity can reference: the model itself when it is inside the Godot project, else a copy under
/// res://models made after asking. `None` with a status when there is no project or the user declines.
fn prop_model_in_project(state: &mut EditorState, path: &std::path::Path) -> Option<std::path::PathBuf> {
    let Some(root) = state.game.project_root.clone() else {
        state.set_status("Open a Godot project first, a model prop has to reference a file Godot can load");
        return None;
    };
    if gt_formats::game::to_res_path(&root, path).is_some() {
        return Some(path.to_path_buf());
    }

    let copy = rfd::MessageDialog::new()
        .set_title("Copy model into the project?")
        .set_description(format!("{} is outside the Godot project, so Godot cannot load it. Copy it into res://models?", path.display()))
        .set_buttons(rfd::MessageButtons::YesNo)
        .show();
    if copy != rfd::MessageDialogResult::Yes {
        state.set_status("Model not imported, a model prop has to be inside the Godot project");
        return None;
    }

    match copy_into_project(&root, path) {
        Ok(target) => {
            let game = state.game.clone();
            state.model_library.rescan(&game);
            Some(target)
        }
        Err(e) => {
            state.set_status(format!("Could not copy the model into the project: {e}"));
            None
        }
    }
}

/// Copies a file into `<root>/models`, never overwriting an existing one.
fn copy_into_project(root: &std::path::Path, path: &std::path::Path) -> Result<std::path::PathBuf, String> {
    let dir = root.join("models");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "model".into());
    let ext = path.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let mut target = dir.join(format!("{stem}{ext}"));
    let mut n = 1;
    while target.exists() {
        target = dir.join(format!("{stem}_{n}{ext}"));
        n += 1;
    }

    std::fs::copy(path, &target).map_err(|e| e.to_string())?;
    Ok(target)
}

/// Imports a model file. Blockbench textures are written into the project texture folder so faces can use them. Other
/// formats become an editable mesh like a model dragged in from the Models panel, or a prop entity for `Prop`, which
/// needs the file inside the Godot project.
pub fn import_model(state: &mut EditorState, path: &std::path::Path, mode: ModelImport, at: DVec3) -> Result<usize, String> {
    let parent = state.insert_parent();
    let is_bb = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("bbmodel"));
    if mode == ModelImport::Brushes && !is_bb {
        return Err("only Blockbench .bbmodel files import as brushes, import it as a mesh instead".into());
    }

    if mode == ModelImport::Mesh && !is_bb {
        place_model_mesh(state, path, at)?;
        return Ok(1);
    }

    if mode == ModelImport::Prop {
        let reference = state
            .game
            .project_root
            .as_deref()
            .and_then(|root| gt_formats::game::to_res_path(root, path))
            .ok_or_else(|| format!("{} is outside the Godot project, Godot can only load a prop's model from inside it", path.display()))?;
        let class = state.prefs.scatter.prop_class.clone();
        state.doc.edit("Place Model", |m, s| {
            let mut e = gt_doc::Entity::new(class);
            e.properties.insert("model".into(), reference.clone());
            e.origin = at;
            let id = m.insert(parent, gt_doc::NodeKind::Entity(e));
            s.clear();
            s.select_node(id);
        });
        return Ok(1);
    }

    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let model = gt_formats::bbmodel::parse(&text).map_err(|e| e.to_string())?;
    let stem = sanitize(&path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "model".into()));
    let mut materials = Vec::new();
    let mut sizes = Vec::new();
    let texture_root = state.game.texture_root().filter(|p| p.is_dir());
    for (i, tex) in model.textures.iter().enumerate() {
        let png =
            if tex.png.is_empty() { path.parent().map(|d| d.join(&tex.path)).and_then(|p| std::fs::read(p).ok()).unwrap_or_default() } else { tex.png.clone() };
        let size = image::load_from_memory(&png).map(|img| DVec2::new(img.width() as f64, img.height() as f64)).unwrap_or(DVec2::splat(16.0));
        sizes.push(size);
        let file_name = format!("{}_{i}.png", sanitize(&tex.name));
        let name = match &texture_root {
            Some(root) => {
                let dir = root.join("models").join(&stem);
                std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                std::fs::write(dir.join(&file_name), &png).map_err(|e| e.to_string())?;
                format!("models/{stem}/{}", file_name.trim_end_matches(".png"))
            }
            None => {
                let file = path.with_file_name(format!("{stem}_{file_name}"));
                std::fs::write(&file, &png).map_err(|e| e.to_string())?;
                file.to_string_lossy().replace('\\', "/")
            }
        };
        materials.push(name);
    }

    if texture_root.is_some() {
        let game = state.game.clone();
        state.materials.rescan(&game);
    }

    let scale = state.game.units_per_meter / crate::models::BB_UNITS_PER_METER;
    let material = |t: Option<usize>| t.and_then(|i| materials.get(i).cloned()).unwrap_or_else(|| "dev/grey".to_string());
    let count = match mode {
        ModelImport::Mesh => {
            let mut mesh = model.to_mesh(scale, at, material);
            mesh.weld(1e-4);
            state.doc.edit("Import Model", |m, s| {
                let id = m.insert(parent, gt_doc::NodeKind::Mesh(mesh));
                s.clear();
                s.select_node(id);
            });
            1
        }
        ModelImport::Brushes => {
            let brushes = model.to_brushes(scale, at, material, |t| t.and_then(|i| sizes.get(i).copied()).unwrap_or(DVec2::splat(16.0)));
            let n = brushes.len();
            state.doc.edit("Import Model", |m, s| {
                s.clear();
                let group = m.insert(parent, gt_doc::NodeKind::Group(gt_doc::Group::new(stem.clone())));
                for b in brushes {
                    m.insert(group, gt_doc::NodeKind::Brush(b));
                }

                s.select_node(group);
            });
            n
        }
        ModelImport::Prop => unreachable!(),
    };
    Ok(count)
}

/// A mesh past either of these is heavy enough that vertex editing gets sluggish, so entering edit
/// mode or placing it warns the user, without ever blocking the action.
pub const HEAVY_VERTS: usize = 20_000;
pub const HEAVY_TRIS: usize = 40_000;

/// Total vertices and triangles across the meshes that mesh edit mode would act on.
fn edit_mesh_weight(state: &EditorState) -> (usize, usize) {
    let mut verts = 0;
    let mut tris = 0;
    for id in state.doc.selection.meshes(&state.doc.map) {
        if let Some(mesh) = state.doc.map.mesh(id) {
            verts += mesh.vertices.len();
            tris += mesh.faces.iter().map(|f| f.indices.len().saturating_sub(2)).sum::<usize>();
        }
    }

    (verts, tris)
}

/// Places a model from the Models panel into the scene as one editable mesh at `at`. The model's
/// textures are written into `res://textures/models/<stem>/` and registered as materials so the mesh
/// keeps its look. Large results are warned about but never blocked.
pub fn place_model_mesh(state: &mut EditorState, path: &std::path::Path, at: DVec3) -> Result<String, String> {
    let upm = state.game.units_per_meter;
    let model = state.models.get(path, upm).ok_or_else(|| "could not load model".to_string())?;
    let stem = sanitize(&path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "model".into()));
    let texture_root = state.game.texture_root().filter(|p| p.is_dir());
    let mut key_to_material: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    if let Some(root) = &texture_root {
        let dir = root.join("models").join(&stem);
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        for (i, (key, img, _pixelated)) in model.textures.iter().enumerate() {
            let file = dir.join(format!("tex{i}.png"));
            img.save(&file).map_err(|e| e.to_string())?;
            key_to_material.insert(key.clone(), format!("models/{stem}/tex{i}"));
        }

        let game = state.game.clone();
        state.materials.rescan(&game);
    }

    let material_of = |key: &str| key_to_material.get(key).cloned().unwrap_or_else(|| "dev/grey".to_string());
    let mi = state.prefs.model_import;
    let scale = crate::models::placement_scale(&model.bounds, upm, mi.autofit, mi.scale as f64);
    let mut mesh = crate::models::model_to_mesh(&model, at, scale, material_of);
    mesh.weld(1e-4);
    let verts = mesh.vertices.len();
    let tris: usize = mesh.faces.iter().map(|f| f.indices.len().saturating_sub(2)).sum();
    let parent = state.insert_parent();
    state.doc.edit("Place Model", |m, s| {
        let id = m.insert(parent, gt_doc::NodeKind::Mesh(mesh));
        s.clear();
        s.select_node(id);
    });
    let name = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "model".into());
    let scaled = if (scale - 1.0).abs() > 1e-3 { format!(", scaled {scale:.3}x to fit") } else { String::new() };
    // Warn but do not hinder: heavy meshes stay placeable, the user just gets a heads up.
    if verts > HEAVY_VERTS || tris > HEAVY_TRIS {
        Ok(format!("Placed {name} as an editable mesh, {verts} vertices, {tris} triangles{scaled}. That is a lot, editing may be slow."))
    } else {
        Ok(format!("Placed {name} as an editable mesh ({verts} vertices, {tris} triangles){scaled}"))
    }
}

/// Lays a decal sheet on a surface: a quad facing along `normal`, textured with `material` and blended
/// over the surface so the texture's alpha shows through. `size` is width and height in map units, one
/// metre square by default, and it can be moved, rotated and scaled with the ordinary tools.
fn create_decal(state: &mut EditorState, material: &str, at: DVec3, normal: DVec3, size: Option<[f64; 2]>) {
    let n = normal.normalize_or(DVec3::Y);
    // Any two axes in the surface plane; avoid a degenerate cross when the normal is near vertical.
    let up = if n.y.abs() > 0.9 { DVec3::Z } else { DVec3::Y };
    let metre = state.game.units_per_meter.max(1.0);
    let [w, h] = size.filter(|s| s[0] > 0.0 && s[1] > 0.0).unwrap_or([metre, metre]);
    let u = up.cross(n).normalize_or(DVec3::X) * w * 0.5;
    let v = n.cross(u).normalize_or(DVec3::Y) * h * 0.5;
    let center = at + n * 0.1; // lift just off the surface so it draws cleanly over it
    let corners = [center - u - v, center + u - v, center + u + v, center - u + v];
    let mut mesh = gt_geom::Mesh { vertices: corners.to_vec(), decal: true, ..Default::default() };
    let mut face = gt_geom::MeshFace::new(vec![0, 1, 2, 3], gt_geom::FaceData::new(material, gt_geom::FaceUv::default()));
    face.uvs = vec![[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]];
    mesh.faces.push(face);
    let parent = state.insert_parent();
    state.doc.edit("Create Decal", |m, s| {
        let id = m.insert(parent, gt_doc::NodeKind::Mesh(mesh));
        s.clear();
        s.select_node(id);
    });
    state.set_status(format!("Placed a decal with {material}"));
}

/// `<texture>.hotspots.json` next to a material's albedo image.
pub fn hotspot_path(state: &EditorState, material: &str) -> Option<std::path::PathBuf> {
    state.materials.find(material).and_then(|e| e.path.clone()).map(|tex| tex.with_extension("hotspots.json"))
}

pub fn write_hotspots(state: &EditorState, material: &str, rects: &[[f64; 4]]) -> Result<std::path::PathBuf, String> {
    let path = hotspot_path(state, material).ok_or_else(|| format!("{material} has no texture file to store hotspots next to"))?;
    let text = serde_json::to_string_pretty(&serde_json::json!({ "rects": rects })).map_err(|e| e.to_string())?;
    std::fs::write(&path, text).map_err(|e| e.to_string())?;
    Ok(path)
}

/// Hotspot rectangles of a material from `<texture>.hotspots.json` (`{"rects": [[x, y, w, h], ...]}` in pixels).
pub fn hotspot_rects(state: &EditorState, material: &str) -> Vec<[f64; 4]> {
    let Some(file) = hotspot_path(state, material) else { return Vec::new() };
    let Ok(text) = std::fs::read_to_string(file) else { return Vec::new() };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else { return Vec::new() };
    value["rects"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| {
            let a = r.as_array()?;
            (a.len() >= 4).then(|| [a[0].as_f64().unwrap_or(0.0), a[1].as_f64().unwrap_or(0.0), a[2].as_f64().unwrap_or(1.0), a[3].as_f64().unwrap_or(1.0)])
        })
        .collect()
}

/// Hotspot rectangles in the units face UVs are measured in, which differ from pixels when the material sets
/// `metadata/texture_size`.
pub fn hotspot_rects_uv(state: &EditorState, material: &str) -> Vec<[f64; 4]> {
    let rects = hotspot_rects(state, material);
    let (Some(world), Some(px)) = (state.materials.world_size(material), state.materials.pixel_size(material)) else { return rects };
    let (sx, sy) = (world[0] / px[0].max(1) as f64, world[1] / px[1].max(1) as f64);
    rects.into_iter().map(|[x, y, w, h]| [x * sx, y * sy, w * sx, h * sy]).collect()
}

/// Fits selected brush faces (or every face of selected brushes) to the best matching hotspot rectangle.
fn hotspot_texture(state: &mut EditorState) -> usize {
    let targets: Vec<(NodeId, usize)> = if state.doc.selection.has_faces() {
        state.doc.selection.faces.iter().copied().collect()
    } else {
        state
            .doc
            .selection
            .geometry(&state.doc.map)
            .into_iter()
            .flat_map(|id| {
                let map = &state.doc.map;
                let n = map.brush(id).map(|b| b.faces.len()).or_else(|| map.mesh(id).map(|m| m.faces.len())).unwrap_or(0);
                (0..n).map(move |f| (id, f))
            })
            .collect()
    };
    let mut plans = Vec::new();
    for (id, f) in targets {
        let Some(info) = crate::texture_ops::face_info(&state.doc.map, id, f) else { continue };
        let rects = hotspot_rects_uv(state, &info.material);
        if rects.is_empty() {
            continue;
        }

        let pts = info.points.clone();
        let normal = info.plane.normal;
        let best = rects
            .iter()
            .map(|r| {
                let mut uv = info.uv.clone();
                let aspect = uv.fit_to_rect(&pts, normal, *r, true);
                // Prefer rectangles whose texel density is closest to one texel per unit.
                let density = (uv.scale.x.abs().ln()).abs() + (uv.scale.y.abs().ln()).abs();
                (aspect * 4.0 + density * 0.25, uv)
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((_, uv)) = best {
            plans.push((id, f, uv));
        }
    }

    let n = plans.len();
    if n > 0 {
        state.doc.edit("Hotspot Texture", |m, _| {
            for (id, f, uv) in plans {
                if let Some(b) = m.brush_mut(id) {
                    b.faces[f].data.uv = uv;
                } else if let Some(face) = m.mesh_mut(id).and_then(|mesh| mesh.faces.get_mut(f)) {
                    face.data.uv = uv;
                    face.uvs.clear();
                }
            }
        });
    }

    n
}

/// Selected faces, or the upward facing quad of each selected brush.
fn displacement_candidates(state: &EditorState) -> Vec<(NodeId, usize)> {
    if state.doc.selection.has_faces() {
        return state.doc.selection.faces.iter().copied().filter(|(id, _)| state.doc.map.brush(*id).is_some()).collect();
    }

    state
        .doc
        .selection
        .brushes(&state.doc.map)
        .into_iter()
        .filter_map(|id| {
            let b = state.doc.map.brush(id)?;
            let top = b.faces.iter().enumerate().filter(|(_, f)| f.indices.len() == 4).max_by(|a, b| a.1.plane.normal.y.total_cmp(&b.1.plane.normal.y))?;
            Some((id, top.0))
        })
        .collect()
}

fn launch_godot(state: &mut EditorState, editor: bool) {
    let Some(root) = state.game.project_root.clone() else {
        state.set_status("Open a Godot project first");
        return;
    };
    state.godot.refresh(&state.prefs.godot_path, Some(&root));
    let Some(exe) = state.godot.exe.clone() else {
        state.set_status(GODOT_NOT_FOUND);
        return;
    };
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("--path").arg(&root).stdin(std::process::Stdio::null());
    if editor {
        cmd.arg("--editor");
    }

    if state.stdio_mcp {
        cmd.stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null());
    }

    match cmd.spawn() {
        Ok(_) => state.set_status(format!("Launched {}", exe.display())),
        Err(e) => state.set_status(format!("Could not launch Godot: {e}")),
    }
}

pub const GODOT_NOT_FOUND: &str = "Godot was not found, set the Godot executable in Preferences or the GODOT environment variable";
pub const LIVE_LINK_OFF: &str = "Live mode needs the Godot live link, turn it on in Preferences first";

fn toggle_walkable(state: &mut EditorState) {
    if state.walkable.on {
        state.walkable.set_on(false);
        state.set_status("Walkable area hidden");
        return;
    }

    let offline = || format!("Godot is not running with the GodotTrench addon on port {}, open the project in the Godot editor", state.prefs.live_link_port);
    match crate::walkable::unavailable(state).or_else(|| (!state.link_state.connected).then(offline)) {
        Some(why) => state.set_status(format!("Cannot show the walkable area: {why}")),
        None => {
            state.walkable.set_on(true);
            state.set_status("Baking the walkable area in Godot");
        }
    }
}

fn build_in_godot(state: &mut EditorState) {
    let Some(path) = state.doc.path.clone() else {
        state.set_status("Save the map into the Godot project first, Godot builds maps from their file path");
        return;
    };
    match &state.link {
        Some(link) if state.godot_has_project() => {
            link.request(crate::live_link::Request::Build { path: crate::live_link::godot_path(&path), map: state.doc.map.clone(), live: state.live_active() });
            state.set_status("Building in Godot…");
        }
        _ => state.set_status("Godot is not open with this project, open it with the Godot button in the toolbar"),
    }
}

fn selected_instances(state: &EditorState) -> Vec<(NodeId, gt_doc::map::Instance)> {
    state
        .doc
        .selection
        .nodes
        .iter()
        .filter_map(|id| match state.doc.map.get(*id).map(|n| &n.kind) {
            Some(gt_doc::NodeKind::Instance(i)) => Some((*id, i.clone())),
            _ => None,
        })
        .collect()
}

fn explode_instances(state: &mut EditorState) {
    let instances = selected_instances(state);
    if instances.is_empty() {
        state.set_status("Select prefab instances to explode");
        return;
    }

    let mut contents = Vec::new();
    for (id, inst) in &instances {
        let Some(path) = crate::prefabs::resolve(&inst.path, state.doc.path.as_deref(), state.game.project_root.as_deref()) else { continue };
        let Some(prefab) = state.prefabs.get(&path).map.clone() else {
            state.set_status(format!("Cannot load prefab {}", path.display()));
            return;
        };
        let exported = prefab.layers.iter().filter_map(|l| prefab.get(*l)).filter(|n| !matches!(&n.kind, gt_doc::NodeKind::Layer(l) if l.omit_from_export));
        let roots: Vec<NodeId> = exported.flat_map(|n| n.children.clone()).collect();
        contents.push((*id, inst.clone(), format::nodes_to_string(&prefab, &roots)));
    }

    let game = &state.game;
    state.doc.edit("Explode Instance", |m, s| {
        s.clear();
        for (id, inst, text) in contents {
            let parent = m.get(id).and_then(|n| n.parent).unwrap_or(m.default_layer());
            let Ok(ids) = format::paste_nodes(m, parent, &text) else { continue };
            let mut sel = gt_doc::Selection::default();
            sel.nodes.extend(ids.iter().copied());
            ops::transform_selection(m, &sel, &crate::prefabs::instance_transform(&inst), ops::EditOptions { uv_lock: true, grid: 0.0 });
            if !inst.fixup.is_empty() {
                for new_id in sel.transformables(m) {
                    if let Some(e) = m.entity_mut(new_id) {
                        let declared = game.entity(&e.classname).into_iter().flat_map(|d| &d.properties);
                        let declared = declared
                            .filter(|p| matches!(p.ty, gt_formats::game::PropertyType::TargetSource | gt_formats::game::PropertyType::TargetDestination))
                            .map(|p| p.name.as_str());
                        let keys: std::collections::BTreeSet<&str> = gt_doc::map::FIXUP_KEYS.into_iter().chain(declared).collect();
                        for key in keys {
                            if let Some(v) = e.properties.get_mut(key)
                                && let Some(fixed) = inst.fixup_name(v)
                            {
                                *v = fixed;
                            }
                        }

                        for o in &mut e.outputs {
                            if let Some(fixed) = inst.fixup_name(&o.target) {
                                o.target = fixed;
                            }
                        }
                    }

                    if let Some(gt_doc::NodeKind::Instance(nested)) = m.get_mut(new_id).map(|n| &mut n.kind) {
                        nested.fixup = inst.nested_fixup(&nested.fixup);
                    }
                }
            }

            m.remove(id);
            s.nodes.extend(ids);
        }
    });
}

fn first_selected_material(state: &EditorState) -> Option<String> {
    let map = &state.doc.map;
    if let Some((id, f)) = state.doc.selection.faces.iter().next() {
        return map
            .brush(*id)
            .map(|b| b.faces[*f].data.material.clone())
            .or_else(|| map.mesh(*id).and_then(|m| m.faces.get(*f)).map(|f| f.data.material.clone()));
    }

    state.doc.selection.brushes(map).first().and_then(|id| map.brush(*id)).map(|b| b.faces[0].data.material.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_dialogs_start_in_the_map_folder_else_the_project() {
        let mut state = EditorState::new(Default::default());
        assert_eq!(dialog_dir(&state), None);
        let project = std::env::temp_dir().join(format!("gt_dialog_dir_{}", std::process::id()));
        let maps = project.join("maps");
        std::fs::create_dir_all(&maps).unwrap();
        state.game.project_root = Some(project.clone());
        assert_eq!(dialog_dir(&state).as_deref(), Some(project.as_path()), "an unsaved map starts in the project");
        state.doc.path = Some(maps.join("level.gtm"));
        assert_eq!(dialog_dir(&state).as_deref(), Some(maps.as_path()), "a saved map starts in its own folder");
        std::fs::remove_dir_all(&project).ok();
    }

    #[test]
    fn open_in_godot_shows_the_editor_that_has_the_project() {
        let mut state = EditorState::new(Default::default());
        let root = std::path::PathBuf::from("/games/court");
        state.game.project_root = Some(root.clone());
        state.link_state = crate::live_link::LinkState {
            connected: true,
            project: Some(crate::live_link::path_key(&crate::live_link::godot_path(&root))),
            ..Default::default()
        };
        state.link = Some(crate::live_link::LiveLink::idle());
        execute(&mut state, Action::OpenGodotEditor, &egui::Context::default());
        let requests = state.link.as_ref().unwrap().take_requests();
        assert!(matches!(requests[..], [crate::live_link::Request::Focus]), "asks Godot to come forward, once");
        assert_eq!(state.status, "The Godot editor already has this project open, showing it", "no second editor is launched");
    }

    #[test]
    fn a_preset_starts_a_new_scatter_set_on_its_own_layer() {
        let mut state = EditorState::new(Default::default());
        let layers = state.doc.map.layers.len();
        execute(&mut state, Action::ScatterPreset("rocks".into()), &egui::Context::default());
        let id = state.active_scatter.expect("the new set is active");
        let set = state.doc.map.scatter(id).unwrap();
        assert_eq!(set.name, "rocks");
        assert_eq!(set.items, gt_doc::scatter::preset("rocks").unwrap().1);
        assert_eq!(state.doc.map.layers.len(), layers + 1);
        execute(&mut state, Action::ScatterPreset("rocks".into()), &egui::Context::default());
        assert_eq!(state.doc.map.scatter(state.active_scatter.unwrap()).unwrap().name, "rocks 2", "a second one gets its own name");
    }

    #[test]
    fn repeat_last_runs_the_last_transform_again() {
        let mut state = EditorState::new(Default::default());
        let ctx = egui::Context::default();
        execute(&mut state, Action::RepeatLast, &ctx);
        execute(&mut state, Action::CreateBrushFromBounds, &ctx);
        execute(&mut state, Action::Nudge(DVec3::new(16.0, 0.0, 0.0)), &ctx);
        execute(&mut state, Action::SelectAll, &ctx);
        execute(&mut state, Action::RepeatLast, &ctx);
        let b = state.doc.map.bounds_of(state.doc.selection.nodes.iter().copied());
        assert_eq!(b.min.x, 32.0, "nudged twice");
    }

    #[test]
    fn new_map_keeps_unsaved_work_in_its_tab() {
        let mut state = EditorState::new(Default::default());
        let ctx = egui::Context::default();
        execute(&mut state, Action::CreateBrushFromBounds, &ctx);
        assert!(state.doc.is_modified());
        execute(&mut state, Action::NewMap, &ctx);
        assert_eq!(state.tabs.len(), 1, "the modified map stays open in a tab");
        assert!(state.tabs[0].doc.is_modified() && !state.doc.is_modified());
        execute(&mut state, Action::NewMap, &ctx);
        assert_eq!(state.tabs.len(), 1, "an untouched map is simply replaced");
    }

    #[test]
    fn presets_and_overrides() {
        let mut prefs = Prefs::default();
        let tb = shortcuts(&prefs);
        assert!(tb.iter().any(|(s, a)| *a == Action::ToggleEditMode && s.logical_key == Key::Tab));
        prefs.keymap_preset = "hammer".into();
        let hammer = shortcuts(&prefs);
        assert!(hammer.iter().any(|(s, a)| *a == Action::SetTool(ToolKind::Clip) && s.modifiers.shift && s.logical_key == Key::X));
        assert!(hammer.iter().any(|(s, a)| *a == Action::SetTool(ToolKind::Clip) && s.logical_key == Key::C), "moved actions keep their old key");
        assert!(hammer.iter().any(|(s, a)| *a == Action::MoveToWorld && *s == sc(CTRL, Key::W)));
        assert!(!hammer.iter().any(|(s, a)| *a == Action::CloseTab && *s == sc(CTRL, Key::W)), "a taken chord leaves its old action");
        prefs.key_overrides.insert(Action::CsgSubtract.binding_id(), "Ctrl+Shift+Q".into());
        let custom = shortcuts(&prefs);
        let (s, _) = custom.iter().find(|(_, a)| *a == Action::CsgSubtract).unwrap();
        assert_eq!(shortcut_to_text(s), "Ctrl+Shift+Q");
        assert_eq!(parse_shortcut("Alt+H").map(|s| s.logical_key), Some(Key::H));
        let ids: Vec<String> = bindable_actions().iter().map(|a| a.binding_id()).collect();
        let unique: std::collections::BTreeSet<&String> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len());
    }

    #[test]
    fn f2_renames_in_every_preset_and_shading_keeps_a_key() {
        for preset in PRESETS {
            let bindings = preset_bindings(preset);
            let key = |action: &Action| bindings.iter().filter(|(_, a)| a == action).map(|(s, _)| shortcut_to_text(s)).collect::<Vec<_>>();
            assert_eq!(key(&Action::Rename), ["F2"], "{preset}");
            assert!(!key(&Action::ToggleTextured).is_empty(), "{preset}");
        }

        assert!(preset_bindings("trenchbroom").contains(&(sc(NONE, Key::F4), Action::ToggleTextured)));
    }

    #[test]
    fn rename_needs_one_selected_object() {
        let mut state = EditorState::new(Prefs::default());
        let ctx = egui::Context::default();
        let layer = state.doc.map.default_layer();
        let (a, _) = state.doc.edit("add", |m, s| {
            let a = m.insert(layer, gt_doc::NodeKind::Entity(gt_doc::Entity::new("light")));
            let b = m.insert(layer, gt_doc::NodeKind::Entity(gt_doc::Entity::new("light")));
            s.select_node(a);
            s.select_node(b);
            (a, b)
        });
        execute(&mut state, Action::Rename, &ctx);
        assert_eq!(state.renaming, None);
        state.doc.select(|_, s| {
            s.clear();
            s.select_node(a);
        });
        execute(&mut state, Action::Rename, &ctx);
        assert_eq!((state.renaming, state.outliner_reveal), (Some(a), Some(a)), "the Outliner shows the row with its name field");
    }

    #[test]
    fn every_preset_keeps_a_key_for_every_default_action() {
        for preset in PRESETS {
            let bindings = preset_bindings(preset);
            for (_, action) in trenchbroom_bindings() {
                assert!(bindings.iter().any(|(_, a)| *a == action), "{preset} leaves {action:?} unbound");
            }

            let chords: Vec<&KeyboardShortcut> = bindings.iter().map(|(s, _)| s).collect();
            assert!(chords.iter().enumerate().all(|(i, c)| !chords[..i].contains(c)), "{preset} binds a chord twice");
            let has = |action: Action, s: KeyboardShortcut| bindings.iter().any(|(b, a)| *a == action && *b == s);
            assert!(has(Action::Delete, sc(NONE, Key::Delete)), "{preset}");
            assert!(has(Action::SelectNone, sc(NONE, Key::Escape)), "{preset}");
            assert!(has(Action::ShowCommandPalette, sc(CTRL_SHIFT, Key::P)), "{preset}");
            for action in [Action::CloseTab, Action::CsgMerge, Action::CsgIntersect, Action::ShowShapeDialog, Action::ShowCommandPalette] {
                assert!(bindings.iter().any(|(_, a)| *a == action), "{preset} leaves {action:?} unbound");
            }
        }
    }

    #[test]
    fn digit_shortcuts_match_the_physical_key_under_shift() {
        let mods = Modifiers { ctrl: true, command: true, shift: true, ..Modifiers::NONE };
        let press = |key, physical| egui::Event::Key { key, physical_key: Some(physical), pressed: true, repeat: false, modifiers: mods };
        let store = sc(CTRL_SHIFT, Key::Num1);
        let mut events = vec![press(Key::Exclamationmark, Key::Num1)];
        assert!(consume_physical_digit(&mut events, &store));
        assert!(events.is_empty());
        let mut events = vec![press(Key::Exclamationmark, Key::Num2)];
        assert!(!consume_physical_digit(&mut events, &store));
        assert!(!consume_physical_digit(&mut events, &sc(CTRL_SHIFT, Key::S)), "only digit bindings look at the physical key");
        assert_eq!(events.len(), 1);
    }

    fn brush_at(state: &mut EditorState, x: f64) -> NodeId {
        let parent = state.insert_parent();
        let min = DVec3::new(x, 0.0, 0.0);
        let brush = gt_geom::Brush::from_aabb(&Aabb::new(min, min + DVec3::splat(32.0)), "dev/grey").unwrap();
        state.doc.edit("Create Brush", |m, _| ops::create_brush(m, parent, brush))
    }

    #[test]
    fn undoing_a_new_layer_keeps_objects_on_a_real_layer() {
        let mut state = EditorState::new(Default::default());
        let ctx = egui::Context::default();
        execute(&mut state, Action::AddLayer, &ctx);
        let layer = state.current_layer;
        execute(&mut state, Action::Undo, &ctx);
        assert_eq!(state.current_layer, state.doc.map.default_layer());

        // The removed layer's id comes back as the next node, it must not become a parent.
        let first = brush_at(&mut state, 0.0);
        assert_eq!(first, layer);
        let second = brush_at(&mut state, 64.0);
        let map = &state.doc.map;
        assert_eq!(map.get(second).unwrap().parent, Some(map.default_layer()));
        execute(&mut state, Action::Redo, &ctx);
        assert!(state.doc.map.layers.contains(&state.current_layer));

        state.open_groups.push(first);
        execute(&mut state, Action::SelectNone, &ctx);
        assert!(state.open_groups.is_empty(), "a brush is not an open group");
    }

    #[test]
    fn cutting_the_current_layer_falls_back_to_the_default_layer() {
        let mut state = EditorState::new(Default::default());
        let ctx = egui::Context::default();
        execute(&mut state, Action::AddLayer, &ctx);
        let layer = state.current_layer;
        brush_at(&mut state, 0.0);
        state.doc.select(|_, s| s.select_node(layer));
        execute(&mut state, Action::Cut, &ctx);
        assert!(!state.doc.map.contains(layer));
        assert_eq!(state.current_layer, state.doc.map.default_layer(), "cutting the current layer falls back like delete");
    }

    #[test]
    fn failed_and_empty_commands_record_nothing() {
        let mut state = EditorState::new(Default::default());
        let ctx = egui::Context::default();
        brush_at(&mut state, 0.0);
        execute(&mut state, Action::Undo, &ctx);
        state.doc.mark_saved();
        let steps = state.doc.history.undo_labels().count();
        for action in [
            Action::Group,
            Action::CsgMerge,
            Action::CsgIntersect,
            Action::JoinMeshes,
            Action::CreateBrushEntity("func_door".into()),
            Action::SelectTouching,
            Action::SelectInside,
            Action::DuplicateLinked,
            Action::Paste("not a map".into()),
        ] {
            execute(&mut state, action.clone(), &ctx);
            assert!(!state.doc.is_modified(), "{action:?} marked the map modified");
            assert_eq!(state.doc.history.undo_labels().count(), steps, "{action:?} added an undo step");
            assert!(state.doc.history.can_redo(), "{action:?} dropped the redo step");
        }
    }

    #[test]
    fn duplicate_linked_is_one_undo_step_and_paste_lands_at_the_cursor() {
        let mut state = EditorState::new(Default::default());
        let ctx = egui::Context::default();
        let brush = brush_at(&mut state, 0.0);
        state.doc.select(|_, s| s.select_node(brush));
        let steps = state.doc.history.undo_labels().count();
        execute(&mut state, Action::DuplicateLinked, &ctx);
        assert_eq!(state.doc.history.undo_labels().count(), steps + 1);
        assert_eq!(state.doc.map.nodes.values().filter(|n| matches!(n.kind, gt_doc::NodeKind::Group(_))).count(), 2);

        let text = format::nodes_to_string(&state.doc.map, &[brush]);
        state.cursor_world = Some(DVec3::new(512.0, 0.0, 512.0));
        execute(&mut state, Action::Paste(text), &ctx);
        assert_eq!(state.doc.history.undo_labels().next(), Some("Paste"));
        let b = state.doc.map.bounds_of(state.doc.selection.nodes.iter().copied());
        assert_eq!(b.center().x, 512.0);
        execute(&mut state, Action::Undo, &ctx);
        assert_eq!(state.doc.map.brushes().count(), 2, "one undo takes the paste and its move back");
    }

    #[test]
    fn camera_bookmarks_survive_undo() {
        let mut state = EditorState::new(Default::default());
        let ctx = egui::Context::default();
        brush_at(&mut state, 0.0);
        let bookmark = gt_doc::map::CameraBookmark { position: DVec3::splat(5.0), yaw: 1.0, pitch: 0.5 };
        state.doc.map.editor.cameras.insert(3, bookmark);
        execute(&mut state, Action::Undo, &ctx);
        assert!(state.doc.map.editor.cameras.contains_key(&3));
        execute(&mut state, Action::Redo, &ctx);
        assert!(state.doc.map.editor.cameras.contains_key(&3));
    }

    #[test]
    fn live_mode_needs_the_live_link() {
        let mut state = EditorState::new(Default::default());
        state.prefs.live_link = false;
        execute(&mut state, Action::ToggleLiveMode, &egui::Context::default());
        assert!(!state.prefs.live_mode);
        assert_eq!(state.status, LIVE_LINK_OFF);
    }

    #[test]
    fn model_props_never_reference_files_outside_the_project() {
        let mut state = EditorState::new(Default::default());
        let outside = std::env::temp_dir().join("gt_outside_model.glb");
        assert!(import_model(&mut state, &outside, ModelImport::Prop, DVec3::ZERO).is_err());
        assert!(import_model(&mut state, &outside, ModelImport::Brushes, DVec3::ZERO).is_err());
        assert_eq!(state.doc.map.entity_count(), 0);
    }

    #[test]
    fn installing_nature_models_twice_says_they_are_there() {
        let dir = std::env::temp_dir().join(format!("gt_nature_{}", std::process::id()));
        let mut state = EditorState::new(Default::default());
        state.game.project_root = Some(dir.clone());
        let ctx = egui::Context::default();
        execute(&mut state, Action::InstallNatureModels, &ctx);
        assert!(state.status.starts_with("Installed "), "{}", state.status);
        execute(&mut state, Action::InstallNatureModels, &ctx);
        assert!(state.status.contains("already installed"), "{}", state.status);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn save_all_saves_every_tab_and_discard_closes_one() {
        let dir = std::env::temp_dir().join(format!("gt_save_all_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mut state = EditorState::new(Default::default());
        state.prefs.live_link = false;
        let ctx = egui::Context::default();
        for name in ["a.gtm", "b.gtm"] {
            state.open_tab(Document::from_map(gt_doc::Map::new(), Some(dir.join(name))));
            brush_at(&mut state, 0.0);
        }

        state.switch_tab(0);
        assert_eq!(state.modified_tabs(), vec![1, 2]);
        assert!(save_all(&mut state, &ctx));
        assert!(state.modified_tabs().is_empty());
        assert_eq!(state.active_tab, 0);
        assert!(dir.join("a.gtm").exists() && dir.join("b.gtm").exists());

        state.switch_tab(2);
        brush_at(&mut state, 64.0);
        state.discard_tab();
        assert_eq!(state.tabs.len(), 1);
        assert!(state.modified_tabs().is_empty());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_drop_lands_on_the_first_surface_and_alt_passes_the_roof_in_2d() {
        use crate::camera::ViewKind;
        use gt_core::Ray;
        let mut state = EditorState::new(Default::default());
        let layer = state.doc.map.default_layer();
        let room = Aabb::new(DVec3::ZERO, DVec3::new(256.0, 128.0, 256.0));
        let block = |m: &mut gt_doc::Map, min: DVec3, max: DVec3| {
            m.insert(layer, gt_doc::NodeKind::Brush(gt_geom::Brush::from_aabb(&Aabb::new(min, max), "crate").unwrap()));
        };
        state.doc.edit("room", |m, _| {
            for part in gt_geom::csg::hollow(&gt_geom::Brush::from_aabb(&room, "wall").unwrap(), 16.0) {
                m.insert(layer, gt_doc::NodeKind::Brush(part));
            }

            block(m, DVec3::new(512.0, 0.0, 0.0), DVec3::new(576.0, 64.0, 64.0));
            let trigger = m.insert(layer, gt_doc::NodeKind::Entity(gt_doc::Entity::new("trigger_once")));
            let volume = Aabb::new(DVec3::new(512.0, 64.0, 0.0), DVec3::new(576.0, 256.0, 64.0));
            m.insert(trigger, gt_doc::NodeKind::Brush(gt_geom::Brush::from_aabb(&volume, "tools/trigger").unwrap()));
            // An upper floor over the ground floor, with nothing around them.
            block(m, DVec3::new(1024.0, 0.0, 0.0), DVec3::new(1280.0, 16.0, 256.0));
            block(m, DVec3::new(1024.0, 128.0, 0.0), DVec3::new(1280.0, 144.0, 256.0));
        });

        let down = |x: f64, z: f64| Ray::new(DVec3::new(x, 4096.0, z), -DVec3::Y);
        let (at, normal) = drop_target(&state, &down(128.0, 128.0), ViewKind::Top, false).unwrap();
        assert_eq!((at, normal), (DVec3::new(128.0, 128.0, 128.0), Some(DVec3::Y)), "on the roof, the first surface under the pointer");
        let (at, normal) = drop_target(&state, &down(128.0, 128.0), ViewKind::Top, true).unwrap();
        assert_eq!((at, normal), (DVec3::new(128.0, 16.0, 128.0), Some(DVec3::Y)), "Alt passes the roof and lands on the floor");

        let front = Ray::new(DVec3::new(128.0, 64.0, 4096.0), -DVec3::Z);
        assert_eq!(drop_target(&state, &front, ViewKind::Front, false).unwrap().0, DVec3::new(128.0, 64.0, 256.0), "on the near wall");
        let (at, normal) = drop_target(&state, &front, ViewKind::Front, true).unwrap();
        assert_eq!((at, normal), (DVec3::new(128.0, 64.0, 128.0), None), "halfway between the front and back walls");

        let (at, _) = drop_target(&state, &down(128.0, 128.0), ViewKind::Perspective, true).unwrap();
        assert_eq!(at.y, 128.0, "the 3D view drops on the surface it shows");

        for through in [false, true] {
            let (at, _) = drop_target(&state, &down(544.0, 32.0), ViewKind::Top, through).unwrap();
            assert_eq!(at.y, 64.0, "a solid block takes the drop on top, under the trigger volume around it");
        }

        let (at, _) = drop_target(&state, &down(1152.0, 128.0), ViewKind::Top, false).unwrap();
        assert_eq!(at.y, 144.0, "on the upper floor, not the one under it");
        let (at, _) = drop_target(&state, &down(1152.0, 128.0), ViewKind::Top, true).unwrap();
        assert_eq!(at.y, 16.0, "Alt reaches the floor below");
        assert!(drop_target(&state, &down(2000.0, 0.0), ViewKind::Top, false).is_none());
    }
}
