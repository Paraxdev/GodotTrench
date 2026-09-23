# Custom entities in GDScript

Any script can be an entity. Declare a **signal** for each output and a **method** for each input, and emit the signal
when the thing happens.

## A minimal entity

This is the whole trigger path of `logic_relay`:

```python
@tool
class_name GTRelay extends Node3D
signal triggered(activator: Node)    # the "triggered" output

func trigger(activator: Node = null) -> void:    # the "trigger" input
	if enabled:
		triggered.emit(activator)    # fires every connection on this output
```

Pass the activator as the first `Node` argument of your signals, so chains further down still know who started them.

## Making it placeable

Add a definition to your project's FGD (see [Project setup](../godot/project-setup.md)) and the game config exports it
to the editor with the inputs and outputs found in the script. The **Reference** panel's **Create in project** writes a
class skeleton for you.

## Calling I/O from code

Use the same functions the map uses:

```python
# Open everything the map would reach as "main_door".
for door in GodotTrenchIO.find_targets(self, "main_door", null):
	GodotTrenchIO.invoke(door, &"open", "", null)
```

Go through `invoke` rather than calling the method directly. It fits the parameter to the method's arguments, the same
way map connections do.

To fire every connection on one of an entity's outputs by hand, use
`GodotTrenchIO.fire_output(source, &"triggered", activator)`.
