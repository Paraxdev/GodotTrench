# Triggers

Triggers are invisible brush volumes that react to physics bodies entering them. Draw one quickly with the **Volume**
tool (Shift+E).

## Shared inputs, outputs and keys

Most triggers have these:

* **Inputs:** `enable`, `disable`, `toggle`
* **Outputs:** `triggered(activator)`, `entered(activator)`, `exited(activator)`
* **Keys:** `filter_group` player, `start_disabled` 0

`filter_group` limits a trigger to members of a Godot group. Empty accepts any body.

| Output | Fires |
| --- | --- |
| `entered` | On every accepted entry |
| `triggered` | On entry, respecting the cooldown |
| `exited` | On exit, even while the trigger is disabled |

The cooldown is tracked per body, so two players entering close together each still fire it.

## trigger_once

Fires once and disables itself. `enable` arms it again.

## trigger_multiple

Fires on every entry, at most once per `cooldown`, 0.5 seconds by default.

## trigger_hurt

Damages bodies inside by calling their `take_damage(amount, source)`, `damage * interval` per tick.

* **Outputs:** `hurt(activator)`, per damaged body per tick
* **Keys:** `damage` 10 per second, `damage_method` take_damage, `interval` 0.5, `filter_group` empty

`filter_group` is empty by default, so it hurts everything, not just the player. `hurt` only fires for a body that has
a `take_damage` method, or for any accepted body when `filter_group` names a group.

## trigger_teleport

Moves bodies to the `info_teleport_destination` named in `destination`, with its position and facing. Stops them
unless `keep_velocity` is 1.

* **Outputs:** `teleported(activator)`
* **Keys:** `destination`, `filter_group` player, `keep_velocity` 0, `cooldown` 0.5

## trigger_push

Pushes bodies with `push`, in map units per second.

* **Outputs:** `pushed(activator)`
* **Keys:** `push` 0 384 0, `filter_group` empty, `once` 1

With `once` 1 each entry gets one impulse. With 0 it pushes steadily like a wind tunnel, firing `pushed` once on entry.
A rigid body stops accelerating once it reaches `push` speed.

## trigger_call

Calls a method when triggered, on `!activator` by default, so a trigger can call straight into the player's script.

* **Inputs:** `trigger(activator)` and the shared inputs
* **Outputs:** `called(result)` and the shared outputs
* **Keys:** `call_target` `!activator`, `method`, `arguments` `[]`, `once` 0, `cooldown` 0.5

## trigger_spawn_area

Spawns scenes inside its volume, by default when the player enters. Works like `info_spawner`, see
[Actors, props and effects](actors.md). `toggle` starts or stops the spawn timer.

* **Inputs:** `spawn`, `start`, `stop`, `toggle`, `kill_all`
* **Outputs:** `spawned(node)`, `all_dead`, `exhausted` (once, the first time `total` is reached)
* **Keys:** `scene`, `count` 3, `max_alive` 10, `total` 0, `interval` 0, `spawn_on_enter` 1, `filter_group` player,
  `spawn_group` enemies, `snap_to_ground` 1, `cooldown` 0.5
