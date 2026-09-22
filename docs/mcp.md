# MCP server and scripts

The editor embeds a [Model Context Protocol](https://modelcontextprotocol.io) server so agents such as Claude can inspect and
edit maps, take viewport screenshots and play real mouse and keyboard input into the UI.

* HTTP: start with `--mcp-http` (port 7841) or `--mcp-http=PORT`, or enable it in Preferences.
  This repository's `.mcp.json` points Claude Code at `http://127.0.0.1:7841/mcp`.
  Elsewhere: `claude mcp add --transport http godottrench http://127.0.0.1:7841/mcp`.
* stdio: `claude mcp add godottrench -- /path/to/godottrench --mcp` launches the editor per session.

Tools: `get_state`, `list_nodes`, `get_node`, `run_action`, `create_brush`, `create_mesh`, `mesh_edit`, `texture`,
`create_terrain`, `terrain_edit`, `import_model`, `create_entity`, `update_entity`, `select`, `transform`, `duplicate`, `set_face`,
`hierarchy`, `set_map_properties`, `scatter`, `blend`, `gameplay`, `code_reference`, `map_file`, `open_project`, `set_editor`,
`set_camera`, `screenshot`, `simulate_input`, `validate_map`, `get_game_config`, `run_script`.

`tools/mcp_client.py` is a dependency free command line client, handy for scripting:

```sh
python tools/mcp_client.py call create_brush '{"min":[0,0,0],"max":[64,64,64],"shape":"cylinder"}'
python tools/mcp_client.py call screenshot '{"target":"3d"}' --out view.png
```

## Showcase map scripts

The three showcase maps are built entirely from MCP tool calls. Each one is a script in `examples/mcp/` that you can replay
in a running editor, read as a reference for the tools, or copy as a starting point:

* `mountain_house.json`: a mountain terrain with a flattened plateau, a cabin hanging off the cliff on cantilevers and chains,
  a furnished interior, a hinged door that opens when you walk up, a cliff lift, a respawn teleport and pine and oak forests.
* `church_school.json`: a church with arcades, stained glass, linked pews and chandeliers, a bell tower with double doors,
  a school wing where three chalk buttons count up and open the courtyard gate, and a courtyard floor blending cobble into grass.
* `lighthouse_forest.json`: an island with a striped lighthouse (lathe bands with carved openings, spiral stairs, rotating beacon),
  a keeper's cottage, a ring wall arrayed around the tower, gates opened through a relay, a boat on a `func_train` path,
  wisps spawning along the forest path and a `trigger_call` that calls a Godot method on the beacon.

```sh
godottrench --mcp-http --project godot                                        # editor with the demo project
python tools/mcp_script.py run examples/mcp/lighthouse_forest.json            # replays the script, saves godot/demo/maps/showcase
python tools/mcp_script.py shot view.png --pos 4300,900,3300 --look 2200,700,1200 --shade lit
godot --headless --path godot --script res://tests/build_showcase.gd         # builds res://demo/showcase/*.scn (binary, built scenes embed every mesh and shape)
godot --path godot --script res://tests/showcase_screenshots.gd -- <out dir>  # renders every viewpoint
godot --path godot                                                            # play them: WASD, E use, 1/2/3 switch maps, F3 I/O overlay
```

Scripts are JSON: `{"format": "godottrench-mcp-script", "vars": {...}, "steps": [{"tool", "args", "save", "note"}]}`. A step's
result is stored under its `save` name and later args refer to it with `"$name.path"` (the JSON value) or `"${name.path}"`
(its text inside a longer string). Ids that arrive as text this way, such as `"42"`, are accepted wherever a node id is expected.

* `$$` is a literal `$`, so a Godot node path is written `"$$Player"` and `"get_node($$Door)"`.
* A `$` that cannot start a reference, such as `"$5"` or `"a $ b"`, stays as it is.
* `"$name"` with a name no step saved is an error, so typos never pass silently.
* `$project` is the open Godot project's folder. It is only defined while a project is open, a script that uses it without
  one fails with a hint to call `open_project` first, instead of writing to the drive root.
* `$script_dir` is the folder of the script file. For steps passed inline it is the project folder, or the editor's working
  directory without a project.

Scripts run on the editor's UI thread, so the server waits for `run_script` as long as it takes. Other calls give up after
120 s with a timeout error, and a request the editor dropped says so instead of timing out. `tools/mcp_script.py` waits up
to an hour per call (`--timeout` changes that) and prints the editor's error text when a script cannot start.

## Behaviour worth knowing

* Errors are errors: a tool that could not do its job returns `isError`, including `run_action` failures such as a save
  that failed or a `close_tab` refused because of unsaved changes (pass `args.discard: true` to drop them). `run_action` only reports a `status` set by that action.
* `run_action save` needs a map that already has a file, and actions that would open a native file dialog (open, save as,
  import, export) are refused, use `map_file` with a path instead. `map_file new` and `open` keep a modified map open in
  its own tab like the File menu, `export_map` returns the written `.map` path.
* `run_action copy` and `cut` return the clipboard text, `paste` takes it back as `args.text`, optionally moved by
  `offset` or centered on `origin`, independent of the mouse.
* `select` refuses unknown ids and leaves out hidden or locked nodes (listed in `skipped`), like clicking in a view.
  Creating objects in a locked layer or group is an error, and parents must be layers or groups (or brush entities for
  brushes and meshes).
* `transform` applies translate, rotate, flip and scale_to in that order as one undo step, rotating and flipping about the
  selection center at that point.
* `mesh_edit` checks every face, vertex and edge index against the mesh first.
* `terrain_edit` with only `probe [x, z]` reads the height without editing.
