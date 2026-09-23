# Logic

Logic entities have no body and no visuals. They shape the flow of signals: relaying, counting, branching, timing and
sequencing.

## logic_relay

The basic building block. Forwards whatever triggers it and can be switched off, which makes it the easiest way to
build a chain that only works under some condition.

* **Inputs:** `trigger(activator)`, `enable`, `disable`, `toggle`
* **Outputs:** `triggered(activator)`
* **Keys:** `start_disabled` 0

## logic_branch

Remembers true or false and fires one of two outputs when tested. For "only if the power is on" checks.

* **Inputs:** `set_true`, `set_false`, `toggle`, `test`, `set_and_test(value)`
* **Outputs:** `on_true`, `on_false`
* **Keys:** `start_value` 0

## logic_counter

Counts between a minimum and a maximum, for "press all three buttons" puzzles. The value is clamped and outputs only
fire when it changes. `reset` only fires `changed`.

* **Inputs:** `add(amount)` (1 by default), `subtract(amount)`, `set_value(value)`, `reset`
* **Outputs:** `hit_max`, `hit_min`, `changed(value)`
* **Keys:** `min` 0, `max` 3, `start_value` 0

## logic_timer

Fires `timer` every interval, from the moment the map loads unless `start_on` is 0. Set `random_max` above `random_min`
for a random interval each time.

* **Inputs:** `start`, `stop`, `toggle`, `fire` (once, now)
* **Outputs:** `timer`
* **Keys:** `interval` 1.0, `random_min` 0, `random_max` 0, `start_on` 1, `once` 0

## logic_auto

Fires `map_spawn` once, just after the map is ready. The place to start anything that runs from the beginning.

* **Outputs:** `map_spawn`

## logic_sequence

Fires up to eight numbered steps in order, waiting before each one, the first included.

* **Inputs:** `start`, `stop`, `reset`
* **Outputs:** `step_1` to `step_8`, `step(index)`, `finished`
* **Keys:** `steps` 3 (1 to 8), `interval` 1.0, `times` empty, `start_active` 0, `loop` 0

`times` sets one wait per step as a space separated list, steps without an entry use `interval`. With `loop` on it
starts over and never fires `finished`. `start` is ignored while it runs, `stop` then `start` begins again from step 1.
See the [cutscene tutorial](../../tutorials/cutscene.md).

## logic_script

Runs a GDScript snippet when fired, for logic no combination of inputs can express, like reading the player's health.

* **Inputs:** `run(activator)`, `run_with(parameter)`
* **Outputs:** `ran(result)`
* **Keys:** `source` empty, `expression` empty (a one liner, used when `source` is empty)

In scope: `this`, `activator`, `parameter`, `io` (the `GodotTrenchIO` class) and `tree`. `source` is compiled once when
the map loads.

> **Warning:** Indent nested lines with tabs. Spaces give mixed indentation and the snippet fails to compile.

`run_with` always has a null `activator`.

## logic_call

Calls a method on any node with fixed arguments, for reaching game code such as a score autoload. Unlike plain outputs
it warns when the target does not exist.

* **Inputs:** `trigger(activator)`, `call_with(parameter)`
* **Outputs:** `called(result)`
* **Keys:** `call_target` `/root/Game`, `method` empty, `arguments` `[]`

A `"$parameter"` element in `arguments` is replaced with the incoming parameter. Example: `call_target` `/root/Game`,
`method` `give_item`, `arguments` `["key_red", "$activator"]`.

## logic_debug

Prints a line when it receives an input, and optionally shows it as a label in debug builds.

* **Inputs:** `write(parameter, activator)`, `toggle`
* **Outputs:** `printed(text)`
* **Keys:** `message` "{name} received {parameter} from {activator}" (also `{time}`), `show_label` 1, `trace_all` 0

With `trace_all` on it logs every output in the game, a quick way to watch a whole chain.

## logic_animate

Plays animations on the first `AnimationPlayer` at or under `target`, so I/O can drive a character or prop that carries
its own animations.

* **Inputs:** `play(name)`, `stop`, `queue(name)`, `seek(time)`
* **Outputs:** `finished(anim)`
* **Keys:** `target` `!self`, `animation` empty, `speed` 1.0

`target` can be `!activator`, to animate whoever started the chain.
