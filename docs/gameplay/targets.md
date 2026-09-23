# Targets and parameters

## Targets

| Target | Reaches |
| --- | --- |
| `main_door` | Every entity with that targetname in the same map |
| `door_*` | Every targetname starting with `door_`. Only a trailing `*` works |
| `@enemies` | Every node in the Godot group `enemies`, anywhere in the tree |
| `!player` | The first node in the `player` group |
| `!activator` | Whoever started the chain. `!caller` means the same |
| `!self` | The entity that owns the output |
| `/root/Game` | An absolute node path, handy for autoloads. Must start with `/` |

Targetnames are looked up inside the nearest `FuncGodotMap`, so two maps in one scene cannot reach each other's names.
Groups, node paths and `!player` work across the whole tree.

> **Note:** In Source, `!caller` is the entity that fired the output. Here it means the same as `!activator`. Use
> `!self` for the entity itself.

## Parameters

The parameter is a string, converted when the input is called. A plain value is tried in this order:

1. Integer
2. Float
3. `true` or `false`
4. Three numbers, as a `Vector3`
5. Otherwise it stays a string

A JSON array like `[10, "$activator"]` spreads into several arguments. Each value is then converted to the type the
method declares. If the input's first argument is a `Node`, a text parameter is resolved as a target, so `!player` or a
targetname can be passed straight in.

### Placeholders

`$activator`, `$self` (or `$caller`), `$position` and `$caller_name` are replaced before the call. They are worked out
on the **receiving** entity, so `$position` is the target's position. Inside an array, a placeholder only counts when
the element is exactly that string.

### No parameter

With the parameter left empty:

* The activator goes into the first argument typed as an object or named `activator`, wherever it sits. That is why
  `take_damage(amount, source)` gets it in `source`.
* Any other values the output carried fill the remaining arguments in order. `changed(value)` wired to `set_value`
  passes the value through.

This forwards `false` too, so `switched(false)` wired to a counter's `add` adds `0`. Set an explicit parameter when you
want a fixed value.

## Inputs that are not methods

The input name is matched against the target's methods as written, then in PascalCase, then camelCase, so a C#
`AddScore` answers to `add_score`.

If no method matches and there is a parameter, a property of that name is set instead. The input `speed` with parameter
`4` changes a door's speed.

If neither exists, these work on any node:

| Input | Effect |
| --- | --- |
| `kill` | Frees the node |
| `show`, `enable` | Visible and processing |
| `hide`, `disable` | The opposite |
| `toggle` | Flips visibility |

An entity's own methods always come first, which is why `disable` on a trigger switches the trigger off instead of just
hiding it.
