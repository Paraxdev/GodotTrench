# Wiring overlays

Nodes in an [overlay](overlays.md) join the map's entity I/O in both directions.

| You want | Do this |
| --- | --- |
| A map output to reach an overlay node | Give the node a `GodotTrenchOverlayIO` child and set its *Targetname* |
| An overlay node to fire at map entities | Add a `GodotTrenchOutput` child, pick the signal in *Output* and fill in target, input and parameter |
| Content to ride on a moving entity | Put it under a `GodotTrenchAnchor` with *Target* set to the entity's targetname |

## Receiving map outputs

With `GodotTrenchOverlayIO`, an input calls the node's method of that name, sets its property of that name, or runs a
built in input like `show`, `hide` or `toggle`, so nodes without a script work too: input `emitting` with parameter
`true` starts a `GPUParticles3D`. Every input also emits `input_received(input, parameter, activator)`.

## Firing at the map

A `GodotTrenchOutput` listens to any signal of its parent, so an `Area3D` fires on `body_entered` without code. A
script can add its own signals:

```gdscript
extends StaticBody3D

signal flipped_on(activator: Node)
signal flipped_off(activator: Node)

var on := false

# The player's use key calls use(), GodotTrenchOutput children on flipped_on and flipped_off switch the map's lamps.
func use(activator: Node = null) -> void:
	on = not on
	if on:
		flipped_on.emit(activator)
	else:
		flipped_off.emit(activator)
```

## Anchors

An anchor binds at the offset where you placed it, then follows the entity through rebuilds, live edits and runtime
movement, so a warning sign goes up with its roller door.

## After a rebuild

Wiring survives rebuilds, state does not. Map entities come back in their starting state while the overlay keeps its
own, so connect to the overlay's `map_rebuilt(map)` signal and push your state back into the map there. The demo's
breaker box switches the lamps on again if it was left on.
