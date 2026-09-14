class_name GodotTrenchBlend extends RefCounted
## Faces with a blend_material face property blend two textures by vertex color alpha (0 base, 1 blend), the same
## weights the editor's Blend tool paints. The parser gives such faces a composite texture name, and the material map
## builds a [ShaderMaterial] with gt_blend.gdshader for it.

const SEPARATOR := "|blend|"
const SHADER := preload("res://addons/func_godot/src/godottrench/runtime/gt_blend.gdshader")

static var _nearest_shader: Shader

static func key(base: String, blend: String) -> String:
	return base + SEPARATOR + blend

static func is_blend(texture_name: String) -> bool:
	return texture_name.contains(SEPARATOR)

static func parts(texture_name: String) -> PackedStringArray:
	return texture_name.split(SEPARATOR, true, 1)

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
	var size := base.get_size() if base else Vector2.ONE * settings.inverse_scale_factor
	return [material, size]
