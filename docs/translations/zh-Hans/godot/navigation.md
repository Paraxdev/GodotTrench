---
source: 7e5d26a9fedbc9f43e9edcb92409992b19d24676
---

# 导航 { #navigation }

`TrenchDotNav` 会从构建好的地图烘焙出一份导航网格，这样敌人和 NPC 就能在其中找路。

```gdscript
map.build()
var region := await TrenchDotNav.bake_region(map)
```

`bake_region` 在工作线程上烘焙，在地图下添加一个名为 `navigation` 的 `NavigationRegion3D`，等到其上的寻路可以使用时才返回。传入一个装着多张地图的节点，就能把它们一起烘焙进同一份网格。重新构建会释放这个区域，所以构建之后要再烘焙一次。

## 哪些会算进去 { #what-counts }

烘焙读取的是地图的碰撞，而不是它的网格，所以玩家会撞上的东西，就是代理要绕开的东西。

| 地图中的内容 | 在导航网格中 |
| --- | --- |
| 世界笔刷、`func_detail`、地形和道具碰撞 | 算 |
| 裁剪笔刷 | 算，它们像墙一样挡路 |
| 幻象笔刷、触发器和物理道具 | 不算 |
| 门、按钮、平台和火车 | 不算，所以关着的门不会把门洞封死 |
| `navigation_ignore` 组里的物体 | 不算 |

也就是说，路径会穿过关着的门，所以代理走到门前时得自己把门打开。

## 代理尺寸 { #agent-size }

`TrenchDotNav.profile(radius, height, max_climb, max_slope)` 会给你某一种代理尺寸的设置。半径让路径远离墙壁，高度排除过低的天花板，*max_climb* 是代理能走上去的最高台阶，*max_slope* 是最陡的斜坡。尺寸以米为单位，坡度以度为单位：

| 参数 | 默认值 |
| --- | --- |
| `radius` | 0.3 |
| `height` | 1.8 |
| `max_climb` | 0.3 |
| `max_slope` | 45 |

网格使用 0.1 米的单元格，并把尺寸对齐到这个单元格上。把配置作为第二个参数传入，比如给一个又宽又矮的生物：

```gdscript
var region := await TrenchDotNav.bake_region(map, TrenchDotNav.profile(0.5, 1.2))
```

（`bake_region` 会把导航地图的单元格大小设成这次烘焙用的大小，那张地图上的其他区域必须和它一致。）

## 缓存 { #caching }

一张大地图的第一次烘焙要花几秒钟。结果会按碰撞和设置为键，缓存在 `user://trenchdot_navigation` 里，所以下次用同一张地图启动时，只要一瞬间就能加载。把 `false` 作为第三个参数传入，就能不用缓存来烘焙。

## 检查地图 { #checking-a-map }

`TrenchDotNav.bake(map)` 在调用它的线程上烘焙，并返回 `NavigationMesh`，适合工具和测试。`TrenchDotNav.islands(nav)` 把它互不相连的部分按多边形索引列出来，最大的排在最前。

一个本该走得到、却孤零零成了一座孤岛的房间，通常是门洞被某个低矮的东西挡住了，或者有一级台阶比 `max_climb` 还高。墙顶和厚墙里面的地面也会显示成小孤岛。

> [!WARNING]
> Godot 在搜索了 4096 个多边形之后就会停止寻路，并返回到目前为止的路径，在大地图上这看起来就像一个走不到的地点。`TrenchDotNav.path(node, from, to)` 没有这个限制。对于代理，请把 `NavigationAgent3D` 上的 `path_search_max_polygons` 设为 0。

## 在 TrenchDot 里查看 { #seeing-it-in-trenchdot }

不用运行游戏，直接在 TrenchDot 里就能看到代理会走到哪里。

1. 在 Godot 里打开场景

    把地图保存在项目里面，然后用 Godot 编辑器打开这个项目，并打开一个其 `FuncGodotMap` 构建这张地图的场景。

2. 打开可行走区域

    进入*视图 > 可行走区域 > 显示可行走区域*。代理尺寸就在它的正下方，最下面一行会告诉你多少地方可以走，以及有多少座孤岛被隔开了。

    ![“视图”菜单，“可行走区域”子菜单已展开：显示可行走区域，然后是 半径 0.30 m、高度 1.80 m、最大爬升 0.30 m、最大坡度 45.00 度，以及 尚未烘焙](../assets/navigation/menu-walkable-area.png)

Godot 通过 [Live Link](live-link.md) 烘焙场景，TrenchDot 把结果画在 3D 和 2D 视图里。（如果无法烘焙，状态栏会说明原因，比如地图必须先保存到项目里。）

这些颜色的含义如下：

| 颜色 | 含义 |
| --- | --- |
| 蓝绿色 | 最大的连通区域，大多数代理都会待在这里 |
| 朱红色 | 绿色区域上的代理到不了的孤岛。找一找有没有比*最大爬升*更高的台阶、比*半径*的两倍还窄的缺口，或者比*最大坡度*更陡的斜坡 |

每次编辑之后过一小会儿，叠加层就会重新烘焙。没有[实时模式](live.md)时，Godot 会先按所示构建地图，就像*在 Godot 中构建*那样。

> [!TIP]
> 墙顶和屋顶会显示成小块的朱红色孤岛。要找的是紧挨着绿色地面的朱红色地面。

<div class="td-mcp" markdown>

## MCP { #mcp }

[MCP 服务器](../mcp.md)让代理和脚本可以驱动编辑器。它们用 `walkability` 工具来检查地图，该工具会返回各座孤岛，附带以地图单位计的面积和范围。

</div>
