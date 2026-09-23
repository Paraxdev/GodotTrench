# Nodes

Every node has the same outer shape, with its keys in this order:

| Key | Type | Written | Meaning |
| --- | --- | --- | --- |
| `id` | integer | always | Unique in the map |
| `type` | string | always | `layer`, `group`, `entity`, `brush`, `mesh`, `terrain`, `scatter` or `instance` |
| type fields | | | See below, [Geometry](geometry.md) and [Terrain and scatter](terrain-scatter.md) |
| `hidden` | bool | when `true` | Hidden in the editor |
| `locked` | bool | when `true` | Cannot be selected or edited |
| `children` | array of nodes | when not empty | Child nodes, in order |

`hidden` and `locked` are editor state only, Godot still builds a hidden node. To leave content out of the build, put it
in a layer with `omit_from_export`.

## Ids

Ids are positive integers that stay the same across saves. The live link sends edits by id, scatter sets name their
targets by id, and Godot stores each generated node's id in its `_gt_id` metadata. An id of `0` or a duplicate, which
only a hand edited file has, gets a fresh id on load.

Prefabs reuse the ids of their own file, so the Godot build adds a million times a per occurrence number to the ids of
every node an [instance](#instance) brings in. Two copies of one prefab never share ids, however deep they are nested.

## Where nodes go

| Node | Parent |
| --- | --- |
| Layer | Top level only |
| Group | Layer or group |
| Brush, mesh, terrain, scatter set, point entity, instance | Layer or group |
| Brush or mesh of a brush entity | That entity |

An entity with children is a brush entity. Brushes and meshes outside an entity belong to worldspawn.

## Layer

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `name` | string | required | |
| `color` | string | required | `#rrggbbaa`, `#rrggbb` is accepted when reading |
| `omit_from_export` | bool | `false` | Always written. The build skips the whole layer |

## Group

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `name` | string | required | |
| `link_id` | integer | omitted | Groups sharing a link id are linked copies |
| `transform` | 16 numbers | identity, omitted | This copy's placement relative to the others, 4x4 column major |

Group contents are stored in world space, already placed. `transform` only mirrors an edit in one linked copy into the
others, and the Godot build ignores it and `link_id`.

## Entity

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `classname` | string | required | Entity class from the game config |
| `origin` | `[x, y, z]` | `[0, 0, 0]` | Always written. Position of a point entity |
| `angles` | `[pitch, yaw, roll]` | `[0, 0, 0]` | Always written. Degrees, YXZ order like `rotation_degrees` |
| `properties` | object of string to string | omitted | Entity keys, sorted |
| `outputs` | array | omitted | I/O connections |

Property values are strings whatever their FGD type, for example `"travel": "0 168 0"`. An `angles`, `angle` or
`mangle` property wins over the node's `angles`. The build ignores `origin` and `angles` of a brush entity.

Each output:

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `output` | string | required | Signal on this entity |
| `target` | string | required | See [Targets](../gameplay/parameters.md#targets) |
| `input` | string | required | Method to call on the target |
| `parameter` | string | `""`, omitted | Value passed to the input |
| `delay` | number | `0`, omitted | Seconds before the call |
| `times` | integer | `-1`, omitted | How many times it may fire, `-1` for every time |

## Instance

A reference to another `.gtm` placed with a transform, like Hammer's `func_instance`. Prefabs are placed as instances.

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `path` | string | required | Relative to the referencing map, absolute, or `res://` |
| `origin` | `[x, y, z]` | required | |
| `angles` | `[pitch, yaw, roll]` | required | Degrees, YXZ order |
| `fixup` | string | `""`, omitted | Name prefix for the instance's contents |

The build places the children of every layer of the referenced map that is not omitted from export, transformed by
`origin` and `angles`, and texture projections move with the geometry. The referenced map's worldspawn is not used.
Instances nest up to eight levels.

With a fixup of `p1`, `door` becomes `p1-door` in `targetname`, `target`, `destination`, `call_target`, every property
the FGD declares as `target_source` or `target_destination`, and every output `target`. Values starting with `!`, `@`
or `/` are left alone. Nested fixups add up from the outside in, so `b` inside `a` gives `a-b-door`. *Explode Instance*
in the editor produces the same names.
