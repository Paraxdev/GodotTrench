---
source: 9167db67d391d56c5d5d5ba31e5f260c3c7d0f58
---

# 实体参考 { #entity-reference }

这里是玩法实体包里的实体，按用途分组。每个条目先说明这个实体是做什么的，再列出它的输入（其他实体可以让它做什么）、输出（可以继续往下接线的事件）和键（检查器里的设置）。输出怎样连到输入，见[实体 I/O](../io.md)。

| 页面 | 你会在这里找到什么 |
| --- | --- |
| [逻辑](logic.md) | 决定事情何时发生的不可见辅助实体：中继、“仅当”检查、门、触发器翻转、随机选择和分支、计数器、定时器、地图启动事件和过场时间线 |
| [数学与数值](math.md) | 存储、计算和比较数字的不可见辅助实体，比如用一个分数去开门 |
| [脚本与调用](scripting.md) | 运行一段 GDScript、调用你的游戏代码、播放动画，或者打印一条链正在做什么 |
| [门、移动体和按钮](movers.md) | 会移动的笔刷实体：滑动门和旋转门、升降机、列车和按钮 |
| [触发器](triggers.md) | 不可见的体积，物体走进去时作出反应，用来启动事件、造成伤害、传送、推动或生成 |
| [生成器与路径](actors.md) | 生成敌人或拾取物、传送目的地、路径转角和过场里受脚本控制的角色 |
| [道具、灯光与特效](effects.md) | 可破坏的物理道具、静态模型、爆炸、屏幕文字、可开关的灯、声音、粒子和光照探针 |
| [流式分区](sections.md) | 地标和连接体积，用来把多张地图当成一个世界来玩，没有加载画面 |

## 安装与自定义 { #installing-and-customizing }

这个包不是插件本身的一部分。在*设置此项目*窗口里点击**安装玩法实体**，或者选择*玩法 > 安装玩法实体*，它的定义和脚本就会被复制到 `res://trenchdot/entities`。在那之前，实体面板只列出核心部分：worldspawn、`func_` 辅助实体、`light`、`light_spot`、光照探针和反射探针，还有 `info_player_start`。

这些副本属于你的项目。你可以修改 `scripts/` 里某个实体的脚本，或者 `definitions/` 里它的键和默认值，删掉用不到的实体，也可以添加自己的。插件更新时会提供更新的包，但绝不会覆盖你改过的文件。[项目设置](../../godot/project-setup.md#core-entities-and-the-gameplay-entities-pack)讲了更新是怎么进行的，以及两个定义用了同一个 classname 时会发生什么。

## 怎么看这些条目 { #reading-the-entries }

* 括号里的参数，是输入接受的内容，或者输出带出去的内容。例如 `trigger(activator)` 接收启动这条链的物体或实体。
* 每个带输入或输出的实体都有一个 `targetname` 键，`logic_auto` 除外。其他实体就是用这个名字找到它的。
* 你没设置的键，用列出的默认值。
* 距离的单位是地图单位，每米 32 个。速度的单位是米每秒，`omni_range`、`spot_range` 和 `max_distance` 的单位是米。
* 输入和输出列表是编辑器的选择器提供的内容。运行时，实体脚本上的任何方法都可以当作输入，所以选择器里的并不是全部。

要添加你自己的类，见[用 GDScript 写自定义实体](../custom-entities.md)。

## 不在包里的东西 { #not-in-the-pack }

**核心**随插件一起提供，不需要这个包。Worldspawn 里有太阳、环境光、天空、雾和流式加载的设置，什么都没选中时检查器会显示它。FuncGodot 的类 `func_geo`、`func_detail`、`func_detail_illusionary` 和 `func_illusionary` 用来构建普通几何体。`info_player_start` 标出玩家出生的位置，但它自己不会生成任何东西，把玩家放到那里是你的游戏代码要做的事。[`light` 和 `light_spot`](effects.md) 是可开关的灯。

笔刷实体和模型实体默认都有碰撞，改变这一点的键见[碰撞](collision.md)。

**玩家**不是实体，它是你自己游戏里的一个场景。这个包期望它在 `player` 组里，并且在碰撞层 1 上（触发器就看这一层），还要有一个 `take_damage(amount, source)` 方法，伤害触发器和爆炸会调用它。玩家在别的层上，就要把那一层加进 *Trigger Collision Mask*，见[触发器](triggers.md#shared-inputs-outputs-and-keys)。完整的例子见 [`godot/demo/player.gd`](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/player.gd)。
