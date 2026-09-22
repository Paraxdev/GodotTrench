@tool
class_name DemoLamp extends OmniLight3D
## Switchable light. Inputs: turn_on, turn_off, toggle. Output: switched.

signal switched(on: bool)

@export var start_on: bool = true

func _func_godot_apply_properties(props: Dictionary) -> void:
	start_on = bool(props.get("start_on", start_on))
	visible = start_on

func turn_on() -> void:
	visible = true
	switched.emit(true)

func turn_off() -> void:
	visible = false
	switched.emit(false)

func toggle() -> void:
	if visible:
		turn_off()
	else:
		turn_on()

func is_on() -> bool:
	return visible
