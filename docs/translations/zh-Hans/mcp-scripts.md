---
source: ac40f8ef73c25c546d11381f2a654f03d847b7aa
---

# MCP 脚本 { #mcp-scripts }

MCP 脚本是一个 JSON 文件，里面是一串 [MCP](mcp.md) 工具调用，`run_script` 会按顺序重放它们。它把一张地图是怎么建出来的保存成可读的步骤，所以你可以从头重建地图，也可以在拉取请求里把它当文本来审阅，测试同样能重放它。

[examples/mcp](https://github.com/Paraxdev/TrenchDot/tree/main/examples/mcp) 里的展示地图就是这样建的，它们也是每个工具的示例。

| 脚本 | 建出什么 |
| --- | --- |
| `mountain_house.json` | 悬在悬崖上的小木屋，带一部悬崖升降机和几片森林 |
| `church_school.json` | 一座教堂和一所学校，三个按钮能打开庭院的大门 |
| `lighthouse_forest.json` | 一座带旋转灯塔灯的灯塔，一艘在 `func_train` 上的船，还有不断生成的鬼火 |
| `withered_city.json` | 夜里一座破败的霓虹城市，有闪电、辉光和 `ssr` |
| `night_district.json` | 夜里的一条工业街，有一扇卷帘门和一列高架桥上的火车 |
| `sea_island.json` | [海岛教程](tutorials/sea-island.md) |
| `scripted_scene.json` | [过场动画教程](tutorials/cutscene.md) |
| `logic_playground.json` | 一个给[数学与逻辑节点](gameplay/entities/logic.md)用的大厅：金币、分数、金库门上的一道门和一场灯光抽奖 |

## 运行它们 { #running-them }

用演示项目和它的 HTTP 服务器启动编辑器，见[启动](mcp.md#starting-it)，然后用 `tools/mcp_script.py` 重放脚本。如果编辑器要求[令牌](mcp.md#the-http-token)，用 `--token` 或 `TRENCHDOT_MCP_TOKEN` 传给它。它的 `shot` 命令会从你给的相机位置保存一张截图，`--shade` 选择着色方式：

```sh
trenchdot --mcp-http --project godot                        # the editor with the demo project
python tools/mcp_script.py run examples/mcp/sea_island.json   # replay a script
python tools/mcp_script.py shot view.png --pos 4300,900,3300 --look 2200,700,1200 --shade lit
```

脚本都保存到 `godot/demo/maps/showcase`，只有 `scripted_scene.json` 和 `logic_playground.json` 例外，它们保存到 `godot/demo/maps`。

除非你传 `--continue-on-error`，否则运行会在第一个失败的步骤处停下，`--url` 让运行器指向另一个端口上的编辑器。每次运行在它改动的每张地图里都是一步撤销，名称取自脚本的 `label` 或文件名。

## 脚本格式 { #script-format }

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

`vars` 为你自己的变量设置默认值。传给 `run_script` 的 `vars` 对象会覆盖它们，所以一个脚本可以保存到不同的位置，以这种方式传入的文件夹，脚本可以从中读取，也可以写入。

没有 `tool` 的步骤是注释。带 `save` 的步骤会把结果存在这个名字下，这样后面的步骤就能用它创建的 id。

脚本和地图一样会被分享，所以它们的步骤只能编辑和保存地图。步骤不能打开另一个项目或它的地图、启动 Godot、在其中构建、打开实时模式、关闭 MCP、复制或剪切到剪贴板、丢弃未保存的改动，也不能关闭其他标签页。撤销和重做只会触及脚本自己做过的事，所以你没保存的工作会留在每个标签页里。`screenshot`、`simulate_input`、`walkability` 和 `run_script` 同样不能在脚本里运行，`project_content` 除 `gameplay` 之外的安装也不行。

步骤读取或写入的每个文件，都必须在打开的项目、脚本自己的文件夹或你在 `vars` 里传入的文件夹中，即使其中的链接指向别处也一样。这包括它打开、检查、比较、保存和导出的地图，它导入的地图、模型、预制体和纹理以及它们所指名的文件，以及转换、导入、导出和预制体库写出的一切。地图或模型在别处指名的预制体、模型文件、纹理和 Hammer 实例会被排除在外，网络路径在编辑器查看之前就会被拒绝，不带路径的保存只会写回脚本启动时打开的那张地图。

这条规则同样适用于材质文件、它们的图像和热点文件、预制体库和自然预设。纹理的文件如果在这些文件夹之外，在脚本里就会被当作缺失，所以对它做适配或对齐时会退回默认的纹理尺寸。摘要随后会在 `unread_textures` 里列出读不到的文件。

被拒绝的步骤会连同原因出现在摘要的 `errors` 里。摘要把工具调用计为 `steps`，把注释计为 `notes`。

`args` 里的字符串可以引用保存的结果和几个内置变量：

| 写法 | 含义 |
| --- | --- |
| `"$name.path"` | 保存的 JSON 值。数字用来索引数组，比如 `$room.ids.0` |
| `"${name.path}"` | 它在较长字符串里的文本 |
| `"$a.ids + $b.ids"` | 两者合并成一个数组。`["$a.ids", "$b.ids"]` 也行 |
| `$last` | 上一步的结果 |
| `$project` | 打开的 Godot 项目的文件夹，没有打开项目时是错误 |
| `$script_dir` | 脚本文件所在的文件夹 |
| `$$` | 字面的 `$`，所以节点路径写成 `"$$Player"` |

未知的名称是错误。不能作为名称开头的 `$`，比如 `"$5"`，保持原样。以文本形式给出的 id，比如 `"42"`，在任何需要节点 id 的地方都可以用。

不带花括号的 `$name/rest` 不会展开，所以不带花括号的 `$project/maps` 是错误，而不是一个没人想写的路径。请用 `"${project}/maps"`。

## CSG 之后的 id { #ids-after-csg }

CSG 和裁切步骤会把被切的笔刷换成有新 id 的新笔刷。运行器会改写前面步骤保存的 id 来对应，所以在墙上切出一扇窗之后，`"$wall.ids"` 仍然指整面墙，被移除的笔刷则会从中消失。

只有带节点 id 的键下的值才会被改写：`id`、`ids`、`selection`、`selected`、`nodes`、`letters`、`copies`、`entity`、`group`、`parent`、`insert_parent`、`current_layer`、`open_groups`、`targets`，以及以 `_id` 或 `_ids` 结尾的键。像 `hierarchy reparent` 返回的 `moved` 这样的计数保持不变，请改用它的 `ids`。

> [!TIP]
> 在 `scatter` 和 `blend` 调用里把每项笔刷设置都写明。省略的任何设置都取自编辑器当前的散布面板或混合工具的设置。
