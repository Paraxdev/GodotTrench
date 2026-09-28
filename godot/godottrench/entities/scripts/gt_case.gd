@tool
class_name GTCase extends Node3D
## logic_case: compares a value with case_1 to case_8 in order and fires on_case_N for the first that matches, or
## on_default when none does. Numbers match by value, text must match exactly. Empty cases are skipped.
## Inputs: in_value(value). Outputs: on_case_1..on_case_8, on_default.

signal on_case_1
signal on_case_2
signal on_case_3
signal on_case_4
signal on_case_5
signal on_case_6
signal on_case_7
signal on_case_8
signal on_default

const MAX_CASES := 8

@export var case_1 := ""
@export var case_2 := ""
@export var case_3 := ""
@export var case_4 := ""
@export var case_5 := ""
@export var case_6 := ""
@export var case_7 := ""
@export var case_8 := ""

func _func_godot_apply_properties(props: Dictionary) -> void:
	for i in range(1, MAX_CASES + 1):
		set("case_%d" % i, str(props.get("case_%d" % i, get("case_%d" % i))))

func in_value(value: Variant = null) -> void:
	var v: Variant = GTMath.normalize(value)
	if v != null:
		for i in range(1, MAX_CASES + 1):
			var wanted := str(get("case_%d" % i)).strip_edges()
			if wanted != "" and GTMath.compare(v, GTMath.normalize(wanted)) == 0:
				emit_signal("on_case_%d" % i)
				return
	on_default.emit()
