@tool
class_name GodotTrenchScatter extends Node3D
## Scatter set from a GodotTrench map: trees, rocks or foliage painted in the editor.
## Visuals are MultiMeshInstance3D nodes per model mesh. Props get one static body per instance sharing a collision
## shape, and scenes with scripts are instanced one by one so their behaviour is kept.

@export var kind := "props"
@export var collision := "convex"
@export var instance_count := 0

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
	var per_item: Array[Array] = []
	per_item.resize(items.size())
	for raw in instances:
		if raw is Array and raw.size() >= 8 and int(raw[0]) < items.size():
			per_item[int(raw[0])].append(instance_transform(raw, xform, scale))
	for k in items.size():
		var transforms: Array = per_item[k]
		if transforms.is_empty():
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
		if node.kind != "foliage" and _has_script(template):
			for t in transforms:
				var inst := scene.instantiate()
				inst.name = "%s_%d" % [item_name, node.get_child_count()]
				if inst is Node3D:
					(inst as Node3D).transform = t
				node.add_child(inst)
			template.free()
			continue
		for mi in _mesh_instances(template):
			var rel := _relative_transform(mi, template)
			var mm := MultiMesh.new()
			mm.transform_format = MultiMesh.TRANSFORM_3D
			mm.mesh = mi.mesh
			mm.instance_count = transforms.size()
			var buffer := PackedFloat32Array()
			buffer.resize(transforms.size() * 12)
			for i in transforms.size():
				var t: Transform3D = transforms[i] * rel
				var b := t.basis
				var o := i * 12
				buffer[o] = b.x.x; buffer[o + 1] = b.y.x; buffer[o + 2] = b.z.x; buffer[o + 3] = t.origin.x
				buffer[o + 4] = b.x.y; buffer[o + 5] = b.y.y; buffer[o + 6] = b.z.y; buffer[o + 7] = t.origin.y
				buffer[o + 8] = b.x.z; buffer[o + 9] = b.y.z; buffer[o + 10] = b.z.z; buffer[o + 11] = t.origin.z
			mm.buffer = buffer
			var mmi := MultiMeshInstance3D.new()
			mmi.name = "%s_%s" % [item_name, mi.name]
			mmi.multimesh = mm
			# The headless renderer drops MultiMesh buffers, so scenes built there keep a copy and restore it on load.
			if mm.buffer.size() != buffer.size():
				mmi.set_meta(BUFFER_META, buffer)
			mmi.material_override = mi.material_override
			mmi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_ON if shadows else GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
			if range_end > 0.0:
				mmi.visibility_range_end = range_end
				mmi.visibility_range_end_margin = range_end * 0.1
				mmi.visibility_range_fade_mode = GeometryInstance3D.VISIBILITY_RANGE_FADE_SELF
			node.add_child(mmi)
		if node.collision != "none":
			var shape := _shape_for(template, node.collision)
			if shape:
				var body := StaticBody3D.new()
				body.name = "%s_collision" % item_name
				node.add_child(body)
				for t in transforms:
					var cs := CollisionShape3D.new()
					cs.shape = shape
					cs.transform = t
					body.add_child(cs)
		template.free()
	return node
