@tool
class_name GTPlatform extends AnimatableBody3D
## func_platform: a lift or moving platform between its start and start + [member travel] (map units).
## Bodies standing on it are carried along (AnimatableBody3D sync to physics).
## Modes: 0 toggle moves once per start input, 1 ping pong loops, 2 once goes to the end and stays.
## Inputs: start, stop, toggle, go_to_end, go_to_start. Outputs: reached_end, reached_start, started.

signal reached_end
signal reached_start
signal started

enum Mode { TOGGLE, PING_PONG, ONCE }

@export var travel := Vector3(0, 128, 0)
## Meters per second.
@export var speed := 2.0
## Seconds spent at each end in ping pong mode.
@export var wait := 1.0
@export var mode := Mode.TOGGLE
@export var start_active := false

var at_end := false
var moving := false
var _active := false
var _start: Vector3
var _tween: Tween

func _func_godot_apply_properties(props: Dictionary) -> void:
	travel = GodotTrenchIO.to_vector3(props.get("travel", travel))
	speed = float(props.get("speed", speed))
	wait = float(props.get("wait", wait))
	mode = int(props.get("mode", mode)) as Mode
	start_active = GodotTrenchIO.to_bool(props.get("start_active", start_active))

func _ready() -> void:
	_start = position
	if not Engine.is_editor_hint() and start_active:
		start.call_deferred()

func end_position() -> Vector3:
	return _start + travel / GodotTrenchIO.units_per_meter(self)

func start() -> void:
	_active = true
	started.emit()
	if mode == Mode.ONCE or not at_end:
		go_to_end()
	else:
		go_to_start()

func stop() -> void:
	_active = false
	moving = false
	if _tween:
		_tween.kill()

func toggle() -> void:
	if moving:
		stop()
	else:
		start()

func go_to_end() -> void:
	_move(end_position(), true)

func go_to_start() -> void:
	if mode != Mode.ONCE:
		_move(_start, false)

func _move(target: Vector3, to_end: bool) -> void:
	if _tween:
		_tween.kill()
	moving = true
	_tween = create_tween()
	_tween.set_process_mode(Tween.TWEEN_PROCESS_PHYSICS)
	_tween.tween_property(self, "position", target, position.distance_to(target) / maxf(speed, 0.001))
	_tween.finished.connect(func(): _arrived(to_end))

func _arrived(to_end: bool) -> void:
	moving = false
	at_end = to_end
	if to_end:
		reached_end.emit()
	else:
		reached_start.emit()
	if mode == Mode.PING_PONG and _active and is_inside_tree():
		await get_tree().create_timer(wait).timeout
		if _active and not moving:
			if at_end:
				go_to_start()
			else:
				go_to_end()
