@tool
class_name GTSpawner extends Node3D
## info_spawner: spawns [member scene] around itself on input, on a timer, or once when the map loads.
## Inputs: spawn, start, stop, toggle, kill_all. Outputs: spawned(node), all_dead, exhausted.

signal spawned(node: Node)
signal all_dead
signal exhausted

## Map units of random spread around the spawner.
@export var radius := 64.0
## Seconds between spawns while active, 0 spawns only on input.
@export var interval := 0.0
@export var start_active := false
@export var spawn_on_ready := false
## res:// scene to spawn.
@export var scene := ""
@export var count := 1
@export var max_alive := 5
## Stops after this many, 0 is unlimited.
@export var total := 0
@export var spawn_group := "enemies"
@export var snap_to_ground := true

var logic := GTSpawnLogic.new()
var active := false
var _timer: Timer

func _func_godot_apply_properties(props: Dictionary) -> void:
	scene = str(props.get("scene", scene))
	count = int(props.get("count", count))
	max_alive = int(props.get("max_alive", max_alive))
	total = int(props.get("total", total))
	spawn_group = str(props.get("spawn_group", spawn_group))
	snap_to_ground = GodotTrenchIO.to_bool(props.get("snap_to_ground", snap_to_ground))
	radius = float(props.get("radius", radius))
	interval = float(props.get("interval", interval))
	start_active = GodotTrenchIO.to_bool(props.get("start_active", start_active))
	spawn_on_ready = GodotTrenchIO.to_bool(props.get("spawn_on_ready", spawn_on_ready))
	# A map built inside the running tree applies properties after _ready.
	if is_node_ready() and not Engine.is_editor_hint():
		logic.configure(self, GTSpawnLogic.settings_of(self))

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	logic.configure(self, GTSpawnLogic.settings_of(self))
	logic.spawned.connect(func(n): spawned.emit(n))
	logic.all_dead.connect(func(): all_dead.emit())
	logic.exhausted.connect(func(): exhausted.emit())
	_timer = Timer.new()
	_timer.timeout.connect(spawn)
	add_child(_timer)
	_begin.call_deferred()

func _begin() -> void:
	if spawn_on_ready:
		spawn()
	if start_active:
		start()

func random_point() -> Vector3:
	var r := radius / GodotTrenchIO.units_per_meter(self) * sqrt(randf())
	var a := randf() * TAU
	return global_position + Vector3(cos(a) * r, 0.0, sin(a) * r)

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
