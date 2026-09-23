@tool
@icon("res://addons/func_godot/icons/icon_overlay_io.svg")
class_name GodotTrenchOverlayIO extends Node
## Connects its parent to the entity I/O of the map it sits in, usually inside a [GodotTrenchOverlay].
##
## [member targetname] makes the parent a target for map outputs, like an entity with that targetname. An input calls
## the parent's method of that name, sets its property of that name to the parameter, or runs a built in input
## ([code]show[/code], [code]hide[/code], [code]toggle[/code], [code]enable[/code], [code]disable[/code],
## [code]kill[/code]). Every input is also emitted as [signal input_received], so a node without a script can react
## through a signal connection made in the editor.
##
## To fire at map entities, add [GodotTrenchOutput] children to the parent: they listen to the parent's signals like
## entity outputs do. Scripts can also call [method fire] for an output that is not a signal.

## Targetname map outputs use to reach the parent.
@export var targetname := "":
	set(value):
		targetname = value
		if is_inside_tree():
			_apply()

## A map output reached the parent with [param input].
signal input_received(input: StringName, parameter: String, activator: Node)

var _named: Node

func _enter_tree() -> void:
	_apply()

func _exit_tree() -> void:
	_release()

# The targetname lives on the parent as metadata only while the game runs, so the saved scene never keeps a stale one.
func _apply() -> void:
	if Engine.is_editor_hint():
		return
	_release()
	var parent := get_parent()
	if not parent or targetname == "":
		return
	parent.set_meta(GodotTrenchIO.TARGETNAME_META, targetname)
	_named = parent
	GodotTrenchIO.invalidate(parent)

func _release() -> void:
	if is_instance_valid(_named) and _named.has_meta(GodotTrenchIO.TARGETNAME_META):
		_named.remove_meta(GodotTrenchIO.TARGETNAME_META)
		GodotTrenchIO.invalidate(_named)
	_named = null

## Fires the parent's [GodotTrenchOutput] children named [param output], whether or not the parent has that signal.
func fire(output: StringName, activator: Node = null) -> void:
	var parent := get_parent()
	if parent:
		GodotTrenchIO.fire_output(parent, output, activator)
