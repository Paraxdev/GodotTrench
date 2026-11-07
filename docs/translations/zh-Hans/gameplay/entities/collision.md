---
source: d6310713b2a9d65be25e506f0179132b8e0e10fd
---

# 碰撞 { #collision }

玩法包里的每个笔刷实体和每个模型实体，不用你设置任何东西就有碰撞。门、按钮、升降机、列车和触发器每个笔刷得到一个凸包，`prop_model`、`prop_physics`、`npc_walker` 和散布出来的道具则得到一个根据它们的模型构建的形状。只有默认值不合适时，才需要动下面的键。

## 这些键 { #the-keys }

选中实体，在检查器里找到这几行。把鼠标悬停在某一行上，就能读到每个选项的作用。

![func_door 的检查器，碰撞菜单展开，列出 default、none、box、convex hull、convex decomposition 和 trimesh，下面是 collision_layer 和 collision_mask 两行](../../assets/collision/inspector.png)

| 键 | 默认值 | 作用 |
| --- | --- | --- |
| `collision` | default | 形状：default、none、box、convex hull、convex decomposition 或 trimesh |
| `collision_layer` | 空 | 物体所在的物理层，写成层编号，比如 `1, 3`。`none` 表示不在任何层上 |
| `collision_mask` | 空 | 物体检测的物理层，写法相同。`none` 表示什么都不检测 |

层或掩码留空就保持默认值：笔刷实体用实体定义里的层和掩码，触发器从地图设置里取 *Trigger Collision Mask*，模型实体则用下面的项目设置。

## 挑选形状 { #picking-a-shape }

| 形状 | 笔刷实体 | 模型 |
| --- | --- | --- |
| default | 类的形状，每个笔刷一个凸包 | 项目设置，除非改过，否则每个网格一个凸包。`prop_physics` 用的是由 `size` 得出的盒子 |
| none | 没有碰撞 | 没有碰撞 |
| box | 包住所有笔刷的一个盒子 | 包住模型的一个盒子，是最便宜的形状 |
| convex hull | 每个笔刷一个凸包 | 每个网格一个凸包，适合方方正正的模型 |
| convex decomposition | 和 convex hull 一样，笔刷本来就是凸的 | 好几个跟着凹形模型（比如拱门）走的凸包，构建起来更慢 |
| trimesh | 每一个面，只适用于静态物体 | 每一个三角形，精确但开销最大 |

Trimesh 只对静态物体有效。门、按钮、升降机和列车是可动画物体，算作静态，所以可以用它。触发器、`prop_physics` 和 `npc_walker` 会改为构建凸包，把触发器设成 trimesh，地图构建时会报出一条警告。

如果 `prop_model` 的模型本身已经带有碰撞体，就保留那个碰撞体，不再自己构建形状。

## 项目默认值 { #project-defaults }

键为空的模型实体，遵循 Godot 的 *Project Settings* 里 *Trenchdot > Collision* 下的三项设置。启用 TrenchDot 插件之后它们才会出现。

| 设置 | 默认值 | 作用 |
| --- | --- | --- |
| *Model Shape* | convex | `prop_model` 和 `npc_walker` 上 default 所代表的形状 |
| *Model Layer* | 1 | 模型物体所在的层 |
| *Model Mask* | 1 | 模型物体检测的层 |

散布集合在它自己的 *Collision* 一行里选形状，见[散布](../../editor/scatter.md)。

## 你自己的笔刷实体 { #your-own-brush-entities }

你自己定义的笔刷实体，会按它的 *Node Class* 指名的物体来产生碰撞。*Node Class* 留空时，取它的脚本所继承的物体，没有脚本、或者脚本继承 `Node3D` 时，它就成为 `StaticBody3D`。只有脚本继承某个别的节点，比如 `Marker3D`，才会让它没有碰撞，构建时会对此发出警告。见[用 GDScript 写自定义实体](../custom-entities.md)。

编辑器视图不会画出碰撞形状。Godot 的 *Debug > Visible Collision Shapes* 会在游戏运行时显示它们，无论是在实时会话里还是在完整构建里。笔刷实体和 `prop_model` 把形状保存在构建好的场景里，`prop_physics` 和 `npc_walker` 则在每次场景加载时构建它们的形状，就像它们加载自己的模型那样。如果 `prop_model` 或 `npc_walker` 的模型场景本身已经带有物理体，就保留那个物理体，不再自己构建形状。
