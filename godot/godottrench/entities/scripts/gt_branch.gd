@tool
class_name GTBranch extends Node3D
## logic_branch: stores a boolean and, on test, fires on_true or on_false, for conditional wiring.
## Inputs: set_true, set_false, toggle, test, set_and_test(value). Outputs: on_true, on_false.

signal on_true
signal on_false

@export var start_value := false

var value := false

func _func_godot_apply_properties(props: Dictionary) -> void:
	start_value = GodotTrenchIO.to_bool(props.get("start_value", start_value))
	value = start_value

func _ready() -> void:
	value = start_value

func set_true() -> void:
	value = true

func set_false() -> void:
	value = false

func toggle() -> void:
	value = not value

func test() -> void:
	if value:
		on_true.emit()
	else:
		on_false.emit()

func set_and_test(new_value: Variant = null) -> void:
	if new_value != null:
		value = GodotTrenchIO.to_bool(new_value)
	test()
