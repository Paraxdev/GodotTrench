---
source: 92464837c8b6e8506e85d0270bb18896476cd41b
---

# JSONの構造 { #json-layout }

JSON形式は、古いエディターが保存していた形式です。`--dump`、`.json`での保存、クリップボード、Live Linkも、これを使います。

リーダーは、`\x89GTM`で始まらないファイルをすべてJSONとして扱います。UTF-8のバイトオーダーマークは、あってもなくてもかまいません。古いJSONのマップはそのまま読み込まれ、次に`.tdm`として保存したときにバイナリになります。

[`json_fmt.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/json_fmt.rs)は、差分が読みやすいように出力します。2スペースでインデントし、キーはこれらのページが挙げる順に並べ、短い配列とオブジェクトは1行にまとめます。面は1つにつき1行で、64個までの`vertices`配列も1行なので、ブラシを1つ動かしても、変わるのはそのブラシの行だけです。

自分のツールがこのレイアウトをまねる必要はありません。内容が同じで有効なJSONなら、どれも同じように読み込まれます。

## 変換 { #converting }

| コマンド | 結果 |
| --- | --- |
| `trenchdot --dump map.tdm` | マップをJSONとして出力する |
| `trenchdot --to-json map.tdm map.json` | マップをJSONとして書き出す |
| `trenchdot --to-tdm map.json map.tdm` | バイナリのマップを書き出す。エディターが開けないファイルは拒否する |

変換は、ファイルの内容をエディターに読み込まずにコピーするので、エディターが知らないキーも残ります。

エディターは`.json`のマップも開けますし、*別名で保存*で`.json`の名前を付けるとJSONが書き出されます。バイナリのマップのJSON差分をgitで見る方法は、[gitでのマップファイル](../development.md#map-files-in-git)を参照してください。

## クリップボード { #clipboard }

コピーと貼り付けは、別のラッパーで同じノードの形を使います。

```json
{
  "format": "trenchdot-clipboard",
  "version": 1,
  "nodes": [ ... ]
}
```

`nodes`は、コピーしたサブツリーを、親を除いた形で持ちます。貼り付けは、読み込みと同じチェックを行い、すべてのノードに新しいidを付け、`label`、`hidden`、`locked`を保ちます。コピーしたレイヤーは、その子が貼り付けられます。

Live Linkは、`children`なしで、同じ形の単一のノードを送ります。

## 例 { #example }

`godot/demo/maps/scripted_scene.tdm`を`trenchdot --dump`が出力する形で抜粋し、短くしたものです。床のブラシと、爆発する樽が入っています。

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

床は、640 x 640ユニットで厚さ8ユニットの板で、上面はY 8にあります。最初の面`[0,1,2,3]`は+X側で、+Xから見て反時計回りに並んでいます。

<div class="td-mcp" markdown>

## MCP { #mcp }

[MCPサーバー](../mcp.md)のツールも、このJSON形式でノードを読み、返します。

</div>
