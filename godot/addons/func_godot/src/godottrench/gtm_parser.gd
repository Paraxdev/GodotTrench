class_name GodotTrenchParser extends RefCounted
## Converts GodotTrench .gtm maps into [FuncGodotData] so the regular FuncGodot build pipeline can use them.
##
## GodotTrench stores maps Y-up in Godot's axis convention with exact brush vertices.
## FuncGodot works in id Tech axes internally, so everything is rotated into that space
## ([code]id = (g.z, g.x, g.y)[/code], a pure rotation, so windings and UV axes stay valid).

const _GroupData := FuncGodotData.GroupData
const _EntityData := FuncGodotData.EntityData
const _BrushData := FuncGodotData.BrushData
const _FaceData := FuncGodotData.FaceData
const _ParseData := FuncGodotData.ParseData

const FORMAT_NAME := "godottrench-map"
const MAX_INSTANCE_DEPTH := 8

## Godot space (map units, Y-up) to FuncGodot's id space.
static func to_id(v: Vector3) -> Vector3:
	return Vector3(v.z, v.x, v.y)

static func vec3(a: Variant, fallback := Vector3.ZERO) -> Vector3:
	if a is Array and a.size() >= 3:
		return Vector3(float(a[0]), float(a[1]), float(a[2]))
	return fallback

static func vec2(a: Variant, fallback := Vector2.ONE) -> Vector2:
	if a is Array and a.size() >= 2:
		return Vector2(float(a[0]), float(a[1]))
	return fallback

## GodotTrench angles are node rotation degrees (YXZ). FuncGodot applies (-a0, a1 + 180, -a2) to Quake angles.
static func angles_to_quake(angles: Vector3) -> String:
	return "%s %s %s" % [-angles.x, angles.y - 180.0, -angles.z]

static func rotation_basis(angles: Vector3) -> Basis:
	return Basis.from_euler(Vector3(deg_to_rad(angles.x), deg_to_rad(angles.y), deg_to_rad(angles.z)), EULER_ORDER_YXZ)


class Context:
	var map_settings: FuncGodotMapSettings
	var parse_data: _ParseData
	var map_path: String
	var worldspawn: _EntityData
	var next_group_id := 1
	var instance_depth := 0
	## Transform applied to everything parsed at the current instance level (Godot space, map units).
	var xform := Transform3D.IDENTITY
	var name_prefix := ""


static func parse(text: String, map_settings: FuncGodotMapSettings, parse_data: _ParseData, map_path: String = "") -> _ParseData:
	var json = JSON.parse_string(text)
	if not json is Dictionary or json.get("format", "") != FORMAT_NAME:
		push_error("[GTM] %s is not a GodotTrench map" % map_path)
		return null
	var ctx := Context.new()
	ctx.map_settings = map_settings
	ctx.parse_data = parse_data
	ctx.map_path = map_path

	var world := _EntityData.new()
	world.properties["classname"] = "worldspawn"
	for key in json.get("properties", {}):
		world.properties[key] = str(json["properties"][key])
	world.properties["classname"] = "worldspawn"
	ctx.worldspawn = world
	parse_data.entities.append(world)

	for layer in json.get("layers", []):
		_parse_node(ctx, layer, null)
	return parse_data


static func _make_group(ctx: Context, node: Dictionary, parent: _GroupData, is_layer: bool) -> _GroupData:
	var group := _GroupData.new()
	group.id = int(node.get("id", ctx.next_group_id)) + ctx.instance_depth * 1000000
	ctx.next_group_id += 1
	group.type = _GroupData.GroupType.LAYER if is_layer else _GroupData.GroupType.GROUP
	var label: String = str(node.get("name", "")).replace(" ", "_")
	group.name = ("layer_" if is_layer else "group_") + str(group.id) + ("_" + label if label != "" else "")
	if parent:
		group.parent_id = parent.id
		group.parent = parent
	group.omit = bool(node.get("omit_from_export", false))
	ctx.parse_data.groups.append(group)
	return group


static func _parse_node(ctx: Context, node: Dictionary, group: _GroupData) -> void:
	match node.get("type", ""):
		"layer":
			if bool(node.get("omit_from_export", false)):
				return
			var g := _make_group(ctx, node, group, true)
			for child in node.get("children", []):
				_parse_node(ctx, child, g)
		"group":
			var g := _make_group(ctx, node, group, false)
			for child in node.get("children", []):
				_parse_node(ctx, child, g)
		"brush":
			var brush := _parse_brush(ctx, node)
			if brush:
				ctx.worldspawn.brushes.append(brush)
		"mesh":
			var mesh := _parse_mesh(ctx, node)
			if mesh:
				ctx.worldspawn.brushes.append(mesh)
		"terrain":
			# Terrains stay axis aligned: instances move them but do not rotate them.
			ctx.parse_data.terrains.append({ "data": node, "offset": ctx.xform.origin, "group": group, "id": int(node.get("id", 0)) + ctx.instance_depth * 1000000 })
		"scatter":
			ctx.parse_data.scatters.append({ "data": node, "xform": ctx.xform, "group": group, "id": int(node.get("id", 0)) + ctx.instance_depth * 1000000 })
		"entity":
			_parse_entity(ctx, node, group)
		"instance":
			_parse_instance(ctx, node, group)


static func _parse_entity(ctx: Context, node: Dictionary, group: _GroupData) -> void:
	var ent := _EntityData.new()
	var props: Dictionary = node.get("properties", {})
	for key in props:
		ent.properties[key] = str(props[key])
	ent.properties["classname"] = str(node.get("classname", ""))
	ent.group = group

	for key in ["targetname", "target"]:
		if ent.properties.has(key) and ctx.name_prefix != "" and not str(ent.properties[key]).begins_with("!"):
			ent.properties[key] = ctx.name_prefix + ent.properties[key]

	var children: Array = node.get("children", [])
	if children.is_empty():
		var origin: Vector3 = ctx.xform * vec3(node.get("origin"))
		var angles := vec3(node.get("angles"))
		if ctx.xform != Transform3D.IDENTITY:
			var basis := (ctx.xform.basis * rotation_basis(angles)).orthonormalized()
			var euler := basis.get_euler(EULER_ORDER_YXZ)
			angles = Vector3(rad_to_deg(euler.x), rad_to_deg(euler.y), rad_to_deg(euler.z))
		var id_origin := to_id(origin)
		ent.properties["origin"] = "%s %s %s" % [id_origin.x, id_origin.y, id_origin.z]
		if not ent.properties.has("angles") and not ent.properties.has("angle") and not ent.properties.has("mangle"):
			ent.properties["angles"] = angles_to_quake(angles)
	else:
		for child in children:
			match child.get("type", ""):
				"brush":
					var brush := _parse_brush(ctx, child)
					if brush:
						ent.brushes.append(brush)
				"mesh":
					var mesh := _parse_mesh(ctx, child)
					if mesh:
						ent.brushes.append(mesh)

	var outputs: Array = node.get("outputs", [])
	for o in outputs:
		var conn: Dictionary = o.duplicate()
		var target := str(conn.get("target", ""))
		if ctx.name_prefix != "" and target != "" and not target.begins_with("!"):
			conn["target"] = ctx.name_prefix + target
		ent.outputs.append(conn)
	ctx.parse_data.entities.append(ent)


static func _parse_brush(ctx: Context, node: Dictionary) -> _BrushData:
	var raw_vertices: Array = node.get("vertices", [])
	var faces: Array = node.get("faces", [])
	if raw_vertices.size() < 4 or faces.size() < 4:
		return null
	var scale := ctx.map_settings.scale_factor
	var vertices := PackedVector3Array()
	vertices.resize(raw_vertices.size())
	for i in raw_vertices.size():
		vertices[i] = ctx.xform * vec3(raw_vertices[i])

	var brush := _BrushData.new()
	brush.exact = true
	var origin_texture := ctx.map_settings.origin_texture
	brush.origin = true
	for f in faces:
		var indices: Array = f.get("indices", [])
		if indices.size() < 3:
			continue
		var godot_points := PackedVector3Array()
		for idx in indices:
			godot_points.append(vertices[int(idx)])

		# Newell normal is robust for any convex polygon, including slightly imprecise ones.
		var normal := Vector3.ZERO
		var centroid := Vector3.ZERO
		for i in godot_points.size():
			var cur := godot_points[i]
			var nxt := godot_points[(i + 1) % godot_points.size()]
			normal.x += (cur.y - nxt.y) * (cur.z + nxt.z)
			normal.y += (cur.z - nxt.z) * (cur.x + nxt.x)
			normal.z += (cur.x - nxt.x) * (cur.y + nxt.y)
			centroid += cur
		if normal.length_squared() < 1e-12:
			continue
		centroid /= godot_points.size()
		var id_normal := to_id(normal.normalized())
		var id_centroid := to_id(centroid) * scale
		var plane := Plane(id_normal, id_normal.dot(id_centroid))

		var face := _FaceData.new()
		face.plane = plane
		face.texture = str(f.get("material", ""))
		var blend_material := str(f.get("props", {}).get("blend_material", ""))
		if blend_material != "":
			face.texture = GodotTrenchBlend.key(face.texture, blend_material)
		for p in godot_points:
			face.exact_vertices.append(to_id(p) * scale)

		var uv: Dictionary = f.get("uv", {})
		var u_axis := vec3(uv.get("u_axis"), Vector3.RIGHT)
		var v_axis := vec3(uv.get("v_axis"), Vector3.BACK)
		var offset := vec2(uv.get("offset"), Vector2.ZERO)
		var uv_scale := vec2(uv.get("scale"), Vector2.ONE)
		if ctx.xform != Transform3D.IDENTITY:
			# Keep textures locked to instance geometry.
			var inv_t := ctx.xform.basis.inverse().transposed()
			var t := ctx.xform.origin
			var mu := inv_t * u_axis
			var mv := inv_t * v_axis
			var lu := maxf(mu.length(), 1e-9)
			var lv := maxf(mv.length(), 1e-9)
			offset.x -= mu.dot(t) / uv_scale.x
			offset.y -= mv.dot(t) / uv_scale.y
			uv_scale = Vector2(uv_scale.x / lu, uv_scale.y / lv)
			u_axis = mu / lu
			v_axis = mv / lv
		face.uv_axes.append(to_id(u_axis))
		face.uv_axes.append(to_id(v_axis))
		face.uv = Transform2D.IDENTITY
		face.uv.origin = offset
		face.uv.x = Vector2(uv_scale.x, 0.0) * scale
		face.uv.y = Vector2(0.0, uv_scale.y) * scale
		face.props = f.get("props", {})

		var colors: Array = f.get("colors", [])
		if colors.size() == godot_points.size():
			for c in colors:
				face.vertex_colors.append(Color(float(c[0]), float(c[1]), float(c[2]), float(c[3])))

		var disp = f.get("disp", null)
		if disp is Dictionary and godot_points.size() == 4:
			var power := int(disp.get("power", 3))
			var side := (1 << power) + 1
			var heights: Array = disp.get("heights", [])
			if heights.size() == side * side:
				var grid := GodotTrenchDisplacement.build_grid(godot_points, normal.normalized(), power, heights)
				for p in grid["positions"]:
					face.disp_vertices.append(to_id(p) * scale)
				for p in grid["base"]:
					face.disp_base.append(to_id(p) * scale)
				for nrm in grid["normals"]:
					face.disp_normals.append(to_id(nrm))
				face.disp_indices = grid["triangles"]
				for a in disp.get("alphas", []):
					face.disp_alphas.append(float(a))
				brush.has_disp = true

		if face.texture != origin_texture:
			brush.origin = false
		brush.planes.append(plane)
		brush.faces.append(face)
	if brush.faces.size() < 4:
		return null
	brush.node_id = int(node.get("id", 0)) + ctx.instance_depth * 1000000
	return brush


static func _parse_mesh(ctx: Context, node: Dictionary) -> _BrushData:
	var mesh := GodotTrenchMesh.parse(node, ctx.xform, ctx.map_settings.scale_factor, ctx.map_settings.origin_texture)
	if mesh:
		mesh.node_id = int(node.get("id", 0)) + ctx.instance_depth * 1000000
	return mesh


static func _resolve_instance_path(ctx: Context, path: String) -> String:
	if path.begins_with("res://") or path.begins_with("user://") or path.is_absolute_path():
		return path
	return ctx.map_path.get_base_dir().path_join(path)


static func _parse_instance(ctx: Context, node: Dictionary, group: _GroupData) -> void:
	if ctx.instance_depth >= MAX_INSTANCE_DEPTH:
		push_error("[GTM] instance nesting deeper than %d, skipping %s" % [MAX_INSTANCE_DEPTH, node.get("path", "")])
		return
	var path := _resolve_instance_path(ctx, str(node.get("path", "")))
	var text := FileAccess.get_file_as_string(path)
	if text.is_empty():
		push_error("[GTM] cannot read instance map %s" % path)
		return
	var json = JSON.parse_string(text)
	if not json is Dictionary or json.get("format", "") != FORMAT_NAME:
		push_error("[GTM] instance %s is not a GodotTrench map" % path)
		return

	var saved_xform := ctx.xform
	var saved_prefix := ctx.name_prefix
	var saved_path := ctx.map_path
	var local := Transform3D(rotation_basis(vec3(node.get("angles"))), vec3(node.get("origin")))
	ctx.xform = saved_xform * local
	var fixup := str(node.get("fixup", ""))
	if fixup != "":
		ctx.name_prefix = saved_prefix + fixup + "-"
	ctx.map_path = path
	ctx.instance_depth += 1

	for layer in json.get("layers", []):
		if bool(layer.get("omit_from_export", false)):
			continue
		for child in layer.get("children", []):
			_parse_node(ctx, child, group)

	ctx.instance_depth -= 1
	ctx.xform = saved_xform
	ctx.name_prefix = saved_prefix
	ctx.map_path = saved_path
