# Shortcuts

These are the defaults of the TrenchBroom preset, used by a fresh install. *Help > Keyboard Shortcuts...* lists every
binding and lets you change any of them. *File > Preferences > Keymap preset* switches to Hammer or Blender.

## Tools

Pressing a tool's key again goes back to Select. Esc cancels a drag or stroke in progress, otherwise it returns to
Select, and in Select it clears the selection.

| Key | Tool |
| --- | --- |
| Q | Select, draw and move |
| C | Clip |
| V | Vertex |
| R | Rotate |
| T | Scale |
| Tab | Mesh edit, converting selected brushes |
| Shift+T | Texture |
| P | Paint vertex colours |
| G | Sculpt terrain and displacements |
| Shift+G | Blend materials |
| B | Scatter |
| Shift+E | Volume |
| Shift+P | Path |
| M | Measure |

## Mouse in the views

| Input | Result |
| --- | --- |
| Drag on empty space | Draw a brush |
| Drag the selection | Move. Ctrl copies, Alt moves vertically in 3D, Shift locks the axis |
| Click | Select. Double click selects the whole group |
| Ctrl+click | Add to or remove from the selection |
| Shift+click | Select one face |
| Ctrl+Shift+click | Add or remove a face |
| Shift+drag a face (3D) | Push or pull it |
| Ctrl+Shift+drag a face (3D) | Extrude a new brush |
| Right drag (3D) | Look, WASD to move, Q and E for down and up |
| Middle drag | Pan |
| Alt+left drag (3D, nothing selected) | Orbit |
| Ctrl+wheel (Sculpt, Blend, Paint, Scatter) | Resize the brush |

## Editing

| Key | Action |
| --- | --- |
| Ctrl+Z, Ctrl+Shift+Z or Ctrl+Y | Undo, redo |
| Ctrl+X, Ctrl+C, Ctrl+V | Cut, copy, paste at the cursor |
| Ctrl+D | Duplicate |
| Ctrl+Shift+D | Duplicate linked |
| Ctrl+R | Repeat the last rotate, flip, nudge or duplicate. Loop cut in mesh editing |
| Delete | Delete |
| Ctrl+A, Esc, Ctrl+I | Select all, none, inverse |
| Ctrl+T, Ctrl+Shift+T | Select touching, select inside |
| Ctrl+G, Ctrl+Shift+G | Group, ungroup |
| Ctrl+H, Ctrl+J, Ctrl+Shift+H | Hide, isolate, show all |
| Arrows, Page Up, Page Down | Nudge by one grid step |

## Brushes, meshes and textures

| Key | Action |
| --- | --- |
| Ctrl+K | CSG subtract |
| Ctrl+M | CSG convex merge |
| Ctrl+L | CSG intersect |
| Ctrl+Shift+K | Hollow |
| Ctrl+Shift+B | Shape generator |
| Ctrl+Shift+E | Convert brushes to mesh |
| Ctrl+Shift+J | Join meshes |
| Ctrl+Shift+W | Move brushes to world |
| Ctrl+Shift+U | Toggle UV lock |
| Alt+H | Hotspot fit |

## View and files

| Key | Action |
| --- | --- |
| F or Ctrl+U | Frame the selection, or the whole map |
| `[` and `]`, or `-` and `=` | Halve or double the grid |
| F2 | Cycle shading |
| F3 | Lit Preview |
| Shift+Space | Maximize the view under the pointer |
| Ctrl+Shift+1 to 9, Ctrl+1 to 9 | Store, recall camera bookmarks |
| Ctrl+=, Ctrl+-, Ctrl+0 | Interface scale up, down, reset |
| Ctrl+Shift+P or F1 | Command palette |
| Ctrl+S, Ctrl+Shift+S | Save, save as |
| Ctrl+O | Open map |
| Ctrl+N | New map |
| Ctrl+Shift+N, Ctrl+Tab, Ctrl+W | New tab, next tab, close tab |
| F5 | Run the Godot project |

## Other presets

Presets only take over the keys listed here. Everything else keeps its default, and your own changes sit on top of
whichever preset is active. *Reset overrides* in the shortcuts window clears them.

**Hammer**

| Key | Action |
| --- | --- |
| Shift+S, Shift+X, Shift+V, Shift+A | Select, clip, vertex, texture |
| H, Ctrl+H, U | Hide, isolate, show all |
| Ctrl+W | Move brushes to world |
| Ctrl+M | Shape dialog |
| Ctrl+L | Rotate 90° |
| Ctrl+F4 | Close tab |
| Ctrl+Shift+M, Ctrl+Shift+L | CSG merge, CSG intersect |

**Blender**

| Key | Action |
| --- | --- |
| A, Alt+A | Select all, none |
| X | Delete |
| Shift+D, Alt+D | Duplicate, duplicate linked |
| S | Scale |
| H, Alt+H | Hide, show all |
| / | Isolate |
| Ctrl+J | Join meshes |
| Z | Cycle shading |
| Shift+A | Shape dialog |
| F3 | Command palette |
| Alt+Shift+H | Hotspot fit |
| F4 | Lit Preview |
