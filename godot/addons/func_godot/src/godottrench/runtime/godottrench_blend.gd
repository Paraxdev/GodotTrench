class_name GodotTrenchBlend extends RefCounted
## Faces with a blend_material face property blend two textures by vertex color alpha (0 base, 1 blend), the same
## weights the editor's Blend tool paints. The parser gives such faces a composite texture name, and the material map
## builds a [ShaderMaterial] with gt_blend.gdshader for it.

const SEPARATOR := "|blend|"
## Trails the texture names when the face asks for de-tiling or a different repeat, so faces that want
## different settings get their own material rather than sharing one.
const OPTIONS := "|opt|"
const SHADER := preload("res://addons/func_godot/src/godottrench/runtime/gt_blend.gdshader")

static var _nearest_shader: Shader

static func key(base: String, blend: String, detile := 0.0, uv_scale := 1.0, sharpen := 0.5) -> String:
	var name := base + SEPARATOR + blend
	if detile > 0.0 or not is_equal_approx(uv_scale, 1.0) or not is_equal_approx(sharpen, 0.5):
		name += OPTIONS + ("%.3f,%.3f,%.3f" % [detile, uv_scale, sharpen])
	return name

static func is_blend(texture_name: String) -> bool:
	return texture_name.contains(SEPARATOR)

## The two texture names, without any trailing options.
static func parts(texture_name: String) -> PackedStringArray:
	return texture_name.split(OPTIONS, true, 1)[0].split(SEPARATOR, true, 1)

## The face's de-tile strength, repeat multiplier and de-tile crispness for the painted texture.
static func options(texture_name: String) -> Array:
	var tail := texture_name.split(OPTIONS, true, 1)
	if tail.size() < 2:
		return [0.0, 1.0, 0.5]
	var values := tail[1].split(",")
	var detile := float(values[0]) if values.size() > 0 else 0.0
	var uv_scale := float(values[1]) if values.size() > 1 else 1.0
	var sharpen := float(values[2]) if values.size() > 2 else 0.5
	return [clampf(detile, 0.0, 1.0), maxf(uv_scale, 0.001), clampf(sharpen, 0.0, 1.0)]

static func _material_file(texture_name: String, settings: FuncGodotMapSettings) -> Material:
	var dir := settings.base_material_dir if settings.base_material_dir != "" else settings.base_texture_dir
	var path := dir.path_join(texture_name + "." + settings.material_file_extension)
	return load(path) if ResourceLoader.exists(path) else null

static func albedo(texture_name: String, settings: FuncGodotMapSettings, wads: Array[QuakeWadFile]) -> Texture2D:
	var material := _material_file(texture_name, settings)
	if material is BaseMaterial3D and material.albedo_texture:
		return material.albedo_texture
	return FuncGodotUtil.load_texture(texture_name, wads, settings)

static func _shader(pixelated: bool) -> Shader:
	if not pixelated:
		return SHADER
	if not _nearest_shader:
		_nearest_shader = Shader.new()
		_nearest_shader.code = SHADER.code.replace("filter_linear_mipmap", "filter_nearest_mipmap")
	return _nearest_shader

## Material and base texture size for a composite blend texture name.
static func build(texture_name: String, settings: FuncGodotMapSettings, wads: Array[QuakeWadFile]) -> Array:
	var names := parts(texture_name)
	var base := albedo(names[0], settings, wads)
	var blend := albedo(names[1] if names.size() > 1 else names[0], settings, wads)
	var base_material := _material_file(names[0], settings)
	var pixelated: bool = base_material is BaseMaterial3D and base_material.texture_filter in [BaseMaterial3D.TEXTURE_FILTER_NEAREST, BaseMaterial3D.TEXTURE_FILTER_NEAREST_WITH_MIPMAPS]
	var material := ShaderMaterial.new()
	material.shader = _shader(pixelated)
	material.set_shader_parameter("texture_a", base)
	material.set_shader_parameter("texture_b", blend)
	# Only the painted texture is de-tiled and rescaled: it is the one laid over the surface as a path, the
	# base keeps the look and alignment the face was built with.
	var opts := options(texture_name)
	var size := texture_size(names[0], base, settings)
	# Face UVs follow the base's world size, the painted texture is rescaled so it keeps its own.
	var blend_size := texture_size(names[1] if names.size() > 1 else names[0], blend, settings)
	material.set_shader_parameter("detile_b", opts[0])
	material.set_shader_parameter("uv_scale_b", Vector2.ONE * opts[1] * size / blend_size)
	material.set_shader_parameter("detile_sharpen_b", opts[2])
	_apply_emission(material, "a", names[0], settings)
	_apply_emission(material, "b", names[1] if names.size() > 1 else names[0], settings)
	return [material, size]

## Carries a side's emission into the blend: from its material file, else from an emission map named by the map
## settings' emission pattern, the one FuncGodot's generated materials pick up.
static func _apply_emission(material: ShaderMaterial, side: String, texture_name: String, settings: FuncGodotMapSettings) -> void:
	var source := _material_file(texture_name, settings)
	if source is BaseMaterial3D:
		if not source.emission_enabled:
			return
		material.set_shader_parameter("emission_" + side, source.emission)
		material.set_shader_parameter("emission_energy_" + side, source.emission_energy_multiplier)
		material.set_shader_parameter("emission_multiply_" + side, source.emission_operator == BaseMaterial3D.EMISSION_OP_MULTIPLY)
		if source.emission_texture:
			material.set_shader_parameter("emission_texture_" + side, source.emission_texture)
		return
	var pattern := settings.emission_map_pattern
	if source or pattern.count("%s") != 1:
		return
	for ext in settings.texture_file_extensions:
		var path := (pattern % settings.base_texture_dir.path_join(texture_name)) + "." + ext
		if ResourceLoader.exists(path):
			material.set_shader_parameter("emission_energy_" + side, 1.0)
			material.set_shader_parameter("emission_texture_" + side, load(path))
			return

## The world size one repeat of a texture covers: its material's texture_size metadata, else its pixel size.
static func texture_size(texture_name: String, albedo_texture: Texture2D, settings: FuncGodotMapSettings) -> Vector2:
	var world := FuncGodotUtil.material_texture_size(_material_file(texture_name, settings))
	if world != Vector2.ZERO:
		return world
	return albedo_texture.get_size() if albedo_texture else Vector2.ONE * settings.inverse_scale_factor
