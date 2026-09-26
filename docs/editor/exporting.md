# Exporting to Blender and other 3D tools

*File > Export > glTF Binary (.glb) for Blender and other 3D tools* saves what the map looks like as one 3D model file:
brushes, meshes, terrain and models, with their textures inside. Use it to render a level in Blender, set up a trailer
shot, or hand a blockout to an artist who builds the final meshes over it.

> **Warning:** Only geometry and materials are exported. Entity logic, I/O wiring, triggers, scripts and gameplay
> behaviour are left out, and the file cannot be opened in GodotTrench as a map again. The `.gtm` stays the map, keep
> editing that.

## Steps

1. *File > Export > glTF Binary (.glb) for Blender and other 3D tools*, or type "glTF" in the command palette
   (Ctrl+Shift+P or F1).
2. Pick what goes into the file, see the options below, then click **Export…** and choose where to save. The dialog
   can switch to OBJ too.
3. In Blender, *File > Import > glTF 2.0 (.glb/.gltf)* and pick the file. The defaults are right, the importer turns
   the model Z up on its own.

The status bar says how many objects, materials and triangles went out, and names any material the project has no
image for. Those come out untextured.

| Option | Does |
| --- | --- |
| Whole map, Selection only | Everything, or only the selected objects, still inside their layers and groups |
| Hidden layers and objects | Also exports what is hidden in the views. Off, you get what you see |
| Prop models | The models that props and other point entities show, at their place, rotation and scale |
| Scatter instances | Every tree, rock and tuft of the [scatter sets](scatter.md) as its own object. Off by default, the number next to it says how many objects it adds, and thousands of them make a big file that is slow to open |
| Point entities as empty markers | An empty object named like the entity where each point entity without a model stands, handy for placing lights or cameras in Blender |
| Merge unnamed brushes | Brushes you have not renamed join into one object per layer, group or entity, which keeps Blender's outliner short. Named brushes stay separate |

## What is exported

| In the map | In the file |
| --- | --- |
| Layers and groups | Empty parent objects with the same names, so the Outliner layout carries over |
| Brushes | One object each, named like in the Outliner, `brush12` or the name you gave it |
| Brush entities such as doors and lifts | A parent object named after the entity holding its brushes, where they stand in the editor |
| Meshes, decals | One object each. Decals get blended, double sided materials |
| Terrain | One object. Each triangle takes the texture of its strongest painted layer, so painted blends look stepped |
| Prefab instances | The prefab's contents moved into place, under an object named after the instance |
| Materials | One per material with its albedo texture, plus the normal, roughness, metallic, ambient occlusion and emission maps it has. Cut out and see-through materials keep their alpha, pixel art stays sharp. Built-in dev colours come out as their grid textures |

The file leaves out what the Godot build does not draw: faces with tool textures (clip, skip, origin, sky, trigger,
nodraw, hint), faces hidden against another brush, trigger and other volume entities, and layers set to *Omit From
Export*. Lights, sounds, spawn points, the sky and fog are not exported either.

## Units and axes

Map units are divided by the project's units per meter, 32 unless the map settings say otherwise, so one meter in
Godot is one meter in Blender. The file is Y up like Godot, which is what glTF expects. Each brush, mesh and terrain has
its origin at the center of its bounds, and models keep the origin of the model file.

## When to use OBJ

*File > Export > Wavefront OBJ (.obj)* is for tools that cannot read glTF. It writes three things side by side, for
example `level.obj`, `level.mtl` with the materials, and a `level_textures` folder with copies of the textures. Keep
them together when you move them. In Blender, *File > Import > Wavefront (.obj)*.

OBJ has no parent objects and no way to share one mesh between copies, so every object is written flat in world space
and each scatter instance is a full copy. Materials keep the albedo, normal, roughness and emission textures. Prefer
glTF whenever the other tool reads it.

## Command line

`godottrench --export-glb <map> <out.glb>` and `--export-obj <map> <out.obj>` export with the default options without
opening a window. The map's Godot project is found from its folder. See [Command line](../command-line.md).

{% mcp %}

## MCP

`map_file {op: export_glb, path}` writes a `.glb` and `map_file {op: export_obj, path}` an `.obj` with its `.mtl` and
texture folder, from the open map as it is, saved or not. The options are flags: `selection_only`, `hidden`, `scatter`,
`models` (true unless set), `markers` and `merge_brushes`. The result lists `objects`, `meshes`, `materials`, `images`,
`triangles`, `bytes` and `missing_textures`. `run_action` refuses `export_glb` and `export_obj`, since the menu entries
open a file dialog.

{% endmcp %}
