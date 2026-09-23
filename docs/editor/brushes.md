# Brushes and CSG

Brushes are convex solids. Almost all architecture starts as one.

## Draw, move and copy

All of this uses the **Select** tool (Q).

| Input | Result |
| --- | --- |
| Drag on empty space | Draw a box brush. Height comes from the last brush you drew |
| Drag in the 3D view | Draw on the surface under the cursor |
| Drag the selection | Move it. Shift locks to the main axis |
| Ctrl+drag the selection | Move a copy, leaving the original |
| Alt+drag (3D) | Move vertically |
| Arrow keys, Page Up, Page Down | Nudge by one grid step |
| `[` and `]` | Halve and double the grid |

The selection also shows a gizmo in the 3D view. Arrows move along an axis, squares move in a plane, rings rotate in
15° steps (Shift for 1°) and the boxes scale. *View > Transform Gizmo* hides it.

## Reshape

| Input | Result |
| --- | --- |
| Shift+drag a face (3D) | Push or pull that face |
| Ctrl+Shift+drag a face (3D) | Extrude a new brush out of it |
| Drag a selection edge (2D) | Resize |

## Cut and bend

**Clip** (C) cuts a brush along a plane. Click two or three points, press Tab to choose which side to keep, Enter to
cut. This is how you make slopes, ramps and angled walls.

**Vertex** (V) moves corners and edges directly. The brush is rebuilt as a convex hull afterwards, so you can reshape
it but never dent it. For concave shapes, convert to a [mesh](meshes.md).

## Shape Generator

Ctrl+Shift+B builds cylinders, arches, stairs and other shapes that are tedious by hand. Its **Text** shape builds
block letters from real brushes for signs and floor lettering. Set the letter height, depth and spacing, and choose
upright or lying flat.

## CSG

| Operation | Key | What it does |
| --- | --- | --- |
| Hollow | Ctrl+Shift+K | Turns a brush into walls, 16 units thick by default (*Brush > CSG > Wall thickness*) |
| Subtract | Ctrl+K | Carves the selection out of everything it touches, then deletes the selection |
| Convex Merge | Ctrl+M | Fuses the selection into one convex brush |
| Intersect | Ctrl+L | Keeps only the overlap |

Subtract is how you punch doorways and windows: the selected brush acts as the cutter. The faces a cut opens up take
the cutter's material. Tick *Carved faces keep the target's material* in *Brush > CSG* if you want a window reveal in a
brick wall to stay brick.

## Displacements

For a patch of uneven ground inside a brush level, turn a face into a displacement with *Brush > Displacement* and
sculpt it with the Sculpt tool (G). Outdoor areas are better served by a [terrain](terrain.md).
