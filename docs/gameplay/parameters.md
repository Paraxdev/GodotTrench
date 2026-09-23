# Targets, parameters and inputs

## Targets

The target of a connection, and of keys like `call_target`, decides which nodes receive the input. Most of the time
it is a targetname you gave another entity, the other forms cover groups of things and nodes outside the map.

| Target | Reaches | Use it for |
| --- | --- | --- |
| `main_door` | Every entity with that targetname in the same map | The normal case, one named entity or several sharing a name |
| `door_*` | Every targetname starting with `door_` | Opening a whole set at once. Only a trailing `*` works |
| `@enemies` | Every node in the Godot group `enemies`, anywhere in the tree | Things spawned at runtime or placed by game code |
| `!player` | The first node in the `player` group | Reaching the player when the chain lost the activator |
| `!activator` | Whoever started the chain, `!caller` means the same | Hurting or teleporting the one who touched a trigger |
| `!self` | The entity that owns the output | An entity acting on itself, like a button that disables itself |
| `/root/Game` | An absolute node path, it must start with `/` | Autoloads and other game code outside the map |

Targetnames are looked up inside the nearest `FuncGodotMap`, so two maps in one scene cannot reach each other's names.
Groups, node paths and `!player` work across the whole tree, use them when one map has to talk to another.

> **Note:** In Source, `!caller` is the entity that fired the output. Here it means the same as `!activator`, use
> `!self` for the entity itself.

### Runtime changes

Targetname lookups are cached per map for speed. After spawning or renaming named entities at runtime, call
`GodotTrenchIO.invalidate(node)` with any node in that map so the next lookup sees them. Renaming means changing the
`gt_targetname` meta, the node's `name` plays no part.

Nodes made by a spawner have no targetname. Reach them through their group, for example `@enemies`.

## Parameters

The parameter is a value typed into the connection and handed to the input, like `25` for `take_damage` or a speed
for a door. A plain parameter becomes the first of these that fits: an integer, a float, `true` or `false`, three
numbers as a `Vector3`, else a string.

A JSON array like `[10, "$activator"]` spreads into several arguments, for inputs that take more than one. Values are
converted to the types the method declares, so `1234` reaches a `String` argument as text. When the method takes a node
before any other argument, text like `!player` or a targetname is looked up and passed in as that node.

A parameter can also be one of these placeholders, filled in when the input is called:

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

When a connection fires, the input name is matched against the target in this order:

1. A method of that name, or its PascalCase or camelCase spelling, so a C# `AddScore` answers to `add_score`.
2. A property of that name, set to the parameter or the value the output carried. The input `speed` with parameter `4`
   changes a door's speed, no extra method needed.
3. One of the built-in inputs below, which every node understands.

| Input | Effect |
| --- | --- |
| `kill` | Frees the node, removing it from the game |
| `show`, `enable` | Makes it visible and lets it process again |
| `hide`, `disable` | Makes it invisible and pauses its processing |
| `toggle` | Flips visibility only, processing is left as it is |

The entity's own methods win, so `disable` on a trigger switches the trigger off instead of hiding it. `show` and
`hide` are the built-in ones unless the entity's script defines its own. Any other input prints
`[GT I/O] <node> has no input '<name>'`.
