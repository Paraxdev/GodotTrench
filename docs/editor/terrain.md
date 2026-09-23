# Terrain

A terrain is one object: a grid of heights with up to four texture layers painted over it. You shape it with **Sculpt**
(G), colour it with **Blend** (Shift+G, see [Terrain painting](terrain-painting.md)), and Godot builds it as chunked
meshes with collision that match the editor.

## Creating a terrain

*Terrain > Create Terrain...* asks for two numbers that decide everything else:

| Setting | Meaning |
| --- | --- |
| Resolution | Vertices along each side, 33 to 1025 |
| Cell size | Map units between two vertices |

The dialog shows the resulting size in units and meters. For example, 257 vertices with 64 unit cells give 16384
units, 512 m across.

The terrain is centred on the 3D cursor, or on the **Center** you type in with **3D cursor** unticked. The center's
height is where the terrain's zero sits.

The dialog can also generate a start shape (hills, mountain, island, valley, ridges, with optional erosion) or read a
grayscale heightmap PNG. Generating replaces the heights, so do it first and sculpt afterwards.

## Picking the cell size

Each vertex stores a height and four paint weights, nothing is stored between vertices. The cell size is therefore the
finest detail the terrain can hold, for shape and paint alike. A mountain range wants big cells, a garden small ones.

Picked wrong? The Inspector's **resample** buttons rebuild the grid at 65, 129, 257 or 513 vertices and keep the
heights, paint and holes.

The terrain is split into chunks of 32 by 32 cells for drawing and collision (**Chunk cells** in the Inspector).
Smaller chunks rebuild faster while sculpting and cull better in Godot, larger ones mean fewer nodes.

## Sculpting

Press **G** and drag over the terrain. With nothing selected, the tool works on every terrain in the map.

| Mode | What it does |
| --- | --- |
| Raise, Lower | Adds or removes height while you hold the button. At strength 4 the centre moves about 32 units per second |
| Smooth | Relaxes each vertex towards its neighbours. Use it after every rough pass |
| Flatten | Pulls the ground to the height where the stroke started. Good for shelves, beaches and building plots |
| Noise | Roughens the surface with small random bumps |
| Terrace | Snaps heights to steps of a given size, for stepped cliffs and paddies |
| Hole, Unhole | Cuts whole cells out, for cave mouths and tunnels. Holes are gone from the collision too |

| Modifier | Effect |
| --- | --- |
| Ctrl+wheel | Resize the brush |
| Shift | Swap raise and lower |
| Ctrl during a stroke | Smooth, whatever the mode |

> **Tip:** Work with a huge brush and a low strength. Small brushes make lumps, and lumps are what give a terrain away
> as hand made.

## In Godot

Every terrain becomes a `GodotTrenchTerrain` node, a `StaticBody3D` with one `MeshInstance3D` per chunk. Each chunk
collides with the exact triangles it draws, so holes are open for the player and characters never float or sink.

For gameplay code, `height_at(x, z)` on the node returns the ground height at a local position.
