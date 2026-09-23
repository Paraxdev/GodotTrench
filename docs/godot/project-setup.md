# Project setup

The addon's defaults are enough for a first map. They live inside the addon folder though, and are replaced whenever
you update it. Once you write your own entities or want another texture folder, give the project its own resources.

## Your own resources

The quickest route is to copy the demo's `demo_fgd.tres`, `demo_map_settings.tres` and `demo_game_config.tres` from
`godot/demo` into your project and adjust the paths.

| Resource | Type | Set |
| --- | --- | --- |
| FGD file | `FuncGodotFGDFile` | *Base Fgd Files* to `res://addons/func_godot/fgd/godottrench_default_fgd.tres`. Your own entities go in *Entity Definitions* |
| Map settings | `FuncGodotMapSettings` | *Entity Fgd* to the FGD above, *Base Texture Dir* to your texture folder |
| Game config | `GodotTrenchGameConfig` | *Fgd File* and *Map Settings* to the two above |

Then register them in *Project Settings*:

| Setting | Value |
| --- | --- |
| `func_godot/default_map_settings` | Your map settings |
| `godottrench/game_config` | Your game config |

> **Tip:** Set *Entity Name Property* in the map settings to `targetname`. Built nodes are then named after their
> targetnames, which makes the scene tree much easier to read.

## The game config

`godottrench_game.json` is how the editor learns about your project. It holds every class from the config's FGD with
its properties, size, colour and the inputs and outputs found in its script, plus C# entities, the texture folder and
the unit scale.

The addon exports it again about 1.5 seconds after Godot notices files changing. You can also export by hand from
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
| `godottrench/live_link_enabled` | true | Run the live link server in the Godot editor |
| `godottrench/live_link_port` | 7842 | Must match the port in the editor's preferences |
| `godottrench/threaded_build` | true | Build brushes and meshes on the WorkerThreadPool |
| `godottrench/live_chunk_size` | 16.0 | Live mode chunk size for worldspawn, in meters |
| `godottrench/csharp_entity_dirs` | `res://` | Where C# entities are found at build time |
