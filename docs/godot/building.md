# Building maps

The editor saves a `.gtm` file and never writes Godot scenes itself. In Godot, the addon's `FuncGodotMap` node reads
that file and generates meshes, collision and entity nodes from it. The result is saved with your scene like any
hand-made node, so nothing rebuilds at runtime unless you ask for it.

## The FuncGodotMap node

Add a `FuncGodotMap` to a scene and set it up in the Inspector:

| Property | What it does |
| --- | --- |
| Local Map File | The `res://` path of the `.gtm` to build. This is the normal choice. |
| Global Map File | A map file outside the project. When both are set, this one wins. |
| Auto Rebuild On Save | Rebuilds the node whenever GodotTrench saves this map. On by default. |
| Map Settings | Where textures, materials and entity definitions come from and how big a map unit is. Left empty, it uses the project setting `func_godot/default_map_settings`, see [Project setup](project-setup.md). |
| Warm Up Shaders | Compiles the map's shaders while it loads, so the first seconds of play do not stutter. See [Shader warm-up](warm-up.md). |

**Build Map** in the Inspector generates everything and emits `build_complete`, **Clear Map** removes it again.

The addon only builds `.gtm` files. To use a Quake `.map` or Hammer `.vmf`, open it with *File > Import* in the editor
and save it as `.gtm` first.

> **Warning:** A build deletes every child of the map node except [overlays](overlays.md). Keep your own nodes next to
> the map node, or in an overlay.

## Three ways to build

| Trigger | What gets built | When to use it |
| --- | --- | --- |
| Save in GodotTrench | Every `FuncGodotMap` using that file in the scene open in the Godot editor | The everyday loop, save and look at the result in Godot |
| *Godot > Build in Godot* | The map as it currently is in the editor, saved or not, without writing the file | To try an unsaved change in Godot without committing to it |
| **Build Map** in Godot | The saved file | When GodotTrench is closed, or to rebuild a scene you just opened |

The first two need a Godot editor running with the addon and this project open, and they only touch the scene open
there. A rebuild marks that scene as modified, so save it in Godot to keep the result.

## Building from code

Maps can also be built while the game runs, for example to load a level picked from a menu:

```gdscript
var map := FuncGodotMap.new()
map.map_settings = load("res://demo/demo_map_settings.tres")
map.local_map_file = "res://demo/maps/scripted_scene.gtm"
add_child(map)
map.build()
```

## Environment and sun

The sky, fog and sun keys on worldspawn build a `WorldEnvironment` and a light named `sun` into the map, so a map looks
lit without any setup in Godot. The worldspawn key `environment` picks what gets built:

| `environment` | Builds |
| --- | --- |
| `1` | The environment and the sun. This is the default. |
| `sun_only` | Only the sun, for a game that brings its own `WorldEnvironment` |
| `0` | Nothing |

If the scene already has a `WorldEnvironment` or `DirectionalLight3D` of its own, the map keeps it and adds none, so a
game with its own look always wins. When several maps share one scene, the first one built lights it.

## Build report

`build()` returns a Dictionary, also kept in `build_report`. It tells you how long each step took, how many entities of
each class were built and how many vertices the map has. More usefully, it lists what the build could not find, which
is usually a typo or a file that was renamed or never committed:

| Key | What it lists | What you see in the game |
| --- | --- | --- |
| `missing_classes` | Entity classes that no entity definition describes | The entity builds as a plain node without its script, a point entity as an empty `Marker3D` |
| `missing_models` | Model paths that do not exist, with the entities using each | The prop has no model |
| `missing_materials` | Face textures with neither a material file nor an image | The faces show the placeholder texture |
| `unknown_tool_textures` | Tool textures like `special/clip` that the map settings name differently | The faces are drawn with the tool texture instead of being left out |
| `unresolved_targets` | Outputs and target keys whose target matches no targetname in the map or its overlays | The output fires at nothing |

Tick *Print Build Report* in *Build Flags*, or add `FuncGodotMap.BuildFlags.PRINT_REPORT` to `build_flags` in code, to
print it after every build. A test or CI check can read the Dictionary instead:

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
