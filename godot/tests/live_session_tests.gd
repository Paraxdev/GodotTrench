extends RefCounted
## Live sessions build what a full build of the same map with the same definitions would, run from run_tests.gd:
## await load("res://tests/live_session_tests.gd").new().run(self)

const SETTINGS := "res://demo/demo_map_settings.tres"
const MAP := "res://tests/maps/basic.tdm"
const MAP_JSON := "res://tests/maps/basic_json.tdm"
## The TrenchDotFGDFile the demo's FGD has as a base, which brings the installed Gameplay entities pack.
const CORE_FGD := "res://addons/func_godot/fgd/trenchdot/trenchdot_fgd.tres"
const CSHARP_DIR := "user://td_tests_live_csharp"
const DOOR := 9
const BUTTON := 11

## The run_tests.gd runner, untyped so its check() is reachable.
var t

func check(cond: bool, what: String) -> void:
	t.check(cond, what)

func run(runner: SceneTree) -> void:
	t = runner
	await test_stale_definitions_at_begin()
	test_failed_parse_leaves_no_source()
	test_csharp_definitions_count()
	await test_restart_after_live_edits()
	await test_definitions_change_during_session()

## A map node in the tree, built from the test map's JSON.
func _map() -> FuncGodotMap:
	var map := FuncGodotMap.new()
	map.name = "LiveDefinitionsMap"
	map.map_settings = load(SETTINGS)
	map.local_map_file = MAP
	t.root.add_child(map)
	map.build_from_text(FileAccess.get_file_as_string(MAP_JSON))
	return map

## Runs [param build] while the Gameplay entities pack cannot be found, like before it was installed, and returns what
## it returns.
func _without_pack(build: Callable) -> Variant:
	var core: TrenchDotFGDFile = load(CORE_FGD)
	var installed := core.pack_fgd_path
	core.pack_fgd_path = "res://tests/td_tests_no_such_pack.tres"
	var out: Variant = build.call()
	core.pack_fgd_path = installed
	return out

func _session(map: FuncGodotMap) -> TrenchDotLiveSession:
	var maps: Array[FuncGodotMap] = [map]
	return TrenchDotLiveSession.new(MAP, maps)

## Runs the session's queued rebuilds as if edits had paused, until it is idle.
func _settle(session: TrenchDotLiveSession) -> void:
	for i in 100:
		if not session.pending():
			return
		session._last_change -= TrenchDotLiveSession.IDLE_MSEC + 1
		session.process()

func _is_door(node: Node) -> bool:
	return node is TDDoor and node is AnimatableBody3D and t.collect(node, func(n): return n is CollisionShape3D).size() > 0

func test_stale_definitions_at_begin() -> void:
	print("- a live session rebuilds a scene built with other entity definitions")
	var map: FuncGodotMap = _without_pack(_map)
	var stale: Node = TrenchDotBuild.nodes_by_id(map).get(DOOR)
	check(stale != null and not stale is TDDoor, "without the pack the door builds as a plain node, got %s" % stale)
	check(not TrenchDotBuild.built_with_current_definitions(map), "the scene no longer matches the definitions once the pack is back")
	var session := _session(map)
	check(session.begin(FileAccess.get_file_as_string(MAP_JSON)), "the session takes the map")
	var by_id := TrenchDotBuild.nodes_by_id(map)
	check(_is_door(by_id.get(DOOR)), "begin builds the door with its script and collision, got %s" % by_id.get(DOOR))
	check(by_id.get(BUTTON) is TDButton, "and the button, got %s" % by_id.get(BUTTON))
	var world: Node = by_id.get(0)
	check(TrenchDotBuild.built_with_current_definitions(map), "the build records the definitions it used")
	_session(map).begin(FileAccess.get_file_as_string(MAP_JSON))
	check(TrenchDotBuild.nodes_by_id(map).get(0) == world, "a session over the same map and definitions keeps the scene")
	map.queue_free()
	await t.process_frame

func test_failed_parse_leaves_no_source() -> void:
	print("- a map that fails to build claims no source")
	var map := _map()
	check(map.has_meta(TrenchDotBuild.SOURCE_HASH_META) and map.has_meta(TrenchDotBuild.DEFINITIONS_META), "a build from text records its source and definitions")
	map.build_from_text("not a map")
	check(not map.has_meta(TrenchDotBuild.SOURCE_HASH_META) and not map.has_meta(TrenchDotBuild.DEFINITIONS_META), "a failed build leaves neither, got %s" % [map.get_meta_list()])
	map.build_from_text(FileAccess.get_file_as_string(MAP_JSON))
	map.clear_children()
	check(not map.has_meta(TrenchDotBuild.SOURCE_HASH_META), "and neither does a cleared map")
	map.free()

func test_csharp_definitions_count() -> void:
	print("- C# entity definitions are part of the definitions a scene was built with")
	var setting := TrenchDotCSharp.SETTING
	var had := ProjectSettings.has_setting(setting)
	var dirs: Variant = ProjectSettings.get_setting(setting) if had else null
	DirAccess.make_dir_recursive_absolute(CSHARP_DIR)
	ProjectSettings.set_setting(setting, PackedStringArray([CSHARP_DIR]))
	var settings: FuncGodotMapSettings = load(SETTINGS)
	var before := TrenchDotBuild.definitions_id(settings)
	var file := FileAccess.open(CSHARP_DIR.path_join("LiveGuard.cs"), FileAccess.WRITE)
	file.store_string("[GlobalClass]\n[TrenchDotEntity(\"td_tests_live_guard\")]\npublic partial class LiveGuard : Node3D\n{\n}\n")
	file.close()
	var after := TrenchDotBuild.definitions_id(settings)
	check(TrenchDotBuild.definitions(settings).has("td_tests_live_guard"), "the definitions hold the C# class")
	check(before != after, "a new C# entity class changes the definitions id")
	DirAccess.remove_absolute(CSHARP_DIR.path_join("LiveGuard.cs"))
	check(TrenchDotBuild.definitions_id(settings) == before, "and removing it changes it back")

	# The editor integration keeps the list of source files until Godot sees files change.
	TrenchDotCSharp.keep_file_list = true
	TrenchDotBuild.definitions_id(settings)
	file = FileAccess.open(CSHARP_DIR.path_join("LiveGuard.cs"), FileAccess.WRITE)
	file.store_string("[GlobalClass]
[TrenchDotEntity(\"td_tests_live_guard\")]
public partial class LiveGuard : Node3D
{
}
")
	file.close()
	check(TrenchDotBuild.definitions_id(settings) == before, "a kept file list does not look for new source files")
	var link := TrenchDotEditorIntegration.new()
	link.check_definitions()
	link.free()
	check(TrenchDotBuild.definitions_id(settings) == after, "until the editor integration checks the definitions")
	DirAccess.remove_absolute(CSHARP_DIR.path_join("LiveGuard.cs"))
	check(TrenchDotBuild.definitions_id(settings) == before, "and a kept source file that is gone makes it look again")
	TrenchDotCSharp.keep_file_list = false
	DirAccess.remove_absolute(CSHARP_DIR)
	ProjectSettings.set_setting(setting, dirs if had else null)

func test_restart_after_live_edits() -> void:
	print("- a session starting on a scene changed by live edits builds it again")
	var map := _map()
	var text := FileAccess.get_file_as_string(MAP_JSON)
	var session := _session(map)
	session.begin(text)
	session.apply([{ "op": "remove", "id": DOOR }])
	session.process()
	check(not TrenchDotBuild.nodes_by_id(map).has(DOOR), "the live edit removed the door")
	check(not map.has_meta(TrenchDotBuild.SOURCE_HASH_META), "and the scene no longer names the map it was built from")
	_session(map).begin(text)
	check(_is_door(TrenchDotBuild.nodes_by_id(map).get(DOOR)), "a new session over the original map brings the door back")
	map.queue_free()
	await t.process_frame

func test_definitions_change_during_session() -> void:
	print("- definitions that change while a session runs rebuild the entities no edit touched")
	var text := FileAccess.get_file_as_string(MAP_JSON)
	var session: TrenchDotLiveSession = _without_pack(func() -> TrenchDotLiveSession:
		var s := _session(_map())
		s.begin(text)
		return s)
	var map: FuncGodotMap = session.maps[0]
	var lamp: Dictionary = session._nodes[15].duplicate(true)
	lamp["properties"]["light_energy"] = "4"
	session.apply([{ "op": "set", "id": 15, "parent": 1, "index": 8, "node": lamp }])
	_settle(session)
	check(not TrenchDotBuild.nodes_by_id(map).get(DOOR) is TDDoor, "the session started without the pack builds a plain door")
	session.check_definitions()
	check(session.pending(), "once the pack is back the session queues a build")
	_settle(session)
	var by_id := TrenchDotBuild.nodes_by_id(map)
	check(_is_door(by_id.get(DOOR)) and by_id.get(BUTTON) is TDButton and by_id.get(13) is Area3D, "the door, button and trigger no edit touched are rebuilt, got %s %s %s" % [by_id.get(DOOR), by_id.get(BUTTON), by_id.get(13)])
	check(by_id.get(15) is OmniLight3D and is_equal_approx(by_id[15].light_energy, 4.0), "and the edited lamp keeps its edit")
	var world: Node = by_id.get(0)
	session.check_definitions()
	check(not session.pending(), "unchanged definitions queue nothing")

	var link := TrenchDotEditorIntegration.new()
	link._sessions["map"] = session
	_without_pack(link.check_definitions)
	check(session.pending(), "the editor integration checks every live session")
	link.free()
	session._clear_pending()
	check(TrenchDotBuild.nodes_by_id(map).get(0) == world, "checking alone builds nothing")
	map.queue_free()
	await t.process_frame
