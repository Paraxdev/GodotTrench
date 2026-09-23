extends RefCounted
## Entity, I/O and streaming tests, run from run_tests.gd: await load("res://tests/entity_tests.gd").new().run(self)

const PROBE := "res://tests/helpers/io_probe.gd"

## The run_tests.gd runner, untyped so its check() is reachable.
var t

func check(cond: bool, what: String) -> void:
	t.check(cond, what)

func wait(seconds: float) -> void:
	await t.create_timer(seconds).timeout

func run(runner: SceneTree) -> void:
	t = runner
	await test_saved_scene_state()
	await test_argument_placement()
	await test_outputs_pass_values()
	await test_builtin_show_hide()
	await test_timers_restart()
	await test_movers_redirect()
	await test_trigger_family()
	await test_particles_and_sequence()
	test_streamer_leaves_gameplay_visuals()
	test_hot_reload_uid()
	test_csharp_initializers()
	test_definitions()
	await test_light_fixture()
	await test_prop_model_node()

func _map() -> Node3D:
	var map := Node3D.new()
	t.root.add_child(map)
	return map

func _save_enemy() -> String:
	var enemy := Node3D.new()
	enemy.name = "Enemy"
	var packed := PackedScene.new()
	packed.pack(enemy)
	enemy.free()
	ResourceSaver.save(packed, "user://gt_entity_test_enemy.tscn")
	return "user://gt_entity_test_enemy.tscn"

## Entities built, packed into a scene and loaded again, the way a built map is saved and played.
func test_saved_scene_state() -> void:
	print("- entity state survives saving the built scene")
	var enemy := _save_enemy()
	var scene_root := Node3D.new()
	var built: Array[Node] = []
	var counter := GTCounter.new()
	counter.name = "Counter"
	counter._func_godot_apply_properties({ "min": 0, "max": 5, "start_value": 3 })
	var branch := GTBranch.new()
	branch.name = "Branch"
	branch._func_godot_apply_properties({ "start_value": true })
	var relay := GTRelay.new()
	relay.name = "Relay"
	relay._func_godot_apply_properties({ "start_disabled": true })
	var light := GTLight.new()
	light.name = "Light"
	light._func_godot_apply_properties({ "start_on": false })
	var spot := GTSpotLight.new()
	spot.name = "Spot"
	spot._func_godot_apply_properties({ "start_on": false })
	var barrel := GTPropPhysics.new()
	barrel.name = "Barrel"
	barrel._func_godot_apply_properties({ "health": 5.0 })
	var spawner := GTSpawner.new()
	spawner.name = "Spawner"
	spawner._func_godot_apply_properties({ "scene": enemy, "count": 2, "max_alive": 4, "total": 0, "snap_to_ground": false, "spawn_group": "gt_saved_spawns" })
	for n: Node in [counter, branch, relay, light, spot, barrel, spawner]:
		scene_root.add_child(n)
		n.owner = scene_root
		built.append(n)
	var packed := PackedScene.new()
	check(packed.pack(scene_root) == OK, "built entities pack into a scene")
	scene_root.free()
	var loaded := packed.instantiate()
	t.root.add_child(loaded)
	await t.process_frame
	check(loaded.get_node("Counter").value == 3, "saved counter starts at start_value, got %s" % loaded.get_node("Counter").value)
	check(loaded.get_node("Branch").value == true, "saved branch starts at start_value")
	check(not loaded.get_node("Relay").enabled, "saved start_disabled relay stays disabled")
	check(not loaded.get_node("Light").visible and not loaded.get_node("Light").is_on(), "saved start_on 0 light starts off")
	check(not loaded.get_node("Spot").visible and not loaded.get_node("Spot").is_on(), "saved start_on 0 spot light starts off")
	var hp: Array = []
	var saved_barrel: GTPropPhysics = loaded.get_node("Barrel")
	saved_barrel.freeze = true
	saved_barrel.damaged.connect(func(v): hp.append(v))
	saved_barrel.take_damage(4.0)
	check(hp == [1.0], "saved barrel keeps its health of 5, got %s" % [hp])
	var saved_spawner: GTSpawner = loaded.get_node("Spawner")
	saved_spawner.spawn()
	check(t.get_nodes_in_group(&"gt_saved_spawns").size() == 2, "saved spawner still knows its scene and count")
	loaded.queue_free()
	await t.process_frame

func test_argument_placement() -> void:
	print("- I/O argument placement")
	var map := _map()
	var player := Node3D.new()
	player.name = "Player"
	player.add_to_group(&"player")
	map.add_child(player)
	var probe: Node3D = load(PROBE).new()
	probe.name = "Probe"
	map.add_child(probe)
	var presser := Node3D.new()
	map.add_child(presser)
	await t.process_frame

	check(GodotTrenchIO.activator_arguments(probe, &"take_damage", presser) == [0.0, presser], "the activator fills a node argument after a value argument, got %s" % [GodotTrenchIO.activator_arguments(probe, &"take_damage", presser)])
	GodotTrenchIO.invoke(probe, &"take_damage", "", presser)
	check(probe.calls.back() == ["take_damage", 0.0, presser], "take_damage(amount, source) receives the activator as source, got %s" % [probe.calls.back()])
	GodotTrenchIO.invoke(probe, &"take_damage", "7", presser)
	check(probe.calls.back() == ["take_damage", 7.0, presser], "a parameter fills the value argument and the activator the node one")
	GodotTrenchIO.invoke(probe, &"take_damage", "!player", presser)
	check(probe.calls.back()[0] == "take_damage" and probe.calls.back()[1] == 0.0, "text that fits no argument falls back to the default instead of failing the call")
	GodotTrenchIO.invoke(probe, &"aim", "!player", presser)
	check(probe.calls.back() == ["aim", player, 1.0], "a target parameter resolves to a node for a leading node argument, got %s" % [probe.calls.back()])

	var relay := GTRelay.new()
	map.add_child(relay)
	var fired: Array = []
	relay.triggered.connect(func(a): fired.append(a))
	GodotTrenchIO.invoke(relay, &"trigger", "!player", presser)
	check(fired.back() == player, "relay.trigger with !player passes the player, got %s" % [fired])
	GodotTrenchIO.invoke(relay, &"trigger", "1", presser)
	check(fired.size() == 2 and fired.back() == presser, "a value parameter never lands in a node argument, the activator does")
	var script := GTScript.new()
	script.expression = "activator"
	map.add_child(script)
	var results: Array = []
	script.ran.connect(func(r): results.append(r))
	GodotTrenchIO.invoke(script, &"run", "!player", null)
	check(results.back() == player, "logic_script.run with !player runs with the player as activator")
	var boom := GTExplosion.new()
	boom.radius = 1.0
	map.add_child(boom)
	var booms: Array = []
	boom.exploded.connect(func(): booms.append(true))
	GodotTrenchIO.invoke(boom, &"explode", "1", null)
	check(booms.size() == 1, "explode with a numeric parameter still explodes")
	map.queue_free()
	await t.process_frame

func test_outputs_pass_values() -> void:
	print("- outputs pass their values along")
	var map := _map()
	var probe: Node3D = load(PROBE).new()
	probe.set_meta(GodotTrenchIO.TARGETNAME_META, "probe")
	map.add_child(probe)
	var counter := GTCounter.new()
	counter._func_godot_apply_properties({ "min": 0, "max": 10, "start_value": 0 })
	map.add_child(counter)
	var wire := func(source: Node, output: StringName, input: StringName, parameter := "") -> void:
		var o := GodotTrenchOutput.new()
		o.output = output
		o.target = "probe"
		o.input = input
		o.parameter = parameter
		source.add_child(o)
	wire.call(counter, &"changed", &"record")
	GodotTrenchIO.invalidate(probe)
	await t.process_frame
	counter.add(4)
	check(probe.calls.back() == ["record", 4, null], "changed(value) reaches the input as its parameter, got %s" % [probe.calls.back()])

	var sender: Node3D = load(PROBE).new()
	map.add_child(sender)
	wire.call(sender, &"reported_by", &"record")
	wire.call(sender, &"reported", &"record", "99")
	await t.process_frame
	var who := Node3D.new()
	map.add_child(who)
	sender.reported_by.emit(who, 6)
	check(probe.calls.back() == ["record", 6, who], "the first node is the activator, the other values are passed on, got %s" % [probe.calls.back()])
	sender.reported.emit(5)
	check(probe.calls.back() == ["record", 99, null], "an explicit parameter overrides the passed value")

	var seq := GTSequence.new()
	seq._func_godot_apply_properties({ "steps": 2, "interval": 0.0, "times": "0 0" })
	map.add_child(seq)
	wire.call(seq, &"step", &"speed")
	await t.process_frame
	seq.start()
	check(probe.calls.slice(-2) == [["speed", 1.0], ["speed", 2.0]], "step(index) passes the index, got %s" % [probe.calls.slice(-2)])
	map.queue_free()
	await t.process_frame

func test_builtin_show_hide() -> void:
	print("- built in show and hide")
	var map := _map()
	var plain := Node3D.new()
	map.add_child(plain)
	GodotTrenchIO.invoke(plain, &"hide", "", null)
	check(not plain.visible and plain.process_mode == Node.PROCESS_MODE_DISABLED, "hide on a Node3D hides it and stops processing")
	GodotTrenchIO.invoke(plain, &"show", "", null)
	check(plain.visible and plain.process_mode == Node.PROCESS_MODE_INHERIT, "show on a Node3D shows it and resumes processing")
	var text := GTText.new()
	text._func_godot_apply_properties({ "text": "hi", "place": "world" })
	map.add_child(text)
	await t.process_frame
	var shown: Array = []
	text.shown.connect(func(): shown.append(true))
	GodotTrenchIO.invoke(text, &"show", "", null)
	check(shown.size() == 1 and text.process_mode == Node.PROCESS_MODE_INHERIT, "an entity's own show method still wins")
	map.queue_free()
	await t.process_frame

func test_timers_restart() -> void:
	print("- auto close timers restart")
	var map := _map()
	var door := GTDoor.new()
	door._func_godot_apply_properties({ "travel": "0 1 0", "speed": 1000.0, "wait": 0.5 })
	map.add_child(door)
	await t.process_frame
	door.open()
	await wait(0.2)
	door.close()
	await wait(0.1)
	door.open()
	await wait(0.35)
	check(door.is_open, "a door reopened after closing is not closed by the first opening's timer")
	await wait(0.4)
	check(not door.is_open, "the reopened door closes after its own wait")

	var rot := GTDoorRotating.new()
	rot._func_godot_apply_properties({ "speed": 100000.0, "wait": 0.5, "open_away": false })
	map.add_child(rot)
	await t.process_frame
	rot.open()
	await wait(0.2)
	rot.close()
	await wait(0.1)
	rot.open()
	await wait(0.35)
	check(rot.is_open, "a rotating door reopened after closing keeps its own wait")

	var button := GTButton.new()
	button._func_godot_apply_properties({ "wait": 0.5 })
	map.add_child(button)
	await t.process_frame
	button.press()
	await wait(0.2)
	button.release()
	await wait(0.1)
	button.press()
	await wait(0.35)
	check(button.is_pressed, "a button pressed again is not released by the first press's timer")
	map.queue_free()
	await t.process_frame

func _corner(map: Node3D, targetname: String, pos: Vector3, next := "", wait_time := 0.0) -> GTPathCorner:
	var c := GTPathCorner.new()
	c._func_godot_apply_properties({ "target": next, "wait": wait_time })
	c.set_meta(GodotTrenchIO.TARGETNAME_META, targetname)
	c.position = pos
	map.add_child(c)
	GodotTrenchIO.invalidate(c)
	return c

func test_movers_redirect() -> void:
	print("- npc_walker and func_train redirects")
	var map := _map()
	var a := _corner(map, "gt_a", Vector3(10, 0, 0))
	var b := _corner(map, "gt_b", Vector3(0, 0, 1))
	var npc := GTNpc.new()
	npc._func_godot_apply_properties({ "speed": 10.0 })
	map.add_child(npc)
	await t.process_frame
	var arrived: Array = []
	npc.arrived.connect(func(c): arrived.append(c))
	npc.walk_to("gt_a")
	await t.physics_frame
	npc.walk_to("gt_b")
	await wait(0.6)
	check(arrived.size() >= 1 and arrived[0] == b, "walk_to while walking heads for the new corner at once, got %s" % [arrived])
	check(npc.global_position.distance_to(b.global_position) < 0.05, "the npc ends at the new corner, at %s" % npc.global_position)

	var c1 := _corner(map, "gt_t1", Vector3(0, 0, 0.1), "gt_t2", 0.3)
	var c2 := _corner(map, "gt_t2", Vector3(0, 0, 10))
	var train := GTTrain.new()
	train._func_godot_apply_properties({ "target": "gt_t1", "speed": 20.0, "loop": false, "start_active": false })
	map.add_child(train)
	await t.process_frame
	var reached: Array = []
	train.arrived.connect(func(c): reached.append(c))
	train.start()
	var deadline := Time.get_ticks_msec() + 1000
	while reached.is_empty() and Time.get_ticks_msec() < deadline:
		await t.physics_frame
	train.stop()
	train.start()
	await wait(0.9)
	check(reached.count(c2) == 1, "stop and start during a corner wait runs one leg, not two, got %s" % [reached])
	check(reached.count(c1) == 1, "the train reached the first corner once")
	map.queue_free()
	await t.process_frame

func test_trigger_family() -> void:
	print("- trigger cooldowns, hurt, push and spawn limits")
	var map := _map()
	var trigger := GTTrigger.new()
	trigger._func_godot_apply_properties({ "filter_group": "", "cooldown": 10.0 })
	map.add_child(trigger)
	await t.process_frame
	var fired: Array = []
	trigger.triggered.connect(func(a): fired.append(a))
	var one := StaticBody3D.new()
	var two := StaticBody3D.new()
	map.add_child(one)
	map.add_child(two)
	trigger._body_entered(one)
	trigger._body_entered(two)
	trigger._body_entered(one)
	check(fired == [one, two], "the cooldown is per body, got %s" % [fired])

	var hurt := GTHurt.new()
	hurt._func_godot_apply_properties({ "filter_group": "" })
	map.add_child(hurt)
	var hurt_list: Array = []
	hurt.hurt.connect(func(a): hurt_list.append(a))
	hurt.apply_damage(one)
	var probe: Node3D = load(PROBE).new()
	map.add_child(probe)
	hurt.apply_damage(probe)
	check(hurt_list == [probe], "trigger_hurt only reports bodies that took damage, got %s" % [hurt_list])

	var push := GTPush.new()
	push._func_godot_apply_properties({ "filter_group": "", "once": false })
	map.add_child(push)
	await t.process_frame
	var pushed: Array = []
	push.pushed.connect(func(a): pushed.append(a))
	push._body_entered(one)
	check(pushed == [one], "a continuous push reports pushed when a body starts being pushed")

	var spawner := GTSpawner.new()
	spawner._func_godot_apply_properties({ "scene": _save_enemy(), "count": 1, "total": 1, "snap_to_ground": false, "spawn_group": "gt_exhaust" })
	map.add_child(spawner)
	await t.process_frame
	var exhausted: Array = []
	spawner.exhausted.connect(func(): exhausted.append(true))
	for i in 4:
		spawner.spawn()
	check(exhausted.size() == 1, "exhausted fires once, got %d" % exhausted.size())
	map.queue_free()
	await t.process_frame

func test_particles_and_sequence() -> void:
	print("- particle bursts and zero wait sequence loops")
	var map := _map()
	var particles := GTParticles.new()
	particles._func_godot_apply_properties({ "amount": 4, "lifetime": 0.1, "one_shot": false, "start_emitting": false })
	map.add_child(particles)
	await t.process_frame
	particles.burst()
	check(particles.one_shot and particles.emitting, "burst emits as a single shot")
	var deadline := Time.get_ticks_msec() + 2000
	while particles.one_shot and Time.get_ticks_msec() < deadline:
		await t.process_frame
	check(not particles.one_shot and not particles.emitting, "after the burst a continuous emitter is back to idle")
	particles.start()
	particles.burst()
	particles.stop()
	check(not particles.one_shot and not particles.emitting, "stop during a burst restores continuous mode and stays off")

	var seq := GTSequence.new()
	seq._func_godot_apply_properties({ "steps": 2, "times": "0 0", "loop": true })
	map.add_child(seq)
	var steps: Array = []
	seq.step.connect(func(i): steps.append(i))
	seq.start()
	await t.process_frame
	await t.process_frame
	seq.stop()
	check(steps.size() >= 2 and steps.size() <= 8, "a zero wait loop yields a frame per pass instead of freezing, fired %d" % steps.size())
	map.queue_free()
	await t.process_frame

func _owned_mesh(parent: Node, owner_node: Node, pos: Vector3) -> MeshInstance3D:
	var mesh := MeshInstance3D.new()
	mesh.mesh = BoxMesh.new()
	mesh.position = pos
	parent.add_child(mesh)
	mesh.owner = owner_node
	return mesh

func test_streamer_leaves_gameplay_visuals() -> void:
	print("- streamer only streams build generated geometry")
	var map := Node3D.new()
	t.root.add_child(map)
	var wall := _owned_mesh(map, map, Vector3(500, 0, 0))
	var hidden_wall := _owned_mesh(map, map, Vector3(501, 0, 0))
	hidden_wall.visible = false
	var train := AnimatableBody3D.new()
	map.add_child(train)
	train.owner = map
	var train_mesh := _owned_mesh(train, map, Vector3(500, 0, 0))
	var label := Label3D.new()
	label.position = Vector3(500, 0, 0)
	map.add_child(label)
	var streamer := GodotTrenchStreamer.new()
	streamer.chunk_size = 64.0
	streamer.load_radius = 32.0
	var camera := Camera3D.new()
	map.add_child(camera)
	map.add_child(streamer)
	streamer.camera_path = streamer.get_path_to(camera)
	check(streamer.node_count() == 2, "moving bodies and runtime labels are not streamed, got %d" % streamer.node_count())
	streamer._process(0.0)
	check(not wall.visible and train_mesh.visible and label.visible, "out of range culls map geometry only")
	camera.global_position = Vector3(500, 0, 0)
	streamer._process(0.0)
	check(wall.visible and not hidden_wall.visible, "coming into range keeps a visual gameplay hid hidden")
	map.free()

func test_hot_reload_uid() -> void:
	print("- hot reload resolves uid:// map paths")
	var path := "res://tests/maps/basic.gtm"
	var id := ResourceLoader.get_resource_uid(path)
	var map := FuncGodotMap.new()
	map.local_map_file = ResourceUID.id_to_text(id) if id != ResourceUID.INVALID_ID else path
	check(map.local_map_file.begins_with("uid://"), "test map has a uid")
	var expected := ProjectSettings.globalize_path(path).replace("\\", "/").to_lower()
	check(GodotTrenchHotReload.map_path_key(map) == expected, "uid map path resolves to the saved file, got %s" % GodotTrenchHotReload.map_path_key(map))
	map.free()

func test_csharp_initializers() -> void:
	print("- C# member initializers")
	var src := """
[GodotTrenchEntity("cs_init")]
public partial class CsInit : Node3D
{
	[Export] public Color Tint { get; set; } = new Color(1, 0, 0);
	[Export] public Color Named = Colors.Red;
	[Export] public Vector3 Facing = Vector3.Up;
	[Export] public Vector3 Offset = new Vector3(1.5f, 2, -3);
	[Export] public float Turn = 2 * Mathf.Pi;
	[Export] public double Rate = 0.25d;
	[Export] public int Count = 4;
	[Export] public PackedScene Prefab = GD.Load<PackedScene>("res://demo/scenes/barrel.tscn");
	[Export] public string Label = SomeConstant;
}
"""
	var entries := GodotTrenchCSharp.parse_source(src)
	var props := {}
	for p in entries[0]["properties"]:
		props[p["name"]] = p["default"]
	check(props["tint"] == "255 0 0", "new Color(1, 0, 0) is red, got %s" % props["tint"])
	check(props["named"] == "255 0 0", "Colors.Red is red, got %s" % props["named"])
	check(props["facing"] == "0 1 0", "Vector3.Up is up, got %s" % props["facing"])
	check(props["offset"] == "1.5 2 -3", "new Vector3 literals with suffixes, got %s" % props["offset"])
	check(props["turn"] == "", "an expression leaves the default empty, got %s" % props["turn"])
	check(props["rate"] == "0.25" and props["count"] == "4", "numeric literals with suffixes")
	check(props["prefab"] == "res://demo/scenes/barrel.tscn", "GD.Load path becomes the resource default")
	check(props["label"] == "", "a constant name is not taken as text")

	var node := Node3D.new()
	var script := GDScript.new()
	script.source_code = "extends Node3D\nvar Turn := 6.28\nvar Prefab: PackedScene\nvar Count := 4\n"
	script.reload()
	node.set_script(script)
	GodotTrenchCSharp.apply_properties(node, { "turn": "", "prefab": "res://demo/scenes/barrel.tscn", "count": 7 })
	check(near(node.get("Turn"), 6.28), "an empty key leaves the member's initializer alone")
	check(node.get("Prefab") is PackedScene, "a resource key is loaded, not assigned as text")
	check(node.get("Count") == 7, "set keys are still assigned")
	node.free()

static func near(a: float, b: float) -> bool:
	return absf(a - b) < 0.001

func test_definitions() -> void:
	print("- trigger definitions and the demo lights")
	for classname in ["trigger_hurt", "trigger_teleport", "trigger_push", "trigger_call", "trigger_spawn_area"]:
		var def: FuncGodotFGDEntityClass = load("res://addons/func_godot/fgd/godottrench/%s.tres" % classname)
		check(def.class_properties.has("start_disabled"), "%s exposes start_disabled" % classname)
	for classname in ["trigger_teleport", "trigger_spawn_area", "trigger_call", "trigger_multiple"]:
		var def: FuncGodotFGDEntityClass = load("res://addons/func_godot/fgd/godottrench/%s.tres" % classname)
		check(def.class_properties.has("cooldown"), "%s exposes cooldown" % classname)
	var demo: FuncGodotFGDFile = load("res://demo/demo_fgd.tres")
	var defs := demo.get_entity_definitions()
	check(defs.has("light") and defs["light"].script_class == GTLight, "the demo uses the library light")
	check(defs.has("light_spot") and defs["light_spot"].script_class == GTSpotLight, "the demo uses the library spot light")
	var lamp: DemoLamp = DemoLamp.new()
	t.root.add_child(lamp)
	var switched: Array = []
	lamp.switched.connect(func(on): switched.append(on))
	lamp.turn_on()
	lamp.turn_off()
	lamp.turn_off()
	check(switched == [false], "the demo lamp only reports actual changes, got %s" % [switched])
	lamp.free()

func _lamp_geometry(parent: Node, targetname: String) -> Node3D:
	var lamp := Node3D.new()
	lamp.set_meta(GodotTrenchIO.TARGETNAME_META, targetname)
	var mi := MeshInstance3D.new()
	var mesh := BoxMesh.new()
	var glow := StandardMaterial3D.new()
	glow.emission_enabled = true
	glow.emission = Color(1, 0.8, 0.5)
	mesh.material = glow
	mi.mesh = mesh
	lamp.add_child(mi)
	parent.add_child(lamp)
	return lamp

func _glows(lamp: Node3D) -> bool:
	var mi := lamp.get_child(0) as MeshInstance3D
	return (mi.get_active_material(0) as BaseMaterial3D).emission_enabled

func test_light_fixture() -> void:
	print("- light fixtures follow the light")
	var map := _map()
	var bulb := _lamp_geometry(map, "bulb")
	var tube := _lamp_geometry(map, "tube_1")
	var light := GTLight.new()
	light._func_godot_apply_properties({ "start_on": false, "fixture": "bulb" })
	map.add_child(light)
	var spot := GTSpotLight.new()
	spot._func_godot_apply_properties({ "start_on": false, "fixture": "tube_*", "fixture_off": "dark" })
	map.add_child(spot)
	GodotTrenchIO.invalidate(light)
	await t.process_frame
	check(not bulb.visible, "a light that starts off hides its fixture")
	check(tube.visible and not _glows(tube), "a dark fixture stays visible with its emission off")
	GodotTrenchIO.invoke(light, &"turn_on", "", null)
	GodotTrenchIO.invoke(spot, &"turn_on", "", null)
	check(bulb.visible, "turning the light on shows the fixture")
	check(tube.visible and _glows(tube), "turning the spot on lights the fixture again")
	GodotTrenchIO.invoke(light, &"toggle", "", null)
	GodotTrenchIO.invoke(spot, &"toggle", "", null)
	check(not bulb.visible and not _glows(tube), "switching off follows too, bulb %s" % bulb.visible)
	map.queue_free()
	await t.process_frame

func test_prop_model_node() -> void:
	print("- prop_model picks one node of a model")
	var variants := Node3D.new()
	variants.name = "Variants"
	for i in 2:
		var v := MeshInstance3D.new()
		v.name = "hydrant_%d" % i
		v.mesh = BoxMesh.new()
		v.position = Vector3(10.0 + 90.0 * i, 0, 0)
		v.scale = Vector3.ONE * (1.0 + i)
		var cap := MeshInstance3D.new()
		cap.name = "cap"
		cap.mesh = BoxMesh.new()
		cap.position = Vector3(0, 1, 0)
		v.add_child(cap)
		variants.add_child(v)
		v.owner = variants
		cap.owner = variants
	var packed := PackedScene.new()
	packed.pack(variants)
	variants.free()
	ResourceSaver.save(packed, "user://gt_entity_test_variants.tscn")
	var map := _map()
	var prop := GodotTrenchProp.new()
	map.add_child(prop)
	prop._func_godot_apply_properties({ "model": "user://gt_entity_test_variants.tscn", "model_node": "hydrant_1", "collision": "none" })
	var picked := prop.get_child(0) as Node3D
	check(prop.get_child_count() == 1 and picked.name == &"hydrant_1", "only the named node is kept, got %s" % [prop.get_children()])
	check(picked.position == Vector3.ZERO and picked.scale.is_equal_approx(Vector3.ONE * 2.0), "it sits at the prop origin and keeps its scale, %s" % picked.transform)
	check(picked.get_child_count() == 1, "its children come with it")
	prop._func_godot_apply_properties({ "model_node": "" })
	check(prop.get_child_count() == 1 and prop.get_child(0).get_child_count() == 2, "without model_node the whole model is back")
	map.queue_free()
	await t.process_frame
