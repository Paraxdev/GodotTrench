# Interface and navigation

TrenchDot follows TrenchBroom and Hammer, so if you have used either, most habits carry over. If not, this page
shows you around, and [Coming from Unity or Blender](coming-from-unity-blender.md) maps the ideas you already know.

## The window

![The editor with a small room: the Outliner on the left, the 3D, Top, Front and Side views in the middle, the Inspector on the right and the Materials panel at the bottom](../assets/interface/window.png)

The menus are at the top. A command sits under the kind of object it changes, so CSG is under *Brush* and terrain
sculpting under *Terrain*.

Can't find something? Press Ctrl+Shift+P (or F1) for the command palette and type its name. Commands with a jargon
name, like *Hollow* or *UV Lock*, explain themselves when you hover them, and the palette shows the same line beneath
its list. Escape or a click outside closes it without running anything.

![The command palette with hollow typed in, Brush: Hollow picked with its shortcut, and the line explaining it beneath](../assets/interface/command-palette.png)

Right under the toolbar is the **tool options bar**. It holds the settings of the active tool, or one line on how to
use a tool that has none. Hover the tool's name there, or its button in the toolbar, for a short description.

![The tool options bar with Select Tool hovered, its tooltip describing the tool and ending in F1: manual](../assets/interface/tool-options-tooltip.png)

### The empty map card

On a new, empty map a card in the 3D view tells you where to start: drag in the Top view to draw your first box, and
the Top view marks the spot with a yellow label. Below that it explains the 3D camera and links the
[getting started guide](../getting-started.md). When the open project has maps, **Open a map from this project**
lists them, so you can open a finished one instead.

![The empty map card in the 3D view with Getting Started Guide, Open a map from this project and Hide tips, beside the yellow label in the Top view](../assets/interface/empty-map-card.png)

The card goes away once the map has something in it. **Hide tips** turns the tips off, and *Tips on an empty map* in
*File > Preferences* brings them back.

### Panels

| Panel | Dock | What it is for |
| --- | --- | --- |
| Outliner | Left | The map as a tree of [layers, groups](organizing.md) and objects |
| History | Left | Every edit. Clicking one undoes it and everything after it, clicking a dimmed one redoes up to it |
| Issues | Left | Problems found as you work, like faces with no material. Click one to select the object, most offer a fix |
| Inspector | Right | The selection's properties, such as an entity's keys or a brush's exact position and size |
| Entities | Right | Entity classes to drag into a view |
| UV Editor | Right | [Exact texture placement](uv-editor.md) |
| Scatter | Right | The models the [Scatter](scatter.md) tool paints |
| Materials | Bottom | Your project's textures |
| Models | Bottom | [Model files](models.md) to place |
| Prefabs | Bottom | Ready made pieces from the [prefab library](prefab-library.md) to paste |
| Logic | Starts closed | The [entity wiring as a graph](../gameplay/logic-graph.md), to connect entities and preview what an output sets off |
| Reference | Starts closed | Code for using an entity class from your scripts |

Closed a panel by accident? Open it again from *View > Panels*. *Reset Layout* at the bottom of the same menu puts
everything back where it started.

![The View menu with Panels open, the shown panels ticked and Reset Layout at the bottom](../assets/interface/view-panels-menu.png)

## Help and the manual

Tooltips explain buttons, tabs and settings. For more, point at a tool, a panel tab, a menu entry or an Inspector
section and press **F1**, and the matching page of this manual opens. A Ctrl+click on a tool in the toolbar
(Cmd+click on macOS) does the same. A tooltip ending in *F1: manual* has a page behind it, over anything else F1 opens
the command palette.

*Help > Manual* opens the start of the manual.

![The Help menu with Manual hovered](../assets/interface/help-menu.png)

Once you know your way around, untick *Help text* in *File > Preferences* to hide the short guidance lines in the tool
options bar and the panels. Their tooltips stay.

(Note: links open in your web browser. If no browser opens, the status bar says so and the link is copied to the
clipboard.)

## Views

The views are 3D, Top, Front and Side. The 2D views look straight along one axis, which makes lining things up
easier. With something selected, they print its width above it and its height beside it.

![The Front view with a selected box brush, 96 printed above it and 48 beside it](../assets/interface/front-view-size-labels.png)

A click in a 2D view selects the nearest object along the view, so in the Top view a ceiling hides the walls under it.
Click again at the same spot within about a second and a half, or Alt+click at any time, to step to the next object
under the cursor.

| Do this | To |
| --- | --- |
| Drag the point where the dividers cross | Resize all four views at once |
| Shift+Space | Maximize the view under the pointer, again to restore |
| *View > Views* | Switch between one, two and four views, or bring back a closed view. Single View keeps the view you last worked in, the 3D view to begin with, and while one view is shown picking another view's name switches to it |

![The View menu with Views open: the four views ticked, Single, Two and Four Views, and Maximize View on Shift+Space](../assets/interface/view-views-menu.png)

## Tabs

Each open map gets a tab, and the tab bar shows up under the tool options bar once two are open. Ctrl+Shift+N (or the
**+** button) opens a new tab, Ctrl+Tab switches and Ctrl+W closes one.

![The tab bar with room.tdm and untitled.tdm open and the + button hovered, showing New tab](../assets/interface/map-tabs.png)

*New Map* and importing open in a new tab when the current map has unsaved changes. Closing a modified tab asks to
save, discard or cancel, and quitting lists every unsaved map.

## Camera

| Input | 3D view | 2D views |
| --- | --- | --- |
| Right drag | Look around. WASD moves, Q and E go down and up, Shift triples the speed | Pan |
| Middle drag | Pan | Pan |
| Wheel | Move forward and back, faster the farther away the surface straight ahead is | Zoom towards the cursor |
| Alt+left drag | Orbit, unless the drag starts on the selection, which then moves vertically | |

WASD only flies while you hold the right button, otherwise those keys are shortcuts. Forgot the controls? Hover a
view's tab. Until you first look around, the 3D view also shows them under its name while the pointer is over it
(**Hide tips** on the empty map card turns that off).

**F** frames the selection in every view, or the whole map when nothing is selected. A wheel notch zooms a 2D view by
the same factor at any zoom, so going from a whole large terrain down to a single room takes about 20 notches, and
**F** on the room is quicker.

Ctrl+Shift+1 to 9 stores the 3D camera as a bookmark and Ctrl+1 to 9 jumps back. Bookmarks are saved in the map, and
undo does not touch them.

## Shading

F4 cycles the 3D view through Textured, Flat, Lit Preview and Wireframe, and the view prints the mode at its top left.
Once the map has a [bake](light-baking.md), **Baked Lighting** comes after Lit Preview. F3 jumps straight to
**Lit Preview** and back to Textured, without cycling through the others.

![The 3D view cycling through textured, flat, lit and wireframe shading](../assets/interface/shading-cycle.gif)

Lit Preview shows the map's lights, sun shadows, sky and fog without building in Godot. It only draws the 64 strongest
point and spot lights, so a map with many more can look darker here than in Godot.

## Theme

TrenchDot comes in a dark and a light look. A fresh install starts dark.

1. Open **File > Preferences**. **Theme** is the first row, at the top of the window.
2. Click the dropdown and pick **Dark**, **Light** or **System**.

    ![The Theme dropdown in Preferences, open with Dark, Light and System](../assets/interface/preferences-theme.png)

3. The whole editor changes on the next frame, no restart. The choice is saved with your other preferences.

    ![The editor in the light theme, with the menu bar, the Outliner, the 3D view and the Top view](../assets/interface/theme-light-window.png)

**System** follows the light or dark app mode of your operating system and changes with it. While it is picked, the
dropdown reads *System (Light)* or *System (Dark)* so you can see what your system gave. Where the system does not
report a mode, the editor stays dark.

You can also pick it in **View > Theme**, right below *Interface Scale*, or in the command palette as *View: Theme:
Dark*, *Light* or *System*. No key is bound by default, but you can give each one a key in the keymap editor.

In Light, the 2D views (Top, Front and Side) turn into paper, a light gray background with dark grid lines and brush
edges, so the lines you draw still read at a glance. The 3D view keeps its dark surround in both themes, and so do the
texture canvases of the UV editor and the hotspot editor. A texture looks the same in either theme, and a light
surround would change how you judge its colors.

<div class="td-mcp" markdown>

Over the [MCP server](../mcp.md), `set_editor` with `"theme": "light"` switches the theme. The values are `dark`,
`light` and `system`, and an unknown one is an error that changes nothing. `get_state` reports the choice as
`ui.theme` and what is on screen as `ui.theme_active`.

</div>

## Language

The menus, panels, tooltips and status messages come in English, German, French, Spanish, Portuguese, Russian,
Simplified Chinese and Japanese. A fresh install starts in English.

1. Open **File > Preferences**. **Language** is the second row, right below **Theme**.
2. Click the dropdown and pick your language. Every language is listed by its own name, so you can find yours even when
    the editor is in a language you do not read.

    ![The Language dropdown in Preferences, open with English, Deutsch, Français, Español, Português, Русский, 简体中文 and 日本語](../assets/interface/preferences-language.png)

3. The whole editor switches right away, no restart. The choice is saved with your other preferences.

    ![The File menu in German](../assets/interface/language-file-menu-de.png)

While another language than English is on, the row reads for example *Sprache (Language)*, so you can always find your
way back. You can also pick a language in the command palette as *View: Language: Deutsch*, and the same command works
in every language, since the palette also searches the English names. No key is bound by default, but you can give each
language a key in the keymap editor.

The translations are written by AI and no native speaker has checked them yet, a note under the row says so. A text that
has no translation yet shows in English, and so does a language this version does not know. If a translation reads
wrong, you can fix it in [the locale file of that language](https://github.com/Paraxdev/TrenchDot/tree/main/crates/td_editor/locales),
see [Translations](../development.md#translations).

> [!NOTE]
> The manual follows the language of the editor. With a language picked, **F1** over a tool, panel or menu, and the
> **Help** menu entries, open that page of the manual in your language. The language button in the header of the manual
> switches the language there. A page that is not translated yet, or that has changed since it was translated, shows
> in English with a note on top. The screenshots show the English editor, so look for the same entry in your
> language, or type the English name into the command palette. Entity classnames, map keys, file formats, the log and
> the [MCP server](../mcp.md) stay in English in every language.

<div class="td-mcp" markdown>

Over the [MCP server](../mcp.md), `set_editor` with `"language": "de"` switches the language. The values are `en`, `de`,
`fr`, `es`, `pt`, `ru`, `zh-Hans` and `ja`, in any case, and an unknown one is an error that names them all and changes
nothing. `get_state` reports the choice as `ui.language`. Tool names, replies and errors stay English in every
language. An editor started with `--default-prefs` is in English, which the e2e tests rely on.

</div>

## Interface scale

Ctrl+= and Ctrl+- scale the interface and Ctrl+0 resets it. You find the same as *Interface scale* at the top of
*File > Preferences*. If the monitor reports the wrong display scaling, untick *follow the monitor* right beneath it.

![The top of the Preferences window: Theme set to Dark, Language set to English, Interface scale at 100%, Display scaling with follow the monitor ticked, and below them Tips on an empty map and Help text](../assets/interface/preferences-scale-and-tips.png)
