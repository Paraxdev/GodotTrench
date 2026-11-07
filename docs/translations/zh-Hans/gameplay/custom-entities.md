---
source: 0c1b9cb979394402d977a6f647062a1558b8af7c
---

# 用 GDScript 写自定义实体 { #custom-entities-in-gdscript }

任何脚本都可以成为实体。给每个输出声明一个**信号**，给每个输入声明一个**方法**，然后在事情发生时发出信号。下面这个警报有一个输出 `tripped`，和两个输入 `trip` 与 `disarm`：

```gdscript
@tool
class_name Alarm extends Node3D

signal tripped(activator: Node)

@export var armed := true

func _func_godot_apply_properties(props: Dictionary) -> void:
	armed = TrenchDotIO.to_bool(props.get("armed", armed))

func trip(activator: Node = null) -> void:
	if armed:
		tripped.emit(activator)

func disarm() -> void:
	armed = false
```

请把激活者作为信号的第一个 `Node` 参数，这样后面的链仍然知道是谁启动了它们。

地图的键会传到 `_func_godot_apply_properties` 里。请通过 `TrenchDotIO.to_bool`、`to_vector3` 和 `to_color` 来读取，它们既接受文本，也接受已经转换过的值。你在那里添加的节点会随场景一起保存。你在 `_ready` 里添加的节点不会保存，它们在每次场景加载时都会重新生成，适合只有运行中的游戏才需要的东西。对于带有 *Scene File* 的实体，Godot 会按场景文件里的样子保存来自那个场景的节点，所以你对它们做的修改会丢失。你添加的节点仍然会保存，而对场景自身节点的修改应该放在 `_ready` 里。

脚本需要 `@tool`，因为地图是在 Godot 编辑器里构建的，而在那里只有 tool 脚本会运行它们的方法。

## 让它可以放置 { #making-it-placeable }

编辑器和 Godot 通过你项目的 FGD，也就是实体定义的列表，来了解实体类。每个定义都是一个独立的资源，你选的种类决定了 Godot 构建出什么：

| 你的实体 | 定义 | 要设置什么 |
| --- | --- | --- |
| 点实体，位于某一处，比如灯或拾取物 | `FuncGodotFGDPointClass` | 把 *Script Class* 设为你的脚本。想改为生成一个场景，就设置 *Scene File*，这时 *Script Class* 会替换场景根节点的脚本 |
| 笔刷实体，由笔刷构成，比如门或触发器 | `FuncGodotFGDSolidClass` | 把 *Script Class* 设为你的脚本，把 *Node Class* 设为它继承的物体：`StaticBody3D`、`AnimatableBody3D` 或 `Area3D`。留空时，Godot 会构建脚本所继承的物体，如果脚本继承 `Node3D` 或没有脚本，则构建 `StaticBody3D`。见[碰撞](entities/collision.md) |

这是上面那个警报的定义，保存为 `res://entities/alarm.tres`，放在它的脚本 `alarm.gd` 旁边：

```
[gd_resource type="Resource" script_class="FuncGodotFGDPointClass" load_steps=3 format=3]

[ext_resource type="Script" path="res://addons/func_godot/src/fgd/func_godot_fgd_point_class.gd" id="1_class"]
[ext_resource type="Script" path="res://entities/alarm.gd" id="2_script"]

[resource]
script = ExtResource("1_class")
classname = "alarm"
description = "Trips when told to, until disarmed"
script_class = ExtResource("2_script")
class_properties = Dictionary[String, Variant]({
"targetname": "",
"armed": true
})
node_class = "Node3D"
```

把它添加到 FGD 的 *Entity Definitions* 里，你的地图设置和游戏配置要用同一个 FGD，见[项目设置](../godot/project-setup.md#your-own-resources)。之后游戏配置会把它导出到编辑器，它的信号成为输出，不以 `_` 开头的方法成为输入。如果实体仍然被构建成普通节点，请看[什么都不做的实体](../godot/entities-troubleshooting.md)。

想快速上手的话，打开**参考**面板，选一个和你的相近的类。在 *GDScript 类*标签页上，**在项目中创建**会把那份骨架写到 `res://entities/` 下。

![参考面板的 GDScript 类标签页，悬停在在项目中创建上并显示提示：把文件写到 res://entities/ 下，已存在则不写](../assets/custom-entities/reference-create-in-project.png)

在 *FGD.tres* 标签页上，**在项目中创建**会写出一个定义和它运行的脚本，二者都以类名前面加 `my_` 来命名，所以 `func_button` 会得到 `res://entities/my_func_button.tres` 和 `my_func_button.gd`。把这个 `.tres` 添加到你的 FGD 的 *Entity Definitions* 里，Godot 就会构建它。

![参考面板的 FGD.tres 标签页，悬停在在项目中创建上并显示提示：写出 res://entities/my_func_button.tres，一个有自己类名的定义，以及它运行的脚本](../assets/custom-entities/reference-create-definition.png)

## 在代码里调用 I/O { #calling-io-from-code }

游戏代码可以使用和地图连接相同的机制：

```gdscript
for door in TrenchDotIO.find_targets(self, "main_door", null):
	TrenchDotIO.invoke(door, &"open", "", null)

TrenchDotIO.fire_output(self, &"tripped", activator)
```

请优先用 `invoke`，而不是直接调用方法，它会像地图连接那样让参数适配方法的参数，并处理内置输入。`fire_output` 会手动触发那个输出上的每条连接，在实体没有同名信号时很有用。
