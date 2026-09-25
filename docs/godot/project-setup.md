# Project setup

The addon's defaults are enough for a first map, but updating the addon replaces them. Once you write your own
entities or want another texture folder, give the project its own resources.

## Your own resources

Three resources work together. The FGD file lists the entities your maps can use. The map settings tell a build where
textures are and which FGD to read. The game config tells the GodotTrench editor about both, so it offers the same
entities and textures while you edit.

The quickest route is to copy `demo_fgd.tres`, `demo_map_settings.tres` and `demo_game_config.tres` from `godot/demo`
into your project and adjust the paths. To make them from scratch:

1. Create a `FuncGodotFGDFile`. Set *Base Fgd Files* to `res://addons/func_godot/fgd/godottrench_default_fgd.tres`,
   so you keep the built in entities, and add your own entities under *Entity Definitions*.
2. Create a `FuncGodotMapSettings`. Set *Entity Fgd* to the FGD from step 1 and *Base Texture Dir* to your texture
   folder (default `res://textures`).
3. Create a `GodotTrenchGameConfig`. Set its *Fgd File* and *Map Settings* to the two resources above.
4. In *Project Settings*, point `func_godot/default_map_settings` at your map settings and `godottrench/game_config`
   at your game config.

> **Tip:** Set *Entity Name Property* in the map settings to `targetname`, as the demo does. Built nodes are then named
> after their targetnames, which keeps the scene tree readable.

## The game config

The editor learns your entities, texture folders and unit scale from `res://godottrench_game.json`, written by the
game config. The addon exports it again whenever Godot notices files changing. You can also export by hand from
*Project > Tools*, or headless, for example in CI:

```sh
godot --headless --path <project> --script res://addons/func_godot/src/godottrench/export_game_config_cli.gd
```

## Project settings

Most projects only change the first and third setting. The live link ones matter when another program already uses the
port.

| Setting | Default | Purpose |
| --- | --- | --- |
| `func_godot/default_map_settings` | the addon's defaults | Map settings for every map that does not set its own |
| `func_godot/default_inverse_scale_factor` | 32 | Map units per meter |
| `godottrench/game_config` | the addon's config | The config that writes `godottrench_game.json` |
| `godottrench/auto_export_game_config` | true | Export the game config again when files change |
| `godottrench/live_link_enabled` | true | Run the live link server that GodotTrench talks to |
| `godottrench/live_link_port` | 7842 | Must match *Godot live link* in the editor's preferences. Changes apply right away. Between 1024 and 65535, anything else falls back to 7842 with a warning |
| `godottrench/threaded_build` | true | Convert brushes and terrain chunks on the WorkerThreadPool, so big maps build faster |
| `godottrench/live_chunk_size` | 16.0 | In live mode, worldspawn geometry is split into chunks of this size in meters, so an edit only rebuilds its chunk |
| `godottrench/csharp_entity_dirs` | `res://` | Where C# entities are found at build time |
