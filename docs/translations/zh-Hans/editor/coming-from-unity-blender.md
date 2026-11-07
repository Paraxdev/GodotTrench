---
source: 84383292fb022d6e6369e9ad8b643a67a07d4a67
---

# 从 Unity 或 Blender 转过来 { #coming-from-unity-or-blender }

TrenchDot 是一个笔刷编辑器，和 TrenchBroom、Hammer 一脉相承。关卡一开始是你在网格上画出的实心方块，而不是导入的网格模型。你在 Unity 和 Blender 里学到的大部分东西在这里依然有位置，只是名字和按键不同。这一页把它们对应起来，[入门](../getting-started.md)则会带你做出第一个房间。

## 各个功能在哪里 { #where-things-are }

| 在 Unity 或 Blender 中 | 在 TrenchDot 中 |
| --- | --- |
| Hierarchy、Outliner | **大纲**，用[图层和编组](organizing.md)代替父子对象 |
| Inspector、N 面板 | **检查器**。它的位置和尺寸字段可以输入数值，按 Enter 应用 |
| Scene 视图、视口 | 3D 视图加上顶视图、前视图和侧视图。绘制和对齐在 2D 里最容易 |
| ProBuilder 形状、Add 菜单 | 在任意视图里拖动来画一个方盒[笔刷](brushes.md)，按 Ctrl+Shift+B 可以生成拱门、楼梯和圆柱 |
| 布尔修改器 | *笔刷 > CSG*：减去（Ctrl+K）、合并、相交和挖空。它们会立刻改动笔刷，没有修改器堆栈 |
| 编辑模式 | 笔刷用 **顶点** 工具，[网格](meshes.md)用 **网格** 工具，都在 Tab 上 |
| 预制体 | [预制体](organizing.md#prefabs)，保存为独立的地图，并在每个实例中同步更新 |
| 关联复制（Alt+D） | 关联复制，Ctrl+Shift+D，这样副本共用同一份几何体 |
| 单位，1 米 | 32 个地图单位是一米，默认的网格步长是 16 |

精确的数值输入到右侧的检查器里。选中某个东西，输入新的位置或尺寸，按 Enter。位置指的是所选对象的最低角，而不是它的中心或轴心。

![选中了一个笔刷的检查器，它的位置和尺寸以 X、Y、Z 字段显示在选区分区上方](../assets/coming-from-unity-blender/inspector-position-size.png)

## 不同的按键 { #keys-that-differ }

默认按键参照 TrenchBroom，其中有三个会让 Blender 用户犯迷糊：

* **G** 是地形的雕刻工具，不是抓取。
* **R** 和 **T** 选择旋转和缩放工具，它们通过手柄作用在所选对象上，而不是跟随鼠标。
* **Tab** 就地编辑所选对象。

要移动东西，在任意视图里拖动它，或者使用 3D 视图里的变换小工具，需要精确数值时用检查器。

想保留你的 Blender 按键？把 *文件 > 偏好设置 > 快捷键预设* 切换成 Blender。

![偏好设置窗口中的快捷键预设下拉菜单，展开后有 TrenchBroom、Hammer 和 Blender](../assets/coming-from-unity-blender/keymap-preset.png)

它会加上 A 全选、X 删除、Shift+D 复制、H 和 Alt+H 隐藏和显示、S 缩放工具以及 Shift+A 打开形状生成器。G 仍然是雕刻，而且移动过程中输入数字只在网格工具里有效。完整列表在[其他预设](shortcuts.md#other-presets)。

没有小键盘视图。顶视图、前视图和侧视图始终是打开的，Shift+空格会最大化指针所在的那个，*视图 > 视图* 则在一个、两个和四个视图之间切换。

## 可以沿用的习惯 { #habits-that-carry-over }

在网格工具里，按键遵循 Blender：1、2 和 3 分别对应顶点、边和面，G、R 和 S 配合 X、Y 或 Z 锁定一条轴，E 挤出，Ctrl+R 环切，Ctrl+B 倒角。

按住右键拖动并配合 WASD，可以像 Unity 的 Scene 视图那样飞行浏览 3D 相机，F 聚焦所选对象。Ctrl+Z 和 Ctrl+Shift+Z 撤销和重做，F2 重命名。
