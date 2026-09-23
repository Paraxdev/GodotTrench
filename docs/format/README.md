# The .gtm map format

A `.gtm` file is a GodotTrench map: worldspawn properties, some editor state and a tree of nodes. The editor saves it
as a binary container of zstd compressed chunks. Maps from older editors are one UTF-8 JSON document instead, and every
reader loads both. The two hold the same tree of objects, arrays, strings, numbers and booleans, and these pages
describe it as the JSON that `godottrench --dump` prints.

| Page | Covers |
| --- | --- |
| [Container layout](container.md) | File header, chunks, value encoding |
| [Damaged files](recovery.md) | What a reader recovers and where it reports losses |
| [Nodes](nodes.md) | Node shape, ids, layers, groups, entities, outputs, instances |
| [Geometry](geometry.md) | Brushes, faces, texture projection, displacements, meshes |
| [Terrain and scatter](terrain-scatter.md) | Heightmap terrains and scatter sets |
| [JSON layout](json.md) | The readable form, the clipboard and an example |

The editor reads and writes maps in
[`format.rs`](https://github.com/Paraxdev/GodotTrench/blob/main/crates/gt_doc/src/format.rs), with the container in
[`binary.rs`](https://github.com/Paraxdev/GodotTrench/blob/main/crates/gt_doc/src/binary.rs) and
[`variant.rs`](https://github.com/Paraxdev/GodotTrench/blob/main/crates/gt_doc/src/variant.rs). The Godot addon reads
files with
[`gtm_file.gd`](https://github.com/Paraxdev/godottrench_func/blob/main/src/godottrench/gtm_file.gd)
and builds them with
[`gtm_parser.gd`](https://github.com/Paraxdev/godottrench_func/blob/main/src/godottrench/gtm_parser.gd).

## Converting

| Command | Result |
| --- | --- |
| `godottrench --dump map.gtm` | Prints the map as JSON |
| `godottrench --to-json map.gtm map.json` | Writes the map as JSON |
| `godottrench --to-gtm map.json map.gtm` | Writes a binary map, refusing a file the editor could not open |

The conversions copy the file content without loading it into the editor, so they keep keys the editor does not know.
The editor also opens `.json` maps directly, and *Save As* with a `.json` name writes JSON. To see map changes as JSON
in `git diff`, see [Map files in git](../development.md#map-files-in-git).

## Top level

| Key | Type | Written | Meaning |
| --- | --- | --- | --- |
| `format` | string | always | `"godottrench-map"` |
| `version` | integer | always | Map version, currently 1 |
| `properties` | object of string to string | when not empty | Worldspawn keys, sorted |
| `editor` | object | when not all default | Editor state, never exported |
| `layers` | array of nodes | always | [Layer nodes](nodes.md#layer), in order |

Worldspawn values are strings, like in a `.map` file. `GodotTrenchEnvironment` builds a sky, fog and sun from `sun_angles`,
`sun_color`, `sun_energy`, `ambient_color`, `ambient_energy`, `sky_top_color`, `sky_horizon_color`, `sky_ground_color`,
`sky_energy`, `fog_color`, `fog_density`, `glow_intensity` and `ssr`, and `environment` set to `0` turns it off.

Only `layer` nodes are read from `layers`. A map with no layers gets a `Default` layer when it loads.

The `editor` object:

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `cameras` | object, keys `"1"` to `"9"` | empty | Bookmarks, `{"position": [x, y, z], "yaw": r, "pitch": r}` in radians |
| `cordon` | `{"min": [x, y, z], "max": [x, y, z]}` | none | Cordon box in map units |
| `cordon_enabled` | bool | `false` | Hide objects outside the cordon and leave them out of cordoned exports |

## Versions and compatibility

The container version in the file header only changes with the chunk layout, see [Container layout](container.md).
`format` and `version` describe the data inside.

| Situation | Editor | Godot addon |
| --- | --- | --- |
| Other `format` | Refuses the file | Build error |
| Newer `version` | Refuses the file | Build error asking for an addon update |
| Missing `version` | Read as the oldest version | Same |
| Unknown key | Ignored, and gone after the next save | Ignored |
| Unknown chunk | Kept and written back on save | Skipped |
| Face index out of range | Refuses the map, naming the node | Skips that brush |
| Displacement `heights` of the wrong size | Refuses the map, naming the node | Builds the face flat |

The editor holds a map as typed nodes, so it drops unknown keys on load. Data that an older editor must not lose needs a
map version bump, which older editors refuse, or a chunk of its own, see [Other chunks](container.md#other-chunks).

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
| Mesh `uvs` | Texture widths |
