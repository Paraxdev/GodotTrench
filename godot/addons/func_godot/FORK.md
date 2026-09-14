# FuncGodot (TrenchDot fork)

This addon is a fork of [FuncGodot](https://github.com/func-godot/func_godot_plugin) (MIT, see `LICENSE`),
based on upstream commit `cdf27e3cd8369b405a0ac48649e141238f5cfad5` (2026-09-07).
Upstream class names are kept so the fork stays a drop-in replacement and upstream changes merge cleanly.

## Additions (new files)

| file | purpose |
|------|---------|
| `src/trenchdot/tdm_parser.gd` | Parses TrenchDot `.tdm` maps into `FuncGodotData`: layers, groups, omitted layers, brush and point entities, exact brush vertices, Valve 220 UVs, prefab instances (transform, UV lock, targetname fixup). |
| `src/trenchdot/runtime/trenchdot_io.gd` | Hammer style entity I/O: targetname lookup (with `*` wildcard, `!self`, `!activator`), parameter parsing, input dispatch to methods, properties or built-in inputs (`kill`, `show`, `hide`, `enable`, `disable`, `toggle`). |
| `src/trenchdot/runtime/trenchdot_output.gd` | `TrenchDotOutput` node, one per output. Connects itself to the parent's signal when entering the tree and honours delay and fire count. |
| `src/trenchdot/trenchdot_game_config.gd` | `TrenchDotGameConfig` resource. Exports `trenchdot_game.json` for the editor: entity definitions from an FGD resource (typed properties, sizes, colors, node classes, scenes) plus I/O outputs and inputs taken from script signals and methods. |
| `src/trenchdot/trenchdot_editor_integration.gd` | Project settings, automatic game config export on file system changes, and the live link TCP server (`127.0.0.1:7842`) that rebuilds `FuncGodotMap` nodes when TrenchDot saves a map. |
| `src/trenchdot/export_game_config_cli.gd` | Headless export: `godot --headless --path <project> --script res://addons/func_godot/src/trenchdot/export_game_config_cli.gd` |
| `game_config/trenchdot/trenchdot_game_config.tres` | Default game config resource. |
| `src/trenchdot/trenchdot_displacement.gd` | Displacement grids (same triangulation as the editor), surface arrays with blend weights in vertex color alpha, trimesh collision triangles. |
| `src/trenchdot/runtime/trenchdot_decal.gd` | `TrenchDotDecal`: decal entities with `texture`, `size` (map units) and `modulate` properties. |
| `src/trenchdot/runtime/td_blend.gdshader` | Two texture blend shader driven by vertex color alpha, for displacements and vertex paint. |
| `src/trenchdot/trenchdot_mesh.gd` | `TrenchDotMesh`: polygon mesh nodes (ear clipping triangulation matching the editor, explicit UVs, smoothing angle) as brush data that skips convex clipping. |
| `src/trenchdot/runtime/trenchdot_terrain.gd` | `TrenchDotTerrain`: heightmap terrain nodes as chunked meshes with a `HeightMapShape3D` collider, holes and a splat material. |
| `src/trenchdot/runtime/td_terrain.gdshader` | Four layer splat shader for terrains, triplanar on steep slopes and projected in map space like the editor. Pixel art projects get a nearest filtered variant when the layer materials use nearest filtering. |
| `src/trenchdot/runtime/trenchdot_environment.gd` | `TrenchDotEnvironment`: WorldEnvironment with a procedural sky, fog and a shadowed sun built from worldspawn keys (`sun_angles`, `sun_color`, `sun_energy`, `ambient_color`, `sky_top_color`, `sky_horizon_color`, `sky_ground_color`, `fog_color`, `fog_density`), matching the editor's lit preview. Worldspawn `environment` = `0` turns it off. |
| `src/trenchdot/trenchdot_bbmodel.gd` | `TrenchDotBBModel`: Blockbench parser (cubes, meshes, outliner pivots and rotations, box UV, embedded textures) building a scene with optional collision. |
| `src/trenchdot/bbmodel_import_plugin.gd` | Imports `.bbmodel` files as `PackedScene` resources (scale, collision, nearest filtering). |
| `src/trenchdot/runtime/trenchdot_prop.gd` | `TrenchDotProp`: prop entity instancing a `.bbmodel`, `.glb`, `.gltf` or `.tscn` from its `model` property with none, convex or trimesh collision. |
| `src/trenchdot/runtime/trenchdot_hot_reload.gd` | `TrenchDotHotReload`: autoload for running games that rebuilds maps when the editor saves them (`127.0.0.1:7843`). |
| `src/trenchdot/runtime/trenchdot_scatter.gd` | `TrenchDotScatter`: scatter sets (trees, rocks, foliage) as one MultiMesh per model mesh, one static body per model with a shared shape, or instanced scenes for scripted models. Keeps a copy of the instance transforms when built headless, because the dummy renderer drops MultiMesh buffers. |
| `src/trenchdot/trenchdot_face_cull.gd` | `TrenchDotFaceCull`: leaves faces out of the visual mesh when coplanar faces of other static solids fully cover them (same direction by priority, open meshes first, or back to back between closed solids), the rule the editor preview uses. Collision keeps every face. |
| `src/trenchdot/runtime/trenchdot_blend.gd` | `TrenchDotBlend`: materials for faces with a blend material (`base\|blend` texture keys), mixed by vertex color alpha through `td_blend.gdshader`. |
| `src/trenchdot/runtime/trenchdot_debug_overlay.gd` | `TrenchDotDebugOverlay`: in-game I/O event log and trigger volume display, toggled with F3. |
| `src/trenchdot/trenchdot_csharp.gd` | `TrenchDotCSharp`: reads C# sources as text for `[TrenchDotEntity]` classes, their `[Export]` properties, `[Signal]` outputs and `[TrenchDotInput]` inputs, and applies map properties to the PascalCase members. |
| `src/trenchdot/entities/td_*.gd` | Gameplay entity library: doors (sliding, hinged with `open_away`), gates, platforms, trains with path corners, buttons, triggers (`once`, `multiple`, `call`, `hurt`, `teleport`, `push`, `spawn_area`), spawners and logic (`call`, `relay`, `timer`, `counter`, `auto`, `debug`). |
| `fgd/trenchdot/*.tres` | FGD resources for that library, generated by the editor (`cargo run -p td_editor --example export_addon_fgd`) from the same definitions it uses, with gizmos, inputs and outputs in the class metadata. |

## Changes to upstream files

All changes are small and marked with `TrenchDot` comments.

* `src/core/data.gd`: `FaceData.exact_vertices`, `FaceData.props`, vertex colors and displacement arrays (including explicit `disp_uvs` and `disp_colors`), `BrushData.exact`, `BrushData.has_disp`, `BrushData.is_mesh`, `BrushData.closed`, `BrushData.node_id`, `FaceData.render_hidden`, `ParseData.terrains`, `EntityData.outputs`, `EntityData.node`, pending shape data.
* `src/core/parser.gd`: `.tdm` branch in `parse_map_data`.
* `src/core/geometry_generator.gd`:
  * exact brushes skip hyperplane clipping,
  * faces hidden by `TrenchDotFaceCull` (run before surface generation) are skipped for visuals but kept for collision,
  * displacement faces emit their grid instead of the flat face, other faces of a displacement brush are dropped (Hammer
    behaviour), displacements collide as a trimesh even for convex collision entities, vertex colors are written to
    `ARRAY_COLOR` and follow their vertices through face winding,
  * **bug fix**: `generate_entity_surfaces` no longer runs on the worker thread pool. Creating `ArrayMesh` and shape
    resources from several threads at once corrupted server state and crashed Godot on exit whenever a map had more than one
    brush entity. Vertex generation and winding stay threaded. Shape resources are created after surface generation.
* `src/core/entity_assembler.gd`: remembers the node of each entity and sets up I/O after assembly.
* `src/map/func_godot_map.gd`: accepts `*.tdm`, `auto_rebuild_on_save` for the live link, builds terrains and the worldspawn environment after the entity assembler.
* `src/import/quake_map_import_plugin.gd`: imports `.tdm` so maps ship in exported games.
* `src/func_godot_plugin.gd`: creates the TrenchDot integration node, the *TrenchDot: Export Game Config* tool menu entry and the `.bbmodel` import plugin.
* `src/core/parser.gd` and `src/core/entity_assembler.gd`: merge C# entity definitions and apply their properties.
* `src/util/func_godot_util.gd` (`build_texture_map`) and `src/core/geometry_generator.gd`: blend texture keys build blend materials and force a color array.
* `src/map/func_godot_map.gd`: builds scatter sets after terrains.

The I/O runtime also gained `@group`, node path and `!player` targets, PascalCase fallbacks for C# methods and signals,
JSON array arguments with placeholders and type coercion, and the `TrenchDotIO.events()` bus.

## Tests

`res://tests/run_tests.gd` in the TrenchDot demo project builds the maps in `tests/maps` headless and checks geometry,
transforms, prefabs, omitted layers, I/O chains, displacements, meshes, terrains, model props, the Blockbench importer, the worldspawn environment and the
game config export, the gameplay entities, spawners, scatter sets, blend materials and C# definitions, and finishes with a
playthrough of the lighthouse showcase map (gate relay and `trigger_call`). `res://tests/build_showcase.gd` builds the three
showcase maps into scenes.

```sh
godot --headless --path godot --import
godot --headless --path godot --script res://tests/run_tests.gd
```
