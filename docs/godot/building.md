# Building maps

The editor saves a `.gtm` file and never writes Godot scenes. The addon's `FuncGodotMap` node builds that file into
nodes that are saved with your scene, so nothing rebuilds at runtime unless you ask.

## The FuncGodotMap node

| Property | Meaning |
| --- | --- |
| Local Map File | `res://` path to a `.gtm` |
| Global Map File | A file outside the project, wins when both are set |
| Auto Rebuild On Save | Rebuild when GodotTrench saves this map, on by default |
| Map Settings | Defaults to the project setting `func_godot/default_map_settings` |
| Warm Up Shaders | Compiles the map's shaders on load so play does not stutter, see [Shader warm-up](warm-up.md) |

**Build Map** generates everything and emits `build_complete`, **Clear Map** removes it. The addon only builds `.gtm`,
so open a Quake `.map` or Hammer `.vmf` with *File > Import* in the editor and save it as `.gtm` first.

> **Warning:** A build deletes every child of the map node except [overlays](overlays.md). Keep your own nodes next to
> the map node, or in an overlay.

## Three ways to build

| Trigger | Builds |
| --- | --- |
| Save in GodotTrench | Every `FuncGodotMap` using the file in the scene open in the Godot editor |
| *Godot > Build in Godot* | The map as it is in the editor, saved or not, without writing the file |
| **Build Map** in Godot | The saved file |

The first two need a Godot editor with the addon and this project open, and only touch the open scene. A rebuild marks
the scene as modified, so save it in Godot to keep the result.

## Building from code

```gdscript
var map := FuncGodotMap.new()
map.map_settings = load("res://demo/demo_map_settings.tres")
map.local_map_file = "res://demo/maps/scripted_scene.gtm"
add_child(map)
map.build()
```

## Environment and sun

Worldspawn sky, fog and sun keys build a `WorldEnvironment` and a `sun` light into the map. The worldspawn key
`environment` picks what gets built:

| `environment` | Builds |
| --- | --- |
| `1` | The environment and the sun, the default |
| `sun_only` | Only the sun |
| `0` | Nothing |

A scene that already has a `WorldEnvironment` or `DirectionalLight3D` of its own keeps it, and the map adds none. So a
game with its own look wins, and of several maps in one scene the first one built lights it.

## Build report

`build()` returns a Dictionary, also kept in `build_report`, with the time of each step, entities per class, the vertex
count, and what the build could not find: entity classes without a definition, model paths, materials, tool textures
the map settings name differently, and targets that match nothing. Tick *Print Build Report* in *Build Flags*, or add
`FuncGodotMap.BuildFlags.PRINT_REPORT` to `build_flags` in code, to print it after every build.

```gdscript
var report := map.build()
if not report["missing_models"].is_empty():
	push_error("missing models: %s" % [report["missing_models"]])
```

Missing models, materials and tool textures also print one warning each, however many props share the path.

> **Note:** Overlays you add after `build()` are not there when the report looks for targets, so their targetnames show up
> under `unresolved_targets`.

## Damaged maps

A damaged map still builds from the parts that could be read, with a `[GTM]` warning for each part that could not. See
[the .gtm map format](../format/recovery.md).
