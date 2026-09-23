# Geometry

## Brush

A convex solid stored as its exact corners and faces that index into them, so vertex edits round trip exactly and Godot
builds what the editor shows. Face planes are recomputed on load.

| Key | Type | Meaning |
| --- | --- | --- |
| `vertices` | array of `[x, y, z]` | Corners in map units |
| `faces` | array of faces | One polygon per face |

Each face:

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `indices` | array of integers | required | Vertex indices, at least three, counter-clockwise seen from outside |
| `material` | string | `""` | Always written. Texture name, for example `base/wall` |
| `uv` | object | paraxial | Always written. [Texture projection](#texture-projection) |
| `props` | object of string to string | omitted | Free-form face attributes |
| `disp` | object | omitted | [Displacement](#displacement) |
| `colors` | array of `[r, g, b, a]` | omitted | Vertex paint, one per index |

The Godot build skips a brush with fewer than four vertices or faces. The face props `blend_material`, `blend_detile`,
`blend_uv_scale` and `blend_detile_sharpen` set up material blending, where the vertex color alpha blends from
`material` towards `blend_material`.

## Texture projection

`uv` is a Valve 220 style projection with explicit axes.

| Key | Type | Meaning |
| --- | --- | --- |
| `u_axis`, `v_axis` | `[x, y, z]` | World direction of the texture's u and v |
| `offset` | `[u, v]` | Shift in texels |
| `scale` | `[u, v]` | Map units per texel |
| `rotation` | number | Optional and informational, the axes already contain it |

A point `p` lands on texel `dot(p, u_axis) / scale.x + offset.x`, and the same for v. The final UV divides the texel by
the texture size, which is the material's `texture_size` when it has one, see
[Materials](../godot/materials.md), and otherwise the image's pixel size.

A missing `uv` means `u_axis = [1, 0, 0]` and `v_axis = [0, 0, 1]` at scale 1. When `uv` is present, every key but
`rotation` is required.

## Displacement

A quad face can carry a Hammer style displacement, a grid of vertices pushed along the face normal.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `power` | integer | required | 1 to 4, the grid has `2^power + 1` vertices per side |
| `heights` | array of numbers | required | Offset along the normal per grid vertex |
| `alphas` | array of numbers | omitted | Blend weight per grid vertex, 0 is the face material |

Both arrays are row major with `(2^power + 1)^2` entries. Rows run from the face's first corner towards its fourth,
columns from the first corner towards the second.

## Mesh

An editable polygon mesh, which may be concave, open or non-planar.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `vertices` | array of `[x, y, z]` | required | |
| `faces` | array of faces | required | The keys of brush faces, plus `uvs` |
| `smooth_angle` | number | `0`, omitted | Faces meeting at less than this many degrees share normals, 0 is flat shading |
| `decal` | bool | `false`, omitted | A decal sheet, blended over the surface behind it with both sides visible, see [Decals](../editor/decals.md) |

A mesh face's `uvs` is an array of `[u, v]`, one per index, in texture widths. When present it replaces `uv`, and a
count that does not match `indices` makes the editor refuse the map.
