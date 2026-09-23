@tool
class_name GodotTrenchEditorIntegration extends Node
## Editor side integration with the GodotTrench level editor:
## automatic game config export and a live link that rebuilds maps when GodotTrench saves them.
##
## Live link protocol: newline delimited JSON over TCP on 127.0.0.1, every message answered by one line that echoes its
## [code]seq[/code]. Events:
## [code]status[/code] (project, Godot version, pid and the maps of the edited scene with their build epoch),
## [code]map_saved {path}[/code] and [code]build {path, text}[/code] (full builds), [code]focus[/code],
## [code]export_game_config[/code], and for live mode [code]live_begin {path, text, content}[/code],
## [code]live_resync {path, text}[/code], [code]live_delta {path, ops}[/code] and [code]live_end {path, revert}[/code]
## (see [GodotTrenchLiveSession]).

const SETTING_CONFIG := "godottrench/game_config"
const SETTING_AUTO_EXPORT := "godottrench/auto_export_game_config"
const SETTING_LIVE_LINK := "godottrench/live_link_enabled"
const SETTING_PORT := "godottrench/live_link_port"
const DEFAULT_CONFIG := "res://addons/func_godot/game_config/godottrench/godottrench_game_config.tres"
const DEFAULT_PORT := 7842

var plugin: EditorPlugin
var _server: TCPServer
var _peers: Array[StreamPeerTCP] = []
var _buffers: Dictionary = {}
var _export_timer: Timer
## Path key to build epoch. Goes up whenever a map using the path is built outside a live session or the maps change.
var _epochs: Dictionary = {}
## Path key to the instance ids of the FuncGodotMap nodes that used it at the last status.
var _map_ids: Dictionary = {}
var _sessions: Dictionary = {}

static func ensure_setting(name: String, value: Variant, type: int, hint: int = PROPERTY_HINT_NONE, hint_string: String = "") -> void:
	if not ProjectSettings.has_setting(name):
		ProjectSettings.set_setting(name, value)
	ProjectSettings.add_property_info({ "name": name, "type": type, "hint": hint, "hint_string": hint_string })
	ProjectSettings.set_initial_value(name, value)
	ProjectSettings.set_as_basic(name, true)

## Comparable form of a map path: global, forward slashes, lower case.
static func path_key(path: String) -> String:
	var global := ProjectSettings.globalize_path(path) if path.begins_with("res://") else path
	return global.replace("\\", "/").trim_suffix("/").to_lower()

## The file a map node builds from, with uid:// resolved.
static func resolved_map_path(map: FuncGodotMap) -> String:
	var file: String = map.global_map_file if map.global_map_file != "" else map.local_map_file
	if file.begins_with("uid://"):
		file = ResourceUID.get_id_path(ResourceUID.text_to_id(file))
	return file

func _ready() -> void:
	ensure_setting(SETTING_CONFIG, DEFAULT_CONFIG, TYPE_STRING, PROPERTY_HINT_FILE, "*.tres")
	ensure_setting(SETTING_AUTO_EXPORT, true, TYPE_BOOL)
	ensure_setting(SETTING_LIVE_LINK, true, TYPE_BOOL)
	ensure_setting(SETTING_PORT, DEFAULT_PORT, TYPE_INT)
	ensure_setting(GodotTrenchBuild.SETTING_THREADED, true, TYPE_BOOL)
	ensure_setting(GodotTrenchLiveSession.SETTING_CHUNK_SIZE, GodotTrenchLiveSession.DEFAULT_CHUNK_SIZE, TYPE_FLOAT, PROPERTY_HINT_RANGE, "2,256,1,suffix:m")
	ensure_setting(GodotTrenchCSharp.SETTING, PackedStringArray(["res://"]), TYPE_PACKED_STRING_ARRAY, PROPERTY_HINT_TYPE_STRING, "%d/%d:" % [TYPE_STRING, PROPERTY_HINT_DIR])

	_export_timer = Timer.new()
	_export_timer.one_shot = true
	_export_timer.wait_time = 1.5
	_export_timer.timeout.connect(export_game_config)
	add_child(_export_timer)

	var fs := EditorInterface.get_resource_filesystem()
	if fs and not fs.filesystem_changed.is_connected(_on_filesystem_changed):
		fs.filesystem_changed.connect(_on_filesystem_changed)

	if ProjectSettings.get_setting(SETTING_LIVE_LINK, true):
		start_live_link(int(ProjectSettings.get_setting(SETTING_PORT, DEFAULT_PORT)))

func _exit_tree() -> void:
	if _server:
		_server.stop()
	for p in _peers:
		p.disconnect_from_host()
	_peers.clear()

func _on_filesystem_changed() -> void:
	if ProjectSettings.get_setting(SETTING_AUTO_EXPORT, true):
		_export_timer.start()

func load_config() -> GodotTrenchGameConfig:
	var path: String = ProjectSettings.get_setting(SETTING_CONFIG, DEFAULT_CONFIG)
	if not ResourceLoader.exists(path):
		push_warning("[GodotTrench] game config %s not found" % path)
		return null
	return load(path) as GodotTrenchGameConfig

func export_game_config() -> void:
	var config := load_config()
	if config:
		config.export_file()

func start_live_link(port: int) -> void:
	_server = TCPServer.new()
	var err := _server.listen(port, "127.0.0.1")
	if err != OK:
		push_warning("[GodotTrench] live link could not listen on port %d (%s)" % [port, error_string(err)])
		_server = null
		return
	print("[GodotTrench] live link listening on 127.0.0.1:%d" % port)

## Complete lines at the start of [param buffer], split on the newline byte so multibyte characters are never cut.
## The rest stays in the buffer.
static func take_lines(buffer: PackedByteArray) -> Array:
	var lines: PackedStringArray = []
	var start := 0
	var newline := buffer.find(10)
	while newline >= 0:
		lines.append(buffer.slice(start, newline).get_string_from_utf8())
		start = newline + 1
		newline = buffer.find(10, start)
	return [lines, buffer.slice(start)]

func _process(_delta: float) -> void:
	for session: GodotTrenchLiveSession in _sessions.values():
		session.process()
	if not _server:
		return
	while _server.is_connection_available():
		var peer := _server.take_connection()
		_peers.append(peer)
		_buffers[peer] = PackedByteArray()
	for peer: StreamPeerTCP in _peers.duplicate():
		peer.poll()
		var status: StreamPeerTCP.Status = peer.get_status()
		if status != StreamPeerTCP.STATUS_CONNECTED:
			if status == StreamPeerTCP.STATUS_NONE or status == StreamPeerTCP.STATUS_ERROR:
				_peers.erase(peer)
				_buffers.erase(peer)
			continue
		var available: int = peer.get_available_bytes()
		if available <= 0:
			continue
		var received: Array = peer.get_data(available)
		if received[0] != OK:
			continue
		var buffer: PackedByteArray = _buffers[peer]
		buffer.append_array(received[1])
		var split := take_lines(buffer)
		_buffers[peer] = split[1]
		for line in split[0]:
			var reply := handle_message(line)
			peer.put_data((JSON.stringify(reply) + "\n").to_utf8_buffer())

func handle_message(line: String) -> Dictionary:
	var msg = JSON.parse_string(line)
	if not msg is Dictionary:
		return { "ok": false, "error": "invalid json" }
	var reply := _handle(msg)
	# JSON numbers parse as floats, the editor expects the seq back as an integer.
	if msg.get("seq") is float:
		reply["seq"] = int(msg["seq"])
	return reply

func _handle(msg: Dictionary) -> Dictionary:
	var path := str(msg.get("path", ""))
	match msg.get("event", ""):
		"hello":
			return { "ok": true, "project": ProjectSettings.globalize_path("res://"), "godot": Engine.get_version_info()["string"] }
		"status":
			return status()
		"map_saved":
			_sessions.erase(path_key(path))
			return { "ok": true, "rebuilt": rebuild_maps(path) }
		"build":
			_sessions.erase(path_key(path))
			return { "ok": true, "rebuilt": rebuild_maps(path, str(msg.get("text", ""))) }
		"focus":
			DisplayServer.window_move_to_foreground()
			return { "ok": true }
		"export_game_config":
			export_game_config()
			return { "ok": true }
		"inspect":
			return inspect(path, msg.get("ids", []))
		"live_begin", "live_resync":
			var maps := maps_for(path)
			if maps.is_empty():
				return { "ok": false, "error": "no open scene uses this map" }
			var session := GodotTrenchLiveSession.new(_local_path(path), maps)
			if not session.begin(str(msg.get("text", "")), msg.get("content")):
				return { "ok": false, "error": "invalid map" }
			_sessions[path_key(path)] = session
			return { "ok": true, "epoch": _epochs.get(path_key(path), 0) }
		"live_delta":
			var session: GodotTrenchLiveSession = _sessions.get(path_key(path))
			if not session or not session.apply(msg.get("ops", [])):
				_sessions.erase(path_key(path))
				return { "ok": false, "resync": true }
			return { "ok": true, "epoch": _epochs.get(path_key(path), 0) }
		"live_end":
			_sessions.erase(path_key(path))
			if msg.get("revert", false):
				rebuild_maps(path)
			return { "ok": true }
	return { "ok": false, "error": "unknown event" }

func _local_path(path: String) -> String:
	var local := ProjectSettings.localize_path(path)
	return local if local.begins_with("res://") else path

## FuncGodotMap nodes of the edited scene that rebuild for GodotTrench and use [param path].
func maps_for(path: String) -> Array[FuncGodotMap]:
	var target := path_key(path)
	var out: Array[FuncGodotMap] = []
	out.assign(_scene_maps().filter(func(m: FuncGodotMap) -> bool: return path_key(resolved_map_path(m)) == target))
	return out

func _scene_maps() -> Array[FuncGodotMap]:
	var out: Array[FuncGodotMap] = []
	var root := EditorInterface.get_edited_scene_root()
	if not root:
		return out
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		stack.append_array(n.get_children())
		if n is FuncGodotMap and n.auto_rebuild_on_save and resolved_map_path(n) != "":
			out.append(n)
	return out

func status() -> Dictionary:
	var ids := {}
	var files := {}
	for map in _scene_maps():
		var file := resolved_map_path(map)
		var key := path_key(file)
		if not ids.has(key):
			ids[key] = []
			files[key] = ProjectSettings.globalize_path(file)
		ids[key].append(map.get_instance_id())
		if not map.build_complete.is_connected(_on_map_built):
			map.build_complete.connect(_on_map_built.bind(map))
	for key in ids:
		ids[key].sort()
		if _map_ids.get(key, []) != ids[key]:
			_bump(key)
	_map_ids = ids
	for key in _sessions.keys():
		if not ids.has(key):
			_sessions.erase(key)
	var maps := []
	for key in files:
		maps.append({ "path": files[key], "epoch": _epochs.get(key, 0) })
	return {
		"ok": true, "project": ProjectSettings.globalize_path("res://"), "godot": Engine.get_version_info()["string"],
		"pid": OS.get_process_id(), "maps": maps,
	}

## Generated nodes of the map node ids [param ids] in the first map using [param path], for checking live updates.
func inspect(path: String, ids: Array) -> Dictionary:
	var maps := maps_for(path)
	if maps.is_empty():
		return { "ok": false, "error": "no open scene uses this map" }
	var by_id := GodotTrenchBuild.nodes_by_id(maps[0])
	var nodes := {}
	for raw in ids:
		var node := by_id.get(int(raw)) as Node
		if node:
			var info := { "name": String(node.name), "class": node.get_class(), "children": node.get_child_count(), "instance": node.get_instance_id() }
			if node is Node3D:
				var p: Vector3 = node.position
				info["position"] = [p.x, p.y, p.z]
			nodes[str(int(raw))] = info
	var session: GodotTrenchLiveSession = _sessions.get(path_key(path))
	return { "ok": true, "nodes": nodes, "live": session != null, "pending": session != null and session.pending() }

func _bump(key: String) -> void:
	_epochs[key] = int(_epochs.get(key, 0)) + 1
	_sessions.erase(key)

func _on_map_built(map: FuncGodotMap) -> void:
	if GodotTrenchLiveSession.building == 0 and is_instance_valid(map):
		_bump(path_key(resolved_map_path(map)))

## Fully rebuilds every FuncGodotMap in the edited scene that uses [param path], from [param text] when given (unsaved
## edits) or from the file. Returns how many were rebuilt. Only nodes inside the FuncGodotMap nodes are replaced.
func rebuild_maps(path: String, text: String = "") -> int:
	var fs := EditorInterface.get_resource_filesystem()
	var local := ProjectSettings.localize_path(path)
	if local.begins_with("res://") and fs and text == "":
		fs.update_file(local)
	var maps := maps_for(path)
	for map in maps:
		if text != "":
			map.build_from_text(text)
		else:
			map.build()
	if not maps.is_empty():
		EditorInterface.mark_scene_as_unsaved()
		print("[GodotTrench] rebuilt %d map(s) from %s" % [maps.size(), "unsaved edits of " + path if text != "" else path])
	return maps.size()
