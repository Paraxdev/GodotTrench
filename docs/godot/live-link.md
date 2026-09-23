# Live link protocol

The addon runs a small server in the Godot editor that GodotTrench talks to for rebuilds, live mode and status. You only
need this page to write your own client or debug the connection.

## Transport

Newline delimited JSON over TCP on `127.0.0.1:7842` (`godottrench/live_link_port`). Every request carries a `seq` that
the reply echoes. The editor keeps one connection open and sends a `status` heartbeat every second.

## Events

| Event | Fields | Reply |
| --- | --- | --- |
| `status` | | `project`, `godot`, `pid`, `maps: [{path, epoch}]` for the edited scene |
| `map_saved` | `path` | `rebuilt` |
| `build` | `path`, `text` | `rebuilt` |
| `focus` | | |
| `live_begin`, `live_resync` | `path`, `text` | `epoch` |
| `live_delta` | `path`, `ops` | `epoch`, or `ok: false, resync: true` when the ops do not fit |
| `live_end` | `path`, `revert` | |
| `inspect` | `path`, `ids` | Generated node name, class and position per map node id |
| `export_game_config` | | |

## Live ops

| Op | Fields | Meaning |
| --- | --- | --- |
| `set` | `id`, `parent`, `index`, `node` | Insert or replace a node, as stored in the `.gtm`, without children |
| `remove` | `id` | Remove a node with its subtree, only the topmost id is sent |
| `translate` | `ids`, `offset` | Move nodes, sent while dragging instead of geometry |
| `properties` | `properties` | Replace the worldspawn properties |

Edits only go out once Godot has taken the previous batch, so a slow Godot editor never piles up work.

A map's build `epoch` goes up whenever it is built outside a live session or its `FuncGodotMap` nodes change. The
editor then starts over with `live_begin`.

Generated nodes carry their map node id in the `_gt_id` metadata, and `_gt_group` for layers and groups.

## Testing without a window

Open a scene with a `FuncGodotMap` in a headless Godot editor, start GodotTrench with MCP and the project, then check
what Godot built with `inspect`:

```sh
godot --headless --editor --path godot res://path/scene.tscn
godottrench --mcp-http --project godot
```
