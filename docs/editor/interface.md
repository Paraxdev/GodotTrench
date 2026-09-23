# Interface and navigation

GodotTrench follows TrenchBroom and Hammer, so most habits from either carry over.

## The window

A menu command sits under the kind of object it changes, and the command palette (Ctrl+Shift+P or F1) finds any
command by name. The **tool options bar** under the toolbar holds the settings of the active tool.

| Dock | Panels |
| --- | --- |
| Left | Outliner, History, Issues |
| Right | Inspector, Entities, UV Editor, Scatter |
| Bottom | Materials, Models |

Logic and Reference start closed. Open any panel from *View > Panels*, and *Reset Layout* in the same menu restores the
defaults.

## Views

The views are 3D, Top, Front and Side. With something selected, the 2D views print its width above it and its height
beside it.

| Do this | To |
| --- | --- |
| Drag the point where the dividers cross | Resize all four views at once |
| Shift+Space | Maximize the view under the pointer, again to restore |
| *View > Views* | Switch between one, two and four views, or bring back a closed view |

## Tabs

Each open map gets a tab, and the tab bar appears once two are open. Ctrl+Shift+N opens a new tab, Ctrl+Tab switches
and Ctrl+W closes one. *New Map* and importing open in a new tab when the current map has unsaved changes. Closing a
modified tab asks to save, discard or cancel, and quitting lists every unsaved map.

## Camera

| Input | 3D view | 2D views |
| --- | --- | --- |
| Right drag | Look, WASD moves, Q and E go down and up, Shift triples the speed | Pan |
| Middle drag | Pan | Pan |
| Wheel | Move forward and back | Zoom towards the cursor |
| Alt+left drag, nothing selected | Orbit | |

**F** frames the selection in every view, or the whole map when nothing is selected.

Ctrl+Shift+1 to 9 stores the 3D camera as a bookmark and Ctrl+1 to 9 jumps back. Bookmarks are saved in the map, and
undo does not touch them.

## Shading

F2 cycles the 3D view through Textured, Flat, Lit Preview and Wireframe. F3 toggles **Lit Preview**, which shows the
map's lights, sun shadows, sky and fog before you build in Godot. It draws only the 64 strongest point and spot lights.

## Interface scale

Ctrl+= and Ctrl+- scale the interface and Ctrl+0 resets it. If the monitor reports the wrong display scaling, untick
*follow the monitor* in *File > Preferences*.
