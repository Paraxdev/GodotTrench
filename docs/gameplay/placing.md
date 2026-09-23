# Placing and wiring

## Point entities

Drag one from the Entities panel into a view, or double click it to place it at the cursor. Ctrl or Shift click selects
several cards, and dragging any of them drops them all in a row.

## Brush entities

Doors, buttons and triggers are brushes you convert. Select the brushes, then pick a class from *Create Brush Entity*
in the view's right click menu, or from *Brush > Brush Entity*.

| Tool or menu | Makes |
| --- | --- |
| **Volume** (Shift+E) | Drags out a trigger, spawn area, hurt, teleport, push or area volume in one go |
| **Path** (Shift+P) | Clicks down a chain of `path_corner` entities for trains and walkers |
| **Gameplay** menu | Hinged or sliding doors with their trigger, lifts, platforms, buttons, triggers sized around the selection |

Selected entities show draggable handles for hinges, travel offsets, radii, spot cones and target points.

## Wiring

Outputs are set in the Inspector. To connect two entities, select both and use
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

The **Reference** panel (*View > Panels*) shows how to use any entity class from code: GDScript and C# snippets, a
class skeleton for writing your own and the FGD resource. **Create in project** writes the skeleton into
`res://entities`.
