class_name GodotTrenchLiveSession extends RefCounted
## Live edits from the GodotTrench editor for one map. Keeps the map JSON the editor last sent, applies its node level
## deltas and rebuilds only the generated nodes they touch. Saving in GodotTrench still does a full build.
##
## Ops: [code]set {id, parent, index, node}[/code], [code]remove {id}[/code], [code]translate {ids, offset}[/code] and
## [code]properties {properties}[/code]. [code]node[/code] is the node as stored in the .gtm file, without children.
##
## Loose brushes are merged into one worldspawn mesh by a full build. The first time a session changes one, the
## worldspawn is split into chunks of [constant SETTING_CHUNK_SIZE] meters, so later changes rebuild only the chunks
## they touch, and a dragged brush gets a node of its own until the drag ends. Hidden coplanar faces are worked out
## again for the rebuilt chunks together, which can leave an extra visible face next to chunks that were not rebuilt,
## never a hole. The next full build merges everything again.

const SETTING_CHUNK_SIZE := "godottrench/live_chunk_size"
const DEFAULT_CHUNK_SIZE := 16.0
## Whole map rebuilds (prefab instances, layer omission) wait until edits pause this long.
const IDLE_MSEC := 300
## Entity, terrain and scatter rebuilds stop for the frame after this long and continue next frame.
const FRAME_BUDGET_MSEC := 12
const WORLD_NODE := "_gt_live_world"
const REST_CHUNK := Vector3i(-2147483648, 0, 0)

## Above zero while a session runs a full build, so those builds do not count as builds from outside.
static var building := 0

var path: String
var maps: Array[FuncGodotMap] = []
var model: Dictionary = {}

var _nodes: Dictionary = {}
var _parents: Dictionary = {}
var _groups: Dictionary = {}
var _entities: Dictionary = {}
var _terrains: Dictionary = {}
var _scatters: Dictionary = {}
var _environment := false
var _full := false
var _last_change := 0

## Loose brush id to its bounds before the change (null for new brushes), not yet rebuilt.
var _loose: Dictionary = {}
## Loose brushes moving with translate ops, drawn by their own node.
var _dragged: Dictionary = {}
var _drag_nodes_needed: Dictionary = {}
## Chunk layout, shared by every map of the session: brush id to cell, cell to its brush ids and bounds.
var _cells: Dictionary = {}
var _members: Dictionary = {}
var _cell_bounds: Dictionary = {}
var _chunked: Dictionary = {}

func _init(map_path: String, map_nodes: Array[FuncGodotMap]) -> void:
	path = map_path
	maps = map_nodes

## Takes the whole map. Maps whose scene was built from other content are built from it right away. [param content] is
## the content id the editor computed for the map, matching the one a scene built from the saved binary file keeps.
func begin(text: String, content: Variant = null) -> bool:
	var json = JSON.parse_string(text)
	if not json is Dictionary or json.get("format", "") != GodotTrenchParser.FORMAT_NAME:
		return false
	model = json
	_reindex()
	_clear_pending()
	var text_hash := text.hash()
	for map in _live_maps():
		var built := int(map.get_meta(GodotTrenchBuild.SOURCE_HASH_META, 0))
		if built != text_hash and (content == null or built != int(content)):
			_full_build(map, text)
	return true

## Applies ops to the model and queues the rebuilds they need. False when they do not fit the model, the editor then
## starts over with [method begin].
func apply(ops: Array) -> bool:
	for op in ops:
		if not op is Dictionary:
			return false
		match str(op.get("op", "")):
			"set":
				if not _set_node(op):
					return false
			"remove":
				_remove_node(int(op.get("id", -1)))
			"translate":
				_translate(op.get("ids", []), GodotTrenchParser.vec3(op.get("offset")))
			"properties":
				model["properties"] = op.get("properties", {})
				_environment = true
			_:
				return false
	_last_change = Time.get_ticks_msec()
	_mark_scene_unsaved()
	return true

func pending() -> bool:
	return _full or _environment or not (_groups.is_empty() and _entities.is_empty() and _terrains.is_empty() and _scatters.is_empty() and _loose.is_empty() and _drag_nodes_needed.is_empty())

## Runs queued rebuilds, called every frame.
func process() -> void:
	if not pending():
		return
	var live := _live_maps()
	if _full:
		if Time.get_ticks_msec() - _last_change < IDLE_MSEC:
			return
		_clear_pending()
		var map_text := text()
		for map in live:
			_full_build(map, map_text)
		_mark_scene_unsaved()
		return
	var started := Time.get_ticks_msec()
	for id in _groups.keys():
		for map in live:
			_sync_group(map, id)
		_groups.erase(id)
	if _environment:
		_environment = false
		for map in live:
			_rebuild_environment(map)
	if not _loose.is_empty() or not _drag_nodes_needed.is_empty():
		_update_world(live)
	var generated := {}
	for map in live:
		generated[map] = GodotTrenchBuild.nodes_by_id(map)
	var queues := [_entities, _terrains, _scatters]
	for kind in queues.size():
		var queue: Dictionary = queues[kind]
		for id in queue.keys():
			if Time.get_ticks_msec() - started > FRAME_BUDGET_MSEC:
				return
			for map in live:
				var old: Node = generated[map].get(id)
				match kind:
					0: _rebuild_entity(map, id, old)
					1: _rebuild_terrain(map, id, old)
					2: _rebuild_scatter(map, id, old)
			queue.erase(id)

## The model as map text, for a full build.
func text() -> String:
	return JSON.stringify(model, "", false, true)

#region model

func _clear_pending() -> void:
	_groups.clear()
	_entities.clear()
	_terrains.clear()
	_scatters.clear()
	_environment = false
	_full = false
	_loose.clear()
	_dragged.clear()
	_drag_nodes_needed.clear()
	_cells.clear()
	_members.clear()
	_cell_bounds.clear()
	_chunked.clear()

func _live_maps() -> Array[FuncGodotMap]:
	var out: Array[FuncGodotMap] = []
	for map in maps:
		if is_instance_valid(map):
			out.append(map)
	return out

func _reindex() -> void:
	_nodes.clear()
	_parents.clear()
	for layer in model.get("layers", []):
		_index(layer, -1)

func _index(node: Dictionary, parent: int) -> void:
	var id := int(node.get("id", 0))
	_nodes[id] = node
	_parents[id] = parent
	for child in node.get("children", []):
		_index(child, id)

func _unindex(node: Dictionary) -> void:
	_nodes.erase(int(node.get("id", 0)))
	_parents.erase(int(node.get("id", 0)))
	for child in node.get("children", []):
		_unindex(child)

func _siblings(parent: int) -> Array:
	if parent < 0:
		if not model.has("layers"):
			model["layers"] = []
		return model["layers"]
	var node: Dictionary = _nodes[parent]
	if not node.has("children"):
		node["children"] = []
	return node["children"]

func _detach(id: int) -> void:
	var siblings := _siblings(int(_parents.get(id, -1)))
	for i in siblings.size():
		if int(siblings[i].get("id", -1)) == id:
			siblings.remove_at(i)
			return

func _type(id: int) -> String:
	return str(_nodes.get(id, {}).get("type", ""))

## A brush or mesh that the full build merges into the worldspawn.
func _is_loose(id: int) -> bool:
	return _type(id) in ["brush", "mesh"] and _type(int(_parents.get(id, -1))) != "entity" and not _omitted(id)

func _set_node(op: Dictionary) -> bool:
	var id := int(op.get("id", -1))
	var node = op.get("node")
	if not node is Dictionary:
		return false
	var parent := -1 if op.get("parent") == null else int(op["parent"])
	if parent >= 0 and not _nodes.has(parent):
		return false
	var old: Dictionary = _nodes.get(id, {})
	if not old.is_empty():
		_mark(id, old)
		node["children"] = old.get("children", [])
		_detach(id)
	var siblings := _siblings(parent)
	siblings.insert(clampi(int(op.get("index", siblings.size())), 0, siblings.size()), node)
	_nodes[id] = node
	_parents[id] = parent
	_dragged.erase(id)
	_mark(id, old)
	return true

func _remove_node(id: int) -> void:
	if not _nodes.has(id):
		return
	_mark_subtree(id)
	_detach(id)
	_unindex(_nodes[id])

func _mark_subtree(id: int) -> void:
	_mark(id, _nodes[id])
	_dragged.erase(id)
	for child in _nodes[id].get("children", []):
		_mark_subtree(int(child.get("id", 0)))

## Queues what a change of node [param id] rebuilds. [param old] is the node before the change, empty for new nodes.
func _mark(id: int, old: Dictionary) -> void:
	var node: Dictionary = _nodes.get(id, {})
	match str(node.get("type", "")):
		"layer":
			if not old.is_empty() and bool(old.get("omit_from_export", false)) != bool(node.get("omit_from_export", false)):
				_full = true
			_groups[id] = true
		"group":
			_groups[id] = true
		"instance":
			_full = true
		"entity":
			_entities[id] = true
		"brush", "mesh":
			var parent := int(_parents.get(id, -1))
			if _type(parent) == "entity":
				_entities[parent] = true
			elif not _loose.has(id):
				_loose[id] = null if old.is_empty() else _bounds(old)
		"terrain":
			_terrains[id] = true
		"scatter":
			_scatters[id] = true

func _translate(ids: Array, offset: Vector3) -> void:
	var moved := {}
	var brushes_by_entity := {}
	var dragged_now: Array[int] = []
	for raw in ids:
		var id := int(raw)
		var node: Dictionary = _nodes.get(id, {})
		if node.is_empty():
			continue
		var before := _bounds(node)
		_offset_node(node, offset)
		match str(node.get("type", "")):
			"entity", "scatter", "terrain":
				moved[id] = true
			"brush", "mesh":
				var parent := int(_parents.get(id, -1))
				if _type(parent) == "entity":
					brushes_by_entity[parent] = int(brushes_by_entity.get(parent, 0)) + 1
				elif _dragged.has(id):
					dragged_now.append(id)
				else:
					_dragged[id] = true
					_drag_nodes_needed[id] = true
					if not _loose.has(id):
						_loose[id] = before
			_:
				_full = true
	for entity in brushes_by_entity:
		var solids: Array = _nodes[entity].get("children", []).filter(func(c): return str(c.get("type", "")) in ["brush", "mesh"])
		if brushes_by_entity[entity] == solids.size():
			moved[entity] = true
		else:
			_entities[entity] = true
	for map in _live_maps():
		var by_id := GodotTrenchBuild.nodes_by_id(map)
		for id in moved:
			var generated: Node3D = by_id.get(id) as Node3D
			if generated:
				_move(generated, offset * map.map_settings.scale_factor)
			else:
				_mark(id, {})
		var world := map.get_node_or_null(WORLD_NODE)
		for id in dragged_now:
			var drag_node: Node3D = world.get_node_or_null(_drag_name(id)) as Node3D if world else null
			if drag_node:
				_move(drag_node, offset * map.map_settings.scale_factor)
			else:
				_drag_nodes_needed[id] = true

static func _move(node: Node3D, offset: Vector3) -> void:
	# Animatable bodies synced to physics only take a new position on the next physics step.
	var body := node as AnimatableBody3D
	var synced := body != null and body.sync_to_physics
	if synced:
		body.sync_to_physics = false
	node.position += offset
	if synced:
		body.sync_to_physics = true
	if node is GodotTrenchTerrain:
		_update_terrain_offset(node)

static func _offset_point(value: Variant, offset: Vector3) -> Array:
	var p := GodotTrenchParser.vec3(value) + offset
	return [p.x, p.y, p.z]

func _offset_node(node: Dictionary, offset: Vector3) -> void:
	match str(node.get("type", "")):
		"entity", "instance", "terrain":
			node["origin"] = _offset_point(node.get("origin"), offset)
		"brush", "mesh":
			var vertices: Array = node.get("vertices", [])
			for i in vertices.size():
				vertices[i] = _offset_point(vertices[i], offset)
		"scatter":
			for inst in node.get("instances", []):
				if inst is Array and inst.size() >= 4:
					inst[1] = float(inst[1]) + offset.x
					inst[2] = float(inst[2]) + offset.y
					inst[3] = float(inst[3]) + offset.z

func _omitted(id: int) -> bool:
	var n := id
	while n >= 0:
		if _type(n) == "layer" and bool(_nodes[n].get("omit_from_export", false)):
			return true
		n = int(_parents.get(n, -1))
	return false

## Bounds of a brush or mesh node in map units.
static func _bounds(node: Dictionary) -> AABB:
	var vertices: Array = node.get("vertices", [])
	if vertices.is_empty():
		return AABB()
	var box := AABB(GodotTrenchParser.vec3(vertices[0]), Vector3.ZERO)
	for v in vertices:
		box = box.expand(GodotTrenchParser.vec3(v))
	return box

#endregion

#region building

func _mark_scene_unsaved() -> void:
	if Engine.is_editor_hint():
		Engine.get_singleton(&"EditorInterface").mark_scene_as_unsaved()

func _full_build(map: FuncGodotMap, map_text: String) -> void:
	building += 1
	map.build_from_text(map_text)
	building -= 1

static func _free(node: Node) -> void:
	if node and is_instance_valid(node):
		if node.get_parent():
			node.get_parent().remove_child(node)
		node.queue_free()

## The generated node children of [param id] go under: its nearest layer or group node, or the map.
func _parent_node(map: FuncGodotMap, id: int) -> Node:
	if not map.map_settings.use_groups_hierarchy:
		return map
	var groups := GodotTrenchBuild.groups_by_id(map)
	var n := int(_parents.get(id, -1))
	while n >= 0:
		if groups.has(n):
			return groups[n]
		n = int(_parents.get(n, -1))
	return map

func _sync_group(map: FuncGodotMap, id: int) -> void:
	if not map.map_settings.use_groups_hierarchy:
		return
	var existing: Node = GodotTrenchBuild.groups_by_id(map).get(id)
	var node: Dictionary = _nodes.get(id, {})
	if node.is_empty() or _omitted(id):
		_free(existing)
		return
	var label := str(node.get("name", "")).replace(" ", "_")
	var group_name := ("layer_" if node.get("type") == "layer" else "group_") + str(id) + ("_" + label if label != "" else "")
	var parent := _parent_node(map, id)
	if not existing:
		existing = Node3D.new()
		existing.set_meta(GodotTrenchBuild.GROUP_META, id)
		parent.add_child(existing)
		existing.owner = GodotTrenchBuild.scene_owner(map)
	elif existing.get_parent() != parent:
		existing.reparent(parent, false)
		GodotTrenchBuild.set_owner_recursive(existing, GodotTrenchBuild.scene_owner(map))
	existing.name = group_name

func _map_with(children: Array) -> Dictionary:
	return { "format": GodotTrenchParser.FORMAT_NAME, "properties": model.get("properties", {}), "layers": [{ "type": "layer", "id": 0, "name": "", "children": children }] }

func _generate(map: FuncGodotMap, entities: Array[FuncGodotData.EntityData]) -> FuncGodotEntityAssembler:
	FuncGodotGeometryGenerator.new(map.map_settings, map.hyperplane_size).build(map.build_flags, entities)
	var assembler := FuncGodotEntityAssembler.new(map.map_settings)
	assembler.build_flags = map.build_flags
	return assembler

## Swaps the generated node of an entity for a new one built from [param data], keeping the name of unnamed entities.
## Rebuilt nodes are always added last. Godot 4.7's Scene dock breaks its item cache when a new node shows up in front
## of existing siblings, and then logs "Index p_pos is out of bounds" for later inserts under that parent.
func _replace_entity(map: FuncGodotMap, old: Node, data: FuncGodotData.EntityData, parent: Node, name_index: int) -> Node:
	var list: Array[FuncGodotData.EntityData] = [data]
	var assembler := _generate(map, list)
	var fresh := assembler.generate_entity_node(data, name_index)
	var unnamed := fresh != null and String(fresh.name) == "entity_%d_%s" % [name_index, data.properties.get("classname", "")]
	var old_name := String(old.name) if old else ""
	_free(old)
	var owner := GodotTrenchBuild.scene_owner(map)
	var attached := assembler.attach_entity(fresh, data, parent, owner)
	if not attached:
		return null
	if unnamed and old_name != "":
		attached.name = old_name
	GodotTrenchIO.setup(list, owner)
	GodotTrenchIO.invalidate(map)
	return attached

func _rebuild_entity(map: FuncGodotMap, id: int, old: Node) -> void:
	var node: Dictionary = _nodes.get(id, {})
	if node.is_empty() or node.get("type") != "entity" or _omitted(id):
		_free(old)
		return
	var parse_data := FuncGodotParser.new().parse_gtm(_map_with([node]), map.map_settings, path)
	if parse_data.entities.size() < 2:
		_free(old)
		return
	_replace_entity(map, old, parse_data.entities[1], _parent_node(map, id), id)

func _rebuild_terrain(map: FuncGodotMap, id: int, old: Node) -> void:
	_free(old)
	var node: Dictionary = _nodes.get(id, {})
	if node.get("type") != "terrain" or _omitted(id):
		return
	GodotTrenchTerrain.build_one(map, _parent_node(map, id), node, Vector3.ZERO, id, map.map_settings)

func _rebuild_scatter(map: FuncGodotMap, id: int, old: Node) -> void:
	_free(old)
	var node: Dictionary = _nodes.get(id, {})
	if node.get("type") != "scatter" or _omitted(id):
		return
	GodotTrenchScatter.build_one(map, _parent_node(map, id), node, Transform3D.IDENTITY, id, map.map_settings)

func _rebuild_environment(map: FuncGodotMap) -> void:
	for child in map.get_children():
		if GodotTrenchOverlay.is_kept(child):
			continue
		if (child is WorldEnvironment and child.name == "environment") or (child is DirectionalLight3D and child.name == "sun"):
			_free(child)
	GodotTrenchEnvironment.build(map, model.get("properties", {}))

## Terrain textures are projected in map space, so the shader follows a moved terrain.
static func _update_terrain_offset(terrain: GodotTrenchTerrain) -> void:
	for child in terrain.get_children():
		var mesh_instance := child as MeshInstance3D
		if mesh_instance and mesh_instance.mesh:
			var material := mesh_instance.mesh.surface_get_material(0) as ShaderMaterial
			if material:
				material.set_shader_parameter("map_offset", terrain.position)

#endregion

#region chunked worldspawn

func _chunk_units() -> float:
	var settings: FuncGodotMapSettings = maps[0].map_settings if not maps.is_empty() and is_instance_valid(maps[0]) else null
	var scale := settings.scale_factor if settings else 1.0 / 32.0
	return float(ProjectSettings.get_setting(SETTING_CHUNK_SIZE, DEFAULT_CHUNK_SIZE)) / maxf(scale, 1e-9)

func _cell_for(box: AABB) -> Vector3i:
	return Vector3i((box.get_center() / _chunk_units()).floor())

static func _drag_name(id: int) -> String:
	return "_gt_live_brush_%d" % id

## Brush nodes of the model the worldspawn is made of, prefab instances excluded.
func _loose_ids() -> Array[int]:
	var out: Array[int] = []
	for id in _nodes:
		if _is_loose(id):
			out.append(id)
	return out

## Instance nodes outside omitted layers, their brushes form the chunk that is only rebuilt by a full build.
func _instance_nodes() -> Array:
	var out := []
	for id in _nodes:
		if _type(id) == "instance" and not _omitted(id):
			out.append(_nodes[id])
	return out

func _place(id: int) -> void:
	var cell := _cell_for(_bounds(_nodes[id]))
	_cells[id] = cell
	if not _members.has(cell):
		_members[cell] = {}
	_members[cell][id] = true

func _unplace(id: int) -> void:
	if not _cells.has(id):
		return
	var cell: Vector3i = _cells[id]
	_cells.erase(id)
	if _members.has(cell):
		_members[cell].erase(id)

func _layout() -> void:
	_cells.clear()
	_members.clear()
	for id in _loose_ids():
		if not _dragged.has(id):
			_place(id)

func _update_cell_bounds(cell: Vector3i) -> void:
	var box := AABB()
	var first := true
	for id in _members.get(cell, {}):
		var b := _bounds(_nodes[id])
		box = b if first else box.merge(b)
		first = false
	if first:
		_cell_bounds.erase(cell)
	else:
		_cell_bounds[cell] = box

func _world_node(map: FuncGodotMap) -> Node3D:
	var world := map.get_node_or_null(WORLD_NODE) as Node3D
	if world:
		return world
	_free(GodotTrenchBuild.nodes_by_id(map).get(0))
	world = Node3D.new()
	world.name = WORLD_NODE
	world.set_meta(GodotTrenchBuild.ID_META, 0)
	map.add_child(world)
	world.owner = GodotTrenchBuild.scene_owner(map)
	return world

## Worldspawn entities for [param cells], hidden faces worked out across all of them.
func _chunk_entities(map: FuncGodotMap, cells: Array) -> Dictionary:
	var out := {}
	var list: Array[FuncGodotData.EntityData] = []
	for cell in cells:
		var children := []
		if cell == REST_CHUNK:
			children = _instance_nodes()
		else:
			for id in _members.get(cell, {}):
				children.append(_nodes[id])
		if children.is_empty():
			out[cell] = null
			continue
		var parse_data := FuncGodotParser.new().parse_gtm(_map_with(children), map.map_settings, path)
		var world: FuncGodotData.EntityData = parse_data.entities[0] if not parse_data.entities.is_empty() else null
		if world and world.brushes.is_empty():
			world = null
		out[cell] = world
		if world:
			list.append(world)
	if not list.is_empty():
		_generate(map, list)
	return out

func _chunk_name(cell: Vector3i) -> String:
	return "rest" if cell == REST_CHUNK else "chunk_%d_%d_%d" % [cell.x, cell.y, cell.z]

func _attach_world_part(map: FuncGodotMap, world: Node3D, data: FuncGodotData.EntityData, node_name: String) -> void:
	_free(world.get_node_or_null(node_name))
	if not data:
		return
	var assembler := FuncGodotEntityAssembler.new(map.map_settings)
	assembler.build_flags = map.build_flags
	var node := assembler.attach_entity(assembler.generate_entity_node(data, 0), data, world, GodotTrenchBuild.scene_owner(map))
	if node:
		node.remove_meta(GodotTrenchBuild.ID_META)
		node.name = node_name

## Splits the worldspawn of [param map] into chunks built from the current model.
func _chunk_world(map: FuncGodotMap) -> void:
	var world := _world_node(map)
	for child in world.get_children():
		_free(child)
	var cells: Array = _members.keys()
	cells.append(REST_CHUNK)
	var built := _chunk_entities(map, cells)
	for cell in built:
		_attach_world_part(map, world, built[cell], _chunk_name(cell))
	_chunked[map] = true

func _update_world(live: Array[FuncGodotMap]) -> void:
	if _cells.is_empty() and _members.is_empty():
		_layout()
		for cell in _members:
			_update_cell_bounds(cell)
	var dirty := {}
	for id in _loose:
		var touched: Array[AABB] = []
		if _loose[id] is AABB:
			touched.append(_loose[id])
		if _cells.has(id):
			dirty[_cells[id]] = true
		_unplace(id)
		if _nodes.has(id) and _is_loose(id):
			touched.append(_bounds(_nodes[id]))
			if not _dragged.has(id):
				_place(id)
				dirty[_cells[id]] = true
		for box in touched:
			var grown := box.grow(GodotTrenchFaceCull.COPLANAR_DIST * 2.0)
			for cell in _cell_bounds:
				if _cell_bounds[cell].grow(GodotTrenchFaceCull.COPLANAR_DIST * 2.0).intersects(grown):
					dirty[cell] = true
	_loose.clear()
	for cell in dirty:
		_update_cell_bounds(cell)
	for map in live:
		var world := _world_node(map)
		if _chunked.has(map) and not dirty.is_empty():
			var built := _chunk_entities(map, dirty.keys())
			for cell in built:
				_attach_world_part(map, world, built[cell], _chunk_name(cell))
		if not _chunked.has(map):
			_chunk_world(map)
		for child in world.get_children():
			var child_name := String(child.name)
			if child_name.begins_with("_gt_live_brush_") and not _dragged.has(int(child_name.trim_prefix("_gt_live_brush_"))):
				_free(child)
		for id in _drag_nodes_needed:
			if _dragged.has(id) and _nodes.has(id):
				var parse_data := FuncGodotParser.new().parse_gtm(_map_with([_nodes[id]]), map.map_settings, path)
				var data: FuncGodotData.EntityData = parse_data.entities[0]
				var list: Array[FuncGodotData.EntityData] = [data]
				_generate(map, list)
				_attach_world_part(map, world, data, _drag_name(id))
	_drag_nodes_needed.clear()

#endregion
