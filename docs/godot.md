# Working with Godot

GodotTrench works on its own. A Godot editor with the GodotTrench addon (the FuncGodot fork in `godot/addons/func_godot`)
adds rebuilds on save, live mode and a one click way to open the project.

## Finding Godot

At startup GodotTrench looks for the Godot executable in this order:

1. *Godot executable* in File > Preferences
2. the `GODOT` environment variable
3. `PATH`: `godot`, `godot4` and downloaded builds like `Godot_v4.7.2-stable_win64.exe` (console builds are skipped, mono
   builds are preferred when the project has a `.csproj`)
4. common install folders: WinGet links, Scoop shims, Steam and `Programs/Godot`

When nothing is found the status bar says so and *Run Project* and *Open Project in Godot Editor* are disabled. Everything
else keeps working, including the live link to a Godot editor that was started some other way. Preferences shows the
executable in use.

## The Godot button

The Godot robot at the right end of the toolbar:

| State | Robot | Click |
|---|---|---|
| no project open | grey | nothing |
| Godot not found and no editor connected | grey, the tooltip explains how to set the path | nothing |
| Godot found | normal | opens the project in the Godot editor |
| a Godot editor has this project open | blue | brings that editor to the front |

The link button next to it switches live mode, a spinner shows while Godot is building.

## Saving and building

* **Save** writes the map and Godot does a full rebuild of every `FuncGodotMap` in the edited scene that uses it, as before.
  A running game with the `GodotTrenchHotReload` autoload reloads it too.
* **Godot > Build in Godot** does a full build of the map as it is in GodotTrench, saved or not, without writing the file.
* **Build Map** on the `FuncGodotMap` node in Godot does a full build of the saved file.

Builds only replace the children of the `FuncGodotMap` node. Nodes you add next to it in the scene are never touched.

Full builds read `.gtm` files in one go, convert brushes and terrain chunks on the WorkerThreadPool and create meshes and
shapes on the main thread. The project setting `godottrench/threaded_build` turns the threaded steps off.
`godot --headless --path godot --script res://tests/bench_build.gd -- runs=5 threaded=1` times every build step of the
showcase maps.

## Live mode

Turn it on with the link button, Godot > Live Mode or *Godot live mode* in Preferences (off by default). While the Godot
editor shows a scene with a `FuncGodotMap` using the current map, edits appear in Godot right away, before you save:

* moving point entities, brush entities, terrains and scatter sets moves their node while you drag
* changing an entity, terrain or scatter set rebuilds just that node
* changing loose brushes splits the worldspawn into chunks of `godottrench/live_chunk_size` meters (16 by default) the
  first time, then only rebuilds the chunks a change touches, a dragged brush gets a node of its own until you let go
* layer omission, prefab instances and anything inside them rebuild the whole map once editing pauses

The preview is close to a full build but not always identical: faces covered by coplanar faces of brush entities or of
chunks that were not rebuilt may show until the next full build, and rebuilt nodes move to the end of their parent in
Godot's Scene dock (putting them back in place trips a Scene dock bug in Godot 4.7). Saving always does a full build. When you throw away
unsaved changes (Discard on quit, or New and Open replacing a modified map) Godot goes back to the saved file.

Edits only go out when Godot has taken the previous batch, and a drag sends small `translate` messages instead of the
geometry, so a slow Godot editor never piles up work.

## Live link protocol

Newline delimited JSON over TCP on `127.0.0.1:7842` (`godottrench/live_link_port`). Every request carries a `seq` that the
reply echoes. The editor keeps one connection open and sends a `status` heartbeat every second.

| Event | Fields | Reply |
|---|---|---|
| `status` | | `project`, `godot`, `pid`, `maps: [{path, epoch}]` for the edited scene |
| `map_saved` | `path` | `rebuilt` |
| `build` | `path`, `text` | `rebuilt` |
| `focus` | | |
| `live_begin`, `live_resync` | `path`, `text` | `epoch` |
| `live_delta` | `path`, `ops` | `epoch`, or `ok: false, resync: true` when the ops do not fit |
| `live_end` | `path`, `revert` | |
| `inspect` | `path`, `ids` | generated node name, class and position per map node id |
| `export_game_config` | | |

`ops` are `set {id, parent, index, node}` (the node as stored in the `.gtm` file, without children), `remove {id}` (the top
most removed node), `translate {ids, offset}` and `properties {properties}`. The build `epoch` of a map goes up whenever it
is built outside a live session or its `FuncGodotMap` nodes change, and the editor then starts over with `live_begin`.
Generated nodes carry their map node id in the `_gt_id` metadata (`_gt_group` for layers and groups).
