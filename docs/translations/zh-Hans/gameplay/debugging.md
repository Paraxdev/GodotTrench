---
source: 425ace012cd6d8410cbf64a89bb7dc31275f8980
---

# 调试接线 { #debugging-wiring }

大多数接线错误都是悄无声息地失败：门就是一直关着。请按这个顺序检查，从最快的检查到最彻底的检查。

> [!TIP]
> 门或触发器连玩家都挡不住，说明它是在没有定义的情况下构建的，这不是接线错误。见[什么都不做的实体](../godot/entities-troubleshooting.md)。

## 1. 问题面板 { #1-the-issues-panel }

打开**问题**标签页，它在左侧大纲和历史旁边。它会标出指向没有任何实体拥有的 targetname 的输出，以及目标的类里没有的输入。targetname 打错，是门一直关着最常见的原因。

![问题面板里有一个错误：输出 triggered 指向 trigger_once 上的未知实体“hall_lihgt”，旁边有一个修复按钮](../assets/debugging/issues-panel.png)

点击一个问题，会选中它的实体并在视图里取景，然后在检查器里改正名字。

> [!WARNING]
> 对缺失目标按**修复**不会去猜正确的名字，它会删除这条连接。如果是打错字，请手动改名字。

## 2. 逻辑图 { #2-the-logic-graph }

在[逻辑图](logic-graph.md)里，没有人拥有的目标会显示成一个红色节点，连着一条红色虚线，说明没有实体叫这个名字。

![逻辑图：trigger_once 的 triggered 输出沿着红色虚线连到红色节点 hall_lihgt，没有实体叫这个名字](../assets/debugging/graph-missing-target.png)

**模拟**会在纸面上触发一个输出，并高亮整条链，见[模拟](logic-graph.md#simulate)。

## 3. 游戏里的 F3 { #3-f3-in-game }

在 Godot 里给你的场景添加一个 `TrenchDotDebugOverlay` 节点，或者把它添加为自动加载。运行游戏并按 F3：它会在输出触发时把它们列出来，并画出每个 `Area3D` 体积和它的 targetname，所以你能看出触发器是从没触发过，还是触发了却没指向任何东西。

生成触发器之后请再切换一次，体积是在它打开时收集的。按键是覆盖层的 `toggle_key` 属性。

## 4. 日志 { #4-logging }

放一个 `logic_debug` 实体，并在检查器里勾选它的 `trace_all`。之后每个触发的输出都会打印到 Godot 的输出里。

在代码里，可以连接 `TrenchDotIO.events().fired`，每个输出触发时它都会发出，包括目标解析为空的那些。

```gdscript
TrenchDotIO.events().fired.connect(func(source, output, target, input, parameter):
	print(source.name, ".", output, " -> ", target, ".", input))
```

## 每种错误是什么样子 { #what-each-mistake-looks-like }

| 错误 | 症状 | 解决 |
| --- | --- | --- |
| 目标解析为空 | 第一次触发时出现 `[TD I/O] <node>.<output>: target '<name>' matches no node` | 让目标和实体的 targetname 完全一致，包括大小写 |
| 目标没有这个输入 | Godot 输出里出现 `[TD I/O] <node> has no input '<name>'` | 选一个目标的类里有的输入，见[实体参考](entities/README.md) |
| 实体从不发出的输出 | 构建地图时出现一次 `[TD I/O] <node> has no signal '<output>'` | 选一个类里有的输出，或者在代码里用 `TrenchDotIO.fire_output()` 触发它 |
| 运行时生成或重命名的实体 | 用它的 targetname 找不到 | 调用 `TrenchDotIO.invalidate()`，见[运行时的变化](parameters.md#runtime-changes) |
