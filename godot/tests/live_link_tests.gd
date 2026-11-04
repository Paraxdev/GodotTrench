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
	_test_play(t)
	await _test_foreign_lines(t)
	await _test_hot_reload_lines(t)
	await _test_scene_tabs(t)

static func _client(port: int) -> StreamPeerTCP:
	var client := StreamPeerTCP.new()
	client.connect_to_host("127.0.0.1", port)
	return client

## Runs [param link] (null for one in the tree) and polls [param clients] for some frames, or until [param done]
## holds. Returns the text each client received.
static func _pump(t, link: Node, clients: Array, done := Callable(), frames := 60) -> Array:
	var got := []
	got.resize(clients.size())
	got.fill("")
	for f in frames:
		if link:
			link._process(0.0)
		for i in clients.size():
			var c: StreamPeerTCP = clients[i]
			c.poll()
			if c.get_status() == StreamPeerTCP.STATUS_CONNECTED and c.get_available_bytes() > 0:
				got[i] += c.get_utf8_string(c.get_available_bytes())
		if done.is_valid() and done.call(got):
			break
		await t.process_frame
	return got

## A web page can post to the link port, its request headers come first. The peer is dropped at the first line that
## is not a JSON object, so the build line after them is never handled.
static func _test_foreign_lines(t) -> void:
	var link := TrenchDotEditorIntegration.new()
	link.start_live_link(0)
	var port := link._server.get_local_port()
	var good := _client(port)
	var page := _client(port)
	await _pump(t, link, [good, page], func(_got): return good.get_status() == StreamPeerTCP.STATUS_CONNECTED and page.get_status() == StreamPeerTCP.STATUS_CONNECTED and link._peers.size() == 2)
	good.put_data((JSON.stringify({ "event": "hello", "seq": 1 }) + "\n").to_utf8_buffer())
	var build := JSON.stringify({ "event": "build", "path": ProjectSettings.globalize_path("res://tests/maps/basic.tdm"), "text": "{}", "seq": 2 })
	var hello := JSON.stringify({ "event": "hello", "seq": 3 })
	page.put_data(("POST /live HTTP/1.1\r\nHost: 127.0.0.1:%d\r\nContent-Type: text/plain\r\n\r\n%s\n%s\n" % [port, build, hello]).to_utf8_buffer())
	var got := await _pump(t, link, [good, page], func(got): return got[0].contains("\n") and link._peers.size() == 1)
	var reply = JSON.parse_string(got[0].get_slice("\n", 0))
	t.check(reply is Dictionary and reply.get("ok") == true and reply.get("seq") == 1, "a JSON line is answered, got %s" % [got[0]])
	t.check(link._peers.size() == 1 and link._peers[0] != null and got[1] == "", "a peer sending HTTP headers is dropped unanswered, got %s" % [got[1]])

	var flood := _client(port)
	await _pump(t, link, [flood], func(_got): return flood.get_status() == StreamPeerTCP.STATUS_CONNECTED and link._peers.size() == 2)
	var peer: StreamPeerTCP = link._peers.back()
	var full := PackedByteArray()
	full.resize(TrenchDotEditorIntegration.MAX_BUFFER)
	full.fill(0x7b)
	link._buffers[peer] = full
	flood.put_data("{".to_utf8_buffer())
	await _pump(t, link, [flood], func(_got): return link._peers.size() == 1)
	t.check(link._peers.size() == 1 and not link._buffers.has(peer), "a peer whose unfinished line passes %d bytes is dropped" % TrenchDotEditorIntegration.MAX_BUFFER)
	link.stop_live_link()
	link.free()

## The running game's hot reload: saves of one map in the same frame build it once, and a foreign line or a line too
## long drops the peer.
static func _test_hot_reload_lines(t) -> void:
	var probe := TCPServer.new()
	probe.listen(0, "127.0.0.1")
	var port := probe.get_local_port()
	probe.stop()
	var hot := TrenchDotHotReload.new()
	hot.port = port
	t.root.add_child(hot)
	var map := FuncGodotMap.new()
	map.map_settings = load("res://demo/demo_map_settings.tres")
	map.local_map_file = "res://tests/maps/basic.tdm"
	t.root.add_child(map)
	var builds := [0]
	hot.map_reloaded.connect(func(_m): builds[0] += 1)
	var saved := JSON.stringify({ "event": "map_saved", "path": ProjectSettings.globalize_path(map.local_map_file) }) + "\n"
	var editor := _client(port)
	var page := _client(port)
	var flood := _client(port)
	await _pump(t, null, [editor, page, flood], func(_got): return [editor, page, flood].all(func(c): return c.get_status() == StreamPeerTCP.STATUS_CONNECTED) and hot._peers.size() == 3)
	editor.put_data((saved + saved + saved).to_utf8_buffer())
	page.put_data(("GET / HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n" + saved).to_utf8_buffer())
	# In two halves, a write the reader in this same thread cannot keep up with would block.
	var half := PackedByteArray()
	half.resize(TrenchDotHotReload.MAX_BUFFER / 2 + 1)
	half.fill(0x7b)
	flood.put_data(half)
	await _pump(t, null, [flood], Callable(), 3)
	flood.put_data(half)
	var got := await _pump(t, null, [editor, page, flood], func(got): return got[0].count("\n") == 3 and hot._peers.size() == 1)
	t.check(builds[0] == 1, "three saves of one map in a frame rebuild it once, got %d" % builds[0])
	var replies: Array = Array(got[0].strip_edges().split("\n")).map(func(l): return JSON.parse_string(l))
	t.check(replies.size() == 3 and replies.all(func(r): return r is Dictionary and r.get("rebuilt") == 1.0), "every save is answered, got %s" % [got[0]])
	t.check(hot._peers.size() == 1 and got[1] == "" and got[2] == "", "a peer with HTTP headers or a line too long is dropped, %d peers left" % hot._peers.size())
	map.free()
	hot.free()

static func _test_port_taken_over(t) -> void:
	var first := TrenchDotEditorIntegration.new()
	first.start_live_link(0)
	var port := first._server.get_local_port() if first.is_listening() else 0
	var second := TrenchDotEditorIntegration.new()
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
	var before := [ProjectSettings.get_setting(TrenchDotEditorIntegration.SETTING_PORT), ProjectSettings.get_setting(TrenchDotEditorIntegration.SETTING_LIVE_LINK)]
	var link := TrenchDotEditorIntegration.new()
	ProjectSettings.set_setting(TrenchDotEditorIntegration.SETTING_PORT, port)
	link.apply_link_settings()
	t.check(link.is_listening() and link._server.get_local_port() == port, "the live link listens on the port from the project settings")
	ProjectSettings.set_setting(TrenchDotEditorIntegration.SETTING_LIVE_LINK, false)
	link.apply_link_settings()
	t.check(not link.is_listening(), "turning the live link off stops it without a restart")
	ProjectSettings.set_setting(TrenchDotEditorIntegration.SETTING_PORT, before[0])
	ProjectSettings.set_setting(TrenchDotEditorIntegration.SETTING_LIVE_LINK, before[1])
	link.stop_live_link()
	link.free()

## Playing needs a main scene that exists, and only the Godot editor plays.
static func _test_play(t) -> void:
	const MAIN := "application/run/main_scene"
	var main: Variant = ProjectSettings.get_setting(MAIN)
	var link := TrenchDotEditorIntegration.new()
	var play := func() -> Dictionary: return link._handle({ "event": "play" })
	ProjectSettings.set_setting(MAIN, "")
	var reply: Dictionary = play.call()
	t.check(reply.get("ok") == false and str(reply.get("error")).contains("no main scene"), "play without a main scene says so, got %s" % reply)
	ProjectSettings.set_setting(MAIN, "res://tests/td_tests_no_such_scene.tscn")
	reply = play.call()
	t.check(reply.get("ok") == false and str(reply.get("error")).contains("res://tests/td_tests_no_such_scene.tscn"), "play names a main scene that does not exist, got %s" % reply)
	ProjectSettings.set_setting(MAIN, main)
	reply = play.call()
	t.check(reply.get("ok") == false and str(reply.get("error")).contains("Godot editor"), "outside the editor nothing is played, got %s" % reply)
	link.free()

static func _test_scene_tabs(t) -> void:
	var saved := {}
	for path in EDITOR_STATE:
		if FileAccess.file_exists(path):
			saved[path] = FileAccess.get_file_as_bytes(path)
	var args := ["--headless", "--editor", "--path", ProjectSettings.globalize_path("res://"), TABS_SCENE, "--", "--td-live-link-tabs"]
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
