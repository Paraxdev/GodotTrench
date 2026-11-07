---
source: c7fdbd8dcaa1ac8625c1d6e7014b686194c50f36
---

# 触发器 { #triggers }

> [!NOTE]
> 这些实体来自玩法实体包，见[安装与自定义](README.md#installing-and-customizing)。

不可见的笔刷体积，有物体进入时作出反应。关卡就是靠它们察觉到玩家：门口启动一段过场，深坑造成伤害，弹跳垫把人弹飞。用**体积**工具（Shift+E）来画。

## 共有的输入、输出和键 { #shared-inputs-outputs-and-keys }

下面每个触发器除了自己的之外，都有这些。

* **输入：** `enable`、`disable`、`toggle`
* **输出：** `triggered(activator)`、`entered(activator)`、`exited(activator)`
* **键：** `filter_group` player，`start_disabled` 0

`filter_group` 把触发器限制在那个 Godot 分组里的物体上，所以敌人和箱子不会触发玩家的触发器。留空就接受任何物体。

| 输出 | 何时触发 |
| --- | --- |
| `entered` | 启用期间，每一次被接受的进入 |
| `triggered` | 被接受的进入，对每个物体来说每 `cooldown` 最多一次。大多数接线用的是这个 |
| `exited` | 离开时，即使触发器已被禁用也会触发，用于物体离开时撤销某件事 |

触发器检测物理层 1 上的物体。如果玩家在别的层上，就要把那一层加进地图设置里的 **Trigger Collision Mask**（在 Godot 里是 `FuncGodotMapSettings` 资源），否则触发器永远不会触发。

## trigger_once { #trigger_once }

只触发一次，然后自己禁用，用于过场或提示这样只发生一次的事件。`enable` 会让它重新就绪。

* **键：** `once` 1

## trigger_multiple { #trigger_multiple }

每次进入都触发，对每个物体来说每 `cooldown` 最多一次，用于每次都会发生的事，比如商店的门铃。

* **键：** `cooldown` 0.5

## trigger_hurt { #trigger_hurt }

物体待在里面时持续伤害它们，用于岩浆、尖刺或地图下方的死亡区。它每隔 `interval` 秒对它们调用 `damage_method(amount, source)`，伤害量是 `damage * interval`。

* **输出：** `hurt(activator)`，每个受伤的物体每个时间片一次
* **键：** `damage` 10（每秒），`damage_method` take_damage，`interval` 0.5，`filter_group` 为空

`filter_group` 保持默认的空时，它会伤害任何有这个方法的东西，而不只是玩家。设了分组之后，组里的每个物体都会触发 `hurt`，哪怕它没有这个方法。

## trigger_teleport { #trigger_teleport }

把物体移到 `destination` 里指定名字的实体那里，通常是 [`info_teleport_destination`](actors.md#info_teleport_destination)，并让它们转向那个实体的偏航角。除非 `keep_velocity` 是 1，否则会让它们停下来，设成 1 就适合玩家要不失速地掉进去或跑过去的传送门。

* **输出：** `teleported(activator)`
* **键：** `destination`，`keep_velocity` 0，`cooldown` 0.5

## trigger_push { #trigger_push }

用 `push` 推动物体，它是以地图单位每秒计的速度，可以用视图里的手柄来指向。`once` 为 1 时，每次进入只给一次冲量，就像弹跳垫。为 0 时它会持续地推，像风洞或传送带，刚体的加速到 `push` 的速度就会停下。

* **输出：** `pushed(activator)`，每次进入一次
* **键：** `push` 0 384 0，`filter_group` 为空，`once` 1

## trigger_call { #trigger_call }

被触发时调用一个方法，默认调用 `!activator` 上的，所以触发器可以直接调用玩家的脚本，比如保存一个检查点。`call_target` 和 `arguments` 的用法和 [`logic_call`](scripting.md#logic_call) 里一样。

* **输入：** `trigger(activator)`
* **输出：** `called(result)`
* **键：** `call_target` `!activator`，`method`，`arguments` `[]`，`once` 0，`cooldown` 0.5

## trigger_spawn_area { #trigger_spawn_area }

在它体积内的随机位置生成场景，默认在玩家进入时生成，用于玩家走进房间时的伏击。它的键和 [`info_spawner`](actors.md#info_spawner) 上的用法相同。`interval` 大于 0 时，`start` 会启动一个生成计时器，这里的 `toggle` 切换的是这个计时器，而不是触发器。

* **输入：** `spawn`、`start`、`stop`、`toggle`、`kill_all`
* **输出：** `spawned(node)`、`all_dead`、`exhausted`（只触发一次，第一次达到 `total` 时）
* **键：** `scene`，`count` 3，`max_alive` 10，`total` 0，`interval` 0，`spawn_on_enter` 1，`spawn_group` enemies，`snap_to_ground` 1，`cooldown` 0.5

## trigger_area { #trigger_area }

一个没有脚本的普通 `Area3D` 体积，用来把房间的进入和离开事件直接接给其他实体。它没有上面那些共有的键，对每个物体都会触发，所以需要过滤时，请在它调用的实体里做。

* **输出：** `body_entered(body)`、`body_exited(body)`
* **键：** `targetname`
