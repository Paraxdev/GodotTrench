class_name DemoPlayer extends CharacterBody3D
## Minimal first person player for trying maps: WASD, mouse look, Space jumps, Shift runs, E uses doors and buttons.
## It joins the "player" group, which the GodotTrench triggers filter on by default.

@export var speed := 5.0
@export var run_multiplier := 1.8
@export var jump_velocity := 4.8
@export var look_sensitivity := 0.0025
@export var use_distance := 2.5
@export var max_health := 100.0

## Emitted whenever health changes, so a HUD can follow it.
signal health_changed(health: float)
## Emitted once when health reaches zero.
signal died

var health := 100.0

@onready var head: Node3D = $Head
@onready var camera: Camera3D = $Head/Camera

var _health_label: Label

func _ready() -> void:
	add_to_group(&"player")
	health = max_health
	_build_hud()
	Input.mouse_mode = Input.MOUSE_MODE_CAPTURED

func _build_hud() -> void:
	var canvas := CanvasLayer.new()
	_health_label = Label.new()
	_health_label.set_anchors_and_offsets_preset(Control.PRESET_TOP_LEFT)
	_health_label.offset_left = 16.0
	_health_label.offset_top = 12.0
	canvas.add_child(_health_label)
	add_child(canvas)
	_refresh_hud()

func _refresh_hud() -> void:
	if _health_label:
		_health_label.text = "HP %d" % int(round(health))

## I/O damage entities call this: trigger_hurt, env_explosion and prop_physics all send (amount, source).
func take_damage(amount: float, _source: Node = null) -> void:
	if health <= 0.0:
		return
	health = maxf(health - amount, 0.0)
	_refresh_hud()
	health_changed.emit(health)
	if health <= 0.0:
		died.emit()

func heal(amount: float) -> void:
	health = minf(health + amount, max_health)
	_refresh_hud()
	health_changed.emit(health)

func _unhandled_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion and Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
		rotate_y(-event.relative.x * look_sensitivity)
		head.rotation.x = clampf(head.rotation.x - event.relative.y * look_sensitivity, -1.5, 1.5)
	elif event is InputEventKey and event.pressed and not event.echo:
		match event.keycode:
			KEY_ESCAPE:
				Input.mouse_mode = Input.MOUSE_MODE_VISIBLE
			KEY_E:
				use_target()
	elif event is InputEventMouseButton and event.pressed:
		Input.mouse_mode = Input.MOUSE_MODE_CAPTURED

func _physics_process(delta: float) -> void:
	if not is_on_floor():
		velocity += get_gravity() * delta
	elif Input.is_key_pressed(KEY_SPACE):
		velocity.y = jump_velocity
	var input := Vector2(
		float(Input.is_key_pressed(KEY_D)) - float(Input.is_key_pressed(KEY_A)),
		float(Input.is_key_pressed(KEY_S)) - float(Input.is_key_pressed(KEY_W)))
	var direction := (transform.basis * Vector3(input.x, 0, input.y)).normalized()
	var target_speed := speed * (run_multiplier if Input.is_key_pressed(KEY_SHIFT) else 1.0)
	velocity.x = direction.x * target_speed
	velocity.z = direction.z * target_speed
	move_and_slide()

## Calls use(activator) on the first node up the tree from whatever the camera looks at.
func use_target() -> void:
	var from := camera.global_position
	var query := PhysicsRayQueryParameters3D.create(from, from - camera.global_basis.z * use_distance)
	query.exclude = [get_rid()]
	query.collide_with_areas = false
	var hit := get_world_3d().direct_space_state.intersect_ray(query)
	var node: Node = hit.get("collider")
	while node:
		if node.has_method(&"use"):
			node.call(&"use", self)
			return
		node = node.get_parent()
