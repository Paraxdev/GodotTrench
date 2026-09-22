@tool
class_name GTNpc extends CharacterBody3D
## npc_walker: a scripted actor that walks a chain of path_corner entities and plays animations, for in game
## cutscenes in the Half-Life style. It loads model as its body, then follows target corner to corner.
## Inputs: start, stop, walk_to(corner), play_anim(name), face(target). Outputs: arrived(corner), reached_goal, finished.

signal arrived(corner: Node)
signal reached_goal
signal finished

## res:// scene giving the actor its mesh and AnimationPlayer.
@export var model := ""
## First path_corner to walk to.
@export var target := ""
@export var speed := 2.0
@export var loop := false
@export var start_active := false
@export var walk_anim := "walk"
@export var idle_anim := "idle"
@export var face_travel := true

var _running := false
var _next: Node3D
var _first: Node3D
var _tween: Tween
var _player: AnimationPlayer
## Bumped by every new leg, stop and redirect, so a finished tween or corner wait from an older leg does nothing.
var _leg := 0

func _func_godot_apply_properties(props: Dictionary) -> void:
	model = str(props.get("model", model))
	target = str(props.get("target", target))
	speed = float(props.get("speed", speed))
	loop = GodotTrenchIO.to_bool(props.get("loop", loop))
	start_active = GodotTrenchIO.to_bool(props.get("start_active", start_active))
	walk_anim = str(props.get("walk_anim", walk_anim))
	idle_anim = str(props.get("idle_anim", idle_anim))
	face_travel = GodotTrenchIO.to_bool(props.get("face_travel", face_travel))

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	if model != "" and ResourceLoader.exists(model):
		var packed := load(model) as PackedScene
		if packed:
			add_child(packed.instantiate())
	_player = GTAnimate._find_animation_player(self)
	_play(idle_anim)
	if start_active:
		start.call_deferred()

func _corner(corner_name: String) -> Node3D:
	for n in GodotTrenchIO.find_targets(self, corner_name, null):
		if n is Node3D:
			return n
	return null

func start() -> void:
	if _running:
		return
	if not _next:
		_first = _corner(target)
		_next = _first
	if not _next:
		push_warning("[GT] npc_walker %s has no path_corner named '%s'" % [name, target])
		return
	_running = true
	_play(walk_anim)
	_travel()

func stop() -> void:
	_running = false
	_leg += 1
	if _tween:
		_tween.kill()
	_play(idle_anim)

## Heads for [param corner] at once, abandoning the current leg or corner wait.
func walk_to(corner: Variant = null) -> void:
	var dest := _corner(str(corner)) if corner != null else null
	if not dest:
		return
	_next = dest
	if not _running:
		_running = true
		_play(walk_anim)
	_travel()

func face(target_name: Variant = null) -> void:
	var node := _corner(str(target_name)) if target_name != null else null
	if node and global_position.distance_to(node.global_position) > 0.01:
		look_at(node.global_position, Vector3.UP)

func play_anim(anim: Variant = null) -> void:
	_play(str(anim) if anim != null and str(anim) != "" else walk_anim)

func _play(anim: String) -> void:
	if _player and anim != "" and _player.has_animation(anim):
		_player.play(anim)

func _travel() -> void:
	if not _running or not _next or not is_inside_tree():
		return
	_leg += 1
	var leg := _leg
	if _tween:
		_tween.kill()
	var dest := _next.global_position
	if face_travel and global_position.distance_to(dest) > 0.01:
		look_at(dest, Vector3.UP)
	_tween = create_tween()
	_tween.set_process_mode(Tween.TWEEN_PROCESS_PHYSICS)
	_tween.tween_property(self, "global_position", dest, global_position.distance_to(dest) / maxf(speed, 0.001))
	_tween.finished.connect(_reached.bind(leg))

func _reached(leg: int) -> void:
	if leg != _leg:
		return
	var corner := _next
	arrived.emit(corner)
	if corner.has_signal(&"reached"):
		corner.emit_signal(&"reached", self)
	var wait := float(corner.get("wait")) if "wait" in corner else 0.0
	var next_name := str(corner.get("target")) if "target" in corner else ""
	_next = _corner(next_name) if next_name != "" else null
	if not _next:
		if loop and _first:
			_next = _first
		else:
			_running = false
			_play(idle_anim)
			reached_goal.emit()
			finished.emit()
			return
	if wait > 0.0:
		await get_tree().create_timer(wait).timeout
		if leg != _leg:
			return
	_travel()
