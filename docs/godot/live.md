# Saving, live mode and hot reload

## Rebuild on save

Every save in GodotTrench rebuilds the scene tab Godot shows, when that scene has a `FuncGodotMap` for the saved map
with *Auto Rebuild On Save* on. A scene in another tab that uses the map rebuilds when you switch to its tab. Closing
and reopening a tab or reloading a scene does not break this. The status bar says what Godot did:

| The status bar says | Do this |
| --- | --- |
| Godot rebuilt 1 map node(s) | Nothing. Save the scene in Godot to keep the result |
| The scene is not the current scene tab in Godot | Switch to that tab, it rebuilds then |
| The current scene tab has no `FuncGodotMap` for the map | Open the scene that builds the map, or turn on *Auto Rebuild On Save* on its map node |
| Godot has no scene open | Open the scene with the map in Godot, and if it is open, read on |
| Godot is not running with the GodotTrench addon | Open the project in the Godot editor with the addon enabled |

Only one Godot editor can hold the live link port. A second editor with the same project warns in its Output panel
that the live link could not listen, and GodotTrench keeps talking to the first one, which may have no scene open.
Close the editor you do not use and the other takes the link over within a few seconds. *Godot > Open Project in
Godot Editor* shows an editor that already has the project instead of starting a second one.

## Live mode

Live mode pushes edits into the scene open in the Godot editor before you save, so you can judge lighting and scale in
the real renderer. Turn it on with the link button at the right end of the toolbar or *Godot > Live Mode*. It needs the
Godot live link, which is on by default in *File > Preferences*.

![Edits in GodotTrench showing up in the Godot editor before saving](../screenshots/GodotTrench-demo-live-view.gif)

The map has to be saved inside the project, and the scene open in Godot needs a `FuncGodotMap` using it. The link
button's tooltip says what it is still waiting for.

Drags move nodes while you drag, and most edits rebuild only what they touched, so the preview keeps up. Toggling
*Omit From Export* on a layer and editing prefab instances rebuild the whole map instead, once you pause.

The preview is close to a full build but not always identical. Saving always does a full build, and quitting without
saving sends Godot back to the saved file, so nothing you tried in live mode sticks unless you save it.

## Hot reload in a running game

To see saves in a game that is already running, add this script as an autoload under *Project Settings > Globals >
Autoload*:

```
res://addons/func_godot/src/godottrench/runtime/godottrench_hot_reload.gd
```

On every save it rebuilds the maps that use the saved file and emits `map_reloaded(map)`. Use that signal to put the
player back at a spawn point or reset game state.

It listens on port 7843, which must match *Running game hot reload* in the editor's preferences. It only runs in games
launched from the Godot editor unless you set `enabled_in_exported_builds`, and the editor only sends it saves while
the live link is on.
