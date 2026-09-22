class_name GTSpawnLogic extends RefCounted
## Shared spawning for info_spawner and trigger_spawn_area: instancing, limits, ground snapping and death tracking.

var host: Node3D
var scene: PackedScene
var count := 1
var max_alive := 5
## 0 is unlimited.
var total := 0
var spawn_group := "enemies"
var snap_to_ground := true
var alive: Array[Node] = []
var spawned_total := 0
var _exhausted_sent := false

signal spawned(node: Node)
signal all_dead
signal exhausted

## The spawn settings of a host entity (info_spawner, trigger_spawn_area), read from its exported members so a
## saved scene configures the same as a fresh build.
static func settings_of(from: Node) -> Dictionary:
	return {
		"scene": from.get("scene"),
		"count": from.get("count"),
		"max_alive": from.get("max_alive"),
		"total": from.get("total"),
		"spawn_group": from.get("spawn_group"),
		"snap_to_ground": from.get("snap_to_ground"),
	}

func configure(from: Node3D, props: Dictionary) -> void:
	host = from
	var path := str(props.get("scene", ""))
	scene = load(path) as PackedScene if path != "" and ResourceLoader.exists(path) else null
	if path != "" and not scene:
		push_warning("[GT] %s: scene %s not found" % [from.name, path])
	count = int(props.get("count", count))
	max_alive = int(props.get("max_alive", max_alive))
	total = int(props.get("total", total))
	spawn_group = str(props.get("spawn_group", spawn_group))
	snap_to_ground = GodotTrenchIO.to_bool(props.get("snap_to_ground", snap_to_ground))

func is_exhausted() -> bool:
	return total > 0 and spawned_total >= total

## Spawns up to [member count] instances at points from [param point_at] (a Callable returning a global Vector3).
func spawn(point_at: Callable) -> Array[Node]:
	var out: Array[Node] = []
	if not scene or not host or not host.is_inside_tree():
		return out
	alive = alive.filter(func(n): return is_instance_valid(n))
	for _i in count:
		if alive.size() >= max_alive or is_exhausted():
			break
		var node := scene.instantiate()
		var parent := host.get_parent() if host.get_parent() else host
		parent.add_child(node)
		if node is Node3D:
			var p: Vector3 = point_at.call()
			if snap_to_ground:
				p = ground(p)
			(node as Node3D).global_position = p
		if spawn_group != "":
			node.add_to_group(StringName(spawn_group))
		node.tree_exiting.connect(_on_gone.bind(node))
		alive.append(node)
		spawned_total += 1
		out.append(node)
		spawned.emit(node)
	if is_exhausted() and not _exhausted_sent:
		_exhausted_sent = true
		exhausted.emit()
	return out

func kill_all() -> void:
	for n in alive:
		if is_instance_valid(n):
			n.queue_free()

func _on_gone(node: Node) -> void:
	alive.erase(node)
	if alive.is_empty() and spawned_total > 0:
		all_dead.emit()

## First floor below a point, or the point itself.
func ground(p: Vector3) -> Vector3:
	var space := host.get_world_3d().direct_space_state if host.is_inside_tree() else null
	if not space:
		return p
	var query := PhysicsRayQueryParameters3D.create(p + Vector3.UP * 0.5, p + Vector3.DOWN * 64.0)
	if host is CollisionObject3D:
		query.exclude = [(host as CollisionObject3D).get_rid()]
	var hit := space.intersect_ray(query)
	return hit["position"] if not hit.is_empty() else p
