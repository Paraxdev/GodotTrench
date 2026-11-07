---
source: 92464837c8b6e8506e85d0270bb18896476cd41b
---

# JSON 结构 { #json-layout }

JSON 形式是旧版编辑器保存的格式。`--dump`、保存成 `.json`、剪贴板和 Live Link 用的也是它。

读取器把任何不以 `\x89GTM` 开头的文件都当作 JSON，带不带 UTF-8 字节顺序标记都行。旧的 JSON 地图原样加载，下一次保存成 `.tdm` 时就变成二进制。

[`json_fmt.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/json_fmt.rs) 在输出时让差异保持易读。它按两个空格缩进，键的顺序和这几页列出的一致，短的数组和对象放在一行里。每个面占一行，最多 64 个元素的 `vertices` 数组也占一行，所以移动一个笔刷只会改动那个笔刷的几行。

你自己的工具不必照搬这种布局，内容相同的任何有效 JSON 加载结果都一样。

## 转换 { #converting }

| 命令 | 结果 |
| --- | --- |
| `trenchdot --dump map.tdm` | 把地图输出为 JSON |
| `trenchdot --to-json map.tdm map.json` | 把地图写成 JSON |
| `trenchdot --to-tdm map.json map.tdm` | 写出二进制地图，编辑器打不开的文件会被拒绝 |

转换直接复制文件内容，不会把它加载进编辑器，所以编辑器不认识的键也会保留下来。

编辑器也能打开 `.json` 地图，用 *另存为* 并取 `.json` 的名字就会写出 JSON。想在 git 里对二进制地图做 JSON 差异比较，见[git 里的地图文件](../development.md#map-files-in-git)。

## 剪贴板 { #clipboard }

复制和粘贴用的是同样的节点结构，只是外面包了另一层：

```json
{
  "format": "trenchdot-clipboard",
  "version": 1,
  "nodes": [ ... ]
}
```

`nodes` 存放被复制的子树，不含它们的父节点。粘贴时会做和加载一样的检查，给每个节点一个新的 id，并保留 `label`、`hidden` 和 `locked`。复制一个图层，粘贴出来的是它的子节点。

Live Link 以同样的结构发送单个节点，不带 `children`。

## 示例 { #example }

下面是 `godot/demo/maps/scripted_scene.tdm` 经 `trenchdot --dump` 输出后的一段精简摘录，包含地板笔刷和一个爆炸桶：

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

地板是一块 640 乘 640 单位、厚 8 单位的平板，顶面在 Y 8。它的第一个面 `[0,1,2,3]` 是 +X 一侧，从 +X 看去按逆时针绕向。

<div class="td-mcp" markdown>

## MCP { #mcp }

[MCP 服务器](../mcp.md)的工具也用这种 JSON 形式读取和返回节点。

</div>
