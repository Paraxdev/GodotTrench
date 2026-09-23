# Text, lights and effects

## game_text

Shows a line of text in the world, on the HUD or both. `show` and `hide` switch the label, not the node.

* **Inputs:** `show`, `hide`, `set_text(text)`, `flash(seconds)`
* **Outputs:** `shown`, `hidden`
* **Keys:** `text`, `place` world (or hud, both), `start_visible` 0, `duration` 0, `text_color` 255 255 255,
  `world_size` 0.01, `billboard` 1

`duration` hides it again after that many seconds, 0 keeps it up until `hide`. `flash` shows it for its parameter in
seconds, else `duration`, else 2. With `billboard` on, the world label faces the camera and draws through walls. Turn
it off for signs, which then keep the entity's angles and hide behind walls.

## light and light_spot

Switchable omni and spot lights. `switched` only fires when the state actually changes.

* **Inputs:** `turn_on`, `turn_off`, `toggle`
* **Outputs:** `switched(on)`
* **Keys:** `light_energy` 1.0 (1.5 for spots), `light_color` 255 255 255, `start_on` 1, `shadows` 0, `fixture`,
  `fixture_off` hide, and `omni_range` 10, or `spot_range` 15 and `spot_angle` 35

**Fixtures:** give the lamp's glowing geometry a targetname and put it in `fixture`, a trailing `*` matches a prefix
so `hall_tube_*` covers every tube. It is shown while the light is on and hidden while off, with no wiring. With
`fixture_off` set to `dark` it stays visible with its emission off.

## env_sound

A positional sound.

* **Inputs:** `play`, `stop`, `toggle`
* **Outputs:** `finished`
* **Keys:** `sound`, `volume_db` 0, `max_distance` 0 (no limit), `autoplay_on` 0, `loop_sound` 0

## env_particles

A particle effect for fire, smoke, sparks and dust. `burst` emits one shot, then a running effect carries on.

* **Inputs:** `start`, `stop`, `toggle`, `burst`
* **Outputs:** `finished`
* **Keys:** `amount` 32, `lifetime` 2.0, `one_shot` 0, `start_emitting` 0, `effect` rise, `color` white, `size` 0,
  `blend` auto, `area` 10 1 10

`effect` is `rise` (glowing dust or embers), `fire`, `smoke`, `sparks` or `rain` (falls from a box `area` meters
wide, high and deep). `color` tints it, `size` sets the particle size in meters, and `blend` is `add` for glow or
`mix` for smoke. The defaults of each effect look right on their own.
