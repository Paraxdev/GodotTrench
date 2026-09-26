@tool
class_name GTSpawnArea extends GTTrigger
## trigger_spawn_area: spawns [member scene] at random spots inside its volume when a body of
## [member GTTrigger.filter_group] enters, on input, or on a timer.
## Inputs: spawn, start, stop, toggle, kill_all. Outputs: spawned(node), all_dead, exhausted.

signal spawned(node: Node)
signal all_dead
signal exhausted

@export var interval := 0.0
@export var spawn_on_enter := true
## res:// scene to spawn.
@export var scene := ""
@export var count := 3
@export var max_alive := 10
## Stops after this many, 0 is unlimited.
@export var total := 0
@export var spawn_group := "enemies"
@export var snap_to_ground := true

var logic := GTSpawnLogic.new()
var active := false
var _timer: Timer

func _func_godot_apply_properties(props: Dictionary) -> void:
	super(props)
	scene = str(props.get("scene", scene))
	count = int(props.get("count", count))
	max_alive = int(props.get("max_alive", max_alive))
	total = int(props.get("total", total))
	spawn_group = str(props.get("spawn_group", spawn_group))
	snap_to_ground = GodotTrenchIO.to_bool(props.get("snap_to_ground", snap_to_ground))
	interval = float(props.get("interval", interval))
	spawn_on_enter = GodotTrenchIO.to_bool(props.get("spawn_on_enter", spawn_on_enter))
	if is_node_ready() and not Engine.is_editor_hint():
		logic.configure(self, GTSpawnLogic.settings_of(self))

func _ready() -> void:
	super()
	if Engine.is_editor_hint():
		return
	logic.configure(self, GTSpawnLogic.settings_of(self))
	logic.spawned.connect(func(n): spawned.emit(n))
	logic.all_dead.connect(func(): all_dead.emit())
	logic.exhausted.connect(func(): exhausted.emit())
	_timer = Timer.new()
	_timer.timeout.connect(spawn)
	add_child(_timer)

func _on_triggered(_activator: Node) -> void:
	if spawn_on_enter:
		spawn()

## Random point inside the volume's collision shapes, in global space.
func random_point() -> Vector3:
	var box := AABB()
	var first := true
	for child in get_children():
		if child is CollisionShape3D and child.shape:
			var local: AABB = child.shape.get_debug_mesh().get_aabb()
			var placed: AABB = child.transform * local
			box = placed if first else box.merge(placed)
			first = false
	if first:
		return global_position
	var p := box.position + Vector3(randf() * box.size.x, box.size.y * 0.5, randf() * box.size.z)
	return global_transform * p

func spawn() -> void:
	logic.spawn(random_point)

func start() -> void:
	active = true
	if interval > 0.0 and _timer:
		_timer.start(interval)

func stop() -> void:
	active = false
	if _timer:
		_timer.stop()

func toggle() -> void:
	if active:
		stop()
	else:
		start()

func kill_all() -> void:
	logic.kill_all()
