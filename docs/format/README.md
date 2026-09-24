# The .gtm map format

These pages are for people writing a tool that reads or writes `.gtm` files, such as an importer, a converter or a CI
check. You do not need them to build levels.

A `.gtm` file holds three things: the map's worldspawn properties (map-wide settings such as the sun and sky), some
editor state, and a tree of nodes that holds the actual content. The editor saves it as a binary container of zstd
compressed chunks, which keeps files small and lets a damaged file still open. Maps from older editors are one UTF-8
JSON document instead, and every reader loads both. Both forms hold the same tree, so these pages describe it as the
JSON that `godottrench --dump` prints.

| Page | Read it when you |
| --- | --- |
| [Container layout](container.md) | Parse or write the binary file. It covers the file header, the chunks and how each value is encoded as bytes |
| [Damaged files](recovery.md) | Want to know what still loads from a corrupt file, and where each reader reports what was lost |
| [Nodes](nodes.md) | Walk the tree. It explains the shape every node shares, how ids work, and the layer, group, entity and instance nodes |
| [Geometry](geometry.md) | Read or build brushes and meshes, including their faces, texture projection and displacements |
| [Terrain and scatter](terrain-scatter.md) | Handle heightmap terrains and sets of scattered models |
| [JSON layout](json.md) | Work with the readable form, convert between the two forms, or read what the editor puts on the clipboard |

The reference implementation is
[`format.rs`](https://github.com/Paraxdev/GodotTrench/blob/main/crates/gt_doc/src/format.rs), with the container in
[`binary.rs`](https://github.com/Paraxdev/GodotTrench/blob/main/crates/gt_doc/src/binary.rs) and
[`variant.rs`](https://github.com/Paraxdev/GodotTrench/blob/main/crates/gt_doc/src/variant.rs). The Godot addon reads
files with [`gtm_file.gd`](https://github.com/Paraxdev/godottrench_func/blob/main/src/godottrench/gtm_file.gd) and
builds them with [`gtm_parser.gd`](https://github.com/Paraxdev/godottrench_func/blob/main/src/godottrench/gtm_parser.gd).

## Top level

| Key | Type | Written | Meaning |
| --- | --- | --- | --- |
| `format` | string | always | Always `"godottrench-map"`. It tells a map apart from other JSON |
| `version` | integer | always | Map version, currently 1 |
| `properties` | object of string to string | when not empty | Worldspawn keys, sorted by name |
| `editor` | object | when not all default | Editor view state. It is never exported to Godot |
| `layers` | array of nodes | always | The map's [layer nodes](nodes.md#layer), in order. Everything else hangs below them |

Only `layer` nodes are read from `layers`. A map with no layers gets a `Default` layer when it loads.

Worldspawn values are strings, like in a `.map` file. When Godot builds the map, `GodotTrenchEnvironment` reads these
keys and turns them into a sky, fog and a sun:

| Keys | What they control |
| --- | --- |
| `sun_angles`, `sun_color`, `sun_energy` | The directional sun light: where it points, its color and how bright it is |
| `ambient_color`, `ambient_energy` | Light that reaches surfaces the sun does not, so shadows are not pitch black |
| `sky_top_color`, `sky_horizon_color`, `sky_ground_color`, `sky_energy` | The procedural sky gradient and its brightness |
| `sky_panorama` | A `res://` image that replaces the procedural sky |
| `fog_color`, `fog_density` | Distance fog |
| `glow_intensity`, `ssr` | Bloom on bright surfaces, and screen space reflections when `ssr` is `1` |

Setting `environment` to `sun_only` builds only the sun, for a game that brings its own `WorldEnvironment`, and `0` or
`none` builds nothing, see [environment and sun](../godot/building.md#environment-and-sun). `sky_source` records
which texture the sky faces of an imported map had.

The `editor` object holds view state that only matters inside the editor:

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `cameras` | object, keys `"1"` to `"9"` | empty | Saved camera positions, each `{"position": [x, y, z], "yaw": r, "pitch": r}` with angles in radians |
| `cordon` | `{"min": [x, y, z], "max": [x, y, z]}` | none | The cordon box in map units, used to work on one part of a big map |
| `cordon_enabled` | bool | `false` | Hide objects outside the cordon and leave them out of cordoned exports |

## Versions and compatibility

`format` and `version` describe the data. The container version in the file header is separate and only changes when
the chunk layout does. This table shows how each reader handles a file it does not fully understand:

| Situation | Editor | Godot addon |
| --- | --- | --- |
| Other `format` | Refuses the file | Build error |
| Newer `version` | Refuses the file | Build error asking for an addon update |
| Missing `version` | Reads it as the oldest version | Same |
| Unknown key | Ignores it, and it is gone after the next save | Ignores it |
| Unknown chunk | Keeps it and writes it back on save | Skips it |
| Face index out of range | Refuses the map, naming the node | Skips that brush |
| Displacement `heights` of the wrong size | Refuses the map, naming the node | Builds the face flat |

Because the editor drops unknown keys when it saves, new data that an older editor must not lose needs one of two
things: a map version bump, which older editors refuse to open, or a chunk of its own, which they keep untouched. See
[Other chunks](container.md#other-chunks).

## Coordinates and units

Positions use Godot's axes: Y up, X right, Z towards the viewer. The build divides by the map settings'
`inverse_scale_factor`, 32 by default, so 32 map units are one meter.

| Quantity | Unit |
| --- | --- |
| Positions, sizes, heights, spacing, cell size | Map units |
| Entity, instance and scatter angles | Degrees: pitch around X, yaw around Y, roll around Z, applied in YXZ order like Godot's `rotation_degrees` |
| Camera bookmark yaw and pitch | Radians |
| UV offset | Texels |
| UV scale | Map units per texel |
| Mesh `uvs` | Texture widths, so 1 is one full repeat of the texture |
