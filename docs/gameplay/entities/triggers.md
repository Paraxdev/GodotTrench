# Triggers

Invisible brush volumes that react when a body enters them. They are how a level notices the player: a doorway starts
a cutscene, a pit hurts, a pad launches. Draw one with the **Volume** tool (Shift+E).

## Shared inputs, outputs and keys

Every trigger below has these on top of its own.

* **Inputs:** `enable`, `disable`, `toggle`
* **Outputs:** `triggered(activator)`, `entered(activator)`, `exited(activator)`
* **Keys:** `filter_group` player, `start_disabled` 0

`filter_group` limits the trigger to bodies in that Godot group, so enemies and crates do not set off a player
trigger. Leave it empty to accept any body.

| Output | Fires |
| --- | --- |
| `entered` | On every accepted entry while enabled |
| `triggered` | On an accepted entry, at most once per `cooldown` for each body. Most wiring uses this one |
| `exited` | On exit, even while the trigger is disabled, for undoing something when the body leaves |

Triggers detect bodies on physics layer 1. If the player is on another layer, add it to **Trigger Collision Mask** in
the map settings (the `FuncGodotMapSettings` resource in Godot), or the triggers never fire.

## trigger_once

Fires once and disables itself, for one time events like a cutscene or a hint. `enable` arms it again.

* **Keys:** `once` 1

## trigger_multiple

Fires on every entry, at most once per `cooldown` for each body, for things that happen every time, like a shop bell.

* **Keys:** `cooldown` 0.5

## trigger_hurt

Damages bodies while they stay inside, for lava, spikes or a kill zone below the map. It calls
`damage_method(amount, source)` on them every `interval` seconds, with `damage * interval` as the amount.

* **Outputs:** `hurt(activator)`, per damaged body per tick
* **Keys:** `damage` 10 (per second), `damage_method` take_damage, `interval` 0.5, `filter_group` empty

With the default empty `filter_group` it hurts anything that has the method, not just the player. With a group set,
`hurt` fires for every body of the group, even one without the method.

## trigger_teleport

Moves bodies to the entity named in `destination`, usually an
[`info_teleport_destination`](actors.md#infoteleportdestination), and turns them to face its yaw. It stops them unless
`keep_velocity` is 1, for a portal the player should fall or run through without losing speed.

* **Outputs:** `teleported(activator)`
* **Keys:** `destination`, `keep_velocity` 0, `cooldown` 0.5

## trigger_push

Pushes bodies with `push`, a velocity in map units per second you can aim with its viewport handle. With `once` 1 each
entry gets one impulse, like a jump pad. With 0 it pushes steadily like a wind tunnel or conveyor, and a rigid body
stops accelerating at `push` speed.

* **Outputs:** `pushed(activator)`, once per entry
* **Keys:** `push` 0 384 0, `filter_group` empty, `once` 1

## trigger_call

Calls a method when triggered, on `!activator` by default, so a trigger can call straight into the player's script,
for example to save a checkpoint. `call_target` and `arguments` work as in [`logic_call`](scripting.md#logiccall).

* **Inputs:** `trigger(activator)`
* **Outputs:** `called(result)`
* **Keys:** `call_target` `!activator`, `method`, `arguments` `[]`, `once` 0, `cooldown` 0.5

## trigger_spawn_area

Spawns scenes at random spots inside its volume, by default when the player enters, for an ambush as the player walks
into a room. Its keys work as on [`info_spawner`](actors.md#infospawner). `start` runs a spawn timer when `interval` is
above 0, and `toggle` here switches that timer rather than the trigger.

* **Inputs:** `spawn`, `start`, `stop`, `toggle`, `kill_all`
* **Outputs:** `spawned(node)`, `all_dead`, `exhausted` (once, the first time `total` is reached)
* **Keys:** `scene`, `count` 3, `max_alive` 10, `total` 0, `interval` 0, `spawn_on_enter` 1, `spawn_group` enemies,
  `snap_to_ground` 1, `cooldown` 0.5
