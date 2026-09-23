# Scripts and calls

Point entities for code and animations that plain wiring cannot reach.

## logic_script

Runs a GDScript snippet when fired, for logic that wiring cannot express, like reading the player's health.

* **Inputs:** `run(activator)`, `run_with(parameter)`
* **Outputs:** `ran(result)`, with the snippet's return value
* **Keys:** `source` empty, `expression` empty (a one liner, used when `source` is empty)

The snippet sees `this`, `activator`, `parameter`, `io` (the `GodotTrenchIO` class) and `tree`. A compile error is
logged as a warning when the map loads. `run_with` always has a null `activator`.

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

> **Warning:** Indent nested lines with tabs. Spaces fail to compile with a mixed indentation error.

## logic_call

Calls a method on any target with fixed arguments, for game code such as a score autoload. Unlike a plain output it
warns when the target or the method does not exist.

* **Inputs:** `trigger(activator)`, `call_with(parameter)`
* **Outputs:** `called(result)`
* **Keys:** `call_target` `/root/Game`, `method` empty, `arguments` `[]`

`arguments` is a JSON array, like `["key_red", "$activator"]`, and takes the placeholders from
[Parameters](../parameters.md), resolved on the `logic_call` itself. `call_with` puts its parameter where
`"$parameter"` stands, or passes it as the only argument.

## logic_animate

Plays animations on the first `AnimationPlayer` at or under `target`, for characters and props that carry their own
animations. Set `target` to `!activator` to animate whoever started the chain.

* **Inputs:** `play(name)`, `stop`, `queue(name)`, `seek(time)`
* **Outputs:** `finished(anim)`
* **Keys:** `target` `!self`, `animation` empty (played when `play` has no name), `speed` 1.0

## logic_debug

Prints a line on `write` and shows its recent lines as a floating label in debug builds. With `trace_all` on it logs
every output in the game, a quick way to watch a whole chain.

* **Inputs:** `write(parameter)`, `toggle` (the label), `show`, `hide`
* **Outputs:** `printed(text)`
* **Keys:** `message` `{name} received {parameter} from {activator}` (`{time}` works too), `show_label` 1,
  `trace_all` 0
