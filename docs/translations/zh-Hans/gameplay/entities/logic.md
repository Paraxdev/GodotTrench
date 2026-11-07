---
source: 5168ac40e48ba63850a1be1795dae420bfe6bf5f
---

# 逻辑 { #logic }

> [!NOTE]
> 这些实体来自玩法实体包，见[安装与自定义](README.md#installing-and-customizing)。

不可见的点实体，决定事情何时发生。它们夹在触发器或按钮和被影响的东西之间，在途中对信号整形。处理数字的实体在[数学与数值](math.md)里，运行代码或动画的在[脚本与调用](scripting.md)里。

## logic_relay { #logic_relay }

启用时，把 `trigger` 作为 `triggered` 转发出去。把好几样东西接到同一个中继上，再 `disable` 它，就能一次关掉整条链。

* **输入：** `trigger(activator)`、`enable`、`disable`、`toggle`
* **输出：** `triggered(activator)`
* **键：** `start_disabled` 0

## logic_branch { #logic_branch }

记住真或假，收到 `test` 时触发 `on_true` 或 `on_false`，用于“只有通电门才能开”这样的检查。发电机把它设为真，门的按钮去测试它。

* **输入：** `set_true`、`set_false`、`toggle`、`test`、`set_and_test(value)`
* **输出：** `on_true`、`on_false`
* **键：** `start_value` 0

## logic_gate { #logic_gate }

持有两个真或假的值 `a` 和 `b`，用一种门把它们组合起来，用于“通电**并且**拉了拉杆时门才开”这样的检查。每次输入的变化只要翻转了结果，就会先触发 `changed(on)`，再触发 `on_true` 或 `on_false`，所以不必有谁去询问。`test` 会把当前结果再触发一次，用于别的东西现在就想知道的时候。

* **输入：** `set_a(value)`、`set_b(value)`、`toggle_a`、`toggle_b`、`test`
* **输出：** `on_true`、`on_false`、`changed(on)`
* **键：** `gate` and，`start_a` 0，`start_b` 0

| `gate` | 为真的条件 |
| --- | --- |
| `and` | 两个都为真 |
| `or` | 至少一个为真 |
| `xor` | 恰好一个为真 |
| `nand` | 不是两个都为真 |
| `nor` | 两个都不为真 |
| `xnor` | 两个相同 |
| `not` | `a` 为假，`b` 被忽略 |

不带值的 `set_a` 和 `set_b` 会设为真，所以把触发器的 `entered` 输出接到 `set_a`，就会把 `a` 打开。数字只要不是 0 就为真，文字只有 `true`、`yes` 或 `1` 才为真。起始值不会触发任何东西，没有改变结果的变化也不会。

## logic_flipflop { #logic_flipflop }

每次被触发都会交替一次，所以一个按钮就能把某样东西开了又关。变成开时触发 `on_a`，变成关时触发 `on_b`。`reset` 把它放回 `start_on`，两个都不触发，如果那算一次变化，就只触发 `changed`。

* **输入：** `trigger`、`reset`
* **输出：** `on_a`、`on_b`、`changed(on)`
* **键：** `start_on` 0（开，所以第一次触发会触发 `on_b`）

## logic_case { #logic_case }

把收到的值和最多八个分支比较，触发第一个匹配的分支的输出，都不匹配就触发 `on_default`。它把一个数字，比如 `logic_random` 给出的序号或 `math_value` 的分数，变成不同的事件。

* **输入：** `in_value(value)`
* **输出：** `on_case_1` 到 `on_case_8`，`on_default`
* **键：** `case_1` 到 `case_8`，为空

数字按值匹配，所以分支 `2` 能匹配值 2，也能匹配 `2.0`。文字必须完全一致，`Red` 不是 `red`，分支两边的空格会被忽略。空的分支会被跳过，第一个匹配的生效，没有值的情况会走 `on_default`。

## logic_random { #logic_random }

从最多八个输出里随机挑一个，或者掷一次概率。把 `out_1` 到 `out_3` 接到三盏灯上，`pick` 就会点亮其中一盏。`roll` 则改用概率来决定，比如一个箱子里有没有战利品。

* **输入：** `pick`、`roll`
* **输出：** `out_1` 到 `out_8`，`picked(index)`、`on_success`、`on_fail`
* **键：** `count` 3（1 到 8），`no_repeat` 0，`chance` 50（百分比）

`pick` 先带着所选输出的编号触发 `picked`，再触发那个输出，所以 `picked` 可以接给 [`logic_case`](#logic_case)。开启 `no_repeat` 后，它不会选上一次选过的输出，除非 `count` 是 1。`roll` 以 `chance` 百分比的概率触发 `on_success`，否则触发 `on_fail`，所以 0 永远不会成功，100 永远成功。

## logic_counter { #logic_counter }

在 `min` 和 `max` 之间计数，用于“按下全部三个按钮”这样的谜题：每个按钮发送 `add`，`hit_max` 把门打开。数值会被限制在范围内，只有值变了才会触发东西。变化落在某个上下限时触发 `hit_max` 和 `hit_min`，`reset` 只触发 `changed`。

* **输入：** `add(amount)`（默认 1）、`subtract(amount)`、`set_value(value)`、`reset`
* **输出：** `hit_max`、`hit_min`、`changed(value)`
* **键：** `min` 0，`max` 3，`start_value` 0

## logic_timer { #logic_timer }

一遍又一遍地触发 `timer`，用于任何会重复的东西，比如闪烁的灯或一波波敌人。两次触发之间等待 `interval` 秒，从地图加载起就运行，除非 `start_on` 是 0。`random_max` 大于 `random_min` 时，每次等待的时间在这个范围内随机，重复起来就没那么机械。`once` 让它在第一次触发后停下，这样它就成了一个简单的延迟。

* **输入：** `start`、`stop`、`toggle`、`fire`（立即触发一次）
* **输出：** `timer`
* **键：** `interval` 1.0，`random_min` 0，`random_max` 0，`start_on` 1，`once` 0

## logic_auto { #logic_auto }

地图里所有实体都就绪之后，触发一次 `map_spawn`。从一开始就要运行的东西都接到这里，比如音乐或开场文字。

* **输出：** `map_spawn`

## logic_sequence { #logic_sequence }

一条过场时间线。它按顺序触发最多八个编号的步骤，每一步之前都要等待，第一步也不例外。把每一步接到那一刻发生的事情上，比如门打开、一行文字、一个演员走动。

* **输入：** `start`、`stop`、`reset`
* **输出：** `step_1` 到 `step_8`，`step(index)`、`finished`
* **键：** `steps` 3（1 到 8），`interval` 1.0，`times` 为空，`start_active` 0（从地图加载起运行），`loop` 0

`times` 列出每一步的等待时间，比如 `0 1 0.5`，没有写的步骤用 `interval`。循环的序列永远不会触发 `finished`。它运行时会忽略 `start`，`stop` 之后再 `start`，就从第 1 步重新开始。见[过场教程](../../tutorials/cutscene.md)。
