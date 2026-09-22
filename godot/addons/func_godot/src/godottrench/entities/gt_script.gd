@tool
class_name GTScript extends Node3D
## logic_script: the deeper route. It runs an inline GDScript snippet when fired, so map logic can do anything game
## code can, such as reading and lowering a player's health, spawning nodes or branching, without shipping an entity.
## Inputs: run(activator), run_with(parameter). Output: ran(result).
##
## The snippet is a function body with these names in scope: this (this node), activator (the firing or !activator
## node), parameter (the incoming value), io (the GodotTrenchIO helpers) and tree (the SceneTree). Set expression
## instead for a one liner. Snippets are author provided GDScript and run at the map's own trust level.

signal ran(result: Variant)

@export_multiline var source := ""
## A single expression evaluated when there is no source, with the same names in scope.
@export var expression := ""

var _instance: Object
var _expr: Expression

func _func_godot_apply_properties(props: Dictionary) -> void:
	source = str(props.get("source", source))
	expression = str(props.get("expression", expression))

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	_compile()

func _compile() -> void:
	if source.strip_edges() != "":
		var body := ""
		for line in source.split("\n"):
			body += "\t" + line + "\n"
		var script := GDScript.new()
		script.source_code = "extends RefCounted\nfunc run(this, activator, parameter, io, tree):\n" + body
		var err := script.reload()
		if err != OK:
			push_warning("[GT] logic_script %s failed to compile its source (error %d)" % [name, err])
			return
		_instance = script.new()
	elif expression.strip_edges() != "":
		_expr = Expression.new()
		var err := _expr.parse(expression, ["this", "activator", "parameter", "io", "tree"])
		if err != OK:
			push_warning("[GT] logic_script %s failed to parse its expression: %s" % [name, _expr.get_error_text()])
			_expr = null

func run(activator: Node = null) -> void:
	_execute(activator, null)

func run_with(parameter: Variant = null) -> void:
	_execute(null, parameter)

func _execute(activator: Node, parameter: Variant) -> void:
	var result: Variant = null
	if _instance and _instance.has_method("run"):
		result = _instance.call("run", self, activator, parameter, GodotTrenchIO, get_tree())
	elif _expr:
		result = _expr.execute([self, activator, parameter, GodotTrenchIO, get_tree()], self)
		if _expr.has_execute_failed():
			push_warning("[GT] logic_script %s expression failed: %s" % [name, _expr.get_error_text()])
			result = null
	GodotTrenchIO.events().fired.emit(self, &"ran", "", &"run", str(parameter))
	ran.emit(result)
