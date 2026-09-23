extends Node3D
## Festoon lights strung across the courtyard. The bulbs are MeshInstance3D children and the glow comes from its
## OmniLight3D children. turn_on lights the bulbs one after another, the courtyard trigger of the map calls it.

signal switched(on: bool)

@export var start_on := false
## Seconds from the first bulb to the last.
@export var stagger := 0.6
@export var lit: Material
@export var unlit: Material

var _on := false
var _tween: Tween

func _ready() -> void:
	_on = start_on
	_apply_instant()

func is_on() -> bool:
	return _on

func turn_on() -> void:
	if _on:
		return
	_on = true
	if _tween:
		_tween.kill()
	_tween = create_tween()
	var bulbs := _bulbs()
	var step := stagger / maxf(bulbs.size(), 1)
	for bulb in bulbs:
		_tween.tween_callback(func(): bulb.material_override = lit)
		_tween.tween_interval(step)
	_tween.tween_callback(_set_lights.bind(true))
	switched.emit(true)

func turn_off() -> void:
	if not _on:
		return
	_on = false
	_apply_instant()
	switched.emit(false)

func toggle() -> void:
	if _on:
		turn_off()
	else:
		turn_on()

## Switches off right away without emitting, e.g. to replay the demo.
func reset() -> void:
	_on = false
	_apply_instant()

func _apply_instant() -> void:
	if _tween:
		_tween.kill()
		_tween = null
	for bulb in _bulbs():
		bulb.material_override = lit if _on else unlit
	_set_lights(_on)

func _set_lights(on: bool) -> void:
	for child in get_children():
		if child is Light3D:
			child.visible = on

func _bulbs() -> Array[MeshInstance3D]:
	var out: Array[MeshInstance3D] = []
	for child in get_children():
		if child is MeshInstance3D and String(child.name).begins_with("Bulb"):
			out.append(child)
	return out
