# Meshes

Unlike brushes, meshes can have dents, holes and bent faces, so use them for rocks, pipes, smooth arches and other
shapes brushes cannot make.

*Mesh > Edit Mesh* converts the selected brushes to a mesh and starts editing it. With a mesh selected **Tab** does the
same, and Tab again goes back to objects. Tab on brushes opens the Vertex tool instead, so a stray key press never
turns a brush into a mesh. The Mesh tool button on the toolbar starts editing without converting anything.
Ctrl+Shift+E only converts, Ctrl+Shift+J joins meshes into one.

## Keys

The keys follow Blender.

| Key | Action |
| --- | --- |
| 1, 2, 3 | Work on vertices, edges or faces |
| A, Alt+A | Select all, select none |
| Alt+click, Ctrl+Alt+click | Select the edge loop through the clicked edge, or the ring across it |
| Ctrl+L | Select everything connected to the selection |
| G, R, S | Move, rotate, scale |
| E | Extrude new geometry out of the selection |
| I | Inset, a smaller face inside each selected face |
| Ctrl+B, Ctrl+Shift+B | Bevel edges or vertices, cutting the corner off |
| Ctrl+R | Loop cut, a new ring of edges across the mesh. The wheel or + and - set the number of cuts |
| K | Knife, click two points to cut along that line |
| M | Merge the selected vertices at their center |
| F | Fill, a new face across the selected vertices |
| X or Delete | Delete |
| Shift+D | Duplicate and move |
| P | Separate the selection into a new mesh |
| Alt+F | Flip normals, for faces that show from the wrong side |

While moving, rotating or scaling, press X, Y or Z to lock to that axis, or Shift with the letter to lock it out. Type
a number for an exact amount. Click or Enter confirms, right click or Esc cancels.

## Gizmo

Double click an element to put a move, rotate and scale gizmo on it. It is the clean way to slide a vertex along one
axis, and it works in the 2D views too. Esc hides it and keeps the selection.

The *Mesh* menu holds the rest, such as subdivide, triangulate, solidify, mirror and smooth shading.
