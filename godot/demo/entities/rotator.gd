@tool
class_name DemoRotator extends AnimatableBody3D
## Brush entity spinning around its vertical axis. Inputs: start, stop, toggle.

@export var degrees_per_second: float = 45.0
@export var running := true

func _func_godot_apply_properties(props: Dictionary) -> void:
	degrees_per_second = float(props.get("degrees_per_second", degrees_per_second))
	running = bool(props.get("start_on", running))

func _physics_process(delta: float) -> void:
	if running and not Engine.is_editor_hint():
		rotate_y(deg_to_rad(degrees_per_second) * delta)

func start() -> void:
	running = true

func stop() -> void:
	running = false

func toggle() -> void:
	running = not running
