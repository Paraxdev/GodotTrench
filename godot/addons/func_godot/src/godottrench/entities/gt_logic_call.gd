@tool
class_name GTLogicCall extends Node3D
## logic_call: calls a method in your game code when its trigger input fires, so map logic can reach GDScript or C#
## without writing an entity. Inputs: trigger(activator), call_with(parameter). Output: called(result).

signal called(result: Variant)

## Node path (/root/Game), @group, targetname, !activator, !player or !self.
@export var call_target := "/root/Game"
@export var method := ""
## JSON array. $activator, $self, $position, $caller_name and $parameter are replaced.
@export var arguments := "[]"

func _func_godot_apply_properties(props: Dictionary) -> void:
	call_target = str(props.get("call_target", call_target))
	method = str(props.get("method", method))
	arguments = str(props.get("arguments", arguments))

func trigger(activator: Node = null) -> void:
	called.emit(call_targets(self, call_target, method, arguments, activator))

## The parameter of the incoming output replaces $parameter in the arguments, or becomes the only argument.
func call_with(parameter: Variant = null) -> void:
	var value := JSON.stringify(parameter)
	var text := arguments.replace("\"$parameter\"", value) if arguments.contains("$parameter") else "[%s]" % value
	called.emit(call_targets(self, call_target, method, text, null))

## Calls [param method_name] on every node [param target] resolves to. Returns the last result.
static func call_targets(from: Node, target: String, method_name: String, args_json: String, activator: Node) -> Variant:
	if method_name == "":
		push_warning("[GT] %s has no method to call" % from.name)
		return null
	var result: Variant = null
	var targets := GodotTrenchIO.find_targets(from, target, activator)
	if targets.is_empty():
		push_warning("[GT] %s found no target '%s' for %s()" % [from.name, target, method_name])
	for node in targets:
		if GodotTrenchIO.resolve_method(node, method_name) == &"":
			push_warning("[GT] %s has no method %s (or %s)" % [node.name, method_name, method_name.to_pascal_case()])
			continue
		result = GodotTrenchIO.call_method(node, method_name, GodotTrenchIO.build_arguments(args_json, activator, from), from)
	GodotTrenchIO.events().fired.emit(from, &"call", target, StringName(method_name), args_json)
	return result
