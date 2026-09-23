# MCP scripts

An MCP script is a JSON list of tool calls the editor replays. The showcase maps are built entirely this way, and
their scripts in `examples/mcp` double as a reference for the tools.

## The showcase scripts

| Script | What it builds |
| --- | --- |
| `mountain_house.json` | A cabin hanging off a cliff, with a walk-up door, a cliff lift, a respawn teleport and forests |
| `church_school.json` | A church with stained glass and a bell tower, a school where three buttons open the courtyard gate |
| `lighthouse_forest.json` | A lighthouse island with a rotating beacon, relay gates, a boat on a `func_train` and spawning wisps |
| `withered_city.json` | A stormy ruined city lit by neon block letter signs, lightning as a light `fixture`, glow and `ssr` |
| `night_district.json` | An industrial night street with emissive facades, triggered lamps, a roller door and a viaduct train |
| `sea_island.json` | The [sea island tutorial](tutorials/sea-island.md), step for step |
| `scripted_scene.json` | The [cutscene tutorial](tutorials/cutscene.md) |

## Running them

```sh
godottrench --mcp-http --project godot                                        # editor with the demo project
python tools/mcp_script.py run examples/mcp/lighthouse_forest.json            # replay, saves to godot/demo/maps/showcase
python tools/mcp_script.py shot view.png --pos 4300,900,3300 --look 2200,700,1200 --shade lit
godot --headless --path godot --script res://tests/build_showcase.gd         # build the showcase scenes
godot --path godot --script res://tests/showcase_screenshots.gd -- <out dir>  # render every viewpoint
godot --path godot                                                            # play them
```

Scripts run on the editor's UI thread, so the server waits for `run_script` as long as it takes. Other calls time out
after 120 s. `tools/mcp_script.py` waits up to an hour per call, `--timeout` changes that.

## Script format

```json
{
  "format": "godottrench-mcp-script",
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

A step's result is stored under its `save` name, and later steps refer to it.

| Write | Means |
| --- | --- |
| `"$name.path"` | The saved JSON value |
| `"${name.path}"` | Its text inside a longer string |
| `"$a.ids + $b.ids"` | Joined into one array. `["$a.ids", "$b.ids"]` works too |
| `$$` | A literal `$`, so a node path is `"$$Player"` |
| `$project` | The open Godot project's folder. An error without a project |
| `$script_dir` | The script file's folder |

A `$` that cannot start a reference, like `"$5"`, stays as it is. A name no step saved is an error, so typos never pass
silently. Ids arriving as text, such as `"42"`, are accepted wherever a node id is expected.

## Ids after CSG

CSG and clip steps replace brushes and return `replaced`. The runner then rewrites the ids in every earlier saved
result, so `"$wall.ids"` keeps meaning the wall after a window is cut into it.

| Saved value | After replacement |
| --- | --- |
| An id in a list | Gives way to all its replacements, or drops out when it has none |
| A single id | Its one replacement, a list when there are several, null when removed |
| Other numbers, `vars` | Never touched |

This applies to values under `id`, `ids`, `selection`, `letters`, `copies`, `entity` and keys ending in `_id` or
`_ids`.

> **Tip:** Scatter and blend calls should spell out every brush setting. Anything a call leaves out is filled in from
> the Scatter panel's current settings.
