@tool
extends Node
## Checks in a headless Godot editor that the live link follows scene tabs as they are opened, closed, reloaded and
## switched, that live sessions follow entity definitions that change, and that an editor
## build names entity scripts without @tool. res://tests/live_link_tests.gd starts it with
## godot --headless --editor --path godot res://tests/editor/live_link_tabs.tscn -- --td-live-link-tabs
## and it does nothing when the scene is opened any other way.

const FLAG := "--td-live-link-tabs"
const SELF := "res://tests/editor/live_link_tabs.tscn"
const MAP_SCENE := "res://tests/editor/live_link_map.tscn"
const MAP_FILE := "res://tests/maps/basic.tdm"
const NOT_TOOL := "res://tests/helpers/not_tool_entity.gd"
const MAP_JSON := "res://tests/maps/basic_json.tdm"
## The TrenchDotFGDFile the demo's FGD has as a base, which brings the installed Gameplay entities pack.
const CORE_FGD := "res://addons/func_godot/fgd/trenchdot/trenchdot_fgd.tres"

func _ready() -> void:
	if not Engine.is_editor_hint() or not FLAG in OS.get_cmdline_user_args():
		return
	var base := EditorInterface.get_base_control()
	if not base.has_node("LiveLinkTabsRunner"):
		var runner := Runner.new()
		runner.name = "LiveLinkTabsRunner"
		base.add_child.call_deferred(runner)

class Runner extends Node:
	var failures := 0
	var link: TrenchDotEditorIntegration

	func check(cond: bool, what: String) -> void:
		if not cond:
			failures += 1
			printerr("  FAIL: ", what)

	func frames(n: int) -> void:
		for i in n:
			await get_tree().process_frame

	## Waits up to [param limit] frames for [param done] to hold, and returns whether it did.
	func until(done: Callable, limit := 600) -> bool:
		for i in limit:
			if done.call():
				return true
			await get_tree().process_frame
		return done.call()

	func show_scene(path: String) -> void:
		EditorInterface.open_scene_from_path(path)
		await frames(5)

	func shown_maps() -> Array:
		return link.status()["maps"].map(func(m: Dictionary) -> String: return m["path"])

	func save() -> Dictionary:
		return await link.handle_message(JSON.stringify({ "event": "map_saved", "path": ProjectSettings.globalize_path(MAP_FILE) }))

	func _ready() -> void:
		_run.call_deferred()

	func _run() -> void:
		print("TrenchDot live link scene tab tests")
		await frames(5)
		for n in get_tree().root.find_children("*", "TrenchDotEditorIntegration", true, false):
			link = n
		if not link:
			printerr("  FAIL: the addon's live link is not running")
			get_tree().quit(1)
			return
		# The test calls the link directly, the port stays free for a Godot editor someone is working in.
		link.stop_live_link()
		var map_file := ProjectSettings.globalize_path(MAP_FILE)

		await show_scene(MAP_SCENE)
		check(shown_maps() == [map_file], "an opened scene tab reports its map, got %s" % [shown_maps()])
		check(link.status()["scene"] == MAP_SCENE, "status names the shown scene, got %s" % link.status()["scene"])
		var built := await save()
		check(built["rebuilt"] == 1, "a save rebuilds the shown scene")
		check(built.get("settings") == ["res://demo/demo_map_settings.tres"] and not built.has("missing_classes"), "a save names the map settings it built with, got %s" % built)

		EditorInterface.close_scene()
		await frames(5)
		check(shown_maps().is_empty(), "closing the tab drops its map, got %s" % [shown_maps()])
		var reply := await save()
		check(reply["rebuilt"] == 0 and reply["scene"] == SELF and not reply.has("waiting"), "a save with the tab closed names the shown scene, got %s" % reply)

		await show_scene(MAP_SCENE)
		check(shown_maps() == [map_file], "a reopened scene tab reports its map again, got %s" % [shown_maps()])
		check((await save())["rebuilt"] == 1, "a save rebuilds the reopened scene")

		EditorInterface.reload_scene_from_path(MAP_SCENE)
		await frames(5)
		check(shown_maps() == [map_file], "a reloaded scene tab reports its map, got %s" % [shown_maps()])
		check((await save())["rebuilt"] == 1, "a save rebuilds the reloaded scene")

		await show_scene(SELF)
		check(shown_maps().is_empty(), "a background tab's map is not reported, got %s" % [shown_maps()])
		var background: FuncGodotMap
		for root in EditorInterface.get_open_scene_roots():
			if root.scene_file_path == MAP_SCENE:
				background = TrenchDotEditorIntegration.maps_under(root)[0]
		var builds := [0]
		background.build_complete.connect(func() -> void: builds[0] += 1)
		reply = await save()
		check(reply["rebuilt"] == 0 and reply.get("waiting", []) == [MAP_SCENE], "a save names the background tab that uses the map, got %s" % reply)
		check(builds[0] == 0, "a background tab does not build while hidden")

		await show_scene(MAP_SCENE)
		for i in 600:
			if builds[0] > 0:
				break
			await get_tree().process_frame
		check(builds[0] == 1, "showing the tab again builds the save it missed, %d builds" % builds[0])
		await show_scene(SELF)
		await show_scene(MAP_SCENE)
		await frames(30)
		check(builds[0] == 1, "a missed save is built once, %d builds" % builds[0])
		await _check_definition_hooks()
		_check_tool_scripts()

		print("%d failures" % failures)
		get_tree().quit(1 if failures > 0 else 0)

	func _check_tool_scripts() -> void:
		var def := FuncGodotFGDPointClass.new()
		def.classname = "td_tests_not_tool"
		def.node_class = "Node3D"
		def.script_class = load(NOT_TOOL)
		var fgd := FuncGodotFGDFile.new()
		fgd.base_fgd_files = [load("res://demo/demo_fgd.tres")]
		fgd.entity_definitions = [def]
		var settings: FuncGodotMapSettings = load("res://demo/demo_map_settings.tres").duplicate()
		settings.entity_fgd = fgd
		var map := FuncGodotMap.new()
		map.map_settings = settings
		map.local_map_file = MAP_FILE
		var entity := { "id": 2, "type": "entity", "classname": "td_tests_not_tool", "origin": [0.0, 0.0, 0.0], "angles": [0.0, 0.0, 0.0], "properties": {} }
		var report := map.build_from_text(JSON.stringify({ "format": "trenchdot-map", "properties": {}, "layers": [{ "type": "layer", "id": 1, "children": [entity] }] }))
		var lines := Array(report["warnings"]).filter(func(w: String) -> bool: return w.contains(NOT_TOOL) and w.contains("has no @tool"))
		check(lines.size() == 1, "an editor build names the entity script without @tool, got %s" % [report["warnings"]])
		var built := map.find_children("*td_tests_not_tool*", "", true, false)
		check(built.size() == 1 and not built[0].has_meta(&"applied"), "and does not call its placeholder, got %s" % [built])
		map.free()

	func live(event: String, extra := {}) -> Dictionary:
		var msg := { "event": event, "path": ProjectSettings.globalize_path(MAP_FILE) }
		msg.merge(extra)
		return await link.handle_message(JSON.stringify(msg))

	func door() -> Node:
		var maps := link.maps_for(MAP_FILE)
		return TrenchDotBuild.nodes_by_id(maps[0]).get(9) if not maps.is_empty() else null

	func _check_definition_hooks() -> void:
		var fs := EditorInterface.get_resource_filesystem()
		check(fs.script_classes_updated.is_connected(link.check_definitions) and link.plugin.resource_saved.is_connected(link._on_resource_saved), "the live link checks definitions when script classes change and when resources are saved")
		check(TrenchDotCSharp.keep_file_list, "and keeps the list of C# source files in between")
		var reply := await live("live_begin", { "text": FileAccess.get_file_as_string(MAP_JSON) })
		check(reply.get("ok") == true and door() is TDDoor, "a live session starts with a door, got %s %s" % [reply, door()])
		var core: TrenchDotFGDFile = load(CORE_FGD)
		var installed := core.pack_fgd_path
		core.pack_fgd_path = "res://tests/td_tests_no_such_pack.tres"
		# Note: only saves through the editor's own save path emit resource_saved, ResourceSaver.save does not.
		link.plugin.resource_saved.emit(core)
		check(await until(func() -> bool: return door() != null and not door() is TDDoor), "a saved resource makes the session build with the definitions it has now, got %s" % door())
		core.pack_fgd_path = installed
		link.plugin.resource_saved.emit(core)
		check(await until(func() -> bool: return door() is TDDoor), "and again once the pack is back, got %s" % door())
		await live("live_end")
