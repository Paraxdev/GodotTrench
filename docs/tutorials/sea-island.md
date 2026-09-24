# Sea island 1: terrain

A large island with two hills, cliffs, a beach, paths, woodland and a pier, in five parts. The numbers are the ones
used for the pictures, in map units at 32 per meter.

![The finished island from the south](../assets/sea-island/island-finished.jpg)

## Create the terrain

Open *Terrain > Create Terrain...* and set:

| Setting | Value and why |
| --- | --- |
| Resolution, Cell size | 257 x 257 points, 64 units apart. The dialog shows the resulting size, 16384 units (512 m) |
| Shape | Island, Height 1600, Feature size 3000, Erosion 20, Seed 1. Island rises in the middle and drops towards the edges. Feature size is the width of the largest bumps, and erosion lets steep slopes slump so they look weathered |
| Base, Slope, Peak, Low layer | `showcase/grass`, `showcase/cliff`, `showcase/dirt`, `showcase/sand`. Tile 256 for the first two, 192 for the others. Base covers everything not painted over, the others are meant for steep, high and low ground |
| Center | Untick **3D cursor**, then `0 -576 0` |
| Paint layers from slope and height | Unticked, part 3 paints by hand |

The tile value is how many units one repeat of the texture covers.

![The Create Terrain dialog](../assets/sea-island/create-terrain-dialog.png)

The sea will sit at height 0, so lowering the terrain by 576 floods its outer ring and leaves the middle dry. Later
parts refer to the layers by their slot number: 0 grass, 1 cliff, 2 dirt, 3 sand.

## Add the sea

The sea is a single flat brush the player can walk into.

1. In the Top view, draw a box many times larger than the terrain, or its edge shows on the horizon.
2. In the Front view, make it 32 units thick with its top at height 0.
3. Give it `showcase/water` and set the scale in the Inspector's *Alignment* section to 8 by 8, so the water texture
   does not repeat every few meters.
4. Right click, *Create Brush Entity > func_illusionary*. It renders without collision, so the player can wade in.

![The generated island in the sea, before sculpting](../assets/sea-island/fresh-island.jpg)

Next: [Sea island 2: sculpting](sea-island-sculpt.md).

{% mcp %}

## MCP

The [MCP server](../mcp.md) lets agents and scripts drive the editor.
[examples/mcp/sea_island.json](https://github.com/Paraxdev/GodotTrench/blob/main/examples/mcp/sea_island.json) is an
MCP script that builds the same island, so you can replay it to check a value.

{% endmcp %}
