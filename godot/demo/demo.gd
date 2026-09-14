extends Node3D
## Plays the built showcase maps with a first person player. 1, 2 and 3 switch maps, F3 toggles the I/O debug overlay.
## Build the maps first: godot --headless --path godot --script res://tests/build_showcase.gd

const MAPS := ["res://demo/showcase/mountain_house.scn", "res://demo/showcase/church_school.scn", "res://demo/showcase/lighthouse_forest.scn"]
const PLAYER := preload("res://demo/player.tscn")

@export var start_map := 2

var _level: Node
var _player: DemoPlayer

func _ready() -> void:
	add_child(GodotTrenchDebugOverlay.new())
	load_map(start_map)

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and event.keycode >= KEY_1 and event.keycode < KEY_1 + MAPS.size():
		load_map(event.keycode - KEY_1)

func load_map(index: int) -> void:
	if _level:
		_level.queue_free()
	if _player:
		_player.queue_free()
	var scene := load(MAPS[index]) as PackedScene
	if not scene:
		push_error("%s is missing, run tests/build_showcase.gd first" % MAPS[index])
		return
	_level = scene.instantiate()
	add_child(_level)
	_player = PLAYER.instantiate()
	add_child(_player)
	var spawn := find_spawn(_level)
	if spawn:
		_player.global_transform = Transform3D(Basis(Vector3.UP, spawn.global_rotation.y), spawn.global_position + Vector3.UP * 0.1)

static func find_spawn(root: Node) -> Node3D:
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is Node3D and str(n.name).ends_with("info_player_start"):
			return n
		stack.append_array(n.get_children())
	return null
