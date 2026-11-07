---
source: 078f68d4f428db30dc2dfcb8561a8991ada14154
---

# 转换纹理 { #converting-textures }

Godot 无法加载 `.vtf`、`.wad` 或 `.wal` 文件，所以编辑器会把它们变成 PNG，普通图片带不了设置的地方再加一个 `.tres`。每张纹理都会以面的材质名放进项目的纹理文件夹，这样导入的面一下子就能找到它。

导入地图时，会针对它用到的纹理提供这个功能，参见[导入 Hammer 和 TrenchBroom 地图](importing.md)。想转换整个文件夹，先打开 Godot 项目，然后：

1. 打开转换器

    进入*文件 > 导入 > 转换纹理 (VTF/VMT, WAD, WAL)…*。

    ![文件菜单里导入已展开，鼠标停在转换纹理 (VTF/VMT, WAD, WAL)…上](../assets/importing-textures/file-import-convert-textures.png)

2. 选择文件夹

    选中放着 `.vmt` 和 `.vtf`、`.wad` 或 `.wal` 文件的文件夹。如果已打开的地图缺少其中一些，编辑器会问你是只转换**仅地图所需**的，还是转换**全部**。

    之后状态栏应该会告诉你转换了多少张，比如“纹理：12 converted”，它们会以材质名出现在材质标签页里。

    ![在材质标签页里搜索 castle，显示出导入地图带来的 brick_wall 和 stone_step，提示写着 castle/brick_wall, 1024 x 1024 px](../assets/importing-textures/converted-in-materials.png)

## 在哪里找到它们 { #where-they-are-found }

编辑器会搜索每个游戏存放纹理的常见位置，所以连同游戏文件夹一起复制过来的地图，不用任何设置就能找到它们。

| 来源 | 查找位置 | 命名依据 |
| --- | --- | --- |
| Valve 的 `.vmt` 和 `.vtf` | 地图旁边或往上最多四层文件夹里的 `materials` 文件夹 | `materials` 下面的路径，比如 `brick/brickwall001a` |
| Quake 和 Half-Life 的 `.wad` | 先看 worldspawn 的 `wad` 键，再找附近文件夹里的同名文件 | 纹理名，比如 `{grate` |
| Quake 2 的 `.wal` | 附近的 `textures` 文件夹 | `textures` 下面的路径，比如 `e1u1/floor1_1` |
| 零散的 `.png`、`.jpg`、`.tga` | 附近的 `textures` 文件夹，Qodot 和 FuncGodot 项目就是这样带着它们的 | `textures` 下面的路径，原样复制 |

## Valve 材质 { #valve-materials }

Valve 的 `.vmt` 文件把材质描述成一串参数。下面这些会带到 Godot 材质里，其余的会被省略：

| Valve 材质 | Godot 材质 |
| --- | --- |
| `$basetexture` | 反照率，除非材质是透明的，否则会去掉它的 alpha |
| `$bumpmap`，水面上则是 `$normalmap` | 法线贴图，绿色通道从 DirectX 翻转为 Godot 的方向 |
| `$translucent`、`$additive` | `$translucent` 用 alpha 混合，`$additive` 用叠加混合 |
| `$alphatest`、`$alphatestreference` | 按该阈值做 Alpha 裁剪，默认 0.5 |
| `$selfillum`、`$selfillummask` | 发光的自发光纹理，由基础纹理的 alpha 或遮罩来遮挡 |
| `$nocull`、`UnlitGeneric` | `$nocull` 对应双面，`UnlitGeneric` 对应无阴影着色 |
| `$color` | 反照率颜色 |
| `$surfaceprop` | 材质上的 `metadata/surfaceprop` |
| 带 `include` 的 `patch` | 补丁会复用另一个材质并做少量修改，所以它会变成被包含的那个材质，并应用它的 `insert` 和 `replace` 修改 |

## Quake 纹理 { #quake-textures }

以 `{` 开头的纹理用 Alpha 裁剪镂空，天空是无阴影着色的，`*water` 会存成 `water`。动画纹理，包括多帧的 VTF，只保留第一帧。

## 天空 { #skies }

转换地图的纹理时也会设置它的天空，所以导入的地图会得到一个接近原版的天空。如果地图已经有 `sky_top_color`，就会跳过这一步。

在 `materials/skybox` 里带有 `skyname` 天空盒的 Source 地图，会得到一张全景图，即纹理文件夹里的 `skybox/<skyname>_panorama.png`，并设为 `sky_panorama`。其他地图则根据 `sky_source` 里的纹理得到 `sky_top_color` 和 `sky_horizon_color`。当场景自己没有 WorldEnvironment（Godot 里保存场景天空、雾和环境光的节点）时，`TrenchDotEnvironment` 会根据这些键建一个。

## VTF 格式 { #vtf-formats }

转换器读取 7.0 到 7.5 版本的 VTF，支持这些像素格式：DXT1（带一位 alpha）、DXT3、DXT5、8 位的 RGB 和 RGBA 排列、I8、IA88、A8、RGB565、BGR565、BGRA4444、BGRA5551、BGRX5551、UV88、UVWQ8888、UVLX8888，以及 16 位的 RGBA 格式。其他格式和 VTF 7.6 会被拒绝，并写明是哪种格式，这样你就知道该手动转换哪个文件。

<div class="td-mcp" markdown>

## MCP { #mcp }

通过 [MCP 服务器](../mcp.md)，用 `map_file {op: convert_textures, path}` 转换一个纹理文件夹。加上 `only_missing` 只转换打开的地图缺少的那些，加上 `overwrite` 则替换项目里已有的文件。

</div>
