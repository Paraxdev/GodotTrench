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
python tools/mcp_client.py call screenshot '{"target":"3d","width":1280,"height":720}' --out big.png
```

A viewport screenshot is as large as the docked view and shows it as it is. With `width` and `height` the view's camera
renders offscreen at that size instead, as a beauty shot: only what the game shows, without grid, entity boxes, trigger
volumes, edges, selection outlines, selection tint, I/O links or gizmos. Pass `"overlays": true` to keep the editor overlays in an
offscreen capture, or `"overlays": false` without a size for a beauty shot at the docked size.

## Showcase map scripts

The showcase maps are built entirely from MCP tool calls. Each one is a script in `examples/mcp/` that you can replay
in a running editor, read as a reference for the tools, or copy as a starting point:

* `mountain_house.json`: a mountain terrain with a flattened plateau, a cabin hanging off the cliff on cantilevers and chains,
  a furnished interior, a hinged door that opens when you walk up, a cliff lift, a respawn teleport and pine and oak forests.
* `church_school.json`: a church with arcades, stained glass, linked pews and chandeliers, a bell tower with double doors,
  a school wing where three chalk buttons count up and open the courtyard gate, and a courtyard floor blending cobble into grass.
* `lighthouse_forest.json`: an island with a striped lighthouse (lathe bands with carved openings, spiral stairs, rotating beacon),
  a keeper's cottage, a ring wall arrayed around the tower, gates opened through a relay, a boat on a `func_train` path,
  wisps spawning along the forest path and a `trigger_call` that calls a Godot method on the beacon.
* `withered_city.json`: an abandoned city in a storm where only the neon still burns. The way in is a wet street between
  ruined row houses hung with blade signs (round tops, logo discs and double tube frames) and box signs over the doors,
  all block letters from `create_brush` with `shape: text` turned to face the street, with paper lanterns, cables and
  debris frozen in mid air. A timer throws lightning whose bolt is the light's `fixture`, one sign flickers, one hangs
  askew and one has a dead letter. Glowing letters spelling STILL HERE lie on a cracked plaza, towers are stacked storeys
  with repeated window openings, broken with plane clips, vertex moves and CSG cutters, a shutter opens from a button and
  a lift climbs to a roof that looks out over the signs. Weeds, scrub and dead trees are painted only where the ground
  terrain shows through, and the worldspawn turns on glow and `ssr` so the street mirrors the neon in Godot.
* `night_district.json`: an industrial street at night under sodium lamps, apartment blocks with emissive facades, a corner
  shop with a neon sign and an automatic door, an alley lamp flickering from a random timer with its bulb following through
  `switched`, a warehouse roller door on a switch with a beacon, hall lights and courtyard lamps switched on by triggers,
  a chain-link yard and a train shuttling along a viaduct over scattered waste ground.

`sea_island.json` is not one of the built showcase scenes. It replays the wiki's sea island tutorial step for step: the
generated island lowered into a water brush, sculpted hills, terraced cliffs, a flattened beach, sand, rock and dirt paths
painted with the `blend` tool's height, slope and paint modes, de-tiled layers, then scatter sets for a woodland, hero trees,
bushes, cliff boulders, meadow grass kept off the paths and grass growing on the boulders, and a brush jetty with Poly Haven
props. Its painting calls spell out every brush setting, since the Scatter panel's settings fill in whatever a call leaves out.

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

* `"$a.ids + $b.ids"` joins saved values into one array (arrays are spread, single values appended), so one
  `select` or `duplicate` can take the brushes of several steps. Id lists also accept nested arrays, so
  `["$a.ids", "$b.ids"]` works too.
* `$$` is a literal `$`, so a Godot node path is written `"$$Player"` and `"get_node($$Door)"`.
* A `$` that cannot start a reference, such as `"$5"` or `"a $ b"`, stays as it is.
* `"$name"` with a name no step saved is an error, so typos never pass silently.
* `$project` is the open Godot project's folder. It is only defined while a project is open, a script that uses it without
  one fails with a hint to call `open_project` first, instead of writing to the drive root.
* `$script_dir` is the folder of the script file. For steps passed inline it is the project folder, or the editor's working
  directory without a project.

Steps that replace brushes (`run_action` with `csg_subtract`, `csg_merge`, `csg_intersect`, `csg_hollow` or
`clip_apply`) return `replaced`, a map from each old id to the ids that took its place, empty for brushes that are
gone such as the cutters. The script runner then rewrites the ids in every result saved by an earlier step, so
`"$wall.ids"` keeps meaning the wall after a window is cut into it. The rule, applied to the values under `id`, `ids`,
`selection`, `letters`, `copies`, `entity` and keys ending in `_id` or `_ids`:

* In a list, a replaced id gives way to all of its replacements in place, and drops out when it has none.
* A single id becomes its one replacement, a list of them when there are several, and null when it was removed.
* Other numbers, and variables passed in through `vars`, are never touched.

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
* `create_brush` with `shape: text` lays `text` out as block letter brushes that fill the bounds, lying with their tops
  towards -Z or `standing`, and returns each character's brush ids under `letters` so single letters can be moved or broken.
  The Shape Generator's Text shape builds the same letters by letter height, depth and spacing.
* `create_mesh` takes `rotate: [x, y, z]` in degrees, applied about the mesh center like Godot's `rotation_degrees`
  (Y, then X, then Z, the same order as entity angles), so `[0, 0, 90]` lays a cylinder along X and `[90, 0, 0]`
  along Z. `rotate_y` still turns about Y only.
* `scatter` `paint`, `stroke` and `fill` take `exposed_only` (off by default) with `clearance` (512 units): points with
  geometry above them within that height are skipped, so foliage filled over a terrain stays out from under slabs and
  roofs. The Scatter panel has the same option in its Brush section.
* Paths the tools return (`material_file`, map and project paths, `export_map`) always use forward slashes.
* `validate_map` knows that inputs can call any method or set any property of the target's Godot class or script, as
  the runtime does, so `set_visible` on a `func_illusionary` passes while a typo such as `set_visibel` is reported.
  Classes whose Godot class is unknown are only checked against their declared inputs. Targetnames listed in the map's
  Godot overlay sidecar (`<map>.overlay.json`, see `docs/godot.md`) count as existing targets.
  An entry of `openings` can be `{min, max, count, step, rows, row_step}` to cut a whole grid of windows.
* `run_action clip_apply` takes a plane (`point` and `normal`, or three `points`) and `keep: front|back|both`, and
  `move_vertices {vertices, offset}` runs the vertex tool on the selected brushes, so scripts can break edges without mouse input.
* CSG ops replace the brushes they touch and return `replaced` (see above), and inside scripts the saved ids follow.
  `csg_subtract` takes `args.carve_material`: `cutter` (the default) gives the carved faces the cutter's material,
  `target` gives them the material of the target face they are cut from, like the *Carved faces keep the target's
  material* toggle in *Brush > CSG*.
