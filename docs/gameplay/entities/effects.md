# Props, lights and effects

## prop_physics

A crate or barrel that can be thrown, damaged and broken. It breaks at 0 health, on an impact faster than
`impact_speed`, on `smash`, or `fuse` seconds after `ignite`. An explosive prop blasts when it breaks.

| Key | Default | What it does |
| --- | --- | --- |
| `model` | | The scene it looks like, only for looks |
| `size` | 16 16 16 | Half extents of its box collision, so the default is one meter wide |
| `mass` | 5.0 | Mass in kilograms |
| `health` | 30 | Damage it takes before breaking |
| `explosive` | 0 | Blasts when it breaks |
| `explosion_radius`, `explosion_damage` | 192, 40 | Size and strength of that blast |
| `fuse` | 0.6 | Seconds an ignited prop burns before it breaks |
| `impact_speed` | 0 | Breaks when it hits something faster than this in m/s. 0 never breaks on impact |
| `debris_scene` | | A scene spawned where it broke, like planks |

* **Inputs:** `smash`, `ignite(activator)`, `push(direction)`, `take_damage(amount, source)`
* **Outputs:** `damaged(hp)` with the health left, `broken`

`push` takes a velocity in map units per second. `broken` fires after the blast, so whatever it triggers already sees
the damage.

## prop_model

A static model with optional collision, for furniture, rocks and other props that should not move.

* **Keys:** `model` (.bbmodel, .glb, .gltf or .tscn), `model_node`, `collision` convex (or none, trimesh), `scale` 1.0

`convex` wraps the model in one cheap hull, fine for boxy shapes. `trimesh` follows every triangle, for shapes the
player walks through or on, like an arch. Collision is only generated when the model has none of its own. For a file
that holds several variants side by side, `model_node` keeps only the named node and moves it to the prop's origin.

## env_explosion

A scripted blast, like a wall charge. On `explode` it calls `damage_method` on every node within `radius` with
`damage * (1 - distance / radius)`, pushes rigid bodies away and spawns `effect_scene` for the flash and bang.

* **Inputs:** `explode(activator)`
* **Outputs:** `exploded`
* **Keys:** `radius` 256, `damage` 50, `force` 8 (m/s at the center, whatever the mass), `damage_method` take_damage,
  `effect_scene`

Distance is measured to each node's origin with no line of sight check, so it hurts through walls.

## game_text

Shows a line of text in the world, on the HUD or both, for hints, signs or cutscene subtitles.

| Key | Default | What it does |
| --- | --- | --- |
| `text` | | The text to show |
| `place` | world | `world` floats at the entity, `hud` shows it on screen, `both` does both |
| `start_visible` | 0 | Shown from map load |
| `duration` | 0 | Hides it after that many seconds, 0 keeps it up until `hide` |
| `text_color` | 255 255 255 | Text color |
| `world_size` | 0.01 | World label size in meters per pixel |
| `billboard` | 1 | The world label faces the camera and draws through walls. Turn it off for signs, which then keep the entity's angles and hide behind walls |

* **Inputs:** `show`, `hide`, `set_text(text)`, `flash(seconds)`
* **Outputs:** `shown`, `hidden`

`show` and `hide` switch the label, not the node. `flash` shows it for its parameter in seconds, else `duration`,
else 2.

## light and light_spot

Omni and spot lights that inputs switch on and off, for light switches, alarms and flickering lamps. `switched` only
fires when the state actually changes.

* **Inputs:** `turn_on`, `turn_off`, `toggle`
* **Outputs:** `switched(on)`
* **Keys:** `light_energy` 1.0 (1.5 for spots), `light_color` 255 255 255, `start_on` 1, `shadows` 0 (costs more to
  render), `fixture`, `fixture_off` hide, and `omni_range` 10, or `spot_range` 15 and `spot_angle` 35 (the cone's half
  angle in degrees)

**Fixtures** keep the lamp model in step with the light. Give the lamp's glowing geometry a targetname and put it in
`fixture`, a trailing `*` matches a prefix so `hall_tube_*` covers every tube. It is shown while the light is on and
hidden while off, with no wiring. With `fixture_off` set to `dark` it stays visible with its emission off.

## env_sound

A sound that plays from a point in the level, for alarms, machinery or a crash wired to an event.

* **Inputs:** `play`, `stop`, `toggle`
* **Outputs:** `finished`
* **Keys:** `sound` (a `res://` audio file), `volume_db` 0, `max_distance` 0 (no limit), `autoplay_on` 0 (plays from
  map load), `loop_sound` 0

## env_particles

A particle effect for fire, smoke, sparks, dust and rain, with built in looks so you need not build one yourself.
`burst` emits one shot, then a running effect carries on.

* **Inputs:** `start`, `stop`, `toggle`, `burst`
* **Outputs:** `finished`
* **Keys:** `amount` 32 (particles alive at once), `lifetime` 2.0, `one_shot` 0, `start_emitting` 0, `effect` rise,
  `color` 255 255 255, `size` 0, `blend` auto, `area` 10 1 10

`effect` is `rise` (glowing dust or embers), `fire`, `smoke`, `sparks` or `rain` (falls from a box `area` meters
wide, high and deep). `color` tints it, and white keeps the effect's own colors. `size` sets the particle size in
meters, 0 uses the effect's own. `blend` is `add` for glow or `mix` for smoke. The defaults of each effect look right
on their own, so start with only `effect`.
