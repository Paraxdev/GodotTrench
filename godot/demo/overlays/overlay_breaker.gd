extends StaticBody3D
## A breaker box that lives in the night district overlay, not in the map. Using it (E) flips it and fires
## flipped_on or flipped_off, which its GodotTrenchOutput children send to the courtyard lamps of the map.

signal flipped_on(activator: Node)
signal flipped_off(activator: Node)

@export var on := false
@export var indicator: MeshInstance3D
@export var label: Label3D

func _ready() -> void:
	_show()
	var overlay := _overlay()
	if overlay:
		overlay.map_rebuilt.connect(_on_map_rebuilt)

func _overlay() -> GodotTrenchOverlay:
	var n := get_parent()
	while n and not n is GodotTrenchOverlay:
		n = n.get_parent()
	return n as GodotTrenchOverlay

# A rebuilt map starts with its lamps off, so a breaker left on switches them on again.
func _on_map_rebuilt(_map: FuncGodotMap) -> void:
	if on:
		flipped_on.emit(null)

func use(activator: Node = null) -> void:
	on = not on
	_show()
	if on:
		flipped_on.emit(activator)
	else:
		flipped_off.emit(activator)

# On and off differ in brightness and text as well as hue, so the state reads without color vision.
func _show() -> void:
	if indicator:
		var material := StandardMaterial3D.new()
		material.albedo_color = Color(0.1, 0.1, 0.1)
		material.emission_enabled = true
		material.emission = Color(0.55, 0.85, 1.0) if on else Color(1.0, 0.55, 0.1)
		material.emission_energy_multiplier = 4.0 if on else 0.6
		indicator.material_override = material
	if label:
		label.text = "COURT LIGHTS\n%s" % ("ON" if on else "OFF")
