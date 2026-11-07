---
source: 0215c0eddc11970f765e58b4ce44056eebf2815e
---

# 大地图流式加载 { #streaming-big-maps }

大型室外地图可以把远处的东西藏起来，这样地图的开销取决于玩家看得到的部分，而不是它包含的全部内容。构建时会把地图的网格按网格划分归入各个区块，只有靠近相机的区块才会被绘制。

## 打开它 { #turning-it-on }

1. 什么都不选

    按 Escape，让任何东西都不处于选中状态。右侧的检查器这时会显示*地图 (worldspawn)*。

2. 添加 `chunk_streaming`

    在底部的*键*输入框里输入 `chunk_streaming`，在*值*里输入 `1`，然后点击**添加**。（如果它已经在列表里，直接在那里打开就行。）

    ![worldspawn 检查器，键输入框里是 chunk_streaming，值输入框里是 1，鼠标悬停在“添加”按钮上](../assets/streaming/worldspawn-add-key.png)

    你应该能在其他 worldspawn 键旁边看到它：

    ![勾选了 chunk_streaming 的 worldspawn 检查器](../assets/streaming/worldspawn-chunk-streaming.png)

3. 保存地图

    下一次构建会在地图里添加一个 `TrenchDotStreamer`。

（可选）用同样的办法添加 `chunk_size` 和 `load_radius`：

| 键 | 默认值 | 含义 |
| --- | --- | --- |
| `chunk_size` | 2048 | 区块大小，以地图单位计，即 64 米。区块越小，隐藏得越精确，但检查起来开销越大 |
| `load_radius` | 8192 | 区块离相机多远时仍然保持可见，以地图单位计，即 256 米 |

流式加载只会隐藏网格。碰撞和脚本在各处照常运行，所以它省下的是渲染开销，而不是物理或逻辑的开销。叠加层、运行时生成的内容，以及活动门或火车上的任何东西，都不会被隐藏。

流式加载器会跟随视口当前的相机，没有相机时则跟随 `info_player_start`。想把它固定在某一台相机上，请在构建之后用代码设置它的 `camera_path`，因为重新构建会替换掉这个流式加载器。它只在游戏里运行，所以 Godot 编辑器里始终显示完整的地图。

随着区块的来去，流式加载器会发出 `area_loaded(key, bounds)` 和 `area_unloaded(key, bounds)`，这样你就可以顺便流式加载自己的内容：

```gdscript
streamer.area_loaded.connect(func(key, bounds): spawn_wildlife(bounds))
```

## 流式加载分区 { #streaming-sections }

想把好几张地图当成一个世界来玩，又不想有加载画面？把它拆成分区，再用连接房间把它们接起来。场景树里同一时间只有一个分区。玩家探索的时候，下一个分区在后台加载，等玩家走过连接房间时换进来，两边对得严丝合缝，玩家周围的房间保持原样。这需要[玩法实体包](../gameplay/entities/README.md#installing-and-customizing)。

1. 建一个连接房间

    一条走廊，两侧各有一个拐弯，这样从中间看不到任何一端。在中间放一个 `info_landmark`，再放两个 `func_section_stream` 体积，各盖住一半，在中间相接。把它加进[预制体库](../editor/prefab-library.md)，这样每份副本都完全一样。

2. 把它粘贴到两张地图相接的地方

    在每张与其他地图相接的地图里，把连接房间粘贴到两张地图相接的位置，并给它的地标命名，比如 `lm_hall_cellar`。把每一半的 `section` 设为那一侧的分区，把 `landmark` 设为地标的名字。两张地图用的是同一份副本，名字相同，朝向也相同。

3. 在每个分区地图里烘焙光照

    SDFGI 只有在级联滚动之后才会注意到切换带进来的几何体，所以换进来的分区会一直是暗的。烘焙光照会随分区一起带来。

4. 给每个分区做一个场景

    每个分区一个场景，里面是它构建好的地图，按平常的做法来。

5. 添加流式加载器

    在你的世界场景里添加一个 `TDSectionStreamer` 节点，并设置它的 `first_section` 和玩家，参见 [TDSectionStreamer](../gameplay/entities/sections.md#tdsectionstreamer)。

> [!TIP]
> 让地标的位置保持为 480 地图单位的倍数，或者连接房间所有纹理的重复尺寸的任何倍数，这样两份副本里的纹理就会对得一样整齐。
