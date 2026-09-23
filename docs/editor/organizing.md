# Organizing a map

## Layers

The Outliner shows the map as a tree of layers, groups and objects. New objects go into the current layer, and clicking
a layer makes it current. The eye hides an object or layer, the lock locks it.

For reference geometry that should never reach Godot, right click its layer and pick *Toggle Omit From Export*. Hiding
is not enough, Godot still builds hidden objects.

## Groups

| Input | Result |
| --- | --- |
| Ctrl+G, Ctrl+Shift+G | Group, ungroup |
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

## Issues

The Issues panel checks the map as you work, for brushes with no volume, faces with no material, outputs aimed at
targetnames nothing has and more. Click an issue to select and frame the object. Many have a **Fix** button, and
*Fix all* sits in the header. `missing_material` lists faces whose material the project's texture folder does not
have. MCP's `validate_map` runs the same checks.

## History

The History panel lists every edit. Click an older entry to undo back to it, that entry included. Click a dimmed entry
above the marker to redo up to it.
