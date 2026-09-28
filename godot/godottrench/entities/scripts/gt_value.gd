@tool
class_name GTValue extends Node3D
## math_value: stores a number or a piece of text, for scores, counts and messages other entities read.
## Inputs: set_value(value), add(amount), get_value, reset. Outputs: value(value) for get_value, changed(value)
## whenever the stored value changes. add sums two numbers and appends when either is text.

signal value(value: Variant)
signal changed(value: Variant)

## Number or text, whatever it starts as and reset returns to.
@export var start_value := "0"

var current: Variant = 0

func _func_godot_apply_properties(props: Dictionary) -> void:
	start_value = str(props.get("start_value", start_value))
	current = _start()

func _ready() -> void:
	current = _start()

func _start() -> Variant:
	return GTMath.to_number(start_value, start_value)

func set_value(new_value: Variant = null) -> void:
	_store(GTMath.normalize(new_value))

func add(amount: Variant = 1) -> void:
	var b: Variant = GTMath.normalize(amount)
	if b == null:
		b = 1
	if current is String or b is String:
		_store(GTMath.normalize(str(current) + str(b)))
	else:
		_store(GTMath.tidy(float(current) + float(b)))

func get_value() -> void:
	value.emit(current)

func reset() -> void:
	_store(_start())

func _store(v: Variant) -> void:
	if v == null or (typeof(v) == typeof(current) and v == current):
		return
	current = v
	changed.emit(current)
