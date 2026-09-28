extends RefCounted
## Tests for the math and logic entities of the Gameplay pack, run from run_tests.gd:
## await load("res://tests/logic_math_tests.gd").new().run(self)
## Inputs go through GodotTrenchIO.invoke with the text a map would send, and outputs reach a probe through real
## GodotTrenchOutput connections.

const PROBE := "res://tests/helpers/io_probe.gd"

## The run_tests.gd runner, untyped so its check() is reachable.
var t

func check(cond: bool, what: String) -> void:
	t.check(cond, what)

func run(runner: SceneTree) -> void:
	t = runner
	test_number_helpers()
	await test_calc()
	await test_calc_wiring()
	await test_compare()
	await test_compare_wiring()
	await test_value()
	await test_value_wiring()
	await test_gate()
	await test_random()
	await test_case()
	await test_flipflop()
	await test_built_map()

## A map holding a probe named "probe" that records the inputs it gets, and one named "watch" for a second stream.
func _bench() -> Array:
	var map := Node3D.new()
	t.root.add_child(map)
	var probes: Array = [map]
	for name in ["probe", "watch"]:
		var probe: Node3D = load(PROBE).new()
		probe.set_meta(GodotTrenchIO.TARGETNAME_META, name)
		map.add_child(probe)
		probes.append(probe)
	return probes

func _entity(map: Node, entity: Node, targetname: String, props := {}) -> Node:
	entity.set_meta(GodotTrenchIO.TARGETNAME_META, targetname)
	entity._func_godot_apply_properties(props)
	map.add_child(entity)
	GodotTrenchIO.invalidate(map)
	return entity

func _wire(source: Node, output: String, target: String, input: String, parameter := "") -> void:
	var out := GodotTrenchOutput.new()
	out.output = StringName(output)
	out.target = target
	out.input = StringName(input)
	out.parameter = parameter
	source.add_child(out)

## Each of [param outputs] reaches the probe as record with its own name as the parameter.
func _log(source: Node, outputs: Array) -> void:
	for output in outputs:
		_wire(source, output, "probe", "record", output)

func _send(entity: Node, input: String, parameter := "", activator: Node = null) -> void:
	GodotTrenchIO.invoke(entity, StringName(input), parameter, activator)

func _tags(probe: Node) -> Array:
	return probe.calls.map(func(c): return c[1])

func _same(got: Variant, want: Variant) -> bool:
	return typeof(got) == typeof(want) and got == want

func _gate_result(kind: String, a: bool, b: bool) -> bool:
	match kind:
		"or":
			return a or b
		"xor":
			return a != b
		"nand":
			return not (a and b)
		"nor":
			return not (a or b)
		"xnor":
			return a == b
		"not":
			return not a
	return a and b

func _free(map: Node) -> void:
	map.queue_free()
	await t.process_frame

func test_number_helpers() -> void:
	print("- math helpers read numbers and text")
	check(_same(GTMath.to_number(5), 5) and _same(GTMath.to_number("2.5"), 2.5) and _same(GTMath.to_number(" 7 "), 7), "numbers and numeric text")
	check(_same(GTMath.to_number(2.0), 2) and _same(GTMath.to_number(true), 1) and _same(GTMath.to_number(false), 0), "whole floats are ints and a bool is 1 or 0")
	check(GTMath.to_number("abc") == null and GTMath.to_number(null) == null and _same(GTMath.to_number("abc", 9), 9), "anything else is the fallback")
	check(GTMath.to_number(INF) == null and GTMath.to_number(NAN) == null and GTMath.to_number("nan") == null, "infinity and nan are not numbers")
	check(_same(GTMath.tidy(1.0e20), 1.0e20) and _same(GTMath.tidy(-3.0), -3) and GTMath.tidy(NAN) == null, "tidy keeps huge whole floats as floats")
	var node := Node3D.new()
	check(GTMath.normalize(null) == null and GTMath.normalize(node) == null, "nothing and a node are not stored")
	node.free()
	check(_same(GTMath.normalize("x"), "x") and _same(GTMath.normalize("3"), 3), "text stays text, numeric text becomes a number")
	check(GTMath.compare(1, 2) == -1 and GTMath.compare(2, 1) == 1 and GTMath.compare(2, 2.0) == 0, "numbers compare by value")
	check(GTMath.compare("10", 9) == 1, "numeric text compares as a number, not alphabetically")
	check(GTMath.compare("apple", "banana") == -1 and GTMath.compare("b", "a") == 1 and GTMath.compare("a", "a") == 0, "text compares alphabetically")
	check(GTMath.compare(0.1 + 0.2, 0.3) == 0, "float noise is not a difference")
	check(GTMath.compare(100001, 100000) == 1, "whole numbers compare exactly")

func test_calc() -> void:
	print("- math_calc operations and edge cases")
	var bench := _bench()
	var map: Node3D = bench[0]
	var probe: Node3D = bench[1]
	var calc := GTCalc.new()
	_entity(map, calc, "calc")
	_wire(calc, "result", "probe", "record")
	# operation, operand, incoming value, expected result
	var rows := [
		["add", 1, "5", 6], ["subtract", 2, "5", 3], ["multiply", 3, "4", 12], ["divide", 2, "7", 3.5], ["divide", 2, "8", 4],
		["modulo", 3, "7", 1], ["modulo", 3, "-1", 2], ["power", 2, "10", 100], ["power", 0.5, "9", 3], ["min", 3, "5", 3],
		["max", 3, "5", 5], ["abs", 9, "-4", 4], ["negate", 9, "4", -4], ["round", 9, "2.5", 3], ["round", 9, "-2.5", -3],
		["floor", 9, "-1.5", -2], ["ceil", 9, "1.2", 2], ["add", 1, "2.5", 3.5], ["subtract", 0.5, "1", 0.5], ["multiply", 0, "8", 0],
	]
	for row in rows:
		calc._func_godot_apply_properties({ "operation": row[0], "operand": str(row[1]) })
		probe.calls.clear()
		_send(calc, "calculate", row[2])
		check(probe.calls.size() == 1 and _same(probe.calls[0][1], row[3]), "%s %s %s is %s, got %s" % [row[2], row[0], row[1], row[3], probe.calls])

	calc._func_godot_apply_properties({ "operation": " Add ", "operand": "1" })
	probe.calls.clear()
	_send(calc, "calculate", "")
	var who := Node3D.new()
	map.add_child(who)
	_send(calc, "calculate", "", who)
	_send(calc, "calculate", "abc")
	check(_tags(probe) == [1, 1, 1], "no value, an activator and text all count as 0, got %s" % [_tags(probe)])
	calc._func_godot_apply_properties({ "operand": "x" })
	_send(calc, "calculate", "1")
	check(probe.calls.back()[1] == 2, "an operand that is not a number keeps the old one")

	probe.calls.clear()
	for row in [["divide", 0, "5"], ["modulo", 0, "5"], ["power", -1, "0"], ["power", 0.5, "-8"], ["frobnicate", 1, "5"]]:
		calc._func_godot_apply_properties({ "operation": row[0], "operand": str(row[1]) })
		_send(calc, "calculate", row[2])
	check(probe.calls.is_empty(), "no answer, so nothing fires, for a division by zero, an impossible root or an unknown operation, got %s" % [probe.calls])
	calc._func_godot_apply_properties({ "operation": "divide", "operand": "2" })
	_send(calc, "calculate", "5")
	check(probe.calls.size() == 1 and probe.calls[0][1] == 2.5, "it keeps working after a refused calculation")
	await _free(map)

func test_calc_wiring() -> void:
	print("- math_calc passes its result along")
	var bench := _bench()
	var map: Node3D = bench[0]
	var probe: Node3D = bench[1]
	var counter := GTCounter.new()
	_entity(map, counter, "counter", { "min": 0, "max": 10, "start_value": 0 })
	var times := GTCalc.new()
	_entity(map, times, "times", { "operation": "multiply", "operand": "10" })
	var plus := GTCalc.new()
	_entity(map, plus, "plus", { "operation": "add", "operand": "1" })
	_wire(counter, "changed", "times", "calculate")
	_wire(times, "result", "plus", "calculate")
	_wire(plus, "result", "probe", "record")
	counter.add(3)
	check(probe.calls.size() == 1 and _same(probe.calls[0][1], 31), "changed(3) times 10 plus 1 reaches the probe as 31, got %s" % [probe.calls])
	_wire(counter, "hit_max", "plus", "calculate", "50")
	counter.set_value(10)
	check(_tags(probe).back() == 51, "a connection with its own parameter overrides the passed value, got %s" % [_tags(probe)])
	await _free(map)

func test_compare() -> void:
	print("- math_compare orders numbers and text")
	var bench := _bench()
	var map: Node3D = bench[0]
	var probe: Node3D = bench[1]
	var cmp := GTCompare.new()
	_entity(map, cmp, "cmp", { "compare_value": "5" })
	_log(cmp, ["on_less", "on_equal", "on_not_equal", "on_greater"])
	_send(cmp, "compare")
	check(_tags(probe) == ["on_less", "on_not_equal"], "a fresh comparison holds 0, got %s" % [_tags(probe)])
	var rows := [
		["3", ["on_less", "on_not_equal"]], ["5", ["on_equal"]], ["7", ["on_greater", "on_not_equal"]], ["5.0", ["on_equal"]],
		["10", ["on_greater", "on_not_equal"]], ["-2", ["on_less", "on_not_equal"]],
	]
	for row in rows:
		probe.calls.clear()
		_send(cmp, "set_and_compare", row[0])
		check(_tags(probe) == row[1], "%s against 5 fires %s, got %s" % [row[0], row[1], _tags(probe)])

	probe.calls.clear()
	_send(cmp, "set_value", "1")
	check(probe.calls.is_empty(), "set_value only stores")
	_send(cmp, "set_value", "")
	_send(cmp, "compare")
	check(_tags(probe) == ["on_less", "on_not_equal"], "an empty value keeps the stored 1, got %s" % [_tags(probe)])
	probe.calls.clear()
	_send(cmp, "set_compare_value", "1")
	_send(cmp, "compare")
	check(_tags(probe) == ["on_equal"], "set_compare_value changes what it compares with, got %s" % [_tags(probe)])
	probe.calls.clear()
	_send(cmp, "set_and_compare", "")
	check(_tags(probe) == ["on_equal"], "set_and_compare without a value compares the stored one, got %s" % [_tags(probe)])

	_send(cmp, "set_compare_value", "apple")
	for row in [["apple", ["on_equal"]], ["banana", ["on_greater", "on_not_equal"]], ["aardvark", ["on_less", "on_not_equal"]]]:
		probe.calls.clear()
		_send(cmp, "set_and_compare", row[0])
		check(_tags(probe) == row[1], "%s against apple fires %s, got %s" % [row[0], row[1], _tags(probe)])

	_send(cmp, "set_compare_value", "0.3")
	probe.calls.clear()
	_send(cmp, "set_and_compare", "0.30000000000000004")
	check(_tags(probe) == ["on_equal"], "float noise still counts as equal, got %s" % [_tags(probe)])
	_send(cmp, "set_compare_value", "100000")
	probe.calls.clear()
	_send(cmp, "set_and_compare", "100001")
	check(_tags(probe) == ["on_greater", "on_not_equal"], "whole numbers compare exactly, got %s" % [_tags(probe)])
	cmp._func_godot_apply_properties({ "compare_value": "-1" })
	cmp.value = 0
	probe.calls.clear()
	_send(cmp, "compare")
	check(_tags(probe) == ["on_greater", "on_not_equal"], "a negative compare_value, got %s" % [_tags(probe)])
	await _free(map)

func test_compare_wiring() -> void:
	print("- math_compare compares what a counter reports")
	var bench := _bench()
	var map: Node3D = bench[0]
	var probe: Node3D = bench[1]
	var counter := GTCounter.new()
	_entity(map, counter, "counter", { "min": 0, "max": 10, "start_value": 0 })
	var cmp := GTCompare.new()
	_entity(map, cmp, "cmp", { "compare_value": "3" })
	_wire(counter, "changed", "cmp", "set_and_compare")
	_log(cmp, ["on_less", "on_equal", "on_greater"])
	counter.add(2)
	counter.add(1)
	counter.add(1)
	check(_tags(probe) == ["on_less", "on_equal", "on_greater"], "changed(value) is what gets compared, got %s" % [_tags(probe)])
	await _free(map)

func test_value() -> void:
	print("- math_value stores numbers and text")
	var bench := _bench()
	var map: Node3D = bench[0]
	var probe: Node3D = bench[1]
	var watch: Node3D = bench[2]
	var stored := GTValue.new()
	_entity(map, stored, "score", { "start_value": "5" })
	_wire(stored, "value", "probe", "record")
	_wire(stored, "changed", "watch", "record")
	_send(stored, "get_value")
	check(probe.calls.size() == 1 and _same(probe.calls[0][1], 5), "get_value fires value with the stored number, got %s" % [probe.calls])
	check(watch.calls.is_empty(), "reading does not count as a change")
	for row in [["add", "", 6], ["add", "2.5", 8.5], ["add", "-8.5", 0], ["set_value", "42", 42], ["set_value", "hello", "hello"], ["add", "!", "hello!"],
			["set_value", "3", 3], ["add", "b", "3b"], ["set_value", "x", "x"], ["add", "1", "x1"], ["reset", "", 5]]:
		watch.calls.clear()
		_send(stored, row[0], row[1])
		check(watch.calls.size() == 1 and _same(watch.calls[0][1], row[2]), "%s %s gives %s, got %s" % [row[0], row[1], row[2], watch.calls])

	watch.calls.clear()
	_send(stored, "reset")
	_send(stored, "set_value", "5")
	_send(stored, "set_value", "")
	_send(stored, "add", "0")
	check(watch.calls.is_empty(), "nothing fires while the value stays the same, got %s" % [watch.calls])
	check(_same(stored.current, 5), "an empty set_value keeps the value")

	var text := GTValue.new()
	_entity(map, text, "message", { "start_value": "ready" })
	_wire(text, "changed", "watch", "record")
	_send(text, "add")
	_send(text, "reset")
	check(_tags(watch) == ["ready1", "ready"], "text starts as text, reset returns to it, got %s" % [_tags(watch)])
	var blank := GTValue.new()
	_entity(map, blank, "blank", {})
	check(_same(blank.current, 0), "a value without a start is 0")
	await _free(map)

func test_value_wiring() -> void:
	print("- math_value takes the values of other entities")
	var bench := _bench()
	var map: Node3D = bench[0]
	var watch: Node3D = bench[2]
	var counter := GTCounter.new()
	_entity(map, counter, "counter", { "min": 0, "max": 10, "start_value": 0 })
	var stored := GTValue.new()
	_entity(map, stored, "copy", { "start_value": "0" })
	_wire(counter, "changed", "copy", "set_value")
	_wire(stored, "changed", "watch", "record")
	counter.add(4)
	check(_same(stored.current, 4) and _tags(watch) == [4], "set_value stores what changed(value) passes, got %s" % [stored.current])
	var relay := GTRelay.new()
	_entity(map, relay, "relay")
	_wire(relay, "triggered", "copy", "set_value")
	var who := Node3D.new()
	map.add_child(who)
	relay.trigger(who)
	check(_same(stored.current, 4), "the activator of a relay is not a value")
	_wire(relay, "triggered", "copy", "add", "10")
	relay.trigger()
	check(_same(stored.current, 14), "a connection parameter is the amount to add, got %s" % [stored.current])
	await _free(map)

func test_gate() -> void:
	print("- logic_gate truth tables and change events")
	var bench := _bench()
	var map: Node3D = bench[0]
	var probe: Node3D = bench[1]
	var watch: Node3D = bench[2]
	var gate := GTGate.new()
	_entity(map, gate, "gate")
	_log(gate, ["on_true", "on_false"])
	_wire(gate, "changed", "watch", "record")
	for kind in ["and", "or", "xor", "nand", "nor", "xnor", "not"]:
		for a in [false, true]:
			for b in [false, true]:
				gate._func_godot_apply_properties({ "gate": kind, "start_a": a, "start_b": "1" if b else "0" })
				probe.calls.clear()
				_send(gate, "test")
				var want: String = "on_true" if _gate_result(kind, a, b) else "on_false"
				check(_tags(probe) == [want], "%s(%s, %s) fires %s, got %s" % [kind, a, b, want, _tags(probe)])
	check(probe.calls.size() == 1 and watch.calls.is_empty(), "starting values and test never fire changed")

	gate._func_godot_apply_properties({ "gate": "and", "start_a": false, "start_b": false })
	probe.calls.clear()
	watch.calls.clear()
	_send(gate, "set_a")
	check(probe.calls.is_empty() and watch.calls.is_empty(), "a alone leaves an and gate false, so nothing fires")
	_send(gate, "set_b", "true")
	check(_tags(probe) == ["on_true"] and _tags(watch) == [true], "b completes it, changed(true) and on_true, got %s and %s" % [_tags(probe), _tags(watch)])
	_send(gate, "set_b", "true")
	check(probe.calls.size() == 1, "setting the same value fires nothing more")
	_send(gate, "toggle_a")
	check(_tags(probe) == ["on_true", "on_false"] and _tags(watch) == [true, false], "toggle_a flips it back, got %s and %s" % [_tags(probe), _tags(watch)])
	for row in [["set_a", "1", "a", true], ["set_b", "0", "b", false], ["set_b", "5", "b", true], ["set_a", "no", "a", false], ["set_a", "yes", "a", true],
			["set_a", "false", "a", false]]:
		_send(gate, row[0], row[1])
		check(gate.get(row[2]) == row[3], "%s %s makes %s %s" % [row[0], row[1], row[2], row[3]])
	check(gate.b == true, "b is still true")
	_send(gate, "toggle_b")
	_send(gate, "toggle_b")
	check(gate.b == true, "toggle_b twice is where it started")

	gate._func_godot_apply_properties({ "gate": "not", "start_a": false })
	probe.calls.clear()
	watch.calls.clear()
	_send(gate, "set_b", "true")
	check(probe.calls.is_empty(), "not looks at a only")
	_send(gate, "set_a", "true")
	check(_tags(probe) == ["on_false"] and _tags(watch) == [false], "not turns false once a is true, got %s" % [_tags(probe)])

	gate._func_godot_apply_properties({ "gate": " OR ", "start_a": true, "start_b": false })
	probe.calls.clear()
	_send(gate, "test")
	check(_tags(probe) == ["on_true"], "the gate name ignores case and spaces")
	gate._func_godot_apply_properties({ "gate": "banana", "start_a": true, "start_b": false })
	probe.calls.clear()
	_send(gate, "test")
	check(_tags(probe) == ["on_false"], "an unknown gate acts as and")

	var relay := GTRelay.new()
	_entity(map, relay, "relay")
	gate._func_godot_apply_properties({ "gate": "and", "start_a": false, "start_b": true })
	_wire(relay, "triggered", "gate", "set_a", "true")
	probe.calls.clear()
	relay.trigger()
	check(_tags(probe) == ["on_true"], "a relay can set a through I/O with a parameter, got %s" % [_tags(probe)])
	await _free(map)

func test_random() -> void:
	print("- logic_random picks, never repeats on request and rolls a chance")
	var bench := _bench()
	var map: Node3D = bench[0]
	var probe: Node3D = bench[1]
	var watch: Node3D = bench[2]
	var rnd := GTRandom.new()
	_entity(map, rnd, "rnd", { "count": "3", "no_repeat": "0" })
	var outs := ["out_1", "out_2", "out_3", "out_4", "out_5", "out_6", "out_7", "out_8"]
	_log(rnd, outs + ["on_success", "on_fail"])
	_wire(rnd, "picked", "watch", "record")
	var picks := func(times: int) -> Array:
		probe.calls.clear()
		watch.calls.clear()
		for i in times:
			_send(rnd, "pick")
		return _tags(watch)
	var got: Array = picks.call(300)
	check(got.size() == 300 and got.all(func(i): return i >= 1 and i <= 3), "pick stays within out_1 to out_3")
	check(1 in got and 2 in got and 3 in got, "every output gets picked")
	check(_tags(probe) == got.map(func(i): return "out_%d" % i), "picked(index) names the output that fired")
	check(not _tags(probe).has("on_success") and not _tags(probe).has("on_fail"), "pick does not roll")

	for count in ["0", "-4"]:
		rnd._func_godot_apply_properties({ "count": count })
		got = picks.call(30)
		check(got.all(func(i): return i == 1), "count %s is raised to 1, got %s" % [count, got])
	rnd._func_godot_apply_properties({ "count": "99" })
	got = picks.call(400)
	check(got.all(func(i): return i >= 1 and i <= 8) and got.has(8), "count 99 is lowered to 8, and out_8 fires")
	rnd._func_godot_apply_properties({ "count": "many" })
	check(rnd.count == 8, "a count that is not a number keeps the old one")

	rnd._func_godot_apply_properties({ "count": "3", "no_repeat": "1" })
	got = picks.call(300)
	var repeats := 0
	for i in range(1, got.size()):
		repeats += 1 if got[i] == got[i - 1] else 0
	check(repeats == 0 and got.has(1) and got.has(2) and got.has(3), "no_repeat never picks the same output twice, and still uses every one, %d repeats" % repeats)
	rnd._func_godot_apply_properties({ "count": "2", "no_repeat": true })
	got = picks.call(20)
	var alternates := true
	for i in range(1, got.size()):
		alternates = alternates and got[i] != got[i - 1]
	check(alternates, "with two outputs no_repeat alternates, got %s" % [got])
	rnd._func_godot_apply_properties({ "count": "1", "no_repeat": true })
	got = picks.call(20)
	check(got.size() == 20 and got.all(func(i): return i == 1), "one output cannot avoid repeating and still fires")

	var rolls := func(chance: String, times: int) -> Array:
		rnd._func_godot_apply_properties({ "chance": chance })
		probe.calls.clear()
		for i in times:
			_send(rnd, "roll")
		return _tags(probe)
	check(rolls.call("0", 200).all(func(x): return x == "on_fail"), "0 percent never succeeds")
	check(rolls.call("100", 200).all(func(x): return x == "on_success"), "100 percent always succeeds")
	check(rolls.call("150", 50).all(func(x): return x == "on_success"), "a chance above 100 always succeeds")
	check(rolls.call("-5", 50).all(func(x): return x == "on_fail"), "a negative chance never succeeds")
	got = rolls.call("50", 300)
	check(got.has("on_success") and got.has("on_fail") and got.size() == 300, "50 percent gives both")
	check(not got.any(func(x): return x.begins_with("out_")), "roll does not pick")
	await _free(map)

func test_case() -> void:
	print("- logic_case fires the first matching case")
	var bench := _bench()
	var map: Node3D = bench[0]
	var probe: Node3D = bench[1]
	var sw := GTCase.new()
	_entity(map, sw, "case", { "case_1": "1", "case_2": "red", "case_3": "2", "case_4": "", "case_5": "1", "case_6": "big red", "case_8": "eight" })
	var outputs := ["on_default"]
	for i in range(1, 9):
		outputs.append("on_case_%d" % i)
	_log(sw, outputs)
	var rows := [
		["1", ["on_case_1"]], ["1.0", ["on_case_1"]], ["red", ["on_case_2"]], ["Red", ["on_default"]], ["2", ["on_case_3"]],
		["3", ["on_default"]], ["", ["on_default"]], ["red apple", ["on_default"]], ["big red", ["on_case_6"]], ["eight", ["on_case_8"]],
	]
	for row in rows:
		probe.calls.clear()
		_send(sw, "in_value", row[0])
		check(_tags(probe) == row[1], "'%s' fires %s, got %s" % [row[0], row[1], _tags(probe)])
	probe.calls.clear()
	sw.in_value(" 2 ")
	sw.in_value(2.0)
	sw.in_value(null)
	check(_tags(probe) == ["on_case_3", "on_case_3", "on_default"], "spaces and float values still match, and nothing is the default, got %s" % [_tags(probe)])
	probe.calls.clear()
	var who := Node3D.new()
	map.add_child(who)
	sw.in_value(who)
	check(_tags(probe) == ["on_default"], "a node is never a value")
	sw._func_godot_apply_properties({ "case_1": "", "case_3": "" })
	probe.calls.clear()
	_send(sw, "in_value", "1")
	check(_tags(probe) == ["on_case_5"], "changing the cases changes what matches, got %s" % [_tags(probe)])

	var stored := GTValue.new()
	_entity(map, stored, "score", { "start_value": "0" })
	_wire(stored, "changed", "case", "in_value")
	probe.calls.clear()
	stored.set_value(1)
	stored.set_value("red")
	check(_tags(probe) == ["on_case_5", "on_case_2"], "a value output feeds the case, got %s" % [_tags(probe)])
	await _free(map)

func test_flipflop() -> void:
	print("- logic_flipflop alternates and resets")
	var bench := _bench()
	var map: Node3D = bench[0]
	var probe: Node3D = bench[1]
	var watch: Node3D = bench[2]
	var flip := GTFlipFlop.new()
	_entity(map, flip, "flip")
	_log(flip, ["on_a", "on_b"])
	_wire(flip, "changed", "watch", "record")
	for i in 4:
		_send(flip, "trigger")
	check(_tags(probe) == ["on_a", "on_b", "on_a", "on_b"] and _tags(watch) == [true, false, true, false], "trigger alternates, got %s and %s" % [_tags(probe), _tags(watch)])
	probe.calls.clear()
	watch.calls.clear()
	_send(flip, "trigger")
	_send(flip, "reset")
	check(_tags(probe) == ["on_a"] and _tags(watch) == [true, false], "reset changes the state without on_a or on_b, got %s and %s" % [_tags(probe), _tags(watch)])
	_send(flip, "reset")
	check(watch.calls.size() == 2, "reset at the start fires nothing")
	var who := Node3D.new()
	map.add_child(who)
	_send(flip, "trigger", "", who)
	check(_tags(probe) == ["on_a", "on_a"], "the first trigger after reset is on_a again, an activator makes no difference")

	flip._func_godot_apply_properties({ "start_on": "1" })
	probe.calls.clear()
	watch.calls.clear()
	_send(flip, "trigger")
	check(_tags(probe) == ["on_b"] and _tags(watch) == [false], "starting on, the first trigger turns it off, got %s" % [_tags(probe)])
	_send(flip, "reset")
	check(_tags(watch) == [false, true] and _tags(probe) == ["on_b"], "reset goes back to on, got %s" % [_tags(watch)])

	var relay := GTRelay.new()
	_entity(map, relay, "relay")
	flip._func_godot_apply_properties({ "start_on": false })
	_wire(relay, "triggered", "flip", "trigger")
	probe.calls.clear()
	relay.trigger(who)
	relay.trigger()
	check(_tags(probe) == ["on_a", "on_b"], "a relay switches it, activator or not, got %s" % [_tags(probe)])
	await _free(map)

## A map file with the entities wired the way a level would, built by the addon so the properties arrive as map text.
func test_built_map() -> void:
	print("- a built map of math and logic entities")
	var next_id := [10]
	var ent := func(classname: String, targetname: String, props: Dictionary, outputs: Array) -> Dictionary:
		next_id[0] += 1
		props["targetname"] = targetname
		return { "type": "entity", "id": next_id[0], "classname": classname, "origin": [next_id[0] * 16, 0, 0], "angles": [0, 0, 0], "properties": props, "outputs": outputs }
	var to := func(output: String, target: String, input: String, parameter := "") -> Dictionary:
		return { "output": output, "target": target, "input": input, "parameter": parameter, "delay": 0.0, "times": -1 }
	var children := [
		ent.call("logic_relay", "feed", {}, [to.call("triggered", "score", "add", "2")]),
		ent.call("logic_relay", "undo", {}, [to.call("triggered", "score", "reset")]),
		ent.call("math_value", "score", { "start_value": "0" }, [to.call("changed", "cmp", "set_and_compare"), to.call("changed", "triple", "calculate")]),
		ent.call("math_calc", "triple", { "operation": "multiply", "operand": "3" }, [to.call("result", "probe", "record")]),
		ent.call("math_compare", "cmp", { "compare_value": "5" }, [
			to.call("on_greater", "gate", "set_a", "true"), to.call("on_equal", "gate", "set_a", "true"), to.call("on_less", "gate", "set_a", "false")]),
		ent.call("logic_gate", "gate", { "gate": "and", "start_b": "1" }, [to.call("on_true", "probe", "record", "open"), to.call("on_false", "probe", "record", "closed")]),
		ent.call("logic_relay", "draw", {}, [to.call("triggered", "pick", "pick")]),
		ent.call("logic_random", "pick", { "count": "3" }, [to.call("picked", "case", "in_value")]),
		ent.call("logic_case", "case", { "case_1": "1", "case_2": "2" }, [
			to.call("on_case_1", "probe", "record", "light1"), to.call("on_case_2", "probe", "record", "light2"), to.call("on_default", "probe", "record", "light3")]),
		ent.call("logic_relay", "toggle", {}, [to.call("triggered", "flip", "trigger")]),
		ent.call("logic_flipflop", "flip", {}, [to.call("on_a", "probe", "record", "on"), to.call("on_b", "probe", "record", "off")]),
	]
	var path := OS.get_temp_dir().path_join("gt_math_logic_test_%d.gtm" % OS.get_process_id())
	var map_json := { "format": "godottrench-map", "properties": {}, "layers": [{ "type": "layer", "id": 1, "children": children }] }
	FileAccess.open(path, FileAccess.WRITE).store_string(JSON.stringify(map_json))
	var map := FuncGodotMap.new()
	map.map_settings = load("res://demo/demo_map_settings.tres")
	map.global_map_file = path
	t.root.add_child(map)
	map.build()
	await t.process_frame
	DirAccess.remove_absolute(path)
	var probe: Node3D = load(PROBE).new()
	probe.set_meta(GodotTrenchIO.TARGETNAME_META, "probe")
	map.add_child(probe)
	GodotTrenchIO.invalidate(map)
	var named := func(targetname: String) -> Node:
		return t.find_targetname(map, targetname)
	check(named.call("score") is GTValue and named.call("cmp") is GTCompare and named.call("gate") is GTGate and named.call("case") is GTCase, "the entities are built with their scripts")
	check(named.call("pick") is GTRandom and named.call("triple") is GTCalc and named.call("flip") is GTFlipFlop, "including the random pick, the calculation and the flip flop")
	check(named.call("cmp")._compare == 5 and named.call("triple").operand == 3.0 and named.call("gate").b, "map text becomes numbers and booleans, got %s, %s" % [named.call("cmp")._compare, named.call("gate").b])

	for i in 3:
		_send(named.call("feed"), "trigger")
	check(_tags(probe) == [6, 12, "open", 18] or _tags(probe) == [6, 12, 18, "open"], "three points add up to 6, 12 and 18, and the gate opens past 5, got %s" % [_tags(probe)])
	probe.calls.clear()
	_send(named.call("undo"), "trigger")
	check(_tags(probe).has("closed") and _tags(probe).has(0) and named.call("score").current == 0, "resetting the score closes the gate again, got %s" % [_tags(probe)])
	probe.calls.clear()
	for i in 90:
		_send(named.call("draw"), "trigger")
	var tags := _tags(probe)
	check(tags.size() == 90 and tags.all(func(x): return x in ["light1", "light2", "light3"]), "every draw lights one of three, got %s" % [tags.slice(0, 5)])
	check(tags.has("light1") and tags.has("light2") and tags.has("light3"), "and each of them comes up, the third through the default")
	probe.calls.clear()
	for i in 3:
		_send(named.call("toggle"), "trigger")
	check(_tags(probe) == ["on", "off", "on"], "one button switches on and off, got %s" % [_tags(probe)])
	map.queue_free()
	await t.process_frame
