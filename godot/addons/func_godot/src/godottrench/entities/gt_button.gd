@tool
class_name GTButton extends AnimatableBody3D
## func_button: pressed by input or by game code calling use(). Moves by [member travel] map units while pressed.
## Inputs: press(activator), release, lock, unlock, use(activator). Outputs: pressed(activator), released, locked_use(activator).

signal pressed(activator: Node)
signal released
signal locked_use(activator: Node)

@export var travel := Vector3(0, -2, 0)
## Seconds before popping out again, negative stays pressed.
@export var wait := 1.0
@export var locked := false
@export var interact := true

var is_pressed := false
var _rest: Vector3
## Bumped by every press and release, so a pop out timer from an earlier press does nothing.
var _cycle := 0

func _func_godot_apply_properties(props: Dictionary) -> void:
	travel = GodotTrenchIO.to_vector3(props.get("travel", travel))
	wait = float(props.get("wait", wait))
	locked = GodotTrenchIO.to_bool(props.get("locked", locked))
	interact = GodotTrenchIO.to_bool(props.get("interact", interact))

func _ready() -> void:
	_rest = position

func press(activator: Node = null) -> void:
	if locked:
		locked_use.emit(activator)
		return
	if is_pressed:
		return
	is_pressed = true
	_cycle += 1
	if is_inside_tree():
		create_tween().tween_property(self, "position", _rest + travel / GodotTrenchIO.units_per_meter(self), 0.08)
	pressed.emit(activator)
	if wait >= 0.0 and is_inside_tree():
		var cycle := _cycle
		get_tree().create_timer(maxf(wait, 0.1)).timeout.connect(func():
			if cycle == _cycle:
				release())

func release() -> void:
	if not is_pressed:
		return
	is_pressed = false
	_cycle += 1
	if is_inside_tree():
		create_tween().tween_property(self, "position", _rest, 0.12)
	released.emit()

func lock() -> void:
	locked = true

func unlock() -> void:
	locked = false

func use(activator: Node = null) -> void:
	if interact:
		press(activator)
