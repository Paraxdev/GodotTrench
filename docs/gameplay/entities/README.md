# Entity reference

The entities that ship with the addon, grouped by what you would use them for. Each entry says what the entity is for,
then lists its inputs (what other entities can tell it to do), its outputs (events you can wire onward) and its keys
(the settings in the Inspector). How outputs connect to inputs is explained in [Inputs and outputs](../io.md).

| Page | What you find there |
| --- | --- |
| [Logic](logic.md) | Invisible helpers that decide when things happen: relays, "only if" checks, counters, timers, a map start event and cutscene timelines |
| [Scripts and calls](scripting.md) | Run a GDScript snippet, call your game code, play an animation or print what a chain is doing |
| [Doors, movers and buttons](movers.md) | Brush entities that move: sliding and swinging doors, lifts, trains and buttons |
| [Triggers](triggers.md) | Invisible volumes that react when a body walks into them, to start events, hurt, teleport, push or spawn |
| [Spawners and paths](actors.md) | Spawning enemies or pickups, teleport destinations, path corners and scripted cutscene characters |
| [Props, lights and effects](effects.md) | Breakable physics props, static models, explosions, on screen text, switchable lights, sound and particles |

## Reading the entries

* Arguments in brackets are what an input accepts or what an output passes along, for example `trigger(activator)`
  receives the body or entity that started the chain.
* Every entity with inputs or outputs has a `targetname` key, except `logic_auto`. It is the name other entities use to
  reach it.
* A key you do not set uses the default listed.
* Distances are in map units, 32 per meter. Speeds are in meters per second, and `omni_range`, `spot_range` and
  `max_distance` are in meters.
* The input and output lists are what the editor's pickers offer. At runtime any method on the entity's script works as
  an input, so the picker is not the full list.

To add your own classes, see [Custom entities in GDScript](../custom-entities.md).

## Not in the library

**Worldspawn** and the plain FuncGodot classes `func_geo`, `func_detail`, `func_detail_illusionary` and
`func_illusionary` come from FuncGodot. Worldspawn holds the sun, ambient light, sky, fog and streaming settings, and
the Inspector shows it when nothing is selected.

**`info_player_start` and `trigger_area`** are known to the editor, but only the demo project defines them. Define
them in your own project, or they build as plain nodes and the build logs "No entity definition found".

**The player** is not an entity, it is a scene from your own game. The library expects it in the `player` group and on
collision layer 1, where triggers look, with a `take_damage(amount, source)` method that hurt triggers and explosions
call. A player on another layer needs that layer in *Trigger Collision Mask*, see
[Triggers](triggers.md#shared-inputs-outputs-and-keys). See [`godot/demo/player.gd`](https://github.com/Paraxdev/GodotTrench/blob/main/godot/demo/player.gd) for a
complete one.
