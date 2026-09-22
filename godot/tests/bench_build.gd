extends SceneTree
## Times every step of building the showcase maps, without saving anything.
## godot --headless --path godot --script res://tests/bench_build.gd [-- runs=3 threaded=1]

const SETTINGS := "res://demo/demo_map_settings.tres"
const MAPS := ["mountain_house", "church_school", "lighthouse_forest", "withered_city"]

var _marks: Array = []

func _mark(step: String) -> void:
	_marks.append([step, Time.get_ticks_usec()])

func _initialize() -> void:
	var runs := 3
	for arg in OS.get_cmdline_user_args():
		if arg.begins_with("runs="):
			runs = int(arg.trim_prefix("runs="))
		elif arg.begins_with("threaded="):
			ProjectSettings.set_setting(GodotTrenchBuild.SETTING_THREADED, arg.trim_prefix("threaded=") == "1")
	print("threaded build: %s" % GodotTrenchBuild.threaded())
	var settings: FuncGodotMapSettings = load(SETTINGS)
	for map_name in MAPS:
		var totals := {}
		var order: Array[String] = []
		var whole: Array[float] = []
		for run in runs:
			var map := FuncGodotMap.new()
			map.map_settings = settings
			map.local_map_file = "res://demo/maps/showcase/%s.gtm" % map_name
			root.add_child(map)
			_marks.clear()
			var started := Time.get_ticks_usec()
			_mark("parse")
			var parser := FuncGodotParser.new()
			parser.declare_step.connect(func(step: String) -> void: _mark("parse: " + step.split(" ")[0] + " " + step.split(" ")[1] if step.split(" ").size() > 1 else step))
			var parse_data := parser.parse_map_data(map.local_map_file, settings)
			var generator := FuncGodotGeometryGenerator.new(settings, map.hyperplane_size)
			generator.declare_step.connect(_mark)
			generator.build(map.build_flags, parse_data.entities)
			var assembler := FuncGodotEntityAssembler.new(settings)
			_mark("assemble entities")
			assembler.build(map, parse_data.entities, parse_data.groups)
			_mark("terrains")
			GodotTrenchTerrain.build_all(map, parse_data.terrains, settings)
			_mark("scatter sets")
			GodotTrenchScatter.build_all(map, parse_data.scatters, settings)
			_mark("environment")
			GodotTrenchEnvironment.build(map, parse_data.entities[0].properties)
			_mark("end")
			whole.append((Time.get_ticks_usec() - started) / 1000.0)
			for i in _marks.size() - 1:
				var step: String = _marks[i][0]
				if not totals.has(step):
					totals[step] = []
					order.append(step)
				totals[step].append((_marks[i + 1][1] - _marks[i][1]) / 1000.0)
			map.free()
		whole.sort()
		print("%s: %.0f ms (median of %d)" % [map_name, whole[whole.size() / 2], runs])
		for step in order:
			var times: Array = totals[step]
			times.sort()
			var median: float = times[times.size() / 2]
			if median >= 1.0:
				print("    %7.1f ms  %s" % [median, step])
	quit()
