@tool
class_name GTGate extends Node3D
## logic_gate: two stored booleans, a and b, combined by a gate (and, or, xor, nand, nor, xnor or not, which looks at a
## only). Changing an input that flips the result fires changed(on) and then on_true or on_false, and test fires the
## current result again. A missing value means true, so set_a alone turns a on.
## Inputs: set_a(value), set_b(value), toggle_a, toggle_b, test. Outputs: on_true, on_false, changed(on).

signal on_true
signal on_false
signal changed(on: bool)

@export var gate := "and"
@export var start_a := false
@export var start_b := false

var a := false
var b := false
var _on := false

func _func_godot_apply_properties(props: Dictionary) -> void:
	gate = str(props.get("gate", gate)).strip_edges().to_lower()
	start_a = GodotTrenchIO.to_bool(props.get("start_a", start_a))
	start_b = GodotTrenchIO.to_bool(props.get("start_b", start_b))
	_reset()

func _ready() -> void:
	_reset()

func _reset() -> void:
	a = start_a
	b = start_b
	_on = _result()

func set_a(value: Variant = null) -> void:
	a = _flag(value)
	_update()

func set_b(value: Variant = null) -> void:
	b = _flag(value)
	_update()

func toggle_a() -> void:
	a = not a
	_update()

func toggle_b() -> void:
	b = not b
	_update()

func test() -> void:
	_fire(_on)

func _result() -> bool:
	match gate:
		"or":
			return a or b
		"xor":
			return a != b
		"nand":
			return not (a and b)
		"nor":
			return not (a or b)
		"xnor":
			return a == b
		"not":
			return not a
	return a and b

func _update() -> void:
	var now := _result()
	if now == _on:
		return
	_on = now
	changed.emit(_on)
	_fire(_on)

func _fire(on: bool) -> void:
	if on:
		on_true.emit()
	else:
		on_false.emit()

## Numbers are true unless zero, text follows the map's yes, true and 1.
static func _flag(v: Variant) -> bool:
	if v == null:
		return true
	if v is int or v is float:
		return v != 0
	return GodotTrenchIO.to_bool(v)
