---
source: 12dfe8079d66dd10bb5ca7191aa90e4b3c01fe5b
---

# 脚本过场动画 { #scripted-cutscene }

玩家走进一个房间，弹出一块告示牌，向导走开，一个桶爆炸，一盏灯亮起，出口打开。这一切都不需要你自己写代码：一个计时器实体每一拍触发一个输出，每个输出连到该行动的实体。你可以在演示项目里找到名为 `scripted_scene` 的地图。

![逻辑图面板里的 scripted_scene 地图：start_zone 触发 cutscene.start，cutscene 的各个 step 输出连到向导、桶和提示，桶的 broken 输出连到 hp_readout。灯和出口门在更下面，不在图里](../assets/cutscene/logic-graph.png)

## 时间线 { #the-timeline }

这里的每个实体都来自玩法实体包。如果你的项目还没有它，请先用 *玩法 > 安装玩法实体* 安装。

1. 盖住入口

    在入口上画一个笔刷，把它做成名为 `start_zone` 的 `trigger_once`（右键点击，*创建笔刷实体 > trigger_once*）。玩家第一次走进来时它会触发 `triggered`，然后自己关闭，所以这段过场只会播放一次。

2. 加上时间线

    从 *玩法 > 逻辑* 里选 `logic_sequence`，它会落在 3D 光标处。把它命名为 `cutscene`，把 `steps` 设为 5，`interval` 设为 1.5。启动后它会依次触发 `step_1`、`step_2` 等等，每次间隔 1.5 秒。

    ![玩法菜单，逻辑已展开，放在光标处下面的 logic_sequence 处于悬停状态](../assets/cutscene/menu-logic-sequence.png)

3. 从触发器启动它

    用 *玩法 > 逻辑 > 链接两个所选实体…*，把 `start_zone` 的 `triggered` 输出连到 `cutscene.start`，做法和[按钮开门](button-door.md)一样。

4. 给每一拍接线

    把 `cutscene` 的每个 step 输出连到那一拍里行动的实体，一对一对地连。在[逻辑面板](../gameplay/logic-graph.md)里从输出针脚拖到输入针脚也可以。

| 输出 | 目标和输入 | 发生什么 |
| --- | --- | --- |
| `step_1` | `hint.show` | 一个 `game_text` 显示“跟着向导走” |
| `step_2` | `guide.start` | 一个 `npc_walker` 沿着它的 `path_corner` 链出发，这是一排标记点，每个都指明下一个 |
| `step_3` | `barrel.ignite` | 一个爆炸性的 `prop_physics` 烧着引信，然后爆炸并碎开 |
| `step_4` | `lamp.turn_on` | 灯亮起 |
| `step_5` | `exit_door.open` | 出口门向上滑开 |

选中 `cutscene`，你应该能在它的检查器里的 *输出* 下看到这五条。

![名为 cutscene 的 logic_sequence 的检查器：steps 为 5，interval 为 1.5，输出（I/O，5）从 step_1 到 hint.show 开始](../assets/cutscene/inspector-sequence.png)

序列在每一步之前都会等待，第一步也一样，所以第 1 步在 1.5 秒触发，第 5 步在 7.5 秒触发。如果想要不均匀的节奏，就把 `times` 设为每步一个等待时间，比如 `0.5 2 1 1 3`。

## 读取玩家的生命值 { #reading-the-players-health }

有些反应需要一点内置输入覆盖不到的逻辑。`logic_script` 在被触发时会运行一小段 GDScript。把桶的 `broken` 输出连到 `hp_readout.run`，这是一个 `logic_script`，它的 `source` 会把玩家的生命值写到一个名为 `hpmsg` 的 HUD `game_text` 上：

```gdscript
var players = io.find_targets(this, "!player", null)
if players.is_empty():
	return
var hp = int(players[0].health)
for label in io.find_targets(this, "hpmsg", null):
	label.set_text("HP " + str(hp))
	label.show()
```

这里要用 `!player`，不要用 `!activator`。`!activator` 是引发这条链的那个人，而各个 step 输出和 `broken` 都没有带上任何人。`!player` 总能找到 `player` 组里的节点。`broken` 在爆炸造成伤害之后才触发，所以生命值已经是最新的了。

## 播放它 { #playing-it }

场景脚本 [godot/demo/scenes/scripted_scene.gd](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/scenes/scripted_scene.gd) 只负责构建地图，在 `info_player_start` 处生成玩家，并添加一个 `TrenchDotDebugOverlay`。在游戏里按 **F3** 就能看到每个输出的触发。这是找出接错名字的那一步最快的办法。

<div class="td-mcp" markdown>

## MCP { #mcp }

这张演示地图是通过 [MCP 服务器](../mcp.md)驱动编辑器构建的。调用记录在 [examples/mcp/scripted_scene.json](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/scripted_scene.json) 里。

</div>
