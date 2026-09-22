@tool
class_name GTParticles extends GPUParticles3D
## env_particles: a particle effect toggled through I/O, for smoke, fire, sparks or dust.
## Inputs: start, stop, toggle, burst. It builds a simple upward process material when none is set.

@export var start_emitting := false

func _func_godot_apply_properties(props: Dictionary) -> void:
	amount = int(props.get("amount", amount))
	lifetime = float(props.get("lifetime", lifetime))
	one_shot = GodotTrenchIO.to_bool(props.get("one_shot", one_shot))
	start_emitting = GodotTrenchIO.to_bool(props.get("start_emitting", start_emitting))

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	if not process_material:
		process_material = _default_material()
	if not draw_pass_1:
		draw_pass_1 = QuadMesh.new()
	emitting = start_emitting

func _default_material() -> ParticleProcessMaterial:
	var mat := ParticleProcessMaterial.new()
	mat.direction = Vector3(0, 1, 0)
	mat.spread = 25.0
	mat.initial_velocity_min = 1.0
	mat.initial_velocity_max = 3.0
	mat.gravity = Vector3(0, -2, 0)
	mat.scale_min = 0.05
	mat.scale_max = 0.15
	return mat

func start() -> void:
	emitting = true

func stop() -> void:
	emitting = false

func toggle() -> void:
	emitting = not emitting

func burst() -> void:
	restart()
	emitting = true
