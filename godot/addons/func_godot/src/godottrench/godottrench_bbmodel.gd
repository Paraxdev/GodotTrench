class_name GodotTrenchBBModel extends RefCounted
## Blockbench .bbmodel reader and scene builder. Uses the same conventions as the GodotTrench editor:
## 16 Blockbench units per meter, Y up, north is -Z, rotations about origins in ZYX order.

const FACE_BASIS := {
	"north": [Vector3(0, 0, -1), Vector3(-1, 0, 0), Vector3(0, 1, 0)],
	"south": [Vector3(0, 0, 1), Vector3(1, 0, 0), Vector3(0, 1, 0)],
	"east": [Vector3(1, 0, 0), Vector3(0, 0, -1), Vector3(0, 1, 0)],
	"west": [Vector3(-1, 0, 0), Vector3(0, 0, 1), Vector3(0, 1, 0)],
	"up": [Vector3(0, 1, 0), Vector3(1, 0, 0), Vector3(0, 0, -1)],
	"down": [Vector3(0, -1, 0), Vector3(1, 0, 0), Vector3(0, 0, 1)],
}

static func _vec3(a: Variant) -> Vector3:
	if a is Array and a.size() >= 3:
		return Vector3(float(a[0]), float(a[1]), float(a[2]))
	return Vector3.ZERO

static func pivot_rotation(origin: Vector3, rotation_deg: Vector3) -> Transform3D:
	var b := Basis(Vector3.BACK, deg_to_rad(rotation_deg.z)) * Basis(Vector3.UP, deg_to_rad(rotation_deg.y)) * Basis(Vector3.RIGHT, deg_to_rad(rotation_deg.x))
	return Transform3D(Basis.IDENTITY, origin) * Transform3D(b, Vector3.ZERO) * Transform3D(Basis.IDENTITY, -origin)

static func _box_uv_rect(face: String, offset: Vector2, size: Vector3) -> Array:
	var w := size.x
	var h := size.y
	var d := size.z
	var u := offset.x
	var v := offset.y
	match face:
		"east": return [u, v + d, u + d, v + d + h]
		"north": return [u + d, v + d, u + d + w, v + d + h]
		"west": return [u + d + w, v + d, u + d * 2 + w, v + d + h]
		"south": return [u + d * 2 + w, v + d, u + d * 2 + w * 2, v + d + h]
		"up": return [u + d + w, v + d, u + d, v]
		_: return [u + d + w * 2, v, u + d + w, v + d]

static func _newell(points: PackedVector3Array) -> Vector3:
	return GodotTrenchMesh._newell(points)

## Parses a model into { "polygons": [{positions, uvs, texture}], "textures": [{name, image}] }, positions in Blockbench units.
static func parse(text: String) -> Dictionary:
	var root = JSON.parse_string(text)
	if not root is Dictionary or not root.has("elements") or not root.has("meta"):
		return {}
	var resolution := Vector2(float(root.get("resolution", {}).get("width", 16)), float(root.get("resolution", {}).get("height", 16)))
	var global_box_uv := bool(root.get("meta", {}).get("box_uv", false))

	var textures: Array[Dictionary] = []
	var texture_index := {}
	for i in root.get("textures", []).size():
		var t: Dictionary = root["textures"][i]
		var image := Image.new()
		var source := str(t.get("source", ""))
		var comma := source.find("base64,")
		if comma >= 0:
			image.load_png_from_buffer(Marshalls.base64_to_raw(source.substr(comma + 7)))
		textures.append({
			"name": str(t.get("name", "texture")).trim_suffix(".png"),
			"image": image,
			"uv_size": Vector2(float(t.get("uv_width", resolution.x)), float(t.get("uv_height", resolution.y))),
			"emissive": str(t.get("render_mode", "")) == "emissive",
		})
		if t.has("id"):
			texture_index[str(t["id"])] = i
		if t.has("uuid"):
			texture_index[str(t["uuid"])] = i

	var texture_ref := func(v: Variant) -> int:
		if v is float or v is int:
			return int(v)
		if v is String:
			if texture_index.has(v):
				return texture_index[v]
			if v.is_valid_int():
				return int(v)
		return -1
	var uv_size := func(tex: int) -> Vector2:
		return textures[tex]["uv_size"] if tex >= 0 and tex < textures.size() else resolution

	# Blockbench 5 keeps group data in a separate "groups" list, the outliner only names the uuid.
	var groups := {}
	for g in root.get("groups", []):
		if g is Dictionary and g.has("uuid"):
			groups[str(g["uuid"])] = g
	var element_xform := {}
	var stack: Array = []
	for node in root.get("outliner", []):
		stack.append([node, Transform3D.IDENTITY])
	while not stack.is_empty():
		var item: Array = stack.pop_back()
		var node = item[0]
		var parent: Transform3D = item[1]
		if node is String:
			element_xform[node] = parent
		elif node is Dictionary:
			var data: Dictionary = groups.get(str(node.get("uuid", "")), {})
			var m := parent * pivot_rotation(_vec3(node.get("origin", data.get("origin"))), _vec3(node.get("rotation", data.get("rotation"))))
			for c in node.get("children", []):
				stack.append([c, m])

	var polygons: Array[Dictionary] = []
	for el in root["elements"]:
		if el.get("visibility", true) == false:
			continue
		var group: Transform3D = element_xform.get(str(el.get("uuid", "")), Transform3D.IDENTITY)
		var origin := _vec3(el.get("origin"))
		var rotation := _vec3(el.get("rotation"))
		if str(el.get("type", "cube")) == "mesh":
			var xform := group * Transform3D(Basis.IDENTITY, origin) * pivot_rotation(Vector3.ZERO, rotation)
			var verts: Dictionary = el.get("vertices", {})
			for face in el.get("faces", {}).values():
				var keys: Array = face.get("vertices", [])
				if keys.size() < 3:
					continue
				var pts := PackedVector3Array()
				for k in keys:
					pts.append(xform * _vec3(verts.get(k)))
				var order := range(keys.size())
				if keys.size() == 4:
					var best := -1.0
					for candidate in [[0, 1, 2, 3], [0, 1, 3, 2], [0, 2, 1, 3]]:
						var p := PackedVector3Array()
						for i in candidate:
							p.append(pts[i])
						var area := _newell(p).length()
						if area > best:
							best = area
							order = candidate
				var tex: int = texture_ref.call(face.get("texture", -1))
				var size: Vector2 = uv_size.call(tex)
				var positions := PackedVector3Array()
				var uvs := PackedVector2Array()
				for i in order:
					positions.append(pts[i])
					var uv = face.get("uv", {}).get(keys[i], [0, 0])
					uvs.append(Vector2(float(uv[0]) / size.x, float(uv[1]) / size.y))
				polygons.append({ "positions": positions, "uvs": uvs, "texture": tex })
			continue
		var inflate := float(el.get("inflate", 0.0))
		var from := _vec3(el.get("from")) - Vector3.ONE * inflate
		var to := _vec3(el.get("to")) + Vector3.ONE * inflate
		var xform := group * pivot_rotation(origin, rotation)
		var box_uv := bool(el.get("box_uv", global_box_uv))
		var uv_off_raw: Array = el.get("uv_offset", [0, 0])
		var uv_offset := Vector2(float(uv_off_raw[0]), float(uv_off_raw[1]))
		var size3 := (_vec3(el.get("to")) - _vec3(el.get("from"))).abs().floor()
		for dir in ["north", "south", "east", "west", "up", "down"]:
			var face = el.get("faces", {}).get(dir, null)
			if not face is Dictionary or (face.has("texture") and face["texture"] == null):
				continue
			var tex: int = texture_ref.call(face.get("texture", -1))
			var size: Vector2 = uv_size.call(tex)
			var rect: Array = _box_uv_rect(dir, uv_offset, size3) if box_uv else face.get("uv", [0, 0, 0, 0])
			var basis: Array = FACE_BASIS[dir]
			var corner := func(sr: float, su: float) -> Vector3:
				var p := Vector3.ZERO
				for a in 3:
					var pick: bool
					if basis[0][a] != 0.0:
						pick = basis[0][a] > 0.0
					elif basis[1][a] != 0.0:
						pick = sr * basis[1][a] > 0.0
					else:
						pick = su * basis[2][a] > 0.0
					p[a] = to[a] if pick else from[a]
				return xform * p
			var arr := [[rect[0], rect[1]], [rect[2], rect[1]], [rect[0], rect[3]], [rect[2], rect[3]]]
			var rot := int(face.get("rotation", 0))
			while rot > 0:
				var first = arr[0]
				arr[0] = arr[2]
				arr[2] = arr[3]
				arr[3] = arr[1]
				arr[1] = first
				rot -= 90
			var n := func(c: Array) -> Vector2: return Vector2(float(c[0]) / size.x, float(c[1]) / size.y)
			polygons.append({
				"positions": PackedVector3Array([corner.call(-1.0, 1.0), corner.call(-1.0, -1.0), corner.call(1.0, -1.0), corner.call(1.0, 1.0)]),
				"uvs": PackedVector2Array([n.call(arr[0]), n.call(arr[2]), n.call(arr[3]), n.call(arr[1])]),
				"texture": tex,
			})
	return { "polygons": polygons, "textures": textures, "name": str(root.get("name", "model")) }

## Builds a scene: one MeshInstance3D with a surface per texture, plus optional collision.
static func build_scene(model: Dictionary, scale: float = 1.0 / 16.0, collision: String = "none", filter_nearest: bool = true) -> Node3D:
	var root := Node3D.new()
	root.name = str(model.get("name", "model")).validate_node_name()
	var by_texture := {}
	for poly in model.get("polygons", []):
		var tex: int = poly["texture"]
		if not by_texture.has(tex):
			by_texture[tex] = { "verts": PackedVector3Array(), "normals": PackedVector3Array(), "uvs": PackedVector2Array(), "indices": PackedInt32Array() }
		var s: Dictionary = by_texture[tex]
		var positions: PackedVector3Array = poly["positions"]
		var normal := _newell(positions).normalized()
		var base: int = s["verts"].size()
		for k in positions.size():
			s["verts"].append(positions[k] * scale)
			s["normals"].append(normal)
			s["uvs"].append(poly["uvs"][k])
		var tris := GodotTrenchMesh.triangulate(positions, normal)
		for t in range(0, tris.size(), 3):
			# Godot front faces are clockwise.
			s["indices"].append_array([base + tris[t], base + tris[t + 2], base + tris[t + 1]])
	var mesh := ArrayMesh.new()
	var textures: Array = model.get("textures", [])
	for tex in by_texture.keys():
		var s: Dictionary = by_texture[tex]
		var arrays := []
		arrays.resize(Mesh.ARRAY_MAX)
		arrays[Mesh.ARRAY_VERTEX] = s["verts"]
		arrays[Mesh.ARRAY_NORMAL] = s["normals"]
		arrays[Mesh.ARRAY_TEX_UV] = s["uvs"]
		arrays[Mesh.ARRAY_INDEX] = s["indices"]
		mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
		var material := StandardMaterial3D.new()
		if tex >= 0 and tex < textures.size() and not (textures[tex]["image"] as Image).is_empty():
			material.albedo_texture = ImageTexture.create_from_image(textures[tex]["image"])
			material.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA_SCISSOR
			material.alpha_scissor_threshold = 0.5
			# Blockbench's emissive render mode draws the texture at full brightness.
			if textures[tex].get("emissive", false):
				material.emission_enabled = true
				material.emission = Color.BLACK
				material.emission_texture = material.albedo_texture
		if filter_nearest:
			material.texture_filter = BaseMaterial3D.TEXTURE_FILTER_NEAREST_WITH_MIPMAPS
		material.cull_mode = BaseMaterial3D.CULL_BACK
		mesh.surface_set_material(mesh.get_surface_count() - 1, material)
		mesh.surface_set_name(mesh.get_surface_count() - 1, textures[tex]["name"] if tex >= 0 and tex < textures.size() else "untextured")
	var mi := MeshInstance3D.new()
	mi.name = "mesh"
	mi.mesh = mesh
	root.add_child(mi)
	mi.owner = root
	if collision != "none" and mesh.get_surface_count() > 0:
		var body := StaticBody3D.new()
		body.name = "collision"
		root.add_child(body)
		body.owner = root
		var shape := CollisionShape3D.new()
		shape.name = "shape"
		shape.shape = mesh.create_convex_shape() if collision == "convex" else mesh.create_trimesh_shape()
		body.add_child(shape)
		shape.owner = root
	return root
