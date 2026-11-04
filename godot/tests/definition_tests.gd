extends RefCounted
## How builds treat entity definitions and what they tell about them, run from run_tests.gd:
## await load("res://tests/definition_tests.gd").new().run(self)

const SETTINGS := "res://demo/demo_map_settings.tres"
const MAP := "res://tests/maps/basic.tdm"
const UPSTREAM_FGD := "res://addons/func_godot/fgd/func_godot_fgd.tres"
const TRENCHDOT_FGD := "res://addons/func_godot/fgd/trenchdot_default_fgd.tres"
## Map settings without the Gameplay entities pack, a path the build report can name.
const NO_PACK_SETTINGS := "res://tests/td_no_pack_settings.tres"
const CHILD_ADDER := "res://tests/helpers/child_adder.gd"
const PINE := "res://demo/models/pine.bbmodel"
const PROJECT_SETTINGS := "res://tests/td_project_map_settings.tres"

## The run_tests.gd runner, untyped so its check() is reachable.
var t

func check(cond: bool, what: String) -> void:
	t.check(cond, what)

func run(runner: SceneTree) -> void:
	t = runner
	test_missing_class_warning()
	test_new_resources_use_the_gameplay_fgd()
	await test_added_nodes_are_saved()
	test_map_node_follows_the_project_setting()
	test_unknown_node_class()

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

## The scene a built map is part of, packed and instanced again without entering the tree, so nothing runs _ready.
func _saved(map: FuncGodotMap) -> Node:
	var packed := PackedScene.new()
	packed.pack(map.owner)
	return packed.instantiate()

func _generated(prop: Node) -> Array:
	return prop.get_children().filter(func(c: Node) -> bool: return c.has_meta(&"td_prop_generated"))

func test_added_nodes_are_saved() -> void:
	print("- nodes an entity adds while its properties are applied are saved with the scene")
	var adder := FuncGodotFGDPointClass.new()
	adder.classname = "td_tests_child_adder"
	adder.node_class = "Node3D"
	adder.script_class = load(CHILD_ADDER)
	var root := Node3D.new()
	root.name = "Root"
	var scene := PackedScene.new()
	scene.pack(root)
	root.free()
	var instanced := FuncGodotFGDPointClass.new()
	instanced.classname = "td_tests_scene_adder"
	instanced.scene_file = scene
	instanced.script_class = load(CHILD_ADDER)
	var settings := _settings(_fgd([adder, instanced]))
	var children := [_entity(2, "prop_model", [], { "model": PINE, "targetname": "pine" }), _entity(3, "td_tests_child_adder", [], { "targetname": "adder" }), _entity(4, "td_tests_scene_adder", [], { "targetname": "scene_adder" })]
	var map := _build(settings, children)
	var unowned: Array = t.collect(map, func(n: Node) -> bool: return n != map and n.owner == null).map(func(n: Node) -> String: return String(n.name))
	check(unowned == [], "every built node has an owner, got %s" % [unowned])
	var copy := _saved(map)
	_free(map)
	var prop: Node = t.find_targetname(copy, "pine")
	var parts := _generated(prop) if prop else []
	check(parts.size() == 2, "the saved prop keeps its model and collision, got %s" % [prop.get_children() if prop else null])
	var body: Node = prop.get_node_or_null("prop_collision") if prop else null
	check(body != null and t.collect(body, func(n): return n is CollisionShape3D).size() > 0, "and its collision shapes")
	for target in ["adder", "scene_adder"]:
		var entity: Node = t.find_targetname(copy, target)
		check(entity != null and entity.get_node_or_null("added/inner") != null, "%s keeps the child it added, got %s" % [target, entity.get_children() if entity else null])
		check(entity != null and entity.get_node_or_null("from_ready") == null, "%s does not save what _ready adds" % target)
	copy.free()

	print("- a prop saved without its model gets it back when it enters the tree")
	map = _build(settings, [_entity(2, "prop_model", [], { "model": PINE, "targetname": "pine" })])
	for part in _generated(t.find_targetname(map, "pine")):
		t.collect(part, func(n: Node) -> bool: return true).map(func(n: Node) -> Node: n.owner = null; return n)
	copy = _saved(map)
	_free(map)
	prop = t.find_targetname(copy, "pine")
	check(prop != null and _generated(prop).is_empty(), "the old scene lost the model")
	t.root.add_child(copy)
	check(prop != null and _generated(prop).size() == 2, "_ready rebuilt the model and its collision, got %s" % [prop.get_children() if prop else null])
	copy.queue_free()
	await t.process_frame

func test_map_node_follows_the_project_setting() -> void:
	print("- a map node without map settings follows the project setting")
	var holder := Node3D.new()
	var fresh := FuncGodotMap.new()
	holder.add_child(fresh)
	fresh.owner = holder
	var packed := PackedScene.new()
	packed.pack(holder)
	holder.free()
	var state := packed.get_state()
	var stored := []
	for i in state.get_node_property_count(1):
		stored.append(state.get_node_property_name(1, i))
	check(not &"map_settings" in stored, "a new map node saves no map settings, got %s" % [stored])

	var only := FuncGodotFGDPointClass.new()
	only.classname = "td_tests_project_only"
	only.node_class = "Path3D"
	var settings := _settings(_fgd([only]))
	settings.inverse_scale_factor = 64.0
	settings.take_over_path(PROJECT_SETTINGS)
	var before: Variant = ProjectSettings.get_setting(FuncGodotMap.SETTING_DEFAULT_MAP_SETTINGS)
	ProjectSettings.set_setting(FuncGodotMap.SETTING_DEFAULT_MAP_SETTINGS, PROJECT_SETTINGS)
	var map := _build(null, [_entity(2, "td_tests_project_only")])
	check(_child_named(map, "td_tests_project_only") is Path3D and map.build_report["map_settings"] == PROJECT_SETTINGS, "it builds with the settings the project names, got %s" % _child_named(map, "td_tests_project_only"))
	check(not Array(map.build_report["warnings"]).any(func(w: String) -> bool: return w.contains("default map settings")), "and does not warn about them")
	var built := _child_named(map, "td_tests_project_only")
	ProjectSettings.set_setting(FuncGodotMap.SETTING_DEFAULT_MAP_SETTINGS, "res://tests/td_tests_no_such_settings.tres")
	check(TrenchDotIO.units_per_meter(built) == 64.0, "and keeps their scale for a game exported without them, got %s" % TrenchDotIO.units_per_meter(built))
	ProjectSettings.set_setting(FuncGodotMap.SETTING_DEFAULT_MAP_SETTINGS, PROJECT_SETTINGS)
	_free(map)

	map = _build(load(FuncGodotMap.ADDON_MAP_SETTINGS), [_entity(2, "light")])
	var lines := Array(map.build_report["warnings"]).filter(func(w: String) -> bool: return w.contains("uses the addon's default map settings, not %s from Project Settings" % PROJECT_SETTINGS))
	check(lines.size() == 1, "a map node that stored the addon's settings is told to clear them, got %s" % [map.build_report["warnings"]])
	check(not map.has_meta(FuncGodotMap.UNITS_META), "a map node with its own settings keeps no scale of its own")
	_free(map)
	ProjectSettings.set_setting(FuncGodotMap.SETTING_DEFAULT_MAP_SETTINGS, FuncGodotMap.ADDON_MAP_SETTINGS)
	map = _build(load(FuncGodotMap.ADDON_MAP_SETTINGS), [_entity(2, "light")])
	check(not Array(map.build_report["warnings"]).any(func(w: String) -> bool: return w.contains("default map settings")), "not when the project uses them too")
	_free(map)
	ProjectSettings.set_setting(FuncGodotMap.SETTING_DEFAULT_MAP_SETTINGS, before)

func _solid(classname: String, node_class := "", script: Script = null) -> FuncGodotFGDSolidClass:
	var def := FuncGodotFGDSolidClass.new()
	def.classname = classname
	def.node_class = node_class
	def.script_class = script
	return def

func _point(classname: String, node_class := "", script: Script = null) -> FuncGodotFGDPointClass:
	var def := FuncGodotFGDPointClass.new()
	def.classname = classname
	def.node_class = node_class
	def.script_class = script
	return def

## The definition problems of [param map]'s last build for [param classname].
func _problems(map: FuncGodotMap, classname: String) -> Array:
	return map.build_report["definition_problems"].filter(func(p: Dictionary) -> bool: return p["classname"] == classname)

func test_unknown_node_class() -> void:
	print("- a Node Class that does not resolve builds a Node3D and says so")
	var csharp := _point("td_tests_cs_point", "GtTestsNoSuchCSharpClass")
	csharp.meta_properties = { "csharp": true }
	var settings := _settings(_fgd([_solid("td_tests_bad_solid", "NoSuchBody"), _point("td_tests_bad_point", "NoSuchNode"), csharp]))
	var map := _build(settings, [_entity(2, "td_tests_bad_solid", [_box(3)]), _entity(4, "td_tests_bad_point"), _entity(5, "td_tests_cs_point")])
	var solid := _child_named(map, "td_tests_bad_solid")
	check(solid != null and solid.get_class() == "Node3D" and t.collect(solid, func(n): return n is MeshInstance3D).size() == 1, "the brush entity keeps its mesh, got %s" % solid)
	var problems := _problems(map, "td_tests_bad_solid")
	check(problems.size() == 1 and problems[0]["error"] and problems[0]["problem"] == "td_tests_bad_solid: Node Class NoSuchBody not found, building it as a Node3D", "the report names the class it could not find, got %s" % [problems])
	var point := _child_named(map, "td_tests_bad_point")
	check(point != null and point.get_class() == "Node3D", "the point entity is a Node3D, got %s" % point)
	problems = _problems(map, "td_tests_bad_point")
	check(problems.size() == 1 and not problems[0]["error"] and problems[0]["problem"].contains("NoSuchNode not found"), "and is reported as a warning, got %s" % [problems])
	problems = _problems(map, "td_tests_cs_point")
	check(problems.size() == 1 and problems[0]["problem"].contains("C# entities need the .NET build of Godot, a built C# project and [GlobalClass]"), "a C# class names what it needs, got %s" % [problems])
	check(Array(map.build_report["warnings"]).any(func(w: String) -> bool: return w.contains("NoSuchBody not found")), "the warnings the editor gets list them")
	_free(map)
