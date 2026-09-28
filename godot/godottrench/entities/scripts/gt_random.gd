@tool
class_name GTRandom extends Node3D
## logic_random: pick fires one of out_1 to out_N at random, roll fires on_success with a chance in percent and on_fail
## otherwise. picked(index) carries which output pick chose.
## Inputs: pick, roll. Outputs: out_1..out_8, picked(index), on_success, on_fail.

signal out_1
signal out_2
signal out_3
signal out_4
signal out_5
signal out_6
signal out_7
signal out_8
signal picked(index: int)
signal on_success
signal on_fail

const MAX_OUTPUTS := 8

## How many outputs pick chooses from, 1 to 8.
@export var count := 3
## Never picks the same output twice in a row.
@export var no_repeat := false
## Percent chance that roll succeeds, 0 to 100.
@export var chance := 50.0

var _rng := RandomNumberGenerator.new()
var _last := 0

func _func_godot_apply_properties(props: Dictionary) -> void:
	count = clampi(int(GTMath.to_number(props.get("count", count), count)), 1, MAX_OUTPUTS)
	no_repeat = GodotTrenchIO.to_bool(props.get("no_repeat", no_repeat))
	chance = float(GTMath.to_number(props.get("chance", chance), chance))

func pick() -> void:
	var n := clampi(count, 1, MAX_OUTPUTS)
	var index := _rng.randi_range(1, n)
	if no_repeat and n > 1 and _last > 0:
		# One of the other n - 1 outputs, skipping the last one.
		index = _rng.randi_range(1, n - 1)
		if index >= _last:
			index += 1
	_last = index
	picked.emit(index)
	emit_signal("out_%d" % index)

func roll() -> void:
	if _rng.randf() * 100.0 < chance:
		on_success.emit()
	else:
		on_fail.emit()
