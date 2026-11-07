---
source: c998bca32c1c4bd85f0c95739d78a13d4ede986a
---

# Live Link 协议 { #live-link-protocol }

Live Link 是 TrenchDot 和 Godot 编辑器互相通信的方式。插件会在 Godot 编辑器里运行一个小型服务器，TrenchDot 连上它，用来在你保存时触发重新构建、推送[实时模式](live.md)的编辑，并显示 Godot 是否在运行。

你很少会用到这一页，插件启用后这条链接自己就能工作。它是写给这些情况的：你想自己写一个以同样方式驱动 Godot 的工具，或者想查清连接为什么失败。如果保存后没有重新构建，请先看[保存时重建](live.md#rebuild-on-save)，状态栏通常会告诉你原因。

## 传输 { #transport }

通过 TCP 在 `127.0.0.1:7842` 上传输以换行分隔的 JSON。每条消息有一个 `event` 和一个可选的 `seq`，并且恰好得到一行回复，回复会回显 `seq` 并带有 `ok`。失败时回复 `ok: false` 和一个 `error`。编辑器保持一条连接打开，并每秒发送一次 `status` 心跳。

要换一个端口，两边都要改：Godot 项目设置里的 `trenchdot/live_link_port`，以及 TrenchDot 里 *文件 > 偏好设置* 中的 *Godot Live Link*。

正在运行的游戏里的[热重载](live.md#hot-reload-in-a-running-game)自动加载监听 `127.0.0.1:7843`，只回应 `map_saved`。

## 事件 { #events }

客户端发送这些事件。“回复”一列列出了 Godot 除 `ok` 和 `seq` 之外回复的字段，或者它会做什么。

| 事件 | 字段 | 回复 |
| --- | --- | --- |
| `hello` | | `project` 和 `godot`（版本），用来快速检查链接有没有回应 |
| `status` | | `project`、`godot`（版本）、`pid`、`scene`（Godot 当前显示的场景标签页）以及它的 `maps: [{path, epoch}]` |
| `map_saved` | `path` | `rebuilt`（根据该文件重新构建的地图节点数）、`scene`、`waiting`，以及这次构建的 `missing_classes`、`settings` 和 `warnings` |
| `build` | `path`、`text` | 和 `map_saved` 一样，只是根据 `text` 而不是文件来构建 |
| `focus` | | 把 Godot 窗口带到最前面 |
| `export_game_config` | | 写出 `trenchdot_game.json` |
| `scan` | | 开始扫描项目文件。扫描完成后，每个实时会话都会检查自己的实体定义 |
| `play` | | 保存已打开的场景并运行主场景，项目没有主场景时回复 `ok: false` 和一个 `error` |
| `live_begin` | `path`、`text`、`content` | `epoch` 和 `unresolved` |
| `live_resync` | `path`、`text` | `epoch` 和 `unresolved` |
| `live_delta` | `path`、`ops` | `epoch` 和 `unresolved`，操作对不上时回复 `ok: false, resync: true` |
| `live_end` | `path`、`revert` | 带 `revert` 时，根据已保存的文件重新构建 |
| `inspect` | `path`、`ids` | `nodes`，每个地图节点 id 对应名称、类、子节点数和位置，外加 `live` 和 `pending` |
| `capture` | `path`、`camera`、`width`、`height`、`text` | `png`（base64）、`width`、`height` 和 `warnings`，即构建时记录的错误和警告 |
| `walkable` | `path`、`agent`、`text` | `vertices`（扁平的 x、y、z 列表）、`polygons`、`islands`（多边形索引，最大的在前）、`units_per_meter`、`agent` 和 `msec` |

`capture` 渲染的是第一个使用 `path` 的地图节点的世界。`camera` 包含以地图单位和坐标轴表示的 `position` 和 `forward`，以及 `fov`，即以度为单位的水平视野。带 `text` 时，Godot 会先构建那张地图，否则它会等实时会话追上来。回复大约在十几帧之后到达，期间其他消息照常应答。

`walkable` 用 `TrenchDotNav` 针对 `agent`（`radius`、`height`、`max_climb`、`max_slope`，单位分别是米和度）烘焙整个正在编辑的场景的导航网格，并以地图的单位和坐标轴返回。带 `text` 时，Godot 会先构建那张地图，除非它上一次就是根据同样的文本构建的。

路径是带正斜杠的绝对路径。只有正在编辑的场景（即 Godot 显示的场景标签页）里开启了 *Auto Rebuild On Save* 的 `FuncGodotMap` 节点会参与。`scene` 是它的 `res://` 路径，没有打开场景时为空。

`waiting` 列出其他使用这张地图的已打开场景标签页。`map_saved` 之后，它们会在标签页显示时构建该文件，`build` 之后则不会。对预制体的 `map_saved` 还会重新构建正在编辑的场景里直接或通过其他预制体实例化它的那些地图，并把它们计入 `rebuilt`。

`map_saved` 或 `build` 的构建所报告的内容分成三个列表，只有不为空时才会出现：`missing_classes` 是没有任何定义描述过的类名，按排序给出；`settings` 是这些构建用过的地图设置的路径；`warnings` 是构建报告里最多 20 条单行警告。实时回复里的 `unresolved` 列出那一步在没有定义的情况下构建出来的类名，按排序给出，同样只在有的时候才出现。

`scan` 适合在 Godot 之外写入的文件，比如安装的实体包。如果某个实时会话的场景是用与扫描后的地图设置不同的实体定义构建的，它会再次构建自己的地图。没有 `scan` 或 `play` 的旧版插件会把它们当作未知事件来回应。

`text` 是整张地图的 JSON，当地图有烘焙时，烘焙光照放在 `lightmap` 下，这样实时构建就能保留它。这会让每个图集纹素多出约 11 字节，1024 乘 1024 的图集大约是 11 MB，所以编辑器只在会话开始、新的烘焙之后或大改动之后才再次发送整张地图。

`content` 是已保存文件的 END 块里的内容 id，所以根据未改动文件构建的场景，在会话开始时不会再被构建一次。

端口被占用时，通常是第二个打开同一项目的 Godot 编辑器，插件会在输出面板里警告一次，并每三秒再试一次这个端口，所以另一个编辑器关闭后它就会接手。

## 实时操作 { #live-ops }

| 操作 | 字段 | 含义 |
| --- | --- | --- |
| `set` | `id`、`parent`、`index`、`node` | 插入或替换一个节点，采用 `.tdm` 格式的 JSON 形式，不含子节点 |
| `remove` | `id` | 删除一个节点及其子树，只发送最上层被删除的 id |
| `translate` | `ids`、`offset` | 移动节点，拖动时发送它来代替几何体 |
| `properties` | `properties` | 替换 worldspawn 属性 |

只要地图在实时会话之外被构建，或者使用它的 `FuncGodotMap` 节点发生变化，地图的 `epoch` 就会增加。下一次 `status` 显示新的 epoch 时，就用 `live_begin` 重新开始。

生成的节点在 `_td_id` 元数据里带着它们的地图节点 id，图层和分组节点则在 `_td_group` 里。

## 不开窗口测试 { #testing-without-a-window }

这条链接也能配合无头的 Godot 编辑器使用，适合自动化测试。打开一个带有 `FuncGodotMap` 的场景，然后向 7842 端口发送 `inspect`，就能看到 Godot 构建出了什么：

```sh
godot --headless --editor --path godot res://path/scene.tscn
```

（注意：在刚检出的仓库里，先运行两次 `godot --headless --path godot --import`。只运行一次只会注册插件的类。）

无头的 Godot 什么都不绘制，所以 `capture` 在那里会回复一个错误。要测试截图，请用带窗口的编辑器。

插件测试就是这样做的，见 [`godot/tests/editor/live_link_tabs.gd`](https://github.com/Paraxdev/TrenchDot/blob/main/godot/tests/editor/live_link_tabs.gd)，它在无头编辑器里打开、关闭、重新加载并切换场景标签页，并检查链接报告了什么。

<div class="td-mcp" markdown>

## MCP { #mcp }

[MCP 服务器](../mcp.md)让智能体和脚本可以驱动编辑器，它的一些工具会经过 Live Link。当你要求从 Godot 取一张图时，`screenshot` 工具会使用 `capture`。

如果无头测试还需要编辑地图，就让 TrenchDot 带着它的 MCP 服务器，在与无头 Godot 编辑器相同的项目上启动，然后通过 MCP 编辑地图：

```sh
trenchdot --mcp-http --project godot
```

</div>
