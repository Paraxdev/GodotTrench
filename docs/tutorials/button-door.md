# Button opens a door

The smallest piece of useful wiring, and a good first test that a project is set up right.

## 1. Build it

1. Draw a door sized brush in a doorway. Right click it, *Create Brush Entity > func_door*. In the Inspector set its
   targetname to `main_door`. Its default `travel` of `0 64 0` slides it up 64 units, 2 m.
2. Draw a small brush on the wall next to it and turn it into a `func_button` the same way.
3. Select both and choose *Gameplay > Logic > Link Two Selected Entities...*. Make sure the button is the source, press
   ⇄ if not. Pick `pressed` under *When*, `open` under *call*, then **Link**.

The button now has one output, `pressed > main_door.open`. The Logic panel shows it as
`button.pressed → main_door.open`.

## 2. Let the player press it

Save and build the map in Godot. Nothing presses the button yet, because the addon has no interaction system of its
own. Doors and buttons expose `use(activator)` and leave it to your player to call it.

The demo player does it on **E** with a ray from the camera:

```python
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

The player passes itself, so `pressed(activator)` carries the player and anything further down the chain can target
`!activator`.

## Variations

| Want | Change |
| --- | --- |
| Door closes again by itself | Set the door's `wait` in seconds |
| Locked door | `locked` 1. It fires `locked_use` instead, a hook for a rattle sound or hint text. Another output can `unlock` it later |
| One button, several doors | Name them `gate_1`, `gate_2` and target `gate_*` |
| Swinging door | Use `func_door_rotating` and drag its hinge into place |

## The same thing in code

Underneath it is ordinary Godot. Wired by hand it would be:

```python
# pressed(activator) > main_door.open
button.pressed.connect(func(activator):
	for door in GodotTrenchIO.find_targets(button, "main_door", activator):
		GodotTrenchIO.invoke(door, &"open", "", activator))
```

Connecting `pressed` straight to `door.open` would fail: `pressed` passes one argument and `open()` takes none.
`invoke` fits the arguments to the method.
