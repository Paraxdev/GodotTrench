use egui::{Color32, RichText, Ui};
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use gt_render::Renderer;

use crate::CliArgs;
use crate::camera::ViewKind;
use crate::commands::{self, Action, ModelImport};
use crate::dialogs::{CommandPalette, KeymapWindow, LinkDialog, ScatterPaletteWindow, ShapeDialog, TerrainDialog};
use crate::mcp::tools::{Deferred, InputScript};
use crate::mcp::{McpHost, ToolExecutor, transport};
use crate::mesh_tool::MeshOp;
use crate::panels::{self, PanelState};
use crate::scene::SceneCache;
use crate::state::{EditorState, Prefs, Shade};
use crate::tools::ToolKind;
use crate::toolset::ToolSet;
use crate::viewport::{ViewCtx, Viewport};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    View(usize),
    Outliner,
    Inspector,
    Materials,
    Entities,
    History,
    Issues,
    Uv,
    Reference,
}

pub struct App {
    pub(crate) state: EditorState,
    pub(crate) renderer: Renderer,
    pub(crate) scene: SceneCache,
    pub(crate) viewports: Vec<Viewport>,
    dock: DockState<Tab>,
    panels: PanelState,
    pub(crate) tools: ToolSet,
    actions: Vec<Action>,
    pub(crate) project_generation: u64,
    confirm_close: bool,
    allow_close: bool,
    show_prefs: bool,
    title: String,
    pub(crate) mcp: Option<McpHost>,
    mcp_http_addr: Option<std::net::SocketAddr>,
    pub(crate) input_script: Option<InputScript>,
    pub(crate) deferred: Vec<Deferred>,
    palette: CommandPalette,
    shape_dialog: ShapeDialog,
    terrain_dialog: TerrainDialog,
    keymap: KeymapWindow,
    pub(crate) hotspot_editor: crate::hotspot_editor::HotspotEditor,
    scatter_palette: ScatterPaletteWindow,
    link_dialog: LinkDialog,
    keep_prefs: bool,
}

fn default_dock() -> DockState<Tab> {
    let mut dock = DockState::new(vec![Tab::View(0)]);
    let surface = dock.main_surface_mut();
    let [center, _right] = surface.split_right(NodeIndex::root(), 0.78, vec![Tab::Inspector, Tab::Entities, Tab::Uv]);
    let [center, _left] = surface.split_left(center, 0.2, vec![Tab::Outliner, Tab::History, Tab::Issues]);
    let [views, _bottom] = surface.split_below(center, 0.72, vec![Tab::Materials]);
    let [left_col, right_col] = surface.split_right(views, 0.5, vec![Tab::View(1)]);
    surface.split_below(left_col, 0.5, vec![Tab::View(2)]);
    surface.split_below(right_col, 0.5, vec![Tab::View(3)]);
    dock
}

impl App {
    pub fn new(cc: &eframe::CreationContext, args: CliArgs) -> Self {
        let render_state = cc.wgpu_render_state.as_ref().expect("GodotTrench requires the wgpu renderer");
        let prefs: Prefs = if args.default_prefs { Prefs::default() } else { cc.storage.and_then(|s| eframe::get_value(s, "prefs")).unwrap_or_default() };
        let keep_prefs = !args.default_prefs;
        let mut state = EditorState::new(prefs);
        let project = args.project.clone().or_else(|| state.prefs.recent_projects.first().cloned());
        if let Some(root) = project.as_deref().and_then(gt_formats::game::find_project_root) {
            state.load_project(&root);
        }

        let http_port = args.mcp_http.or(state.prefs.mcp_http.then_some(state.prefs.mcp_port));
        let mut mcp = None;
        let mut mcp_http_addr = None;
        if args.mcp_stdio || http_port.is_some() {
            let host = McpHost::new(cc.egui_ctx.clone());
            let exec: std::sync::Arc<dyn ToolExecutor> = std::sync::Arc::new(host.bridge.clone());
            if args.mcp_stdio {
                transport::spawn_stdio(exec.clone());
            }
            if let Some(port) = http_port {
                match transport::spawn_http(port, exec) {
                    Ok(addr) => {
                        mcp_http_addr = Some(addr);
                        eprintln!("GodotTrench MCP server listening on http://{addr}/mcp");
                    }
                    Err(e) => state.set_status(format!("MCP HTTP server failed on port {port}: {e}")),
                }
            }
            mcp = Some(host);
        }

        if let Some(path) = args.map
            && let Err(e) = state.open_map(&path)
        {
            state.set_status(format!("Could not open {}: {e}", path.display()));
        }
        cc.egui_ctx.set_visuals(visuals());

        let mut viewports =
            vec![Viewport::new(ViewKind::Perspective), Viewport::new(ViewKind::Top), Viewport::new(ViewKind::Front), Viewport::new(ViewKind::Side)];
        for v in &mut viewports[1..] {
            v.camera.zoom = 0.6;
        }
        Self {
            state,
            renderer: Renderer::new(render_state),
            scene: SceneCache::default(),
            viewports,
            dock: default_dock(),
            panels: PanelState::default(),
            tools: ToolSet::default(),
            actions: Vec::new(),
            project_generation: 1,
            confirm_close: false,
            allow_close: false,
            show_prefs: false,
            title: String::new(),
            mcp,
            mcp_http_addr,
            input_script: None,
            deferred: Vec::new(),
            palette: CommandPalette::default(),
            shape_dialog: ShapeDialog::default(),
            terrain_dialog: TerrainDialog::default(),
            keymap: KeymapWindow::default(),
            hotspot_editor: Default::default(),
            scatter_palette: Default::default(),
            link_dialog: Default::default(),
            keep_prefs,
        }
    }

    fn collect_input_actions(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() || self.keymap.open {
            return;
        }
        if self.tools.keys(ctx, &mut self.state) {
            return;
        }
        let events = ctx.input(|i| i.events.clone());
        for e in events {
            match e {
                egui::Event::Copy => self.actions.push(Action::Copy),
                egui::Event::Cut => self.actions.push(Action::Cut),
                egui::Event::Paste(text) => self.actions.push(Action::Paste(text)),
                _ => {}
            }
        }
        let mut shortcuts = commands::shortcuts(&self.state.prefs);
        // Most specific modifier combinations first, so Ctrl+Shift+Z is not taken by Ctrl+Z.
        shortcuts.sort_by_key(|(s, _)| std::cmp::Reverse(s.modifiers.shift as u8 + s.modifiers.command as u8 + s.modifiers.alt as u8));
        for (shortcut, action) in shortcuts {
            if ctx.input_mut(|i| i.consume_shortcut(&shortcut)) {
                self.actions.push(action);
            }
        }
    }

    /// Runs actions that need the app's viewports, tools or dialogs. Returns the actions left for `commands::execute`.
    pub(crate) fn run_app_action(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::ShowCommandPalette => self.palette.toggle(),
            Action::ShowShapeDialog => self.shape_dialog.open = true,
            Action::ShowTerrainDialog => self.terrain_dialog.open = true,
            Action::ShowKeymap => self.keymap.open = true,
            Action::ShowScatterPalette => self.scatter_palette.open = true,
            Action::ShowLinkDialog => {
                self.link_dialog.open_for(&self.state);
                if !self.link_dialog.open {
                    self.state.set_status("Select exactly two entities to link");
                }
            }
            Action::ShowReference => {
                if let Some(found) = self.dock.find_tab(&Tab::Reference) {
                    let _ = self.dock.set_active_tab(found);
                } else {
                    self.dock.push_to_focused_leaf(Tab::Reference);
                }
            }
            Action::ShowUvEditor => {
                if let Some(found) = self.dock.find_tab(&Tab::Uv) {
                    let _ = self.dock.set_active_tab(found);
                } else {
                    self.dock.push_to_focused_leaf(Tab::Uv);
                }
            }
            Action::MeshOp(op) => {
                if self.state.tool != ToolKind::Mesh {
                    self.state.tool = ToolKind::Mesh;
                    self.tools.sync(&self.state);
                }
                self.tools.mesh.prune(&self.state);
                if self.tools.mesh.selection.values().all(|s| s.is_empty()) && !matches!(op, MeshOp::ShadeSmooth | MeshOp::ShadeFlat) {
                    self.tools.mesh.select_all(&self.state);
                }
                self.tools.mesh.run(&mut self.state, op);
            }
            Action::ShowHotspotEditor => {
                let material = crate::texture_ops::target_faces(&self.state)
                    .first()
                    .and_then(|(id, f)| crate::texture_ops::face_info(&self.state.doc.map, *id, *f))
                    .map(|i| i.material)
                    .unwrap_or_else(|| self.state.current_material.clone());
                self.hotspot_editor.open_for(&material);
            }
            Action::AlignTextureToView => {
                let cam = &self.viewports[0].camera;
                let (right, up) = (cam.right(), cam.up());
                let faces = crate::texture_ops::target_faces(&self.state);
                let n = crate::texture_ops::align_to_view(&mut self.state, &faces, right, up);
                self.state.set_status(format!("Aligned {n} faces to the 3D view"));
            }
            Action::MeshUv(kind) => {
                let cam = &self.viewports[0].camera;
                let view = (cam.right(), cam.up());
                let faces = crate::texture_ops::target_faces(&self.state);
                let n = crate::texture_ops::mesh_uv(&mut self.state, &faces, kind, view);
                self.state.set_status(if n == 0 { "Select mesh faces or meshes first".to_string() } else { format!("{} UVs on {n} mesh faces", kind.label()) });
            }
            Action::StoreCamera(slot) => {
                let cam = &self.viewports[0].camera;
                let bookmark = gt_doc::map::CameraBookmark { position: cam.position, yaw: cam.yaw, pitch: cam.pitch };
                self.state.doc.map.editor.cameras.insert(slot, bookmark);
                self.state.doc.revision += 1;
                self.state.set_status(format!("Stored camera {slot}"));
            }
            Action::RecallCamera(slot) => match self.state.doc.map.editor.cameras.get(&slot).copied() {
                Some(b) => {
                    let cam = &mut self.viewports[0].camera;
                    cam.position = b.position;
                    cam.yaw = b.yaw;
                    cam.pitch = b.pitch;
                    self.state.set_status(format!("Camera {slot}"));
                }
                None => self.state.set_status(format!("Camera bookmark {slot} is empty (Ctrl+Shift+{slot} stores it)")),
            },
            other => return Some(other),
        }
        None
    }

    fn menu_bar(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let prefs = self.state.prefs.clone();
        let actions = &mut self.actions;
        let item = |ui: &mut Ui, label: &str, action: Action, actions: &mut Vec<Action>| {
            let shortcut = commands::shortcut_text(&ctx, &prefs, &action);
            let mut button = egui::Button::new(label);
            if let Some(s) = shortcut {
                button = button.shortcut_text(s);
            }
            if ui.add(button).clicked() {
                actions.push(action);
                ui.close();
            }
        };
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                item(ui, "New Map", Action::NewMap, actions);
                item(ui, "New Tab", Action::NewTab, actions);
                item(ui, "Open Map…", Action::OpenMap, actions);
                ui.menu_button("Open Recent", |ui| {
                    for path in self.state.prefs.recent_files.clone() {
                        if ui.button(path.display().to_string()).clicked() {
                            if let Err(e) = commands::open_map_in_tab(&mut self.state, &path) {
                                self.state.set_status(format!("Open failed: {e}"));
                            }
                            self.project_generation += 1;
                            ui.close();
                        }
                    }
                });
                item(ui, "Save", Action::Save, actions);
                item(ui, "Save As…", Action::SaveAs, actions);
                item(ui, "Close Tab", Action::CloseTab, actions);
                ui.separator();
                item(ui, "Import .map (TrenchBroom / Quake)…", Action::ImportQuakeMap, actions);
                item(ui, "Import .vmf (Hammer)…", Action::ImportVmf, actions);
                item(ui, "Export .map (Valve 220)…", Action::ExportQuakeMap, actions);
                item(ui, "Export .map (cordon only)…", Action::ExportQuakeMapCordon, actions);
                ui.menu_button("Import Model", |ui| {
                    item(ui, "Blockbench as Mesh…", Action::ImportModel(ModelImport::Mesh), actions);
                    item(ui, "Blockbench as Brushes…", Action::ImportModel(ModelImport::Brushes), actions);
                    item(ui, "Place Model Prop (.bbmodel, .glb)…", Action::ImportModel(ModelImport::Prop), actions);
                    item(ui, "Reload Models", Action::ReloadModels, actions);
                });
                ui.separator();
                item(ui, "Open Godot Project…", Action::OpenProject, actions);
                item(ui, "Reload Game Config", Action::ReloadProject, actions);
                ui.menu_button("Recent Projects", |ui| {
                    for path in self.state.prefs.recent_projects.clone() {
                        if ui.button(path.display().to_string()).clicked() {
                            self.state.load_project(&path);
                            self.project_generation += 1;
                            ui.close();
                        }
                    }
                });
                ui.separator();
                if ui.button("Preferences…").clicked() {
                    self.show_prefs = true;
                    ui.close();
                }
                item(ui, "Keyboard Shortcuts…", Action::ShowKeymap, actions);
            });
            ui.menu_button("Edit", |ui| {
                item(ui, "Undo", Action::Undo, actions);
                item(ui, "Redo", Action::Redo, actions);
                ui.separator();
                item(ui, "Cut", Action::Cut, actions);
                item(ui, "Copy", Action::Copy, actions);
                item(ui, "Duplicate", Action::Duplicate, actions);
                item(ui, "Delete", Action::Delete, actions);
                ui.separator();
                item(ui, "Select All", Action::SelectAll, actions);
                item(ui, "Select None", Action::SelectNone, actions);
                item(ui, "Select Inverse", Action::SelectInverse, actions);
                item(ui, "Select Touching", Action::SelectTouching, actions);
                item(ui, "Select Inside", Action::SelectInside, actions);
                item(ui, "Select Siblings", Action::SelectSiblings, actions);
                item(ui, "Select Same Material", Action::SelectSameMaterial, actions);
                ui.separator();
                item(ui, "Group", Action::Group, actions);
                item(ui, "Ungroup", Action::Ungroup, actions);
                item(ui, "Duplicate Linked", Action::DuplicateLinked, actions);
                item(ui, "Unlink Groups", Action::UnlinkGroups, actions);
                item(ui, "Close Group", Action::CloseGroup, actions);
                ui.separator();
                item(ui, "Create Prefab from Selection…", Action::CreatePrefab, actions);
                item(ui, "Insert Prefab…", Action::InsertPrefab, actions);
                item(ui, "Explode Instance", Action::ExplodeInstances, actions);
                item(ui, "Open Prefab", Action::OpenPrefab, actions);
                ui.separator();
                item(ui, "Hide Selected", Action::HideSelected, actions);
                item(ui, "Isolate Selected", Action::IsolateSelected, actions);
                item(ui, "Show All", Action::UnhideAll, actions);
                item(ui, "Lock Selected", Action::LockSelected, actions);
                item(ui, "Unlock All", Action::UnlockAll, actions);
            });
            ui.menu_button("Brush", |ui| {
                item(ui, "CSG Subtract", Action::CsgSubtract, actions);
                item(ui, "CSG Convex Merge", Action::CsgMerge, actions);
                item(ui, "CSG Intersect", Action::CsgIntersect, actions);
                item(ui, "Hollow", Action::CsgHollow, actions);
                ui.horizontal(|ui| {
                    ui.label("Hollow thickness");
                    ui.add(egui::DragValue::new(&mut self.state.hollow_thickness).range(0.125..=1024.0));
                });
                ui.separator();
                for axis in 0..3 {
                    item(ui, &format!("Rotate {} +90°", commands::axis_name(axis)), Action::Rotate { axis, degrees: 90.0 }, actions);
                    item(ui, &format!("Rotate {} -90°", commands::axis_name(axis)), Action::Rotate { axis, degrees: -90.0 }, actions);
                }
                ui.separator();
                for axis in 0..3 {
                    item(ui, &format!("Flip {}", commands::axis_name(axis)), Action::Flip { axis }, actions);
                }
                ui.separator();
                item(ui, "Snap Vertices to Grid", Action::SnapVertices, actions);
                ui.menu_button("Create Displacement", |ui| {
                    for power in [2u8, 3, 4] {
                        item(ui, &format!("Power {power} ({0}x{0})", (1 << power) + 1), Action::CreateDisplacement(power), actions);
                    }
                });
                item(ui, "Remove Displacement", Action::RemoveDisplacement, actions);
                item(ui, "Sew Displacements", Action::SewDisplacements, actions);
                item(ui, "Shape Generator…", Action::ShowShapeDialog, actions);
                item(ui, "Hotspot Texture", Action::HotspotTexture, actions);
                item(ui, "Move Brushes to World", Action::MoveToWorld, actions);
                ui.menu_button("Create Brush Entity", |ui| {
                    for def in self.state.game.solid_entities() {
                        if ui.button(&def.classname).clicked() {
                            actions.push(Action::CreateBrushEntity(def.classname.clone()));
                            ui.close();
                        }
                    }
                });
            });
            ui.menu_button("Mesh", |ui| {
                item(ui, "Edit Mesh (toggle)", Action::EditMesh, actions);
                item(ui, "Convert Brushes to Mesh", Action::ConvertToMesh, actions);
                item(ui, "Convert Meshes to Brushes", Action::ConvertToBrushes, actions);
                item(ui, "Join", Action::JoinMeshes, actions);
                ui.separator();
                for op in MeshOp::ALL {
                    item(ui, op.label(), Action::MeshOp(op), actions);
                }
            });
            ui.menu_button("Texture", |ui| {
                item(ui, "Texture Tool", Action::SetTool(ToolKind::Texture), actions);
                item(ui, "UV Editor", Action::ShowUvEditor, actions);
                item(ui, "Hotspot Editor…", Action::ShowHotspotEditor, actions);
                ui.separator();
                ui.menu_button("Justify", |ui| {
                    for j in gt_geom::Justify::ALL {
                        item(ui, j.label(), Action::Justify(j), actions);
                    }
                    let mut one = self.state.treat_as_one;
                    if ui.checkbox(&mut one, "Treat as one").changed() {
                        actions.push(Action::ToggleTreatAsOne);
                    }
                });
                item(ui, "Align to 3D View", Action::AlignTextureToView, actions);
                item(ui, "Reset Alignment", Action::ResetTexture, actions);
                item(ui, "Copy Material and Alignment", Action::CopyAlignment, actions);
                item(ui, "Paste Alignment", Action::PasteAlignment, actions);
                ui.menu_button("Texel Density", |ui| {
                    for d in [0.125, 0.25, 0.5, 1.0, 2.0, 4.0] {
                        item(ui, &format!("{d} units per pixel"), Action::TexelDensity(d), actions);
                    }
                });
                item(ui, "Hotspot Texture", Action::HotspotTexture, actions);
                ui.separator();
                ui.menu_button("Mesh UVs", |ui| {
                    for k in crate::texture_ops::MeshUvKind::ALL {
                        item(ui, k.label(), Action::MeshUv(k), actions);
                    }
                });
                ui.separator();
                item(ui, "Set Blend Material (current)", Action::SetBlendMaterial, actions);
                item(ui, "Clear Blend Material", Action::ClearBlendMaterial, actions);
                ui.separator();
                item(ui, "Reload Materials", Action::ReloadMaterials, actions);
            });
            ui.menu_button("Terrain", |ui| {
                item(ui, "Create Terrain…", Action::ShowTerrainDialog, actions);
                item(ui, "Sculpt Tool", Action::SetTool(ToolKind::Sculpt), actions);
                item(ui, "Auto Paint Layers", Action::TerrainAutoPaint, actions);
                item(ui, "Flatten", Action::TerrainFlatten, actions);
                item(ui, "Blend Tool", Action::SetTool(ToolKind::Blend), actions);
                ui.separator();
                item(ui, "Scatter Tool", Action::SetTool(ToolKind::Scatter), actions);
                item(ui, "Scatter Palette…", Action::ShowScatterPalette, actions);
                ui.menu_button("Scatter Preset", |ui| {
                    for preset in gt_doc::scatter::PRESETS {
                        item(ui, preset, Action::ScatterPreset(preset.to_string()), actions);
                    }
                });
                item(ui, "New Scatter Set (new layer)", Action::NewScatterSet, actions);
                item(ui, "Fill Scatter Targets", Action::ScatterFill, actions);
                item(ui, "Scatter Sets to Entities", Action::ScatterToEntities, actions);
                item(ui, "Install Nature Models", Action::InstallNatureModels, actions);
            });
            ui.menu_button("Gameplay", |ui| {
                use crate::entity_wizards::{DoorKind, HingeSide, SlideDirection};
                ui.label(RichText::new("From the selected brushes").weak());
                item(ui, "Hinged Door, left hinge", Action::MakeDoor { kind: DoorKind::Hinged { side: HingeSide::Left, angle: 95.0 }, trigger: true }, actions);
                item(
                    ui,
                    "Hinged Door, right hinge",
                    Action::MakeDoor { kind: DoorKind::Hinged { side: HingeSide::Right, angle: 95.0 }, trigger: true },
                    actions,
                );
                item(ui, "Sliding Door, up", Action::MakeDoor { kind: DoorKind::Sliding { direction: SlideDirection::Up, lip: 4.0 }, trigger: true }, actions);
                item(
                    ui,
                    "Sliding Door, sideways",
                    Action::MakeDoor { kind: DoorKind::Sliding { direction: SlideDirection::Left, lip: 4.0 }, trigger: true },
                    actions,
                );
                item(ui, "Lift / Moving Platform", Action::MakePlatform, actions);
                ui.menu_button("Volume Around Selection", |ui| {
                    for class in crate::volume_tool::VOLUME_CLASSES {
                        item(ui, class, Action::VolumeAroundSelection(class.to_string()), actions);
                    }
                });
                ui.separator();
                item(ui, "Volume Tool", Action::SetTool(ToolKind::Volume), actions);
                item(ui, "Link Two Selected Entities…", Action::ShowLinkDialog, actions);
                ui.separator();
                ui.label(RichText::new("Place at the cursor").weak());
                for class in [
                    "info_spawner",
                    "logic_call",
                    "logic_relay",
                    "logic_timer",
                    "logic_counter",
                    "logic_auto",
                    "logic_debug",
                    "path_corner",
                    "info_teleport_destination",
                ] {
                    if self.state.game.entity(class).is_some() {
                        item(ui, class, Action::CreatePointEntity { classname: class.to_string(), at: None }, actions);
                    }
                }
                ui.separator();
                item(ui, "Entity and Code Reference", Action::ShowReference, actions);
            });
            ui.menu_button("Tools", |ui| {
                for t in ToolKind::all() {
                    item(ui, &format!("{} Tool", t.label()), Action::SetTool(t), actions);
                }
            });
            ui.menu_button("View", |ui| {
                item(ui, "Focus Selection", Action::FocusSelection, actions);
                item(ui, "Cycle Shading (textured, flat, lit, wireframe)", Action::ToggleTextured, actions);
                item(ui, "Wireframe", Action::SetShade(Shade::Wireframe), actions);
                item(ui, "Lit Preview", Action::SetShade(Shade::Lit), actions);
                ui.separator();
                item(ui, "Set Cordon from Selection", Action::SetCordonFromSelection, actions);
                item(ui, "Toggle Cordon", Action::ToggleCordon, actions);
                item(ui, "Clear Cordon", Action::ClearCordon, actions);
                ui.menu_button("Camera Bookmarks", |ui| {
                    for n in 1..=9u8 {
                        ui.horizontal(|ui| {
                            let stored = self.state.doc.map.editor.cameras.contains_key(&n);
                            if ui.add_enabled(stored, egui::Button::new(format!("Go to {n}"))).clicked() {
                                actions.push(Action::RecallCamera(n));
                                ui.close();
                            }
                            if ui.button(format!("Store {n}")).clicked() {
                                actions.push(Action::StoreCamera(n));
                                ui.close();
                            }
                        });
                    }
                });
                if ui.button("Reset Layout").clicked() {
                    self.dock = default_dock();
                    ui.close();
                }
                ui.separator();
                for (tab, label) in [
                    (Tab::Outliner, "Outliner"),
                    (Tab::Inspector, "Inspector"),
                    (Tab::Materials, "Materials"),
                    (Tab::Entities, "Entities"),
                    (Tab::History, "History"),
                    (Tab::Issues, "Issues"),
                    (Tab::Uv, "UV Editor"),
                    (Tab::Reference, "Reference"),
                ] {
                    if ui.button(label).clicked() {
                        if let Some(found) = self.dock.find_tab(&tab) {
                            let _ = self.dock.set_active_tab(found);
                        } else {
                            self.dock.push_to_focused_leaf(tab);
                        }
                        ui.close();
                    }
                }
            });
            ui.menu_button("Godot", |ui| {
                item(ui, "Open Project in Godot Editor", Action::OpenGodotEditor, actions);
                item(ui, "Run Project", Action::RunGodotProject, actions);
                item(ui, "Reload Game Config", Action::ReloadProject, actions);
            });
            ui.menu_button("Help", |ui| {
                item(ui, "Command Palette", Action::ShowCommandPalette, actions);
                item(ui, "Keyboard Shortcuts", Action::ShowKeymap, actions);
                ui.separator();
                ui.label("GodotTrench: brush and mesh level editor for Godot");
                ui.label("3D: RMB look + WASD fly (Q/E down/up), MMB pan, Alt+LMB orbit");
                ui.label("2D: RMB/MMB pan, wheel zoom, drag edges to resize");
                ui.label("Drag empty space to draw a brush, drag selection to move (Alt vertical, Ctrl duplicate)");
                ui.label("Shift+click selects faces, Shift+drag a face resizes, Ctrl+Shift+drag extrudes");
                ui.label("Tab edits meshes Blender style: 1/2/3 modes, G/R/S, E extrude, I inset, Ctrl+R loop cut, K knife");
            });
        });
    }

    fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            for t in ToolKind::all() {
                if ui.selectable_label(self.state.tool == t, t.label()).clicked() {
                    self.actions.push(Action::SetTool(t));
                }
            }
            ui.separator();
            ui.label("Grid");
            egui::ComboBox::from_id_salt("grid").selected_text(format!("{}", self.state.grid)).width(60.0).show_ui(ui, |ui| {
                for g in [0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0, 512.0, 1024.0] {
                    ui.selectable_value(&mut self.state.grid, g, format!("{g}"));
                }
            });
            ui.checkbox(&mut self.state.snap, "Snap");
            ui.checkbox(&mut self.state.uv_lock, "UV lock");
            egui::ComboBox::from_id_salt("shade").selected_text(self.state.prefs.shade.label()).width(70.0).show_ui(ui, |ui| {
                for s in Shade::ALL {
                    ui.selectable_value(&mut self.state.prefs.shade, s, s.label());
                }
            });
            ui.separator();
            match self.state.tool {
                ToolKind::Sculpt | ToolKind::Paint => {
                    use gt_doc::terrain::SculptMode;
                    if self.state.tool == ToolKind::Sculpt {
                        egui::ComboBox::from_id_salt("sculpt_mode").selected_text(format!("{:?}", self.state.sculpt.mode)).width(90.0).show_ui(ui, |ui| {
                            for m in SculptMode::ALL {
                                ui.selectable_value(&mut self.state.sculpt.mode, m, format!("{m:?}"));
                            }
                        });
                        if self.state.sculpt.mode == SculptMode::PaintLayer {
                            ui.add(egui::DragValue::new(&mut self.state.sculpt.layer).range(0..=3).prefix("layer "));
                        }
                        if self.state.sculpt.mode == SculptMode::Terrace {
                            ui.add(egui::DragValue::new(&mut self.state.sculpt.terrace_step).range(1.0..=1024.0).prefix("step "));
                        }
                    } else {
                        ui.color_edit_button_rgba_unmultiplied(&mut self.state.paint_color);
                    }
                    ui.add(egui::DragValue::new(&mut self.state.sculpt.radius).range(1.0..=8192.0).prefix("radius "));
                    ui.add(egui::DragValue::new(&mut self.state.sculpt.strength).range(0.01..=256.0).speed(0.1).prefix("strength "));
                    ui.separator();
                }
                ToolKind::Texture => {
                    ui.checkbox(&mut self.state.treat_as_one, "treat as one");
                    for j in gt_geom::Justify::ALL {
                        if ui.small_button(j.label()).on_hover_text("Justify the selected faces").clicked() {
                            self.actions.push(Action::Justify(j));
                        }
                    }
                    if ui.small_button("View").on_hover_text("Align to the 3D view").clicked() {
                        self.actions.push(Action::AlignTextureToView);
                    }
                    if ui.small_button("Hotspot").clicked() {
                        self.actions.push(Action::HotspotTexture);
                    }
                    ui.separator();
                }
                ToolKind::Scatter => {
                    let sets: Vec<(gt_core::NodeId, String)> =
                        self.state.doc.map.scatters().map(|(id, s)| (id, format!("{} ({})", s.name, s.instances.len()))).collect();
                    let active = crate::scatter_tool::active_set(&self.state);
                    let current = active.and_then(|a| sets.iter().find(|(id, _)| *id == a)).map(|(_, n)| n.clone()).unwrap_or_else(|| "new layer".into());
                    egui::ComboBox::from_id_salt("scatter_set").selected_text(current).width(140.0).show_ui(ui, |ui| {
                        if ui.selectable_label(active.is_none(), "new set on a new layer").clicked() {
                            self.actions.push(Action::NewScatterSet);
                        }
                        for (id, name) in &sets {
                            if ui.selectable_label(active == Some(*id), name).clicked() {
                                self.actions.push(Action::ActivateScatter(*id));
                            }
                        }
                    });
                    let preset = if self.state.prefs.scatter.preset.is_empty() { "custom".to_string() } else { self.state.prefs.scatter.preset.clone() };
                    egui::ComboBox::from_id_salt("scatter_preset").selected_text(preset).width(90.0).show_ui(ui, |ui| {
                        for p in gt_doc::scatter::PRESETS {
                            if ui.selectable_label(self.state.prefs.scatter.preset == p, p).clicked() {
                                self.actions.push(Action::ScatterPreset(p.to_string()));
                            }
                        }
                    });
                    let s = &mut self.state.prefs.scatter;
                    ui.add(egui::DragValue::new(&mut s.radius).range(8.0..=16384.0).prefix("radius "));
                    ui.add(egui::DragValue::new(&mut s.rules.density).range(0.01..=64.0).speed(0.05).prefix("density "));
                    ui.add(egui::DragValue::new(&mut s.rules.slope[1]).range(0.0..=90.0).prefix("max slope ").suffix("°"));
                    ui.checkbox(&mut s.rules.only_targets, "target only");
                    if ui.button("Palette…").clicked() {
                        self.actions.push(Action::ShowScatterPalette);
                    }
                    if ui.button("Fill").on_hover_text("Fill the set's target surfaces").clicked() {
                        self.actions.push(Action::ScatterFill);
                    }
                    ui.separator();
                }
                ToolKind::Blend => {
                    use gt_doc::blend::{BlendMode, Falloff};
                    let b = &mut self.state.blend;
                    egui::ComboBox::from_id_salt("blend_mode").selected_text(b.mode.label()).width(80.0).show_ui(ui, |ui| {
                        for m in BlendMode::ALL {
                            ui.selectable_value(&mut b.mode, m, m.label());
                        }
                    });
                    egui::ComboBox::from_id_salt("blend_falloff").selected_text(b.falloff.label()).width(80.0).show_ui(ui, |ui| {
                        for f in Falloff::ALL {
                            ui.selectable_value(&mut b.falloff, f, f.label());
                        }
                    });
                    ui.add(egui::DragValue::new(&mut b.layer).range(0..=3).prefix("layer "));
                    ui.add(egui::DragValue::new(&mut b.radius).range(2.0..=16384.0).prefix("radius "));
                    ui.add(egui::Slider::new(&mut b.strength, 0.01..=1.0).text("strength"));
                    match b.mode {
                        BlendMode::Slope => {
                            ui.add(egui::DragValue::new(&mut b.slope[0]).range(0.0..=90.0).suffix("°"));
                            ui.add(egui::DragValue::new(&mut b.slope[1]).range(0.0..=90.0).suffix("°"));
                        }
                        BlendMode::Height => {
                            ui.add(egui::DragValue::new(&mut b.height[0]).prefix("from "));
                            ui.add(egui::DragValue::new(&mut b.height[1]).prefix("to "));
                        }
                        BlendMode::Noise => {
                            ui.add(egui::DragValue::new(&mut b.noise_scale).range(4.0..=16384.0).prefix("size "));
                        }
                        _ => {}
                    }
                    if ui.button("Blend material").on_hover_text("Use the current material as the second texture of the selected faces").clicked() {
                        self.actions.push(Action::SetBlendMaterial);
                    }
                    ui.separator();
                }
                ToolKind::Volume => {
                    let p = &mut self.state.prefs;
                    egui::ComboBox::from_id_salt("volume_class").selected_text(p.volume_class.clone()).width(140.0).show_ui(ui, |ui| {
                        for class in crate::volume_tool::VOLUME_CLASSES {
                            ui.selectable_value(&mut p.volume_class, class.to_string(), class);
                        }
                    });
                    ui.add(egui::DragValue::new(&mut p.volume_height).range(1.0..=8192.0).prefix("height "));
                    ui.separator();
                }
                ToolKind::Mesh => {
                    use crate::mesh_tool::Component;
                    for c in [Component::Vertex, Component::Edge, Component::Face] {
                        if ui.selectable_label(self.tools.mesh.component == c, c.label()).clicked() {
                            self.tools.mesh.component = c;
                        }
                    }
                    for (label, op) in [
                        ("Subdivide", MeshOp::Subdivide),
                        ("Merge", MeshOp::MergeCenter),
                        ("Fill", MeshOp::Fill),
                        ("Delete", MeshOp::Delete),
                        ("Smooth", MeshOp::ShadeSmooth),
                        ("Flat", MeshOp::ShadeFlat),
                    ] {
                        if ui.button(label).clicked() {
                            self.actions.push(Action::MeshOp(op));
                        }
                    }
                    ui.separator();
                }
                _ => {}
            }
            for (label, action) in
                [("Subtract", Action::CsgSubtract), ("Merge", Action::CsgMerge), ("Intersect", Action::CsgIntersect), ("Hollow", Action::CsgHollow)]
            {
                if ui.button(label).clicked() {
                    self.actions.push(action);
                }
            }
            ui.separator();
            let project = match &self.state.game.project_root {
                Some(p) => format!("{} ({})", self.state.game.name, p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()),
                None => "No Godot project".into(),
            };
            if ui.button(RichText::new(project).color(Color32::from_rgb(140, 190, 255))).on_hover_text("Open a Godot project folder").clicked() {
                self.actions.push(Action::OpenProject);
            }
        });
    }

    fn tab_bar(&mut self, ui: &mut Ui) {
        let (titles, active) = self.state.tab_titles();
        if titles.len() < 2 {
            return;
        }
        ui.horizontal(|ui| {
            for (i, title) in titles.iter().enumerate() {
                if ui.selectable_label(i == active, title).clicked() && i != active {
                    self.state.switch_tab(i);
                }
            }
            if ui.small_button("+").on_hover_text("New tab").clicked() {
                self.actions.push(Action::NewTab);
            }
        });
    }

    fn status_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let s = &self.state;
            let stats = self.scene.stats;
            let alpha = if s.status_age() > 6.0 { 120 } else { 230 };
            ui.label(RichText::new(&s.status).color(Color32::from_rgba_unmultiplied(230, 230, 230, alpha)));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!(
                    "{} brushes  {} meshes  {} terrains  {} entities  {} tris",
                    stats.brushes, stats.meshes, stats.terrains, stats.entities, stats.triangles
                ));
                ui.separator();
                if let Some(c) = s.cursor_world {
                    ui.monospace(format!("{:8.1} {:8.1} {:8.1}", c.x, c.y, c.z));
                }
                ui.separator();
                let sel = &s.doc.selection;
                if sel.has_faces() {
                    ui.label(format!("{} faces selected", sel.faces.len()));
                } else if !sel.nodes.is_empty() {
                    ui.label(format!("{} selected", sel.nodes.len()));
                }
                if s.doc.map.editor.cordon_enabled {
                    ui.separator();
                    ui.label(RichText::new("cordon").color(Color32::from_rgb(255, 210, 60)));
                }
                if !s.open_groups.is_empty() {
                    ui.separator();
                    ui.label(RichText::new(format!("{} open groups", s.open_groups.len())).color(Color32::from_rgb(160, 200, 255)));
                }
            });
        });
    }

    fn prefs_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_prefs;
        egui::Window::new("Preferences").open(&mut open).resizable(false).show(ctx, |ui| {
            let p = &mut self.state.prefs;
            egui::Grid::new("prefs").num_columns(2).show(ui, |ui| {
                ui.label("Fly speed");
                ui.add(egui::Slider::new(&mut p.fly_speed, 64.0..=8192.0).logarithmic(true));
                ui.end_row();
                ui.label("Look sensitivity");
                ui.add(egui::Slider::new(&mut p.look_sensitivity, 0.001..=0.02));
                ui.end_row();
                ui.label("Invert look Y");
                ui.checkbox(&mut p.invert_y, "");
                ui.end_row();
                ui.label("Grid opacity (3D)");
                ui.add(egui::Slider::new(&mut p.grid_alpha, 0.0..=1.0));
                ui.end_row();
                ui.label("Texture filtering");
                let before = p.texture_filter;
                egui::ComboBox::from_id_salt("prefs_filter").selected_text(format!("{:?}", p.texture_filter)).show_ui(ui, |ui| {
                    use crate::state::TextureFilter;
                    ui.selectable_value(&mut p.texture_filter, TextureFilter::Auto, "Auto (Godot material)");
                    ui.selectable_value(&mut p.texture_filter, TextureFilter::Nearest, "Nearest (pixel art)");
                    ui.selectable_value(&mut p.texture_filter, TextureFilter::Linear, "Linear");
                });
                if p.texture_filter != before {
                    self.state.material_reload = true;
                }
                ui.end_row();
                ui.label("Godot executable");
                ui.horizontal(|ui| {
                    let mut text = p.godot_path.to_string_lossy().into_owned();
                    if ui.add(egui::TextEdit::singleline(&mut text).hint_text("auto (GODOT env or PATH)").desired_width(220.0)).changed() {
                        p.godot_path = std::path::PathBuf::from(text);
                    }
                    if ui.button("…").clicked()
                        && let Some(f) = rfd::FileDialog::new().pick_file()
                    {
                        p.godot_path = f;
                    }
                });
                ui.end_row();
                ui.label("Autosave (minutes, 0 = off)");
                ui.add(egui::DragValue::new(&mut p.autosave_minutes).range(0.0..=60.0));
                ui.end_row();
                ui.label("Godot live link");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut p.live_link, "rebuild maps in Godot on save");
                    ui.add(egui::DragValue::new(&mut p.live_link_port).range(1024..=65535));
                });
                ui.end_row();
                ui.label("Running game hot reload");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut p.hot_reload, "reload maps in a running game");
                    ui.add(egui::DragValue::new(&mut p.hot_reload_port).range(1024..=65535));
                });
                ui.end_row();
                ui.label("Keymap preset");
                egui::ComboBox::from_id_salt("prefs_keymap").selected_text(p.keymap_preset.clone()).show_ui(ui, |ui| {
                    for preset in commands::PRESETS {
                        ui.selectable_value(&mut p.keymap_preset, preset.to_string(), preset);
                    }
                });
                ui.end_row();
                ui.label("MCP server (HTTP, localhost)");
                ui.horizontal(|ui| {
                    ui.checkbox(&mut p.mcp_http, "enable on startup");
                    ui.add(egui::DragValue::new(&mut p.mcp_port).range(1024..=65535));
                });
                ui.end_row();
            });
            match self.mcp_http_addr {
                Some(addr) => {
                    ui.label(format!("MCP running at http://{addr}/mcp"));
                    ui.code(format!("claude mcp add --transport http godottrench http://{addr}/mcp"));
                }
                None => {
                    ui.label("MCP HTTP server is not running. Enable it and restart, or launch with --mcp-http.");
                }
            }
        });
        self.show_prefs = open;
    }

    fn close_dialog(&mut self, ctx: &egui::Context) {
        let any_modified = self.state.doc.is_modified() || self.state.tabs.iter().any(|t| t.doc.is_modified());
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_close && any_modified {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
        }
        if !self.confirm_close {
            return;
        }
        egui::Modal::new(egui::Id::new("confirm_close")).show(ctx, |ui| {
            ui.heading("Unsaved changes");
            ui.label(format!("Save changes to {} before closing?", self.state.doc.title()));
            if self.state.tabs.iter().any(|t| t.doc.is_modified()) {
                ui.label("Other tabs have unsaved changes too.");
            }
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    commands::execute(&mut self.state, Action::Save, ctx);
                    if !self.state.doc.is_modified() && !self.state.tabs.iter().any(|t| t.doc.is_modified()) {
                        self.allow_close = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    self.confirm_close = false;
                }
                if ui.button("Discard").clicked() {
                    self.allow_close = true;
                    self.confirm_close = false;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                if ui.button("Cancel").clicked() {
                    self.confirm_close = false;
                }
            });
        });
    }
}

struct Tabs<'a> {
    state: &'a mut EditorState,
    renderer: &'a mut Renderer,
    scene: &'a SceneCache,
    viewports: &'a mut [Viewport],
    panels: &'a mut PanelState,
    tools: &'a mut ToolSet,
    actions: &'a mut Vec<Action>,
}

impl TabViewer for Tabs<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        match tab {
            Tab::View(i) => self.viewports.get(*i).map(|v| v.kind().label()).unwrap_or("View").into(),
            Tab::Outliner => "Outliner".into(),
            Tab::Inspector => "Inspector".into(),
            Tab::Materials => "Materials".into(),
            Tab::Entities => "Entities".into(),
            Tab::History => "History".into(),
            Tab::Issues => "Issues".into(),
            Tab::Uv => "UV Editor".into(),
            Tab::Reference => "Reference".into(),
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Tab) {
        match tab {
            Tab::View(i) => {
                let Some(vp) = self.viewports.get_mut(*i) else { return };
                let mut cx = ViewCtx { state: self.state, renderer: self.renderer, scene: self.scene, actions: self.actions, tools: self.tools };
                vp.ui(ui, &mut cx);
            }
            Tab::Outliner => panels::outliner(ui, self.state, self.panels, self.actions),
            Tab::Inspector => panels::inspector(ui, self.state, self.panels, self.actions),
            Tab::Materials => panels::material_browser(ui, self.state, self.panels, self.actions),
            Tab::Entities => panels::entity_browser(ui, self.state, self.panels, self.actions),
            Tab::History => panels::history(ui, self.state),
            Tab::Issues => panels::issues(ui, self.state, self.panels, self.actions),
            Tab::Uv => panels::uv_editor(ui, self.state, self.panels, self.actions),
            Tab::Reference => panels::reference(ui, self.state, self.panels, self.actions),
        }
    }

    fn id(&mut self, tab: &mut Tab) -> egui::Id {
        egui::Id::new(("tab", format!("{tab:?}")))
    }

    fn is_closeable(&self, tab: &Tab) -> bool {
        !matches!(tab, Tab::View(_))
    }

    fn closeable(&mut self, tab: &mut Tab) -> bool {
        !matches!(tab, Tab::View(_))
    }

    fn clear_background(&self, tab: &Tab) -> bool {
        !matches!(tab, Tab::View(_))
    }

    fn scroll_bars(&self, tab: &Tab) -> [bool; 2] {
        if matches!(tab, Tab::View(_)) { [false, false] } else { [true, true] }
    }
}

impl eframe::App for App {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        self.feed_input_script(raw_input);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.tools.sync(&self.state);
        self.process_mcp(&ctx);
        self.tools.sync(&self.state);
        self.collect_input_actions(&ctx);

        egui::Panel::top("menu").show(ui, |ui| self.menu_bar(ui));
        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));
        if !self.state.tabs.is_empty() {
            egui::Panel::top("map_tabs").show(ui, |ui| self.tab_bar(ui));
        }
        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));

        if std::mem::take(&mut self.state.material_reload) {
            let game = self.state.game.clone();
            self.state.materials.rescan(&game);
            self.project_generation += 1;
        }
        if std::mem::take(&mut self.state.scene_reset) {
            self.scene.invalidate();
            self.tools.mesh.selection.clear();
        }
        self.scene.update(&mut self.renderer, &mut self.state, self.project_generation);
        let cam = &self.viewports[0].camera;
        let focus = cam.position + cam.forward() * 1200.0;
        self.scene.update_shadows(&mut self.renderer, focus, self.state.prefs.shade == Shade::Lit);

        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
            let mut tabs = Tabs {
                state: &mut self.state,
                renderer: &mut self.renderer,
                scene: &self.scene,
                viewports: &mut self.viewports,
                panels: &mut self.panels,
                tools: &mut self.tools,
                actions: &mut self.actions,
            };
            DockArea::new(&mut self.dock).show_leaf_collapse_buttons(false).show_inside(ui, &mut tabs);
        });

        self.palette.show(&ctx, &self.state, &mut self.actions);
        self.shape_dialog.show(&ctx, &mut self.state);
        self.terrain_dialog.show(&ctx, &mut self.state);
        self.keymap.show(&ctx, &mut self.state);
        self.hotspot_editor.show(&ctx, &mut self.state, &mut self.actions);
        self.scatter_palette.show(&ctx, &mut self.state, &mut self.actions);
        self.link_dialog.show(&ctx, &mut self.state);

        for action in std::mem::take(&mut self.actions) {
            let Some(action) = self.run_app_action(action) else { continue };
            let project_before = self.state.game.project_root.clone();
            commands::execute(&mut self.state, action, &ctx);
            if self.state.game.project_root != project_before {
                self.project_generation += 1;
            }
        }
        if let Some(bounds) = self.state.focus_request.take() {
            for v in &mut self.viewports {
                v.focus(&bounds);
            }
        }

        self.prefs_window(&ctx);
        self.close_dialog(&ctx);
        self.state.tick_autosave();
        self.state.poll_live_link();
        self.finish_input_script(&ctx);

        let title = format!("{} - GodotTrench", self.state.doc.title());
        if title != self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.keep_prefs {
            eframe::set_value(storage, "prefs", &self.state.prefs);
        }
    }
}

fn visuals() -> egui::Visuals {
    let mut v = egui::Visuals::dark();
    v.panel_fill = Color32::from_rgb(30, 31, 36);
    v.window_fill = Color32::from_rgb(34, 35, 41);
    v.extreme_bg_color = Color32::from_rgb(20, 21, 25);
    v.selection.bg_fill = Color32::from_rgb(170, 90, 40);
    v.hyperlink_color = Color32::from_rgb(255, 160, 80);
    v
}
