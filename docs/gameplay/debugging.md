# Debugging wiring

Most wiring mistakes fail quietly: the door just stays shut. Check in this order, from the cheapest check to the
most thorough.

1. **Issues panel.** It flags outputs aimed at a targetname no entity has, and inputs the target's class does not
   have. A typo in a targetname is the most common reason a door stays shut.
2. **Logic panel.** It fires an output on paper and shows the whole chain, see
   [Placing and wiring](placing.md#checking-wiring-without-godot).
3. **F3 in game.** Add a `GodotTrenchDebugOverlay` node to your scene, or as an autoload. F3 lists outputs as they fire
   and draws every `Area3D` volume with its targetname, so you see whether a trigger never fired or fired at nothing.
   Toggle it again after spawning triggers, volumes are collected when it turns on. The key is the overlay's
   `toggle_key` property.
4. **Logging.** Place a `logic_debug` entity with `trace_all` set to 1 to print every output that fires to the Godot
   output. From code, connect to `GodotTrenchIO.events().fired`, which is emitted for every output, including ones
   whose target resolved to nothing.

```gdscript
GodotTrenchIO.events().fired.connect(func(source, output, target, input, parameter):
	print(source.name, ".", output, " -> ", target, ".", input))
```

## What each mistake looks like

| Mistake | Symptom | Fix |
| --- | --- | --- |
| Target resolves to nothing | `[GT I/O] <node>.<output>: target '<name>' matches no node` the first time it fires | Make the target match the entity's targetname exactly, case included |
| Target has no such input | `[GT I/O] <node> has no input '<name>'` in the Godot output | Pick an input the target's class has, see the [Entity reference](entities/README.md) |
| Output the entity never emits | `[GT I/O] <node> has no signal '<output>'` once, when the map is built | Pick an output the class has, or fire it from code with `GodotTrenchIO.fire_output()` |
| Entity spawned or renamed at runtime | Not found by its targetname | Call `GodotTrenchIO.invalidate()`, see [Runtime changes](parameters.md#runtime-changes) |
