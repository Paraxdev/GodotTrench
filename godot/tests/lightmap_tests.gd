extends RefCounted
## Lighting baked in the GodotTrench editor: baked faces get their atlas coordinates as UV2, faces changed since the
## bake point at the fallback row, the build adds a LightmapGI with the atlases and sets each light's bake mode, and
## use_baked_lighting off leaves all of that out.
##
## res://tests/run_tests.gd calls [method run], [param t] is that script's instance for its check helpers.

const SETTINGS := "res://demo/demo_map_settings.tres"
const BAKED := "res://tests/maps/baked.gtm"

static func run(t) -> void:
	print("- baked lighting")
	await _test_synthetic(t)
	await _test_editor_bake(t)

static func _map_json() -> Dictionary:
	var light := PackedByteArray()
	light.resize(4 * 2 * 6)
	for k in 4 * 2 * 3:
		light.encode_half(k * 2, 0.5)
	var shadow := PackedByteArray()
	shadow.resize(8)
	shadow.fill(200)
	# u = x / 128 + 0.25, v = z / 128 + 0.25 in map units, for every face of brush 2 and of the stale brush 3.
	var rows := [1.0 / 128.0, 0.0, 0.0, 0.25, 0.0, 0.0, 1.0 / 128.0, 0.25]
	var keys := []
	var chart_rows := []
	for node in [2, 3]:
		for face in 6:
			keys.append_array([node, face])
			chart_rows.append_array(rows)
	var entity := func(id: int, props: Dictionary, x: float) -> Dictionary:
		return { "type": "entity", "id": id, "classname": "light", "origin": [x, 32, 32], "properties": props }
	return {
		"format": "godottrench-map", "version": 1,
		"properties": { "sun_angles": "-40 -45", "sun_bake_mode": "bounce" },
		"layers": [{ "type": "layer", "id": 1, "children": [
			t_box(2, Vector3(0, 0, 0), Vector3(64, 64, 64)),
			t_box(3, Vector3(256, 0, 0), Vector3(320, 64, 64)),
			t_box(4, Vector3(512, 0, 0), Vector3(576, 64, 64)),
			entity.call(10, {}, 32),
			entity.call(11, { "targetname": "lamp" }, 96),
			entity.call(12, { "bake_mode": "bounce" }, 160),
		] }],
		"lightmap": {
			"version": 1, "width": 4, "height": 2, "texel_size": 16,
			"light": Marshalls.raw_to_base64(light), "shadow": Marshalls.raw_to_base64(shadow),
			"ao": Marshalls.raw_to_base64(shadow), "chart_keys": keys, "chart_rows": chart_rows,
			"stale": [3], "fallback": [0.25, 0.25, 0.25],
		},
	}

static func t_box(id: int, min_c: Vector3, max_c: Vector3) -> Dictionary:
	var vertices := []
	for y in [min_c.y, max_c.y]:
		for c in [[min_c.x, min_c.z], [max_c.x, min_c.z], [max_c.x, max_c.z], [min_c.x, max_c.z]]:
			vertices.append([c[0], y, c[1]])
	var faces := []
	for indices in [[4, 7, 6, 5], [0, 1, 2, 3], [1, 5, 6, 2], [0, 3, 7, 4], [3, 2, 6, 7], [0, 4, 5, 1]]:
		faces.append({ "indices": indices, "material": "showcase/cobble" })
	return { "type": "brush", "id": id, "vertices": vertices, "faces": faces }

static func _build(t, path: String, use_baked := true) -> FuncGodotMap:
	var map := FuncGodotMap.new()
	map.map_settings = load(SETTINGS)
	map.local_map_file = path
	map.use_baked_lighting = use_baked
	t.root.add_child(map)
	map.build()
	return map

## UV2 of every vertex of the baked meshes under [param map], keyed by its position in meters.
static func _uv2s(map: Node) -> Dictionary:
	var out := {}
	for mi in map.find_children("*", "MeshInstance3D", true, false):
		if not mi.has_meta(GodotTrenchLightmap.BAKED_META):
			continue
		for s in mi.mesh.get_surface_count():
			var arrays: Array = mi.mesh.surface_get_arrays(s)
			if arrays[Mesh.ARRAY_TEX_UV2] == null:
				continue
			for k in arrays[Mesh.ARRAY_VERTEX].size():
				out[(mi.global_transform * arrays[Mesh.ARRAY_VERTEX][k]).snappedf(0.001)] = arrays[Mesh.ARRAY_TEX_UV2][k]
	return out

static func _test_synthetic(t) -> void:
	var path := OS.get_temp_dir().path_join("gt_lightmap_test.gtm")
	FileAccess.open(path, FileAccess.WRITE).store_string(JSON.stringify(_map_json()))
	var map := _build(t, path)
	var gi := map.get_node_or_null(GodotTrenchLightmap.NODE_NAME) as LightmapGI
	t.check(gi != null and gi.light_data != null, "a baked map gets a LightmapGI")
	if gi:
		var data := gi.light_data
		t.check(data.get_user_count() >= 1, "the baked mesh is a user of the light data, got %d" % data.get_user_count())
		var texture: Texture2DArray = data.lightmap_textures[0]
		t.check(texture.get_width() == 4 and texture.get_height() == 3, "the atlas gets a fallback row, %dx%d" % [texture.get_width(), texture.get_height()])
		t.check(not data.shadowmask_textures.is_empty(), "the sun shadow mask goes along")
		t.check(gi.shadowmask_mode == LightmapGIData.SHADOWMASK_MODE_REPLACE, "a sun that stays real time uses the shadow mask")
		for k in data.get_user_count():
			t.check(gi.get_node_or_null(data.get_user_path(k)) is MeshInstance3D, "user %d points at a mesh" % k)

	var uv2 := _uv2s(map)
	var corner = uv2.get(Vector3(2, 0, 2))
	t.check(corner is Vector2 and corner.is_equal_approx(Vector2(0.75, 0.5)), "a baked corner maps through its rows, squeezed for the fallback row, got %s" % [corner])
	var fallback := Vector2(0.5, 2.5 / 3.0)
	for p in [Vector3(10, 0, 2), Vector3(18, 2, 0)]:
		t.check(uv2.get(p) is Vector2 and (uv2[p] as Vector2).is_equal_approx(fallback), "an unbaked or changed brush at %s uses the fallback row, got %s" % [p, uv2.get(p)])

	var modes := {}
	for light in map.find_children("*", "Light3D", true, false):
		modes[snappedf(light.global_position.x, 0.01)] = light.light_bake_mode
	t.check(modes.get(1.0) == Light3D.BAKE_STATIC, "a plain light is baked")
	t.check(modes.get(3.0) == Light3D.BAKE_DISABLED, "a light I/O can switch stays real time")
	t.check(modes.get(5.0) == Light3D.BAKE_DYNAMIC, "bake_mode bounce keeps the direct light real time")
	map.queue_free()

	map = _build(t, path, false)
	t.check(map.get_node_or_null(GodotTrenchLightmap.NODE_NAME) == null and _uv2s(map).is_empty(), "use_baked_lighting off leaves the bake out")
	map.queue_free()
	DirAccess.remove_absolute(path)
	await t.process_frame

## A room baked by the editor and saved in a binary map: the LMAP chunk is read and its light reaches the atlas.
static func _test_editor_bake(t) -> void:
	var map := _build(t, BAKED)
	var gi := map.get_node_or_null(GodotTrenchLightmap.NODE_NAME) as LightmapGI
	t.check(gi != null and gi.light_data.get_user_count() >= 1, "the editor's bake builds into a LightmapGI")
	# Headless Godot keeps no texture data, so the light is read from the chunk the atlas is made of.
	var lightmap := GodotTrenchLightmap.decode(GodotTrenchGtmFile.read(BAKED)["map"].get("lightmap"))
	var light: PackedByteArray = lightmap.get("light", PackedByteArray())
	var brightest := 0.0
	for k in range(0, light.size(), 6):
		brightest = maxf(brightest, light.decode_half(k))
	t.check(brightest > 0.2, "the LMAP chunk holds the baked light, brightest %s" % brightest)
	var uv2 := _uv2s(map)
	t.check(not uv2.is_empty() and uv2.values().all(func(v): return v.x >= 0.0 and v.x <= 1.0 and v.y >= 0.0 and v.y <= 1.0), "every baked vertex lands inside the atlas")
	map.queue_free()
	await t.process_frame
