# Live mode and hot reload

GodotTrench works on its own. A Godot editor running the addon adds rebuilds on save, live mode and one click project
launching.

## Live mode

Live mode pushes edits into the scene open in the Godot editor before you save, so you can judge lighting and scale in
the real renderer. Turn it on with the link button in the toolbar, *Godot > Live Mode* or Preferences. It is off by
default and needs the map to have been saved once.

| You change | Godot does |
| --- | --- |
| Drag an entity, terrain or scatter set | Moves its node while you drag |
| Edit an entity, terrain or scatter set | Rebuilds just that node |
| Edit loose brushes | Splits worldspawn into chunks of `godottrench/live_chunk_size` meters (16) once, then rebuilds only touched chunks |
| Layer omission, prefab instances | Rebuilds the whole map once editing pauses |

The preview is close to a full build but not always identical. Saving always does a full build. Discarding unsaved
changes on quit sends Godot back to the saved file.

## Hot reload in a running game

To see saves in a game that is already running, add this autoload under *Project Settings > Globals > Autoload*:

```
res://addons/func_godot/src/godottrench/runtime/godottrench_hot_reload.gd
```

It listens on port 7843, rebuilds matching maps on every save and then emits `map_reloaded(map)`. Use that signal to put
the player back at a spawn point or reset game state. It only runs in games launched from the Godot editor, unless you
set `enabled_in_exported_builds`.

## Finding the Godot executable

The editor only needs Godot for *Run Project* (F5) and *Open Project in Godot Editor*. Saving and live mode go through
the addon. It looks in this order:

1. *Godot executable* in *File > Preferences*
2. The `GODOT` environment variable
3. `PATH`: `godot`, `godot4` and downloaded builds such as `Godot_v4.7.2-stable_win64.exe`. Mono builds are preferred
   when the project has a `.csproj`
4. Common install folders: WinGet, Scoop, Steam and `Programs/Godot`

## The Godot button

The Godot robot at the right end of the toolbar shows the connection state.

| Robot | Meaning | Click |
| --- | --- | --- |
| Grey | No project open, or Godot not found (the tooltip explains) | Nothing |
| Normal | Godot found | Opens the project in the Godot editor |
| Blue | A Godot editor has this project open | Brings it to the front |

A spinner next to it shows while Godot is building.
