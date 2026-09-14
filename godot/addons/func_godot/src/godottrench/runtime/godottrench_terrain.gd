@tool
class_name GodotTrenchTerrain extends StaticBody3D
## Heightmap terrain built from a GodotTrench terrain node: chunked meshes with a four layer blend shader
## and a [HeightMapShape3D] for collision.

const SHADER := preload("res://addons/func_godot/src/godottrench/runtime/gt_terrain.gdshader")

## Vertex count along x and z.
@export var resolution := Vector2i(2, 2)
## Meters between vertices.
@export var cell_size := 1.0
## Heights in meters relative to this node, row major (z then x).
@export var heights := PackedFloat32Array()

## Height of the terrain surface in local space at a local x/z position, or NAN outside.
func height_at(local_x: float, local_z: float) -> float:
	var fx := local_x / cell_size
	var fz := local_z / cell_size
	if fx < 0.0 or fz < 0.0 or fx > resolution.x - 1 or fz > resolution.y - 1:
		return NAN
	var i := mini(int(fx), resolution.x - 2)
	var j := mini(int(fz), resolution.y - 2)
	var tx := fx - i
	var tz := fz - j
	var h := func(x: int, z: int) -> float: return heights[z * resolution.x + x]
	var top := lerpf(h.call(i, j), h.call(i + 1, j), tx)
	var bottom := lerpf(h.call(i, j + 1), h.call(i + 1, j + 1), tx)
	return lerpf(top, bottom, tz)

static var _nearest_shader: Shader

## Layer textures use nearest filtering when their material resource does (pixel art projects).
static func _is_pixelated(texture_name: String, settings: FuncGodotMapSettings) -> bool:
	var dir := settings.base_material_dir if settings.base_material_dir != "" else settings.base_texture_dir
	var path := dir.path_join(texture_name + "." + settings.material_file_extension)
	var material: Material = load(path) if ResourceLoader.exists(path) else settings.default_material
	return material is BaseMaterial3D and material.texture_filter in [BaseMaterial3D.TEXTURE_FILTER_NEAREST, BaseMaterial3D.TEXTURE_FILTER_NEAREST_WITH_MIPMAPS, BaseMaterial3D.TEXTURE_FILTER_NEAREST_WITH_MIPMAPS_ANISOTROPIC]

static func _shader(pixelated: bool) -> Shader:
	if not pixelated:
		return SHADER
	if not _nearest_shader:
		_nearest_shader = Shader.new()
		_nearest_shader.code = SHADER.code.replace("filter_linear_mipmap_anisotropic", "filter_nearest_mipmap")
	return _nearest_shader

static func _decode_f32(text: String) -> PackedFloat32Array:
	return Marshalls.base64_to_raw(text).to_float32_array() if text != "" else PackedFloat32Array()

static func _decode_u8(text: String) -> PackedByteArray:
	return Marshalls.base64_to_raw(text) if text != "" else PackedByteArray()

## Builds terrain nodes for every parsed terrain and adds them under the map (or their group).
static func build_all(map_node: Node3D, terrains: Array[Dictionary], settings: FuncGodotMapSettings) -> Array[GodotTrenchTerrain]:
	var out: Array[GodotTrenchTerrain] = []
	if terrains.is_empty():
		return out
	# Same owner rule as the entity assembler: the edited scene in the editor, the map node when built outside a tree.
	var scene_root: Node = map_node
	if map_node.is_inside_tree():
		scene_root = map_node.get_tree().edited_scene_root
	for entry in terrains:
		var terrain := create(entry["data"], entry.get("offset", Vector3.ZERO), settings)
		if not terrain:
			continue
		terrain.name = "terrain_%d" % int(entry.get("id", out.size()))
		var group = entry.get("group", null)
		var parent: Node = map_node
		if settings.use_groups_hierarchy and group and group.node:
			parent = group.node
		parent.add_child(terrain)
		if scene_root:
			terrain.owner = scene_root
			for child in terrain.get_children():
				child.owner = scene_root
		out.append(terrain)
	return out

static func create(data: Dictionary, offset: Vector3, settings: FuncGodotMapSettings) -> GodotTrenchTerrain:
	var res_raw: Array = data.get("resolution", [0, 0])
	var res := Vector2i(int(res_raw[0]), int(res_raw[1]))
	var raw_heights := _decode_f32(str(data.get("heights", "")))
	if res.x < 2 or res.y < 2 or raw_heights.size() != res.x * res.y:
		push_error("[GTM] terrain height data does not match its resolution")
		return null
	var scale := settings.scale_factor
	var cell := float(data.get("cell_size", 32.0)) * scale
	var splat := _decode_u8(str(data.get("splat", "")))
	var holes := _decode_u8(str(data.get("holes", "")))
	var chunk_cells := maxi(int(data.get("chunk_cells", 32)), 1)

	var t := GodotTrenchTerrain.new()
	t.position = (GodotTrenchParser.vec3(data.get("origin")) + offset) * scale
	t.resolution = res
	t.cell_size = cell
	t.heights = PackedFloat32Array()
	t.heights.resize(raw_heights.size())
	for k in raw_heights.size():
		t.heights[k] = raw_heights[k] * scale

	var material := ShaderMaterial.new()
	var layers: Array = data.get("layers", [])
	var tiles := Vector4(8, 8, 8, 8)
	var pixelated := false
	for l in 4:
		var layer: Dictionary = layers[mini(l, layers.size() - 1)] if not layers.is_empty() else {}
		var texture_name := str(layer.get("material", ""))
		if texture_name != "":
			material.set_shader_parameter("layer%d" % l, FuncGodotUtil.load_texture(texture_name, [], settings))
			pixelated = pixelated or _is_pixelated(texture_name, settings)
		tiles[l] = float(layer.get("tile", 256.0)) * scale
	material.shader = _shader(pixelated)
	material.set_shader_parameter("tiles", tiles)
	material.set_shader_parameter("map_offset", t.position)

	var w := res.x
	var cells := Vector2i(res.x - 1, res.y - 1)
	var h := func(i: int, j: int) -> float: return t.heights[clampi(j, 0, res.y - 1) * w + clampi(i, 0, res.x - 1)]
	var cj := 0
	while cj < cells.y:
		var ci := 0
		while ci < cells.x:
			var cw := mini(chunk_cells, cells.x - ci)
			var ch := mini(chunk_cells, cells.y - cj)
			var verts := PackedVector3Array()
			var normals := PackedVector3Array()
			var colors := PackedColorArray()
			var uvs := PackedVector2Array()
			for j in range(cj, cj + ch + 1):
				for i in range(ci, ci + cw + 1):
					verts.append(Vector3(i * cell, h.call(i, j), j * cell))
					var dx: float = h.call(i + 1, j) - h.call(i - 1, j)
					var dz: float = h.call(i, j + 1) - h.call(i, j - 1)
					normals.append(Vector3(-dx, 2.0 * cell, -dz).normalized())
					var k := j * w + i
					if splat.size() >= (k + 1) * 4:
						colors.append(Color(splat[k * 4] / 255.0, splat[k * 4 + 1] / 255.0, splat[k * 4 + 2] / 255.0, splat[k * 4 + 3] / 255.0))
					else:
						colors.append(Color(1, 0, 0, 0))
					uvs.append(Vector2(float(i) / cells.x, float(j) / cells.y))
			var row := cw + 1
			var indices := PackedInt32Array()
			for y in range(cj, cj + ch):
				for x in range(ci, ci + cw):
					if holes.size() > y * cells.x + x and holes[y * cells.x + x] != 0:
						continue
					var p00 := (y - cj) * row + (x - ci)
					var p10 := p00 + 1
					var p01 := p00 + row
					var p11 := p01 + 1
					# Same alternating diagonal as the editor, wound clockwise for Godot's front faces.
					if (x + y) % 2 == 0:
						indices.append_array([p00, p11, p01, p00, p10, p11])
					else:
						indices.append_array([p00, p10, p01, p01, p10, p11])
			if not indices.is_empty():
				var arrays := []
				arrays.resize(Mesh.ARRAY_MAX)
				arrays[Mesh.ARRAY_VERTEX] = verts
				arrays[Mesh.ARRAY_NORMAL] = normals
				arrays[Mesh.ARRAY_COLOR] = colors
				arrays[Mesh.ARRAY_TEX_UV] = uvs
				arrays[Mesh.ARRAY_INDEX] = indices
				var mesh := ArrayMesh.new()
				mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
				mesh.surface_set_material(0, material)
				var mi := MeshInstance3D.new()
				mi.name = "chunk_%d_%d" % [ci / chunk_cells, cj / chunk_cells]
				mi.mesh = mesh
				t.add_child(mi)
			ci += chunk_cells
		cj += chunk_cells

	# HeightMapShape3D samples are one unit apart, so the shape is scaled uniformly by the cell size.
	var shape := HeightMapShape3D.new()
	shape.map_width = res.x
	shape.map_depth = res.y
	var scaled := PackedFloat32Array()
	scaled.resize(t.heights.size())
	for k in t.heights.size():
		scaled[k] = t.heights[k] / cell
	shape.map_data = scaled
	var collision := CollisionShape3D.new()
	collision.name = "collision"
	collision.shape = shape
	collision.scale = Vector3.ONE * cell
	collision.position = Vector3(cells.x * cell * 0.5, 0.0, cells.y * cell * 0.5)
	t.add_child(collision)
	return t
