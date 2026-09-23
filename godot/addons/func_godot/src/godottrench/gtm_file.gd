class_name GodotTrenchGtmFile extends RefCounted
## Reads .gtm files: the chunked, zstd compressed container the editor saves and the JSON text of older maps. The byte
## layout is in docs/format/container.md, the editor's side in crates/gt_doc/src/binary.rs. Chunk payloads use Godot's own
## Variant encoding, so each one decodes with a single [method @GlobalScope.bytes_to_var] call.

const MAGIC := [0x89, 0x47, 0x54, 0x4D, 0x0D, 0x0A, 0x1A, 0x0A]
const CONTAINER_VERSION := 1
const _SYNC := 0x43544789
const _HEADER_SIZE := 24
const _CHECK_SEED := 0x47544D43
const _FLAG_ZSTD := 1
const _MAX_RAW := 1 << 30
## The same limit as MAX_TREE_DEPTH in crates/gt_doc/src/binary.rs.
const _MAX_TREE_DEPTH := 60
const RECOVERED_LAYER := "Recovered"

## Reads [param path], or the copy Godot imported when only that is there, as in an exported game. Returns
## [code]{ "map": Dictionary or null, "error": String, "problems": PackedStringArray, "content": int }[/code], see
## [method decode].
static func read(path: String) -> Dictionary:
	if FileAccess.file_exists(path):
		return decode(FileAccess.get_file_as_bytes(path))
	var imported: Resource = load(path) if ResourceLoader.exists(path) else null
	if imported is QuakeMapFile:
		return decode(imported.map_bytes if not imported.map_bytes.is_empty() else imported.map_data.to_utf8_buffer())
	return _failed("cannot read %s" % path)

## Reads [param path] and reports what a damaged file lost as warnings. Returns the map, or null after reporting why
## it cannot be read.
static func load_map(path: String) -> Variant:
	var file := read(path)
	for problem in file["problems"]:
		push_warning("[GTM] %s: %s" % [path, problem])
	if file["map"] == null:
		push_error("[GTM] %s: %s" % [path, file["error"]])
	return file["map"]

static func is_binary(bytes: PackedByteArray) -> bool:
	return bytes.size() >= 4 and bytes.slice(0, 4) == PackedByteArray(MAGIC.slice(0, 4))

## Decodes a whole file. [code]map[/code] is the map in the shape JSON.parse_string gives for a JSON map, except that
## terrain [code]heights[/code], [code]splat[/code] and [code]holes[/code] are PackedByteArray instead of base64 and long
## number arrays may be packed arrays. [code]problems[/code] says what a damaged file lost, nodes whose parent was lost
## sit in a layer named [constant RECOVERED_LAYER]. [code]content[/code] is the content id of the END chunk, 0 for JSON.
static func decode(bytes: PackedByteArray) -> Dictionary:
	if not is_binary(bytes):
		var text := bytes.get_string_from_utf8().trim_prefix("﻿")
		var json = JSON.parse_string(text) if text != "" else null
		return { "map": json, "error": "" if json is Dictionary else "not a GodotTrench map", "problems": PackedStringArray(), "content": 0 }
	if bytes.slice(0, 5) == PackedByteArray([0x89, 0x47, 0x54, 0x4D, 0x0A]) or bytes.slice(0, 6) == PackedByteArray([0x89, 0x47, 0x54, 0x4D, 0x0D, 0x0D]):
		return _failed("the file was saved or transferred as text and its line endings were changed, restore it from version control")
	var version := bytes.decode_u32(8) if bytes.size() >= 12 else CONTAINER_VERSION
	if version > CONTAINER_VERSION:
		return _failed("container version %d is newer than this addon supports (%d), update the addon" % [version, CONTAINER_VERSION])

	var problems := PackedStringArray()
	var head = null
	var end = null
	var nodes := []
	var parents := PackedInt64Array()
	var chunks := 0
	var damaged := 0
	var pos := MAGIC.size() + 4
	while pos < bytes.size():
		var h := _header(bytes, pos)
		if h.is_empty():
			var next := _next_sync(bytes, pos + 1)
			problems.append("bytes %d to %d are damaged and were skipped" % [pos, next])
			damaged += 1
			pos = next
			continue
		var start := pos + _HEADER_SIZE
		var stored: int = h["stored"]
		if start + stored > bytes.size():
			problems.append("the file ends inside the %s chunk at byte %d" % [h["tag"].strip_edges(), pos])
			damaged += 1
			pos = _next_sync(bytes, start)
			continue
		var at := pos
		pos = start + stored
		chunks += 1
		var tag: String = h["tag"]
		if not tag in ["HEAD", "NODE", "END "]:
			continue
		var value = _payload(bytes.slice(start, pos), h)
		if not value is Dictionary:
			var lost: String = { "NODE": ", the nodes stored in it are lost", "HEAD": ", the worldspawn properties and editor state are lost" }.get(tag, "")
			problems.append("the %s chunk at byte %d is damaged%s" % [tag.strip_edges(), at, lost])
			damaged += 1
			continue
		match tag:
			"HEAD":
				head = value
			"NODE":
				var batch_nodes: Array = value.get("nodes", [])
				var batch_parents: PackedInt64Array = value.get("parents", PackedInt64Array())
				for i in mini(batch_nodes.size(), batch_parents.size()):
					if batch_nodes[i] is Dictionary:
						nodes.append(batch_nodes[i])
						parents.append(batch_parents[i])
			_:
				end = value

	if head == null:
		if end == null:
			return _failed("the map header is damaged and the file has no intact end chunk to take the map version from")
		head = { "format": GodotTrenchParser.FORMAT_NAME, "version": end.get("version", 0) }
	if end == null:
		problems.append("the file ends early, anything saved after the last complete chunk is lost" if damaged == 0 else "the end chunk is missing, so the number of lost nodes is unknown")
	else:
		var expected := int(end.get("nodes", 0))
		if expected > nodes.size():
			problems.append("%d of %d nodes could not be read" % [expected - nodes.size(), expected])
		var expected_chunks := int(end.get("chunks", 0))
		if damaged == 0 and expected_chunks > chunks - 1:
			problems.append("%d chunks are missing" % (expected_chunks - (chunks - 1)))

	# Id to [node, depth]. Nesting past _MAX_TREE_DEPTH only comes from a crafted file and is cut into orphans.
	var by_id := {}
	var layers := []
	var orphans := []
	for i in nodes.size():
		var node: Dictionary = nodes[i]
		if node.get("type", "") == "scatter":
			_restore_instances(node)
		var parent := parents[i]
		var depth := 0
		if parent == 0:
			layers.append(node)
		elif by_id.has(parent) and by_id[parent][1] < _MAX_TREE_DEPTH:
			var owner: Dictionary = by_id[parent][0]
			depth = by_id[parent][1] + 1
			if owner.has("children"):
				owner["children"].append(node)
			else:
				owner["children"] = [node]
		else:
			orphans.append(node)
			depth = 1
		by_id[int(node.get("id", 0))] = [node, depth]
	if not orphans.is_empty():
		layers.append({ "id": 0, "type": "layer", "name": RECOVERED_LAYER, "color": "#e0a030ff", "omit_from_export": false, "children": orphans })
		problems.append("%d nodes whose parent was lost were moved to the layer \"%s\"" % [orphans.size(), RECOVERED_LAYER])
	var map: Dictionary = head
	map["layers"] = layers
	return { "map": map, "error": "", "problems": problems, "content": int(end.get("content", 0)) if end != null else 0 }

## The content id a scene built from [param path] records, see [constant GodotTrenchBuild.SOURCE_HASH_META]. For a
## binary map that is the id in its END chunk, which the editor sends along when a live session starts, for a JSON
## map the hash of its text.
static func content_id(path: String) -> int:
	var bytes := FileAccess.get_file_as_bytes(path)
	if not is_binary(bytes):
		return bytes.get_string_from_utf8().hash()
	var pos := MAGIC.size() + 4
	while pos < bytes.size():
		var h := _header(bytes, pos)
		if h.is_empty():
			return 0
		var start: int = pos + _HEADER_SIZE
		pos = start + int(h["stored"])
		if h["tag"] == "END " and pos <= bytes.size():
			var end = _payload(bytes.slice(start, pos), h)
			return int(end.get("content", 0)) if end is Dictionary else 0
	return 0

static func _failed(error: String) -> Dictionary:
	return { "map": null, "error": error, "problems": PackedStringArray(), "content": 0 }

static func _header(bytes: PackedByteArray, at: int) -> Dictionary:
	if at + _HEADER_SIZE > bytes.size() or bytes.decode_u32(at) != _SYNC:
		return {}
	var tag_bits := bytes.decode_u32(at + 4)
	var flags := bytes.decode_u32(at + 8)
	var raw := bytes.decode_u32(at + 12)
	var stored := bytes.decode_u32(at + 16)
	if tag_bits ^ flags ^ raw ^ stored ^ _CHECK_SEED != bytes.decode_u32(at + 20) or raw > _MAX_RAW:
		return {}
	var tag := bytes.slice(at + 4, at + 8)
	for c in tag:
		if c < 0x20 or c > 0x7E:
			return {}
	return { "tag": tag.get_string_from_ascii(), "flags": flags, "raw": raw, "stored": stored }

static func _next_sync(bytes: PackedByteArray, from: int) -> int:
	var at := bytes.find(0x89, from)
	while at >= 0 and at + 4 <= bytes.size():
		if bytes.decode_u32(at) == _SYNC:
			return at
		at = bytes.find(0x89, at + 1)
	return bytes.size()

static func _payload(data: PackedByteArray, h: Dictionary) -> Variant:
	var raw_size: int = h["raw"]
	var raw := data
	if int(h["flags"]) & _FLAG_ZSTD:
		if raw_size == 0 or data.is_empty():
			return null
		raw = data.decompress(raw_size, FileAccess.COMPRESSION_ZSTD)
	if raw.size() != raw_size:
		return null
	return bytes_to_var(raw)

## Scatter instances stored as integer columns go back to one [code][item, x, y, z, pitch, yaw, roll, scale][/code]
## array each. The steps are INSTANCE_SCALES in crates/gt_doc/src/variant.rs, and the divisions give exactly the
## floats the editor saved.
static func _restore_instances(node: Dictionary) -> void:
	var flat = node.get("instances")
	if not flat is PackedInt32Array or flat.size() % 8 != 0:
		return
	var n: int = flat.size() / 8
	var out := []
	out.resize(n)
	for i in n:
		out[i] = [float(flat[i]), flat[n + i] / 100.0, flat[2 * n + i] / 100.0, flat[3 * n + i] / 100.0, flat[4 * n + i] / 10.0, flat[5 * n + i] / 10.0, flat[6 * n + i] / 10.0, flat[7 * n + i] / 1000.0]
	node["instances"] = out
