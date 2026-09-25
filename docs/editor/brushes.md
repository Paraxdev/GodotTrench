# Brushes and CSG

A brush is a convex solid, a shape with no dents such as a box or a wedge. Brushes are quick to draw, snap cleanly to
the grid and become meshes with matching collision in Godot. These inputs belong to the **Select** tool (Q).

## Draw and move

| Input | Result |
| --- | --- |
| Drag off the selection | Draw a box brush. In 3D it starts on the surface under the cursor and is one grid step thick, in 2D it takes the depth of the last brush you drew, 128 units for the first |
| Drag the selection | Move it, Shift locks to the main axis |
| Ctrl+drag the selection | Move a copy, the original stays |
| Alt+drag the selection (3D) | Move vertically |
| Arrow keys, Page Up, Page Down | Nudge by one grid step |
| `[` and `]` | Halve and double the grid |

In 3D the selection also gets a move, rotate and scale gizmo. Rotation snaps to 15°, or 1° with Shift.
*View > Transform Gizmo* hides it.

## Exact position and size

For an exact placement, such as a door cutter as deep as the wall, type the numbers into the **Inspector**. With
brushes or meshes selected it shows the **position** and **size** of their bounds. Position is the lowest corner, the
smallest X, Y and Z, which is the corner that usually sits on the grid. A new size scales the selection away from that
corner, so the position stays put. Dragging a field steps by the grid while snapping is on, a typed value is kept
exactly. The same fields appear for a brush entity and for a selection of several objects.

## Reshape

| Input | Result |
| --- | --- |
| Shift+drag a selected face (3D) | Push or pull that face |
| Ctrl+Shift+drag a selected face (3D) | Extrude a new brush out of it |
| Drag a selection edge (2D) | Resize |

**Clip** (C) cuts along a plane, for slopes, ramps and angled walls. Click two or three points, press Tab to cycle
between keeping the front, the back or both halves, and Enter to cut. Backspace removes the last point.

**Vertex** (V) drags corners, edge midpoints and face centers. A move that would give the brush a dent is refused, so
for dents and overhangs convert it to a [mesh](meshes.md). Delete removes the selected vertices.

## Shape Generator

*Brush > Shape Generator* (Ctrl+Shift+B) builds cylinders, arches, stairs, roofs and other shapes that would take many
hand drawn brushes. Some are always made as a smooth [mesh](meshes.md), and cylinders, cones and spheres can be. The
**Text** shape builds block letters from brushes, for signs and floor lettering.

## CSG

| Operation | Key | Result |
| --- | --- | --- |
| Subtract | Ctrl+K | Carves the selection out of every brush it touches, then deletes the selection |
| Hollow | Ctrl+Shift+K | Turns a brush into walls around its old volume, 16 units thick unless you change *Wall thickness* in *Brush > CSG* |
| Convex Merge | Ctrl+M | Fuses the selection into one convex brush that wraps it |
| Intersect | Ctrl+L | Keeps only where the selected brushes overlap |

To punch a doorway or window, place a brush where the hole goes and Subtract it. The faces a cut opens
take the cutter's material, unless you tick *Carved faces keep the target's material* in *Brush > CSG*.

## Displacements

For uneven ground inside a brush level, like a rubble slope, select a four sided face, use
*Brush > Displacement > Create* and sculpt it with G. Outdoor areas want a [terrain](terrain.md).
