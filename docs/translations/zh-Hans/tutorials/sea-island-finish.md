---
source: 5d131fe08a12db6615a97a4cbd455746685c5f02
---

# 海岛 5：码头和 Godot { #sea-island-5-pier-and-godot }

## 码头 { #the-pier }

1. 给它单独的图层

    点击大纲顶部的**添加图层**。新图层会成为当前图层，所以码头会落在它上面，你可以把它和地形分开隐藏或锁定。

    ![大纲里悬停着添加图层，以及它的提示：新建图层，它会成为当前图层，新对象会放进去](../assets/sea-island/outliner-add-layer.png)

2. 把它搭起来

    按 **Q** 切换到选择工具，因为用散布工具拖动仍然会绘制模型。然后从沙滩中间向海里画几个普通笔刷，尺寸参照下面的表格。

3. 加上道具

    打开 *文件 > 导入 > 模型道具 (.bbmodel, .glb)…*，从演示项目的 `models/polyhaven` 文件夹里选模型：甲板上放两个 `containers/wooden_crate_01` 和一个 `containers/wine_barrel_01`，靠着一根柱子放一个 `nautical/lifebuoy`，水里放一个 `nautical/ocean_buoy`。每一个都会落在 3D 光标处，再从那里移到位置上。

    ![文件菜单，导入已展开，模型道具 (.bbmodel, .glb)… 处于悬停状态](../assets/sea-island/menu-import-model-prop.png)

| 部分 | 尺寸 | 材质 |
| --- | --- | --- |
| 甲板 | 宽 128，长 1100，厚 8，顶面高出水面 48 | `showcase/planks` |
| 柱子 | 16 个单位见方，两排，每 200 个单位一根，从水下 320 一直到甲板 | `showcase/beam` |
| 栏杆 | 远端的两根短柱，高 40 | `showcase/beam` |

道具是通过 `res://` 路径引用它的模型的，所以文件必须在打开的 Godot 项目里面。如果你选了别处的文件，编辑器会先提议把它复制到 `res://models`。

![从海上看到的码头](../assets/sea-island/pier.jpg)

## 在 Godot 里看看 { #see-it-in-godot }

把地图保存在你的 Godot 项目里，然后用 `FuncGodotMap` 构建它，参见[构建地图](../godot/building.md)。地图的每个部分会变成不同种类的节点：

| 地图部分 | 你在 Godot 里得到什么 |
| --- | --- |
| 地形 | 一个分成若干块的 `TrenchDotTerrain`，每块都是混合四个图层的网格，并且有自己的碰撞 |
| 大海 | `func_illusionary` 笔刷，可见但没有碰撞 |
| 树、灌木和巨石集合 | MultiMesh，另外每个模型有一个静态体，为每个实例提供一个碰撞形状，所有实例共用同一个形状 |
| 草集合 | 没有碰撞的 MultiMesh，会随距离淡出 |

![Godot 里完成的海岛](../assets/sea-island/godot-overview.jpg)

![Godot 里的码头](../assets/sea-island/godot-pier.jpg)
