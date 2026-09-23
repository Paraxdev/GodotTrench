@tool
@icon("res://addons/func_godot/icons/icon_anchor3d.svg")
class_name GodotTrenchAnchor extends Node3D
## Keeps its children on a map entity. Wherever the entity ends up, after a rebuild, a live edit in GodotTrench or
## at runtime (a door opening, a train on its track), the anchor follows at the same offset.
##
## Place the anchor where its content belongs and set [member target] to the entity's targetname: it binds to the
## entity at that offset. Moving the anchor afterwards changes the offset, moving the entity moves the anchor.
## Put it inside a [GodotTrenchOverlay], generated entity nodes are replaced on every build.

## Targetname of the entity to follow. A wildcard or @group follows the first match.
@export var target := "":
	set(value):
		if value == target:
			return
		target = value
		bound = false
		_node = null
		_synced = false
## The anchor's transform in the entity's space, kept up to date while the anchor is moved.
@export_storage var offset := Transform3D.IDENTITY
## False until the anchor has taken its offset from where it was placed.
@export_storage var bound := false

## Emitted when the anchor moved because its entity moved or was rebuilt.
signal followed(entity: Node3D)

const RETRY_MSEC := 500

var _node: Node3D
var _synced := false
var _last_target := Transform3D.IDENTITY
var _last_self := Transform3D.IDENTITY
var _next_try := 0

func _ready() -> void:
	refresh()

func _process(_delta: float) -> void:
	refresh()

## The entity this anchor follows, or null while it is missing.
func entity() -> Node3D:
	if is_instance_valid(_node) and _node.is_inside_tree():
		return _node
	_node = null
	_synced = false
	if target == "" or not is_inside_tree() or Time.get_ticks_msec() < _next_try:
		return null
	for n in GodotTrenchIO.find_targets(self, target, null):
		if n is Node3D and n != self and not is_ancestor_of(n) and n.is_inside_tree():
			_node = n
			return _node
	_next_try = Time.get_ticks_msec() + RETRY_MSEC
	return null

## Follows the entity if it moved, or takes a new offset if the anchor itself was moved. It runs every frame, and the
## map calls it with [param retry] after every build so a missing entity is looked up again right away. Returns false
## while the entity is missing.
func refresh(retry := false) -> bool:
	if retry:
		_next_try = 0
	var node := entity()
	if not node:
		return false
	var at := node.global_transform
	if not bound:
		offset = at.affine_inverse() * global_transform
		bound = true
	elif not _synced or not at.is_equal_approx(_last_target):
		global_transform = at * offset
		followed.emit(node)
	elif not global_transform.is_equal_approx(_last_self):
		offset = at.affine_inverse() * global_transform
	_last_target = at
	_last_self = global_transform
	_synced = true
	return true

func _get_configuration_warnings() -> PackedStringArray:
	if target == "":
		return PackedStringArray(["Set target to the targetname of the map entity to follow."])
	return PackedStringArray()
