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

## The node that scopes targetname lookups: the enclosing FuncGodotMap, else the scene root.
static func lookup_root(from: Node) -> Node:
	var n := from
	while n:
		if n is FuncGodotMap:
			return n
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
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n.has_meta(TARGETNAME_META):
			var key := str(n.get_meta(TARGETNAME_META))
			if not cache.has(key):
				cache[key] = []
			cache[key].append(n)
		stack.append_array(n.get_children())
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

## Declared argument count of a method, -1 when unknown (C# methods may not report it).
static func method_arg_count(node: Object, method: StringName) -> int:
	for m in node.get_method_list():
		if m["name"] == method:
			return m["args"].size()
	return -1

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

## Calls [param method] on [param node] with [param args], trimming to the declared argument count and converting
## simple values to the declared parameter types.
static func call_method(node: Object, method: StringName, args: Array) -> Variant:
	var resolved := resolve_method(node, method)
	if resolved == &"":
		return null
	var call_args := args.duplicate()
	for m in node.get_method_list():
		if m["name"] == resolved:
			var declared: Array = m["args"]
			call_args.resize(mini(call_args.size(), declared.size()))
			for i in call_args.size():
				call_args[i] = coerce(call_args[i], int(declared[i].get("type", TYPE_NIL)))
			break
	return node.callv(resolved, call_args)

## Delivers an input to a node: a method, a property, or one of the built-in inputs.
static func invoke(node: Node, input: StringName, parameter: String, activator: Node) -> void:
	if not is_instance_valid(node):
		return
	var method := resolve_method(node, input)
	if method != &"":
		var args := build_arguments(parameter, activator, node)
		var count := method_arg_count(node, method)
		if args.is_empty() and count > 0:
			args = [activator]
		call_method(node, method, args)
		return
	var value: Variant = parse_parameter(parameter)
	if input in node and value != null:
		node.set(input, value)
		return
	match str(input).to_lower():
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
			push_warning("[GT I/O] %s has no input '%s'" % [node.name, input])

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
