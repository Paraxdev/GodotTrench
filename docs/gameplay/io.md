# How entity I/O works

Entity I/O is how a map makes things happen without writing code, wired the way Hammer does it for Source. Every
entity has **outputs**, events it announces such as a button being pressed, and **inputs**, things it can be told to
do such as a door opening. A connection says: when this output fires, call that input on a target. The wiring is saved
in the map, so a level designer can hook a button to a door and the door opens in game.

## A connection

Each output of an entity can hold any number of connections. One connection has these fields:

| Field | What it does |
| --- | --- |
| Output | The event on this entity that starts the connection, for example `pressed` on a button |
| Target | Which entities receive the call, usually a targetname, see [Targets](parameters.md#targets) |
| Input | What the targets are told to do, for example `open` on a door |
| Parameter | An optional value handed to the input, like an amount of damage, see [Parameters](parameters.md#parameters) |
| Delay | Seconds to wait before calling the input, 0 calls it at once. Use it to stagger events, like lights coming on one after another |
| Times | How many times the connection may fire. -1 means every time, 1 makes a one shot event |

In Godot an output is a signal and an input is a method: `func_button` has `signal pressed(activator)` and `func_door`
has `func open()`. Your own scripts take part the same way, see [Custom entities](custom-entities.md).

## When an output fires

After *Delay* seconds the input is called on every node the target finds. A target that finds nothing does nothing.
For a targetname or node path that is almost always a typo, so it prints a warning the first time it fires. A delayed
call is dropped if its entity left the tree in the meantime, for example because it was killed.

Without a delay the whole chain runs inside the signal's `emit`, before the code that emitted it carries on. That
matters when your script reads state right after emitting, since the doors and counters further down have already
reacted.

## The activator

The activator is whoever started the chain, usually the player who pressed the button or walked into the trigger.
Further down the chain `!activator` means that node, so a trigger can hurt or teleport exactly the one who touched it.

It is taken from the first `Node` argument of the signal that fired, so it only survives while every signal in the
chain passes one along. The built-in signals behave like this:

| Signals | What happens to the activator |
| --- | --- |
| `triggered`, `pressed`, `entered`, `exited`, `hurt`, `teleported`, `pushed` | Passed on unchanged, these signals carry the node that caused them |
| `arrived(corner)`, `spawned(node)`, `reached(train)` | Replaced by the node in brackets, so further down `!activator` is that corner, spawned node or train |
| `opened`, `timer`, `step_N`, `on_true`, `broken`, `switched`, `map_spawn` | Dropped. These carry no node, so from here on `!activator` finds nothing |

> **Tip:** Need the player several steps down a chain that lost the activator? Target `!player` instead, it does not
> depend on the chain.
