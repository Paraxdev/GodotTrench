@tool
class_name GTLight extends OmniLight3D
## light: a switchable omni light. Inputs: turn_on, turn_off, toggle. Output: switched(on).

signal switched(on: bool)

@export var start_on := true

var _on := true

func _func_godot_apply_properties(props: Dictionary) -> void:
	light_energy = float(props.get("light_energy", light_energy))
	if props.has("light_color"):
		light_color = GodotTrenchIO.to_color(props.get("light_color"))
	omni_range = float(props.get("omni_range", omni_range))
	shadow_enabled = GodotTrenchIO.to_bool(props.get("shadows", shadow_enabled))
	start_on = GodotTrenchIO.to_bool(props.get("start_on", start_on))
	_on = start_on
	# A map built inside the running tree applies properties after _ready.
	if is_node_ready() and not Engine.is_editor_hint():
		visible = _on

func _ready() -> void:
	_on = start_on
	if Engine.is_editor_hint():
		return
	visible = _on

func turn_on() -> void:
	if _on:
		return
	_on = true
	visible = true
	switched.emit(true)

func turn_off() -> void:
	if not _on:
		return
	_on = false
	visible = false
	switched.emit(false)

func toggle() -> void:
	if _on:
		turn_off()
	else:
		turn_on()

func is_on() -> bool:
	return _on
