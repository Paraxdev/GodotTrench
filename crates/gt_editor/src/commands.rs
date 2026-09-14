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
    OpenProject,
    Save,
    SaveAs,
    Undo,
    Redo,
    RepeatLast,
    Delete,
    Duplicate,
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
    CreateDisplacement(u8),
    RemoveDisplacement,
    SewDisplacements,
    ImportQuakeMap,
    ExportQuakeMap,
    ExportQuakeMapCordon,
    ImportVmf,
    ImportModel(ModelImport),
    ReloadModels,
    ConvertToMesh,
    ConvertToBrushes,
    JoinMeshes,
    /// Enters mesh editing, converting selected brushes first.
    EditMesh,
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
    ReloadMaterials,
    /// Makes a scatter set the one the scatter tool paints into.
    ActivateScatter(NodeId),
    /// The next scatter stroke starts a new set on a new layer.
    NewScatterSet,
    ScatterFill,
    ScatterPreset(String),
    ShowScatterPalette,
    InstallNatureModels,
    /// Replaces the selected scatter sets with prop entities.
    ScatterToEntities,
    /// Uses the current material as the blend material of the selected faces.
    SetBlendMaterial,
    ClearBlendMaterial,
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
            Action::MeshOp(op) => format!("Mesh: {}", op.label()),
            Action::StoreCamera(n) => format!("Store Camera {n}"),
            Action::RecallCamera(n) => format!("Recall Camera {n}"),
            Action::SetShade(s) => format!("Shade {}", s.label()),
            Action::Justify(j) => format!("Justify Texture {}", j.label()),
            Action::TexelDensity(d) => format!("Texel Density {d}"),
            Action::MeshUv(k) => format!("Mesh UVs: {}", k.label()),
            Action::UiScaleUp => "Increase UI Scale".into(),
            Action::UiScaleDown => "Decrease UI Scale".into(),
            Action::UiScaleReset => "Reset UI Scale".into(),
            other => format!("{other:?}"),
        }
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
        (sc(NONE, Key::Tab), Action::EditMesh),
        (sc(NONE, Key::B), Action::SetTool(ToolKind::Scatter)),
        (sc(SHIFT, Key::G), Action::SetTool(ToolKind::Blend)),
        (sc(SHIFT, Key::E), Action::SetTool(ToolKind::Volume)),
        (sc(NONE, Key::M), Action::SetTool(ToolKind::Measure)),
        (sc(SHIFT, Key::P), Action::SetTool(ToolKind::Path)),
        (sc(SHIFT, Key::T), Action::SetTool(ToolKind::Texture)),
        (sc(CTRL, Key::U), Action::FocusSelection),
        (sc(NONE, Key::F), Action::FocusSelection),
        (sc(NONE, Key::F2), Action::ToggleTextured),
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
    ];
    for (i, key) in DIGITS.iter().enumerate() {
        out.push((sc(CTRL, *key), Action::RecallCamera(i as u8 + 1)));
        out.push((sc(CTRL_SHIFT, *key), Action::StoreCamera(i as u8 + 1)));
    }
    out
}

/// Key bindings for a preset. Hammer and Blender presets replace some TrenchBroom keys with their own.
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
        ],
        _ => Vec::new(),
    };
    for (shortcut, action) in replace {
        base.retain(|(s, a)| *s != shortcut && *a != action);
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
        Action::ImportModel(ModelImport::Mesh),
        Action::ImportModel(ModelImport::Brushes),
        Action::ImportModel(ModelImport::Prop),
        Action::ConvertToBrushes,
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
        Action::ShowScatterPalette,
        Action::SetBlendMaterial,
        Action::MakePlatform,
        Action::ShowLinkDialog,
        Action::ShowReference,
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

pub fn execute(state: &mut EditorState, action: Action, ctx: &egui::Context) {
    let opts = state.opts();
    let parent = state.insert_parent();
    let open_groups = state.open_groups.clone();
    let grid = state.grid;
    match action {
        Action::NewMap => {
            state.reset_document(Document::new());
            state.set_status("New map");
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
            let mut dialog = rfd::FileDialog::new().add_filter("GodotTrench map", &["gtm"]);
            if let Some(root) = &state.game.project_root {
                dialog = dialog.set_directory(root);
            }
            if let Some(path) = dialog.pick_file()
                && let Err(e) = open_map_in_tab(state, &path)
            {
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
            let mut dialog = rfd::FileDialog::new().add_filter("GodotTrench map", &["gtm"]).set_file_name("map.gtm");
            if let Some(root) = &state.game.project_root {
                dialog = dialog.set_directory(root);
            }
            if let Some(path) = dialog.save_file()
                && let Err(e) = state.save_map(&path)
            {
                state.set_status(format!("Save failed: {e}"));
            }
        }
        Action::Undo => {
            if let Some(l) = state.doc.undo() {
                state.set_status(format!("Undo {l}"));
            }
        }
        Action::Redo => {
            if let Some(l) = state.doc.redo() {
                state.set_status(format!("Redo {l}"));
            }
        }
        Action::RepeatLast => state.set_status("Repeat is available for transforms (nudge, rotate, flip)"),
        Action::Delete => {
            if state.doc.selection.nodes.is_empty() {
                return;
            }
            let n = state.doc.edit("Delete", ops::delete_selection);
            state.set_status(format!("Deleted {n} objects"));
        }
        Action::Duplicate => {
            let offset = DVec3::new(grid, 0.0, grid);
            state.doc.edit("Duplicate", |m, s| ops::duplicate_selection(m, s, offset, opts));
        }
        Action::DuplicateLinked => {
            let offset = DVec3::new(grid, 0.0, grid);
            let has_groups = state.doc.selection.nodes.iter().any(|id| matches!(state.doc.map.get(*id).map(|n| &n.kind), Some(gt_doc::NodeKind::Group(_))));
            if !has_groups {
                if state.doc.selection.nodes.is_empty() {
                    return;
                }
                state.doc.edit("Group", |m, s| ops::group_selection(m, s, "Linked", parent));
            }
            let n = state.doc.edit("Duplicate Linked", |m, s| ops::duplicate_linked(m, s, offset, opts)).len();
            state.set_status(format!("Created {n} linked group(s). Edits inside one update the others"));
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
        Action::SelectTouching => state.doc.edit("Select Touching", |m, s| ops::select_touching(m, s, &open_groups, false)),
        Action::SelectInside => state.doc.edit("Select Inside", |m, s| ops::select_touching(m, s, &open_groups, true)),
        Action::SelectSiblings => state.doc.select(ops::select_siblings),
        Action::SelectSameMaterial => {
            let mat = first_selected_material(state).unwrap_or_else(|| state.current_material.clone());
            state.doc.select(|m, s| ops::select_by_material(m, s, &mat, &open_groups));
        }
        Action::Group => {
            let name = "Group".to_string();
            if state.doc.edit("Group", |m, s| ops::group_selection(m, s, &name, parent)).is_none() {
                state.set_status("Nothing to group");
            }
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
            let n = state.doc.edit("CSG Subtract", ops::csg_subtract);
            state.set_status(format!("Subtracted from {n} brushes"));
        }
        Action::CsgMerge => {
            let mat = state.current_material.clone();
            if state.doc.edit("CSG Merge", |m, s| ops::csg_merge(m, s, &mat)).is_none() {
                state.set_status("Select at least two brushes to merge");
            }
        }
        Action::CsgIntersect => {
            if state.doc.edit("CSG Intersect", ops::csg_intersect).is_none() {
                state.set_status("Selected brushes do not intersect");
            }
        }
        Action::CsgHollow => {
            let t = state.hollow_thickness.max(state.grid.min(state.hollow_thickness));
            state.doc.edit("Hollow", |m, s| ops::csg_hollow(m, s, t));
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
            state.set_status("Mesh edit mode (Tab returns to object mode)");
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
            if state.doc.edit("Join Meshes", ops::join_meshes).is_none() {
                state.set_status("Select two or more meshes or brushes to join");
            }
        }
        Action::FocusSelection => {
            let b = selection_bounds(state);
            state.focus_request = Some(if b.is_empty() { state.doc.map.bounds_of(state.doc.map.layers.clone()) } else { b });
        }
        Action::CreateBrushEntity(classname) => {
            if state.doc.edit("Create Brush Entity", |m, s| ops::create_brush_entity(m, s, &classname, parent)).is_none() {
                state.set_status("Select brushes first");
            }
        }
        Action::CreatePointEntity { classname, at } => {
            let origin = state.snap(at.or(state.cursor_world).unwrap_or(DVec3::ZERO));
            state.doc.edit("Create Entity", |m, s| {
                let id = ops::create_point_entity(m, parent, &classname, origin);
                s.clear();
                s.select_node(id);
            });
        }
        Action::MoveToWorld => {
            let layer = state.current_layer;
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
            let result = state.doc.edit("Paste", |m, s| {
                let ids = format::paste_nodes(m, parent, &text)?;
                s.clear();
                s.nodes.extend(ids.iter().copied());
                Ok::<_, format::FormatError>(ids)
            });
            match result {
                Ok(ids) => {
                    // Paste at cursor: center the pasted objects under the mouse, snapped to grid.
                    if let Some(cursor) = state.cursor_world {
                        let b = state.doc.map.bounds_of(ids.iter().copied());
                        if !b.is_empty() {
                            let offset = state.snap(cursor - b.center());
                            let offset = DVec3::new(offset.x, 0.0, offset.z);
                            // Part of the paste undo step, the snapshot was taken before pasting.
                            ops::translate_selection(&mut state.doc.map, &state.doc.selection, offset, opts);
                            state.doc.revision += 1;
                        }
                    }
                }
                Err(_) => {
                    state.doc.undo();
                    state.set_status("Clipboard does not contain GodotTrench objects");
                }
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
        Action::OpenGodotEditor | Action::RunGodotProject => launch_godot(state, action == Action::OpenGodotEditor),
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
            state.doc.edit("Auto Paint Terrain", |m, _| {
                for id in &ids {
                    if let Some(t) = m.terrain_mut(*id) {
                        let b = t.bounds();
                        let span = (b.max.y - b.min.y).max(1.0);
                        t.auto_paint(0.35, b.min.y - t.origin.y + span * 0.8, b.min.y - t.origin.y + span * 0.08);
                    }
                }
            });
        }
        Action::ImportQuakeMap => {
            if let Some(path) = rfd::FileDialog::new().add_filter("Quake / TrenchBroom map", &["map"]).pick_file() {
                match import_quake_map(state, &path) {
                    Ok(()) => state.set_status(format!("Imported {}", path.display())),
                    Err(e) => state.set_status(format!("Import failed: {e}")),
                }
            }
        }
        Action::ImportVmf => {
            if let Some(path) = rfd::FileDialog::new().add_filter("Hammer map", &["vmf"]).pick_file() {
                match import_vmf(state, &path) {
                    Ok(()) => state.set_status(format!("Imported {}", path.display())),
                    Err(e) => state.set_status(format!("Import failed: {e}")),
                }
            }
        }
        Action::ExportQuakeMap | Action::ExportQuakeMapCordon => {
            let name = state.doc.path.as_ref().and_then(|p| p.file_stem()).map(|s| format!("{}.map", s.to_string_lossy())).unwrap_or_else(|| "map.map".into());
            if let Some(path) = rfd::FileDialog::new().add_filter("Quake / TrenchBroom map", &["map"]).set_file_name(name).save_file() {
                let options = gt_formats::quake_map::ExportOptions { cordon: action == Action::ExportQuakeMapCordon, ..Default::default() };
                match std::fs::write(&path, gt_formats::quake_map::export_with(&state.doc.map, options)) {
                    Ok(()) => state.set_status(format!("Exported {}", path.display())),
                    Err(e) => state.set_status(format!("Export failed: {e}")),
                }
            }
        }
        Action::ImportModel(mode) => {
            if let Some(path) = rfd::FileDialog::new().add_filter("Models", &["bbmodel", "glb", "gltf"]).pick_file() {
                let at = state.snap(state.cursor_world.unwrap_or(DVec3::ZERO));
                match import_model(state, &path, mode, at) {
                    Ok(n) => state.set_status(format!("Imported {} ({n} object(s))", path.display())),
                    Err(e) => state.set_status(format!("Import failed: {e}")),
                }
            }
        }
        Action::ReloadModels => {
            state.models.clear();
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
        Action::AlignTextureToView | Action::MeshUv(_) | Action::ShowHotspotEditor => {}
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
            let mut dialog = rfd::FileDialog::new().add_filter("GodotTrench map", &["gtm"]);
            if let Some(dir) = state.doc.path.as_ref().and_then(|p| p.parent()) {
                dialog = dialog.set_directory(dir);
            }
            if let Some(path) = dialog.pick_file() {
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
            state.active_scatter = None;
            state.set_status("The next scatter stroke starts a new set on its own layer");
        }
        Action::ScatterFill => {
            let mut rng = gt_doc::scatter::Rng::new(time_seed());
            match crate::scatter_tool::fill(state, &mut rng) {
                Ok(n) => state.set_status(format!("Filled the scatter targets with {n} instances")),
                Err(e) => state.set_status(e),
            }
        }
        Action::ScatterPreset(name) => {
            if crate::scatter_tool::apply_preset(state, &name) {
                let missing = state.game.resolve_res(gt_doc::scatter::NATURE_DIR).is_some_and(|d| !d.join("pine.bbmodel").is_file());
                if missing {
                    let _ = crate::scatter_tool::install_nature(state, false);
                }
                state.set_status(format!("Scatter palette: {name}"));
            }
        }
        Action::InstallNatureModels => match crate::scatter_tool::install_nature(state, false) {
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
        Action::ShowScatterPalette | Action::ShowLinkDialog | Action::ShowReference => {}
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

pub fn time_seed() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(7)
}

/// Opens a map in a new tab, or in the current one when it is an untouched empty map.
pub fn open_map_in_tab(state: &mut EditorState, path: &std::path::Path) -> Result<(), String> {
    let active = state.active_tab.min(state.tabs.len());
    let existing = (0..=state.tabs.len()).position(|i| {
        let doc = if i == active { &state.doc } else { &state.tabs[if i > active { i - 1 } else { i }].doc };
        doc.path.as_deref() == Some(path)
    });
    if let Some(i) = existing {
        state.switch_tab(i);
        return Ok(());
    }
    let untouched = state.doc.path.is_none() && !state.doc.is_modified() && state.doc.map.nodes.len() <= 1;
    if untouched {
        return state.open_map(path);
    }
    let map = format::load(path).map_err(|e| e.to_string())?;
    state.open_tab(Document::from_map(map, Some(path.to_path_buf())));
    if let Some(root) = gt_formats::game::find_project_root(path)
        && state.game.project_root.as_deref() != Some(root.as_path())
    {
        state.load_project(&root);
    }
    state.set_status(format!("Opened {}", path.display()));
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
    let Some(path) = rfd::FileDialog::new()
        .add_filter("GodotTrench map", &["gtm"])
        .set_directory(map_path.parent().unwrap_or(std::path::Path::new(".")))
        .set_file_name("prefab.gtm")
        .save_file()
    else {
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
    state.doc.edit("Create Prefab", |m, s| {
        ops::delete_selection(m, s);
        let id = m.insert(
            parent,
            gt_doc::NodeKind::Instance(gt_doc::map::Instance { path: reference.clone(), origin: pivot, angles: DVec3::ZERO, fixup: String::new() }),
        );
        s.select_node(id);
    });
    state.set_status(format!("Prefab saved to {}", path.display()));
}

/// Opens a Quake `.map` as a new, unsaved GodotTrench document.
pub fn import_quake_map(state: &mut EditorState, path: &std::path::Path) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let map = gt_formats::quake_map::import(&text).map_err(|e| e.to_string())?;
    state.reset_document(Document::from_map(map, None));
    state.doc.revision += 1;
    Ok(())
}

/// Opens a Hammer `.vmf` in a new tab.
pub fn import_vmf(state: &mut EditorState, path: &std::path::Path) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let map = gt_formats::vmf::import(&text).map_err(|e| e.to_string())?;
    let mut doc = Document::from_map(map, None);
    doc.revision += 1;
    state.open_tab(doc);
    Ok(())
}

fn sanitize(name: &str) -> String {
    name.chars().map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c.to_ascii_lowercase() } else { '_' }).collect()
}

/// Imports a model file. Blockbench textures are written into the project texture folder so faces can use them.
pub fn import_model(state: &mut EditorState, path: &std::path::Path, mode: ModelImport, at: DVec3) -> Result<usize, String> {
    let parent = state.insert_parent();
    let is_bb = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("bbmodel"));
    if mode == ModelImport::Prop || !is_bb {
        let reference = match &state.game.project_root {
            Some(root) => gt_formats::game::to_res_path(root, path).unwrap_or_else(|| path.to_string_lossy().replace('\\', "/")),
            None => path.to_string_lossy().replace('\\', "/"),
        };
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
        let rects = hotspot_rects(state, &info.material);
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

/// Godot executable from preferences, the GODOT environment variable, or PATH.
pub fn godot_executable(state: &EditorState) -> Option<std::path::PathBuf> {
    if !state.prefs.godot_path.as_os_str().is_empty() && state.prefs.godot_path.is_file() {
        return Some(state.prefs.godot_path.clone());
    }
    if let Some(p) = std::env::var_os("GODOT").map(std::path::PathBuf::from).filter(|p| p.is_file()) {
        return Some(p);
    }
    let names = if cfg!(windows) { vec!["godot.exe", "godot4.exe", "Godot.exe"] } else { vec!["godot", "godot4"] };
    std::env::var_os("PATH").and_then(|paths| std::env::split_paths(&paths).flat_map(|dir| names.iter().map(move |n| dir.join(n))).find(|p| p.is_file()))
}

fn launch_godot(state: &mut EditorState, editor: bool) {
    let Some(root) = state.game.project_root.clone() else {
        state.set_status("Open a Godot project first");
        return;
    };
    let Some(exe) = godot_executable(state) else {
        state.set_status("Godot executable not found. Set it in Preferences or the GODOT environment variable");
        return;
    };
    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("--path").arg(&root);
    if editor {
        cmd.arg("--editor");
    }
    match cmd.spawn() {
        Ok(_) => state.set_status(format!("Launched {}", exe.display())),
        Err(e) => state.set_status(format!("Could not launch Godot: {e}")),
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
        let roots: Vec<NodeId> = prefab.layers.iter().flat_map(|l| prefab.get(*l).map(|n| n.children.clone()).unwrap_or_default()).collect();
        contents.push((*id, inst.clone(), format::nodes_to_string(&prefab, &roots)));
    }
    state.doc.edit("Explode Instance", |m, s| {
        s.clear();
        for (id, inst, text) in contents {
            let parent = m.get(id).and_then(|n| n.parent).unwrap_or(m.default_layer());
            let Ok(ids) = format::paste_nodes(m, parent, &text) else { continue };
            let mut sel = gt_doc::Selection::default();
            sel.nodes.extend(ids.iter().copied());
            ops::transform_selection(m, &sel, &crate::prefabs::instance_transform(&inst), ops::EditOptions { uv_lock: true, grid: 0.0 });
            if !inst.fixup.is_empty() {
                let prefix = format!("{}-", inst.fixup);
                for new_id in sel.transformables(m) {
                    if let Some(e) = m.entity_mut(new_id) {
                        for key in ["targetname", "target"] {
                            if let Some(v) = e.properties.get_mut(key).filter(|v| !v.is_empty() && !v.starts_with('!')) {
                                *v = format!("{prefix}{v}");
                            }
                        }
                        for o in &mut e.outputs {
                            if !o.target.starts_with('!') {
                                o.target = format!("{prefix}{}", o.target);
                            }
                        }
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
    fn presets_and_overrides() {
        let mut prefs = Prefs::default();
        let tb = shortcuts(&prefs);
        assert!(tb.iter().any(|(s, a)| *a == Action::EditMesh && s.logical_key == Key::Tab));
        prefs.keymap_preset = "hammer".into();
        let hammer = shortcuts(&prefs);
        assert!(hammer.iter().any(|(s, a)| *a == Action::SetTool(ToolKind::Clip) && s.modifiers.shift && s.logical_key == Key::X));
        assert!(!hammer.iter().any(|(s, a)| *a == Action::SetTool(ToolKind::Clip) && s.logical_key == Key::C));
        prefs.key_overrides.insert(Action::CsgSubtract.binding_id(), "Ctrl+Shift+Q".into());
        let custom = shortcuts(&prefs);
        let (s, _) = custom.iter().find(|(_, a)| *a == Action::CsgSubtract).unwrap();
        assert_eq!(shortcut_to_text(s), "Ctrl+Shift+Q");
        assert_eq!(parse_shortcut("Alt+H").map(|s| s.logical_key), Some(Key::H));
        let ids: Vec<String> = bindable_actions().iter().map(|a| a.binding_id()).collect();
        let unique: std::collections::BTreeSet<&String> = ids.iter().collect();
        assert_eq!(ids.len(), unique.len());
    }
}
