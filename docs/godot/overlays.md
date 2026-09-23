# Overlays

Some content belongs to the game rather than the level: a vending machine with its own script, a particle effect tuned
in Godot, a lamp with an AnimationPlayer. An **overlay** keeps that content on the map without losing it on every
rebuild.

![The night district courtyard: string lights, fireflies and a sign from the overlay, lamps from the map](../assets/overlays/night-district-courtyard.jpg)

## Adding one

Add a `GodotTrenchOverlay` node as a direct child of the `FuncGodotMap` and put your nodes under it. Builds, Clear Map,
rebuilds on save, live mode and hot reload all skip it. Its nodes stay the same instances, with the same signals and
state, and are saved with your scene.

For a single node, a lighter option: a direct child of the map in the `godottrench_keep` group survives builds too.

## Positions

Overlay content uses the map's local space in meters. At 32 units per meter, a child at `(2, 0, -3)` sits where
GodotTrench shows `(64, 0, -96)`. The overlay itself stays at the map origin, so moving the map moves everything on it.

An overlay placed elsewhere in the scene can point at its map with *Map Path* and follows the map's transform.

## Wiring overlays to the map

| You want | Do this |
| --- | --- |
| A map output to reach an overlay node | Give the node a `GodotTrenchOverlayIO` child and set its *Targetname* |
| An overlay node to fire at map entities | Add a `GodotTrenchOutput` child, pick the signal in *Output* and fill in target, input and parameter |
| Content to ride on a moving entity | Put it under a `GodotTrenchAnchor` with *Target* set to the entity's targetname |

With `GodotTrenchOverlayIO`, an input calls the node's method of that name, sets its property of that name, or runs a
built in input like `show`, `hide` or `toggle`. That is enough for nodes without a script: input `emitting` with
parameter `true` starts a `GPUParticles3D`. The component also emits `input_received(input, parameter, activator)`.

A `GodotTrenchOutput` can listen to any signal, so an `Area3D` fires on `body_entered` without code. Your own script can
declare its own signal:

```python
extends StaticBody3D
## A breaker box. The player's use key calls use(), and GodotTrenchOutput
## children listening to flipped_on and flipped_off switch the map's lamps.

signal flipped_on(activator: Node)
signal flipped_off(activator: Node)

var on := false

func use(activator: Node = null) -> void:
	on = not on
	if on:
		flipped_on.emit(activator)
	else:
		flipped_off.emit(activator)
```

An anchor binds at the offset where you placed it, then follows the entity through rebuilds, live edits and runtime
movement, so a warning plate goes up with its roller door.

## After a rebuild

Targets are looked up by name when an output fires, so wiring survives rebuilds. State does not: map entities come
back in their starting state while the overlay keeps its own. The overlay emits `map_rebuilt(map)` after every build,
the moment to push your state back into the map.

## What the level designer sees

The editor cannot show Godot nodes, so the overlay describes itself. On build and on scene save it writes
`<map>.overlay.json` next to the map, with the name, bounds and targetnames of each direct child.

GodotTrench draws each one as a dashed blue ghost box with its name, so nobody builds a wall through the breaker box.
*View > Godot Overlays* hides them. Overlay targetnames also count as real targets in the Issues panel and the output
editor. Turn off *Share With Editor* to keep an overlay out of the file.

## Example

`godot/demo/overlays/night_district_overlay.tscn` has a breaker box that switches the courtyard lamps, string lights
and fireflies a map trigger turns on, steam from a manhole, and a plate riding the warehouse roller door.
