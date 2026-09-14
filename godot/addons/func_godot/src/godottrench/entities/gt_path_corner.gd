@tool
class_name GTPathCorner extends Marker3D
## path_corner: one stop of a func_train path. [member target] names the next corner.
## Output: reached(train).

signal reached(train: Node)

@export var target := ""
## Seconds a train waits here.
@export var wait := 0.0
## New train speed in m/s from here on, 0 keeps the current speed.
@export var speed := 0.0

func _func_godot_apply_properties(props: Dictionary) -> void:
	target = str(props.get("target", target))
	wait = float(props.get("wait", wait))
	speed = float(props.get("speed", speed))
