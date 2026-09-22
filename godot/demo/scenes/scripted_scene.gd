extends Node3D
## A small playable cutscene built from res://demo/maps/scripted_scene.gtm. Walk forward into the trigger and the
## scene plays itself through entity I/O: a guide spawns and walks a path, text appears, an explosive barrel goes off
## and hurts you, a light switches on and the exit opens. WASD to move, mouse to look, F3 toggles the I/O overlay.

const PLAYER := preload("res://demo/player.tscn")
const MAP := "res://demo/maps/scripted_scene.gtm"
const SETTINGS := "res://demo/demo_map_settings.tres"

func _ready() -> void:
	add_child(GodotTrenchDebugOverlay.new())
	var map := FuncGodotMap.new()
	map.map_settings = load(SETTINGS)
	map.local_map_file = MAP
	add_child(map)
	map.build()
	var player := PLAYER.instantiate()
	add_child(player)
	var spawn := find_spawn(map)
	if spawn:
		player.global_transform = Transform3D(Basis(Vector3.UP, spawn.global_rotation.y), spawn.global_position + Vector3.UP * 0.1)

static func find_spawn(root: Node) -> Node3D:
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Node3D and str(n.name).ends_with("info_player_start"):
			return n
		stack.append_array(n.get_children())
	return null
