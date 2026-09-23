# Debugging wiring

Most wiring mistakes fail quietly. Check in this order.

## 1. In the editor

**The Issues panel** flags outputs aimed at targetnames no entity has. A typo in a targetname is the most common reason
a door stays shut.

**The Logic panel** fires an output on paper and shows the whole chain, see
[Placing and wiring](placing.md#checking-wiring-without-godot).

## 2. In the running game

Add a `GodotTrenchDebugOverlay` node to your scene and press **F3** in game. It lists outputs as they fire and draws
every trigger volume. That answers the usual question: either the trigger never fired, or it fired at a target that
does not exist.

Volumes are collected when you turn the overlay on, so toggle it again after spawning new triggers.

## 3. In code

`GodotTrenchIO.events()` emits `fired(source, output, target, input, parameter)` for every output, including ones whose
target resolved to nothing:

```python
GodotTrenchIO.events().fired.connect(func(source, output, target, input, parameter):
	print(source.name, ".", output, " -> ", target, ".", input))
```

A `logic_debug` entity with `trace_all` set to 1 logs the same without code.

## What each mistake looks like

| Mistake | Symptom |
| --- | --- |
| Target resolves to nothing | Nothing at all, no warning |
| Target has no such input | `[GT I/O] <node> has no input '<name>'` in the Godot output |
| Output the entity never emits | A warning once, when the map is built |

## Spawned and renamed entities

Targetnames are cached per map. After spawning or renaming named entities at runtime, call
`GodotTrenchIO.invalidate(node)`. Renaming means changing the `gt_targetname` meta, a node's `name` plays no part.

Spawned nodes have no targetname at all. Reach them through their group, for example `@enemies`.
