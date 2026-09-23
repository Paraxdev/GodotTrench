# MCP server

The editor embeds a [Model Context Protocol](https://modelcontextprotocol.io) server. Agents such as Claude and your own
scripts can inspect and edit maps, take screenshots and play mouse and keyboard input into the UI, using the same
operations you do.

## Starting it

| Transport | How |
| --- | --- |
| HTTP | `--mcp-http` (port 7841) or `--mcp-http=PORT`, or turn it on in *File > Preferences* |
| stdio | `--mcp`, the client launches the editor per session |

Registering it with Claude Code:

```sh
claude mcp add --transport http godottrench http://127.0.0.1:7841/mcp   # HTTP
claude mcp add godottrench -- /path/to/godottrench --mcp                 # stdio
```

This repository's `.mcp.json` already points Claude Code at the HTTP server. Preferences also shows the command.

## Tools

| Area | Tools |
| --- | --- |
| Inspect | `get_state`, `list_nodes`, `get_node`, `validate_map`, `get_game_config`, `code_reference` |
| Create | `create_brush`, `create_mesh`, `create_terrain`, `create_entity`, `import_model` |
| Edit | `update_entity`, `select`, `transform`, `duplicate`, `set_face`, `mesh_edit`, `texture`, `terrain_edit`, `hierarchy`, `set_map_properties` |
| Paint and gameplay | `scatter`, `blend`, `gameplay` |
| Editor | `run_action`, `map_file`, `open_project`, `set_editor`, `set_camera`, `screenshot`, `simulate_input` |
| Scripts | `run_script`, see [MCP scripts](mcp-scripts.md) |

## Command line client

`tools/mcp_client.py` is a dependency free client:

```sh
python tools/mcp_client.py call create_brush '{"min":[0,0,0],"max":[64,64,64],"shape":"cylinder"}'
python tools/mcp_client.py call screenshot '{"target":"3d"}' --out view.png
python tools/mcp_client.py call screenshot '{"target":"3d","width":1280,"height":720}' --out big.png
```

## Screenshots

| Call | Result |
| --- | --- |
| No size | The docked view as it is, overlays included |
| `width` and `height` | A beauty shot rendered offscreen: no grid, entity boxes, gizmos or selection |
| `"overlays": true` with a size | Offscreen, keeping the editor overlays |
| `"overlays": false` without a size | Beauty shot at the docked size |

## Behaviour worth knowing

**Errors are errors.** A tool that could not do its job returns `isError`, including failed `run_action` calls such as
a refused `close_tab` (pass `args.discard: true` to drop unsaved changes).

**No file dialogs.** Actions that would open one (open, save as, import, export) are refused, use `map_file` with a
path. `run_action save` needs a map that already has a file.

**Selection follows the UI.** `select` refuses unknown ids and skips hidden or locked nodes (listed in `skipped`).
Creating into a locked layer or group is an error.

**CSG replaces ids.** `csg_subtract`, `csg_merge`, `csg_intersect`, `csg_hollow` and `clip_apply` return `replaced`, a
map from old ids to the new ones. `csg_subtract` takes `args.carve_material`, `cutter` (default) or `target`.

**Paths** returned by tools always use forward slashes.

### Tool notes

| Tool | Note |
| --- | --- |
| `run_action copy`, `cut`, `paste` | Copy and cut return the clipboard text, paste takes it as `args.text` with optional `offset` or `origin` |
| `run_action clip_apply` | Takes a plane (`point` and `normal`, or three `points`) and `keep: front, back or both` |
| `run_action move_vertices` | `{vertices, offset}` runs the vertex tool on the selected brushes |
| `transform` | Applies translate, rotate, flip and scale_to in that order, as one undo step |
| `create_brush` `shape: text` | Block letters filling the bounds. Returns each character's ids under `letters` |
| `create_mesh` `rotate` | `[x, y, z]` degrees, Y then X then Z like Godot. `[0, 0, 90]` lays a cylinder along X |
| `terrain_edit` | With only `probe [x, z]` it reads the height without editing |
| `mesh_edit` | Checks every index against the mesh first |
| `scatter` | `paint`, `stroke` and `fill` take `exposed_only` with `clearance` (512 units) |
| `validate_map` | Accepts any method or property of the target's class or script as an input, and overlay targetnames as targets |
| `validate_map` `openings` | An entry `{min, max, count, step, rows, row_step}` cuts a whole grid of windows |
