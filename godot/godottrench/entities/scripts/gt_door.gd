@tool
class_name GTDoor extends AnimatableBody3D
## func_door: slides by [member travel] map units when opened and back when closed.
## Inputs: open, close, toggle, lock, unlock, use(activator). Outputs: opened, closed, started_opening, started_closing,
## locked_use(activator).

signal opened
signal closed
signal started_opening
signal started_closing
signal locked_use(activator: Node)

## Offset in map units.
@export var travel := Vector3(0, 64, 0)
## Meters per second.
@export var speed := 2.0
## Seconds before closing again, negative stays open.
@export var wait := -1.0
@export var locked := false
@export var start_open := false
## Opens when game code calls use(), e.g. a player interaction ray.
@export var interact := true

var is_open := false
var _closed_position: Vector3
var _tween: Tween
## Bumped by every open and close, so an auto close timer from an earlier opening does nothing.
var _cycle := 0

func _func_godot_apply_properties(props: Dictionary) -> void:
	if props.has("travel"):
		travel = GodotTrenchIO.to_vector3(props["travel"])
	elif props.has("move_distance"):
		travel = Vector3(0, float(props["move_distance"]), 0)
	speed = float(props.get("speed", speed))
	wait = float(props.get("wait", wait))
	locked = GodotTrenchIO.to_bool(props.get("locked", locked))
	start_open = GodotTrenchIO.to_bool(props.get("start_open", start_open))
	interact = GodotTrenchIO.to_bool(props.get("interact", interact))

func _ready() -> void:
	_closed_position = position
	if not Engine.is_editor_hint() and start_open:
		position = open_position()
		is_open = true

## Position of the door when fully open, in the parent's space.
func open_position() -> Vector3:
	return _closed_position + travel / GodotTrenchIO.units_per_meter(self)

func open() -> void:
	if is_open:
		return
	if locked:
		locked_use.emit(null)
		return
	is_open = true
	_cycle += 1
	started_opening.emit()
	_move_to(open_position(), opened)

func close() -> void:
	if not is_open:
		return
	is_open = false
	_cycle += 1
	started_closing.emit()
	_move_to(_closed_position, closed)

func toggle() -> void:
	if is_open:
		close()
	else:
		open()

func lock() -> void:
	locked = true

func unlock() -> void:
	locked = false

func use(activator: Node = null) -> void:
	if not interact:
		return
	if locked:
		locked_use.emit(activator)
		return
	toggle()

func _move_to(target: Vector3, done: Signal) -> void:
	if _tween:
		_tween.kill()
	var duration := position.distance_to(target) / maxf(speed, 0.001)
	_tween = create_tween()
	_tween.tween_property(self, "position", target, duration)
	var cycle := _cycle
	_tween.finished.connect(func():
		done.emit()
		if is_open and wait >= 0.0 and is_inside_tree():
			get_tree().create_timer(wait).timeout.connect(func():
				if cycle == _cycle:
					close()))
