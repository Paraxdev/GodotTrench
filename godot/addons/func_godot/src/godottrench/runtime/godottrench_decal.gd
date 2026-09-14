@tool
class_name GodotTrenchDecal extends Decal
## Decal entity for GodotTrench maps. Properties:
## [code]texture[/code] res:// path of the albedo texture, [code]size[/code] projector size in map units (x, depth, z),
## [code]modulate[/code] color, [code]normal_texture[/code] optional normal map.

func _func_godot_apply_properties(props: Dictionary) -> void:
	var scale_factor := 1.0 / float(ProjectSettings.get_setting("func_godot/default_inverse_scale_factor", 32.0))
	var tex = props.get("texture", "")
	if tex is String and tex != "" and ResourceLoader.exists(tex):
		texture_albedo = load(tex)
	var normal_tex = props.get("normal_texture", "")
	if normal_tex is String and normal_tex != "" and ResourceLoader.exists(normal_tex):
		texture_normal = load(normal_tex)
	var s = props.get("size", Vector3(64, 32, 64))
	if s is String:
		var parts: PackedFloat64Array = s.split_floats(" ")
		s = Vector3(parts[0], parts[1], parts[2]) if parts.size() >= 3 else Vector3(64, 32, 64)
	if s is Vector3:
		size = s * scale_factor
	var tint = props.get("modulate", null)
	if tint is Color:
		modulate = tint
