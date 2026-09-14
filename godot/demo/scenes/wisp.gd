class_name DemoWisp extends Node3D
## Forest wisp spawned by trigger_spawn_area in the lighthouse showcase. Hovers, drifts toward the player and fades after a while.

@export var lifetime := 20.0
@export var speed := 1.2

var _age := 0.0
var _home := Vector3.ZERO
var _phase := randf() * TAU

func _ready() -> void:
	_home = global_position

func _process(delta: float) -> void:
	_age += delta
	var player := get_tree().get_first_node_in_group("player") as Node3D
	if player:
		_home = _home.move_toward(player.global_position, speed * delta)
	global_position = _home + Vector3(cos(_phase + _age) * 0.6, 1.2 + sin(_age * 2.3) * 0.25, sin(_phase + _age) * 0.6)
	if _age > lifetime:
		queue_free()
