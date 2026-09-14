//! Floating windows: command palette, shape generator, terrain generator and keymap editor.

use egui::{Key, RichText};
use gt_core::{Aabb, DVec3};
use gt_doc::NodeKind;
use gt_geom::heightfield::{TerrainGen, TerrainShape};
use gt_geom::{Brush, Terrain, TerrainLayer, mesh_shapes, shapes};

use crate::commands::{self, Action, ModelImport};
use crate::mesh_tool::MeshOp;
use crate::state::{EditorState, Shade};
use crate::tools::ToolKind;

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

fn palette_entries(state: &EditorState) -> Vec<(String, Action)> {
    let mut out: Vec<(String, Action)> = vec![
        ("File: New Map".into(), Action::NewMap),
        ("File: Open Map".into(), Action::OpenMap),
        ("File: Save".into(), Action::Save),
        ("File: Save As".into(), Action::SaveAs),
        ("File: Import .map".into(), Action::ImportQuakeMap),
        ("File: Export .map".into(), Action::ExportQuakeMap),
        ("Godot: Open Godot Project".into(), Action::OpenProject),
        ("Godot: Reload Game Config".into(), Action::ReloadProject),
        ("Godot: Open Project in Godot Editor".into(), Action::OpenGodotEditor),
        ("Godot: Run Project".into(), Action::RunGodotProject),
        ("Edit: Undo".into(), Action::Undo),
        ("Edit: Redo".into(), Action::Redo),
        ("Edit: Duplicate".into(), Action::Duplicate),
        ("Edit: Delete".into(), Action::Delete),
        ("Edit: Copy".into(), Action::Copy),
        ("Edit: Cut".into(), Action::Cut),
        ("Select: All".into(), Action::SelectAll),
        ("Select: None".into(), Action::SelectNone),
        ("Select: Inverse".into(), Action::SelectInverse),
        ("Select: Touching".into(), Action::SelectTouching),
        ("Select: Inside".into(), Action::SelectInside),
        ("Select: Siblings".into(), Action::SelectSiblings),
        ("Select: Same Material".into(), Action::SelectSameMaterial),
        ("Group: Group Selection".into(), Action::Group),
        ("Group: Ungroup".into(), Action::Ungroup),
        ("Group: Close Group".into(), Action::CloseGroup),
        ("Prefab: Create from Selection".into(), Action::CreatePrefab),
        ("Prefab: Insert".into(), Action::InsertPrefab),
        ("Prefab: Explode Instance".into(), Action::ExplodeInstances),
        ("Prefab: Open".into(), Action::OpenPrefab),
        ("View: Hide Selected".into(), Action::HideSelected),
        ("View: Isolate Selected".into(), Action::IsolateSelected),
        ("View: Show All".into(), Action::UnhideAll),
        ("View: Lock Selected".into(), Action::LockSelected),
        ("View: Unlock All".into(), Action::UnlockAll),
        ("View: Focus Selection".into(), Action::FocusSelection),
        ("View: Cycle Shading".into(), Action::ToggleTextured),
        ("Grid: Smaller".into(), Action::GridDown),
        ("Grid: Larger".into(), Action::GridUp),
        ("Grid: Toggle Snap".into(), Action::ToggleSnap),
        ("Brush: CSG Subtract".into(), Action::CsgSubtract),
        ("Brush: CSG Convex Merge".into(), Action::CsgMerge),
        ("Brush: CSG Intersect".into(), Action::CsgIntersect),
        ("Brush: Hollow".into(), Action::CsgHollow),
        ("Brush: Snap Vertices to Grid".into(), Action::SnapVertices),
        ("Brush: Move to World".into(), Action::MoveToWorld),
        ("Brush: Shape Generator".into(), Action::ShowShapeDialog),
        ("Displacement: Create (power 2)".into(), Action::CreateDisplacement(2)),
        ("Displacement: Create (power 3)".into(), Action::CreateDisplacement(3)),
        ("Displacement: Create (power 4)".into(), Action::CreateDisplacement(4)),
        ("Displacement: Remove".into(), Action::RemoveDisplacement),
        ("Displacement: Sew".into(), Action::SewDisplacements),
        ("Texture: Toggle UV Lock".into(), Action::ToggleUvLock),
        ("Texture: Hotspot Fit Faces".into(), Action::HotspotTexture),
        ("Texture: UV Editor".into(), Action::ShowUvEditor),
        ("Layer: Add Layer".into(), Action::AddLayer),
        ("View: Shade Textured".into(), Action::SetShade(Shade::Textured)),
        ("View: Shade Flat".into(), Action::SetShade(Shade::Flat)),
        ("View: Lit Preview".into(), Action::SetShade(Shade::Lit)),
        ("View: Shade Wireframe".into(), Action::SetShade(Shade::Wireframe)),
        ("View: Set Cordon from Selection".into(), Action::SetCordonFromSelection),
        ("View: Toggle Cordon".into(), Action::ToggleCordon),
        ("View: Clear Cordon".into(), Action::ClearCordon),
        ("File: Import Hammer .vmf".into(), Action::ImportVmf),
        ("File: Import Model as Mesh (.bbmodel)".into(), Action::ImportModel(ModelImport::Mesh)),
        ("File: Import Model as Brushes (.bbmodel)".into(), Action::ImportModel(ModelImport::Brushes)),
        ("File: Place Model Prop (.bbmodel, .glb)".into(), Action::ImportModel(ModelImport::Prop)),
        ("File: Export .map (cordon only)".into(), Action::ExportQuakeMapCordon),
        ("File: New Tab".into(), Action::NewTab),
        ("File: Close Tab".into(), Action::CloseTab),
        ("File: Next Tab".into(), Action::NextTab),
        ("Help: Keyboard Shortcuts".into(), Action::ShowKeymap),
        ("Godot: Reload Models".into(), Action::ReloadModels),
        ("Group: Duplicate Linked".into(), Action::DuplicateLinked),
        ("Group: Unlink".into(), Action::UnlinkGroups),
        ("Mesh: Edit Mode".into(), Action::EditMesh),
        ("Mesh: Convert Brushes to Mesh".into(), Action::ConvertToMesh),
        ("Mesh: Convert Meshes to Brushes".into(), Action::ConvertToBrushes),
        ("Mesh: Join".into(), Action::JoinMeshes),
        ("Terrain: Create Terrain".into(), Action::ShowTerrainDialog),
        ("Terrain: Flatten".into(), Action::TerrainFlatten),
        ("Terrain: Auto Paint Layers".into(), Action::TerrainAutoPaint),
        ("Scatter: New Set on a New Layer".into(), Action::NewScatterSet),
        ("Scatter: Fill Targets".into(), Action::ScatterFill),
        ("Scatter: Palette".into(), Action::ShowScatterPalette),
        ("Scatter: Install Nature Models".into(), Action::InstallNatureModels),
        ("Scatter: Convert Selected Sets to Entities".into(), Action::ScatterToEntities),
        ("Texture: Set Blend Material".into(), Action::SetBlendMaterial),
        ("Texture: Clear Blend Material".into(), Action::ClearBlendMaterial),
        ("Gameplay: Make Lift".into(), Action::MakePlatform),
        ("Gameplay: Link Selected Entities".into(), Action::ShowLinkDialog),
        ("Help: Entity and Code Reference".into(), Action::ShowReference),
        ("Edit: Repeat Last".into(), Action::RepeatLast),
        ("Group: Open Group".into(), Action::OpenGroup),
        ("Brush: Box from Last Bounds".into(), Action::CreateBrushFromBounds),
        ("Godot: Reload Materials".into(), Action::ReloadMaterials),
        ("Texture: Tool".into(), Action::SetTool(ToolKind::Texture)),
        ("Texture: Align to 3D View".into(), Action::AlignTextureToView),
        ("Texture: Reset Alignment".into(), Action::ResetTexture),
        ("Texture: Copy Material and Alignment".into(), Action::CopyAlignment),
        ("Texture: Paste Alignment".into(), Action::PasteAlignment),
        ("Texture: Toggle Treat as One".into(), Action::ToggleTreatAsOne),
        ("Texture: Hotspot Editor".into(), Action::ShowHotspotEditor),
    ];
    for j in gt_geom::Justify::ALL {
        out.push((format!("Texture: Justify {}", j.label()), Action::Justify(j)));
    }
    for d in [0.125, 0.25, 0.5, 1.0, 2.0, 4.0] {
        out.push((format!("Texture: Texel Density {d}"), Action::TexelDensity(d)));
    }
    for k in crate::texture_ops::MeshUvKind::ALL {
        out.push((format!("Mesh UVs: {}", k.label()), Action::MeshUv(k)));
    }
    for preset in gt_doc::scatter::PRESETS {
        out.push((format!("Scatter: Preset {preset}"), Action::ScatterPreset(preset.to_string())));
    }
    for class in crate::volume_tool::VOLUME_CLASSES {
        out.push((format!("Gameplay: {class} Around Selection"), Action::VolumeAroundSelection(class.to_string())));
    }
    {
        use crate::entity_wizards::{DoorKind, HingeSide, SlideDirection};
        for (label, kind) in [
            ("Hinged Door (left hinge)", DoorKind::Hinged { side: HingeSide::Left, angle: 95.0 }),
            ("Hinged Door (right hinge)", DoorKind::Hinged { side: HingeSide::Right, angle: 95.0 }),
            ("Sliding Door (up)", DoorKind::Sliding { direction: SlideDirection::Up, lip: 4.0 }),
            ("Sliding Door (sideways)", DoorKind::Sliding { direction: SlideDirection::Left, lip: 4.0 }),
        ] {
            out.push((format!("Gameplay: {label} with Trigger"), Action::MakeDoor { kind: kind.clone(), trigger: true }));
            out.push((format!("Gameplay: {label}"), Action::MakeDoor { kind, trigger: false }));
        }
    }
    for op in MeshOp::ALL {
        out.push((format!("Mesh: {}", op.label()), Action::MeshOp(op)));
    }
    for n in 1..=9u8 {
        out.push((format!("Camera: Store Bookmark {n}"), Action::StoreCamera(n)));
        out.push((format!("Camera: Recall Bookmark {n}"), Action::RecallCamera(n)));
    }
    for t in ToolKind::all() {
        out.push((format!("Tool: {}", t.label()), Action::SetTool(t)));
    }
    for axis in 0..3 {
        let a = commands::axis_name(axis);
        out.push((format!("Transform: Rotate {a} +90"), Action::Rotate { axis, degrees: 90.0 }));
        out.push((format!("Transform: Rotate {a} -90"), Action::Rotate { axis, degrees: -90.0 }));
        out.push((format!("Transform: Flip {a}"), Action::Flip { axis }));
    }
    for def in state.game.solid_entities() {
        out.push((format!("Brush Entity: {}", def.classname), Action::CreateBrushEntity(def.classname.clone())));
    }
    for def in state.game.point_entities() {
        out.push((format!("Entity: Place {}", def.classname), Action::CreatePointEntity { classname: def.classname.clone(), at: None }));
    }
    for layer in &state.doc.map.layers {
        if let Some(n) = state.doc.map.get(*layer) {
            out.push((format!("Layer: Move Selection to {}", n.name()), Action::MoveToLayer(*layer)));
        }
    }
    out
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
        let mut matches: Vec<(i32, String, Action)> =
            palette_entries(state).into_iter().filter_map(|(label, action)| fuzzy_score(&self.query, &label).map(|s| (s, label, action))).collect();
        matches.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
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
            color: egui::Color32::from_black_alpha(160),
        });
        egui::Window::new("Command Palette").title_bar(false).frame(frame).anchor(egui::Align2::CENTER_TOP, [0.0, 80.0]).fixed_size([460.0, 380.0]).show(
            ctx,
            |ui| {
                let edit = ui.add(egui::TextEdit::singleline(&mut self.query).hint_text("Type a command…").desired_width(f32::INFINITY));
                edit.request_focus();
                if edit.changed() {
                    self.selected = 0;
                }
                ui.separator();
                egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                    for (i, (_, label, action)) in matches.iter().enumerate().take(200) {
                        let shortcut = commands::shortcut_text(ctx, &state.prefs, action).unwrap_or_default();
                        let resp = ui.horizontal(|ui| {
                            let r = ui.selectable_label(i == self.selected, label);
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
            },
        );
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
}

impl ShapeKind {
    const ALL: [ShapeKind; 14] = [
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
    ];

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
}

impl Default for ShapeDialog {
    fn default() -> Self {
        Self { open: false, kind: ShapeKind::Cylinder, sides: 16, thickness: 16.0, steps: 8, as_mesh: false, smooth: true }
    }
}

impl ShapeDialog {
    pub fn generate(&self, bounds: &Aabb, material: &str) -> Vec<NodeKind> {
        let brushes = |list: Vec<Brush>| -> Vec<NodeKind> { list.into_iter().map(NodeKind::Brush).collect() };
        let smooth = |mut m: gt_geom::Mesh| {
            m.smooth_angle = if self.smooth { m.smooth_angle.max(40.0) } else { 0.0 };
            vec![NodeKind::Mesh(m)]
        };
        let as_mesh = self.as_mesh || self.kind.mesh_only();
        let size = bounds.size();
        match (self.kind, as_mesh) {
            (ShapeKind::Cylinder, true) => smooth(mesh_shapes::cylinder(bounds, self.sides, material)),
            (ShapeKind::Cone, true) => smooth(mesh_shapes::cone(bounds, self.sides, material)),
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
            (ShapeKind::Cylinder, false) => brushes(shapes::cylinder(bounds, self.sides, material).into_iter().collect()),
            (ShapeKind::Cone, false) => brushes(shapes::cone(bounds, self.sides, material).into_iter().collect()),
            (ShapeKind::Sphere, false) => brushes(shapes::sphere(bounds, self.sides, (self.sides / 2).max(3), material)),
            (ShapeKind::Wedge, _) => brushes(shapes::wedge(bounds, material).into_iter().collect()),
            (ShapeKind::Spike, _) => brushes(shapes::spike(bounds, material).into_iter().collect()),
            (ShapeKind::Arch, _) => brushes(shapes::arch(bounds, self.sides, self.thickness, material)),
            (ShapeKind::Pipe, _) => brushes(shapes::pipe(bounds, self.sides, self.thickness, material)),
            (ShapeKind::Stairs, _) => brushes(shapes::stairs(bounds, self.steps, material)),
            (ShapeKind::SpiralStairs, _) => brushes(shapes::spiral_stairs(bounds, self.steps, self.thickness, 1.0, material)),
        }
    }

    pub fn show(&mut self, ctx: &egui::Context, state: &mut EditorState) {
        let mut open = self.open;
        let brushes = state.doc.selection.brushes(&state.doc.map);
        let bounds = if brushes.is_empty() { state.last_bounds } else { state.doc.map.bounds_of(brushes.iter().copied()) };
        egui::Window::new("Shape Generator").open(&mut open).resizable(false).show(ctx, |ui| {
            egui::ComboBox::from_label("Shape").selected_text(format!("{:?}", self.kind)).show_ui(ui, |ui| {
                for k in ShapeKind::ALL {
                    ui.selectable_value(&mut self.kind, k, format!("{k:?}"));
                }
            });
            if matches!(
                self.kind,
                ShapeKind::Cylinder | ShapeKind::Cone | ShapeKind::Sphere | ShapeKind::Arch | ShapeKind::Pipe | ShapeKind::Torus | ShapeKind::ArchWall
            ) {
                ui.add(egui::Slider::new(&mut self.sides, 3..=96).text(if matches!(self.kind, ShapeKind::Arch | ShapeKind::ArchWall) {
                    "segments"
                } else {
                    "sides"
                }));
            }
            if matches!(self.kind, ShapeKind::Arch | ShapeKind::Pipe | ShapeKind::Torus | ShapeKind::ArchWall | ShapeKind::GableRoof | ShapeKind::SpiralStairs)
            {
                ui.add(egui::DragValue::new(&mut self.thickness).range(1.0..=4096.0).prefix(if self.kind == ShapeKind::SpiralStairs {
                    "inner radius "
                } else {
                    "thickness "
                }));
            }
            if matches!(self.kind, ShapeKind::Stairs | ShapeKind::SpiralStairs | ShapeKind::Grid) {
                ui.add(egui::Slider::new(&mut self.steps, 1..=128).text(if self.kind == ShapeKind::Grid { "divisions" } else { "steps" }));
            }
            if !self.kind.mesh_only() && matches!(self.kind, ShapeKind::Cylinder | ShapeKind::Cone | ShapeKind::Sphere) {
                ui.checkbox(&mut self.as_mesh, "Create as editable mesh");
            }
            if self.as_mesh || matches!(self.kind, ShapeKind::Torus) {
                ui.checkbox(&mut self.smooth, "Smooth shading");
            }
            let size = bounds.size();
            ui.label(format!("Bounds {} x {} x {} ({})", size.x, size.y, size.z, if brushes.is_empty() { "last brush" } else { "replaces selection" }));
            let preview = self.generate(&bounds, &state.current_material);
            ui.label(RichText::new(format!("{} object(s)", preview.len())).weak());
            if ui.add_enabled(!preview.is_empty(), egui::Button::new("Create")).clicked() {
                let parent = state.insert_parent();
                state.doc.edit("Create Shape", |m, s| {
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
        }
    }
}

/// Builds a terrain from dialog style parameters. `heightmap` values are 0..1.
pub fn make_terrain(
    origin: DVec3,
    resolution: u32,
    cell_size: f64,
    params: &TerrainGen,
    layers: &[(String, f64)],
    heightmap: Option<(&[f32], [u32; 2])>,
    auto_paint: bool,
) -> Terrain {
    let mut t = Terrain::new(origin, [resolution, resolution], cell_size, "");
    t.layers = layers.iter().filter(|(m, _)| !m.is_empty()).map(|(m, tile)| TerrainLayer { material: m.clone(), tile: *tile }).collect();
    match heightmap {
        Some((values, size)) => {
            t.import_heightmap(values, size, params.height);
            t.erode(params.erosion_iterations, cell_size * 0.9);
        }
        None => t.generate(params),
    }
    if auto_paint && t.layers.len() > 1 {
        let b = t.bounds();
        let span = (b.max.y - b.min.y).max(1.0);
        t.auto_paint(0.35, b.min.y - origin.y + span * 0.78, b.min.y - origin.y + span * 0.06);
    }
    t
}

impl TerrainDialog {
    pub fn show(&mut self, ctx: &egui::Context, state: &mut EditorState) {
        let mut open = self.open;
        egui::Window::new("Create Terrain").open(&mut open).resizable(false).show(ctx, |ui| {
            egui::Grid::new("terrain_grid").num_columns(2).show(ui, |ui| {
                ui.label("Resolution");
                egui::ComboBox::from_id_salt("terrain_res").selected_text(format!("{0} x {0}", self.resolution)).show_ui(ui, |ui| {
                    for r in [33, 65, 129, 257, 513, 1025] {
                        ui.selectable_value(&mut self.resolution, r, format!("{r} x {r}"));
                    }
                });
                ui.end_row();
                ui.label("Cell size");
                ui.add(egui::DragValue::new(&mut self.cell_size).range(4.0..=1024.0));
                ui.end_row();
                ui.label("Size");
                let side = (self.resolution - 1) as f64 * self.cell_size;
                ui.label(format!("{side} units ({:.0} m)", side / state.game.units_per_meter));
                ui.end_row();
                ui.label("Shape");
                egui::ComboBox::from_id_salt("terrain_shape").selected_text(format!("{:?}", self.params.shape)).show_ui(ui, |ui| {
                    for s in [TerrainShape::Flat, TerrainShape::Hills, TerrainShape::Mountain, TerrainShape::Island, TerrainShape::Valley, TerrainShape::Ridges]
                    {
                        ui.selectable_value(&mut self.params.shape, s, format!("{s:?}"));
                    }
                });
                ui.end_row();
                ui.label("Height");
                ui.add(egui::DragValue::new(&mut self.params.height).range(0.0..=65536.0));
                ui.end_row();
                ui.label("Feature size");
                ui.add(egui::DragValue::new(&mut self.params.feature_size).range(16.0..=65536.0));
                ui.end_row();
                ui.label("Octaves / roughness");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut self.params.octaves).range(1..=10));
                    ui.add(egui::Slider::new(&mut self.params.roughness, 0.1..=0.9));
                });
                ui.end_row();
                ui.label("Erosion");
                ui.add(egui::DragValue::new(&mut self.params.erosion_iterations).range(0..=200));
                ui.end_row();
                ui.label("Seed");
                ui.add(egui::DragValue::new(&mut self.params.seed));
                ui.end_row();
                for (i, name) in ["Base layer", "Slope layer", "Peak layer", "Low layer"].iter().enumerate() {
                    ui.label(*name);
                    ui.horizontal(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.layers[i]).desired_width(140.0));
                        if ui.button("use current").clicked() {
                            self.layers[i] = state.current_material.clone();
                        }
                        ui.add(egui::DragValue::new(&mut self.tiles[i]).range(8.0..=8192.0).prefix("tile "));
                    });
                    ui.end_row();
                }
                ui.label("Heightmap");
                ui.horizontal(|ui| {
                    ui.label(
                        self.heightmap
                            .as_ref()
                            .map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())
                            .unwrap_or_else(|| "procedural".into()),
                    );
                    if ui.button("Import PNG…").clicked() {
                        self.heightmap = rfd::FileDialog::new().add_filter("Heightmap", &["png", "tga", "bmp"]).pick_file();
                    }
                    if self.heightmap.is_some() && ui.button("clear").clicked() {
                        self.heightmap = None;
                    }
                });
                ui.end_row();
            });
            ui.checkbox(&mut self.auto_paint, "Paint layers from slope and height");
            if ui.button("Create").clicked() {
                let side = (self.resolution - 1) as f64 * self.cell_size;
                let center = state.cursor_world.map(|c| state.snap(c)).unwrap_or(DVec3::ZERO);
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
                );
                let parent = state.insert_parent();
                state.doc.edit("Create Terrain", |m, s| {
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
        egui::Window::new("Keyboard Shortcuts").open(&mut open).default_size([520.0, 520.0]).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Preset");
                egui::ComboBox::from_id_salt("keymap_preset").selected_text(state.prefs.keymap_preset.clone()).show_ui(ui, |ui| {
                    for p in commands::PRESETS {
                        ui.selectable_value(&mut state.prefs.keymap_preset, p.to_string(), p);
                    }
                });
                if ui.button("Reset overrides").clicked() {
                    state.prefs.key_overrides.clear();
                }
                ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("filter").desired_width(160.0));
            });
            if self.recording.is_some() {
                ui.label(RichText::new("Press the new shortcut (Esc cancels)").color(egui::Color32::from_rgb(255, 180, 80)));
            }
            ui.separator();
            let active = commands::shortcuts(&state.prefs);
            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("keymap").striped(true).num_columns(3).show(ui, |ui| {
                    for action in commands::bindable_actions() {
                        let label = action.label();
                        if !self.filter.is_empty() && fuzzy_score(&self.filter, &label).is_none() {
                            continue;
                        }
                        let id = action.binding_id();
                        let keys: Vec<String> = active.iter().filter(|(_, a)| *a == action).map(|(s, _)| commands::shortcut_to_text(s)).collect();
                        ui.label(label);
                        ui.label(RichText::new(if keys.is_empty() { "unbound".to_string() } else { keys.join(", ") }).monospace());
                        ui.horizontal(|ui| {
                            if ui.small_button(if self.recording.as_deref() == Some(id.as_str()) { "…" } else { "Set" }).clicked() {
                                self.recording = Some(id.clone());
                            }
                            if ui.small_button("Clear").clicked() {
                                state.prefs.key_overrides.insert(id.clone(), String::new());
                            }
                            if state.prefs.key_overrides.contains_key(&id) && ui.small_button("Default").clicked() {
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
/// Scatter palette: weighted models with their spread, scale and alignment, plus the placement rules.
#[derive(Default)]
pub struct ScatterPaletteWindow {
    pub open: bool,
    new_source: String,
}

impl ScatterPaletteWindow {
    pub fn show(&mut self, ctx: &egui::Context, state: &mut EditorState, actions: &mut Vec<Action>) {
        use crate::state::ScatterOutput;
        if !self.open {
            return;
        }
        let mut open = self.open;
        egui::Window::new("Scatter Palette").open(&mut open).default_width(640.0).show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label("Presets");
                for preset in gt_doc::scatter::PRESETS {
                    if ui.selectable_label(state.prefs.scatter.preset == preset, preset).clicked() {
                        actions.push(Action::ScatterPreset(preset.to_string()));
                    }
                }
                if ui.small_button("Install nature models").on_hover_text(gt_doc::scatter::NATURE_DIR).clicked() {
                    actions.push(Action::InstallNatureModels);
                }
            });
            let project = state.game.project_root.clone();
            let s = &mut state.prefs.scatter;
            ui.horizontal(|ui| {
                ui.label("Kind");
                for k in gt_doc::ScatterKind::ALL {
                    ui.selectable_value(&mut s.kind, k, k.label());
                }
                ui.separator();
                ui.label("Output");
                ui.selectable_value(&mut s.output, ScatterOutput::Set, "scatter layer");
                ui.selectable_value(&mut s.output, ScatterOutput::Entities, "entities");
                if s.output == ScatterOutput::Entities {
                    ui.add(egui::TextEdit::singleline(&mut s.prop_class).desired_width(90.0)).on_hover_text("Class used for models");
                }
            });
            ui.separator();
            let mut remove = None;
            egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                egui::Grid::new("palette_items").num_columns(9).striped(true).show(ui, |ui| {
                    for h in ["model", "weight", "scale", "", "spread", "align", "tilt", "sink", ""] {
                        ui.label(RichText::new(h).strong());
                    }
                    ui.end_row();
                    for (k, item) in s.palette.iter_mut().enumerate() {
                        let exists = project.as_ref().is_none_or(|root| {
                            crate::scatter_tool::is_classname(&item.source) || item.source.strip_prefix("res://").is_none_or(|rel| root.join(rel).is_file())
                        });
                        let label = if exists { RichText::new(item.label()) } else { RichText::new(item.label()).color(egui::Color32::from_rgb(255, 130, 90)) };
                        ui.label(label).on_hover_text(if exists {
                            item.source.clone()
                        } else {
                            format!("{} is missing, install the nature models or fix the path", item.source)
                        });
                        ui.add(egui::DragValue::new(&mut item.weight).range(0.0..=100.0).speed(0.05));
                        ui.add(egui::DragValue::new(&mut item.scale[0]).range(0.01..=50.0).speed(0.01));
                        ui.add(egui::DragValue::new(&mut item.scale[1]).range(0.01..=50.0).speed(0.01));
                        ui.add(egui::DragValue::new(&mut item.spacing).range(0.0..=4096.0)).on_hover_text("Minimum distance to other instances");
                        ui.add(egui::Slider::new(&mut item.align, 0.0..=1.0).show_value(false)).on_hover_text("Upright to aligned with the surface");
                        ui.add(egui::DragValue::new(&mut item.tilt).range(0.0..=90.0).suffix("°"));
                        ui.add(egui::DragValue::new(&mut item.sink).range(-256.0..=256.0));
                        if ui.small_button("×").clicked() {
                            remove = Some(k);
                        }
                        ui.end_row();
                    }
                });
            });
            if let Some(k) = remove {
                s.palette.remove(k);
                s.preset.clear();
            }
            ui.horizontal(|ui| {
                if ui.button("Add models…").on_hover_text("Pick several .bbmodel, .glb, .gltf or .tscn files at once").clicked() {
                    let mut dialog = rfd::FileDialog::new().add_filter("Models and scenes", &["bbmodel", "glb", "gltf", "tscn", "scn"]);
                    if let Some(root) = &project {
                        dialog = dialog.set_directory(root);
                    }
                    if let Some(paths) = dialog.pick_files() {
                        let items = crate::scatter_tool::items_from_paths(state, &paths);
                        let s = &mut state.prefs.scatter;
                        for item in items {
                            if !s.palette.iter().any(|p| p.source == item.source) {
                                s.palette.push(item);
                            }
                        }
                        s.preset.clear();
                    }
                }
                ui.add(egui::TextEdit::singleline(&mut self.new_source).hint_text("res:// model or classname").desired_width(200.0));
                if ui.button("Add").clicked() && !self.new_source.trim().is_empty() {
                    let s = &mut state.prefs.scatter;
                    s.palette.push(gt_doc::ScatterItem::new(self.new_source.trim()));
                    s.preset.clear();
                    self.new_source.clear();
                }
                if ui.button("Add selected props").clicked() {
                    let sources: Vec<String> = state
                        .doc
                        .selection
                        .nodes
                        .iter()
                        .filter_map(|id| state.doc.map.entity(*id))
                        .map(|e| e.property("model").map(str::to_string).unwrap_or_else(|| e.classname.clone()))
                        .collect();
                    let s = &mut state.prefs.scatter;
                    for src in sources {
                        if !s.palette.iter().any(|p| p.source == src) {
                            s.palette.push(gt_doc::ScatterItem::new(src));
                        }
                    }
                }
            });
            ui.separator();
            let s = &mut state.prefs.scatter;
            egui::Grid::new("scatter_rules").num_columns(2).show(ui, |ui| {
                ui.label("Brush radius");
                ui.add(egui::DragValue::new(&mut s.radius).range(8.0..=16384.0));
                ui.end_row();
                ui.label("Density");
                ui.add(egui::DragValue::new(&mut s.rules.density).range(0.01..=64.0).speed(0.05))
                    .on_hover_text("Attempts per 64 x 64 units, spread limits how many fit");
                ui.end_row();
                ui.label("Slope");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut s.rules.slope[0]).range(0.0..=90.0).suffix("°"));
                    ui.add(egui::DragValue::new(&mut s.rules.slope[1]).range(0.0..=90.0).suffix("°"));
                });
                ui.end_row();
                ui.label("Height");
                ui.horizontal(|ui| {
                    let mut limited = s.rules.height.is_some();
                    if ui.checkbox(&mut limited, "limit").changed() {
                        s.rules.height = limited.then_some([-1024.0, 4096.0]);
                    }
                    if let Some(h) = &mut s.rules.height {
                        ui.add(egui::DragValue::new(&mut h[0]));
                        ui.add(egui::DragValue::new(&mut h[1]));
                    }
                });
                ui.end_row();
                ui.label("Edge falloff");
                ui.add(egui::Slider::new(&mut s.rules.falloff, 0.0..=1.0));
                ui.end_row();
                ui.label("Erase amount");
                ui.add(egui::Slider::new(&mut s.erase_amount, 0.05..=1.0));
                ui.end_row();
                ui.label("");
                ui.checkbox(&mut s.rules.only_targets, "only paint on the set's target surfaces");
                ui.end_row();
                ui.label("");
                ui.checkbox(&mut s.avoid_other_sets, "keep spread against other scatter sets");
                ui.end_row();
                ui.label("");
                ui.checkbox(&mut s.erase_palette_only, "erase only the palette entries above");
                ui.end_row();
            });
            ui.horizontal(|ui| {
                if ui.button("New set on a new layer").clicked() {
                    actions.push(Action::NewScatterSet);
                }
                if ui.button("Fill targets").clicked() {
                    actions.push(Action::ScatterFill);
                }
                if ui.button("Scatter tool").clicked() {
                    actions.push(Action::SetTool(ToolKind::Scatter));
                }
            });
        });
        self.open = open;
    }
}

/// Connects an output of one selected entity to an input of the other.
#[derive(Default)]
pub struct LinkDialog {
    pub open: bool,
    from: Option<gt_core::NodeId>,
    to: Option<gt_core::NodeId>,
    output: String,
    input: String,
    parameter: String,
    delay: f64,
}

impl LinkDialog {
    pub fn open_for(&mut self, state: &EditorState) {
        let entities: Vec<gt_core::NodeId> = state.doc.selection.nodes.iter().copied().filter(|id| state.doc.map.entity(*id).is_some()).collect();
        if entities.len() != 2 {
            return;
        }
        self.from = Some(entities[0]);
        self.to = Some(entities[1]);
        let (outs, ins) = crate::entity_wizards::link_options(state, entities[0], entities[1]);
        self.output = outs.first().cloned().unwrap_or_default();
        self.input = ins.first().cloned().unwrap_or_default();
        self.parameter.clear();
        self.delay = 0.0;
        self.open = true;
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
        egui::Window::new("Link Entities").open(&mut open).resizable(false).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&from_name).strong());
                if ui.small_button("⇄").on_hover_text("Swap").clicked() {
                    std::mem::swap(&mut self.from, &mut self.to);
                }
                ui.label(RichText::new(&to_name).strong());
            });
            egui::Grid::new("link_grid").num_columns(2).show(ui, |ui| {
                ui.label("When");
                egui::ComboBox::from_id_salt("link_out").selected_text(&self.output).show_ui(ui, |ui| {
                    for o in outs {
                        ui.selectable_value(&mut self.output, o.clone(), o);
                    }
                });
                ui.end_row();
                ui.label("call");
                egui::ComboBox::from_id_salt("link_in").selected_text(&self.input).show_ui(ui, |ui| {
                    for i in ins {
                        ui.selectable_value(&mut self.input, i.clone(), i);
                    }
                });
                ui.end_row();
                ui.label("parameter");
                ui.text_edit_singleline(&mut self.parameter);
                ui.end_row();
                ui.label("delay");
                ui.add(egui::DragValue::new(&mut self.delay).range(0.0..=3600.0).speed(0.05).suffix(" s"));
                ui.end_row();
            });
            apply = ui.button("Link").clicked();
        });
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

    #[test]
    fn fuzzy_ranks_word_starts() {
        assert!(fuzzy_score("csg", "Brush: CSG Subtract").is_some());
        assert!(fuzzy_score("xyz", "Brush: CSG Subtract").is_none());
        let a = fuzzy_score("sub", "Brush: CSG Subtract").unwrap();
        let b = fuzzy_score("sub", "Select: Same Material but").unwrap_or(i32::MIN);
        assert!(a > b);
    }

    #[test]
    fn palette_ranks_grid_larger_first() {
        let state = EditorState::new(crate::state::Prefs::default());
        let mut matches: Vec<(i32, String, Action)> =
            palette_entries(&state).into_iter().filter_map(|(label, action)| fuzzy_score("grid larger", &label).map(|s| (s, label, action))).collect();
        matches.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        assert_eq!(matches[0].2, Action::GridUp);
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
    fn terrain_from_parameters() {
        let params = TerrainGen { shape: TerrainShape::Island, height: 300.0, ..Default::default() };
        let layers = vec![("grass".to_string(), 256.0), ("rock".to_string(), 128.0)];
        let t = make_terrain(DVec3::ZERO, 33, 32.0, &params, &layers, None, true);
        assert_eq!(t.layers.len(), 2);
        assert_eq!(t.splat.len(), 33 * 33 * 4);
        let hm: Vec<f32> = (0..16).map(|i| i as f32 / 15.0).collect();
        let from_image = make_terrain(DVec3::ZERO, 17, 32.0, &TerrainGen { erosion_iterations: 0, ..params }, &layers, Some((&hm, [4, 4])), false);
        assert!((from_image.height(16, 16) - 300.0).abs() < 1e-3);
    }
}
