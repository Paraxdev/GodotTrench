extends RefCounted
## GodotTrenchNav: two rooms joined by a doorway with a closed func_door, and a func_detail clip wall that cuts off the
## far end of the second room.
##
## res://tests/run_tests.gd calls [method run], [param t] is that script's instance for its check helpers.

const SETTINGS := "res://demo/demo_map_settings.tres"

static func run(t) -> void:
	print("- navigation bake")
	_test_profile(t)
	var path := OS.get_temp_dir().path_join("gt_nav_test.gtm")
	FileAccess.open(path, FileAccess.WRITE).store_string(JSON.stringify(_map_json(t)))
	var level := Node3D.new()
	t.root.add_child(level)
	var map := FuncGodotMap.new()
	map.map_settings = load(SETTINGS)
	map.global_map_file = path
	map.position = Vector3(100, 0, 0)
	level.add_child(map)
	map.build()
	await _test_bake(t, map)
	await _test_region(t, map)
	level.queue_free()
	DirAccess.remove_absolute(path)
	await t.process_frame

static func _test_profile(t) -> void:
	var nav := GodotTrenchNav.profile(0.25, 1.75, 0.35)
	t.check(t.near(nav.agent_radius, 0.3) and t.near(nav.agent_height, 1.8) and t.near(nav.agent_max_climb, 0.3), "agent sizes snap to the 0.1 m cells like the bake would, got %s %s %s" % [nav.agent_radius, nav.agent_height, nav.agent_max_climb])

static func _map_json(t) -> Dictionary:
	var world := [
		t.box_node(2, Vector3(-256, -32, -128), Vector3(256, 0, 128)),
		t.box_node(3, Vector3(-16, 0, -128), Vector3(16, 128, -32)),
		t.box_node(4, Vector3(-16, 0, 32), Vector3(16, 128, 128)),
		t.box_node(5, Vector3(-16, 96, -32), Vector3(16, 128, 32)),
	]
	var door := { "type": "entity", "id": 10, "classname": "func_door", "properties": { "targetname": "door" }, "children": [t.box_node(11, Vector3(-8, 0, -32), Vector3(8, 96, 32))] }
	var clip := { "type": "entity", "id": 20, "classname": "func_detail", "properties": { "targetname": "clip_wall" }, "children": [t.box_node(21, Vector3(150, 0, -128), Vector3(170, 128, 128), "special/clip")] }
	var trigger := { "type": "entity", "id": 30, "classname": "trigger_once", "properties": {}, "children": [t.box_node(31, Vector3(-200, 0, -100), Vector3(-100, 64, 100))] }
	return { "format": "godottrench-map", "properties": {}, "layers": [{ "type": "layer", "id": 1, "children": world + [door, clip, trigger] }] }

static func _test_bake(t, map: FuncGodotMap) -> void:
	var own := NavigationMesh.new()
	own.agent_radius = 0.42
	GodotTrenchNav.bake(map, own)
	t.check(t.near(own.agent_radius, 0.42) and own.get_polygon_count() == 0, "the profile passed in is not changed")
	var nav := GodotTrenchNav.bake(map)
	t.check(nav and nav.get_polygon_count() > 0, "a built map bakes")
	if not nav:
		return
	var bounds := AABB(nav.vertices[0], Vector3.ZERO)
	for v in nav.vertices:
		bounds = bounds.expand(v)
	t.check(bounds.position.x > -8.1 and bounds.end.x < 8.1, "the mesh is in the map's local space, spans %s" % bounds)
	var room_a := Vector3(-6, 0, 0)
	var room_b := Vector3(3, 0, 2)
	var past_clip := Vector3(7, 0, 0)
	t.check(_same_island(nav, room_a, room_b), "the closed door leaves the doorway open")
	t.check(not _same_island(nav, room_b, past_clip), "the clip wall cuts the far end off")
	var islands := GodotTrenchNav.islands(nav)
	t.check(islands.size() >= 2 and islands[0].size() >= islands[1].size() and _covers(nav, islands[0], room_a), "the biggest island comes first")
	var door := t.find_named(map, "entity_door") as AnimatableBody3D
	t.check(door and door.collision_layer == 1 and door.get_groups().is_empty(), "the door keeps its collision and gets no groups")
	var clip: Node = t.find_named(map, "entity_clip_wall")
	clip.add_to_group(GodotTrenchNav.IGNORE_GROUP)
	t.check(_same_island(GodotTrenchNav.bake(map), room_b, past_clip), "a body in the ignore group is left out")
	clip.remove_from_group(GodotTrenchNav.IGNORE_GROUP)

static func _test_region(t, map: FuncGodotMap) -> void:
	var agent := GodotTrenchNav.profile(0.4)
	var first := await GodotTrenchNav.bake_async(map, agent)
	var cached := Time.get_ticks_usec()
	var again := await GodotTrenchNav.bake_async(map, agent)
	cached = Time.get_ticks_usec() - cached
	t.check(first and again and again != first and again.get_polygon_count() == first.get_polygon_count(), "the second bake of the same map loads from the cache")
	t.check(cached < 1_000_000, "a cache load is quick, took %d us" % cached)
	var region := await GodotTrenchNav.bake_region(map, agent)
	t.check(region and region.get_parent() == map and region.name == GodotTrenchNav.REGION_NAME, "bake_region adds a navigation region under the map")
	if not region:
		return
	t.check(t.near(NavigationServer3D.map_get_cell_size(region.get_navigation_map()), 0.1), "the navigation map takes the bake's cell size")
	var from := map.to_global(Vector3(-6, 0, 0))
	var to := map.to_global(Vector3(3, 0, 2))
	var route := GodotTrenchNav.path(map, from, to)
	t.check(route.size() > 1 and route[route.size() - 1].distance_to(to) < 0.5, "a path goes through the closed door, ends at %s" % [route[route.size() - 1] if route.size() > 0 else null])
	var blocked := GodotTrenchNav.path(map, from, map.to_global(Vector3(7, 0, 0)))
	t.check(blocked.size() == 0 or blocked[blocked.size() - 1].x < map.to_global(Vector3(5, 0, 0)).x, "no path crosses the clip wall")
	t.check(await GodotTrenchNav.bake_region(map, agent) == region, "a second bake reuses the region")

static func _same_island(nav: NavigationMesh, a: Vector3, b: Vector3) -> bool:
	for island in GodotTrenchNav.islands(nav):
		if _covers(nav, island, a) and _covers(nav, island, b):
			return true
	return false

static func _covers(nav: NavigationMesh, polygons: PackedInt32Array, p: Vector3) -> bool:
	for polygon in polygons:
		var outline := PackedVector2Array()
		var height := 0.0
		for v in nav.get_polygon(polygon):
			outline.append(Vector2(nav.vertices[v].x, nav.vertices[v].z))
			height = nav.vertices[v].y
		if absf(height - p.y) < 0.5 and Geometry2D.is_point_in_polygon(Vector2(p.x, p.z), outline):
			return true
	return false
