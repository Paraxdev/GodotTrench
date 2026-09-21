@tool
class_name GodotTrenchScatter extends Node3D
## Scatter set from a GodotTrench map: trees, rocks or foliage painted in the editor.
## Visuals are MultiMeshInstance3D nodes per model mesh. Props get one static body per instance sharing a collision
## shape, and scenes with scripts are instanced one by one so their behaviour is kept.

@export var kind := "props"
@export var collision := "convex"
@export var instance_count := 0
## Map units per chunk cell the instances were split into, 0 when the set is one MultiMesh per mesh.
@export var chunk_size := 0.0

const BUFFER_META := &"gt_transforms"

func _ready() -> void:
	restore_buffers()

## Reapplies instance transforms kept in metadata when the MultiMesh buffer was lost.
func restore_buffers() -> void:
	for child in get_children():
		var mmi := child as MultiMeshInstance3D
		if not mmi or not mmi.multimesh or not mmi.has_meta(BUFFER_META):
			continue
		var buffer: PackedFloat32Array = mmi.get_meta(BUFFER_META)
		if mmi.multimesh.buffer.size() != buffer.size():
			mmi.multimesh.buffer = buffer

static func build_all(map_node: Node3D, scatters: Array[Dictionary], settings: FuncGodotMapSettings) -> Array[GodotTrenchScatter]:
	var out: Array[GodotTrenchScatter] = []
	for entry in scatters:
		var group = entry.get("group", null)
		var parent: Node = map_node
		if settings.use_groups_hierarchy and group and group.node:
			parent = group.node
		var node := build_one(map_node, parent, entry["data"], entry.get("xform", Transform3D.IDENTITY), int(entry.get("id", out.size())), settings)
		if node:
			out.append(node)
	return out

## Creates one scatter set node named after its map node id under [param parent], owned like the other generated nodes.
static func build_one(map_node: Node, parent: Node, data: Dictionary, xform: Transform3D, id: int, settings: FuncGodotMapSettings) -> GodotTrenchScatter:
	var node := create(data, xform, settings)
	if not node:
		return null
	node.name = ("scatter_%d_%s" % [id, str(data.get("name", ""))]).validate_node_name()
	node.set_meta(GodotTrenchBuild.ID_META, id)
	parent.add_child(node)
	var scene_root := GodotTrenchBuild.scene_owner(map_node)
	node.owner = scene_root
	_set_owner(node, scene_root)
	return node

static func _set_owner(node: Node, owner_node: Node) -> void:
	for child in node.get_children():
		if child.owner == null:
			child.owner = owner_node
		# Instanced scenes keep their own internal owners.
		if child.scene_file_path == "":
			_set_owner(child, owner_node)

static func _relative_transform(node: Node, root: Node) -> Transform3D:
	var t := Transform3D.IDENTITY
	var n := node
	while n and n != root:
		if n is Node3D:
			t = (n as Node3D).transform * t
		n = n.get_parent()
	return t

static func _mesh_instances(root: Node) -> Array[MeshInstance3D]:
	var out: Array[MeshInstance3D] = []
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is MeshInstance3D and (n as MeshInstance3D).mesh:
			out.append(n)
		stack.append_array(n.get_children())
	return out

static func _has_script(root: Node) -> bool:
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n.get_script() != null:
			return true
		stack.append_array(n.get_children())
	return false

static func _shape_for(template: Node, mode: String) -> Shape3D:
	var faces := PackedVector3Array()
	for mi in _mesh_instances(template):
		var rel := _relative_transform(mi, template)
		for v in mi.mesh.get_faces():
			faces.append(rel * v)
	var shape: Shape3D = null
	if faces.size() >= 3:
		if mode == "trimesh":
			var concave := ConcavePolygonShape3D.new()
			concave.set_faces(faces)
			shape = concave
		else:
			var convex := ConvexPolygonShape3D.new()
			convex.points = faces
			shape = convex
	return shape

## One instance transform in the parent's space (meters).
static func instance_transform(raw: Array, xform: Transform3D, scale_factor: float) -> Transform3D:
	var pos := Vector3(float(raw[1]), float(raw[2]), float(raw[3]))
	var angles := Vector3(deg_to_rad(float(raw[4])), deg_to_rad(float(raw[5])), deg_to_rad(float(raw[6])))
	var basis := xform.basis * Basis.from_euler(angles, EULER_ORDER_YXZ).scaled(Vector3.ONE * float(raw[7]))
	return Transform3D(basis, (xform * pos) * scale_factor)

## Grid cell an instance falls into, in map units, matching Scatter::chunks() in the editor.
static func _cell(raw: Array, chunk_size: float) -> Vector3i:
	if chunk_size <= 0.0:
		return Vector3i.ZERO
	return Vector3i(floori(float(raw[1]) / chunk_size), floori(float(raw[2]) / chunk_size), floori(float(raw[3]) / chunk_size))

## Cells in a fixed order, so a set always builds the same way.
static func _sorted_cells(buckets: Dictionary) -> Array:
	var cells := buckets.keys()
	cells.sort_custom(func(a: Vector3i, b: Vector3i) -> bool:
		if a.x != b.x:
			return a.x < b.x
		if a.y != b.y:
			return a.y < b.y
		return a.z < b.z)
	return cells

## Material a palette entry draws with: its own override, else the set's, else the model's own materials.
static func _item_material(data: Dictionary, item: Dictionary, settings: FuncGodotMapSettings) -> Material:
	var name := str(item.get("material", "")) if item.get("material", "") != null else ""
	if name == "":
		name = str(data.get("material", "")) if data.get("material", "") != null else ""
	if name == "":
		return null
	var dir: String = settings.base_material_dir if settings.base_material_dir != "" else settings.base_texture_dir
	var path := dir.path_join(name + "." + settings.material_file_extension)
	if ResourceLoader.exists(path):
		return load(path) as Material
	# No material resource for the name, fall back to its plain texture.
	var texture := FuncGodotUtil.load_texture(name, [], settings)
	if not texture:
		push_warning("[GodotTrench] scatter material %s not found" % name)
		return null
	var material := StandardMaterial3D.new()
	material.albedo_texture = texture
	return material

## One MultiMeshInstance3D for the transforms of a single mesh in a single chunk.
static func _chunk_instance(mi: MeshInstance3D, rel: Transform3D, transforms: Array, name: String, shadows: bool, range_end: float) -> MultiMeshInstance3D:
	var mm := MultiMesh.new()
	mm.transform_format = MultiMesh.TRANSFORM_3D
	mm.mesh = mi.mesh
	mm.instance_count = transforms.size()
	var buffer := PackedFloat32Array()
	buffer.resize(transforms.size() * 12)
	var box := AABB()
	var mesh_box := mi.mesh.get_aabb()
	for i in transforms.size():
		var t: Transform3D = transforms[i] * rel
		var b := t.basis
		var o := i * 12
		buffer[o] = b.x.x; buffer[o + 1] = b.y.x; buffer[o + 2] = b.z.x; buffer[o + 3] = t.origin.x
		buffer[o + 4] = b.x.y; buffer[o + 5] = b.y.y; buffer[o + 6] = b.z.y; buffer[o + 7] = t.origin.y
		buffer[o + 8] = b.x.z; buffer[o + 9] = b.y.z; buffer[o + 10] = b.z.z; buffer[o + 11] = t.origin.z
		box = (t * mesh_box) if i == 0 else box.merge(t * mesh_box)
	mm.buffer = buffer
	var mmi := MultiMeshInstance3D.new()
	mmi.name = name
	mmi.multimesh = mm
	# The headless renderer drops MultiMesh buffers, so scenes built there keep a copy and restore it on load.
	if mm.buffer.size() != buffer.size():
		mmi.set_meta(BUFFER_META, buffer)
	# Without an explicit box a chunk whose buffer was dropped reports an empty AABB and is culled away.
	mmi.custom_aabb = box
	mmi.material_override = mi.material_override
	mmi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_ON if shadows else GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	if range_end > 0.0:
		mmi.visibility_range_end = range_end
		mmi.visibility_range_end_margin = range_end * 0.1
		mmi.visibility_range_fade_mode = GeometryInstance3D.VISIBILITY_RANGE_FADE_SELF
	return mmi

static func create(data: Dictionary, xform: Transform3D, settings: FuncGodotMapSettings) -> GodotTrenchScatter:
	var node := GodotTrenchScatter.new()
	node.kind = str(data.get("kind", "props"))
	node.collision = str(data.get("collision", "none" if node.kind == "foliage" else "convex"))
	var scale := settings.scale_factor
	var items: Array = data.get("items", [])
	var instances: Array = data.get("instances", [])
	node.instance_count = instances.size()
	var shadows := bool(data.get("cast_shadows", true))
	var range_end := float(data.get("visibility_range", 0.0)) * scale
	node.chunk_size = float(data.get("chunk_size", 0.0))
	var props_as_multimesh := bool(data.get("static_props_multimesh", false))
	# Per palette entry, the instance transforms grouped by chunk cell, so each cell becomes a MultiMesh with its
	# own bounds that Godot frustum culls on its own instead of one set spanning the whole map.
	var per_item: Array[Dictionary] = []
	per_item.resize(items.size())
	for k in items.size():
		per_item[k] = {}
	for raw in instances:
		if raw is Array and raw.size() >= 8 and int(raw[0]) < items.size():
			var cell := _cell(raw, node.chunk_size)
			var buckets: Dictionary = per_item[int(raw[0])]
			if not buckets.has(cell):
				buckets[cell] = []
			buckets[cell].append(instance_transform(raw, xform, scale))
	for k in items.size():
		var buckets: Dictionary = per_item[k]
		if buckets.is_empty():
			continue
		var source := str(items[k].get("source", ""))
		if source == "" or not ResourceLoader.exists(source):
			push_warning("[GodotTrench] scatter model %s not found" % source)
			continue
		var scene := load(source) as PackedScene
		if not scene:
			continue
		var template := scene.instantiate()
		var item_name := source.get_file().get_basename().validate_node_name()
		var override_material := _item_material(data, items[k], settings)
		var cells := _sorted_cells(buckets)
		var all: Array[Transform3D] = []
		for cell: Vector3i in cells:
			all.append_array(buckets[cell])
		if node.kind != "foliage" and not props_as_multimesh and _has_script(template):
			for t in all:
				var inst := scene.instantiate()
				inst.name = "%s_%d" % [item_name, node.get_child_count()]
				if inst is Node3D:
					(inst as Node3D).transform = t
				if override_material:
					for mi in _mesh_instances(inst):
						mi.material_override = override_material
				node.add_child(inst)
			template.free()
			continue
		for mi in _mesh_instances(template):
			var rel := _relative_transform(mi, template)
			for cell: Vector3i in cells:
				var chunk_name := "%s_%s" % [item_name, mi.name]
				if node.chunk_size > 0.0:
					chunk_name = "%s_%d_%d_%d" % [chunk_name, cell.x, cell.y, cell.z]
				var chunk := _chunk_instance(mi, rel, buckets[cell], chunk_name.validate_node_name(), shadows, range_end)
				if override_material:
					chunk.material_override = override_material
				node.add_child(chunk)
		if node.collision != "none":
			var shape := _shape_for(template, node.collision)
			if shape:
				var body := StaticBody3D.new()
				body.name = "%s_collision" % item_name
				node.add_child(body)
				for t in all:
					var cs := CollisionShape3D.new()
					cs.shape = shape
					cs.transform = t
					body.add_child(cs)
		template.free()
	return node
