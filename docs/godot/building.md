# Building maps

The editor never writes Godot scenes. It saves a `.gtm` file, and the addon's `FuncGodotMap` node turns that file into
nodes.

## The FuncGodotMap node

| Property | Meaning |
| --- | --- |
| Local Map File | `res://` path to a `.gtm`, `.map` or `.vmf` |
| Global Map File | A file outside the project. Wins when both are set |
| Map Settings | Falls back to the project setting `func_godot/default_map_settings` |
| Auto Rebuild On Save | Rebuild when GodotTrench saves this map. On by default |

**Build Map** generates everything, **Clear Map** removes it. The built nodes belong to the scene and are saved in the
`.tscn`, so nothing rebuilds at runtime unless you ask.

## What a build does

1. Deletes every child of the map node, except [overlays](overlays.md).
2. Turns brushes into meshes and collision.
3. Creates entities, applies their properties and connects their outputs.
4. Adds terrain, scatter, the worldspawn environment and the chunk streamer.
5. Emits `build_complete`.

> **Warning:** Anything you put under the map node by hand is gone after the next build. Keep your own nodes next to
> the map node, or in an [overlay](overlays.md).

## Three ways to build

| Trigger | Builds |
| --- | --- |
| Save in GodotTrench | Every `FuncGodotMap` using the file in the scene open in the Godot editor |
| *Godot > Build in Godot* | The map as it is in the editor, saved or not, without writing the file |
| **Build Map** in Godot | The saved file |

Maps in scenes that are not open are left alone until you open them and build. A rebuild marks the scene as modified,
so save it in Godot to keep the result.

The editor's status bar reports how many map nodes were rebuilt, that no open scene uses the map, or that no Godot with
the addon answered.

## Building from code

```python
var map := FuncGodotMap.new()
map.map_settings = load("res://demo/demo_map_settings.tres")
map.local_map_file = "res://demo/maps/scripted_scene.gtm"
add_child(map)
map.build()
```

## Streaming big maps

Large outdoor maps can show and hide their visuals by distance. Select nothing so the Inspector shows worldspawn, then
set:

| Key | Default | Meaning |
| --- | --- | --- |
| `chunk_streaming` | 0 | Set to 1 to turn it on |
| `chunk_size` | 2048 | Cell size in map units |
| `load_radius` | 8192 | How far from the camera cells stay visible |

Streaming only hides meshes, collision and scripts keep running everywhere. It saves rendering cost, not physics or
logic. The streamer emits `area_loaded(key, bounds)` and `area_unloaded(key, bounds)` if you want to stream your own
content alongside.

## Build performance

Brushes and terrain chunks are converted on the WorkerThreadPool, meshes and shapes are created on the main thread. The
project setting `godottrench/threaded_build` turns the threaded steps off. To time each step on the showcase maps:

```sh
godot --headless --path godot --script res://tests/bench_build.gd -- runs=5 threaded=1
```
