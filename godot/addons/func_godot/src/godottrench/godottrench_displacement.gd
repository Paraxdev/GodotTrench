class_name GodotTrenchDisplacement extends RefCounted
## Builds displacement grids from GodotTrench face data and appends them to FuncGodot surface arrays.

## Grid data for a quad face. [param corners] are Godot space map units in face winding order
## (counter-clockwise around [param normal]); returns positions, base positions, normals and triangles.
static func build_grid(corners: PackedVector3Array, normal: Vector3, power: int, heights: Array) -> Dictionary:
	var n := (1 << power) + 1
	var base := PackedVector3Array()
	var positions := PackedVector3Array()
	base.resize(n * n)
	positions.resize(n * n)
	for j in n:
		var v := float(j) / float(n - 1)
		for i in n:
			var u := float(i) / float(n - 1)
			var p: Vector3 = corners[0] * (1.0 - u) * (1.0 - v) + corners[1] * u * (1.0 - v) + corners[2] * u * v + corners[3] * (1.0 - u) * v
			var k := j * n + i
			base[k] = p
			var h := float(heights[k]) if k < heights.size() else 0.0
			positions[k] = p + normal * h
	# Same diagonal pattern as the editor so the surface matches exactly.
	var tris := PackedInt32Array()
	for j in n - 1:
		for i in n - 1:
			var a := j * n + i
			var b := a + 1
			var c := a + n + 1
			var d := a + n
			if (i + j) % 2 == 0:
				tris.append_array([a, b, c, a, c, d])
			else:
				tris.append_array([a, b, d, b, c, d])
	var normals := PackedVector3Array()
	normals.resize(n * n)
	for t in range(0, tris.size(), 3):
		var fn := (positions[tris[t + 1]] - positions[tris[t]]).cross(positions[tris[t + 2]] - positions[tris[t]])
		for m in 3:
			normals[tris[t + m]] += fn
	for k in n * n:
		normals[k] = normals[k].normalized() if normals[k].length_squared() > 1e-12 else normal
	return { "size": n, "base": base, "positions": positions, "normals": normals, "triangles": tris }

## Collision triangles in the entity's local OpenGL space.
static func triangles(face: FuncGodotData.FaceData, xf: Callable) -> PackedVector3Array:
	var out := PackedVector3Array()
	# Godot treats clockwise triangles as front facing, the grid is counter-clockwise, so each triangle is reversed.
	for t in range(0, face.disp_indices.size(), 3):
		out.append(xf.call(face.disp_vertices[face.disp_indices[t]]))
		out.append(xf.call(face.disp_vertices[face.disp_indices[t + 2]]))
		out.append(xf.call(face.disp_vertices[face.disp_indices[t + 1]]))
	return out

## Appends the displacement to a surface array. Returns the number of vertices added.
static func append_surface(arrays: Array, face: FuncGodotData.FaceData, xf: Callable, texture_size: Vector2, index_offset: int, use_colors: bool) -> int:
	var count := face.disp_vertices.size()
	var explicit_uvs := face.disp_uvs.size() == count
	var explicit_colors := face.disp_colors.size() == count
	for k in count:
		arrays[Mesh.ARRAY_VERTEX].append(xf.call(face.disp_vertices[k]))
		arrays[Mesh.ARRAY_NORMAL].append(FuncGodotUtil.id_to_opengl(face.disp_normals[k]))
		if explicit_uvs:
			arrays[Mesh.ARRAY_TEX_UV].append(face.disp_uvs[k])
		else:
			arrays[Mesh.ARRAY_TEX_UV].append(FuncGodotUtil.get_face_vertex_uv(face.disp_base[k], face, texture_size))
		for j in 4:
			arrays[Mesh.ARRAY_TANGENT].append(face.tangents[j] if face.tangents.size() >= 4 else 0.0)
		if use_colors:
			if explicit_colors:
				arrays[Mesh.ARRAY_COLOR].append(face.disp_colors[k])
			else:
				var alpha := face.disp_alphas[k] if k < face.disp_alphas.size() else 0.0
				arrays[Mesh.ARRAY_COLOR].append(Color(1, 1, 1, alpha))
	for t in range(0, face.disp_indices.size(), 3):
		arrays[Mesh.ARRAY_INDEX].append(face.disp_indices[t] + index_offset)
		arrays[Mesh.ARRAY_INDEX].append(face.disp_indices[t + 2] + index_offset)
		arrays[Mesh.ARRAY_INDEX].append(face.disp_indices[t + 1] + index_offset)
	return count
