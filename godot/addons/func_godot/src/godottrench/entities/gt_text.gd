@tool
class_name GTText extends Node3D
## game_text: shows a line of text in the world (a Label3D) and/or on the HUD (a CanvasLayer label).
## Inputs: show, hide, set_text(text), flash(seconds). Outputs: shown, hidden.

signal shown
signal hidden

@export var text := ""
## world, hud or both.
@export var place := "world"
@export var start_visible := false
## Seconds to stay up before hiding on its own, 0 stays until hide.
@export var duration := 0.0
@export var text_color := Color.WHITE
## World label size in meters per pixel.
@export var world_size := 0.01
## Faces the camera and draws over walls. Off keeps the node's rotation and depth, for signs.
@export var billboard := true

var _label3d: Label3D
var _canvas: CanvasLayer
var _hud: Label
## Bumped by every show and hide, so a hide timer from an earlier showing does nothing.
var _cycle := 0

func _func_godot_apply_properties(props: Dictionary) -> void:
	text = str(props.get("text", text))
	place = str(props.get("place", place))
	start_visible = GodotTrenchIO.to_bool(props.get("start_visible", start_visible))
	duration = float(props.get("duration", duration))
	if props.has("text_color"):
		text_color = GodotTrenchIO.to_color(props.get("text_color"))
	world_size = float(props.get("world_size", world_size))
	billboard = GodotTrenchIO.to_bool(props.get("billboard", billboard))

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	_build()
	_set_shown(start_visible)

func _build() -> void:
	if place == "world" or place == "both":
		_label3d = Label3D.new()
		_label3d.text = text
		_label3d.billboard = BaseMaterial3D.BILLBOARD_ENABLED if billboard else BaseMaterial3D.BILLBOARD_DISABLED
		_label3d.pixel_size = world_size
		_label3d.modulate = text_color
		_label3d.no_depth_test = billboard
		add_child(_label3d)
	if place == "hud" or place == "both":
		_canvas = CanvasLayer.new()
		_hud = Label.new()
		_hud.text = text
		_hud.add_theme_color_override("font_color", text_color)
		_hud.add_theme_font_size_override("font_size", 28)
		_hud.add_theme_color_override("font_outline_color", Color(0, 0, 0, 0.7))
		_hud.add_theme_constant_override("outline_size", 6)
		_hud.set_anchors_and_offsets_preset(Control.PRESET_BOTTOM_WIDE)
		_hud.offset_bottom = -48.0
		_hud.offset_top = -96.0
		_hud.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
		_hud.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
		_canvas.add_child(_hud)
		add_child(_canvas)

func set_text(new_text: Variant = "") -> void:
	text = str(new_text)
	if _label3d:
		_label3d.text = text
	if _hud:
		_hud.text = text

func show() -> void:
	_set_shown(true)

func hide() -> void:
	_set_shown(false)

func flash(seconds: Variant = null) -> void:
	var secs := duration if duration > 0.0 else 2.0
	if seconds != null and str(seconds).is_valid_float():
		secs = float(seconds)
	_set_shown(true)
	var cycle := _cycle
	await get_tree().create_timer(maxf(secs, 0.01)).timeout
	if is_inside_tree() and cycle == _cycle:
		_set_shown(false)

func _set_shown(value: bool) -> void:
	_cycle += 1
	if _label3d:
		_label3d.visible = value
	if _canvas:
		_canvas.visible = value
	if value:
		shown.emit()
		if duration > 0.0:
			_arm_hide(duration)
	else:
		hidden.emit()

func _arm_hide(secs: float) -> void:
	var cycle := _cycle
	var timer := get_tree().create_timer(secs)
	timer.timeout.connect(func() -> void:
		if is_inside_tree() and cycle == _cycle:
			_set_shown(false))
