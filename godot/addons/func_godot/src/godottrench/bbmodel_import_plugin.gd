@tool
class_name GodotTrenchBBModelImportPlugin extends EditorImportPlugin
## Imports Blockbench .bbmodel files as scenes, so props placed in GodotTrench load directly in Godot.

func _get_importer_name() -> String:
	return "godottrench.bbmodel"

func _get_visible_name() -> String:
	return "Blockbench Model (GodotTrench)"

func _get_recognized_extensions() -> PackedStringArray:
	return PackedStringArray(["bbmodel"])

func _get_save_extension() -> String:
	return "scn"

func _get_resource_type() -> String:
	return "PackedScene"

func _get_priority() -> float:
	return 1.0

func _get_import_order() -> int:
	return 0

func _get_preset_count() -> int:
	return 1

func _get_preset_name(_preset: int) -> String:
	return "Default"

func _get_import_options(_path: String, _preset: int) -> Array[Dictionary]:
	return [
		{ "name": "units_per_meter", "default_value": 16.0 },
		{ "name": "collision", "default_value": "none", "property_hint": PROPERTY_HINT_ENUM, "hint_string": "none,convex,trimesh" },
		{ "name": "nearest_filter", "default_value": true },
	]

func _get_option_visibility(_path: String, _option: StringName, _options: Dictionary) -> bool:
	return true

func _import(source_file: String, save_path: String, options: Dictionary, _platform_variants: Array[String], _gen_files: Array[String]) -> Error:
	var text := FileAccess.get_file_as_string(source_file)
	var model := GodotTrenchBBModel.parse(text)
	if model.is_empty():
		push_error("[GodotTrench] %s is not a Blockbench model" % source_file)
		return ERR_PARSE_ERROR
	var root := GodotTrenchBBModel.build_scene(model, 1.0 / float(options.get("units_per_meter", 16.0)), str(options.get("collision", "none")), bool(options.get("nearest_filter", true)))
	var packed := PackedScene.new()
	var err := packed.pack(root)
	root.free()
	if err != OK:
		return err
	return ResourceSaver.save(packed, "%s.%s" % [save_path, _get_save_extension()])
