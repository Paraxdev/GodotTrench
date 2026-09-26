@tool
class_name GTPush extends GTTrigger
## trigger_push: jump pads, wind and conveyors. [member push] is a velocity in map units per second.
## With once, entering bodies get the velocity once, otherwise bodies inside keep accelerating towards it.
## Output: pushed(activator), when a body gets its impulse or starts being pushed.

signal pushed(activator: Node)

@export var push := Vector3(0, 384, 0)

func _func_godot_apply_properties(props: Dictionary) -> void:
	super(props)
	push = GodotTrenchIO.to_vector3(props.get("push", push))
	once = GodotTrenchIO.to_bool(props.get("once", true))
	cooldown = 0.0

func velocity() -> Vector3:
	return push / GodotTrenchIO.units_per_meter(self)

func _ready() -> void:
	super()
	# once here means a single impulse per entry, not a trigger that disables itself.
	if not Engine.is_editor_hint():
		set_meta(&"gt_push_impulse", once)
		once = false

func _on_triggered(activator: Node) -> void:
	if get_meta(&"gt_push_impulse", true):
		impulse(activator)
	else:
		pushed.emit(activator)

func impulse(body: Node) -> void:
	var v := velocity()
	if body is RigidBody3D:
		var rb := body as RigidBody3D
		rb.linear_velocity = rb.linear_velocity - rb.linear_velocity.project(v.normalized()) + v
	elif "velocity" in body:
		var current: Vector3 = body.get("velocity")
		body.set("velocity", current - current.project(v.normalized()) + v)
	pushed.emit(body)

func _physics_process(delta: float) -> void:
	if Engine.is_editor_hint() or not enabled or get_meta(&"gt_push_impulse", true):
		return
	var v := velocity()
	for body in bodies_inside():
		if body is RigidBody3D:
			var rb := body as RigidBody3D
			if rb.linear_velocity.dot(v.normalized()) < v.length():
				rb.apply_central_force(v * rb.mass * 4.0)
		elif "velocity" in body:
			var current: Vector3 = body.get("velocity")
			body.set("velocity", current.move_toward(v, v.length() * delta * 4.0))
