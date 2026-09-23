# How entity I/O works

Gameplay is wired the way Hammer does it for Source. No scripts needed, the wiring is saved in the map.

Every entity has **outputs**, things it announces, and **inputs**, things it can be told to do. A connection says: when
this output fires, call that input on a target. A button's `pressed` output calls a door's `open` input, and the door
opens.

## A connection

| Field | Meaning |
| --- | --- |
| Output | The event on this entity, for example `pressed` |
| Target | Who to call, usually a targetname. See [Targets and parameters](targets.md) |
| Input | The method to call on each target, for example `open` |
| Parameter | Optional value passed to the input |
| Delay | Seconds to wait before calling |
| Times | How often it may fire. -1 means no limit |

Underneath it is plain Godot. An output is a signal and an input is a method: `func_button` has
`signal pressed(activator)`, `func_door` has `func open()`. Your own scripts can take part without any special API, see
[Custom entities](custom-entities.md).

## When an output fires

Each connection is built as a `GodotTrenchOutput` child node listening to its entity's signal. When the signal fires:

1. The fire is counted against *Times*. If the limit is used up, it stops.
2. It waits *Delay* seconds.
3. The target resolves to a list of nodes. None found means nothing happens.
4. The input is called on each node with the parameter.

With no delay all of this runs immediately, inside the call that emitted the signal, which decides the order things
happen in. With a delay the call goes on a timer, and it is dropped if the entity leaves the tree first.

## The activator

The activator is whoever started the chain, usually the player. It is the first `Node` argument of the signal that
fired. When the player walks into a trigger, `triggered(activator)` carries the player's body, and a target of
`!activator` further down means that player.

It only survives while every signal in the chain passes a node along.

| Signals | Activator |
| --- | --- |
| `triggered`, `pressed`, `entered`, `exited`, `hurt`, `teleported`, `pushed` | Passed on |
| `arrived(corner)`, `spawned(node)`, `reached(train)` | Replaced by that node |
| `opened`, `timer`, `step_N`, `on_true`, `broken`, `switched`, `map_spawn` | Dropped |

> **Tip:** Need the player several steps down a chain that lost the activator? Target `!player` instead, it does not
> depend on the chain.

## Next

[Targets and parameters](targets.md) covers who a target reaches and how values are passed. The
[I/O cheatsheet](cheatsheet.md) has all of it on one page.
