# Organizing a map

## Layers

The Outliner shows the map as a tree of layers, groups and objects. New objects go into the current layer, click a
layer to make it current. The eye hides, the lock locks.

Mark a layer *Omit From Export* for reference geometry that should never reach Godot. Hiding alone is not enough,
Godot still builds hidden objects.

## Groups

| Input | Result |
| --- | --- |
| Ctrl+G, Ctrl+Shift+G | Group, ungroup |
| Click | Select the object inside the group |
| Double click | Select the whole group |

## Linked copies

**Duplicate Linked** (Ctrl+Shift+D) makes a copy that stays in sync with its original, so fixing one window frame
fixes all of them. *Edit > Groups > Unlink Groups* breaks the link when one copy has to differ.

## Prefabs

For pieces you reuse across maps, *Edit > Prefabs > Create from Selection...* saves the selection as its own `.gtm` and
replaces it with an instance. Every instance updates when the prefab file changes.

> **Note:** Save the map once before creating a prefab. Prefab paths are stored relative to the map.

Placing a prefab twice would give two entities the same targetname. Set the instance's **fixup** (for example `p1`) and
every name inside it gets that prefix, `door` becomes `p1-door`, so each copy's wiring stays separate.

## The Issues panel

Issues checks the map as you work: brushes with no volume, faces with no material, outputs aimed at targetnames nothing
has, classes with no definition and more. Click an issue to select and frame the object. Most have a **Fix** button,
and *Fix all* sits in the header.

It checks that a face has a material name, not that the material exists in your Godot project.

## History

The History panel lists every edit. Click an older entry to undo back to it, including that entry. Click a dimmed entry
above the marker to redo up to it.
