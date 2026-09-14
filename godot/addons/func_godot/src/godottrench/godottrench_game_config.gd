@tool
@icon("res://addons/func_godot/icons/icon_godot_ranger.svg")
class_name GodotTrenchGameConfig extends Resource
## Game configuration for the GodotTrench level editor.
##
## Exports [code]godottrench_game.json[/code] with entity definitions from a [FuncGodotFGDFile], texture settings
## from [FuncGodotMapSettings], and I/O outputs and inputs derived from entity scripts (signals and methods).

const FORMAT_VERSION := 1

@export_tool_button("Export GodotTrench Game Config") var _export_button: Callable = export_file

@export var game_name: String = "FuncGodot"
@export var fgd_file: FuncGodotFGDFile = preload("res://addons/func_godot/fgd/func_godot_fgd.tres")
@export var map_settings: FuncGodotMapSettings = preload("res://addons/func_godot/func_godot_default_map_settings.tres")
## Written relative to the project, the GodotTrench editor looks for it in the project root.
@export_file("*.json") var output_path: String = "res://godottrench_game.json"
## Texture size assumed by the editor when an image cannot be read.
@export var fallback_texture_size: int = 64
## Folders searched for C# entity classes ([GodotTrenchEntity]) and C# node classes named by FGD entries.
@export var csharp_source_dirs: PackedStringArray = ["res://"]

static func _id_to_godot(v: Vector3) -> Array:
	return [v.y, v.z, v.x]

static func _color_hex(c: Color) -> String:
	return "#" + c.to_html(true)

static func _property_def(name: String, value: Variant, description: Variant) -> Dictionary:
	var desc := ""
	var default_override: Variant = null
	if description is Array and description.size() > 0:
		desc = str(description[0])
		if description.size() > 1:
			default_override = description[1]
	elif description != null:
		desc = str(description)
	var def := { "name": name, "type": "string", "default": "", "description": desc }
	match typeof(value):
		TYPE_INT:
			def["type"] = "int"
			def["default"] = str(value)
		TYPE_FLOAT:
			def["type"] = "float"
			def["default"] = str(value)
		TYPE_BOOL:
			def["type"] = "bool"
			def["default"] = "1" if value else "0"
		TYPE_STRING, TYPE_STRING_NAME:
			def["default"] = str(value)
		TYPE_VECTOR2, TYPE_VECTOR2I:
			def["type"] = "vector2"
			def["default"] = "%s %s" % [value.x, value.y]
		TYPE_VECTOR3, TYPE_VECTOR3I:
			def["type"] = "vector3"
			def["default"] = "%s %s %s" % [value.x, value.y, value.z]
		TYPE_VECTOR4, TYPE_VECTOR4I:
			def["default"] = "%s %s %s %s" % [value[0], value[1], value[2], value[3]]
		TYPE_COLOR:
			def["type"] = "color"
			def["default"] = "%s %s %s" % [value.r8, value.g8, value.b8]
		TYPE_DICTIONARY:
			def["type"] = "choices"
			var options: Array = []
			for label in value:
				options.append([str(label), str(value[label])])
			def["options"] = options
			if default_override != null:
				def["default"] = str(default_override)
			elif value.size() > 0:
				def["default"] = str(value[value.keys()[0]])
		TYPE_ARRAY:
			def["type"] = "flags"
			var options: Array = []
			var defaults: Array = []
			var sum := 0
			for flag in value:
				if flag is Array and flag.size() >= 2:
					options.append([str(flag[0]), str(flag[1])])
					if flag.size() > 2 and flag[2]:
						defaults.append(int(flag[1]))
						sum += int(flag[1])
			def["options"] = options
			def["default_flags"] = defaults
			def["default"] = str(sum)
		TYPE_NODE_PATH:
			def["type"] = "target_destination"
		TYPE_OBJECT:
			if value is Resource:
				def["type"] = "resource"
				def["default"] = value.resource_path
			else:
				def["type"] = "target_source"
		TYPE_NIL:
			def["type"] = "target_source"
	# FuncGodot stores names as plain strings, the editor offers pickers for them.
	if name == "targetname":
		def["type"] = "target_source"
	elif name in ["target", "killtarget", "parent"]:
		def["type"] = "target_destination"
	return def

static func _script_io(script: Script, outputs: Array, inputs: Array) -> void:
	if not script:
		return
	for s in script.get_script_signal_list():
		var params: Array = s.get("args", []).map(func(a): return a["name"])
		outputs.append({ "name": s["name"], "parameter": ", ".join(params) })
	var seen := {}
	for m in script.get_script_method_list():
		var method_name: String = m["name"]
		if method_name.begins_with("_") or seen.has(method_name):
			continue
		seen[method_name] = true
		var params: Array = m.get("args", []).map(func(a): return a["name"])
		inputs.append({ "name": method_name, "parameter": ", ".join(params) })

static func _class_io(node_class: String, outputs: Array) -> void:
	if node_class == "" or not ClassDB.class_exists(node_class):
		return
	var cls := node_class
	# Walk up to (but not including) Node3D so e.g. Area3D reports body_entered but not tree_entered.
	while cls != "" and cls != "Node3D" and cls != "Node" and cls != "Object":
		for s in ClassDB.class_get_signal_list(cls, true):
			var params: Array = s.get("args", []).map(func(a): return a["name"])
			outputs.append({ "name": s["name"], "parameter": ", ".join(params) })
		cls = ClassDB.get_parent_class(cls)

func build_config() -> Dictionary:
	var entities: Array = []
	var defs: Dictionary = fgd_file.get_entity_definitions() if fgd_file else {}
	for classname in defs:
		var def: FuncGodotFGDEntityClass = defs[classname]
		var is_solid := def is FuncGodotFGDSolidClass
		var meta: Dictionary = def.meta_properties
		var entry := {
			"classname": classname,
			"type": "solid" if is_solid else "point",
			"description": def.description,
			"color": _color_hex(meta.get("color", Color(0.8, 0.8, 0.8))),
			"node_class": def.node_class,
			"group": classname.get_slice("_", 0),
		}
		var size: Variant = meta.get("size", null)
		if size is AABB:
			# FuncGodot stores FGD sizes as AABB(min, max) in id axes.
			entry["size"] = [_id_to_godot(size.position), _id_to_godot(size.size)]

		var props: Array = []
		var descriptions: Dictionary = def.class_property_descriptions
		var declared_types: Dictionary = meta.get("property_types", {})
		for key in def.class_properties:
			var prop := _property_def(key, def.class_properties[key], descriptions.get(key, null))
			if declared_types.has(key):
				prop["type"] = str(declared_types[key])
			props.append(prop)
		entry["properties"] = props
		if meta.get("gizmos", []) is Array and not meta.get("gizmos", []).is_empty():
			entry["gizmos"] = meta["gizmos"]

		var outputs: Array = []
		var inputs: Array = []
		var script: Script = def.get("script_class")
		# C# global classes cannot load in non .NET builds, their I/O is read from the source instead.
		if not script and def.node_class != "" and not ClassDB.class_exists(def.node_class) and not FuncGodotEntityAssembler.get_script_by_class_name(def.node_class):
			var cs := GodotTrenchCSharp.class_io(def.node_class, csharp_source_dirs)
			if not cs.is_empty():
				outputs.append_array(cs["outputs"])
				inputs.append_array(cs["inputs"])
				entry["script"] = cs["script"]
		if def is FuncGodotFGDPointClass and def.scene_file:
			entry["scene"] = def.scene_file.resource_path
			var scene: PackedScene = def.scene_file
			var ext: String = scene.resource_path.get_extension().to_lower()
			if ext == "glb" or ext == "gltf":
				entry["model"] = scene.resource_path
			var state: SceneState = scene.get_state()
			if state and state.get_node_count() > 0:
				for i in state.get_node_property_count(0):
					if state.get_node_property_name(0, i) == &"script":
						script = state.get_node_property_value(0, i)
				if entry["node_class"] == "":
					entry["node_class"] = str(state.get_node_type(0))
		if script:
			entry["script"] = script.resource_path
		elif def.node_class != "" and not ClassDB.class_exists(def.node_class):
			script = FuncGodotEntityAssembler.get_script_by_class_name(def.node_class)
		_script_io(script, outputs, inputs)
		_class_io(str(entry["node_class"]), outputs)
		# Curated lists keep helper methods of entity scripts out of the editor's input pickers.
		for pair in [["outputs", outputs], ["inputs", inputs]]:
			var allowed = meta.get(pair[0], null)
			if allowed is Array and not allowed.is_empty():
				var list: Array = pair[1]
				var kept := list.filter(func(io): return io["name"] in allowed)
				for name_value in allowed:
					if not kept.any(func(io): return io["name"] == name_value):
						kept.append({ "name": name_value, "parameter": "" })
				list.assign(kept)
		entry["outputs"] = outputs
		entry["inputs"] = inputs
		entities.append(entry)

	var known := entities.map(func(e): return e["classname"])
	for cs_entry in GodotTrenchCSharp.scan(csharp_source_dirs):
		if not cs_entry["classname"] in known:
			entities.append(cs_entry)

	var ms := map_settings if map_settings else FuncGodotMapSettings.new()
	return {
		"format": "godottrench-game",
		"version": FORMAT_VERSION,
		"name": game_name,
		"units_per_meter": ms.inverse_scale_factor,
		"textures": {
			"base_dir": ms.base_texture_dir,
			"extensions": Array(ms.texture_file_extensions),
			"material_dir": ms.base_material_dir,
			"material_extension": ms.material_file_extension,
			"fallback_size": fallback_texture_size,
		},
		"tool_textures": { "clip": ms.clip_texture, "skip": ms.skip_texture, "origin": ms.origin_texture },
		"entities": entities,
	}

func export_file() -> Error:
	var text := JSON.stringify(build_config(), "  ", false)
	var file := FileAccess.open(output_path, FileAccess.WRITE)
	if not file:
		push_error("[GodotTrench] cannot write %s" % output_path)
		return ERR_FILE_CANT_WRITE
	file.store_string(text)
	file.close()
	print("[GodotTrench] game config exported to ", ProjectSettings.globalize_path(output_path))
	return OK
