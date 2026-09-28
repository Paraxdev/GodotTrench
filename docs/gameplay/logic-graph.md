# The Logic graph

The **Logic** panel shows the map's entity I/O as a graph. Every wired entity and every logic entity is a node, and
every output connection is a wire from an output pin on the right of one node to an input pin on the left of another.
The graph is a view of the same outputs the Inspector edits, so a change in either shows in the other at once, and the
map, the Godot build and the game work exactly as they did without it.

Open it from *View > Panels > Logic*. It needs room, so drag its tab into the bottom dock next to Materials, or into a
view's place.

## Reading the graph

| What you see | What it means |
| --- | --- |
| Node title | The entity's targetname, else its name from *Rename*, else its classname. The classname is below it |
| Pins | Inputs on the left, outputs on the right, from the [entity definition](entities/README.md) plus any name the map already uses on that entity. A filled pin has a wire |
| Yellow pin name | The name is not in the entity's definition, the Issues panel warns about it too |
| Wire label | The delay, `once` or how many times it may fire, and the parameter, when they are set |
| Blue node and wire | A [target](parameters.md#targets) only the running game resolves, such as `!player`, `@enemies`, a node path or a wildcard like `door_*` |
| Red node, dashed red wire | A targetname no entity has, usually a typo |

Only wired entities and logic entities have a node. Select any other entity, in a view or the Outliner, and it shows
up too, ready to be wired.

## Wiring

1. Drag from an output pin to an input pin. The connection is added to the source entity's outputs. A target without
   a targetname gets a free one, like `door_2`, in the same step.
2. Drop the wire on a node's title instead, to pick from all its inputs or type any name.
3. Drop it on empty space to add a logic entity there, already wired to the pin you dragged from.

Right click empty space, or use **Add**, to add a logic entity without a wire. The list starts with the logic
entities, relays, timers, counters and the like, then other entities that have inputs or outputs. New entities are
point entities, and the Godot build makes a node for each, so they need a place in the map. They go on a layer called
*Logic*, next to the entity they are wired to, or in front of the 3D view's camera.

Click a wire to edit its output, target, input, parameter, delay and times in the box at the top right. Its **Delete**
button, or the Delete key while the pointer is over the graph, removes it. Every change is one step in *History*, so
undo works as everywhere else.

> **Note:** Without the Gameplay entities pack there are no logic entities to add. The Add menu then offers to
> install it.

## Moving around

| To | Do |
| --- | --- |
| Zoom | Turn the wheel |
| Pan | Drag with the middle or right button |
| Select | Click a node, Shift or Ctrl click adds, drag on empty space to box select. The entities are selected in the views and the Outliner too, and the other way round |
| Frame in the views | Double click a node |
| Move | Drag nodes. Their positions are saved in the map |
| Tidy up | **Arrange** lays out the selected nodes by signal flow. With fewer than two selected, every node goes back to the automatic layout |
| See everything | **Fit** |

Nodes nobody moved follow the automatic layout, left to right the way signals flow. Once you move one, the others keep
their places, and a new node appears next to what it is wired to. Blue and red marker nodes are not entities, so they
cannot be moved. They sit next to the node that targets them.

## Simulate

Select one entity, press **Simulate** and pick one of its outputs, or right click a node. The graph numbers every wire
the output would set off in yellow, and a list at the bottom left shows each step with the time it arrives after the
delays. It follows the runtime's rules: targets are found by targetname, and relays, timers, counters, doors and the
other common entities pass the chain on. Runtime targets like `!player` are shown but not followed. Click a step to
select its wire.

Simulate works on paper, without Godot. What really happens in game is covered in [Debugging wiring](debugging.md).
