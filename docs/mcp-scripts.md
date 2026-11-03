# MCP scripts

An MCP script is a JSON file with a list of [MCP](mcp.md) tool calls, and `run_script` replays them in order. It
keeps how a map was built as readable steps, so you can rebuild the map from scratch and review it as text in a pull
request, and the tests can replay it too.

The showcase maps in [examples/mcp](https://github.com/Paraxdev/TrenchDot/tree/main/examples/mcp) are built this
way, and they double as examples for every tool.

| Script | Builds |
| --- | --- |
| `mountain_house.json` | A cabin hanging off a cliff, with a cliff lift and forests |
| `church_school.json` | A church and a school where three buttons open the courtyard gate |
| `lighthouse_forest.json` | A lighthouse with a rotating beacon, a boat on a `func_train` and spawning wisps |
| `withered_city.json` | A ruined neon city at night with lightning, glow and `ssr` |
| `night_district.json` | An industrial street at night with a roller door and a viaduct train |
| `sea_island.json` | The [sea island tutorial](tutorials/sea-island.md) |
| `scripted_scene.json` | The [cutscene tutorial](tutorials/cutscene.md) |
| `logic_playground.json` | A hall for the [math and logic nodes](gameplay/entities/logic.md): coins, a score, a gate on the vault door and a light lottery |

## Running them

Start the editor with the demo project and its HTTP server, see [Starting it](mcp.md#starting-it), then replay a
script with `tools/mcp_script.py`. If the editor asks for a [token](mcp.md#the-http-token), pass it with `--token` or
`TRENCHDOT_MCP_TOKEN`. Its `shot` command saves a screenshot from the camera position you give it, and `--shade`
picks the shading:

```sh
trenchdot --mcp-http --project godot                        # the editor with the demo project
python tools/mcp_script.py run examples/mcp/sea_island.json   # replay a script
python tools/mcp_script.py shot view.png --pos 4300,900,3300 --look 2200,700,1200 --shade lit
```

The scripts save into `godot/demo/maps/showcase`, except `scripted_scene.json` and `logic_playground.json`, which save
into `godot/demo/maps`.

A run stops at the first failed step unless you pass `--continue-on-error`, and `--url` points the runner at an editor
on another port. Each run is one undo step in every map it changes, named after the script's `label` or its file.

## Script format

```json
{
  "format": "trenchdot-mcp-script",
  "label": "Hollow room",
  "vars": { "save_dir": "${project}/maps" },
  "steps": [
    { "note": "A hollow room." },
    { "tool": "map_file", "args": { "op": "new" } },
    { "tool": "create_brush", "args": { "min": [0, 0, 0], "max": [256, 128, 256] }, "save": "room" },
    { "tool": "select", "args": { "ids": "$room.ids" } },
    { "tool": "run_action", "args": { "action": "csg_hollow" } },
    { "tool": "map_file", "args": { "op": "save", "path": "${save_dir}/room.tdm" } }
  ]
}
```

`vars` sets defaults for your own variables. A `vars` object passed to `run_script` overrides them, so one script can
save to different places, and the folders you pass this way are ones the script may read from and write into.

A step without `tool` is a comment. A step with `save` stores its result under that name, so later steps can use the
ids it created.

Scripts get shared like maps, so their steps only edit and save the map. A step can't open another project or one of its
maps, start Godot, build in it, turn on live mode, copy or cut to the clipboard, throw away unsaved changes or close
other tabs. Undo and redo only reach what the script did itself, so the work you haven't saved stays in every tab.
`screenshot`, `simulate_input`, `walkability` and `run_script` can't run inside a script either, and neither can
`project_content` installs other than `gameplay`.

Every file a step reads or writes has to be in the open project, the script's own folder or a folder you pass in `vars`,
also when a link inside them points somewhere else. That covers the maps it opens, checks, compares, saves and exports,
the maps, models, prefabs and textures it imports with the files they name, and everything that conversions, imports,
exports and the prefab library write. Prefabs, model files, textures and Hammer instances that a map or model names
elsewhere are left out, network paths are refused before the editor looks at them, and a save without a path writes back
only to the map that was open when the script started.

A refused step shows up in the summary's `errors` with the reason. The summary counts tool calls as `steps` and comments
as `notes`.

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

An unknown name is an error. A `$` that can't start a name, like `"$5"`, stays as it is. Ids given as text, such as
`"42"`, work wherever a node id is expected.

`$name/rest` without braces doesn't expand, so an unbraced `$project/maps` is an error rather than a path nobody meant
to write. Use `"${project}/maps"`.

## Ids after CSG

CSG and clip steps replace the brushes they cut with new ones that have new ids. The runner rewrites ids saved by
earlier steps to match, so `"$wall.ids"` still means the whole wall after you cut a window into it, and a removed brush
drops out.

Only values under keys that carry node ids are rewritten: `id`, `ids`, `selection`, `selected`, `nodes`,
`letters`, `copies`, `entity`, `group`, `parent`, `insert_parent`, `current_layer`, `open_groups`, `targets` and keys
ending in `_id` or `_ids`. Counts such as `moved` from `hierarchy reparent` stay as they are, use its `ids` instead.

> [!TIP]
> Spell out every brush setting in `scatter` and `blend` calls. Anything left out comes from the editor's
> current Scatter panel or Blend tool settings.
