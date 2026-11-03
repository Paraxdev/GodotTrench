# The .tdm map format

These pages are for you if you're writing a tool that reads or writes `.tdm` files, like an importer, a converter or a
CI check. You don't need any of this to build levels.

A `.tdm` file holds three things: the worldspawn properties, a bit of editor state, and a tree of nodes with the actual
content.

The editor saves it as a binary container of zstd compressed chunks, so files stay small and a damaged file still
opens. Older editors saved one UTF-8 JSON document instead, and every reader loads both. Both forms hold the same tree,
so these pages show it as the JSON that `trenchdot --dump` prints.

| Page | Read it when you |
| --- | --- |
| [Container layout](container.md) | Parse or write the binary file. It covers the file header, the chunks and how each value is encoded as bytes |
| [Damaged files](recovery.md) | Want to know what still loads from a corrupt file, and where each reader reports what was lost |
| [Nodes](nodes.md) | Walk the tree. It explains the shape every node shares, how ids work, and the layer, group, entity and instance nodes |
| [Geometry](geometry.md) | Read or build brushes and meshes, including their faces, texture projection and displacements |
| [Terrain and scatter](terrain-scatter.md) | Handle heightmap terrains and sets of scattered models |
| [JSON layout](json.md) | Work with the readable form, convert between the two forms, or read what the editor puts on the clipboard |

Want to see working code? The reference implementation is
[`format.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/format.rs), with the container in
[`binary.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/binary.rs) and
[`variant.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/variant.rs). On the Godot side,
[`tdm_file.gd`](https://github.com/Paraxdev/trenchdot_func/blob/main/src/trenchdot/tdm_file.gd) reads the file and
[`tdm_parser.gd`](https://github.com/Paraxdev/trenchdot_func/blob/main/src/trenchdot/tdm_parser.gd) builds it.

## Top level

| Key | Type | Written | Meaning |
| --- | --- | --- | --- |
| `format` | string | always | Always `"trenchdot-map"`. It tells a map apart from other JSON |
| `version` | integer | always | Map version, currently 1 |
| `properties` | object of string to string | when not empty | Worldspawn keys, sorted by name |
| `editor` | object | when not all default | Editor view state. It is never exported to Godot |
| `textures` | object | when any texture has settings | [Texture settings](geometry.md#texture-settings) by material name |
| `layers` | array of nodes | always | The map's [layer nodes](nodes.md#layer), in order. Everything else hangs below them |
| `lightmap` | object | JSON maps with a bake | The baked lighting, which a binary map keeps in its [LMAP chunk](container.md#lmap) |

Only `layer` nodes are read from `layers`, and a map without any gets a `Default` layer when it loads.

Worldspawn values are always strings, like in a `.map` file. When Godot builds the map, `TrenchDotEnvironment` turns
these keys into a sky, fog and a sun:

| Keys | What they control |
| --- | --- |
| `sun_angles`, `sun_color`, `sun_energy` | The directional sun light: where it points, its color and how bright it is |
| `ambient_color`, `ambient_energy` | Light that reaches surfaces the sun does not, so shadows are not pitch black |
| `sky_top_color`, `sky_horizon_color`, `sky_ground_color`, `sky_energy` | The procedural sky gradient and its brightness |
| `sky_panorama` | A `res://` image that replaces the procedural sky in Godot. The editor preview and its bake keep using the sky colors |
| `fog_color`, `fog_density` | Distance fog |
| `glow_intensity`, `ssr` | Bloom on bright surfaces, and screen space reflections when `ssr` is `1` |

If your game brings its own `WorldEnvironment`, set `environment` to `sun_only` and only the sun gets built. `0` or
`none` builds nothing, see [environment and sun](../godot/building.md#environment-and-sun). `sky_source` remembers
which texture the sky faces of an imported map had.

The `editor` object is view state that only matters inside the editor:

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `cameras` | object, keys `"1"` to `"9"` | empty | Saved camera positions, each `{"position": [x, y, z], "yaw": r, "pitch": r}` with angles in radians |
| `cordon` | `{"min": [x, y, z], "max": [x, y, z]}` | none | The cordon box in map units, used to work on one part of a big map |
| `cordon_enabled` | bool | `false` | Hide objects outside the cordon and leave them out of cordoned exports |
| `bake` | `{"quality", "texel_size", "softness", "backend"}` | none | The options the map was last [baked](../editor/light-baking.md) with, so the next bake starts from them |
| `graph_frames` | array | empty | Frames of the [Logic graph](../gameplay/logic-graph.md#frames), each `{"title", "rect": [x, y, width, height], "color"}` in graph points. A damaged one is dropped on load |

## Versions and compatibility

`format` and `version` describe the data. The container version in the file header is separate, it only changes when
the chunk layout does.

Here is what each reader does with a file it doesn't fully understand:

| Situation | Editor | Godot addon |
| --- | --- | --- |
| Other `format` | Refuses the file | Build error |
| Newer `version` | Refuses the file | Build error asking for an addon update |
| Missing `version` | Reads it as the oldest version | Same |
| Unknown key | Ignores it, and it is gone after the next save | Ignores it |
| Unknown chunk | Keeps it and writes it back on save | Skips it |
| Face index out of range | Refuses the map, naming the node | Skips that brush |
| Displacement `heights` of the wrong size | Refuses the map, naming the node | Builds the face flat |

The editor drops unknown keys when it saves. So if you add data that an older editor must not lose, you need either a
map version bump, which older editors refuse to open, or a chunk of its own, which they keep untouched. See
[Other chunks](container.md#other-chunks).

## Coordinates and units

Positions use Godot's axes: Y up, X right, Z towards the viewer. The build divides by the map settings'
`inverse_scale_factor`, 32 by default, so 32 map units make one meter.

| Quantity | Unit |
| --- | --- |
| Positions, sizes, heights, spacing, cell size | Map units |
| Entity, instance and scatter angles | Degrees: pitch around X, yaw around Y, roll around Z, applied in YXZ order like Godot's `rotation_degrees` |
| Camera bookmark yaw and pitch | Radians |
| UV offset | Texels |
| UV scale | Map units per texel |
| Mesh `uvs` | Texture widths, so 1 is one full repeat of the texture |
