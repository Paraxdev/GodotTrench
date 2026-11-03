# Nodes

A map's content is a tree of nodes. Every node has the same outer shape, with its keys in this order:

| Key | Type | Written | Meaning |
| --- | --- | --- | --- |
| `id` | integer | always | Unique in the map |
| `type` | string | always | `layer`, `group`, `entity`, `brush`, `mesh`, `terrain`, `scatter` or `instance` |
| type fields | | | The keys of that node type, see below, [Geometry](geometry.md) and [Terrain and scatter](terrain-scatter.md) |
| `label` | string | when set | Name given with *Rename*, see below |
| `graph` | array of 2 numbers | when set | Where an entity's node sits in the [Logic graph](../gameplay/logic-graph.md) |
| `hidden` | bool | when `true` | Hidden in the editor |
| `locked` | bool | when `true` | Cannot be selected or edited in the editor |
| `children` | array of nodes | when not empty | Child nodes, in order |

`hidden` and `locked` are editor state only, so Godot still builds a hidden node. To keep something out of the build,
put it in a layer with `omit_from_export`.

`label` is the name you gave a brush, mesh, entity, instance or terrain in the editor, shown instead of a default name
like `brush12`. Layers, groups and scatter sets never have one, they keep theirs in `name`.

Godot only uses `label` for terrains, it names the node `terrain_<id>_<label>`. Brushes and meshes merge into their
entity, and an entity takes its node name from `targetname`. Editors older than the key still read files with it, but
drop it when they save.

`graph` is editor state too. It's where an entity's node sits in the Logic graph, in graph points, and only nodes
someone moved have it. Godot ignores it, and a value that isn't two numbers is dropped on load.

## Ids

Ids are positive integers that stay the same across saves, so other data can point at a node. The live link sends
edits by id, scatter sets name their surfaces by id, and Godot stores each generated node's id in its `_td_id`
metadata. An id of `0` or a duplicate, which only a hand edited file has, gets a fresh id on load.

A prefab is a separate `.tdm` file with ids of its own, so two copies of it would clash. To avoid that, the Godot build
adds a million times a per occurrence number to the id of every node an [instance](#instance) brings in. Two copies of
one prefab never share ids, however deep they're nested.

## Where nodes go

| Node | Allowed parent |
| --- | --- |
| Layer | Top level only |
| Group | Layer or group |
| Brush, mesh, terrain, scatter set, point entity, instance | Layer or group |
| Brush or mesh of a brush entity | That entity |

An entity with children is a brush entity, a door made of brushes for example. Brushes and meshes outside any entity
belong to worldspawn.

## Layer

A top level part of the map, shown in the editor's Outliner.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `name` | string | required | Name shown in the Outliner |
| `color` | string | required | `#rrggbbaa`. `#rrggbb` is accepted when reading |
| `omit_from_export` | bool | `false` | Always written. When true, the build skips the whole layer, for reference geometry that should never reach Godot |

## Group

Nodes gathered so you select and move them together.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `name` | string | required | Name shown in the Outliner |
| `link_id` | integer | omitted | Groups sharing a link id are linked copies, made with *Duplicate Linked* |
| `transform` | 16 numbers | identity, omitted | This copy's placement relative to the others, a 4x4 matrix in column major order |

Group contents are stored in world space, already in place. A [linked copy](../editor/organizing.md#groups) repeats an
edit made in one copy in all the others, and `transform` is just what the editor uses to carry the edit from one copy
to the next. The Godot build ignores both `transform` and `link_id`.

## Entity

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `classname` | string | required | Entity class from the game config, for example `prop_physics` |
| `origin` | `[x, y, z]` | `[0, 0, 0]` | Always written. Position of a point entity |
| `angles` | `[pitch, yaw, roll]` | `[0, 0, 0]` | Always written. Degrees, YXZ order like `rotation_degrees` |
| `properties` | object of string to string | omitted | Entity keys, sorted by name |
| `outputs` | array | omitted | I/O connections, see below |

Property values are always strings, whatever their FGD type, for example `"travel": "0 168 0"`.

An `angles`, `angle` or `mangle` property wins over the node's `angles` and is read the Quake way: pitch and roll
negated, yaw turned by 180 degrees. On classes starting with `light`, `mangle` is yaw, pitch and roll instead. An
`angle` of -1 or -2 points straight up or down. The editor draws the entity facing the same way, and turning it there
moves the property into the node's `angles`.

The build ignores `origin` and `angles` of a brush entity, since its brushes already sit where they belong.

Each output is one [I/O connection](../gameplay/io.md): when this entity fires a signal, call a method on a target.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `output` | string | required | Signal on this entity that triggers the connection |
| `target` | string | required | Who receives the call, see [Targets](../gameplay/parameters.md#targets) |
| `input` | string | required | Method to call on the target |
| `parameter` | string | `""`, omitted | Value passed to the input |
| `delay` | number | `0`, omitted | Seconds to wait before the call |
| `times` | integer | `-1`, omitted | How many times it may fire, `-1` for every time |

## Instance

A reference to another `.tdm` file, placed with a position and rotation, like Hammer's `func_instance`. This is how
prefabs get placed: the content lives once in its own file, and every instance shows it.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `path` | string | required | The referenced map, relative to the referencing map, absolute, or `res://` |
| `origin` | `[x, y, z]` | required | Where the instance is placed |
| `angles` | `[pitch, yaw, roll]` | required | Degrees, YXZ order |
| `fixup` | string | `""`, omitted | Name prefix for the instance's contents, see below |

The build takes the children of every layer in the referenced map that isn't omitted from export, and moves them by
`origin` and `angles`. Texture projections move along with the geometry. The referenced map's worldspawn isn't used.
Instances nest up to eight levels deep.

A Godot build reads each referenced map once, and all the instances of a map together may add 2,097,152 values, a
value being an array item or dictionary entry of the referenced map, with every 8 characters of text counting as one
more. Past that the build leaves the remaining instances out and reports an error. Ordinary maps stay far below it,
only instances of instances multiplied many times over reach it.

With a fixup of `p1`, `door` becomes `p1-door` in `targetname`, `target`, `destination`, `call_target`, every property
the FGD declares as `target_source` or `target_destination`, and every output `target`. Values starting with `!`, `@`
or `/` are left alone. Nested fixups add up from the outside in, so `b` inside `a` gives `a-b-door`, and *Explode
Instance* in the editor gives the same names.
