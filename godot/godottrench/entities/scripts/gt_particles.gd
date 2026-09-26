@tool
class_name GTParticles extends GPUParticles3D
## env_particles: a particle effect toggled through I/O, for smoke, fire, sparks or dust.
## Inputs: start, stop, toggle, burst. Without a process material or draw pass of its own it builds them from
## effect: rise sends soft glowing puffs upward, fire a flickering flame, smoke grey puffs that grow as they climb,
## sparks bright streaks that fall under gravity, rain streaks from a box of area meters (width, height, depth) above
## the node. Particles are soft round sprites that fade in and out over their lifetime. color tints the effect, white
## keeps its own colors, size is a particle's size in meters (a spark's or raindrop's length) and 0 keeps the effect's,
## blend picks additive glow or mixed smoke instead of the effect's own.

const EFFECTS: PackedStringArray = ["rise", "fire", "smoke", "sparks", "rain"]

@export var start_emitting := false
@export_enum("rise", "fire", "smoke", "sparks", "rain") var effect := "rise"
@export var area := Vector3(10, 1, 10)
@export var color := Color.WHITE
## Particle size in meters, 0 uses the effect's own.
@export var size := 0.0
@export_enum("auto", "add", "mix") var blend := "auto"

var _burst_id := 0
## While a burst runs on a continuous emitter: whether it was emitting before, so it can resume afterwards.
var _burst_resume: Variant = null

func _func_godot_apply_properties(props: Dictionary) -> void:
	amount = int(props.get("amount", amount))
	lifetime = float(props.get("lifetime", lifetime))
	one_shot = GodotTrenchIO.to_bool(props.get("one_shot", one_shot))
	start_emitting = GodotTrenchIO.to_bool(props.get("start_emitting", start_emitting))
	effect = str(props.get("effect", effect))
	var box := str(props.get("area", "")).split_floats(" ", false)
	if box.size() >= 3:
		area = Vector3(box[0], box[1], box[2])
	if props.has("color"):
		color = GodotTrenchIO.to_color(props.get("color"))
	size = float(props.get("size", size))
	blend = str(props.get("blend", blend))

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	if not effect in EFFECTS:
		push_warning("env_particles %s: unknown effect \"%s\", using rise. Known effects: %s" % [name, effect, ", ".join(EFFECTS)])
		effect = "rise"
	if effect == "rain":
		_setup_rain()
	else:
		if not process_material:
			process_material = _process_material()
		if not draw_pass_1:
			draw_pass_1 = _draw_pass()
		visibility_aabb = AABB(Vector3(-4, -2, -4), Vector3(8, 4.0 + lifetime * 3.0, 8))
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
		streak.size = Vector2(0.02, size if size > 0.0 else 0.6)
		var look := StandardMaterial3D.new()
		look.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		look.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		look.billboard_mode = BaseMaterial3D.BILLBOARD_FIXED_Y
		look.albedo_color = Color(0.72, 0.8, 1.0, 0.3) * color
		look.blend_mode = BaseMaterial3D.BLEND_MODE_ADD if blend == "add" else BaseMaterial3D.BLEND_MODE_MIX
		streak.material = look
		draw_pass_1 = streak
	preprocess = lifetime
	var fall := lifetime * 25.0
	visibility_aabb = AABB(Vector3(-area.x * 0.5, -area.y * 0.5 - fall, -area.z * 0.5), Vector3(area.x, area.y + fall, area.z))

func _process_material() -> ParticleProcessMaterial:
	var mat := ParticleProcessMaterial.new()
	mat.direction = Vector3(0, 1, 0)
	mat.angle_min = -180.0
	mat.angle_max = 180.0
	match effect:
		"fire":
			mat.emission_shape = ParticleProcessMaterial.EMISSION_SHAPE_SPHERE
			mat.emission_sphere_radius = 0.12
			mat.spread = 10.0
			mat.initial_velocity_min = 0.4
			mat.initial_velocity_max = 1.0
			mat.gravity = Vector3(0, 1.5, 0)
			mat.scale_curve = _curve(1.0, 0.25)
			mat.color_ramp = _ramp([Color(1, 0.9, 0.6, 0), Color(1, 0.75, 0.3, 1), Color(1, 0.35, 0.05, 0.8), Color(0.5, 0.05, 0, 0)], [0.0, 0.1, 0.5, 1.0])
		"smoke":
			mat.emission_shape = ParticleProcessMaterial.EMISSION_SHAPE_SPHERE
			mat.emission_sphere_radius = 0.2
			mat.spread = 15.0
			mat.initial_velocity_min = 0.5
			mat.initial_velocity_max = 1.0
			mat.gravity = Vector3(0, 0.3, 0)
			mat.damping_min = 0.1
			mat.damping_max = 0.2
			mat.angular_velocity_min = -20.0
			mat.angular_velocity_max = 20.0
			mat.scale_curve = _curve(0.5, 1.5)
			mat.color_ramp = _ramp([Color(0.35, 0.35, 0.35, 0), Color(0.3, 0.3, 0.3, 0.65), Color(0.25, 0.25, 0.25, 0)], [0.0, 0.2, 1.0])
		"sparks":
			mat.angle_min = 0.0
			mat.angle_max = 0.0
			mat.spread = 50.0
			mat.initial_velocity_min = 2.5
			mat.initial_velocity_max = 5.0
			mat.gravity = Vector3(0, -9.8, 0)
			mat.scale_min = 0.5
			mat.color_ramp = _ramp([Color(1, 0.95, 0.7, 1), Color(1, 0.6, 0.15, 1), Color(1, 0.3, 0, 0)], [0.0, 0.5, 1.0])
		_:
			mat.spread = 25.0
			mat.initial_velocity_min = 1.0
			mat.initial_velocity_max = 3.0
			mat.gravity = Vector3(0, -2, 0)
			mat.scale_min = 0.6
			mat.color_ramp = _ramp([Color(1, 1, 1, 0), Color(1, 1, 1, 0.8), Color(1, 1, 1, 0)], [0.0, 0.15, 1.0])
	return mat

func _draw_pass() -> QuadMesh:
	var quad := QuadMesh.new()
	var look := StandardMaterial3D.new()
	look.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	look.vertex_color_use_as_albedo = true
	look.albedo_color = color
	var additive := effect != "smoke"
	if blend == "add" or blend == "mix":
		additive = blend == "add"
	look.blend_mode = BaseMaterial3D.BLEND_MODE_ADD if additive else BaseMaterial3D.BLEND_MODE_MIX
	look.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED if additive else BaseMaterial3D.SHADING_MODE_PER_VERTEX
	if effect == "sparks":
		var length := size if size > 0.0 else 0.15
		quad.size = Vector2(length * 0.15, length)
		look.albedo_texture = _soft_texture(true)
		transform_align = GPUParticles3D.TRANSFORM_ALIGN_Z_BILLBOARD_Y_TO_VELOCITY
	else:
		var side: float = size if size > 0.0 else {"fire": 0.4, "smoke": 0.8}.get(effect, 0.25)
		quad.size = Vector2(side, side)
		look.albedo_texture = _soft_texture(false)
		look.billboard_mode = BaseMaterial3D.BILLBOARD_PARTICLES
		look.proximity_fade_enabled = true
		look.proximity_fade_distance = side * 0.5
	quad.material = look
	return quad

## A white dot whose alpha falls off from the center, or a streak for sparks.
static func _soft_texture(streak: bool) -> GradientTexture2D:
	var tex := GradientTexture2D.new()
	tex.width = 16 if streak else 64
	tex.height = 64
	tex.fill = GradientTexture2D.FILL_RADIAL
	tex.fill_from = Vector2(0.5, 0.5)
	tex.fill_to = Vector2(1.0, 0.5) if streak else Vector2(0.5, 0.0)
	tex.gradient = Gradient.new()
	tex.gradient.set_color(0, Color(1, 1, 1, 1))
	tex.gradient.set_color(1, Color(1, 1, 1, 0))
	tex.gradient.add_point(0.4, Color(1, 1, 1, 0.6))
	return tex

static func _ramp(colors: Array[Color], offsets: Array[float]) -> GradientTexture1D:
	var gradient := Gradient.new()
	gradient.colors = PackedColorArray(colors)
	gradient.offsets = PackedFloat32Array(offsets)
	var tex := GradientTexture1D.new()
	tex.gradient = gradient
	return tex

static func _curve(from: float, to: float) -> CurveTexture:
	var curve := Curve.new()
	curve.max_value = maxf(from, to)
	curve.add_point(Vector2(0, from))
	curve.add_point(Vector2(1, to))
	var tex := CurveTexture.new()
	tex.curve = curve
	return tex

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
