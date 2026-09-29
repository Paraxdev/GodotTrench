extends RefCounted
## Section streaming tests, run from run_tests.gd: await load("res://tests/section_tests.gd").new().run(self)

const SETTINGS := "res://demo/demo_map_settings.tres"
const A := "user://gt_section_test_a.scn"
const B := "user://gt_section_test_b.scn"

## The run_tests.gd runner, untyped so its check() is reachable.
var t

func check(cond: bool, what: String) -> void:
	t.check(cond, what)

func run(runner: SceneTree) -> void:
	t = runner
	print("- section streaming across a connector")
	# The same connector in both sections, around a landmark at different spots of each map.
	_save_section(A, Vector3(0, 0, 0), 10)
	_save_section(B, Vector3(1024, 0, 512), 20)
	var player_node := Node3D.new()
	player_node.name = "Player"
	var player_scene := PackedScene.new()
	player_scene.pack(player_node)
	player_node.free()

	var streamer := GTSectionStreamer.new()
	streamer.first_section = A
	streamer.player_scene = player_scene
	t.root.add_child(streamer)
	await t.process_frame
	check(streamer.current_name == A and streamer.player != null, "starts in the first section with the player")
	var scale: float = (load(SETTINGS) as FuncGodotMapSettings).scale_factor
	var start := streamer.player.global_position
	check(start.distance_to(Vector3(-300, 0, -160) * scale) < 0.01, "the player spawns at the first section's info_player_start, got %s" % start)

	# Six units past the swap line into the other half, so a body there would still reach back over the line.
	var past := Vector3(0, 0.05, 6.0 * scale)
	streamer.player.global_position = past
	for i in 30:
		await t.physics_frame
		if streamer.current_name == B:
			break
	check(streamer.current_name == B, "crossing the middle of the connector swaps to the other section")
	var here := streamer._landmark(streamer.current, "lm")
	check(here != null and here.global_position.distance_to(Vector3.ZERO) < 0.001, "the new section's landmark lands where the old one was")
	check(streamer.player.global_position == past, "the player never moves")
	var maps := streamer.get_children().filter(func(c: Node) -> bool: return c is FuncGodotMap)
	check(maps.size() == 1, "only one section is in the tree, got %d" % maps.size())
	check(streamer.swaps == 1, "one swap, not back and forth while the player stands there")

	streamer.player.global_position = Vector3(0, 0.05, -6.0 * scale)
	for i in 30:
		await t.physics_frame
		if streamer.current_name == A:
			break
	check(streamer.current_name == A and streamer.swaps == 2, "turning back swaps to the first section again")
	check(streamer._landmark(streamer.current, "lm").global_position.distance_to(Vector3.ZERO) < 0.001, "lined up the other way round too")

	streamer.sections_dir = "user://"
	check(streamer.section_path("gt_section_test_b") == B, "a short name is found in sections_dir, got %s" % streamer.section_path("gt_section_test_b"))
	streamer.free()
	for path in [A, B]:
		DirAccess.remove_absolute(ProjectSettings.globalize_path(path))

## A map with a floor, a connector around a landmark at [param at] and a player start, built and saved as a scene.
func _save_section(path: String, at: Vector3, first_id: int) -> void:
	var floor_box: Dictionary = t.box_node(first_id, at + Vector3(-512, -16, -512), at + Vector3(512, 0, 512))
	var half := func(id: int, section: String, lo: Vector3, hi: Vector3) -> Dictionary:
		return { "id": id, "type": "entity", "classname": "func_section_stream", "origin": [0.0, 0.0, 0.0], "angles": [0.0, 0.0, 0.0],
			"properties": { "section": section, "landmark": "lm" }, "outputs": [], "children": [t.box_node(id + 1, at + lo, at + hi, "special/trigger")] }
	var landmark := { "id": first_id + 1, "type": "entity", "classname": "info_landmark", "origin": [at.x, at.y, at.z], "angles": [0.0, 0.0, 0.0],
		"properties": { "targetname": "lm" }, "outputs": [] }
	var spawn := { "id": first_id + 2, "type": "entity", "classname": "info_player_start", "origin": [at.x - 300, 0.0, at.z - 160], "angles": [0.0, 0.0, 0.0],
		"properties": {}, "outputs": [] }
	var children := [floor_box, landmark, spawn, half.call(first_id + 3, A, Vector3(-400, 0, -208), Vector3(48, 112, 0)),
		half.call(first_id + 5, B, Vector3(-48, 0, 0), Vector3(400, 112, 208))]
	var map_json := { "format": "godottrench-map", "properties": {}, "layers": [{ "type": "layer", "id": 1, "children": children }] }
	var gtm := OS.get_temp_dir().path_join("gt_section_test.gtm")
	FileAccess.open(gtm, FileAccess.WRITE).store_string(JSON.stringify(map_json))
	var map := FuncGodotMap.new()
	map.name = "FuncGodotMap"
	map.map_settings = load(SETTINGS)
	map.local_map_file = gtm
	t.root.add_child(map)
	map.build()
	_own(map, map)
	var packed := PackedScene.new()
	packed.pack(map)
	ResourceSaver.save(packed, path)
	map.free()
	DirAccess.remove_absolute(gtm)

func _own(node: Node, owner_node: Node) -> void:
	for child in node.get_children():
		child.owner = owner_node
		if child.scene_file_path == "":
			_own(child, owner_node)
