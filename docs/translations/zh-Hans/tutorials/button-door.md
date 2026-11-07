---
source: 19de62e149bf31bc7f0453a04737b8fbd5510db4
---

# 按钮开门 { #button-opens-a-door }

墙上的一个按钮，玩家按下后会把门滑开。这是实体 I/O 最小也最实用的一个例子：按钮触发一个叫 `pressed` 的输出，一条连接把它变成对门的 `open` 输入的调用。

> [!TIP]
> 如果输出、输入和 `targetname` 对你还很陌生，请先读[实体 I/O 如何工作](../gameplay/io.md)。只要几分钟，而且本页是以它为基础的。

## 接线 { #wire-it }

`func_door` 和 `func_button` 来自玩法实体包。如果你的项目还没有它，请用 *玩法 > 安装玩法实体*，或者用 *设置此项目* 窗口（*Godot > 安装或更新插件…*）里的 **安装玩法实体**。

1. 做出门

    在门洞里画一个门大小的笔刷，怎样开出门洞可以看[入门](../getting-started.md#4-build-a-room)。右键点击这个笔刷，选择 *创建笔刷实体 > func_door*。

    ![橙色门笔刷上的右键菜单，创建笔刷实体已展开，func_door 处于悬停状态](../assets/button-door/create-brush-entity-func-door.png)

2. 给它起名

    在右侧的检查器里，把 `main_door` 输入到 `targetname`。其他实体就是靠这个名字找到门的。让 `travel` 保持 `0 64 0`，这样门打开时会向上滑 64 个单位（2 米）。

    ![func_door 的检查器，targetname 为 main_door，travel 为 0 64 0](../assets/button-door/inspector-targetname.png)

3. 做出按钮

    在门旁边的墙上画一个小笔刷，用同样的方法把它变成 `func_button`。

4. 同时选中两者

    先点击按钮，再按住 Ctrl 点击门，可以在视图里选，也可以在左侧的大纲里选。有墙挡着？在大纲顶部的筛选框里输入 `func`，列表里就只剩这两个了。

    ![按 func 筛选后的大纲，func_door (main_door) 和 func_button 都已选中](../assets/button-door/outliner-filter-func.png)

5. 把它们连起来

    打开 *玩法 > 逻辑 > 链接两个所选实体…*，或者按检查器顶部的 **连接…**。

    ![玩法菜单，逻辑已展开，链接两个所选实体… 处于悬停状态](../assets/button-door/menu-link-two-selected.png)

    触发的那个实体必须在左边，这里就是按钮。对话框会把你先做的那个放在左边，也就是门，所以请按两个名字之间的小交换按钮。在 *当* 下选 `pressed`，在 *调用* 下选 `open`，然后点 **链接**。

    ![链接实体窗口，左边是 func_button，右边是 func_door (main_door)，当 pressed，调用 open](../assets/button-door/link-dialog.png)

左下角的状态栏现在应该显示 `pressed > main_door.open`。可以这样读：“当 pressed 时，对 main_door 调用 open”。

![显示 pressed > main_door.open 的状态栏](../assets/button-door/status-linked.png)

单独选中按钮，你会在它的检查器里的 *输出* 下找到这条连接，以后可以在那里修改或删除它。

![func_button 的输出部分：输出 pressed，目标 main_door，输入 open](../assets/button-door/inspector-button-output.png)

> [!TIP]
> *玩法 > 门与移动体*（在检查器的 *玩法* 部分里也有）一键就能把所选笔刷变成铰链门或滑动门。它会给门命名，设好开启方式，并在门前放一个 `trigger_multiple`，玩家走近时就会开门。如果只想让按钮开门，就删掉这个触发器。
>
> ![玩法菜单，门与移动体已展开：左铰链或右铰链的铰链门，向上或向侧面滑动的滑动门，以及升降机、移动平台](../assets/button-door/menu-doors-and-movers.png)

## 让玩家按下它 { #let-the-player-press-it }

插件本身没有交互系统，因为每个游戏处理“使用”的方式都不一样。门和按钮有一个 `use(activator)` 方法，由你的玩家脚本来调用。演示里的玩家在按 **E** 时这样做：从摄像机发出一条射线，对击中的第一个带有 `use` 的东西调用它（[godot/demo/player.gd](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/player.gd)）：

```gdscript
func use_target() -> void:
	var from := camera.global_position
	var query := PhysicsRayQueryParameters3D.create(from, from - camera.global_basis.z * use_distance)
	query.exclude = [get_rid()]
	query.collide_with_areas = false
	var hit := get_world_3d().direct_space_state.intersect_ray(query)
	var node: Node = hit.get("collider")
	while node:
		if node.has_method(&"use"):
			node.call(&"use", self)
			return
		node = node.get_parent()
```

玩家把自己当作激活者传进去，所以 `pressed(activator)` 会带着玩家一起走。链里后面的输出可以把 `!activator` 作为目标，对按下按钮的那个人起作用。

## 变化 { #variations }

| 你想要 | 要改什么 |
| --- | --- |
| 门只能从按钮打开 | 取消勾选门的 `interact`。否则在门上按 **E** 也会把它打开 |
| 门自己关上 | 把门的 `wait` 设为它保持开着的秒数。默认值 -1 会让它一直开着 |
| 上锁的门 | 勾选 `locked`。门会忽略 `open` 和 `use`，改为触发 `locked_use`，直到某个输出调用它的 `unlock` 输入，例如来自钥匙拾取物的输出 |
| 一个按钮开几扇门 | 把门命名为 `gate_1`、`gate_2` 等等，然后以 `gate_*` 为目标。结尾的 `*` 会匹配所有以 `gate_` 开头的名字 |
| 摆动的门 | 改用 `func_door_rotating`，并在视图里把它的铰链手柄拖到铰链该在的位置 |

## 用代码做同样的接线 { #the-same-wiring-in-code }

更想在 GDScript 里而不是在编辑器里连接？下面的做法和上面的连接效果一样：

```gdscript
# pressed(activator) > main_door.open
button.pressed.connect(func(activator):
	for door in TrenchDotIO.find_targets(button, "main_door", activator):
		TrenchDotIO.invoke(door, &"open", "", activator))
```

直接把 `pressed` 连到 `door.open` 会失败，因为 `pressed` 会传一个参数，而 `open()` 不接受参数。`invoke` 会让参数适配这个方法，和地图连接的做法一样。
