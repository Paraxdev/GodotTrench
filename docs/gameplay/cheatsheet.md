# I/O cheatsheet

## When an output fires

1. The entity emits a signal, like `pressed(activator)`.
2. The fire counts against *Times*, then waits *Delay* seconds.
3. The target becomes a list of nodes. None found means nothing happens.
4. The input is called on each node.

## Targets

| Write | Reaches |
| --- | --- |
| `main_door` | That targetname, in the same map |
| `door_*` | Targetnames starting with `door_` |
| `@enemies` | The Godot group, whole tree |
| `!player` | First node in group `player` |
| `!activator` | Who started the chain (`!caller` too) |
| `!self` | The entity with the output |
| `/root/Game` | An absolute node path |

## Parameters

A plain value is tried as int, float, bool, then `x y z` as a `Vector3`, else it stays a string. A JSON `[a, b]` spreads
into several arguments. `$activator`, `$self`, `$position` and `$caller_name` are filled in on the **receiving** entity.

No parameter: the activator goes into the first object argument, and other values the output carried fill the rest in
order.

## Who keeps the activator

| Signals | Activator |
| --- | --- |
| `triggered`, `pressed`, `entered`, `exited`, `hurt`, `teleported`, `pushed` | Passed on |
| `arrived(corner)`, `spawned(node)`, `reached(train)` | Replaced |
| `opened`, `timer`, `step_N`, `on_true`, `broken`, `switched`, `map_spawn` | Dropped |

Lost it? Target `!player`.

## Inputs every node has

`kill`, `show`, `hide`, `enable`, `disable` and `toggle`, used when the entity has no method of that name. An unknown
input with a parameter sets the property of that name.

## logic_script scope

`this`, `activator`, `parameter`, `io` and `tree`. Indent with tabs.

```python
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

## Nothing happened?

Issues panel, then Logic panel, then F3 in game, then `GodotTrenchIO.events()`. See
[Debugging wiring](debugging.md).
