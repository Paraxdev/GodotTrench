class_name GTSectionStreamer extends Node3D
## Plays a chain of map sections as one world with no loading screens. Each section is a scene with a built map. Where
## two sections meet, both maps hold an identical connector room: an info_landmark of the same name and two
## func_section_stream halves, one naming each side. Only one section is ever in the tree. When the player crosses
## into the half that names another section, that section is placed so the two landmarks coincide and the old one is
## taken out in the same frame. The player never moves and the room around them is the same in both copies, so nothing
## on screen changes.
##
## Sections next door load on background threads and are instanced ahead of time while the player explores, so a swap
## only adds a finished scene to the tree. The section left behind is kept out of the tree until the next swap, which
## makes turning straight back instant. Use baked lighting in sections, SDFGI only notices geometry a swap brings in
## once its cascades scroll.
##
## Add it to your world scene, set [member first_section] and either [member player_scene] or [member player_path].

## A section replaced another.
signal section_changed(from: String, to: String)

## Seconds ambient sound takes to cross fade on a swap, so a section's hum does not cut out.
const AUDIO_FADE := 1.5
const SILENT_DB := -60.0

## The section the game starts in: a scene path, or a name found in [member sections_dir].
@export var first_section := ""
## Folder short section names are found in, as name.tscn or name.scn. Empty means sections are named by path.
@export_dir var sections_dir := ""
## The player to spawn at the first section's info_player_start.
@export var player_scene: PackedScene
## Or a player already in the scene. With neither, the first node in the "player" group a section brings is adopted.
@export var player_path: NodePath
## Meters from the origin a section may drift before the whole world moves back. Loops add up offsets, this keeps float
## precision. A move makes SDFGI and similar effects rebuild, so it is kept rare.
@export var recenter_distance := 8000.0

var current: Node3D
var current_name := ""
var player: Node3D
## Swaps done so far and how long the last one took in microseconds, for tests and debugging.
var swaps := 0
var last_swap_usec := 0

var _previous: Node3D
var _previous_name := ""
var _packed := {}
var _loading := {}
## Instances made ahead of time on worker threads: name -> { "task": id, "node": Node3D or null }.
var _ahead := {}
var _mutex := Mutex.new()
# Headless Godot's dummy renderer is not thread safe, meshes loaded on a thread corrupt it, so tools and tests load
# sections on the main thread.
var _threaded := DisplayServer.get_name() != "headless"

func _ready() -> void:
	if Engine.is_editor_hint() or first_section == "":
		return
	current = _instance_now(first_section)
	if current == null:
		return
	current_name = first_section
	add_child(current)
	if player_scene:
		player = player_scene.instantiate()
		add_child(player)
	elif not player_path.is_empty():
		player = get_node_or_null(player_path)
	else:
		player = get_tree().get_first_node_in_group(&"player") as Node3D
		if player and current.is_ancestor_of(player):
			player.reparent(self)
	var start := _find(current, func(n: Node) -> bool: return n is Node3D and String(n.name).contains("info_player_start")) as Node3D
	if player and start and (player_scene or not player_path.is_empty()):
		player.global_transform = start.global_transform
	_drop_extra_players(current)
	_prefetch()

## The scene path of [param section].
func section_path(section: String) -> String:
	if section.begins_with("res://") or section.begins_with("uid://") or sections_dir == "":
		return section
	for ext in ["tscn", "scn"]:
		var path := sections_dir.path_join("%s.%s" % [section, ext])
		if ResourceLoader.exists(path):
			return path
	return sections_dir.path_join(section + ".tscn")

func _physics_process(_delta: float) -> void:
	_poll()
	if player == null or current == null:
		return
	# One point decides which half the player is in. The body would straddle the swap line and sit in both halves,
	# swapping back and forth every frame.
	var query := PhysicsPointQueryParameters3D.new()
	query.position = player.global_position + Vector3.UP * 0.5
	query.collide_with_areas = true
	query.collide_with_bodies = false
	for hit in get_world_3d().direct_space_state.intersect_point(query, 8):
		var stream := hit["collider"] as GTSectionStream
		if stream == null or stream.section == "" or stream.section == current_name or not current.is_ancestor_of(stream):
			continue
		var next := _take(stream.section)
		# Not loaded yet, the player is still inside the half, so this runs again next frame.
		if next:
			_swap(stream, next)
		return

## Swaps to [param section] through the connector at [param landmark] at once, wherever the player is.
func force_swap(section: String, landmark: String) -> bool:
	for node in get_tree().get_nodes_in_group(&"gt_section_streams"):
		var stream := node as GTSectionStream
		if stream and stream.section == section and stream.landmark == landmark and current.is_ancestor_of(stream):
			var next := _take(section)
			if next == null:
				next = _instance_now(section)
			return next != null and _swap(stream, next)
	return false

func _swap(stream: GTSectionStream, next: Node3D) -> bool:
	var here := _landmark(current, stream.landmark)
	var there := _landmark(next, stream.landmark)
	if here == null or there == null:
		push_warning("GTSectionStreamer: section %s or %s has no info_landmark %s" % [current_name, stream.section, stream.landmark])
		return false
	var started := Time.get_ticks_usec()
	var old := current
	var old_name := current_name
	next.transform = (global_transform.affine_inverse() * here.global_transform) * _relative(there, next).affine_inverse()
	_fade_out(old)
	remove_child(old)
	add_child(next)
	current = next
	current_name = stream.section
	if _previous and _previous != next:
		_previous.queue_free()
	_previous = old
	_previous_name = old_name
	_drop_extra_players(next)
	_fade_in(next)
	swaps += 1
	last_swap_usec = Time.get_ticks_usec() - started
	_prefetch()
	_recenter()
	section_changed.emit(old_name, current_name)
	return true

## A ready instance of [param section], or null while it is still loading.
func _take(section: String) -> Node3D:
	if section == _previous_name and _previous:
		var back := _previous
		_previous = null
		_previous_name = ""
		return back
	_mutex.lock()
	var ahead: Dictionary = _ahead.get(section, {})
	var node: Node3D = ahead.get("node")
	_mutex.unlock()
	if node:
		WorkerThreadPool.wait_for_task_completion(ahead["task"])
		_ahead.erase(section)
		return node
	if _packed.has(section) and not _ahead.has(section):
		return _instance_now(section)
	_request(section)
	return null

func _instance_now(section: String) -> Node3D:
	var packed: PackedScene = _packed.get(section)
	if packed == null:
		packed = load(section_path(section)) as PackedScene
		if packed == null:
			push_warning("GTSectionStreamer: no section scene %s" % section_path(section))
			return null
		_packed[section] = packed
	return packed.instantiate() as Node3D

func _request(section: String) -> void:
	if _packed.has(section) or _loading.has(section):
		return
	var path := section_path(section)
	if not ResourceLoader.exists(path):
		push_warning("GTSectionStreamer: no section scene %s" % path)
		return
	if not _threaded:
		_packed[section] = load(path)
		_instance_ahead(section)
		return
	ResourceLoader.load_threaded_request(path, "PackedScene", true)
	_loading[section] = path

func _poll() -> void:
	for section: String in _loading.keys():
		var path: String = _loading[section]
		var status := ResourceLoader.load_threaded_get_status(path)
		if status == ResourceLoader.THREAD_LOAD_IN_PROGRESS:
			continue
		_loading.erase(section)
		if status == ResourceLoader.THREAD_LOAD_LOADED:
			_packed[section] = ResourceLoader.load_threaded_get(path)
			_instance_ahead(section)
		else:
			push_warning("GTSectionStreamer: loading section %s failed" % path)

## Instances [param section] on a worker thread, so taking it later costs only adding it to the tree.
func _instance_ahead(section: String) -> void:
	if _ahead.has(section) or section == current_name or section == _previous_name:
		return
	var packed: PackedScene = _packed[section]
	var entry := { "node": null }
	_ahead[section] = entry
	if not _threaded:
		entry["node"] = packed.instantiate()
		entry["task"] = WorkerThreadPool.add_task(func() -> void: pass)
		return
	entry["task"] = WorkerThreadPool.add_task(func() -> void:
		var node := packed.instantiate() as Node3D
		_mutex.lock()
		entry["node"] = node
		_mutex.unlock())

## Starts loading every section a connector of the current one leads to, and drops what is no longer next door.
func _prefetch() -> void:
	var near := _neighbours()
	for section in near:
		if _packed.has(section):
			_instance_ahead(section)
		else:
			_request(section)
	for section: String in _ahead.keys():
		if section in near:
			continue
		var entry: Dictionary = _ahead[section]
		WorkerThreadPool.wait_for_task_completion(entry["task"])
		var node: Node = entry["node"]
		if node:
			node.free()
		_ahead.erase(section)
	for section: String in _packed.keys():
		if section != current_name and section != _previous_name and not section in near:
			_packed.erase(section)

func _neighbours() -> Array[String]:
	var out: Array[String] = []
	for node in get_tree().get_nodes_in_group(&"gt_section_streams"):
		var stream := node as GTSectionStream
		if stream and current.is_ancestor_of(stream) and stream.section != current_name and stream.section != "" and not stream.section in out:
			out.append(stream.section)
	return out

## Finishes what is still loading or being instanced, so nothing runs on a thread after the streamer is gone.
func _exit_tree() -> void:
	for section: String in _loading.keys():
		ResourceLoader.load_threaded_get(_loading[section])
	_loading.clear()
	for section: String in _ahead.keys():
		WorkerThreadPool.wait_for_task_completion(_ahead[section]["task"])
		var node: Node = _ahead[section]["node"]
		if node:
			node.free()
	_ahead.clear()
	if _previous:
		_previous.free()
		_previous = null

## A section built with a player of its own, like an info_player_start that spawns one, brings a second player.
func _drop_extra_players(section: Node) -> void:
	for node in get_tree().get_nodes_in_group(&"player"):
		if node != player and section.is_ancestor_of(node):
			node.queue_free()

func _landmark(section: Node, landmark: String) -> Node3D:
	return _find(section, func(n: Node) -> bool: return n is GTLandmark and (n as GTLandmark).targetname == landmark) as Node3D

## Transform of [param node] relative to [param root], which need not be in the tree.
static func _relative(node: Node3D, root: Node3D) -> Transform3D:
	var t := Transform3D.IDENTITY
	var n: Node = node
	while n and n != root:
		if n is Node3D:
			t = (n as Node3D).transform * t
		n = n.get_parent()
	return t

static func _find(root: Node, match_node: Callable) -> Node:
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if match_node.call(n):
			return n
		stack.append_array(n.get_children())
	return null

static func _audio(root: Node) -> Array[Node]:
	var out: Array[Node] = []
	var stack: Array[Node] = [root]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is AudioStreamPlayer3D or n is AudioStreamPlayer:
			out.append(n)
		stack.append_array(n.get_children())
	return out

## Leaves a fading copy of every sound [param section] is playing, and remembers them so they resume if the section
## comes back.
func _fade_out(section: Node) -> void:
	var playing: Array = []
	for p in _audio(section):
		if not p.playing:
			continue
		var at: float = p.get_playback_position()
		playing.append([p, at])
		var copy: Node = p.duplicate(0)
		add_child(copy)
		if copy is Node3D:
			(copy as Node3D).global_transform = (p as Node3D).global_transform
		copy.play(at)
		var tween := copy.create_tween()
		tween.tween_property(copy, ^"volume_db", SILENT_DB, AUDIO_FADE)
		tween.tween_callback(copy.queue_free)
	section.set_meta(&"gt_streamer_playing", playing)

## Fades in what [param section] plays: ambient players usually start in _ready, a section coming back resumes the
## sounds it was playing when it left.
func _fade_in(section: Node) -> void:
	if section.has_meta(&"gt_streamer_playing"):
		for entry: Array in section.get_meta(&"gt_streamer_playing"):
			if is_instance_valid(entry[0]):
				entry[0].play(entry[1])
		section.remove_meta(&"gt_streamer_playing")
	# Players made in _ready exist only after this frame's ready pass.
	_fade_in_players.call_deferred(section)

func _fade_in_players(section: Node) -> void:
	if not is_instance_valid(section) or section != current:
		return
	for p in _audio(section):
		var target: float = p.volume_db
		p.volume_db = SILENT_DB
		p.create_tween().tween_property(p, ^"volume_db", target, AUDIO_FADE)

## Moves the section and the player back toward the origin when the section has drifted too far.
func _recenter() -> void:
	var offset := current.position
	if offset.length() < recenter_distance:
		return
	offset.y = 0.0
	for child in get_children():
		if child is Node3D:
			(child as Node3D).position -= offset
