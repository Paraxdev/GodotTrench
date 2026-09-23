# Doors, movers and buttons

Brush entities that carry whatever stands on them.

> **Note:** Doors and buttons do not react to the player on their own. Your player script calls `use(activator)` on
> them, see [Button opens a door](../../tutorials/button-door.md). Set `interact` to 0 for one that only I/O or a
> script should move.

## func_door

Slides by `travel` when opened. `wait` -1 stays open, a positive value closes it again after that many seconds. A locked
door ignores `open` and `use` and fires `locked_use` instead.

* **Inputs:** `open`, `close`, `toggle`, `lock`, `unlock`, `use(activator)`
* **Outputs:** `opened`, `closed`, `started_opening`, `started_closing`, `locked_use(activator)`
* **Keys:** `travel` 0 64 0, `speed` 2.0, `wait` -1, `locked` 0, `start_open` 0, `interact` 1

## func_door_rotating

Swings around a hinge you drag into place in the viewport. `open` takes the activator so `open_away` can swing it
away from them.

* **Inputs:** `open(activator)`, `close`, `toggle(activator)`, `lock`, `unlock`, `use(activator)`
* **Outputs:** same as `func_door`
* **Keys:** `hinge` 0 0 0 (relative to the door's center), `axis` y, `open_angle` 95 (negative swings the other way),
  `speed` 120 (degrees per second), `open_away` 1, `wait` -1, `locked` 0, `start_open` 0, `interact` 1

`func_gate` is an older alias with only `hinge`, `open_angle` 100 and `speed` 90 as keys. Use `func_door_rotating`.

## func_platform

A lift or moving platform between its start and `start + travel`.

* **Inputs:** `start`, `stop`, `toggle`, `go_to_end`, `go_to_start`
* **Outputs:** `reached_end`, `reached_start`, `started`
* **Keys:** `travel` 0 128 0, `speed` 2.0, `wait` 1.0, `mode` 0, `start_active` 0

| `mode` | `start` does |
| --- | --- |
| 0, toggle | Moves to the other end |
| 1, ping pong | Moves back and forth until `stop`, waiting `wait` seconds at each end |
| 2, once | Moves to the end and stays there |

## func_train

Follows a chain of `path_corner` entities from `target`. It starts moving on map load and loops, so set `start_active`
or `loop` to 0 for a one off ride. Each corner's `wait` and `speed` apply on arrival.

* **Inputs:** `start`, `stop`, `toggle`
* **Outputs:** `arrived(corner)`, `finished` (only when not looping)
* **Keys:** `target`, `speed` 3.0, `loop` 1, `start_active` 1, `orient` 0 (1 turns it to face where it goes)

## func_button

Moves by `travel` while pressed and fires `pressed`. It pops out again after `wait` seconds, -1 keeps it pressed.

* **Inputs:** `press(activator)`, `release`, `lock`, `unlock`, `use(activator)`
* **Outputs:** `pressed(activator)`, `released`, `locked_use(activator)`
* **Keys:** `travel` 0 -2 0, `wait` 1.0, `locked` 0, `interact` 1
