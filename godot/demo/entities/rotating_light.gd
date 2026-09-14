@tool
class_name DemoRotatingLight extends SpotLight3D
## Lighthouse beam: a spot light turning around the vertical axis. Inputs: turn_on, turn_off, toggle.

@export var degrees_per_second: float = 60.0

func _func_godot_apply_properties(props: Dictionary) -> void:
	degrees_per_second = float(props.get("degrees_per_second", degrees_per_second))
	visible = bool(props.get("start_on", true))

func _process(delta: float) -> void:
	if visible and not Engine.is_editor_hint():
		rotate_y(deg_to_rad(degrees_per_second) * delta)

func turn_on() -> void:
	visible = true

func turn_off() -> void:
	visible = false

func toggle() -> void:
	visible = not visible
