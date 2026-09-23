@tool
class_name GodotTrenchStreamer extends Node3D
## Invisible chunk loader and render culler for a built GodotTrench map.
##
## It buckets the map's generated visuals into a grid and shows only the chunks near a camera, so a large map
## costs what the player can see rather than what the map contains. It has no visuals of its own and touches
## nothing but [member Node3D.visible] on visual nodes, so scripts, physics and gameplay keep running.
##
## Only geometry the map build generated is streamed, overlays are left alone. Visuals created at runtime (game_text and logic_debug
## labels, spawned scenes) and everything under a moving body (func_train, func_door, npc_walker, prop_physics)
## are left alone, since their chunk would be wrong as soon as they move. A visual that gameplay hid stays hidden
## when its chunk comes back into range.
##
## Set it up by giving worldspawn the key [code]chunk_streaming[/code] (plus optional [code]chunk_size[/code]
## and [code]load_radius[/code] in map units) and the map build adds one. It follows the viewport's current
## camera by default, so it works with your own player, and falls back to an [code]info_player_start[/code]
## while no camera exists. Point [member camera_path] at a camera to pin it to one.
##
## Culling only runs in the running game, never while editing, so the editor keeps showing the whole map.
##
## [signal area_loaded] and [signal area_unloaded] report chunks coming and going, so game code can follow the
## player without tracking distances itself. From GDScript:
## [codeblock]
## streamer.area_loaded.connect(func(key, bounds): spawn_wildlife(bounds))
## [/codeblock]
## and from C#:
## [codeblock]
## streamer.Connect("area_loaded", Callable.From((Vector3I key, Aabb bounds) => SpawnWildlife(bounds)));
## [/codeblock]

## A chunk came into range. [param key] is its cell on the streaming grid, [param bounds] the box it covers in
## world space. Streaming starts from nothing drawn, so the first frame reports every chunk already in range.
signal area_loaded(key: Vector3i, bounds: AABB)

## A chunk went out of range. Its meshes stop drawing, its collision and scripts keep running.
signal area_unloaded(key: Vector3i, bounds: AABB)

## Meters per chunk cell. Smaller cells cull more tightly but cost more chunks to check.
@export var chunk_size := 64.0
## Meters from the camera a chunk stays visible within.
@export var load_radius := 192.0
## Extra meters a visible chunk keeps before it is hidden again, so chunks do not flicker on the boundary.
@export var unload_margin := 16.0
## Camera that drives the streaming. Empty follows the viewport's current camera.
@export var camera_path: NodePath
@export var enabled := true

## One grid cell: the visuals inside it and the box they cover.
class Chunk:
	var key: Vector3i
	var bounds: AABB
	var nodes: Array[Node3D] = []
	var shown := true
	## Nodes this streamer hid, so coming back into range only restores those and not ones gameplay hid.
	var hidden: Array[Node3D] = []

	func set_shown(value: bool) -> void:
		shown = value
		if value:
			for node in hidden:
				if is_instance_valid(node):
					node.visible = true
			hidden.clear()
			return
		for node in nodes:
			if is_instance_valid(node) and node.visible:
				node.visible = false
				hidden.append(node)

var _chunks: Array[Chunk] = []
var _fallback := Vector3.INF
## Camera position the last scan ran from, so a barely moving camera does not rescan every frame.
var _last_eye := Vector3.INF

func _ready() -> void:
	rebuild()

## How many chunks the map was split into, for tooling and tests.
func chunk_count() -> int:
	return _chunks.size()

## Visuals the streamer manages in total, for tooling and tests.
func node_count() -> int:
	var n := 0
	for chunk in _chunks:
		n += chunk.nodes.size()
	return n

## Visuals currently shown, for tooling and tests.
func visible_count() -> int:
	var n := 0
	for chunk in _chunks:
		if chunk.shown:
			n += chunk.nodes.size()
	return n

## True when the chunk covering [param point] is in range and drawing. Points outside every chunk, such as open
## sky, report false. Collision and scripts run whatever this says, it only reports what is drawn.
func is_area_loaded(point: Vector3) -> bool:
	for chunk in _chunks:
		if chunk.bounds.has_point(point):
			return chunk.shown
	return false

## The chunks in range right now, as their grid keys.
func loaded_areas() -> Array[Vector3i]:
	var out: Array[Vector3i] = []
	for chunk in _chunks:
		if chunk.shown:
			out.append(chunk.key)
	return out

func _visual_aabb(node: Node3D) -> AABB:
	var vi := node as VisualInstance3D
	var local := vi.get_aabb()
	var box := AABB(node.global_transform * local.position, Vector3.ZERO)
	for i in 8:
		box = box.expand(node.global_transform * local.get_endpoint(i))
	return box

## Every build generated visual below [param node], descending through plain containers so a scatter set's
## chunks and a terrain's chunks are streamed one by one rather than all at once.
func _collect(node: Node, out: Array[Node3D]) -> void:
	for child in node.get_children():
		if child == self or child is Light3D or child is WorldEnvironment or is_moving_body(child) or GodotTrenchOverlay.is_kept(child):
			continue
		if child is VisualInstance3D:
			# Nodes without an owner were added at runtime, by entity scripts or game code, not by the build.
			if child.owner != null:
				out.append(child)
			continue
		if child is Node3D:
			_collect(child, out)

## Bodies that move at runtime, their visuals travel with them and cannot be bucketed by where they started.
static func is_moving_body(node: Node) -> bool:
	return node is AnimatableBody3D or node is RigidBody3D or node is CharacterBody3D or node is PhysicalBone3D

## Rereads the map's visuals and regroups them. Called on load, and again whenever the map is rebuilt.
func rebuild() -> void:
	for chunk in _chunks:
		chunk.set_shown(true)
	_chunks.clear()
	_fallback = Vector3.INF
	_last_eye = Vector3.INF
	var parent := get_parent()
	if not parent:
		set_process(false)
		return
	for node in parent.get_children():
		if node is Marker3D and String(node.name).contains("player_start"):
			_fallback = (node as Marker3D).global_position
			break

	var visuals: Array[Node3D] = []
	_collect(parent, visuals)
	var cell := maxf(chunk_size, 0.001)
	var buckets: Dictionary = {}
	for node in visuals:
		var box := _visual_aabb(node)
		var center := box.get_center()
		var key := Vector3i(floori(center.x / cell), floori(center.y / cell), floori(center.z / cell))
		var chunk: Chunk = buckets.get(key)
		if not chunk:
			chunk = Chunk.new()
			chunk.key = key
			chunk.bounds = box
			buckets[key] = chunk
		else:
			chunk.bounds = chunk.bounds.merge(box)
		chunk.nodes.append(node)
	_chunks.assign(buckets.values())
	# Editing shows the whole map, only the running game streams.
	var streaming := enabled and not _chunks.is_empty() and not Engine.is_editor_hint()
	set_process(streaming)
	if streaming:
		# Start from nothing drawn, so the first frame reports everything it brings in through area_loaded and a
		# listener connected before then sees the whole sequence.
		for chunk in _chunks:
			chunk.set_shown(false)

func _viewer() -> Vector3:
	if not camera_path.is_empty():
		var pinned := get_node_or_null(camera_path) as Camera3D
		if pinned:
			return pinned.global_position
	var camera := get_viewport().get_camera_3d() if is_inside_tree() else null
	if camera:
		return camera.global_position
	return _fallback

## Shortest distance from [param point] to the box, 0 inside it.
static func _distance_to(box: AABB, point: Vector3) -> float:
	var end := box.position + box.size
	var nearest := Vector3(clampf(point.x, box.position.x, end.x), clampf(point.y, box.position.y, end.y), clampf(point.z, box.position.z, end.z))
	return point.distance_to(nearest)

func _process(_delta: float) -> void:
	if not enabled:
		return
	var eye := _viewer()
	if not eye.is_finite():
		return
	# A chunk only changes state when the camera crosses its boundary, and the hysteresis above gives
	# unload_margin of slack, so rescanning before the camera has drifted a fraction of that margin cannot
	# change any answer. Standing still in a crowded area therefore costs one vector compare per frame, and
	# the scan itself never runs more often than the player actually moves.
	var step := maxf(unload_margin * 0.25, 0.5)
	if _last_eye.is_finite() and eye.distance_squared_to(_last_eye) < step * step:
		return
	_last_eye = eye
	var show_within := maxf(load_radius, 0.0)
	var hide_beyond := show_within + maxf(unload_margin, 0.0)
	for chunk in _chunks:
		var distance := _distance_to(chunk.bounds, eye)
		# Hysteresis: a chunk has to fall past the margin before it is hidden again, so chunks sitting on the
		# boundary do not flicker as the camera drifts.
		var shown := distance <= (hide_beyond if chunk.shown else show_within)
		if shown == chunk.shown:
			continue
		chunk.set_shown(shown)
		if shown:
			area_loaded.emit(chunk.key, chunk.bounds)
		else:
			area_unloaded.emit(chunk.key, chunk.bounds)

## Vertex attributes carried over when a mesh is split. Tangents are four floats per vertex and are handled
## separately, everything here is one entry per vertex.
const SPLIT_ARRAYS := [Mesh.ARRAY_VERTEX, Mesh.ARRAY_NORMAL, Mesh.ARRAY_COLOR, Mesh.ARRAY_TEX_UV, Mesh.ARRAY_TEX_UV2]

## Empty arrays of the same types as [param src], so the pieces keep the surface's vertex format.
static func _blank_like(src: Array) -> Array:
	var out := []
	out.resize(Mesh.ARRAY_MAX)
	for a in SPLIT_ARRAYS:
		if src[a] != null:
			out[a] = src[a].duplicate()
			out[a].clear()
	if src[Mesh.ARRAY_TANGENT] != null:
		out[Mesh.ARRAY_TANGENT] = PackedFloat32Array()
	out[Mesh.ARRAY_INDEX] = PackedInt32Array()
	return out

## Splits [param mesh] into one mesh per [param cell] sized cube, by the cell each triangle's centroid falls in.
## Vertices are reindexed per piece, so the pieces together hold about as much data as the original.
static func split_mesh(mesh: ArrayMesh, cell: float) -> Dictionary:
	var pieces: Dictionary = {}
	var surfaces := mesh.get_surface_count()
	for s in surfaces:
		var src := mesh.surface_get_arrays(s)
		var verts: PackedVector3Array = src[Mesh.ARRAY_VERTEX]
		var indices: PackedInt32Array = src[Mesh.ARRAY_INDEX]
		if indices == null or indices.is_empty():
			indices = PackedInt32Array(range(verts.size()))
		var tangents = src[Mesh.ARRAY_TANGENT]
		var i := 0
		while i + 2 < indices.size():
			var tri := [indices[i], indices[i + 1], indices[i + 2]]
			var centroid: Vector3 = (verts[tri[0]] + verts[tri[1]] + verts[tri[2]]) / 3.0
			var key := Vector3i(floori(centroid.x / cell), floori(centroid.y / cell), floori(centroid.z / cell))
			if not pieces.has(key):
				var slots := []
				slots.resize(surfaces)
				pieces[key] = slots
			var bucket: Array = pieces[key]
			if bucket[s] == null:
				bucket[s] = { "arrays": _blank_like(src), "remap": {} }
			var arrays: Array = bucket[s]["arrays"]
			var remap: Dictionary = bucket[s]["remap"]
			for old: int in tri:
				var mapped: int = remap.get(old, -1)
				if mapped < 0:
					mapped = arrays[Mesh.ARRAY_VERTEX].size()
					remap[old] = mapped
					for a in SPLIT_ARRAYS:
						if src[a] != null:
							arrays[a].append(src[a][old])
					if tangents != null:
						for k in 4:
							arrays[Mesh.ARRAY_TANGENT].append(tangents[old * 4 + k])
				arrays[Mesh.ARRAY_INDEX].append(mapped)
			i += 3

	var out: Dictionary = {}
	for key: Vector3i in pieces:
		var piece := ArrayMesh.new()
		var bucket: Array = pieces[key]
		for s in surfaces:
			if bucket[s] == null:
				continue
			piece.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, bucket[s]["arrays"])
			piece.surface_set_material(piece.get_surface_count() - 1, mesh.surface_get_material(s))
		if piece.get_surface_count() > 0:
			out[key] = piece
	return out

## Replaces meshes that span more than one cell with one MeshInstance3D per cell, so the renderer and the
## streamer can drop them one at a time. Worldspawn is built as a single merged mesh, which would otherwise be
## one chunk covering the whole map. Collision is left alone, only what is drawn is split.
static func split_visuals(root: Node, cell: float) -> int:
	if cell <= 0.0:
		return 0
	var oversized: Array[MeshInstance3D] = []
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n != root and GodotTrenchOverlay.is_kept(n):
			continue
		var mi := n as MeshInstance3D
		if mi and mi.mesh and mi.mesh is ArrayMesh:
			var size := mi.mesh.get_aabb().size
			if maxf(size.x, maxf(size.y, size.z)) > cell:
				oversized.append(mi)
		stack.append_array(n.get_children())

	var made := 0
	for mi in oversized:
		var pieces := split_mesh(mi.mesh, cell)
		if pieces.size() < 2:
			continue
		var parent := mi.get_parent()
		var base := String(mi.name)
		var index := 0
		for key: Vector3i in pieces:
			var piece := MeshInstance3D.new()
			piece.name = "%s_%d" % [base, index]
			piece.mesh = pieces[key]
			piece.transform = mi.transform
			piece.gi_mode = mi.gi_mode
			piece.cast_shadow = mi.cast_shadow
			piece.layers = mi.layers
			piece.material_override = mi.material_override
			parent.add_child(piece)
			piece.owner = mi.owner
			index += 1
			made += 1
		parent.remove_child(mi)
		mi.queue_free()
	return made

## Adds a streamer to [param map_node] when worldspawn asks for one. Returns it, or null.
static func build(map_node: Node3D, properties: Dictionary, settings: FuncGodotMapSettings) -> GodotTrenchStreamer:
	if str(properties.get("chunk_streaming", "0")) in ["0", "", "false"]:
		return null
	var scale := settings.scale_factor
	var node := GodotTrenchStreamer.new()
	node.name = "streamer"
	node.chunk_size = float(properties.get("chunk_size", 2048.0)) * scale
	node.load_radius = float(properties.get("load_radius", 8192.0)) * scale
	node.unload_margin = node.chunk_size * 0.25
	map_node.add_child(node)
	node.owner = GodotTrenchBuild.scene_owner(map_node)
	split_visuals(map_node, node.chunk_size)
	return node
