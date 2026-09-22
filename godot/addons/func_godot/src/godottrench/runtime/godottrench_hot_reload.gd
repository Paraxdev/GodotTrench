class_name GodotTrenchHotReload extends Node
## Autoload for running games: rebuilds FuncGodotMap nodes when the GodotTrench editor saves their map.
## Add it as an autoload (Project Settings > Globals) or instance it in a debug scene.
## Same protocol as the editor live link, on port 7843 by default.

@export var port := 7843
@export var enabled_in_exported_builds := false

var _server: TCPServer
var _peers: Array[StreamPeerTCP] = []
var _buffers := {}

signal map_reloaded(map: FuncGodotMap)

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	if not OS.has_feature("editor") and not enabled_in_exported_builds:
		return
	_server = TCPServer.new()
	if _server.listen(port, "127.0.0.1") != OK:
		push_warning("[GodotTrench] hot reload could not listen on port %d" % port)
		_server = null

func _exit_tree() -> void:
	if _server:
		_server.stop()

func _process(_delta: float) -> void:
	if not _server:
		return
	while _server.is_connection_available():
		var peer := _server.take_connection()
		_peers.append(peer)
		_buffers[peer] = ""
	for peer: StreamPeerTCP in _peers.duplicate():
		peer.poll()
		if peer.get_status() != StreamPeerTCP.STATUS_CONNECTED:
			if peer.get_status() in [StreamPeerTCP.STATUS_NONE, StreamPeerTCP.STATUS_ERROR]:
				_peers.erase(peer)
				_buffers.erase(peer)
			continue
		var available := peer.get_available_bytes()
		if available <= 0:
			continue
		_buffers[peer] += peer.get_utf8_string(available)
		var buffer: String = _buffers[peer]
		while buffer.contains("\n"):
			var line := buffer.get_slice("\n", 0)
			buffer = buffer.substr(line.length() + 1)
			var msg = JSON.parse_string(line)
			var reply := { "ok": false }
			if msg is Dictionary and msg.get("event", "") == "map_saved":
				reply = { "ok": true, "rebuilt": reload(str(msg.get("path", ""))) }
			peer.put_data((JSON.stringify(reply) + "\n").to_utf8_buffer())
		_buffers[peer] = buffer

## The absolute, lowercased file [param map] builds from, with uid:// resolved, for comparing against saved paths.
static func map_path_key(map: FuncGodotMap) -> String:
	var file: String = map.global_map_file if map.global_map_file != "" else map.local_map_file
	if file.begins_with("uid://"):
		var id := ResourceUID.text_to_id(file)
		file = ResourceUID.get_id_path(id) if ResourceUID.has_id(id) else ""
	if file == "":
		return ""
	return ProjectSettings.globalize_path(file).replace("\\", "/").to_lower()

## Rebuilds every map in the running scene tree that uses [param path]. Returns how many were rebuilt.
func reload(path: String) -> int:
	var target := path.replace("\\", "/").to_lower()
	var count := 0
	var stack: Array[Node] = [get_tree().root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		stack.append_array(n.get_children())
		if n is FuncGodotMap:
			var key := map_path_key(n)
			if key != "" and key == target:
				n.build()
				count += 1
				map_reloaded.emit(n)
	return count
