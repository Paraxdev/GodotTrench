# Custom entities in GDScript

Any script can be an entity. Declare a **signal** for each output and a **method** for each input, and emit the signal
when the thing happens.

```gdscript
@tool
class_name Alarm extends Node3D

signal tripped(activator: Node)

@export var armed := true

func _func_godot_apply_properties(props: Dictionary) -> void:
	armed = GodotTrenchIO.to_bool(props.get("armed", armed))

func trip(activator: Node = null) -> void:
	if armed:
		tripped.emit(activator)

func disarm() -> void:
	armed = false
```

Make the activator the first `Node` argument of your signals, so chains further down still know who started them.
Map keys arrive in `_func_godot_apply_properties`. Read them through `GodotTrenchIO.to_bool`, `to_vector3` and
`to_color`, which accept both text and already converted values.

## Making it placeable

Add a definition for it to your project's FGD with the script as its script class, see
[Project setup](../godot/project-setup.md). The game config then exports it to the editor, with its signals as outputs
and its methods not starting with `_` as inputs. The **Reference** panel's **Create in project** writes a class
skeleton for any entity.

## Calling I/O from code

Use the same functions the map uses:

```gdscript
for door in GodotTrenchIO.find_targets(self, "main_door", null):
	GodotTrenchIO.invoke(door, &"open", "", null)
```

Go through `invoke` rather than calling the method directly. It fits the parameter to the method's arguments the way
map connections do, and handles the built-in inputs.

To fire every connection on one of an entity's outputs by hand, use
`GodotTrenchIO.fire_output(source, &"tripped", activator)`.
