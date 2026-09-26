@tool
class_name GTSequence extends Node3D
## logic_sequence: a cutscene timeline. On start it fires step_1, step_2 up to step_8 in order, waiting between them,
## so a non programmer can script a scene by wiring each step to an entity. Inputs: start, stop, reset.
## Outputs: step_1..step_8, step(index), finished.

signal step_1
signal step_2
signal step_3
signal step_4
signal step_5
signal step_6
signal step_7
signal step_8
signal step(index: int)
signal finished

## How many steps to fire, 1 to 8.
@export var steps := 3
## Seconds between steps, used when a step has no time of its own.
@export var interval := 1.0
## Space separated per step delays in seconds, e.g. "0 1 0.5", overriding interval where present.
@export var times := ""
@export var start_active := false
@export var loop := false

var _running := false
var _index := 0
var _run_id := 0

func _func_godot_apply_properties(props: Dictionary) -> void:
	steps = clampi(int(props.get("steps", steps)), 1, 8)
	interval = float(props.get("interval", interval))
	times = str(props.get("times", times))
	start_active = GodotTrenchIO.to_bool(props.get("start_active", start_active))
	loop = GodotTrenchIO.to_bool(props.get("loop", loop))

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	if start_active:
		start.call_deferred()

func start() -> void:
	if _running:
		return
	_running = true
	_index = 0
	_run_id += 1
	_run(_run_id)

func stop() -> void:
	_running = false

func reset() -> void:
	stop()
	_index = 0

## [param run_id] ends a timeline whose timer is still pending when stop and start happen within one wait.
func _run(run_id: int) -> void:
	while true:
		var waited := false
		while _running and run_id == _run_id and _index < steps:
			var wait := _delay_for(_index)
			if wait > 0.0:
				waited = true
				await get_tree().create_timer(wait).timeout
			if not _running or run_id != _run_id or not is_inside_tree():
				return
			_index += 1
			_fire(_index)
		if not _running or run_id != _run_id:
			return
		if not loop:
			break
		_index = 0
		# A loop whose steps all wait 0 would otherwise run forever within one frame.
		if not waited:
			await get_tree().process_frame
			if not _running or run_id != _run_id or not is_inside_tree():
				return
	_running = false
	finished.emit()

func _delay_for(i: int) -> float:
	var parts := times.split(" ", false)
	if i < parts.size() and parts[i].is_valid_float():
		return parts[i].to_float()
	return maxf(interval, 0.0)

func _fire(n: int) -> void:
	step.emit(n)
	match n:
		1:
			step_1.emit()
		2:
			step_2.emit()
		3:
			step_3.emit()
		4:
			step_4.emit()
		5:
			step_5.emit()
		6:
			step_6.emit()
		7:
			step_7.emit()
		8:
			step_8.emit()
