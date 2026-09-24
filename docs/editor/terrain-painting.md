# Terrain painting

The **Blend** tool (Shift+G) paints a terrain's [layers](terrain-layers.md), for example sand along a beach or rock on
a cliff. Like Sculpt, it works on the selected terrains, or on all of them when nothing is selected.

![The Blend tool's options in slope mode](../assets/terrain-painting/blend-toolbar.png)

## The Blend tool

Pick a mode, a falloff, the **layer** (0 to 3), the radius and a strength in the tool options bar. Several light passes
give softer edges than one strong pass.

| Mode | What it paints |
| --- | --- |
| paint | Adds the chosen layer under the brush |
| erase | Paints back towards layer 0, the base ground |
| smooth | Softens the borders between layers |
| sharpen | Lets the strongest layer win, for crisp borders |
| noise | Only in cloud shaped patches of the given **size**, for moss, dry grass or gravel |
| slope | Only where the ground is between two angles. `30` to `90` puts rock on anything steeper than 30° |
| height | Only between two world heights, for sand at a waterline or snow above a tree line |

Slope and height make huge brushes safe, a hill sized brush in slope mode only turns the cliffs to rock.

The falloff decides how the strength fades from the centre of the brush to its rim.

| Falloff | Shape |
| --- | --- |
| smooth | A gentle fade, the usual choice |
| linear | An even fade from centre to rim |
| constant | The whole disc at the same strength, for hard edges |
| spray | Random speckles, for pebbles and flowers |

Shift turns paint, noise, slope and height into erase, and Ctrl smooths. Ctrl+wheel resizes the brush.

## Auto Paint

*Terrain > Blend > Auto Paint Layers* repaints the selected terrains from their shape, a quick first pass before you
paint by hand:

| Layer | Goes on |
| --- | --- |
| 1 | Slopes steeper than about 50° |
| 2 | The top 20% of the height range |
| 3 | The bottom 8% of the height range |

Layer 0 covers everything else. A terrain with fewer layers gets the missing bands in its last layer.

The range starts at the lowest point, which on an island is the sea floor. Tick **Bands from sea level** to measure from
a height you give instead.

> **Warning:** Auto Paint replaces all paint, so sculpt first, auto paint, and paint by hand last.

## How narrow you can paint

Paint lives on the vertices, so a stroke needs a radius of about two cells for a clean band. Below one cell it turns
into a staircase. For a narrow track, use smaller cells, or lay a mesh or [decal](decals.md) on top.

![Strokes with radius 40, 96 and 160 on 64 unit cells, next to the grid of vertices that holds the paint](../assets/terrain-painting/brush-width-grid.jpg)
