@tool
class_name GodotTrenchEnvironment extends RefCounted
## Builds a WorldEnvironment with a procedural sky, fog and a sun light from worldspawn keys, matching the
## GodotTrench editor's lit preview: sun_angles ("pitch yaw"), sun_color, sun_energy, ambient_color,
## sky_top_color, sky_horizon_color, sky_ground_color, fog_color and fog_density (per meter). For night maps
## ambient_energy and sky_energy scale the ambient light and the sky, and glow_intensity above 0 turns on glow so
## emissive materials and lamps bloom, while ssr 1 turns on screen space reflections for wet streets and glossy
## floors. Colors are "r g b" in 0..255 or 0..1. Set the worldspawn key "environment" to 0 to skip it.

const KEYS := ["sun_angles", "sky_top_color", "sky_horizon_color", "sky_ground_color", "fog_color", "fog_density", "ambient_energy", "sky_energy", "glow_intensity", "ssr"]

static func parse_color(text: String, fallback: Color) -> Color:
	var parts := text.split_floats(" ", false)
	if parts.size() < 3:
		return fallback
	var scale := 255.0 if parts[0] > 1.0 or parts[1] > 1.0 or parts[2] > 1.0 else 1.0
	return Color(parts[0] / scale, parts[1] / scale, parts[2] / scale)

static func wants_environment(properties: Dictionary) -> bool:
	if str(properties.get("environment", "1")) == "0":
		return false
	for key in KEYS:
		if properties.has(key):
			return true
	return false

## Returns the created nodes (WorldEnvironment and DirectionalLight3D), empty when the worldspawn has no environment keys.
static func build(map_node: Node3D, properties: Dictionary) -> Array[Node]:
	var out: Array[Node] = []
	if not wants_environment(properties):
		return out
	var sky_material := ProceduralSkyMaterial.new()
	sky_material.sky_top_color = parse_color(str(properties.get("sky_top_color", "")), Color(0.32, 0.5, 0.78))
	sky_material.sky_horizon_color = parse_color(str(properties.get("sky_horizon_color", "")), Color(0.72, 0.8, 0.88))
	sky_material.ground_bottom_color = parse_color(str(properties.get("sky_ground_color", "")), Color(0.42, 0.44, 0.46))
	sky_material.ground_horizon_color = sky_material.sky_horizon_color
	sky_material.energy_multiplier = maxf(str(properties.get("sky_energy", "1")).to_float(), 0.0)
	var sky := Sky.new()
	sky.sky_material = sky_material

	var env := Environment.new()
	env.background_mode = Environment.BG_SKY
	env.sky = sky
	env.ambient_light_source = Environment.AMBIENT_SOURCE_SKY
	if properties.has("ambient_color"):
		env.ambient_light_color = parse_color(str(properties["ambient_color"]), Color(0.26, 0.28, 0.33))
		env.ambient_light_sky_contribution = 0.5
	env.ambient_light_energy = maxf(str(properties.get("ambient_energy", "1")).to_float(), 0.0)
	env.tonemap_mode = Environment.TONE_MAPPER_FILMIC
	var glow := str(properties.get("glow_intensity", "0")).to_float()
	if glow > 0.0:
		env.glow_enabled = true
		env.glow_intensity = glow
		env.glow_bloom = 0.05
		env.glow_hdr_threshold = 0.9
		env.glow_blend_mode = Environment.GLOW_BLEND_MODE_SOFTLIGHT
	if str(properties.get("ssr", "0")) == "1":
		env.ssr_enabled = true
		env.ssr_max_steps = 96
	var density := str(properties.get("fog_density", "0")).to_float()
	if density > 0.0:
		env.fog_enabled = true
		env.fog_density = density
		env.fog_sky_affect = 0.3
		env.fog_light_color = parse_color(str(properties.get("fog_color", "")), Color(0.7, 0.78, 0.86))

	var world_env := WorldEnvironment.new()
	world_env.name = "environment"
	world_env.environment = env
	out.append(world_env)

	var sun := DirectionalLight3D.new()
	sun.name = "sun"
	var angles := str(properties.get("sun_angles", "-40 -45")).split_floats(" ", false)
	if angles.size() >= 2:
		sun.rotation_degrees = Vector3(angles[0], angles[1], 0.0)
	sun.light_color = parse_color(str(properties.get("sun_color", "")), Color(1.0, 0.96, 0.88))
	sun.light_energy = str(properties.get("sun_energy", "1.1")).to_float()
	sun.shadow_enabled = true
	sun.directional_shadow_max_distance = 400.0
	out.append(sun)

	var scene_root := GodotTrenchBuild.scene_owner(map_node)
	for node in out:
		map_node.add_child(node)
		node.owner = scene_root
	return out
