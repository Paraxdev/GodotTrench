# Building maps

The editor never writes Godot scenes. It saves a `.gtm` file, and the addon's `FuncGodotMap` node turns that file into
nodes that are saved with your scene, so nothing rebuilds at runtime unless you ask.

## The FuncGodotMap node

| Property | Meaning |
| --- | --- |
| Local Map File | `res://` path to a `.gtm` |
| Global Map File | A file outside the project, wins when both are set |
| Auto Rebuild On Save | Rebuild when GodotTrench saves this map, on by default |
| Map Settings | Defaults to the project setting `func_godot/default_map_settings` |

**Build Map** generates everything and **Clear Map** removes it. A build first deletes every child of the map node
except [overlays](overlays.md), then adds brushes as meshes and collision, entities with their properties and outputs,
terrain, scatter, the worldspawn environment and the [chunk streamer](streaming.md). It emits `build_complete` when done.

The addon only builds `.gtm`. Open a Quake `.map` or Hammer `.vmf` in the editor with *File > Import* and save it as
`.gtm` first.

> **Warning:** Anything you put under the map node by hand is gone after the next build. Keep your own nodes next to
> the map node, or in an [overlay](overlays.md).

## Three ways to build

| Trigger | Builds |
| --- | --- |
| Save in GodotTrench | Every `FuncGodotMap` using the file in the scene open in the Godot editor |
| *Godot > Build in Godot* | The map as it is in the editor, saved or not, without writing the file |
| **Build Map** in Godot | The saved file |

The first two need a Godot editor with the addon and this project open. Scenes that are not open are left alone until
you open them and build. A rebuild marks the scene as modified, so save it in Godot to keep the result.

## Building from code

```gdscript
var map := FuncGodotMap.new()
map.map_settings = load("res://demo/demo_map_settings.tres")
map.local_map_file = "res://demo/maps/scripted_scene.gtm"
add_child(map)
map.build()
```

## Damaged maps and build speed

A damaged map still builds from the parts that could be read, with a `[GTM]` warning for each part that could not. See
[the .gtm map format](../format/recovery.md).

Brushes and terrain chunks are converted on the WorkerThreadPool, while meshes and shapes are created on the main
thread. The project setting `godottrench/threaded_build` turns the threaded steps off. To time each step on the
showcase maps:

```sh
godot --headless --path godot --script res://tests/bench_build.gd -- runs=5 threaded=1
```
