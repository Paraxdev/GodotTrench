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
	test_scatter_and_blend()
	test_csharp_entities()
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

	var blend := GodotTrenchBlend.key("showcase/cobble", "showcase/grass")
	check(GodotTrenchBlend.is_blend(blend) and GodotTrenchBlend.parts(blend) == PackedStringArray(["showcase/cobble", "showcase/grass"]), "blend texture names round trip")
	var built: Array = GodotTrenchBlend.build(blend, settings, [])
	var mat := built[0] as ShaderMaterial
	check(mat != null and mat.get_shader_parameter("texture_a") != null and mat.get_shader_parameter("texture_b") != null, "blend material samples both textures")

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
