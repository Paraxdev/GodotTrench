---
source: 3df4b9a4462dd78307ebb473881223aa02ea1321
---

# 夜间照明 { #night-lighting }

做夜晚的地图时，你要把全局光调暗，这样就由你放置的灯来干活。

## 把地图调暗 { #darken-the-map }

按 Escape，或者在视图的空白处点击，让任何东西都不处于选中状态。右侧的检查器这时会显示*地图 (worldspawn)*，也就是整张地图的设置。

设置下面这些键。如果某个键还没有列出来，就在底部的*键*输入框里输入它，把值填在旁边，然后点击**添加**。

| 键 | 默认值 | 夜晚怎么做 |
| --- | --- | --- |
| `sun_energy`、`sun_color` | 1.1，`255 245 224` | 调低强度，选一种淡蓝色，太阳看起来就像月亮 |
| `ambient_energy`、`sky_energy` | 1.0，1.0 | 两个都调低，这样没被照亮的地方和天空都会变暗 |
| `glow_intensity` | 0 | 大于 0 时，灯和[发光材质](materials.md#glowing-materials)在 Godot 里会产生泛光 |
| `ssr` | 0 | 设为 1 会打开屏幕空间反射，这样潮湿的街道和光亮的地板就会映出灯光 |

夜晚街区展示地图用的就是这些：

![检查器显示夜晚街区的地图 (worldspawn)：ambient_energy 0.8、glow_intensity 0.9、sky_energy 0.6、淡蓝色的 sun_color 和 sun_energy 0.16](../assets/night-lighting/worldspawn-night.png)

## 在光照预览里检查 { #check-it-in-lit-preview }

按 F3，或者点击工具栏里的光照预览，就能看到带着灯光、天空和雾的地图。

![工具栏上，鼠标悬停在“光照预览”按钮上，显示它的工具提示“着色：光照预览 (F3)”](../assets/night-lighting/toolbar-lit-preview.png)

你应该能看到灯、亮着的窗户和发光的招牌在暗沉的天空下格外醒目：

![光照预览里的夜晚街区街道：路灯、亮着的窗户，以及暗沉天空下发光的 Corner Shop 招牌](../assets/night-lighting/street-lit-preview.jpg)

光照预览会显示上面所有的键，只有泛光和反射除外，所以那两项请到 Godot 里检查。想要灯的反弹光，又不想每一帧都为它付出开销，就[烘焙光照](../editor/light-baking.md)。

## 灯具 { #lamp-fixtures }

一盏灯是一个 `light` 或 `light_spot` 实体，加上这盏灯的几何体和它会发光的灯泡。给这个几何体一个 targetname，并把它填进灯的 `fixture` 键里（末尾的 `*` 匹配前缀）。

这样几何体不用连线就会跟着灯走：灯亮时显示，灯灭时隐藏。把 `fixture_off` 设为 `dark`，则会让它保持可见，只是关掉它的发光。

![一盏灯的检查器，targetname 为 alley_lamp，fixture 为 alley_bulb，fixture_off 设为 dark（发光关闭）](../assets/night-lighting/light-fixture-keys.png)

其他的灯光键，参见[道具、灯光与特效](../gameplay/entities/effects.md)。

<div class="td-mcp" markdown>

## MCP { #mcp }

展示地图是由 [MCP 脚本](../mcp-scripts.md)通过 [MCP 服务器](../mcp.md)驱动编辑器构建出来的。`examples/mcp/night_district.json` 和 `examples/mcp/withered_city.json` 就是这样构建出来的完整夜晚地图。

</div>
