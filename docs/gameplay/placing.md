# Placing and wiring

## Placing entities

Point entities are things that sit at one spot, like lights, spawn points and sounds. Drag one from the **Entities**
panel into a view, or double click its card to place it at the cursor. Ctrl or Shift click selects several cards, and
dragging them drops them all in a row.

Doors, buttons and triggers are brush entities: they take their shape from brushes you built. Select the brushes, then
pick a class from *Create Brush Entity* in the view's right click menu or from *Brush > Brush Entity*. The common ones
have quicker routes:

| Tool or menu | What it makes |
| --- | --- |
| **Volume** tool (Shift+E) | Drags out any trigger volume in one go, without building a brush first |
| **Path** tool (Shift+P) | Click by click, a chain of `path_corner` entities that trains and walkers follow |
| *Gameplay > Doors and Movers* | A hinged or sliding door with its trigger, or a lift, from the selected brushes |
| *Gameplay > Triggers* | A trigger volume sized around the selection |

A selected door, platform, button, push trigger, spawner or explosion shows handles in the view. Drag them to set its
hinge, how far it travels, or its radius, instead of typing numbers.

## Wiring

Outputs are set in the **Inspector**, where each connection is one row with the fields explained in
[How entity I/O works](io.md). To connect two entities quickly, select both and use
*Gameplay > Logic > Link Two Selected Entities...*, which opens a dialog to pick the output and input.

> **Warning:** The Link dialog does not take the entity you clicked first as the source. Check the direction and press
> ⇄ to swap it if needed.

## Checking wiring without Godot

Open the **Logic** panel (*View > Panels*), pick a named entity and one of its outputs, and press **Fire**. It lists
every `source.output → target.input` step that would follow, with delays, so you can follow a long chain without
building and running the game. Two colours point out steps worth a second look:

| Colour | Meaning |
| --- | --- |
| Red | The target does not exist, usually a typo in a targetname |
| Blue | A runtime target like `!player` or `@enemies`, which only exists in game, so it is not followed further |

## Code for any entity

The **Reference** panel (*View > Panels*) shows GDScript and C# snippets for any entity class, useful when game code
has to talk to a map entity, for example to open a door or listen for a button. It also offers a skeleton for writing
your own class and its FGD resource. **Create in project** writes the file into `res://entities/`.
