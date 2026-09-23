# Actors, props and effects

These are point entities.

## info_spawner

Spawns a scene around itself, up to `max_alive` at a time and `total` overall (0 is unlimited). Spawned nodes join
`spawn_group` and are added next to the spawner.

* **Inputs:** `spawn`, `start`, `stop`, `toggle`, `kill_all`
* **Outputs:** `spawned(node)`, `all_dead`, `exhausted` (once, the first time `total` is reached)
* **Keys:** `scene`, `count` 1, `max_alive` 5, `total` 0, `interval` 0, `radius` 64, `start_active` 0,
  `spawn_on_ready` 0, `spawn_group` enemies, `snap_to_ground` 1

Spawned nodes have no targetname. Reach them with `@enemies`, or whichever group you chose.

## info_teleport_destination

Marks where `trigger_teleport` sends things. Only has a `targetname`.

## path_corner

One stop on a path, with `target` naming the next. Trains and walkers fire its `reached` output on arrival. Click
chains down quickly with the **Path** tool (Shift+P).

* **Outputs:** `reached(train)`
* **Keys:** `target`, `wait` 0, `speed` 0 (trains only)

## npc_walker

A scripted actor that walks a chain of path corners playing animations, for cutscenes rather than AI. It tweens its
position, with no gravity or collision.

* **Inputs:** `start`, `stop`, `walk_to(corner)`, `play_anim(name)`, `face(target)`
* **Outputs:** `arrived(corner)`, `reached_goal`, `finished`
* **Keys:** `model`, `target`, `speed` 2.0, `loop` 0, `start_active` 0, `walk_anim` walk, `idle_anim` idle,
  `face_travel` 1

`face` and `walk_to` accept any target, so `face` with parameter `!player` turns it towards the player.

## game_text

Shows a line of text in the world, on the HUD or both. `show` and `hide` toggle the label, not the node.

* **Inputs:** `show`, `hide`, `set_text(text)`, `flash(seconds)`
* **Outputs:** `shown`, `hidden`
* **Keys:** `text`, `place` world, hud or both, `start_visible` 0, `duration` 0, `text_color` 255 255 255,
  `world_size` 0.01, `billboard` 1

With `billboard` on, the world label faces the camera and draws through walls. Turn it off for signs, the label then
keeps the entity's angles and hides behind walls.

## prop_physics

A physics prop that can be thrown, damaged and broken, like crates and barrels. Damage, a hard impact or `smash` breaks
it. An explosive prop blasts when it breaks, after its `fuse` if it was ignited.

* **Inputs:** `smash`, `ignite(activator)`, `push(direction)`, `take_damage(amount, source)`
* **Outputs:** `damaged(hp)`, `broken`
* **Keys:** `model`, `size` 16 16 16 (half extents), `mass` 5.0, `health` 30, `explosive` 0, `explosion_radius` 192,
  `explosion_damage` 40, `fuse` 0.6, `impact_speed` 0, `debris_scene`

`broken` fires after the blast, so whatever it triggers already sees the damage.

## prop_model

A static model with optional collision, for props that should not move.

* **Keys:** `model` (.bbmodel, .glb, .gltf or .tscn), `model_node`, `collision` convex (or none, trimesh), `scale` 1.0

Some files hold several variants side by side, like a pack of fire hydrants in one glTF. `model_node` keeps only that
node and its children, moved to the prop's origin.

## env_explosion

On `explode`, damages every node within `radius` by `damage * (1 - distance / radius)` and pushes rigid bodies away.

* **Inputs:** `explode(activator)`
* **Outputs:** `exploded`
* **Keys:** `radius` 256, `damage` 50, `force` 8, `damage_method` take_damage, `effect_scene`

Distance is measured to each node's origin, with no line of sight check. The blast reaches nodes outside the map too,
such as a player next to the `FuncGodotMap`.

## light and light_spot

Switchable omni and spot lights. `switched` only fires when the state actually changes.

* **Inputs:** `turn_on`, `turn_off`, `toggle`
* **Outputs:** `switched(on)`
* **Keys:** `light_energy` 1.0 (1.5 for spots), `light_color`, `start_on` 1, `shadows` 0, `fixture`, `fixture_off`
  hide, and `omni_range` 10, or `spot_range` 15 and `spot_angle` 35

**Fixtures:** give the lamp's glowing geometry a targetname and put it in `fixture` (a trailing `*` matches a prefix,
so `hall_tube_*` covers every tube). It follows the light with no wiring: shown while on, hidden while off. With
`fixture_off` set to `dark` it stays visible with its glow off.

## env_sound

A positional sound. `max_distance` 0 means no limit.

* **Inputs:** `play`, `stop`, `toggle`
* **Outputs:** `finished`
* **Keys:** `sound`, `volume_db` 0, `max_distance` 0, `autoplay_on` 0, `loop_sound` 0

## env_particles

A particle effect for smoke, fire, sparks and dust. `burst` fires one shot.

* **Inputs:** `start`, `stop`, `toggle`, `burst`
* **Keys:** `amount` 32, `lifetime` 2.0, `one_shot` 0, `start_emitting` 0, `effect` rise, `area` 10 1 10

Without a process material of its own it builds one from `effect`: `rise` sends puffs upward, `rain` drops streaks
from a box `area` meters wide, high and deep.
