# Math and values

> **Note:** These entities come with the Gameplay entities pack, see
> [Installing and customizing](README.md#installing-and-customizing).

Invisible point entities that store, calculate and compare numbers, so a score, a count or a price can steer the map
without a script. They are wired like every entity. A number travels along a connection as the value of an output, like
`changed(value)` or `result(value)`, and an input with no parameter of its own receives it, see
[No parameter](../parameters.md#no-parameter). A parameter typed into the connection wins over the value.

Whole numbers stay whole, `2 + 3` is `5` and not `5.0`, and a number that arrives as text like `2.5` counts as a number.
Anything that is not a number counts as text.

## math_value

Stores one number or piece of text, the way a variable does. Buttons `add` to it, and `changed` tells the rest of the
map what it is now. It fires `changed` only when the value differs from before.

* **Inputs:** `set_value(value)`, `add(amount)` (1 by default), `get_value`, `reset`
* **Outputs:** `value(value)`, `changed(value)`
* **Keys:** `start_value` 0, a number or text

`add` sums two numbers and appends when either one is text, so adding `!` to `hello` gives `hello!`. `get_value` fires
`value` with what is stored, for when something needs the number without waiting for a change. `reset` goes back to
`start_value`. A `set_value` without a value keeps what is stored.

## math_calc

Does one calculation on the number it receives and fires the answer. It keeps nothing, so the same input always gives
the same answer. Chain several to work something out in steps.

* **Inputs:** `calculate(value)`, a missing value counts as 0
* **Outputs:** `result(value)`
* **Keys:** `operation` add, `operand` 1

| `operation` | `result` |
| --- | --- |
| `add`, `subtract`, `multiply` | the value plus, minus or times `operand` |
| `divide` | the value divided by `operand` |
| `modulo` | the remainder of the division, never negative for a positive `operand`, so `-1` modulo `3` is `2` |
| `power` | the value raised to `operand` |
| `min`, `max` | the smaller or the larger of the two |
| `abs`, `negate` | the value without its sign, or with the sign flipped |
| `round`, `floor`, `ceil` | the nearest whole number (halves go away from 0), rounded down, rounded up |

`abs`, `negate`, `round`, `floor` and `ceil` ignore `operand`. When there is no answer, like `divide` or `modulo` by 0 or
the root of a negative number, nothing fires and Godot prints a warning once.

## math_compare

Compares a stored value with `compare_value` and fires which side it landed on. `on_not_equal` fires next to `on_less`
and `on_greater`, for when only a difference matters.

* **Inputs:** `set_value(value)`, `set_compare_value(value)`, `compare`, `set_and_compare(value)`
* **Outputs:** `on_less`, `on_equal`, `on_not_equal`, `on_greater`
* **Keys:** `compare_value` 0

`set_value` and `set_compare_value` only store, `compare` compares what is stored, and `set_and_compare` does both, so it
is the input to wire to a `changed(value)` output. Numbers compare by value, and `10` is more than `9`. Two pieces of
text compare in dictionary order, with capital letters before small ones. Nearly equal decimals, like `0.1 + 0.2` and `0.3`, count as equal.

## Example: a score that opens a door

Three coin buttons and a lever open a vault door once at least three coins are in.

1. Place a `math_value` named `score`, a `math_compare` named `enough` with `compare_value` 3, and a `logic_gate`
   named `door_gate` with `gate` and.
2. Wire each coin button's `pressed` to `score`, input `add`. Each press adds 1.
3. Wire `score`'s `changed` to `enough`, input `set_and_compare`. It passes the new score on.
4. Wire `enough`'s `on_less` to `door_gate`, input `set_a`, parameter `false`. Wire `on_equal` and `on_greater` the same
   way with parameter `true`. The gate's `a` now says whether there are enough coins.
5. Wire the lever's `pressed` to `door_gate`, input `toggle_b`.
6. Wire `door_gate`'s `on_true` to the door with input `open`, and `on_false` with input `close`.

The door opens when the third coin lands while the lever is pulled, or the other way round. Pulling the lever back or
resetting the score closes it. To show the score, wire `changed` of `score` to a `game_text` with input `set_text`, or
put a `math_calc` in between to show points instead of coins.

The logic playground map builds this hall, and adds a light lottery with `logic_random` and `logic_case` and a lamp
switch with `logic_flipflop`. Open `godot/demo/maps/logic_playground.gtm` in the demo project, or read how it is made in
[`examples/mcp/logic_playground.json`](https://github.com/Paraxdev/GodotTrench/blob/main/examples/mcp/logic_playground.json).

{% mcp %}

## MCP

The script above places the entities with `gameplay place`, one call each:

```json
{ "tool": "gameplay", "args": { "op": "place", "classname": "math_value", "origin": [-280, 40, 40],
  "properties": { "targetname": "score" },
  "outputs": [ { "output": "changed", "target": "enough", "input": "set_and_compare" } ] } }
{ "tool": "gameplay", "args": { "op": "place", "classname": "math_compare", "origin": [-280, 40, -40],
  "properties": { "targetname": "enough", "compare_value": "3" },
  "outputs": [ { "output": "on_greater", "target": "door_gate", "input": "set_a", "parameter": "true" } ] } }
```

{% endmcp %}
