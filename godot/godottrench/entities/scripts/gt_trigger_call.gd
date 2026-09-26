@tool
class_name GTTriggerCall extends GTTrigger
## trigger_call: calls [member method] on [member call_target] when a body enters, with [member arguments] as a JSON
## array ($activator, $self, $position and $caller_name are replaced). Works with GDScript and C# (PascalCase) methods.
## Input: trigger(activator). Output: called(result).

signal called(result: Variant)

## Node path (/root/Game), @group, targetname, !activator, !player or !self.
@export var call_target := "!activator"
@export var method := ""
@export var arguments := "[]"

func _func_godot_apply_properties(props: Dictionary) -> void:
	super(props)
	call_target = str(props.get("call_target", call_target))
	method = str(props.get("method", method))
	arguments = str(props.get("arguments", arguments))

func trigger(activator: Node = null) -> void:
	fire(activator)

func _on_triggered(activator: Node) -> void:
	called.emit(GTLogicCall.call_targets(self, call_target, method, arguments, activator))
