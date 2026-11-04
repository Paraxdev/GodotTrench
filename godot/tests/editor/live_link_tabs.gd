@tool
extends Node
## Checks in a headless Godot editor that the live link follows scene tabs as they are opened, closed, reloaded and
## switched, that live sessions follow entity definitions that change or appear on disk and prefabs that are saved, that
## a pack copied in is read once Godot lists it, that prop_physics shows a preview the scene does not save, and that an
## editor build names entity scripts without @tool. res://tests/live_link_tests.gd starts it with
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
## A definition and its script class written while a session runs, removed again afterwards.
const NEW_DEFS := "res://tests/td_tests_new_definitions"
const NEW_CLASS := "TDTestsNewEntity"
## A pack written while Godot runs, removed again afterwards.
const LATE_PACK := "res://tests/td_tests_late_pack"
const LATE_CLASS := "TDTestsLateBase"

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
		await _check_scan()
		await _check_prefab_save()
		await _check_late_pack()
		_check_prop_physics_preview()
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
		var nudge := { "ops": [{ "op": "translate", "ids": [15], "offset": [0.0, 0.0, 0.0] }] }
		reply = await live("live_delta", nudge)
		check(reply.get("ok") == true and not reply.has("unresolved"), "a live reply leaves out the classes of a build the session started by itself, got %s" % reply)
		var session: TrenchDotLiveSession = link._sessions.get(TrenchDotEditorIntegration.path_key(ProjectSettings.globalize_path(MAP_FILE)))
		var door_edit := { "ops": [{ "op": "set", "id": 9, "parent": 1, "index": 9, "node": session._nodes[9].duplicate(true) }] }
		reply = await live("live_delta", door_edit)
		check(reply.get("unresolved") == ["func_door"], "and names those of the entities it rebuilds itself, got %s" % reply)
		core.pack_fgd_path = installed
		link.plugin.resource_saved.emit(core)
		check(await until(func() -> bool: return door() is TDDoor), "and again once the pack is back, got %s" % door())
		reply = await live("live_delta", nudge)
		check(reply.get("ok") == true and not reply.has("unresolved"), "after which a live reply names none, got %s" % reply)
		await live("live_end")

	## A definition and the script class it names appear on disk while a session runs, the way installing an entity
	## pack or writing a custom entity does, and the scan event makes the session build them.
	func _check_scan() -> void:
		var auto_export: Variant = ProjectSettings.get_setting(TrenchDotEditorIntegration.SETTING_AUTO_EXPORT)
		ProjectSettings.set_setting(TrenchDotEditorIntegration.SETTING_AUTO_EXPORT, false)
		var map: FuncGodotMap = link.maps_for(MAP_FILE)[0]
		var settings_before := map.map_settings
		var fgd := TrenchDotFGDFile.new()
		fgd.base_fgd_files = [load("res://demo/demo_fgd.tres")]
		fgd.pack_fgd_path = NEW_DEFS.path_join("fgd.tres")
		var settings: FuncGodotMapSettings = load("res://demo/demo_map_settings.tres").duplicate()
		settings.entity_fgd = fgd
		map.map_settings = settings
		var entity := { "id": 2, "type": "entity", "classname": "td_tests_new_entity", "origin": [0.0, 0.0, 0.0], "angles": [0.0, 0.0, 0.0], "properties": {} }
		var text := JSON.stringify({ "format": "trenchdot-map", "properties": {}, "layers": [{ "type": "layer", "id": 1, "children": [entity] }] })
		var reply := await live("live_begin", { "text": text })
		check(reply.get("unresolved") == ["td_tests_new_entity"], "a live session started before the definition exists names its class, got %s" % reply)

		DirAccess.make_dir_recursive_absolute(NEW_DEFS)
		var script := FileAccess.open(NEW_DEFS.path_join("new_entity.gd"), FileAccess.WRITE)
		script.store_string("@tool\nclass_name %s extends StaticBody3D\n" % NEW_CLASS)
		script.close()
		var def := FuncGodotFGDPointClass.new()
		def.classname = "td_tests_new_entity"
		def.node_class = NEW_CLASS
		var pack := FuncGodotFGDFile.new()
		pack.entity_definitions = [def]
		ResourceSaver.save(pack, fgd.pack_fgd_path)
		reply = await live("scan")
		check(reply.get("ok") == true, "the scan event is answered, got %s" % reply)
		var built := func() -> Node: return TrenchDotBuild.nodes_by_id(map).get(2)
		var is_new := func() -> bool:
			var node: Node = built.call()
			return node is StaticBody3D and node.get_script() != null and node.get_script().get_global_name() == NEW_CLASS
		check(await until(is_new, 1200), "after the scan the session builds the entity with its new class, got %s" % built.call())
		var number = reply.get("scan")
		check(number is int and number > 0 and link._scans_done == number, "the reply numbers the scan, which counts as done then, got %s and %d" % [reply, link._scans_done])
		reply = await live("live_delta", { "ops": [{ "op": "set", "id": 2, "parent": 1, "index": 0, "node": entity }] })
		check(reply.get("ok") == true and not reply.has("unresolved"), "and live edits of it no longer name it, got %s" % reply)

		# Godot reads modified times in whole seconds, the edit has to land in a later one to be seen.
		var saved_at := FileAccess.get_modified_time(fgd.pack_fgd_path)
		def.node_class = "Area3D"
		var copy := "user://td_tests_edited_fgd.tres"
		ResourceSaver.save(pack, copy)
		var edited := FileAccess.get_file_as_string(copy)
		DirAccess.remove_absolute(copy)
		OS.delay_msec(maxi(0, int((saved_at + 1.05 - Time.get_unix_time_from_system()) * 1000.0)))
		var out := FileAccess.open(fgd.pack_fgd_path, FileAccess.WRITE)
		out.store_string(edited)
		out.close()
		await live("scan")
		check(await until(func() -> bool: return built.call() is Area3D, 1200), "the scan event reads a definition edited outside Godot again, got %s" % built.call())

		await live("live_end")
		check(link.status().get("scanned") == number + 1, "the status names the last scan done, got %s" % link.status().get("scanned"))
		map.map_settings = settings_before
		for file in DirAccess.get_files_at(NEW_DEFS):
			DirAccess.remove_absolute(NEW_DEFS.path_join(file))
		DirAccess.remove_absolute(NEW_DEFS)
		var fs := EditorInterface.get_resource_filesystem()
		fs.scan()
		await until(func() -> bool: return not fs.is_scanning() and not ProjectSettings.get_global_class_list().any(func(c): return c["class"] == NEW_CLASS), 1200)
		ProjectSettings.set_setting(TrenchDotEditorIntegration.SETTING_AUTO_EXPORT, auto_export)
		# Changed settings start the link again.
		link.stop_live_link()

	func _check_prefab_save() -> void:
		var save_prefab := { "event": "map_saved", "path": ProjectSettings.globalize_path("res://tests/maps/prefab.tdm") }
		var map: FuncGodotMap = link.maps_for(MAP_FILE)[0]
		map.build()
		var plight := map.find_child("entity_p1-plight", true, false)
		var reply := await link.handle_message(JSON.stringify(save_prefab))
		var rebuilt := map.find_child("entity_p1-plight", true, false)
		check(reply.get("rebuilt") == 1 and plight != null and rebuilt != null and rebuilt != plight, "saving a prefab rebuilds the map that instances it, got %s" % reply)

		await live("live_begin", { "text": FileAccess.get_file_as_string(MAP_JSON) })
		var session: TrenchDotLiveSession = link._sessions.get(TrenchDotEditorIntegration.path_key(ProjectSettings.globalize_path(MAP_FILE)))
		await until(func() -> bool: return session and not session.pending())
		reply = await link.handle_message(JSON.stringify(save_prefab))
		check(reply.get("ok") == true and reply.get("rebuilt") == 0 and session != null and session.pending(), "and queues a full build of a live map that instances it, got %s" % reply)
		await live("live_end")

	## prop_physics builds its body when it enters the tree. In the editor that body is a preview the scene never saves,
	## the game builds its own.
	func _check_prop_physics_preview() -> void:
		var scene := Node3D.new()
		var prop := TDPropPhysics.new()
		scene.add_child(prop)
		prop.owner = scene
		add_child(scene)
		var preview := prop.get_children(true)
		check(prop.get_child_count() == 0 and preview.any(func(n: Node) -> bool: return n is CollisionShape3D) and preview.any(func(n: Node) -> bool: return n is MeshInstance3D), "prop_physics shows its box and collision in the editor, got %s" % [preview])
		var packed := PackedScene.new()
		packed.pack(scene)
		var copy := packed.instantiate()
		check(copy.get_child(0).get_child_count(true) == 0, "and the saved scene leaves them out")
		copy.free()
		scene.free()

	func _write(path: String, text: String) -> void:
		var file := FileAccess.open(path, FileAccess.WRITE)
		file.store_string(text)
		file.close()

	## A pack FGD with a point class td_tests_late_<name> for each script <name>.gd of [param scripts].
	func _late_fgd(scripts: Array) -> String:
		var text := "[gd_resource type=\"Resource\" script_class=\"FuncGodotFGDFile\" format=3]\n\n"
		text += "[ext_resource type=\"Script\" path=\"res://addons/func_godot/src/fgd/func_godot_fgd_file.gd\" id=\"fgd\"]\n"
		text += "[ext_resource type=\"Script\" path=\"res://addons/func_godot/src/fgd/func_godot_fgd_point_class.gd\" id=\"point\"]\n"
		for name: String in scripts:
			text += "[ext_resource type=\"Script\" path=\"%s\" id=\"%s\"]\n" % [LATE_PACK.path_join(name + ".gd"), name]
		var defs := PackedStringArray()
		for name: String in scripts:
			text += "\n[sub_resource type=\"Resource\" id=\"def_%s\"]\nscript = ExtResource(\"point\")\nclassname = \"td_tests_late_%s\"\nscript_class = ExtResource(\"%s\")\nnode_class = \"Node3D\"\n" % [name, name, name]
			defs.append("SubResource(\"def_%s\")" % name)
		return text + "\n[resource]\nscript = ExtResource(\"fgd\")\nentity_definitions = [%s]\n" % ", ".join(defs)

	## A pack copied in while Godot runs, whose entity script extends a class another new script of it declares, the
	## way the TrenchDot editor installs the Gameplay pack. Reading it before the scan registers that class would
	## break the script for good.
	func _check_late_pack() -> void:
		var fs := EditorInterface.get_resource_filesystem()
		await until(func() -> bool: return not fs.is_scanning(), 1200)
		DirAccess.make_dir_recursive_absolute(LATE_PACK)
		_write(LATE_PACK.path_join("base.gd"), "@tool\nclass_name %s extends Node3D\n" % LATE_CLASS)
		_write(LATE_PACK.path_join("entity.gd"), "@tool\nextends %s\n" % LATE_CLASS)
		_write(LATE_PACK.path_join("broken.gd"), "@tool\nextends %s\n" % LATE_CLASS)
		_write(LATE_PACK.path_join("fgd.tres"), _late_fgd(["entity", "broken"]))
		# Note: the pack's scripts have to be freed before their files go, or Godot now and then crashes on exit.
		await _read_late_pack(fs)
		for file in DirAccess.get_files_at(LATE_PACK):
			DirAccess.remove_absolute(LATE_PACK.path_join(file))
		DirAccess.remove_absolute(LATE_PACK)
		fs.scan()
		await until(func() -> bool: return not fs.is_scanning() and not ProjectSettings.get_global_class_list().any(func(c): return c["class"] == LATE_CLASS), 1200)

	func _read_late_pack(fs: EditorFileSystem) -> void:
		# Something else loads a pack script before the scan, as an older addon did, and Godot keeps it broken.
		var broken: Script = load(LATE_PACK.path_join("broken.gd"))
		check(broken != null and not broken.can_instantiate(), "a script loaded before its base class is registered is broken")
		var fgd := TrenchDotFGDFile.new()
		fgd.pack_fgd_path = LATE_PACK.path_join("fgd.tres")
		check(not fgd.get_entity_definitions().has("td_tests_late_entity"), "a pack Godot does not list yet is not read")
		check(fgd.waiting_for_scan, "and its FGD says so")
		var settings := FuncGodotMapSettings.new()
		settings.entity_fgd = fgd
		var report := TrenchDotBuildReport.new(MAP_FILE)
		report.use_settings(settings)
		report.missing_classes.append("func_door")
		var warned := Array(report.warnings()).filter(func(w: String) -> bool: return w.contains("no definition"))
		check(warned.size() == 1 and warned[0].contains("Godot is still scanning its files") and warned[0].contains("Build the map again"), "a build that left out the pack says Godot is still scanning it, got %s" % [warned])
		check(not ResourceLoader.has_cached(LATE_PACK.path_join("entity.gd")), "so its scripts are not loaded either")
		var listed := func() -> bool: return fs.get_file_type(fgd.pack_fgd_path) != "" and not fs.is_scanning() and ProjectSettings.get_global_class_list().any(func(c): return c["class"] == LATE_CLASS)
		check(await until(listed, 1200), "reading it starts the scan that lists it")
		var defs := fgd.get_entity_definitions()
		var entity: Script = defs["td_tests_late_entity"].script_class if defs.has("td_tests_late_entity") else null
		check(entity != null and entity.can_instantiate() and not fgd.waiting_for_scan, "after the scan the pack is read and its script compiles, got %s" % entity)
		check(defs.has("td_tests_late_broken") and defs["td_tests_late_broken"].script_class == broken and broken.can_instantiate(), "and reading it compiles the broken one again")
		fs.scan()
		TrenchDotFGDFile.files_changed()
		check(fs.is_scanning() and fgd.get_entity_definitions().has("td_tests_late_entity"), "a build while Godot scans keeps the pack it read before")
		await until(func() -> bool: return not fs.is_scanning(), 1200)
