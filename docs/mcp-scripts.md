# MCP scripts

An MCP script is a JSON file with a list of [MCP](mcp.md) tool calls that the editor replays in order with
`run_script`. It records how a map was built as readable steps, so the map can be rebuilt from scratch, reviewed as
text in a pull request and replayed by the tests. The showcase maps in
[examples/mcp](https://github.com/Paraxdev/GodotTrench/tree/main/examples/mcp) are built this way and double as
examples for every tool. The runner below talks to the editor's HTTP MCP server, see [Starting it](mcp.md#starting-it).

| Script | Builds |
| --- | --- |
| `mountain_house.json` | A cabin hanging off a cliff, with a cliff lift and forests |
| `church_school.json` | A church and a school where three buttons open the courtyard gate |
| `lighthouse_forest.json` | A lighthouse with a rotating beacon, a boat on a `func_train` and spawning wisps |
| `withered_city.json` | A ruined neon city at night with lightning, glow and `ssr` |
| `night_district.json` | An industrial street at night with a roller door and a viaduct train |
| `sea_island.json` | The [sea island tutorial](tutorials/sea-island.md) |
| `scripted_scene.json` | The [cutscene tutorial](tutorials/cutscene.md) |

## Running them

Start the editor with the demo project, then replay a script with `tools/mcp_script.py`. Its `shot` command saves a
screenshot from a given camera position, with `--shade` picking the view's shading:

```sh
godottrench --mcp-http --project godot                        # the editor with the demo project
python tools/mcp_script.py run examples/mcp/sea_island.json   # replay a script
python tools/mcp_script.py shot view.png --pos 4300,900,3300 --look 2200,700,1200 --shade lit
```

The scripts save into `godot/demo/maps/showcase`, `scripted_scene.json` into `godot/demo/maps`. A run stops at the
first failed step unless you pass `--continue-on-error`, and `--url` points the runner at an editor on another port.
Each run is one undo step, named after the script's `label` or its file.

## Script format

```json
{
  "format": "godottrench-mcp-script",
  "label": "Hollow room",
  "vars": { "save_dir": "${project}/maps" },
  "steps": [
    { "note": "A hollow room." },
    { "tool": "map_file", "args": { "op": "new" } },
    { "tool": "create_brush", "args": { "min": [0, 0, 0], "max": [256, 128, 256] }, "save": "room" },
    { "tool": "select", "args": { "ids": "$room.ids" } },
    { "tool": "run_action", "args": { "action": "csg_hollow" } },
    { "tool": "map_file", "args": { "op": "save", "path": "${save_dir}/room.gtm" } }
  ]
}
```

`vars` sets default values for variables of your own. A `vars` object passed to `run_script` overrides them, so one
script can save to different places.

A step without `tool` is a comment. A step with `save` stores its result under that name, so later steps can use the
ids it created. `screenshot`, `simulate_input` and `run_script` cannot run inside a script. The summary a run returns
counts tool calls as `steps` and comments as `notes`.

Strings in `args` can refer to saved results and a few built-in variables:

| Write | Means |
| --- | --- |
| `"$name.path"` | The saved JSON value. Numbers index arrays, as in `$room.ids.0` |
| `"${name.path}"` | Its text inside a longer string |
| `"$a.ids + $b.ids"` | Both joined into one array. `["$a.ids", "$b.ids"]` works too |
| `$last` | The previous step's result |
| `$project` | The open Godot project's folder, an error when none is open |
| `$script_dir` | The script file's folder |
| `$$` | A literal `$`, so a node path is `"$$Player"` |

An unknown name is an error. A `$` that cannot start a name, like `"$5"`, stays as it is. Ids given as text, such as
`"42"`, work wherever a node id is expected. `$name/rest` without braces does not expand, so an unbraced
`$project/maps` is an error rather than a path nobody meant to write. Use `"${project}/maps"`.

## Ids after CSG

CSG and clip steps replace the brushes they cut with new ones that have new ids. The runner rewrites ids saved by
earlier steps to match, so `"$wall.ids"` still means the whole wall after a window is cut into it, and a removed brush
drops out. Only values under `id`, `ids`, `selection`, `letters`, `copies`, `entity` and keys ending in `_id` or `_ids`
are rewritten.

> **Tip:** Spell out every brush setting in `scatter` and `blend` calls. Anything left out comes from the editor's
> current Scatter panel or Blend tool settings.
