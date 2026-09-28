@tool
class_name GTFlipFlop extends Node3D
## logic_flipflop: every trigger flips it, firing on_a when it turns on and on_b when it turns off, so one button can
## switch something on and off. reset puts it back to its starting state without firing on_a or on_b.
## Inputs: trigger, reset. Outputs: on_a, on_b, changed(on).

signal on_a
signal on_b
signal changed(on: bool)

## Starts on, so the first trigger fires on_b.
@export var start_on := false

var on := false

func _func_godot_apply_properties(props: Dictionary) -> void:
	start_on = GodotTrenchIO.to_bool(props.get("start_on", start_on))
	on = start_on

func _ready() -> void:
	on = start_on

func trigger() -> void:
	on = not on
	changed.emit(on)
	if on:
		on_a.emit()
	else:
		on_b.emit()

func reset() -> void:
	if on == start_on:
		return
	on = start_on
	changed.emit(on)
