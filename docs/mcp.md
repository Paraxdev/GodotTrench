# MCP server

MCP, the [Model Context Protocol](https://modelcontextprotocol.io), is a standard way for AI agents and scripts to call
into an application. The editor has an MCP server built in, so an agent such as Claude Code, or a Python script, can
inspect and edit maps, take screenshots and play input into the UI. You would use it to let an AI assistant build or
review a level, to script repetitive edits, or to test the editor automatically. The server is off unless you start
it, and it only listens on your own machine.

## Starting it

| Transport | How to start it |
| --- | --- |
| HTTP | Launch the editor with `--mcp-http` for port 7841, or `--mcp-http=PORT` for another port. To have it start every time, tick *enable on startup* next to *MCP server* in *File > Preferences* and restart |
| stdio | Launch with `--mcp`. The client starts the editor itself for each session and talks to it over standard input and output |

HTTP suits an editor you keep open and work in yourself while an agent helps. stdio suits a client that should own
its own editor. To connect Claude Code to either:

```sh
claude mcp add --transport http godottrench http://127.0.0.1:7841/mcp   # HTTP
claude mcp add godottrench -- /path/to/godottrench --mcp                 # stdio
```

The repository's `.mcp.json` already points Claude Code at the HTTP server.

`GODOTTRENCH_MCP_URL` overrides the URL that [tools/mcp_client.py](https://github.com/Paraxdev/GodotTrench/blob/main/tools/mcp_client.py)
and [tools/mcp_script.py](https://github.com/Paraxdev/GodotTrench/blob/main/tools/mcp_script.py) connect to, for an
editor started on a different port than `.mcp.json` expects.

## Tools

| Area | What it is for | Tools |
| --- | --- | --- |
| Inspect | Read the map, the editor state, the entity definitions and how to use an entity from game code, without changing anything | `get_state`, `summarize_map`, `list_nodes`, `get_node`, `changes_since`, `validate_map`, `get_game_config`, `code_reference` |
| Create | Add brushes, meshes, terrains, entities and imported models | `create_brush`, `create_mesh`, `create_terrain`, `create_entity`, `import_model` |
| Edit | Change what exists: entity keys, selection, placement, faces, textures, terrain, layers and groups, and worldspawn | `update_entity`, `select`, `transform`, `duplicate`, `set_face`, `mesh_edit`, `texture`, `terrain_edit`, `hierarchy`, `set_map_properties` |
| Paint and gameplay | Scatter models, blend materials, and turn brushes into doors, platforms and buttons | `scatter`, `blend`, `gameplay` |
| Editor | Run menu actions, open and save files, change editor settings, move the camera, take screenshots, send input and check where a player can walk | `run_action`, `map_file`, `open_project`, `set_editor`, `set_camera`, `screenshot`, `simulate_input`, `walkability` |
| Project | Add the nature models or the demo to the open Godot project | `project_content` |
| Scripts | Replay a saved list of tool calls, see [MCP scripts](mcp-scripts.md) | `run_script` |

Each tool's description lists its operations and arguments, so an agent learns them from the server itself.
[tools/mcp_client.py](https://github.com/Paraxdev/GodotTrench/blob/main/tools/mcp_client.py) prints them with
`list`, and calls a tool from the shell:

```sh
python tools/mcp_client.py call screenshot '{"target":"3d","width":1280,"height":720}' --out shot.png
```

## Screenshots

`screenshot` returns a PNG of an editor view. Its arguments decide what the image shows:

| Arguments | Result |
| --- | --- |
| `target` only | The docked view as it is, overlays included |
| `width` and `height` | Rendered offscreen up to 4096 pixels, without grid, entity boxes, gizmos or selection |
| `width`, `height` and `"overlays": true` | Offscreen, with the editor overlays |
| `"overlays": false` | A clean shot at the docked size |

### What Godot renders

`"source": "godot"` asks the Godot editor to render the map with its real lights, environment, fog and post
processing, and returns that PNG with any errors or warnings Godot logged while building it. Use it to judge lighting
and materials without launching the game.

```sh
python tools/mcp_client.py call screenshot '{"source":"godot","position":[0,256,512],"look_at":[0,64,0]}' --out godot.png
```

1. Save the map inside the project that is open in GodotTrench.
2. Open that project in the Godot editor, with a scene whose `FuncGodotMap` builds the map.
3. Call `screenshot` with `"source": "godot"`. Without `position` and `look_at` it uses the 3D view's camera.

Without [live mode](godot/live.md) Godot first builds the map as shown in GodotTrench, like *Build in Godot*, and
`"build": false` captures the scene as Godot has it. `width` and `height` default to 1280 by 720, and `fov` to the
3D view's.

> **Note:** Godot needs a window to render, a headless Godot returns an error instead of an image. The window may be
> in the background or minimized.

## Reading and reviewing a map

| Step | Tool |
| --- | --- |
| Get an overview | `summarize_map` lists the layers, entity classes, I/O links and materials, and any missing assets |
| Find nodes | `list_nodes` filters by `layer`, `material`, `property` (`{"model": "*crate*"}`) or a `box` in the map |
| Look at one | `get_node` returns one node. `compact: true` leaves out vertices and faces |
| Show what you changed | `changes_since` compares against the last save, a number of `undo_steps` back, or a map `file` |
| Take it back | Each call and each `run_script` is one undo step named `MCP: ...`. `run_action undo` with `steps` undoes several |

See [Reviewing changes](editor/reviewing.md) for keeping maps in git.

## Behaviour worth knowing

| Topic | Behaviour |
| --- | --- |
| Errors | A tool that could not do its job returns `isError`, including a failed `run_action` |
| File dialogs | Actions that open one are refused, use `map_file` with a path. `run_action save` needs a map that already has a file |
| Unsaved tabs | `run_action close_tab` refuses unsaved changes unless `args.discard` is true. `args.close_others` closes every tab but the active one instead |
| Map paths | `map_file open` and `open_tab` resolve the path to an absolute one before opening, so a later `save` cannot land somewhere else because the process directory changed |
| Selection | `select` refuses unknown ids and skips hidden or locked nodes, listing them in `skipped` |
| CSG | `csg_subtract`, `csg_merge`, `csg_intersect`, `csg_hollow` and `clip_apply` return `replaced`, old ids mapped to new ones |
| Timeouts | Calls time out after 120 s, except `run_script` and `project_content` |
| Content question | The question a new project gets about ready made content closes on the first call other than `get_state`, `screenshot` and `simulate_input`, and does not open again in that session |
| Script variables | `$name/rest` (unbraced) is an error when `name` is a known variable, since only `$name` alone or `${name}/rest` expand it. Write `${project}/maps` |

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
| `validate_map` | Accepts any method or property of the target's Godot class or script as an input. `coplanar_faces` flags faces that will flicker. Every issue has `bounds`. `path` checks a saved map without opening it, `project: true` checks every map in the project |
| `run_action` `reload_materials` | Rescans the texture folder now, though new files are found on their own anyway |
| `set_editor` `live_link_port` | Talks to a Godot editor whose project sets another `godottrench/live_link_port` |
| `scatter` `preset` | Adds the embedded Blockbench models a preset needs. A glTF preset in a project without the nature pack is an error that names `project_content` |
| `project_content` | `install: "nature"` or `"demo"` downloads from the release and answers once the files are in. Without arguments it says what the project has |
