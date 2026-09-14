extends SceneTree
## Renders the built showcase scenes from demo/maps/showcase/viewpoints.json.
## godot --path godot --script res://tests/showcase_screenshots.gd -- <output folder> [map name]

func _initialize() -> void:
	var args := OS.get_cmdline_user_args()
	var out_dir := args[0] if args.size() > 0 else OS.get_user_data_dir()
	var only := args[1] if args.size() > 1 else ""
	DirAccess.make_dir_recursive_absolute(out_dir)
	root.size = Vector2i(1280, 720)
	var views: Dictionary = JSON.parse_string(FileAccess.get_file_as_string("res://demo/maps/showcase/viewpoints.json"))
	for name in views.keys():
		if only != "" and only != name:
			continue
		var scene: PackedScene = load("res://demo/showcase/%s.tscn" % name)
		var instance := scene.instantiate()
		root.add_child(instance)
		var camera := Camera3D.new()
		camera.far = 1200.0
		camera.fov = 70.0
		instance.add_child(camera)
		camera.make_current()
		for view in views[name]["views"]:
			var p: Array = view["position"]
			var t: Array = view["target"]
			camera.look_at_from_position(Vector3(p[0], p[1], p[2]) / 32.0, Vector3(t[0], t[1], t[2]) / 32.0)
			for i in 30:
				await process_frame
			var image := root.get_viewport().get_texture().get_image()
			var path := out_dir.path_join("%s_%s_godot.png" % [name, view["name"]])
			image.save_png(path)
			print("saved ", path)
		instance.queue_free()
		await process_frame
	quit()
