extends RefCounted
## How builds treat entity definitions and what they tell about them, run from run_tests.gd:
## await load("res://tests/definition_tests.gd").new().run(self)

const SETTINGS := "res://demo/demo_map_settings.tres"
const MAP := "res://tests/maps/basic.tdm"
const UPSTREAM_FGD := "res://addons/func_godot/fgd/func_godot_fgd.tres"
const TRENCHDOT_FGD := "res://addons/func_godot/fgd/trenchdot_default_fgd.tres"
## Map settings without the Gameplay entities pack, a path the build report can name.
const NO_PACK_SETTINGS := "res://tests/td_no_pack_settings.tres"

## The run_tests.gd runner, untyped so its check() is reachable.
var t

func check(cond: bool, what: String) -> void:
	t.check(cond, what)

func run(runner: SceneTree) -> void:
	t = runner
	test_missing_class_warning()
	test_new_resources_use_the_gameplay_fgd()

## The demo's map settings with [param fgd] as their Entity Fgd.
func _settings(fgd: FuncGodotFGDFile) -> FuncGodotMapSettings:
	var settings: FuncGodotMapSettings = load(SETTINGS).duplicate()
	settings.save_generated_materials = false
	settings.entity_fgd = fgd
	return settings

## An FGD with the demo's definitions and [param defs] on top.
func _fgd(defs: Array) -> FuncGodotFGDFile:
	var fgd := FuncGodotFGDFile.new()
	fgd.base_fgd_files = [load(SETTINGS).entity_fgd]
	fgd.entity_definitions.assign(defs)
	return fgd

func _entity(id: int, classname: String, children := [], properties := {}) -> Dictionary:
	return { "id": id, "type": "entity", "classname": classname, "origin": [0.0, 0.0, 0.0], "angles": [0.0, 0.0, 0.0], "properties": properties, "children": children }

func _box(id: int, x := 0.0) -> Dictionary:
	return t.box_node(id, Vector3(x, 0, 0), Vector3(x + 64, 64, 64))

## A map node under a scene root, built from [param children] with [param settings].
func _build(settings: FuncGodotMapSettings, children: Array) -> FuncGodotMap:
	var scene := Node3D.new()
	var map := FuncGodotMap.new()
	map.map_settings = settings
	map.local_map_file = MAP
	scene.add_child(map)
	map.owner = scene
	var text := JSON.stringify({ "format": "trenchdot-map", "properties": {}, "layers": [{ "type": "layer", "id": 1, "children": children }] })
	map.build_from_text(text)
	return map

func _free(map: FuncGodotMap) -> void:
	(map.owner if map.owner else map).free()

func _child_named(map: Node, part: String) -> Node:
	for n in t.collect(map, func(n): return String(n.name).contains(part)):
		return n
	return null

func test_missing_class_warning() -> void:
	print("- classes without a definition warn once, naming the map settings and their FGD")
	var settings := _settings(load(UPSTREAM_FGD))
	settings.take_over_path(NO_PACK_SETTINGS)
	var map := _build(settings, [_entity(2, "td_tests_unknown_point"), _entity(3, "td_tests_unknown_brush", [_box(4)]), _entity(5, "func_door", [_box(6, 128)])])
	var report := map.build_report
	_free(map)
	check(report["map_settings"] == NO_PACK_SETTINGS and report["entity_fgd"] == UPSTREAM_FGD, "the report names the map settings and their FGD, got %s %s" % [report["map_settings"], report["entity_fgd"]])
	var lines := Array(report["warnings"]).filter(func(w: String) -> bool: return w.contains("no definition"))
	check(lines.size() == 1, "one warning for all classes without a definition, got %s" % [report["warnings"]])
	var line: String = lines[0] if lines.size() == 1 else ""
	for part in ["3 entity classes", NO_PACK_SETTINGS, "Entity Fgd " + UPSTREAM_FGD, "td_tests_unknown_point", "td_tests_unknown_brush", "func_door", "without script or collision"]:
		check(line.contains(part), "the warning names %s, got %s" % [part, line])
	check(line.contains("%s is an FGD of the addon" % UPSTREAM_FGD) and line.contains("Set the Entity Fgd of %s to the FGD your game config uses" % NO_PACK_SETTINGS), "with the pack installed and an FGD of the addon it points to the game config's FGD, got %s" % line)
	var own := FuncGodotFGDFile.new()
	own.take_over_path("res://tests/td_tests_own_fgd.tres")
	var own_map := _build(_settings(own), [_entity(5, "func_door", [_box(6, 128)])])
	var own_lines := Array(own_map.build_report["warnings"]).filter(func(w: String) -> bool: return w.contains("no definition"))
	_free(own_map)
	check(own_lines.size() == 1 and own_lines[0].contains("does not include res://addons/func_godot/fgd/trenchdot_default_fgd.tres"), "and with an FGD of the project it names the FGD that brings it, got %s" % [own_lines])

	var clean := _build(load(SETTINGS), [_entity(2, "light")])
	var clean_report := clean.build_report
	_free(clean)
	check(not Array(clean_report["warnings"]).any(func(w: String) -> bool: return w.contains("no definition")), "a map whose classes are all defined has no such warning")

	var many := { "missing_classes": ["b_class", "a_class"], "map_settings": "res://b.tres", "warnings": [] }
	for i in 25:
		many["warnings"].append("warning %d" % (i % 23))
	var summary := TrenchDotEditorIntegration.build_summary([report, many, clean_report])
	check(summary.get("missing_classes") == ["a_class", "b_class", "func_door", "td_tests_unknown_brush", "td_tests_unknown_point"], "the reply lists the missing classes of every build once, sorted, got %s" % [summary.get("missing_classes")])
	check(summary.get("settings") == ["res://b.tres", SETTINGS, NO_PACK_SETTINGS], "and the map settings they used, got %s" % [summary.get("settings")])
	var warnings: Array = summary.get("warnings", [])
	check(warnings.size() == TrenchDotEditorIntegration.MAX_REPLY_WARNINGS and warnings[0] == line, "and the first %d warnings, got %d" % [TrenchDotEditorIntegration.MAX_REPLY_WARNINGS, warnings.size()])
	check(TrenchDotEditorIntegration.build_summary([clean_report, {}]).keys() == ["settings"], "a clean build only names its settings")
	var link := TrenchDotEditorIntegration.new()
	var reply := link.built_reply(MAP, [report], false)
	check(reply.get("rebuilt") == 1 and reply.get("missing_classes", []).size() == 3 and reply.get("settings") == [NO_PACK_SETTINGS], "a save reply carries them, got %s" % [reply])
	link.free()

func test_new_resources_use_the_gameplay_fgd() -> void:
	print("- new map settings and game configs start on the TrenchDot FGD")
	var settings := FuncGodotMapSettings.new()
	check(settings.entity_fgd.resource_path == TRENCHDOT_FGD, "new map settings use %s, got %s" % [TRENCHDOT_FGD, settings.entity_fgd.resource_path])
	check(TrenchDotGameConfig.new().fgd_file.resource_path == TRENCHDOT_FGD, "and so does a new game config")
	var map := _build(settings, [_entity(2, "func_door", [_box(3)]), _entity(4, "light")])
	var door := _child_named(map, "func_door")
	var light := _child_named(map, "light")
	check(door is AnimatableBody3D and door.get_script() != null and t.collect(door, func(n): return n is CollisionShape3D).size() == 1, "a door built with them moves and collides, got %s" % door)
	check(light is OmniLight3D, "and a light is a light, got %s" % light)
	check(map.build_report["missing_classes"].is_empty(), "no class is missing, got %s" % [map.build_report["missing_classes"]])
	_free(map)
