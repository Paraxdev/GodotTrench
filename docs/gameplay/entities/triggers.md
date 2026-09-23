# Triggers

Triggers are invisible brush volumes that react to physics bodies entering them. Draw one quickly with the **Volume**
tool (Shift+E).

## Shared inputs, outputs and keys

Every trigger below has these:

* **Inputs:** `enable`, `disable`, `toggle`
* **Outputs:** `triggered(activator)`, `entered(activator)`, `exited(activator)`
* **Keys:** `filter_group` player, `start_disabled` 0

`filter_group` limits a trigger to bodies in that Godot group, empty accepts any body.

| Output | Fires |
| --- | --- |
| `entered` | On every accepted entry while enabled |
| `triggered` | On an accepted entry, at most once per `cooldown` for each body |
| `exited` | On exit, even while the trigger is disabled |

## trigger_once

Fires once and disables itself. `enable` arms it again.

## trigger_multiple

Fires on every entry, at most once per `cooldown` for each body, 0.5 seconds by default.

## trigger_hurt

Calls `damage_method(amount, source)` on bodies inside every `interval` seconds, with `damage * interval` as the
amount.

* **Outputs:** `hurt(activator)`, per damaged body per tick
* **Keys:** `damage` 10 (per second), `damage_method` take_damage, `interval` 0.5, `filter_group` empty

`filter_group` is empty by default, so it hurts anything with that method, not just the player. With a `filter_group`
set, `hurt` fires for every body of the group even without the method.

## trigger_teleport

Moves bodies to the entity named in `destination`, usually an `info_teleport_destination`, and turns them to its yaw.
It stops them unless `keep_velocity` is 1.

* **Outputs:** `teleported(activator)`
* **Keys:** `destination`, `keep_velocity` 0, `cooldown` 0.5

## trigger_push

Pushes bodies with `push`, a velocity in map units per second. With `once` 1 each entry gets one impulse. With 0 it
pushes steadily like a wind tunnel, and a rigid body stops accelerating at `push` speed.

* **Outputs:** `pushed(activator)`, once per entry
* **Keys:** `push` 0 384 0, `filter_group` empty, `once` 1

## trigger_call

Calls a method when triggered, on `!activator` by default, so a trigger can call straight into the player's script.
`arguments` works as in [`logic_call`](scripting.md#logiccall).

* **Inputs:** `trigger(activator)`
* **Outputs:** `called(result)`
* **Keys:** `call_target` `!activator`, `method`, `arguments` `[]`, `once` 0, `cooldown` 0.5

## trigger_spawn_area

Spawns scenes at random spots inside its volume, by default when the player enters. It works like `info_spawner`, see
[Spawners and paths](actors.md). `start` runs a spawn timer when `interval` is above 0.

* **Inputs:** `spawn`, `start`, `stop`, `toggle`, `kill_all`
* **Outputs:** `spawned(node)`, `all_dead`, `exhausted` (once, the first time `total` is reached)
* **Keys:** `scene`, `count` 3, `max_alive` 10, `total` 0, `interval` 0, `spawn_on_enter` 1, `spawn_group` enemies,
  `snap_to_ground` 1, `cooldown` 0.5
