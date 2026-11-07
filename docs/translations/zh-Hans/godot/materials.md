---
source: 6f3b7a09923ac7b7e6e7e7d49ae11666c6d47d02
---

# 材质 { #materials }

每个面都通过一条路径指向一种材质，这条路径是相对于地图设置里 *Base Texture Dir* 的，不带扩展名，比如 `showcase/planks`。

Godot 构建地图时，会在那个文件夹里找 `showcase/planks.tres`，如果你设置了 *Base Material Dir*，则改在那里找，找到后原样使用。没有 `.tres` 吗？那么插件会用这张图片做一个普通材质。TrenchDot 预览的是同一个 `.tres`，所以两边看起来一样。

只有当普通图片不够用的时候，才需要 `.tres`，比如要改比例、加发光，或者做成能透过去看的。只有 `albedo_color` 而没有纹理的 `.tres` 也行，编辑器会用那个颜色来画它。

新的纹理和材质会在几秒钟内出现在“材质”标签页里，不需要重新加载。

## 纹理尺寸 { #texture-size }

默认情况下，一个纹理像素覆盖一个地图单位，所以一张 64 像素的纹理每两米重复一次。拿来做像素画很棒，但一张 1024 像素的照片贴在两米宽的墙上，会被拉伸到 32 米。

要解决这个问题，告诉材质一次重复覆盖多少个地图单位：

```
[resource]
albedo_texture = ExtResource("1_albedo")
metadata/texture_size = Vector2(64, 64)
```

（你也可以在 Godot 的检查器里的 *Metadata* 下添加它，类型是 `Vector2` 或单个数字。）

把鼠标悬停在“材质”标签页里的材质上就能检查，工具提示会显示图像尺寸，以及一次重复覆盖多少个单位。

![“材质”标签页里 night/shop_sign 的工具提示：1024 x 256 px，每次重复 256 x 64 单位，自发光 x3](../assets/materials/tooltip-texture-size.png)

现在你可以换上更清晰的图片，而不用改动地图。地形图层不理会它，它们的 `tile` 设置已经决定了重复。

## 发光材质 { #glowing-materials }

当材质文件设置了 `emission_enabled` 时，这个材质就会发光。发光颜色为黑色时，发光纹理会完全按所绘的样子发光：

```
[resource]
albedo_texture = ExtResource("1_albedo")
emission_enabled = true
emission = Color(0, 0, 0, 1)
emission_energy_multiplier = 2.5
emission_texture = ExtResource("2_emission")
```

没有材质文件吗？在反照率图片旁边放一张名字相同、再加上 `_emission` 的图片，比如 `night/sign.png` 旁边放 `night/sign_emission.png`，它就会成为发光贴图。

发光材质在“材质”标签页里会带一个黄色的*发光*标签：

![“材质”标签页里的 night 文件夹材质，其中 apartment_lit、neon_red 和 shop_sign 带有黄色的发光标签](../assets/materials/glow-tags.png)

按 F3 打开光照预览，就能在 3D 视图里看到它们发光。它们周围的泛光只有在 Godot 里才看得到，参见[夜间照明](night-lighting.md)。

## 透明度 { #transparency }

当纹理的 alpha 通道应该让后面的东西透出来时，就在材质文件里设置 `transparency`。每种模式在质量和开销之间的取舍各不相同：

| `transparency` | 原理 | 适用于 | 注意 |
| --- | --- | --- | --- |
| `1` 混合 | 把表面和它后面的东西混合在一起 | 玻璃、淡淡的污渍、烟雾 | 不投射阴影 |
| `2` 裁剪 | 像素要么完全显示，要么被切掉 | 近处的硬边镂空，比如格栅和树叶 | 细小的部分在远处会消失 |
| `3` 哈希 | 对像素做抖动，alpha 越低，显示的像素越少 | 远处看到的细小镂空，比如铁丝网 | 柔和的边缘会变成噪点 |

“材质”标签页里的工具提示会告诉你一个材质用的是哪种模式：

![“材质”标签页里 night/chainlink 的工具提示：Alpha 哈希，法线贴图，双面](../assets/materials/tooltip-alpha-hash.png)
