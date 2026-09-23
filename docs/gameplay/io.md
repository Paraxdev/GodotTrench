# How entity I/O works

Gameplay is wired the way Hammer does it for Source, and the wiring is saved in the map. An entity has **outputs**,
events it announces, and **inputs**, things it can be told to do. A connection says: when this output fires, call that
input on a target.

## A connection

| Field | Meaning |
| --- | --- |
| Output | The event on this entity, for example `pressed` |
| Target | Who to call, usually a targetname, see [Targets](targets.md) |
| Input | The method to call on each target, for example `open` |
| Parameter | Optional value passed to the input, see [Parameters and inputs](parameters.md) |
| Delay | Seconds to wait before calling |
| Times | How often it may fire, -1 means no limit |

In Godot an output is a signal and an input is a method: `func_button` has `signal pressed(activator)` and `func_door`
has `func open()`. Your own scripts take part the same way, see [Custom entities](custom-entities.md).

## When an output fires

After *Delay* seconds the input is called on every node the target finds. A target that finds nothing does nothing,
without a warning. A delayed call is dropped if its entity left the tree in the meantime.

Without a delay the whole chain runs inside the signal's `emit`, before the code that emitted it carries on.

## The activator

The activator is whoever started the chain, usually the player, and `!activator` further down the chain means that
node. It is the first `Node` argument of the signal that fired, so it only survives while every signal in the chain
passes one along.

| Signals | Activator |
| --- | --- |
| `triggered`, `pressed`, `entered`, `exited`, `hurt`, `teleported`, `pushed` | Passed on |
| `arrived(corner)`, `spawned(node)`, `reached(train)` | Replaced by that node |
| `opened`, `timer`, `step_N`, `on_true`, `broken`, `switched`, `map_spawn` | Dropped |

> **Tip:** Need the player several steps down a chain that lost the activator? Target `!player` instead, it does not
> depend on the chain.
