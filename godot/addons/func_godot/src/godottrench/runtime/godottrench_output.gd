@tool
class_name GodotTrenchOutput extends Node
## One Hammer style output connection. Listens to [member output] on its parent entity.

@export var output: StringName
## Receivers: targetname (trailing * wildcard), !self, !activator, !player, @group or a /root/... node path.
@export var target: String
@export var input: StringName
## Passed to the input. A JSON array such as [10, "$activator"] spreads into several arguments.
@export var parameter: String
@export var delay: float = 0.0
## -1 fires every time, otherwise the number of times this output may fire.
@export var times: int = -1

signal fired(targets: Array[Node])

var _fired := 0
var _callable := Callable(self, "fire")

## The parent's signal for [member output], also accepting the PascalCase name C# signals get.
func _signal_name(parent: Node) -> StringName:
	if parent.has_signal(output):
		return output
	var pascal := StringName(str(output).to_pascal_case())
	return pascal if parent.has_signal(pascal) else &""

func _enter_tree() -> void:
	var parent := get_parent()
	if not parent:
		return
	var sig := _signal_name(parent)
	if sig != &"" and not parent.is_connected(sig, _callable):
		parent.connect(sig, _callable)

func _exit_tree() -> void:
	var parent := get_parent()
	if not parent:
		return
	var sig := _signal_name(parent)
	if sig != &"" and parent.is_connected(sig, _callable):
		parent.disconnect(sig, _callable)

func fire(...args: Array) -> void:
	if Engine.is_editor_hint():
		return
	if times >= 0 and _fired >= times:
		return
	_fired += 1
	var activator: Node = null
	for a in args:
		if a is Node:
			activator = a
			break
	if delay > 0.0:
		await get_tree().create_timer(delay).timeout
	if not is_inside_tree():
		return
	GodotTrenchIO.events().fired.emit(get_parent(), output, target, input, parameter)
	var targets := GodotTrenchIO.find_targets(self, target, activator)
	for t in targets:
		GodotTrenchIO.invoke(t, input, parameter, activator)
	fired.emit(targets)
