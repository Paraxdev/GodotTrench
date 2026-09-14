use egui::{Color32, RichText, Ui};
use egui_dock::{DockArea, DockState, NodeIndex, TabViewer};
use gt_render::Renderer;

use crate::CliArgs;
use crate::camera::ViewKind;
use crate::commands::{self, Action, ModelImport};
use crate::dialogs::{CommandPalette, KeymapWindow, LinkDialog, ScatterPaletteWindow, ShapeDialog, TerrainDialog};
use crate::icons;
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

const PANEL_TABS: [Tab; 8] = [Tab::Outliner, Tab::Inspector, Tab::Materials, Tab::Entities, Tab::History, Tab::Issues, Tab::Uv, Tab::Reference];

fn tab_title(tab: Tab) -> &'static str {
    match tab {
        Tab::View(_) => "View",
        Tab::Outliner => "Outliner",
        Tab::Inspector => "Inspector",
        Tab::Materials => "Materials",
        Tab::Entities => "Entities",
        Tab::History => "History",
        Tab::Issues => "Issues",
        Tab::Uv => "UV Editor",
        Tab::Reference => "Reference",
    }
}

/// Focuses a panel tab, reopening it in the focused dock leaf when it was closed.
fn show_tab(dock: &mut DockState<Tab>, tab: Tab) {
    if let Some(found) = dock.find_tab(&tab) {
        let _ = dock.set_active_tab(found);
    } else {
        dock.push_to_focused_leaf(tab);
    }
}

/// Mesh operations grouped for the Mesh menu.
const MESH_OP_GROUPS: [(&str, &[MeshOp]); 4] = [
    ("Topology", &[MeshOp::Subdivide, MeshOp::Triangulate, MeshOp::Solidify, MeshOp::Fill, MeshOp::MergeCenter, MeshOp::MergeByDistance]),
    ("Components", &[MeshOp::SelectLinked, MeshOp::Duplicate, MeshOp::Separate, MeshOp::Dissolve, MeshOp::Delete]),
    ("Normals and Shading", &[MeshOp::ShadeSmooth, MeshOp::ShadeFlat, MeshOp::Flip]),
    ("Transform", &[MeshOp::Mirror(0), MeshOp::Mirror(1), MeshOp::Mirror(2), MeshOp::Smooth, MeshOp::SnapToGrid]),
];

const MENU_WIDTH: f32 = 230.0;

struct MenuCx<'a> {
    ctx: egui::Context,
    shortcuts: Vec<(egui::KeyboardShortcut, Action)>,
    actions: &'a mut Vec<Action>,
}

impl MenuCx<'_> {
    fn shortcut(&self, action: &Action) -> Option<String> {
        self.shortcuts.iter().find(|(_, a)| a == action).map(|(s, _)| self.ctx.format_shortcut(s))
    }

    fn push_if_clicked(&mut self, ui: &mut Ui, button: egui::Button, action: Action) {
        if ui.add(button).clicked() {
            self.actions.push(action);
            ui.close();
        }
    }

    fn item(&mut self, ui: &mut Ui, icon: Option<icons::Icon>, label: &str, action: Action) {
        let shortcut = self.shortcut(&action);
        self.push_if_clicked(ui, menu_button(icon, label, shortcut), action);
    }

    /// Menu item for actions driven by OS clipboard events instead of key bindings.
    fn item_keys(&mut self, ui: &mut Ui, icon: Option<icons::Icon>, label: &str, keys: &str, action: Action) {
        self.push_if_clicked(ui, menu_button(icon, label, Some(keys.to_string())), action);
    }

    fn toggle(&mut self, ui: &mut Ui, label: &str, on: bool, action: Action) {
        let shortcut = self.shortcut(&action);
        self.push_if_clicked(ui, menu_button(on.then_some(icons::CHECK), label, shortcut), action);
    }

    fn point_entities(&mut self, ui: &mut Ui, game: &gt_formats::game::GameConfig, classes: &[&str]) {
        let mut any = false;
        for class in classes.iter().filter(|c| game.entity(c).is_some()) {
            any = true;
            self.item(ui, None, class, Action::CreatePointEntity { classname: class.to_string(), at: None });
        }
        if !any {
            empty_hint(ui, "Not defined by this game config");
        }
    }
}

fn menu_button<'a>(icon: Option<icons::Icon>, label: &str, shortcut: Option<String>) -> egui::Button<'a> {
    let button = egui::Button::new((icons::atom(icon, icons::SMALL), label.to_string())).image_tint_follows_text_color(true);
    match shortcut {
        Some(s) => button.shortcut_text(s),
        None => button,
    }
}

fn sub_menu<R>(ui: &mut Ui, icon: Option<icons::Icon>, label: &str, add_contents: impl FnOnce(&mut Ui) -> R) {
    use egui::containers::menu::SubMenuButton;
    let button =
        egui::Button::new((icons::atom(icon, icons::SMALL), label.to_string())).right_text(SubMenuButton::RIGHT_ARROW).image_tint_follows_text_color(true);
    SubMenuButton::from_button(button).ui(ui, |ui| {
        ui.set_min_width(MENU_WIDTH * 0.8);
        add_contents(ui)
    });
}

fn empty_hint(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).weak().italics());
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map(|c| c.to_uppercase().chain(chars).collect()).unwrap_or_default()
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
        icons::install(&cc.egui_ctx);

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
            Action::ShowReference => show_tab(&mut self.dock, Tab::Reference),
            Action::ShowUvEditor => show_tab(&mut self.dock, Tab::Uv),
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
        use crate::entity_wizards::{DoorKind, HingeSide, SlideDirection};
        let mut m = MenuCx { ctx: ui.ctx().clone(), shortcuts: commands::shortcuts(&self.state.prefs), actions: &mut self.actions };
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                ui.set_min_width(MENU_WIDTH);
                m.item(ui, Some(icons::NEW), "New Map", Action::NewMap);
                m.item(ui, Some(icons::OPEN), "Open Map…", Action::OpenMap);
                sub_menu(ui, Some(icons::RECENT), "Open Recent", |ui| {
                    let recent = self.state.prefs.recent_files.clone();
                    if recent.is_empty() {
                        empty_hint(ui, "No recent maps");
                    }
                    for path in recent {
                        if ui.button(path.display().to_string()).clicked() {
                            if let Err(e) = commands::open_map_in_tab(&mut self.state, &path) {
                                self.state.set_status(format!("Open failed: {e}"));
                            }
                            self.project_generation += 1;
                            ui.close();
                        }
                    }
                });
                ui.separator();
                m.item(ui, Some(icons::SAVE), "Save", Action::Save);
                m.item(ui, None, "Save As…", Action::SaveAs);
                ui.separator();
                sub_menu(ui, None, "Tabs", |ui| {
                    m.item(ui, Some(icons::PLUS), "New Tab", Action::NewTab);
                    m.item(ui, None, "Next Tab", Action::NextTab);
                    m.item(ui, None, "Close Tab", Action::CloseTab);
                });
                sub_menu(ui, Some(icons::IMPORT), "Import", |ui| {
                    m.item(ui, None, ".map (TrenchBroom, Quake)…", Action::ImportQuakeMap);
                    m.item(ui, None, ".vmf (Hammer)…", Action::ImportVmf);
                    ui.separator();
                    m.item(ui, None, "Model Prop (.bbmodel, .glb)…", Action::ImportModel(ModelImport::Prop));
                    m.item(ui, None, "Blockbench Model as Mesh…", Action::ImportModel(ModelImport::Mesh));
                    m.item(ui, None, "Blockbench Model as Brushes…", Action::ImportModel(ModelImport::Brushes));
                });
                sub_menu(ui, Some(icons::EXPORT), "Export", |ui| {
                    m.item(ui, None, ".map (Valve 220)…", Action::ExportQuakeMap);
                    m.item(ui, None, ".map, cordon only…", Action::ExportQuakeMapCordon);
                });
                ui.separator();
                if ui.add(menu_button(Some(icons::SETTINGS), "Preferences…", None)).clicked() {
                    self.show_prefs = true;
                    ui.close();
                }
            });
            ui.menu_button("Edit", |ui| {
                ui.set_min_width(MENU_WIDTH);
                m.item(ui, Some(icons::UNDO), "Undo", Action::Undo);
                m.item(ui, Some(icons::REDO), "Redo", Action::Redo);
                ui.separator();
                m.item_keys(ui, None, "Cut", "Ctrl+X", Action::Cut);
                m.item_keys(ui, Some(icons::COPY), "Copy", "Ctrl+C", Action::Copy);
                if ui.add(menu_button(Some(icons::PASTE), "Paste", Some("Ctrl+V".into()))).clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::RequestPaste);
                    ui.close();
                }
                m.item(ui, None, "Duplicate", Action::Duplicate);
                m.item(ui, Some(icons::DELETE), "Delete", Action::Delete);
                ui.separator();
                sub_menu(ui, Some(icons::SELECT), "Select", |ui| {
                    m.item(ui, None, "All", Action::SelectAll);
                    m.item(ui, None, "None", Action::SelectNone);
                    m.item(ui, None, "Inverse", Action::SelectInverse);
                    ui.separator();
                    m.item(ui, None, "Touching", Action::SelectTouching);
                    m.item(ui, None, "Inside", Action::SelectInside);
                    m.item(ui, None, "Siblings", Action::SelectSiblings);
                    m.item(ui, None, "Same Material", Action::SelectSameMaterial);
                });
                sub_menu(ui, Some(icons::GROUP), "Groups", |ui| {
                    m.item(ui, None, "Group", Action::Group);
                    m.item(ui, None, "Ungroup", Action::Ungroup);
                    ui.separator();
                    m.item(ui, None, "Open Group", Action::OpenGroup);
                    m.item(ui, None, "Close Group", Action::CloseGroup);
                    ui.separator();
                    m.item(ui, None, "Duplicate Linked", Action::DuplicateLinked);
                    m.item(ui, None, "Unlink Groups", Action::UnlinkGroups);
                });
                sub_menu(ui, Some(icons::INSTANCE), "Prefabs", |ui| {
                    m.item(ui, None, "Create from Selection…", Action::CreatePrefab);
                    m.item(ui, None, "Insert Prefab…", Action::InsertPrefab);
                    ui.separator();
                    m.item(ui, None, "Open Prefab", Action::OpenPrefab);
                    m.item(ui, None, "Explode Instance", Action::ExplodeInstances);
                });
                sub_menu(ui, Some(icons::LAYER), "Layers", |ui| {
                    m.item(ui, Some(icons::PLUS), "Add Layer", Action::AddLayer);
                    ui.separator();
                    for layer in self.state.doc.map.layers.clone() {
                        if let Some(name) = self.state.doc.map.get(layer).map(|n| n.name()) {
                            m.item(ui, None, &format!("Move Selection to {name}"), Action::MoveToLayer(layer));
                        }
                    }
                });
                ui.separator();
                sub_menu(ui, Some(icons::EYE), "Hide and Lock", |ui| {
                    m.item(ui, Some(icons::EYE_OFF), "Hide Selected", Action::HideSelected);
                    m.item(ui, None, "Isolate Selected", Action::IsolateSelected);
                    m.item(ui, Some(icons::EYE), "Show All", Action::UnhideAll);
                    ui.separator();
                    m.item(ui, Some(icons::LOCK), "Lock Selected", Action::LockSelected);
                    m.item(ui, Some(icons::UNLOCK), "Unlock All", Action::UnlockAll);
                });
                m.item(ui, None, "Repeat Last", Action::RepeatLast);
            });
            ui.menu_button("Brush", |ui| {
                ui.set_min_width(MENU_WIDTH);
                m.item(ui, Some(icons::SHAPES), "Shape Generator…", Action::ShowShapeDialog);
                m.item(ui, Some(icons::BRUSH), "Box from Last Bounds", Action::CreateBrushFromBounds);
                ui.separator();
                sub_menu(ui, Some(icons::CSG_SUBTRACT), "CSG", |ui| {
                    m.item(ui, Some(icons::CSG_SUBTRACT), "Subtract", Action::CsgSubtract);
                    m.item(ui, Some(icons::CSG_MERGE), "Convex Merge", Action::CsgMerge);
                    m.item(ui, Some(icons::CSG_INTERSECT), "Intersect", Action::CsgIntersect);
                    ui.separator();
                    m.item(ui, Some(icons::CSG_HOLLOW), "Hollow", Action::CsgHollow);
                    ui.horizontal(|ui| {
                        ui.add_space(icons::SMALL + ui.spacing().item_spacing.x);
                        ui.label("Wall thickness");
                        ui.add(egui::DragValue::new(&mut self.state.hollow_thickness).range(0.125..=1024.0));
                    });
                });
                sub_menu(ui, Some(icons::ROTATE), "Transform", |ui| {
                    for axis in 0..3 {
                        let a = commands::axis_name(axis);
                        m.item(ui, None, &format!("Rotate {a} +90°"), Action::Rotate { axis, degrees: 90.0 });
                        m.item(ui, None, &format!("Rotate {a} -90°"), Action::Rotate { axis, degrees: -90.0 });
                    }
                    ui.separator();
                    for axis in 0..3 {
                        m.item(ui, None, &format!("Flip {}", commands::axis_name(axis)), Action::Flip { axis });
                    }
                    ui.separator();
                    sub_menu(ui, None, "Nudge by Grid", |ui| {
                        for axis in 0..3 {
                            for sign in [1.0, -1.0] {
                                let mut offset = gt_core::DVec3::ZERO;
                                offset[axis] = sign * self.state.grid;
                                let label = format!("{}{}", if sign > 0.0 { "+" } else { "-" }, commands::axis_name(axis));
                                m.item(ui, None, &label, Action::Nudge(offset));
                            }
                        }
                    });
                    m.item(ui, Some(icons::GRID), "Snap Vertices to Grid", Action::SnapVertices);
                });
                sub_menu(ui, Some(icons::TERRAIN), "Displacement", |ui| {
                    for power in [2u8, 3, 4] {
                        m.item(ui, None, &format!("Create, Power {power} ({0}x{0})", (1 << power) + 1), Action::CreateDisplacement(power));
                    }
                    ui.separator();
                    m.item(ui, None, "Sew Displacements", Action::SewDisplacements);
                    m.item(ui, None, "Remove Displacement", Action::RemoveDisplacement);
                });
                ui.separator();
                sub_menu(ui, Some(icons::ENTITY), "Brush Entity", |ui| {
                    let defs: Vec<(String, String)> = self.state.game.solid_entities().map(|d| (d.classname.clone(), d.description.clone())).collect();
                    if defs.is_empty() {
                        empty_hint(ui, "No brush entities, open a Godot project");
                    }
                    for (class, description) in defs {
                        let resp = ui.add(menu_button(None, &class, None));
                        if resp.clicked() {
                            m.actions.push(Action::CreateBrushEntity(class));
                            ui.close();
                        } else if !description.is_empty() {
                            resp.on_hover_text(description);
                        }
                    }
                });
                m.item(ui, None, "Move Brushes to World", Action::MoveToWorld);
            });
            ui.menu_button("Mesh", |ui| {
                ui.set_min_width(MENU_WIDTH);
                m.item(ui, Some(icons::MESH), "Edit Mesh", Action::EditMesh);
                ui.separator();
                m.item(ui, None, "Convert Brushes to Mesh", Action::ConvertToMesh);
                m.item(ui, None, "Convert Meshes to Brushes", Action::ConvertToBrushes);
                m.item(ui, None, "Join Meshes", Action::JoinMeshes);
                ui.separator();
                for (label, ops) in MESH_OP_GROUPS {
                    sub_menu(ui, None, label, |ui| {
                        for op in ops {
                            m.item(ui, None, op.label(), Action::MeshOp(*op));
                        }
                    });
                }
            });
            ui.menu_button("Texture", |ui| {
                ui.set_min_width(MENU_WIDTH);
                m.item(ui, Some(icons::TEXTURE), "Texture Tool", Action::SetTool(ToolKind::Texture));
                m.item(ui, None, "UV Editor", Action::ShowUvEditor);
                m.item(ui, None, "Hotspot Editor…", Action::ShowHotspotEditor);
                ui.separator();
                m.toggle(ui, "UV Lock", self.state.uv_lock, Action::ToggleUvLock);
                ui.separator();
                sub_menu(ui, None, "Justify", |ui| {
                    for j in gt_geom::Justify::ALL {
                        m.item(ui, None, j.label(), Action::Justify(j));
                    }
                    ui.separator();
                    m.toggle(ui, "Treat as One", self.state.treat_as_one, Action::ToggleTreatAsOne);
                });
                sub_menu(ui, None, "Alignment", |ui| {
                    m.item(ui, Some(icons::FOCUS), "Align to 3D View", Action::AlignTextureToView);
                    m.item(ui, None, "Reset Alignment", Action::ResetTexture);
                    ui.separator();
                    m.item(ui, Some(icons::COPY), "Copy Material and Alignment", Action::CopyAlignment);
                    m.item(ui, Some(icons::PASTE), "Paste Alignment", Action::PasteAlignment);
                });
                sub_menu(ui, None, "Texel Density", |ui| {
                    for d in [0.125, 0.25, 0.5, 1.0, 2.0, 4.0] {
                        m.item(ui, None, &format!("{d} units per pixel"), Action::TexelDensity(d));
                    }
                });
                sub_menu(ui, None, "Mesh UVs", |ui| {
                    for k in crate::texture_ops::MeshUvKind::ALL {
                        m.item(ui, None, k.label(), Action::MeshUv(k));
                    }
                });
                m.item(ui, None, "Hotspot Fit", Action::HotspotTexture);
            });
            ui.menu_button("Terrain", |ui| {
                ui.set_min_width(MENU_WIDTH);
                m.item(ui, Some(icons::TERRAIN), "Create Terrain…", Action::ShowTerrainDialog);
                ui.separator();
                sub_menu(ui, Some(icons::SCULPT), "Sculpt", |ui| {
                    m.item(ui, Some(icons::SCULPT), "Sculpt Tool", Action::SetTool(ToolKind::Sculpt));
                    ui.separator();
                    m.item(ui, None, "Flatten", Action::TerrainFlatten);
                });
                sub_menu(ui, Some(icons::BLEND), "Blend", |ui| {
                    m.item(ui, Some(icons::BLEND), "Blend Tool", Action::SetTool(ToolKind::Blend));
                    ui.separator();
                    m.item(ui, None, "Auto Paint Layers", Action::TerrainAutoPaint);
                    m.item(ui, None, "Set Blend Material (current)", Action::SetBlendMaterial);
                    m.item(ui, None, "Clear Blend Material", Action::ClearBlendMaterial);
                });
                sub_menu(ui, Some(icons::SCATTER), "Scatter", |ui| {
                    m.item(ui, Some(icons::SCATTER), "Scatter Tool", Action::SetTool(ToolKind::Scatter));
                    m.item(ui, None, "Scatter Palette…", Action::ShowScatterPalette);
                    sub_menu(ui, None, "Preset", |ui| {
                        for preset in gt_doc::scatter::PRESETS {
                            m.item(ui, None, preset, Action::ScatterPreset(preset.to_string()));
                        }
                    });
                    ui.separator();
                    m.item(ui, Some(icons::PLUS), "New Scatter Set (new layer)", Action::NewScatterSet);
                    m.item(ui, None, "Fill Scatter Targets", Action::ScatterFill);
                    m.item(ui, None, "Scatter Sets to Entities", Action::ScatterToEntities);
                    ui.separator();
                    m.item(ui, None, "Install Nature Models", Action::InstallNatureModels);
                });
            });
            ui.menu_button("Gameplay", |ui| {
                ui.set_min_width(MENU_WIDTH);
                let game = &self.state.game;
                sub_menu(ui, Some(icons::DOOR), "Doors and Movers", |ui| {
                    empty_hint(ui, "From the selected brushes");
                    let hinged = |side| DoorKind::Hinged { side, angle: 95.0 };
                    let sliding = |direction| DoorKind::Sliding { direction, lip: 4.0 };
                    m.item(ui, None, "Hinged Door, left hinge", Action::MakeDoor { kind: hinged(HingeSide::Left), trigger: true });
                    m.item(ui, None, "Hinged Door, right hinge", Action::MakeDoor { kind: hinged(HingeSide::Right), trigger: true });
                    m.item(ui, None, "Sliding Door, up", Action::MakeDoor { kind: sliding(SlideDirection::Up), trigger: true });
                    m.item(ui, None, "Sliding Door, sideways", Action::MakeDoor { kind: sliding(SlideDirection::Left), trigger: true });
                    ui.separator();
                    m.item(ui, None, "Lift, Moving Platform", Action::MakePlatform);
                });
                sub_menu(ui, Some(icons::VOLUME), "Triggers", |ui| {
                    m.item(ui, Some(icons::VOLUME), "Volume Tool", Action::SetTool(ToolKind::Volume));
                    ui.separator();
                    empty_hint(ui, "Around the selection");
                    for class in crate::volume_tool::VOLUME_CLASSES {
                        m.item(ui, None, class, Action::VolumeAroundSelection(class.to_string()));
                    }
                });
                sub_menu(ui, Some(icons::LINK), "Logic", |ui| {
                    m.item(ui, Some(icons::LINK), "Link Two Selected Entities…", Action::ShowLinkDialog);
                    ui.separator();
                    empty_hint(ui, "Place at the cursor");
                    m.point_entities(ui, game, &["logic_relay", "logic_timer", "logic_counter", "logic_call", "logic_auto", "logic_debug"]);
                });
                sub_menu(ui, Some(icons::ENTITY), "Spawning", |ui| {
                    empty_hint(ui, "Place at the cursor");
                    m.point_entities(ui, game, &["info_spawner", "info_teleport_destination"]);
                });
                sub_menu(ui, Some(icons::PATH), "Paths", |ui| {
                    m.item(ui, Some(icons::PATH), "Path Tool", Action::SetTool(ToolKind::Path));
                    ui.separator();
                    empty_hint(ui, "Place at the cursor");
                    m.point_entities(ui, game, &["path_corner"]);
                });
            });
            ui.menu_button("Tools", |ui| {
                ui.set_min_width(MENU_WIDTH);
                for (i, group) in ToolKind::GROUPS.iter().enumerate() {
                    if i > 0 {
                        ui.separator();
                    }
                    for t in group.iter().copied() {
                        let action = if t == ToolKind::Mesh { Action::EditMesh } else { Action::SetTool(t) };
                        let shortcut = m.shortcut(&action);
                        let button = menu_button(Some(icons::tool(t)), &format!("{} Tool", t.label()), shortcut).selected(self.state.tool == t);
                        if ui.add(button).on_hover_text(panels::tool_help(t)).clicked() {
                            m.actions.push(Action::SetTool(t));
                            ui.close();
                        }
                    }
                }
            });
            ui.menu_button("View", |ui| {
                ui.set_min_width(MENU_WIDTH);
                m.item(ui, Some(icons::FOCUS), "Focus Selection", Action::FocusSelection);
                ui.separator();
                sub_menu(ui, Some(icons::shade(self.state.prefs.shade)), "Shading", |ui| {
                    for s in Shade::ALL {
                        let shortcut = m.shortcut(&Action::SetShade(s));
                        let label = if s == Shade::Lit { "Lit Preview".to_string() } else { capitalize(s.label()) };
                        if ui.add(menu_button(Some(icons::shade(s)), &label, shortcut).selected(self.state.prefs.shade == s)).clicked() {
                            m.actions.push(Action::SetShade(s));
                            ui.close();
                        }
                    }
                    ui.separator();
                    m.item(ui, None, "Cycle Shading", Action::ToggleTextured);
                });
                sub_menu(ui, Some(icons::GRID), "Grid", |ui| {
                    m.item(ui, None, "Larger Grid", Action::GridUp);
                    m.item(ui, None, "Smaller Grid", Action::GridDown);
                    ui.separator();
                    m.toggle(ui, "Snap to Grid", self.state.snap, Action::ToggleSnap);
                });
                sub_menu(ui, None, "Cordon", |ui| {
                    m.item(ui, None, "Set Cordon from Selection", Action::SetCordonFromSelection);
                    m.toggle(ui, "Cordon Enabled", self.state.doc.map.editor.cordon_enabled, Action::ToggleCordon);
                    m.item(ui, None, "Clear Cordon", Action::ClearCordon);
                });
                sub_menu(ui, None, "Camera Bookmarks", |ui| {
                    egui::Grid::new("camera_bookmarks").num_columns(2).show(ui, |ui| {
                        for n in 1..=9u8 {
                            let stored = self.state.doc.map.editor.cameras.contains_key(&n);
                            let recall = menu_button(None, &format!("Go to {n}"), m.shortcut(&Action::RecallCamera(n)));
                            if ui.add_enabled(stored, recall).clicked() {
                                m.actions.push(Action::RecallCamera(n));
                                ui.close();
                            }
                            if ui.add(egui::Button::new(format!("Store {n}")).shortcut_text(m.shortcut(&Action::StoreCamera(n)).unwrap_or_default())).clicked()
                            {
                                m.actions.push(Action::StoreCamera(n));
                                ui.close();
                            }
                            ui.end_row();
                        }
                    });
                });
                ui.separator();
                sub_menu(ui, None, "Panels", |ui| {
                    for tab in PANEL_TABS {
                        let open = self.dock.find_tab(&tab).is_some();
                        if ui.add(menu_button(open.then_some(icons::CHECK), tab_title(tab), None)).clicked() {
                            show_tab(&mut self.dock, tab);
                            ui.close();
                        }
                    }
                    ui.separator();
                    if ui.add(menu_button(None, "Reset Layout", None)).clicked() {
                        self.dock = default_dock();
                        ui.close();
                    }
                });
            });
            ui.menu_button("Godot", |ui| {
                ui.set_min_width(MENU_WIDTH);
                m.item(ui, Some(icons::PLAY), "Run Project", Action::RunGodotProject);
                m.item(ui, None, "Open Project in Godot Editor", Action::OpenGodotEditor);
                ui.separator();
                m.item(ui, Some(icons::OPEN), "Open Godot Project…", Action::OpenProject);
                sub_menu(ui, Some(icons::RECENT), "Recent Projects", |ui| {
                    let recent = self.state.prefs.recent_projects.clone();
                    if recent.is_empty() {
                        empty_hint(ui, "No recent projects");
                    }
                    for path in recent {
                        if ui.button(path.display().to_string()).clicked() {
                            self.state.load_project(&path);
                            self.project_generation += 1;
                            ui.close();
                        }
                    }
                });
                ui.separator();
                sub_menu(ui, None, "Reload", |ui| {
                    m.item(ui, None, "Game Config", Action::ReloadProject);
                    m.item(ui, None, "Materials", Action::ReloadMaterials);
                    m.item(ui, None, "Models", Action::ReloadModels);
                });
            });
            ui.menu_button("Help", |ui| {
                ui.set_min_width(MENU_WIDTH);
                m.item(ui, Some(icons::COMMAND), "Command Palette", Action::ShowCommandPalette);
                m.item(ui, Some(icons::KEYBOARD), "Keyboard Shortcuts…", Action::ShowKeymap);
                m.item(ui, Some(icons::REFERENCE), "Entity and Code Reference", Action::ShowReference);
                ui.separator();
                ui.label(RichText::new("GodotTrench, a brush and mesh level editor for Godot").strong());
                for line in [
                    "3D: RMB look + WASD fly (Q/E down/up), MMB pan, Alt+LMB orbit",
                    "2D: RMB/MMB pan, wheel zoom, drag edges to resize",
                    "Drag empty space to draw a brush, drag selection to move (Alt vertical, Ctrl duplicate)",
                    "Shift+click selects faces, Shift+drag a face resizes, Ctrl+Shift+drag extrudes",
                    "Tab edits meshes Blender style: 1/2/3 modes, G/R/S, E extrude, I inset, Ctrl+R loop cut, K knife",
                ] {
                    ui.label(RichText::new(line).weak());
                }
            });
        });
    }

    fn toolbar(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let shortcuts = commands::shortcuts(&self.state.prefs);
        let tip = |label: &str, action: &Action| match shortcuts.iter().find(|(_, a)| a == action) {
            Some((s, _)) => format!("{label} ({})", ctx.format_shortcut(s)),
            None => label.to_string(),
        };
        let group_gap = |ui: &mut Ui| {
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);
        };
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for (icon, label, action) in
                [(icons::NEW, "New Map", Action::NewMap), (icons::OPEN, "Open Map", Action::OpenMap), (icons::SAVE, "Save", Action::Save)]
            {
                if icons::button(ui, icon, label, tip(label, &action)).clicked() {
                    self.actions.push(action);
                }
            }
            group_gap(ui);
            let history = [
                (icons::UNDO, "Undo", Action::Undo, self.state.doc.history.can_undo()),
                (icons::REDO, "Redo", Action::Redo, self.state.doc.history.can_redo()),
            ];
            for (icon, label, action, enabled) in history {
                let resp = ui.add_enabled_ui(enabled, |ui| icons::button(ui, icon, label, tip(label, &action))).inner;
                if resp.clicked() {
                    self.actions.push(action);
                }
            }
            group_gap(ui);
            for (i, group) in ToolKind::GROUPS.iter().enumerate() {
                if i > 0 {
                    ui.add_space(6.0);
                }
                for t in group.iter().copied() {
                    let action = if t == ToolKind::Mesh { Action::EditMesh } else { Action::SetTool(t) };
                    let tooltip = format!("{}\n{}", tip(&format!("{} tool", t.label()), &action), panels::tool_help(t));
                    if icons::toggle(ui, icons::tool(t), self.state.tool == t, t.label(), tooltip).clicked() {
                        self.actions.push(Action::SetTool(t));
                    }
                }
            }
            group_gap(ui);
            ui.add(icons::GRID.image(icons::TOOLBAR).tint(ui.visuals().text_color())).on_hover_text("Grid size, [ and ] change it");
            egui::ComboBox::from_id_salt("grid").selected_text(format!("{}", self.state.grid)).width(56.0).show_ui(ui, |ui| {
                for g in [0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0, 512.0, 1024.0] {
                    ui.selectable_value(&mut self.state.grid, g, format!("{g}"));
                }
            });
            ui.add_space(2.0);
            if icons::toggle(ui, icons::SNAP, self.state.snap, "Snap to grid", tip("Snap to grid", &Action::ToggleSnap)).clicked() {
                self.actions.push(Action::ToggleSnap);
            }
            let uv_tip = format!("{}\nTextures stay fixed to faces while moving and rotating", tip("UV lock", &Action::ToggleUvLock));
            if icons::toggle(ui, icons::UV_LOCK, self.state.uv_lock, "UV lock", uv_tip).clicked() {
                self.actions.push(Action::ToggleUvLock);
            }
            group_gap(ui);
            for (icon, label, help, action) in [
                (icons::CSG_SUBTRACT, "CSG subtract", "Carve the selected brushes out of the brushes they touch", Action::CsgSubtract),
                (icons::CSG_MERGE, "CSG convex merge", "Merge the selected brushes into one convex brush", Action::CsgMerge),
                (icons::CSG_INTERSECT, "CSG intersect", "Keep only the volume shared by the selected brushes", Action::CsgIntersect),
                (icons::CSG_HOLLOW, "Hollow", "Turn the selected brushes into walls (thickness in Brush > CSG)", Action::CsgHollow),
            ] {
                if icons::button(ui, icon, label, format!("{}\n{help}", tip(label, &action))).clicked() {
                    self.actions.push(action);
                }
            }
            group_gap(ui);
            for s in Shade::ALL {
                let label = format!("{} shading", capitalize(s.label()));
                if icons::toggle(ui, icons::shade(s), self.state.prefs.shade == s, &label, tip(&label, &Action::SetShade(s))).clicked() {
                    self.actions.push(Action::SetShade(s));
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let project = match &self.state.game.project_root {
                    Some(p) => format!("{} ({})", self.state.game.name, p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()),
                    None => "Open Godot project…".into(),
                };
                let button = egui::Button::image_and_text(icons::OPEN.image(icons::SMALL), RichText::new(project).color(Color32::from_rgb(140, 190, 255)))
                    .image_tint_follows_text_color(true);
                if ui.add(button).on_hover_text("Open a Godot project folder").clicked() {
                    self.actions.push(Action::OpenProject);
                }
            });
        });
    }

    fn tool_options(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            let tool = self.state.tool;
            let help = panels::tool_help(tool);
            ui.add(icons::tool(tool).image(icons::SMALL).tint(ui.visuals().strong_text_color()));
            ui.label(RichText::new(format!("{} tool", tool.label())).strong()).on_hover_text(help);
            ui.separator();
            match tool {
                ToolKind::Sculpt | ToolKind::Paint => {
                    use gt_doc::terrain::SculptMode;
                    if tool == ToolKind::Sculpt {
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
                        ui.label("Color");
                        ui.color_edit_button_rgba_unmultiplied(&mut self.state.paint_color);
                    }
                    ui.add(egui::DragValue::new(&mut self.state.sculpt.radius).range(1.0..=8192.0).prefix("radius "));
                    ui.add(egui::DragValue::new(&mut self.state.sculpt.strength).range(0.01..=256.0).speed(0.1).prefix("strength "));
                }
                ToolKind::Texture => {
                    ui.label("Justify");
                    for j in gt_geom::Justify::ALL {
                        if ui.small_button(j.label()).on_hover_text("Justify the selected faces").clicked() {
                            self.actions.push(Action::Justify(j));
                        }
                    }
                    ui.checkbox(&mut self.state.treat_as_one, "Treat as one");
                    ui.separator();
                    if ui.small_button("Align to View").on_hover_text("Project the selected faces along the 3D camera").clicked() {
                        self.actions.push(Action::AlignTextureToView);
                    }
                    if ui.small_button("Hotspot Fit").on_hover_text("Fit to the best rectangle of <texture>.hotspots.json").clicked() {
                        self.actions.push(Action::HotspotTexture);
                    }
                }
                ToolKind::Scatter => {
                    let sets: Vec<(gt_core::NodeId, String)> =
                        self.state.doc.map.scatters().map(|(id, s)| (id, format!("{} ({})", s.name, s.instances.len()))).collect();
                    let active = crate::scatter_tool::active_set(&self.state);
                    let current = active.and_then(|a| sets.iter().find(|(id, _)| *id == a)).map(|(_, n)| n.clone()).unwrap_or_else(|| "new layer".into());
                    ui.label("Set");
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
                    ui.label("Preset");
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
                    ui.checkbox(&mut s.rules.only_targets, "Targets only");
                    ui.separator();
                    if ui.small_button("Palette…").clicked() {
                        self.actions.push(Action::ShowScatterPalette);
                    }
                    if ui.small_button("Fill").on_hover_text("Fill the set's target surfaces").clicked() {
                        self.actions.push(Action::ScatterFill);
                    }
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
                            ui.add(egui::DragValue::new(&mut b.slope[0]).range(0.0..=90.0).prefix("slope ").suffix("°"));
                            ui.add(egui::DragValue::new(&mut b.slope[1]).range(0.0..=90.0).prefix("to ").suffix("°"));
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
                    ui.separator();
                    if ui.small_button("Set Blend Material").on_hover_text("Use the current material as the second texture of the selected faces").clicked() {
                        self.actions.push(Action::SetBlendMaterial);
                    }
                }
                ToolKind::Volume => {
                    let p = &mut self.state.prefs;
                    ui.label("Class");
                    egui::ComboBox::from_id_salt("volume_class").selected_text(p.volume_class.clone()).width(140.0).show_ui(ui, |ui| {
                        for class in crate::volume_tool::VOLUME_CLASSES {
                            ui.selectable_value(&mut p.volume_class, class.to_string(), class);
                        }
                    });
                    ui.add(egui::DragValue::new(&mut p.volume_height).range(1.0..=8192.0).prefix("height "));
                }
                ToolKind::Mesh => {
                    use crate::mesh_tool::Component;
                    for c in [Component::Vertex, Component::Edge, Component::Face] {
                        if ui.selectable_label(self.tools.mesh.component == c, capitalize(c.label())).clicked() {
                            self.tools.mesh.component = c;
                        }
                    }
                    ui.separator();
                    for (label, op) in [
                        ("Subdivide", MeshOp::Subdivide),
                        ("Merge", MeshOp::MergeCenter),
                        ("Fill", MeshOp::Fill),
                        ("Delete", MeshOp::Delete),
                        ("Smooth", MeshOp::ShadeSmooth),
                        ("Flat", MeshOp::ShadeFlat),
                    ] {
                        if ui.small_button(label).on_hover_text(op.label()).clicked() {
                            self.actions.push(Action::MeshOp(op));
                        }
                    }
                }
                _ => {
                    ui.label(RichText::new(help).weak());
                }
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
            other => tab_title(*other).into(),
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
        egui::Panel::top("tool_options").show(ui, |ui| self.tool_options(ui));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_menu_groups_cover_every_op() {
        let grouped: Vec<MeshOp> = MESH_OP_GROUPS.iter().flat_map(|(_, ops)| ops.iter().copied()).collect();
        assert_eq!(grouped.len(), MeshOp::ALL.len());
        assert!(MeshOp::ALL.iter().all(|op| grouped.contains(op)));
    }

    #[test]
    fn capitalizes_labels() {
        assert_eq!(capitalize("wireframe"), "Wireframe");
        assert_eq!(capitalize(""), "");
    }
}
