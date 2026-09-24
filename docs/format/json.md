# JSON layout

The JSON form is what older editors saved. It is also what `--dump`, a `.json` save, the clipboard and the live link
use. Readers treat any file that does not start with `\x89GTM` as JSON, with or without a UTF-8 byte order mark. An old
JSON map loads as it is and becomes binary on its next `.gtm` save.

[`json_fmt.rs`](https://github.com/Paraxdev/GodotTrench/blob/main/crates/gt_doc/src/json_fmt.rs) prints it so that
diffs stay readable. It indents by two spaces, keeps keys in the order the tables in these pages list them, and puts
short arrays and objects on one line. Every face is one line, and so is a `vertices` array of up to 64 entries, so
moving one brush changes only that brush's lines. Any valid JSON with the same content loads the same, so a tool that
writes JSON does not have to copy this layout.

## Converting

| Command | Result |
| --- | --- |
| `godottrench --dump map.gtm` | Prints the map as JSON |
| `godottrench --to-json map.gtm map.json` | Writes the map as JSON |
| `godottrench --to-gtm map.json map.gtm` | Writes a binary map, refusing a file the editor could not open |

The conversions copy the file content without loading it into the editor, so they keep keys the editor does not know.
The editor also opens `.json` maps, and *Save As* with a `.json` name writes JSON. To see JSON diffs of binary maps in
git, see [Map files in git](../development.md#map-files-in-git).

## Clipboard

Copy and paste use the same node shape in another wrapper:

```json
{
  "format": "godottrench-clipboard",
  "version": 1,
  "nodes": [ ... ]
}
```

`nodes` holds the copied subtrees without their parents. Pasting runs the same checks as loading, gives every node a
fresh id and keeps `hidden` and `locked`. A copied layer pastes its children. The live link sends single nodes in the
same shape, without `children`.

## Example

A trimmed excerpt of `godot/demo/maps/scripted_scene.gtm` as `godottrench --dump` prints it, with the floor brush and
an explosive barrel:

```json
{
  "format": "godottrench-map",
  "version": 1,
  "properties": {
    "ambient_color": "60 66 82",
    "classname": "worldspawn",
    "message": "Scripted Scene",
    "sky_ground_color": "60 62 70",
    "sky_horizon_color": "120 130 150",
    "sky_top_color": "40 54 92",
    "sun_angles": "-40 -55",
    "sun_energy": "0.6"
  },
  "layers": [
    {
      "id": 1,
      "type": "layer",
      "name": "Default",
      "color": "#7e67e6ff",
      "omit_from_export": false,
      "children": [
        {
          "id": 2,
          "type": "brush",
          "vertices": [[320.0,8.0,-320.0],[320.0,8.0,320.0],[320.0,0.0,320.0],[320.0,0.0,-320.0],[-320.0,8.0,320.0],[-320.0,8.0,-320.0],[-320.0,0.0,-320.0],[-320.0,0.0,320.0]],
          "faces": [
            {"indices":[0,1,2,3],"material":"showcase/cobble","uv":{"u_axis":[0.0,0.0,-1.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[4,5,6,7],"material":"showcase/cobble","uv":{"u_axis":[0.0,0.0,1.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[1,0,5,4],"material":"showcase/cobble","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,0.0,1.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[3,2,7,6],"material":"showcase/cobble","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,0.0,1.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[2,1,4,7],"material":"showcase/cobble","uv":{"u_axis":[1.0,0.0,0.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}},
            {"indices":[6,5,0,3],"material":"showcase/cobble","uv":{"u_axis":[-1.0,0.0,0.0],"v_axis":[0.0,-1.0,0.0],"offset":[0.0,0.0],"scale":[1.0,1.0],"rotation":0.0}}
          ]
        },
        {
          "id": 17,
          "type": "entity",
          "classname": "prop_physics",
          "origin": [0.0,24.0,176.0],
          "angles": [0.0,0.0,0.0],
          "properties": {
            "explosion_damage": "25",
            "explosion_radius": "256",
            "explosive": "1",
            "health": "5",
            "model": "res://demo/scenes/barrel.tscn",
            "size": "16 24 16",
            "targetname": "barrel"
          },
          "outputs": [{"output":"broken","target":"hp_readout","input":"run"}]
        }
      ]
    }
  ]
}
```

The floor is a 640 by 640 unit slab 8 units thick with its top at Y 8. Its first face, `[0,1,2,3]`, is the +X side,
wound counter-clockwise seen from +X.

{% mcp %}

## MCP

The tools of the [MCP server](../mcp.md) read and return nodes in this JSON form too.

{% endmcp %}
