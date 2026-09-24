# Spawners and paths

## info_spawner

Spawns a scene, such as an enemy or a pickup, at random spots within `radius`. Fire `spawn` from a trigger for an
ambush, or set an `interval` and `start` it for a steady stream. To spread spawns over a room, use
[`trigger_spawn_area`](triggers.md#triggerspawnarea).

| Key | Default | What it does |
| --- | --- | --- |
| `scene` | | The `res://` scene to spawn |
| `count` | 1 | How many to spawn each time |
| `max_alive` | 5 | Never more alive at once, extra spawns are skipped |
| `total` | 0 | Stops for good after this many, 0 is unlimited |
| `interval` | 0 | Seconds between spawns while the timer runs. 0 spawns only on `spawn` |
| `radius` | 64 | How far from the spawner they may appear. Drag its handle in the viewport |
| `start_active` | 0 | Runs the timer from map load |
| `spawn_on_ready` | 0 | Spawns once when the map loads |
| `spawn_group` | enemies | Godot group added to every spawned node |
| `snap_to_ground` | 1 | Drops each spawn onto the floor below it |

* **Inputs:** `spawn`, `start`, `stop`, `toggle`, `kill_all`
* **Outputs:** `spawned(node)`, `all_dead`, `exhausted` (once, the first time `total` is reached)

`all_dead` fires when every spawned node is gone, for "clear the room to open the door". Spawned nodes have no
targetname, reach them through their `spawn_group`, like `@enemies`.

## info_teleport_destination

Marks where `trigger_teleport` sends bodies. Point it the way the player should look on arrival, its yaw becomes their
facing. Its only key is `targetname`.

## path_corner

One stop on the route of a `func_train` or `npc_walker`, with `target` naming the next. Click a chain down with the
**Path** tool (Shift+P). It fires `reached` when a train or walker arrives, for a sound or event at that stop.

* **Outputs:** `reached(train)`
* **Keys:** `target`, `wait` 0 (seconds to wait here), `speed` 0 (new train speed from here on, 0 keeps it)

`speed` only affects trains, walkers keep their own.

## npc_walker

A scripted actor that walks a chain of path corners playing animations, for cutscenes rather than AI, like a guard
passing a window. It moves in straight lines, ignoring gravity and walls, so lay the corners where it can walk.

| Key | Default | What it does |
| --- | --- | --- |
| `model` | | A scene giving it a mesh and an `AnimationPlayer` |
| `target` | | The first path corner to walk to |
| `speed` | 2.0 | Meters per second |
| `loop` | 0 | Returns to the first corner after the last |
| `start_active` | 0 | Walks from map load |
| `walk_anim`, `idle_anim` | walk, idle | Animations played while walking and while stopped |
| `face_travel` | 1 | Turns to face where it walks |

* **Inputs:** `start`, `stop`, `walk_to(corner)`, `play_anim(name)`, `face(target)`
* **Outputs:** `arrived(corner)`, `reached_goal`, `finished` (both at the last corner when not looping)

`face` and `walk_to` accept any target, like `!player`. `walk_to` drops the current leg and heads straight there.
