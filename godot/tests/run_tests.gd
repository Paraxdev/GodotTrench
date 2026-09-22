extends SceneTree
## Headless tests for the GodotTrench fork of FuncGodot.
## godot --headless --path godot --script res://tests/run_tests.gd

const MAP := "res://tests/maps/basic.gtm"
const SETTINGS := "res://demo/demo_map_settings.tres"
const EPS := 0.001

var failures: Array[String] = []
var checks := 0

func check(cond: bool, what: String) -> void:
	checks += 1
	if not cond:
		failures.append(what)
		printerr("  FAIL: ", what)

func near(a: Variant, b: Variant, eps := EPS) -> bool:
	if a is Vector3:
		return (a - b).length() <= eps
	return absf(float(a) - float(b)) <= eps

func find_named(root: Node, name_part: String) -> Node:
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if String(n.name) == name_part:
			return n
		stack.append_array(n.get_children())
	return null

func collect(root: Node, predicate: Callable) -> Array[Node]:
	var out: Array[Node] = []
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if predicate.call(n):
			out.append(n)
		stack.append_array(n.get_children())
	return out

func build_map() -> FuncGodotMap:
	var map := FuncGodotMap.new()
	map.name = "TestMap"
	map.map_settings = load(SETTINGS)
	map.local_map_file = MAP
	root.add_child(map)
	map.build()
	return map

func _initialize() -> void:
	print("GodotTrench FuncGodot fork tests")
	await process_frame
	await test_parser()
	await test_build()
	await test_terrain()
	await test_meshes_terrain_props()
	test_bbmodel()
	await test_map_export()
	await test_game_config()
	await test_io_targets()
	await test_gameplay_entities()
	await test_spawner()
	await test_light()
	await test_text()
	await test_animate()
	await test_prop_and_explosion()
	await test_npc_path()
	await test_sequence()
	await test_logic_script()
	await test_more_entities()
	await test_scripted_scene()
	test_scatter_and_blend()
	test_face_cull()
	test_interior_face_culling()
	test_chunk_streamer()
	test_csharp_entities()
	await test_live_session()
	test_live_link_lines()
	test_threaded_build_matches()
	await test_showcase_playthrough()
	print("%d checks, %d failures" % [checks, failures.size()])
	quit(1 if failures.size() > 0 else 0)

func test_parser() -> void:
	print("- parser")
	var parse_data := FuncGodotData.ParseData.new()
	var text := FileAccess.get_file_as_string(MAP)
	var result := GodotTrenchParser.parse(text, load(SETTINGS), parse_data, MAP)
	check(result != null, "parse returns data")
	var world: FuncGodotData.EntityData = result.entities[0]
	check(world.properties["classname"] == "worldspawn", "entity 0 is worldspawn")
	# floor + 3 wall pieces + 2 props + prefab pillar. The omitted layer brush must not be there.
	check(world.brushes.size() == 7, "world brush count is 7, got %d" % world.brushes.size())
	for b in world.brushes:
		check(b.exact, "world brushes are exact")
		for f in b.faces:
			for v in f.exact_vertices:
				check(v.length() < 1000.0 / 32.0 + 40.0, "omitted layer geometry excluded")
	var classnames := result.entities.map(func(e): return e.properties["classname"])
	check(classnames.count("light") == 2, "two lights including the prefab one")
	check(result.groups.any(func(g): return g.name.begins_with("group_") and g.name.ends_with("props")), "props group parsed")
	var prefab_light = result.entities.filter(func(e): return e.properties.get("targetname", "") == "p1-plight")
	check(prefab_light.size() == 1, "prefab targetname gets fixup prefix")

	# Face planes must point out of their brush.
	var floor_brush: FuncGodotData.BrushData = world.brushes[0]
	var center := Vector3.ZERO
	var count := 0
	for f in floor_brush.faces:
		for v in f.exact_vertices:
			center += v
			count += 1
	center /= count
	for f in floor_brush.faces:
		check(f.plane.distance_to(center) < 0.0, "face plane faces outwards")

func test_build() -> void:
	print("- build")
	var map := build_map()
	await process_frame

	var world := find_named(map, "entity_0_worldspawn")
	check(world != null, "worldspawn node built")
	if world:
		var meshes := collect(world, func(n): return n is MeshInstance3D)
		check(meshes.size() == 1, "worldspawn has one mesh instance")
		if meshes.size() == 1:
			var aabb: AABB = meshes[0].get_aabb()
			check(near(aabb.position.x, -8.0) and near(aabb.end.x, 8.0), "world mesh x extent is floor size, got %s" % aabb)
			check(near(aabb.position.y, -0.5), "world mesh bottom at -0.5 m")
			check(aabb.end.z <= 8.0 + EPS, "omitted layer excluded from mesh")
			var mesh: ArrayMesh = meshes[0].mesh
			var names := []
			for i in mesh.get_surface_count():
				names.append(mesh.surface_get_name(i))
			check(names.has("base/floor") and names.has("base/wall") and names.has("base/metal"), "surfaces per material, got %s" % [names])

	var door := find_named(map, "entity_door1")
	check(door != null and door is GTDoor, "door built with the addon door script")
	if door:
		check(near(door.position, Vector3(0, 1.5, -7.75)), "door origin at brush center, got %s" % door.position)
		var shapes := collect(door, func(n): return n is CollisionShape3D)
		check(shapes.size() == 1 and shapes[0].shape is ConvexPolygonShape3D, "door has convex collision")
		var outputs := collect(door, func(n): return n is GodotTrenchOutput)
		check(outputs.size() == 1 and outputs[0].target == "lamp", "door output relay created")

	var lamp := find_named(map, "entity_lamp")
	check(lamp != null and lamp is OmniLight3D, "lamp built")
	if lamp:
		check(near(lamp.position, Vector3(0, 3.5, 0)), "lamp position converted, got %s" % lamp.position)
		check(near(lamp.rotation_degrees.y, 90.0, 0.01), "lamp yaw 90, got %s" % lamp.rotation_degrees)
		check(near(lamp.light_energy, 2.5), "light_energy applied")
		check(not lamp.visible, "lamp starts off")

	var start := collect(map, func(n): return n is Marker3D)
	check(start.size() == 1, "player start built")
	if start.size() == 1:
		check(near(start[0].position, Vector3(0, 0, 4)), "player start position")
		check(near(start[0].rotation_degrees.y, 45.0, 0.01), "player start yaw 45, got %s" % start[0].rotation_degrees)

	var plight := find_named(map, "entity_p1-plight")
	check(plight != null, "prefab light instanced")
	if plight:
		check(near(plight.position, Vector3(6.25, 2.0, -3.0)), "prefab light transformed, got %s" % plight.position)

	var trigger := collect(map, func(n): return n is Area3D)
	check(trigger.size() == 1, "trigger area built")
	if trigger.size() == 1:
		check(collect(trigger[0], func(n): return n is MeshInstance3D).is_empty(), "trigger has no visuals")

	# I/O chain: button pressed -> door1 open (0.1 s delay) -> door opened -> lamp turn_on.
	var button := find_named(map, "entity_button1")
	check(button != null and button is GTButton, "button built")
	if button and door and lamp:
		check(button.is_connected("pressed", Callable(collect(button, func(n): return n is GodotTrenchOutput)[0], "fire")), "button signal connected")
		button.press()
		check(not door.is_open, "delay holds the input back")
		await create_timer(0.2).timeout
		check(door.is_open, "door opened by button output")
		var deadline := Time.get_ticks_msec() + 5000
		while not lamp.visible and Time.get_ticks_msec() < deadline:
			await process_frame
		check(lamp.visible, "lamp turned on by door opened output")
	map.queue_free()
	await process_frame

func test_terrain() -> void:
	print("- displacements and vertex paint")
	var map := FuncGodotMap.new()
	map.map_settings = load(SETTINGS)
	map.local_map_file = "res://tests/maps/terrain.gtm"
	root.add_child(map)
	map.build()
	await process_frame
	var world := find_named(map, "entity_0_worldspawn")
	check(world != null, "terrain world built")
	if not world:
		return
	var meshes := collect(world, func(n): return n is MeshInstance3D)
	check(meshes.size() == 1, "one world mesh")
	if meshes.size() == 1:
		var mesh: ArrayMesh = meshes[0].mesh
		var aabb: AABB = mesh.get_aabb()
		check(near(aabb.end.y, 2.0, 0.01), "displacement peak at 64 units = 2 m, got %s" % aabb)
		check(near(aabb.position.y, 0.0, 0.01), "displacement brush sides are not rendered, got %s" % aabb)
		var floor_surface := -1
		var metal_surface := -1
		for i in mesh.get_surface_count():
			if mesh.surface_get_name(i) == "base/floor":
				floor_surface = i
			elif mesh.surface_get_name(i) == "base/metal":
				metal_surface = i
		check(floor_surface >= 0 and metal_surface >= 0, "terrain and cube surfaces")
		if floor_surface >= 0:
			var arrays := mesh.surface_get_arrays(floor_surface)
			check(arrays[Mesh.ARRAY_VERTEX].size() == 81, "power 3 grid has 81 vertices, got %d" % arrays[Mesh.ARRAY_VERTEX].size())
			check(arrays[Mesh.ARRAY_INDEX].size() == 128 * 3, "64 grid quads = 128 triangles, got %d indices" % arrays[Mesh.ARRAY_INDEX].size())
			var colors: PackedColorArray = arrays[Mesh.ARRAY_COLOR]
			check(colors.size() == 81 and colors.to_byte_array().size() > 0, "blend weights stored as vertex colors")
			var painted := 0
			for c in colors:
				if c.a > 0.5:
					painted += 1
			check(painted > 0 and painted < 81, "some vertices carry blend alpha, got %d" % painted)
			# Front faces must point up: Godot winds clockwise, so the geometric normal of the first triangle is +Y when reversed.
			var verts: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
			var idx: PackedInt32Array = arrays[Mesh.ARRAY_INDEX]
			var n := (verts[idx[1]] - verts[idx[0]]).cross(verts[idx[2]] - verts[idx[0]])
			check(n.y < 0.0, "triangles are clockwise when seen from above")
		if metal_surface >= 0:
			var colors: PackedColorArray = mesh.surface_get_arrays(metal_surface)[Mesh.ARRAY_COLOR]
			check(colors.size() == 24, "cube surface has colors")
			var red := 0
			for c in colors:
				if c.r > 0.9 and c.g < 0.1:
					red += 1
			check(red == 3, "painted corner is red on its three faces, got %d" % red)
	var shapes := collect(world, func(n): return n is CollisionShape3D)
	var concave := shapes.filter(func(s): return s.shape is ConcavePolygonShape3D)
	var convex := shapes.filter(func(s): return s.shape is ConvexPolygonShape3D)
	check(concave.size() == 1, "displacement collides as trimesh")
	check(convex.size() == 1, "cube keeps its convex hull and the displacement brush gets none")
	if concave.size() == 1:
		var top := -INF
		for v in concave[0].shape.get_faces():
			top = maxf(top, v.y)
		check(near(top, 2.0, 0.01), "collision follows the sculpted surface")
	var decals := collect(map, func(n): return n is Decal)
	check(decals.size() == 1, "decal entity built")
	if decals.size() == 1:
		var decal: Decal = decals[0]
		check(decal is GodotTrenchDecal, "decal uses the GodotTrench decal script")
		check(decal.texture_albedo != null, "decal texture loaded from res:// path")
		check(near(decal.size, Vector3(3, 2, 3)), "decal size converted to meters, got %s" % decal.size)
		check(near(decal.position, Vector3(4, 2, 4)), "decal position, got %s" % decal.position)
	map.queue_free()
	await process_frame

func test_meshes_terrain_props() -> void:
	print("- meshes, heightmap terrain and model props")
	var map := FuncGodotMap.new()
	map.map_settings = load(SETTINGS)
	map.local_map_file = "res://tests/maps/geometry.gtm"
	root.add_child(map)
	map.build()
	await process_frame
	var world := find_named(map, "entity_0_worldspawn")
	check(world != null, "geometry world built")
	if world:
		var meshes := collect(world, func(n): return n is MeshInstance3D)
		check(meshes.size() == 1, "one world mesh for both meshes")
		if meshes.size() == 1:
			var mesh: ArrayMesh = meshes[0].mesh
			var aabb := mesh.get_aabb()
			check(near(aabb.end.y, 3.0, 0.01), "extruded mesh top at 96 units = 3 m, got %s" % aabb)
			var wall := -1
			var metal := -1
			for i in mesh.get_surface_count():
				if mesh.surface_get_name(i) == "base/wall":
					wall = i
				elif mesh.surface_get_name(i) == "base/metal":
					metal = i
			check(wall >= 0 and metal >= 0, "mesh surfaces per material")
			if wall >= 0:
				var arrays := mesh.surface_get_arrays(wall)
				# 10 quads after the extrusion, 2 triangles each.
				check(arrays[Mesh.ARRAY_INDEX].size() == 60, "extruded cube has 20 triangles, got %d indices" % arrays[Mesh.ARRAY_INDEX].size())
				var verts: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
				var idx: PackedInt32Array = arrays[Mesh.ARRAY_INDEX]
				var outward := 0
				var center := Vector3(1, 1.5, 1)
				for t in range(0, idx.size(), 3):
					var a := verts[idx[t]]
					var n := (verts[idx[t + 2]] - a).cross(verts[idx[t + 1]] - a)
					if n.dot((a + verts[idx[t + 1]] + verts[idx[t + 2]]) / 3.0 - center) > 0.0:
						outward += 1
				check(outward == idx.size() / 3, "mesh triangles face outwards (clockwise), %d of %d" % [outward, idx.size() / 3])
			if metal >= 0:
				var arrays := mesh.surface_get_arrays(metal)
				var normals: PackedVector3Array = arrays[Mesh.ARRAY_NORMAL]
				var smooth := 0
				for n in normals:
					if absf(n.y) < 0.01 and absf(n.x) > 0.05 and absf(n.x) < 0.98:
						smooth += 1
				check(smooth > 10, "cylinder sides have smooth normals, got %d" % smooth)
				var uvs: PackedVector2Array = arrays[Mesh.ARRAY_TEX_UV]
				check(uvs.has(Vector2(1, 1)), "explicit mesh uvs are used")
		var concave := collect(world, func(n): return n is CollisionShape3D and n.shape is ConcavePolygonShape3D)
		check(concave.size() == 1, "meshes collide as a trimesh")

	var terrains := collect(map, func(n): return n is GodotTrenchTerrain)
	check(terrains.size() == 1, "terrain node built")
	if terrains.size() == 1:
		var t: GodotTrenchTerrain = terrains[0]
		check(near(t.position, Vector3(-10, -0.25, -10)), "terrain origin in meters, got %s" % t.position)
		check(t.resolution == Vector2i(17, 17), "terrain resolution")
		var chunks := collect(t, func(n): return n is MeshInstance3D)
		check(chunks.size() == 1, "one chunk for a small terrain")
		var shapes := collect(t, func(n): return n is CollisionShape3D and n.shape is HeightMapShape3D)
		check(shapes.size() == 1, "terrain has a heightmap collision shape")
		check(near(t.height_at(10.0, 10.0), 5.0, 0.01), "center height is 160 units = 5 m, got %s" % t.height_at(10.0, 10.0))
		if chunks.size() == 1:
			var arrays: Array = (chunks[0] as MeshInstance3D).mesh.surface_get_arrays(0)
			var colors: PackedColorArray = arrays[Mesh.ARRAY_COLOR]
			check(colors[8 * 17 + 8].g > 0.9, "painted layer weight at the center")
			var mat := (chunks[0] as MeshInstance3D).mesh.surface_get_material(0) as ShaderMaterial
			check(mat != null and mat.get_shader_parameter("layer1") != null, "terrain blend material has layer textures")
			check(mat != null and near(mat.get_shader_parameter("map_offset"), t.position), "terrain textures are projected in map space")
		# Physics query onto the heightmap.
		await physics_frame
		await physics_frame
		var space := (t as Node3D).get_world_3d().direct_space_state
		var hit := space.intersect_ray(PhysicsRayQueryParameters3D.create(Vector3(0, 50, 0), Vector3(0, -50, 0)))
		check(not hit.is_empty() and near(hit["position"].y, 4.75, 0.05), "ray hits the terrain at the center height, got %s" % hit)

	var props := collect(map, func(n): return n is GodotTrenchProp)
	check(props.size() == 1, "prop entity built")
	if props.size() == 1:
		var prop: GodotTrenchProp = props[0]
		check(collect(prop, func(n): return n is MeshInstance3D).size() == 1, "bbmodel scene instanced under the prop")
		check(collect(prop, func(n): return n is CollisionShape3D).size() == 1, "prop has convex collision")
		check(near(prop.position, Vector3(8, 0, 0)), "prop position, got %s" % prop.position)

	var envs := collect(map, func(n): return n is WorldEnvironment)
	check(envs.size() == 1, "worldspawn sky keys build a WorldEnvironment")
	if envs.size() == 1:
		var env: Environment = envs[0].environment
		check(env.fog_enabled and near(env.fog_density, 0.004, 0.00001), "fog density from worldspawn, got %s" % env.fog_density)
		var sky := env.sky.sky_material as ProceduralSkyMaterial
		check(sky != null and near(sky.sky_top_color.r, 0.2, 0.01), "sky top color from worldspawn")
	var suns := collect(map, func(n): return n is DirectionalLight3D)
	check(suns.size() == 1 and near(suns[0].rotation_degrees, Vector3(-30, 60, 0), 0.01), "sun rotation from sun_angles")
	map.queue_free()
	await process_frame

func test_bbmodel() -> void:
	print("- Blockbench importer")
	var text := FileAccess.get_file_as_string("res://demo/models/pine.bbmodel")
	var model := GodotTrenchBBModel.parse(text)
	check(model.has("polygons") and model["polygons"].size() > 20, "pine parsed into polygons")
	check((model["textures"][0]["image"] as Image).get_width() == 32, "embedded texture decoded")
	var scene := GodotTrenchBBModel.build_scene(model, 1.0 / 16.0, "trimesh")
	var mi: MeshInstance3D = scene.get_node("mesh")
	var aabb := mi.mesh.get_aabb()
	check(near(aabb.end.y, 11.0, 0.01), "pine is 176 units = 11 m tall, got %s" % aabb)
	check(scene.get_node("collision/shape").shape is ConcavePolygonShape3D, "trimesh collision option")
	var imported := load("res://demo/models/pine.bbmodel")
	check(imported is PackedScene, "editor import produces a PackedScene")
	scene.free()

func world_mesh_aabb(file: String) -> AABB:
	var map := FuncGodotMap.new()
	map.map_settings = load(SETTINGS)
	map.local_map_file = file
	root.add_child(map)
	map.build()
	var world := find_named(map, "entity_0_worldspawn")
	var aabb := AABB()
	if world:
		for m in collect(world, func(n): return n is MeshInstance3D):
			aabb = m.get_aabb()
	var names := collect(map, func(n): return n.name.begins_with("entity_")).map(func(n): return String(n.name))
	map.set_meta("names", names)
	map.free()
	return aabb

func test_map_export() -> void:
	print("- .map export read by upstream FuncGodot")
	var from_gtm := world_mesh_aabb(MAP)
	var from_map := world_mesh_aabb("res://tests/maps/basic_export.map")
	# The prefab is not part of the .map, it sits inside the floor bounds so the extents still match.
	check(from_map.size.length() > 1.0, "exported .map builds world geometry")
	check(near(from_map.position, from_gtm.position, 0.01) and near(from_map.end, from_gtm.end, 0.01), "world bounds match: map %s vs gtm %s" % [from_map, from_gtm])

func test_game_config() -> void:
	print("- game config export")
	var config: GodotTrenchGameConfig = load("res://demo/demo_game_config.tres")
	config.output_path = "user://godottrench_game_test.json"
	check(config.export_file() == OK, "export succeeds")
	var data = JSON.parse_string(FileAccess.get_file_as_string(config.output_path))
	check(data is Dictionary, "exported json parses")
	if not data is Dictionary:
		return
	check(data["units_per_meter"] == 32.0, "units per meter exported")
	check(data["tool_textures"]["clip"] == "special/clip", "tool textures exported")
	var by_name := {}
	for e in data["entities"]:
		by_name[e["classname"]] = e
	check(by_name.has("func_door") and by_name["func_door"]["type"] == "solid", "func_door exported as solid")
	check(by_name.has("light") and by_name["light"]["type"] == "point", "light exported as point")
	if by_name.has("func_door"):
		var outs: Array = by_name["func_door"]["outputs"].map(func(o): return o["name"])
		var ins: Array = by_name["func_door"]["inputs"].map(func(o): return o["name"])
		check(outs.has("opened") and outs.has("closed"), "door outputs from script signals, got %s" % [outs])
		check(ins.has("open") and ins.has("toggle"), "door inputs from script methods, got %s" % [ins])
		var props: Array = by_name["func_door"]["properties"]
		check(props.any(func(p): return p["name"] == "targetname" and p["type"] == "target_source"), "targetname typed as target_source")
	if by_name.has("trigger_area"):
		var outs: Array = by_name["trigger_area"]["outputs"].map(func(o): return o["name"])
		check(outs.has("body_entered"), "Area3D signals exported, got %s" % [outs])
	if by_name.has("info_player_start"):
		var size: Array = by_name["info_player_start"]["size"]
		check(size[0] == [-16.0, 0.0, -16.0] and size[1] == [16.0, 56.0, 16.0], "sizes converted to Y-up, got %s" % [size])

func test_io_targets() -> void:
	print("- I/O targets, arguments and C# style methods")
	var map := Node3D.new()
	map.name = "IoMap"
	root.add_child(map)
	var receiver: Node3D = load("res://tests/helpers/io_receiver.gd").new()
	receiver.name = "Game"
	receiver.add_to_group(&"scorekeepers")
	map.add_child(receiver)
	var relay := GTRelay.new()
	relay.set_meta(GodotTrenchIO.TARGETNAME_META, "relay_1")
	map.add_child(relay)
	await process_frame
	check(GodotTrenchIO.find_targets(relay, "@scorekeepers", null) == [receiver], "@group targets")
	check(GodotTrenchIO.find_targets(relay, "/root/IoMap/Game", null) == [receiver], "absolute node path targets")
	check(GodotTrenchIO.find_targets(relay, "relay_*", null) == [relay], "wildcard targetnames")
	var events: Array = []
	var on_event := func(s, o, t, i, p): events.append([o, t, i])
	GodotTrenchIO.events().fired.connect(on_event)
	GodotTrenchIO.invoke(receiver, &"add_score", "[10, \"$caller_name\"]", null)
	check(receiver.calls.back() == ["add_score", 10, "Game"], "JSON array parameter spreads into arguments, got %s" % [receiver.calls])
	GodotTrenchIO.invoke(receiver, &"open_vault", "1234", null)
	check(receiver.calls.back() == ["OpenVault", "1234"], "snake_case input finds the PascalCase (C#) method")
	var call := GTLogicCall.new()
	call.call_target = "@scorekeepers"
	call.method = "add_score"
	call.arguments = "[\"$parameter\", \"bonus\"]"
	map.add_child(call)
	var results: Array = []
	call.called.connect(func(r): results.append(r))
	call.call_with(7)
	check(receiver.calls.back() == ["add_score", 7, "bonus"] and results == [14], "logic_call passes the parameter and returns the result")
	var overlay := GodotTrenchDebugOverlay.new()
	root.add_child(overlay)
	var output := GodotTrenchOutput.new()
	output.output = &"triggered"
	output.target = "/root/IoMap/Game"
	output.input = &"add_score"
	output.parameter = "[3, \"relay\"]"
	relay.add_child(output)
	relay.trigger(null)
	check(receiver.calls.back() == ["add_score", 3, "relay"], "outputs call game code through node paths")
	check(events.any(func(e): return e[0] == &"triggered"), "event bus reports fired outputs")
	GodotTrenchIO.events().fired.disconnect(on_event)
	check(overlay.lines.size() >= 1 and overlay.lines[overlay.lines.size() - 1].contains("relay_1.triggered > /root/IoMap/Game.add_score"), "debug overlay logs I/O events, got %s" % [overlay.lines])
	overlay.queue_free()
	map.queue_free()
	await process_frame

func _leaf(parent: Node3D, size: Vector3) -> void:
	var shape := CollisionShape3D.new()
	var box := BoxShape3D.new()
	box.size = size
	shape.shape = box
	parent.add_child(shape)

func test_gameplay_entities() -> void:
	print("- gameplay entities")
	var map := Node3D.new()
	root.add_child(map)
	var door := GTDoorRotating.new()
	door._func_godot_apply_properties({ "hinge": "-32 0 0", "open_angle": 90.0, "speed": 900.0, "open_away": false })
	door.position = Vector3(2, 0, 0)
	map.add_child(door)
	await process_frame
	check(near(door.pivot(), Vector3(1, 0, 0)), "hinge converted to meters relative to the door, got %s" % door.pivot())
	door.open()
	var deadline := Time.get_ticks_msec() + 2000
	while door._angle < 89.9 and Time.get_ticks_msec() < deadline:
		await process_frame
	await physics_frame
	await physics_frame
	check(door.is_open and near(door.position, Vector3(1, 0, -1), 0.01), "door swung 90 degrees around its hinge, at %s" % door.position)
	var player := Node3D.new()
	map.add_child(player)
	player.position = Vector3(2, 0, -3)
	door.open_away = true
	door.close()
	await create_timer(0.3).timeout
	check(signf(door.angle_for(player)) != signf(door.angle_for(null)) or door.angle_for(player) == door.open_angle, "open_away picks a side from the activator")

	var lift := GTPlatform.new()
	lift._func_godot_apply_properties({ "travel": "0 64 0", "speed": 20.0, "mode": 1, "wait": 0.05 })
	map.add_child(lift)
	await process_frame
	var reached := []
	lift.reached_end.connect(func(): reached.append("end"))
	lift.reached_start.connect(func(): reached.append("start"))
	lift.start()
	deadline = Time.get_ticks_msec() + 3000
	while reached.size() < 2 and Time.get_ticks_msec() < deadline:
		await process_frame
	check(reached.slice(0, 2) == ["end", "start"], "ping pong platform goes up and comes back, got %s" % [reached])
	lift.stop()

	var counter := GTCounter.new()
	counter._func_godot_apply_properties({ "min": 0, "max": 3, "start_value": 0 })
	map.add_child(counter)
	var hit := []
	counter.hit_max.connect(func(): hit.append(true))
	for i in 3:
		GodotTrenchIO.invoke(counter, &"add", "1", null)
	check(counter.value == 3 and hit.size() == 1, "counter fires hit_max once at its limit")
	counter.reset()
	var presser := Node3D.new()
	map.add_child(presser)
	var button := GTButton.new()
	map.add_child(button)
	counter.set_meta(GodotTrenchIO.TARGETNAME_META, "count_presses")
	var press_output := GodotTrenchOutput.new()
	press_output.output = &"pressed"
	press_output.target = "count_presses"
	press_output.input = &"add"
	button.add_child(press_output)
	await process_frame
	button.use(presser)
	check(counter.value == 1, "a button output without a parameter adds the default amount instead of passing the player, value %s" % counter.value)
	GodotTrenchIO.invoke(counter, &"subtract", "", presser)
	check(counter.value == 0, "empty parameters fall back to the default amount")
	counter.add(presser)
	check(counter.value == 1, "a node passed as the amount counts as one")
	check(GodotTrenchIO.activator_arguments(door, &"open", presser) == [presser], "inputs with an activator argument still receive the activator")
	check(GodotTrenchIO.activator_arguments(counter, &"add", presser) == [], "value inputs get no activator")

	var trigger := GTTriggerCall.new()
	var receiver: Node3D = load("res://tests/helpers/io_receiver.gd").new()
	receiver.name = "Game"
	map.add_child(receiver)
	trigger._func_godot_apply_properties({ "call_target": str(receiver.get_path()), "method": "add_score", "arguments": "[5, \"area\"]", "once": true, "filter_group": "player" })
	map.add_child(trigger)
	await process_frame
	var body := CharacterBody3D.new()
	body.add_to_group(&"player")
	map.add_child(body)
	trigger._body_entered(body)
	trigger._body_entered(body)
	check(receiver.calls == [["add_score", 5, "area"]], "trigger_call calls game code once for a player, got %s" % [receiver.calls])
	var stranger := CharacterBody3D.new()
	map.add_child(stranger)
	trigger.enable()
	trigger._last_fired = -100.0
	trigger._body_entered(stranger)
	check(receiver.calls.size() == 1, "filter_group ignores other bodies")

	var hurt := GTHurt.new()
	hurt._func_godot_apply_properties({ "damage": 10.0, "interval": 0.5, "filter_group": "" })
	map.add_child(hurt)
	hurt.apply_damage(receiver)
	check(near(receiver.health, 95.0), "trigger_hurt calls take_damage with damage per interval, health %s" % receiver.health)
	map.queue_free()
	await process_frame

func test_spawner() -> void:
	print("- spawners")
	var map := Node3D.new()
	root.add_child(map)
	var enemy := Node3D.new()
	enemy.name = "Enemy"
	var packed := PackedScene.new()
	packed.pack(enemy)
	enemy.free()
	ResourceSaver.save(packed, "user://gt_test_enemy.tscn")
	var spawner := GTSpawner.new()
	spawner._func_godot_apply_properties({ "scene": "user://gt_test_enemy.tscn", "count": 3, "max_alive": 4, "total": 5, "radius": 64.0, "snap_to_ground": false, "spawn_group": "enemies" })
	map.add_child(spawner)
	await process_frame
	var spawned := []
	var dead := []
	var exhausted := []
	spawner.spawned.connect(func(n): spawned.append(n))
	spawner.all_dead.connect(func(): dead.append(true))
	spawner.exhausted.connect(func(): exhausted.append(true))
	spawner.spawn()
	check(spawned.size() == 3 and get_nodes_in_group(&"enemies").size() == 3, "spawns count instances into the group")
	check(spawned.all(func(n): return (n as Node3D).global_position.distance_to(spawner.global_position) <= 2.0 + 0.001), "spawns stay within the radius")
	spawner.spawn()
	check(spawned.size() == 4, "max_alive limits living spawns, got %d" % spawned.size())
	spawner.kill_all()
	await process_frame
	check(dead.size() == 1, "all_dead fires when every spawn is gone")
	spawner.spawn()
	check(spawned.size() == 5 and exhausted.size() >= 1, "total stops spawning and fires exhausted")
	map.queue_free()
	await process_frame

func test_scatter_and_blend() -> void:
	print("- scatter sets and blend materials")
	var settings: FuncGodotMapSettings = load(SETTINGS)
	var data := {
		"type": "scatter", "name": "trees", "kind": "props", "collision": "convex", "cast_shadows": true, "visibility_range": 3200,
		"items": [{ "source": "res://demo/models/pine.bbmodel" }, { "source": "res://demo/models/missing.bbmodel" }],
		"instances": [[0, 320, 0, 0, 0, 90, 0, 2.0], [0, -320, 16, 64, 0, 0, 0, 1.0], [1, 0, 0, 0, 0, 0, 0, 1.0]],
	}
	var scatter := GodotTrenchScatter.create(data, Transform3D.IDENTITY, settings)
	var multimeshes := collect(scatter, func(n): return n is MultiMeshInstance3D)
	check(multimeshes.size() == 1 and (multimeshes[0] as MultiMeshInstance3D).multimesh.instance_count == 2, "one multimesh with both pine instances")
	var t := GodotTrenchScatter.instance_transform(data["instances"][0], Transform3D.IDENTITY, settings.scale_factor)
	check(near(t.origin, Vector3(10, 0, 0)) and near(t.basis.get_scale(), Vector3(2, 2, 2), 0.001), "instance position in meters and scale, got %s" % t)
	check(near(t.basis * Vector3.FORWARD, Vector3.LEFT * 2.0, 0.001), "instance yaw 90 degrees")
	check((multimeshes[0] as MultiMeshInstance3D).material_override == null, "without an override the models keep their own materials")

	# A material on the set retextures every model, one on a palette entry wins for that entry.
	var retextured: Dictionary = data.duplicate(true)
	retextured["material"] = "showcase/grass"
	var set_wide := GodotTrenchScatter.create(retextured, Transform3D.IDENTITY, settings)
	var set_meshes := collect(set_wide, func(n): return n is MultiMeshInstance3D)
	check(set_meshes.size() == 1 and (set_meshes[0] as MultiMeshInstance3D).material_override != null, "the set's material is applied to its multimesh")
	retextured["items"][0]["material"] = "showcase/cobble"
	var per_item := GodotTrenchScatter.create(retextured, Transform3D.IDENTITY, settings)
	var item_meshes := collect(per_item, func(n): return n is MultiMeshInstance3D)
	var set_albedo = (set_meshes[0] as MultiMeshInstance3D).material_override.albedo_texture
	var item_albedo = (item_meshes[0] as MultiMeshInstance3D).material_override.albedo_texture
	check(item_albedo != null and item_albedo != set_albedo, "the palette entry's own material wins over the set's")
	set_wide.free()
	per_item.free()
	if multimeshes.size() == 1:
		var mmi := multimeshes[0] as MultiMeshInstance3D
		check(near(mmi.visibility_range_end, 100.0), "visibility range in meters")
		# Headless drops the MultiMesh buffer, the saved copy must survive a pack and load.
		var kept: PackedFloat32Array = mmi.get_meta(GodotTrenchScatter.BUFFER_META, mmi.multimesh.buffer)
		check(kept.size() == 24, "instance transforms kept for saving, got %d floats" % kept.size())
		var packed := PackedScene.new()
		scatter.owner = null
		for c in scatter.get_children():
			c.owner = scatter
		check(packed.pack(scatter) == OK, "scatter packs into a scene")
		var copy := packed.instantiate() as GodotTrenchScatter
		copy.restore_buffers()
		var restored := (copy.get_child(0) as MultiMeshInstance3D).multimesh
		check(restored.instance_count == 2 and (restored.buffer.size() == 24 or copy.get_child(0).has_meta(GodotTrenchScatter.BUFFER_META)), "restored scatter keeps its instances")
		copy.free()
	var shapes := collect(scatter, func(n): return n is CollisionShape3D)
	check(shapes.size() == 2 and shapes[0].shape == shapes[1].shape, "props share one collision shape per model")
	scatter.free()

	# Nodes without an owner are silently left out when the editor saves the scene.
	var map_node := Node3D.new()
	var built_sets := GodotTrenchScatter.build_all(map_node, [{ "data": data, "xform": Transform3D.IDENTITY, "group": null, "id": 1 }], settings)
	var unowned := collect(map_node, func(n): return n != map_node and n.owner != map_node)
	check(built_sets.size() == 1 and unowned.is_empty(), "built scatter sets are owned by the scene, unowned: %s" % [unowned])
	var saved := PackedScene.new()
	saved.pack(map_node)
	var reloaded := saved.instantiate()
	check(collect(reloaded, func(n): return n is MultiMeshInstance3D).size() == 1, "scatter sets survive saving the scene")
	reloaded.free()
	map_node.free()

	var blend := GodotTrenchBlend.key("showcase/cobble", "showcase/grass")
	check(GodotTrenchBlend.is_blend(blend) and GodotTrenchBlend.parts(blend) == PackedStringArray(["showcase/cobble", "showcase/grass"]), "blend texture names round trip")
	var built: Array = GodotTrenchBlend.build(blend, settings, [])
	var mat := built[0] as ShaderMaterial
	check(mat != null and mat.get_shader_parameter("texture_a") != null and mat.get_shader_parameter("texture_b") != null, "blend material samples both textures")
	check(GodotTrenchBlend.options(blend) == [0.0, 1.0, 0.5], "a plain blend tiles as authored")

	# A path face asks for de-tiling, a tighter repeat and a crispness, which ride along with the texture name
	# so faces with different settings do not share one material.
	var path := GodotTrenchBlend.key("showcase/cobble", "showcase/grass", 0.75, 4.0, 0.9)
	check(GodotTrenchBlend.parts(path) == PackedStringArray(["showcase/cobble", "showcase/grass"]), "options do not disturb the texture names")
	var opts := GodotTrenchBlend.options(path)
	check(is_equal_approx(opts[0], 0.75) and is_equal_approx(opts[1], 4.0) and is_equal_approx(opts[2], 0.9), "options round trip, got %s" % [opts])
	check(path != blend, "a de-tiled face gets its own material")
	var path_mat := GodotTrenchBlend.build(path, settings, [])[0] as ShaderMaterial
	check(is_equal_approx(path_mat.get_shader_parameter("detile_b"), 0.75), "the painted texture is de-tiled")
	check(path_mat.get_shader_parameter("uv_scale_b").is_equal_approx(Vector2.ONE * 4.0), "the painted texture takes the repeat")
	check(is_equal_approx(path_mat.get_shader_parameter("detile_sharpen_b"), 0.9), "the painted texture keeps its crispness")
	# Older names without the crispness still load, they just take the default.
	check(is_equal_approx(GodotTrenchBlend.options("a|blend|b|opt|0.500,2.000")[2], 0.5), "a two value option tail still parses")

func test_face_cull() -> void:
	print("- coplanar face culling")
	var tiles: Array[PackedVector2Array] = []
	for j in 3:
		for i in 3:
			tiles.append(PackedVector2Array([Vector2(i, j), Vector2(i + 1, j), Vector2(i + 1, j + 1), Vector2(i, j + 1)]))
	# The center tile first would punch a hole.
	var center_first: Array[PackedVector2Array] = [tiles[4]]
	for k in tiles.size():
		if k != 4:
			center_first.append(tiles[k])
	var square := PackedVector2Array([Vector2(0, 0), Vector2(3, 0), Vector2(3, 3), Vector2(0, 3)])
	check(GodotTrenchFaceCull.fully_covered(square, center_first), "a grid of covers hides a face in any order")
	check(not GodotTrenchFaceCull.fully_covered(square, center_first.slice(0, 8)), "one missing tile keeps the face")

	var box_vertices := []
	for y in [-24.0, -2.0]:
		for c in [[0.0, 0.0], [128.0, 0.0], [128.0, 128.0], [0.0, 128.0]]:
			box_vertices.append([c[0], y, c[1]])
	var box_faces := []
	for indices in [[4, 7, 6, 5], [0, 1, 2, 3], [1, 5, 6, 2], [0, 3, 7, 4], [3, 2, 6, 7], [0, 4, 5, 1]]:
		box_faces.append({ "indices": indices, "material": "showcase/cobble" })
	var grid_vertices := []
	for j in 3:
		for i in 3:
			grid_vertices.append([i * 64.0, -2.0, j * 64.0])
	var grid_faces := []
	for j in 2:
		for i in 2:
			var a := j * 3 + i
			grid_faces.append({ "indices": [a, a + 3, a + 4, a + 1], "material": "showcase/cobble", "props": { "blend_material": "showcase/grass" } })
	var map_json := {
		"format": "godottrench-map", "properties": {},
		"layers": [{ "type": "layer", "id": 1, "children": [
			{ "type": "brush", "id": 2, "vertices": box_vertices, "faces": box_faces },
			{ "type": "mesh", "id": 3, "vertices": grid_vertices, "faces": grid_faces },
		] }],
	}
	var settings: FuncGodotMapSettings = load(SETTINGS)
	var path := OS.get_temp_dir().path_join("gt_face_cull_test.gtm")
	FileAccess.open(path, FileAccess.WRITE).store_string(JSON.stringify(map_json))
	var data := FuncGodotParser.new().parse_map_data(path, settings)
	DirAccess.remove_absolute(path)
	var generator := FuncGodotGeometryGenerator.new(settings)
	generator.build(0, data.entities)
	var brush: FuncGodotData.BrushData = data.entities[0].brushes[0]
	var sheet: FuncGodotData.BrushData = data.entities[0].brushes[1]
	var hidden := brush.faces.filter(func(f): return f.render_hidden)
	check(not sheet.closed and brush.closed, "a grid mesh is an open sheet")
	# Planes are in id space, up is +Z.
	check(hidden.size() == 1 and hidden[0].plane.normal.is_equal_approx(Vector3(0, 0, 1)), "the brush top under a blend sheet is hidden, got %s" % [hidden.map(func(f): return f.plane.normal)])
	check(sheet.faces.all(func(f): return not f.render_hidden), "the blend sheet draws")
	check(data.entities[0].pending_convex_points.is_empty() and data.entities[0].shapes.size() >= 1, "collision is still built")

func box_node(id: int, min_c: Vector3, max_c: Vector3, material := "showcase/cobble") -> Dictionary:
	var vertices := []
	for y in [min_c.y, max_c.y]:
		for c in [[min_c.x, min_c.z], [max_c.x, min_c.z], [max_c.x, max_c.z], [min_c.x, max_c.z]]:
			vertices.append([c[0], y, c[1]])
	var faces := []
	for indices in [[4, 7, 6, 5], [0, 1, 2, 3], [1, 5, 6, 2], [0, 3, 7, 4], [3, 2, 6, 7], [0, 4, 5, 1]]:
		faces.append({ "indices": indices, "material": material })
	return { "type": "brush", "id": id, "vertices": vertices, "faces": faces }

func cull_brushes(children: Array) -> Array[FuncGodotData.BrushData]:
	var settings: FuncGodotMapSettings = load(SETTINGS)
	var path := OS.get_temp_dir().path_join("gt_interior_cull_test.gtm")
	var map_json := { "format": "godottrench-map", "properties": {}, "layers": [{ "type": "layer", "id": 1, "children": children }] }
	FileAccess.open(path, FileAccess.WRITE).store_string(JSON.stringify(map_json))
	var data := FuncGodotParser.new().parse_map_data(path, settings)
	DirAccess.remove_absolute(path)
	FuncGodotGeometryGenerator.new(settings).build(0, data.entities)
	return data.entities[0].brushes

func test_interior_face_culling() -> void:
	print("- interior face culling")
	# A small box swallowed by a big one: the whole inner surface is interior and never drawn.
	var swallowed := cull_brushes([box_node(2, Vector3(-256, -256, -256), Vector3(256, 256, 256)), box_node(3, Vector3(-32, -32, -32), Vector3(32, 32, 32))])
	check(swallowed[1].faces.all(func(f): return f.render_hidden), "every face of the buried box is dropped")
	check(swallowed[0].faces.all(func(f): return not f.render_hidden), "the outer shell keeps its faces")

	# Two boxes poking out of each other keep everything: no whole face is buried.
	var overlapping := cull_brushes([box_node(2, Vector3(0, 0, 0), Vector3(64, 64, 64)), box_node(3, Vector3(32, 32, 32), Vector3(96, 96, 96))])
	for b: FuncGodotData.BrushData in overlapping:
		check(b.faces.all(func(f): return not f.render_hidden), "half overlapping solids keep their faces")

func test_chunk_streamer() -> void:
	print("- chunk streaming and render culling")
	# Two boxes far apart, so they land in different chunks and one can be culled while the other shows.
	var near_box := box_node(2, Vector3(-64, -64, -64), Vector3(64, 64, 64))
	var far_box := box_node(3, Vector3(8192, -64, -64), Vector3(8320, 64, 64))
	var map_json := {
		"format": "godottrench-map",
		"properties": { "chunk_streaming": "1", "chunk_size": "512", "load_radius": "2048" },
		"layers": [{ "type": "layer", "id": 1, "children": [near_box, far_box] }],
	}
	var path := OS.get_temp_dir().path_join("gt_streamer_test.gtm")
	FileAccess.open(path, FileAccess.WRITE).store_string(JSON.stringify(map_json))
	var map := FuncGodotMap.new()
	map.map_settings = load(SETTINGS)
	map.local_map_file = path
	root.add_child(map)
	map.build()

	var streamer := find_named(map, "streamer") as GodotTrenchStreamer
	check(streamer != null, "worldspawn chunk_streaming builds a streamer")
	if streamer:
		var scale: float = (load(SETTINGS) as FuncGodotMapSettings).scale_factor
		check(is_equal_approx(streamer.chunk_size, 512.0 * scale), "chunk size converted to meters, got %f" % streamer.chunk_size)
		check(streamer.chunk_count() >= 2, "the two distant boxes land in different chunks, got %d" % streamer.chunk_count())
		# Drive it from a camera next to the near box: the far box's chunk is culled, the near one stays.
		var loaded: Array[Vector3i] = []
		var unloaded: Array[Vector3i] = []
		streamer.area_loaded.connect(func(key: Vector3i, _b: AABB): loaded.append(key))
		streamer.area_unloaded.connect(func(key: Vector3i, _b: AABB): unloaded.append(key))
		var camera := Camera3D.new()
		map.add_child(camera)
		camera.global_position = Vector3.ZERO
		streamer.camera_path = streamer.get_path_to(camera)
		streamer._process(0.0)
		var near_shown := streamer.visible_count()
		check(near_shown > 0 and near_shown < streamer.node_count(), "near chunks draw and far ones do not, %d of %d" % [near_shown, streamer.node_count()])
		check(not loaded.is_empty(), "area_loaded reports the chunks that came into range")
		check(streamer.is_area_loaded(Vector3.ZERO), "the chunk under the camera reports loaded")
		check(not streamer.is_area_loaded(Vector3(8256, 0, 0) * scale), "the far box is not loaded yet")

		# Walking over to the far box loads it and drops the one behind.
		loaded.clear()
		camera.global_position = Vector3(8256, 0, 0) * scale
		streamer._process(0.0)
		check(streamer.is_area_loaded(Vector3(8256, 0, 0) * scale), "the far chunk loads when the camera reaches it")
		check(not loaded.is_empty() and not unloaded.is_empty(), "moving away fires area_loaded and area_unloaded")
		# Meshes are all the streamer touches, collision and scripts keep running everywhere.
		var bodies := collect(map, func(n): return n is CollisionShape3D)
		check(bodies.all(func(n): return not n.is_queued_for_deletion()), "collision shapes are left alone")

		# A camera that is not moving must not rescan: standing in a busy area costs nothing per frame.
		loaded.clear()
		unloaded.clear()
		var settled := streamer.visible_count()
		for _i in 30:
			streamer._process(0.016)
		check(loaded.is_empty() and unloaded.is_empty(), "a still camera fires no chunk signals")
		check(streamer.visible_count() == settled, "a still camera changes nothing")
		# A nudge smaller than the hysteresis margin is still inside the slack and does not rescan either.
		camera.global_position += Vector3(0.05, 0, 0)
		streamer._process(0.016)
		check(loaded.is_empty() and unloaded.is_empty(), "a tiny step stays within the margin")
	map.free()
	DirAccess.remove_absolute(path)

	# Splitting a mesh must not lose or move geometry, only share it out between the pieces.
	var sphere := SphereMesh.new()
	sphere.radius = 8.0
	sphere.height = 16.0
	var source := ArrayMesh.new()
	source.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, sphere.get_mesh_arrays())
	var pieces := GodotTrenchStreamer.split_mesh(source, 4.0)
	var after := 0
	var merged := AABB()
	var first := true
	for key: Vector3i in pieces:
		var piece: ArrayMesh = pieces[key]
		after += triangle_count(piece)
		merged = piece.get_aabb() if first else merged.merge(piece.get_aabb())
		first = false
	check(pieces.size() > 1, "a mesh wider than the cell splits into pieces, got %d" % pieces.size())
	check(after == triangle_count(source), "splitting keeps every triangle, %d of %d" % [after, triangle_count(source)])
	check(merged.size.distance_to(source.get_aabb().size) < 0.01, "the pieces cover the same bounds, %s vs %s" % [merged.size, source.get_aabb().size])

func triangle_count(mesh: ArrayMesh) -> int:
	var n := 0
	for s in mesh.get_surface_count():
		var arrays := mesh.surface_get_arrays(s)
		var indices: PackedInt32Array = arrays[Mesh.ARRAY_INDEX]
		n += (indices.size() if indices and not indices.is_empty() else arrays[Mesh.ARRAY_VERTEX].size()) / 3
	return n

func test_csharp_entities() -> void:
	print("- C# entity definitions")
	var entries := GodotTrenchCSharp.parse_source(FileAccess.get_file_as_string("res://tests/csharp/NpcGuard.cs"), "res://tests/csharp/NpcGuard.cs")
	check(entries.size() == 2, "two attributed classes, got %d" % entries.size())
	if entries.size() == 2:
		var guard: Dictionary = entries[0]
		check(guard["classname"] == "npc_guard" and guard["type"] == "point" and guard["node_class"] == "NpcGuard", "guard class read")
		check(guard["size"] == [[-16.0, 0.0, -16.0], [16.0, 64.0, 16.0]], "size attribute read")
		var props := {}
		for p in guard["properties"]:
			props[p["name"]] = p
		check(props.has("walk_speed") and props["walk_speed"]["type"] == "float" and props["walk_speed"]["default"] == "3.5", "exported float with default, got %s" % [props.get("walk_speed")])
		check(props.has("patrol_offset") and props["patrol_offset"]["default"] == "0 0 128", "exported Vector3 default")
		check(props.has("start_asleep") and props["start_asleep"]["default"] == "1", "exported bool field")
		check(guard["outputs"] == [{ "name": "alerted", "parameter": "activator, level" }], "signals become outputs, got %s" % [guard["outputs"]])
		check(guard["inputs"].map(func(i): return i["name"]) == ["alert", "go_to_sleep"], "marked methods become inputs, got %s" % [guard["inputs"]])
		check(entries[1]["type"] == "solid" and entries[1]["inputs"][0]["name"] == "try_code", "unmarked public methods are inputs when none are marked")
	var defs := GodotTrenchCSharp.definitions(PackedStringArray(["res://tests/csharp"]))
	check(defs.has("npc_guard") and defs["npc_guard"] is FuncGodotFGDPointClass and defs["npc_guard"].node_class == "NpcGuard", "FuncGodot definitions from C#")
	var config: GodotTrenchGameConfig = load("res://demo/demo_game_config.tres")
	config.csharp_source_dirs = PackedStringArray(["res://tests/csharp"])
	config.output_path = "user://godottrench_game_cs.json"
	config.export_file()
	var data = JSON.parse_string(FileAccess.get_file_as_string(config.output_path))
	var by_name := {}
	for e in data.get("entities", []):
		by_name[e["classname"]] = e
	check(by_name.has("func_vault_door") and by_name["func_vault_door"]["csharp"] == true, "C# entities exported for the editor")
	if by_name.has("func_door_rotating"):
		var door: Dictionary = by_name["func_door_rotating"]
		check(door.get("gizmos", []).size() == 1 and door["gizmos"][0]["type"] == "hinge", "gizmos exported from FGD meta")
		check(door["properties"].any(func(p): return p["name"] == "hinge" and p["type"] == "vector3"), "property types exported from FGD meta")
		check(not door["inputs"].any(func(i): return i["name"] == "angle_for"), "helper methods are not inputs")
	check(by_name.has("trigger_spawn_area") and by_name.has("logic_debug") and by_name.has("info_spawner"), "addon entity library exported")
func world_vertex_count(map: FuncGodotMap) -> int:
	var world: Node = GodotTrenchBuild.nodes_by_id(map).get(0)
	var count := 0
	if not world:
		return count
	for mesh_instance: MeshInstance3D in collect(world, func(n): return n is MeshInstance3D):
		for s in mesh_instance.mesh.get_surface_count():
			count += mesh_instance.mesh.surface_get_arrays(s)[Mesh.ARRAY_VERTEX].size()
	return count

func test_live_session() -> void:
	print("- live session")
	var text := FileAccess.get_file_as_string(MAP)
	var map := FuncGodotMap.new()
	map.name = "LiveMap"
	map.map_settings = load(SETTINGS)
	map.local_map_file = MAP
	root.add_child(map)
	map.build_from_text(text)
	await process_frame
	var by_id := GodotTrenchBuild.nodes_by_id(map)
	check(by_id.has(0) and by_id.has(9) and by_id.has(15) and by_id.has(16), "generated nodes carry their map node ids, got %s" % [by_id.keys()])
	check(GodotTrenchBuild.groups_by_id(map).has(6), "group nodes carry their map node ids")
	var world_before: Node = by_id[0]
	var maps: Array[FuncGodotMap] = [map]
	var session := GodotTrenchLiveSession.new(MAP, maps)
	check(session.begin(text), "the session takes the map")
	check(GodotTrenchBuild.nodes_by_id(map)[0] == world_before, "a scene built from the same map is not rebuilt when the session starts")

	var lamp: Node3D = by_id[15]
	check(session.apply([{ "op": "translate", "ids": [15], "offset": [32.0, 0.0, 0.0] }]), "translate applies")
	check(near(lamp.position, Vector3(1, 3.5, 0)), "translate moves the lamp right away, got %s" % lamp.position)
	check(near(float(session._nodes[15]["origin"][0]), 32.0), "translate moves the lamp in the model")
	var lamp_json: Dictionary = session._nodes[15].duplicate(true)
	lamp_json["properties"]["light_energy"] = "5"
	session.apply([{ "op": "set", "id": 15, "parent": 1, "index": 8, "node": lamp_json }])
	session.process()
	by_id = GodotTrenchBuild.nodes_by_id(map)
	var lamp2 := by_id.get(15) as OmniLight3D
	check(lamp2 != null and lamp2 != lamp and near(lamp2.light_energy, 5.0), "a property change rebuilds that entity")
	check(lamp2 != null and String(lamp2.name) == "entity_lamp" and near(lamp2.position, Vector3(1, 3.5, 0)), "the rebuilt lamp keeps its name and new position")
	check(by_id[0] == world_before, "entity edits leave the world mesh alone")

	var door: Node3D = by_id[9]
	var door_position := door.position
	session.apply([{ "op": "translate", "ids": [10], "offset": [0.0, 16.0, 0.0] }])
	check(near(door.position, door_position + Vector3(0, 0.5, 0)), "moving every brush of a brush entity moves its node, got %s" % door.position)
	session.apply([{ "op": "set", "id": 10, "parent": 9, "index": 0, "node": session._nodes[10].duplicate(true) }])
	session.process()
	var door2 := GodotTrenchBuild.nodes_by_id(map).get(9) as Node3D
	check(door2 != door and door2 is GTDoor, "a changed door brush rebuilds the door")
	check(door2 != null and near(door2.position, door_position + Vector3(0, 0.5, 0)), "a brush entity rebuilt alone keeps its brush center origin, got %s" % (door2.position if door2 else null))
	check(door2 != null and collect(door2, func(n): return n is GodotTrenchOutput).size() == 1, "the rebuilt door gets its output relay")

	session.apply([{ "op": "remove", "id": 16 }])
	session.apply([{ "op": "set", "id": 40, "parent": 6, "index": 2, "node": { "id": 40, "type": "entity", "classname": "light", "origin": [0.0, 64.0, 64.0], "angles": [0.0, 0.0, 0.0], "properties": {} } }])
	session.process()
	by_id = GodotTrenchBuild.nodes_by_id(map)
	check(not by_id.has(16) and collect(map, func(n): return n is Marker3D).is_empty(), "a removed entity is gone")
	check(by_id.get(40) is OmniLight3D and by_id[40].get_parent() == GodotTrenchBuild.groups_by_id(map).get(6), "a new entity is built inside its group")

	var top := func(root_node: Node) -> float:
		var y := -INF
		for mi: MeshInstance3D in collect(root_node, func(n): return n is MeshInstance3D):
			y = maxf(y, (mi.global_transform * mi.get_aabb()).end.y)
		return y
	session.apply([{ "op": "translate", "ids": [7], "offset": [0.0, 100.0, 0.0] }])
	session.process()
	var world_after: Node = GodotTrenchBuild.nodes_by_id(map).get(0)
	check(world_after != world_before and world_after.name == GodotTrenchLiveSession.WORLD_NODE and world_after.get_parent() == map, "dragging a loose brush replaces the world with chunks")
	var drag := world_after.get_node_or_null("_gt_live_brush_7") as Node3D
	check(drag != null and near(top.call(drag), 5.125, 0.01), "the dragged brush has a node of its own, top %s" % (top.call(drag) if drag else null))
	session.apply([{ "op": "translate", "ids": [7], "offset": [0.0, 100.0, 0.0] }])
	check(drag != null and near(top.call(drag), 8.25, 0.01), "further drag steps move that node right away")
	session.apply([{ "op": "set", "id": 7, "parent": 6, "index": 0, "node": session._nodes[7].duplicate(true) }])
	session.process()
	check(world_after.get_node_or_null("_gt_live_brush_7") == null and world_after.get_children().any(func(n): return String(n.name).begins_with("chunk_")), "when the drag ends the brush goes back into a chunk")
	check(near(top.call(world_after), 8.25, 0.01), "the chunks show the moved brush, top %s" % top.call(world_after))
	var fresh := FuncGodotMap.new()
	fresh.map_settings = map.map_settings
	fresh.local_map_file = MAP
	root.add_child(fresh)
	fresh.build_from_text(session.text())
	check(world_vertex_count(fresh) == world_vertex_count(map), "the live world matches a full build of the same map")
	fresh.free()

	var heights := Marshalls.raw_to_base64(PackedFloat32Array([0.0, 0.0, 0.0, 0.0]).to_byte_array())
	session.apply([{ "op": "set", "id": 41, "parent": 1, "index": 0, "node": { "id": 41, "type": "terrain", "origin": [0.0, 0.0, 0.0], "resolution": [2, 2], "cell_size": 64.0, "heights": heights, "layers": [{ "material": "base/floor" }] } }])
	session.process()
	var terrain := GodotTrenchBuild.nodes_by_id(map).get(41) as GodotTrenchTerrain
	check(terrain != null, "a new terrain is built")
	if terrain:
		session.apply([{ "op": "translate", "ids": [41], "offset": [64.0, 0.0, 0.0] }])
		var material := (terrain.get_child(0) as MeshInstance3D).mesh.surface_get_material(0) as ShaderMaterial
		check(near(terrain.position, Vector3(2, 0, 0)) and near(material.get_shader_parameter("map_offset"), Vector3(2, 0, 0)), "a moved terrain keeps its texture projection")

	check(not session.apply([{ "op": "set", "id": 42, "parent": 999, "index": 0, "node": { "id": 42, "type": "entity", "classname": "light" } }]), "ops that do not fit the model ask for a resync")

	session.apply([{ "op": "translate", "ids": [17], "offset": [0.0, 0.0, 64.0] }])
	session._last_change -= GodotTrenchLiveSession.IDLE_MSEC + 1
	session.process()
	var plight := find_named(map, "entity_p1-plight") as Node3D
	check(plight != null and near(plight.position, Vector3(6.25, 2.0, -1.0)), "moving a prefab instance rebuilds the map from the model, got %s" % (plight.position if plight else null))
	check(GodotTrenchLiveSession.building == 0 and not session.pending(), "the session is idle again")
	map.queue_free()
	await process_frame

func test_threaded_build_matches() -> void:
	print("- threaded building matches single threaded")
	var path := "res://demo/maps/showcase/lighthouse_forest.gtm"
	var settings: FuncGodotMapSettings = load(SETTINGS)
	var text := FileAccess.get_file_as_string(path)
	var runs := []
	for threaded in [false, true]:
		ProjectSettings.set_setting(GodotTrenchBuild.SETTING_THREADED, threaded)
		var data := FuncGodotParser.new().parse_gtm(text, settings, path)
		var brushes := []
		for e in data.entities:
			for b in e.brushes:
				brushes.append([b.node_id, b.faces.size(), b.faces[0].exact_vertices])
		var terrain := GodotTrenchTerrain.create(data.terrains[0]["data"], Vector3.ZERO, settings)
		var chunks := []
		for mi in terrain.get_children().filter(func(n): return n is MeshInstance3D):
			var arrays: Array = mi.mesh.surface_get_arrays(0)
			chunks.append([String(mi.name), arrays[Mesh.ARRAY_VERTEX], arrays[Mesh.ARRAY_NORMAL], arrays[Mesh.ARRAY_INDEX]])
		terrain.free()
		runs.append([brushes, chunks])
	ProjectSettings.set_setting(GodotTrenchBuild.SETTING_THREADED, true)
	check(runs[0][0].size() >= 32 and runs[0][0] == runs[1][0], "threaded parsing keeps every brush and its order, %d brushes" % runs[0][0].size())
	check(runs[0][1].size() > 1 and runs[0][1] == runs[1][1], "threaded terrain chunks are identical, %d chunks" % runs[0][1].size())

func test_live_link_lines() -> void:
	print("- live link line splitting")
	var bytes := "{\"a\":\"ü\"}\n{\"b\":1}\n{\"c\"".to_utf8_buffer()
	var first := GodotTrenchEditorIntegration.take_lines(bytes.slice(0, 7))
	check(first[0].is_empty() and first[1].size() == 7, "no line before the newline, a cut multibyte character stays buffered")
	var rest: PackedByteArray = first[1]
	rest.append_array(bytes.slice(7))
	var split := GodotTrenchEditorIntegration.take_lines(rest)
	check(split[0] == PackedStringArray(["{\"a\":\"ü\"}", "{\"b\":1}"]), "complete lines decode whole, got %s" % [split[0]])
	check(split[1].get_string_from_utf8() == "{\"c\"", "the unfinished line stays buffered")

func test_light() -> void:
	print("- light switching")
	var light := GTLight.new()
	light._func_godot_apply_properties({ "light_energy": 2.0, "omni_range": 8.0, "start_on": false })
	root.add_child(light)
	await process_frame
	check(not light.visible and not light.is_on(), "light starts off")
	var switched: Array = []
	light.switched.connect(func(on): switched.append(on))
	GodotTrenchIO.invoke(light, &"turn_on", "", null)
	check(light.visible and light.is_on() and switched.back() == true, "turn_on switches it on and fires switched(true)")
	GodotTrenchIO.invoke(light, &"toggle", "", null)
	check(not light.is_on() and switched.back() == false, "toggle switches it off, got %s" % [switched])
	light.queue_free()
	await process_frame

func test_text() -> void:
	print("- game_text in world and hud")
	var text := GTText.new()
	text._func_godot_apply_properties({ "text": "Follow me", "place": "both", "start_visible": false })
	root.add_child(text)
	await process_frame
	check(text._label3d != null and text._hud != null, "both world and hud labels built")
	check(text._label3d.text == "Follow me" and not text._label3d.visible, "world label set and hidden at start")
	var shown: Array = []
	text.shown.connect(func(): shown.append(true))
	GodotTrenchIO.invoke(text, &"show", "", null)
	check(text._label3d.visible and text._hud.visible and shown.size() == 1, "show reveals both labels and fires shown")
	GodotTrenchIO.invoke(text, &"set_text", "Go left", null)
	check(text._label3d.text == "Go left" and text._hud.text == "Go left", "set_text updates both labels, got '%s'" % text._label3d.text)
	GodotTrenchIO.invoke(text, &"hide", "", null)
	check(not text._label3d.visible, "hide hides it")
	text.queue_free()
	await process_frame

func test_animate() -> void:
	print("- logic_animate drives an AnimationPlayer")
	var map := Node3D.new()
	root.add_child(map)
	var actor := Node3D.new()
	actor.set_meta(GodotTrenchIO.TARGETNAME_META, "actor")
	var player := AnimationPlayer.new()
	var lib := AnimationLibrary.new()
	var clip := Animation.new()
	clip.length = 0.1
	lib.add_animation(&"wave", clip)
	player.add_animation_library(&"", lib)
	actor.add_child(player)
	map.add_child(actor)
	var driver := GTAnimate.new()
	driver._func_godot_apply_properties({ "target": "actor", "animation": "wave" })
	map.add_child(driver)
	await process_frame
	GodotTrenchIO.invoke(driver, &"play", "", null)
	check(player.current_animation == "wave", "logic_animate played the target's animation, got '%s'" % player.current_animation)
	map.queue_free()
	await process_frame

func test_prop_and_explosion() -> void:
	print("- prop_physics and env_explosion")
	var map := Node3D.new()
	root.add_child(map)
	var victim: Node3D = load("res://tests/helpers/io_receiver.gd").new()
	victim.name = "Victim"
	map.add_child(victim)
	victim.global_position = Vector3(1, 0, 0)
	var far: Node3D = load("res://tests/helpers/io_receiver.gd").new()
	far.name = "Far"
	map.add_child(far)
	far.global_position = Vector3(1000, 0, 0)
	var boom := GTExplosion.new()
	boom._func_godot_apply_properties({ "radius": 128.0, "damage": 40.0, "force": 0.0 })
	map.add_child(boom)
	boom.global_position = Vector3.ZERO
	await process_frame
	var exploded: Array = []
	boom.exploded.connect(func(): exploded.append(true))
	boom.explode(null)
	check(exploded.size() == 1, "explosion fires exploded")
	check(victim.health < 100.0, "explosion damaged a node in radius, health %s" % victim.health)
	check(far.health == 100.0, "explosion ignores nodes outside radius, health %s" % far.health)

	var barrel := GTPropPhysics.new()
	barrel._func_godot_apply_properties({ "health": 10.0, "explosive": true, "explosion_radius": 128.0, "explosion_damage": 25.0, "size": "16 16 16" })
	map.add_child(barrel)
	barrel.global_position = Vector3.ZERO
	await process_frame
	var near_hp: Node3D = load("res://tests/helpers/io_receiver.gd").new()
	near_hp.name = "Bystander"
	map.add_child(near_hp)
	near_hp.global_position = Vector3(1, 0, 0)
	var broke: Array = []
	barrel.broken.connect(func(): broke.append(true))
	barrel.take_damage(25.0, null)
	check(broke.size() == 1, "barrel breaks when its health runs out")
	await process_frame
	check(near_hp.health < 100.0, "an explosive barrel blasts nearby nodes when broken, health %s" % near_hp.health)
	map.queue_free()
	await process_frame

func test_npc_path() -> void:
	print("- npc_walker follows a path")
	var map := Node3D.new()
	root.add_child(map)
	var c1 := GTPathCorner.new()
	c1.set_meta(GodotTrenchIO.TARGETNAME_META, "npc_corner_1")
	c1._func_godot_apply_properties({ "target": "npc_corner_2" })
	c1.position = Vector3(4, 0, 0)
	map.add_child(c1)
	var c2 := GTPathCorner.new()
	c2.set_meta(GodotTrenchIO.TARGETNAME_META, "npc_corner_2")
	c2.position = Vector3(4, 0, 4)
	map.add_child(c2)
	var npc := GTNpc.new()
	npc._func_godot_apply_properties({ "target": "npc_corner_1", "speed": 100.0 })
	map.add_child(npc)
	await process_frame
	var arrived: Array = []
	var finished: Array = []
	npc.arrived.connect(func(c): arrived.append(c))
	npc.finished.connect(func(): finished.append(true))
	npc.start()
	var deadline := Time.get_ticks_msec() + 3000
	while finished.is_empty() and Time.get_ticks_msec() < deadline:
		await process_frame
	check(arrived.size() == 2, "npc reached both corners, got %s" % arrived.size())
	check(finished.size() == 1 and npc.global_position.distance_to(Vector3(4, 0, 4)) < 0.2, "npc finished at the last corner, at %s" % npc.global_position)
	map.queue_free()
	await process_frame

func test_sequence() -> void:
	print("- logic_sequence timeline")
	var seq := GTSequence.new()
	seq._func_godot_apply_properties({ "steps": 3, "interval": 0.05 })
	root.add_child(seq)
	await process_frame
	var fired: Array = []
	var order: Array = []
	seq.step.connect(func(i): order.append(i))
	seq.step_1.connect(func(): fired.append(1))
	seq.step_2.connect(func(): fired.append(2))
	seq.step_3.connect(func(): fired.append(3))
	var done: Array = []
	seq.finished.connect(func(): done.append(true))
	seq.start()
	var deadline := Time.get_ticks_msec() + 2000
	while done.is_empty() and Time.get_ticks_msec() < deadline:
		await process_frame
	check(fired == [1, 2, 3] and order == [1, 2, 3], "sequence fires steps in order, got %s" % [fired])
	check(done.size() == 1, "sequence fires finished once")
	seq.queue_free()
	await process_frame

func test_logic_script() -> void:
	print("- logic_script runs inline GDScript")
	var map := Node3D.new()
	root.add_child(map)
	var receiver: Node3D = load("res://tests/helpers/io_receiver.gd").new()
	receiver.name = "ScriptPlayer"
	receiver.add_to_group(&"player")
	map.add_child(receiver)
	await process_frame
	var scripted := GTScript.new()
	scripted._func_godot_apply_properties({ "source": "activator.take_damage(30, this)\nreturn activator.health" })
	map.add_child(scripted)
	await process_frame
	var results: Array = []
	scripted.ran.connect(func(r): results.append(r))
	scripted.run(receiver)
	check(near(receiver.health, 70.0), "logic_script read and lowered the activator's health, got %s" % receiver.health)
	check(results.size() == 1 and near(results.back(), 70.0), "logic_script returns its result, got %s" % [results])

	var expr := GTScript.new()
	expr._func_godot_apply_properties({ "expression": "activator.health" })
	map.add_child(expr)
	await process_frame
	var read: Array = []
	expr.ran.connect(func(r): read.append(r))
	expr.run(receiver)
	check(read.size() == 1 and near(read.back(), 70.0), "logic_script expression reads data, got %s" % [read])
	map.queue_free()
	await process_frame

## Adds a Hammer style output connection from [param source] to a named target, like the map builder does.
func test_more_entities() -> void:
	print("- spot light, particles, branch and sound")
	var map := Node3D.new()
	root.add_child(map)

	var spot := GTSpotLight.new()
	spot._func_godot_apply_properties({ "light_energy": 2.0, "spot_range": 12.0, "spot_angle": 30.0, "start_on": false })
	map.add_child(spot)
	await process_frame
	check(not spot.visible and not spot.is_on(), "spot light starts off")
	var switched: Array = []
	spot.switched.connect(func(on): switched.append(on))
	GodotTrenchIO.invoke(spot, &"turn_on", "", null)
	check(spot.is_on() and spot.visible and switched.back() == true, "spot turn_on switches on")
	check(near(spot.spot_range, 12.0) and near(spot.spot_angle, 30.0), "spot properties applied")

	var particles := GTParticles.new()
	particles._func_godot_apply_properties({ "amount": 48, "lifetime": 1.5, "start_emitting": false })
	map.add_child(particles)
	await process_frame
	check(particles.amount == 48 and not particles.emitting, "particles start idle with amount set")
	GodotTrenchIO.invoke(particles, &"start", "", null)
	check(particles.emitting, "start emits particles")
	GodotTrenchIO.invoke(particles, &"toggle", "", null)
	check(not particles.emitting, "toggle stops particles")

	var branch := GTBranch.new()
	branch._func_godot_apply_properties({ "start_value": false })
	map.add_child(branch)
	var hits: Array = []
	branch.on_true.connect(func(): hits.append("true"))
	branch.on_false.connect(func(): hits.append("false"))
	GodotTrenchIO.invoke(branch, &"test", "", null)
	check(hits == ["false"], "branch tests false by default")
	GodotTrenchIO.invoke(branch, &"set_true", "", null)
	GodotTrenchIO.invoke(branch, &"test", "", null)
	check(hits == ["false", "true"], "branch tests true after set_true")
	branch.set_and_test(false)
	check(hits.back() == "false", "set_and_test evaluates the passed value")

	var sound := GTSound.new()
	sound._func_godot_apply_properties({ "volume_db": -6.0, "loop_sound": true, "max_distance": 20.0 })
	map.add_child(sound)
	await process_frame
	check(near(sound.volume_db, -6.0) and sound.loop_sound and near(sound.max_distance, 20.0), "env_sound applies its properties")
	sound.toggle()

	map.queue_free()
	await process_frame

func _wire(source: Node, output: StringName, target: String, input: StringName, parameter := "") -> void:
	var out := GodotTrenchOutput.new()
	out.output = output
	out.target = target
	out.input = input
	out.parameter = parameter
	source.add_child(out)

func test_scripted_scene() -> void:
	print("- scripted scene playthrough")
	var map := Node3D.new()
	root.add_child(map)
	var player: Node3D = load("res://tests/helpers/io_receiver.gd").new()
	player.name = "ScenePlayer"
	player.add_to_group(&"player")
	map.add_child(player)
	player.global_position = Vector3(0, 0, 6)

	var seq := GTSequence.new()
	seq.set_meta(GodotTrenchIO.TARGETNAME_META, "cutscene")
	seq._func_godot_apply_properties({ "steps": 3, "interval": 0.03 })
	map.add_child(seq)
	var hint := GTText.new()
	hint.set_meta(GodotTrenchIO.TARGETNAME_META, "hint")
	hint._func_godot_apply_properties({ "text": "Follow me", "place": "world" })
	map.add_child(hint)
	var barrel := GTPropPhysics.new()
	barrel.set_meta(GodotTrenchIO.TARGETNAME_META, "barrel")
	barrel._func_godot_apply_properties({ "health": 5.0, "explosive": true, "explosion_radius": 384.0, "explosion_damage": 20.0, "size": "16 16 16" })
	map.add_child(barrel)
	barrel.global_position = Vector3(0, 0, 6)
	var lamp := GTLight.new()
	lamp.set_meta(GodotTrenchIO.TARGETNAME_META, "lamp")
	lamp._func_godot_apply_properties({ "start_on": false })
	map.add_child(lamp)

	_wire(seq, &"step_1", "hint", &"show")
	_wire(seq, &"step_2", "barrel", &"smash")
	_wire(seq, &"step_3", "lamp", &"turn_on")
	await process_frame

	seq.start()
	var deadline := Time.get_ticks_msec() + 2000
	while not lamp.is_on() and Time.get_ticks_msec() < deadline:
		await process_frame
	await process_frame
	check(hint._label3d != null and hint._label3d.visible, "step 1 output showed the hint text")
	check(player.health < 100.0, "step 2 smashed the barrel and its blast hurt the player through I/O, health %s" % player.health)
	check(lamp.is_on(), "step 3 output turned the light on")
	map.queue_free()
	await process_frame

func find_targetname(root: Node, targetname: String) -> Node:
	for n in collect(root, func(n): return str(n.get_meta(GodotTrenchIO.TARGETNAME_META, "")) == targetname):
		return n
	return null

## Plays the lighthouse map built by examples/mcp/lighthouse_forest.json: the gate sensor relay and the beacon trigger_call.
func test_showcase_playthrough() -> void:
	print("- showcase playthrough (lighthouse_forest.gtm)")
	var map := FuncGodotMap.new()
	map.map_settings = load(SETTINGS)
	map.local_map_file = "res://demo/maps/showcase/lighthouse_forest.gtm"
	root.add_child(map)
	map.build()
	await process_frame
	var spawn := load("res://demo/demo.gd").find_spawn(map) as Node3D
	check(spawn != null, "map has an info_player_start")
	var player: DemoPlayer = load("res://demo/player.tscn").instantiate()
	root.add_child(player)
	var units := func(v: Vector3) -> Vector3: return v * map.map_settings.scale_factor
	var gate := find_targetname(map, "gate_left") as GTDoorRotating
	var beacon := find_targetname(map, "beacon") as Light3D
	check(gate != null and beacon != null, "gate and beacon entities built")
	if not spawn or not gate or not beacon:
		player.free()
		map.free()
		return
	player.global_position = spawn.global_position + Vector3.UP * 0.1
	for i in 4:
		await physics_frame
	check(not gate.is_open and gate._angle == 0.0, "gate starts closed")
	player.global_position = units.call(Vector3(1490, 400, 1200))
	var deadline := Time.get_ticks_msec() + 3000
	while absf(gate._angle) < 10.0 and Time.get_ticks_msec() < deadline:
		await physics_frame
	check(absf(gate._angle) >= 10.0, "walking into the gate sensor swings the gate through the relay, angle %.1f" % gate._angle)
	player.global_position = units.call(Vector3(2200, 1170, 1200))
	deadline = Time.get_ticks_msec() + 2000
	while not near(beacon.light_energy, 30.0) and Time.get_ticks_msec() < deadline:
		await physics_frame
	check(near(beacon.light_energy, 30.0), "trigger_call ran set_param(0, 30) on the beacon, energy %.1f" % beacon.light_energy)
	player.free()
	map.free()
