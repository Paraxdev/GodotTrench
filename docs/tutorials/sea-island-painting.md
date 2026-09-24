# Sea island 3: painting

Switch to the Blend tool with **Shift+G**, with the terrain or nothing selected. Its settings are in the tool options.
The **slope** and **height** modes only paint where the ground is within a range of steepness or height. A radius of
12000 covers the whole island, so one click in these modes paints every shore or cliff at once.

## Sand and rock

| Setting | Sand | Rock |
| --- | --- | --- |
| Mode | height, from `-3000` to `40` | slope, from `32` to `90` |
| Falloff | constant, full strength across the whole brush | constant |
| Layer | 3 | 1 |
| Radius, strength | 12000, 1 | 12000, 1 |

Click once in the middle of the island for each.

![Height mode: everything below 40 units becomes sand](../assets/sea-island/sand-height.jpg)

![Slope mode: everything steeper than 32° becomes rock](../assets/sea-island/rock-slope.jpg)

To widen the beach, paint layer 3 along the south shore in **paint** mode, falloff smooth, radius 700, strength 0.7.
The smooth falloff fades the brush towards its rim, so the edge of the sand stays soft.

## Paths

Mode **paint**, falloff smooth, layer 2, radius 140, strength 0.8. Drag:

1. From the top of the beach up to the main hill in long curves.
2. From the main hilltop along the ridge to the cliff top.
3. From the east end of the beach to the second hill.

![Painting the path up the main hill](../assets/sea-island/paths.jpg)

![The path up close](../assets/sea-island/path-close.jpg)

## Break up the repeat

From far away a repeating texture shows as a grid. Select the terrain and set each layer's **detile** and **sharpen**
in the Inspector. Detile turns and shifts every tile by a random amount so the repeat stops showing, and sharpen keeps
the result crisp instead of soft.

| Layer | Detile | Sharpen |
| --- | --- | --- |
| 0 grass | 0.5 | 0.5 |
| 1 cliff | 0.7 | 0.4 |
| 2 dirt | 0.5 | 0.5 |
| 3 sand | 0.4 | 0.5 |

Next: [Sea island 4: scatter](sea-island-scatter.md).
