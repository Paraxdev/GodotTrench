@tool
class_name GodotTrenchEditorIntegration extends Node
## Editor side integration with the GodotTrench level editor:
## automatic game config export and a live link that rebuilds maps when GodotTrench saves them.
##
## Live link protocol: newline delimited JSON over TCP on 127.0.0.1, e.g.
## [code]{"event": "map_saved", "path": "C:/game/maps/level.gtm"}[/code]

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

static func ensure_setting(name: String, value: Variant, type: int, hint: int = PROPERTY_HINT_NONE, hint_string: String = "") -> void:
	if not ProjectSettings.has_setting(name):
		ProjectSettings.set_setting(name, value)
	ProjectSettings.add_property_info({ "name": name, "type": type, "hint": hint, "hint_string": hint_string })
	ProjectSettings.set_initial_value(name, value)
	ProjectSettings.set_as_basic(name, true)

func _ready() -> void:
	ensure_setting(SETTING_CONFIG, DEFAULT_CONFIG, TYPE_STRING, PROPERTY_HINT_FILE, "*.tres")
	ensure_setting(SETTING_AUTO_EXPORT, true, TYPE_BOOL)
	ensure_setting(SETTING_LIVE_LINK, true, TYPE_BOOL)
	ensure_setting(SETTING_PORT, DEFAULT_PORT, TYPE_INT)

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

func _process(_delta: float) -> void:
	if not _server:
		return
	while _server.is_connection_available():
		var peer := _server.take_connection()
		_peers.append(peer)
		_buffers[peer] = ""
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
		var chunk: String = peer.get_utf8_string(available)
		_buffers[peer] += chunk
		var buffer: String = _buffers[peer]
		while buffer.contains("\n"):
			var line := buffer.get_slice("\n", 0)
			buffer = buffer.substr(line.length() + 1)
			var reply := handle_message(line)
			peer.put_data((JSON.stringify(reply) + "\n").to_utf8_buffer())
		_buffers[peer] = buffer

func handle_message(line: String) -> Dictionary:
	var msg = JSON.parse_string(line)
	if not msg is Dictionary:
		return { "ok": false, "error": "invalid json" }
	match msg.get("event", ""):
		"hello":
			return { "ok": true, "project": ProjectSettings.globalize_path("res://"), "godot": Engine.get_version_info()["string"] }
		"map_saved":
			var rebuilt := rebuild_maps(str(msg.get("path", "")))
			return { "ok": true, "rebuilt": rebuilt }
		"export_game_config":
			export_game_config()
			return { "ok": true }
	return { "ok": false, "error": "unknown event" }

## Rebuilds every FuncGodotMap in the edited scene that references [param path]. Returns how many were rebuilt.
func rebuild_maps(path: String) -> int:
	var target := path.replace("\\", "/").to_lower()
	var root := EditorInterface.get_edited_scene_root()
	if not root:
		return 0
	var fs := EditorInterface.get_resource_filesystem()
	var local := ProjectSettings.localize_path(path)
	if local.begins_with("res://") and fs:
		fs.update_file(local)
	var count := 0
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		stack.append_array(n.get_children())
		if n is FuncGodotMap and n.auto_rebuild_on_save:
			var file: String = n.global_map_file if n.global_map_file != "" else n.local_map_file
			if file.begins_with("uid://"):
				file = ResourceUID.get_id_path(ResourceUID.text_to_id(file))
			if ProjectSettings.globalize_path(file).replace("\\", "/").to_lower() == target:
				n.build()
				count += 1
	if count > 0:
		EditorInterface.mark_scene_as_unsaved()
		print("[GodotTrench] rebuilt %d map(s) from %s" % [count, path])
	return count
