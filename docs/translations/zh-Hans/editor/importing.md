---
source: e0934aeaf4aaffb804a36e5ff5821a38614fc413
---

# 导入 Hammer 和 TrenchBroom 地图 { #importing-hammer-and-trenchbroom-maps }

手上有 Hammer 或 TrenchBroom 做的关卡？把它带过来，不用重做。*文件 > 导入*可以把 Hammer 的 `.vmf`，或者 TrenchBroom、Quake、Qodot 的 `.map`，打开成一张新的未保存地图（如果已打开的地图有未保存的更改，就开在新标签页里）。把它存成 `.tdm`，Godot 就会像构建其他地图一样构建它。

## 步骤 { #steps }

1. 先打开 Godot 项目

    用*Godot > 打开 Godot 项目…*打开这张地图所属的项目，因为转换后的纹理会放进它的纹理文件夹。

2. 导入地图

    进入*文件 > 导入*，选择*.vmf（Hammer）…*或*.map（TrenchBroom, Quake）…*，然后选文件。

    ![文件菜单里导入已展开，鼠标停在.vmf（Hammer）…上，下面是转换纹理和模型导入](../assets/importing/file-import-menu.png)

3. 转换纹理

    如果地图附近有它的纹理，编辑器会针对项目里缺少的那些问你“转换地图的纹理？”。选是，参见[转换纹理](importing-textures.md)。

4. 查看问题标签页

    打开左上角的问题标签页，它在大纲和历史旁边。项目里仍然缺少的每个材质都会显示成一条警告，它的面会画成棋盘格。

    ![问题标签页里有两条警告，材质 'castle/brick_wall' 不在项目里（12 个面），3D 视图里那些墙和台阶画成灰色棋盘格](../assets/importing/issues-missing-materials.png)

    这些面保留着材质名，所以之后补上纹理就能修好它们。等每张纹理都到位，这些警告就消失了。

    ![转换纹理之后的同一张地图：问题标签页写着未发现问题，墙上显示出砖纹理](../assets/importing/textures-converted.png)

5. 保存

    用*文件 > 另存为…*，把它存成项目里的 `.tdm`。

> [!NOTE]
> 单位保持原样。Source 地图大约按每米 40 单位制作，Quake 地图大约是 32，所以除非项目用了别的比例，Source 地图出来会比在它的游戏里稍大一点。

## 导入会保留什么 { #what-the-import-keeps }

| Hammer | 在 TrenchDot 里 |
| --- | --- |
| 工具材质（`tools/toolsnodraw`、`toolsclip`、`toolsplayerclip`、`toolstrigger`、`toolsorigin` 等） | 项目的跳过、碰撞体和原点纹理，以及 `special/trigger` |
| 天空面 | 项目的天空纹理，默认是 `special/sky` |
| 置换 | 带着高度和混合 alpha 的置换 |
| `func_instance` | 一个装着实例内容的分组，已移到位。里面的名称会套用实例的 fixup，它的 `$` 和 `#` 替换会被填好，指向 `instance:name;Input` 的输出会接到真正的实体上 |
| 可见组 | 图层 |
| 隐藏的对象 | 隐藏的节点 |
| 实体键和输出 | 实体属性和输出 |

TrenchBroom 的图层和分组仍然是图层和分组，Quake 的工具纹理也以同样的方式变成项目的工具纹理。

只需要几个笔刷？在 TrenchBroom 里复制它们，再在 TrenchDot 里按 Ctrl+V，它们会以同样的方式转换后过来，参见[预制体库](prefab-library.md#pasting-from-trenchbroom)。

天空面在 Godot 里有碰撞，但什么都不画。worldspawn 的键 `sky_source` 保留着它们原来的纹理，转换纹理时会把它变成程序化天空，参见[转换纹理](importing-textures.md#skies)。

## 已知限制 { #known-limitations }

| 限制 | 怎么办 |
| --- | --- |
| `.vpk` 或 `.pak` 压缩包里的纹理不会被读取 | 先把它们解压出来，比如用 VPKEdit |
| 实例会在地图旁边和上层文件夹里查找 | 保持游戏 `maps` 文件夹的布局。找不到的实例会保留为 `func_instance` 实体 |
| Valve 和 Quake 的实体类在新项目里没有定义，所以编辑器不认识它们的键 | 把它们加进你的 FGD，或者换成你自己的类。问题标签页会列出它们 |
| Source 材质的一些功能被省略了：`$basetexture2` 混合、细节纹理、环境贴图和代理 | 在 Godot 里设置 |
| Quake 3 的面片（曲面）和笔刷图元会被跳过。状态栏会说跳过了多少 | 用网格重建它们 |
| 顶点会横向移动（而不只是上下移动）的置换，会变成没有混合 alpha 的网格 | 当作网格来编辑 |
