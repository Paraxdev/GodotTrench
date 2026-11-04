# Live link protocol

The live link is how TrenchDot and the Godot editor talk. The addon runs a small server inside the Godot editor, and
TrenchDot connects to it to trigger rebuilds when you save, stream [live mode](live.md) edits and show whether Godot
is running.

You will rarely need this page, the link just works once the addon is enabled. It is here for when you write your own
tool that drives Godot the same way, or want to find out why the connection fails. If a save does not rebuild, start
with [Rebuild on save](live.md#rebuild-on-save) instead, the status bar usually says why.

## Transport

Newline delimited JSON over TCP on `127.0.0.1:7842`. Each message has an `event` and an optional `seq`, and gets exactly
one reply line that echoes the `seq` and has `ok`. Failures reply `ok: false` with an `error`. The editor keeps one
connection open and sends a `status` heartbeat every second.

To use another port, change it on both sides: `trenchdot/live_link_port` in Godot's project settings and *Godot live
link* in *File > Preferences* in TrenchDot.

The [hot reload](live.md#hot-reload-in-a-running-game) autoload in a running game listens on `127.0.0.1:7843` and only
answers `map_saved`.

## Events

A client sends these events. The Reply column lists the fields Godot answers with besides `ok` and `seq`, or what it
does.

| Event | Fields | Reply |
| --- | --- | --- |
| `hello` | | `project` and `godot` (version), a quick check that the link answers |
| `status` | | `project`, `godot` (version), `pid`, `scene` (the scene tab Godot shows) and `maps: [{path, epoch}]` for it |
| `map_saved` | `path` | `rebuilt`, the number of map nodes rebuilt from the file, `scene`, `waiting`, and the build's `missing_classes`, `settings` and `warnings` |
| `build` | `path`, `text` | Like `map_saved`, built from `text` instead of the file |
| `focus` | | Brings the Godot window to the front |
| `export_game_config` | | Writes `trenchdot_game.json` |
| `scan` | | Starts a scan of the project files. Once it is done, every live session checks its entity definitions |
| `play` | | Saves the open scenes and plays the main scene, or `ok: false` with an `error` when the project has none |
| `live_begin` | `path`, `text`, `content` | `epoch` and `unresolved` |
| `live_resync` | `path`, `text` | `epoch` and `unresolved` |
| `live_delta` | `path`, `ops` | `epoch` and `unresolved`, or `ok: false, resync: true` when the ops do not fit |
| `live_end` | `path`, `revert` | With `revert`, rebuilds from the saved file |
| `inspect` | `path`, `ids` | `nodes` with name, class, child count and position per map node id, plus `live` and `pending` |
| `capture` | `path`, `camera`, `width`, `height`, `text` | `png` (base64), `width`, `height` and `warnings`, the errors and warnings logged while building |
| `walkable` | `path`, `agent`, `text` | `vertices` (flat x, y, z list), `polygons`, `islands` (polygon indices, largest first), `units_per_meter`, `agent` and `msec` |

`capture` renders the world of the first map node using `path`. `camera` holds `position` and `forward` in map units
and axes and `fov`, the horizontal field of view in degrees. With `text` Godot builds that map first, otherwise it
waits for a live session to catch up. The reply comes after about a dozen frames, and other messages are answered
meanwhile.

`walkable` bakes the navigation mesh of the whole edited scene with `TrenchDotNav` for `agent` (`radius`, `height`,
`max_climb`, `max_slope` in meters and degrees) and returns it in the map's units and axes. With `text` Godot builds
that map first unless it was last built from the same text.

Paths are absolute with forward slashes. Only `FuncGodotMap` nodes in the edited scene, the scene tab Godot shows, with
*Auto Rebuild On Save* on take part. `scene` is its `res://` path, or empty with no scene open.

`waiting` lists the other open scene tabs that use the map. After a `map_saved` they build the file when their tab is
shown, after a `build` they do not. A `map_saved` for a prefab also rebuilds the maps of the edited scene that instance
it, directly or through other prefabs, and counts them in `rebuilt`.

What the builds of a `map_saved` or `build` reported comes in three lists, each only when it is not empty:
`missing_classes` holds the sorted classnames that no definition described, `settings` the paths of the map settings
the builds used, and `warnings` up to 20 one line warnings of the build report. `unresolved` in a live reply lists the
sorted classnames that step built without a definition, also only when there are any.

`scan` suits files written outside Godot, like an installed entity pack. A live session whose scene was built with
other entity definitions than its map settings hold after the scan builds its map again. An addon without `scan` or
`play` answers them as an unknown event.

`text` is the whole map as JSON, with the baked lighting under `lightmap` when the map has a bake, so a live build keeps
it. That adds about 11 bytes per atlas texel, around 11 MB for a 1024 by 1024 atlas, so the editor sends the whole map
again only when a session starts, after a new bake or after a big change.

`content` is the content id in the END chunk of the saved file, so a scene built from an unchanged file is not built
again when a session starts.

When the port is taken, usually by a second Godot editor with the same project, the addon warns once in the Output
panel and tries the port again every three seconds, so it takes over when the other editor closes.

## Live ops

| Op | Fields | Meaning |
| --- | --- | --- |
| `set` | `id`, `parent`, `index`, `node` | Insert or replace a node, in the JSON shape of the `.tdm` format, without children |
| `remove` | `id` | Remove a node with its subtree, only the topmost removed id is sent |
| `translate` | `ids`, `offset` | Move nodes, sent while dragging instead of geometry |
| `properties` | `properties` | Replace the worldspawn properties |

A map's `epoch` goes up whenever it is built outside a live session or the `FuncGodotMap` nodes using it change. When
the next `status` shows a new epoch, start over with `live_begin`.

Generated nodes carry their map node id in the `_td_id` metadata, and layer and group nodes in `_td_group`.

## Testing without a window

The link also works with a headless Godot editor, which suits automated tests. Open a scene with a `FuncGodotMap` in
it, then send `inspect` to port 7842 to see what Godot built:

```sh
godot --headless --editor --path godot res://path/scene.tscn
```

(Note: on a fresh checkout, run `godot --headless --path godot --import` twice first. One pass only registers the
addon's classes.)

A headless Godot draws nothing, so `capture` answers with an error there. Test captures with a windowed editor.

The addon tests do this in
[`godot/tests/editor/live_link_tabs.gd`](https://github.com/Paraxdev/TrenchDot/blob/main/godot/tests/editor/live_link_tabs.gd),
which opens, closes, reloads and switches scene tabs in a headless editor and checks what the link reports.

<div class="td-mcp" markdown>

## MCP

The [MCP server](../mcp.md) lets agents and scripts drive the editor, and some of its tools go through the live link.
The `screenshot` tool uses `capture` when asked for a picture from Godot.

For headless tests that edit the map too, start TrenchDot with its MCP server on the same project as the headless
Godot editor and edit the map through MCP:

```sh
trenchdot --mcp-http --project godot
```

</div>
