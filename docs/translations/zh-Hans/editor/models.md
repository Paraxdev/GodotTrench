---
source: d2dc61c8ee306263eda3f36520611e0790a27732
---

# 模型 { #models }

模型是用别的程序做的 3D 文件，比如 Blender 或 Blockbench。TrenchDot 能读取 glTF、OBJ、Blockbench、STL、MD2 和 MD3 文件。

你可以在底部的 **模型** 标签页里找到它们，它在材质和预制体旁边。它会列出 `res://models` 下的所有东西，装好之后还有 `res://trenchdot/nature` 里的自然模型。

![模型标签页：搜索框、文件夹和来源筛选、排序、Slug、缩略图大小滑块、适应、尺寸倍数、重新扫描按钮和导入…，下面是一个由自然模型缩略图组成的网格](../assets/models/models-panel.png)

新项目一开始没有模型。*Godot > 向项目添加内容…* 会添加自然模型，选择演示内容时还会带上 `res://models/polyhaven` 里的 Poly Haven 道具。或者把你自己的文件复制到 `res://models`，然后按 ⟳ 按钮重新扫描。

## 放置模型 { #placing-a-model }

模型有两种方式放进地图。根据你是否想在编辑器里改变它的形状来选。

| 放置为 | 在地图中存储的内容 | 适用于 |
| --- | --- | --- |
| 可编辑网格 | 一份几何体的副本，不再跟随文件 | 想用[网格工具](meshes.md)编辑的形状 |
| `prop_model` 实体 | 只有一个 `res://` 路径，由 Godot 加载文件 | 保持原样的道具。你修改文件时，每一份副本都会跟着变 |

### 作为可编辑网格 { #as-an-editable-mesh }

1. （可选）设置大小

    模型工具栏里的 适应 默认是勾选的。如果模型最长的一边小于四分之一米或大于 256 米，它会把模型缩放到 2 米，这样很小或很大的文件导入后依然可用。旁边的 × 字段会在此基础上再乘以一个倍数。两者都在放置时生效，所以请在拖动之前设置好。

2. 拖进视图

    把模型标签页里的缩略图拖进任意视图。它会落在指针下的表面上。

    ![从模型标签页拖进 3D 视图的一个酒桶，它作为可编辑网格落在地板上](../assets/models/drag-model-into-view.gif)

对于磁盘上任意位置的文件，使用模型工具栏末尾的 **导入…**。

### 作为道具 { #as-a-prop }

1. 选择文件

    前往 *文件 > 导入 > 模型道具 (.bbmodel, .glb)…* 并选择模型。

    ![文件菜单中展开了导入，鼠标悬停在模型道具 (.bbmodel, .glb)… 上](../assets/models/file-import-model-prop.png)

2. 检查检查器

    它会作为 `prop_model` 实体落在 3D 光标处，它的 `model` 键里是 `res://` 路径。

    ![prop_model 实体的检查器，model 键设为一个 res://models/polyhaven 路径](../assets/models/inspector-prop-model.png)

道具必须指向 Godot 能加载的文件。如果选了项目之外的文件，编辑器会提示先把它复制到 `res://models`。

Blockbench 模型还有两种导入方式：*文件 > 导入 > Blockbench 模型转网格…* 和 *Blockbench 模型转笔刷…*。作为笔刷导入时，每个立方体都会变成一个笔刷，并按文件名分到一个编组下。

## 已放置网格的纹理 { #textures-of-a-placed-mesh }

可编辑网格会保留模型的外观。它的纹理会被复制到项目纹理文件夹里的 `models/<name>` 下，其中 `<name>` 是模型在模型标签页里的路径，如果模型来自别处，就是它的文件名。选中网格，检查器会列出它所用的材质：

![已放置网格的检查器：5966 个顶点，以及材质 models/polyhaven/containers/wine_barrel_01/tex0](../assets/models/inspector-mesh-materials.png)

如果普通图片画出来和模型不一样，项目的材质文件夹里还会放进一个 Godot 材质。这样镂空的树叶、像素画和发光的部分（包括 glTF 模型的自发光颜色和自发光贴图）在 Godot 里就能和这里画得一样。要替换材质，在选中网格时点击材质标签页里的一个材质，*编辑 > 撤销* 可以把它们换回来。

放置绝不会覆盖文件。再放置同一个模型时，它会复用自己的纹理，并保留你在 Godot 里调过的材质。如果该文件夹里已经有别的模型的纹理，比如 `rock.glb` 旁边的 `rock.bbmodel` 的纹理，新的纹理会放进带编号的文件夹，比如 `models/rock_2`。撤销不会动这些文件。如果纹理无法写入，比如项目是只读的，网格就会不带纹理地导入，状态栏会说明原因。

纹理按像素判断是否相同，所以另一个程序只是重新保存过的纹理依然会被复用。在其中一个上画过东西？之前放置的网格会保留你的版本，下一次放置则会在带编号的文件夹里得到全新的副本，以及一个新的材质。要让之后的放置也用上你的改动，请从材质标签页把你的材质应用到它们上面。

> [!NOTE]
> 只有在没有同名材质文件时，FuncGodot 才会用项目的 `default_material` 来生成面的材质。已放置模型中得到了材质文件的面，会用那个文件来绘制，所以设为 `default_material` 的自定义着色器不会作用到它们。要改变它们，请在 Godot 里编辑那个文件。材质扩展名不是 `tres` 的项目不会得到材质文件，而是使用它的默认材质。

## 缩略图的更多操作 { #more-on-a-thumbnail }

右键点击缩略图，可以把它放置在光标处、复制路径或者在磁盘上找到文件。当它的资源包有网址时，会出现 *打开资源包页面*。

![wine_barrel_01 缩略图的右键菜单，鼠标悬停在放置在光标处上，然后是复制路径、在文件管理器中显示和打开资源包页面](../assets/models/thumbnail-menu.png)

## 资源包署名 { #pack-credits }

免费的模型包通常要求署名。把一个 `pack.json` 放进模型所在的文件夹，或者它上面的任意一级文件夹（取最近的那个），面板就知道模型来自哪个包、是谁做的、使用什么许可证。把鼠标悬停在缩略图上就能看到：

![wine_barrel_01 缩略图的工具提示：它的路径、资源包：Poly Haven、作者、许可证：CC0、资源包网址和模型署名](../assets/models/thumbnail-credit-tooltip.png)

```json
{
  "name": "Nature Kit",
  "short": "Nature",
  "author": "Some Artist",
  "license": "CC0",
  "url": "https://example.com/nature-kit",
  "credits": [{ "asset": "pine", "author": "Other Artist" }]
}
```

勾选 Slug 时，`short` 是每个缩略图上的标签，缺省则使用 `name`。`credits` 会覆盖单个模型的作者、许可证或网址，按不带扩展名的文件名匹配。没有 `pack.json` 的文件夹会列在它们顶层文件夹的名字下。
