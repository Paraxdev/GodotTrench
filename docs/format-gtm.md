# The .gtm map format

A `.gtm` file is a GodotTrench map: one UTF-8 JSON document holding worldspawn properties, some editor state and a tree
of nodes (layers, groups, entities, brushes, meshes, terrains, scatter sets and prefab instances). The editor reads and
writes it through `crates/gt_doc/src/format.rs`, and the Godot addon builds it with
`godot/addons/func_godot/src/godottrench/gtm_parser.gd`. Everything below is taken from those two readers and the serde
definitions they use.

## Format name and version

Every map starts with

```json
{
  "format": "godottrench-map",
  "version": 1,
```

The editor refuses a file whose `format` is anything else, and refuses a `version` newer than the one it knows (currently
1) instead of opening it and silently dropping data it does not understand. Older versions go through a migration step
before parsing; version 1 is the first format, so that step does nothing yet. A file without `version` counts as the
oldest format. Keys the editor does not know are ignored on load and therefore lost on the next save.

Before building anything, the editor checks the geometry a hand edit could break: face indices, displacement and terrain
array sizes. A file that fails is refused with the node id and the reason, for example
`node 12: face 3 uses vertex 9, but there are only 8 vertices`, rather than opening half broken. Pasted nodes go through
the same check.

The Godot importer checks `format` but not `version`.

Saving writes `<name>.gtm.tmp` first and renames it over the map, so a crash never leaves a truncated file.

## Top level object

| Key | Type | Written | Meaning |
|---|---|---|---|
| `format` | string | always | `"godottrench-map"` |
| `version` | integer | always | format version, 0 when missing |
| `properties` | object of string to string | when not empty | worldspawn key/values |
| `editor` | object | when not all default | editor state, see below |
| `layers` | array of nodes | always | the layer nodes, in order |

`properties` are the worldspawn entity's keys. Values are always strings, like in a `.map` file, and keys are written in
sorted order. The Godot importer copies them onto the worldspawn and then forces `classname` to `worldspawn`. The
environment keys (`sun_angles`, `sun_color`, `sun_energy`, `ambient_color`, `sky_top_color`, `sky_horizon_color`,
`sky_ground_color`, `fog_color`, `fog_density`, and `environment = 0` to switch it off) are read by both the editor's lit
preview and `GodotTrenchEnvironment`.

Only nodes of type `layer` are taken from `layers`, anything else at that level is skipped by the editor. A file with no
layers gets a `Default` layer on load.

### Editor data

`editor` holds state that belongs to the map but is never exported. Each field is left out while it has its default, and
the whole object is left out when all of them do.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `cameras` | object, slot `"1"` to `"9"` to bookmark | empty | saved 3D viewpoints, each `{"position": [x, y, z], "yaw": r, "pitch": r}` with angles in radians |
| `cordon` | `{"min": [x, y, z], "max": [x, y, z]}` | none | cordon box in map units |
| `cordon_enabled` | bool | `false` | objects outside the cordon are hidden and left out of cordoned exports |

## Nodes

Every node is an object with the same outer shape:

| Key | Type | Written | Meaning |
|---|---|---|---|
| `id` | integer | always | node id, unique in the map |
| `type` | string | always | `layer`, `group`, `entity`, `brush`, `mesh`, `terrain`, `scatter` or `instance` |
| fields of the type | | | see the sections below, written inline next to `id` and `type` |
| `hidden` | bool | when `true` | hidden in the editor views |
| `locked` | bool | when `true` | cannot be selected or edited |
| `children` | array of nodes | when not empty | child nodes, in order |

Keys come out in that order: `id`, `type`, the type's own fields, then `hidden`, `locked` and `children`.

`hidden` and `locked` are editor state and apply to the whole subtree. The Godot importer reads neither, so a hidden
brush is still built. To leave content out of the build put it on a layer with `omit_from_export`.

### Ids

Ids are positive integers that stay the same across saves. They are how the rest of the system refers to a node: the
live link sends edits by id, scatter sets name their target surfaces by id, and Godot tags every generated node with the
id it came from (`_gt_id` metadata, see [godot.md](godot.md)). The editor hands out new ids counting up from the highest id
in the file. When loading, an id of `0` or one already used earlier in the file (which only happens in hand edited
files) is replaced with a fresh one.

Nodes inside prefab instances come from another file and may reuse ids of the including map, so the Godot importer
offsets them by one million per level of instance nesting.

### Tree rules

The file does not enforce a schema for which node can sit where, but the editor produces and the importer expects this
shape:

* layers only at the top level
* groups under layers or other groups
* brushes, meshes, terrains, scatter sets, point entities and instances under layers or groups
* an entity with children is a brush entity, its geometry is its brush and mesh children

Brushes and meshes that are not inside an entity belong to worldspawn.

## Layer

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | string | required | |
| `color` | string | required | `#rrggbbaa` (`#rrggbb` is accepted when reading), the layer's tint in the editor |
| `omit_from_export` | bool | `false` | always written, the importer skips the whole layer |

## Group

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | string | required | |
| `link_id` | integer | none, omitted | groups sharing a link id are linked copies of each other |
| `transform` | 16 numbers | identity, omitted | placement of this copy relative to the others in its link set |

The contents of a group are stored in world space, already placed. `transform` is not applied again when building, it
only records how the linked copies relate so that an edit in one copy can be mirrored into the others: the editor maps
the edited contents back through the inverse of that copy's transform and out through each other copy's. The matrix is
a 4x4 in column-major order. The Godot importer turns groups into FuncGodot groups and ignores `link_id` and `transform`.

## Entity

| Key | Type | Default | Meaning |
|---|---|---|---|
| `classname` | string | required | entity class from the game config |
| `origin` | `[x, y, z]` | `[0, 0, 0]` | always written, position of a point entity |
| `angles` | `[pitch, yaw, roll]` | `[0, 0, 0]` | always written, degrees |
| `properties` | object of string to string | empty, omitted | entity keys, written sorted |
| `outputs` | array | empty, omitted | I/O connections, see below |

An entity without children is a point entity placed by `origin` and `angles`. An entity with children is a brush entity:
its shape comes from its direct `brush` and `mesh` children, and `origin` and `angles` are still written but not used by
the importer.

`angles` are node rotation degrees around X (pitch), Y (yaw) and Z (roll), applied in Godot's YXZ order, the same as
`rotation_degrees` on a `Node3D`. The importer converts them to Quake style angles for FuncGodot, unless the entity already
has an `angles`, `angle` or `mangle` property, which then wins.

Property values are strings whatever the FGD type, for example `"light_energy": "2.5"` or `"travel": "0 168 0"`.
`targetname` names the entity for I/O.

### Outputs

Each output is a Hammer style connection: when `output` fires on this entity, call `input` on every entity matched by
`target`.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `output` | string | required | signal or output name on this entity |
| `target` | string | required | targetname (with `*` wildcards), `@group`, a node path, `!player`, `!activator` or `!self` |
| `input` | string | required | method to call on the target |
| `parameter` | string | `""`, omitted | value passed to the input |
| `delay` | number | `0`, omitted | seconds before the input is called |
| `times` | integer | `-1`, omitted | how many times it may fire, `-1` for every time |

How targets resolve and parameters spread into arguments is described in [gameplay.md](gameplay.md).

## Brush

A brush is a convex solid stored as its exact vertices and a list of faces that index into them.

| Key | Type | Meaning |
|---|---|---|
| `vertices` | array of `[x, y, z]` | corner positions in map units |
| `faces` | array of faces | one polygon per face |

Quake style `.map` files store three points per plane and every tool has to intersect the planes again to find the
corners, which drifts in floating point, turns vertex edits lossy and can make two programs disagree about the same
brush. A `.gtm` brush stores the corners themselves, so vertex editing round trips exactly and Godot builds exactly what
the editor shows: the importer marks these brushes as exact and hands the face polygons to FuncGodot instead of letting
it clip planes. The face planes are recomputed from the vertices on load and are not stored. Corners produced by
clipping planes are snapped to whole numbers when they are within 0.00001 of one.

Each face:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `indices` | array of integers | required | vertex indices, counter-clockwise seen from outside the brush |
| `material` | string | `""` | always written, texture name as FuncGodot looks it up, for example `base/wall` |
| `uv` | object | paraxial projection | always written, texture projection, see below |
| `props` | object of string to string | empty, omitted | free-form face attributes |
| `disp` | object | none, omitted | displacement, see below |
| `colors` | array of `[r, g, b, a]` | empty, omitted | vertex paint, one color per entry of `indices` |

The editor refuses a map where a face has fewer than three indices or an index past the last vertex. The importer skips a face with fewer than three indices and
a brush with fewer than four vertices or four faces.

Face `props` carry surface flags and similar per face data. The keys `blend_material`, `blend_detile`, `blend_uv_scale`
and `blend_detile_sharpen` set up material blending: the vertex color alpha blends from `material` towards
`blend_material`.

### Texture projection

`uv` is a Valve 220 style projection with explicit axes, so a face's texture never depends on which axis a tool would
have picked for it:

| Key | Type | Meaning |
|---|---|---|
| `u_axis` | `[x, y, z]` | world direction of the texture's u |
| `v_axis` | `[x, y, z]` | world direction of the texture's v |
| `offset` | `[u, v]` | shift in texels |
| `scale` | `[u, v]` | map units per texel |
| `rotation` | number | degrees, optional, defaults to `0` |

A point `p` on the face lands on texel `dot(p, u_axis) / scale.x + offset.x` (the same for v), and the texel is divided
by the texture's pixel size for the final UV. `rotation` is informational only, the axes already contain it. When `uv`
is missing the projection is the axis aligned one for an upward facing face (`u_axis = [1, 0, 0]`,
`v_axis = [0, 0, 1]`), and when `uv` is present all fields except `rotation` are required.

### Displacement

A quad face (exactly four indices) can carry a Hammer style displacement, a grid of vertices pushed along the face
normal.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `power` | integer | required | the grid has `2^power + 1` vertices per side, normally 2, 3 or 4 |
| `heights` | array of numbers | required | offset along the face normal per grid vertex, in map units |
| `alphas` | array of numbers | empty, omitted | blend weight per grid vertex, 0 is the face material, 1 the blend material |

Both arrays are row major with `(2^power + 1)^2` entries. Rows run from the face's first corner towards its fourth,
columns from the first corner towards its second. A displacement whose `heights` count does not match is ignored.

## Mesh

Editable polygon meshes. Unlike brushes they may be concave, open or non-planar.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `vertices` | array of `[x, y, z]` | required | positions in map units |
| `faces` | array of faces | required | |
| `smooth_angle` | number | `0`, omitted | faces meeting at less than this many degrees share normals, 0 is flat shading |
| `decal` | bool | `false`, omitted | a decal sheet made by the decal tool |

Mesh faces have the same keys as brush faces (`indices`, `material`, `uv`, `props`, `disp`, `colors`), with `indices`
counter-clockwise seen from the front, plus one more:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `uvs` | array of `[u, v]` | empty, omitted | explicit UV per corner in texture space (1.0 is one texture width), used instead of `uv` when there is one per index |

## Terrain

A heightmap on a regular grid. The bulk data is base64 so a large terrain stays one line per array instead of thousands
of numbers.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `origin` | `[x, y, z]` | required | world position of the first grid vertex, heights are added to its y |
| `resolution` | `[nx, nz]` | required | vertex count along X and Z |
| `cell_size` | number | required | map units between grid vertices |
| `heights` | base64 string | required | little endian 32-bit floats, one per vertex |
| `layers` | array | required | up to four texture layers |
| `splat` | base64 string | empty, omitted | four weight bytes per vertex, one per layer |
| `holes` | base64 string | empty, omitted | one byte per cell, non-zero cells are holes |
| `chunk_cells` | integer | `32` | cells per chunk side for rendering and collision |

Vertex `(i, j)` is at `origin + (i * cell_size, heights[j * nx + i], j * cell_size)`, so X runs along a row and Z along
the rows. `splat` uses the same vertex order, and the four weights are normalized when read, with an empty or all zero
entry meaning the first layer. `holes` is ordered the same way over the `(nx - 1) * (nz - 1)` cells.

Each layer:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `material` | string | required | texture name |
| `tile` | number | `256` | map units covered by one texture repeat |
| `detile` | number | `0` | 0 to 1, how strongly the repeat is broken up |
| `detile_sharpen` | number | `0.5` | 0 to 1, how crisp the de-tiled result stays |

Terrains stay axis aligned. A prefab instance moves a terrain but does not rotate it.

## Scatter

A scatter set stores many model instances (trees, rocks, grass) painted onto surfaces as a single node.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `name` | string | required | |
| `kind` | string | `props` | `props` for scene instances that keep scripts and collision, `foliage` for MultiMesh instances without collision |
| `targets` | array of node ids | empty, omitted | surfaces the set is painted on |
| `items` | array | required | the palette, see below |
| `collision` | string | `convex` | `none`, `convex` or `trimesh` |
| `cast_shadows` | bool | `true` | |
| `visibility_range` | number | `0` | map units beyond which instances are hidden, 0 shows them at any distance |
| `chunk_size` | number | `0` | grid cell in map units the instances are split into so Godot culls each cell, 0 keeps one MultiMesh (new sets use 2048) |
| `static_props_multimesh` | bool | `false` | draw prop scenes that carry scripts as MultiMesh too, dropping their scripts |
| `material` | string | none, omitted | material drawn instead of the models' own, for the whole set |
| `instances` | array | empty | the placed instances |

Except for `targets` and `material` every field is always written.

Each palette item:

| Key | Type | Default | Meaning |
|---|---|---|---|
| `source` | string | required | `res://` model or scene (`.bbmodel`, `.glb`, `.gltf`, `.tscn`) |
| `weight` | number | `1` | relative chance of being picked |
| `scale` | `[min, max]` | `[0.8, 1.2]` | random uniform scale range |
| `spacing` | number | `48` | minimum distance in map units to every other instance |
| `align` | number | `0` | 0 keeps instances upright, 1 aligns them to the surface normal |
| `random_yaw` | bool | `true` | |
| `tilt` | number | `0` | largest random lean in degrees |
| `sink` | number | `0` | map units pushed into the surface |
| `material` | string | none, omitted | material override for this item, wins over the set's |

Instances are stored as one flat array each, `[item, x, y, z, pitch, yaw, roll, scale]`, rather than as objects, so a set
with thousands of trees stays small and diffs line by line. `item` indexes into `items`, the position is in map units,
the angles are degrees in the same YXZ order as entity angles. On save the position is rounded to 0.01, the angles to 0.1
and the scale to 0.001. Reading needs at least eight numbers and ignores any extra.

## Instance

A reference to another `.gtm` file placed with a transform, like a Hammer `func_instance`. Prefabs are built from these.

| Key | Type | Default | Meaning |
|---|---|---|---|
| `path` | string | required | path relative to the referencing map, or a `res://` path |
| `origin` | `[x, y, z]` | required | |
| `angles` | `[pitch, yaw, roll]` | required | degrees, YXZ like entity angles |
| `fixup` | string | `""`, omitted | name prefix for the instance's contents |

When building, the importer reads the referenced map and places the children of each of its layers that is not omitted
under the instance's group, transformed by `origin` and `angles`. The referenced map's worldspawn properties are not
used. Brush and mesh texture projections are transformed along with the geometry, so textures stay locked to it.
Instances may nest up to eight levels deep.

A prefab placed twice would otherwise produce two entities with the same targetname, and a button in one copy would
open the door in both. `fixup` avoids that: with a fixup of `p1`, the `targetname` and `target` properties and the
`target` of every output inside the instance get `p1-` prepended, so `door` becomes `p1-door` and the copy's own wiring
still connects. Empty values and values starting with `!` (`!player`, `!activator`, `!self`), `@` (groups) or `/` (node
paths) are left alone, a `*` wildcard is prefixed like any name. Nested instances add their prefixes outside in, so a
fixup of `b` inside `a` gives `a-b-door`, and an inner instance without a fixup uses the outer one.

*Explode Instance* in the editor gives the same names as a Godot build. It skips the prefab's omitted layers, prefixes
the entities it creates and folds its own fixup into any nested instance, so exploding `a` and then `b` also ends at
`a-b-door`.

## Coordinates and units

Maps are stored in Godot's axis convention: Y is up, X right and Z towards the viewer, right handed. Positions are in map
units. The Godot build divides by the `FuncGodotMapSettings` inverse scale factor, 32 by default, so 32 units are one
meter. Internally the importer rotates everything into FuncGodot's id Tech axes (`id = (z, x, y)`); since that is a pure
rotation, face windings and UV axes stay valid, but nothing in the file uses id axes.

| Quantity | Unit |
|---|---|
| positions, sizes, heights, spacing, cell size | map units |
| entity, instance and scatter angles | degrees, pitch X, yaw Y, roll Z, YXZ order |
| camera bookmark yaw and pitch | radians |
| UV offset | texels |
| UV scale | map units per texel |
| mesh `uvs` | texture widths |

## Layout on disk

The JSON is written by a small custom printer (`crates/gt_doc/src/json_fmt.rs`) so that diffs of a map stay readable:

* two space indentation and a trailing newline
* keys in the order of the definitions above, and property objects sorted by key
* an array or object whose compact form fits in 100 characters is written on one line, unless it has non-empty
  `children`
* every face is written on one line whatever its length
* a `vertices` array with up to 64 entries is written on one line

Moving one brush therefore changes the lines of that brush and nothing else. Any valid JSON with the same content loads
the same, the layout only matters for version control.

## Clipboard

Copy and paste use the same node shape inside a different wrapper:

```json
{
  "format": "godottrench-clipboard",
  "version": 1,
  "nodes": [ ... ]
}
```

`nodes` holds the copied subtrees, each with all of its descendants. Their parents are not included. Pasting checks
`format` only, inserts every node under the paste target with fresh ids and keeps `hidden` and `locked`. A copied layer
is not pasted as a layer, its children are pasted instead. The live link to Godot sends single nodes in the same shape as
well, without `children`.

## Example

A trimmed excerpt of `godot/demo/maps/scripted_scene.gtm`, with the floor brush, an explosive barrel and the sequence
that drives the scene. The file's other nodes are left out.

```json
{
  "format": "godottrench-map",
  "version": 1,
  "properties": {
    "ambient_color": "60 66 82",
    "classname": "worldspawn",
    "message": "Scripted Scene",
    "sky_top_color": "40 54 92",
    "sun_angles": "-40 -55",
    "sun_energy": "0.6"
  },
  "layers": [
    {
      "id": 1,
      "type": "layer",
      "name": "Default",
      "color": "#7e67e6ff",
      "omit_from_export": false,
      "children": [
        {
          "id": 2,
          "type": "brush",
          "vertices": [[320.0,0.0,-320.0],[320.0,0.0,320.0],[320.0,-16.0,320.0],[320.0,-16.0,-320.0],[-320.0,0.0,320.0],[-320.0,0.0,-320.0],[-320.0,-16.0,-320.0],[-320.0,-16.0,320.0]],
          "faces": [
            {"indices":[0,1,2,3],"material":"base/floor","uv":{"u_axis":[0.0,0.0,-1.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[4,5,6,7],"material":"base/floor","uv":{"u_axis":[0.0,0.0,1.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[1,0,5,4],"material":"base/floor","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,0.0,1.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[3,2,7,6],"material":"base/floor","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,0.0,1.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[2,1,4,7],"material":"base/floor","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[6,5,0,3],"material":"base/floor","uv":{"u_axis":[-1.0,0.0,0.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}}
          ]
        },
        {
          "id": 14,
          "type": "entity",
          "classname": "prop_physics",
          "origin": [0.0,24.0,176.0],
          "angles": [0.0,0.0,0.0],
          "properties": {
            "explosion_damage": "25",
            "explosion_radius": "256",
            "explosive": "1",
            "health": "5",
            "model": "res://demo/scenes/barrel.tscn",
            "size": "16 24 16",
            "targetname": "barrel"
          },
          "outputs": [{"output":"broken","target":"hp_readout","input":"run"}]
        },
        {
          "id": 16,
          "type": "entity",
          "classname": "logic_sequence",
          "origin": [0.0,8.0,296.0],
          "angles": [0.0,0.0,0.0],
          "properties": {"interval":"1.5","steps":"5","targetname":"cutscene"},
          "outputs": [
            {"output":"step_1","target":"hint","input":"show"},
            {"output":"step_2","target":"guide","input":"start"},
            {"output":"step_3","target":"barrel","input":"ignite"},
            {"output":"step_4","target":"lamp","input":"turn_on"},
            {"output":"step_5","target":"exit_door","input":"open"}
          ]
        }
      ]
    }
  ]
}
```

The floor is a 640 by 640 unit slab 16 units thick (20 by 20 meters at the default scale) whose top sits at Y 0. Its
first face, `[0,1,2,3]`, is the +X side: its corners all have x = 320 and run counter-clockwise when seen from +X. The
barrel is a point entity 24 units (0.75 m) above the floor, and when it breaks it calls `run` on `hp_readout`.
`godot/demo/maps/demo.gtm` shows brush entities, a group and a prefab instance with a fixup, and
`godot/demo/maps/terrain.gtm` a terrain with a displacement.
