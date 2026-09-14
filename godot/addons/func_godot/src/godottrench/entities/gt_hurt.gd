@tool
class_name GTHurt extends GTTrigger
## trigger_hurt: calls [member damage_method](amount, source) on bodies inside every [member interval] seconds.
## Output: hurt(activator).

signal hurt(activator: Node)

## Damage per second.
@export var damage := 10.0
@export var damage_method := "take_damage"
@export var interval := 0.5

var _elapsed := 0.0

func _func_godot_apply_properties(props: Dictionary) -> void:
	super(props)
	damage = float(props.get("damage", damage))
	damage_method = str(props.get("damage_method", damage_method))
	interval = float(props.get("interval", interval))

func _physics_process(delta: float) -> void:
	if Engine.is_editor_hint() or not enabled:
		return
	_elapsed += delta
	if _elapsed < interval:
		return
	_elapsed = 0.0
	for body in bodies_inside():
		apply_damage(body)

func apply_damage(body: Node) -> void:
	if GodotTrenchIO.resolve_method(body, damage_method) != &"":
		GodotTrenchIO.call_method(body, damage_method, [damage * interval, self])
	hurt.emit(body)
