@tool
class_name GTDebug extends Node3D
## logic_debug: prints inputs with their parameter and activator, shows a floating label in debug builds, and with
## trace_all logs every I/O event of the map. Inputs: write(parameter), toggle (show and hide are built in). Output: printed(text).

signal printed(text: String)

@export var message := "{name} received {parameter} from {activator}"
@export var show_label := true
@export var trace_all := false

## Recent lines, newest last.
var lines: PackedStringArray = []
var _label: Label3D

func _func_godot_apply_properties(props: Dictionary) -> void:
	message = str(props.get("message", message))
	show_label = GodotTrenchIO.to_bool(props.get("show_label", show_label))
	trace_all = GodotTrenchIO.to_bool(props.get("trace_all", trace_all))

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	if show_label and OS.is_debug_build():
		_label = Label3D.new()
		_label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
		_label.pixel_size = 0.004
		_label.no_depth_test = true
		_label.text = str(get_meta(GodotTrenchIO.TARGETNAME_META, name))
		add_child(_label)
	if trace_all:
		GodotTrenchIO.events().fired.connect(_on_event)

func _exit_tree() -> void:
	if GodotTrenchIO.events().fired.is_connected(_on_event):
		GodotTrenchIO.events().fired.disconnect(_on_event)

func _on_event(source: Node, output: StringName, target: String, input: StringName, parameter: String) -> void:
	var src := str(source.get_meta(GodotTrenchIO.TARGETNAME_META, source.name)) if is_instance_valid(source) else "?"
	_write("[I/O] %s.%s > %s.%s(%s)" % [src, output, target, input, parameter])

func write(parameter: Variant = null, activator: Node = null) -> void:
	var text := message.format({
		"name": str(get_meta(GodotTrenchIO.TARGETNAME_META, name)),
		"parameter": str(parameter),
		"activator": str(activator.name) if activator else "nobody",
		"time": "%.2f" % (Time.get_ticks_msec() / 1000.0),
	})
	_write(text)
	printed.emit(text)

func _write(text: String) -> void:
	print_rich("[color=violet]%s[/color]" % text)
	lines.append(text)
	if lines.size() > 32:
		lines.remove_at(0)
	if _label:
		_label.text = "\n".join(lines.slice(maxi(0, lines.size() - 6)))

func toggle() -> void:
	if _label:
		_label.visible = not _label.visible
