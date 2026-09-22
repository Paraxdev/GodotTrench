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

## A C# numeric literal such as 3.5f, -2, 1e3d or 10m as text, else "".
static func _cs_number(text: String) -> String:
	var v := text.strip_edges()
	if v.length() > 1 and v[v.length() - 1].to_lower() in ["f", "d", "m"]:
		v = v.substr(0, v.length() - 1)
	return v if v.is_valid_float() else ""

## The numbers inside a C# constructor call like new Vector3(1, 2f, 3), or [] when any argument is not a literal.
static func _cs_call_numbers(text: String) -> Array:
	var open := text.find("(")
	var close := text.rfind(")")
	if open < 0 or close < open or text.substr(close + 1).strip_edges() != "":
		return []
	var out := []
	for part in text.substr(open + 1, close - open - 1).split(",", false):
		var n := _cs_number(part)
		if n == "":
			return []
		out.append(n.to_float())
	return out

const _CS_VECTORS := {
	"Zero": "0 0 0", "One": "1 1 1", "Up": "0 1 0", "Down": "0 -1 0", "Left": "-1 0 0", "Right": "1 0 0",
	"Forward": "0 0 -1", "Back": "0 0 1",
}

## The map default for a C# member initializer. An initializer this cannot read, such as 2 * Mathf.Pi, gives "", so
## the key stays empty and the member keeps the value its C# initializer gives it.
static func _cs_default(value: String, type: String) -> String:
	var v := value.strip_edges().trim_suffix(";").strip_edges()
	match type:
		"float", "int":
			if v == "":
				return "0"
			var n := _cs_number(v)
			if n != "" and type == "int" and not n.is_valid_int():
				return ""
			return n
		"bool":
			if v == "" or v == "false":
				return "0"
			return "1" if v == "true" else ""
		"vector3":
			if v == "":
				return "0 0 0"
			if v.begins_with("Vector3."):
				return _CS_VECTORS.get(v.trim_prefix("Vector3."), "")
			if v.begins_with("new"):
				var nums := _cs_call_numbers(v)
				if nums.size() == 3:
					return "%s %s %s" % [_gd_num(nums[0]), _gd_num(nums[1]), _gd_num(nums[2])]
			return ""
		"color":
			var color: Variant = null
			if v.begins_with("Colors."):
				var named := Color.from_string(v.trim_prefix("Colors."), Color(-1, -1, -1))
				color = named if named.r >= 0.0 else null
			elif v.begins_with("new") or v.begins_with("Color.FromHtml") or v.begins_with("Color.FromString"):
				var quoted := RegEx.create_from_string("\"([^\"]*)\"").search(v)
				if quoted:
					var parsed := Color.from_string(quoted.get_string(1), Color(-1, -1, -1))
					color = parsed if parsed.r >= 0.0 else null
				else:
					var nums := _cs_call_numbers(v)
					if nums.size() == 3 or nums.size() == 4:
						color = Color(nums[0], nums[1], nums[2])
			return "%d %d %d" % [color.r8, color.g8, color.b8] if color is Color else ""
		"resource":
			var path := RegEx.create_from_string("\"((?:res|uid)://[^\"]*)\"").search(v)
			return path.get_string(1) if path else ""
		"target_destination":
			var np := RegEx.create_from_string("^(?:new\\s+NodePath\\s*\\(\\s*)?\"([^\"]*)\"\\s*\\)?$").search(v)
			return np.get_string(1) if np else ""
	if v.length() >= 2 and v.begins_with("\"") and v.ends_with("\"") and not v.substr(1, v.length() - 2).contains("\""):
		return v.substr(1, v.length() - 2)
	return ""

static func _gd_num(f: float) -> String:
	return str(int(f)) if f == floorf(f) and absf(f) < 1.0e9 else str(f)

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
			# An empty default stays text, so the key arrives empty and apply_properties leaves the member alone.
			if str(p["default"]) == "":
				props[p["name"]] = ""
				continue
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
	var types := {}
	for prop in node.get_property_list():
		types[prop["name"]] = int(prop["type"])
	for key in properties:
		var member := str(key).to_pascal_case()
		if not member in node:
			continue
		var value: Variant = properties[key]
		# An empty key means the map did not set it, the member keeps its C# initializer.
		if value is String and value == "":
			continue
		var current: Variant = node.get(member)
		var type: int = types.get(member, typeof(current))
		match type:
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
			TYPE_NODE_PATH:
				value = NodePath(str(value))
			TYPE_OBJECT:
				if value is String:
					if not ResourceLoader.exists(value):
						push_warning("[GT] %s: resource %s for %s not found" % [node.name, value, member])
						continue
					value = load(value)
		node.set(member, value)
	return true
