@tool
class_name GTCalc extends Node3D
## math_calc: applies one operation to the number it receives and its operand, then fires the result.
## Inputs: calculate(value). Outputs: result(value).

signal result(value: Variant)

## add, subtract, multiply, divide, modulo, power, min, max, abs, negate, round, floor or ceil.
@export var operation := "add"
## The second number. abs, negate, round, floor and ceil ignore it.
@export var operand := 1.0

var _warned := false

func _func_godot_apply_properties(props: Dictionary) -> void:
	operation = str(props.get("operation", operation)).strip_edges().to_lower()
	operand = float(GTMath.to_number(props.get("operand", operand), operand))

## Nothing fires when there is no answer, like a division by zero, and a warning says why once.
func calculate(value: Variant = null) -> void:
	var a := float(GTMath.to_number(value, 0))
	var b := operand
	var answer: Variant = null
	match operation:
		"add":
			answer = a + b
		"subtract":
			answer = a - b
		"multiply":
			answer = a * b
		"divide":
			if b != 0.0:
				answer = a / b
		"modulo":
			if b != 0.0:
				answer = fposmod(a, b)
		"power":
			answer = pow(a, b)
		"min":
			answer = minf(a, b)
		"max":
			answer = maxf(a, b)
		"abs":
			answer = absf(a)
		"negate":
			answer = -a
		"round":
			answer = roundf(a)
		"floor":
			answer = floorf(a)
		"ceil":
			answer = ceilf(a)
		_:
			_warn("has no operation '%s'" % operation)
			return
	var number: Variant = GTMath.tidy(answer) if answer != null else null
	if number == null:
		_warn("has no answer for %s %s %s" % [a, operation, b])
		return
	result.emit(number)

func _warn(reason: String) -> void:
	if not _warned:
		_warned = true
		push_warning("[GT math_calc] %s %s" % [name, reason])
