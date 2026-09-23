# Container layout

A binary `.gtm` is a 12 byte file header followed by chunks, back to back until the end of the file. All numbers are
little endian and fixed width.

| Offset | Size | Meaning |
| --- | --- | --- |
| 0 | 8 | Magic `89 47 54 4D 0D 0A 1A 0A`, that is `\x89GTM\r\n\x1a\n` |
| 8 | 4 | Container version, u32, currently 1 |

A reader refuses a container version newer than its own, and reports a file whose `\r\n` was changed by a line ending
conversion as mangled instead of reading it.

## Chunks

Each chunk is a 24 byte header followed by its payload.

| Offset | Size | Meaning |
| --- | --- | --- |
| 0 | 4 | Sync marker `89 47 54 43`, that is `\x89GTC` |
| 4 | 4 | Tag, four printable ASCII characters such as `NODE` or `END ` |
| 8 | 4 | Flags, u32. Bit 0 means the payload is a zstd frame |
| 12 | 4 | Raw size, u32, the payload size after decompression |
| 16 | 4 | Stored size, u32, the payload bytes that follow the header |
| 20 | 4 | Header check, u32: tag read as a u32, XOR flags, XOR raw size, XOR stored size, XOR `0x47544D43` |

A compressed payload is one zstd frame with a content checksum. A raw size above 1 GiB marks a damaged header. The
editor writes one `HEAD` chunk, then the `NODE` chunks, then any unknown chunks it kept from the file it loaded, and an
`END` chunk last.

### HEAD

The [top level object](README.md#top-level) without `layers`.

### NODE

A batch of whole nodes:

| Key | Type | Meaning |
| --- | --- | --- |
| `parents` | int64 array | The parent id of each node, 0 for a layer |
| `nodes` | array of node objects | The nodes, each without `children` |

Nodes are written in tree order, so a parent always comes before its children, in the same chunk or an earlier one. The
editor starts a new chunk once a batch passes 64 KiB before compression, and never splits a node. A reader rebuilds the
tree by appending each node to the `children` of the node its parent id names.

### END

| Key | Type | Meaning |
| --- | --- | --- |
| `version` | integer | The map version, so the nodes still load when `HEAD` is damaged |
| `nodes` | integer | How many nodes the `NODE` chunks hold |
| `chunks` | integer | How many chunks come before `END` |
| `content` | integer | Content id |

The content id is a 64 bit FNV-1a hash of the raw `HEAD` and `NODE` payloads in file order, cut to its low 52 bits so
it survives as a JSON number. Godot stores it with a scene built from the file, and the editor sends it when a
[live session](../godot/live-link.md) starts, so an unchanged map is not rebuilt.

### Other chunks

Readers skip a chunk whose tag they do not know. The editor keeps it as stored and writes it back after its `NODE`
chunks on save. An older editor may have deleted nodes it refers to, so a new chunk type must stay valid without them.

## Value encoding

Payloads use Godot's binary Variant serialization, the format of `var_to_bytes`, limited to the types below, so the
addon decodes each chunk with one `bytes_to_var` call. Every value starts with a u32 header whose low byte is the type.
Bit 16 of the header marks a 64 bit integer or float.

| Type | Header | Body |
| --- | --- | --- |
| null | 0 | Nothing |
| bool | 1 | u32, 0 or 1 |
| integer | 2 | i32, or i64 with bit 16 set |
| float | 3 | f32, or f64 with bit 16 set. The editor always writes f64 |
| string | 4 | u32 byte length, UTF-8 bytes, zero padding to a multiple of 4 |
| dictionary | 27 | u32 count, then each key (a string) followed by its value |
| array | 28 | u32 count, then the values |
| bytes | 29 | u32 length, the bytes, zero padding to a multiple of 4 |
| int32 array | 30 | u32 count, an i32 each |
| int64 array | 31 | u32 count, an i64 each |
| float32 array | 32 | u32 count, an f32 each |
| float64 array | 33 | u32 count, an f64 each |

Readers ignore bit 31 of a container or array count, which Godot sets for shared containers. JSON integers are written
as integers and JSON floats as floats, so converting to binary and back gives the same JSON. Three kinds of value are
stored in a smaller form, and a reader has to accept both forms:

| JSON value | Stored as | When |
| --- | --- | --- |
| Array of 32 or more numbers | int32 array when all are integers that fit, float32 array when all are floats a 32 bit float holds exactly, float64 array for other floats | Always |
| Terrain `heights`, `splat` and `holes` (base64 text) | bytes, the decoded data | The base64 is in canonical form |
| Scatter `instances` | One int32 array of columns | Every value is a float and a whole multiple of its step |

The scatter columns hold every item index, then every x, and so on through the eight fields. Each value is an integer
count of its step: 1 for the item, 100 per unit for the position, 10 per degree for the angles and 1000 for the scale.
Dividing gives back the exact float, for example `1234 / 100.0` is `12.34`.

> **Note:** `gtm_file.gd` turns scatter columns back into instance arrays but leaves terrain bytes as `PackedByteArray`
> and long number arrays as packed arrays. GDScript that reads a map has to accept those next to plain arrays.
