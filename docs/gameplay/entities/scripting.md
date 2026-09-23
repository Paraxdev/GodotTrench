# Scripts and calls

These point entities reach code and animations that plain wiring cannot.

## logic_script

Runs a GDScript snippet when fired, for logic no combination of inputs can express, like reading the player's health.

* **Inputs:** `run(activator)`, `run_with(parameter)`
* **Outputs:** `ran(result)`, with the snippet's return value
* **Keys:** `source` empty, `expression` empty (a one liner, used when `source` is empty)

In scope are `this`, `activator`, `parameter`, `io` (the `GodotTrenchIO` class) and `tree`. `source` is compiled once
when the map loads, and a compile error is logged as a warning. `run_with` always has a null `activator`.

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

> **Warning:** Indent nested lines with tabs. The snippet is wrapped in a function indented by a tab, so spaces give
> mixed indentation and it fails to compile.

## logic_call

Calls a method on any target with fixed arguments, for reaching game code such as a score autoload. Unlike a plain
output it warns when the target or the method does not exist.

* **Inputs:** `trigger(activator)`, `call_with(parameter)`
* **Outputs:** `called(result)`
* **Keys:** `call_target` `/root/Game`, `method` empty, `arguments` `[]`

`arguments` is a JSON array and takes the placeholders from [Parameters](../parameters.md), worked out on the
`logic_call` itself. `call_with` puts its parameter where `"$parameter"` stands, or passes it as the only argument.
Example: `method` `give_item`, `arguments` `["key_red", "$activator"]`.

## logic_animate

Plays animations on the first `AnimationPlayer` at or under `target`, so I/O can drive a character or prop that carries
its own animations. `target` can be `!activator` to animate whoever started the chain.

* **Inputs:** `play(name)`, `stop`, `queue(name)`, `seek(time)`
* **Outputs:** `finished(anim)`
* **Keys:** `target` `!self`, `animation` empty (played when `play` has no name), `speed` 1.0

## logic_debug

Prints a line when it receives `write`, and shows its recent lines as a floating label in debug builds. With
`trace_all` on it logs every output in the game, a quick way to watch a whole chain.

* **Inputs:** `write(parameter)`, `toggle` (the label), `show`, `hide`
* **Outputs:** `printed(text)`
* **Keys:** `message` `{name} received {parameter} from {activator}` (`{time}` works too), `show_label` 1,
  `trace_all` 0
