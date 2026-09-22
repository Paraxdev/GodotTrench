@tool
class_name GTParticles extends GPUParticles3D
## env_particles: a particle effect toggled through I/O, for smoke, fire, sparks or dust.
## Inputs: start, stop, toggle, burst. Without a process material it builds one from effect: rise sends puffs
## upward, rain drops streaks from a box of area meters (width, height, depth) above the node.

@export var start_emitting := false
@export_enum("rise", "rain") var effect := "rise"
@export var area := Vector3(10, 1, 10)

var _burst_id := 0
## While a burst runs on a continuous emitter: whether it was emitting before, so it can resume afterwards.
var _burst_resume: Variant = null

func _func_godot_apply_properties(props: Dictionary) -> void:
	amount = int(props.get("amount", amount))
	lifetime = float(props.get("lifetime", lifetime))
	one_shot = GodotTrenchIO.to_bool(props.get("one_shot", one_shot))
	start_emitting = GodotTrenchIO.to_bool(props.get("start_emitting", start_emitting))
	effect = str(props.get("effect", effect))
	var size := str(props.get("area", "")).split_floats(" ", false)
	if size.size() >= 3:
		area = Vector3(size[0], size[1], size[2])

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	if effect == "rain":
		_setup_rain()
	if not process_material:
		process_material = _default_material()
	if not draw_pass_1:
		draw_pass_1 = QuadMesh.new()
	emitting = start_emitting

func _setup_rain() -> void:
	if not process_material:
		var mat := ParticleProcessMaterial.new()
		mat.emission_shape = ParticleProcessMaterial.EMISSION_SHAPE_BOX
		mat.emission_box_extents = area * 0.5
		mat.direction = Vector3(0.08, -1, 0)
		mat.spread = 2.0
		mat.initial_velocity_min = 16.0
		mat.initial_velocity_max = 22.0
		mat.gravity = Vector3(0, -9.8, 0)
		process_material = mat
	if not draw_pass_1:
		var streak := QuadMesh.new()
		streak.size = Vector2(0.02, 0.6)
		var look := StandardMaterial3D.new()
		look.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		look.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		look.billboard_mode = BaseMaterial3D.BILLBOARD_FIXED_Y
		look.albedo_color = Color(0.72, 0.8, 1.0, 0.3)
		streak.material = look
		draw_pass_1 = streak
	preprocess = lifetime
	var fall := lifetime * 25.0
	visibility_aabb = AABB(Vector3(-area.x * 0.5, -area.y * 0.5 - fall, -area.z * 0.5), Vector3(area.x, area.y + fall, area.z))

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
	_end_burst()
	emitting = true

func stop() -> void:
	_end_burst()
	emitting = false

func toggle() -> void:
	var was := emitting
	_end_burst()
	emitting = not was

## Emits exactly one shot. A continuous emitter goes back to how it was afterwards, emitting again if it was on.
func burst() -> void:
	if _burst_resume == null and not one_shot:
		_burst_resume = emitting
	_burst_id += 1
	var id := _burst_id
	one_shot = true
	restart()
	if _burst_resume == null or not is_inside_tree():
		return
	await finished
	if id == _burst_id:
		_end_burst()

func _end_burst() -> void:
	if _burst_resume == null:
		return
	var resume: bool = _burst_resume
	_burst_resume = null
	_burst_id += 1
	one_shot = false
	emitting = resume
