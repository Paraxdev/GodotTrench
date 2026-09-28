# Logic

> **Note:** These entities come with the Gameplay entities pack, see
> [Installing and customizing](README.md#installing-and-customizing).

Invisible point entities that decide when things happen. They sit between a trigger or button and what it affects,
and shape the signal on the way. Entities that work with numbers are in [Math and values](math.md), and the ones that run
code or animations in [Scripts and calls](scripting.md).

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

## logic_gate

Holds two true or false values, `a` and `b`, and combines them with a gate, for checks like "the door opens when the
power is on **and** the lever is pulled". Each input change that flips the result fires `changed(on)` and then `on_true`
or `on_false`, so nothing has to ask. `test` fires the current result again, for when something else needs to know now.

* **Inputs:** `set_a(value)`, `set_b(value)`, `toggle_a`, `toggle_b`, `test`
* **Outputs:** `on_true`, `on_false`, `changed(on)`
* **Keys:** `gate` and, `start_a` 0, `start_b` 0

| `gate` | True when |
| --- | --- |
| `and` | both are true |
| `or` | at least one is true |
| `xor` | exactly one is true |
| `nand` | not both are true |
| `nor` | neither is true |
| `xnor` | both are the same |
| `not` | `a` is false, `b` is ignored |

`set_a` and `set_b` without a value set true, so a trigger's `entered` output wired to `set_a` switches `a` on. A number
is true unless it is 0, and text is true only for `true`, `yes` or `1`. The starting values do not fire anything, and
neither does a change that leaves the result as it was.

## logic_flipflop

Alternates every time it is triggered, so one button can switch something on and off again. `on_a` fires when it turns
on and `on_b` when it turns off. `reset` puts it back to `start_on` without firing either, and fires only `changed` if
that was a change.

* **Inputs:** `trigger`, `reset`
* **Outputs:** `on_a`, `on_b`, `changed(on)`
* **Keys:** `start_on` 0 (on, so the first trigger fires `on_b`)

## logic_case

Compares the value it receives with up to eight cases and fires the output of the first one that matches, or
`on_default` when none does. It turns a number, like the index from a `logic_random` or the score of a `math_value`, into
different events.

* **Inputs:** `in_value(value)`
* **Outputs:** `on_case_1` to `on_case_8`, `on_default`
* **Keys:** `case_1` to `case_8`, empty

Numbers match by value, so the case `2` matches a value of 2 and of `2.0`. Text has to match exactly, `Red` is not
`red`, and spaces around a case are ignored. Empty cases are skipped, the first match wins, and a value that is missing
goes to `on_default`.

## logic_random

Picks one of up to eight outputs at random, or rolls a chance. Wire `out_1` to `out_3` to three lights and `pick` lights
one of them. `roll` decides by chance instead, for example whether a chest holds loot.

* **Inputs:** `pick`, `roll`
* **Outputs:** `out_1` to `out_8`, `picked(index)`, `on_success`, `on_fail`
* **Keys:** `count` 3 (1 to 8), `no_repeat` 0, `chance` 50 (percent)

`pick` fires `picked` with the number of the output it chose and then that output, so `picked` can feed a
[`logic_case`](#logic_case). With `no_repeat` it never chooses the output it chose last time, unless `count` is 1.
`roll` fires `on_success` with a probability of `chance` percent and `on_fail` otherwise, so 0 never succeeds and 100
always does.

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
