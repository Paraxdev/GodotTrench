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
## Newest map version this importer understands, the same as FORMAT_VERSION in crates/gt_doc/src/format.rs.
const FORMAT_VERSION := 1
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


## Entity keys that name other entities and so get an instance's fixup, besides any property an entity definition
## declares as a target. Kept in step with FIXUP_KEYS in crates/gt_doc/src/map.rs.
const FIXUP_KEYS: Array[String] = ["targetname", "target", "destination", "call_target"]

class Context:
	var map_settings: FuncGodotMapSettings
	## Classname to the keys that get a fixup, filled on first use inside an instance.
	var fixup_keys: Dictionary = {}
	var entity_defs: Variant = null
	var parse_data: _ParseData
	var map_path: String
	var worldspawn: _EntityData
	var next_group_id := 1
	var instance_depth := 0
	## Id namespace of the instance occurrence currently being parsed, 0 at the top level. Every [method _parse_instance]
	## call gets its own value from [member next_instance_ns], so two copies of the same prefab never share ids, however
	## deep they are nested.
	var instance_ns := 0
	var next_instance_ns := 1
	## Transform applied to everything parsed at the current instance level (Godot space, map units).
	var xform := Transform3D.IDENTITY
	var name_prefix := ""
	## Brushes and meshes waiting to be converted, see [method GodotTrenchParser._convert_geometry].
	var geometry: Array[Dictionary] = []


static func parse(text: String, map_settings: FuncGodotMapSettings, parse_data: _ParseData, map_path: String = "") -> _ParseData:
	return parse_dict(JSON.parse_string(text), map_settings, parse_data, map_path)


## Like [method parse] for a map that is already parsed JSON, e.g. the live model of the GodotTrench editor.
static func parse_dict(json: Variant, map_settings: FuncGodotMapSettings, parse_data: _ParseData, map_path: String = "") -> _ParseData:
	if not _readable(json, map_path):
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
	world.node_id = 0
	ctx.worldspawn = world
	parse_data.entities.append(world)

	for layer in json.get("layers", []):
		if _is_layer(layer):
			_parse_node(ctx, layer, null)
	_convert_geometry(ctx)
	return parse_data


## Refuses what the editor refuses to open: another format, or a version newer than this importer knows.
static func _readable(json: Variant, path: String) -> bool:
	if not json is Dictionary or json.get("format", "") != FORMAT_NAME:
		push_error("[GTM] %s is not a GodotTrench map" % path)
		return false
	if int(json.get("version", 0)) > FORMAT_VERSION:
		push_error("[GTM] %s is map version %s, newer than this addon supports (%s), update the addon" % [path, json.get("version"), FORMAT_VERSION])
		return false
	return true

## The editor only keeps layers at the top of a map, so anything else there is ignored here too.
static func _is_layer(node: Variant) -> bool:
	return node is Dictionary and node.get("type", "") == "layer"

## Leaves a slot for a brush or mesh node in [param entity], filled by [method _convert_geometry].
static func _queue_geometry(ctx: Context, node: Dictionary, entity: _EntityData) -> void:
	entity.brushes.append(null)
	ctx.geometry.append({ "node": node, "xform": ctx.xform, "ns": ctx.instance_ns, "entity": entity, "slot": entity.brushes.size() - 1 })


## Converts the queued brushes and meshes, on worker threads for bigger maps. Only data is created there.
static func _convert_geometry(ctx: Context) -> void:
	var jobs := ctx.geometry
	var results := []
	results.resize(jobs.size())
	var convert := func(i: int) -> void:
		var job: Dictionary = jobs[i]
		var node: Dictionary = job["node"]
		var brush: _BrushData
		if node.get("type", "") == "mesh":
			brush = GodotTrenchMesh.parse(node, job["xform"], ctx.map_settings.scale_factor, ctx.map_settings.origin_texture)
		else:
			brush = _parse_brush(ctx.map_settings, job["xform"], node)
		if brush:
			brush.node_id = int(node.get("id", 0)) + int(job["ns"]) * 1000000
		results[i] = brush
	if GodotTrenchBuild.threaded() and jobs.size() >= 32:
		var task := WorkerThreadPool.add_group_task(convert, jobs.size(), -1, false, "Convert GodotTrench brushes")
		WorkerThreadPool.wait_for_group_task_completion(task)
	else:
		for i in jobs.size():
			convert.call(i)
	var touched := {}
	for i in jobs.size():
		var entity: _EntityData = jobs[i]["entity"]
		entity.brushes[jobs[i]["slot"]] = results[i]
		touched[entity] = true
	for entity: _EntityData in touched:
		for k in range(entity.brushes.size() - 1, -1, -1):
			if entity.brushes[k] == null:
				entity.brushes.remove_at(k)
	jobs.clear()


static func _make_group(ctx: Context, node: Dictionary, parent: _GroupData, is_layer: bool) -> _GroupData:
	var group := _GroupData.new()
	group.id = int(node.get("id", ctx.next_group_id)) + ctx.instance_ns * 1000000
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
		"brush", "mesh":
			_queue_geometry(ctx, node, ctx.worldspawn)
		"terrain":
			# Terrains stay axis aligned: an instance rotates the terrain's center about the instance origin, like
			# Terrain::transformed in the editor, but never the grid itself. GodotTrenchTerrain.create does the rotation.
			ctx.parse_data.terrains.append({ "data": node, "xform": ctx.xform, "group": group, "id": int(node.get("id", 0)) + ctx.instance_ns * 1000000 })
		"scatter":
			ctx.parse_data.scatters.append({ "data": node, "xform": ctx.xform, "group": group, "id": int(node.get("id", 0)) + ctx.instance_ns * 1000000 })
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
	ent.node_id = int(node.get("id", 0)) + ctx.instance_ns * 1000000

	if ctx.name_prefix != "":
		for key in _fixup_keys(ctx, ent.properties["classname"]):
			if ent.properties.has(key):
				ent.properties[key] = _fixup(ctx, ent.properties[key])

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
			if child.get("type", "") in ["brush", "mesh"]:
				_queue_geometry(ctx, child, ent)

	var outputs: Array = node.get("outputs", [])
	for o in outputs:
		var conn: Dictionary = o.duplicate()
		if conn.has("target"):
			conn["target"] = _fixup(ctx, str(conn["target"]))
		ent.outputs.append(conn)
	ctx.parse_data.entities.append(ent)


## Runs on worker threads, so it only creates data.
static func _parse_brush(map_settings: FuncGodotMapSettings, xform: Transform3D, node: Dictionary) -> _BrushData:
	var raw_vertices: Array = node.get("vertices", [])
	var faces: Array = node.get("faces", [])
	if raw_vertices.size() < 4 or faces.size() < 4:
		return null
	var scale := map_settings.scale_factor
	var vertices := PackedVector3Array()
	vertices.resize(raw_vertices.size())
	for i in raw_vertices.size():
		vertices[i] = xform * vec3(raw_vertices[i])

	var brush := _BrushData.new()
	brush.exact = true
	var origin_texture := map_settings.origin_texture
	brush.origin = true
	for f in faces:
		var indices: Array = f.get("indices", [])
		if indices.size() < 3:
			continue
		var godot_points := PackedVector3Array()
		for idx in indices:
			var i := int(idx)
			if i < 0 or i >= vertices.size():
				push_error("[GTM] brush %s has a face indexing vertex %d, out of range for %d vertices, skipping the brush" % [str(int(node["id"])) if node.has("id") else "?", i, vertices.size()])
				return null
			godot_points.append(vertices[i])

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
		var face_props: Dictionary = f.get("props", {})
		var blend_material := str(face_props.get("blend_material", ""))
		if blend_material != "":
			var detile := float(face_props.get("blend_detile", 0.0))
			var uv_scale := float(face_props.get("blend_uv_scale", 1.0))
			var sharpen := float(face_props.get("blend_detile_sharpen", 0.5))
			face.texture = GodotTrenchBlend.key(face.texture, blend_material, detile, uv_scale, sharpen)
		for p in godot_points:
			face.exact_vertices.append(to_id(p) * scale)

		var uv: Dictionary = f.get("uv", {})
		var u_axis := vec3(uv.get("u_axis"), Vector3.RIGHT)
		var v_axis := vec3(uv.get("v_axis"), Vector3.BACK)
		var offset := vec2(uv.get("offset"), Vector2.ZERO)
		var uv_scale := vec2(uv.get("scale"), Vector2.ONE)
		if xform != Transform3D.IDENTITY:
			# Keep textures locked to instance geometry.
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
	return brush


static func _resolve_instance_path(ctx: Context, path: String) -> String:
	if path.begins_with("res://") or path.begins_with("user://") or path.is_absolute_path():
		return path
	return ctx.map_path.get_base_dir().path_join(path)


## [param name] as named inside the instance being parsed. Special (!self), group (@doors) and node path targets are
## left alone, the same rule as Instance::fixup_name in the editor.
static func _fixup(ctx: Context, name: String) -> String:
	if ctx.name_prefix == "" or name == "" or name[0] in ["!", "@", "/"]:
		return name
	return ctx.name_prefix + name

static func _fixup_keys(ctx: Context, classname: String) -> Array[String]:
	if ctx.fixup_keys.has(classname):
		return ctx.fixup_keys[classname]
	if ctx.entity_defs == null:
		var fgd: FuncGodotFGDFile = ctx.map_settings.entity_fgd if ctx.map_settings else null
		ctx.entity_defs = fgd.get_entity_definitions() if fgd else {}
	var keys: Array[String] = FIXUP_KEYS.duplicate()
	var def = ctx.entity_defs.get(classname)
	if def is FuncGodotFGDEntityClass:
		var declared: Dictionary = def.meta_properties.get("property_types", {})
		for key in declared:
			if str(declared[key]) in ["target_source", "target_destination"] and not key in keys:
				keys.append(str(key))
	ctx.fixup_keys[classname] = keys
	return keys

static func _parse_instance(ctx: Context, node: Dictionary, group: _GroupData) -> void:
	if ctx.instance_depth >= MAX_INSTANCE_DEPTH:
		push_error("[GTM] instance nesting deeper than %d, skipping %s" % [MAX_INSTANCE_DEPTH, node.get("path", "")])
		return
	var path := _resolve_instance_path(ctx, str(node.get("path", "")))
	var json = GodotTrenchGtmFile.load_map(path)
	if json == null or not _readable(json, path):
		return

	var saved_xform := ctx.xform
	var saved_prefix := ctx.name_prefix
	var saved_path := ctx.map_path
	var saved_ns := ctx.instance_ns
	var local := Transform3D(rotation_basis(vec3(node.get("angles"))), vec3(node.get("origin")))
	ctx.xform = saved_xform * local
	var fixup := str(node.get("fixup", ""))
	if fixup != "":
		ctx.name_prefix = saved_prefix + fixup + "-"
	ctx.map_path = path
	ctx.instance_depth += 1
	# Every occurrence of an instance gets its own id namespace, so two copies of one prefab never collide, whichever
	# depth they nest at. Only the top level (namespace 0) keeps the ids the main map document was saved with.
	ctx.instance_ns = ctx.next_instance_ns
	ctx.next_instance_ns += 1

	for layer in json.get("layers", []):
		if not _is_layer(layer) or bool(layer.get("omit_from_export", false)):
			continue
		for child in layer.get("children", []):
			_parse_node(ctx, child, group)

	ctx.instance_depth -= 1
	ctx.instance_ns = saved_ns
	ctx.xform = saved_xform
	ctx.name_prefix = saved_prefix
	ctx.map_path = saved_path
