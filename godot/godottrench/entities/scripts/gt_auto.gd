@tool
class_name GTAuto extends Node3D
## logic_auto: fires map_spawn once the map is loaded and every entity is ready.

signal map_spawn

func _ready() -> void:
	if not Engine.is_editor_hint():
		(func(): map_spawn.emit()).call_deferred()
