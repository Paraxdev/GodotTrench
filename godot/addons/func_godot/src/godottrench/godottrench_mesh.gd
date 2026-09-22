class_name GodotTrenchMesh extends RefCounted
## Converts GodotTrench mesh nodes (free form polygons) into FuncGodot brush data with custom surfaces.
## Faces are triangulated here, smooth normals follow the node's smoothing angle exactly like the editor.

const _BrushData := FuncGodotData.BrushData
const _FaceData := FuncGodotData.FaceData

static func _newell(points: PackedVector3Array) -> Vector3:
	var n := Vector3.ZERO
	for i in points.size():
		var cur := points[i]
		var nxt := points[(i + 1) % points.size()]
		n.x += (cur.y - nxt.y) * (cur.z + nxt.z)
		n.y += (cur.z - nxt.z) * (cur.x + nxt.x)
		n.z += (cur.x - nxt.x) * (cur.y + nxt.y)
	return n

static func _cross2(a: Vector2, b: Vector2, c: Vector2) -> float:
	return (b - a).cross(c - a)

## Triangle corner indices of a polygon, counter-clockwise around [param normal]. Ear clipping, mirroring
## gt_geom::polygon::triangulate exactly (including its shorter-diagonal quad split) so the editor and the Godot
## build triangulate the same face the same way.
static func triangulate(points: PackedVector3Array, normal: Vector3) -> PackedInt32Array:
	var n := points.size()
	if n < 3:
		return PackedInt32Array()
	if n == 3:
		return PackedInt32Array([0, 1, 2])
	var nn := normal.normalized() if normal.length_squared() > 1e-12 else Vector3.UP
	var helper := Vector3.UP if absf(nn.y) < 0.9 else Vector3.RIGHT
	var u := helper.cross(nn).normalized()
	var v := nn.cross(u)
	var p2 := PackedVector2Array()
	for p in points:
		p2.append(Vector2(p.dot(u), p.dot(v)))

	if n == 4:
		var convex_quad := true
		for i in 4:
			if _cross2(p2[i], p2[(i + 1) % 4], p2[(i + 2) % 4]) <= 0.0:
				convex_quad = false
				break
		if convex_quad:
			# Split along the shorter diagonal, it keeps slivers out of near-planar quads.
			if p2[0].distance_squared_to(p2[2]) <= p2[1].distance_squared_to(p2[3]):
				return PackedInt32Array([0, 1, 2, 0, 2, 3])
			return PackedInt32Array([0, 1, 3, 1, 2, 3])

	var idx: Array[int] = []
	for i in n:
		idx.append(i)
	var out := PackedInt32Array()
	var guard := 0
	while idx.size() > 3 and guard < n * n:
		guard += 1
		var m := idx.size()
		var clipped := false
		for k in m:
			var ia: int = idx[(k + m - 1) % m]
			var ib: int = idx[k]
			var ic: int = idx[(k + 1) % m]
			var a := p2[ia]
			var b := p2[ib]
			var c := p2[ic]
			if _cross2(a, b, c) <= 1e-12:
				continue
			var blocked := false
			for j in idx:
				if j != ia and j != ib and j != ic and p2[j] != a and p2[j] != b and p2[j] != c and _cross2(a, b, p2[j]) >= -1e-9 and _cross2(b, c, p2[j]) >= -1e-9 and _cross2(c, a, p2[j]) >= -1e-9:
					blocked = true
					break
			if blocked:
				continue
			out.append_array([ia, ib, ic])
			idx.remove_at(k)
			clipped = true
			break
		if not clipped:
			break
	if idx.size() == 3:
		out.append_array([idx[0], idx[1], idx[2]])
	elif idx.size() > 3:
		for k in range(1, idx.size() - 1):
			out.append_array([idx[0], idx[k], idx[k + 1]])
	return out

## Brush data for one mesh node. [param xform] places it (Godot space, map units), [param scale] converts to FuncGodot units.
static func parse(node: Dictionary, xform: Transform3D, scale: float, origin_texture: String) -> _BrushData:
	var raw_vertices: Array = node.get("vertices", [])
	var raw_faces: Array = node.get("faces", [])
	var decal := bool(node.get("decal", false))
	if raw_vertices.size() < 3 or raw_faces.is_empty():
		return null
	var mirror := xform.basis.determinant() < 0.0
	var vertices := PackedVector3Array()
	vertices.resize(raw_vertices.size())
	for i in raw_vertices.size():
		vertices[i] = xform * GodotTrenchParser.vec3(raw_vertices[i])
	var smooth_angle := float(node.get("smooth_angle", 0.0))

	# Face corner lists, normals and the faces around each vertex.
	var faces: Array[Dictionary] = []
	var vertex_faces: Dictionary = {}
	for f in raw_faces:
		var indices: Array = f.get("indices", [])
		var uvs: Array = f.get("uvs", [])
		var colors: Array = f.get("colors", [])
		if indices.size() < 3:
			continue
		if mirror:
			indices = indices.duplicate()
			indices.reverse()
			uvs = uvs.duplicate()
			uvs.reverse()
			colors = colors.duplicate()
			colors.reverse()
		var pts := PackedVector3Array()
		var valid := true
		for idx in indices:
			if int(idx) < 0 or int(idx) >= vertices.size():
				valid = false
				break
			pts.append(vertices[int(idx)])
		if not valid:
			continue
		var raw_normal := _newell(pts)
		if raw_normal.length_squared() < 1e-12:
			continue
		var fi := faces.size()
		faces.append({ "indices": indices, "points": pts, "normal": raw_normal, "unit": raw_normal.normalized(), "uvs": uvs, "colors": colors, "src": f })
		for idx in indices:
			if not vertex_faces.has(int(idx)):
				vertex_faces[int(idx)] = []
			vertex_faces[int(idx)].append(fi)

	var cos_limit := cos(deg_to_rad(smooth_angle)) - 1e-6
	var brush := _BrushData.new()
	brush.exact = true
	brush.has_disp = true
	brush.is_mesh = true
	brush.origin = false
	var edge_uses: Dictionary = {}
	for face_info in faces:
		var indices: Array = face_info["indices"]
		for k in indices.size():
			var a := int(indices[k])
			var b := int(indices[(k + 1) % indices.size()])
			var edge := Vector2i(mini(a, b), maxi(a, b))
			edge_uses[edge] = int(edge_uses.get(edge, 0)) + 1
	brush.closed = edge_uses.values().all(func(n): return n == 2)
	for face_info in faces:
		var pts: PackedVector3Array = face_info["points"]
		var unit: Vector3 = face_info["unit"]
		var indices: Array = face_info["indices"]
		var src: Dictionary = face_info["src"]

		var face := _FaceData.new()
		var centroid := Vector3.ZERO
		for p in pts:
			centroid += p
		centroid /= pts.size()
		var id_normal := GodotTrenchParser.to_id(unit)
		face.plane = Plane(id_normal, id_normal.dot(GodotTrenchParser.to_id(centroid) * scale))
		face.texture = str(src.get("material", ""))
		var face_props: Dictionary = src.get("props", {})
		var blend_material := str(face_props.get("blend_material", ""))
		if blend_material != "":
			var detile := float(face_props.get("blend_detile", 0.0))
			var uv_scale := float(face_props.get("blend_uv_scale", 1.0))
			var sharpen := float(face_props.get("blend_detile_sharpen", 0.5))
			face.texture = GodotTrenchBlend.key(face.texture, blend_material, detile, uv_scale, sharpen)
		if decal:
			face.texture += GodotTrenchDecalMesh.SUFFIX
		for p in pts:
			var id_p := GodotTrenchParser.to_id(p) * scale
			face.exact_vertices.append(id_p)
			face.disp_vertices.append(id_p)
			face.disp_base.append(id_p)
		for k in pts.size():
			var n := unit
			if smooth_angle > 0.0:
				var sum := Vector3.ZERO
				for other in vertex_faces.get(int(indices[k]), []):
					var of: Dictionary = faces[other]
					if (of["unit"] as Vector3).dot(unit) >= cos_limit:
						sum += of["normal"]
				if sum.length_squared() > 1e-12:
					n = sum.normalized()
			face.disp_normals.append(GodotTrenchParser.to_id(n))
		face.disp_indices = triangulate(pts, unit)

		var uvs: Array = face_info["uvs"]
		if uvs.size() == pts.size():
			for uv in uvs:
				face.disp_uvs.append(Vector2(float(uv[0]), float(uv[1])))
		var colors: Array = face_info["colors"]
		if colors.size() == pts.size():
			for c in colors:
				face.disp_colors.append(Color(float(c[0]), float(c[1]), float(c[2]), float(c[3])))

		var uv: Dictionary = src.get("uv", {})
		var u_axis := GodotTrenchParser.vec3(uv.get("u_axis"), Vector3.RIGHT)
		var v_axis := GodotTrenchParser.vec3(uv.get("v_axis"), Vector3.BACK)
		var offset := GodotTrenchParser.vec2(uv.get("offset"), Vector2.ZERO)
		var uv_scale := GodotTrenchParser.vec2(uv.get("scale"), Vector2.ONE)
		if xform != Transform3D.IDENTITY:
			var inv_t := xform.basis.inverse().transposed()
			var t := xform.origin
			var mu := inv_t * u_axis
			var mv := inv_t * v_axis
			var lu := maxf(mu.length(), 1e-9)
			var lv := maxf(mv.length(), 1e-9)
			offset.x -= mu.dot(t) / uv_scale.x
			offset.y -= mv.dot(t) / uv_scale.y
			uv_scale = Vector2(uv_scale.x / lu, uv_scale.y / lv)
			u_axis = mu / lu
			v_axis = mv / lv
		face.uv_axes.append(GodotTrenchParser.to_id(u_axis))
		face.uv_axes.append(GodotTrenchParser.to_id(v_axis))
		face.uv = Transform2D.IDENTITY
		face.uv.origin = offset
		face.uv.x = Vector2(uv_scale.x, 0.0) * scale
		face.uv.y = Vector2(0.0, uv_scale.y) * scale
		face.props = src.get("props", {})
		brush.planes.append(face.plane)
		brush.faces.append(face)
	if brush.faces.is_empty():
		return null
	return brush
