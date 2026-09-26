@tool
class_name GTTrain extends AnimatableBody3D
## func_train: follows a chain of path_corner entities starting at [member target].
## Inputs: start, stop, toggle. Outputs: arrived(corner), finished.

signal arrived(corner: Node)
signal finished

@export var target := ""
## Meters per second.
@export var speed := 3.0
@export var loop := true
@export var start_active := true
## Turns to face the direction of travel.
@export var orient := false

var _running := false
var _next: Node3D
var _first: Node3D
var _tween: Tween
## Bumped by every new leg and stop, so a finished tween or corner wait from an older leg does nothing.
var _leg := 0

func _func_godot_apply_properties(props: Dictionary) -> void:
	target = str(props.get("target", target))
	speed = float(props.get("speed", speed))
	loop = GodotTrenchIO.to_bool(props.get("loop", loop))
	start_active = GodotTrenchIO.to_bool(props.get("start_active", start_active))
	orient = GodotTrenchIO.to_bool(props.get("orient", orient))

func _ready() -> void:
	if not Engine.is_editor_hint() and start_active:
		start.call_deferred()

func _corner(corner_name: String) -> Node3D:
	for n in GodotTrenchIO.find_targets(self, corner_name, null):
		if n is Node3D:
			return n
	return null

func start() -> void:
	if _running:
		return
	if not _next:
		_first = _corner(target)
		_next = _first
	if not _next:
		push_warning("[GT] func_train %s has no path_corner named '%s'" % [name, target])
		return
	_running = true
	_travel()

func stop() -> void:
	_running = false
	_leg += 1
	if _tween:
		_tween.kill()

func toggle() -> void:
	if _running:
		stop()
	else:
		start()

func _travel() -> void:
	if not _running or not _next or not is_inside_tree():
		return
	_leg += 1
	var leg := _leg
	if _tween:
		_tween.kill()
	var dest := _next.global_position
	if orient and global_position.distance_to(dest) > 0.01:
		look_at(dest, Vector3.UP)
	_tween = create_tween()
	_tween.set_process_mode(Tween.TWEEN_PROCESS_PHYSICS)
	_tween.tween_property(self, "global_position", dest, global_position.distance_to(dest) / maxf(speed, 0.001))
	_tween.finished.connect(_reached.bind(leg))

func _reached(leg: int) -> void:
	if leg != _leg:
		return
	var corner := _next
	arrived.emit(corner)
	if corner.has_signal(&"reached"):
		corner.emit_signal(&"reached", self)
	var wait := float(corner.get("wait")) if "wait" in corner else 0.0
	var new_speed := float(corner.get("speed")) if "speed" in corner else 0.0
	if new_speed > 0.0:
		speed = new_speed
	var next_name := str(corner.get("target")) if "target" in corner else ""
	_next = _corner(next_name) if next_name != "" else null
	if not _next and loop:
		_next = _first
	if not _next:
		_running = false
		finished.emit()
		return
	if wait > 0.0:
		await get_tree().create_timer(wait).timeout
		if leg != _leg:
			return
	_travel()
