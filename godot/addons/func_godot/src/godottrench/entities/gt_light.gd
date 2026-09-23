@tool
class_name GTLight extends OmniLight3D
## light: a switchable omni light. Inputs: turn_on, turn_off, toggle. Output: switched(on).
## fixture names the lamp geometry that follows the light, see GodotTrenchIO.apply_fixture.

signal switched(on: bool)

@export var start_on := true
@export var fixture := ""
@export_enum("hide", "dark") var fixture_off := "hide"

var _on := true

func _func_godot_apply_properties(props: Dictionary) -> void:
	light_energy = float(props.get("light_energy", light_energy))
	if props.has("light_color"):
		light_color = GodotTrenchIO.to_color(props.get("light_color"))
	omni_range = float(props.get("omni_range", omni_range))
	shadow_enabled = GodotTrenchIO.to_bool(props.get("shadows", shadow_enabled))
	start_on = GodotTrenchIO.to_bool(props.get("start_on", start_on))
	fixture = str(props.get("fixture", fixture))
	fixture_off = str(props.get("fixture_off", fixture_off))
	_on = start_on
	# A map built inside the running tree applies properties after _ready.
	if is_node_ready() and not Engine.is_editor_hint():
		visible = _on
		_sync_fixture.call_deferred()

func _ready() -> void:
	_on = start_on
	if Engine.is_editor_hint():
		return
	visible = _on
	# Deferred so fixtures later in the map are in the tree when their names are looked up.
	_sync_fixture.call_deferred()

func _sync_fixture() -> void:
	GodotTrenchIO.apply_fixture(self, fixture, _on, fixture_off)

func turn_on() -> void:
	if _on:
		return
	_on = true
	visible = true
	_sync_fixture()
	switched.emit(true)

func turn_off() -> void:
	if not _on:
		return
	_on = false
	visible = false
	_sync_fixture()
	switched.emit(false)

func toggle() -> void:
	if _on:
		turn_off()
	else:
		turn_on()

func is_on() -> bool:
	return _on
