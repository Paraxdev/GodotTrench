# Prefab library

The **Prefabs** panel, next to Materials and Models, holds ready made pieces to paste into a map: stairs, doorways,
pillars, small rooms and details. It works like the prefab collections TrenchBroom mappers copy from, only built into
the editor. Pasting a piece copies its brushes and entities into the map, with no link back to the library, so you can
reshape or retexture the copy freely.

| Do this | To |
| --- | --- |
| Click a card | Paste the piece at the pointer, like Ctrl+V |
| Right click a card, *Copy* | Put the piece on the clipboard, to paste into another map or share as text |
| Right click, *Open to edit* | Open one of your project's pieces as a map (built-in pieces cannot be edited) |
| Type in *Search prefabs* or pick a category | Find a piece |

Each card draws its piece from the side that shows the most: from above for floor plans like a stair, from the front
for walls like a doorway. Lighter means nearer.

## Your own pieces

Select the objects, open **Add the selection**, give the piece a name and a category, and click **Add**. The editor
saves them as a `.gtm` in `res://prefab_library/<category>/`, moved so the bottom center of the selection sits on the
origin, and the piece shows up as a card. **Keep position** leaves it where it is instead, for a piece you built
around the map's origin on purpose, like the pieces of a [random fill](random-fill.md) kit. Every `.gtm` in that folder is a piece, so you can also save maps there
yourself, or share a folder of pieces between projects.

The built-in pieces use the `dev/` materials every project has. Retexture a pasted piece with the Materials panel, or
build your own pieces in your project's materials.

To cover a whole area with pieces picked at random, use [Random fill](random-fill.md).

> **Tip:** A piece is plain map content once pasted. For something that should stay in sync everywhere it is used, like
> a window frame repeated across a building, make a [prefab instance](organizing.md#prefabs) instead.

## Pasting from TrenchBroom

GodotTrench also pastes what TrenchBroom puts on the clipboard, so brushes copied in TrenchBroom, or from a TrenchBroom
prefab collection on the web, paste straight into a map with Ctrl+V. Groups come across as groups, entities as
entities, and the axes are turned the same way as when [importing a .map](importing.md). Texture names are kept, other
games' tool textures become the project's.

{% mcp %}
## MCP

The `prefab_library` tool lists pieces (`op: list`), pastes one by its `category/name` (`op: paste`, with `origin` or
`offset`), returns its clipboard text (`op: copy`), and adds objects to the project's library (`op: add`, the selection
unless `ids` are given). `run_action` with `paste` accepts TrenchBroom clipboard text as well as GodotTrench's.
{% endmcp %}
