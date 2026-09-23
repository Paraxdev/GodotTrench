# Interface and navigation

GodotTrench borrows its habits from TrenchBroom and Hammer. If you know either, most of it will feel familiar.

## The window

**Menus** group commands by what they act on: File, Edit, Brush, Mesh, Texture, Terrain, Gameplay, Tools, View, Godot
and Help. Look for a command under the object it changes.

**The tool options bar** under the toolbar shows the settings of the active tool, or a one line hint on how to use it.

**The views** are 3D, Top, Front and Side. The 2D views show the selection's size in their corner, the quickest way to
check that a doorway really is 64 units wide.

**Panels** dock around the views. The defaults are the Outliner, History and Issues on the left, the Inspector,
Entities, UV Editor and Scatter on the right, and Materials and Models at the bottom. **Logic** and **Reference** are
not shown by default, open them from *View > Panels*. *Reset Layout* in the same menu puts everything back.

> **Tip:** Can't find a command? The command palette (Ctrl+Shift+P or F1) finds it by name.

## Arranging the views

| Do this | To |
| --- | --- |
| Drag the point where the dividers cross | Resize all four views at once |
| Shift+Space | Maximize the view under the pointer, press again to restore |
| Click × on a view tab | Close it, *View > Views* brings it back |
| *View > Views* | Switch between one, two and four views |

The layout is kept for the next start.

## Moving the camera

| Input | 3D view | 2D views |
| --- | --- | --- |
| Right drag | Look around, WASD to move, Q and E for down and up | Pan |
| Middle drag | Pan | Pan |
| Wheel | Dolly | Zoom towards the cursor |
| Alt+left drag | Orbit, when nothing is selected | |
| Shift | Move three times as fast | |

**F** frames the selection in every view, or the whole map when nothing is selected.

**Camera bookmarks:** Ctrl+Shift+1 to 9 stores the 3D camera, Ctrl+1 to 9 jumps back. Bookmarks are saved in the map
and undo does not touch them.

## Shading

F2 cycles the 3D view between Textured, Flat, Lit Preview and Wireframe. **Lit Preview** (F3) shows the map's lights,
sun shadows, sky and fog, a good check before you go to Godot. It previews up to 64 point and spot lights, the brightest
first.

## Interface scale

Ctrl+= and Ctrl+- scale the interface, Ctrl+0 resets it. The scale is also in *View > Interface Scale* and
*File > Preferences*. It multiplies the monitor's display scaling, which you can turn off in Preferences if your monitor
reports the wrong value.
