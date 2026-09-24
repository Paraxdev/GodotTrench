# Nodes

A map's content is a tree of nodes. Every node has the same outer shape, with its keys in this order:

| Key | Type | Written | Meaning |
| --- | --- | --- | --- |
| `id` | integer | always | Unique in the map |
| `type` | string | always | `layer`, `group`, `entity`, `brush`, `mesh`, `terrain`, `scatter` or `instance` |
| type fields | | | The keys of that node type, see below, [Geometry](geometry.md) and [Terrain and scatter](terrain-scatter.md) |
| `hidden` | bool | when `true` | Hidden in the editor |
| `locked` | bool | when `true` | Cannot be selected or edited in the editor |
| `children` | array of nodes | when not empty | Child nodes, in order |

`hidden` and `locked` are editor state only, so Godot still builds a hidden node. To leave content out of the build,
put it in a layer with `omit_from_export`.

## Ids

Ids are positive integers that stay the same across saves, so other data can point at a node. The live link sends
edits by id, scatter sets name the surfaces they grow on by id, and Godot stores each generated node's id in its
`_gt_id` metadata. An id of `0` or a duplicate, which only a hand edited file has, gets a fresh id on load.

A prefab is a separate `.gtm` file with ids of its own, so two copies of it would clash. To prevent that, the Godot
build adds a million times a per occurrence number to the ids of every node an [instance](#instance) brings in. Two
copies of one prefab never share ids, however deep they are nested.

## Where nodes go

| Node | Allowed parent |
| --- | --- |
| Layer | Top level only |
| Group | Layer or group |
| Brush, mesh, terrain, scatter set, point entity, instance | Layer or group |
| Brush or mesh of a brush entity | That entity |

An entity with children is a brush entity, such as a door made of brushes. Brushes and meshes outside an entity belong
to worldspawn, the static world geometry.

## Layer

A top level part of the map, shown in the editor's Outliner.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `name` | string | required | Name shown in the Outliner |
| `color` | string | required | `#rrggbbaa`. `#rrggbb` is accepted when reading |
| `omit_from_export` | bool | `false` | Always written. When true, the build skips the whole layer, for reference geometry that should never reach Godot |

## Group

Nodes gathered so they are selected and moved together.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `name` | string | required | Name shown in the Outliner |
| `link_id` | integer | omitted | Groups sharing a link id are linked copies, made with *Duplicate Linked* |
| `transform` | 16 numbers | identity, omitted | This copy's placement relative to the others, a 4x4 matrix in column major order |

Group contents are stored in world space, already placed. A [linked copy](../editor/organizing.md#groups) repeats an
edit made in one copy in all the others, and `transform` is only what the editor uses to map the edit from one copy
onto the next. The Godot build ignores both `transform` and `link_id`.

## Entity

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `classname` | string | required | Entity class from the game config, for example `prop_physics` |
| `origin` | `[x, y, z]` | `[0, 0, 0]` | Always written. Position of a point entity |
| `angles` | `[pitch, yaw, roll]` | `[0, 0, 0]` | Always written. Degrees, YXZ order like `rotation_degrees` |
| `properties` | object of string to string | omitted | Entity keys, sorted by name |
| `outputs` | array | omitted | I/O connections, see below |

Property values are strings whatever their FGD type, for example `"travel": "0 168 0"`. An `angles`, `angle` or
`mangle` property wins over the node's `angles`. The build ignores `origin` and `angles` of a brush entity, since its
brushes already sit where they belong.

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

A reference to another `.gtm` file, placed with a position and rotation, like Hammer's `func_instance`. This is how
prefabs are placed: the content lives once in its own file and every instance shows it.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `path` | string | required | The referenced map, relative to the referencing map, absolute, or `res://` |
| `origin` | `[x, y, z]` | required | Where the instance is placed |
| `angles` | `[pitch, yaw, roll]` | required | Degrees, YXZ order |
| `fixup` | string | `""`, omitted | Name prefix for the instance's contents, see below |

The build takes the children of every layer of the referenced map that is not omitted from export, and places them
transformed by `origin` and `angles`. Texture projections move with the geometry. The referenced map's worldspawn is
not used. Instances nest up to eight levels.

The fixup keeps entity names unique when one prefab is placed several times. With a fixup of `p1`, `door` becomes
`p1-door` in `targetname`, `target`, `destination`, `call_target`, every property the FGD declares as `target_source`
or `target_destination`, and every output `target`. Values starting with `!`, `@` or `/` are left alone. Nested fixups
add up from the outside in, so `b` inside `a` gives `a-b-door`. *Explode Instance* in the editor produces the same
names.
