# Placing and wiring

## Placing entities

Drag a point entity from the **Entities** panel into a view, or double click its card to place it at the cursor. Ctrl
or Shift click selects several cards, and dragging them drops them all in a row.

Doors, buttons and triggers are brush entities. Select the brushes, then pick a class from *Create Brush Entity* in the
view's right click menu or from *Brush > Brush Entity*. These shortcuts build the common ones for you:

| Tool or menu | Makes |
| --- | --- |
| **Volume** tool (Shift+E) | Drags out any trigger volume in one go |
| **Path** tool (Shift+P) | Clicks down a chain of `path_corner` entities for trains and walkers |
| *Gameplay > Doors and Movers* | A hinged or sliding door with its trigger, or a lift, from the selected brushes |
| *Gameplay > Triggers* | A trigger volume sized around the selection |

A selected door, platform, button, push trigger, spawner or explosion shows draggable handles for its hinge, travel or
radius.

## Wiring

Outputs are set in the **Inspector**. To connect two entities, select both and use
*Gameplay > Logic > Link Two Selected Entities...*.

> **Warning:** The Link dialog does not take the entity you clicked first as the source. Check the direction and press
> ⇄ to swap it if needed.

## Checking wiring without Godot

Open the **Logic** panel (*View > Panels*), pick a named entity and one of its outputs, and press **Fire**. It lists
every `source.output → target.input` step that would follow, with delays.

| Colour | Meaning |
| --- | --- |
| Red | The target does not exist |
| Blue | A runtime target like `!player` or `@enemies`, not followed further |

## Code for any entity

The **Reference** panel (*View > Panels*) shows GDScript and C# snippets for using an entity class from code, a class
skeleton for writing your own and its FGD resource. **Create in project** writes the file into `res://entities/`.
