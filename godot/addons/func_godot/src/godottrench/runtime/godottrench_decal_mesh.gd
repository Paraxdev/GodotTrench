class_name GodotTrenchDecalMesh extends RefCounted
## Materials for mesh nodes flagged as decals. Kept free of other GodotTrench scripts because the FuncGodot material
## map uses it, and depending on the parser from there makes a cyclic reference.

## Trails the texture name of a decal mesh's faces, so the material map gives them a cut out, double sided variant.
const SUFFIX := "|decal|"

static func is_decal(texture_name: String) -> bool:
	return texture_name.ends_with(SUFFIX)

static func base(texture_name: String) -> String:
	return texture_name.trim_suffix(SUFFIX)

## The editor draws decal meshes with the texture's alpha cut out at 0.5 and both sides visible. A shader material
## is used as it is, since its alpha handling is its own.
static func material(base: Material) -> Material:
	if not base is BaseMaterial3D:
		return base
	var mat := base.duplicate() as BaseMaterial3D
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA_SCISSOR
	mat.alpha_scissor_threshold = 0.5
	mat.cull_mode = BaseMaterial3D.CULL_DISABLED
	return mat
