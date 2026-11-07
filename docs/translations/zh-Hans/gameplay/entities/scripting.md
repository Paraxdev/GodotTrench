---
source: 2bf109094dcc42ebb11cf6c736e210c0e4e79b02
---

# 脚本与调用 { #scripts-and-calls }

> [!NOTE]
> 这些实体来自玩法实体包，见[安装与自定义](README.md#installing-and-customizing)。

用来处理普通接线够不着的事情的点实体：游戏状态、你自己的游戏代码、动画，以及查明一条链为什么没有触发。

## logic_script { #logic_script }

被触发时运行一段 GDScript，用于内置实体没有提供的一点点逻辑，比如读取玩家的生命值，不用专门写一个自定义实体。

* **输入：** `run(activator)`、`run_with(parameter)`
* **输出：** `ran(result)`，带着这段代码的返回值
* **键：** `source` 为空，`expression` 为空（单行写法，`source` 为空时使用）

这段代码能用到 `this`（实体本身）、`activator`（启动这条链的那个人）、`parameter`（传入的值）、`io`（`TrenchDotIO` 的辅助函数，比如按名字查找实体）和 `tree`（`SceneTree`）。`run_with` 的 `activator` 永远是 null。编译错误会在地图加载时作为警告记入日志。下面这段代码会伤害玩家：

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

> [!WARNING]
> 嵌套的行请用制表符缩进。用空格会因为混合缩进错误而无法编译。

> [!WARNING]
> 这段代码是随地图一起带来的。游戏每次运行它都会执行，权限和项目自己的脚本一样，所以对待带代码的下载地图，要像对待任何下载来的程序一样：它在你的电脑上什么都能做。问题面板会对每个带代码的 `logic_script`、每个能交给它代码的输出、以及每个含有这两者之一的预制体发出警告，玩这张地图之前请先看一看。

## logic_call { #logic_call }

用固定的参数调用任意目标上的方法，用来接入你的游戏代码，比如在 `Game` 自动加载上加分。和普通输出不同，目标或方法不存在时它会发出警告，所以拼写错误会在日志里露出来。C# 方法使用它们的 PascalCase 名字。

* **输入：** `trigger(activator)`、`call_with(parameter)`
* **输出：** `called(result)`
* **键：** `call_target` `/root/Game`，`method` 为空，`arguments` `[]`

`call_target` 可以是节点路径、`@group`、targetname、`!activator` 或 `!self`。`arguments` 是一个 JSON 数组，比如 `["key_red", "$activator"]`，可以使用[目标、参数和输入](../parameters.md)里的占位符，在 `logic_call` 自己身上解析。`call_with` 会把它的参数放在 `"$parameter"` 的位置，或者把它当作唯一的参数传过去。

## logic_animate { #logic_animate }

在 `target` 上或其下的第一个 `AnimationPlayer` 上播放动画，用于自带动画的角色和道具。把 `target` 设为 `!activator`，就能让启动这条链的那个人动起来。

* **输入：** `play(name)`、`stop`、`queue(name)`、`seek(time)`
* **输出：** `finished(anim)`
* **键：** `target` `!self`，`animation` 为空（`play` 没带名字时播放它），`speed` 1.0

## logic_debug { #logic_debug }

收到 `write` 时打印一行，并在调试版本里把最近几行显示成一个浮动标签，这样你能在游戏里看到一条链有没有走到这里，以及带着什么参数。打开 `trace_all` 后，它会记录游戏里的每一个输出，是观察整条链的快捷办法。见[调试接线](../debugging.md)。

* **输入：** `write(parameter)`、`toggle`（标签）、`show`、`hide`
* **输出：** `printed(text)`
* **键：** `message` `{name} received {parameter} from {activator}`（也可以用 `{time}`），`show_label` 1，`trace_all` 0
