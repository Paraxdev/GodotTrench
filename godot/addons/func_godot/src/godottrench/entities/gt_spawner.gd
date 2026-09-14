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

var logic := GTSpawnLogic.new()
var active := false
var _timer: Timer
var _props: Dictionary = {}

func _func_godot_apply_properties(props: Dictionary) -> void:
	_props = props
	radius = float(props.get("radius", radius))
	interval = float(props.get("interval", interval))
	start_active = GodotTrenchIO.to_bool(props.get("start_active", start_active))
	spawn_on_ready = GodotTrenchIO.to_bool(props.get("spawn_on_ready", spawn_on_ready))

func _ready() -> void:
	if Engine.is_editor_hint():
		return
	logic.configure(self, _props)
	logic.spawned.connect(func(n): spawned.emit(n))
	logic.all_dead.connect(func(): all_dead.emit())
	logic.exhausted.connect(func(): exhausted.emit())
	_timer = Timer.new()
	_timer.timeout.connect(spawn)
	add_child(_timer)
	if spawn_on_ready:
		spawn.call_deferred()
	if start_active:
		start.call_deferred()

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
