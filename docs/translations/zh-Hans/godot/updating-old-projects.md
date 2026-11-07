---
source: 4c71c99f5fabec89f69f0c99554b4f3a60fb4148
---

# 更新 GodotTrench 项目 { #updating-a-godottrench-project }

TrenchDot 以前叫 GodotTrench，测试版有一阵子叫 LevelBench。如果你在那时建过项目或保存过地图，它们仍然能用。你只需要更新一次插件。

## 步骤 { #steps }

1. 启动新的编辑器。第一次启动时，它会把你的偏好设置、信任的项目和最近的文件从旧的设置文件夹复制过来。旧文件夹原封不动，所以旧编辑器也仍然能用。
2. 打开你的 Godot 项目。设置窗口会打开，并告诉你项目里的插件比编辑器旧，即使版本号一样也会这么说。请先关闭 Godot，再点击**将插件更新到**加上版本号。
3. 在 Godot 里打开一次项目。插件会把它的项目设置从 `godottrench/` 移到 `trenchdot/`，并写出 `trenchdot_game.json` 来取代 `godottrench_game.json`。

## 哪些会保留 { #what-carries-over }

| 你原来有 | 会发生什么 |
| --- | --- |
| `.gtm` 地图 | 它们照常打开。状态行会写着“已把 room.gtm 从 GodotTrench 格式更新，请保存以保留”。保存时会保留 `.gtm` 这个名字，所以你的场景仍然找得到这张地图。新地图保存为 `.tdm`。 |
| `GODOTTRENCH_` 环境变量 | 它们仍然有效。两个名字都设置时，以 `TRENCHDOT_` 的为准。 |
| `res://godottrench/entities` 里的玩法包 | 它继续有效，你自己继承它的类的脚本也仍然能编译。如果你同时安装了新的包，两边都定义了某个实体时，以新的为准。只要你的场景还在用旧文件夹，就让它留在原处。 |
| `res://godottrench/` 里的自然包和模型 | 摆放了它们的地图继续有效。散布预设会在 `res://trenchdot/nature` 里找，所以想绘制新的散布，请重新添加自然包。 |
| 由旧插件构建的场景 | 它们能加载，因为 Godot 会通过 uid 找到被移动过的插件脚本。再构建一次地图就会刷新它们。 |

<div class="td-mcp" markdown>

## MCP 客户端 { #mcp-clients }

MCP 服务器现在叫 `trenchdot`。请在你客户端的配置里把它改名。令牌变量是 `TRENCHDOT_MCP_TOKEN`，旧的 `GODOTTRENCH_MCP_TOKEN` 仍然有效。

</div>
