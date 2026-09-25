# Organizing a map

## Layers

The Outliner shows the map as a tree of layers, groups and objects. Layers let you sort a map into parts, such as one
building per layer, and hide or lock a part while you work on the rest. New objects go into the current layer, and
clicking a layer makes it current. The eye hides an object or layer, the lock stops it from being selected or edited.

Brush and mesh rows show their size and lowest corner, so the walls a Hollow or a Subtract leaves behind can be told
apart. Hold the pointer over a row to see its materials and bounds, and to outline the object in every view.

Ctrl+click a row to add it to the selection or take it out, and Shift+click to select every row between the last
clicked one and this one. Ctrl+Shift+click adds that range to what is already selected. The Outliner is often the
easier place to pick objects that hide behind others in the views.

For reference geometry that should never reach Godot, right click its layer and pick *Toggle Omit From Export*. Hiding
is not enough, Godot still builds hidden objects.

## Groups

A group keeps objects together so they select and move as one, like the brushes of a table.

| Input | Result |
| --- | --- |
| Ctrl+G, Ctrl+Shift+G | Group the selection, ungroup it |
| Click | Select the object inside the group |
| Double click | Select the whole group |

**Duplicate Linked** (Ctrl+Shift+D) makes a copy that stays in sync with its original, so fixing one window frame fixes
all of them. A click on a linked group selects the whole group, since its copies are edited as one. *Edit > Groups >
Unlink Groups* breaks the link when one copy has to differ.

## Prefabs

For pieces you reuse across maps, *Edit > Prefabs > Create from Selection…* saves the selection as its own `.gtm` and
replaces it with an instance. Every instance updates when the prefab file changes.

> **Note:** Save the map once before creating a prefab. Prefab paths are stored relative to the map.

Placing a prefab twice would give two entities the same targetname. Set each instance's **fixup**, for example `p1`,
and every name inside it gets that prefix, so `door` becomes `p1-door` and each copy's wiring stays separate.

## Cordon

A cordon limits the editor to one part of a big map. Select what you want to work on and choose *View > Cordon > Set
Cordon from Selection*, or **Cordon** in the Inspector. Everything that does not touch the cordon's box, drawn in
yellow, is hidden and cannot be selected. *Cordon Enabled* in the same menu switches it off and on again, and *Clear
Cordon* removes it. The cordon is saved with the map.

It only changes what the editor shows, Godot still builds the whole map. To hand one area to another tool, *File >
Export > .map, cordon only* writes just the objects inside it.

## Issues

The Issues panel checks the map as you work and lists problems that would break or look wrong in Godot, such as brushes
with no volume, faces with no material, or outputs aimed at a targetname nothing has. Click an issue to select and
frame the object. Many have a **Fix** button, and *Fix all* sits in the header.

A `coplanar_faces` issue means two brushes have faces on the same plane that overlap, and they will flicker in Godot. Move
one face off the plane or trim the overlap.

A `missing_model` issue means a prop's `model` names a file that does not exist, so the prop builds with no mesh. When a
file of the same name with another extension exists, `chair.gltf` for `chair.glb`, the message names it.

## History

The History panel lists every edit. Click an older entry to undo back to it, that entry included. Click a dimmed entry
above the marker to redo up to it.

{% mcp %}

## MCP

An agent or script driving the editor over the [MCP server](../mcp.md) can run the same checks as the Issues panel
with the `validate_map` tool.

{% endmcp %}
