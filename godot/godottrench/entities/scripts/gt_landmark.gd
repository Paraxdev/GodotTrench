@tool
class_name GTLandmark extends Node3D
## info_landmark: the shared point of a connector room between two streamed sections. Both sections hold an identical
## copy of the room with a landmark of the same targetname at the same spot, and GTSectionStreamer lines the next
## section up so the two landmarks coincide.

@export var targetname := ""

func _func_godot_apply_properties(props: Dictionary) -> void:
	targetname = str(props.get("targetname", targetname))
