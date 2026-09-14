class_name GodotTrenchCSharp extends RefCounted
## C# entity definitions without .tres files. C# classes marked with [GodotTrenchEntity("classname", ...)] are read
## from source text, so this works in any Godot build and in the editor export:
##
## [codeblock]
## [GlobalClass]
## [GodotTrenchEntity("npc_guard", Description = "Patrolling guard", Color = "#ff4040", Size = "-16 0 -16 16 64 16")]
## public partial class NpcGuard : CharacterBody3D
## {
##     [Signal] public delegate void AlertedEventHandler(Node activator);
##     [Export] public float Speed { get; set; } = 3.0f;
##     [GodotTrenchInput] public void Alert(Node activator) { }
## }
## [/codeblock]
##
## Map property names are the snake_case form of exported members (speed), inputs and outputs too (alert, alerted).
## The I/O runtime finds the PascalCase C# names, and properties are assigned to the matching C# members when the
## class has no _func_godot_apply_properties method.

const SETTING := "godottrench/csharp_entity_dirs"
const SKIP_DIRS := [".godot", ".import", "addons", "bin", "obj"]

static var _cache: Dictionary = {}
static var _cache_key := ""

static func source_dirs() -> PackedStringArray:
	if ProjectSettings.has_setting(SETTING):
		return PackedStringArray(ProjectSettings.get_setting(SETTING))
	return PackedStringArray(["res://"])

static func _collect_files(dir: String, out: PackedStringArray) -> void:
	var da := DirAccess.open(dir)
	if not da:
		return
	for f in da.get_files():
		if f.get_extension().to_lower() == "cs":
			out.append(dir.path_join(f))
	for d in da.get_directories():
		if not d in SKIP_DIRS and not d.begins_with("."):
			_collect_files(dir.path_join(d), out)

static func _attribute_args(text: String) -> Dictionary:
	var out := { "positional": [] }
	var rx := RegEx.create_from_string("(\\w+)\\s*=\\s*(\"(?:[^\"\\\\]|\\\\.)*\"|[\\w.+-]+)|(\"(?:[^\"\\\\]|\\\\.)*\")")
	for m in rx.search_all(text):
		if m.get_string(3) != "":
			out["positional"].append(m.get_string(3).substr(1, m.get_string(3).length() - 2))
		else:
			var value := m.get_string(2)
			if value.begins_with("\""):
				value = value.substr(1, value.length() - 2)
			out[m.get_string(1)] = value
	return out

static func _cs_type(cs: String) -> String:
	match cs.strip_edges():
		"float", "double":
			return "float"
		"int", "long", "uint", "short":
			return "int"
		"bool":
			return "bool"
		"Vector3":
			return "vector3"
		"Color":
			return "color"
		"NodePath":
			return "target_destination"
		"PackedScene", "Resource", "Texture2D", "AudioStream":
			return "resource"
	return "string"

static func _cs_default(value: String, type: String) -> String:
	var v := value.strip_edges().trim_suffix(";").strip_edges()
	match type:
		"float", "int":
			return v.trim_suffix("f").trim_suffix("d") if v != "" else "0"
		"bool":
			return "1" if v == "true" else "0"
		"vector3":
			var inside := v.substr(v.find("(") + 1) if v.contains("(") else v
			var nums := RegEx.create_from_string("-?[\\d.]+").search_all(inside).map(func(m): return m.get_string().trim_suffix("f"))
			return " ".join(nums) if nums.size() >= 3 else "0 0 0"
	return v.trim_prefix("\"").trim_suffix("\"")

static func _params(text: String) -> String:
	var names: PackedStringArray = []
	for part in text.split(",", false):
		var bits := part.strip_edges().split(" ", false)
		if bits.size() >= 2:
			names.append(bits[bits.size() - 1].split("=")[0].strip_edges().to_snake_case())
	return ", ".join(names)

## Entity descriptions found in one C# source text.
static func parse_source(text: String, path: String = "") -> Array[Dictionary]:
	var out: Array[Dictionary] = []
	var class_rx := RegEx.create_from_string("\\[GodotTrenchEntity\\(([^\\]]*)\\)\\][\\s\\S]*?class\\s+(\\w+)\\s*:\\s*([\\w.]+)")
	var matches := class_rx.search_all(text)
	for i in matches.size():
		var m := matches[i]
		var body_end := matches[i + 1].get_start() if i + 1 < matches.size() else text.length()
		var body := text.substr(m.get_end(), body_end - m.get_end())
		var args := _attribute_args(m.get_string(1))
		var positional: Array = args["positional"]
		var classname := str(positional[0]) if positional.size() > 0 else m.get_string(2).to_snake_case()
		var entry := {
			"classname": classname,
			"type": "solid" if str(args.get("Solid", "false")) == "true" else "point",
			"description": str(args.get("Description", "")),
			"color": str(args.get("Color", "#cc80ffff")),
			"node_class": m.get_string(2),
			"base_class": m.get_string(3),
			"script": path,
			"group": classname.get_slice("_", 0),
			"csharp": true,
			"properties": [],
			"outputs": [],
			"inputs": [],
		}
		var size := str(args.get("Size", "")).split_floats(" ", false)
		if size.size() >= 6:
			entry["size"] = [[size[0], size[1], size[2]], [size[3], size[4], size[5]]]
		for s in RegEx.create_from_string("\\[Signal\\]\\s*public\\s+delegate\\s+void\\s+(\\w+)EventHandler\\s*\\(([^)]*)\\)").search_all(body):
			entry["outputs"].append({ "name": s.get_string(1).to_snake_case(), "parameter": _params(s.get_string(2)) })
		var member_rx := RegEx.create_from_string("\\[Export[^\\]]*\\]\\s*public\\s+([\\w<>.]+)\\s+(\\w+)\\s*(?:\\{[^}]*\\})?\\s*(?:=\\s*([^;\\n]+))?")
		for e in member_rx.search_all(body):
			var type := _cs_type(e.get_string(1))
			entry["properties"].append({
				"name": e.get_string(2).to_snake_case(),
				"type": type,
				"default": _cs_default(e.get_string(3), type),
				"description": "",
			})
		var marked := RegEx.create_from_string("\\[GodotTrenchInput[^\\]]*\\]\\s*public\\s+[\\w<>.]+\\s+(\\w+)\\s*\\(([^)]*)\\)").search_all(body)
		var methods := marked if not marked.is_empty() else RegEx.create_from_string("public\\s+void\\s+([A-Z]\\w*)\\s*\\(([^)]*)\\)").search_all(body)
		for meth in methods:
			entry["inputs"].append({ "name": meth.get_string(1).to_snake_case(), "parameter": _params(meth.get_string(2)) })
		if not entry["properties"].any(func(p): return p["name"] == "targetname"):
			entry["properties"].push_front({ "name": "targetname", "type": "target_source", "default": "", "description": "Name" })
		out.append(entry)
	return out

## Signals and public void methods of a C# class by name, for FGD entries whose node_class is a C# global class.
## Returns {"outputs": [...], "inputs": [...]} or an empty Dictionary when no source declares the class.
static func class_io(class_name_text: String, dirs: PackedStringArray = source_dirs()) -> Dictionary:
	var files: PackedStringArray = []
	for d in dirs:
		_collect_files(d, files)
	var decl := RegEx.create_from_string("class\\s+%s\\b" % class_name_text)
	for f in files:
		var text := FileAccess.get_file_as_string(f)
		var m := decl.search(text)
		if not m:
			continue
		var body := text.substr(m.get_end())
		var next_class := RegEx.create_from_string("\\n\\s*(?:public\\s+)?(?:partial\\s+)?class\\s+\\w+").search(body)
		if next_class:
			body = body.substr(0, next_class.get_start())
		var outputs: Array = []
		var inputs: Array = []
		for s in RegEx.create_from_string("\\[Signal\\]\\s*public\\s+delegate\\s+void\\s+(\\w+)EventHandler\\s*\\(([^)]*)\\)").search_all(body):
			outputs.append({ "name": s.get_string(1).to_snake_case(), "parameter": _params(s.get_string(2)) })
		for meth in RegEx.create_from_string("public\\s+void\\s+([A-Z]\\w*)\\s*\\(([^)]*)\\)").search_all(body):
			inputs.append({ "name": meth.get_string(1).to_snake_case(), "parameter": _params(meth.get_string(2)) })
		return { "outputs": outputs, "inputs": inputs, "script": f }
	return {}

## Every [GodotTrenchEntity] class under [param dirs], cached until a source file changes.
static func scan(dirs: PackedStringArray = source_dirs()) -> Array[Dictionary]:
	var files: PackedStringArray = []
	for d in dirs:
		_collect_files(d, files)
	var key := ""
	for f in files:
		key += "%s:%d;" % [f, FileAccess.get_modified_time(f)]
	if key == _cache_key and _cache.has("entries"):
		return _cache["entries"]
	var entries: Array[Dictionary] = []
	for f in files:
		var text := FileAccess.get_file_as_string(f)
		if text.contains("GodotTrenchEntity"):
			entries.append_array(parse_source(text, f))
	_cache = { "entries": entries }
	_cache_key = key
	return entries

## FuncGodot definitions for C# entities, merged into the map's FGD definitions at build time.
static func definitions(dirs: PackedStringArray = source_dirs()) -> Dictionary:
	var out := {}
	for e in scan(dirs):
		var def: FuncGodotFGDEntityClass = FuncGodotFGDSolidClass.new() if e["type"] == "solid" else FuncGodotFGDPointClass.new()
		def.classname = e["classname"]
		def.description = e["description"]
		def.node_class = e["node_class"]
		var props: Dictionary[String, Variant] = {}
		for p in e["properties"]:
			match p["type"]:
				"float":
					props[p["name"]] = float(p["default"])
				"int":
					props[p["name"]] = int(p["default"])
				"bool":
					props[p["name"]] = p["default"] == "1"
				_:
					props[p["name"]] = str(p["default"])
		def.class_properties = props
		def.meta_properties = { "color": Color.from_string(e["color"], Color(0.8, 0.5, 1.0)), "csharp": true }
		if def is FuncGodotFGDSolidClass:
			def.collision_shape_type = FuncGodotFGDSolidClass.CollisionShapeType.CONVEX
		out[e["classname"]] = def
	return out

## Assigns map properties to C# members by their PascalCase names. Returns false when the node handles them itself.
static func apply_properties(node: Node, properties: Dictionary) -> bool:
	if node.has_method("_func_godot_apply_properties"):
		return false
	for key in properties:
		var member := str(key).to_pascal_case()
		if member in node:
			var current: Variant = node.get(member)
			var value: Variant = properties[key]
			match typeof(current):
				TYPE_VECTOR3:
					value = GodotTrenchIO.to_vector3(value)
				TYPE_COLOR:
					value = GodotTrenchIO.to_color(value)
				TYPE_BOOL:
					value = GodotTrenchIO.to_bool(value)
				TYPE_FLOAT:
					value = float(value)
				TYPE_INT:
					value = int(value)
			node.set(member, value)
	return true
