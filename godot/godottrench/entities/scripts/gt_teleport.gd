@tool
class_name GTTeleport extends GTTrigger
## trigger_teleport: moves entering bodies to the [member destination] entity and turns them to its facing.
## Output: teleported(activator).

signal teleported(activator: Node)

## targetname of an info_teleport_destination or any Node3D entity.
@export var destination := ""
@export var keep_velocity := false

func _func_godot_apply_properties(props: Dictionary) -> void:
	super(props)
	destination = str(props.get("destination", destination))
	keep_velocity = GodotTrenchIO.to_bool(props.get("keep_velocity", keep_velocity))

func _on_triggered(activator: Node) -> void:
	if not activator is Node3D:
		return
	var dest: Node3D = null
	for n in GodotTrenchIO.find_targets(self, destination, activator):
		if n is Node3D:
			dest = n
			break
	if not dest:
		push_warning("[GT] trigger_teleport %s: destination '%s' not found" % [name, destination])
		return
	var body := activator as Node3D
	body.global_position = dest.global_position
	body.global_rotation = Vector3(body.global_rotation.x, dest.global_rotation.y, body.global_rotation.z)
	if not keep_velocity:
		if "velocity" in body:
			body.set("velocity", Vector3.ZERO)
		if body is RigidBody3D:
			(body as RigidBody3D).linear_velocity = Vector3.ZERO
	teleported.emit(body)
