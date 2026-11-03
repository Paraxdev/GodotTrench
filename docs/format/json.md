# JSON layout

The JSON form is what older editors saved. It's also what `--dump`, a `.json` save, the clipboard and the live link
use.

Readers treat any file that doesn't start with `\x89GTM` as JSON, with or without a UTF-8 byte order mark. An old JSON
map loads as it is and turns binary on its next `.tdm` save.

[`json_fmt.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/json_fmt.rs) prints it so diffs
stay readable. It indents by two spaces, keeps keys in the order these pages list them, and puts short arrays and
objects on one line. Every face is one line, and so is a `vertices` array of up to 64 entries, so moving one brush
only changes that brush's lines.

Your own tool doesn't have to copy this layout, any valid JSON with the same content loads the same.

## Converting

| Command | Result |
| --- | --- |
| `trenchdot --dump map.tdm` | Prints the map as JSON |
| `trenchdot --to-json map.tdm map.json` | Writes the map as JSON |
| `trenchdot --to-gtm map.json map.tdm` | Writes a binary map, refusing a file the editor could not open |

The conversions copy the file content without loading it into the editor, so they keep keys the editor doesn't know.

The editor opens `.json` maps too, and *Save As* with a `.json` name writes JSON. For JSON diffs of binary maps in git,
see [Map files in git](../development.md#map-files-in-git).

## Clipboard

Copy and paste use the same node shape in another wrapper:

```json
{
  "format": "trenchdot-clipboard",
  "version": 1,
  "nodes": [ ... ]
}
```

`nodes` holds the copied subtrees without their parents. Pasting runs the same checks as loading, gives every node a
fresh id and keeps `label`, `hidden` and `locked`. A copied layer pastes its children.

The live link sends single nodes in the same shape, without `children`.

## Example

A trimmed excerpt of `godot/demo/maps/scripted_scene.tdm` as `trenchdot --dump` prints it, with the floor brush and
an explosive barrel:

```json
{
  "format": "trenchdot-map",
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

<div class="td-mcp" markdown>

## MCP

The tools of the [MCP server](../mcp.md) read and return nodes in this JSON form too.

</div>
