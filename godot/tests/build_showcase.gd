extends SceneTree
## Builds the showcase maps into ready to play scenes (the environment comes from worldspawn keys).
## godot --headless --path godot --script res://tests/build_showcase.gd [-- map_name ...]
## Names after -- build only those maps, the others keep their saved scenes.
## A map with a Godot overlay scene (res://demo/overlays/<map>_overlay.tscn) gets it on top, instanced before the build
## so the build keeps it, and its overlay sidecar is written next to the map for GodotTrench.

const SETTINGS := "res://demo/demo_map_settings.tres"
const MAPS := ["mountain_house", "church_school", "lighthouse_forest", "withered_city", "night_district"]

func reown(node: Node, old_owner: Node, new_owner: Node) -> void:
	for child in node.get_children():
		if child.owner == old_owner or child.owner == null:
			child.owner = new_owner
		reown(child, old_owner, new_owner)

func count(node: Node, predicate: Callable) -> int:
	var n := 1 if predicate.call(node) else 0
	for c in node.get_children():
		n += count(c, predicate)
	return n

func _initialize() -> void:
	var failures := 0
	var only := OS.get_cmdline_user_args()
	for name in MAPS:
		if not only.is_empty() and not only.has(name):
			continue
		var started := Time.get_ticks_msec()
		var map_file := "res://demo/maps/showcase/%s.gtm" % name
		var root := Node3D.new()
		root.name = name.to_pascal_case()

		var map := FuncGodotMap.new()
		map.name = "Map"
		map.map_settings = load(SETTINGS)
		map.local_map_file = map_file
		root.add_child(map)
		map.owner = root
		var overlay_file := "res://demo/overlays/%s_overlay.tscn" % name
		var overlay: GodotTrenchOverlay = null
		if ResourceLoader.exists(overlay_file):
			overlay = (load(overlay_file) as PackedScene).instantiate() as GodotTrenchOverlay
			map.add_child(overlay)
			overlay.owner = root
		map.build()
		reown(map, map, root)
		if overlay:
			print("%s: overlay %s, sidecar %s" % [name, overlay_file, overlay.write_sidecar()])

		var meshes := count(map, func(n): return n is MeshInstance3D)
		var terrains := count(map, func(n): return n is GodotTrenchTerrain)
		var scatters := count(map, func(n): return n is GodotTrenchScatter)
		var instances := count(map, func(n): return n is MultiMeshInstance3D)
		var lights := count(map, func(n): return n is Light3D)
		var environments := count(map, func(n): return n is WorldEnvironment)
		var packed := PackedScene.new()
		var err := packed.pack(root)
		if err == OK:
			err = ResourceSaver.save(packed, "res://demo/showcase/%s.scn" % name)
		print("%s: %d mesh instances, %d terrains, %d scatter sets (%d multimeshes), %d lights, built in %.1f s, save %s" % [name, meshes, terrains, scatters, instances, lights, (Time.get_ticks_msec() - started) / 1000.0, error_string(err)])
		if err != OK or terrains != 1 or scatters < 2 or instances < 4 or meshes < 10 or environments != 1:
			failures += 1
		root.free()
	quit(1 if failures > 0 else 0)
