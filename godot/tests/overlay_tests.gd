extends RefCounted
## Map overlays: Godot content under a GodotTrenchOverlay survives full builds, Clear Map, live sessions and hot
## reload, joins the map's I/O in both directions, anchors follow their entities, and the sidecar describes it for
## the level editor. The last test plays the night district demo overlay.
##
## res://tests/run_tests.gd calls [method run], [param t] is that script's instance for its check helpers.

const MAP := "res://tests/maps/basic.gtm"
const SETTINGS := "res://demo/demo_map_settings.tres"
const PROBE := preload("res://tests/helpers/io_probe.gd")

static func run(t) -> void:
	print("- overlays")
	await _test_survives_rebuild(t)
	await _test_map_output_reaches_overlay(t)
	await _test_overlay_output_reaches_map(t)
	await _test_anchor_follows_entity(t)
	await _test_live_session_and_hot_reload(t)
	await _test_external_overlay(t)
	_test_streamer_leaves_overlay_alone(t)
	await _test_sidecar(t)
	await _test_night_district_overlay(t)

## A level scene in the tree with a map built from [param text] (basic.gtm when empty).
static func _level(t, text := "") -> FuncGodotMap:
	var level := Node3D.new()
	level.name = "Level"
	t.root.add_child(level)
	var map := FuncGodotMap.new()
	map.name = "Map"
	map.map_settings = load(SETTINGS)
	map.local_map_file = MAP
	level.add_child(map)
	map.owner = level
	if text == "":
		map.build()
	else:
		map.build_from_text(text)
	return map

static func _overlay(map: FuncGodotMap) -> GodotTrenchOverlay:
	var overlay := GodotTrenchOverlay.new()
	overlay.name = "Overlay"
	overlay.share_with_editor = false
	map.add_child(overlay)
	overlay.owner = map.owner
	return overlay

static func _add(parent: Node, node: Node, node_name: String) -> Node:
	node.name = node_name
	parent.add_child(node)
	var scene_owner: Node = parent.owner if parent.owner else parent
	node.owner = scene_owner
	return node

static func _io(parent: Node, targetname: String) -> GodotTrenchOverlayIO:
	var io := GodotTrenchOverlayIO.new()
	io.targetname = targetname
	return _add(parent, io, "IO") as GodotTrenchOverlayIO

static func _output(parent: Node, output: String, target: String, input: String, parameter := "") -> GodotTrenchOutput:
	var relay := GodotTrenchOutput.new()
	relay.output = StringName(output)
	relay.target = target
	relay.input = StringName(input)
	relay.parameter = parameter
	return _add(parent, relay, "out_%s_%s" % [output, target]) as GodotTrenchOutput

static func _by_targetname(map: Node, targetname: String) -> Node:
	var found: Node = null
	var stack: Array[Node] = [map]
	while not stack.is_empty():
		var n: Node = stack.pop_back()
		if n is GodotTrenchOverlay:
			continue
		if str(n.get_meta(GodotTrenchIO.TARGETNAME_META, "")) == targetname:
			found = n
		stack.append_array(n.get_children())
	return found

static func _basic_with_outputs(outputs: Array) -> String:
	var json: Dictionary = JSON.parse_string(FileAccess.get_file_as_string(MAP))
	for layer in json["layers"]:
		for node in layer.get("children", []):
			if node.get("type") == "entity" and node.get("properties", {}).get("targetname", "") == "button1":
				node["outputs"].append_array(outputs)
	return JSON.stringify(json)

static func _test_survives_rebuild(t) -> void:
	var map := _level(t)
	var level := map.owner
	await t.process_frame
	var overlay := _overlay(map)
	var decor := _add(overlay, MeshInstance3D.new(), "Decor") as MeshInstance3D
	decor.mesh = BoxMesh.new()
	decor.position = Vector3(1, 2, 3)
	var prop := _add(overlay, StaticBody3D.new(), "Prop")
	var shape := _add(prop, CollisionShape3D.new(), "Shape") as CollisionShape3D
	shape.shape = BoxShape3D.new()
	_io(prop, "overlay_prop")
	var copied := _add(overlay, Node3D.new(), "CopiedLamp")
	copied.set_meta(GodotTrenchBuild.ID_META, 15)
	var kept := _add(map, Node3D.new(), "KeptMarker")
	kept.add_to_group(GodotTrenchOverlay.KEEP_GROUP, true)
	var loose := _add(map, Node3D.new(), "LooseUserNode")
	var content_before: Array = t.collect(overlay, func(_n): return true)

	for i in 3:
		map.build()
		await t.process_frame
	t.check(is_instance_valid(overlay) and overlay.get_parent() == map, "the overlay survives three full builds")
	t.check(is_instance_valid(kept) and kept.get_parent() == map, "a node in the godottrench_keep group survives full builds")
	t.check(not is_instance_valid(loose), "other user nodes directly under the map are still freed by a build")
	var content_after: Array = t.collect(overlay, func(_n): return true)
	t.check(content_after.size() == content_before.size() and content_after.all(func(n): return content_before.has(n)), "the overlay keeps the same node instances, got %d of %d" % [content_after.size(), content_before.size()])
	t.check(t.collect(map, func(n): return n is GodotTrenchOverlay).size() == 1, "the overlay is not duplicated")
	t.check(content_after.all(func(n): return n == overlay or n.owner == level) and overlay.owner == level, "overlay nodes keep the scene root as owner")
	t.check(map.get_child(0) == overlay and map.get_child(1) == kept and String(map.get_child(2).name) == "entity_0_worldspawn", "kept nodes stay in front of the generated ones, got %s" % [map.get_children().map(func(n): return String(n.name))])
	t.check(GodotTrenchBuild.nodes_by_id(map).get(15) is GTLight, "a generated node id copied into an overlay is not taken for the entity")
	overlay.position = Vector3(5, 0, 0)
	t.check(overlay.transform.is_equal_approx(Transform3D.IDENTITY), "an overlay under a map stays at the map origin")

	map.clear_children()
	t.check(is_instance_valid(overlay) and map.get_child_count() == 2, "Clear Map keeps overlays and kept nodes")
	map.build()
	var packed := PackedScene.new()
	t.check(packed.pack(level) == OK, "a level with an overlay packs")
	var copy := packed.instantiate()
	var copy_map: Node = copy.get_node("Map")
	var copy_overlay := copy_map.get_node_or_null("Overlay") as GodotTrenchOverlay
	t.check(copy_overlay != null and copy_overlay.get_node_or_null("Decor") is MeshInstance3D and copy_overlay.get_node_or_null("Prop/IO") is GodotTrenchOverlayIO, "the saved scene holds the overlay content")
	t.check(copy_overlay != null and (copy_overlay.get_node("Decor") as Node3D).position.is_equal_approx(Vector3(1, 2, 3)), "overlay content keeps its transform in the saved scene")
	t.check(t.collect(copy, func(n): return n is GodotTrenchOverlay).size() == 1 and copy_map.get_node_or_null("entity_0_worldspawn") != null, "the saved scene has one overlay and the generated map")
	copy.free()
	level.free()

static func _test_map_output_reaches_overlay(t) -> void:
	var text := _basic_with_outputs([
		{ "output": "pressed", "target": "overlay_fx", "input": "emitting", "parameter": "true" },
		{ "output": "pressed", "target": "overlay_probe", "input": "record", "parameter": "7" },
		{ "output": "pressed", "target": "overlay_sign", "input": "hide" },
		{ "output": "pressed", "target": "overlay_bare", "input": "ring" },
	])
	var map := _level(t, text)
	var overlay := _overlay(map)
	var fx := _add(overlay, GPUParticles3D.new(), "Fx") as GPUParticles3D
	fx.emitting = false
	_io(fx, "overlay_fx")
	var probe := _add(overlay, PROBE.new(), "Probe")
	_io(probe, "overlay_probe")
	var sign_label := _add(overlay, Label3D.new(), "Sign") as Label3D
	_io(sign_label, "overlay_sign")
	var bare := _add(overlay, Node3D.new(), "Bare")
	var bare_io := _io(bare, "overlay_bare")
	var heard: Array = []
	bare_io.input_received.connect(func(input, parameter, _activator): heard.append([input, parameter]))
	await t.process_frame

	for attempt in 2:
		fx.emitting = false
		sign_label.visible = true
		probe.calls.clear()
		heard.clear()
		var button := _by_targetname(map, "button1")
		GodotTrenchIO.fire_output(button, &"pressed", null)
		t.check(fx.emitting, "round %d: a map output sets a property on an overlay node" % attempt)
		t.check(probe.calls == [["record", 7, null]], "round %d: a map output calls an overlay method, got %s" % [attempt, probe.calls])
		t.check(not sign_label.visible, "round %d: built in inputs reach overlay nodes" % attempt)
		t.check(heard == [[&"ring", ""]], "round %d: GodotTrenchOverlayIO emits input_received, got %s" % [attempt, heard])
		map.build_from_text(text)
		await t.process_frame
	t.check(fx.get_meta(GodotTrenchIO.TARGETNAME_META, "") == "overlay_fx", "the overlay targetname is set on its parent at runtime")
	var io: GodotTrenchOverlayIO = fx.get_node("IO")
	io.targetname = "renamed_fx"
	fx.emitting = false
	GodotTrenchIO.fire_output(_by_targetname(map, "button1"), &"pressed", null)
	t.check(not fx.emitting, "a renamed overlay target no longer answers to its old name")
	fx.remove_child(io)
	t.check(not fx.has_meta(GodotTrenchIO.TARGETNAME_META), "removing the GodotTrenchOverlayIO removes the targetname")
	io.free()
	map.owner.free()

static func _test_overlay_output_reaches_map(t) -> void:
	var map := _level(t)
	var overlay := _overlay(map)
	var area := _add(overlay, Area3D.new(), "Pad") as Area3D
	_output(area, "body_entered", "lamp", "toggle")
	var lever := _add(overlay, Node3D.new(), "Lever")
	var lever_io := _io(lever, "lever")
	_output(lever, "pulled", "door1", "open")
	await t.process_frame

	var lamp := _by_targetname(map, "lamp") as GTLight
	t.check(lamp != null and not lamp.is_on(), "the map lamp starts off")
	var body := StaticBody3D.new()
	area.body_entered.emit(body)
	t.check(lamp.is_on(), "an overlay signal output toggles a map entity")
	lever_io.fire(&"pulled")
	var door := _by_targetname(map, "door1") as GTDoor
	t.check(door != null and door.is_open, "GodotTrenchOverlayIO.fire sends an output without a signal")

	map.build()
	await t.process_frame
	var lamp2 := _by_targetname(map, "lamp") as GTLight
	t.check(lamp2 != null and lamp2 != lamp and not lamp2.is_on(), "the rebuilt lamp is a new node")
	area.body_entered.emit(body)
	t.check(lamp2.is_on(), "overlay outputs reach the rebuilt entity, targets resolve when they fire")
	body.free()
	map.owner.free()

static func _test_anchor_follows_entity(t) -> void:
	var text := FileAccess.get_file_as_string(MAP)
	var map := _level(t, text)
	var overlay := _overlay(map)
	await t.process_frame
	var door := GodotTrenchBuild.nodes_by_id(map)[9] as Node3D
	var anchor := GodotTrenchAnchor.new()
	anchor.position = door.position + Vector3(0, 1, 0)
	_add(overlay, anchor, "DoorSign")
	var sign_label := _add(anchor, Label3D.new(), "Sign") as Label3D
	anchor.target = "door1"
	t.check(anchor.refresh(), "the anchor finds its entity")
	t.check(anchor.bound and anchor.offset.origin.is_equal_approx(Vector3(0, 1, 0)), "the anchor binds at the offset it was placed at, got %s" % anchor.offset.origin)

	var maps: Array[FuncGodotMap] = [map]
	var session := GodotTrenchLiveSession.new(MAP, maps)
	session.begin(text)
	session.apply([{ "op": "translate", "ids": [10], "offset": [64.0, 0.0, 0.0] }])
	anchor.refresh()
	t.check(anchor.global_position.is_equal_approx(door.global_position + Vector3(0, 1, 0)), "a live drag moves the anchor with its entity")
	var door_x := door.position.x
	session.apply([{ "op": "set", "id": 10, "parent": 9, "index": 0, "node": session._nodes[10].duplicate(true) }])
	session.process()
	await t.process_frame
	var door2 := GodotTrenchBuild.nodes_by_id(map)[9] as Node3D
	t.check(door2 != door and anchor.global_position.is_equal_approx(door2.global_position + Vector3(0, 1, 0)), "a live entity rebuild keeps the anchor on the new node")
	t.check(t.near(door2.position.x, door_x), "the rebuilt door kept the dragged position")

	session._translate([9, 10], Vector3(0, 0, -96))
	map.build_from_text(session.text())
	await t.process_frame
	var door3 := GodotTrenchBuild.nodes_by_id(map)[9] as Node3D
	t.check(anchor.global_position.is_equal_approx(door3.global_position + Vector3(0, 1, 0)), "after a full build the anchor sits on the moved entity, anchor %s door %s" % [anchor.global_position, door3.global_position])
	t.check(sign_label.get_parent() == anchor and is_instance_valid(sign_label), "anchored content survives")
	door3.position += Vector3(0, 2, 0)
	await t.process_frame
	t.check(anchor.global_position.is_equal_approx(door3.global_position + Vector3(0, 1, 0)), "the anchor follows its entity moving at runtime")
	anchor.global_position += Vector3(0.5, 0, 0)
	anchor.refresh()
	t.check(anchor.offset.origin.is_equal_approx(Vector3(0.5, 1, 0)), "moving the anchor itself changes its offset, got %s" % anchor.offset.origin)
	map.owner.free()

static func _test_live_session_and_hot_reload(t) -> void:
	var text := FileAccess.get_file_as_string(MAP)
	var map := _level(t, text)
	var overlay := _overlay(map)
	var decor := _add(overlay, MeshInstance3D.new(), "Decor")
	var probe := _add(overlay, PROBE.new(), "Probe")
	_io(probe, "overlay_probe")
	await t.process_frame
	var maps: Array[FuncGodotMap] = [map]
	var session := GodotTrenchLiveSession.new(MAP, maps)
	t.check(session.begin(text), "the live session takes the map")
	var lamp_json: Dictionary = session._nodes[15].duplicate(true)
	lamp_json["properties"]["light_energy"] = "4"
	session.apply([{ "op": "set", "id": 15, "parent": 1, "index": 8, "node": lamp_json }])
	session.apply([{ "op": "translate", "ids": [7], "offset": [0.0, 32.0, 0.0] }])
	session.apply([{ "op": "properties", "properties": { "classname": "worldspawn", "sun_energy": "0.5" } }])
	session.process()
	t.check(is_instance_valid(decor) and is_instance_valid(probe) and decor.get_parent() == overlay, "entity, world chunk and environment live rebuilds keep the overlay")
	var button: Dictionary = session._nodes[11].duplicate(true)
	button["outputs"] = [{ "output": "pressed", "target": "overlay_probe", "input": "record", "parameter": "3" }]
	session.apply([{ "op": "set", "id": 11, "parent": 1, "index": 7, "node": button }])
	session.process()
	await t.process_frame
	GodotTrenchIO.fire_output(_by_targetname(map, "button1"), &"pressed", null)
	t.check(probe.calls == [["record", 3, null]], "an output added live reaches the overlay, got %s" % [probe.calls])
	session.apply([{ "op": "set", "id": 17, "parent": 1, "index": 10, "node": session._nodes[17].duplicate(true) }])
	session._last_change = -1000000
	session.process()
	t.check(is_instance_valid(decor) and t.collect(map, func(n): return n is GodotTrenchOverlay).size() == 1, "a live full build keeps the overlay once")

	var reloader := GodotTrenchHotReload.new()
	reloader.port = 7899
	t.root.add_child(reloader)
	var rebuilt := reloader.reload(ProjectSettings.globalize_path(MAP))
	t.check(rebuilt >= 1 and is_instance_valid(decor) and decor.get_parent() == overlay, "hot reload rebuilds the map and keeps the overlay")
	reloader.free()
	map.owner.free()

static func _test_external_overlay(t) -> void:
	var map := _level(t)
	var level := map.owner
	var overlay := GodotTrenchOverlay.new()
	overlay.name = "ElsewhereOverlay"
	overlay.share_with_editor = false
	level.add_child(overlay)
	overlay.owner = level
	overlay.map_path = overlay.get_path_to(map)
	var probe := _add(overlay, PROBE.new(), "Probe")
	_io(probe, "far_probe")
	var pad := _add(overlay, Area3D.new(), "Pad") as Area3D
	_output(pad, "body_entered", "lamp", "turn_on")
	map.position = Vector3(10, 0, 0)
	await t.process_frame
	t.check(overlay.global_transform.is_equal_approx(map.global_transform), "an overlay pointing at its map copies the map transform")
	var button := _by_targetname(map, "button1")
	t.check(GodotTrenchIO.find_targets(button, "far_probe", null) == [probe], "map entities find targets in an overlay outside the map")
	var body := StaticBody3D.new()
	pad.body_entered.emit(body)
	t.check((_by_targetname(map, "lamp") as GTLight).is_on(), "nodes in an overlay outside the map reach map entities")
	body.free()
	level.free()

static func _test_streamer_leaves_overlay_alone(t) -> void:
	var map := FuncGodotMap.new()
	var overlay := GodotTrenchOverlay.new()
	map.add_child(overlay)
	var big := MeshInstance3D.new()
	var plane := PlaneMesh.new()
	plane.size = Vector2(200, 200)
	var arrays := plane.get_mesh_arrays()
	var mesh := ArrayMesh.new()
	mesh.add_surface_from_arrays(Mesh.PRIMITIVE_TRIANGLES, arrays)
	big.mesh = mesh
	overlay.add_child(big)
	GodotTrenchStreamer.split_visuals(map, 16.0)
	t.check(big.get_parent() == overlay and big.mesh == mesh, "chunk streaming does not split overlay meshes")
	map.free()

static func _test_sidecar(t) -> void:
	var dir := OS.get_temp_dir().path_join("gt_overlay_test")
	DirAccess.make_dir_recursive_absolute(dir)
	var map_file := dir.path_join("overlay_map.gtm")
	FileAccess.open(map_file, FileAccess.WRITE).store_string(FileAccess.get_file_as_string(MAP))
	FileAccess.open(dir.path_join("prefab.gtm"), FileAccess.WRITE).store_string(FileAccess.get_file_as_string("res://tests/maps/prefab.gtm"))
	var sidecar := dir.path_join("overlay_map.overlay.json")
	if FileAccess.file_exists(sidecar):
		DirAccess.remove_absolute(sidecar)
	var level := Node3D.new()
	t.root.add_child(level)
	var map := FuncGodotMap.new()
	map.name = "Map"
	map.map_settings = load(SETTINGS)
	map.global_map_file = map_file
	level.add_child(map)
	map.build()
	var overlay := _overlay(map)
	var crate := _add(overlay, MeshInstance3D.new(), "Crate") as MeshInstance3D
	var box := BoxMesh.new()
	box.size = Vector3(1, 2, 1)
	crate.mesh = box
	crate.position = Vector3(2, 1, -3)
	_io(crate, "crate")
	var lamp := _add(overlay, OmniLight3D.new(), "Lamp") as OmniLight3D
	lamp.position = Vector3(0, 4, 0)
	var anchor := _add(overlay, GodotTrenchAnchor.new(), "OnDoor") as GodotTrenchAnchor
	anchor.target = "door1"
	t.check(GodotTrenchOverlay.sidecar_path(map) == sidecar, "the sidecar sits next to the map")
	t.check(overlay.write_sidecar() == sidecar, "the overlay writes its sidecar")
	var data = JSON.parse_string(FileAccess.get_file_as_string(sidecar))
	t.check(data is Dictionary and data.get("format") == GodotTrenchOverlay.SIDECAR_FORMAT and data.get("map") == "overlay_map.gtm" and int(data.get("units_per_meter", 0)) == 32, "the sidecar names its format, map and scale")
	var items: Array = data["overlays"][0]["items"] if data is Dictionary else []
	var by_name := {}
	for item in items:
		by_name[item["name"]] = item
	t.check(by_name.has("Crate") and by_name["Crate"]["min"] == [48.0, 0.0, -112.0] and by_name["Crate"]["max"] == [80.0, 64.0, -80.0], "item bounds are in map units, got %s" % [by_name.get("Crate")])
	t.check(by_name.has("Crate") and by_name["Crate"].get("targetnames") == ["crate"], "items list their targetnames")
	t.check(by_name.has("Lamp") and by_name["Lamp"]["min"] == [0.0, 128.0, 0.0] and by_name["Lamp"]["max"] == [0.0, 128.0, 0.0], "a light is the point it sits at, not its range")
	t.check(by_name.has("OnDoor") and by_name["OnDoor"].get("anchor") == "door1", "anchored items name their entity")
	var other := _overlay(map)
	other.name = "Other"
	_add(other, Node3D.new(), "Marker")
	other.write_sidecar()
	overlay.write_sidecar()
	data = JSON.parse_string(FileAccess.get_file_as_string(sidecar))
	t.check(data["overlays"].size() == 2, "overlays of one map share the sidecar, got %d entries" % data["overlays"].size())
	level.free()

## The demo overlay of the night district: its breaker switches the courtyard lamps, the courtyard trigger lights its
## string lights and fireflies, and its garage sign rides the roller door. All of it through three builds.
static func _test_night_district_overlay(t) -> void:
	var level := Node3D.new()
	level.name = "NightDistrict"
	t.root.add_child(level)
	var map := FuncGodotMap.new()
	map.name = "Map"
	map.map_settings = load(SETTINGS)
	map.local_map_file = "res://demo/maps/showcase/night_district.gtm"
	level.add_child(map)
	var overlay_scene := load("res://demo/overlays/night_district_overlay.tscn") as PackedScene
	t.check(overlay_scene != null, "the night district overlay scene loads")
	if not overlay_scene:
		level.free()
		return
	var overlay := overlay_scene.instantiate() as GodotTrenchOverlay
	overlay.share_with_editor = false
	map.add_child(overlay)
	var breaker := overlay.get_node("Breaker")
	var strings := overlay.get_node("CourtStringLights")
	var fireflies := overlay.get_node("CourtFireflies") as GPUParticles3D
	var garage_sign := overlay.get_node("GarageSign") as GodotTrenchAnchor
	for attempt in 3:
		map.build()
		await t.process_frame
		await t.process_frame
		t.check(overlay.get_parent() == map and map.get_child(0) == overlay and t.collect(map, func(n): return n is GodotTrenchOverlay).size() == 1, "round %d: the demo overlay is kept once" % attempt)
		var lamps: Array = ["court_lamp_1", "court_lamp_2", "court_lamp_3"].map(func(n): return _by_targetname(map, n))
		t.check(lamps.all(func(l): return l is GTLight and not l.is_on()), "round %d: the courtyard lamps start off" % attempt)
		breaker.use(null)
		t.check(lamps.all(func(l): return l.is_on()), "round %d: the overlay breaker switches the map's courtyard lamps on" % attempt)
		breaker.use(null)
		t.check(lamps.all(func(l): return not l.is_on()), "round %d: and off again" % attempt)
		strings.reset()
		fireflies.emitting = false
		var trigger := _by_targetname(map, "court_entry")
		GodotTrenchIO.fire_output(trigger, &"triggered", null)
		t.check(fireflies.emitting, "round %d: the courtyard trigger starts the overlay fireflies" % attempt)
		await t.create_timer(2.2).timeout
		t.check(strings.is_on(), "round %d: the courtyard trigger lights the overlay string lights" % attempt)
		var door := _by_targetname(map, "garage_door") as Node3D
		t.check(garage_sign.refresh() and garage_sign.global_position.is_equal_approx(door.global_transform * garage_sign.offset.origin), "round %d: the garage sign sits on the roller door" % attempt)
		door.position.y += 1.5
		await t.process_frame
		t.check(garage_sign.global_position.is_equal_approx(door.global_transform * garage_sign.offset.origin), "round %d: the garage sign rides the door as it opens" % attempt)
	breaker.use(null)
	map.build()
	var rebuilt: Array = ["court_lamp_1", "court_lamp_2", "court_lamp_3"].map(func(n): return _by_targetname(map, n))
	t.check(rebuilt.all(func(l): return l is GTLight and l.is_on()), "a breaker left on switches the rebuilt lamps on again through map_rebuilt")
	level.free()
