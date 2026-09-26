@tool
class_name GTSound extends AudioStreamPlayer3D
## env_sound: a positional sound played through I/O. Inputs: play, stop, toggle. Output: finished.

## res:// audio stream to play.
@export var sound := ""
@export var autoplay_on := false
@export var loop_sound := false

func _func_godot_apply_properties(props: Dictionary) -> void:
	sound = str(props.get("sound", sound))
	volume_db = float(props.get("volume_db", volume_db))
	max_distance = float(props.get("max_distance", max_distance))
	autoplay_on = GodotTrenchIO.to_bool(props.get("autoplay_on", autoplay_on))
	loop_sound = GodotTrenchIO.to_bool(props.get("loop_sound", loop_sound))

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	if sound != "" and ResourceLoader.exists(sound):
		stream = load(sound)
	if loop_sound and not finished.is_connected(_replay):
		finished.connect(_replay)
	if autoplay_on:
		play.call_deferred()

func _replay() -> void:
	if loop_sound and is_inside_tree():
		play()

func toggle() -> void:
	if playing:
		stop()
	else:
		play()
