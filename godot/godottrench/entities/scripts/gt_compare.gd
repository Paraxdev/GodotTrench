@tool
class_name GTCompare extends Node3D
## math_compare: compares a stored value with compare_value and fires on_less, on_equal or on_greater, and on_not_equal
## besides the first and last. Numbers compare by value, anything else by its text.
## Inputs: set_value(value), set_compare_value(value), compare, set_and_compare(value).
## Outputs: on_less, on_equal, on_not_equal, on_greater.

signal on_less
signal on_equal
signal on_not_equal
signal on_greater

@export var compare_value := 0.0

var value: Variant = 0
var _compare: Variant = 0

func _func_godot_apply_properties(props: Dictionary) -> void:
	compare_value = float(GTMath.to_number(props.get("compare_value", compare_value), compare_value))
	_compare = GTMath.normalize(compare_value)

func _ready() -> void:
	value = 0
	_compare = GTMath.normalize(compare_value)

func set_value(new_value: Variant = null) -> void:
	var v: Variant = GTMath.normalize(new_value)
	if v != null:
		value = v

func set_compare_value(new_value: Variant = null) -> void:
	var v: Variant = GTMath.normalize(new_value)
	if v != null:
		_compare = v

func compare() -> void:
	var order := GTMath.compare(value, _compare)
	if order < 0:
		on_less.emit()
	elif order > 0:
		on_greater.emit()
	else:
		on_equal.emit()
	if order != 0:
		on_not_equal.emit()

## Without a value it compares the one already stored.
func set_and_compare(new_value: Variant = null) -> void:
	set_value(new_value)
	compare()
