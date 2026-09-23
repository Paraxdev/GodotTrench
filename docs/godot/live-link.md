# Live link protocol

The addon runs a server in the Godot editor that GodotTrench uses for rebuilds, live mode and status. You only need
this page to write your own client or debug the connection.

## Transport

Newline delimited JSON over TCP on `127.0.0.1:7842` (`godottrench/live_link_port`). Each message has an `event` and an
optional `seq`, and gets exactly one reply line that echoes the `seq` and has `ok`. Failures reply `ok: false` with an
`error`. The editor keeps one connection open and sends a `status` heartbeat every second.

The [hot reload](live.md#hot-reload-in-a-running-game) autoload in a running game listens on `127.0.0.1:7843` and only
answers `map_saved`.

## Events

| Event | Fields | Reply |
| --- | --- | --- |
| `status` | | `project`, `godot` (version), `pid`, `maps: [{path, epoch}]` for the edited scene |
| `map_saved` | `path` | `rebuilt`, the number of map nodes rebuilt from the file |
| `build` | `path`, `text` | `rebuilt`, built from `text` instead of the file |
| `focus` | | Brings the Godot window to the front |
| `export_game_config` | | Writes `godottrench_game.json` |
| `live_begin` | `path`, `text`, `content` | `epoch` |
| `live_resync` | `path`, `text` | `epoch` |
| `live_delta` | `path`, `ops` | `epoch`, or `ok: false, resync: true` when the ops do not fit |
| `live_end` | `path`, `revert` | With `revert`, rebuilds from the saved file |
| `inspect` | `path`, `ids` | `nodes` with name, class, child count and position per map node id, plus `live` and `pending` |

Paths are absolute with forward slashes, and only `FuncGodotMap` nodes in the edited scene with *Auto Rebuild On Save*
on take part. `text` is the whole map as JSON. `content` is the content id in the END chunk of the saved file, so a
scene built from an unchanged file is not built again when a session starts.

## Live ops

| Op | Fields | Meaning |
| --- | --- | --- |
| `set` | `id`, `parent`, `index`, `node` | Insert or replace a node, in the JSON shape of the `.gtm` format, without children |
| `remove` | `id` | Remove a node with its subtree, only the topmost removed id is sent |
| `translate` | `ids`, `offset` | Move nodes, sent while dragging instead of geometry |
| `properties` | `properties` | Replace the worldspawn properties |

A map's `epoch` goes up whenever it is built outside a live session or the `FuncGodotMap` nodes using it change. When
the next `status` shows a new epoch, start over with `live_begin`.

Generated nodes carry their map node id in the `_gt_id` metadata, and layer and group nodes in `_gt_group`.

## Testing without a window

Open a scene with a `FuncGodotMap` in a headless Godot editor and start GodotTrench with its MCP server on the same
project. Edit the map through MCP, then send `inspect` to port 7842 to see what Godot built:

```sh
godot --headless --editor --path godot res://path/scene.tscn
godottrench --mcp-http --project godot
```
