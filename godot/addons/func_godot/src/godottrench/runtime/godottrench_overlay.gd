@tool
@icon("res://addons/func_godot/icons/icon_overlay3d.svg")
class_name GodotTrenchOverlay extends Node3D
## Godot content that shares a map's space and survives every rebuild of it.
##
## Add it as a direct child of a [FuncGodotMap] and put anything below it: props, decor, lights, particles, scenes
## with scripts. Building the map, Clear Map, live edits from GodotTrench and hot reload in a running game leave it
## where it is, with the same node instances, and it is saved with your scene like any other node.
##
## Content is authored in meters in the map's local space, the space the generated nodes use, so with the default
## [member FuncGodotMapSettings.inverse_scale_factor] of 32 a child at (2, 0, -3) sits at (64, 0, -96) in GodotTrench.
## The overlay itself always sits at the map origin: as a child its transform stays at identity, and with
## [member map_path] it copies the map's global transform.
##
## Overlay nodes join the map's entity I/O: give one a [GodotTrenchOverlayIO] to make it a target, add
## [GodotTrenchOutput] children to fire at map entities, and use a [GodotTrenchAnchor] to keep content on an entity
## that moves.
##
## A single node can also survive rebuilds without an overlay: add it to the [code]godottrench_keep[/code] group and
## keep it a direct child of the map.

## Persistent group for single nodes directly under a map that the build keeps.
const KEEP_GROUP := &"godottrench_keep"
## Runtime group of every overlay in the tree, for overlays that point at their map with [member map_path].
const GROUP := &"_gt_overlays"
const SIDECAR_FORMAT := "godottrench-overlay"
const SIDECAR_VERSION := 1

## The map this overlay belongs to when it is not a child of one. Empty uses the parent.
@export var map_path: NodePath:
	set(value):
		map_path = value
		update_configuration_warnings()
		if is_inside_tree():
			set_process(not map_path.is_empty())
## Write [code]<map>.overlay.json[/code] next to the map file when the map is built and when the scene is saved in
## the editor, so GodotTrench shows this overlay's content as read only ghost boxes and knows its targetnames.
@export var share_with_editor := true

## Emitted after the map rebuilt, once anchors have followed their entities.
signal map_rebuilt(map: FuncGodotMap)

## True for nodes the map build keeps: overlays and nodes in [constant KEEP_GROUP].
static func is_kept(node: Node) -> bool:
	return node is GodotTrenchOverlay or node.is_in_group(KEEP_GROUP)

## Number of kept children at the front of [param map], generated nodes go after them.
static func leading_kept(map: Node) -> int:
	var n := 0
	for child in map.get_children():
		if not is_kept(child):
			break
		n += 1
	return n

## Overlays of [param map] that live elsewhere in the tree and point at it with [member map_path].
static func external_overlays(map: Node) -> Array[GodotTrenchOverlay]:
	var out: Array[GodotTrenchOverlay] = []
	if not map.is_inside_tree():
		return out
	for n in map.get_tree().get_nodes_in_group(GROUP):
		var overlay := n as GodotTrenchOverlay
		if overlay and overlay.get_map() == map and not map.is_ancestor_of(overlay):
			out.append(overlay)
	return out

## Called by [FuncGodotMap] after every build.
static func notify_built(map: FuncGodotMap) -> void:
	var overlays: Array[GodotTrenchOverlay] = []
	for child in map.get_children():
		if child is GodotTrenchOverlay:
			overlays.append(child)
	overlays.append_array(external_overlays(map))
	for overlay in overlays:
		overlay._on_map_built(map)

func get_map() -> FuncGodotMap:
	if map_path.is_empty():
		return get_parent() as FuncGodotMap
	return get_node_or_null(map_path) as FuncGodotMap

func _enter_tree() -> void:
	add_to_group(GROUP)
	set_notify_local_transform(true)
	if get_parent() is FuncGodotMap and map_path.is_empty():
		transform = Transform3D.IDENTITY

func _ready() -> void:
	set_process(not map_path.is_empty())
	_follow_map()

func _exit_tree() -> void:
	remove_from_group(GROUP)

func _process(_delta: float) -> void:
	_follow_map()

func _notification(what: int) -> void:
	match what:
		NOTIFICATION_LOCAL_TRANSFORM_CHANGED:
			if map_path.is_empty() and get_parent() is FuncGodotMap and not transform.is_equal_approx(Transform3D.IDENTITY):
				transform = Transform3D.IDENTITY
		NOTIFICATION_EDITOR_PRE_SAVE:
			if share_with_editor and get_map():
				write_sidecar()

func _follow_map() -> void:
	if map_path.is_empty() or not is_inside_tree():
		return
	var map := get_map()
	if map and map.is_inside_tree() and not global_transform.is_equal_approx(map.global_transform):
		global_transform = map.global_transform

func _get_configuration_warnings() -> PackedStringArray:
	if not map_path.is_empty():
		return PackedStringArray() if get_map() else PackedStringArray(["map_path does not point at a FuncGodotMap."])
	if get_parent() is FuncGodotMap:
		return PackedStringArray()
	return PackedStringArray(["Add the overlay as a direct child of a FuncGodotMap, or point map_path at one. Inside generated nodes it is freed by the next build."])

func _on_map_built(map: FuncGodotMap) -> void:
	_follow_map()
	for anchor in anchors():
		anchor.refresh(true)
	map_rebuilt.emit(map)
	if share_with_editor and Engine.is_editor_hint():
		write_sidecar()

## Every [GodotTrenchAnchor] below this overlay.
func anchors() -> Array[GodotTrenchAnchor]:
	var out: Array[GodotTrenchAnchor] = []
	var stack: Array[Node] = [self]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is GodotTrenchAnchor:
			out.append(n)
		stack.append_array(n.get_children())
	return out

#region sidecar

## The file [param map] builds from, with uid:// resolved.
static func map_file(map: FuncGodotMap) -> String:
	var file: String = map.global_map_file if map.global_map_file != "" else map.local_map_file
	if file.begins_with("uid://"):
		var id := ResourceUID.text_to_id(file)
		file = ResourceUID.get_id_path(id) if ResourceUID.has_id(id) else ""
	return file

## Path of the overlay sidecar for [param map]: the map file with [code].overlay.json[/code] instead of its extension.
static func sidecar_path(map: FuncGodotMap) -> String:
	var file := map_file(map)
	return "" if file == "" else file.get_basename() + ".overlay.json"

## Identifies this overlay in the sidecar: its own scene file, else the scene it is saved in plus its node path.
func source_key() -> String:
	if scene_file_path != "":
		return scene_file_path
	if owner and owner.scene_file_path != "":
		return "%s::%s" % [owner.scene_file_path, owner.get_path_to(self)]
	return String(name)

## Transform of [param node] relative to this overlay, from local transforms so it also works outside the tree.
func _to_overlay(node: Node) -> Transform3D:
	var xform := Transform3D.IDENTITY
	var n := node
	while n and n != self:
		if n is Node3D:
			xform = (n as Node3D).transform * xform
		n = n.get_parent()
	return xform

static func _xform_box(xform: Transform3D, box: AABB) -> AABB:
	var out := AABB(xform * box.position, Vector3.ZERO)
	for i in 8:
		out = out.expand(xform * box.get_endpoint(i))
	return out

## Bounds of [param item] and everything below it, in meters relative to this overlay. Lights count as their position,
## a light's range is not its size, and an item without anything visible is the point it sits at.
func item_bounds(item: Node) -> AABB:
	var box := AABB()
	var first := true
	var stack: Array[Node] = [item]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		stack.append_array(n.get_children())
		var local := AABB()
		# Particles and labels report no size without a renderer, so their bounds come from their settings.
		if n is GeometryInstance3D and (n as GeometryInstance3D).custom_aabb.has_volume():
			local = (n as GeometryInstance3D).custom_aabb
		elif n is GPUParticles3D:
			local = (n as GPUParticles3D).visibility_aabb
		elif n is Label3D:
			local = _label_box(n as Label3D)
		elif n is GeometryInstance3D:
			local = (n as GeometryInstance3D).get_aabb()
		elif n is CollisionShape3D and (n as CollisionShape3D).shape and (n as CollisionShape3D).shape.get_debug_mesh():
			local = (n as CollisionShape3D).shape.get_debug_mesh().get_aabb()
		elif not n is Light3D:
			continue
		var placed := _xform_box(_to_overlay(n), local)
		box = placed if first else box.merge(placed)
		first = false
	if first:
		return AABB(_to_overlay(item).origin, Vector3.ZERO)
	return box

static func _label_box(label: Label3D) -> AABB:
	var font := label.font if label.font else ThemeDB.fallback_font
	if not font or label.text == "":
		return AABB()
	var width := label.width if label.autowrap_mode != TextServer.AUTOWRAP_OFF else -1.0
	var size := font.get_multiline_string_size(label.text, HORIZONTAL_ALIGNMENT_CENTER, width, label.font_size) * label.pixel_size
	return AABB(Vector3(-size.x * 0.5, -size.y * 0.5, 0.0), Vector3(size.x, size.y, 0.0))

static func _map_units(v: Vector3, units: float) -> Array:
	var p := v * units
	return [snappedf(p.x, 0.01), snappedf(p.y, 0.01), snappedf(p.z, 0.01)]

## This overlay as a sidecar entry: one item per direct child, with its bounds in map units.
func sidecar_entry(units_per_meter: float) -> Dictionary:
	var items := []
	for child in get_children():
		if not child is Node3D:
			continue
		var box := item_bounds(child)
		var item := {
			"name": String(child.name),
			"class": child.get_class(),
			"min": _map_units(box.position, units_per_meter),
			"max": _map_units(box.end, units_per_meter),
		}
		var names := PackedStringArray()
		var stack: Array[Node] = [child]
		while not stack.is_empty():
			var n: Node = stack.pop_back()
			stack.append_array(n.get_children())
			var io := n as GodotTrenchOverlayIO
			if io and io.targetname != "" and not names.has(io.targetname):
				names.append(io.targetname)
			elif n.has_meta(GodotTrenchIO.TARGETNAME_META) and not names.has(str(n.get_meta(GodotTrenchIO.TARGETNAME_META))):
				names.append(str(n.get_meta(GodotTrenchIO.TARGETNAME_META)))
		if not names.is_empty():
			item["targetnames"] = Array(names)
		if child is GodotTrenchAnchor and (child as GodotTrenchAnchor).target != "":
			item["anchor"] = (child as GodotTrenchAnchor).target
		items.append(item)
	return { "source": source_key(), "name": String(name), "items": items }

## Writes or updates this overlay's entry in the map's sidecar. Entries of other overlays of the same map are kept,
## entries whose scene file no longer exists are dropped. Returns the sidecar path, or "" when there is no map file.
func write_sidecar() -> String:
	var map := get_map()
	if not map:
		return ""
	var path := sidecar_path(map)
	if path == "":
		return ""
	var units := map.map_settings.inverse_scale_factor if map.map_settings else 32.0
	var entry := sidecar_entry(units)
	var overlays := []
	if FileAccess.file_exists(path):
		var old = JSON.parse_string(FileAccess.get_file_as_string(path))
		if old is Dictionary and old.get("overlays") is Array:
			for o in old["overlays"]:
				if not o is Dictionary:
					continue
				var source := str(o.get("source", ""))
				var scene := source.get_slice("::", 0)
				if source == entry["source"] or (scene.begins_with("res://") and not FileAccess.file_exists(scene)):
					continue
				overlays.append(o)
	overlays.append(entry)
	overlays.sort_custom(func(a, b): return str(a.get("source", "")) < str(b.get("source", "")))
	var text := _sidecar_text(map_file(map).get_file(), units, overlays)
	if FileAccess.file_exists(path) and FileAccess.get_file_as_string(path) == text:
		return path
	var file := FileAccess.open(path, FileAccess.WRITE)
	if not file:
		push_warning("[GodotTrench] could not write %s" % path)
		return ""
	file.store_string(text)
	return path

## The sidecar as JSON with one item per line, so it stays readable and diffs stay small.
static func _sidecar_text(map_name: String, units: float, overlays: Array) -> String:
	var blocks := PackedStringArray()
	for o in overlays:
		var items := PackedStringArray()
		for item in o.get("items", []):
			items.append("        " + JSON.stringify(item))
		blocks.append("    { \"source\": %s, \"name\": %s, \"items\": [\n%s\n    ] }" % [JSON.stringify(str(o.get("source", ""))), JSON.stringify(str(o.get("name", ""))), ",\n".join(items)])
	return "{\n  \"format\": \"%s\",\n  \"version\": %d,\n  \"map\": %s,\n  \"units_per_meter\": %s,\n  \"overlays\": [\n%s\n  ]\n}\n" % [SIDECAR_FORMAT, SIDECAR_VERSION, JSON.stringify(map_name), JSON.stringify(units), ",\n".join(blocks)]

#endregion
