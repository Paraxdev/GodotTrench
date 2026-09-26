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

func _ready() -> void:
	value = start_value

func add(amount: Variant = 1) -> void:
	set_value(value + _to_int(amount, 1))

func subtract(amount: Variant = 1) -> void:
	set_value(value - _to_int(amount, 1))

func set_value(new_value: Variant) -> void:
	var v := clampi(_to_int(new_value, value), min_value, max_value)
	if v == value:
		return
	value = v
	changed.emit(value)
	if value >= max_value:
		hit_max.emit()
	elif value <= min_value:
		hit_min.emit()

## Map parameters arrive as numbers or text, anything else (a node, an empty value) falls back.
static func _to_int(v: Variant, fallback: int) -> int:
	if v is int or v is float or v is bool:
		return int(v)
	if v is String and v.is_valid_float():
		return int(float(v))
	return fallback

func reset() -> void:
	value = start_value
	changed.emit(value)
