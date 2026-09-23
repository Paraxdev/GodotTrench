@tool
class_name GodotTrenchTerrain extends StaticBody3D
## Heightmap terrain built from a GodotTrench terrain node: chunked meshes with a four layer blend shader,
## each chunk colliding as a [ConcavePolygonShape3D] built from the same triangles it renders.

const SHADER := preload("res://addons/func_godot/src/godottrench/runtime/gt_terrain.gdshader")

## Vertex count along x and z.
@export var resolution := Vector2i(2, 2)
## Meters between vertices.
@export var cell_size := 1.0
## Heights in meters relative to this node, row major (z then x).
@export var heights := PackedFloat32Array()

## Barycentric weights of [param px]/[param pz] in the XZ projection of triangle abc, or a negative weight when
## the point is outside it.
static func _barycentric(px: float, pz: float, a: Vector3, b: Vector3, c: Vector3) -> Vector3:
	var v0x := b.x - a.x
	var v0z := b.z - a.z
	var v1x := c.x - a.x
	var v1z := c.z - a.z
	var v2x := px - a.x
	var v2z := pz - a.z
	var den := v0x * v1z - v1x * v0z
	if absf(den) < 1e-12:
		return Vector3(-1.0, -1.0, -1.0)
	var v := (v2x * v1z - v1x * v2z) / den
	var w := (v0x * v2z - v2x * v0z) / den
	return Vector3(1.0 - v - w, v, w)

## Height of the terrain surface in local space at a local x/z position, or NAN outside. Follows the same
## triangles as the rendered mesh and Terrain::height_at, respecting the alternating diagonal.
func height_at(local_x: float, local_z: float) -> float:
	var fx := local_x / cell_size
	var fz := local_z / cell_size
	if fx < 0.0 or fz < 0.0 or fx > resolution.x - 1 or fz > resolution.y - 1:
		return NAN
	var ci := mini(int(fx), resolution.x - 2)
	var cj := mini(int(fz), resolution.y - 2)
	var h := func(x: int, z: int) -> float: return heights[z * resolution.x + x]
	var p00 := Vector3(ci * cell_size, h.call(ci, cj), cj * cell_size)
	var p01 := Vector3(ci * cell_size, h.call(ci, cj + 1), (cj + 1) * cell_size)
	var p11 := Vector3((ci + 1) * cell_size, h.call(ci + 1, cj + 1), (cj + 1) * cell_size)
	var p10 := Vector3((ci + 1) * cell_size, h.call(ci + 1, cj), cj * cell_size)
	var tris: Array = [[p00, p01, p11], [p00, p11, p10]] if (ci + cj) % 2 == 0 else [[p00, p01, p10], [p01, p11, p10]]
	for tri in tris:
		var bary := _barycentric(local_x, local_z, tri[0], tri[1], tri[2])
		if bary.x >= -1e-6 and bary.y >= -1e-6 and bary.z >= -1e-6:
			return bary.x * tri[0].y + bary.y * tri[1].y + bary.z * tri[2].y
	return h.call(ci, cj)

static var _nearest_shader: Shader

## The material resource a layer's texture name stands for, the one brushes with that texture get.
static func _layer_material(texture_name: String, settings: FuncGodotMapSettings) -> Material:
	var dir := settings.base_material_dir if settings.base_material_dir != "" else settings.base_texture_dir
	var path := dir.path_join(texture_name + "." + settings.material_file_extension)
	return load(path) if ResourceLoader.exists(path) else settings.default_material

## Layer textures use nearest filtering when their material resource does (pixel art projects).
static func _is_pixelated(texture_name: String, settings: FuncGodotMapSettings) -> bool:
	var material := _layer_material(texture_name, settings)
	return material is BaseMaterial3D and material.texture_filter in [BaseMaterial3D.TEXTURE_FILTER_NEAREST, BaseMaterial3D.TEXTURE_FILTER_NEAREST_WITH_MIPMAPS, BaseMaterial3D.TEXTURE_FILTER_NEAREST_WITH_MIPMAPS_ANISOTROPIC]

## Passes an emissive layer material's emission on to the terrain shader's slot [param slot], so a glowing
## material painted on the terrain glows like it does on a brush.
static func _set_layer_glow(material: ShaderMaterial, slot: int, source: Material, energy: Vector4, textured: Vector4, multiply: Vector4) -> Array[Vector4]:
	if source is BaseMaterial3D and source.emission_enabled:
		material.set_shader_parameter("glow_color%d" % slot, source.emission)
		energy[slot] = source.emission_energy_multiplier
		multiply[slot] = 1.0 if source.emission_operator == BaseMaterial3D.EMISSION_OP_MULTIPLY else 0.0
		if source.emission_texture:
			material.set_shader_parameter("glow_texture%d" % slot, source.emission_texture)
			textured[slot] = 1.0
	return [energy, textured, multiply]

static func _shader(pixelated: bool) -> Shader:
	if not pixelated:
		return SHADER
	if not _nearest_shader:
		_nearest_shader = Shader.new()
		_nearest_shader.code = SHADER.code.replace("filter_linear_mipmap_anisotropic", "filter_nearest_mipmap")
	return _nearest_shader

## Terrain arrays are raw bytes in a binary .gtm and base64 in JSON.
static func _decode_u8(value: Variant) -> PackedByteArray:
	if value is PackedByteArray:
		return value
	var text := str(value) if value != null else ""
	return Marshalls.base64_to_raw(text) if text != "" else PackedByteArray()

## Builds terrain nodes for every parsed terrain and adds them under the map (or their group).
static func build_all(map_node: Node3D, terrains: Array[Dictionary], settings: FuncGodotMapSettings) -> Array[GodotTrenchTerrain]:
	var out: Array[GodotTrenchTerrain] = []
	for entry in terrains:
		var group = entry.get("group", null)
		var parent: Node = map_node
		if settings.use_groups_hierarchy and group and group.node:
			parent = group.node
		var terrain := build_one(map_node, parent, entry["data"], entry.get("xform", Transform3D.IDENTITY), int(entry.get("id", out.size())), settings)
		if terrain:
			out.append(terrain)
	return out

## Creates one terrain node named after its map node id under [param parent], owned like the other generated nodes.
## [param xform] places it: a [Transform3D] for an instance occurrence (rotates the terrain's center about the
## instance, like Terrain::transformed, the grid itself stays axis aligned), or a plain [Vector3] offset for a
## live rebuild that only moves the terrain.
static func build_one(map_node: Node, parent: Node, data: Dictionary, xform: Variant, id: int, settings: FuncGodotMapSettings) -> GodotTrenchTerrain:
	var terrain := create(data, xform, settings)
	if not terrain:
		return null
	terrain.name = "terrain_%d" % id
	terrain.set_meta(GodotTrenchBuild.ID_META, id)
	parent.add_child(terrain)
	var scene_root := GodotTrenchBuild.scene_owner(map_node)
	terrain.owner = scene_root
	for child in terrain.get_children():
		child.owner = scene_root
	return terrain

## Surface arrays of the chunk starting at cell [param start], empty when every cell is a hole.
static func _chunk_arrays(start: Vector2i, chunk_cells: int, res: Vector2i, cell: float, heights: PackedFloat32Array, splat: PackedByteArray, holes: PackedByteArray) -> Array:
	var w := res.x
	var cells := Vector2i(res.x - 1, res.y - 1)
	var ci := start.x
	var cj := start.y
	var cw := mini(chunk_cells, cells.x - ci)
	var ch := mini(chunk_cells, cells.y - cj)
	var verts := PackedVector3Array()
	var normals := PackedVector3Array()
	var colors := PackedColorArray()
	var uvs := PackedVector2Array()
	for j in range(cj, cj + ch + 1):
		for i in range(ci, ci + cw + 1):
			var here := heights[j * w + i]
			verts.append(Vector3(i * cell, here, j * cell))
			var dx: float = heights[j * w + mini(i + 1, res.x - 1)] - heights[j * w + maxi(i - 1, 0)]
			var dz: float = heights[mini(j + 1, res.y - 1) * w + i] - heights[maxi(j - 1, 0) * w + i]
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
	if indices.is_empty():
		return []
	var arrays := []
	arrays.resize(Mesh.ARRAY_MAX)
	arrays[Mesh.ARRAY_VERTEX] = verts
	arrays[Mesh.ARRAY_NORMAL] = normals
	arrays[Mesh.ARRAY_COLOR] = colors
	arrays[Mesh.ARRAY_TEX_UV] = uvs
	arrays[Mesh.ARRAY_INDEX] = indices
	return arrays

static func create(data: Dictionary, xform: Variant, settings: FuncGodotMapSettings) -> GodotTrenchTerrain:
	var res_raw: Array = data.get("resolution", [0, 0])
	var res := Vector2i(int(res_raw[0]), int(res_raw[1]))
	var raw_heights := _decode_u8(data.get("heights")).to_float32_array()
	if res.x < 2 or res.y < 2 or raw_heights.size() != res.x * res.y:
		push_error("[GTM] terrain height data does not match its resolution")
		return null
	var scale := settings.scale_factor
	var cell := float(data.get("cell_size", 32.0)) * scale
	var splat := _decode_u8(data.get("splat"))
	var holes := _decode_u8(data.get("holes"))
	var chunk_cells := maxi(int(data.get("chunk_cells", 32)), 1)

	var t := GodotTrenchTerrain.new()
	var t_xform: Transform3D = xform if xform is Transform3D else Transform3D(Basis.IDENTITY, xform as Vector3)
	# Terrains stay axis aligned: like Terrain::transformed, the instance rotates the terrain's center about
	# its own origin, not the grid, so a rotated prefab's terrain does not tilt.
	var raw_cell := float(data.get("cell_size", 32.0))
	var local_origin := GodotTrenchParser.vec3(data.get("origin"))
	var size := Vector3((res.x - 1) * raw_cell, 0.0, (res.y - 1) * raw_cell)
	var center := local_origin + size * 0.5
	var new_origin: Vector3 = t_xform * center - size * 0.5
	t.position = new_origin * scale
	t.resolution = res
	t.cell_size = cell
	t.heights = PackedFloat32Array()
	t.heights.resize(raw_heights.size())
	for k in raw_heights.size():
		t.heights[k] = raw_heights[k] * scale

	var material := ShaderMaterial.new()
	var layers: Array = data.get("layers", [])
	var tiles := Vector4(8, 8, 8, 8)
	var detiles := Vector4.ZERO
	var sharpens := Vector4(0.5, 0.5, 0.5, 0.5)
	var glow := [Vector4.ZERO, Vector4.ZERO, Vector4.ZERO]
	var pixelated := false
	for l in 4:
		# Weight in a slot without a layer shows the first layer, like the editor's prepare_terrain_material.
		var layer: Dictionary = (layers[l] if l < layers.size() else layers[0]) if not layers.is_empty() else {}
		var texture_name := str(layer.get("material", ""))
		if texture_name != "":
			material.set_shader_parameter("layer%d" % l, FuncGodotUtil.load_texture(texture_name, [], settings))
			pixelated = pixelated or _is_pixelated(texture_name, settings)
			glow = _set_layer_glow(material, l, _layer_material(texture_name, settings), glow[0], glow[1], glow[2])
		tiles[l] = float(layer.get("tile", 256.0)) * scale
		detiles[l] = clampf(float(layer.get("detile", 0.0)), 0.0, 1.0)
		sharpens[l] = clampf(float(layer.get("detile_sharpen", 0.5)), 0.0, 1.0)
	material.shader = _shader(pixelated)
	material.set_shader_parameter("tiles", tiles)
	material.set_shader_parameter("detiles", detiles)
	material.set_shader_parameter("sharpens", sharpens)
	material.set_shader_parameter("glow_energy", glow[0])
	material.set_shader_parameter("glow_textured", glow[1])
	material.set_shader_parameter("glow_multiply", glow[2])
	material.set_shader_parameter("map_offset", t.position)

	var cells := Vector2i(res.x - 1, res.y - 1)
	var chunks: Array[Vector2i] = []
	for cj in range(0, cells.y, chunk_cells):
		for ci in range(0, cells.x, chunk_cells):
			chunks.append(Vector2i(ci, cj))
	var heights := t.heights
	var surfaces := []
	surfaces.resize(chunks.size())
	var build_chunk := func(c: int) -> void:
		surfaces[c] = _chunk_arrays(chunks[c], chunk_cells, res, cell, heights, splat, holes)
	# GodotTrench: chunk arrays are plain data, only the ArrayMesh resources are created on this thread.
	if GodotTrenchBuild.threaded() and chunks.size() > 1:
		var task := WorkerThreadPool.add_group_task(build_chunk, chunks.size(), -1, false, "Build GodotTrench terrain chunks")
		WorkerThreadPool.wait_for_group_task_completion(task)
	else:
		for c in chunks.size():
			build_chunk.call(c)
	for c in chunks.size():
		if surfaces[c].is_empty():
			continue
		var mesh := ArrayMesh.new()
		mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, surfaces[c])
		mesh.surface_set_material(0, material)
		var mi := MeshInstance3D.new()
		mi.name = "chunk_%d_%d" % [chunks[c].x / chunk_cells, chunks[c].y / chunk_cells]
		mi.mesh = mesh
		t.add_child(mi)

	# HeightMapShape3D can only punch holes by setting a whole vertex to NAN, which would also remove its other,
	# solid cells, and it always splits a cell along one fixed diagonal. Holes and the alternating diagonal are
	# per cell, not per vertex, so collision instead reuses the exact triangles the chunks above were built
	# from: one ConcavePolygonShape3D per chunk, holes and all, matching the visuals and Terrain::cell_triangles.
	for c in chunks.size():
		var arrays: Array = surfaces[c]
		if arrays.is_empty():
			continue
		var chunk_verts: PackedVector3Array = arrays[Mesh.ARRAY_VERTEX]
		var chunk_indices: PackedInt32Array = arrays[Mesh.ARRAY_INDEX]
		var faces := PackedVector3Array()
		faces.resize(chunk_indices.size())
		for k in chunk_indices.size():
			faces[k] = chunk_verts[chunk_indices[k]]
		var shape := ConcavePolygonShape3D.new()
		shape.set_faces(faces)
		var collision := CollisionShape3D.new()
		collision.name = "collision_%d_%d" % [chunks[c].x / chunk_cells, chunks[c].y / chunk_cells]
		collision.shape = shape
		t.add_child(collision)
	return t
