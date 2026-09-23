use egui::{RichText, Ui};
use egui_dock::{DockArea, DockState, Node, NodeIndex, SplitNode, TabViewer, Tree};
use gt_render::Renderer;

use crate::CliArgs;
use crate::camera::ViewKind;
use crate::commands::{self, Action, ModelImport};
use crate::dialogs::{CommandPalette, KeymapWindow, LinkDialog, ShapeDialog, TerrainDialog};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
enum Tab {
    View(usize),
    Outliner,
    Inspector,
    Materials,
    Models,
    Entities,
    History,
    Issues,
    Logic,
    Uv,
    Reference,
    Scatter,
}

/// A dockable side panel, used to map panels to tabs and to show a hover tooltip on each tab.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Panel {
    Outliner,
    Inspector,
    Materials,
    Entities,
    History,
    Issues,
    Uv,
    Reference,
    Scatter,
}

impl Panel {
    pub const ALL: [Panel; 9] =
        [Panel::Outliner, Panel::Inspector, Panel::Materials, Panel::Entities, Panel::History, Panel::Issues, Panel::Uv, Panel::Reference, Panel::Scatter];

    /// One or two sentences on what the panel is for, shown as the panel tab tooltip.
    pub fn help(self) -> &'static str {
        match self {
            Panel::Outliner => {
                "Every layer, group, brush, mesh and entity in the map as a tree. Click to select, the eye hides and the lock locks. Clicking a layer makes it the current layer new objects go into, right click it to rename or omit it from the build."
            }
            Panel::Inspector => {
                "Edits whatever is selected: entity properties and outputs, face materials and UVs, scatter sets and terrains. With nothing selected it shows the map's own settings (worldspawn), like sun, sky and fog."
            }
            Panel::Materials => {
                "The textures and materials of the Godot project. Click one to apply it to the selection, drag it onto a face, right click for favourites. Search and folders help in big projects."
            }
            Panel::Entities => {
                "Entity classes from the game config as cards. Drag one into a view or double click to place it at the cursor, Ctrl or Shift click selects several to drag in together. Brush entities come with a box, or wrap the selected brushes on double click."
            }
            Panel::History => "Every edit in order. Click an older entry to undo back to it, click a later one to redo.",
            Panel::Issues => {
                "Problems found in the map, like invalid brushes, missing materials or outputs pointing at nothing. Click one to select the object, most offer a fix."
            }
            Panel::Uv => "Edits the UVs of selected mesh faces directly, like the UV editor of a 3D modelling program.",
            Panel::Scatter => {
                "The active scatter set as model cards: drag models in from the Models panel, switch them on or off and weigh them. Below are the surfaces the set paints onto, with an eyedropper, and the brush."
            }
            Panel::Reference => {
                "How to use the selected entity from code: GDScript and C# snippets, the FGD resource, and buttons that create the script in your project. Drag the divider to resize the class list, double click it to fit the names."
            }
        }
    }
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
    /// Asking whether to save the active map before its tab closes.
    confirm_tab_close: bool,
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
    link_dialog: LinkDialog,
    keep_prefs: bool,
    window_fitted: bool,
    toolbar_fit: ToolbarFit,
    /// Height of the tool options bar contents last frame.
    tool_options_height: f32,
    /// Scale shown while its slider is dragged, applied on release so the slider does not move under the pointer.
    ui_scale_draft: Option<f32>,
    /// The product logo drawn at the left of the menu bar.
    logo: egui::TextureHandle,
    /// Last single scatter set the selection settled on, so selecting one activates it for painting only on change.
    last_selected_scatter: Option<gt_core::NodeId>,
    /// Tool of the last frame, so picking the scatter tool can bring up the Scatter panel once.
    last_tool: ToolKind,
    /// The layout from before Maximize View, restored when it is toggled off.
    maximized: Option<DockState<Tab>>,
    /// The view last under the pointer, the one Maximize View fills the area with.
    active_view: usize,
}

const PREFS_LABEL_WIDTH: f32 = 180.0;
const UI_SCALE_PRESETS: [f32; 6] = [0.75, 1.0, 1.25, 1.5, 1.75, 2.0];

const PANEL_TABS: [Tab; 11] =
    [Tab::Outliner, Tab::Inspector, Tab::Materials, Tab::Models, Tab::Entities, Tab::History, Tab::Issues, Tab::Logic, Tab::Uv, Tab::Reference, Tab::Scatter];

fn tab_title(tab: Tab) -> &'static str {
    match tab {
        Tab::View(_) => "View",
        Tab::Outliner => "Outliner",
        Tab::Inspector => "Inspector",
        Tab::Materials => "Materials",
        Tab::Models => "Models",
        Tab::Entities => "Entities",
        Tab::History => "History",
        Tab::Issues => "Issues",
        Tab::Logic => "Logic",
        Tab::Uv => "UV Editor",
        Tab::Reference => "Reference",
        Tab::Scatter => "Scatter",
    }
}

fn panel_tab(panel: Panel) -> Tab {
    match panel {
        Panel::Outliner => Tab::Outliner,
        Panel::Inspector => Tab::Inspector,
        Panel::Materials => Tab::Materials,
        Panel::Entities => Tab::Entities,
        Panel::History => Tab::History,
        Panel::Issues => Tab::Issues,
        Panel::Uv => Tab::Uv,
        Panel::Reference => Tab::Reference,
        Panel::Scatter => Tab::Scatter,
    }
}

fn tab_panel(tab: Tab) -> Option<Panel> {
    Panel::ALL.into_iter().find(|p| panel_tab(*p) == tab)
}

const TOOLBAR_ID: &str = "toolbar";
const TOOL_OPTIONS_ID: &str = "tool_options";

/// Tallest the tool options bar can be dragged, unless its contents wrap into more rows.
const TOOL_OPTIONS_MAX_HEIGHT: f32 = 48.0;
/// Room kept between the toolbar icons and the project button on the right.
const PROJECT_BUTTON_GAP: f32 = 16.0;

/// Contents of a top bar the user can drag taller, centered vertically in the extra height. Returns their height.
fn bar_contents(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) -> f32 {
    let id = ui.id().with("bar_content_height");
    let content_height = ui.ctx().data(|d| d.get_temp::<f32>(id)).unwrap_or_else(|| ui.available_height());
    ui.add_space(((ui.available_height() - content_height) * 0.5).floor().max(0.0));
    let height = ui.scope(add_contents).response.rect.height();
    // A resizable panel keeps the size its contents fill, without this the dragged height snaps back on release.
    ui.take_available_space();
    if (height - content_height).abs() > 0.5 {
        ui.ctx().data_mut(|d| d.insert_temp(id, height));
        ui.ctx().request_repaint();
    }

    height
}

/// Frame margins and separator line a top panel adds around its contents.
fn bar_margin(style: &egui::Style) -> f32 {
    egui::Frame::side_top_panel(style).total_margin().sum().y + style.visuals.widgets.noninteractive.bg_stroke.width.round()
}

/// What the toolbar measured last frame, used to size its icons for the next one.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct ToolbarFit {
    icon: f32,
    icons: usize,
    /// Width of the icon groups, without the project button.
    tools_width: f32,
    project_width: f32,
    content_height: f32,
}

impl ToolbarFit {
    /// Icons follow the bar height between the default and the maximum size, and shrink when they would no longer fit on one
    /// row. Every icon button adds the same width, so the size that fits follows from last frame's measurement.
    fn icon_size(&self, bar_height: f32, width: f32, padding: f32) -> f32 {
        let from_height = bar_height - padding;
        let fitting = match self.icons {
            0 => icons::TOOLBAR,
            n => self.icon + (width - self.project_width - PROJECT_BUTTON_GAP - self.tools_width) / n as f32,
        };
        from_height.min(fitting).clamp(icons::TOOLBAR, icons::TOOLBAR_MAX).floor()
    }

    /// Tallest the bar can be dragged: a row of the largest icons, or the wrapped rows of the smallest ones.
    fn max_height(&self, padding: f32) -> f32 {
        let wrapped = if self.icon <= icons::TOOLBAR { self.content_height } else { 0.0 };
        (icons::TOOLBAR_MAX + padding).max(wrapped)
    }
}

/// Smallest width or height a view keeps while the grid corner is dragged.
const MIN_VIEW_SIZE: f32 = 80.0;
const GRID_CORNER_GRAB: f32 = 14.0;

fn split_node(node: &Node<Tab>) -> Option<&SplitNode> {
    match node {
        Node::Horizontal(s) | Node::Vertical(s) => Some(s),
        _ => None,
    }
}

/// The 2x2 view grid: a left and right split whose sides are each split top and bottom into views.
/// Returns that split followed by the splits of its left and right column.
fn view_grid(tree: &Tree<Tab>) -> Option<[NodeIndex; 3]> {
    let views = |i: NodeIndex| i.0 < tree.len() && tree[i].tabs().is_some_and(|t| !t.is_empty() && t.iter().all(|t| matches!(t, Tab::View(_))));
    let columns = |i: NodeIndex| i.0 < tree.len() && matches!(tree[i], Node::Vertical(_)) && views(i.left()) && views(i.right());
    (0..tree.len())
        .map(NodeIndex)
        .find(|h| matches!(tree[*h], Node::Horizontal(_)) && columns(h.left()) && columns(h.right()))
        .map(|h| [h, h.left(), h.right()])
}

fn split_point(split: &SplitNode, vertical: bool) -> f32 {
    let (min, size) = if vertical { (split.rect.min.y, split.rect.height()) } else { (split.rect.min.x, split.rect.width()) };
    min + size * split.fraction
}

/// Split fraction that puts the separator at `pos`, keeping both sides at least `MIN_VIEW_SIZE`.
fn fraction_at(min: f32, size: f32, pos: f32) -> f32 {
    if size <= MIN_VIEW_SIZE * 2.0 {
        return 0.5;
    }

    (pos - min).clamp(MIN_VIEW_SIZE, size - MIN_VIEW_SIZE) / size
}

/// Handle where the view separators cross, dragging it resizes all four views at once.
fn view_grid_corner(ui: &mut Ui, dock: &mut DockState<Tab>) {
    let tree = dock.main_surface_mut();
    let Some([h, left, right]) = view_grid(tree) else { return };
    let (Some(hs), Some(ls), Some(rs)) = (split_node(&tree[h]), split_node(&tree[left]), split_node(&tree[right])) else { return };
    // The two columns can have their separators at different heights, the handle sits between them and aligns both.
    let corner = egui::pos2(split_point(hs, false), (split_point(ls, true) + split_point(rs, true)) * 0.5);
    let response = ui
        .interact(egui::Rect::from_center_size(corner, egui::Vec2::splat(GRID_CORNER_GRAB)), ui.id().with("view_grid_corner"), egui::Sense::drag())
        .on_hover_and_drag_cursor(egui::CursorIcon::Move)
        .on_hover_text("Drag to resize all four views");
    if response.hovered() || response.dragged() {
        ui.painter().circle_filled(corner, 4.0, ui.visuals().selection.bg_fill);
    }

    if !response.dragged() {
        return;
    }

    let Some(pointer) = response.interact_pointer_pos() else { return };
    if let Node::Horizontal(s) = &mut tree[h] {
        s.fraction = fraction_at(s.rect.min.x, s.rect.width(), pointer.x);
    }

    for column in [left, right] {
        if let Node::Vertical(s) = &mut tree[column] {
            s.fraction = fraction_at(s.rect.min.y, s.rect.height(), pointer.y);
        }
    }

    ui.ctx().request_repaint();
}

/// Focuses a panel tab, reopening it next to `beside` when it was closed.
fn reveal_tab(dock: &mut DockState<Tab>, tab: Tab, beside: Tab) {
    ensure_tab(dock, tab, beside);
    show_tab(dock, tab);
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

    /// Menu item that is greyed out with `why` on hover when `enabled` is false.
    fn item_enabled(&mut self, ui: &mut Ui, icon: Option<icons::Icon>, label: &str, action: Action, enabled: bool, why: &str) {
        let shortcut = self.shortcut(&action);
        let response = ui.add_enabled(enabled, menu_button(icon, label, shortcut)).on_disabled_hover_text(why);
        if response.clicked() {
            self.actions.push(action);
            ui.close();
        }
    }

    /// Menu item for actions driven by OS clipboard events instead of key bindings.
    fn item_keys(&mut self, ui: &mut Ui, icon: Option<icons::Icon>, label: &str, keys: &str, action: Action) {
        self.push_if_clicked(ui, menu_button(icon, label, Some(keys.to_string())), action);
    }

    fn toggle(&mut self, ui: &mut Ui, label: &str, on: bool, action: Action) {
        let shortcut = self.shortcut(&action);
        self.push_if_clicked(ui, menu_button(on.then_some(icons::CHECK), label, shortcut), action);
    }

    fn toggle_enabled(&mut self, ui: &mut Ui, label: &str, on: bool, action: Action, enabled: bool, why: &str) {
        let shortcut = self.shortcut(&action);
        if ui.add_enabled(enabled, menu_button(on.then_some(icons::CHECK), label, shortcut)).on_disabled_hover_text(why).clicked() {
            self.actions.push(action);
            ui.close();
        }
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

fn menu<R>(ui: &mut Ui, title: &'static str, add_contents: impl FnOnce(&mut Ui) -> R) {
    ui.menu_button(title, add_contents);
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

const DOCK_KEY: &str = "dock";

/// A saved layout from an older build can name views that no longer exist or list a tab twice.
fn valid_dock(dock: &DockState<Tab>) -> bool {
    let tabs: Vec<Tab> = dock.iter_all_tabs().map(|(_, tab)| *tab).collect();
    let unique = tabs.iter().enumerate().all(|(i, t)| !tabs[..i].contains(t));
    unique && tabs.iter().all(|t| !matches!(t, Tab::View(n) if *n >= 4)) && tabs.iter().any(|t| matches!(t, Tab::View(_)))
}

/// Adds a panel tab a restored layout is missing, next to `beside` (or the default dock's copy of it).
/// Layouts saved before a panel existed do not carry it, so this brings it in on the next launch.
fn ensure_tab(dock: &mut DockState<Tab>, tab: Tab, beside: Tab) {
    if dock.find_tab(&tab).is_some() {
        return;
    }

    if let Some(path) = dock.find_tab(&beside) {
        dock[path.surface][path.node].append_tab(tab);
    } else {
        dock.push_to_focused_leaf(tab);
    }
}

/// The views shown in the dock.
fn open_views(dock: &DockState<Tab>) -> Vec<usize> {
    dock.iter_all_tabs()
        .filter_map(|(_, t)| match t {
            Tab::View(i) => Some(*i),
            _ => None,
        })
        .collect()
}

/// Views keep a close button while another view stays open, the dock always shows at least one.
fn view_closeable(tab: &Tab, views_open: usize) -> bool {
    !matches!(tab, Tab::View(_)) || views_open > 1
}

/// Shows exactly `views` where the views were: one fills the area, two sit side by side, three put the second
/// beside the other two stacked, four make the 2x2 grid of the default layout.
fn set_views(dock: &mut DockState<Tab>, views: &[usize]) {
    let Some(&first) = views.first() else { return };
    let Some(anchor) = dock.iter_all_tabs().find_map(|(_, t)| matches!(t, Tab::View(_)).then_some(*t)) else { return };
    while let Some(path) = dock.find_tab_from(|t| matches!(t, Tab::View(_)) && *t != anchor) {
        dock.remove_tab(path);
    }

    let Some(path) = dock.find_tab(&anchor) else { return };
    if let Some(tabs) = dock[path.surface][path.node].tabs_mut() {
        tabs[path.tab.0] = Tab::View(first);
    }

    let _ = dock.set_active_tab(path);
    let tree = &mut dock[path.surface];
    let tab = |k: usize| vec![Tab::View(views[k])];
    match views.len() {
        1 => {}
        2 => {
            tree.split_right(path.node, 0.5, tab(1));
        }
        3 => {
            let [left, _] = tree.split_right(path.node, 0.5, tab(1));
            tree.split_below(left, 0.5, tab(2));
        }
        _ => {
            let [left, right] = tree.split_right(path.node, 0.5, tab(1));
            tree.split_below(left, 0.5, tab(2));
            tree.split_below(right, 0.5, tab(3));
        }
    }
}

fn default_dock() -> DockState<Tab> {
    let mut dock = DockState::new(vec![Tab::View(0)]);
    let surface = dock.main_surface_mut();
    let [center, _right] = surface.split_right(NodeIndex::root(), 0.78, vec![Tab::Inspector, Tab::Entities, Tab::Uv, Tab::Scatter]);
    let [center, _left] = surface.split_left(center, 0.2, vec![Tab::Outliner, Tab::History, Tab::Issues]);
    let [views, _bottom] = surface.split_below(center, 0.72, vec![Tab::Materials, Tab::Models]);
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
        state.stdio_mcp = args.mcp_stdio;
        let project = args.project.clone().or_else(|| state.prefs.recent_projects.first().cloned());
        if let Some(root) = project.as_deref().and_then(gt_formats::game::find_project_root) {
            state.load_project(&root);
        }

        state.materials.watch(cc.egui_ctx.clone());
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

        let mut map_failed = false;
        if let Some(path) = args.map
            && let Err(e) = state.open_map(&path)
        {
            state.set_status(format!("Could not open {}: {e}", path.display()));
            map_failed = true;
        }

        state.godot.refresh(&state.prefs.godot_path, state.game.project_root.as_deref());
        if !state.godot.found() {
            let warning = format!("{}. Run Project and Open in Godot stay disabled until then", commands::GODOT_NOT_FOUND);
            eprintln!("GodotTrench: {warning}");
            if !map_failed {
                state.set_status(warning);
            }
        } else if !map_failed {
            let autosaves = state.recoverable_autosaves().len();
            if autosaves > 0 {
                state.set_status(format!("Found {autosaves} autosaved untitled map(s), File > Open Recent lists them for recovery"));
            }
        }

        state.link = Some(crate::live_link::LiveLink::start(Some(cc.egui_ctx.clone())));
        let dock: Option<DockState<Tab>> = if keep_prefs { cc.storage.and_then(|s| eframe::get_value(s, DOCK_KEY)).filter(valid_dock) } else { None };
        cc.egui_ctx.set_visuals(crate::theme::visuals());
        // UI scale shortcuts go through the keymap so they are rebindable and saved in the preferences.
        cc.egui_ctx.options_mut(|o| o.zoom_with_keyboard = false);
        icons::install(&cc.egui_ctx);

        let mut viewports =
            vec![Viewport::new(ViewKind::Perspective), Viewport::new(ViewKind::Top), Viewport::new(ViewKind::Front), Viewport::new(ViewKind::Side)];
        for v in &mut viewports[1..] {
            v.camera.zoom = 0.6;
        }

        let mut dock = dock.unwrap_or_else(default_dock);
        ensure_tab(&mut dock, Tab::Models, Tab::Materials);
        ensure_tab(&mut dock, Tab::Scatter, Tab::Inspector);
        Self {
            state,
            renderer: Renderer::new(render_state),
            scene: SceneCache::default(),
            viewports,
            dock,
            panels: PanelState::default(),
            tools: ToolSet::default(),
            actions: Vec::new(),
            project_generation: 1,
            confirm_close: false,
            confirm_tab_close: false,
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
            link_dialog: Default::default(),
            keep_prefs,
            window_fitted: false,
            toolbar_fit: ToolbarFit::default(),
            tool_options_height: 0.0,
            ui_scale_draft: None,
            logo: crate::brand::texture(&cc.egui_ctx),
            last_selected_scatter: None,
            last_tool: ToolKind::Select,
            maximized: None,
            active_view: 0,
        }
    }

    /// Selecting a single scatter set makes it the active paint target. Tracked on change so it does not fight a
    /// "new set on a new layer" that clears the active set while a set stays selected.
    fn sync_active_scatter(&mut self) {
        let mut it = self.state.doc.selection.nodes.iter().copied();
        let selected = match (it.next(), it.next()) {
            (Some(id), None) => self.state.doc.map.scatter(id).is_some().then_some(id),
            _ => None,
        };
        if selected != self.last_selected_scatter {
            self.last_selected_scatter = selected;
            if let Some(id) = selected {
                self.state.active_scatter = Some(id);
            }
        }
    }

    fn collect_input_actions(&mut self, ctx: &egui::Context) {
        if ctx.egui_wants_keyboard_input() || self.keymap.open {
            return;
        }

        // While a view flies with WASD, Q and E, ToolSet::keys swallows plain keys so no single key shortcut fires.
        self.tools.flying |= self.viewports.iter().any(|v| v.is_flying());
        if self.panels.uv.take_escape(ctx) || self.tools.keys(ctx, &mut self.state) {
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
            if ctx.input_mut(|i| commands::consume_shortcut(i, &shortcut)) {
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
            Action::ShowScatterPanel => reveal_tab(&mut self.dock, Tab::Scatter, Tab::Inspector),
            Action::ShowLinkDialog => {
                self.link_dialog.open_for(&self.state);
                if !self.link_dialog.open {
                    self.state.set_status("Select exactly two entities to link");
                }
            }
            Action::ShowReference => show_tab(&mut self.dock, Tab::Reference),
            Action::ShowPreferences => self.show_prefs = true,
            Action::ToggleMaximizeView => self.toggle_maximize(),
            Action::ViewLayout(n) => {
                self.maximized = None;
                let views: Vec<usize> = if n <= 1 { vec![self.view_to_maximize()] } else { (0..(n as usize).min(self.viewports.len())).collect() };
                set_views(&mut self.dock, &views);
            }
            Action::ToggleView(view) => self.toggle_view(view),
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
            Action::EditHotspots(material) => self.hotspot_editor.open_for(&material),
            Action::CloseTab if self.state.doc.is_modified() => self.confirm_tab_close = true,
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

    /// The shown view last under the pointer, else the first shown view.
    fn view_to_maximize(&self) -> usize {
        let open = open_views(&self.dock);
        if open.contains(&self.active_view) { self.active_view } else { open.iter().copied().min().unwrap_or(0) }
    }

    fn toggle_maximize(&mut self) {
        if let Some(saved) = self.maximized.take() {
            self.dock = saved;
            return;
        }

        if open_views(&self.dock).len() < 2 {
            self.state.set_status("Only one view is shown, View > Views brings the others back");
            return;
        }

        let view = self.view_to_maximize();
        self.maximized = Some(self.dock.clone());
        set_views(&mut self.dock, &[view]);
    }

    fn toggle_view(&mut self, view: usize) {
        self.maximized = None;
        let mut open = open_views(&self.dock);
        if open.contains(&view) {
            if open.len() == 1 {
                self.state.set_status("The last view stays open");
            } else if let Some(path) = self.dock.find_tab(&Tab::View(view)) {
                self.dock.remove_tab(path);
            }
        } else if view < self.viewports.len() {
            open.push(view);
            open.sort_unstable();
            set_views(&mut self.dock, &open);
        }
    }

    fn menu_bar(&mut self, ui: &mut Ui) {
        use crate::entity_wizards::{DoorKind, HingeSide, SlideDirection};
        let mut m = MenuCx { ctx: ui.ctx().clone(), shortcuts: commands::shortcuts(&self.state.prefs), actions: &mut self.actions };
        let logo = self.logo.clone();
        egui::MenuBar::new().ui(ui, |ui| {
            // Product logo, scaled down to sit alongside the menus, with a left gutter so it does not
            // jam against the window edge and read as off-centre.
            ui.add_space(6.0);
            let logo_size = egui::Vec2::splat(icons::SMALL + 6.0);
            ui.add(egui::Image::new(&logo).fit_to_exact_size(logo_size)).on_hover_text("GodotTrench");
            ui.add_space(6.0);
            menu(ui, "File", |ui| {
                ui.set_min_width(MENU_WIDTH);
                m.item(ui, Some(icons::NEW), "New Map", Action::NewMap);
                m.item(ui, Some(icons::OPEN), "Open Map…", Action::OpenMap);
                sub_menu(ui, Some(icons::RECENT), "Open Recent", |ui| {
                    let recent = self.state.prefs.recent_files.clone();
                    if recent.is_empty() {
                        empty_hint(ui, "No recent maps");
                    }

                    let autosaves: Vec<std::path::PathBuf> = self.state.recoverable_autosaves().into_iter().take(8).collect();
                    let first_autosave = recent.len();
                    for (i, path) in recent.into_iter().chain(autosaves).enumerate() {
                        if i == first_autosave {
                            ui.separator();
                            empty_hint(ui, "Autosaved untitled maps");
                        }

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
                    m.item(ui, None, "Convert Textures (VTF/VMT, WAD, WAL)…", Action::ConvertTextures);
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
            menu(ui, "Edit", |ui| {
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
            menu(ui, "Brush", |ui| {
                ui.set_min_width(MENU_WIDTH);
                m.item(ui, Some(icons::SHAPES), "Shape Generator…", Action::ShowShapeDialog);
                m.item(ui, Some(icons::BRUSH), "Box from Last Bounds", Action::CreateBrushFromBounds);
                ui.separator();
                sub_menu(ui, Some(icons::CSG_SUBTRACT), "CSG", |ui| {
                    m.item(ui, Some(icons::CSG_SUBTRACT), "Subtract", Action::CsgSubtract);
                    ui.horizontal(|ui| {
                        ui.add_space(icons::SMALL + ui.spacing().item_spacing.x);
                        let mut keep = self.state.carve_material == gt_geom::csg::CarveMaterial::Target;
                        if ui
                            .checkbox(&mut keep, "Carved faces keep the target's material")
                            .on_hover_text("Off: the cut faces take the cutter's material")
                            .changed()
                        {
                            self.state.carve_material = if keep { gt_geom::csg::CarveMaterial::Target } else { gt_geom::csg::CarveMaterial::Cutter };
                        }
                    });
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
            menu(ui, "Mesh", |ui| {
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
            menu(ui, "Texture", |ui| {
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
            menu(ui, "Terrain", |ui| {
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
                    crate::dialogs::auto_paint_base_ui(ui, &mut self.state.auto_paint_base);
                    ui.separator();
                    m.item(ui, None, "Set Blend Material (current)", Action::SetBlendMaterial);
                    m.item(ui, None, "Clear Blend Material", Action::ClearBlendMaterial);
                });
                sub_menu(ui, Some(icons::SCATTER), "Scatter", |ui| {
                    m.item(ui, Some(icons::SCATTER), "Scatter Tool", Action::SetTool(ToolKind::Scatter));
                    m.item(ui, None, "Scatter Panel", Action::ShowScatterPanel);
                    ui.separator();
                    m.item(ui, Some(icons::PLUS), "New Empty Scatter Set", Action::NewScatterSet);
                    sub_menu(ui, None, "New Set from Preset", |ui| {
                        for preset in gt_doc::scatter::PRESETS {
                            m.item(ui, None, preset, Action::ScatterPreset(preset.to_string()));
                        }
                    });
                    m.item(ui, None, "Fill Scatter Targets", Action::ScatterFill);
                    m.item(ui, None, "Scatter Sets to Entities", Action::ScatterToEntities);
                    ui.separator();
                    m.item(ui, None, "Install Nature Models", Action::InstallNatureModels);
                });
            });
            menu(ui, "Gameplay", |ui| {
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
            menu(ui, "Tools", |ui| {
                ui.set_min_width(MENU_WIDTH);
                for (i, group) in ToolKind::GROUPS.iter().enumerate() {
                    if i > 0 {
                        ui.separator();
                    }

                    for t in group.iter().copied() {
                        let shortcut = m.shortcut(&Action::SetTool(t));
                        let button = menu_button(Some(icons::tool(t)), &format!("{} Tool", t.label()), shortcut).selected(self.state.tool == t);
                        if ui.add(button).on_hover_text(panels::tool_help(t)).clicked() {
                            m.actions.push(Action::SetTool(t));
                            ui.close();
                        }
                    }
                }
            });
            menu(ui, "View", |ui| {
                ui.set_min_width(MENU_WIDTH);
                m.item(ui, Some(icons::FOCUS), "Focus Selection", Action::FocusSelection);
                m.toggle(ui, "Transform Gizmo", self.state.prefs.transform_gizmo, Action::ToggleTransformGizmo);
                m.toggle(ui, "Godot Overlays", self.state.prefs.godot_overlays, Action::ToggleGodotOverlays);
                ui.separator();
                sub_menu(ui, None, "Views", |ui| {
                    let open = open_views(&self.dock);
                    for (i, vp) in self.viewports.iter().enumerate() {
                        m.toggle(ui, vp.kind().label(), open.contains(&i), Action::ToggleView(i));
                    }

                    ui.separator();
                    for (n, label) in [(1u8, "Single View"), (2, "Two Views"), (4, "Four Views")] {
                        let current = self.maximized.is_none() && open.len() == n as usize;
                        if ui.add(menu_button(current.then_some(icons::CHECK), label, m.shortcut(&Action::ViewLayout(n)))).clicked() {
                            m.actions.push(Action::ViewLayout(n));
                            ui.close();
                        }
                    }

                    ui.separator();
                    m.toggle(ui, "Maximize View", self.maximized.is_some(), Action::ToggleMaximizeView);
                });
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
                sub_menu(ui, None, "Interface Scale", |ui| {
                    m.item(ui, None, "Increase", Action::UiScaleUp);
                    m.item(ui, None, "Decrease", Action::UiScaleDown);
                    m.item(ui, None, "Reset", Action::UiScaleReset);
                    ui.separator();
                    for scale in UI_SCALE_PRESETS {
                        let current = (self.state.prefs.ui_scale - scale).abs() < 0.001;
                        if ui.add(menu_button(current.then_some(icons::CHECK), &format!("{:.0}%", scale * 100.0), None)).clicked() {
                            self.state.prefs.ui_scale = scale;
                            ui.close();
                        }
                    }
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
            menu(ui, "Godot", |ui| {
                ui.set_min_width(MENU_WIDTH);
                let found = self.state.godot.found();
                m.item_enabled(ui, Some(icons::PLAY), "Run Project", Action::RunGodotProject, found, commands::GODOT_NOT_FOUND);
                m.item_enabled(ui, Some(icons::GODOT), "Open Project in Godot Editor", Action::OpenGodotEditor, found, commands::GODOT_NOT_FOUND);
                let open = self.state.godot_has_project();
                m.item_enabled(ui, None, "Build in Godot", Action::BuildInGodot, open, "Needs the Godot editor with this project open");
                let live = self.state.prefs.live_mode;
                m.toggle_enabled(ui, "Live Mode", live, Action::ToggleLiveMode, live || self.state.prefs.live_link, commands::LIVE_LINK_OFF);
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
            menu(ui, "Help", |ui| {
                ui.set_min_width(MENU_WIDTH);
                m.item(ui, Some(icons::COMMAND), "Command Palette", Action::ShowCommandPalette);
                m.item(ui, Some(icons::KEYBOARD), "Keyboard Shortcuts…", Action::ShowKeymap);
                m.item(ui, None, "Entity and Code Reference", Action::ShowReference);
                ui.separator();
                ui.label(RichText::new("GodotTrench, a brush and mesh level editor for Godot").strong());
                for line in [
                    "3D: RMB look + WASD fly (Q/E down/up), MMB pan, Alt+LMB orbit",
                    "2D: RMB/MMB pan, wheel zoom, drag edges to resize",
                    "Click selects an object, double click its group. Drag empty space to draw a brush, drag selection to move (Alt vertical, Ctrl duplicate)",
                    "3D gizmo: arrows and squares move, rings rotate, boxes scale",
                    "Shift+click selects faces, Shift+drag a face resizes, Ctrl+Shift+drag extrudes",
                    "Tab edits meshes Blender style: 1/2/3 modes, G/R/S, E extrude, I inset, Ctrl+R loop cut, K knife",
                ] {
                    ui.label(RichText::new(line).weak());
                }
            });
        });
    }

    fn toolbar(&mut self, ui: &mut Ui, bar_height: f32) {
        let ctx = ui.ctx().clone();
        let size = self.toolbar_fit.icon_size(bar_height, ui.available_width(), ui.spacing().button_padding.y * 2.0);
        let mut icon_count = 0;
        let mut tools_rect = egui::Rect::NOTHING;
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
            let mut group = egui::Rect::NOTHING;
            for (icon, label, action) in
                [(icons::NEW, "New Map", Action::NewMap), (icons::OPEN, "Open Map", Action::OpenMap), (icons::SAVE, "Save", Action::Save)]
            {
                let resp = icons::button(ui, icon, size, label, tip(label, &action));
                group |= resp.rect;
                icon_count += 1;
                if resp.clicked() {
                    self.actions.push(action);
                }
            }

            tools_rect |= group;
            group = egui::Rect::NOTHING;
            group_gap(ui);
            let history = [
                (icons::UNDO, "Undo", Action::Undo, self.state.doc.history.can_undo()),
                (icons::REDO, "Redo", Action::Redo, self.state.doc.history.can_redo()),
            ];
            for (icon, label, action, enabled) in history {
                let resp = ui.add_enabled_ui(enabled, |ui| icons::button(ui, icon, size, label, tip(label, &action))).inner;
                group |= resp.rect;
                icon_count += 1;
                if resp.clicked() {
                    self.actions.push(action);
                }
            }

            tools_rect |= group;
            group = egui::Rect::NOTHING;
            group_gap(ui);
            for (i, tools) in ToolKind::GROUPS.iter().enumerate() {
                if i > 0 {
                    ui.add_space(6.0);
                }

                for t in tools.iter().copied() {
                    let tooltip = format!("{}\n{}", tip(&format!("{} tool", t.label()), &Action::SetTool(t)), panels::tool_help(t));
                    let resp = icons::toggle(ui, icons::tool(t), size, self.state.tool == t, t.label(), tooltip);
                    group |= resp.rect;
                    icon_count += 1;
                    if resp.clicked() {
                        self.actions.push(Action::SetTool(t));
                    }
                }
            }

            tools_rect |= group;
            group = egui::Rect::NOTHING;
            group_gap(ui);
            icon_count += 1;
            group |= ui.add(icons::GRID.image(size).tint(ui.visuals().text_color())).on_hover_text("Grid size, [ and ] change it").rect;
            group |= egui::ComboBox::from_id_salt("grid")
                .selected_text(format!("{}", self.state.grid))
                .width(56.0)
                .show_ui(ui, |ui| {
                    for g in [0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0, 512.0, 1024.0] {
                        ui.selectable_value(&mut self.state.grid, g, format!("{g}"));
                    }
                })
                .response
                .rect;
            ui.add_space(2.0);
            let resp = icons::toggle(ui, icons::SNAP, size, self.state.snap, "Snap to grid", tip("Snap to grid", &Action::ToggleSnap));
            group |= resp.rect;
            icon_count += 1;
            if resp.clicked() {
                self.actions.push(Action::ToggleSnap);
            }

            let uv_tip = format!("{}\nTextures stay fixed to faces while moving and rotating", tip("UV lock", &Action::ToggleUvLock));
            let resp = icons::toggle(ui, icons::UV_LOCK, size, self.state.uv_lock, "UV lock", uv_tip);
            group |= resp.rect;
            icon_count += 1;
            if resp.clicked() {
                self.actions.push(Action::ToggleUvLock);
            }

            tools_rect |= group;
            group = egui::Rect::NOTHING;
            group_gap(ui);
            for (icon, label, help, action) in [
                (icons::CSG_SUBTRACT, "CSG subtract", "Carve the selected brushes out of the brushes they touch", Action::CsgSubtract),
                (icons::CSG_MERGE, "CSG convex merge", "Merge the selected brushes into one convex brush", Action::CsgMerge),
                (icons::CSG_INTERSECT, "CSG intersect", "Keep only the volume shared by the selected brushes", Action::CsgIntersect),
                (icons::CSG_HOLLOW, "Hollow", "Turn the selected brushes into walls (thickness in Brush > CSG)", Action::CsgHollow),
            ] {
                let resp = icons::button(ui, icon, size, label, format!("{}\n{help}", tip(label, &action)));
                group |= resp.rect;
                icon_count += 1;
                if resp.clicked() {
                    self.actions.push(action);
                }
            }

            tools_rect |= group;
            group = egui::Rect::NOTHING;
            group_gap(ui);
            for s in Shade::ALL {
                let label = format!("{} shading", capitalize(s.label()));
                let resp = icons::toggle(ui, icons::shade(s), size, self.state.prefs.shade == s, &label, tip(&label, &Action::SetShade(s)));
                group |= resp.rect;
                icon_count += 1;
                if resp.clicked() {
                    self.actions.push(Action::SetShade(s));
                }
            }

            tools_rect |= group;
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let godot_rect = self.godot_buttons(ui, size);
                let project = match &self.state.game.project_root {
                    Some(p) => format!("{} ({})", self.state.game.name, p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()),
                    None => "Open Godot project…".into(),
                };
                let button = egui::Button::image_and_text(icons::OPEN.image(icons::SMALL), RichText::new(project).color(crate::theme::CYAN))
                    .image_tint_follows_text_color(true);
                let resp = ui.add(button).on_hover_text("Open a Godot project folder");
                self.toolbar_fit.project_width = (resp.rect | godot_rect).width();
                if resp.clicked() {
                    self.actions.push(Action::OpenProject);
                }
            });
        });
        self.toolbar_fit.icon = size;
        self.toolbar_fit.icons = icon_count;
        self.toolbar_fit.tools_width = tools_rect.width();
    }

    /// Godot robot button (open the project in Godot, or show the editor that has it open) and the live mode toggle, laid
    /// out right to left. Returns the rect they cover.
    fn godot_buttons(&mut self, ui: &mut Ui, size: f32) -> egui::Rect {
        let s = &self.state;
        let open = s.godot_has_project();
        let (enabled, tip, action) = if s.game.project_root.is_none() {
            (false, "Open a Godot project first".to_string(), None)
        } else if open {
            let version = if s.link_state.godot.is_empty() { String::new() } else { format!(" {}", s.link_state.godot) };
            (true, format!("Godot{version} has this project open, click to show it"), Some(Action::FocusGodot))
        } else if s.godot.found() {
            let hint = if s.link_state.outdated { "\nThe running Godot has an older GodotTrench addon, update it for live mode" } else { "" };
            (true, format!("Open project in Godot{hint}"), Some(Action::OpenGodotEditor))
        } else {
            (false, commands::GODOT_NOT_FOUND.to_string(), None)
        };
        let mut image = icons::GODOT.image(size).alt_text("Open project in Godot");
        if open {
            image = image.tint(crate::theme::CYAN);
        }

        let button = egui::Button::image(image).image_tint_follows_text_color(!open).frame_when_inactive(false);
        let resp = ui.add_enabled(enabled, button).on_hover_text(tip.as_str()).on_disabled_hover_text(tip.as_str());
        let mut rect = resp.rect;
        if resp.clicked()
            && let Some(action) = action
        {
            self.actions.push(action);
        }

        let live = s.prefs.live_mode;
        let live_tip = match (live, open, s.live_active()) {
            (true, _, true) => "Live mode on: edits reach Godot before you save. Click to turn it off",
            (true, true, false) => "Live mode on, waiting for Godot to show a scene that uses this map",
            (true, false, _) => "Live mode on, waiting for the Godot editor with this project",
            (false, _, _) if !s.prefs.live_link => commands::LIVE_LINK_OFF,
            (false, true, _) => "Live mode: send edits to Godot before saving",
            (false, false, _) => "Live mode needs the Godot editor with this project open",
        };
        let toggle = ui.add_enabled_ui(live || (open && s.prefs.live_link), |ui| icons::toggle(ui, icons::LINK, size, live, "Godot live mode", live_tip)).inner;
        let toggle = toggle.on_disabled_hover_text(live_tip);
        rect |= toggle.rect;
        if toggle.clicked() {
            self.actions.push(Action::ToggleLiveMode);
        }

        if self.state.link_state.busy {
            rect |= ui.add(egui::Spinner::new().size(size * 0.8)).on_hover_text("Godot is building").rect;
        }

        ui.add_space(PROJECT_BUTTON_GAP * 0.5);
        rect
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
                    let current = active.and_then(|a| sets.iter().find(|(id, _)| *id == a)).map(|(_, n)| n.clone()).unwrap_or_else(|| "none".into());
                    ui.label("Set");
                    egui::ComboBox::from_id_salt("scatter_set").selected_text(current).width(140.0).show_ui(ui, |ui| {
                        for (id, name) in &sets {
                            if ui.selectable_label(active == Some(*id), name).clicked() {
                                self.actions.push(Action::ActivateScatter(*id));
                            }
                        }

                        ui.separator();
                        if ui.selectable_label(false, "New empty set").clicked() {
                            self.actions.push(Action::NewScatterSet);
                        }
                    });
                    let erase = self.state.scatter_erase;
                    if ui.selectable_label(!erase, "Paint").clicked() {
                        self.state.scatter_erase = false;
                    }

                    if ui.selectable_label(erase, "Erase").clicked() {
                        self.state.scatter_erase = true;
                    }

                    let s = &mut self.state.prefs.scatter;
                    ui.add(egui::DragValue::new(&mut s.radius).range(8.0..=16384.0).prefix("radius "));
                    ui.add(egui::DragValue::new(&mut s.rules.density).range(0.01..=64.0).speed(0.05).prefix("density "));
                    ui.add(egui::DragValue::new(&mut s.rules.slope[1]).range(0.0..=90.0).prefix("max slope ").suffix("°"));
                    let mut follow = !s.rules.only_targets;
                    if ui
                        .checkbox(&mut follow, "Follow cursor")
                        .on_hover_text("Paint on any surface or scattered prop under the brush instead of the set's targets")
                        .changed()
                    {
                        s.rules.only_targets = !follow;
                    }

                    let picking = self.state.scatter_eyedropper;
                    if icons::toggle(ui, icons::EYEDROPPER, 16.0, picking, "Pick target", "The next click in a view adds or removes a target, like Alt+click")
                        .clicked()
                    {
                        self.state.scatter_eyedropper = !picking;
                    }

                    ui.separator();
                    if ui.small_button("Scatter panel").on_hover_text("Models, targets and brush of the active set").clicked() {
                        self.actions.push(Action::ShowScatterPanel);
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

                    let p = &mut self.state.prefs;
                    ui.add(egui::DragValue::new(&mut p.blend_uv_scale).range(0.01..=64.0).speed(0.05).prefix("repeat "))
                        .on_hover_text("How much more often the blend material repeats across the face");
                    ui.add(egui::DragValue::new(&mut p.blend_detile).range(0.0..=1.0).speed(0.02).prefix("detile "))
                        .on_hover_text("Turns and shifts every tile and blends the joins, so a path does not look repeated");
                    ui.add(egui::DragValue::new(&mut p.blend_detile_sharpen).range(0.0..=1.0).speed(0.02).prefix("sharpen "))
                        .on_hover_text("Keeps the de-tiled texture crisp: 0 mixes the tiles evenly and looks soft, 1 mixes only where they join");
                    let (detile, uv_scale, sharpen) = (p.blend_detile, p.blend_uv_scale, p.blend_detile_sharpen);
                    if ui.small_button("Apply Tiling").on_hover_text("Give the selected faces the repeat and de-tiling above").clicked() {
                        self.actions.push(Action::SetBlendTiling { detile, uv_scale, sharpen });
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

                if ui.small_button("×").on_hover_text(format!("Close {title}")).clicked() {
                    self.state.switch_tab(i);
                    self.actions.push(Action::CloseTab);
                }

                ui.add_space(6.0);
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
            ui.label(RichText::new(&s.status).color(crate::theme::FG.gamma_multiply_u8(alpha)));
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
                    ui.label(RichText::new("cordon").color(crate::theme::YELLOW));
                }

                if !s.open_groups.is_empty() {
                    ui.separator();
                    ui.label(RichText::new(format!("{} open groups", s.open_groups.len())).color(crate::theme::CYAN));
                }
            });
        });
    }

    fn apply_ui_scale(&self, ctx: &egui::Context) {
        let Some(native) = ctx.native_pixels_per_point() else { return };
        let zoom = self.state.prefs.ui_zoom_factor(native);
        if (ctx.zoom_factor() - zoom).abs() > 0.001 {
            ctx.set_zoom_factor(zoom);
        }
    }

    /// A saved or default window size can be larger than a scaled monitor, which pushes panels off screen.
    fn fit_window_to_monitor(&mut self, ctx: &egui::Context) {
        if self.window_fitted || !self.keep_prefs {
            return;
        }

        let (monitor, outer, maximized) = ctx.input(|i| (i.viewport().monitor_size, i.viewport().outer_rect, i.viewport().maximized));
        let (Some(monitor), Some(outer)) = (monitor, outer) else { return };
        self.window_fitted = true;
        if maximized != Some(true) && (outer.width() > monitor.x * 0.98 || outer.height() > monitor.y * 0.95) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        }
    }

    fn ui_scale_prefs(&mut self, ui: &mut Ui) {
        let native = ui.ctx().native_pixels_per_point().unwrap_or(1.0);
        let p = &mut self.state.prefs;
        ui.label("Interface scale");
        ui.horizontal(|ui| {
            let mut value = self.ui_scale_draft.unwrap_or(p.ui_scale);
            let slider = egui::Slider::new(&mut value, crate::state::UI_SCALE_MIN..=crate::state::UI_SCALE_MAX)
                .step_by(0.05)
                .custom_formatter(|v, _| format!("{:.0}%", v * 100.0))
                .custom_parser(|s| s.trim().trim_end_matches('%').trim().parse::<f64>().ok().map(|v| v / 100.0));
            let response = ui.add(slider);
            if response.dragged() {
                self.ui_scale_draft = Some(value);
            } else {
                if response.changed() || self.ui_scale_draft.is_some() {
                    p.ui_scale = value;
                }

                self.ui_scale_draft = None;
            }

            if ui.button("Reset").clicked() {
                p.ui_scale = 1.0;
                p.follow_display_scaling = true;
            }
        });
        ui.end_row();
        ui.label("Display scaling");
        let mut follow = p.follow_display_scaling;
        ui.checkbox(&mut follow, format!("follow the monitor ({:.0}%)", native * 100.0))
            .on_hover_text("When off, the interface scale is exact pixels per point, for monitors that report the wrong scaling");
        p.set_follow_display_scaling(follow, native);
        ui.end_row();
    }

    fn prefs_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_prefs;
        egui::Window::new("Preferences").open(&mut open).resizable(false).show(ctx, |ui| {
            egui::Grid::new("prefs_ui").num_columns(2).min_col_width(PREFS_LABEL_WIDTH).show(ui, |ui| self.ui_scale_prefs(ui));
            ui.separator();
            let p = &mut self.state.prefs;
            egui::Grid::new("prefs").num_columns(2).min_col_width(PREFS_LABEL_WIDTH).show(ui, |ui| {
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
                ui.label("");
                match &self.state.godot.exe {
                    Some(exe) => ui.label(RichText::new(format!("Using {}", exe.display())).weak()),
                    None => ui.label(RichText::new("Not found, Run Project and Open in Godot are disabled").color(crate::theme::YELLOW)),
                };
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
                ui.label("Godot live mode");
                ui.add_enabled_ui(p.live_link, |ui| ui.checkbox(&mut p.live_mode, "push edits to Godot before saving"))
                    .response
                    .on_hover_text("While the Godot editor shows a scene that uses the map, edits appear there right away. Saving still does a full rebuild.");
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

        let modified = self.state.modified_tabs();
        if modified.is_empty() {
            self.confirm_close = false;
            self.allow_close = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        egui::Modal::new(egui::Id::new("confirm_close")).show(ctx, |ui| {
            ui.heading("Unsaved changes");
            ui.label(if modified.len() == 1 { "This map has unsaved changes:" } else { "These maps have unsaved changes:" });
            for i in &modified {
                ui.label(RichText::new(self.state.tab_doc(*i).title()).strong());
            }

            ui.horizontal(|ui| {
                let save = if modified.len() == 1 { "Save" } else { "Save All" };
                if ui.button(save).clicked() {
                    if commands::save_all(&mut self.state, ctx) {
                        self.allow_close = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    } else {
                        self.state.set_status("Not every map was saved, quitting was cancelled");
                    }

                    self.confirm_close = false;
                }

                let discard = if modified.len() == 1 { "Discard" } else { "Discard All" };
                if ui.button(discard).clicked() {
                    self.state.revert_all_live_changes();
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

    fn close_tab_dialog(&mut self, ctx: &egui::Context) {
        if !self.confirm_tab_close {
            return;
        }

        if !self.state.doc.is_modified() {
            self.confirm_tab_close = false;
            return;
        }

        egui::Modal::new(egui::Id::new("confirm_tab_close")).show(ctx, |ui| {
            ui.heading("Unsaved changes");
            ui.label(format!("Save changes to {} before closing its tab?", self.state.doc.title()));
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    commands::execute(&mut self.state, Action::Save, ctx);
                    if !self.state.doc.is_modified() {
                        commands::execute(&mut self.state, Action::CloseTab, ctx);
                    }

                    self.confirm_tab_close = false;
                }

                if ui.button("Discard").clicked() {
                    self.state.discard_tab();
                    self.confirm_tab_close = false;
                }

                if ui.button("Cancel").clicked() {
                    self.confirm_tab_close = false;
                }
            });
        });
    }
}

struct Tabs<'a> {
    views_open: usize,
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

    fn on_tab_button(&mut self, tab: &mut Tab, response: &egui::Response) {
        if let Some(panel) = tab_panel(*tab) {
            response.clone().on_hover_text(panel.help());
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
            Tab::Models => panels::model_browser(ui, self.state, self.panels, self.actions, Some(&mut *self.renderer)),
            Tab::Entities => panels::entity_browser(ui, self.state, self.panels, self.actions),
            Tab::History => panels::history(ui, self.state),
            Tab::Issues => panels::issues(ui, self.state, self.panels, self.actions),
            Tab::Logic => panels::logic_panel(ui, self.state, self.panels),
            Tab::Uv => panels::uv_editor(ui, self.state, self.panels, self.actions),
            Tab::Reference => panels::reference(ui, self.state, self.panels, self.actions),
            Tab::Scatter => crate::scatter_panel::scatter_panel(ui, self.state, self.actions, Some(&mut *self.renderer)),
        }
    }

    fn id(&mut self, tab: &mut Tab) -> egui::Id {
        egui::Id::new(("tab", format!("{tab:?}")))
    }

    fn is_closeable(&self, tab: &Tab) -> bool {
        view_closeable(tab, self.views_open)
    }

    fn closeable(&mut self, tab: &mut Tab) -> bool {
        view_closeable(tab, self.views_open)
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
        self.apply_ui_scale(&ctx);
        self.fit_window_to_monitor(&ctx);
        self.tools.sync(&self.state);
        self.state.tick_godot(&ctx);
        self.process_mcp(&ctx);
        self.state.validate_insert_context();
        self.tools.sync(&self.state);
        self.collect_input_actions(&ctx);

        egui::Panel::top("menu").show(ui, |ui| self.menu_bar(ui));
        let margin = bar_margin(&ctx.global_style());
        let padding = ctx.global_style().spacing.button_padding.y * 2.0;
        let toolbar = egui::Panel::top(TOOLBAR_ID).resizable(true).max_size(self.toolbar_fit.max_height(padding) + margin).show(ui, |ui| {
            let bar_height = ui.available_height();
            bar_contents(ui, |ui| self.toolbar(ui, bar_height))
        });
        self.toolbar_fit.content_height = toolbar.inner;
        let options_max = TOOL_OPTIONS_MAX_HEIGHT.max(self.tool_options_height) + margin;
        let options = egui::Panel::top(TOOL_OPTIONS_ID).resizable(true).max_size(options_max).show(ui, |ui| bar_contents(ui, |ui| self.tool_options(ui)));
        self.tool_options_height = options.inner;
        if !self.state.tabs.is_empty() {
            egui::Panel::top("map_tabs").show(ui, |ui| self.tab_bar(ui));
        }

        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));

        if self.state.materials.changed_on_disk() {
            self.state.material_reload = true;
        }

        if std::mem::take(&mut self.state.material_reload) {
            let game = self.state.game.clone();
            self.state.materials.rescan(&game);
            self.state.materials_rescanned = true;
        }

        if std::mem::take(&mut self.state.materials_rescanned) {
            self.project_generation += 1;
        }

        if std::mem::take(&mut self.state.scene_reset) {
            self.scene.invalidate();
            self.tools.reset();
        }

        self.scene.update(&mut self.renderer, &mut self.state, self.project_generation);
        let cam = &self.viewports[0].camera;
        let focus = cam.position + cam.forward() * 1200.0;
        self.scene.update_shadows(&mut self.renderer, focus, self.state.prefs.shade == Shade::Lit);

        // The viewports read this to know a ctrl+click should grab all of a brush's faces for UV work. The UV
        // Editor is docked by default, so only its being the visible tab counts, else ctrl+click could never
        // add objects to the selection.
        self.state.uv_panel_open = self
            .dock
            .find_tab(&Tab::Uv)
            .is_some_and(|p| self.dock.leaf(egui_dock::NodePath { surface: p.surface, node: p.node }).is_ok_and(|leaf| leaf.active == p.tab));
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ui, |ui| {
            let mut tabs = Tabs {
                views_open: open_views(&self.dock).len(),
                state: &mut self.state,
                renderer: &mut self.renderer,
                scene: &self.scene,
                viewports: &mut self.viewports,
                panels: &mut self.panels,
                tools: &mut self.tools,
                actions: &mut self.actions,
            };
            DockArea::new(&mut self.dock).show_leaf_collapse_buttons(false).show_inside(ui, &mut tabs);
            // Closing every tab of a leaf at once can take the last view with it.
            let open = open_views(&self.dock);
            if open.is_empty() {
                self.dock.push_to_first_leaf(Tab::View(0));
            }

            if let Some(hovered) = open.into_iter().find(|i| self.viewports.get(*i).is_some_and(|v| v.hovered)) {
                self.active_view = hovered;
            }

            view_grid_corner(ui, &mut self.dock);
        });

        self.palette.show(&ctx, &self.state, &mut self.actions);
        self.shape_dialog.show(&ctx, &mut self.state);
        self.terrain_dialog.show(&ctx, &mut self.state);
        self.keymap.show(&ctx, &mut self.state);
        self.hotspot_editor.show(&ctx, &mut self.state, &mut self.actions);
        self.link_dialog.show(&ctx, &mut self.state);
        panels::dnd_preview(&ctx, &mut self.state);

        for action in std::mem::take(&mut self.actions) {
            let Some(action) = self.run_app_action(action) else { continue };
            let project_before = self.state.game.project_root.clone();
            commands::execute(&mut self.state, action, &ctx);
            if self.state.game.project_root != project_before {
                self.project_generation += 1;
            }
        }

        self.sync_active_scatter();
        if self.state.tool != self.last_tool {
            if self.state.tool == ToolKind::Scatter {
                reveal_tab(&mut self.dock, Tab::Scatter, Tab::Inspector);
            } else {
                self.state.scatter_eyedropper = false;
            }

            self.last_tool = self.state.tool;
        }

        if let Some(bounds) = self.state.focus_request.take() {
            for v in &mut self.viewports {
                v.focus(&bounds);
            }
        }

        self.prefs_window(&ctx);
        self.close_dialog(&ctx);
        self.close_tab_dialog(&ctx);
        self.state.tick_autosave();
        self.state.poll_live_link();
        self.finish_input_script(&ctx);

        let title = format!("{} - GodotTrench", self.state.doc.title());
        if title != self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    fn persist_egui_memory(&self) -> bool {
        self.keep_prefs
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if self.keep_prefs {
            eframe::set_value(storage, "prefs", &self.state.prefs);
            // A maximized view is a temporary state, the next launch starts from the layout it replaced.
            eframe::set_value(storage, DOCK_KEY, self.maximized.as_ref().unwrap_or(&self.dock));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_layouts_show_one_two_or_four_views() {
        let mut dock = default_dock();
        assert_eq!(open_views(&dock).len(), 4);
        let panels = |d: &DockState<Tab>| d.iter_all_tabs().filter(|(_, t)| !matches!(t, Tab::View(_))).count();
        let panel_count = panels(&dock);

        set_views(&mut dock, &[2]);
        assert_eq!(open_views(&dock), [2], "a single view fills the view area");
        assert_eq!(panels(&dock), panel_count, "the panels stay where they were");
        assert!(view_grid(dock.main_surface()).is_none());

        set_views(&mut dock, &[0, 1]);
        let mut open = open_views(&dock);
        open.sort_unstable();
        assert_eq!(open, [0, 1]);

        set_views(&mut dock, &[0, 1, 2, 3]);
        assert!(view_grid(dock.main_surface()).is_some(), "four views make the same 2x2 grid as the default, so the grid corner drags them");
        let mut open = open_views(&dock);
        open.sort_unstable();
        assert_eq!(open, [0, 1, 2, 3]);
        assert_eq!(panels(&dock), panel_count);
        assert!(valid_dock(&dock));
    }

    #[test]
    fn view_tabs_close_while_another_view_is_open() {
        assert!(view_closeable(&Tab::View(1), 4), "the Top, Front and Side tabs close like panels");
        assert!(!view_closeable(&Tab::View(0), 1), "the last view never closes");
        assert!(view_closeable(&Tab::Outliner, 1));

        let mut dock = default_dock();
        let path = dock.find_tab(&Tab::View(3)).unwrap();
        dock.remove_tab(path);
        assert_eq!(open_views(&dock).len(), 3);
        assert!(valid_dock(&dock), "a layout with a closed view is still saved and restored");
        let mut open = open_views(&dock);
        open.push(3);
        open.sort_unstable();
        set_views(&mut dock, &open);
        assert!(dock.find_tab(&Tab::View(3)).is_some(), "View > Views brings it back");
        assert!(view_grid(dock.main_surface()).is_some());
    }

    #[test]
    fn mesh_menu_groups_cover_every_op() {
        let grouped: Vec<MeshOp> = MESH_OP_GROUPS.iter().flat_map(|(_, ops)| ops.iter().copied()).collect();
        assert_eq!(grouped.len(), MeshOp::ALL.len());
        assert!(MeshOp::ALL.iter().all(|op| grouped.contains(op)));
    }

    #[test]
    fn saved_dock_layouts_round_trip_and_bad_ones_are_rejected() {
        let mut dock = default_dock();
        if let Node::Horizontal(split) = &mut dock.main_surface_mut()[NodeIndex::root()] {
            split.fraction = 0.6;
        }

        let text = ron::to_string(&dock).unwrap();
        let loaded: DockState<Tab> = ron::from_str(&text).unwrap();
        assert!(valid_dock(&loaded));
        assert!(matches!(&loaded.main_surface()[NodeIndex::root()], Node::Horizontal(s) if s.fraction == 0.6));
        assert_eq!(loaded.iter_all_tabs().count(), dock.iter_all_tabs().count());

        assert!(!valid_dock(&DockState::new(vec![Tab::View(7)])));
        assert!(!valid_dock(&DockState::new(vec![Tab::View(0), Tab::View(0)])));
        assert!(!valid_dock(&DockState::new(vec![Tab::Outliner])));
    }

    #[test]
    fn migration_adds_the_models_tab_next_to_materials() {
        // A layout saved before the Models tab existed: Materials but no Models.
        let mut dock = DockState::new(vec![Tab::View(0)]);
        dock.main_surface_mut().split_below(NodeIndex::root(), 0.7, vec![Tab::Materials]);
        assert!(dock.find_tab(&Tab::Models).is_none());
        ensure_tab(&mut dock, Tab::Models, Tab::Materials);
        let materials = dock.find_tab(&Tab::Materials).expect("materials still there");
        let models = dock.find_tab(&Tab::Models).expect("models added");
        assert_eq!((materials.surface, materials.node), (models.surface, models.node), "models sits in the materials leaf");
        // Running it again does not duplicate the tab.
        ensure_tab(&mut dock, Tab::Models, Tab::Materials);
        assert_eq!(dock.iter_all_tabs().filter(|(_, t)| **t == Tab::Models).count(), 1);
    }

    #[test]
    fn finds_the_view_grid_in_the_default_layout() {
        let dock = default_dock();
        let tree = dock.main_surface();
        let [h, left, right] = view_grid(tree).expect("default layout has a 2x2 view grid");
        assert!(matches!(tree[h], Node::Horizontal(_)));
        assert!(matches!((&tree[left], &tree[right]), (Node::Vertical(_), Node::Vertical(_))));
        let views: Vec<Tab> = [left.left(), left.right(), right.left(), right.right()].iter().flat_map(|i| tree[*i].tabs().unwrap().to_vec()).collect();
        assert_eq!(views, vec![Tab::View(0), Tab::View(2), Tab::View(1), Tab::View(3)]);
        assert!(view_grid(DockState::new(vec![Tab::View(0)]).main_surface()).is_none());
    }

    #[test]
    fn grid_corner_keeps_views_visible() {
        assert_eq!(fraction_at(100.0, 1000.0, 600.0), 0.5);
        assert_eq!(fraction_at(100.0, 1000.0, 0.0), MIN_VIEW_SIZE / 1000.0);
        assert_eq!(fraction_at(100.0, 1000.0, 5000.0), 1.0 - MIN_VIEW_SIZE / 1000.0);
        assert_eq!(fraction_at(0.0, 100.0, 90.0), 0.5);
    }

    #[test]
    fn toolbar_icons_follow_the_bar_height_within_limits() {
        let fit = ToolbarFit { icon: 18.0, icons: 30, tools_width: 900.0, project_width: 150.0, content_height: 20.0 };
        assert_eq!(fit.icon_size(20.0, 2000.0, 2.0), icons::TOOLBAR, "default bar height keeps the default size");
        assert_eq!(fit.icon_size(32.0, 2000.0, 2.0), 30.0);
        assert_eq!(fit.icon_size(400.0, 2000.0, 2.0), icons::TOOLBAR_MAX, "capped");
        // 2000 wide leaves 934 spare points, 31 more per icon, so a narrower window limits the size before the height does.
        assert_eq!(fit.icon_size(400.0, 1300.0, 2.0), 25.0);
        assert_eq!(fit.icon_size(400.0, 600.0, 2.0), icons::TOOLBAR, "never below the default, the toolbar wraps instead");
        assert_eq!(fit.max_height(2.0), icons::TOOLBAR_MAX + 2.0);
        let wrapped = ToolbarFit { content_height: 120.0, ..fit };
        assert_eq!(wrapped.max_height(2.0), 120.0, "wrapped rows are never clipped");
        assert_eq!(ToolbarFit { icon: 30.0, ..wrapped }.max_height(2.0), icons::TOOLBAR_MAX + 2.0);
    }

    #[test]
    fn capitalizes_labels() {
        assert_eq!(capitalize("wireframe"), "Wireframe");
        assert_eq!(capitalize(""), "");
    }
}
