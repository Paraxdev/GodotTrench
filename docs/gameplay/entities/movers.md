# Doors, movers and buttons

These are brush entities: draw the brush, then convert it. They move by animating their body, so they carry players
standing on them. Speeds are in meters per second, or degrees per second for rotation.

> **Note:** None of these react to the player on their own. Your player script calls `use(activator)` on them, see
> [Button opens a door](../../tutorials/button-door.md). Set `interact` to 0 for doors that should only open from a
> trigger or a script.

## func_door

Slides by `travel` when opened. `wait` -1 stays open, a positive value closes again after that many seconds. A locked
door ignores `open` and `use` and fires `locked_use` instead.

* **Inputs:** `open`, `close`, `toggle`, `lock`, `unlock`, `use(activator)`
* **Outputs:** `opened`, `closed`, `started_opening`, `started_closing`, `locked_use(activator)`
* **Keys:** `travel` 0 64 0, `speed` 2.0, `wait` -1, `locked` 0, `start_open` 0, `interact` 1

## func_door_rotating

Swings around a hinge, which you drag into place in the viewport. `open_away` swings it away from whoever opens it,
which is why `open` takes the activator here.

* **Inputs:** `open(activator)`, `close`, `toggle(activator)`, `lock`, `unlock`, `use(activator)`
* **Outputs:** same as `func_door`
* **Keys:** `hinge` 0 0 0 (relative to the door), `axis` y, `open_angle` 95, `speed` 120, `open_away` 1, `wait` -1,
  `locked` 0, `start_open` 0, `interact` 1

## func_gate

A rotating door with fewer options, for gates and simple swinging panels. Runs the same script as
`func_door_rotating`.

* **Inputs:** `open`, `close`, `toggle`
* **Outputs:** `opened`, `closed`
* **Keys:** `hinge`, `open_angle` 100, `speed` 90

## func_platform

Moves between its start and `start + travel`.

* **Inputs:** `start`, `stop`, `toggle`, `go_to_end`, `go_to_start`
* **Outputs:** `reached_end`, `reached_start`, `started`
* **Keys:** `travel` 0 128 0, `speed` 2.0, `wait` 1.0, `mode` 0, `start_active` 0

| `mode` | Behaviour |
| --- | --- |
| 0 | One way per toggle |
| 1 | Ping pong, waiting `wait` seconds at each end |
| 2 | Goes once |

## func_train

Follows a chain of `path_corner` entities, starting at `target`. It moves from the moment the map loads and loops, so
set `start_active` or `loop` to 0 for a one off ride.

* **Inputs:** `start`, `stop`, `toggle`
* **Outputs:** `arrived(corner)`, `finished` (only when not looping)
* **Keys:** `target`, `speed` 3.0, `loop` 1, `start_active` 1, `orient` 0

## func_button

Moves by `travel` while pressed and fires `pressed`. `wait` -1 keeps it pressed.

* **Inputs:** `press(activator)`, `release`, `lock`, `unlock`, `use(activator)`
* **Outputs:** `pressed(activator)`, `released`, `locked_use(activator)`
* **Keys:** `travel` 0 -2 0, `wait` 1.0, `locked` 0, `interact` 1
