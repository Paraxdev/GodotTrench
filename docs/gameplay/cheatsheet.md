# I/O cheatsheet

A one page summary of [How entity I/O works](io.md) and [Targets, parameters and inputs](parameters.md), for when you
know the idea and only need the details.

## When an output fires

The signal fires, counts against *Times*, waits *Delay*, then calls the input on every node the target finds. None
found means nothing happens, and a targetname or node path that finds nothing warns once.

## Targets

| Write | Reaches |
| --- | --- |
| `main_door` | Every entity with that targetname, in the same map |
| `door_*` | Every targetname starting with `door_` |
| `@enemies` | Every node in that Godot group, anywhere in the tree |
| `!player` | The first node in group `player` |
| `!activator` | Whoever started the chain, `!caller` means the same |
| `!self` | The entity that owns the output |
| `/root/Game` | The node at that absolute path |

## Parameters

A plain value is tried as int, float, bool, then `x y z` as a `Vector3`, else it stays a string. A JSON `[a, b]` spreads
into several arguments. `$activator`, `$self`, `$position` and `$caller_name` are filled in on the **receiving** entity.

With no parameter, the activator goes into the first object argument, and other values the output carried fill the
rest in order.

## Who keeps the activator

| Signals | Activator |
| --- | --- |
| `triggered`, `pressed`, `entered`, `exited`, `hurt`, `teleported`, `pushed` | Passed on unchanged |
| `arrived(corner)`, `spawned(node)`, `reached(train)` | Replaced by the node in brackets |
| `opened`, `timer`, `step_N`, `on_true`, `broken`, `switched`, `map_spawn` | Dropped |

If the chain lost it and you need the player, target `!player`.

## Inputs every node has

`kill`, `show`, `hide`, `enable`, `disable` and `toggle` work on any node, as long as the entity has no method of that
name. An input that matches a property sets it to the parameter.

## logic_script scope

A [`logic_script`](entities/scripting.md) runs a snippet of GDScript when fired. Inside it, `this` is the logic_script
itself, `activator` the node that started the chain, `parameter` the value passed in, `io` the `GodotTrenchIO` helpers
and `tree` the `SceneTree`. Indent with tabs.

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

## Nothing happened?

Check the Issues panel, then the Logic panel, then F3 in game, then log `GodotTrenchIO.events()`. See
[Debugging wiring](debugging.md).
