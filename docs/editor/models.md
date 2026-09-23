# Models

The Models panel lists the models under `res://models` and the installed nature models under
`res://godottrench/nature`. It reads glTF, OBJ, Blockbench, STL, MD2 and MD3 files.

## Placing a model

A model can go into the map in two ways.

| Placed as | How | Stored in the map | Use it for |
| --- | --- | --- | --- |
| Editable mesh | Drag from the Models panel, or its **Import…** button for a file anywhere on disk | A copy of the geometry | Shapes you want to change |
| `prop_model` entity | *File > Import > Model Prop* | Only a `res://` path, Godot loads the file | Props you keep as they are |

A prop has to point at a file Godot can load. Picking one outside the project offers to copy it into `res://models`
first.

## Finding models

**Sort** groups by folder by default and also sorts by name, source or most recent. The two menus next to the search
field narrow the list to one folder or one source. Right click a thumbnail to place it at the cursor, copy its path or
open its pack's page.

## Pack credits

A `pack.json` in a model's folder, or the nearest folder above it, says which pack the model came from.

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

`short` is the coloured slug on each thumbnail, `name` is used when it is missing. `credits` overrides the author,
license or url of single models, matched by file name without extension. Hovering a thumbnail shows the full credit,
and sorting by source shows each pack's license. Folders without a `pack.json` are listed under their top folder's
name.
