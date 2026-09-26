# Terrain

A terrain is a grid of heights with up to four texture layers painted over it. Use it for open ground like hills,
beaches and valleys, where shaping brushes by hand would take forever. Shape it with **Sculpt** (G), then texture it
with [layers](terrain-layers.md) and the [Blend tool](terrain-painting.md).

## Creating a terrain

*Terrain > Create Terrain…* opens a dialog. Two settings decide the size:

| Setting | Meaning |
| --- | --- |
| Resolution | How many vertices run along each side, from 33 to 1025. More vertices hold more detail but cost more to draw and sculpt |
| Cell size | The distance in map units between two neighbouring vertices |

For example, 257 vertices with 64 unit cells give 16384 units, 512 m. The terrain is centred on the 3D cursor, or on
the **Center** you type in with **3D cursor** unticked, and that height becomes the terrain's zero.

**Shape** gives you a starting point instead of a flat plane: hills, a mountain, an island, a valley or ridges.
**Import PNG…** reads a grayscale heightmap instead, where brighter pixels are higher. Both only work on creation. The
layer fields set up the paint in the same step, see [Auto Paint](terrain-painting.md#auto-paint).

## Picking the cell size

Nothing sits between two vertices, so the cell size is the finest detail the terrain can hold. A mountain range wants
big cells, a garden small ones. If you picked wrong, the Inspector's **Resample** menu rebuilds the grid at 65, 129,
257 or 513 vertices along the longer side and keeps the heights, paint and holes. The terrain keeps its size, so more
vertices means smaller cells.

**Chunk cells** (32 by default) splits the terrain into square pieces for drawing and collision. Smaller chunks rebuild
faster while sculpting, larger ones mean fewer nodes in Godot.

## Sculpting

Press **G** and drag over the terrain. It works on the selected terrains, or on all of them when nothing is selected.

| Mode | What it does |
| --- | --- |
| Raise, Lower | Builds height up or digs it down for as long as you hold the button. A click makes a small bump |
| Smooth | Relaxes each vertex towards its neighbours, to soften lumps and sharp ridges |
| Flatten | Pulls the ground to the height where the stroke started, for building pads, roads and plateaus |
| Noise | Adds small random bumps, so flat ground looks less artificial |
| Terrace | Snaps heights to steps of **step** units, for rice fields or stepped cliffs |
| Hole, Unhole | Cuts whole cells out or fills them back, collision included. Use it for cave or bunker entrances |

Ctrl+wheel resizes the brush, Shift swaps raise and lower, and Ctrl smooths whatever the mode.

Strength sets how fast the brush works. Raise and Lower move the centre of the brush 8 units a second per point of
strength whatever the radius, so the default strength 4 builds 32 units a second. For hills and islands turn the
strength up rather than holding for a minute. Keep the radius above the terrain's cell size, a smaller brush only
nudges the one vertex under it.

> **Tip:** Shape with a large brush and smooth after every rough pass. Small brushes leave lumps.

## In Godot

Every terrain becomes a `GodotTrenchTerrain` node, a `StaticBody3D` (Godot's node for solid ground that never moves)
whose collision matches what it draws. For gameplay code, `height_at(x, z)` returns the ground height at a local
position, handy for placing things on the ground at runtime.

{% mcp %}

## MCP

The `terrain_edit` tool's `sculpt` op applies a single dab, and its strength is how far that dab moves the centre, again
whatever the radius. Holding a stroke for one second at strength 4 matches a dab of strength 32.

{% endmcp %}
