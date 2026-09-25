@tool
extends Node
## Checks in a headless Godot editor that the live link follows scene tabs as they are opened, closed, reloaded and
## switched. res://tests/live_link_tests.gd starts it with
## godot --headless --editor --path godot res://tests/editor/live_link_tabs.tscn -- --gt-live-link-tabs
## and it does nothing when the scene is opened any other way.

const FLAG := "--gt-live-link-tabs"
const SELF := "res://tests/editor/live_link_tabs.tscn"
const MAP_SCENE := "res://tests/editor/live_link_map.tscn"
const MAP_FILE := "res://tests/maps/basic.gtm"

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
	var link: GodotTrenchEditorIntegration

	func check(cond: bool, what: String) -> void:
		if not cond:
			failures += 1
			printerr("  FAIL: ", what)

	func frames(n: int) -> void:
		for i in n:
			await get_tree().process_frame

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
		print("GodotTrench live link scene tab tests")
		await frames(5)
		for n in get_tree().root.find_children("*", "GodotTrenchEditorIntegration", true, false):
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
		check((await save())["rebuilt"] == 1, "a save rebuilds the shown scene")

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
				background = GodotTrenchEditorIntegration.maps_under(root)[0]
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

		print("%d failures" % failures)
		get_tree().quit(1 if failures > 0 else 0)
