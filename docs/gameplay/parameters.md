# Targets, parameters and inputs

## Targets

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

### Runtime changes

Targetname lookups are cached per map. After spawning or renaming named entities at runtime, call
`GodotTrenchIO.invalidate(node)` with any node in that map. Renaming means changing the `gt_targetname` meta, the node's
`name` plays no part.

Nodes made by a spawner have no targetname. Reach them through their group, for example `@enemies`.

## Parameters

A plain parameter becomes the first of these that fits: an integer, a float, `true` or `false`, three numbers as a
`Vector3`, else a string. A JSON array like `[10, "$activator"]` spreads into several arguments. Values are converted
to the types the method declares, and when it takes a node before any other argument, text like `!player` or a
targetname is passed in as that node.

| Placeholder | Becomes |
| --- | --- |
| `$activator` | The activator |
| `$self`, `$caller` | The receiving entity |
| `$position` | The receiving entity's global position |
| `$caller_name` | The receiving entity's targetname, else its node name |

Placeholders are filled in on the **receiving** entity, not the one that fired. They must be the whole parameter or a
whole array element, they are not replaced inside longer text.

## No parameter

With the parameter left empty, the activator goes into the first argument typed as an object, or untyped and named
`activator`, wherever it sits. That is why `take_damage(amount, source)` gets it in `source`. Other values the output
carried fill the remaining arguments in order, so `changed(value)` wired to `set_value` passes the value on.

That forwards `false` too, so `switched(false)` wired to a counter's `add` adds 0. Set an explicit parameter when you
want a fixed value.

## How an input is found

1. A method of that name, or its PascalCase or camelCase spelling, so a C# `AddScore` answers to `add_score`.
2. A property of that name, set to the parameter or the value the output carried. The input `speed` with parameter `4`
   changes a door's speed.
3. One of the built-in inputs below.

| Input | Effect |
| --- | --- |
| `kill` | Frees the node |
| `show`, `enable` | Makes it visible and processing |
| `hide`, `disable` | Makes it invisible and stops processing |
| `toggle` | Flips visibility |

The entity's own methods win, so `disable` on a trigger switches the trigger off instead of hiding it. `show` and
`hide` are the built-in ones unless the entity's script defines its own. Any other input prints
`[GT I/O] <node> has no input '<name>'`.
