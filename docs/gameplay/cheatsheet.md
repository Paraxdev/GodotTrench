# I/O cheatsheet

## When an output fires

The signal fires, counts against *Times*, waits *Delay*, then calls the input on every node the target finds. None
found means nothing happens, and a targetname or node path that finds nothing warns once.

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

`kill`, `show`, `hide`, `enable`, `disable` and `toggle`, used when the entity has no method of that name. An input
that matches a property sets it to the parameter.

## logic_script scope

`this`, `activator`, `parameter`, `io` and `tree`. Indent with tabs.

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

## Nothing happened?

Issues panel, then Logic panel, then F3 in game, then `GodotTrenchIO.events()`. See
[Debugging wiring](debugging.md).
