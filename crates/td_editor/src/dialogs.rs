//! Floating windows: command palette, shape generator, terrain generator and keymap editor.

use egui::{Key, RichText};
use td_core::{Aabb, DVec3};
use td_doc::NodeKind;
use td_geom::heightfield::{TerrainGen, TerrainShape};
use td_geom::{Brush, Terrain, TerrainLayer, mesh_shapes, shapes};

use crate::commands::{self, Action, ModelImport};
use crate::i18n::{self, Lang, Msg, en, tr, tr_n, trf};
use crate::mesh_tool::MeshOp;
use crate::state::{EditorState, Shade};
use crate::tools::ToolKind;
use crate::widgets;

/// Subsequence match score, higher is better. None if `query` is not a subsequence of `text`.
pub fn fuzzy_score(query: &str, text: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }

    let text_lower = text.to_lowercase();
    let mut score = 0;
    let mut last: Option<usize> = None;
    let chars: Vec<char> = text_lower.chars().collect();
    let mut pos = 0;
    for qc in query.to_lowercase().chars().filter(|c| !c.is_whitespace()) {
        let found = (pos..chars.len()).find(|i| chars[*i] == qc)?;
        score += 10;
        if last.is_some_and(|l| l + 1 == found) {
            score += 8;
        }

        if found == 0 || !chars[found - 1].is_alphanumeric() {
            score += 6;
        }

        score -= (found - pos).min(10) as i32;
        last = Some(found);
        pos = found + 1;
    }

    Some(score)
}

#[derive(Default)]
pub struct CommandPalette {
    pub open: bool,
    query: String,
    selected: usize,
}

/// A command listed as "Category: Name", `category` and `name` are English keys. `en` keeps the English label for
/// searching, `shown` is in the editor's language.
fn entry(category: &str, name: &str, action: Action) -> (Msg, Action) {
    let en = format!("{}: {}", i18n::english(category), i18n::english(name));
    (Msg { en, shown: format!("{}: {}", tr(category), tr(name)) }, action)
}

/// Like [`entry`] for a name that is built at run time, `name` is called again in English for the English label.
fn entry_with(category: &str, name: impl Fn() -> String, action: Action) -> (Msg, Action) {
    let en = {
        let _en = i18n::scope(Lang::En);
        format!("{}: {}", i18n::english(category), name())
    };
    (Msg { en, shown: format!("{}: {}", tr(category), name()) }, action)
}

fn palette_entries(state: &EditorState) -> Vec<(Msg, Action)> {
    let mut out: Vec<(Msg, Action)> = vec![
        entry(en!("File"), en!("New Map"), Action::NewMap),
        entry(en!("File"), en!("Open Map"), Action::OpenMap),
        entry(en!("File"), en!("Save"), Action::Save),
        entry(en!("File"), en!("Save As"), Action::SaveAs),
        entry(en!("File"), en!("Import .map"), Action::ImportQuakeMap),
        entry(en!("File"), en!("Export .map"), Action::ExportQuakeMap),
        entry(en!("File"), en!("Export glTF Binary (.glb) for Blender"), Action::ExportGlb),
        entry(en!("File"), en!("Export Wavefront OBJ (.obj)"), Action::ExportObj),
        entry(en!("Godot"), en!("Open Godot Project"), Action::OpenProject),
        entry(en!("Godot"), en!("Reload Game Config"), Action::ReloadProject),
        entry(en!("Godot"), en!("Open Project in Godot Editor"), Action::OpenGodotEditor),
        entry(en!("Godot"), en!("Run Project"), Action::RunGodotProject),
        entry(en!("Godot"), en!("Build in Godot"), Action::BuildInGodot),
        entry(en!("Godot"), en!("Toggle Live Mode"), Action::ToggleLiveMode),
        entry(en!("Edit"), en!("Undo"), Action::Undo),
        entry(en!("Edit"), en!("Redo"), Action::Redo),
        entry(en!("Edit"), en!("Duplicate"), Action::Duplicate),
        entry(en!("Edit"), en!("Rename"), Action::Rename),
        entry(en!("Edit"), en!("Delete"), Action::Delete),
        entry(en!("Edit"), en!("Copy"), Action::Copy),
        entry(en!("Edit"), en!("Cut"), Action::Cut),
        entry(en!("Select"), en!("All"), Action::SelectAll),
        entry(en!("Select"), en!("None"), Action::SelectNone),
        entry(en!("Select"), en!("Inverse"), Action::SelectInverse),
        entry(en!("Select"), en!("Touching"), Action::SelectTouching),
        entry(en!("Select"), en!("Inside"), Action::SelectInside),
        entry(en!("Select"), en!("Siblings"), Action::SelectSiblings),
        entry(en!("Select"), en!("Same Material"), Action::SelectSameMaterial),
        entry(en!("Group"), en!("Group Selection"), Action::Group),
        entry(en!("Group"), en!("Ungroup"), Action::Ungroup),
        entry(en!("Group"), en!("Close Group"), Action::CloseGroup),
        entry(en!("Prefab"), en!("Create from Selection"), Action::CreatePrefab),
        entry(en!("Prefab"), en!("Insert"), Action::InsertPrefab),
        entry(en!("Prefab"), en!("Explode Instance"), Action::ExplodeInstances),
        entry(en!("Prefab"), en!("Open"), Action::OpenPrefab),
        entry(en!("View"), en!("Hide Selected"), Action::HideSelected),
        entry(en!("View"), en!("Isolate Selected"), Action::IsolateSelected),
        entry(en!("View"), en!("Show All"), Action::UnhideAll),
        entry(en!("View"), en!("Lock Selected"), Action::LockSelected),
        entry(en!("View"), en!("Unlock All"), Action::UnlockAll),
        entry(en!("View"), en!("Focus Selection"), Action::FocusSelection),
        entry(en!("View"), en!("Cycle Shading"), Action::ToggleTextured),
        entry(en!("View"), en!("Increase Interface Scale"), Action::UiScaleUp),
        entry(en!("View"), en!("Decrease Interface Scale"), Action::UiScaleDown),
        entry(en!("View"), en!("Reset Interface Scale"), Action::UiScaleReset),
        entry(en!("View"), en!("Maximize View"), Action::ToggleMaximizeView),
        entry(en!("View"), en!("Single View"), Action::ViewLayout(1)),
        entry(en!("View"), en!("Two Views"), Action::ViewLayout(2)),
        entry(en!("View"), en!("Four Views"), Action::ViewLayout(4)),
        entry(en!("Grid"), en!("Smaller"), Action::GridDown),
        entry(en!("Grid"), en!("Larger"), Action::GridUp),
        entry(en!("Grid"), en!("Toggle Snap"), Action::ToggleSnap),
        entry(en!("Brush"), en!("CSG Subtract"), Action::CsgSubtract),
        entry(en!("Brush"), en!("CSG Convex Merge"), Action::CsgMerge),
        entry(en!("Brush"), en!("CSG Intersect"), Action::CsgIntersect),
        entry(en!("Brush"), en!("Hollow"), Action::CsgHollow),
        entry(en!("Brush"), en!("Snap Vertices to Grid"), Action::SnapVertices),
        entry(en!("Brush"), en!("Move to World"), Action::MoveToWorld),
        entry(en!("Brush"), en!("Shape Generator"), Action::ShowShapeDialog),
        entry(en!("Brush"), en!("Add Trim (baseboards, molding)"), Action::ShowTrimDialog),
        entry(en!("Brush"), en!("Random Fill from the prefab library"), Action::ShowRandomFillDialog),
        entry(en!("Displacement"), en!("Create (power 2)"), Action::CreateDisplacement(2)),
        entry(en!("Displacement"), en!("Create (power 3)"), Action::CreateDisplacement(3)),
        entry(en!("Displacement"), en!("Create (power 4)"), Action::CreateDisplacement(4)),
        entry(en!("Displacement"), en!("Remove"), Action::RemoveDisplacement),
        entry(en!("Displacement"), en!("Sew"), Action::SewDisplacements),
        entry(en!("Texture"), en!("Toggle UV Lock"), Action::ToggleUvLock),
        entry(en!("Texture"), en!("Hotspot Fit"), Action::HotspotTexture),
        entry(en!("Texture"), en!("UV Editor"), Action::ShowUvEditor),
        entry(en!("Layer"), en!("Add Layer"), Action::AddLayer),
        entry(en!("Godot"), en!("Bake Lighting"), Action::BakeLighting),
        entry(en!("View"), en!("Set Cordon from Selection"), Action::SetCordonFromSelection),
        entry(en!("View"), en!("Toggle Cordon"), Action::ToggleCordon),
        entry(en!("View"), en!("Clear Cordon"), Action::ClearCordon),
        entry(en!("File"), en!("Import Hammer .vmf"), Action::ImportVmf),
        entry(en!("File"), en!("Convert Textures (VTF/VMT, WAD, WAL)"), Action::ConvertTextures),
        entry(en!("File"), en!("Import Model as Mesh (.bbmodel)"), Action::ImportModel(ModelImport::Mesh)),
        entry(en!("File"), en!("Import Model as Brushes (.bbmodel)"), Action::ImportModel(ModelImport::Brushes)),
        entry(en!("File"), en!("Place Model Prop (.bbmodel, .glb)"), Action::ImportModel(ModelImport::Prop)),
        entry(en!("File"), en!("Export .map, Cordon Only"), Action::ExportQuakeMapCordon),
        entry(en!("File"), en!("New Tab"), Action::NewTab),
        entry(en!("File"), en!("Close Tab"), Action::CloseTab),
        entry(en!("File"), en!("Next Tab"), Action::NextTab),
        entry(en!("Help"), en!("Keyboard Shortcuts"), Action::ShowKeymap),
        entry(en!("Godot"), en!("Reload Models"), Action::ReloadModels),
        entry(en!("Godot"), en!("Add Content to Project"), Action::ShowProjectContent),
        entry(en!("Godot"), en!("Install or Update Addon"), Action::ShowAddonSetup),
        entry(en!("Group"), en!("Duplicate Linked"), Action::DuplicateLinked),
        entry(en!("Group"), en!("Unlink Groups"), Action::UnlinkGroups),
        entry(en!("Mesh"), en!("Edit Mesh"), Action::EditMesh),
        entry(en!("Edit"), en!("Toggle Edit Mode"), Action::ToggleEditMode),
        entry(en!("Mesh"), en!("Convert Brushes to Mesh"), Action::ConvertToMesh),
        entry(en!("Mesh"), en!("Convert Meshes to Brushes"), Action::ConvertToBrushes),
        entry(en!("Mesh"), en!("Join Meshes"), Action::JoinMeshes),
        entry(en!("Terrain"), en!("Create Terrain"), Action::ShowTerrainDialog),
        entry(en!("Terrain"), en!("Flatten"), Action::TerrainFlatten),
        entry(en!("Terrain"), en!("Auto Paint Layers"), Action::TerrainAutoPaint),
        entry(en!("Scatter"), en!("New Empty Set"), Action::NewScatterSet),
        entry(en!("Scatter"), en!("Fill Targets"), Action::ScatterFill),
        entry(en!("Scatter"), en!("Panel"), Action::ShowScatterPanel),
        entry(en!("Scatter"), en!("Install Nature Models"), Action::InstallNatureModels),
        entry(en!("Scatter"), en!("Sets to Entities"), Action::ScatterToEntities),
        entry(en!("Texture"), en!("Set Blend Material"), Action::SetBlendMaterial),
        entry(en!("Texture"), en!("Clear Blend Material"), Action::ClearBlendMaterial),
        entry(en!("Gameplay"), en!("Lift, Moving Platform"), Action::MakePlatform),
        entry(en!("Gameplay"), en!("Install or Update Gameplay Entities"), Action::InstallGameplayEntities),
        entry(en!("Gameplay"), en!("Link Selected Entities"), Action::ShowLinkDialog),
        entry(en!("Gameplay"), en!("Logic Graph"), Action::ShowLogicGraph),
        entry(en!("Help"), en!("Entity and Code Reference"), Action::ShowReference),
        entry(en!("File"), en!("Preferences"), Action::ShowPreferences),
        entry(en!("View"), en!("Toggle Transform Gizmo"), Action::ToggleTransformGizmo),
        entry(en!("View"), en!("Toggle Godot Overlays"), Action::ToggleGodotOverlays),
        entry(en!("View"), en!("Toggle Walkable Area"), Action::ToggleWalkable),
        entry(en!("Edit"), en!("Repeat Last"), Action::RepeatLast),
        entry(en!("Group"), en!("Open Group"), Action::OpenGroup),
        entry(en!("Brush"), en!("Box from Last Bounds"), Action::CreateBrushFromBounds),
        entry(en!("Godot"), en!("Reload Materials"), Action::ReloadMaterials),
        entry(en!("Texture"), en!("Align to 3D View"), Action::AlignTextureToView),
        entry(en!("Texture"), en!("Reset Alignment"), Action::ResetTexture),
        entry(en!("Texture"), en!("Copy Material and Alignment"), Action::CopyAlignment),
        entry(en!("Texture"), en!("Paste Alignment"), Action::PasteAlignment),
        entry(en!("Texture"), en!("Toggle Treat as One"), Action::ToggleTreatAsOne),
        entry(en!("Texture"), en!("Hotspot Editor"), Action::ShowHotspotEditor),
    ];
    for s in Shade::ALL {
        out.push(entry_with(en!("Shading"), || tr(s.title()).into(), Action::SetShade(s)));
    }

    for v in crate::bake::View::ALL {
        out.push(entry_with(en!("Shading"), || trf!("Baked View, {view}", view = tr(v.label())), Action::SetBakeView(v)));
    }

    for t in crate::state::UiTheme::ALL {
        out.push(entry_with(en!("View"), || trf!("Theme: {theme}", theme = tr(t.label())), Action::SetTheme(t)));
    }

    for l in crate::i18n::Lang::ALL {
        out.push(entry_with(en!("View"), || trf!("Language: {language}", language = l.name()), Action::SetLanguage(l)));
    }

    for j in td_geom::Justify::ALL {
        out.push(entry_with(en!("Texture"), || trf!("Justify {side}", side = tr(j.label())), Action::Justify(j)));
    }

    for d in [0.125, 0.25, 0.5, 1.0, 2.0, 4.0] {
        out.push(entry_with(en!("Texture"), || trf!("Texel Density {d} units per pixel", d), Action::TexelDensity(d)));
    }

    for k in crate::texture_ops::MeshUvKind::ALL {
        out.push(entry_with(en!("Mesh UVs"), || tr(k.label()).into(), Action::MeshUv(k)));
    }

    for preset in td_doc::scatter::PRESETS {
        out.push(entry_with(en!("Scatter"), || trf!("Preset {preset}", preset), Action::ScatterPreset(preset.to_string())));
    }

    for class in crate::volume_tool::VOLUME_CLASSES {
        out.push(entry_with(en!("Gameplay"), || trf!("{class} Around Selection", class), Action::VolumeAroundSelection(class.to_string())));
    }

    {
        use crate::entity_wizards::{DoorKind, HingeSide, SlideDirection};
        for kind in [
            DoorKind::Hinged { side: HingeSide::Left, angle: 95.0 },
            DoorKind::Hinged { side: HingeSide::Right, angle: 95.0 },
            DoorKind::Sliding { direction: SlideDirection::Up, lip: 4.0 },
            DoorKind::Sliding { direction: SlideDirection::Left, lip: 4.0 },
        ] {
            for trigger in [true, false] {
                let action = Action::MakeDoor { kind: kind.clone(), trigger };
                out.push(entry_with(en!("Gameplay"), || action.title(), action.clone()));
            }
        }
    }

    for op in MeshOp::ALL {
        out.push(entry_with(en!("Mesh"), || tr(op.label()).into(), Action::MeshOp(op)));
    }

    for n in 1..=9u8 {
        out.push(entry_with(en!("Camera"), || trf!("Store Bookmark {n}", n), Action::StoreCamera(n)));
        out.push(entry_with(en!("Camera"), || trf!("Go to Bookmark {n}", n), Action::RecallCamera(n)));
    }

    for t in ToolKind::all() {
        out.push(entry_with(en!("Tool"), || tr(t.label()).into(), Action::SetTool(t)));
    }

    for axis in 0..3 {
        let a = commands::axis_name(axis);
        out.push(entry_with(en!("Transform"), || trf!("Rotate {axis} +90", axis = a), Action::Rotate { axis, degrees: 90.0 }));
        out.push(entry_with(en!("Transform"), || trf!("Rotate {axis} -90", axis = a), Action::Rotate { axis, degrees: -90.0 }));
        out.push(entry_with(en!("Transform"), || trf!("Flip {axis}", axis = a), Action::Flip { axis }));
        for sign in [1.0, -1.0] {
            let mut offset = DVec3::ZERO;
            offset[axis] = sign * state.grid;
            let step = format!("{}{a}", if sign > 0.0 { "+" } else { "-" });
            out.push(entry_with(en!("Transform"), || trf!("Nudge by Grid {step}", step), Action::Nudge(offset)));
        }
    }

    for def in state.game.solid_entities() {
        out.push(entry_with(en!("Brush Entity"), || def.classname.clone(), Action::CreateBrushEntity(def.classname.clone())));
    }

    for def in state.game.point_entities() {
        let class = &def.classname;
        out.push(entry_with(en!("Entity"), || trf!("Place {class}", class), Action::CreatePointEntity { classname: class.clone(), at: None }));
    }

    for layer in &state.doc.map.layers {
        if let Some(n) = state.doc.map.get(*layer) {
            let name = n.name();
            out.push(entry_with(en!("Layer"), || trf!("Move Selection to {name}", name), Action::MoveToLayer(*layer)));
        }
    }

    if !state.godot.found() {
        let runs = state.godot_has_project();
        out.retain(|(_, a)| !matches!(a, Action::OpenGodotEditor) && (runs || !matches!(a, Action::RunGodotProject)));
    }

    out
}

/// The entries that match `query` in the shown label or the English one, best first.
fn ranked(query: &str, entries: Vec<(Msg, Action)>) -> Vec<(i32, Msg, Action)> {
    let mut matches: Vec<(i32, Msg, Action)> = entries
        .into_iter()
        .filter_map(|(label, action)| [fuzzy_score(query, &label.shown), fuzzy_score(query, &label.en)].into_iter().flatten().max().map(|s| (s, label, action)))
        .collect();
    matches.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.shown.cmp(&b.1.shown)));
    matches
}

impl CommandPalette {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        self.query.clear();
        self.selected = 0;
    }

    pub fn show(&mut self, ctx: &egui::Context, state: &EditorState, actions: &mut Vec<Action>) {
        if !self.open {
            return;
        }

        let matches = ranked(&self.query, palette_entries(state));
        self.selected = self.selected.min(matches.len().saturating_sub(1));

        let (up, down, enter, escape) = ctx.input_mut(|i| {
            (
                i.consume_key(egui::Modifiers::NONE, Key::ArrowUp),
                i.consume_key(egui::Modifiers::NONE, Key::ArrowDown),
                i.consume_key(egui::Modifiers::NONE, Key::Enter),
                i.consume_key(egui::Modifiers::NONE, Key::Escape),
            )
        });
        if escape {
            self.open = false;
            return;
        }

        if down {
            self.selected = (self.selected + 1).min(matches.len().saturating_sub(1));
        }

        if up {
            self.selected = self.selected.saturating_sub(1);
        }

        let mut run: Option<Action> = None;
        if enter && let Some((_, _, a)) = matches.get(self.selected) {
            run = Some(a.clone());
        }

        let frame = egui::Frame::window(&ctx.global_style()).fill(ctx.global_style().visuals.window_fill.gamma_multiply(1.0)).shadow(egui::Shadow {
            offset: [0, 8],
            blur: 24,
            spread: 0,
            color: egui::Color32::from_black_alpha(crate::theme::colors().modal_shadow_alpha),
        });
        // Modal, so a click outside closes the palette instead of reaching the view under it.
        let area = egui::Modal::default_area(egui::Id::new("command_palette")).anchor(egui::Align2::CENTER_TOP, [0.0, 80.0]);
        let modal =
            egui::Modal::new(egui::Id::new("command_palette")).area(area).frame(frame).backdrop_color(egui::Color32::from_black_alpha(60)).show(ctx, |ui| {
                ui.set_width(460.0);
                let edit = ui.add(egui::TextEdit::singleline(&mut self.query).hint_text(tr("Type a command…")).desired_width(f32::INFINITY));
                edit.request_focus();
                if edit.changed() {
                    self.selected = 0;
                }

                ui.separator();
                let help = matches.get(self.selected).and_then(|(_, _, a)| a.help()).map(tr);
                egui::ScrollArea::vertical().max_height(if help.is_some() { 270.0 } else { 320.0 }).show(ui, |ui| {
                    for (i, (_, label, action)) in matches.iter().enumerate().take(200) {
                        let shortcut = commands::shortcut_text(ctx, &state.prefs, action).unwrap_or_default();
                        let resp = ui.horizontal(|ui| {
                            let r = ui.selectable_label(i == self.selected, &label.shown);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| ui.label(RichText::new(shortcut).weak()));
                            r
                        });
                        if i == self.selected {
                            resp.inner.scroll_to_me(None);
                        }

                        if resp.inner.clicked() {
                            run = Some(action.clone());
                        }
                    }
                });
                if let Some(help) = help {
                    ui.separator();
                    ui.label(RichText::new(help).weak());
                }
            });
        if modal.should_close() {
            self.open = false;
            return;
        }

        if let Some(a) = run {
            actions.push(a);
            self.open = false;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeKind {
    Cylinder,
    Cone,
    Sphere,
    Wedge,
    Spike,
    Arch,
    Pipe,
    Stairs,
    SpiralStairs,
    Torus,
    ArchWall,
    GableRoof,
    Spire,
    Grid,
    Text,
}

impl ShapeKind {
    const ALL: [ShapeKind; 15] = [
        ShapeKind::Cylinder,
        ShapeKind::Cone,
        ShapeKind::Sphere,
        ShapeKind::Wedge,
        ShapeKind::Spike,
        ShapeKind::Arch,
        ShapeKind::Pipe,
        ShapeKind::Stairs,
        ShapeKind::SpiralStairs,
        ShapeKind::Torus,
        ShapeKind::ArchWall,
        ShapeKind::GableRoof,
        ShapeKind::Spire,
        ShapeKind::Grid,
        ShapeKind::Text,
    ];

    fn label(&self) -> &'static str {
        match self {
            ShapeKind::Cylinder => en!("Cylinder"),
            ShapeKind::Cone => en!("Cone"),
            ShapeKind::Sphere => en!("Sphere"),
            ShapeKind::Wedge => en!("Wedge"),
            ShapeKind::Spike => en!("Spike"),
            ShapeKind::Arch => en!("Arch"),
            ShapeKind::Pipe => en!("Pipe"),
            ShapeKind::Stairs => en!("Stairs"),
            ShapeKind::SpiralStairs => en!("Spiral Stairs"),
            ShapeKind::Torus => en!("Torus"),
            ShapeKind::ArchWall => en!("Arch Wall"),
            ShapeKind::GableRoof => en!("Gable Roof"),
            ShapeKind::Spire => en!("Spire"),
            ShapeKind::Grid => en!("Grid"),
            ShapeKind::Text => en!("Text"),
        }
    }

    /// Shapes that only exist as meshes.
    fn mesh_only(&self) -> bool {
        matches!(self, ShapeKind::Torus | ShapeKind::ArchWall | ShapeKind::GableRoof | ShapeKind::Spire | ShapeKind::Grid)
    }
}

pub struct ShapeDialog {
    pub open: bool,
    kind: ShapeKind,
    sides: usize,
    thickness: f64,
    steps: usize,
    as_mesh: bool,
    smooth: bool,
    axis: shapes::Axis,
    text: TextShape,
}

/// Block letter settings of the Text shape, laid out from the corner of the bounds.
#[derive(Clone, Debug, PartialEq)]
pub struct TextShape {
    pub text: String,
    pub letter_height: f64,
    pub depth: f64,
    /// Gap between letters in font cells.
    pub spacing: f64,
    /// Upright letters reading from +Z, else lying on the floor with their tops towards -Z.
    pub standing: bool,
}

impl Default for TextShape {
    fn default() -> Self {
        Self { text: "TEXT".into(), letter_height: 64.0, depth: 8.0, spacing: 1.0, standing: true }
    }
}

impl TextShape {
    pub fn brushes(&self, corner: DVec3, material: &str) -> Result<Vec<Brush>, String> {
        let bounds = td_geom::block_text::sized_bounds(&self.text, corner, self.letter_height, self.depth, self.standing, self.spacing)?;
        let letters = td_geom::block_text::layout(&self.text, &bounds, self.standing, self.spacing)?;
        Ok(letters.iter().flatten().filter_map(|b| Brush::from_aabb(b, material).ok()).collect())
    }
}

impl Default for ShapeDialog {
    fn default() -> Self {
        Self {
            open: false,
            kind: ShapeKind::Cylinder,
            sides: 16,
            thickness: 16.0,
            steps: 8,
            as_mesh: false,
            smooth: true,
            axis: shapes::Axis::Y,
            text: TextShape::default(),
        }
    }
}

impl ShapeDialog {
    pub fn generate(&self, bounds: &Aabb, material: &str) -> Vec<NodeKind> {
        let brushes = |list: Vec<Brush>| -> Vec<NodeKind> { list.into_iter().map(NodeKind::Brush).collect() };
        let smooth = |mut m: td_geom::Mesh| {
            m.smooth_angle = if self.smooth { m.smooth_angle.max(40.0) } else { 0.0 };
            vec![NodeKind::Mesh(m)]
        };
        let as_mesh = self.as_mesh || self.kind.mesh_only();
        let size = bounds.size();
        let (upright, turn) = self.axis.frame(bounds);
        match (self.kind, as_mesh) {
            (ShapeKind::Cylinder, true) => smooth(mesh_shapes::cylinder(&upright, self.sides, material).transformed(&turn, true)),
            (ShapeKind::Cone, true) => smooth(mesh_shapes::cone(&upright, self.sides, material).transformed(&turn, true)),
            (ShapeKind::Sphere, true) => smooth(mesh_shapes::sphere(bounds, self.sides, (self.sides / 2).max(3), material)),
            (ShapeKind::Torus, _) => {
                let r = size.x.min(size.z) * 0.5;
                smooth(mesh_shapes::torus(
                    bounds.center(),
                    (r - self.thickness * 0.5).max(1.0),
                    (self.thickness * 0.5).min(r),
                    self.sides,
                    (self.sides / 2).max(3),
                    material,
                ))
            }
            (ShapeKind::ArchWall, _) => vec![NodeKind::Mesh(mesh_shapes::arch_wall(
                bounds,
                (size.x - self.thickness * 2.0).max(1.0),
                (size.y - self.thickness).max(1.0),
                self.sides,
                material,
            ))],
            (ShapeKind::GableRoof, _) => vec![NodeKind::Mesh(mesh_shapes::gable_roof(bounds, size.z > size.x, self.thickness, self.thickness, material))],
            (ShapeKind::Spire, _) => vec![NodeKind::Mesh(mesh_shapes::spire(bounds, 4, material))],
            (ShapeKind::Grid, _) => vec![NodeKind::Mesh(mesh_shapes::grid(bounds, self.steps, self.steps, material))],
            (ShapeKind::Cylinder, false) => brushes(shapes::cylinder(&upright, self.sides, material).map(|b| self.axis.lay(b, bounds)).into_iter().collect()),
            (ShapeKind::Cone, false) => brushes(shapes::cone(&upright, self.sides, material).map(|b| self.axis.lay(b, bounds)).into_iter().collect()),
            (ShapeKind::Sphere, false) => brushes(shapes::sphere(bounds, self.sides, (self.sides / 2).max(3), material)),
            (ShapeKind::Wedge, _) => brushes(shapes::wedge(bounds, material).into_iter().collect()),
            (ShapeKind::Spike, _) => brushes(shapes::spike(bounds, material).into_iter().collect()),
            (ShapeKind::Arch, _) => brushes(shapes::arch(bounds, self.sides, self.thickness, material)),
            (ShapeKind::Pipe, _) => brushes(shapes::pipe(bounds, self.sides, self.thickness, material)),
            (ShapeKind::Stairs, _) => brushes(shapes::stairs(bounds, self.steps, material)),
            (ShapeKind::SpiralStairs, _) => brushes(shapes::spiral_stairs(bounds, self.steps, self.thickness, 1.0, material)),
            (ShapeKind::Text, _) => brushes(self.text.brushes(bounds.min, material).unwrap_or_default()),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, state: &mut EditorState) {
        let mut open = self.open;
        let brushes = state.doc.selection.brushes(&state.doc.map);
        let bounds = if brushes.is_empty() { state.last_bounds } else { state.doc.map.bounds_of(brushes.iter().copied()) };
        widgets::window(en!("Shape Generator")).open(&mut open).resizable(false).show(ctx, |ui| {
            egui::ComboBox::new("shape_kind", tr("Shape")).selected_text(tr(self.kind.label())).show_ui(ui, |ui| {
                for k in ShapeKind::ALL {
                    ui.selectable_value(&mut self.kind, k, tr(k.label()));
                }
            });
            if matches!(
                self.kind,
                ShapeKind::Cylinder | ShapeKind::Cone | ShapeKind::Sphere | ShapeKind::Arch | ShapeKind::Pipe | ShapeKind::Torus | ShapeKind::ArchWall
            ) {
                ui.add(egui::Slider::new(&mut self.sides, 3..=96).text(tr(if matches!(self.kind, ShapeKind::Arch | ShapeKind::ArchWall) {
                    en!("segments")
                } else {
                    en!("sides")
                })));
            }

            if matches!(self.kind, ShapeKind::Arch | ShapeKind::Pipe | ShapeKind::Torus | ShapeKind::ArchWall | ShapeKind::GableRoof | ShapeKind::SpiralStairs)
            {
                ui.add(
                    egui::DragValue::new(&mut self.thickness)
                        .range(1.0..=4096.0)
                        .prefix(format!("{} ", tr(if self.kind == ShapeKind::SpiralStairs { en!("inner radius") } else { en!("thickness") })))
                        .suffix(" u"),
                );
            }

            if matches!(self.kind, ShapeKind::Cylinder | ShapeKind::Cone) {
                ui.horizontal(|ui| {
                    ui.label(tr("Axis"));
                    for a in shapes::Axis::ALL {
                        ui.selectable_value(&mut self.axis, a, a.label());
                    }
                })
                .response
                .on_hover_text(tr("The axis the shape runs along, Y stands it upright, X or Z lays it down like a pipe"));
            }

            if matches!(self.kind, ShapeKind::Stairs | ShapeKind::SpiralStairs | ShapeKind::Grid) {
                ui.add(egui::Slider::new(&mut self.steps, 1..=128).text(tr(if self.kind == ShapeKind::Grid { en!("divisions") } else { en!("steps") })));
            }

            if !self.kind.mesh_only() && matches!(self.kind, ShapeKind::Cylinder | ShapeKind::Cone | ShapeKind::Sphere) {
                ui.checkbox(&mut self.as_mesh, tr("Create as editable mesh"));
            }

            if self.as_mesh || matches!(self.kind, ShapeKind::Torus) {
                ui.checkbox(&mut self.smooth, tr("Smooth shading"));
            }

            if self.kind == ShapeKind::Text {
                let t = &mut self.text;
                egui::Grid::new("shape_text").num_columns(2).show(ui, |ui| {
                    ui.label(tr("Text"));
                    ui.add(egui::TextEdit::multiline(&mut t.text).desired_rows(2).desired_width(180.0))
                        .on_hover_text(tr("A-Z, 0-9, space and . , ! ? ' - + : /, Enter starts a new line"));
                    ui.end_row();
                    ui.label(tr("Letter height"));
                    ui.add(egui::DragValue::new(&mut t.letter_height).range(1.0..=4096.0).suffix(" u"));
                    ui.end_row();
                    ui.label(tr("Depth"));
                    ui.add(egui::DragValue::new(&mut t.depth).range(0.125..=4096.0).suffix(" u"));
                    ui.end_row();
                    ui.label(tr("Spacing"));
                    ui.add(egui::DragValue::new(&mut t.spacing).range(0.0..=10.0).speed(0.05)).on_hover_text(tr("Gap between letters in font cells"));
                    ui.end_row();
                    ui.label(tr("Orientation"));
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut t.standing, true, tr("standing"));
                        ui.selectable_value(&mut t.standing, false, tr("lying"));
                    });
                    ui.end_row();
                });
                if let Err(e) = t.brushes(bounds.min, &state.current_material) {
                    ui.colored_label(ui.visuals().warn_fg_color, e);
                }
            }

            let size = bounds.size();
            let source = tr(if brushes.is_empty() { en!("last brush") } else { en!("replaces selection") });
            if self.kind == ShapeKind::Text {
                let c = bounds.min;
                ui.label(trf!("Starts at {x} {y} {z} ({source})", x = c.x, y = c.y, z = c.z, source));
            } else {
                ui.label(trf!("Bounds {x} x {y} x {z} ({source})", x = size.x, y = size.y, z = size.z, source));
            }

            let preview = self.generate(&bounds, &state.current_material);
            ui.label(RichText::new(tr_n!(preview.len(), "{n} object", "{n} objects")).weak());
            if ui.add_enabled(!preview.is_empty(), egui::Button::new(tr("Create"))).clicked() {
                let parent = state.insert_parent();
                state.doc.edit(en!("Create Shape"), |m, s| {
                    let target_parent = brushes.first().and_then(|b| m.get(*b)).and_then(|n| n.parent).unwrap_or(parent);
                    for b in &brushes {
                        m.remove(*b);
                    }

                    s.clear();
                    for kind in preview {
                        let id = m.insert(target_parent, kind);
                        s.nodes.insert(id);
                    }
                });
            }
        });
        self.open = open;
    }
}

pub struct TerrainDialog {
    pub open: bool,
    resolution: u32,
    cell_size: f64,
    params: TerrainGen,
    layers: [String; 4],
    tiles: [f64; 4],
    auto_paint: bool,
    heightmap: Option<std::path::PathBuf>,
    /// Center of the new terrain, its y is the height of the terrain's zero. None places it at the 3D cursor.
    center: Option<[f64; 3]>,
}

impl Default for TerrainDialog {
    fn default() -> Self {
        Self {
            open: false,
            resolution: 129,
            cell_size: 64.0,
            params: TerrainGen::default(),
            layers: ["dev/green".into(), "dev/grey".into(), "dev/blue".into(), "dev/orange".into()],
            tiles: [256.0; 4],
            auto_paint: true,
            heightmap: None,
            center: None,
        }
    }
}

/// The base Auto Paint measures its height bands from: the lowest point, or a sea level the user sets.
pub fn auto_paint_base_ui(ui: &mut egui::Ui, base: &mut Option<f64>) {
    ui.horizontal(|ui| {
        let mut sea = base.is_some();
        if ui
            .checkbox(&mut sea, tr("Bands from sea level"))
            .on_hover_text(tr("Measures the peak and low bands from this height instead of the lowest point"))
            .changed()
        {
            *base = sea.then_some(0.0);
        }

        if let Some(h) = base {
            ui.add(egui::DragValue::new(h).speed(1.0).prefix("y "));
        }
    });
}

/// Builds a terrain from dialog style parameters. `heightmap` values are 0..1.
#[allow(clippy::too_many_arguments)]
pub fn make_terrain(
    origin: DVec3,
    resolution: u32,
    cell_size: f64,
    params: &TerrainGen,
    layers: &[(String, f64)],
    heightmap: Option<(&[f32], [u32; 2])>,
    auto_paint: bool,
    sea_level: Option<f64>,
) -> Terrain {
    let mut t = Terrain::new(origin, [resolution, resolution], cell_size, "");
    t.layers =
        layers.iter().filter(|(m, _)| !m.is_empty()).take(td_geom::heightfield::MAX_LAYERS).map(|(m, tile)| TerrainLayer::new(m.clone(), *tile)).collect();
    match heightmap {
        Some((values, size)) => {
            t.import_heightmap(values, size, params.height);
            t.erode(params.erosion_iterations, cell_size * 0.9);
        }
        None => t.generate(params),
    }

    if auto_paint && t.layers.len() > 1 {
        t.auto_paint_from(sea_level);
    }

    t
}

fn terrain_shape_label(shape: TerrainShape) -> &'static str {
    match shape {
        TerrainShape::Flat => en!("Flat"),
        TerrainShape::Hills => en!("Hills"),
        TerrainShape::Mountain => en!("Mountain"),
        TerrainShape::Island => en!("Island"),
        TerrainShape::Valley => en!("Valley"),
        TerrainShape::Ridges => en!("Ridges"),
    }
}

impl TerrainDialog {
    pub fn show(&mut self, ctx: &egui::Context, state: &mut EditorState) {
        let mut open = self.open;
        widgets::window(en!("Create Terrain")).open(&mut open).resizable(false).show(ctx, |ui| {
            egui::Grid::new("terrain_grid").num_columns(2).show(ui, |ui| {
                ui.label(tr("Resolution"));
                egui::ComboBox::from_id_salt("terrain_res").selected_text(format!("{0} x {0}", self.resolution)).show_ui(ui, |ui| {
                    for r in [33, 65, 129, 257, 513, 1025] {
                        ui.selectable_value(&mut self.resolution, r, format!("{r} x {r}"));
                    }
                });
                ui.end_row();
                ui.label(tr("Cell size"));
                ui.add(egui::DragValue::new(&mut self.cell_size).range(4.0..=1024.0).suffix(" u"));
                ui.end_row();
                ui.label(tr("Size"));
                let side = (self.resolution - 1) as f64 * self.cell_size;
                ui.label(trf!("{side} units ({meters} m)", side, meters = format!("{:.0}", side / state.game.units_per_meter)));
                ui.end_row();
                ui.label(tr("Shape"));
                egui::ComboBox::from_id_salt("terrain_shape").selected_text(tr(terrain_shape_label(self.params.shape))).show_ui(ui, |ui| {
                    for s in [TerrainShape::Flat, TerrainShape::Hills, TerrainShape::Mountain, TerrainShape::Island, TerrainShape::Valley, TerrainShape::Ridges]
                    {
                        ui.selectable_value(&mut self.params.shape, s, tr(terrain_shape_label(s)));
                    }
                });
                ui.end_row();
                ui.label(tr("Height"));
                ui.add(egui::DragValue::new(&mut self.params.height).range(0.0..=65536.0).suffix(" u"));
                ui.end_row();
                ui.label(tr("Feature size"));
                ui.add(egui::DragValue::new(&mut self.params.feature_size).range(16.0..=65536.0).suffix(" u"));
                ui.end_row();
                ui.label(tr("Octaves / roughness"));
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.params.octaves).range(1..=10));
                    ui.add(egui::Slider::new(&mut self.params.roughness, 0.1..=0.9));
                });
                ui.end_row();
                ui.label(tr("Erosion"));
                ui.add(egui::DragValue::new(&mut self.params.erosion_iterations).range(0..=200));
                ui.end_row();
                ui.label(tr("Seed"));
                ui.add(egui::DragValue::new(&mut self.params.seed));
                ui.end_row();
                for (i, name) in [en!("Base layer"), en!("Slope layer"), en!("Peak layer"), en!("Low layer")].into_iter().enumerate() {
                    ui.label(tr(name));
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.layers[i]).desired_width(140.0));
                        if ui.button(tr("Use Current")).clicked() {
                            self.layers[i] = state.current_material.clone();
                        }

                        ui.add(egui::DragValue::new(&mut self.tiles[i]).range(8.0..=8192.0).prefix(format!("{} ", tr("tile"))).suffix(" u"));
                    });
                    ui.end_row();
                }

                ui.label(tr("Center"));
                ui.horizontal(|ui| {
                    let mut at_cursor = self.center.is_none();
                    if ui.checkbox(&mut at_cursor, tr("3D cursor")).changed() {
                        let cursor = state.cursor_world.map(|c| state.snap(c)).unwrap_or(DVec3::ZERO);
                        // Adding 0.0 turns a snapped -0.0 into 0.0, which the field would show as "-0".
                        self.center = (!at_cursor).then(|| cursor.to_array().map(|v| v + 0.0));
                    }

                    if let Some(c) = &mut self.center {
                        for (v, axis) in c.iter_mut().zip(["x ", "y ", "z "]) {
                            ui.add(egui::DragValue::new(v).speed(8.0).prefix(axis));
                        }
                    }
                });
                ui.end_row();
                ui.label(tr("Heightmap"));
                ui.horizontal(|ui| {
                    ui.label(
                        self.heightmap
                            .as_ref()
                            .map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())
                            .unwrap_or_else(|| tr("procedural").into()),
                    );
                    if ui.button(tr("Import PNG…")).clicked() {
                        self.heightmap = commands::file_dialog(state, |d| d.add_filter(tr("Heightmap"), &["png", "tga", "bmp"]).pick_file());
                    }

                    if self.heightmap.is_some() && ui.button(tr("Clear")).clicked() {
                        self.heightmap = None;
                    }
                });
                ui.end_row();
            });
            ui.checkbox(&mut self.auto_paint, tr("Paint layers from slope and height"));
            if self.auto_paint {
                auto_paint_base_ui(ui, &mut state.auto_paint_base);
            }

            if ui.button(tr("Create")).clicked() {
                let side = (self.resolution - 1) as f64 * self.cell_size;
                let cursor = || state.cursor_world.map(|c| state.snap(c)).unwrap_or(DVec3::ZERO);
                let center = self.center.map(DVec3::from_array).unwrap_or_else(cursor);
                let origin = DVec3::new(center.x - side * 0.5, center.y, center.z - side * 0.5);
                let image = self.heightmap.as_ref().and_then(|p| image::open(p).ok()).map(|img| {
                    let gray = img.to_luma16();
                    let values: Vec<f32> = gray.pixels().map(|p| p.0[0] as f32 / 65535.0).collect();
                    (values, [gray.width(), gray.height()])
                });
                let layers: Vec<(String, f64)> = self.layers.iter().cloned().zip(self.tiles).collect();
                let t = make_terrain(
                    origin,
                    self.resolution,
                    self.cell_size,
                    &self.params,
                    &layers,
                    image.as_ref().map(|(v, s)| (v.as_slice(), *s)),
                    self.auto_paint,
                    state.auto_paint_base,
                );
                let parent = state.insert_parent();
                state.doc.edit(en!("Create Terrain"), |m, s| {
                    let id = m.insert(parent, NodeKind::Terrain(t));
                    s.clear();
                    s.select_node(id);
                });
                state.tool = ToolKind::Sculpt;
                self.open = false;
            }
        });
        if !open {
            self.open = false;
        }
    }
}

#[derive(Default)]
pub struct KeymapWindow {
    pub open: bool,
    filter: String,
    recording: Option<String>,
}

impl KeymapWindow {
    pub fn show(&mut self, ctx: &egui::Context, state: &mut EditorState) {
        if !self.open {
            return;
        }

        if let Some(id) = self.recording.clone() {
            let captured = ctx.input_mut(|i| {
                let found = i.events.iter().find_map(|e| match e {
                    egui::Event::Key { key, pressed: true, modifiers, .. } if !matches!(key, Key::Escape) => {
                        Some(egui::KeyboardShortcut::new(*modifiers, *key))
                    }
                    _ => None,
                });
                let escape = i.consume_key(egui::Modifiers::NONE, Key::Escape);
                i.events.clear();
                (found, escape)
            });
            match captured {
                (_, true) => self.recording = None,
                (Some(s), _) => {
                    state.prefs.key_overrides.insert(id, commands::shortcut_to_text(&s));
                    self.recording = None;
                }
                _ => {}
            }
        }

        let mut open = self.open;
        widgets::window(en!("Keyboard Shortcuts")).open(&mut open).default_size([520.0, 520.0]).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(tr("Preset"));
                egui::ComboBox::from_id_salt("keymap_preset").selected_text(commands::preset_label(&state.prefs.keymap_preset)).show_ui(ui, |ui| {
                    for p in commands::PRESETS {
                        ui.selectable_value(&mut state.prefs.keymap_preset, p.to_string(), commands::preset_label(p));
                    }
                });
                if ui.button(tr("Reset overrides")).clicked() {
                    state.prefs.key_overrides.clear();
                }

                ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text(tr("filter")).desired_width(160.0));
            });
            if self.recording.is_some() {
                ui.label(RichText::new(tr("Press the new shortcut (Esc cancels)")).color(crate::theme::colors().yellow));
            }

            ui.separator();
            let active = commands::shortcuts(&state.prefs);
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("keymap").striped(true).num_columns(3).show(ui, |ui| {
                    for action in commands::bindable_actions() {
                        let label = action.title();
                        if !self.filter.is_empty() && fuzzy_score(&self.filter, &label).or_else(|| fuzzy_score(&self.filter, &action.label())).is_none() {
                            continue;
                        }

                        let id = action.binding_id();
                        let keys: Vec<String> = active.iter().filter(|(_, a)| *a == action).map(|(s, _)| commands::shortcut_to_text(s)).collect();
                        ui.label(label);
                        ui.label(RichText::new(if keys.is_empty() { tr("unbound").to_string() } else { keys.join(", ") }).monospace());
                        ui.horizontal(|ui| {
                            if ui.small_button(if self.recording.as_deref() == Some(id.as_str()) { "…" } else { tr("Set") }).clicked() {
                                self.recording = Some(id.clone());
                            }

                            if ui.small_button(tr("Clear")).clicked() {
                                state.prefs.key_overrides.insert(id.clone(), String::new());
                            }

                            if state.prefs.key_overrides.contains_key(&id) && ui.small_button(tr("Default")).clicked() {
                                state.prefs.key_overrides.remove(&id);
                            }
                        });
                        ui.end_row();
                    }
                });
            });
        });
        self.open = open;
    }
}
/// Connects an output of one selected entity to an input of the other.
#[derive(Default)]
pub struct LinkDialog {
    pub open: bool,
    from: Option<td_core::NodeId>,
    to: Option<td_core::NodeId>,
    output: String,
    input: String,
    parameter: String,
    delay: f64,
}

impl LinkDialog {
    pub fn open_for(&mut self, state: &EditorState) {
        let entities: Vec<td_core::NodeId> = state.doc.selection.nodes.iter().copied().filter(|id| state.doc.map.entity(*id).is_some()).collect();
        if entities.len() != 2 {
            return;
        }

        self.from = Some(entities[0]);
        self.to = Some(entities[1]);
        self.output.clear();
        self.input.clear();
        self.fit_choices(state);
        self.parameter.clear();
        self.delay = 0.0;
        self.open = true;
    }

    /// Swaps source and target. Outputs belong to the source class and inputs to the target, so choices that the new
    /// pair does not offer go back to the first option.
    pub fn swap(&mut self, state: &EditorState) {
        std::mem::swap(&mut self.from, &mut self.to);
        self.fit_choices(state);
    }

    fn fit_choices(&mut self, state: &EditorState) {
        let (Some(from), Some(to)) = (self.from, self.to) else { return };
        let (outs, ins) = crate::entity_wizards::link_options(state, from, to);
        if !outs.contains(&self.output) {
            self.output = outs.first().cloned().unwrap_or_default();
        }

        if !ins.contains(&self.input) {
            self.input = ins.first().cloned().unwrap_or_default();
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, state: &mut EditorState) {
        let (Some(from), Some(to)) = (self.from, self.to) else { return };
        if !self.open {
            return;
        }

        if state.doc.map.entity(from).is_none() || state.doc.map.entity(to).is_none() {
            self.open = false;
            return;
        }

        let name = |id| state.doc.map.get(id).map(|n| n.name()).unwrap_or_default();
        let (from_name, to_name) = (name(from), name(to));
        let (outs, ins) = crate::entity_wizards::link_options(state, from, to);
        let mut open = self.open;
        let mut apply = false;
        let mut swap = false;
        widgets::window(en!("Link Entities")).open(&mut open).resizable(false).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&from_name).strong());
                swap = ui.small_button("⇄").on_hover_text(tr("Swap")).clicked();

                ui.label(RichText::new(&to_name).strong());
            });
            egui::Grid::new("link_grid").num_columns(2).show(ui, |ui| {
                ui.label(tr("When"));
                egui::ComboBox::from_id_salt("link_out").selected_text(&self.output).show_ui(ui, |ui| {
                    for o in outs {
                        ui.selectable_value(&mut self.output, o.clone(), o);
                    }
                });
                ui.end_row();
                ui.label(tr("call"));
                egui::ComboBox::from_id_salt("link_in").selected_text(&self.input).show_ui(ui, |ui| {
                    for i in ins {
                        ui.selectable_value(&mut self.input, i.clone(), i);
                    }
                });
                ui.end_row();
                ui.label(tr("parameter"));
                ui.text_edit_singleline(&mut self.parameter);
                ui.end_row();
                ui.label(tr("delay"));
                ui.add(egui::DragValue::new(&mut self.delay).range(0.0..=3600.0).speed(0.05).suffix(" s"));
                ui.end_row();
            });
            apply = ui.button(tr("Link")).clicked();
        });
        if swap {
            self.swap(state);
        }

        if apply {
            if let (Some(from), Some(to)) = (self.from, self.to) {
                match crate::entity_wizards::link(state, from, to, &self.output, &self.input, &self.parameter, self.delay) {
                    Ok(c) => state.set_status(format!("{} > {}.{}", c.output, c.target, c.input)),
                    Err(e) => state.set_status(e),
                }
            }

            open = false;
        }

        self.open = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palette_frame(ctx: &egui::Context, palette: &mut CommandPalette, state: &EditorState, time: &mut f64, events: Vec<egui::Event>) -> Vec<Action> {
        *time += 0.05;
        let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 800.0));
        ctx.begin_pass(egui::RawInput { events, time: Some(*time), screen_rect: Some(screen), ..Default::default() });
        let mut actions = Vec::new();
        palette.show(ctx, state, &mut actions);
        ctx.end_pass().drop_without_applying_deltas();
        actions
    }

    #[test]
    fn escape_or_a_click_outside_closes_the_palette_without_running_anything() {
        let (ctx, state, mut time) = (egui::Context::default(), EditorState::new(Default::default()), 0.0);
        let mut palette = CommandPalette::default();
        let key = |key| egui::Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE };
        palette.toggle();
        for _ in 0..3 {
            palette_frame(&ctx, &mut palette, &state, &mut time, vec![]);
        }

        assert!(palette_frame(&ctx, &mut palette, &state, &mut time, vec![key(egui::Key::Escape)]).is_empty());
        assert!(!palette.open, "Escape closes it");

        palette.toggle();
        for _ in 0..3 {
            palette_frame(&ctx, &mut palette, &state, &mut time, vec![]);
        }

        let outside = egui::pos2(40.0, 700.0);
        let press = |pressed| egui::Event::PointerButton { pos: outside, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE };
        let mut ran = palette_frame(&ctx, &mut palette, &state, &mut time, vec![egui::Event::PointerMoved(outside)]);
        ran.extend(palette_frame(&ctx, &mut palette, &state, &mut time, vec![press(true)]));
        ran.extend(palette_frame(&ctx, &mut palette, &state, &mut time, vec![press(false)]));
        assert!(ran.is_empty() && !palette.open, "a click outside closes it, ran {ran:?}");
    }

    #[test]
    fn fuzzy_ranks_word_starts() {
        assert!(fuzzy_score("csg", "Brush: CSG Subtract").is_some());
        assert!(fuzzy_score("xyz", "Brush: CSG Subtract").is_none());
        let a = fuzzy_score("sub", "Brush: CSG Subtract").unwrap();
        let b = fuzzy_score("sub", "Select: Same Material but").unwrap_or(i32::MIN);
        assert!(a > b);
    }

    #[test]
    fn the_shape_dialog_builds_block_text() {
        let mut dialog = ShapeDialog { kind: ShapeKind::Text, ..Default::default() };
        dialog.text = TextShape { text: "HI".into(), letter_height: 70.0, depth: 4.0, spacing: 1.0, standing: false };
        let corner = Aabb::new(DVec3::new(100.0, 8.0, 0.0), DVec3::new(164.0, 72.0, 64.0));
        let nodes = dialog.generate(&corner, "dev/orange");
        assert!(nodes.len() >= 4, "H is three boxes, I at least one: {}", nodes.len());
        let mut bounds = Aabb::EMPTY;
        for n in &nodes {
            let NodeKind::Brush(b) = n else { panic!("text is brushes") };
            bounds = bounds.union(&b.bounds());
        }

        assert_eq!(bounds.min.y, 8.0, "letters start at the corner of the bounds");
        assert!(
            (bounds.size().y - 4.0).abs() < 1e-9 && (bounds.size().z - 70.0).abs() < 1e-9,
            "lying letters are depth thick and letter height long: {bounds:?}"
        );
        dialog.text.standing = true;
        let upright = dialog.generate(&corner, "m").iter().fold(Aabb::EMPTY, |acc, n| if let NodeKind::Brush(b) = n { acc.union(&b.bounds()) } else { acc });
        assert!((upright.size().y - 70.0).abs() < 1e-9);
        dialog.text.text = "Ö".into();
        assert!(dialog.generate(&corner, "m").is_empty(), "unsupported letters create nothing");
        assert!(dialog.text.brushes(DVec3::ZERO, "m").unwrap_err().contains("no block letter"));
    }

    #[test]
    fn the_palette_lists_every_language() {
        let state = EditorState::new(crate::state::Prefs::default());
        let entries = palette_entries(&state);
        let languages: Vec<_> = entries.iter().filter(|(_, a)| matches!(a, Action::SetLanguage(_))).collect();
        assert_eq!(languages.len(), crate::i18n::Lang::ALL.len());
        for l in crate::i18n::Lang::ALL {
            assert!(languages.iter().any(|(label, a)| *a == Action::SetLanguage(l) && label.en == format!("View: Language: {}", l.name())));
        }

        let hits = |q: &str| entries.iter().filter(|(label, _)| fuzzy_score(q, &label.en).is_some() && label.en.starts_with("View: Language")).count();
        assert_eq!(hits("language deutsch"), 1);
        assert_eq!(hits("language"), 8);
    }

    #[test]
    fn palette_ranks_grid_larger_first() {
        let state = EditorState::new(crate::state::Prefs::default());
        let matches = ranked("grid larger", palette_entries(&state));
        assert_eq!(matches[0].2, Action::GridUp);
        assert_eq!(matches[0].1.en, "Grid: Larger");
    }

    #[test]
    fn the_palette_finds_a_command_by_its_english_or_its_translated_name() {
        let _ja =
            crate::i18n::test_catalogue(Lang::Ja, r#"{ "Grid": "グリッド", "Larger": "大きく", "Smaller": "小さく", "Select": "選択", "All": "すべて" }"#);
        let state = EditorState::new(crate::state::Prefs::default());
        let _scope = i18n::scope(Lang::Ja);
        let entries = palette_entries(&state);
        let up = entries.iter().find(|(_, a)| *a == Action::GridUp).unwrap().0.clone();
        assert_eq!((up.en.as_str(), up.shown.as_str()), ("Grid: Larger", "グリッド: 大きく"));
        for query in ["grid larger", "グリッド: 大きく", "グリッド 大きく"] {
            let matches = ranked(query, entries.clone());
            assert_eq!(matches[0].2, Action::GridUp, "{query}");
        }

        let down = ranked("グリッド小", entries.clone());
        assert_eq!(down[0].2, Action::GridDown);
        assert!(ranked("zzzz", entries.clone()).is_empty());
        assert_eq!(ranked("", entries.clone()).len(), entries.len(), "an empty query lists everything");
    }

    #[test]
    fn command_help_stays_short() {
        use crate::i18n::{Lang, scope, tr};
        let state = EditorState::new(crate::state::Prefs::default());
        let helps: Vec<_> = palette_entries(&state).into_iter().filter_map(|(label, action)| action.help().map(|h| (label.en, h))).collect();
        assert!(helps.len() > 30, "{} commands explain themselves", helps.len());
        for lang in Lang::ALL {
            let _scope = scope(lang);
            let limit = if matches!(lang, Lang::ZhHans | Lang::Ja) { 60 } else { 100 };
            for (label, help) in &helps {
                let shown = tr(help);
                assert!(shown.chars().count() <= limit, "{} {label}: {} characters", lang.code(), shown.chars().count());
            }
        }
    }

    #[test]
    fn palette_lists_each_command_once() {
        let state = EditorState::new(crate::state::Prefs::default());
        let entries = palette_entries(&state);
        for (i, (label, action)) in entries.iter().enumerate() {
            let twin = entries[i + 1..].iter().find(|(_, other)| other == action);
            assert!(twin.is_none(), "{} and {} run the same command", label.en, twin.unwrap().0.en);
        }

        assert!(entries.iter().any(|(_, a)| *a == Action::ToggleEditMode));
    }

    #[test]
    fn the_generator_and_keymap_windows_open_under_their_english_ids_in_german() {
        let _de = crate::i18n::test_catalogue(
            Lang::De,
            r#"{ "Shape Generator": "Formgenerator", "Create Terrain": "Gelände erstellen", "Keyboard Shortcuts": "Tastenkürzel", "Link Entities": "Entitäten verknüpfen" }"#,
        );
        let _scope = i18n::scope(Lang::De);
        let mut state = EditorState::new(crate::state::Prefs::default());
        let layer = state.doc.map.default_layer();
        state.doc.edit("add", |m, s| {
            let a = td_doc::ops::create_point_entity(m, layer, "logic_branch", DVec3::ZERO);
            let b = td_doc::ops::create_point_entity(m, layer, "light", DVec3::X);
            s.select_node(a);
            s.select_node(b);
        });
        let (mut shape, mut terrain, mut keymap, mut link) = (ShapeDialog::default(), TerrainDialog::default(), KeymapWindow::default(), LinkDialog::default());
        (shape.open, terrain.open, keymap.open) = (true, true, true);
        link.open_for(&state);
        assert!(link.open);
        let ctx = egui::Context::default();
        for time in [0.0, 0.1, 0.2] {
            ctx.begin_pass(egui::RawInput {
                time: Some(time),
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 900.0))),
                ..Default::default()
            });
            shape.show(&ctx, &mut state);
            terrain.show(&ctx, &mut state);
            keymap.show(&ctx, &mut state);
            link.show(&ctx, &mut state);
            ctx.end_pass().drop_without_applying_deltas();
        }

        for english in ["Shape Generator", "Create Terrain", "Keyboard Shortcuts", "Link Entities"] {
            let id = egui::Id::new(Some(english));
            assert!(ctx.memory(|m| m.area_rect(id)).is_some(), "{english} is open under its English id");
            assert!(ctx.memory(|m| m.area_rect(egui::Id::new(Some(tr(english))))).is_none(), "{english} is not under its German id");
        }
    }

    #[test]
    fn no_two_commands_have_the_same_name_in_any_language() {
        let state = EditorState::new(crate::state::Prefs::default());
        for lang in Lang::ALL {
            let _scope = i18n::scope(lang);
            let entries = palette_entries(&state);
            let mut seen = std::collections::HashMap::new();
            for (label, action) in &entries {
                if let Some(other) = seen.insert(label.shown.clone(), action) {
                    panic!("{} {:?} and {:?} are both called {}", lang.code(), other, action, label.shown);
                }
            }
        }
    }

    #[test]
    fn a_connected_godot_editor_runs_the_project_without_an_executable() {
        let mut state = EditorState::new(crate::state::Prefs::default());
        state.godot.exe = None;
        let runs = |state: &EditorState| palette_entries(state).iter().any(|(_, a)| *a == Action::RunGodotProject);
        assert!(!runs(&state));
        let root = std::path::PathBuf::from("C:/game");
        state.game.project_root = Some(root.clone());
        state.link_state.connected = true;
        state.link_state.project = Some(crate::live_link::path_key(&crate::live_link::godot_path(&root)));
        assert!(runs(&state) && !palette_entries(&state).iter().any(|(_, a)| *a == Action::OpenGodotEditor));
    }

    #[test]
    fn jargon_commands_explain_themselves() {
        use crate::i18n::{Lang, scope, tr};
        let state = EditorState::new(crate::state::Prefs::default());
        for (label, action) in palette_entries(&state) {
            let jargon = ["CSG", "Hollow", "UV Lock", "Displacement", "Cordon", "Prefab", "Duplicate Linked", "Convex"].iter().any(|j| label.en.contains(j));
            let help = action.help();
            assert!(!jargon || help.is_some(), "{} needs a help line", label.en);
            for lang in Lang::ALL {
                let _scope = scope(lang);
                let shown = help.map(tr);
                assert!(
                    shown.is_none_or(|h| !h.contains(['\u{2013}', '\u{2014}']) && !h.contains(" - ")),
                    "{} {}: no dashes as punctuation",
                    lang.code(),
                    label.en
                );
            }
        }
    }

    #[test]
    fn swapping_a_link_picks_output_and_input_of_the_new_pair() {
        let mut state = EditorState::new(crate::state::Prefs::default());
        let layer = state.doc.map.default_layer();
        let (branch, light) = state.doc.edit("add", |m, s| {
            let branch = td_doc::ops::create_point_entity(m, layer, "logic_branch", DVec3::ZERO);
            let light = td_doc::ops::create_point_entity(m, layer, "light", DVec3::X);
            s.select_node(branch);
            s.select_node(light);
            (branch, light)
        });
        let mut dialog = LinkDialog::default();
        dialog.open_for(&state);
        let (from, to) = (dialog.from.unwrap(), dialog.to.unwrap());
        assert_eq!((from, to), (branch.min(light), branch.max(light)));
        for _ in 0..2 {
            dialog.swap(&state);
            let (outs, ins) = crate::entity_wizards::link_options(&state, dialog.from.unwrap(), dialog.to.unwrap());
            assert_eq!(dialog.output, outs.first().cloned().unwrap_or_default());
            assert!(ins.contains(&dialog.input), "{} is not an input of the target", dialog.input);
        }
    }

    #[test]
    fn shape_dialog_generates_meshes_and_spirals() {
        let bounds = Aabb::new(DVec3::new(-64.0, 0.0, -64.0), DVec3::new(64.0, 256.0, 64.0));
        let mut d = ShapeDialog { kind: ShapeKind::SpiralStairs, steps: 16, thickness: 16.0, ..Default::default() };
        let stairs = d.generate(&bounds, "m");
        assert_eq!(stairs.len(), 16);
        d.kind = ShapeKind::Cylinder;
        d.as_mesh = true;
        assert!(matches!(d.generate(&bounds, "m").as_slice(), [NodeKind::Mesh(m)] if m.smooth_angle > 0.0));
        for k in [ShapeKind::Torus, ShapeKind::ArchWall, ShapeKind::GableRoof, ShapeKind::Spire, ShapeKind::Grid] {
            d.kind = k;
            assert!(!d.generate(&bounds, "m").is_empty(), "{k:?}");
        }
    }

    #[test]
    fn shape_dialog_lays_a_cylinder_down_along_its_axis() {
        let bounds = Aabb::new(DVec3::new(-100.0, 70.0, -4.0), DVec3::new(100.0, 78.0, 4.0));
        for as_mesh in [false, true] {
            let d = ShapeDialog { kind: ShapeKind::Cylinder, axis: shapes::Axis::X, as_mesh, sides: 10, ..Default::default() };
            let got = match d.generate(&bounds, "m").as_slice() {
                [NodeKind::Brush(b)] => b.bounds(),
                [NodeKind::Mesh(m)] => m.bounds(),
                other => panic!("one shape, got {}", other.len()),
            };
            assert!((got.size().x - 200.0).abs() < 1e-6 && got.size().y <= 8.0 + 1e-6, "runs along X, got {got:?}");
        }
    }

    #[test]
    fn terrain_from_parameters() {
        let params = TerrainGen { shape: TerrainShape::Island, height: 300.0, ..Default::default() };
        let layers = vec![("grass".to_string(), 256.0), ("rock".to_string(), 128.0)];
        let t = make_terrain(DVec3::ZERO, 33, 32.0, &params, &layers, None, true, None);
        assert_eq!(t.layers.len(), 2);
        assert_eq!(t.splat.len(), 33 * 33 * 4);
        let hm: Vec<f32> = (0..16).map(|i| i as f32 / 15.0).collect();
        let from_image = make_terrain(DVec3::ZERO, 17, 32.0, &TerrainGen { erosion_iterations: 0, ..params }, &layers, Some((&hm, [4, 4])), false, None);
        assert!((from_image.height(16, 16) - 300.0).abs() < 1e-3);
    }
}
