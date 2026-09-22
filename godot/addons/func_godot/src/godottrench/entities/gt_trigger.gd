@tool
class_name GTTrigger extends Area3D
## trigger_once and trigger_multiple: fires when a body of [member filter_group] enters.
## Inputs: enable, disable, toggle. Outputs: triggered(activator), entered(activator), exited(activator).
## Subclasses override [method _on_triggered] to act on the activator.

signal triggered(activator: Node)
signal entered(activator: Node)
signal exited(activator: Node)

## Only bodies in this group count, empty accepts every body.
@export var filter_group := "player"
@export var start_disabled := false
## Disables itself after firing once.
@export var once := false
## Seconds before the same body fires it again.
@export var cooldown := 0.5

var enabled := true
## Seconds each body last fired it at, by instance id.
var _last_fired: Dictionary = {}

func _func_godot_apply_properties(props: Dictionary) -> void:
	filter_group = str(props.get("filter_group", filter_group))
	start_disabled = GodotTrenchIO.to_bool(props.get("start_disabled", start_disabled))
	once = GodotTrenchIO.to_bool(props.get("once", once))
	cooldown = float(props.get("cooldown", cooldown))
	# A map built inside the running tree applies properties after _ready.
	if is_node_ready() and not Engine.is_editor_hint():
		enabled = not start_disabled

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	enabled = not start_disabled
	body_entered.connect(_body_entered)
	body_exited.connect(_body_exited)

func accepts(body: Node) -> bool:
	return filter_group == "" or body.is_in_group(StringName(filter_group))

func _body_entered(body: Node) -> void:
	if not enabled or not accepts(body):
		return
	entered.emit(body)
	var now := Time.get_ticks_msec() / 1000.0
	var id := body.get_instance_id()
	if now - float(_last_fired.get(id, -1.0e9)) < cooldown:
		return
	if _last_fired.size() > 64:
		for key in _last_fired.keys():
			if now - float(_last_fired[key]) >= cooldown:
				_last_fired.erase(key)
	_last_fired[id] = now
	fire(body)

func _body_exited(body: Node) -> void:
	if accepts(body):
		exited.emit(body)

## Fires as if [param activator] had entered, used by inputs and tests.
func fire(activator: Node = null) -> void:
	if not enabled:
		return
	triggered.emit(activator)
	_on_triggered(activator)
	if once:
		enabled = false

func _on_triggered(_activator: Node) -> void:
	pass

func enable() -> void:
	enabled = true

func disable() -> void:
	enabled = false

func toggle() -> void:
	enabled = not enabled

## Bodies of the filter group currently inside.
func bodies_inside() -> Array[Node]:
	var out: Array[Node] = []
	for b in get_overlapping_bodies():
		if accepts(b):
			out.append(b)
	return out
