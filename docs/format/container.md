# Container layout

A binary `.gtm` is a 12 byte file header followed by chunks, back to back until the end of the file. All numbers are
little endian and fixed width.

| Offset | Size | Meaning |
| --- | --- | --- |
| 0 | 8 | Magic `89 47 54 4D 0D 0A 1A 0A`, that is `\x89GTM\r\n\x1a\n` |
| 8 | 4 | Container version, u32, currently 1 |

A reader refuses a container version newer than its own. The `\r\n` in the magic is there to catch a file that went
through a line ending conversion, for example in git, and a reader reports such a file as mangled instead of reading
it.

## Chunks

Each chunk is a 24 byte header followed by its payload. The sync marker and the header check let a reader tell a real
chunk header from random bytes, which is how it finds the next good chunk in a [damaged file](recovery.md).

| Offset | Size | Meaning |
| --- | --- | --- |
| 0 | 4 | Sync marker `89 47 54 43`, that is `\x89GTC` |
| 4 | 4 | Tag, four printable ASCII characters such as `NODE` or `END ` |
| 8 | 4 | Flags, u32. Bit 0 means the payload is a zstd frame |
| 12 | 4 | Raw size, u32, the payload size after decompression |
| 16 | 4 | Stored size, u32, the number of payload bytes that follow the header |
| 20 | 4 | Header check, u32: the tag read as a u32, XOR flags, XOR raw size, XOR stored size, XOR `0x47544D43` |

A compressed payload is one zstd frame with a content checksum. A raw size above 1 GiB marks a damaged header. The
editor writes the chunks in this order: one `HEAD` chunk, then the `NODE` chunks, then any unknown chunks it kept from
the file it loaded, an `LMAP` chunk when the map has baked lighting, and an `END` chunk last.

### HEAD

The [top level object](README.md#top-level) without `layers`.

### NODE

A batch of whole nodes:

| Key | Type | Meaning |
| --- | --- | --- |
| `parents` | int64 array | The parent id of each node, 0 for a layer |
| `nodes` | array of node objects | The nodes, each without `children` |

Nodes are written in tree order, so a parent always comes before its children, in the same chunk or an earlier one. The
editor starts a new chunk once a batch passes 64 KiB before compression, and never splits a node across chunks, so a
damaged chunk only takes its own nodes with it. A reader rebuilds the tree by appending each node to the `children` of
the node its parent id names.

### END

The `END` chunk lets a reader check that nothing is missing:

| Key | Type | Meaning |
| --- | --- | --- |
| `version` | integer | The map version, so the nodes still load when `HEAD` is damaged |
| `nodes` | integer | How many nodes the `NODE` chunks hold |
| `chunks` | integer | How many chunks come before `END` |
| `content` | integer | Content id, see below |

The content id is a 64 bit FNV-1a hash of the raw `HEAD` and `NODE` payloads in file order, cut to its low 52 bits so
it survives as a JSON number. Godot keeps it with the built scene, so a [live session](../godot/live-link.md) can tell
that a map has not changed and skip rebuilding it.

### LMAP

The [baked lighting](../editor/light-baking.md), written only when the map has a bake. It is one dictionary with 18
keys, stored as its own chunk so the rest of the map loads fast and an older editor keeps it untouched. The editor
compresses it with a faster zstd level than the other chunks, since it is large. Positions and distances are in map units.

| Key | Type | Meaning |
| --- | --- | --- |
| `version` | integer | Layout version, currently 1. A reader drops a bake with a newer one |
| `width`, `height` | integer | Light map size in texels |
| `scene` | integer | Fingerprint of the geometry, lights and sky the bake was made from, to tell when it is out of date |
| `texel_size` | float | Map units per texel |
| `light` | bytes | Linear light, three little endian half floats (red, green, blue) per texel, row by row |
| `shadow` | bytes | How much of the sun reaches each texel, 0 to 255 |
| `ao` | bytes | Ambient occlusion per texel, 255 where nothing is near |
| `chart_keys` | int64 array | A node id and a face index per chart, the face is 0 for a terrain |
| `chart_rows` | float32 array | Eight floats per chart, two rows `a b c d` that give the chart's 0 to 1 light map coordinate as `a*x + b*y + c*z + d` |
| `nodes` | int64 array | Pairs of a node id and the fingerprint of its geometry when it was baked |
| `stale` | int64 array | Nodes whose geometry changed since, or that are gone. Readers skip their charts |
| `fallback` | float32 array | The average light as red, green and blue, for faces without a chart |
| `probe_points` | float32 array | Light probe positions, three floats each |
| `probe_sh` | float32 array | Nine red, green and blue spherical harmonics coefficients per probe, as Godot stores them |
| `probe_tetrahedra` | int32 array | Four probe indices per tetrahedron |
| `probe_bsp_planes` | float32 array | A plane per BSP node, normal and distance |
| `probe_bsp_children` | int32 array | The child over and under each plane: another node when positive, tetrahedron `t` as `-t - 1`, nothing as the smallest int32 |

The probe keys follow the layout of Godot's `LightmapGIData`, and a reader leaves the probes out when their sizes do
not add up. A chunk that fails to load is dropped with a warning to bake again. In a JSON map the same dictionary sits
under a top level `lightmap` key, with `light`, `shadow` and `ao` as base64 text.

### Other chunks

Readers skip a chunk whose tag they do not know. The editor keeps such a chunk exactly as stored and writes it back
after its `NODE` chunks on save. This is the way to add data that older editors must carry along without
understanding it. Because an older editor may have deleted nodes the chunk refers to, a new chunk type must stay valid
when those nodes are gone.

## Value encoding

Each payload is one value in Godot's binary Variant serialization, the format of `var_to_bytes`, using only the types
below. Every value starts with a u32 header whose low byte is the type. Bit 16 of the header marks a 64 bit integer or
float.

| Type | Header | Body |
| --- | --- | --- |
| null | 0 | Nothing |
| bool | 1 | u32, 0 or 1 |
| integer | 2 | i32, or i64 with bit 16 set |
| float | 3 | f32, or f64 with bit 16 set. The editor always writes f64 |
| string | 4 | u32 byte length, then the UTF-8 bytes, then zero padding to a multiple of 4 |
| dictionary | 27 | u32 count, then each key (a string) followed by its value |
| array | 28 | u32 count, then the values |
| bytes | 29 | u32 length, then the bytes, then zero padding to a multiple of 4 |
| int32 array | 30 | u32 count, then an i32 each |
| int64 array | 31 | u32 count, then an i64 each |
| float32 array | 32 | u32 count, then an f32 each |
| float64 array | 33 | u32 count, then an f64 each |

Readers ignore bit 31 of a container or array count, which Godot sets for shared containers. JSON integers are written
as integers and JSON floats as floats, so converting to binary and back gives the same JSON.

To keep files small, three kinds of value are stored in a more compact form. A reader has to accept both the compact
and the plain form:

| JSON value | Stored as | When |
| --- | --- | --- |
| Array of 32 or more numbers | An int32 array when all are integers that fit. A float32 array when all are floats that a 32 bit float holds exactly. A float64 array for other floats | Always |
| Terrain `heights`, `splat` and `holes` (base64 text) | bytes, the decoded data | When the base64 is in canonical form |
| Scatter `instances` | One int32 array of columns | When every value is a float and a whole multiple of its step |

The scatter columns hold every item index first, then every x, and so on through the eight fields of an instance. Each
value is stored as an integer count of its step: 1 for the item, 100 per unit for the position, 10 per degree for the
angles and 1000 for the scale. Dividing gives back the exact float, for example `1234 / 100.0` is `12.34`.

> **Note:** `gtm_file.gd` turns scatter columns back into instance arrays, but leaves terrain bytes as `PackedByteArray`
> and long number arrays as packed arrays. GDScript that reads a map has to accept those next to plain arrays.
