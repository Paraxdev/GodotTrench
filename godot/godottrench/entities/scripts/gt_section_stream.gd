@tool
class_name GTSectionStream extends Area3D
## func_section_stream: one half of a connector room. While the player stands in it, GTSectionStreamer makes sure the
## section it names is the one loaded, swapping sections around the info_landmark both copies of the room share. A
## connector carries two, one per side, and the line where they meet is where the swap happens, so it has to lie
## where neither end of the connector can be seen.

## The section on this side: a scene path, or a name the streamer finds in its sections folder.
@export var section := ""
## targetname of the info_landmark in this connector.
@export var landmark := ""

func _func_godot_apply_properties(props: Dictionary) -> void:
	section = str(props.get("section", section))
	landmark = str(props.get("landmark", landmark))

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	add_to_group(&"gt_section_streams")
	# The streamer asks the physics space which half holds the player, it never monitors bodies.
	monitoring = false
