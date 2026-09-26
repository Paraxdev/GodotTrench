# Models

Models are 3D files made in another program, such as Blender or Blockbench. The Models panel lists the models under
`res://models` and the installed nature models under `res://godottrench/nature`. It reads glTF, OBJ, Blockbench, STL,
MD2 and MD3 files. A new project has neither, *Godot > Add Content to Project…* adds the nature models, and with the
demo the showcase props in `res://models/polyhaven`.

## Placing a model

A model can go into the map in two ways. Pick by whether you want to change its shape in the editor.

| Placed as | How | Stored in the map | Use it for |
| --- | --- | --- | --- |
| Editable mesh | Drag from the Models panel, or its **Import…** button for a file anywhere on disk | A copy of the geometry, which no longer follows the file | Shapes you want to edit with the [mesh tools](meshes.md) |
| `prop_model` entity | *File > Import > Model Prop* | Only a `res://` path, Godot loads the file | Props you keep as they are. Every copy follows the file when you change it |

An editable mesh keeps the model's look. Its textures are copied into `models/<name in the Models panel>` in the
project's texture folder, which a new project gets the first time a model is placed, with a Godot material next to each
one where needed, so cut out leaves, pixel art and glowing parts draw in Godot like they do here. The Inspector lists
the materials a mesh draws with. A material clicked in the Materials panel while the mesh is still selected replaces
them, *Edit > Undo* brings them back.

A prop has to point at a file Godot can load. Picking one outside the project offers to copy it into `res://models`
first.

Right click a thumbnail to place it at the cursor, copy its path, open its pack's page, or *Show in File Manager* to
find the file on disk.

## Pack credits

Free model packs usually ask for credit. A `pack.json` in a model's folder, or the nearest folder above it, says which
pack the model came from, so the panel can show who made it and under which license.

```json
{
  "name": "Nature Kit",
  "short": "Nature",
  "author": "Some Artist",
  "license": "CC0",
  "url": "https://example.com/nature-kit",
  "credits": [{ "asset": "pine", "author": "Other Artist" }]
}
```

`short` is the label on each thumbnail and falls back to `name`. `credits` overrides the author, license or url of
single models, matched by file name without extension. Hover a thumbnail for the full credit. Folders without a
`pack.json` are listed under their top folder's name.
