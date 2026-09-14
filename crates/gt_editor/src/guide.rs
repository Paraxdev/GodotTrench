//! Built in help: a spotlight tour over the live interface, the Guide window that shows the same chapters as
//! searchable pages, and tips.
//!
//! Widgets register their screen rects as [`Anchor`]s with [`mark`] while they are drawn, the tour reads them back at the
//! end of the frame to cut a hole into the dimmed window and to place its popover next to them.

use egui::{Align, Color32, Id, Layout, Order, Pos2, Rect, RichText, ScrollArea, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};

use crate::commands::{self, Action};
use crate::icons;
use crate::panels::tool_help;
use crate::state::{EditorState, Shade};
use crate::tools::ToolKind;

const ACCENT: Color32 = crate::theme::ACCENT;
const DONE: Color32 = crate::theme::SUCCESS;
const POPOVER_WIDTH: f32 = 360.0;

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
}

impl Panel {
    pub const ALL: [Panel; 8] =
        [Panel::Outliner, Panel::Inspector, Panel::Materials, Panel::Entities, Panel::History, Panel::Issues, Panel::Uv, Panel::Reference];

    pub fn title(self) -> &'static str {
        match self {
            Panel::Outliner => "Outliner",
            Panel::Inspector => "Inspector",
            Panel::Materials => "Materials",
            Panel::Entities => "Entities",
            Panel::History => "History",
            Panel::Issues => "Issues",
            Panel::Uv => "UV Editor",
            Panel::Reference => "Reference",
        }
    }

    /// One or two sentences on what the panel is for, used by the tour and the panel tab tooltips.
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
            Panel::Reference => {
                "How to use the selected entity from code: GDScript and C# snippets, the FGD resource, and buttons that create the script in your project. Drag the divider to resize the class list, double click it to fit the names."
            }
        }
    }
}

/// A part of the interface the tour can point at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Anchor {
    /// Nothing in particular, the popover sits in the middle of the window.
    Screen,
    MenuBar,
    Menu(&'static str),
    ToolbarFile,
    ToolbarHistory,
    ToolbarTools,
    Tool(ToolKind),
    ToolbarGrid,
    ToolbarCsg,
    ToolbarShading,
    ToolbarProject,
    ToolOptions,
    StatusBar,
    View3d,
    Views2d,
    Panel(Panel),
}

fn anchor_id(anchor: Anchor) -> Id {
    Id::new(("guide_anchor", anchor))
}

/// Records where `anchor` is drawn this frame. Marking the same anchor several times in a frame (the three 2D views)
/// keeps every rect.
pub fn mark(ctx: &egui::Context, anchor: Anchor, rect: Rect) {
    if !rect.is_positive() {
        return;
    }
    let frame = ctx.cumulative_frame_nr();
    ctx.data_mut(|d| {
        let slot = d.get_temp_mut_or_insert_with(anchor_id(anchor), || (frame, Vec::<Rect>::new()));
        if slot.0 != frame {
            *slot = (frame, Vec::new());
        }
        slot.1.push(rect);
    });
}

/// Rects of `anchor` drawn this frame or the previous one. Empty when it is not on screen, like a closed panel.
pub fn anchor_rects(ctx: &egui::Context, anchor: Anchor) -> Vec<Rect> {
    let frame = ctx.cumulative_frame_nr();
    ctx.data(|d| d.get_temp::<(u64, Vec<Rect>)>(anchor_id(anchor))).filter(|(f, _)| f + 1 >= frame).map(|(_, r)| r).unwrap_or_default()
}

pub struct Task {
    pub label: &'static str,
    pub done: fn(&EditorState) -> bool,
}

pub struct Step {
    pub anchor: Anchor,
    pub title: String,
    pub body: String,
    /// Shortcuts shown with the user's current key bindings.
    pub keys: Vec<(&'static str, Action)>,
    pub button: Option<(&'static str, Action)>,
    /// Something to try, checked live against the editor state.
    pub task: Option<Task>,
}

fn step(anchor: Anchor, title: impl Into<String>, body: impl Into<String>) -> Step {
    Step { anchor, title: title.into(), body: body.into(), keys: Vec::new(), button: None, task: None }
}

impl Step {
    fn keys(mut self, keys: impl IntoIterator<Item = (&'static str, Action)>) -> Self {
        self.keys.extend(keys);
        self
    }

    fn button(mut self, label: &'static str, action: Action) -> Self {
        self.button = Some((label, action));
        self
    }

    fn task(mut self, label: &'static str, done: fn(&EditorState) -> bool) -> Self {
        self.task = Some(Task { label, done });
        self
    }

    fn matches(&self, query: &str) -> bool {
        self.title.to_lowercase().contains(query) || self.body.to_lowercase().contains(query) || self.keys.iter().any(|(k, _)| k.to_lowercase().contains(query))
    }
}

pub struct Chapter {
    pub id: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub steps: Vec<Step>,
}

pub const TIPS: [&str; 16] = [
    "Hover any toolbar button to see its shortcut and a short explanation.",
    "The command palette (Ctrl+Shift+P or F1) runs any command by typing a few letters of its name.",
    "Shift+click selects a single face, Ctrl+Shift+click adds more faces.",
    "Ctrl+click adds objects to the selection or removes them from it.",
    "Hold Ctrl while dragging the selection to drag out a copy, Alt moves it vertically in the 3D view.",
    "F frames the selection in every view.",
    "[ and ] change the grid size.",
    "With the texture tool, Alt+click picks up a face's material and alignment and right click applies it elsewhere.",
    "Duplicate Linked (Ctrl+Shift+D) keeps copies identical, perfect for repeated windows, pillars or lamps.",
    "Ctrl+Shift+1 to 9 stores a camera bookmark, Ctrl+1 to 9 jumps back to it.",
    "Ctrl+H hides the selection, Ctrl+J isolates it, Edit > Hide and Lock > Show All brings everything back.",
    "The Issues panel catches broken brushes and outputs without a target before Godot does.",
    "Save while the Godot editor is open and it rebuilds the map there automatically.",
    "Ctrl+= and Ctrl+- scale the whole interface, the size is remembered.",
    "Opening a map that lives inside a Godot project loads that project too.",
    "Shortcuts in these tips are the TrenchBroom preset, Help > Keyboard Shortcuts shows and changes yours.",
];

fn has_project(s: &EditorState) -> bool {
    s.game.project_root.is_some()
}

fn has_brush(s: &EditorState) -> bool {
    s.doc.map.brush_count() > 0
}

fn has_selection(s: &EditorState) -> bool {
    !s.doc.selection.nodes.is_empty()
}

fn has_face_selection(s: &EditorState) -> bool {
    s.doc.selection.has_faces()
}

fn has_entity(s: &EditorState) -> bool {
    s.doc.map.entity_count() > 0
}

fn has_layers(s: &EditorState) -> bool {
    s.doc.map.layers.len() > 1
}

fn has_terrain(s: &EditorState) -> bool {
    s.doc.map.terrains().next().is_some()
}

fn is_saved(s: &EditorState) -> bool {
    s.doc.path.is_some() && !s.doc.is_modified()
}

fn saved_in_project(s: &EditorState) -> bool {
    match (&s.doc.path, &s.game.project_root) {
        (Some(path), Some(root)) => path.starts_with(root),
        _ => false,
    }
}

/// Why a tool is worth using, shown above its short how-to.
fn tool_purpose(tool: ToolKind) -> &'static str {
    match tool {
        ToolKind::Select => "The default tool for drawing, selecting, moving and resizing brushes.",
        ToolKind::Clip => "Cuts brushes along a plane, for slopes, ramps and angled walls.",
        ToolKind::Vertex => "Moves single corners and edges for shapes a box cannot make. Brushes always stay convex.",
        ToolKind::Rotate => "Rotates the selection around its center.",
        ToolKind::Scale => "Stretches the selection by its bounding box.",
        ToolKind::Mesh => "Turns brushes into polygon meshes you edit like in Blender, for organic or detailed shapes that do not have to be convex.",
        ToolKind::Texture => "Hammer style texturing right in the 3D view.",
        ToolKind::Paint => "Paints vertex colors on faces, for dirt, tint or fake lighting.",
        ToolKind::Sculpt => "Shapes terrains and displacements.",
        ToolKind::Blend => "Mixes materials: terrain layers, displacement alpha and faces that have a blend material.",
        ToolKind::Scatter => "Paints trees, rocks and foliage onto the surfaces you choose.",
        ToolKind::Volume => "Drags out trigger and gameplay volumes in one step.",
        ToolKind::Path => "Lays down path_corner chains for trains and moving platforms.",
        ToolKind::Measure => "Measures the distance between two points.",
    }
}

fn tool_action(tool: ToolKind) -> Action {
    if tool == ToolKind::Mesh { Action::EditMesh } else { Action::SetTool(tool) }
}

fn tool_step(tool: ToolKind) -> Step {
    step(Anchor::Tool(tool), format!("{} tool", tool.label()), format!("{}\n\n{}", tool_purpose(tool), tool_help(tool)))
        .keys([("Shortcut", tool_action(tool))])
        .button("Try it", tool_action(tool))
}

pub fn chapters() -> Vec<Chapter> {
    let mut list = vec![
        Chapter {
            id: "welcome",
            title: "Welcome",
            summary: "What GodotTrench is and how this guide works.",
            steps: vec![
                step(
                    Anchor::Screen,
                    "Welcome to GodotTrench",
                    "GodotTrench is a level editor for Godot in the spirit of TrenchBroom and Hammer. You build maps from brushes, meshes, terrains and entities, save them as .gtm files inside your Godot project, and the FuncGodot (GodotTrench) addon turns them into scenes.\n\nThis tour points at the real interface. Next and Back move through it and the list at the top jumps to any chapter, they do not depend on each other. The tour never locks the editor, so try every step while it is open.",
                ),
                step(Anchor::Screen, "Chapters", String::new()),
                step(
                    Anchor::Menu("Help"),
                    "Find help again",
                    "Help > Guided Tour brings this tour back and Help > Guide shows every chapter as a page you can search and keep open next to your work. The command palette finds any command by name.",
                )
                .keys([("Command palette", Action::ShowCommandPalette), ("Keyboard shortcuts", Action::ShowKeymap)]),
            ],
        },
        Chapter {
            id: "interface",
            title: "The window",
            summary: "What every part of the window is for.",
            steps: vec![
                step(
                    Anchor::MenuBar,
                    "Menu bar",
                    "Every command lives in these menus, grouped by what it works on: File, Edit, Brush, Mesh, Texture, Terrain, Gameplay, Tools, View, Godot and Help. Entries show their keyboard shortcut on the right.",
                ),
                step(
                    Anchor::ToolbarFile,
                    "New, open and save",
                    "Maps are .gtm files. Save them inside your Godot project so the addon can build them. Several maps can be open at once as tabs (File > Tabs).",
                )
                .keys([("New map", Action::NewMap), ("Open map", Action::OpenMap), ("Save", Action::Save)]),
                step(Anchor::ToolbarHistory, "Undo and redo", "Every change can be undone, the History panel lists them all.")
                    .keys([("Undo", Action::Undo), ("Redo", Action::Redo)]),
                step(
                    Anchor::ToolbarTools,
                    "Tools",
                    "The active tool decides what the mouse does in the views. Hover a tool for its shortcut and a short how-to, the Tools chapter goes through each of them.",
                ),
                step(
                    Anchor::ToolbarGrid,
                    "Grid, snap and UV lock",
                    "The number is the grid size in map units. The magnet turns snapping to the grid on and off, the lock keeps textures fixed to faces while you move and rotate brushes.\n\nBy default 32 map units are one meter in Godot, so a 128 unit tall room is 4 meters high.",
                )
                .keys([("Larger grid", Action::GridUp), ("Smaller grid", Action::GridDown), ("Snap", Action::ToggleSnap)]),
                step(
                    Anchor::ToolbarCsg,
                    "CSG",
                    "Subtract carves the selected brushes out of the brushes they touch, Merge joins the selection into one convex brush, Intersect keeps only the overlap and Hollow turns brushes into walls.",
                ),
                step(
                    Anchor::ToolbarShading,
                    "Shading",
                    "Textured, flat colors, lit preview (the map's lights, sun, sky and fog, close to what Godot shows) and wireframe.",
                ),
                step(
                    Anchor::ToolbarProject,
                    "Current Godot project",
                    "The open Godot project. Click to open another one. Without a project the editor still works, with the built in entities and dev textures.",
                ),
                step(
                    Anchor::ToolOptions,
                    "Tool options",
                    "Settings of the active tool: the brush radius for sculpting, the class for volumes, justify buttons for textures and so on. The name on the left shows how the tool works when you hover it.",
                ),
                step(Anchor::View3d, "3D view", "The perspective view. Hold the right mouse button to look around and fly with WASD."),
                step(Anchor::Views2d, "2D views", "Top, front and side views. They are the precise way to draw and resize brushes on the grid."),
            ],
        },
    ];
    let panel_steps: Vec<Step> =
        [Panel::Outliner, Panel::History, Panel::Issues, Panel::Inspector, Panel::Entities, Panel::Uv, Panel::Materials, Panel::Reference]
            .into_iter()
            .map(|p| step(Anchor::Panel(p), p.title(), p.help()))
            .collect();
    list[1].steps.extend(panel_steps);
    list[1].steps.extend([
        step(
            Anchor::StatusBar,
            "Status bar",
            "The result of the last command on the left, read it when something did not work. On the right the object counts, the cursor position in map units and the selection.",
        ),
        step(
            Anchor::Screen,
            "Make the layout yours",
            "Drag panel tabs to dock them somewhere else, close the ones you do not need and bring them back from View > Panels. View > Panels > Reset Layout restores this layout. Hovering a panel tab tells you what it is for.",
        ),
    ]);

    list.extend([
        Chapter {
            id: "setup",
            title: "Set up a Godot project",
            summary: "Install Godot and the addon, create a project and connect it to the editor.",
            steps: vec![
                step(
                    Anchor::Screen,
                    "What you need",
                    "• Godot 4.7 or newer. Take the .NET build if you want C# entities.\n• The FuncGodot (GodotTrench) addon, the addon zip on the GodotTrench releases page.\n• Optional: the demo project from the same page. It already contains the addon, textures and example maps, so with it you can skip the next two steps.",
                ),
                step(
                    Anchor::Screen,
                    "Create the Godot project",
                    "1. Start Godot and click Create in the Project Manager.\n2. Pick a name and an empty folder, keep the Forward+ renderer and press Create.\n3. Godot opens the new project. Keep it open, you need it for the next step.",
                ),
                step(
                    Anchor::Screen,
                    "Install the addon",
                    "1. Unzip the addon into the project so there is an addons/func_godot folder next to project.godot.\n2. In Godot open Project > Project Settings > Plugins and enable FuncGodot (GodotTrench).\n3. Put your textures in a textures folder (res://textures by default), subfolders become folders in the Materials panel.\n\nWith the plugin enabled Godot writes godottrench_game.json into the project whenever files change. It tells GodotTrench about your entities, textures and scale. Project > Tools > GodotTrench: Export Game Config writes it right away.",
                ),
                step(
                    Anchor::Menu("File"),
                    "Tell GodotTrench where Godot is",
                    "GodotTrench looks for Godot in the GODOT environment variable, your PATH and the usual install folders. If it is somewhere else, open File > Preferences and set Godot executable to your Godot program. Without it GodotTrench still works, only Run Project and Open in Godot stay disabled.",
                )
                .button("Open Preferences", Action::ShowPreferences),
                step(
                    Anchor::ToolbarProject,
                    "Open the project",
                    "Click here or use Godot > Open Godot Project…, then pick the folder that contains project.godot. Recent projects are remembered and the last one opens again on startup.",
                )
                .button("Open Godot Project…", Action::OpenProject)
                .task("Open a Godot project", has_project),
                step(
                    Anchor::StatusBar,
                    "Check that it loaded",
                    "The status bar says Loaded game config when everything is connected.\n\nIf it says there is no godottrench_game.json, the plugin has not exported it yet: enable the plugin in Godot or run Project > Tools > GodotTrench: Export Game Config, then Godot > Reload > Game Config here.",
                ),
                step(
                    Anchor::Panel(Panel::Materials),
                    "Your textures",
                    "The Materials panel now lists the project's textures. After adding textures in Godot, press the reload button in this panel or use Godot > Reload > Materials.",
                ),
                step(
                    Anchor::Panel(Panel::Entities),
                    "Your entities",
                    "The entity list comes from the game config: the GodotTrench library (doors, triggers, spawners, logic) plus your own FGD entities and C# classes.",
                ),
                step(
                    Anchor::Menu("Godot"),
                    "Open it in Godot",
                    "The Godot robot at the right end of the toolbar (or Godot > Open Project in Godot Editor) starts Godot on this project, and Run Project starts the game. Keep the Godot editor open while you map: it rebuilds maps when you save, the robot turns blue while it is connected, and the link button next to it turns on live mode, which shows your edits in Godot before you save.",
                )
                .button("Open in Godot Editor", Action::OpenGodotEditor)
                .keys([("Run project", Action::RunGodotProject)]),
            ],
        },
        Chapter {
            id: "navigation",
            title: "Moving around",
            summary: "Camera controls for the 3D and 2D views.",
            steps: vec![
                step(
                    Anchor::View3d,
                    "Fly the 3D camera",
                    "• Hold the right mouse button to look around, then WASD to fly and Q or E to go down or up. Shift flies faster.\n• Middle mouse drag pans, the wheel moves forward and back.\n• Alt + left drag on empty space orbits.\n• Fly speed and look sensitivity are in Preferences.",
                )
                .keys([("Focus selection", Action::FocusSelection)]),
                step(
                    Anchor::Views2d,
                    "Pan and zoom the 2D views",
                    "Right or middle mouse drag pans, the wheel zooms. Each view looks straight along one axis, so what you draw there lands exactly on the grid. The status bar shows the cursor position.",
                ),
                step(Anchor::ToolbarShading, "Switch shading", "Wireframe lets you see through walls, lit preview shows lights, shadows, sky and fog.")
                    .keys([("Cycle shading", Action::ToggleTextured), ("Lit preview", Action::SetShade(Shade::Lit))]),
                step(
                    Anchor::ToolbarGrid,
                    "Pick a grid size",
                    "Use a grid that fits what you build: 32 or 64 for rooms, 8 or 4 for trims and details. Everything you draw and move snaps to it.",
                )
                .keys([("Larger grid", Action::GridUp), ("Smaller grid", Action::GridDown)]),
                step(
                    Anchor::Menu("View"),
                    "Bookmarks and focus",
                    "View > Camera Bookmarks stores up to nine 3D camera positions in the map. Focus Selection frames the selection in every view.",
                )
                .keys([("Store bookmark 1", Action::StoreCamera(1)), ("Go to bookmark 1", Action::RecallCamera(1))]),
            ],
        },
        Chapter {
            id: "first_room",
            title: "Build a first room",
            summary: "Draw, move and resize brushes, hollow them and apply materials.",
            steps: vec![
                step(
                    Anchor::Tool(ToolKind::Select),
                    "Start with the select tool",
                    "Brushes are convex blocks and the basic building material. Most building happens with the select tool.",
                )
                .keys([("Select tool", Action::SetTool(ToolKind::Select))])
                .button("Select tool", Action::SetTool(ToolKind::Select)),
                step(
                    Anchor::Views2d,
                    "Draw a brush",
                    "Left drag on empty space in a view to draw a box. In a 2D view the depth comes from the last brush you drew or selected, in the 3D view the box starts on the surface under the cursor.",
                )
                .task("Draw a brush", has_brush),
                step(
                    Anchor::View3d,
                    "Select and move",
                    "• Click an object to select just that object, even inside a group. Double click selects the whole group around it. Ctrl+click adds to or removes from the selection, clicking empty space deselects.\n• Drag the selection to move it. Hold Alt to move vertically, Ctrl to drag out a copy.",
                )
                .task("Select something", has_selection),
                step(
                    Anchor::View3d,
                    "Move, rotate and scale with the gizmo",
                    "The selection shows a gizmo in the 3D view:\n• Drag an arrow to move along that axis, a colored square to move in its plane, the center dot to move freely.\n• Drag a ring to rotate in 15 degree steps, hold Shift for single degrees.\n• Drag the box at the end of an arrow to scale along that axis.\nMoves and scales snap to the grid. View > Transform Gizmo hides it.",
                )
                .keys([("Toggle the gizmo", Action::ToggleTransformGizmo)]),
                step(
                    Anchor::Views2d,
                    "Resize",
                    "In a 2D view drag an edge of the selection to resize it. In the 3D view Shift+drag a face to push or pull it, Ctrl+Shift+drag extrudes a new brush from it.",
                ),
                step(
                    Anchor::ToolbarCsg,
                    "Hollow it into a room",
                    "Select the brush and press Hollow: it becomes walls, floor and ceiling of the thickness set in Brush > CSG. For a doorway draw a brush through a wall, keep it selected and Subtract it.",
                )
                .keys([("Hollow", Action::CsgHollow), ("Subtract", Action::CsgSubtract)])
                .button("Hollow selection", Action::CsgHollow),
                step(
                    Anchor::Panel(Panel::Materials),
                    "Apply materials",
                    "Click a material to apply it to the selected brushes or faces, or drag it straight onto a face in a view. Shift+click a face in the 3D view to select just that face.",
                )
                .task("Select a face with Shift+click", has_face_selection),
                step(
                    Anchor::Panel(Panel::Inspector),
                    "Fine tune in the Inspector",
                    "With faces selected the Inspector shows material, offset, scale, rotation and justify buttons. For entities it shows their properties and outputs.",
                ),
                step(Anchor::Panel(Panel::History), "Undo freely", "Every edit is listed in History, click an older entry to go back to it.")
                    .keys([("Undo", Action::Undo), ("Redo", Action::Redo)]),
                step(
                    Anchor::ToolbarFile,
                    "Save",
                    "Save the map inside your Godot project, a maps folder works well. An autosave copy is written every few minutes, Preferences sets how often.",
                )
                .keys([("Save", Action::Save), ("Save as", Action::SaveAs)])
                .task("Save the map", is_saved),
            ],
        },
    ]);

    let mut tools = vec![step(
        Anchor::ToolbarTools,
        "The toolbar tools",
        "Tools are grouped: brush editing, mesh editing, surfaces, terrain and nature, and gameplay helpers. Q or Esc always goes back to the select tool.",
    )];
    tools.extend(ToolKind::GROUPS.iter().flat_map(|g| g.iter().copied()).filter(|t| *t != ToolKind::Select).map(tool_step));
    tools.push(
        step(
            Anchor::Menu("Brush"),
            "Shapes and displacements",
            "Brush > Shape Generator builds cylinders, arches, stairs, pipes, spheres and more inside the selection's bounds. Brush > Displacement turns the selected quad faces (or the top face of selected brushes) into a grid you can sculpt.",
        )
        .keys([("Shape generator", Action::ShowShapeDialog)])
        .button("Open Shape Generator", Action::ShowShapeDialog),
    );
    list.push(Chapter { id: "tools", title: "The tools", summary: "What each tool does and when to reach for it.", steps: tools });

    list.extend([
        Chapter {
            id: "entities",
            title: "Entities and gameplay",
            summary: "Place entities, wire them together and use them from code.",
            steps: vec![
                step(
                    Anchor::Panel(Panel::Entities),
                    "Place entities",
                    "Drag a card into a view and the entity rests on the surface you drop it on, double click places it at the cursor. Ctrl or Shift click selects several cards, dragging one of them places them all in a row. Brush entities (doors, triggers, func_detail) arrive with a box brush, or select brushes first and double click the class to wrap them.",
                )
                .task("Place an entity", has_entity),
                step(
                    Anchor::Panel(Panel::Inspector),
                    "Properties and outputs",
                    "The Inspector lists the entity's properties, hover a name for its description. Outputs connect entities like in Hammer: when something happens on this entity, call an input on another one, for example a button opening a door. Targets are targetnames, @groups or node paths.",
                ),
                step(
                    Anchor::Menu("Gameplay"),
                    "Gameplay wizards",
                    "Select brushes and turn them into a hinged or sliding door (with a walk up trigger), a lift or a button. Triggers wraps a volume around the selection, Logic places relays, timers and counters, and Link connects two selected entities.",
                ),
                tool_step(ToolKind::Volume),
                step(
                    Anchor::View3d,
                    "Gizmo handles",
                    "Selected entities show handles in the views for door hinges, travel distance, light radius, spot cones and spawn areas. Drag them with the select tool.",
                ),
                step(
                    Anchor::Panel(Panel::Reference),
                    "Code reference",
                    "Pick an entity to see how to use it from GDScript and C#, with snippets to copy. C# classes marked with [GodotTrenchEntity] become entities without any .tres file.",
                )
                .button("Open Reference", Action::ShowReference),
            ],
        },
        Chapter {
            id: "organize",
            title: "Organize big maps",
            summary: "Layers, groups, prefabs, hiding and fixing problems.",
            steps: vec![
                step(
                    Anchor::Panel(Panel::Outliner),
                    "Layers",
                    "Layers split the map into parts you can hide, lock or omit from the build. New objects go into the current layer, click a layer to make it current. Each scatter set gets its own layer.",
                )
                .button("Add Layer", Action::AddLayer)
                .task("Add a second layer", has_layers),
                step(
                    Anchor::Menu("Edit"),
                    "Groups",
                    "Group objects to keep them together: a click still picks one object inside, a double click selects the whole group so it moves as one. Duplicate Linked makes copies that stay identical when you change any of them, clicking one of those always selects the whole copy.",
                )
                .keys([("Group", Action::Group), ("Ungroup", Action::Ungroup), ("Duplicate linked", Action::DuplicateLinked)]),
                step(
                    Anchor::Menu("Edit"),
                    "Prefabs",
                    "Edit > Prefabs > Create from Selection saves the selection as its own .gtm file. Insert it anywhere as an instance, change the prefab and every instance follows.",
                ),
                step(Anchor::Menu("Edit"), "Hide, isolate and lock", "Hide what is in the way, isolate to see only the selection, lock objects so clicks go through them.")
                    .keys([("Hide selected", Action::HideSelected), ("Isolate selected", Action::IsolateSelected), ("Show all", Action::UnhideAll)]),
                step(Anchor::Panel(Panel::Issues), "Fix problems early", Panel::Issues.help()),
                step(
                    Anchor::Menu("View"),
                    "Cordon",
                    "View > Cordon limits the views and the .map export to a box around the selection, handy in huge maps.",
                ),
                step(
                    Anchor::Menu("Help"),
                    "Find and rebind commands",
                    "The command palette searches every command by name. Keyboard Shortcuts rebinds any of them or switches to Hammer or Blender style keys.",
                )
                .keys([("Command palette", Action::ShowCommandPalette), ("Keyboard shortcuts", Action::ShowKeymap)]),
            ],
        },
        Chapter {
            id: "terrain",
            title: "Terrain and nature",
            summary: "Generate terrain, sculpt it, paint its layers and scatter trees and rocks.",
            steps: vec![
                step(
                    Anchor::Menu("Terrain"),
                    "Create a terrain",
                    "Terrain > Create Terrain… generates hills, mountains, islands or valleys, or imports a heightmap PNG, with up to four material layers painted by slope and height.",
                )
                .button("Create Terrain…", Action::ShowTerrainDialog)
                .task("Create a terrain", has_terrain),
                tool_step(ToolKind::Sculpt),
                step(Anchor::ToolOptions, "Brush settings", "Mode, radius and strength of the sculpt, blend and scatter brushes live here. Ctrl+wheel in a view resizes the brush."),
                tool_step(ToolKind::Blend),
                tool_step(ToolKind::Scatter).button("Open Scatter Palette", Action::ShowScatterPalette),
                step(
                    Anchor::Menu("Terrain"),
                    "Scatter sets and nature models",
                    "Terrain > Scatter > Install Nature Models copies the default tree and rock models into the project. Alt+click a surface with the scatter tool to make it a target, so a forest never covers the house on top of the terrain, and Fill covers every target at once.",
                ),
            ],
        },
        Chapter {
            id: "godot",
            title: "Into Godot",
            summary: "Build the map in Godot, live link and play it.",
            steps: vec![
                step(
                    Anchor::ToolbarFile,
                    "Save inside the project",
                    "Godot only sees files inside the project folder, for example res://maps/level.gtm. The save dialog starts in the project folder.",
                )
                .keys([("Save", Action::Save)])
                .task("Save the map inside the open project", saved_in_project),
                step(
                    Anchor::Screen,
                    "Build the map in Godot",
                    "1. In Godot open or create a 3D scene.\n2. Add a FuncGodotMap node.\n3. In the Inspector set Local Map File to your .gtm file.\n4. Press Build Map at the top of the Inspector.\n\nBrushes become meshes with collision, entities become nodes with their scripts and outputs are connected.",
                ),
                step(
                    Anchor::Screen,
                    "Live link",
                    "While the Godot editor runs with the plugin, saving here rebuilds every FuncGodotMap that uses the map (Auto Rebuild On Save on the node).\n\nFor a game that is already running, add res://addons/func_godot/src/godottrench/runtime/godottrench_hot_reload.gd as an autoload and saving reloads the map in the game too. Both links can be turned off in Preferences.",
                ),
                step(
                    Anchor::Menu("Godot"),
                    "Play it",
                    "Godot > Run Project starts the game with the project's main scene. Add a GodotTrenchDebugOverlay node to see fired outputs and trigger volumes in game, F3 toggles it.",
                )
                .keys([("Run project", Action::RunGodotProject)]),
                step(
                    Anchor::ToolbarShading,
                    "Preview the look",
                    "Lit preview uses the same worldspawn sun, sky and fog settings that the addon turns into the Godot environment, so the map looks close to the game before you build it.",
                )
                .keys([("Lit preview", Action::SetShade(Shade::Lit))]),
                step(
                    Anchor::Screen,
                    "Where to go next",
                    "• Help > Guide keeps every chapter as a searchable page, with tips at the end.\n• docs/tools.md, docs/gameplay.md and docs/mcp.md in the repository go deeper.\n• Hover buttons and panel tabs for tooltips.\n\nHappy mapping!",
                ),
            ],
        },
    ]);

    let contents: Vec<String> = list.iter().enumerate().skip(1).map(|(i, c)| format!("{}. {}: {}", i + 1, c.title, c.summary)).collect();
    list[0].steps[1].body = format!("{}\n\nFinished chapters get a check mark in Help > Guide.", contents.join("\n"));
    list
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Page {
    Chapter(usize),
    Tips,
}

enum Nav {
    To(usize, usize),
    Next,
    Back,
    Page,
    Close,
}

pub struct Guide {
    pub chapters: Vec<Chapter>,
    pub window_open: bool,
    pub welcome_open: bool,
    page: Page,
    search: String,
    scroll_to: Option<usize>,
    tour: Option<(usize, usize)>,
    reveal: Option<Anchor>,
    popover_size: Vec2,
}

impl Guide {
    pub fn new(welcome: bool) -> Self {
        Self {
            chapters: chapters(),
            window_open: false,
            welcome_open: welcome,
            page: Page::Chapter(0),
            search: String::new(),
            scroll_to: None,
            tour: None,
            reveal: None,
            popover_size: vec2(POPOVER_WIDTH + 30.0, 240.0),
        }
    }

    /// Chapter and step the tour shows, None when it is closed.
    pub fn tour(&self) -> Option<(usize, usize)> {
        self.tour
    }

    pub fn start_tour(&mut self, chapter: usize, step: usize) {
        let chapter = chapter.min(self.chapters.len() - 1);
        let step = step.min(self.chapters[chapter].steps.len() - 1);
        self.tour = Some((chapter, step));
        self.reveal = Some(self.chapters[chapter].steps[step].anchor);
        self.welcome_open = false;
    }

    /// Starts at the first chapter not finished yet.
    pub fn resume_tour(&mut self, prefs: &crate::state::Prefs) {
        let chapter = self.chapters.iter().position(|c| !prefs.guide_done.iter().any(|d| d == c.id)).unwrap_or(0);
        self.start_tour(chapter, 0);
    }

    pub fn stop_tour(&mut self) {
        self.tour = None;
    }

    pub fn open_page(&mut self, chapter: usize) {
        self.window_open = true;
        self.search.clear();
        self.page = Page::Chapter(chapter.min(self.chapters.len() - 1));
    }

    /// The anchor of a step the tour just moved to, so the app can bring its panel to the front.
    pub fn take_reveal(&mut self) -> Option<Anchor> {
        self.reveal.take()
    }

    pub fn show(&mut self, ctx: &egui::Context, state: &mut EditorState, actions: &mut Vec<Action>) {
        self.welcome_ui(ctx, state);
        self.window_ui(ctx, state, actions);
        self.tour_ui(ctx, state, actions);
    }

    fn finish_chapter(&self, chapter: usize, state: &mut EditorState) {
        let id = self.chapters[chapter].id;
        if !state.prefs.guide_done.iter().any(|d| d == id) {
            state.prefs.guide_done.push(id.to_string());
        }
    }

    fn navigate(&mut self, nav: Nav, state: &mut EditorState) {
        let Some((c, s)) = self.tour else { return };
        match nav {
            Nav::To(chapter, step) => self.start_tour(chapter, step),
            Nav::Next if s + 1 < self.chapters[c].steps.len() => self.start_tour(c, s + 1),
            Nav::Next => {
                self.finish_chapter(c, state);
                if c + 1 < self.chapters.len() {
                    self.start_tour(c + 1, 0);
                } else {
                    self.tour = None;
                    state.set_status("Tour finished. Help > Guide has every chapter as a page");
                }
            }
            Nav::Back if s > 0 => self.start_tour(c, s - 1),
            Nav::Back if c > 0 => self.start_tour(c - 1, self.chapters[c - 1].steps.len() - 1),
            Nav::Back => {}
            Nav::Page => {
                self.open_page(c);
                self.scroll_to = Some(s);
            }
            Nav::Close => self.tour = None,
        }
    }

    fn welcome_ui(&mut self, ctx: &egui::Context, state: &mut EditorState) {
        if !self.welcome_open {
            return;
        }
        let mut open = true;
        let mut choice = None;
        egui::Window::new("Welcome to GodotTrench")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .open(&mut open)
            .show(ctx, |ui| {
                ui.set_width(400.0);
                ui.label("A brush based level editor for Godot. New here? The guided tour walks you through the window, setting up a Godot project and building a first map, one chapter at a time.");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(RichText::new("Take the guided tour").strong()).clicked() {
                        choice = Some(Nav::Next);
                    }
                    if ui.button("Browse the guide").clicked() {
                        choice = Some(Nav::Page);
                    }
                    if ui.button("Not now").clicked() {
                        choice = Some(Nav::Close);
                    }
                });
                ui.add_space(4.0);
                ui.label(RichText::new("Both stay in the Help menu.").weak());
            });
        if open && choice.is_none() {
            return;
        }
        self.welcome_open = false;
        state.prefs.guide_welcome_seen = true;
        match choice {
            Some(Nav::Next) => self.start_tour(0, 0),
            Some(Nav::Page) => self.open_page(0),
            _ => {}
        }
    }

    fn window_ui(&mut self, ctx: &egui::Context, state: &mut EditorState, actions: &mut Vec<Action>) {
        if !self.window_open {
            return;
        }
        let mut open = true;
        let mut start = None;
        egui::Window::new("Guide").open(&mut open).default_size([800.0, 580.0]).min_size([520.0, 320.0]).show(ctx, |ui| {
            ui.horizontal_top(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(210.0);
                    self.index_ui(ui, state, &mut start);
                });
                ui.separator();
                ScrollArea::vertical().id_salt(("guide_page", self.page, self.search.is_empty())).auto_shrink([false, false]).show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                        if !self.search.trim().is_empty() {
                            self.search_ui(ui);
                        } else {
                            match self.page {
                                Page::Chapter(c) => self.chapter_ui(ui, c, state, actions, &mut start),
                                Page::Tips => tips_ui(ui),
                            }
                        }
                    });
                });
            });
        });
        self.window_open = open;
        if let Some((c, s)) = start {
            self.start_tour(c, s);
        }
    }

    fn index_ui(&mut self, ui: &mut Ui, state: &EditorState, start: &mut Option<(usize, usize)>) {
        ui.add(egui::TextEdit::singleline(&mut self.search).hint_text("Search the guide").desired_width(f32::INFINITY));
        ui.add_space(4.0);
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            if ui.button(RichText::new("Start the guided tour").strong()).clicked() {
                *start = Some((0, 0));
            }
            ui.separator();
            for (i, chapter) in self.chapters.iter().enumerate() {
                let done = state.prefs.guide_done.iter().any(|d| d == chapter.id);
                let atom: egui::Atom = if done { icons::CHECK.image(icons::SMALL).tint(DONE).into() } else { icons::spacer(icons::SMALL) };
                let selected = self.search.trim().is_empty() && self.page == Page::Chapter(i);
                let button = egui::Button::new((atom, format!("{}. {}", i + 1, chapter.title))).selected(selected).frame_when_inactive(selected);
                let resp = ui.add(button).on_hover_text(chapter.summary);
                if resp.clicked() {
                    self.page = Page::Chapter(i);
                    self.search.clear();
                }
            }
            ui.separator();
            let selected = self.search.trim().is_empty() && self.page == Page::Tips;
            if ui.add(egui::Button::new((icons::spacer(icons::SMALL), "Tips")).selected(selected).frame_when_inactive(selected)).clicked() {
                self.page = Page::Tips;
                self.search.clear();
            }
        });
    }

    fn chapter_ui(&mut self, ui: &mut Ui, c: usize, state: &EditorState, actions: &mut Vec<Action>, start: &mut Option<(usize, usize)>) {
        let chapter = &self.chapters[c];
        ui.heading(chapter.title);
        ui.label(RichText::new(chapter.summary).weak());
        ui.add_space(4.0);
        if ui.button("Show me in the editor").on_hover_text("Starts the tour at this chapter").clicked() {
            *start = Some((c, 0));
        }
        ui.separator();
        for (s, step) in chapter.steps.iter().enumerate() {
            let title = ui.label(RichText::new(&step.title).strong().size(15.0));
            if self.scroll_to == Some(s) {
                title.scroll_to_me(Some(Align::TOP));
            }
            step_body(ui, state, step, actions, (c, s));
            if step.anchor != Anchor::Screen && ui.small_button("Show me").on_hover_text("Points at it in the editor").clicked() {
                *start = Some((c, s));
            }
            ui.add_space(10.0);
        }
        self.scroll_to = None;
        ui.separator();
        ui.horizontal(|ui| {
            if c > 0 && ui.button(format!("‹ {}", self.chapters[c - 1].title)).clicked() {
                self.page = Page::Chapter(c - 1);
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if c + 1 < self.chapters.len() {
                    if ui.button(format!("{} ›", self.chapters[c + 1].title)).clicked() {
                        self.page = Page::Chapter(c + 1);
                    }
                } else if ui.button("Tips ›").clicked() {
                    self.page = Page::Tips;
                }
            });
        });
    }

    fn search_ui(&mut self, ui: &mut Ui) {
        let query = self.search.trim().to_lowercase();
        let mut found = 0;
        let mut open = None;
        for (c, chapter) in self.chapters.iter().enumerate() {
            for (s, step) in chapter.steps.iter().enumerate().filter(|(_, step)| step.matches(&query)) {
                found += 1;
                let resp = ui.add(egui::Button::new(RichText::new(format!("{}: {}", chapter.title, step.title)).strong()).frame(false));
                let preview: String = step.body.chars().take(140).collect();
                ui.label(RichText::new(if preview.len() < step.body.len() { format!("{preview}…") } else { preview }).weak());
                ui.add_space(6.0);
                if resp.clicked() {
                    open = Some((c, s));
                }
            }
        }
        let tips: Vec<&str> = TIPS.iter().copied().filter(|t| t.to_lowercase().contains(&query)).collect();
        if !tips.is_empty() {
            ui.label(RichText::new("Tips").strong());
            for tip in &tips {
                ui.label(format!("• {tip}"));
            }
        }
        if found == 0 && tips.is_empty() {
            ui.label(RichText::new("Nothing matches. Try another word, or the command palette for commands.").weak());
        }
        if let Some((c, s)) = open {
            self.page = Page::Chapter(c);
            self.scroll_to = Some(s);
            self.search.clear();
        }
    }

    fn tour_ui(&mut self, ctx: &egui::Context, state: &mut EditorState, actions: &mut Vec<Action>) {
        let Some((c, s)) = self.tour else { return };
        let screen = ctx.content_rect();
        let chapter = &self.chapters[c];
        let step = &chapter.steps[s];
        let targets: Vec<Rect> = match step.anchor {
            Anchor::Screen => Vec::new(),
            anchor => anchor_rects(ctx, anchor).into_iter().map(|r| r.intersect(screen)).filter(|r| r.is_positive()).collect(),
        };
        spotlight(ctx, screen, &targets);

        let bounds = targets.iter().copied().reduce(Rect::union);
        let pos = place(bounds, self.popover_size, screen);
        let mut nav = None;
        let response = egui::Area::new(Id::new("guide_tour")).order(Order::Foreground).fixed_pos(pos).constrain_to(screen).show(ctx, |ui| {
            egui::Frame::popup(ui.style()).stroke(Stroke::new(1.5, ACCENT)).inner_margin(egui::Margin::same(12)).show(ui, |ui| {
                ui.set_width(POPOVER_WIDTH);
                ui.horizontal(|ui| {
                    egui::ComboBox::from_id_salt("guide_tour_chapter")
                        .width(POPOVER_WIDTH - 80.0)
                        .selected_text(format!("{}. {}", c + 1, chapter.title))
                        .show_ui(ui, |ui| {
                            for (i, other) in self.chapters.iter().enumerate() {
                                if ui.selectable_label(i == c, format!("{}. {}", i + 1, other.title)).clicked() {
                                    nav = Some(Nav::To(i, 0));
                                }
                            }
                        })
                        .response
                        .on_hover_text("Jump to any chapter");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("Close").clicked() {
                            nav = Some(Nav::Close);
                        }
                    });
                });
                ui.add_space(6.0);
                ui.label(RichText::new(&step.title).size(17.0).strong());
                ui.add_space(2.0);
                step_body(ui, state, step, actions, (c, s));
                if targets.is_empty() && step.anchor != Anchor::Screen {
                    ui.label(RichText::new("That part of the window is not visible right now. View > Panels brings closed panels back.").weak().italics());
                }
                ui.add_space(4.0);
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{} / {}", s + 1, chapter.steps.len())).weak());
                    if ui.small_button("Open as page").on_hover_text("Shows this chapter in the Guide window").clicked() {
                        nav = Some(Nav::Page);
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let next = match (s + 1 == chapter.steps.len(), c + 1 == self.chapters.len()) {
                            (false, _) => "Next",
                            (true, false) => "Next chapter",
                            (true, true) => "Finish",
                        };
                        if ui.button(RichText::new(next).strong()).clicked() {
                            nav = Some(Nav::Next);
                        }
                        if ui.add_enabled(c > 0 || s > 0, egui::Button::new("Back")).clicked() {
                            nav = Some(Nav::Back);
                        }
                    });
                });
            });
        });
        self.popover_size = response.response.rect.size();
        if let Some(nav) = nav {
            self.navigate(nav, state);
            ctx.request_repaint();
        }
    }
}

fn step_body(ui: &mut Ui, state: &EditorState, step: &Step, actions: &mut Vec<Action>, salt: (usize, usize)) {
    if !step.body.is_empty() {
        ui.label(&step.body);
    }
    if !step.keys.is_empty() {
        ui.add_space(2.0);
        egui::Grid::new(("guide_keys", salt)).num_columns(2).spacing([16.0, 2.0]).show(ui, |ui| {
            for (label, action) in &step.keys {
                ui.label(RichText::new(*label).weak());
                let keys = commands::shortcut_text(ui.ctx(), &state.prefs, action).unwrap_or_else(|| "not bound".into());
                ui.label(RichText::new(keys).monospace());
                ui.end_row();
            }
        });
    }
    if let Some(task) = &step.task {
        ui.add_space(2.0);
        ui.horizontal(|ui| {
            if (task.done)(state) {
                ui.add(icons::CHECK.image(icons::SMALL).tint(DONE));
                ui.label(RichText::new(task.label).color(DONE));
            } else {
                ui.label(RichText::new("Try it:").color(ACCENT));
                ui.label(task.label);
            }
        });
    }
    if let Some((label, action)) = &step.button {
        ui.add_space(2.0);
        if ui.button(*label).clicked() {
            actions.push(action.clone());
        }
    }
}

fn tips_ui(ui: &mut Ui) {
    ui.heading("Tips");
    ui.add_space(4.0);
    for tip in TIPS {
        ui.label(format!("• {tip}"));
        ui.add_space(2.0);
    }
}

/// Dims the window except for `targets` and outlines them.
fn spotlight(ctx: &egui::Context, screen: Rect, targets: &[Rect]) {
    // Painted into the background layer after the panels and views, so floating windows and menus stay undimmed.
    // A mesh instead of rects avoids the anti aliased seams where the dimmed cells meet.
    let painter = ctx.layer_painter(egui::LayerId::background());
    let holes: Vec<Rect> = targets.iter().map(|t| t.expand(3.0).intersect(screen)).filter(|r| r.is_positive()).collect();
    let dim = Color32::from_black_alpha(if holes.is_empty() { 70 } else { 120 });
    let mut mesh = egui::Mesh::default();
    for cell in dim_cells(screen, &holes) {
        mesh.add_colored_rect(cell, dim);
    }
    painter.add(egui::Shape::mesh(mesh));
    if holes.is_empty() {
        return;
    }
    let pulse = ((ctx.input(|i| i.time) * 3.0).sin() * 0.5 + 0.5) as f32;
    for hole in &holes {
        painter.rect_stroke(*hole, 4.0, Stroke::new(1.5 + pulse * 1.5, ACCENT), StrokeKind::Outside);
    }
    ctx.request_repaint_after(std::time::Duration::from_millis(50));
}

/// Splits `screen` along the hole edges and returns the cells outside every hole.
fn dim_cells(screen: Rect, holes: &[Rect]) -> Vec<Rect> {
    let mut xs = vec![screen.min.x, screen.max.x];
    let mut ys = vec![screen.min.y, screen.max.y];
    for h in holes {
        xs.extend([h.min.x, h.max.x]);
        ys.extend([h.min.y, h.max.y]);
    }
    for v in [&mut xs, &mut ys] {
        v.sort_by(f32::total_cmp);
        v.dedup();
    }
    let mut cells = Vec::new();
    for y in ys.windows(2) {
        for x in xs.windows(2) {
            let cell = Rect::from_min_max(pos2(x[0], y[0]), pos2(x[1], y[1]));
            if cell.is_positive() && !holes.iter().any(|h| h.contains(cell.center())) {
                cells.push(cell);
            }
        }
    }
    cells
}

/// Top left corner for a popover of `size` next to `target`: below, right, left or above, whichever fits on screen,
/// otherwise inside the target's top right corner.
fn place(target: Option<Rect>, size: Vec2, screen: Rect) -> Pos2 {
    let bounds = screen.shrink(8.0);
    let clamp_x = |x: f32| x.clamp(bounds.min.x, (bounds.max.x - size.x).max(bounds.min.x));
    let clamp_y = |y: f32| y.clamp(bounds.min.y, (bounds.max.y - size.y).max(bounds.min.y));
    let Some(t) = target else {
        return pos2(clamp_x(screen.center().x - size.x * 0.5), clamp_y(screen.center().y - size.y * 0.5));
    };
    let gap = 14.0;
    [
        pos2(clamp_x(t.min.x), t.max.y + gap),
        pos2(t.max.x + gap, clamp_y(t.min.y)),
        pos2(t.min.x - gap - size.x, clamp_y(t.min.y)),
        pos2(clamp_x(t.min.x), t.min.y - gap - size.y),
    ]
    .into_iter()
    .find(|p| bounds.contains_rect(Rect::from_min_size(*p, size)))
    .unwrap_or_else(|| pos2(clamp_x(t.max.x - size.x - gap), clamp_y(t.min.y + gap)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chapters_are_complete_and_unique() {
        let list = chapters();
        let mut ids = std::collections::HashSet::new();
        for chapter in &list {
            assert!(ids.insert(chapter.id), "duplicate chapter id {}", chapter.id);
            assert!(!chapter.steps.is_empty(), "{} has no steps", chapter.id);
            for step in &chapter.steps {
                assert!(!step.title.is_empty() && !step.body.is_empty(), "{}: empty step {:?}", chapter.id, step.title);
            }
        }
        let tools: Vec<ToolKind> = list
            .iter()
            .flat_map(|c| &c.steps)
            .filter_map(|s| match s.anchor {
                Anchor::Tool(t) => Some(t),
                _ => None,
            })
            .collect();
        assert!(ToolKind::all().iter().all(|t| tools.contains(t)), "every tool is explained");
        assert!(Panel::ALL.iter().all(|p| list[1].steps.iter().any(|s| s.anchor == Anchor::Panel(*p))), "the window chapter shows every panel");
    }

    #[test]
    fn guide_text_avoids_dashes_as_punctuation() {
        let texts = chapters().into_iter().flat_map(|c| c.steps.into_iter().flat_map(|s| [s.title, s.body])).chain(TIPS.iter().map(|t| t.to_string()));
        for text in texts {
            assert!(!text.contains('\u{2014}') && !text.contains('\u{2013}') && !text.contains(" - "), "{text}");
        }
    }

    #[test]
    fn dims_everything_but_the_holes() {
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(100.0, 100.0));
        let holes = [Rect::from_min_max(pos2(10.0, 10.0), pos2(40.0, 40.0)), Rect::from_min_max(pos2(60.0, 60.0), pos2(90.0, 90.0))];
        let cells = dim_cells(screen, &holes);
        let area: f32 = cells.iter().map(|c| c.area()).sum();
        assert!((area - (100.0 * 100.0 - 2.0 * 30.0 * 30.0)).abs() < 0.01);
        assert!(cells.iter().all(|c| holes.iter().all(|h| !h.intersects(c.shrink(0.01)))));
        assert_eq!(dim_cells(screen, &[]), vec![screen]);
    }

    #[test]
    fn popover_stays_on_screen() {
        let screen = Rect::from_min_size(Pos2::ZERO, vec2(1600.0, 900.0));
        let size = vec2(380.0, 260.0);
        let fits = |p: Pos2| screen.contains_rect(Rect::from_min_size(p, size));
        let menu = Rect::from_min_size(pos2(1500.0, 0.0), vec2(60.0, 20.0));
        let p = place(Some(menu), size, screen);
        assert!(fits(p) && p.y >= menu.max.y, "below a menu at the right edge");
        let status = Rect::from_min_size(pos2(0.0, 880.0), vec2(1600.0, 20.0));
        let p = place(Some(status), size, screen);
        assert!(fits(p) && p.y + size.y <= status.min.y, "above the status bar");
        let p = place(Some(screen), size, screen);
        assert!(fits(p), "inside a target that covers the screen");
        assert!(fits(place(None, size, screen)));
    }

    #[test]
    fn anchors_collect_rects_per_frame() {
        let ctx = egui::Context::default();
        let a = Rect::from_min_size(Pos2::ZERO, vec2(10.0, 10.0));
        let b = Rect::from_min_size(pos2(20.0, 0.0), vec2(10.0, 10.0));
        mark(&ctx, Anchor::Views2d, a);
        mark(&ctx, Anchor::Views2d, b);
        assert_eq!(anchor_rects(&ctx, Anchor::Views2d), vec![a, b]);
        assert!(anchor_rects(&ctx, Anchor::View3d).is_empty());
    }
}
