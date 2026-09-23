# Meshes and models

Brushes are great for architecture and poor for anything organic. Meshes can be concave, open and non-planar.

## Editing meshes

Select brushes and press Tab (or *Mesh > Edit Mesh*) to convert them and start editing. Picking the Mesh tool from the
toolbar does not convert anything. The controls follow Blender:

| Key | Action |
| --- | --- |
| 1, 2, 3 | Vertex, edge, face mode |
| G, R, S | Move, rotate, scale |
| E | Extrude |
| I | Inset |
| Ctrl+R | Loop cut |
| K | Knife |
| Double click an element | Show the move, rotate and scale gizmo on it |
| Esc | Hide the gizmo, the selection stays |

The gizmo is the clean way to slide a vertex along one axis, and it works in the 2D views too.

Ctrl+Shift+E converts brushes to a mesh, Ctrl+Shift+J joins meshes.

## Importing models

The Models panel lists everything under `res://models` and the installed nature models, with thumbnails. Drag a model
into a view to place it, or use **Import…** for a file outside `res://`. glTF, OBJ, Blockbench and a few other formats
work.

There are two ways a model can end up in a map:

| Placed as | What is stored | Use it for |
| --- | --- | --- |
| Editable mesh (drag from Models) | The geometry itself, a copy | Shapes you want to change |
| `prop_model` entity | Only a `res://` path, Godot loads the file at runtime | Props you keep as they are |

Placing a prop from outside the project offers to copy the file into `res://models` first.

## Model packs and credits

A `pack.json` in a model's folder, or a folder above it, names the pack it came from: a short name, author, license and
url, and optionally the author of single models. The short name shows as a coloured slug on each thumbnail, hovering
shows the full credit, and sorting by source shows each pack's license. Folders without one are listed under their top
folder's name.

**Sort** groups by folder by default and also offers name, source and most recent. The two menus next to the search
field narrow the list to one folder or one source.
