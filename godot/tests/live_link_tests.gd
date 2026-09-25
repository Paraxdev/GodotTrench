extends RefCounted
## The live link server: an editor whose port is taken takes it over once it is free, and the live link follows scene
## tabs in a real headless Godot editor, see res://tests/editor/live_link_tabs.gd.
##
## res://tests/run_tests.gd calls [method run], [param t] is that script's instance for its check helpers.

const TABS_SCENE := "res://tests/editor/live_link_tabs.tscn"
const TABS_TIMEOUT_MSEC := 300000
## The editor saves its open scenes and recent files here, a test run must not change what a person sees next time.
const EDITOR_STATE := ["res://.godot/editor/editor_layout.cfg", "res://.godot/editor/project_metadata.cfg"]

static func run(t) -> void:
	print("- live link port and scene tabs")
	_test_port_taken_over(t)
	_test_settings_apply_at_once(t)
	await _test_scene_tabs(t)

static func _test_port_taken_over(t) -> void:
	var first := GodotTrenchEditorIntegration.new()
	first.start_live_link(0)
	var port := first._server.get_local_port() if first.is_listening() else 0
	var second := GodotTrenchEditorIntegration.new()
	second.start_live_link(port)
	t.check(port > 0 and not second.is_listening(), "a second editor cannot listen on a port in use")
	second._next_listen = 0
	second._process(0.0)
	t.check(not second.is_listening(), "it keeps trying while the port is taken")
	first.stop_live_link()
	second._next_listen = 0
	second._process(0.0)
	t.check(second.is_listening(), "it takes the port over once the first editor is gone")
	second.stop_live_link()
	first.free()
	second.free()

static func _test_settings_apply_at_once(t) -> void:
	var probe := TCPServer.new()
	probe.listen(0, "127.0.0.1")
	var port := probe.get_local_port()
	probe.stop()
	var before := [ProjectSettings.get_setting(GodotTrenchEditorIntegration.SETTING_PORT), ProjectSettings.get_setting(GodotTrenchEditorIntegration.SETTING_LIVE_LINK)]
	var link := GodotTrenchEditorIntegration.new()
	ProjectSettings.set_setting(GodotTrenchEditorIntegration.SETTING_PORT, port)
	link.apply_link_settings()
	t.check(link.is_listening() and link._server.get_local_port() == port, "the live link listens on the port from the project settings")
	ProjectSettings.set_setting(GodotTrenchEditorIntegration.SETTING_LIVE_LINK, false)
	link.apply_link_settings()
	t.check(not link.is_listening(), "turning the live link off stops it without a restart")
	ProjectSettings.set_setting(GodotTrenchEditorIntegration.SETTING_PORT, before[0])
	ProjectSettings.set_setting(GodotTrenchEditorIntegration.SETTING_LIVE_LINK, before[1])
	link.stop_live_link()
	link.free()

static func _test_scene_tabs(t) -> void:
	var saved := {}
	for path in EDITOR_STATE:
		if FileAccess.file_exists(path):
			saved[path] = FileAccess.get_file_as_bytes(path)
	var args := ["--headless", "--editor", "--path", ProjectSettings.globalize_path("res://"), TABS_SCENE, "--", "--gt-live-link-tabs"]
	var pid := OS.create_process(OS.get_executable_path(), args)
	var start := Time.get_ticks_msec()
	while pid > 0 and OS.is_process_running(pid) and Time.get_ticks_msec() - start < TABS_TIMEOUT_MSEC:
		await t.create_timer(0.2).timeout
	if pid > 0 and OS.is_process_running(pid):
		OS.kill(pid)
		t.check(false, "the headless editor scene tab test timed out")
	else:
		var code := OS.get_process_exit_code(pid) if pid > 0 else -1
		t.check(code == 0, "the live link follows scene tabs in a headless editor, exit code %d, see its output above" % code)
	for path in EDITOR_STATE:
		if saved.has(path):
			var file := FileAccess.open(path, FileAccess.WRITE)
			file.store_buffer(saved[path])
		elif FileAccess.file_exists(path):
			DirAccess.remove_absolute(ProjectSettings.globalize_path(path))
