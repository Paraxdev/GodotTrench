# Button opens a door

A button on the wall slides a door open when the player presses it. This is the smallest useful piece of entity I/O:
the button fires an output named `pressed`, and a link turns that into a call of the door's `open` input.

> **Tip:** If outputs, inputs and `targetname` are new to you, read [How entity I/O works](../gameplay/io.md) first. It
> takes a few minutes and this page builds on it.

## Wire it

1. Draw a door sized brush in a doorway, [Getting started](../getting-started.md#4-build-a-room) shows how to cut
   one. Right click the brush, *Create Brush Entity > func_door*, and set its `targetname` to `main_door` in the
   Inspector. The name is how other entities find the door. The default `travel` of `0 64 0` slides it up 64 units,
   2 m, when it opens.
2. Draw a small brush on the wall next to it and make it a `func_button` the same way.
3. Select both entities: click the button, then Ctrl+click the door, in a view or in the Outliner. Typing `func` in
   the Outliner's *Filter* box lists only these two, which helps when walls are in the way. Then choose
   *Gameplay > Logic > Link Two Selected Entities...*. The entity that fires, the button, must be on the left, press
   ⇄ if it is not. Pick `pressed` under *When* and `open` under *call*, then **Link**.

The status bar confirms the new output, `pressed > main_door.open`, which reads "when pressed, call open on
main_door".

> **Tip:** *Gameplay > Doors and Movers*, also in the Inspector's *Gameplay* section, turns the selected brush into
> a hinged or sliding door in one click. It names the door, sets how it opens and adds a `trigger_multiple` in front
> of it that opens the door when the player walks up. Delete that trigger when only the button should open the door.

## Let the player press it

The addon has no interaction system of its own, since every game handles "use" differently. Doors and buttons have a
`use(activator)` method and your player script calls it. The demo player does it on **E** by casting a ray from the
camera and calling `use` on the first thing it hits that has one (`godot/demo/player.gd`):

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

The player passes itself as the activator, so `pressed(activator)` carries the player along. Later outputs in the
chain can then target `!activator` to act on whoever pressed the button.

## Variations

| You want | What to change |
| --- | --- |
| Door only opens from the button | Set the door's `interact` to 0. Otherwise pressing **E** on the door opens it too |
| Door closes by itself | Set the door's `wait` to the seconds it stays open. The default, -1, keeps it open |
| Locked door | Set `locked` to 1. The door ignores `open` and `use` and fires `locked_use` instead, until an output calls its `unlock` input, for example from a key pickup |
| One button, several doors | Name the doors `gate_1`, `gate_2` and so on and target `gate_*`. A trailing `*` matches every name that starts with `gate_` |
| Swinging door | Use `func_door_rotating` instead and drag its hinge handle in the viewport to where the hinge should be |

## The same wiring in code

If you would rather connect things in GDScript than in the editor, this does the same as the link above:

```gdscript
# pressed(activator) > main_door.open
button.pressed.connect(func(activator):
	for door in GodotTrenchIO.find_targets(button, "main_door", activator):
		GodotTrenchIO.invoke(door, &"open", "", activator))
```

Connecting `pressed` straight to `door.open` fails, because `pressed` passes one argument and `open()` takes none.
`invoke` fits the arguments to the method, the same way map links do.
