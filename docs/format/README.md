# The .gtm map format

These pages are for people writing a tool that reads or writes `.gtm` files, such as an importer, a converter or a CI
check. You do not need them to build levels.

A `.gtm` file holds a map's worldspawn properties, some editor state and a tree of nodes. The editor saves it as a
binary container of zstd compressed chunks, which is small and survives partial damage. Maps from older editors are one
UTF-8 JSON document instead, and every reader loads both. Both forms hold the same tree, so these pages describe it as
the JSON that `godottrench --dump` prints.

| Page | Read it when you |
| --- | --- |
| [Container layout](container.md) | Parse or write the binary file: header, chunks and how values are encoded |
| [Damaged files](recovery.md) | Want to know what survives a corrupt file and where readers report the losses |
| [Nodes](nodes.md) | Walk the tree: node shape, ids, layers, groups, entities, outputs and instances |
| [Geometry](geometry.md) | Read or build brushes, faces, texture projection, displacements and meshes |
| [Terrain and scatter](terrain-scatter.md) | Handle heightmap terrains and scattered models |
| [JSON layout](json.md) | Work with the readable form, convert between forms, or read the clipboard |

The reference implementation is
[`format.rs`](https://github.com/Paraxdev/GodotTrench/blob/main/crates/gt_doc/src/format.rs), with the container in
[`binary.rs`](https://github.com/Paraxdev/GodotTrench/blob/main/crates/gt_doc/src/binary.rs) and
[`variant.rs`](https://github.com/Paraxdev/GodotTrench/blob/main/crates/gt_doc/src/variant.rs). The Godot addon reads
files with [`gtm_file.gd`](https://github.com/Paraxdev/godottrench_func/blob/main/src/godottrench/gtm_file.gd) and
builds them with [`gtm_parser.gd`](https://github.com/Paraxdev/godottrench_func/blob/main/src/godottrench/gtm_parser.gd).

## Top level

| Key | Type | Written | Meaning |
| --- | --- | --- | --- |
| `format` | string | always | `"godottrench-map"`, tells a map apart from other JSON |
| `version` | integer | always | Map version, currently 1 |
| `properties` | object of string to string | when not empty | Worldspawn keys, sorted |
| `editor` | object | when not all default | Editor state, never exported to Godot |
| `layers` | array of nodes | always | [Layer nodes](nodes.md#layer), in order |

Only `layer` nodes are read from `layers`. A map with no layers gets a `Default` layer when it loads.

Worldspawn values are strings, like in a `.map` file. `GodotTrenchEnvironment` turns these keys into a sky, fog and sun
when Godot builds the map:

| Keys | Control |
| --- | --- |
| `sun_angles`, `sun_color`, `sun_energy` | The directional sun light |
| `ambient_color`, `ambient_energy` | Light that reaches surfaces the sun does not |
| `sky_top_color`, `sky_horizon_color`, `sky_ground_color`, `sky_energy` | The procedural sky gradient and its brightness |
| `sky_panorama` | A `res://` image that replaces the procedural sky |
| `fog_color`, `fog_density` | Distance fog |
| `glow_intensity`, `ssr` | Bloom on bright surfaces, and screen space reflections when `ssr` is `1` |

`environment` set to `sun_only` builds only the sun, and `0` or `none` builds nothing, see
[environment and sun](../godot/building.md#environment-and-sun). `sky_source` records which texture the sky faces of an
imported map had.

The `editor` object holds view state that only matters inside the editor:

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `cameras` | object, keys `"1"` to `"9"` | empty | Camera bookmarks, `{"position": [x, y, z], "yaw": r, "pitch": r}` in radians |
| `cordon` | `{"min": [x, y, z], "max": [x, y, z]}` | none | Cordon box in map units |
| `cordon_enabled` | bool | `false` | Hide objects outside the cordon and leave them out of cordoned exports |

## Versions and compatibility

`format` and `version` describe the data. The container version in the file header is separate and only changes when
the chunk layout does. This table shows how each reader handles a file it does not fully understand:

| Situation | Editor | Godot addon |
| --- | --- | --- |
| Other `format` | Refuses the file | Build error |
| Newer `version` | Refuses the file | Build error asking for an addon update |
| Missing `version` | Read as the oldest version | Same |
| Unknown key | Ignored, and gone after the next save | Ignored |
| Unknown chunk | Kept and written back on save | Skipped |
| Face index out of range | Refuses the map, naming the node | Skips that brush |
| Displacement `heights` of the wrong size | Refuses the map, naming the node | Builds the face flat |

Because unknown keys are dropped on save, new data that an older editor must not lose needs either a map version bump,
which older editors refuse, or a chunk of its own, see [Other chunks](container.md#other-chunks).

## Coordinates and units

Positions use Godot's axes: Y up, X right, Z towards the viewer. The build divides by the map settings'
`inverse_scale_factor`, 32 by default, so 32 map units are one meter.

| Quantity | Unit |
| --- | --- |
| Positions, sizes, heights, spacing, cell size | Map units |
| Entity, instance and scatter angles | Degrees, pitch X, yaw Y, roll Z, applied in YXZ order |
| Camera bookmark yaw and pitch | Radians |
| UV offset | Texels |
| UV scale | Map units per texel |
| Mesh `uvs` | Texture widths, so 1 is one full repeat of the texture |
