@tool
class_name GodotTrenchProp extends Node3D
## Prop entity that instantiates a model given by its "model" property (.bbmodel, .glb, .gltf or .tscn).
## "collision" can be none, convex or trimesh. "model_node" keeps only the node of that name from the model, placed
## at the prop's origin, for files that hold several variants side by side.

@export var model: String = ""
@export var model_node: String = ""
@export_enum("none", "convex", "trimesh") var collision: String = "convex"

func _func_godot_apply_properties(properties: Dictionary) -> void:
	model = str(properties.get("model", model))
	model_node = str(properties.get("model_node", model_node)).strip_edges()
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
	var instance: Node = scene.instantiate(flag)
	if model_node != "":
		var picked := _pick_node(instance)
		if not picked:
			push_warning("[GodotTrench] prop model %s has no node %s" % [model, model_node])
			instance.free()
			return
		instance = picked
	instance.set_meta(&"gt_prop_generated", true)
	add_child(instance)
	if owner:
		instance.owner = owner
		if model_node != "":
			_own_subtree(instance)
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

## Takes the node named model_node out of the instanced model with its transform relative to the model root, minus the
## offset, and frees the rest.
func _pick_node(instance: Node) -> Node:
	var picked: Node = instance if instance.name == model_node else instance.find_child(model_node, true, false)
	if not picked:
		return null
	if picked == instance:
		if picked is Node3D:
			(picked as Node3D).position = Vector3.ZERO
		return picked
	var t := Transform3D.IDENTITY
	var n: Node = picked
	while n and n != instance:
		if n is Node3D:
			t = (n as Node3D).transform * t
		n = n.get_parent()
	picked.get_parent().remove_child(picked)
	instance.free()
	if picked is Node3D:
		t.origin = Vector3.ZERO
		(picked as Node3D).transform = t
	return picked

## Nodes of a picked subtree lost their scene owner, so a saved build would drop them.
func _own_subtree(node: Node) -> void:
	for c in node.get_children():
		c.owner = owner
		_own_subtree(c)

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
