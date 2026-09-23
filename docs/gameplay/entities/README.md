# Entity reference

Every entity class that ships with the addon, split by what you use it for:

| Page | Entities |
| --- | --- |
| [Logic](logic.md) | Relays, branches, counters, timers, sequences, scripts |
| [Doors, movers and buttons](movers.md) | Doors, gates, platforms, trains, buttons |
| [Triggers](triggers.md) | Volumes that react to bodies entering them |
| [Actors, props and effects](actors.md) | Spawners, paths, NPC walkers, text, props, lights, sound, particles |

## Reading the entries

Each entry lists inputs, outputs and keys with their defaults. Arguments in brackets are what an input accepts or an
output passes along.

* Every entity with I/O has a `targetname`, except `logic_auto`.
* Key values are converted to int, float, bool, Vector3 or Color from the FGD defaults. Missing keys use the defaults
  listed.
* Distances are in map units, 32 per meter. `omni_range`, `spot_range` and `max_distance` are in meters.
* The lists are what the editor's pickers offer. Any public method on the script works as an input at runtime.

The definitions live in `crates/gt_formats/src/builtin_entities.json`. The Godot copies in
`addons/func_godot/fgd/godottrench` are generated from it.

## Not in the library

**Worldspawn** and the plain FuncGodot classes `func_geo`, `func_detail`, `func_detail_illusionary` and
`func_illusionary` come from the base definitions. Worldspawn holds the sun, ambient light, sky, fog and streaming
settings. The Inspector shows it when nothing is selected.

**`info_player_start` and `trigger_area`** are known to the editor but only defined by the demo project. Add
definitions for them in your own project, or they build as plain nodes with a warning.

**The player** is not an entity. The library expects it to be in the `player` group, on collision layer 1, with a
`take_damage(amount, source)` method. `godot/demo/player.gd` is a complete example.
