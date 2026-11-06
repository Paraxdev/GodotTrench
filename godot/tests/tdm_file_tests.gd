extends RefCounted
## The binary .tdm container: binary maps build the same as their JSON, damaged and cut off files keep what is
## intact and say what was lost, and JSON maps still load.
##
## res://tests/run_tests.gd calls [method run], [param t] is that script's instance for its check helpers.

const SETTINGS := "res://demo/demo_map_settings.tres"
const DAMAGED := "res://demo/maps/showcase/church_school.tdm"

static func run(t) -> void:
	print("- binary .tdm files")
	for pair in [["res://tests/maps/basic.tdm", "res://tests/maps/basic_json.tdm"], ["res://tests/maps/geometry.tdm", "res://tests/maps/geometry_json.tdm"]]:
		await _test_builds_like_json(t, pair[0], pair[1])
	_test_reads_json(t)
	_test_damaged_chunk(t)
	_test_truncated(t)
	_test_refused(t)
	_test_budgets(t)
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
	var bytes := FileAccess.get_file_as_bytes("res://tests/maps/basic_json.tdm")
	t.check(not TrenchDotTdmFile.is_binary(bytes), "the JSON twin is text")
	var json := TrenchDotTdmFile.read("res://tests/maps/basic_json.tdm")
	var binary := TrenchDotTdmFile.read("res://tests/maps/basic.tdm")
	t.check(json["map"] is Dictionary and json["problems"].is_empty(), "a JSON .tdm still reads")
	t.check(_node_count(json["map"]) == _node_count(binary["map"]) and _node_count(json["map"]) > 10, "both hold the same nodes")
	t.check(binary["content"] != 0 and binary["content"] == TrenchDotTdmFile.content_id("res://tests/maps/basic.tdm"), "the content id is read from the END chunk")

## Start offsets of every chunk of an intact file.
static func _chunks(bytes: PackedByteArray) -> Array[int]:
	var out: Array[int] = []
	var pos := 12
	while pos < bytes.size():
		var h := TrenchDotTdmFile._header(bytes, pos)
		out.append(pos)
		pos += 24 + int(h["stored"])
	return out

static func _test_damaged_chunk(t) -> void:
	var bytes := FileAccess.get_file_as_bytes(DAMAGED)
	var intact := TrenchDotTdmFile.decode(bytes)
	var total := _node_count(intact["map"])
	t.check(intact["problems"].is_empty() and total > 100, "the intact showcase map reads without problems")
	var chunks := _chunks(bytes)
	t.check(chunks.size() > 6, "the showcase map has many chunks, got %d" % chunks.size())
	var start := chunks[3]
	var broken := bytes.duplicate()
	broken[start + 24 + (chunks[4] - start - 24) / 2] ^= 0x5a
	var file := TrenchDotTdmFile.decode(broken)
	var map: Dictionary = file["map"]
	var problems := "; ".join(file["problems"])
	var kept := _node_count(map) - (1 if map["layers"].back()["name"] == TrenchDotTdmFile.RECOVERED_LAYER else 0)
	t.check(kept < total and kept > total / 2, "a damaged chunk loses only its nodes, kept %d of %d" % [kept, total])
	t.check(problems.contains("nodes stored in it are lost") and problems.contains("of %d nodes could not be read" % total), "the damage is reported, got %s" % problems)
	var data := FuncGodotParser.new().parse_tdm(map, load(SETTINGS), DAMAGED)
	t.check(data.entities.size() > 1, "the rest of the map still builds")

	broken = bytes.duplicate()
	broken[chunks[2] + 17] ^= 0x01
	file = TrenchDotTdmFile.decode(broken)
	t.check(file["map"] != null and _node_count(file["map"]) > total / 2, "a damaged chunk header skips to the next chunk")

static func _test_truncated(t) -> void:
	var bytes := FileAccess.get_file_as_bytes(DAMAGED)
	var total := _node_count(TrenchDotTdmFile.decode(bytes)["map"])
	var file := TrenchDotTdmFile.decode(bytes.slice(0, bytes.size() / 2))
	var kept := _node_count(file["map"])
	t.check(kept > 0 and kept < total, "a cut off file keeps what came before the cut, %d of %d" % [kept, total])
	t.check(not file["problems"].is_empty(), "and says it was cut off")

static func _test_refused(t) -> void:
	var bytes := FileAccess.get_file_as_bytes("res://tests/maps/basic.tdm")
	var mangled := bytes.duplicate()
	mangled.remove_at(4)
	var file := TrenchDotTdmFile.decode(mangled)
	t.check(file["map"] == null and str(file["error"]).contains("line endings"), "line ending damage is named")
	var newer := bytes.duplicate()
	newer.encode_u32(8, TrenchDotTdmFile.CONTAINER_VERSION + 1)
	file = TrenchDotTdmFile.decode(newer)
	t.check(file["map"] == null and str(file["error"]).contains("newer"), "a newer container is refused")

## A container of [param chunks], each [code][tag, payload, flags, raw size][/code], with headers that check out.
static func _container(chunks: Array) -> PackedByteArray:
	var out := PackedByteArray(TrenchDotTdmFile.MAGIC)
	out.resize(12)
	out.encode_u32(8, TrenchDotTdmFile.CONTAINER_VERSION)
	for c in chunks:
		var tag := (c[0] as String).to_ascii_buffer().decode_u32(0)
		var payload: PackedByteArray = c[1]
		var header := PackedByteArray()
		header.resize(24)
		header.encode_u32(0, TrenchDotTdmFile._SYNC)
		header.encode_u32(4, tag)
		header.encode_u32(8, c[2])
		header.encode_u32(12, c[3])
		header.encode_u32(16, payload.size())
		header.encode_u32(20, tag ^ c[2] ^ c[3] ^ payload.size() ^ TrenchDotTdmFile._CHECK_SEED)
		out.append_array(header)
		out.append_array(payload)
	return out

## A crafted container unpacks a few kilobytes into gigabytes. A chunk past the file's budget of values or of
## unpacked bytes is refused before it is decoded.
static func _test_budgets(t) -> void:
	var head := var_to_bytes({ "format": "trenchdot-map", "version": 1 })
	var node := var_to_bytes({ "nodes": [{ "id": 2, "type": "layer", "name": "Kept" }], "parents": PackedInt64Array([0]) })
	# A dictionary whose nodes array holds as many nils as the whole file may hold values, 64 MiB that zstd packs small.
	var count: int = TrenchDotTdmFile._MAX_VALUES
	var bomb := PackedByteArray()
	bomb.resize(32 + count * 4)
	bomb.encode_u32(0, TYPE_DICTIONARY)
	bomb.encode_u32(4, 1)
	bomb.encode_u32(8, TYPE_STRING)
	bomb.encode_u32(12, 5)
	for i in 5:
		bomb[16 + i] = "nodes".unicode_at(i)
	bomb.encode_u32(24, TYPE_ARRAY)
	bomb.encode_u32(28, count)
	var packed := bomb.compress(FileAccess.COMPRESSION_ZSTD)
	var file := _container([["HEAD", head, 0, head.size()], ["NODE", packed, 1, bomb.size()], ["NODE", node, 0, node.size()]])
	var start := Time.get_ticks_msec()
	var read := TrenchDotTdmFile.decode(file)
	var ms := Time.get_ticks_msec() - start
	var problems := "; ".join(read["problems"])
	t.check(file.size() < 100000 and problems.contains("holds more values than a map may"), "a %d byte file unpacking into %d values is refused, got %s" % [file.size(), count, problems])
	t.check(ms < 2000, "and quickly, in %d ms" % ms)
	t.check(read["map"] != null and read["map"]["layers"].is_empty(), "nothing after the refused chunk is read either")

	# Chunks that claim a gigabyte between them: the first uses the budget up, the next is not unpacked at all.
	file = _container([["HEAD", head, 0, head.size()], ["NODE", PackedByteArray([1, 2, 3, 4]), 0, (1 << 30) - head.size()], ["NODE", node, 0, node.size()]])
	read = TrenchDotTdmFile.decode(file)
	problems = "; ".join(read["problems"])
	t.check(problems.contains("would unpack past the 1024 MiB a map may hold") and read["map"]["layers"].is_empty(), "a chunk past the unpacked size budget is not read, got %s" % problems)
	file = _container([["HEAD", head, 0, head.size()], ["NODE", node, 0, node.size()]])
	t.check(TrenchDotTdmFile.decode(file)["map"]["layers"].size() == 1, "the same chunk inside both budgets reads")

	# Tiny entities whose values fit easily, but not what the nodes built from them take, then a chunk that still fits.
	var tiny := []
	tiny.resize(TrenchDotTdmFile._MAX_VALUES / TrenchDotTdmFile._NODE_VALUES["entity"] + 1)
	tiny.fill({ "id": 0, "type": "entity", "classname": "" })
	var parents := PackedInt64Array()
	parents.resize(tiny.size())
	parents.fill(2)
	var crowd := var_to_bytes({ "parents": parents, "nodes": tiny })
	packed = crowd.compress(FileAccess.COMPRESSION_ZSTD)
	file = _container([["HEAD", head, 0, head.size()], ["NODE", node, 0, node.size()], ["NODE", packed, 1, crowd.size()], ["NODE", node, 0, node.size()]])
	start = Time.get_ticks_msec()
	read = TrenchDotTdmFile.decode(file)
	ms = Time.get_ticks_msec() - start
	problems = "; ".join(read["problems"])
	t.check(file.size() < 100000 and problems.contains("NODE chunk at byte %d holds more values than a map may" % (12 + 48 + head.size() + node.size())), "a %d byte file of %d tiny entities is refused, got %s" % [file.size(), tiny.size(), problems])
	t.check(ms < 2000, "and quickly, in %d ms" % ms)
	t.check(read["map"]["layers"].size() == 2, "the chunks around it are read")

	var nested := var_to_bytes({ "parents": PackedInt64Array([2]), "nodes": [{ "id": 3, "type": "group", "name": "g", "children": [{ "id": 4, "type": "group", "name": "hidden" }] }] })
	file = _container([["HEAD", head, 0, head.size()], ["NODE", node, 0, node.size()], ["NODE", nested, 0, nested.size()]])
	var group: Dictionary = TrenchDotTdmFile.decode(file)["map"]["layers"][0]["children"][0]
	t.check(group["name"] == "g" and not group.has("children"), "nodes nested inside a batch are dropped, got %s" % group)

static func _test_scatter_instances(t) -> void:
	var node := { "type": "scatter", "instances": PackedInt32Array([0, 2, 1234, -1, -500, 0, 102450, 300, 0, 450, 3599, 0, -1, 0, 1250, 800]) }
	TrenchDotTdmFile._restore_instances(node)
	var expected := [[0.0, 12.34, -5.0, 1024.5, 0.0, 359.9, -0.1, 1.25], [2.0, -0.01, 0.0, 3.0, 45.0, 0.0, 0.0, 0.8]]
	t.check(node["instances"] == expected, "scatter instance columns give the saved floats exactly, got %s" % [node["instances"]])
