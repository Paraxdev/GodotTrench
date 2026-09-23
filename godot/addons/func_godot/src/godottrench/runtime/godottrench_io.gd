class_name GodotTrenchIO extends RefCounted
## Hammer style entity input/output support for maps built from GodotTrench .gtm files.
##
## Named entities store their targetname as node metadata, and every output becomes a [GodotTrenchOutput]
## child that connects itself to the parent's signal when it enters the tree. Nothing depends on
## persistent groups or connections, so saved and runtime built scenes behave the same.
##
## Targets can be a targetname (with a trailing * wildcard), [code]!self[/code], [code]!activator[/code],
## [code]!player[/code] (first node in the "player" group), [code]@group[/code] for every node in a group, or an absolute
## node path such as [code]/root/Game[/code] to reach autoloads. Inputs call methods by name, falling back to the
## PascalCase spelling so C# methods work with snake_case names in the map.

const TARGETNAME_META := &"gt_targetname"
const CACHE_META := &"gt_target_cache"

## Emits [signal fired] for every output that fires anywhere, for debug overlays, logs and achievements.
class Events extends RefCounted:
	signal fired(source: Node, output: StringName, target: String, input: StringName, parameter: String)

static var _events: Events

static func events() -> Events:
	if not _events:
		_events = Events.new()
	return _events

## Called by the entity assembler once all entity nodes exist.
static func setup(entities: Array[FuncGodotData.EntityData], scene_root: Node) -> void:
	for data in entities:
		if not data.node:
			continue
		var targetname := str(data.properties.get("targetname", ""))
		if targetname != "":
			data.node.set_meta(TARGETNAME_META, targetname)

	for data in entities:
		if not data.node or data.outputs.is_empty():
			continue
		var index := 0
		for conn in data.outputs:
			var output := StringName(str(conn.get("output", "")))
			if output == &"":
				continue
			var relay := GodotTrenchOutput.new()
			relay.name = "gt_output_%d_%s" % [index, output]
			relay.output = output
			relay.target = str(conn.get("target", ""))
			relay.input = StringName(str(conn.get("input", "")))
			relay.parameter = str(conn.get("parameter", ""))
			relay.delay = float(conn.get("delay", 0.0))
			relay.times = int(conn.get("times", -1))
			data.node.add_child(relay)
			relay.owner = scene_root
			index += 1
			if not data.node.has_signal(output) and not data.node.has_signal(StringName(str(output).to_pascal_case())):
				push_warning("[GT I/O] %s has no signal '%s', fire it with GodotTrenchIO.fire_output()" % [data.node.name, output])

## Fires every output named [param output] on [param source] manually, e.g. for entities without that signal.
static func fire_output(source: Node, output: StringName, activator: Node = null) -> void:
	for child in source.get_children():
		if child is GodotTrenchOutput and (child.output == output or str(child.output).to_pascal_case() == str(output)):
			child.fire(activator)

## The node that scopes targetname lookups: the enclosing FuncGodotMap (or the map an overlay outside it points at),
## else the scene root.
static func lookup_root(from: Node) -> Node:
	var n := from
	while n:
		if n is FuncGodotMap:
			return n
		if n is GodotTrenchOverlay and not (n as GodotTrenchOverlay).map_path.is_empty():
			var map := (n as GodotTrenchOverlay).get_map()
			if map:
				return map
		n = n.get_parent()
	var tree := from.get_tree()
	if tree and tree.current_scene:
		return tree.current_scene
	return from.get_tree().root if from.get_tree() else from

static func _named_nodes(root: Node) -> Dictionary:
	var cache: Dictionary = root.get_meta(CACHE_META, {}) if root.has_meta(CACHE_META) else {}
	var valid := not cache.is_empty()
	for list in cache.values():
		for n in list:
			if not is_instance_valid(n):
				valid = false
	if valid:
		return cache
	cache = {}
	var stack: Array[Node] = [root]
	if root is FuncGodotMap:
		stack.append_array(GodotTrenchOverlay.external_overlays(root))
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n.has_meta(TARGETNAME_META):
			var key := str(n.get_meta(TARGETNAME_META))
			if not cache.has(key):
				cache[key] = []
			cache[key].append(n)
		stack.append_array(n.get_children())
	# Metadata would be saved with an edited scene.
	if not Engine.is_editor_hint():
		root.set_meta(CACHE_META, cache)
	return cache

## Forgets cached targetname lookups, call after spawning or renaming named entities at runtime.
static func invalidate(from: Node) -> void:
	var root := lookup_root(from)
	if root.has_meta(CACHE_META):
		root.remove_meta(CACHE_META)

static func find_targets(from: Node, target: String, activator: Node) -> Array[Node]:
	var out: Array[Node] = []
	if target == "":
		return out
	match target:
		"!self":
			out.append(from.get_parent() if from is GodotTrenchOutput else from)
			return out
		"!activator", "!caller":
			if activator:
				out.append(activator)
			return out
		"!player":
			var tree := from.get_tree()
			var player := tree.get_first_node_in_group(&"player") if tree else null
			if player:
				out.append(player)
			return out
	if target.begins_with("@"):
		var tree := from.get_tree()
		if tree:
			out.append_array(tree.get_nodes_in_group(StringName(target.substr(1))))
		return out
	if target.begins_with("/"):
		var node := from.get_node_or_null(NodePath(target)) if from.is_inside_tree() else null
		if node:
			out.append(node)
		return out
	var named := _named_nodes(lookup_root(from))
	if target.ends_with("*"):
		var prefix := target.trim_suffix("*")
		for key in named:
			if str(key).begins_with(prefix):
				for n in named[key]:
					if is_instance_valid(n):
						out.append(n)
	else:
		for n in named.get(target, []):
			if is_instance_valid(n):
				out.append(n)
	return out

static func parse_parameter(parameter: String) -> Variant:
	if parameter == "":
		return null
	if parameter.is_valid_int():
		return parameter.to_int()
	if parameter.is_valid_float():
		return parameter.to_float()
	var lower := parameter.to_lower()
	if lower == "true" or lower == "false":
		return lower == "true"
	var parts := parameter.split(" ", false)
	if parts.size() == 3 and parts[0].is_valid_float() and parts[1].is_valid_float() and parts[2].is_valid_float():
		return Vector3(parts[0].to_float(), parts[1].to_float(), parts[2].to_float())
	return parameter

## Name of a method on [param node] matching [param input] exactly, in PascalCase (C#) or camelCase, else "".
static func resolve_method(node: Object, input: StringName) -> StringName:
	if node.has_method(input):
		return input
	for variant in [str(input).to_pascal_case(), str(input).to_camel_case()]:
		if node.has_method(variant):
			return StringName(variant)
	return &""

## Declared arguments and defaults of [param method] on [param node], as get_method_list() reports them.
static func _method_info(node: Object, method: StringName) -> Dictionary:
	for m in node.get_method_list():
		if m["name"] == method:
			return m
	return {}

## True for an argument that takes a node: typed as an object, or untyped and named activator.
static func _takes_node(declared: Dictionary) -> bool:
	var type := int(declared.get("type", TYPE_NIL))
	return type == TYPE_OBJECT or (type == TYPE_NIL and str(declared.get("name", "")) == "activator")

## What an argument gets when nothing is passed for it: its declared default, else the empty value of its type.
static func _default_argument(info: Dictionary, index: int) -> Variant:
	var declared: Array = info.get("args", [])
	var defaults: Array = info.get("default_args", [])
	var first_default := declared.size() - defaults.size()
	if index >= first_default and index - first_default < defaults.size():
		return defaults[index - first_default]
	var type := int(declared[index].get("type", TYPE_NIL)) if index < declared.size() else TYPE_NIL
	return null if type == TYPE_NIL or type == TYPE_OBJECT else type_convert(null, type)

## Index of the first argument not in [param slots] that takes a node ([param nodes]) or a value, else -1.
static func _first_free(declared: Array, slots: Dictionary, nodes: bool) -> int:
	for i in declared.size():
		if not slots.has(i) and _takes_node(declared[i]) == nodes:
			return i
	return -1

## Argument list from [param slots] (index to value), the method's defaults filling the gaps up to the last one set.
static func _slots_to_args(info: Dictionary, slots: Dictionary) -> Array:
	var args: Array = []
	if slots.is_empty():
		return args
	var last: int = slots.keys().max()
	for i in last + 1:
		args.append(slots[i] if slots.has(i) else _default_argument(info, i))
	return args

static func _place_activator(info: Dictionary, slots: Dictionary, activator: Node) -> void:
	if not activator:
		return
	var at := _first_free(info.get("args", []), slots, true)
	if at >= 0:
		slots[at] = activator

## Arguments for an input fired without a parameter: the activator goes into the first argument typed as an object or
## named activator, wherever it is, and the method's defaults fill the arguments before it. Inputs taking only values,
## like a counter's add(amount), are called without arguments so their defaults apply instead of receiving the player.
static func activator_arguments(node: Object, method: StringName, activator: Node) -> Array:
	var info := _method_info(node, method)
	var slots := {}
	_place_activator(info, slots, activator)
	return _slots_to_args(info, slots)

## Arguments for an input. A parameter goes into the first value argument, or into a leading node argument when it
## names a target (!player, !self, a targetname). Without a parameter, [param values] an output passed along (the
## hp of damaged(hp)) fill the arguments of their kind in order. The activator then takes the first free node
## argument, and the method's defaults fill the gaps. [param from] resolves target names, !self meaning its entity.
static func input_arguments(node: Object, method: StringName, parameter: String, activator: Node, from: Node = null, values: Array = []) -> Array:
	var info := _method_info(node, method)
	var declared: Array = info.get("args", [])
	var slots := {}
	var text := parameter.strip_edges()
	if parameter == "":
		_place_activator(info, slots, activator)
		for v in values:
			var at := _first_free(declared, slots, typeof(v) == TYPE_OBJECT)
			if at >= 0:
				slots[at] = v
		return _slots_to_args(info, slots)
	if text.begins_with("[") and JSON.parse_string(text) is Array:
		var list := build_arguments(parameter, activator, node as Node)
		if declared.is_empty():
			return list
		for i in list.size():
			slots[i] = list[i]
		_place_activator(info, slots, activator)
		return _slots_to_args(info, slots)
	var value: Variant = substitute(parse_parameter(parameter), activator, node as Node)
	var node_at := _first_free(declared, slots, true)
	var value_at := _first_free(declared, slots, false)
	var as_node: Object = null
	if node_at >= 0 and (value_at < 0 or node_at < value_at):
		as_node = _as_object(value, from if from else node as Node, activator)
	if as_node:
		slots[node_at] = as_node
	elif value_at >= 0 and typeof(value) != TYPE_OBJECT:
		slots[value_at] = value
	_place_activator(info, slots, activator)
	return _slots_to_args(info, slots)

## [param value] as a node: nodes pass through, text is resolved as a target from [param from], anything else is null.
static func _as_object(value: Variant, from: Node, activator: Node) -> Object:
	if typeof(value) == TYPE_OBJECT:
		return value if is_instance_valid(value) else null
	if (value is String or value is StringName) and str(value) != "" and from and from.is_inside_tree():
		var found := find_targets(from, str(value), activator)
		return found[0] if not found.is_empty() else null
	return null

## Arguments for a call: a JSON array parameter spreads into several arguments, placeholders are replaced.
static func build_arguments(parameter: String, activator: Node, caller: Node) -> Array:
	var text := parameter.strip_edges()
	if text.begins_with("["):
		var parsed = JSON.parse_string(text)
		if parsed is Array:
			return parsed.map(func(a): return substitute(a, activator, caller))
	var value: Variant = parse_parameter(parameter)
	return [] if value == null else [substitute(value, activator, caller)]

## Replaces "$activator", "$self", "$position" and "$caller_name" in call arguments.
static func substitute(value: Variant, activator: Node, caller: Node) -> Variant:
	if not value is String:
		return value
	match value:
		"$activator":
			return activator
		"$self", "$caller":
			return caller
		"$position":
			return caller.global_position if caller is Node3D and caller.is_inside_tree() else Vector3.ZERO
		"$caller_name":
			return str(caller.get_meta(TARGETNAME_META, caller.name)) if caller else ""
	return value

## Converts a map value to a declared argument type, so "1234" reaches a String parameter and 5 an int one.
static func coerce(value: Variant, type: int) -> Variant:
	if type == TYPE_NIL or typeof(value) == type or value == null:
		return value
	match type:
		TYPE_STRING, TYPE_STRING_NAME:
			return str(value) if not value is Object else value
		TYPE_INT:
			return int(value) if value is float or value is bool or (value is String and value.is_valid_int()) else value
		TYPE_FLOAT:
			return float(value) if value is int or (value is String and value.is_valid_float()) else value
		TYPE_BOOL:
			return to_bool(value)
		TYPE_VECTOR3:
			return to_vector3(value) if value is String else value
		TYPE_COLOR:
			return to_color(value) if value is String else value
	return value

## True when [param value] can be passed to an argument of [param type] without Godot refusing the call.
static func _fits(value: Variant, type: int) -> bool:
	match type:
		TYPE_STRING, TYPE_STRING_NAME:
			return value is String or value is StringName
		TYPE_INT, TYPE_FLOAT, TYPE_BOOL:
			return value is int or value is float or value is bool
		TYPE_VECTOR3, TYPE_COLOR:
			return typeof(value) == type
	return true

## Calls [param method] on [param node] with [param args], trimming to the declared argument count and converting
## simple values to the declared parameter types. Text for a node argument is resolved as a target from [param from],
## and a value that still does not fit its argument is replaced by the argument's default, so a call never fails on
## a type mismatch.
static func call_method(node: Object, method: StringName, args: Array, from: Node = null) -> Variant:
	var resolved := resolve_method(node, method)
	if resolved == &"":
		return null
	var call_args := args.duplicate()
	var info := _method_info(node, resolved)
	if not info.is_empty():
		var declared: Array = info["args"]
		call_args.resize(mini(call_args.size(), declared.size()))
		for i in call_args.size():
			var type := int(declared[i].get("type", TYPE_NIL))
			if type == TYPE_OBJECT:
				call_args[i] = _as_object(call_args[i], from if from else node as Node, null)
				continue
			var value: Variant = coerce(call_args[i], type)
			if not _fits(value, type):
				push_warning("[GT I/O] %s.%s: '%s' does not fit argument %s, using its default" % [node, resolved, str(value), declared[i].get("name", i)])
				value = _default_argument(info, i)
			call_args[i] = value
	return node.callv(resolved, call_args)

## True when the script on [param node] (GDScript or C#) defines [param input] itself, not only its engine class.
static func _script_defines(node: Object, input: StringName) -> bool:
	var script: Script = node.get_script()
	var wanted := str(input).to_lower()
	while script:
		for m in script.get_script_method_list():
			if str(m["name"]).to_lower() == wanted:
				return true
		script = script.get_base_script()
	return false

## Delivers an input to a node: a method, a property, or one of the built-in inputs. [param caller] is the output
## (or node) the call comes from, for resolving target names in the parameter. With an empty parameter,
## [param values] are the values the output passed along.
static func invoke(node: Node, input: StringName, parameter: String, activator: Node, caller: Node = null, values: Array = []) -> void:
	if not is_instance_valid(node):
		return
	var listened := false
	for child in node.get_children():
		if child is GodotTrenchOverlayIO:
			child.input_received.emit(input, parameter, activator)
			listened = true
	var lower := str(input).to_lower()
	# Node3D and CanvasItem have native show() and hide() that only change visibility. The built in inputs also
	# stop processing, so they win unless the entity's own script defines show or hide.
	var builtin_visibility := (lower == "show" or lower == "hide") and not _script_defines(node, input)
	var method := &"" if builtin_visibility else resolve_method(node, input)
	if method != &"":
		call_method(node, method, input_arguments(node, method, parameter, activator, caller, values), caller)
		return
	var value: Variant = parse_parameter(parameter) if parameter != "" else (values[0] if not values.is_empty() else null)
	if not builtin_visibility and input in node and value != null:
		node.set(input, value)
		return
	match lower:
		"kill":
			node.queue_free()
		"show", "enable":
			if "visible" in node:
				node.visible = true
			node.process_mode = Node.PROCESS_MODE_INHERIT
		"hide", "disable":
			if "visible" in node:
				node.visible = false
			node.process_mode = Node.PROCESS_MODE_DISABLED
		"toggle":
			if "visible" in node:
				node.visible = not node.visible
		_:
			if not listened:
				push_warning("[GT I/O] %s has no input '%s'" % [node.name, input])

## Makes the fixture of a light (nodes named [param target], a trailing * matches a prefix) follow its state: shown
## while on and hidden while off, or with [param mode] "dark" always shown with the emission of its materials off
## while the light is off.
static func apply_fixture(light: Node, target: String, on: bool, mode: String = "hide") -> void:
	if target == "" or not light.is_inside_tree():
		return
	for n in find_targets(light, target, null):
		if n == light:
			continue
		if mode == "dark":
			if "visible" in n:
				n.visible = true
			_set_emission(n, on)
		elif "visible" in n:
			n.visible = on

const FIXTURE_META := &"gt_fixture_overrides"

static func _set_emission(node: Node, on: bool) -> void:
	var stack: Array[Node] = [node]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		stack.append_array(n.get_children())
		var mi := n as MeshInstance3D
		if not mi or not mi.mesh:
			continue
		var count := mi.mesh.get_surface_count()
		if not mi.has_meta(FIXTURE_META):
			var own: Array = []
			for i in count:
				own.append(mi.get_surface_override_material(i))
			mi.set_meta(FIXTURE_META, own)
		var own: Array = mi.get_meta(FIXTURE_META)
		for i in count:
			var original: Material = own[i] if i < own.size() else null
			if on:
				mi.set_surface_override_material(i, original)
				continue
			var base := original if original else mi.mesh.surface_get_material(i)
			if base is BaseMaterial3D and (base as BaseMaterial3D).emission_enabled:
				var dark := base.duplicate() as BaseMaterial3D
				dark.emission_enabled = false
				mi.set_surface_override_material(i, dark)

## Map property helpers for entity scripts, map values arrive as strings.
static func to_bool(value: Variant) -> bool:
	if value is bool:
		return value
	var s := str(value).strip_edges().to_lower()
	return s == "1" or s == "true" or s == "yes"

static func to_vector3(value: Variant) -> Vector3:
	if value is Vector3:
		return value
	var p := str(value).split_floats(" ", false)
	return Vector3(p[0], p[1], p[2]) if p.size() >= 3 else Vector3.ZERO

## "r g b" in 0..255 or 0..1.
static func to_color(value: Variant) -> Color:
	if value is Color:
		return value
	var p := str(value).split_floats(" ", false)
	if p.size() < 3:
		return Color.WHITE
	var scale := 255.0 if p[0] > 1.0 or p[1] > 1.0 or p[2] > 1.0 else 1.0
	return Color(p[0] / scale, p[1] / scale, p[2] / scale)

## Map units ("x y z") to meters in Godot, using the map's scale.
static func map_vector(value: Variant, units_per_meter: float = 32.0) -> Vector3:
	return to_vector3(value) / maxf(units_per_meter, 0.0001)

## Units per meter of the map that built [param node], 32 when unknown.
static func units_per_meter(node: Node) -> float:
	var n := node
	while n:
		if n is FuncGodotMap and n.map_settings:
			return n.map_settings.inverse_scale_factor
		n = n.get_parent()
	return 32.0
