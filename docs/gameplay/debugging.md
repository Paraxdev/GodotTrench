# Debugging wiring

Most wiring mistakes fail quietly. Check in this order.

1. **Issues panel.** It flags outputs aimed at a targetname no entity has, and inputs the target's class does not
   have. A typo in a targetname is the most common reason a door stays shut.
2. **Logic panel.** It fires an output on paper and shows the whole chain, see
   [Placing and wiring](placing.md#checking-wiring-without-godot).
3. **F3 in game.** Add a `GodotTrenchDebugOverlay` node to your scene, or as an autoload. F3 lists outputs as they fire
   and draws every `Area3D` volume with its targetname, so you see whether a trigger never fired or fired at nothing.
   Toggle it again after spawning triggers, volumes are collected when it turns on.
4. **Code.** `GodotTrenchIO.events()` emits `fired` for every output, including ones whose target resolved to nothing.
   A `logic_debug` entity with `trace_all` set to 1 logs the same without code.

```gdscript
GodotTrenchIO.events().fired.connect(func(source, output, target, input, parameter):
	print(source.name, ".", output, " -> ", target, ".", input))
```

## What each mistake looks like

| Mistake | Symptom |
| --- | --- |
| Target resolves to nothing | Nothing at all, no warning |
| Target has no such input | `[GT I/O] <node> has no input '<name>'` in the Godot output |
| Output the entity never emits | `[GT I/O] <node> has no signal '<output>'` once, when the map is built |
| Entity spawned or renamed at runtime | Not found by its targetname, see [Targets](targets.md#runtime-changes) |
