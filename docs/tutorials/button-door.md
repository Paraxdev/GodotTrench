# Button opens a door

## Wire it

1. Draw a door sized brush in a doorway. Right click it, *Create Brush Entity > func_door*, and set its `targetname`
   to `main_door` in the Inspector. The default `travel` of `0 64 0` slides it up 64 units, 2 m.
2. Draw a small brush on the wall next to it and make it a `func_button` the same way.
3. Select both entities and choose *Gameplay > Logic > Link Two Selected Entities...*. The button must be on the left,
   press ⇄ if it is not. Pick `pressed` under *When* and `open` under *call*, then **Link**.

The status bar confirms the new output, `pressed > main_door.open`.

## Let the player press it

The addon has no interaction system of its own. Doors and buttons have a `use(activator)` method and your player
calls it. The demo player does it on **E** with a ray from the camera (`godot/demo/player.gd`):

```gdscript
func use_target() -> void:
	var from := camera.global_position
	var query := PhysicsRayQueryParameters3D.create(from, from - camera.global_basis.z * use_distance)
	query.exclude = [get_rid()]
	query.collide_with_areas = false
	var hit := get_world_3d().direct_space_state.intersect_ray(query)
	var node: Node = hit.get("collider")
	while node:
		if node.has_method(&"use"):
			node.call(&"use", self)
			return
		node = node.get_parent()
```

The player passes itself, so `pressed(activator)` carries the player and later outputs can target `!activator`.

## Variations

| Want | Change |
| --- | --- |
| Door only opens from the button | Set the door's `interact` to 0, otherwise pressing **E** on the door opens it too |
| Door closes by itself | Set the door's `wait` in seconds, -1 keeps it open |
| Locked door | Set `locked` to 1. It ignores `open` and `use` and fires `locked_use` instead, until an output calls `unlock` |
| One button, several doors | Name them `gate_1`, `gate_2` and target `gate_*` |
| Swinging door | Use `func_door_rotating` and drag its hinge handle into place |

## The same wiring in code

```gdscript
# pressed(activator) > main_door.open
button.pressed.connect(func(activator):
	for door in GodotTrenchIO.find_targets(button, "main_door", activator):
		GodotTrenchIO.invoke(door, &"open", "", activator))
```

Connecting `pressed` straight to `door.open` fails, because `pressed` passes one argument and `open()` takes none.
`invoke` fits the arguments to the method.
