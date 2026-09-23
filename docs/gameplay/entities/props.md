# Props and explosions

These are point entities.

## prop_physics

A physics prop that can be thrown, damaged and broken, like crates and barrels. Damage down to 0 health, an impact
faster than `impact_speed` or `smash` breaks it. An explosive prop blasts when it breaks, after its `fuse` if it was
ignited.

* **Inputs:** `smash`, `ignite(activator)`, `push(direction)`, `take_damage(amount, source)`
* **Outputs:** `damaged(hp)` with the health left, `broken`
* **Keys:** `model`, `size` 16 16 16, `mass` 5.0, `health` 30, `explosive` 0, `explosion_radius` 192,
  `explosion_damage` 40, `fuse` 0.6, `impact_speed` 0 (m/s, 0 never breaks on impact), `debris_scene`

Its collision is a box of half extents `size`, and `model` is a scene for the looks only. `push` takes a velocity in map
units per second. `broken` fires after the blast, so whatever it triggers already sees the damage.

## prop_model

A static model with optional collision, for props that should not move.

* **Keys:** `model` (.bbmodel, .glb, .gltf or .tscn), `model_node`, `collision` convex (or none, trimesh), `scale` 1.0

Collision is only generated when the model has none of its own. Some files hold several variants side by side, like a
pack of fire hydrants in one glTF. `model_node` keeps only that node and its children, moved to the prop's origin.

## env_explosion

On `explode` it calls `damage_method` on every node within `radius` with `damage * (1 - distance / radius)`, pushes
rigid bodies away and spawns `effect_scene`.

* **Inputs:** `explode(activator)`
* **Outputs:** `exploded`
* **Keys:** `radius` 256, `damage` 50, `force` 8 (m/s at the center, scaled by mass), `damage_method` take_damage,
  `effect_scene`

Distance is measured to each node's origin, with no line of sight check. The blast reaches nodes outside the map too,
such as a player added next to the `FuncGodotMap`.
