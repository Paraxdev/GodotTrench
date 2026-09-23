class_name GodotTrenchBuild extends RefCounted
## Shared helpers for building GodotTrench maps: node ownership and the map node ids kept on generated nodes.

## Map node id of a generated entity, terrain or scatter node.
const ID_META := &"_gt_id"
## Map node id of a generated layer or group node.
const GROUP_META := &"_gt_group"
## What the node was last fully built from: the hash of the map text for a build from text or a JSON file, the END
## chunk content id for a binary file, see [method GodotTrenchGtmFile.content_id].
const SOURCE_HASH_META := &"_gt_source_hash"
## Project setting: build steps that only compute data run on the WorkerThreadPool.
const SETTING_THREADED := "godottrench/threaded_build"

static func threaded() -> bool:
	return bool(ProjectSettings.get_setting(SETTING_THREADED, true))

## Owner for generated nodes: the edited scene when the map is part of it, so the nodes are saved with the scene.
static func scene_owner(map_node: Node) -> Node:
	if map_node.is_inside_tree():
		var edited := map_node.get_tree().edited_scene_root
		if edited and (edited == map_node or edited.is_ancestor_of(map_node)):
			return edited
	return map_node.owner if map_node.owner else map_node

## Sets [param owner_node] on [param node] and its generated children, leaving the insides of instanced scenes alone.
static func set_owner_recursive(node: Node, owner_node: Node) -> void:
	if node != owner_node:
		node.owner = owner_node
	if node.scene_file_path != "" and node != owner_node:
		return
	for child in node.get_children():
		set_owner_recursive(child, owner_node)

## Generated nodes below [param map_node] by map node id, for entities, terrains and scatter sets. Overlays are
## skipped, so a generated node copied into one is never taken for the original.
static func nodes_by_id(map_node: Node) -> Dictionary:
	var out := {}
	var stack: Array[Node] = [map_node]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n != map_node and GodotTrenchOverlay.is_kept(n):
			continue
		if n != map_node and n.has_meta(ID_META):
			out[int(n.get_meta(ID_META))] = n
			continue
		stack.append_array(n.get_children())
	return out

## Generated layer and group nodes below [param map_node] by map node id.
static func groups_by_id(map_node: Node) -> Dictionary:
	var out := {}
	var stack: Array[Node] = [map_node]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n != map_node and GodotTrenchOverlay.is_kept(n):
			continue
		if n.has_meta(GROUP_META):
			out[int(n.get_meta(GROUP_META))] = n
		if n == map_node or n.has_meta(GROUP_META):
			stack.append_array(n.get_children())
	return out
