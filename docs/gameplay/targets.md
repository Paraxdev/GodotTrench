# Targets

The target of a connection, and of keys like `call_target`, picks the nodes that receive the input.

| Target | Reaches |
| --- | --- |
| `main_door` | Every entity with that targetname in the same map |
| `door_*` | Every targetname starting with `door_`, only a trailing `*` works |
| `@enemies` | Every node in the Godot group `enemies`, anywhere in the tree |
| `!player` | The first node in the `player` group |
| `!activator` | Whoever started the chain, `!caller` means the same |
| `!self` | The entity that owns the output |
| `/root/Game` | An absolute node path, handy for autoloads, must start with `/` |

Targetnames are looked up inside the nearest `FuncGodotMap`, so two maps in one scene cannot reach each other's names.
Groups, node paths and `!player` work across the whole tree.

> **Note:** In Source, `!caller` is the entity that fired the output. Here it means the same as `!activator`, use
> `!self` for the entity itself.

## Runtime changes

Targetname lookups are cached per map. After spawning or renaming named entities at runtime, call
`GodotTrenchIO.invalidate(node)` with any node in that map. Renaming means changing the `gt_targetname` meta, the node's
`name` plays no part.

Nodes made by a spawner have no targetname. Reach them through their group, for example `@enemies`.
