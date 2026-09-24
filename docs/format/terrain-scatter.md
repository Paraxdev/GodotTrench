# Terrain and scatter

## Terrain

A heightmap on a regular grid: one height per grid vertex, painted with up to four texture layers. The bulk data is
base64 text in JSON, and raw bytes in the [binary container](container.md#value-encoding).

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `origin` | `[x, y, z]` | required | World position of the first vertex |
| `resolution` | `[nx, nz]` | required | Vertex count along X and Z, at least 2 each |
| `cell_size` | number | required | Map units between neighbouring vertices |
| `heights` | base64 | required | Little endian 32 bit floats, one height per vertex |
| `layers` | array | required | Up to four texture layers, see below |
| `splat` | base64 | omitted | Four weight bytes per vertex, one per layer, saying how much each layer shows there |
| `holes` | base64 | omitted | One byte per cell, non-zero cuts the cell out of the terrain |
| `chunk_cells` | integer | `32` | Always written. Cells per chunk side. The terrain is split into chunks of this size for rendering and collision |

Vertex `(i, j)` sits at `origin + (i * cell_size, heights[j * nx + i], j * cell_size)`. `splat` uses the same order,
its weights are normalized on read, and an all zero entry means layer 0. `holes` covers the `(nx - 1) * (nz - 1)`
cells in the same order.

Each layer:

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `material` | string | required | Material painted by this layer |
| `tile` | number | `256` | Map units per texture repeat |
| `detile` | number | `0` | 0 to 1, how strongly the repeat is broken up so the grid pattern stops showing |
| `detile_sharpen` | number | `0.5` | 0 to 1, how crisp the de-tiled result stays |

See [Terrain layers](../editor/terrain-layers.md) for how these settings look in practice.

Terrains stay axis aligned. An instance moves a terrain's center by its whole transform, rotation included, but never
rotates the grid itself.

## Scatter

A scatter set is many copies of a few models, such as trees, rocks or grass, painted onto surfaces and stored as one
node. See [Scatter](../editor/scatter.md) for the editor side. Every key except `targets` and `material` is always
written.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `name` | string | required | Name shown in the editor |
| `kind` | string | `props` | What Godot builds. `props` keeps scripts and collision, `foliage` is a MultiMesh without collision |
| `targets` | array of node ids | omitted | The surfaces the set is painted on |
| `items` | array | required | The models it scatters, see below |
| `collision` | string | `none` for foliage, else `convex` | Collision shape per instance: `none`, `convex` or `trimesh` |
| `cast_shadows` | bool | `true` | Whether the instances cast shadows |
| `visibility_range` | number | `0` | Hide instances beyond this distance, 0 is never |
| `chunk_size` | number | `0` | Split the set into cells of this size so Godot can cull what is off screen, 0 is one cell. New sets use 2048 |
| `static_props_multimesh` | bool | `false` | Draw prop scenes that carry scripts as MultiMesh too. Cheaper for many props, but their scripts are dropped |
| `material` | string | omitted | Material override for the whole set |
| `instances` | array | empty | The placed instances, see below |

Copying a set together with a surface it targets makes the copy target the copied surface.

Each item is one model the set can place:

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `source` | string | required | `res://` model or scene |
| `weight` | number | `1` | Relative chance of this item being picked |
| `scale` | `[min, max]` | `[0.8, 1.2]` | Range for a random uniform scale |
| `spacing` | number | `48` | Minimum distance to every other instance |
| `align` | number | `0` | 0 stands upright, 1 follows the surface normal, values between lean part way |
| `random_yaw` | bool | `true` | Turn each instance to a random heading |
| `tilt` | number | `0` | Largest random lean in degrees |
| `sink` | number | `0` | Map units pushed into the surface, so roots and rock bases do not float |
| `material` | string | omitted | Material override for this item. It wins over the set's |
| `enabled` | bool | `true`, omitted | Whether the brush paints this item. Instances already placed stay |

Each instance is a flat array `[item, x, y, z, pitch, yaw, roll, scale]`, where `item` indexes `items` and the angles
are degrees in YXZ order. On save the position is rounded to 0.01, the angles to 0.1 and the scale to 0.001, which lets
the container store them as [integer columns](container.md#value-encoding).
