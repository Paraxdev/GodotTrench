# Entity reference

The entities that ship with the addon, grouped by what you would use them for. Each entry says what the entity is
for, then lists its inputs (what other entities can tell it to do), its outputs (events you can wire onward) and its
keys (the settings in the Inspector). How outputs connect to inputs is explained in [Inputs and outputs](../io.md).

| Page | What you find there |
| --- | --- |
| [Logic](logic.md) | Invisible helpers that route signals: relays, branches, counters, timers, `logic_auto` and sequences |
| [Scripts and calls](scripting.md) | `logic_script`, `logic_call`, `logic_animate` and `logic_debug`, for reaching game code, animations and debugging |
| [Doors, movers and buttons](movers.md) | Brush entities that move: doors, platforms, trains and buttons |
| [Triggers](triggers.md) | Invisible volumes that react when a body walks into them |
| [Spawners and paths](actors.md) | Spawners, teleport destinations, path corners and scripted NPC walkers |
| [Props, lights and effects](effects.md) | Physics and model props, explosions, text, switchable lights, sound and particles |

## Reading the entries

* Arguments in brackets are what an input accepts or an output passes along.
* Every entity with I/O has a `targetname` key, except `logic_auto`. It is the name other entities use to reach it.
* A missing key uses the default listed.
* Distances are in map units, 32 per meter. Speeds are in meters per second, and `omni_range`, `spot_range` and
  `max_distance` are in meters.
* The lists are what the editor's pickers offer. Any method on the script works as an input at runtime.

To add your own classes, see [Custom entities in GDScript](../custom-entities.md).

## Not in the library

**Worldspawn** and the plain FuncGodot classes `func_geo`, `func_detail`, `func_detail_illusionary` and
`func_illusionary` come from FuncGodot. Worldspawn holds the sun, ambient light, sky, fog and streaming settings, and
the Inspector shows it when nothing is selected.

**`info_player_start` and `trigger_area`** are known to the editor, but only the demo project defines them. Define
them in your own project, or they build as plain nodes and the build logs "No entity definition found".

**The player** is not an entity. The library expects it in the `player` group, on collision layer 1 where triggers
look, with a `take_damage(amount, source)` method. See
[`godot/demo/player.gd`](https://github.com/Paraxdev/GodotTrench/blob/main/godot/demo/player.gd) for a complete one.
