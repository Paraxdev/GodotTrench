extends RefCounted
## The binary .gtm container: binary maps build the same as their JSON, damaged and cut off files keep what is
## intact and say what was lost, and JSON maps still load.
##
## res://tests/run_tests.gd calls [method run], [param t] is that script's instance for its check helpers.

const SETTINGS := "res://demo/demo_map_settings.tres"
const DAMAGED := "res://demo/maps/showcase/church_school.gtm"

static func run(t) -> void:
	print("- binary .gtm files")
	for pair in [["res://tests/maps/basic.gtm", "res://tests/maps/basic_json.gtm"], ["res://tests/maps/geometry.gtm", "res://tests/maps/geometry_json.gtm"]]:
		await _test_builds_like_json(t, pair[0], pair[1])
	_test_reads_json(t)
	_test_damaged_chunk(t)
	_test_truncated(t)
	_test_refused(t)
	_test_scatter_instances(t)

static func _build(t, path: String) -> FuncGodotMap:
	var map := FuncGodotMap.new()
	map.map_settings = load(SETTINGS)
	map.local_map_file = path
	t.root.add_child(map)
	map.build()
	return map

## Every generated node with its transform, and the vertices of every mesh and collision shape. The counter in names
## Godot makes up, like [code]@CollisionShape3D@3[/code], differs between two builds and is left out.
static func _scene(root: Node) -> Array:
	var out := []
	var made_up := RegEx.create_from_string("@(\\w+)@\\d+")
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_front()
		var entry := [made_up.sub(str(root.get_path_to(n)), "@$1", true), n.get_class()]
		if n is Node3D:
			entry.append(n.transform)
		if n is MeshInstance3D and n.mesh:
			for s in n.mesh.get_surface_count():
				entry.append(n.mesh.surface_get_arrays(s)[Mesh.ARRAY_VERTEX])
		if n is CollisionShape3D and n.shape is ConcavePolygonShape3D:
			entry.append(n.shape.get_faces())
		if n is CollisionShape3D and n.shape is ConvexPolygonShape3D:
			entry.append(n.shape.points)
		out.append(entry)
		stack.append_array(n.get_children())
	return out

static func _test_builds_like_json(t, binary: String, json: String) -> void:
	# One at a time, the second map would leave the sun to the first.
	var a := _build(t, binary)
	var scene_a := _scene(a)
	a.free()
	var b := _build(t, json)
	t.check(scene_a.size() > 5 and scene_a == _scene(b), "%s builds the same as its JSON twin, %d nodes" % [binary, scene_a.size()])
	b.free()
	await t.process_frame

static func _node_count(map: Dictionary) -> int:
	var count := 0
	var stack: Array = map.get("layers", []).duplicate()
	while not stack.is_empty():
		var n: Dictionary = stack.pop_back()
		count += 1
		stack.append_array(n.get("children", []))
	return count

static func _test_reads_json(t) -> void:
	var bytes := FileAccess.get_file_as_bytes("res://tests/maps/basic_json.gtm")
	t.check(not GodotTrenchGtmFile.is_binary(bytes), "the JSON twin is text")
	var json := GodotTrenchGtmFile.read("res://tests/maps/basic_json.gtm")
	var binary := GodotTrenchGtmFile.read("res://tests/maps/basic.gtm")
	t.check(json["map"] is Dictionary and json["problems"].is_empty(), "a JSON .gtm still reads")
	t.check(_node_count(json["map"]) == _node_count(binary["map"]) and _node_count(json["map"]) > 10, "both hold the same nodes")
	t.check(binary["content"] != 0 and binary["content"] == GodotTrenchGtmFile.content_id("res://tests/maps/basic.gtm"), "the content id is read from the END chunk")

## Start offsets of every chunk of an intact file.
static func _chunks(bytes: PackedByteArray) -> Array[int]:
	var out: Array[int] = []
	var pos := 12
	while pos < bytes.size():
		var h := GodotTrenchGtmFile._header(bytes, pos)
		out.append(pos)
		pos += 24 + int(h["stored"])
	return out

static func _test_damaged_chunk(t) -> void:
	var bytes := FileAccess.get_file_as_bytes(DAMAGED)
	var intact := GodotTrenchGtmFile.decode(bytes)
	var total := _node_count(intact["map"])
	t.check(intact["problems"].is_empty() and total > 100, "the intact showcase map reads without problems")
	var chunks := _chunks(bytes)
	t.check(chunks.size() > 6, "the showcase map has many chunks, got %d" % chunks.size())
	var start := chunks[3]
	var broken := bytes.duplicate()
	broken[start + 24 + (chunks[4] - start - 24) / 2] ^= 0x5a
	var file := GodotTrenchGtmFile.decode(broken)
	var map: Dictionary = file["map"]
	var problems := "; ".join(file["problems"])
	var kept := _node_count(map) - (1 if map["layers"].back()["name"] == GodotTrenchGtmFile.RECOVERED_LAYER else 0)
	t.check(kept < total and kept > total / 2, "a damaged chunk loses only its nodes, kept %d of %d" % [kept, total])
	t.check(problems.contains("nodes stored in it are lost") and problems.contains("of %d nodes could not be read" % total), "the damage is reported, got %s" % problems)
	var data := FuncGodotParser.new().parse_gtm(map, load(SETTINGS), DAMAGED)
	t.check(data.entities.size() > 1, "the rest of the map still builds")

	broken = bytes.duplicate()
	broken[chunks[2] + 17] ^= 0x01
	file = GodotTrenchGtmFile.decode(broken)
	t.check(file["map"] != null and _node_count(file["map"]) > total / 2, "a damaged chunk header skips to the next chunk")

static func _test_truncated(t) -> void:
	var bytes := FileAccess.get_file_as_bytes(DAMAGED)
	var total := _node_count(GodotTrenchGtmFile.decode(bytes)["map"])
	var file := GodotTrenchGtmFile.decode(bytes.slice(0, bytes.size() / 2))
	var kept := _node_count(file["map"])
	t.check(kept > 0 and kept < total, "a cut off file keeps what came before the cut, %d of %d" % [kept, total])
	t.check(not file["problems"].is_empty(), "and says it was cut off")

static func _test_refused(t) -> void:
	var bytes := FileAccess.get_file_as_bytes("res://tests/maps/basic.gtm")
	var mangled := bytes.duplicate()
	mangled.remove_at(4)
	var file := GodotTrenchGtmFile.decode(mangled)
	t.check(file["map"] == null and str(file["error"]).contains("line endings"), "line ending damage is named")
	var newer := bytes.duplicate()
	newer.encode_u32(8, GodotTrenchGtmFile.CONTAINER_VERSION + 1)
	file = GodotTrenchGtmFile.decode(newer)
	t.check(file["map"] == null and str(file["error"]).contains("newer"), "a newer container is refused")

static func _test_scatter_instances(t) -> void:
	var node := { "type": "scatter", "instances": PackedInt32Array([0, 2, 1234, -1, -500, 0, 102450, 300, 0, 450, 3599, 0, -1, 0, 1250, 800]) }
	GodotTrenchGtmFile._restore_instances(node)
	var expected := [[0.0, 12.34, -5.0, 1024.5, 0.0, 359.9, -0.1, 1.25], [2.0, -0.01, 0.0, 3.0, 45.0, 0.0, 0.0, 0.8]]
	t.check(node["instances"] == expected, "scatter instance columns give the saved floats exactly, got %s" % [node["instances"]])
