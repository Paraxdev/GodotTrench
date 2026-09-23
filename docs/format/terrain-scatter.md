# Terrain and scatter

## Terrain

A heightmap on a regular grid. The bulk data is base64 text in JSON, and raw bytes in the
[binary container](container.md#value-encoding).

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `origin` | `[x, y, z]` | required | World position of the first vertex |
| `resolution` | `[nx, nz]` | required | Vertex count along X and Z, at least 2 each |
| `cell_size` | number | required | Map units between vertices |
| `heights` | base64 | required | Little endian 32 bit floats, one per vertex |
| `layers` | array | required | Up to four texture layers |
| `splat` | base64 | omitted | Four weight bytes per vertex, one per layer |
| `holes` | base64 | omitted | One byte per cell, non-zero is a hole |
| `chunk_cells` | integer | `32` | Always written. Cells per chunk side for rendering and collision |

Vertex `(i, j)` sits at `origin + (i * cell_size, heights[j * nx + i], j * cell_size)`. `splat` uses the same order, its
weights are normalized on read, and an all zero entry means layer 0. `holes` covers the `(nx - 1) * (nz - 1)` cells in
the same order.

Each layer:

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `material` | string | required | |
| `tile` | number | `256` | Map units per texture repeat |
| `detile` | number | `0` | 0 to 1, how strongly the repeat is broken up |
| `detile_sharpen` | number | `0.5` | 0 to 1, how crisp the de-tiled result stays |

Terrains stay axis aligned. An instance moves a terrain's center by its whole transform, rotation included, but never
rotates the grid itself.

## Scatter

Many model instances painted onto surfaces, stored as one node. Every key except `targets` and `material` is always
written.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `name` | string | required | |
| `kind` | string | `props` | `props` keeps scripts and collision, `foliage` is MultiMesh without collision |
| `targets` | array of node ids | omitted | Surfaces the set is painted on |
| `items` | array | required | The models it scatters |
| `collision` | string | `none` for foliage, else `convex` | `none`, `convex` or `trimesh` |
| `cast_shadows` | bool | `true` | |
| `visibility_range` | number | `0` | Hide instances beyond this distance, 0 is never |
| `chunk_size` | number | `0` | Split into cells this size for culling, 0 is one cell. New sets use 2048 |
| `static_props_multimesh` | bool | `false` | Draw scripted prop scenes as MultiMesh too, dropping their scripts |
| `material` | string | omitted | Material override for the whole set |
| `instances` | array | empty | The placed instances |

Copying a set together with a surface it targets makes the copy target the copied surface.

Each item:

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `source` | string | required | `res://` model or scene |
| `weight` | number | `1` | Relative chance of being picked |
| `scale` | `[min, max]` | `[0.8, 1.2]` | Random uniform scale |
| `spacing` | number | `48` | Minimum distance to every other instance |
| `align` | number | `0` | 0 upright, 1 aligned to the surface normal |
| `random_yaw` | bool | `true` | |
| `tilt` | number | `0` | Largest random lean in degrees |
| `sink` | number | `0` | Map units pushed into the surface |
| `material` | string | omitted | Override for this item, wins over the set's |
| `enabled` | bool | `true`, omitted | Whether the brush paints this item |

Each instance is a flat array `[item, x, y, z, pitch, yaw, roll, scale]`, where `item` indexes `items` and the angles
are degrees in YXZ order. On save the position is rounded to 0.01, the angles to 0.1 and the scale to 0.001, which lets
the container store them as [integer columns](container.md#value-encoding).
