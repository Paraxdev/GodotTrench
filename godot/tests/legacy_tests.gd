extends RefCounted
## Projects set up under the addon's old name, GodotTrench, keep working, run from run_tests.gd:
## await load("res://tests/legacy_tests.gd").new().run(self)

const SETTINGS := "res://demo/demo_map_settings.tres"
const MAP := "res://tests/maps/basic.tdm"
const MAP_JSON := "res://tests/maps/basic_json.tdm"
const OLD_MAP := "res://tests/td_tests_legacy_map.gtm"
const OLD_PREFAB := "res://tests/td_tests_legacy_prefab.gtm"
const OLD_BINARY := "res://tests/td_tests_legacy_binary.gtm"
const OLD_PACK := "res://tests/td_tests_old_pack"
const OLD_GAME_FILE := "res://godottrench_game.json"
const GAME_FILE := "res://trenchdot_game.json"

## The run_tests.gd runner, untyped so its check() is reachable.
var t

func check(cond: bool, what: String) -> void:
	t.check(cond, what)

func run(runner: SceneTree) -> void:
	t = runner
	print("- projects from before the rename")
	test_classname_alias()
	test_old_map_format()
	await test_old_maps_build()
	await test_live_session()
	test_project_settings()
	test_game_config_file()
	test_csharp_attributes()
	await test_saved_scene_metadata()
	await test_old_pack()
	test_old_names_stay_in_one_place()

func _entity(id: int, classname: String, properties := {}) -> Dictionary:
	return { "id": id, "type": "entity", "classname": classname, "origin": [0.0, 0.0, 0.0], "angles": [0.0, 0.0, 0.0], "properties": properties, "children": [] }

func _old_text(children: Array, format := TrenchDotLegacy.MAP_FORMAT) -> String:
	return JSON.stringify({ "format": format, "properties": {}, "layers": [{ "type": "layer", "id": 1, "children": children }] })

func _write(path: String, text: String) -> void:
	var file := FileAccess.open(path, FileAccess.WRITE)
	file.store_string(text)
	file.close()

func _definition(classname: String, node_class: String) -> FuncGodotFGDPointClass:
	var def := FuncGodotFGDPointClass.new()
	def.classname = classname
	def.node_class = node_class
	return def

func _settings(fgd: FuncGodotFGDFile) -> FuncGodotMapSettings:
	var settings: FuncGodotMapSettings = load(SETTINGS).duplicate()
	settings.save_generated_materials = false
	settings.entity_fgd = fgd
	return settings

func _fgd(defs: Array) -> FuncGodotFGDFile:
	var fgd := FuncGodotFGDFile.new()
	fgd.base_fgd_files = [load(SETTINGS).entity_fgd]
	fgd.entity_definitions.assign(defs)
	return fgd

func _map(settings: FuncGodotMapSettings, file: String) -> FuncGodotMap:
	var map := FuncGodotMap.new()
	map.map_settings = settings
	map.local_map_file = file
	t.root.add_child(map)
	return map

func _entity_nodes(map: FuncGodotMap, part: String) -> Array[Node]:
	return t.collect(map, func(n: Node) -> bool: return n != map and String(n.name).contains(part))

func test_classname_alias() -> void:
	print("- gt_ classnames build as their td_ definitions")
	var defs := { "td_door": 1, "td_lamp": 2, "gt_lamp": 3, "gt_plain": 4 }
	check(TrenchDotLegacy.classname_alias("gt_door", defs) == "td_door", "a gt_ name without a definition of its own takes the td_ one")
	check(TrenchDotLegacy.classname_alias("gt_lamp", defs) == "gt_lamp", "one that has its own definition keeps it")
	check(TrenchDotLegacy.classname_alias("gt_missing", defs) == "gt_missing", "and one with neither stays as it is")
	check(TrenchDotLegacy.classname_alias("td_door", defs) == "td_door" and TrenchDotLegacy.classname_alias("func_door", defs) == "func_door", "other names are left alone")
	check(TrenchDotLegacy.classname_alias("func_gt_door", defs) == "func_gt_door", "only a prefix counts")

func test_old_map_format() -> void:
	print("- maps saved as godottrench-map parse")
	var parsed := FuncGodotParser.new().parse_tdm(_old_text([_entity(2, "light")]), load(SETTINGS), MAP)
	check(parsed.entities.size() == 2, "a map with the old format name has its entities, got %d" % parsed.entities.size())
	var other := FuncGodotParser.new().parse_tdm(_old_text([_entity(2, "light")], "something-else"), load(SETTINGS), MAP)
	check(other.entities.is_empty(), "and another format name is still refused")
	check(TrenchDotParser.is_map_format("trenchdot-map") and TrenchDotParser.is_map_format(TrenchDotLegacy.MAP_FORMAT) and not TrenchDotParser.is_map_format(null), "the check takes both names")
	check(TrenchDotBuild.MAP_FILES == PackedStringArray(["tdm", TrenchDotLegacy.MAP_EXTENSION]), "prefab paths take both extensions")

func test_old_maps_build() -> void:
	print("- .gtm maps build, with gt_ entities as td_ ones and prefabs in .gtm files")
	var marker := _definition("td_tests_legacy_marker", "Marker3D")
	var both_old := _definition("gt_tests_legacy_both", "Node3D")
	var both_new := _definition("td_tests_legacy_both", "Marker3D")
	var settings := _settings(_fgd([marker, both_old, both_new]))
	_write(OLD_PREFAB, _old_text([_entity(2, "gt_tests_legacy_marker", { "targetname": "inner" })]))
	var instance := { "id": 5, "type": "instance", "path": OLD_PREFAB.get_file(), "origin": [0.0, 0.0, 0.0], "angles": [0.0, 0.0, 0.0] }
	_write(OLD_MAP, _old_text([_entity(2, "gt_tests_legacy_marker", { "targetname": "outer" }), _entity(3, "gt_tests_legacy_both"), _entity(4, "gt_tests_legacy_unknown"), instance]))
	var map := _map(settings, OLD_MAP)
	var report := map.build()
	check(Array(report.get("missing_classes", [])) == ["gt_tests_legacy_unknown"], "only the class with no definition is missing, got %s" % [report.get("missing_classes")])
	var outer: Node = t.find_targetname(map, "outer")
	check(outer is Marker3D, "a gt_ entity builds with the td_ definition, got %s" % outer)
	var inner: Node = t.find_targetname(map, "inner")
	check(inner is Marker3D, "so does one in a prefab saved as .gtm, got %s" % inner)
	var both := _entity_nodes(map, "gt_tests_legacy_both")
	check(both.size() == 1 and not both[0] is Marker3D, "a gt_ name that has a definition of its own keeps it, got %s" % [both])
	map.free()

	var parsed := FuncGodotParser.new().parse_map_data(OLD_MAP, settings)
	check(parsed.entities.size() > 1, "parse_map_data reads the .gtm file")

	DirAccess.copy_absolute(ProjectSettings.globalize_path(MAP), ProjectSettings.globalize_path(OLD_BINARY))
	check(TrenchDotTdmFile.is_binary(FileAccess.get_file_as_bytes(OLD_BINARY)), "a binary map keeps its container magic")
	var expected := _map(load(SETTINGS), MAP)
	expected.build()
	var old := _map(load(SETTINGS), OLD_BINARY)
	old.build()
	check(old.get_child_count() > 0 and old.get_child_count() == expected.get_child_count(), "and a binary .gtm builds like the .tdm, got %d and %d children" % [old.get_child_count(), expected.get_child_count()])
	expected.free()
	old.free()
	for file in [OLD_MAP, OLD_PREFAB, OLD_BINARY]:
		DirAccess.remove_absolute(ProjectSettings.globalize_path(file))
	var wrong := _map(load(SETTINGS), "res://tests/td_tests_legacy_map.gtx")
	check(wrong.verify() == ERR_FILE_UNRECOGNIZED, "other extensions are still refused")
	wrong.free()
	await t.process_frame

func test_live_session() -> void:
	print("- live sessions take the old format name")
	var map := _map(load(SETTINGS), MAP)
	map.build_from_text(FileAccess.get_file_as_string(MAP_JSON))
	var maps: Array[FuncGodotMap] = [map]
	var session := TrenchDotLiveSession.new(MAP, maps)
	check(session.begin(FileAccess.get_file_as_string(MAP_JSON).replace("trenchdot-map", TrenchDotLegacy.MAP_FORMAT)), "a session begins with a map of the old format name")
	check(not session.begin(FileAccess.get_file_as_string(MAP_JSON).replace("trenchdot-map", "other")), "and refuses any other")
	map.free()
	await t.process_frame

func test_project_settings() -> void:
	print("- godottrench/ project settings move to trenchdot/")
	var old_keys := ["godottrench/live_link_port", "godottrench/collision/model_shape", "godottrench/threaded_build", "godottrench/game_config"]
	var new_keys := ["trenchdot/live_link_port", "trenchdot/collision/model_shape", "trenchdot/threaded_build", "trenchdot/game_config"]
	var kept := {}
	for key in new_keys:
		kept[key] = ProjectSettings.get_setting(key) if ProjectSettings.has_setting(key) else null
		ProjectSettings.set_setting(key, null)
	check(not TrenchDotLegacy.migrate_settings(), "a project without old settings changes nothing")
	ProjectSettings.set_setting("godottrench/live_link_port", 7911)
	ProjectSettings.set_setting("godottrench/collision/model_shape", "box")
	ProjectSettings.set_setting("godottrench/threaded_build", false)
	ProjectSettings.set_setting("godottrench/game_config", "res://addons/func_godot/game_config/godottrench/godottrench_game_config.tres")
	ProjectSettings.set_setting("trenchdot/threaded_build", true)
	check(TrenchDotLegacy.migrate_settings(), "old settings are moved")
	check(ProjectSettings.get_setting("trenchdot/live_link_port") == 7911 and ProjectSettings.get_setting("trenchdot/collision/model_shape") == "box", "their values arrive under the new names, nested keys too")
	check(ProjectSettings.get_setting("trenchdot/threaded_build") == true, "a new setting that is set already keeps its value")
	check(ProjectSettings.get_setting("trenchdot/game_config") == "res://addons/func_godot/game_config/trenchdot/trenchdot_game_config.tres", "the old default game config becomes the new one")
	check(not old_keys.any(func(k: String) -> bool: return ProjectSettings.has_setting(k)), "and the old ones are gone")
	var saved := "user://td_tests_legacy_project.godot"
	ProjectSettings.save_custom(saved)
	var text := FileAccess.get_file_as_string(saved)
	check(not text.contains("[godottrench]") and text.contains("live_link_port=7911"), "so the saved project file no longer has the section")
	DirAccess.remove_absolute(ProjectSettings.globalize_path(saved))
	check(not TrenchDotLegacy.migrate_settings(), "migrating again changes nothing, so the project is not saved again")
	for key in new_keys:
		ProjectSettings.set_setting(key, kept[key])

func test_game_config_file() -> void:
	print("- the game config goes to trenchdot_game.json and the old file is removed")
	var kept := FileAccess.get_file_as_bytes(GAME_FILE) if FileAccess.file_exists(GAME_FILE) else PackedByteArray()
	var config: TrenchDotGameConfig = (load("res://demo/demo_game_config.tres") as TrenchDotGameConfig).duplicate()
	config.csharp_source_dirs = []
	_write(OLD_GAME_FILE, "{}")
	var elsewhere := config.duplicate() as TrenchDotGameConfig
	elsewhere.output_path = "user://td_tests_legacy_game.json"
	elsewhere.export_file()
	check(FileAccess.file_exists(OLD_GAME_FILE), "a config written somewhere else leaves the old file")
	DirAccess.remove_absolute(ProjectSettings.globalize_path(elsewhere.output_path))
	config.output_path = OLD_GAME_FILE
	check(config.export_file() == OK, "a config that still names the old file exports")
	var data: Variant = JSON.parse_string(FileAccess.get_file_as_string(GAME_FILE))
	check(data is Dictionary and data.get("format") == "trenchdot-game", "to the new file")
	check(not FileAccess.file_exists(OLD_GAME_FILE), "and the old file is deleted")
	if kept.is_empty():
		DirAccess.remove_absolute(ProjectSettings.globalize_path(GAME_FILE))
	else:
		var file := FileAccess.open(GAME_FILE, FileAccess.WRITE)
		file.store_buffer(kept)
		file.close()

func test_csharp_attributes() -> void:
	print("- C# entities marked with the old attribute names are read")
	var source := "[%sEntity(\"npc_old\", Color = \"#ff4040\")]\npublic partial class NpcOld : Node3D\n{\n\t[%sInput] public void Wake(Node activator) { }\n\tpublic void Helper() { }\n}\n" % [TrenchDotLegacy.CSHARP_PREFIX, TrenchDotLegacy.CSHARP_PREFIX]
	var entries := TrenchDotCSharp.parse_source(source, "res://NpcOld.cs")
	check(entries.size() == 1 and entries[0]["classname"] == "npc_old", "the class is an entity, got %s" % [entries.map(func(e): return e["classname"])])
	check(entries.size() == 1 and entries[0]["inputs"].map(func(i): return i["name"]) == ["wake"], "and its marked method is its only input")

func test_saved_scene_metadata() -> void:
	print("- scenes saved under the old names get their metadata back")
	for old in TrenchDotLegacy.METAS:
		var key: StringName = TrenchDotLegacy.METAS[old]
		check(str(old) != str(key) and (str(old).begins_with("_gt_") or str(old).begins_with("gt_") or str(old).begins_with("godottrench_")), "%s is an old name" % old)
	var constants := {
		&"_gt_id": TrenchDotBuild.ID_META, &"_gt_group": TrenchDotBuild.GROUP_META, &"_gt_source_hash": TrenchDotBuild.SOURCE_HASH_META,
		&"_gt_definitions": TrenchDotBuild.DEFINITIONS_META, &"_gt_units_per_meter": FuncGodotMap.UNITS_META,
		&"gt_targetname": TrenchDotIO.TARGETNAME_META, &"gt_transforms": TrenchDotScatter.BUFFER_META, &"godottrench_baked": TrenchDotLightmap.BAKED_META,
	}
	for old in constants:
		check(TrenchDotLegacy.METAS.get(old) == constants[old], "%s becomes %s" % [old, constants[old]])
	check(TrenchDotLegacy.METAS.get(&"gt_prop_generated") == &"td_prop_generated", "gt_prop_generated becomes the key a prop's generated nodes carry")

	var map := FuncGodotMap.new()
	map.set_meta(&"_gt_units_per_meter", 16.0)
	var door := Node3D.new()
	door.name = "door"
	door.set_meta(&"gt_targetname", "front")
	door.set_meta(&"_gt_id", 7)
	map.add_child(door)
	var kept_node := Node3D.new()
	kept_node.name = "mine"
	kept_node.add_to_group(TrenchDotLegacy.KEEP_GROUP, true)
	map.add_child(kept_node)
	var generated := Node3D.new()
	generated.name = "generated"
	map.add_child(generated)
	t.root.add_child(map)
	check(map.get_meta(FuncGodotMap.UNITS_META, 0.0) == 16.0 and not map.has_meta(&"_gt_units_per_meter"), "a map node that enters the tree renames its own metadata")
	check(door.get_meta(TrenchDotIO.TARGETNAME_META, "") == "front" and door.get_meta(TrenchDotBuild.ID_META, -1) == 7 and door.get_meta_list().size() == 2, "and that of the nodes below it")
	check(t.find_targetname(map, "front") == door, "so entities are found by their saved targetnames")
	check(TrenchDotIO.units_per_meter(door) == 16.0, "and the units per meter stay")
	map.clear_children()
	check(map.get_children() == [kept_node], "nodes in the old keep group survive a rebuild")
	map.free()

	var script_made := Node3D.new()
	script_made.name = "chair"
	script_made.set_meta(&"gt_targetname", "meddle_chair")
	t.root.add_child(script_made)
	check(TrenchDotIO.find_targets(script_made, "meddle_chair", null) == [script_made], "a script that still sets gt_targetname at runtime is found by its target name")
	script_made.free()
	await t.process_frame

## Writes a pack the way an install before the rename left it, with a script that calls the old class names.
func _write_old_pack() -> void:
	DirAccess.make_dir_recursive_absolute(OLD_PACK)
	_write(OLD_PACK.path_join("gt_thing.gd"), "@tool\nclass_name GTTestsThing extends Node3D\n\nvar on := false\n\nfunc _func_godot_apply_properties(properties: Dictionary) -> void:\n\ton = GodotTrenchIO.to_bool(properties.get(\"on\", false)) and GodotTrenchBuild.MAP_FILES.size() > 0\n")
	var thing := _definition("legacy_thing", "Node3D")
	thing.script_class = load(OLD_PACK.path_join("gt_thing.gd"))
	thing.class_properties = { "on": false }
	var door := FuncGodotFGDSolidClass.new()
	door.classname = "func_door"
	door.description = "old door"
	var fgd := FuncGodotFGDFile.new()
	fgd.entity_definitions.assign([thing, door])
	ResourceSaver.save(fgd, OLD_PACK.path_join("gameplay_fgd.tres"))

func _remove_old_pack() -> void:
	for file in DirAccess.get_files_at(OLD_PACK):
		DirAccess.remove_absolute(OLD_PACK.path_join(file))
	DirAccess.remove_absolute(OLD_PACK)

func test_old_pack() -> void:
	print("- an entity pack installed under the old name keeps working beside the new one")
	_write_old_pack()
	var core := load("res://addons/func_godot/fgd/trenchdot/trenchdot_fgd.tres") as TrenchDotFGDFile
	var fgd := core.duplicate() as TrenchDotFGDFile
	check(fgd.legacy_pack_path == TrenchDotLegacy.PACK_FGD and TrenchDotLegacy.PACK_FGD == "res://godottrench/entities/gameplay_fgd.tres", "the old pack is read from where it was installed")
	check(fgd.legacy_pack_fgd() == null, "a project without it gets nothing from there")
	fgd.legacy_pack_path = OLD_PACK.path_join("gameplay_fgd.tres")
	var defs := fgd.get_entity_definitions()
	check(defs.has("legacy_thing"), "definitions of the old pack are found")
	check(defs.has("func_door") and defs["func_door"].description != "old door", "the new pack's definition wins where both define a classname")
	check(TrenchDotGameConfig.definition_sources(fgd).get("func_door", []).size() == 2, "and the game config lists both as a clash")
	var thing: FuncGodotFGDEntityClass = defs.get("legacy_thing")
	check(thing != null and thing.script_class != null and thing.script_class.can_instantiate(), "an old pack script compiles, the old class names it calls still exist")
	var settings := _settings(fgd)
	var map := _map(settings, MAP)
	map.build_from_text(_old_text([_entity(2, "legacy_thing", { "on": "1" })]))
	var node: Node = _entity_nodes(map, "legacy_thing").front() if not _entity_nodes(map, "legacy_thing").is_empty() else null
	check(node != null and node.get_script() == thing.script_class and node.get("on") == true, "and an entity of the old pack builds with it, got %s" % node)
	check(not map.build_report.get("missing_classes", []).has("legacy_thing"), "so its class is not reported missing")
	map.free()
	await t.process_frame
	_remove_old_pack()

func test_old_names_stay_in_one_place() -> void:
	print("- the addon spells the old names in its legacy files only")
	var stand_ins := ["GodotTrenchBuild", "GodotTrenchCollision", "GodotTrenchIO", "GodotTrenchProp"]
	var found: Array[String] = []
	for c in ProjectSettings.get_global_class_list():
		var class_name_text := String(c["class"])
		if String(c["path"]).begins_with("res://addons/func_godot/") and (class_name_text.begins_with("GodotTrench") or class_name_text.begins_with("GT")):
			found.append(class_name_text)
			check(c["base"] == class_name_text.replace("GodotTrench", "TrenchDot"), "%s extends the class that replaced it" % class_name_text)
	found.sort()
	check(found == stand_ins, "the only old class names are the stand ins for entity scripts of an old pack, got %s" % [found])
	var old := RegEx.create_from_string("(?i:godottrench)|\\bgt_|\\bGT[A-Z][a-z]|\\bgtm\\b")
	var offenders: Array[String] = []
	var dirs: Array[String] = ["res://addons/func_godot"]
	while not dirs.is_empty():
		var dir: String = dirs.pop_back()
		for sub in DirAccess.get_directories_at(dir):
			dirs.append(dir.path_join(sub))
		for file in DirAccess.get_files_at(dir):
			var path := dir.path_join(file)
			if file.get_extension() == "gd" and not path.contains("/legacy/") and not file.begins_with("trenchdot_legacy") and old.search(FileAccess.get_file_as_string(path)):
				offenders.append(path.trim_prefix("res://addons/func_godot/"))
	check(offenders.is_empty(), "no other script spells an old name, got %s" % [offenders])
