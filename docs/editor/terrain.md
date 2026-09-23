# Terrain

A terrain is a grid of heights with up to four texture layers painted over it. Shape it with **Sculpt** (G), then
texture it with [layers](terrain-layers.md) and the [Blend tool](terrain-painting.md).

## Creating a terrain

*Terrain > Create Terrain…* opens a dialog. Two settings decide the size:

| Setting | Meaning |
| --- | --- |
| Resolution | Vertices along each side, from 33 to 1025 |
| Cell size | Map units between two vertices |

For example, 257 vertices with 64 unit cells give 16384 units, 512 m. The terrain is centred on the 3D cursor, or on
the **Center** you type in with **3D cursor** unticked, and that height becomes the terrain's zero.

**Shape** starts it as hills, a mountain, an island, a valley or ridges, and **Import PNG…** reads a grayscale
heightmap instead. Both only work on creation. The layer fields set up the paint in the same step, see
[Auto Paint](terrain-painting.md#auto-paint).

## Picking the cell size

Nothing sits between two vertices, so the cell size is the finest detail the terrain can hold. A mountain range wants
big cells, a garden small ones. The Inspector's **resample** buttons rebuild the grid at 65, 129, 257 or 513 vertices
and keep the heights, paint and holes.

**Chunk cells** (32 by default) sets the chunk size for drawing and collision. Smaller chunks rebuild faster while
sculpting, larger ones mean fewer nodes in Godot.

## Sculpting

Press **G** and drag over the terrain. It works on the selected terrains, or on all of them when nothing is selected.

| Mode | Result |
| --- | --- |
| Raise, Lower | Adds or removes height while you hold the button |
| Smooth | Relaxes each vertex towards its neighbours |
| Flatten | Pulls the ground to the height where the stroke started |
| Noise | Adds small random bumps |
| Terrace | Snaps heights to steps of **step** units |
| Hole, Unhole | Cuts whole cells out or fills them back, collision included |

Ctrl+wheel resizes the brush, Shift swaps raise and lower, and Ctrl smooths whatever the mode.

> **Tip:** Use a large brush at low strength and smooth after every rough pass. Small brushes leave lumps.

## In Godot

Every terrain becomes a `GodotTrenchTerrain` node, a `StaticBody3D` whose collision matches what it draws. For gameplay
code, `height_at(x, z)` returns the ground height at a local position.
