# Baked lighting in Godot

A map [baked in the editor](../editor/light-baking.md) carries its light map in the `.gtm`, so Godot does not bake
anything itself. Each build turns the stored bake into Godot's own lightmap node.

## What the build adds

| Part | What it gets |
| --- | --- |
| A `LightmapGI` named `baked_lighting` | A child of the `FuncGodotMap`, holding the light map, the sun shadow mask and the light probes |
| Baked meshes | Their light map coordinates as UV2 and a static GI mode, streamed chunks included |
| Lights and the sun | Godot's bake mode from their `bake_mode` or `sun_bake_mode`, see [Light keys](../editor/light-baking.md#light-keys) |

With the sun on `bounce`, the sun stays real time and the baked shadow mask shades what lies past its shadow distance.

Faces that changed after the bake point at one extra row of the map's average light, so they look plausible but flat
until you bake again.

## Light probes

Light probes are points in the open space of the map that store the light arriving from every direction. Moving
objects, like the player, props and doors, take their light from the probes around them, so they pick up the baked
bounce light and blend in.

The bake places probes on a 2 meter grid over the box around the baked geometry and drops the ones inside solids. A
`light_probe` entity adds one more where the grid is too coarse, such as a narrow corridor or a stair. A map gets
about 1000 probes at most, hand placed ones included, so the grid spreads out on a bigger map.

## Probe entities

| Entity | Kind | Godot node | Use it for |
| --- | --- | --- | --- |
| `light_probe` | Point | `LightmapProbe` | An extra light probe for the bake, see above |
| `env_reflection_probe` | Brush volume | `ReflectionProbe` sized to the brush | Reflections that match a room on glossy floors and metal |
| `env_voxel_gi` | Brush volume | `VoxelGI` sized to the brush | Real time bounce light that follows moving lights and objects |

The brushes of the two volumes only give the box, the build removes them. Their keys are listed in
[Props, lights and effects](../gameplay/entities/effects.md#lightprobe-envreflectionprobe-and-envvoxelgi).

An `env_voxel_gi` is baked in the Godot editor each time the map builds. Set `bake_on_build` to 0 on a big volume to
bake it by hand with the **Bake** button of the `VoxelGI` node instead. Maps built while the game runs skip that bake.

## Using another lightmapper

Untick **Use Baked Lighting** on the `FuncGodotMap` to leave the stored bake out, for example to bake with Godot's
own `LightmapGI`. The build then adds no `baked_lighting` node and leaves the lights' bake modes alone. Tick *Unwrap
UV2* in *Build Flags* so the meshes get the UV2 a Godot bake needs.
