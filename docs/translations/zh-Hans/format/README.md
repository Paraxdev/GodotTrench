---
source: d3f0019b345d8a4b85cb01b22ccb9fd1cdee21d4
---

# .tdm 地图格式 { #the-tdm-map-format }

如果你要写读取或写入 `.tdm` 文件的工具，比如导入器、转换器或 CI 检查，这几页就是写给你的。做关卡本身用不到这些内容。

一个 `.tdm` 文件包含三样东西：worldspawn 属性、一点编辑器状态，以及一棵存放实际内容的节点树。

编辑器把它保存成由 zstd 压缩的数据块组成的二进制容器，所以文件很小，损坏的文件也依然能打开。旧版编辑器保存的是一份 UTF-8 JSON 文档，所有读取器两种都能加载。两种形式存放的是同一棵树，所以这几页都用 `trenchdot --dump` 输出的 JSON 来展示它。

| 页面 | 这些情况请读它 |
| --- | --- |
| [容器结构](container.md) | 你要解析或写入二进制文件。它讲文件头、数据块，以及每个值怎样编码成字节 |
| [损坏的文件](recovery.md) | 你想知道损坏的文件里还有什么能加载，以及每个读取器在哪里报告丢失的内容 |
| [节点](nodes.md) | 你要遍历这棵树。它讲每个节点共有的结构、id 怎样工作，以及图层、组、实体和实例节点 |
| [几何体](geometry.md) | 你要读取或构建笔刷和网格，包括它们的面、纹理投影和置换 |
| [地形与散布](terrain-scatter.md) | 你要处理高度图地形和成组散布的模型 |
| [JSON 结构](json.md) | 你要处理可读的那种形式、在两种形式之间转换，或者读编辑器放进剪贴板的内容 |

想看能跑的代码？参考实现是 [`format.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/format.rs)，容器在 [`binary.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/binary.rs) 和 [`variant.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/variant.rs) 里。Godot 一侧，[`tdm_file.gd`](https://github.com/Paraxdev/trenchdot_func/blob/main/src/trenchdot/tdm_file.gd) 读取文件，[`tdm_parser.gd`](https://github.com/Paraxdev/trenchdot_func/blob/main/src/trenchdot/tdm_parser.gd) 负责构建。

## 顶层 { #top-level }

| 键 | 类型 | 何时写入 | 含义 |
| --- | --- | --- | --- |
| `format` | 字符串 | 总是 | 永远是 `"trenchdot-map"`。用来把地图和其他 JSON 区分开 |
| `version` | 整数 | 总是 | 地图版本，目前是 1 |
| `properties` | 字符串到字符串的对象 | 不为空时 | worldspawn 的键，按名称排序 |
| `editor` | 对象 | 不全是默认值时 | 编辑器的视图状态。永远不会导出到 Godot |
| `textures` | 对象 | 有纹理带设置时 | 按材质名存放的[纹理设置](geometry.md#texture-settings) |
| `layers` | 节点数组 | 总是 | 地图的[图层节点](nodes.md#layer)，按顺序排列。其他所有内容都挂在它们下面 |
| `lightmap` | 对象 | 烘焙过的 JSON 地图 | 烘焙出的光照，二进制地图把它放在 [LMAP 数据块](container.md#lmap)里 |

`layers` 里只会读取 `layer` 节点，没有任何图层的地图在加载时会得到一个 `Default` 图层。

worldspawn 的值永远是字符串，和 `.map` 文件里一样。Godot 构建地图时，`TrenchDotEnvironment` 会把这些键变成天空、雾和太阳：

| 键 | 控制什么 |
| --- | --- |
| `sun_angles`、`sun_color`、`sun_energy` | 平行光的太阳：朝向、颜色和亮度 |
| `ambient_color`、`ambient_energy` | 照到太阳照不到的表面上的光，这样阴影就不会漆黑一片 |
| `sky_top_color`、`sky_horizon_color`、`sky_ground_color`、`sky_energy` | 程序化天空的渐变及其亮度 |
| `sky_panorama` | 在 Godot 里替换程序化天空的 `res://` 图像。编辑器预览和它的烘焙仍然使用天空颜色 |
| `fog_color`、`fog_density` | 距离雾 |
| `glow_intensity`、`ssr` | 明亮表面上的泛光，以及 `ssr` 为 `1` 时的屏幕空间反射 |

如果你的游戏自带 `WorldEnvironment`，把 `environment` 设为 `sun_only`，就只会构建太阳。设为 `0` 或 `none` 则什么都不构建，见[环境与太阳](../godot/building.md#environment-and-sun)。`sky_source` 记录导入地图的天空面原来用的是哪张纹理。

`editor` 对象是只在编辑器里有意义的视图状态：

| 键 | 类型 | 默认值 | 含义 |
| --- | --- | --- | --- |
| `cameras` | 对象，键为 `"1"` 到 `"9"` | 空 | 保存的相机位置，每个是 `{"position": [x, y, z], "yaw": r, "pitch": r}`，角度单位是弧度 |
| `cordon` | `{"min": [x, y, z], "max": [x, y, z]}` | 无 | 以地图单位表示的隔离框，用来处理大地图的某一部分 |
| `cordon_enabled` | 布尔 | `false` | 隐藏隔离框外的对象，并让它们不进入按隔离框的导出 |
| `bake` | `{"quality", "texel_size", "softness", "backend"}` | 无 | 地图上次[烘焙](../editor/light-baking.md)时用的选项，下次烘焙从它们开始 |
| `graph_frames` | 数组 | 空 | [逻辑图](../gameplay/logic-graph.md#frames)的框，每个是 `{"title", "rect": [x, y, width, height], "color"}`，单位是图的点。损坏的框在加载时被丢弃 |

## 版本与兼容性 { #versions-and-compatibility }

`format` 和 `version` 描述的是数据。文件头里的容器版本是另一回事，只有数据块布局变了它才会变。

遇到没有完全看懂的文件时，各个读取器是这样做的：

| 情况 | 编辑器 | Godot 插件 |
| --- | --- | --- |
| `format` 不同 | 拒绝这个文件 | 构建报错 |
| `version` 更新 | 拒绝这个文件 | 构建报错，提示更新插件 |
| 缺少 `version` | 当作最旧的版本读取 | 相同 |
| 未知的键 | 忽略它，下次保存后它就没了 | 忽略它 |
| 未知的数据块 | 保留它，保存时原样写回 | 跳过它 |
| 面索引越界 | 拒绝这张地图，并指出是哪个节点 | 跳过那个笔刷 |
| 置换的 `heights` 大小不对 | 拒绝这张地图，并指出是哪个节点 | 把这个面构建成平的 |

编辑器保存时会丢掉未知的键。所以如果你加的数据不能让旧版编辑器弄丢，就得要么提升地图版本，旧版编辑器会拒绝打开；要么另开一个自己的数据块，旧版编辑器会原封不动地保留它。见[其他数据块](container.md#other-chunks)。

## 坐标与单位 { #coordinates-and-units }

位置使用 Godot 的坐标轴：Y 向上，X 向右，Z 朝向观察者。构建时会除以地图设置里的 `inverse_scale_factor`，默认是 32，所以 32 个地图单位等于一米。

| 量 | 单位 |
| --- | --- |
| 位置、尺寸、高度、间距、格子大小 | 地图单位 |
| 实体、实例和散布的角度 | 度：俯仰绕 X，偏航绕 Y，翻滚绕 Z，按 YXZ 顺序应用，和 Godot 的 `rotation_degrees` 一样 |
| 相机书签的偏航和俯仰 | 弧度 |
| UV 偏移 | 纹素 |
| UV 缩放 | 每纹素的地图单位 |
| 网格的 `uvs` | 纹理宽度，所以 1 就是纹理完整重复一次 |
