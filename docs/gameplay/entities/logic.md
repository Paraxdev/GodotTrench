# Logic

Invisible point entities that route signals. Entities that run code or animations are in
[Scripts and calls](scripting.md).

## logic_relay

Passes `trigger` on as `triggered` while enabled. Disable it to switch a chain off.

* **Inputs:** `trigger(activator)`, `enable`, `disable`, `toggle`
* **Outputs:** `triggered(activator)`
* **Keys:** `start_disabled` 0

## logic_branch

Holds true or false and fires one of two outputs on `test`, for checks like "only if the power is on".

* **Inputs:** `set_true`, `set_false`, `toggle`, `test`, `set_and_test(value)`
* **Outputs:** `on_true`, `on_false`
* **Keys:** `start_value` 0

## logic_counter

Counts between `min` and `max`, for "press all three buttons" puzzles. The value is clamped and nothing fires unless it
changes. `hit_max` and `hit_min` fire when a change lands on a limit, `reset` fires only `changed`.

* **Inputs:** `add(amount)` (1 by default), `subtract(amount)`, `set_value(value)`, `reset`
* **Outputs:** `hit_max`, `hit_min`, `changed(value)`
* **Keys:** `min` 0, `max` 3, `start_value` 0

## logic_timer

Fires `timer` every `interval` seconds from map load, unless `start_on` is 0. With `random_max` above `random_min` each
wait is random in that range. `once` stops it after the first firing.

* **Inputs:** `start`, `stop`, `toggle`, `fire` (once, now)
* **Outputs:** `timer`
* **Keys:** `interval` 1.0, `random_min` 0, `random_max` 0, `start_on` 1, `once` 0

## logic_auto

Fires `map_spawn` once, after every entity of the map is ready. Start anything that runs from the beginning here.

* **Outputs:** `map_spawn`

## logic_sequence

A cutscene timeline. It fires up to eight numbered steps in order and waits before each one, the first included.

* **Inputs:** `start`, `stop`, `reset`
* **Outputs:** `step_1` to `step_8`, `step(index)`, `finished`
* **Keys:** `steps` 3 (1 to 8), `interval` 1.0, `times` empty, `start_active` 0, `loop` 0

`times` lists a wait per step, like `0 1 0.5`, and steps without one use `interval`. A looping sequence never fires
`finished`. `start` is ignored while it runs, and after `stop` it begins again at step 1. See the
[cutscene tutorial](../../tutorials/cutscene.md).
