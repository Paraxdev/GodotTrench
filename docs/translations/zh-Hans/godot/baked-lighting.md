---
source: 5d04cb3d9a7e005f4a371160c23d337e8ee707a7
---

# Godot 中的烘焙光照 { #baked-lighting-in-godot }

当你[在编辑器里烘焙一张地图](../editor/light-baking.md)时，光照贴图会保存在 `.tdm` 里面。Godot 自己不烘焙任何东西，每次构建只是把存好的烘焙结果变成 Godot 自己的光照贴图节点。

## 构建会添加什么 { #what-the-build-adds }

| 部分 | 它得到什么 |
| --- | --- |
| 一个名为 `baked_lighting` 的 `LightmapGI` | `FuncGodotMap` 的子节点，里面装着光照贴图、太阳阴影遮罩和光照探针 |
| 烘焙过的网格 | 它们的光照贴图坐标作为 UV2，以及静态 GI 模式，流式加载的区块也包括在内 |
| 灯和太阳 | 取自它们的 `bake_mode` 或 `sun_bake_mode` 的 Godot 烘焙模式，参见[光照键](../editor/light-baking.md#light-keys) |

太阳设为 `bounce` 时，太阳保持实时，烘焙出的阴影遮罩会给超出它阴影距离的地方着色。

烘焙之后你改动过的面，会退回到地图的平均光照，所以它们看起来还说得过去，但是很平，直到你再烘焙一次。

## 光照探针 { #light-probes }

会动的物体，比如玩家、道具和门，从它们周围的光照探针取光，这样它们就能接收到烘焙出来的反弹光，融入场景。

烘焙会在烘焙几何体周围的包围盒上，按 2 米的网格放置探针，并丢掉落在实体内部的那些。在网格太稀疏的地方，比如窄走廊或楼梯，可以添加一个 `light_probe` 实体来多放一个。一张地图最多有大约 1000 个探针，手动放置的也算在内，所以在更大的地图上，网格会变稀。

## 探针实体 { #probe-entities }

| 实体 | 种类 | Godot 节点 | 适用于 |
| --- | --- | --- | --- |
| `light_probe` | 点 | `LightmapProbe` | 烘焙时多加一个光照探针，见上文 |
| `env_reflection_probe` | 笔刷体积 | 大小与笔刷相同的 `ReflectionProbe` | 在光亮的地板和金属上，得到与房间相符的反射 |
| `env_voxel_gi` | 笔刷体积 | 大小与笔刷相同的 `VoxelGI` | 跟随移动的灯和物体的实时反弹光 |

这三个都能在“实体”标签页里找到。把 `light_probe` 拖进视图。对于那两个体积，围着房间画一个盒子，然后右键点击卡片，选择**将所选笔刷转为它**。

![“实体”标签页过滤到 env_，env_voxel_gi 卡片上点了右键，“将所选笔刷转为它”高亮显示](../assets/baked-lighting/entities-turn-brushes-into.png)

笔刷只提供这个盒子，构建时会把它移除。这三个实体的键都列在[道具、灯光与特效](../gameplay/entities/effects.md#light_probe-env_reflection_probe-and-env_voxel_gi)里。

`env_voxel_gi` 每次地图构建时，都会在 Godot 编辑器里烘焙。体积很大时，请把 `bake_on_build` 设为 0，改为用 `VoxelGI` 节点上的**烘焙**按钮手动烘焙。游戏运行时构建的地图会跳过这次烘焙。

## 使用别的光照贴图工具 { #using-another-lightmapper }

想改用 Godot 自己的 `LightmapGI` 来烘焙？取消勾选 `FuncGodotMap` 上的 **Use Baked Lighting**，就能把存好的烘焙结果排除在外。这样构建就不会添加 `baked_lighting` 节点，也不会动灯的烘焙模式。在 *Build Flags* 里勾选 *Unwrap UV2*，让网格拿到 Godot 烘焙所需的 UV2。
