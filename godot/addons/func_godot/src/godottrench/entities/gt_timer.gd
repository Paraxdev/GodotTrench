@tool
class_name GTTimer extends Node3D
## logic_timer: fires timer every interval, or at random intervals between random_min and random_max.
## Inputs: start, stop, toggle, fire. Output: timer.

signal timer

@export var interval := 1.0
@export var random_min := 0.0
@export var random_max := 0.0
@export var start_on := true
@export var once := false

var running := false
var _timer: Timer

func _func_godot_apply_properties(props: Dictionary) -> void:
	interval = float(props.get("interval", interval))
	random_min = float(props.get("random_min", random_min))
	random_max = float(props.get("random_max", random_max))
	start_on = GodotTrenchIO.to_bool(props.get("start_on", start_on))
	once = GodotTrenchIO.to_bool(props.get("once", once))

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	_timer = Timer.new()
	_timer.one_shot = true
	_timer.timeout.connect(_tick)
	add_child(_timer)
	if start_on:
		start()

func _next_wait() -> float:
	if random_max > random_min:
		return randf_range(random_min, random_max)
	return maxf(interval, 0.01)

func start() -> void:
	running = true
	if _timer:
		_timer.start(_next_wait())

func stop() -> void:
	running = false
	if _timer:
		_timer.stop()

func toggle() -> void:
	if running:
		stop()
	else:
		start()

func fire() -> void:
	timer.emit()

func _tick() -> void:
	timer.emit()
	if once:
		running = false
	elif running:
		_timer.start(_next_wait())
