@tool
class_name GTDoorRotating extends AnimatableBody3D
## func_door_rotating (and func_gate): swings [member open_angle] degrees around a hinge.
## [member hinge] is the hinge position relative to the door center in map units. With [member open_away] the door
## swings away from whoever opens it.
## Inputs: open(activator), close, toggle(activator), lock, unlock, use(activator).
## Outputs: opened, closed, started_opening, started_closing, locked_use(activator).

signal opened
signal closed
signal started_opening
signal started_closing
signal locked_use(activator: Node)

@export var hinge := Vector3.ZERO
## "x", "y" or "z".
@export var axis := "y"
@export var open_angle := 95.0
## Degrees per second.
@export var speed := 120.0
@export var open_away := true
## Seconds before closing again, negative stays open.
@export var wait := -1.0
@export var locked := false
@export var start_open := false
@export var interact := true

var is_open := false
var _closed: Transform3D
var _angle := 0.0
var _tween: Tween
## Bumped by every open and close, so an auto close timer from an earlier opening does nothing.
var _cycle := 0

func _func_godot_apply_properties(props: Dictionary) -> void:
	hinge = GodotTrenchIO.to_vector3(props.get("hinge", hinge))
	axis = str(props.get("axis", axis))
	open_angle = float(props.get("open_angle", open_angle))
	speed = float(props.get("speed", speed))
	open_away = GodotTrenchIO.to_bool(props.get("open_away", open_away))
	wait = float(props.get("wait", wait))
	locked = GodotTrenchIO.to_bool(props.get("locked", locked))
	start_open = GodotTrenchIO.to_bool(props.get("start_open", start_open))
	interact = GodotTrenchIO.to_bool(props.get("interact", interact))

func _ready() -> void:
	_closed = transform
	if not Engine.is_editor_hint() and start_open:
		is_open = true
		_set_angle(open_angle)

func axis_vector() -> Vector3:
	match axis.to_lower():
		"x":
			return Vector3.RIGHT
		"z":
			return Vector3.BACK
	return Vector3.UP

## Hinge position in the parent's space.
func pivot() -> Vector3:
	return _closed.origin + _closed.basis * (hinge / GodotTrenchIO.units_per_meter(self))

func _set_angle(value: float) -> void:
	_angle = value
	var p := pivot()
	var turn := Transform3D(Basis(axis_vector(), deg_to_rad(value)), Vector3.ZERO)
	transform = Transform3D(Basis.IDENTITY, p) * turn * Transform3D(Basis.IDENTITY, -p) * _closed

## Angle that swings the far edge away from [param activator].
func angle_for(activator: Node) -> float:
	if not open_away or not activator is Node3D or not is_inside_tree() or not get_parent() is Node3D:
		return open_angle
	var parent := get_parent() as Node3D
	var local_activator := parent.to_local((activator as Node3D).global_position)
	var leaf := _closed.origin - pivot()
	var swing_dir := axis_vector().cross(leaf)
	var toward := (local_activator - _closed.origin).dot(swing_dir) * signf(open_angle)
	return -absf(open_angle) * signf(open_angle) if toward > 0.0 else open_angle

func open(activator: Node = null) -> void:
	if is_open:
		return
	if locked:
		locked_use.emit(activator)
		return
	is_open = true
	_cycle += 1
	started_opening.emit()
	_swing(angle_for(activator), opened)

func close() -> void:
	if not is_open:
		return
	is_open = false
	_cycle += 1
	started_closing.emit()
	_swing(0.0, closed)

func toggle(activator: Node = null) -> void:
	if is_open:
		close()
	else:
		open(activator)

func lock() -> void:
	locked = true

func unlock() -> void:
	locked = false

func use(activator: Node = null) -> void:
	if interact:
		toggle(activator)

func _swing(target: float, done: Signal) -> void:
	if _tween:
		_tween.kill()
	_tween = create_tween()
	_tween.tween_method(_set_angle, _angle, target, absf(target - _angle) / maxf(speed, 0.001))
	var cycle := _cycle
	_tween.finished.connect(func():
		done.emit()
		if is_open and wait >= 0.0 and is_inside_tree():
			get_tree().create_timer(wait).timeout.connect(func():
				if cycle == _cycle:
					close()))
