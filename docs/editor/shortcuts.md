# Shortcuts

These are the defaults of the TrenchBroom preset. *Help > Keyboard Shortcuts…* lists every binding and rebinds any of
them, and *File > Preferences > Keymap preset* switches to Hammer or Blender.

## Tools

Pressing a tool's key again goes back to Select. Esc cancels a drag or stroke, otherwise returns to Select, and in
Select clears the selection.

| Key | Tool |
| --- | --- |
| Q | Select, draw and move |
| C | Clip |
| V | Vertex |
| R | Rotate |
| T | Scale |
| Tab | Mesh editing, converting selected brushes |
| Shift+T | Texture |
| P | Vertex paint |
| G | Sculpt terrain and displacements |
| Shift+G | Blend terrain layers |
| B | Scatter |
| Shift+E | Volume |
| Shift+P | Path |
| M | Measure |

## Mouse in the views

| Input | Result |
| --- | --- |
| Drag off the selection | Draw a brush |
| Drag the selection | Move. Ctrl copies, Shift locks the axis, Alt moves vertically in 3D |
| Click, double click | Select, select the whole group |
| Ctrl+click | Add to or remove from the selection |
| Shift+click, Ctrl+Shift+click | Select one face, add or remove a face |
| Shift+drag a selected face (3D) | Push or pull it |
| Ctrl+Shift+drag a selected face (3D) | Extrude a new brush |
| Right drag (3D) | Look, WASD to move, Q and E for down and up |
| Middle drag | Pan |
| Alt+left drag (3D, nothing selected) | Orbit |
| Ctrl+wheel (Sculpt, Blend, Paint, Scatter) | Resize the brush |

## Editing

| Key | Action |
| --- | --- |
| Ctrl+Z, Ctrl+Shift+Z or Ctrl+Y | Undo, redo |
| Ctrl+X, Ctrl+C, Ctrl+V | Cut, copy, paste under the mouse |
| Ctrl+D, Ctrl+Shift+D | Duplicate, duplicate linked |
| Ctrl+R | Repeat the last rotate, flip, nudge or duplicate. Loop cut in mesh editing |
| Delete or Backspace | Delete |
| Ctrl+A, Esc, Ctrl+I | Select all, none, inverse |
| Ctrl+T, Ctrl+Shift+T | Select touching, select inside |
| Ctrl+G, Ctrl+Shift+G | Group, ungroup |
| Ctrl+H, Ctrl+J, Ctrl+Shift+H | Hide, isolate, show all |
| Arrows, Page Up, Page Down | Nudge by one grid step |

## Brushes, meshes and textures

| Key | Action |
| --- | --- |
| Ctrl+K, Ctrl+M, Ctrl+L | CSG subtract, convex merge, intersect |
| Ctrl+Shift+K | Hollow |
| Ctrl+Shift+B | Shape generator |
| Ctrl+Shift+E, Ctrl+Shift+J | Convert brushes to mesh, join meshes |
| Ctrl+Shift+W | Move brushes to world |
| Ctrl+Shift+U | Toggle UV lock |
| Alt+H | Hotspot fit |

## View and files

| Key | Action |
| --- | --- |
| F or Ctrl+U | Frame the selection, or the whole map |
| `[` and `]`, or `-` and `=` | Halve or double the grid |
| F2, F3 | Cycle shading, toggle Lit Preview |
| Shift+Space | Maximize the view under the pointer |
| Ctrl+Shift+1 to 9, Ctrl+1 to 9 | Store, recall camera bookmarks |
| Ctrl+=, Ctrl+-, Ctrl+0 | Interface scale up, down, reset |
| Ctrl+Shift+P or F1 | Command palette |
| Ctrl+N, Ctrl+O | New map, open map |
| Ctrl+S, Ctrl+Shift+S | Save, save as |
| Ctrl+Shift+N, Ctrl+Tab, Ctrl+W | New tab, next tab, close tab |
| F5 | Run the Godot project |

## Other presets

A preset only changes the keys below. Your own bindings sit on top of any preset, *Reset overrides* in the shortcuts
window clears them.

| Key | Hammer preset |
| --- | --- |
| Shift+S, Shift+X, Shift+V, Shift+A | Select, clip, vertex, texture tool |
| H, Ctrl+H, U | Hide, isolate, show all |
| Ctrl+W | Move brushes to world |
| Ctrl+M | Shape generator |
| Ctrl+L | Rotate 90° around the vertical axis |
| Ctrl+Shift+M, Ctrl+Shift+L | CSG convex merge, intersect |
| Ctrl+F4 | Close tab |

| Key | Blender preset |
| --- | --- |
| A, Alt+A | Select all, none |
| X | Delete |
| Shift+D, Alt+D | Duplicate, duplicate linked |
| S | Scale tool |
| H, Alt+H, / | Hide, show all, isolate |
| Ctrl+J | Join meshes |
| Z | Cycle shading |
| Shift+A | Shape generator |
| F3 | Command palette |
| F4 | Lit Preview |
| Alt+Shift+H | Hotspot fit |
