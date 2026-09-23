# The .gtm map format

A `.gtm` file is a GodotTrench map: worldspawn properties, some editor state and a tree of nodes (layers, groups,
entities, brushes, meshes, terrains, scatter sets and prefab instances). The editor saves it as a compressed binary
container of independent chunks. Maps from older editors are one UTF-8 JSON document instead, and both kinds load
everywhere a map is read. The two hold the same data, a tree of objects, arrays, strings, numbers and booleans, which
this page describes as the JSON that `godottrench --dump` prints.

The editor reads and writes maps through `crates/gt_doc/src/format.rs`, with the container in
`crates/gt_doc/src/binary.rs` and its value encoding in `crates/gt_doc/src/variant.rs`. The Godot addon reads the file
with `godot/addons/func_godot/src/godottrench/gtm_file.gd` and builds it with `gtm_parser.gd` in the same folder.
Everything below is taken from those readers and the serde definitions they use.

## Why a chunked binary file

A JSON map is fragile and big. One damaged byte makes the whole document unreadable, and the showcase map
`withered_city.gtm` took 6.5 MB as JSON. The binary container compresses each chunk with zstd, which brings that map
to about 410 KB, and it splits the nodes into chunks that are checked and decompressed on their own. A damaged or
missing piece of the file then costs only the nodes stored in it, and everything else still opens. Chunks also leave
room for data a future editor adds: a reader skips chunk types it does not know, and the editor writes them back
unchanged.

## Container layout

All numbers in the container are little endian and fixed width. A file starts with a 12 byte header:

| Offset | Size | Content |
|---|---|---|
| 0 | 8 | magic bytes `89 47 54 4D 0D 0A 1A 0A`, that is `\x89GTM\r\n\x1a\n` |
| 8 | 4 | container version, u32, currently 1 |

The magic follows PNG. Its first byte is not ASCII, so tools do not take the file for text, and the `\r\n` and `\n`
pair change when a tool converts line endings anyway, so a file mangled that way is reported as such instead of being
read as garbage. A reader refuses a container version newer than its own.

Chunks follow the header back to back until the end of the file. Each chunk is a 24 byte header and its payload:

| Offset | Size | Field |
|---|---|---|
| 0 | 4 | sync marker `89 47 54 43`, that is `\x89GTC` |
| 4 | 4 | tag, four ASCII characters such as `NODE` |
| 8 | 4 | flags, u32. Bit 0 means the payload is a zstd frame, the other bits are written as 0 |
| 12 | 4 | raw size, u32, the payload size after decompression |
| 16 | 4 | stored size, u32, how many payload bytes follow the header |
| 20 | 4 | header check, u32: the tag read as a u32, XOR flags, XOR raw size, XOR stored size, XOR `0x47544D43` |

A compressed payload is exactly one zstd frame written with its content checksum, so decompressing it also verifies
it. Storing the raw size lets Godot decompress a chunk with `PackedByteArray.decompress(raw_size,
FileAccess.COMPRESSION_ZSTD)`. Readers treat a raw size above 1 GiB as a damaged header.

The editor writes one `HEAD` chunk, then `NODE` chunks, then any chunks it kept from the file it loaded, and an `END`
chunk last.

### HEAD

The top level of the map without its layers: a dictionary with `format`, `version`, `properties` and `editor`, see
[Top level object](#top-level-object).

### NODE

A batch of whole nodes, as a dictionary with two keys:

| Key | Type | Meaning |
|---|---|---|
| `parents` | int64 array | the id of each node's parent, 0 for the layers at the top level |
| `nodes` | array of node objects | the nodes, each without its `children` |

Nodes are written in tree order, every parent before its children, so a parent is always in the same chunk or an
earlier one. The editor starts a new chunk once a batch passes 64 KiB before compression. A node is never split, so a
big terrain or scatter set can make a chunk of its own. Reading a file rebuilds the tree by appending each node to the
`children` of the node its parent id names.

### END

A dictionary that lets a reader tell a complete file from a cut off one:

| Key | Type | Meaning |
|---|---|---|
| `version` | integer | the map version, the same as in HEAD, so the nodes still load when HEAD is damaged |
| `nodes` | integer | how many nodes the NODE chunks hold |
| `chunks` | integer | how many chunks come before END |
| `content` | integer | content id, see below |

The content id is a 64 bit FNV-1a hash of the raw HEAD and NODE payloads in file order, cut to its low 52 bits so that
it survives as a JSON number. Godot keeps the id with a scene it built from the file, and the editor sends the id of
its map when a live session starts, so an unchanged map is not rebuilt. See [Live link](godot/live-link.md).

### Other chunks

Readers skip a chunk whose tag they do not know. The editor keeps such chunks as they are stored, flags, sizes and
payload, and writes them back after its NODE chunks when it saves, so an older editor does not destroy what a newer
one added. Their position between other chunks is not kept, and an older editor may have changed the nodes around
them, so a new chunk type must stay valid when nodes it refers to are gone.

## Value encoding

Payloads use Godot's binary Variant serialization, the format `var_to_bytes` writes, limited to the types below. The
Godot addon decodes each chunk with one `bytes_to_var` call, which runs in C++ and returns the same dictionaries and
arrays that `JSON.parse_string` returns for a JSON map. Reading the showcase maps takes about a fifth less time than
parsing their JSON did, and a decoder written in GDScript would have been several times slower than either.

Every value starts with a u32 header. Its low byte is the type, and bit 16 marks 64 bit integers and floats.

| Type | Header | Body |
|---|---|---|
| null | 0 | nothing |
| bool | 1 | u32, 0 or 1 |
| integer | 2 | i32, or i64 with bit 16 set |
| float | 3 | f32, or f64 with bit 16 set. The editor always writes f64 |
| string | 4 | u32 byte length, the UTF-8 bytes, zero bytes up to a multiple of 4 |
| dictionary | 27 | u32 count, then each key (a string) followed by its value |
| array | 28 | u32 count, then the values |
| bytes | 29 | u32 length, the bytes, zero bytes up to a multiple of 4 |
| int32 array | 30 | u32 count, an i32 each |
| int64 array | 31 | u32 count, an i64 each |
| float32 array | 32 | u32 count, an f32 each |
| float64 array | 33 | u32 count, an f64 each |

Readers ignore bit 31 of a count, which Godot uses for shared containers. A JSON integer is written as an integer and
a JSON float as a float, so converting a map to the binary container and back gives exactly the JSON it came from.
Three storage choices keep the files small, and a reader undoes them to get the JSON back:

* An array of at least 32 numbers is written as a packed array: an int32 array when all are integers that fit, a
  float32 array when all are floats that a 32 bit float holds exactly, and a float64 array otherwise. Shorter arrays,
  like the three coordinates of a vertex, stay plain arrays, because Godot decodes many tiny packed arrays slower than
  tiny arrays and zstd compresses both to the same size. A reader has to accept either form.
* The terrain keys `heights`, `splat` and `holes`, base64 text in JSON, are written as bytes: the decoded data.
* The `instances` of a scatter set are written as one int32 array when every instance is eight floats that are whole
  multiples of the steps the editor rounds them to. The array holds every item index, then every x, and so on through
  the eight fields, each field as an integer count of its step: 1 for the item, 100 per unit for the position, 10 per
  degree for the angles and 1000 for the scale. Dividing gives back the exact floats, for example `1234 / 100.0` is
  `12.34`. As 64 bit floats these values hardly compress, as integers they come out about a third smaller. A set whose
  values do not all fit is written as plain arrays.

## Damaged files

A reader walks the chunks from byte 12. Where it finds no valid chunk header, meaning the sync marker, the header
check, the raw size limit and a printable tag, it searches forward for the next sync marker and goes on from there.
A chunk whose payload does not decompress, fails its checksum, has the wrong size or does not decode is dropped. What
that loses depends on the chunk:

* A damaged NODE chunk loses the nodes stored in it and nothing else.
* A node whose parent was lost is not dropped. It moves, with everything below it, into a layer named `Recovered` at
  the end of the map. The layer gets a fresh id when the map loads. Nodes nested more than 60 levels deep, which only
  a crafted file has, are moved there as well, so reading a file cannot run out of stack.
* A damaged HEAD chunk loses the worldspawn properties and the editor state. The map version is then taken from END,
  and a file with neither is refused.
* A file without an END chunk was cut off. Everything up to the last complete chunk loads. When END is there, its node
  count tells how many nodes are missing.

The editor opens what it could read, lists what was lost in the status bar and marks the map as unsaved. Saving writes
the recovered map, and like every save it first copies the old file to `<map>.gtm.bak`, so the damaged original is
still there to restore from version control or a backup. Prefabs that load damaged are reported in the status bar as
well. Godot builds what it could read and prints a warning per problem, prefixed with `[GTM]` and the file path.
`godottrench --dump` prints what could be read and writes the problems to stderr.

Saving writes `<name>.gtm.tmp` first and renames it over the map, so a crash never leaves a truncated file.

## Versions and compatibility

A map has three version markers. The container version in the file header changes only when the chunk layout changes
in a way a reader cannot skip. The map `format` and `version` in HEAD describe the data inside, as they always have.
Every map starts with

```json
{
  "format": "godottrench-map",
  "version": 1,
```

| Situation | Editor | Godot importer |
| --- | --- | --- |
| Other `format` | Refuses the file | Build error |
| Newer `version` | Refuses the file rather than drop data | Build error asking for an addon update |
| Missing `version` | Treated as the oldest format | Same |
| Unknown keys | Ignored, and lost on the next save | Ignored |
| Unknown chunk | Kept and written back on save | Skipped |
| Bad face index, wrong array size | Refuses the map, naming the node and reason | Skips that brush, builds a bad displacement flat |

Keys the editor does not know inside HEAD or a node are dropped on load, because it holds the map as typed nodes. New
node data that an older editor must not lose therefore comes with a map version bump, which older editors refuse, or
goes into a chunk of its own, see [Other chunks](#other-chunks). `godottrench --to-json` and `--to-gtm` convert without
loading the map into the editor, so they keep unknown keys. JSON has no place for chunks, so `--to-json` and saving a map
as `.json` leave out chunks the editor does not know.

## JSON maps

Maps saved before the binary container are JSON text, and the JSON layout is also what the clipboard, the live link
and the MCP tools use for nodes. Readers tell the two apart by the first bytes: a file starting with `\x89GTM` is a
container, anything else is read as JSON, with or without a UTF-8 byte order mark. Old maps load as they are and are
saved in the binary format from then on.

For hand editing, `godottrench --to-json map.gtm map.json` writes the readable layout described in
[JSON layout](#json-layout), the editor opens `.json` files directly, and *Save As* with a `.json` name writes JSON
again. `godottrench --to-gtm map.json map.gtm` turns it back into a binary map with the same content, after checking
that the editor could open it. To see map changes as JSON in `git diff`, set up the text conversion described in
[Development](development.md#map-files-in-git).

## Top level

| Key | Type | Written | Meaning |
| --- | --- | --- | --- |
| `format` | string | always | `"godottrench-map"` |
| `version` | integer | always | Format version |
| `properties` | object of string to string | when not empty | Worldspawn keys, sorted |
| `editor` | object | when not all default | Editor state, never exported |
| `layers` | array of nodes | always | Layer nodes, in order |

Worldspawn values are always strings, like in a `.map` file. The environment keys (`sun_angles`, `sun_color`,
`sun_energy`, `ambient_color`, `sky_top_color`, `sky_horizon_color`, `sky_ground_color`, `fog_color`, `fog_density`,
`ambient_energy`, `sky_energy`, `glow_intensity`, `ssr`, and `environment = 0` to switch it off) are read by
`GodotTrenchEnvironment`.

Only `layer` nodes are taken from `layers`. A file with no layers gets a `Default` layer on load.

### Editor data

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `cameras` | object, `"1"` to `"9"` | empty | Bookmarks, `{"position": [x, y, z], "yaw": r, "pitch": r}` in radians |
| `cordon` | `{"min", "max"}` | none | Cordon box in map units |
| `cordon_enabled` | bool | `false` | Hide objects outside the cordon and leave them out of cordoned exports |

## Nodes

Every node has the same outer shape, with keys in this order:

| Key | Type | Written | Meaning |
| --- | --- | --- | --- |
| `id` | integer | always | Unique in the map |
| `type` | string | always | `layer`, `group`, `entity`, `brush`, `mesh`, `terrain`, `scatter` or `instance` |
| type fields | | | See the sections below |
| `hidden` | bool | when `true` | Hidden in the editor |
| `locked` | bool | when `true` | Cannot be selected or edited |
| `children` | array of nodes | when not empty | Child nodes, in order |

`hidden` and `locked` are editor state only. Godot still builds a hidden brush, use a layer with `omit_from_export` to
leave content out.

### Ids

Ids are positive integers that stay the same across saves. The live link sends edits by id, scatter sets name their
targets by id, and Godot tags every generated node with its id in `_gt_id` metadata. An id of `0` or a duplicate (only
possible by hand editing) is replaced with a fresh one on load.

Nodes inside prefab instances may reuse ids of the including map, so the importer offsets them by one million per
level of nesting.

### Where nodes go

| Node | Parent |
| --- | --- |
| Layers | Top level only |
| Groups | Layers or groups |
| Brushes, meshes, terrains, scatter sets, point entities, instances | Layers or groups |
| Brushes and meshes of a brush entity | That entity |

An entity with children is a brush entity. Brushes and meshes outside an entity belong to worldspawn.

## Layer

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `name` | string | required | |
| `color` | string | required | `#rrggbbaa` (`#rrggbb` accepted when reading) |
| `omit_from_export` | bool | `false` | Always written. The importer skips the whole layer |

## Group

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `name` | string | required | |
| `link_id` | integer | omitted | Groups sharing a link id are linked copies |
| `transform` | 16 numbers | identity, omitted | This copy's placement relative to the others, 4x4 column-major |

Group contents are stored in world space, already placed. `transform` is only used to mirror an edit in one linked
copy into the others. The importer ignores `link_id` and `transform`.

## Entity

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `classname` | string | required | Entity class from the game config |
| `origin` | `[x, y, z]` | `[0, 0, 0]` | Always written. Position of a point entity |
| `angles` | `[pitch, yaw, roll]` | `[0, 0, 0]` | Always written. Degrees, YXZ order like `rotation_degrees` |
| `properties` | object of string to string | omitted | Entity keys, sorted |
| `outputs` | array | omitted | I/O connections |

Property values are strings whatever the FGD type, for example `"travel": "0 168 0"`. An `angles`, `angle` or `mangle`
property wins over the node's `angles`. Brush entities still write `origin` and `angles`, the importer ignores them.

### Outputs

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `output` | string | required | Signal on this entity |
| `target` | string | required | See [Targets and parameters](gameplay/targets.md) |
| `input` | string | required | Method to call on the target |
| `parameter` | string | `""`, omitted | Value passed to the input |
| `delay` | number | `0`, omitted | Seconds before the call |
| `times` | integer | `-1`, omitted | How many times it may fire, `-1` for always |

## Brush

A convex solid stored as its exact vertices and faces that index into them.

| Key | Type | Meaning |
| --- | --- | --- |
| `vertices` | array of `[x, y, z]` | Corners in map units |
| `faces` | array of faces | One polygon per face |

Quake `.map` files store three points per plane, and every tool re-intersects the planes, which drifts. Storing the
corners makes vertex edits round trip exactly, and Godot builds exactly what the editor shows. Face planes are
recomputed on load. Clipped corners within 0.00001 of a whole number are snapped to it.

Each face:

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `indices` | array of integers | required | Vertex indices, counter-clockwise from outside |
| `material` | string | `""` | Always written. Texture name, for example `base/wall` |
| `uv` | object | paraxial | Always written. Texture projection |
| `props` | object of string to string | omitted | Free-form face attributes |
| `disp` | object | omitted | Displacement |
| `colors` | array of `[r, g, b, a]` | omitted | Vertex paint, one per index |

Faces need at least three indices. The importer skips a brush with fewer than four vertices or faces.

Face `props` `blend_material`, `blend_detile`, `blend_uv_scale` and `blend_detile_sharpen` set up material blending: the
vertex color alpha blends from `material` towards `blend_material`.

### Texture projection

`uv` is a Valve 220 style projection with explicit axes:

| Key | Type | Meaning |
| --- | --- | --- |
| `u_axis`, `v_axis` | `[x, y, z]` | World direction of the texture's u and v |
| `offset` | `[u, v]` | Shift in texels |
| `scale` | `[u, v]` | Map units per texel |
| `rotation` | number | Informational only, optional, the axes already contain it |

A point `p` lands on texel `dot(p, u_axis) / scale.x + offset.x`, the same for v. The texel is divided by the texture
size for the final UV: the material's `texture_size` when set (see [Materials and lighting](godot/materials.md)),
otherwise the image's pixel size.

When `uv` is missing it defaults to `u_axis = [1, 0, 0]`, `v_axis = [0, 0, 1]`. When present, every field but
`rotation` is required.

### Displacement

A quad face can carry a Hammer style displacement, a grid pushed along the face normal.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `power` | integer | required | 1 to 4, the grid has `2^power + 1` vertices per side |
| `heights` | array of numbers | required | Offset along the normal per grid vertex |
| `alphas` | array of numbers | omitted | Blend weight per grid vertex, 0 is the face material |

Both arrays are row major with `(2^power + 1)^2` entries. Rows run from the first corner towards the fourth, columns
from the first towards the second.

## Mesh

Editable polygon meshes. They may be concave, open or non-planar.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `vertices` | array of `[x, y, z]` | required | |
| `faces` | array of faces | required | Same keys as brush faces, plus `uvs` |
| `smooth_angle` | number | `0`, omitted | Faces meeting below this angle share normals, 0 is flat |
| `decal` | bool | `false`, omitted | A decal sheet: alpha cut at 0.5, both sides drawn |

A mesh face's `uvs` is an array of `[u, v]`, one per index, in texture space (1.0 is one texture width). It is used
instead of `uv` when present.

## Terrain

A heightmap on a regular grid. In JSON the bulk data is base64, so a large terrain stays one line per array instead of
thousands of numbers, and the binary container stores the same bytes raw.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `origin` | `[x, y, z]` | required | World position of the first vertex |
| `resolution` | `[nx, nz]` | required | Vertex count along X and Z |
| `cell_size` | number | required | Map units between vertices |
| `heights` | base64 | required | Little endian 32-bit floats, one per vertex |
| `layers` | array | required | Up to four texture layers |
| `splat` | base64 | omitted | Four weight bytes per vertex, one per layer |
| `holes` | base64 | omitted | One byte per cell, non-zero is a hole |
| `chunk_cells` | integer | `32` | Cells per chunk side |

Vertex `(i, j)` sits at `origin + (i * cell_size, heights[j * nx + i], j * cell_size)`. `splat` uses the same order,
weights are normalized on read and an all zero entry means layer 0. `holes` covers the `(nx - 1) * (nz - 1)` cells in
the same order.

Each layer:

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `material` | string | required | |
| `tile` | number | `256` | Map units per texture repeat |
| `detile` | number | `0` | 0 to 1, how strongly the repeat is broken up |
| `detile_sharpen` | number | `0.5` | 0 to 1, how crisp the de-tiled result stays |

Terrains stay axis aligned. A prefab instance moves a terrain but does not rotate it.

## Scatter

Many model instances painted onto surfaces, stored as one node. Every field except `targets` and `material` is always
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
| `chunk_size` | number | `0` | Split into cells this size for culling. New sets use 2048 |
| `static_props_multimesh` | bool | `false` | Draw scripted prop scenes as MultiMesh too, dropping their scripts |
| `material` | string | omitted | Material override for the whole set |
| `instances` | array | empty | The placed instances |

Duplicating a set together with a surface it targets makes the copy target the copied surface.

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

Each instance is a flat array `[item, x, y, z, pitch, yaw, roll, scale]`, so thousands of trees stay small and the
JSON diffs line by line. The binary container stores them as integer columns, see [Value encoding](#value-encoding).
`item` indexes `items`, angles are degrees in YXZ order. On save the position is rounded to 0.01, angles to 0.1 and
scale to 0.001.

## Instance

A reference to another `.gtm` placed with a transform, like Hammer's `func_instance`. Prefabs are built from these.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `path` | string | required | Relative to the referencing map, or `res://` |
| `origin` | `[x, y, z]` | required | |
| `angles` | `[pitch, yaw, roll]` | required | Degrees, YXZ |
| `fixup` | string | `""`, omitted | Name prefix for the instance's contents |

The importer places the children of every non omitted layer of the referenced map, transformed by `origin` and
`angles`. Its worldspawn is not used. Texture projections move with the geometry. Instances nest up to eight levels.

**Fixup.** With a fixup of `p1`, `door` becomes `p1-door` in `targetname`, `target`, `destination`, `call_target`, any
property declared as `target_source` or `target_destination`, and every output `target`. Values starting with `!`, `@`
or `/` are left alone. Nested fixups add up outside in, `b` inside `a` gives `a-b-door`. *Explode Instance* in the
editor produces the same names.

## Coordinates and units

Godot's axes: Y up, X right, Z towards the viewer, right handed. The build divides by the inverse scale factor, 32 by
default, so 32 units are one meter. Nothing in the file uses id Tech axes.

| Quantity | Unit |
| --- | --- |
| Positions, sizes, heights, spacing, cell size | Map units |
| Entity, instance and scatter angles | Degrees, pitch X, yaw Y, roll Z, YXZ order |
| Camera bookmark yaw and pitch | Radians |
| UV offset | Texels |
| UV scale | Map units per texel |
| Mesh `uvs` | Texture widths |

## JSON layout

The JSON of `--dump`, `--to-json`, a `.json` save and older `.gtm` files comes from a custom printer
(`crates/gt_doc/src/json_fmt.rs`) that keeps diffs readable: two space indentation, keys in the order
above, and short arrays and objects on one line. Every face is one line, and so is a `vertices` array of up to 64
entries. Moving one brush changes that brush's lines and nothing else.

Any valid JSON with the same content loads the same, the layout only matters for version control.

## Clipboard

Copy and paste use the same node shape in a different wrapper:

```json
{
  "format": "godottrench-clipboard",
  "version": 1,
  "nodes": [ ... ]
}
```

`nodes` holds the copied subtrees without their parents. Pasting runs the same checks as loading, gives every node a
fresh id and keeps `hidden` and `locked`. A copied layer pastes its children. The live link sends single nodes in the
same shape, without `children`.

## Example

A trimmed excerpt of `godot/demo/maps/scripted_scene.gtm` as `godottrench --dump` prints it, with the floor brush, an
explosive barrel and the sequence that drives the scene. The file's other nodes are left out.

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
