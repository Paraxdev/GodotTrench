@tool
class_name GTExplosion extends Node3D
## env_explosion: on the explode input it pushes rigid bodies away and calls take_damage on nodes within radius,
## then optionally spawns an effect scene. Output: exploded.

signal exploded

## Blast radius in map units.
@export var radius := 256.0
@export var damage := 50.0
## Impulse at the center in meters per second, scaled by each body's mass, falling off to the edge.
@export var force := 8.0
@export var damage_method := "take_damage"
## res:// scene spawned at the blast, for a particle or sound effect.
@export var effect_scene := ""

func _func_godot_apply_properties(props: Dictionary) -> void:
	radius = float(props.get("radius", radius))
	damage = float(props.get("damage", damage))
	force = float(props.get("force", force))
	damage_method = str(props.get("damage_method", damage_method))
	effect_scene = str(props.get("effect_scene", effect_scene))

func explode(_activator: Node = null) -> void:
	if not is_inside_tree():
		return
	var meters := radius / GodotTrenchIO.units_per_meter(self)
	var root := GodotTrenchIO.lookup_root(self)
	var origin := global_position
	for node in _nodes_in_radius(_blast_root(), origin, meters):
		var falloff := clampf(1.0 - origin.distance_to(node.global_position) / maxf(meters, 0.001), 0.0, 1.0)
		if node is RigidBody3D:
			var body := node as RigidBody3D
			var dir := body.global_position - origin
			dir = dir.normalized() if dir.length() > 0.001 else Vector3.UP
			body.apply_central_impulse(dir * force * falloff * body.mass)
		if node != self and GodotTrenchIO.resolve_method(node, damage_method) != &"":
			GodotTrenchIO.call_method(node, damage_method, [damage * falloff, self])
	_spawn_effect(root, origin)
	exploded.emit()

## The blast is spatial, so it reaches the whole scene rather than only the map that owns it. A player is usually
## added next to the FuncGodotMap, not under it.
func _blast_root() -> Node:
	var tree := get_tree()
	if tree.current_scene and tree.current_scene.is_ancestor_of(self):
		return tree.current_scene
	return tree.root

func _spawn_effect(root: Node, origin: Vector3) -> void:
	if effect_scene == "" or not ResourceLoader.exists(effect_scene):
		return
	var packed := load(effect_scene) as PackedScene
	if not packed:
		return
	var fx := packed.instantiate()
	root.add_child(fx)
	if fx is Node3D:
		(fx as Node3D).global_position = origin

## Every Node3D at or under [param root] within [param meters] of [param center].
static func _nodes_in_radius(root: Node, center: Vector3, meters: float) -> Array[Node3D]:
	var out: Array[Node3D] = []
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Node3D and (n as Node3D).is_inside_tree() and center.distance_to((n as Node3D).global_position) <= meters:
			out.append(n)
		stack.append_array(n.get_children())
	return out
