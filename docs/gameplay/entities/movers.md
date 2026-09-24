# Doors, movers and buttons

Brush entities that move as one body and carry whatever stands on them, so a lift takes the player along.

> **Note:** Doors and buttons do not react to the player on their own. Your player script calls `use(activator)` on
> them, for example from an interaction ray, see [Button opens a door](../../tutorials/button-door.md). Set `interact`
> to 0 for one that only I/O or a script should move.

## func_door

A sliding door, hatch or shutter. It moves by `travel` when opened and back when closed, drag the handle in the
viewport to set it. A locked door ignores `open` and `use` and fires `locked_use` instead, for a "locked" message.

| Key | Default | What it does |
| --- | --- | --- |
| `travel` | 0 64 0 | Offset it slides by, in map units |
| `speed` | 2.0 | Meters per second |
| `wait` | -1 | -1 stays open, a positive value closes it again after that many seconds |
| `locked` | 0 | Starts locked |
| `start_open` | 0 | Starts in the open position |
| `interact` | 1 | Opens when the player calls `use` on it |

* **Inputs:** `open`, `close`, `toggle`, `lock`, `unlock`, `use(activator)`
* **Outputs:** `opened`, `closed`, `started_opening`, `started_closing`, `locked_use(activator)`

## func_door_rotating

A door or gate that swings around a hinge you drag into place in the viewport. `open` takes the activator so
`open_away` can swing it away from them rather than into their face.

| Key | Default | What it does |
| --- | --- | --- |
| `hinge` | 0 0 0 | Hinge position relative to the door's center |
| `axis` | y | Swing axis. `y` is an upright hinge, `x` or `z` make a hatch or drawbridge |
| `open_angle` | 95 | Degrees to swing, negative swings the other way |
| `speed` | 120 | Degrees per second |
| `open_away` | 1 | Swings away from whoever opens it |
| `wait`, `locked`, `start_open`, `interact` | -1, 0, 0, 1 | As on `func_door` |

* **Inputs:** `open(activator)`, `close`, `toggle(activator)`, `lock`, `unlock`, `use(activator)`
* **Outputs:** same as `func_door`

`func_gate` is an older alias with only `hinge`, `open_angle` 100 and `speed` 90 as keys. Use `func_door_rotating`.

## func_platform

A lift or moving platform between its start and `start + travel`, for elevators or floors that cross a pit.

* **Inputs:** `start`, `stop`, `toggle`, `go_to_end`, `go_to_start`
* **Outputs:** `reached_end`, `reached_start`, `started`
* **Keys:** `travel` 0 128 0, `speed` 2.0, `wait` 1.0, `mode` 0, `start_active` 0 (moves from map load)

| `mode` | `start` does | Typical use |
| --- | --- | --- |
| 0, toggle | Moves to the other end | A lift called by a button |
| 1, ping pong | Moves back and forth until `stop`, waiting `wait` seconds at each end | A platform that keeps cycling |
| 2, once | Moves to the end and stays there | A bridge that extends once |

## func_train

Follows a chain of `path_corner` entities from `target`, for anything that runs a route with several stops, like a tram
or a cart on a rail. Each corner's `wait` and `speed` apply on arrival. It starts moving on map load and loops, so set
`start_active` or `loop` to 0 for a one off ride.

* **Inputs:** `start`, `stop`, `toggle`
* **Outputs:** `arrived(corner)`, `finished` (only when not looping)
* **Keys:** `target`, `speed` 3.0, `loop` 1, `start_active` 1, `orient` 0 (1 turns it to face where it goes)

## func_button

Sinks by `travel` while pressed and fires `pressed`. It pops out again after `wait` seconds, while -1 keeps it pressed
for a one time switch. A locked button fires `locked_use` instead, handy for a panel that needs power first.

* **Inputs:** `press(activator)`, `release`, `lock`, `unlock`, `use(activator)`
* **Outputs:** `pressed(activator)`, `released`, `locked_use(activator)`
* **Keys:** `travel` 0 -2 0, `wait` 1.0, `locked` 0, `interact` 1
