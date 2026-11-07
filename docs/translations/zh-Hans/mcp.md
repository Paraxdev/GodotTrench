---
source: d5a662c31724dba93b77bd318ab3a570c43e95b6
---

# MCP 服务器 { #mcp-server }

编辑器内置了一个 [MCP](https://modelcontextprotocol.io) 服务器。像 Claude Code 这样的代理，或者一个普通的 Python 脚本，都可以用它来检查和编辑地图、截图，以及向界面发送输入。

你可以用它让 AI 助手搭建或审阅关卡，用脚本完成重复的编辑，或者自动测试编辑器。服务器在你启动它之前是关闭的，并且只在你自己的机器上监听。

## 启动它 { #starting-it }

| 传输方式 | 如何启动 |
| --- | --- |
| HTTP | 带 `--mcp-http` 启动编辑器会用 7841 端口，用 `--mcp-http=PORT` 则用别的端口。想让它每次都自动启动，请打开 *文件 > 偏好设置*，进入 *高级*，勾选 *MCP 服务器（HTTP，本机）* 旁边的 *启动时启用*，设置旁边的端口，然后重启 |
| stdio | 带 `--mcp` 启动。客户端每次会话都自己启动编辑器，并通过标准输入和输出与它通信 |

如果你一直开着编辑器、自己在里面工作，同时让代理帮忙，就选 HTTP。如果客户端应该拥有自己的编辑器，或者有其他人也在用同一台电脑，就选 stdio。要把 Claude Code 连到其中任意一种：

```sh
claude mcp add --transport http trenchdot http://127.0.0.1:7841/mcp   # HTTP
claude mcp add trenchdot -- /path/to/trenchdot --mcp                 # stdio
```

仓库里的 `.mcp.json` 已经让 Claude Code 指向 HTTP 服务器。*文件 > 偏好设置* 会显示正在运行的那个服务器的命令，*复制命令* 会把它放进剪贴板。

在别的端口上启动了编辑器？设置 `TRENCHDOT_MCP_URL`，[tools/mcp_client.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_client.py) 和 [tools/mcp_script.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_script.py) 就会改为连到那里。

### HTTP 令牌 { #the-http-token }

你电脑上的任何程序都能访问 HTTP 服务器，登录了这台电脑的其他人也可以。要把他们挡在外面，请在 *文件 > 偏好设置* 的 *高级* 下勾选 *要求令牌*。从下一个请求开始，服务器只会回应以 `Authorization: Bearer <token>` 形式带上它的令牌的请求。偏好设置里的命令随后会包含这个令牌，请用它来代替 `.mcp.json`。stdio 不需要令牌，只有启动编辑器的客户端才能和它通信。

你第一次需要令牌时，编辑器会生成一个随机令牌，并把它保存在 `mcp-token` 里，这是只有你能读取的文件，放在它的设置旁边：Windows 上是 `%APPDATA%\TrenchDot\data`，Linux 上是 `~/.local/share/trenchdot`，macOS 上是 `~/Library/Application Support/TrenchDot`。带 `--default-prefs` 的运行会把自己的令牌保存在临时文件夹的 `trenchdot-default-prefs.mcp-token` 里。

想为某一次运行自己指定令牌，可以设置 `TRENCHDOT_MCP_TOKEN`，或者用 `--mcp-token=<token>` 启动，它优先于这个变量。两者都会无视偏好设置而启用令牌，*要求令牌* 会显示为已勾选且置灰。`tools/mcp_client.py` 和 `tools/mcp_script.py` 只有在你用 `--token` 或 `TRENCHDOT_MCP_TOKEN` 给了它们令牌时才会发送。

## 关闭 MCP { #turning-mcp-off }

想把代理和脚本完全挡在编辑器之外，请在 *文件 > 偏好设置* 里 MCP 部分的顶部取消勾选 *MCP*。HTTP 服务器会立刻关闭并断开已打开的连接，stdio 上的客户端对每个请求都会收到错误，其他 MCP 设置会变灰。这个设置会立即保存，所以你之后启动的每个不带 `--default-prefs` 的编辑器都会保持 MCP 关闭，直到你再次勾选它，勾选后 HTTP 服务器会在它的端口上重新启动。

![偏好设置的 MCP 部分：MCP 已勾选，HTTP 服务器一行带着它的端口，以及要求令牌](assets/mcp/prefs-mcp.png)

想让某一次运行无视偏好设置而关闭 MCP，请把 `TRENCHDOT_DISABLE_MCP` 设为除空值和 `0` 以外的任何值。这时 *MCP* 会显示为未勾选且置灰。MCP 关闭期间，`--mcp` 和 `--mcp-http` 不会启动服务器。编辑器仍然会打开，它的状态行会说明 MCP 已关闭，stdio 上的客户端会收到一个说明这一点的错误，而不是干等。

代理也可以关闭 MCP，用 `set_editor` 加 `"mcp": false`。只有你能把它重新打开。

## 工具 { #tools }

| 领域 | 用途 | 工具 |
| --- | --- | --- |
| 检查 | 读取地图、编辑器状态、实体定义，以及如何在游戏代码里使用某个实体，不做任何修改 | `get_state`, `summarize_map`, `list_nodes`, `get_node`, `changes_since`, `validate_map`, `get_game_config`, `code_reference` |
| 创建 | 添加笔刷、网格、地形、实体和导入的模型，并沿面的边缘修剪 | `create_brush`, `create_mesh`, `create_terrain`, `create_entity`, `import_model`, `trim` |
| 编辑 | 修改已有的东西：实体键、选择、放置、面、纹理、地形、图层和分组，以及 worldspawn | `update_entity`, `select`, `transform`, `duplicate`, `set_face`, `mesh_edit`, `texture`, `terrain_edit`, `hierarchy`, `set_map_properties`, `bake_lighting` |
| 绘制和玩法 | 散布模型，混合材质，从预制体库里粘贴部件，并把笔刷变成门、平台和按钮 | `scatter`, `blend`, `prefab_library`, `random_fill`, `gameplay` |
| 编辑器 | 运行菜单操作，打开和保存文件，修改编辑器设置，移动摄像机，截图，发送输入，并检查玩家能在哪里行走 | `run_action`, `map_file`, `open_project`, `set_editor`, `set_camera`, `screenshot`, `simulate_input`, `walkability` |
| 项目 | 把玩法实体包、自然模型或演示添加到打开的 Godot 项目里 | `project_content` |
| 脚本 | 重放一份保存好的工具调用列表，参见 [MCP 脚本](mcp-scripts.md) | `run_script` |

每个工具的描述都列出了它的操作和参数，所以代理可以直接从服务器那里学会它们。[tools/mcp_client.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_client.py) 用 `list` 把它们打印出来，并能从 shell 里调用工具：

```sh
python tools/mcp_client.py call screenshot '{"target":"3d","width":1280,"height":720}' --out shot.png
```

## 截图 { #screenshots }

`screenshot` 会返回整个编辑器窗口的 PNG，或者当 `target` 是 `3d`、`top`、`front` 或 `side` 时返回其中一个视图。对于视图，其他参数决定你得到什么：

| 参数 | 结果 |
| --- | --- |
| 无 | 整个编辑器窗口 |
| 只有 `target` | 保持原样的那个停靠视图，包含叠加层 |
| `width` 和 `height` | 离屏渲染，最大 4096 像素，不带网格、实体框、操纵器或选择 |
| `width`、`height` 和 `"overlays": true` | 离屏渲染，带编辑器叠加层 |
| `"overlays": false` | 停靠大小的干净截图 |

### Godot 渲染的内容 { #what-godot-renders }

`"source": "godot"` 会请 Godot 编辑器用它真实的灯光、环境、雾和后期处理来渲染地图。你会拿回那张 PNG，外加 Godot 在构建过程中记录的所有错误或警告。想在不启动游戏的情况下判断光照和材质，这很方便。

```sh
python tools/mcp_client.py call screenshot '{"source":"godot","position":[0,256,512],"look_at":[0,64,0]}' --out godot.png
```

1. 把地图保存在 TrenchDot 里打开的那个项目里面。
2. 在 Godot 编辑器里打开那个项目，用一个其 `FuncGodotMap` 会构建这张地图的场景。
3. 用 `"source": "godot"` 调用 `screenshot`。不带 `position` 和 `look_at` 时，它会使用 3D 视图的摄像机。

没有[实时模式](godot/live.md)时，Godot 会先按 TrenchDot 里显示的样子构建地图，就像 *在 Godot 中构建* 一样。`"build": false` 会按 Godot 现有的样子捕获场景。`width` 和 `height` 默认是 1280 x 720，`fov` 默认取 3D 视图的值。

> [!NOTE]
> Godot 需要一个窗口才能渲染，无头模式的 Godot 会返回错误而不是图像。这个窗口可以在后台或最小化。

## 读取和审阅地图 { #reading-and-reviewing-a-map }

| 步骤 | 工具 |
| --- | --- |
| 获得概览 | `summarize_map` 列出图层、实体类、I/O 连接和材质，以及所有缺失的资源 |
| 找节点 | `list_nodes` 可以按 `layer`、`material`、`property`（`{"model": "*crate*"}`）或地图中的 `box` 筛选 |
| 看某一个 | `get_node` 返回一个节点。`compact: true` 会省去顶点和面 |
| 显示你改了什么 | `changes_since` 会和上次保存、往回 `undo_steps` 步，或者某个地图 `file` 作比较 |
| 撤回 | 每次调用和每个 `run_script` 都是一个名为 `MCP: ...` 的撤销步骤。带 `steps` 的 `run_action undo` 可以撤销多步 |

想把地图放进 git 来管理，参见[审阅更改](editor/reviewing.md)。

## 值得了解的行为 { #behavior-worth-knowing }

| 主题 | 行为 |
| --- | --- |
| 浏览器 | HTTP 服务器会拒绝任何带 `Origin` 头的请求，以及任何不是 `application/json` 的 POST，所以网页访问不到它，连从 localhost 提供的网页也不行 |
| 错误 | 没能完成任务的工具会返回 `isError`，失败的 `run_action` 也一样 |
| 文件对话框 | 会打开对话框的操作会被拒绝，请用带路径的 `map_file`。`run_action save` 需要一张已经有文件的地图 |
| 未保存的标签页 | `run_action close_tab` 会拒绝丢弃未保存的更改，除非 `args.discard` 为 true。`args.close_others` 则会关闭除当前标签页以外的所有标签页 |
| 文件路径 | `map_file`、`run_script`、`import_model`、`changes_since` 和 `validate_map` 里的 `res://` 和相对路径从打开的项目开始算，没有项目时从编辑器的工作目录开始算。`map_file open` 和 `open_tab` 会在打开之前把路径解析成绝对路径，所以之后的 `save` 不会落到别的地方 |
| 编辑器设置 | `transform` 以及 `run_action rotate` 和 `flip` 以选区中心为轴，只有在吸附开启时才吸附到网格，并且在 UV 锁定开启时保持纹理锁定。`transform` 接受 `snap` 和 `uv_lock` 来为一次调用覆盖这一点，`duplicate` 接受 `uv_lock`。`run_action sculpt` 和 `blend` 使用人可以修改的工具设置，`terrain_edit` 使用固定的默认值 |
| 选择 | `select` 会拒绝未知的 id，并跳过隐藏或锁定的节点，把它们列在 `skipped` 里 |
| CSG | `csg_subtract`、`csg_merge`、`csg_intersect`、`csg_hollow` 和 `clip_apply` 会返回 `replaced`，把旧 id 映射到新 id |
| 超时 | 调用在 120 秒后超时，`run_script`、`project_content` 和 `bake_lighting` 除外 |
| 内容问题 | 新项目会被问到的关于现成内容的问题，会在第一次调用 `get_state`、`screenshot` 和 `simulate_input` 以外的工具时关闭，并且在那次会话里不会再次打开 |
| 脚本变量 | 当 `name` 是已知变量时，`$name/rest`（不带花括号）是错误的，因为只有单独的 `$name` 或 `${name}/rest` 才会展开。请写成 `${project}/maps` |

## 工具说明 { #tool-notes }

| 工具 | 说明 |
| --- | --- |
| `run_action` `copy`、`cut`、`paste` | 复制和剪切会返回剪贴板文本，粘贴把它当作 `args.text`，可以再带一个 `offset` 或 `origin` |
| `run_action` `clip_apply` | 接受一个平面（`point` 和 `normal`，或三个 `points`）和 `keep`：`front`、`back` 或 `both` |
| `run_action` `create_decal` | 放置一个[贴花](editor/decals.md)，请用它来代替很薄的笔刷 |
| `create_brush` | `openings` 会开出门和窗，`{min, max, count, step, rows, row_step}` 会把其中一个重复成网格 |
| `create_mesh` `rotate` | 以 Godot 的 YXZ 顺序给出 `[x, y, z]` 度数，所以 `[0, 0, 90]` 会让圆柱体沿 X 轴躺下 |
| `transform` | 按平移、旋转、翻转、scale_to 的顺序应用，作为一个撤销步骤。除非另有说明，`rotate` 会绕 Y 轴转 90 度 |
| `create_terrain` | 和其他工具不同，`origin` 是地形的最小角。`center` 则改为把它放在某个点周围 |
| `create_brush`、`create_mesh` | 对笔刷来说，山形屋脊沿 Z 轴延伸，对网格来说沿 X 轴延伸。两个工具都接受 `ridge_x` 和 `ridge_z` |
| `map_file` 导出 | `path` 是写出的文件，为了旧的调用方而保留，`exported` 同样给出它，`map_path` 是打开的地图 |
| `terrain_edit` | 不带 `op` 的 `probe [x, z]` 只读取高度。`clear_layer` 需要 `layer` |
| `validate_map` | 接受目标的 Godot 类或脚本里的任何方法或属性作为输入。`coplanar_faces` 会标出会闪烁的面。每个问题都带有 `bounds`。`path` 会检查一张保存好的地图而不打开它，`project: true` 会检查项目里的每张地图 |
| `run_action` `place_entities` | 接受 `args.classnames`，以及 `at` 和 `normal`，让实体像从实体标签页里拖出来那样落在某个表面上。没有 classnames 会报错 |
| `simulate_input` 拖动 | 带 `"release": false` 的拖动会在调用返回后继续按住按钮和它的修饰键，这样截图就能显示拖动进行中的样子，例如光标旁边的一张卡片。之后某次调用里的 `release` 事件会在指针所在的位置，或者它的 `x` 和 `y`，或者 `world` 处放下它，只带 `x` 和 `y` 之一的 release 是错误。拖动和它的修饰键会一直保持按住，真实键盘上也是，直到那次 release、一次 `new_map` 或成功的 `close_tab` 操作，或者 MCP 关闭，所以千万别忘了 release。在那之前，点击和拖动都会被拒绝 |
| `run_action` `reload_materials` | 立即重新扫描纹理文件夹，不过新文件本来也会被自动发现 |
| `set_editor` `live_link_port` | 与一个其项目设置了另一个 `trenchdot/live_link_port` 的 Godot 编辑器通信 |
| `scatter` `preset` | 添加预设所需的内嵌 Blockbench 模型。在没有自然素材包的项目里使用 glTF 预设会报错，错误里会点名 `project_content` |
| `bake_lighting` | 像[烘焙光照](editor/light-baking.md)一样烘焙，并等待它完成。省略的选项会沿用地图上次烘焙时的设置。`bake: false` 只切换 `view`，`remove: true` 会丢弃烘焙。`out_of_date` 说明地图自烘焙以来是否改动过 |
| `project_content` | `install: "nature"` 或 `"demo"` 会从发布版下载，文件到位后才回应。`install: "gameplay"` 会从项目的插件复制玩法实体包，在脚本里也一样。不带参数时它会说明项目里有什么 |
