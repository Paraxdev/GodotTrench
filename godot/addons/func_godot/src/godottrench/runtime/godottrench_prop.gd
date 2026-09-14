@tool
class_name GodotTrenchProp extends Node3D
## Prop entity that instantiates a model given by its "model" property (.bbmodel, .glb, .gltf or .tscn).
## "collision" can be none, convex or trimesh.

@export var model: String = ""
@export_enum("none", "convex", "trimesh") var collision: String = "convex"

func _func_godot_apply_properties(properties: Dictionary) -> void:
	model = str(properties.get("model", model))
	collision = str(properties.get("collision", collision))
	rebuild()

func rebuild() -> void:
	for child in get_children():
		if child.has_meta(&"gt_prop_generated"):
			remove_child(child)
			child.free()
	if model == "" or not ResourceLoader.exists(model):
		if model != "":
			push_warning("[GodotTrench] prop model %s not found" % model)
		return
	var scene := load(model) as PackedScene
	if not scene:
		return
	var flag := PackedScene.GEN_EDIT_STATE_INSTANCE if Engine.is_editor_hint() else PackedScene.GEN_EDIT_STATE_DISABLED
	var instance := scene.instantiate(flag)
	instance.set_meta(&"gt_prop_generated", true)
	add_child(instance)
	if owner:
		instance.owner = owner
	if collision == "none" or _has_collision(instance):
		return
	var body := StaticBody3D.new()
	body.name = "prop_collision"
	body.set_meta(&"gt_prop_generated", true)
	add_child(body)
	if owner:
		body.owner = owner
	for mi in _mesh_instances(instance):
		if not mi.mesh:
			continue
		var shape := CollisionShape3D.new()
		shape.shape = mi.mesh.create_convex_shape() if collision == "convex" else mi.mesh.create_trimesh_shape()
		shape.transform = global_transform.affine_inverse() * mi.global_transform if is_inside_tree() else _relative_transform(mi)
		body.add_child(shape)
		if owner:
			shape.owner = owner

func _relative_transform(node: Node3D) -> Transform3D:
	var t := Transform3D.IDENTITY
	var n: Node = node
	while n and n != self:
		if n is Node3D:
			t = (n as Node3D).transform * t
		n = n.get_parent()
	return t

func _has_collision(node: Node) -> bool:
	if node is CollisionObject3D:
		return true
	for c in node.get_children():
		if _has_collision(c):
			return true
	return false

func _mesh_instances(node: Node) -> Array[MeshInstance3D]:
	var out: Array[MeshInstance3D] = []
	var stack: Array[Node] = [node]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is MeshInstance3D:
			out.append(n)
		stack.append_array(n.get_children())
	return out
