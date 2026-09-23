# JSON layout

Maps saved before the binary container are JSON text, and the same JSON is what `--dump`, `--to-json`, a `.json` save,
the clipboard, the live link and the MCP tools use. Readers treat any file that does not start with `\x89GTM` as JSON,
with or without a UTF-8 byte order mark. Old maps load as they are and are saved in the binary format from then on.

The printer in [`json_fmt.rs`](https://github.com/Paraxdev/GodotTrench/blob/main/crates/gt_doc/src/json_fmt.rs) keeps
diffs readable. It indents by two spaces, keeps keys in the order the tables in these pages list them, and puts short
arrays and objects on one line. Every face is one line, and so is a `vertices` array of up to 64 entries, so moving one
brush changes only that brush's lines. Any valid JSON with the same content loads the same.

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

The floor is a 640 by 640 unit slab 8 units thick whose top sits at Y 8. Its first face, `[0,1,2,3]`, is the +X side,
with corners that run counter-clockwise seen from +X. The barrel is a point entity 16 units above the floor, and when it
breaks it calls `run` on `hp_readout`.
