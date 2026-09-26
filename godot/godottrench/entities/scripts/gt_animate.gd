@tool
class_name GTAnimate extends Node3D
## logic_animate: plays animations on the AnimationPlayer under a target entity, so I/O can drive doors, props or
## characters that carry their own animations. Inputs: play(name), stop, queue(name), seek(time). Output: finished(anim).

signal finished(anim: String)

## Entity whose AnimationPlayer to drive: a targetname, !self, !activator, @group or a node path.
@export var target := "!self"
## Animation played when play fires without a name.
@export var animation := ""
@export var speed := 1.0

func _func_godot_apply_properties(props: Dictionary) -> void:
	target = str(props.get("target", target))
	animation = str(props.get("animation", animation))
	speed = float(props.get("speed", speed))

func _player_for(activator: Node) -> AnimationPlayer:
	for node in GodotTrenchIO.find_targets(self, target, activator):
		var found := _find_animation_player(node)
		if found:
			return found
	return null

## The first AnimationPlayer at or under [param node].
static func _find_animation_player(node: Node) -> AnimationPlayer:
	if node is AnimationPlayer:
		return node
	for child in node.get_children():
		var found := _find_animation_player(child)
		if found:
			return found
	return null

func play(anim: Variant = null, activator: Node = null) -> void:
	var player := _player_for(activator)
	if not player:
		push_warning("[GT] logic_animate %s found no AnimationPlayer under '%s'" % [name, target])
		return
	var clip := animation
	if anim != null and str(anim) != "":
		clip = str(anim)
	if not player.has_animation(clip):
		push_warning("[GT] logic_animate %s: no animation '%s'" % [name, clip])
		return
	if not player.animation_finished.is_connected(_on_finished):
		player.animation_finished.connect(_on_finished)
	player.speed_scale = speed
	player.play(clip)

func queue(anim: Variant = null, activator: Node = null) -> void:
	var player := _player_for(activator)
	if player:
		var clip := animation
		if anim != null and str(anim) != "":
			clip = str(anim)
		player.queue(clip)

func stop(activator: Node = null) -> void:
	var player := _player_for(activator)
	if player:
		player.stop()

func seek(time: Variant = 0.0, activator: Node = null) -> void:
	var player := _player_for(activator)
	if player:
		player.seek(float(time), true)

func _on_finished(anim_name: StringName) -> void:
	finished.emit(str(anim_name))
