---
source: 63266bb349fb8b02f543d43498bfdee3d8c3f3cd
---

# 门、移动体和按钮 { #doors-movers-and-buttons }

> [!NOTE]
> 这些实体来自玩法实体包，见[安装与自定义](README.md#installing-and-customizing)。

作为一个整体移动、并带着站在上面的东西一起走的笔刷实体，所以升降机会把玩家一起带走。

> [!NOTE]
> 门和按钮自己不会对玩家作出反应。你的玩家脚本要对它们调用 `use(activator)`，比如从一条交互射线里调用，见[按钮开门](../../tutorials/button-door.md)。如果有的门只该由 I/O 或脚本来动，就把 `interact` 设为 0。

## func_door { #func_door }

滑动门、舱口或卷帘。打开时它移动 `travel` 这么远，关上时再移回来，在视图里拖动手柄就能设置。上锁的门会忽略 `open` 和 `use`，改为触发 `locked_use`，用来显示“已上锁”的提示。

| 键 | 默认值 | 作用 |
| --- | --- | --- |
| `travel` | 0 64 0 | 它滑动的偏移量，单位是地图单位 |
| `speed` | 2.0 | 米每秒 |
| `wait` | -1 | -1 表示一直开着，正数表示过这么多秒后再关上 |
| `locked` | 0 | 一开始就上锁 |
| `start_open` | 0 | 一开始就在打开的位置 |
| `interact` | 1 | 玩家对它调用 `use` 时打开 |

* **输入：** `open`、`close`、`toggle`、`lock`、`unlock`、`use(activator)`
* **输出：** `opened`、`closed`、`started_opening`、`started_closing`、`locked_use(activator)`

## func_door_rotating { #func_door_rotating }

绕着铰链摆动的门或栅栏门，铰链在视图里拖到位。`open` 会接收激活者，所以 `open_away` 能让它朝远离激活者的方向摆开，而不是撞到他们脸上。

| 键 | 默认值 | 作用 |
| --- | --- | --- |
| `hinge` | 0 0 0 | 铰链相对于门中心的位置 |
| `axis` | y | 摆动轴。`y` 是竖直的铰链，`x` 或 `z` 用来做舱口或吊桥 |
| `open_angle` | 95 | 摆动的角度，负数则朝另一边摆 |
| `speed` | 120 | 度每秒 |
| `open_away` | 1 | 朝远离开门者的方向摆动 |
| `wait`、`locked`、`start_open`、`interact` | -1、0、0、1 | 和 `func_door` 一样 |

* **输入：** `open(activator)`、`close`、`toggle(activator)`、`lock`、`unlock`、`use(activator)`
* **输出：** 和 `func_door` 一样

`func_gate` 是较旧的别名，键只有 `hinge`、`open_angle` 100 和 `speed` 90。请用 `func_door_rotating`。

## func_platform { #func_platform }

在起点和 `start + travel` 之间来回的升降机或移动平台，用于电梯或跨越深坑的地板。

* **输入：** `start`、`stop`、`toggle`、`go_to_end`、`go_to_start`
* **输出：** `reached_end`、`reached_start`、`started`
* **键：** `travel` 0 128 0，`speed` 2.0，`wait` 1.0，`mode` 0，`start_active` 0（从地图加载起就移动）

| `mode` | `start` 的作用 | 典型用途 |
| --- | --- | --- |
| 0，切换 | 移到另一端 | 用按钮呼叫的升降机 |
| 1，往返 | 来回移动直到 `stop`，每一端等待 `wait` 秒 | 一直循环的平台 |
| 2，一次 | 移到终点并停在那里 | 只伸出一次的桥 |

## func_train { #func_train }

沿着一串 `path_corner` 实体行进，从 `target` 开始，用于任何要跑一条有多个停靠点的路线的东西，比如电车或轨道上的矿车。每个转角的 `wait` 和 `speed` 在到达时生效。它在地图加载时就开始移动并且循环，所以只想跑一趟的话，把 `start_active` 或 `loop` 设为 0。

* **输入：** `start`、`stop`、`toggle`
* **输出：** `arrived(corner)`、`finished`（只在不循环时）
* **键：** `target`、`speed` 3.0、`loop` 1、`start_active` 1、`orient` 0（1 会让它转向行进的方向）

## func_button { #func_button }

被按下时下沉 `travel` 这么多，并触发 `pressed`。过 `wait` 秒后它会弹回来，而 -1 让它一直保持按下，做成一次性的开关。上锁的按钮改为触发 `locked_use`，适合需要先通电的面板。

* **输入：** `press(activator)`、`release`、`lock`、`unlock`、`use(activator)`
* **输出：** `pressed(activator)`、`released`、`locked_use(activator)`
* **键：** `travel` 0 -2 0，`wait` 1.0，`locked` 0，`interact` 1
