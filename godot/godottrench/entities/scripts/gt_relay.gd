@tool
class_name GTRelay extends Node3D
## logic_relay: forwards trigger to its triggered output while enabled.
## Inputs: trigger(activator), enable, disable, toggle. Output: triggered(activator).

signal triggered(activator: Node)

@export var start_disabled := false

var enabled := true

func _func_godot_apply_properties(props: Dictionary) -> void:
	start_disabled = GodotTrenchIO.to_bool(props.get("start_disabled", start_disabled))
	enabled = not start_disabled

func _ready() -> void:
	enabled = not start_disabled

func trigger(activator: Node = null) -> void:
	if enabled:
		triggered.emit(activator)

func enable() -> void:
	enabled = true

func disable() -> void:
	enabled = false

func toggle() -> void:
	enabled = not enabled
