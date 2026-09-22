@tool
class_name GTPropPhysics extends RigidBody3D
## prop_physics: a throwable, breakable physics prop such as a crate or barrel. Take damage, a hard enough impact or
## a smash input breaks it, and an explosive prop fires a blast when it breaks.
## Inputs: smash, ignite, push(direction), take_damage(amount, source). Outputs: damaged(hp), broken.

signal damaged(hp: float)
signal broken

## res:// scene used for the visual, its collision is ignored in favor of a box from size.
@export var model := ""
## Half extents of the box collision in map units.
@export var size := Vector3(16, 16, 16)
@export var health := 30.0
@export var explosive := false
@export var explosion_radius := 192.0
@export var explosion_damage := 40.0
## Seconds an ignited prop burns before it breaks.
@export var fuse := 0.6
## Breaks when it hits something faster than this, in meters per second, 0 never breaks on impact.
@export var impact_speed := 0.0
## res:// scene spawned where it broke, for debris.
@export var debris_scene := ""

var _hp := 30.0
var _broken := false

func _func_godot_apply_properties(props: Dictionary) -> void:
	model = str(props.get("model", model))
	if props.has("size"):
		size = GodotTrenchIO.to_vector3(props.get("size"))
	health = float(props.get("health", health))
	explosive = GodotTrenchIO.to_bool(props.get("explosive", explosive))
	explosion_radius = float(props.get("explosion_radius", explosion_radius))
	explosion_damage = float(props.get("explosion_damage", explosion_damage))
	fuse = float(props.get("fuse", fuse))
	impact_speed = float(props.get("impact_speed", impact_speed))
	debris_scene = str(props.get("debris_scene", debris_scene))
	mass = float(props.get("mass", mass))
	_hp = health

func _ready() -> void:
	_hp = health
	if Engine.is_editor_hint():
		return
	_build_body()
	if impact_speed > 0.0:
		contact_monitor = true
		max_contacts_reported = 4

func _build_body() -> void:
	var full := (size / GodotTrenchIO.units_per_meter(self)) * 2.0
	var shape := CollisionShape3D.new()
	var box := BoxShape3D.new()
	box.size = full
	shape.shape = box
	add_child(shape)
	if model != "" and ResourceLoader.exists(model):
		var packed := load(model) as PackedScene
		if packed:
			add_child(packed.instantiate())
			return
	var mesh := MeshInstance3D.new()
	var box_mesh := BoxMesh.new()
	box_mesh.size = full
	mesh.mesh = box_mesh
	add_child(mesh)

func _integrate_forces(state: PhysicsDirectBodyState3D) -> void:
	if _broken or impact_speed <= 0.0:
		return
	if state.get_contact_count() > 0 and linear_velocity.length() >= impact_speed:
		smash.call_deferred()

func take_damage(amount: Variant = 0.0, _source: Node = null) -> void:
	if _broken:
		return
	_hp -= _to_float(amount)
	damaged.emit(_hp)
	if _hp <= 0.0:
		smash()

func ignite(_activator: Node = null) -> void:
	if _broken:
		return
	if fuse > 0.0:
		await get_tree().create_timer(fuse).timeout
	if is_inside_tree() and not _broken:
		smash()

func push(direction: Variant = null) -> void:
	var v := GodotTrenchIO.to_vector3(direction) if direction != null else Vector3.ZERO
	apply_central_impulse(v / GodotTrenchIO.units_per_meter(self) * mass)

func smash() -> void:
	if _broken:
		return
	_broken = true
	var parent := get_parent()
	if explosive and parent:
		var boom := GTExplosion.new()
		boom.radius = explosion_radius
		boom.damage = explosion_damage
		parent.add_child(boom)
		boom.global_position = global_position
		boom.explode(self)
		boom.queue_free()
	if debris_scene != "" and ResourceLoader.exists(debris_scene) and parent:
		var packed := load(debris_scene) as PackedScene
		if packed:
			var debris := packed.instantiate()
			parent.add_child(debris)
			if debris is Node3D:
				(debris as Node3D).global_position = global_position
	broken.emit()
	queue_free()

static func _to_float(v: Variant) -> float:
	if v is float or v is int or v is bool:
		return float(v)
	if v is String and (v as String).is_valid_float():
		return float(v)
	return 0.0
