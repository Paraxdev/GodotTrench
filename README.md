# GodotTrench

A brush based level editor for Godot, written in Rust. It aims for TrenchBroom parity plus Hammer / Hammer++ features
(entity I/O, prefabs, displacements, decals, vertex paint, lit preview) and Godot specific QoL.

heavily inspired by TrenchBroom and Hammer++
GodotTrench focuses more on Godot, allowing features like live view and a more custom plugin for Godot (based on FuncGodot)

Maps are saved as `.gtm` (JSON, Y-up, Godot units convention) and loaded in Godot by the GodotTrench fork of
[FuncGodot](https://github.com/func-godot/func_godot_plugin) in `godot/addons/func_godot`.

# Disclosure
This Project makes use of AI / LLMs, specifically Claude.
Expect bugs and weird shenanigans, you are open to create issues or pull requests with fixes. 

If you want to use AI to create pull requests you must ensure it isn't sloppy (like heavy emdash use, useless and unnecessary comments and a heavy use of emojis)

you also by creating a pull request you vouch for the changes the AI made, and if it causes any faults you own the blame.

you must pass the whole test suite and preferably write tests for the feature you changed or added.

## Godot addon

`godot/` contains a example project containing the addon and demo scenes. you may use the demo scenes in your project.

to use in your own project, add a `FuncGodotMap` node, point it at a `.gtm` file and press *Build Map*. Entity definitions are FuncGodot FGD resources;

the addon exports them to `godottrench_game.json`, which the editor reads when you open the project folder.

## Building

```sh
cargo run -p gt_editor --release            # start the editor
cargo run -p gt_editor --release -- map.gtm # open a map
```

Requires a GPU with Vulkan, DX12 or Metal support.

## Showcase maps

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
godot --headless --path godot --script res://tests/build_showcase.gd         # builds res://demo/showcase/*.tscn
godot --path godot --script res://tests/showcase_screenshots.gd -- <out dir>  # renders every viewpoint
godot --path godot                                                            # play them: WASD, E use, 1/2/3 switch maps, F3 I/O overlay
```

Scripts are JSON: `{"format": "godottrench-mcp-script", "vars": {...}, "steps": [{"tool", "args", "save", "note"}]}`. A step's
result is stored under its `save` name and later args refer to it with `"$name.path"` (the JSON value) or `"${name.path}"`
(inside strings). `$project` and `$script_dir` are predefined. Textures and the demo Blockbench models come from
`cargo run -p gt_samples --bin showcase --release`.

## Level design tools

* **Scatter** (`B`): paints trees, rocks or foliage into a scatter set that lives on its own layer. A set only lands on its
  target surfaces (Alt+click a brush, mesh or terrain to add or remove it as a target), so painting a forest on a terrain never
  covers the house on top of it. The palette window has presets (forest, pines, undergrowth, rocks, grass) and takes your own
  models (`.bbmodel`, `.glb`, `.gltf`, `.tscn`), each with a weight, spacing (spread), scale range, normal alignment, tilt and sink.
  LMB paints, Shift+LMB erases (optionally only chosen items), Ctrl+wheel resizes the brush, *Fill* covers the targets with slope
  and height limits. Foliage sets skip collision and shadows and fade out by distance. In Godot every set becomes MultiMeshes with
  shared collision shapes, or instanced scenes when the model has scripts. *Scatter Sets to Entities* (Terrain menu) turns a set into `prop_model` entities.
* **Blend** (`Shift+G`): paints terrain splat layers, displacement alpha and a second material on brush and mesh faces
  (Texture menu, *Blend material*). Modes: paint, erase, smooth, sharpen, noise, slope and height masks, with smooth, linear,
  constant and spray falloffs. Godot uses the same blend through `gt_blend.gdshader`.
* **Volume** (`Shift+E`): drag out a `trigger_once`, `trigger_multiple`, `trigger_call`, spawn area, hurt, teleport, push or plain area volume in one step.
* **Gameplay wizards** (Gameplay menu and the selection panel): turn brushes into a door that swings on its left or right hinge or
  slides in any direction (optionally with a walk-up trigger), a lift, a button that fires a target, a trigger or spawn area around the
  selection, and *Link…* to connect any output to any input.
* **Gizmos**: selected entities show draggable handles for hinges, travel offsets, radii, spot cones, spawn areas and
  target points. Entity definitions declare them (`"gizmos"` in the game config) or they are inferred from property names.
* **Reference** tab: for the selected entity class, GDScript and C# classes you can drop into your project, usage snippets,
  the FGD resource, and the C# helper with the attributes described below.

## Gameplay entities and C#

The fork ships a ready entity library (`godot/addons/func_godot/fgd/godottrench`): `func_door`, `func_door_rotating`,
`func_gate`, `func_platform`, `func_train` with `path_corner`, `func_button`, `trigger_once`, `trigger_multiple`,
`trigger_call`, `trigger_spawn_area`, `trigger_hurt`, `trigger_teleport`, `trigger_push`, `info_spawner`,
`info_teleport_destination`, `logic_call`, `logic_relay`, `logic_timer`, `logic_counter`, `logic_auto` and `logic_debug`.

Outputs target entities by targetname (with `*` wildcards), `@group`, a node path (`/root/Game/Score`), `!player`, `!activator`
or `!self`. Inputs call any method on the target, GDScript or C# (`add_score` also finds `AddScore`), and a JSON array parameter
is spread into arguments with `$activator`, `$self`, `$position` and `$caller_name` placeholders, converted to the declared types.
`trigger_call` and `logic_call` call a method directly, for example `call_target = "/root/Game"`, `method = "give_item"`,
`arguments = ["key_red", "$activator"]`. `GodotTrenchIO.events()` reports every fired output, and
`GodotTrenchDebugOverlay` shows them in game (F3) together with the trigger volumes.

C# classes can be entities without any `.tres` file. Add the helper from the Reference tab (it defines the attributes) and list your
source folders in `GodotTrenchGameConfig.csharp_source_dirs`:

```csharp
[GlobalClass]
[GodotTrenchEntity("npc_guard", Description = "Patrolling guard", Color = "#ff4040ff", Size = "-16 0 -16 16 64 16")]
public partial class NpcGuard : CharacterBody3D
{
    [Signal] public delegate void AlertedEventHandler(Node activator, int level);   // output "alerted"
    [Export] public float WalkSpeed { get; set; } = 3.5f;                           // property "walk_speed"
    [GodotTrenchInput] public void Alert(Node activator) { }                        // input "alert"
}
```

## Controls (TrenchBroom style)

* 3D: hold right mouse to look, WASD to fly (Q/E down/up while looking), middle mouse pans, wheel dollies, Alt+left drag orbits.
* 2D: right or middle mouse pans, wheel zooms.
* Left drag on empty space draws a brush, drag the selection to move it (Alt: vertical, Ctrl: duplicate, Shift: lock axis).
* Shift+click selects faces, Shift+drag a face resizes the brush, Ctrl+Shift+drag extrudes. In 2D views drag selection edges to resize.
* Tools: `C` clip (Tab changes kept side, Enter applies), `V` vertex, `R` rotate, `T` scale, `G` sculpt, `P` paint,
  `B` scatter, `Shift+G` blend, `Shift+E` volume, `M` measure, `Shift+P` path, `Shift+T` texture, `Tab` edit mesh,
  `Q`/`Esc` back to select. Ctrl+wheel resizes the scatter, blend and sculpt brushes.
* `[` / `]` grid, `Ctrl+D` duplicate, `Ctrl+G` group, `Ctrl+K` subtract, `Ctrl+M` merge, `Ctrl+H` hide, `Ctrl+J` isolate, `F` focus.
* `Ctrl+Shift+E` convert to mesh, `Ctrl+Shift+J` join meshes, `Ctrl+Shift+N` new tab, `Ctrl+Tab` next tab, `Ctrl+W` close tab.
* Every binding can be changed in the *Keyboard Shortcuts* window, starting from the TrenchBroom, Hammer or Blender preset
  chosen in Preferences.

## MCP server (AI agents and automation)

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

## Tests

```sh
cargo test --workspace                                            # unit tests + headless UI tests (egui_kittest)
cargo test -p gt_editor --test e2e -- --ignored --test-threads=1  # end to end: launches the editor, drives it over MCP

cargo run -p gt_formats --example demo_maps                       # regenerate the Godot test maps
godot --headless --path godot --import
godot --headless --path godot --script res://tests/run_tests.gd   # addon tests: .gtm build, I/O, gameplay entities, scatter, blend, C#, a showcase playthrough
```

The `example_mcp_scripts_replay` end to end test replays every script in `examples/mcp` and checks the result.

# Thanks

this project couldn't exist with the work of: <br>
[TrenchBroom](https://github.com/TrenchBroom/TrenchBroom) as it inspired the whole ui and features
and [FuncGodot](https://github.com/func-godot) which was forked and modified to have a more advanced set of features.
see [FORK.md](/godot/addons/func_godot/FORK.md) for changes made
