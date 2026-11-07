---
source: 602da12da979c292b377fb3a10cadcc22f1babd5
---

# C# 实体 { #c-entities }

C# 类不需要 `.tres` 定义，就可以成为可放置的实体。插件会读取你的源文件：信号变成输出，导出的成员变成键，带标记的方法变成输入。

运行 C# 实体需要 Godot 的 .NET 版本。编辑器无论如何都会列出它们，但只有 Godot 运行 .NET 版本、C# 项目已构建并且这个类带有 `[GlobalClass]` 时，才能构建出来。否则 C# 笔刷实体会被构建成没有碰撞的普通 `Node3D`，点实体则是一个空节点，Godot 的输出面板会说 `Node Class ... not found`，并列出这三件要检查的事。

## 1. 添加辅助 { #1-add-the-helper }

打开**参考**面板（*视图 > 面板 > 参考*），按它左上角的**向项目添加 C# 辅助**。

![参考面板，悬停在向项目添加 C# 辅助上，提示写着：写出 TrenchDot.cs，C# 实体使用的桥接文件](../assets/csharp/reference-csharp-helper.png)

它会写出 `res://addons/func_godot/csharp/TrenchDot.cs`，里面是各种特性。没有它，你的实体代码无法编译。

## 2. 声明一个实体 { #2-declare-an-entity }

```csharp
[GlobalClass]
[TrenchDotEntity("npc_guard", Description = "Patrolling guard",
    Color = "#ff4040ff", Size = "-16 0 -16 16 64 16")]
public partial class NpcGuard : CharacterBody3D
{
    [Signal] public delegate void AlertedEventHandler(Node activator, int level); // output "alerted"
    [Export] public float WalkSpeed { get; set; } = 3.5f;                          // key "walk_speed"
    [TrenchDotInput] public void Alert(Node activator) { }                        // input "alert"
}
```

| 部分 | 为什么重要 |
| --- | --- |
| `[GlobalClass]` | 必须有，地图构建按类名创建节点 |
| `Size` | 编辑器为这个实体画出的方盒，先是最小点再是最大点，单位是地图单位 |
| `Solid = true` | 做成笔刷实体，比如门，而不是点实体 |
| `[TrenchDotInput]` | 标记输入。如果没有方法带它，每个首字母大写的公开 `void` 方法都是输入 |

名称在地图里是 snake_case，在 C# 里是 PascalCase。要自己读取键，请实现 `_func_godot_apply_properties(Dictionary props)`，并使用辅助类的读取函数，比如 `TrenchDot.Float(props, "walk_speed", 3.5f)`。

> [!TIP]
> 参考面板的 *C# 类*标签页会写出一份骨架，带有任意实体的信号、导出和输入，是你自己实体的快速起点。它不含 `[TrenchDotEntity]` 这一行，需要你自己加上。

## 插件去哪里找 { #where-the-addon-looks }

编辑器和地图构建各自会扫描文件夹来查找 C# 实体类，这些文件夹在两个地方设置：

| 使用者 | 设置 |
| --- | --- |
| 编辑器（导出游戏配置） | 游戏配置资源上的 *Csharp Source Dirs* |
| 地图构建 | 项目设置 `trenchdot/csharp_entity_dirs` |

两者默认都是 `res://`，并跳过 `addons`、`bin`、`obj` 和隐藏文件夹。

> [!NOTE]
> C# 类不能取代库里的实体。如果 FGD 已经定义了它的类名，以 FGD 为准。

## 从 C# 使用 I/O { #io-from-c }

```csharp
var guide = TrenchDot.FindTargets(this, "guide")[0];
guide.Connect("finished", Callable.From(() => GD.Print("cutscene over")));

foreach (Node barrel in TrenchDot.FindTargets(this, "barrel"))
    TrenchDot.Invoke(barrel, "ignite", "", this);

TrenchDot.FireOutput(this, "alerted", player);
```

参考面板的*在 C# 中使用*标签页会为任何实体生成这段代码。

<div class="td-mcp" markdown>

## MCP { #mcp }

通过 [MCP 服务器](../mcp.md)驱动编辑器的智能体或脚本，可以从 `code_reference` 工具得到同样生成的代码。

</div>
