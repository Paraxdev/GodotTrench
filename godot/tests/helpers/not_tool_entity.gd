extends Node3D
## A test entity script without @tool, which the Godot editor runs as a placeholder.

func _func_godot_apply_properties(_properties: Dictionary) -> void:
	set_meta(&"applied", true)
