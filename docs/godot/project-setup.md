# Project setup

The addon's defaults are enough for a first map, but updating the addon replaces them. Once you write your own
entities or want another texture folder, give the project its own resources.

## Your own resources

The quickest route is to copy `demo_fgd.tres`, `demo_map_settings.tres` and `demo_game_config.tres` from `godot/demo`
into your project and adjust the paths.

| Resource | Type | Set |
| --- | --- | --- |
| FGD file | `FuncGodotFGDFile` | *Base Fgd Files* to `res://addons/func_godot/fgd/godottrench_default_fgd.tres`. Your own entities go in *Entity Definitions* |
| Map settings | `FuncGodotMapSettings` | *Entity Fgd* to the FGD above, *Base Texture Dir* to your texture folder (default `res://textures`) |
| Game config | `GodotTrenchGameConfig` | *Fgd File* and *Map Settings* to the two above |

Then point `func_godot/default_map_settings` and `godottrench/game_config` in *Project Settings* at your map settings
and game config.

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

| Setting | Default | Purpose |
| --- | --- | --- |
| `func_godot/default_map_settings` | the addon's defaults | Map settings for every map that does not set its own |
| `func_godot/default_inverse_scale_factor` | 32 | Map units per meter |
| `godottrench/game_config` | the addon's config | The config that writes `godottrench_game.json` |
| `godottrench/auto_export_game_config` | true | Export again when files change |
| `godottrench/live_link_enabled` | true | Run the live link server, read when the Godot editor starts |
| `godottrench/live_link_port` | 7842 | Must match *Godot live link* in the editor's preferences, read when the Godot editor starts |
| `godottrench/threaded_build` | true | Convert brushes and terrain chunks on the WorkerThreadPool |
| `godottrench/live_chunk_size` | 16.0 | Live mode chunk size for worldspawn, in meters |
| `godottrench/csharp_entity_dirs` | `res://` | Where C# entities are found at build time |
