# MCP server

The editor embeds a [Model Context Protocol](https://modelcontextprotocol.io) server, so agents and scripts can
inspect and edit maps, take screenshots and play input into the UI.

## Starting it

| Transport | How |
| --- | --- |
| HTTP | `--mcp-http` (port 7841) or `--mcp-http=PORT`. Or tick *enable on startup* in *File > Preferences* and restart |
| stdio | `--mcp`, the client launches the editor for each session |

```sh
claude mcp add --transport http godottrench http://127.0.0.1:7841/mcp   # HTTP
claude mcp add godottrench -- /path/to/godottrench --mcp                 # stdio
```

The repository's `.mcp.json` already points Claude Code at the HTTP server.

## Tools

| Area | Tools |
| --- | --- |
| Inspect | `get_state`, `summarize_map`, `list_nodes`, `get_node`, `changes_since`, `validate_map`, `get_game_config`, `code_reference` |
| Create | `create_brush`, `create_mesh`, `create_terrain`, `create_entity`, `import_model` |
| Edit | `update_entity`, `select`, `transform`, `duplicate`, `set_face`, `mesh_edit`, `texture`, `terrain_edit`, `hierarchy`, `set_map_properties` |
| Paint and gameplay | `scatter`, `blend`, `gameplay` |
| Editor | `run_action`, `map_file`, `open_project`, `set_editor`, `set_camera`, `screenshot`, `simulate_input` |
| Scripts | `run_script`, see [MCP scripts](mcp-scripts.md) |

Each tool's description lists its operations and arguments.
[tools/mcp_client.py](https://github.com/Paraxdev/GodotTrench/blob/main/tools/mcp_client.py) prints them with
`list`, and calls a tool from the shell:

```sh
python tools/mcp_client.py call screenshot '{"target":"3d","width":1280,"height":720}' --out shot.png
```

## Screenshots

| Arguments | Result |
| --- | --- |
| `target` only | The docked view as it is, overlays included |
| `width` and `height` | Rendered offscreen up to 4096 pixels, without grid, entity boxes, gizmos or selection |
| `width`, `height` and `"overlays": true` | Offscreen, with the editor overlays |
| `"overlays": false` | A clean shot at the docked size |

## Reading and reviewing a map

| Step | Tool |
| --- | --- |
| Get an overview | `summarize_map`: layers, entity classes, I/O links, materials, missing assets |
| Find nodes | `list_nodes` with `layer`, `material`, `property` (`{"model": "*crate*"}`) or `box` |
| Look at one | `get_node` with `compact: true` leaves out vertices and faces |
| Show what you changed | `changes_since`, against the last save, `undo_steps` back or a map `file` |
| Take it back | Each call and each `run_script` is one undo step named `MCP: ...`, `run_action undo` with `steps` undoes several |

See [Reviewing changes](editor/reviewing.md) for keeping maps in git.

## Behaviour worth knowing

| Topic | Behaviour |
| --- | --- |
| Errors | A tool that could not do its job returns `isError`, including a failed `run_action` |
| File dialogs | Actions that open one are refused, use `map_file` with a path. `run_action save` needs a map that already has a file |
| Unsaved tabs | `run_action close_tab` refuses unsaved changes unless `args.discard` is true |
| Selection | `select` refuses unknown ids and skips hidden or locked nodes, listing them in `skipped` |
| CSG | `csg_subtract`, `csg_merge`, `csg_intersect`, `csg_hollow` and `clip_apply` return `replaced`, old ids mapped to new ones |
| Timeouts | Calls time out after 120 s, except `run_script` |

## Tool notes

| Tool | Note |
| --- | --- |
| `run_action` `copy`, `cut`, `paste` | Copy and cut return the clipboard text, paste takes it as `args.text` with an optional `offset` or `origin` |
| `run_action` `clip_apply` | Takes a plane (`point` and `normal`, or three `points`) and `keep`: `front`, `back` or `both` |
| `run_action` `create_decal` | Places a [decal](editor/decals.md), use it instead of thin brushes |
| `create_brush` | `openings` carves doors and windows, `{min, max, count, step, rows, row_step}` repeats one as a grid |
| `create_mesh` `rotate` | `[x, y, z]` degrees in Godot's YXZ order, so `[0, 0, 90]` lays a cylinder along X |
| `transform` | Applies translate, rotate, flip and scale_to in that order, as one undo step |
| `terrain_edit` | `probe [x, z]` without `op` only reads the height |
| `validate_map` | Accepts any method or property of the target's Godot class or script as an input. `coplanar_faces` flags faces that will flicker |
| `run_action` `reload_materials` | Rescans the texture folder now, though new files are found on their own anyway |
