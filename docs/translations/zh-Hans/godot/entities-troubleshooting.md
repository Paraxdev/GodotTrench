---
source: 1b2a2ceda473607c8db087c552aff86851d08ef2
---

# 什么都不做的实体 { #entities-that-do-nothing }

当 Godot 构建一个没有定义的实体时，这个实体只是个空壳。像门这样的笔刷实体会保留它的网格，但会变成一个没有脚本也没有碰撞的普通 `Node3D`，点实体则变成一个空的 `Marker3D`。当定义和它的实体对不上时也会这样，比如某个笔刷实体的定义构建不出物理体。出现这种情况时，TrenchDot 和 Godot 都会告诉你，这一页逐条解释这些消息。

## 在哪里看到 { #where-you-see-it }

1. 状态栏

    在 Godot 里打开着这张地图的场景时，在 TrenchDot 里保存地图。当 Godot 构建了它没有定义的类，左下角的状态栏会点出它们，以及 Godot 构建时用的地图设置：

    ![状态栏写着 Godot 重新构建了 1 个地图节点，但 res://crypt_settings.tres 没有 func_button 和 func_door 的定义，所以它们被构建为普通节点](../assets/entities-troubleshooting/status-missing-classes.png)

2. 问题面板

    打开**问题**标签页，它在左边的大纲和历史旁边。它列出每个被 Godot 构建成普通节点的类，附带修复办法，这次构建的其他警告以“Godot：”开头。点击一行就会选中那个实体。

    ![问题面板里有两条警告，Godot 把 func_button 和 func_door 构建成了普通节点，每条都说明要往 Base Fgd Files 里加哪个文件](../assets/entities-troubleshooting/issues-missing-classes.png)

    在 Godot 构建这张地图之前，问题面板会拿你的实体和 Godot 构建时用的 FGD 做比较，这个 FGD 是插件随[游戏配置](project-setup.md#the-game-config)一起导出的。那个 FGD 没有的实体，会得到“Godot使用……构建，其 FGD ……没有……”这样的警告。

    当 Godot 显示的是它在某个定义还不存在时构建的场景，状态栏和问题面板会说“Godot 显示的……是用与其当前地图设置不同的实体定义构建的”。选择*Godot > 在 Godot 中构建*，或者在 Godot 里按 **Build Map** 再保存，就能重新构建它。**运行项目**对它运行的场景也会说同样的话。

3. Godot 的输出面板

    在 Godot 里对 `FuncGodotMap` 节点按 **Build Map**。Godot 编辑器底部的输出面板会针对没有定义的类给出一条 `[TrenchDot]` 警告，每个对不上的定义也各给一条。在节点的 *Build Flags* 下勾选 *Print Build Report*，可以看到完整的[构建报告](building.md#build-report)。

## 没有定义的实体 { #entities-without-a-definition }

| 消息说 | 原因 | 怎么办 |
| --- | --- | --- |
| Install the Gameplay entities pack | 门、按钮、触发器、道具和生成器都随这个包提供，而项目里没有它 | 在 TrenchDot 里选择*玩法 > 安装玩法实体*，参见[项目设置](project-setup.md#installing-and-updating-the-pack) |
| Your FGD lacks the Gameplay entities pack, add `trenchdot_default_fgd.tres` to its Base Fgd Files | 你的 FGD 没有 *Base Fgd Files* | 把 `res://addons/func_godot/fgd/trenchdot_default_fgd.tres` 加进地图设置在 *Entity Fgd* 里指定的那个 FGD 的 *Base Fgd Files* |
| The pack is installed, but the *Entity Fgd* is an FGD of the addon | 地图设置用的是 FuncGodot 自己的 FGD，或者插件里的另一个文件，插件更新时会把它替换掉 | 把那些地图设置的 *Entity Fgd* 设为你的游戏配置用的 FGD |
| A class of your own has no definition | 它在你的游戏配置的 FGD 里，却不在 Godot 构建所用地图设置的 *Entity Fgd* 里 | 把那些地图设置的 *Entity Fgd* 设为你的游戏配置用的 FGD，参见[你自己的资源](project-setup.md#your-own-resources) |
| The map node uses the addon's default map settings, not the ones from Project Settings | 这个节点是用旧版插件添加的，它把默认的地图设置存进了场景里 | 在 Godot 里选中这个地图节点，在检查器里清空 *Map Settings* |
| The scene was built with other entity definitions | 它是在这个包或某个定义还不存在时构建的，之后没再构建过 | 按 **Build Map** 并保存场景。在[实时模式](live.md#when-live-mode-rebuilds-the-whole-map)下，Godot 会自己做这件事 |

## 对不上的定义 { #definitions-that-do-not-fit }

这些消息来自 Godot 的输出面板，保存之后也会作为“Godot：”警告出现在问题面板里。最后两种，问题面板在你保存之前就会标出来。

| 消息说 | 原因 | 怎么办 |
| --- | --- | --- |
| `func_crusher builds as a Marker3D, so its brushes have no collision` | 定义没有 *Node Class*，它的脚本继承的节点又不是物理体 | 把 *Node Class* 设为 `StaticBody3D`、`AnimatableBody3D` 或 `Area3D`，或者让脚本继承其中之一 |
| `func_crusher: Node Class AnimatedBody3D not found, building it as a Node3D` | 拼写错误，或者是 Godot 还没注册的 `class_name` | 改正名字。C# 的情况参见 [C# 实体](../gameplay/csharp.md) |
| `alarm: res://entities/alarm.gd has no @tool` | 编辑器只运行 tool 脚本，所以地图里的键永远到不了这个节点 | 在脚本顶部加上 `@tool` |
| `light is a point entity, so its brushes are dropped` | 一个笔刷实体用了 `FuncGodotFGDPointClass` | 把定义改成 `FuncGodotFGDSolidClass` |
| `func_door is a brush entity without brushes` | 一个没有东西可以构建的笔刷实体 | 给它笔刷，或者把定义改成点类 |

## 构建出来却没有反应的实体 { #entities-that-build-but-do-not-react }

| 你看到的 | 原因 | 怎么办 |
| --- | --- | --- |
| 触发器从不触发，Godot 的输出说触发器忽略了你的玩家，因为它的 `filter_group` 里没有任何节点 | 默认情况下，触发器只对物理层 1 上、属于 `player` 组的物体有反应 | 把你的玩家加进 `player` 组。在别的层上，就把那一层加进地图设置的 *Trigger Collision Mask* |
| 门和按钮从不移动 | 要等你的游戏代码对它们调用 `use()` 它们才会动 | 参见[让玩家按下它](../tutorials/button-door.md#let-the-player-press-it) |
| `info_spawner` 和 `npc_walker` 在 Godot 编辑器里看起来是空的 | 它们只在运行的游戏里生成或加载自己的模型 | 参见[生成器和路径](../gameplay/entities/actors.md) |
| Godot 的输出里，`res://trenchdot/entities` 中的某个脚本出现 `Parse Error` | Godot 在认识某个包脚本用到的类之前就编译了它 | 重启 Godot，然后按 **Build Map** 并保存 |
