---
source: 978a2cda95dff6236396e32b6d494bed11f3f94b
---

# 生成器与路径 { #spawners-and-paths }

> [!NOTE]
> 这些实体来自玩法实体包，见[安装与自定义](README.md#installing-and-customizing)。

## info_spawner { #info_spawner }

在 `radius` 范围内的随机位置生成一个场景，比如敌人或拾取物。要做伏击，就从触发器对它触发 `spawn`；想要源源不断地生成，就设一个 `interval` 再 `start` 它。要把生成点分散到整个房间里，请用 [`trigger_spawn_area`](triggers.md#trigger_spawn_area)。

| 键 | 默认值 | 作用 |
| --- | --- | --- |
| `scene` | | 要生成的 `res://` 场景 |
| `count` | 1 | 每次生成多少个 |
| `max_alive` | 5 | 同时存活的数量永远不超过它，多出来的生成会被跳过 |
| `total` | 0 | 生成这么多之后就彻底停下，0 表示不限 |
| `interval` | 0 | 计时器运行时，两次生成之间的秒数。0 表示只在收到 `spawn` 时生成 |
| `radius` | 64 | 它们可以出现在离生成器多远的地方。在视图里拖动它的手柄 |
| `start_active` | 0 | 从地图加载起就运行计时器 |
| `spawn_on_ready` | 0 | 地图加载时生成一次 |
| `spawn_group` | enemies | 加到每个生成出来的节点上的 Godot 分组 |
| `snap_to_ground` | 1 | 把每个生成物放到它下方的地面上 |

* **输入：** `spawn`、`start`、`stop`、`toggle`、`kill_all`
* **输出：** `spawned(node)`、`all_dead`、`exhausted`（只触发一次，第一次达到 `total` 时）

`all_dead` 在生成的节点全都消失时触发，用于“清掉房间里的敌人才开门”。生成的节点没有 targetname，要通过它们的 `spawn_group` 来找到，比如 `@enemies`。

用默认的键时，在有东西对它触发 `spawn` 之前它什么也不生成。想要不靠触发器就生成，就设 `spawn_on_ready`，或者设一个 `interval` 并打开 `start_active`。生成只发生在运行中的游戏里，在 Godot 编辑器里，生成器只是一个空节点。

## info_teleport_destination { #info_teleport_destination }

标出 `trigger_teleport` 把物体送到哪里。让它朝向玩家到达后该看的方向，它的偏航角就是他们的朝向。它唯一的键是 `targetname`。

## path_corner { #path_corner }

`func_train` 或 `npc_walker` 路线上的一个停靠点，用 `target` 指出下一个。用**路径**工具（Shift+P）一路点下去就能连成一条链。当列车或步行者到达时它会触发 `reached`，用于在那一站播放声音或触发事件。

* **输出：** `reached(train)`
* **键：** `target`，`wait` 0（在这里等待的秒数），`speed` 0（从这里起列车的新速度，0 表示保持不变）

`speed` 只影响列车，步行者保持自己的速度。

## npc_walker { #npc_walker }

沿着一串路径转角行走并播放动画的受脚本控制的演员，用于过场而不是 AI，比如一个从窗前走过的守卫。它沿直线移动，无视重力和墙壁，所以转角要摆在它走得过去的地方。

| 键 | 默认值 | 作用 |
| --- | --- | --- |
| `model` | | 给它网格和一个 `AnimationPlayer` 的场景 |
| `target` | | 要走向的第一个路径转角 |
| `speed` | 2.0 | 米每秒 |
| `loop` | 0 | 走完最后一个转角后回到第一个 |
| `start_active` | 0 | 从地图加载起就行走 |
| `walk_anim`、`idle_anim` | walk、idle | 行走时和停下时播放的动画 |
| `face_travel` | 1 | 转向它行走的方向 |
| `collision` | default | 根据它的模型构建的形状，让其他物体能撞到它，见[碰撞](collision.md) |
| `collision_layer`、`collision_mask` | 空 | 像 `1, 3` 这样的层编号，留空则保持项目默认值 |

* **输入：** `start`、`stop`、`walk_to(corner)`、`play_anim(name)`、`face(target)`
* **输出：** `arrived(corner)`、`reached_goal`、`finished`（不循环时，两者都在最后一个转角触发）

`face` 和 `walk_to` 接受任何目标，比如 `!player`。`walk_to` 会放弃当前这一段，直接朝那里走去。

它只在运行中的游戏里加载 `model`，所以在 Godot 编辑器里它是一个空节点。
