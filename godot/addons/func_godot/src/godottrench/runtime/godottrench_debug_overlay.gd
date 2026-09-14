class_name GodotTrenchDebugOverlay extends CanvasLayer
## In-game level debugging: F3 (by default) shows a log of every I/O event and draws trigger volumes with their
## targetnames. Add it as an autoload or drop it into a scene.

@export var toggle_key := KEY_F3
@export var max_lines := 14
@export var volume_color := Color(1.0, 0.55, 0.1, 0.18)

var lines: PackedStringArray = []
var _label: Label
var _volumes: Array[Node] = []

func _ready() -> void:
	layer = 100
	_label = Label.new()
	_label.position = Vector2(12, 12)
	_label.add_theme_color_override(&"font_color", Color(1.0, 0.8, 0.5))
	_label.add_theme_color_override(&"font_outline_color", Color.BLACK)
	_label.add_theme_constant_override(&"outline_size", 4)
	add_child(_label)
	visible = false
	GodotTrenchIO.events().fired.connect(_on_event)

func _exit_tree() -> void:
	if GodotTrenchIO.events().fired.is_connected(_on_event):
		GodotTrenchIO.events().fired.disconnect(_on_event)
	_clear_volumes()

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventKey and event.pressed and not event.echo and event.keycode == toggle_key:
		set_overlay(not visible)

func set_overlay(on: bool) -> void:
	visible = on
	_clear_volumes()
	if on:
		_show_volumes()

func _on_event(source: Node, output: StringName, target: String, input: StringName, parameter: String) -> void:
	var src := str(source.get_meta(GodotTrenchIO.TARGETNAME_META, source.name)) if is_instance_valid(source) else "?"
	var line := "%6.2f  %s.%s > %s.%s%s" % [Time.get_ticks_msec() / 1000.0, src, output, target, input, "(%s)" % parameter if parameter != "" else ""]
	lines.append(line)
	if lines.size() > max_lines:
		lines.remove_at(0)
	if _label:
		_label.text = "I/O events (F3)\n" + "\n".join(lines)

func _clear_volumes() -> void:
	for v in _volumes:
		if is_instance_valid(v):
			v.queue_free()
	_volumes.clear()

func _show_volumes() -> void:
	var tree := get_tree()
	if not tree or not tree.current_scene:
		return
	var material := StandardMaterial3D.new()
	material.albedo_color = volume_color
	material.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	material.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	material.cull_mode = BaseMaterial3D.CULL_DISABLED
	material.no_depth_test = true
	var stack: Array[Node] = [tree.current_scene]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		stack.append_array(n.get_children())
		if not n is Area3D:
			continue
		for child in n.get_children():
			if child is CollisionShape3D and child.shape:
				var mesh := MeshInstance3D.new()
				mesh.mesh = child.shape.get_debug_mesh()
				mesh.material_override = material
				child.add_child(mesh)
				_volumes.append(mesh)
		var label := Label3D.new()
		label.text = str(n.get_meta(GodotTrenchIO.TARGETNAME_META, n.name))
		label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
		label.no_depth_test = true
		label.pixel_size = 0.006
		n.add_child(label)
		_volumes.append(label)
