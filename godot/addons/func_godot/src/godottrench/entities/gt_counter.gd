@tool
class_name GTCounter extends Node3D
## logic_counter: counts add and subtract inputs, fires hit_max and hit_min at its limits.
## Inputs: add(amount), subtract(amount), set_value(value), reset. Outputs: hit_max, hit_min, changed(value).

signal hit_max
signal hit_min
signal changed(value: int)

@export var min_value := 0
@export var max_value := 3
@export var start_value := 0

var value := 0

func _func_godot_apply_properties(props: Dictionary) -> void:
	min_value = int(props.get("min", min_value))
	max_value = int(props.get("max", max_value))
	start_value = int(props.get("start_value", start_value))
	value = start_value

func add(amount: Variant = 1) -> void:
	set_value(value + (int(amount) if amount != null else 1))

func subtract(amount: Variant = 1) -> void:
	set_value(value - (int(amount) if amount != null else 1))

func set_value(new_value: Variant) -> void:
	var v := clampi(int(new_value), min_value, max_value)
	if v == value:
		return
	value = v
	changed.emit(value)
	if value >= max_value:
		hit_max.emit()
	elif value <= min_value:
		hit_min.emit()

func reset() -> void:
	value = start_value
	changed.emit(value)
