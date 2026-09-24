# Logic

Invisible point entities that decide when things happen. They sit between a trigger or button and what it affects,
and shape the signal on the way. Entities that run code or animations are in [Scripts and calls](scripting.md).

## logic_relay

Passes `trigger` on as `triggered` while enabled. Wire several things to one relay, then `disable` it to switch the
whole chain off at once.

* **Inputs:** `trigger(activator)`, `enable`, `disable`, `toggle`
* **Outputs:** `triggered(activator)`
* **Keys:** `start_disabled` 0

## logic_branch

Remembers true or false and fires `on_true` or `on_false` on `test`, for checks like "the door only opens if the power
is on". The generator sets it true, the door button tests it.

* **Inputs:** `set_true`, `set_false`, `toggle`, `test`, `set_and_test(value)`
* **Outputs:** `on_true`, `on_false`
* **Keys:** `start_value` 0

## logic_counter

Counts between `min` and `max`, for puzzles like "press all three buttons": each button sends `add`, and `hit_max`
opens the door. The value is clamped and nothing fires unless it changes. `hit_max` and `hit_min` fire when a change
lands on a limit, `reset` fires only `changed`.

* **Inputs:** `add(amount)` (1 by default), `subtract(amount)`, `set_value(value)`, `reset`
* **Outputs:** `hit_max`, `hit_min`, `changed(value)`
* **Keys:** `min` 0, `max` 3, `start_value` 0

## logic_timer

Fires `timer` again and again, for anything that repeats, like a flickering light or waves of enemies. It waits
`interval` seconds between firings and runs from map load, unless `start_on` is 0. With `random_max` above `random_min`
each wait is random in that range, so repeats feel less mechanical. `once` stops it after the first firing, which
makes it a simple delay.

* **Inputs:** `start`, `stop`, `toggle`, `fire` (once, now)
* **Outputs:** `timer`
* **Keys:** `interval` 1.0, `random_min` 0, `random_max` 0, `start_on` 1, `once` 0

## logic_auto

Fires `map_spawn` once, after every entity of the map is ready. Wire anything that runs from the start here, like music
or an intro text.

* **Outputs:** `map_spawn`

## logic_sequence

A cutscene timeline. It fires up to eight numbered steps in order and waits before each one, the first included. Wire
each step to what happens at that moment, a door opening, a line of text, an actor walking.

* **Inputs:** `start`, `stop`, `reset`
* **Outputs:** `step_1` to `step_8`, `step(index)`, `finished`
* **Keys:** `steps` 3 (1 to 8), `interval` 1.0, `times` empty, `start_active` 0 (runs from map load), `loop` 0

`times` lists a wait per step, like `0 1 0.5`, and steps without one use `interval`. A looping sequence never fires
`finished`. `start` is ignored while it runs, and after `stop` it begins again at step 1. See the
[cutscene tutorial](../../tutorials/cutscene.md).
