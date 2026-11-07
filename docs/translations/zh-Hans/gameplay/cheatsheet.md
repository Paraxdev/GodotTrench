---
source: 414dc5a3711ba57511bd4ecc18dc19f159af44d7
---

# I/O 速查表 { #io-cheatsheet }

[实体 I/O 是怎么工作的](io.md)和[目标、参数和输入](parameters.md)的一页纸摘要，适合你已经懂了思路、只想查细节的时候。

## 输出触发时 { #when-an-output-fires }

信号触发，计入*次数*，等待*延迟*，然后对目标找到的每个节点调用输入。如果一个都没找到，就什么也不会发生，而找不到任何东西的 targetname 或节点路径会警告一次。

## 目标 { #targets }

| 写法 | 触达 |
| --- | --- |
| `main_door` | 同一张地图里带这个 targetname 的每个实体 |
| `door_*` | 每个以 `door_` 开头的 targetname |
| `@enemies` | 树中任何位置、属于该 Godot 分组的每个节点 |
| `!player` | `player` 分组里的第一个节点 |
| `!activator` | 启动这条链的那个人，`!caller` 的意思相同 |
| `!self` | 拥有这个输出的实体 |
| `/root/Game` | 位于该绝对路径的节点 |

## 参数 { #parameters }

普通的值会依次尝试当作整数、浮点数、布尔值，然后把 `x y z` 当作 `Vector3`，否则保持为字符串。JSON 的 `[a, b]` 会展开成多个参数。`$activator`、`$self`、`$position` 和 `$caller_name` 是在**接收**的实体上填入的。

没有参数时，激活者会进入第一个对象参数，输出带来的其他值按顺序填入其余参数。

## 谁保留激活者 { #who-keeps-the-activator }

| 信号 | 激活者 |
| --- | --- |
| `triggered`、`pressed`、`entered`、`exited`、`hurt`、`teleported`、`pushed` | 原样传下去 |
| `arrived(corner)`、`spawned(node)`、`reached(train)` | 被括号里的节点替换 |
| `opened`、`timer`、`step_N`、`on_true`、`on_case_N`、`result`、`broken`、`switched`、`map_spawn` | 被丢弃 |

如果链丢了它而你需要玩家，就以 `!player` 为目标。

## 每个节点都有的输入 { #inputs-every-node-has }

`kill`、`show`、`hide`、`enable`、`disable` 和 `toggle` 对任何节点都有效，前提是实体没有同名的方法。匹配到某个属性的输入，会把它设置为参数。

## logic_script 的作用域 { #logic_script-scope }

[`logic_script`](entities/scripting.md) 在被触发时运行一小段 GDScript。在其中，`this` 是 logic_script 本身，`activator` 是启动这条链的节点，`parameter` 是传入的值，`io` 是 `TrenchDotIO` 的辅助函数，`tree` 是 `SceneTree`。用制表符缩进。

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

## 什么都没发生？ { #nothing-happened }

先查问题面板，再查逻辑图，然后在游戏里按 F3，再记录 `TrenchDotIO.events()`。见[调试接线](debugging.md)。
