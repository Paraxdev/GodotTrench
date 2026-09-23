# Props and explosions

## prop_physics

A crate or barrel that can be thrown, damaged and broken. It breaks at 0 health, on an impact faster than
`impact_speed` or on `smash`. An explosive prop blasts when it breaks, after its `fuse` if it was ignited.

* **Inputs:** `smash`, `ignite(activator)`, `push(direction)`, `take_damage(amount, source)`
* **Outputs:** `damaged(hp)` with the health left, `broken`
* **Keys:** `model`, `size` 16 16 16, `mass` 5.0, `health` 30, `explosive` 0, `explosion_radius` 192,
  `explosion_damage` 40, `fuse` 0.6, `impact_speed` 0 (m/s, 0 never breaks on impact), `debris_scene`

Its collision is a box with `size` as half extents, so the default is one meter wide, and `model` is only for looks.
`push` takes a velocity in map units per second. `broken` fires after the blast, so whatever it triggers already sees
the damage.

## prop_model

A static model with optional collision, for props that should not move.

* **Keys:** `model` (.bbmodel, .glb, .gltf or .tscn), `model_node`, `collision` convex (or none, trimesh), `scale` 1.0

Collision is only generated when the model has none of its own. For a file that holds several variants side by side,
`model_node` keeps only the named node and moves it to the prop's origin.

## env_explosion

On `explode` it calls `damage_method` on every node within `radius` with `damage * (1 - distance / radius)`, pushes
rigid bodies away and spawns `effect_scene`.

* **Inputs:** `explode(activator)`
* **Outputs:** `exploded`
* **Keys:** `radius` 256, `damage` 50, `force` 8 (m/s at the center, whatever the mass), `damage_method` take_damage,
  `effect_scene`

Distance is measured to each node's origin with no line of sight check, so it hurts through walls.
