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

## Triangle corner indices of a polygon, counter-clockwise around [param normal].
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
	var flat := PackedVector2Array()
	for p in points:
		flat.append(Vector2(p.dot(u), p.dot(v)))
	var tris := Geometry2D.triangulate_polygon(flat)
	if tris.is_empty():
		for k in range(1, n - 1):
			tris.append_array([0, k, k + 1])
		return tris
	# Geometry2D may return either winding, triangles are flipped to match the polygon.
	for t in range(0, tris.size(), 3):
		var a := flat[tris[t]]
		var b := flat[tris[t + 1]]
		var c := flat[tris[t + 2]]
		if (b - a).cross(c - a) < 0.0:
			var tmp := tris[t + 1]
			tris[t + 1] = tris[t + 2]
			tris[t + 2] = tmp
	return tris

## Brush data for one mesh node. [param xform] places it (Godot space, map units), [param scale] converts to FuncGodot units.
static func parse(node: Dictionary, xform: Transform3D, scale: float, origin_texture: String) -> _BrushData:
	var raw_vertices: Array = node.get("vertices", [])
	var raw_faces: Array = node.get("faces", [])
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
		var blend_material := str(src.get("props", {}).get("blend_material", ""))
		if blend_material != "":
			face.texture = GodotTrenchBlend.key(face.texture, blend_material)
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
