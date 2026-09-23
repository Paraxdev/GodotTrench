# Brushes and CSG

Brushes are convex solids. These inputs belong to the **Select** tool (Q).

## Draw and move

| Input | Result |
| --- | --- |
| Drag off the selection | Draw a box brush. In 3D it starts on the surface under the cursor and is one grid step thick, in 2D it takes the depth of the last brush you drew |
| Drag the selection | Move it, Shift locks to the main axis |
| Ctrl+drag the selection | Move a copy |
| Alt+drag the selection (3D) | Move vertically |
| Arrow keys, Page Up, Page Down | Nudge by one grid step |
| `[` and `]` | Halve and double the grid |

In 3D the selection also gets a move, rotate and scale gizmo. Rotation snaps to 15°, or 1° with Shift.
*View > Transform Gizmo* hides it.

## Reshape

| Input | Result |
| --- | --- |
| Shift+drag a selected face (3D) | Push or pull that face |
| Ctrl+Shift+drag a selected face (3D) | Extrude a new brush out of it |
| Drag a selection edge (2D) | Resize |

**Clip** (C) cuts along a plane, for slopes, ramps and angled walls. Click two or three points, press Tab to cycle
between keeping the front, the back or both halves, and Enter to cut. Backspace removes the last point.

**Vertex** (V) drags corners, edge midpoints and face centers. A move that would make the brush concave is refused, so
for dents and overhangs convert it to a [mesh](meshes.md). Delete removes the selected vertices.

## Shape Generator

*Brush > Shape Generator* (Ctrl+Shift+B) builds cylinders, arches, stairs and other shapes. Its **Text** shape
builds block letters from brushes, for signs and floor lettering.

## CSG

| Operation | Key | Result |
| --- | --- | --- |
| Subtract | Ctrl+K | Carves the selection out of every brush it touches, then deletes the selection |
| Hollow | Ctrl+Shift+K | Turns a brush into walls, 16 units thick unless you change *Wall thickness* in *Brush > CSG* |
| Convex Merge | Ctrl+M | Fuses the selection into one convex brush |
| Intersect | Ctrl+L | Keeps only the overlap |

Subtract punches doorways and windows. The faces a cut opens take the cutter's material, unless you tick
*Carved faces keep the target's material* in *Brush > CSG*.

## Displacements

For uneven ground inside a brush level, select a four sided face, use *Brush > Displacement > Create* and sculpt it with
G. Outdoor areas want a [terrain](terrain.md).
