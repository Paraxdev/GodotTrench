# Spawners and paths

## info_spawner

Spawns a scene within `radius`, up to `max_alive` at a time and `total` overall, 0 being unlimited. `start` runs a
spawn timer when `interval` is above 0.

* **Inputs:** `spawn`, `start`, `stop`, `toggle`, `kill_all`
* **Outputs:** `spawned(node)`, `all_dead`, `exhausted` (once, the first time `total` is reached)
* **Keys:** `scene`, `count` 1, `max_alive` 5, `total` 0, `interval` 0, `radius` 64, `start_active` 0,
  `spawn_on_ready` 0, `spawn_group` enemies, `snap_to_ground` 1

Spawned nodes have no targetname. Reach them through their `spawn_group`, like `@enemies`.

## info_teleport_destination

Marks where `trigger_teleport` sends bodies, its yaw becomes their facing. Its only key is `targetname`.

## path_corner

One stop on a path, with `target` naming the next. It fires `reached` when a train or walker arrives. Click chains
down with the **Path** tool (Shift+P).

* **Outputs:** `reached(train)`
* **Keys:** `target`, `wait` 0 (seconds to wait here), `speed` 0 (new train speed from here on, 0 keeps it)

`speed` only affects trains, walkers keep their own.

## npc_walker

A scripted actor that walks a chain of path corners playing animations, for cutscenes rather than AI. It moves in
straight lines, ignoring gravity and walls. `model` is a scene giving it a mesh and an `AnimationPlayer`.

* **Inputs:** `start`, `stop`, `walk_to(corner)`, `play_anim(name)`, `face(target)`
* **Outputs:** `arrived(corner)`, `reached_goal`, `finished` (both at the last corner when not looping)
* **Keys:** `model`, `target`, `speed` 2.0, `loop` 0, `start_active` 0, `walk_anim` walk, `idle_anim` idle,
  `face_travel` 1

`face` and `walk_to` accept any target, like `!player`. `walk_to` drops the current leg and heads straight there.
