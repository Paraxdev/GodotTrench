---
source: 2de051fc5fd93ade8349b194b97a6c34a8399111
---

# 投射贴花 { #projected-decals }

投射贴花是一个盒子，它会把一种材质照到盒子里的一切东西上。与[普通贴花](decals.md)那种平面薄片不同，它会顺着碰到的任何东西贴合，所以一块污渍可以漫过台阶、绕着管子包裹，或者铺在起伏的地面上。盒子里的道具和模型也会被贴上。

它在构建出的场景里是一个 Godot 的 `Decal` 节点，绘制开销比普通贴花更大。只在平面薄片不够用的地方使用它。

> [!WARNING]
> 投射贴花对 GPU 的负担比[普通贴花](decals.md)更重。Godot 会为盒子在屏幕上覆盖的每个像素计算贴花，所以盒子多，或者盒子大，都会拖慢游戏，在较弱的显卡上尤其明显。平面上用普通贴花，盒子在够用的前提下尽量小，投射贴花只留给确实需要包裹效果的地方。

## 把它加进你的项目 { #adding-it-to-your-project }

投射贴花以 `infodecal` 的名字随[玩法实体包](../godot/project-setup.md#core-entities-and-the-gameplay-entities-pack)一起提供，所以设置它就是安装这个包。

1. 安装或更新这个包

    点击*设置此项目*窗口里的**安装玩法实体**，如果包已经在你的项目里，则点击玩法菜单里的**更新玩法实体**。[安装和更新这个包](../godot/project-setup.md#installing-and-updating-the-pack)会一步步带你完成。

2. 确认编辑器能看到它

    你应该会在检查器旁边的**实体**标签页里看到一张 `infodecal` 卡片。在标签页顶部的搜索框里输入 `infodecal` 可以很快找到它。

    ![实体标签页搜索了 infodecal，鼠标停在卡片上，提示写着投射贴花](../assets/projected-decals/entities-infodecal-card.png)

> [!NOTE]
> 如果你在这个包带有 `infodecal` 之前就做过自己的 `infodecal` 定义，更新之后请把它从你的 FGD 里拿掉。否则这个类会被定义两次，编辑器会对此发出警告。

> [!TIP]
> 没有使用这个包？把它的定义
> [`infodecal.tres`](https://github.com/Paraxdev/trenchdot_func/blob/main/gameplay_pack/definitions/infodecal.tres)
> 复制进你的项目，并把它加到你自己 FGD 文件的 *Entity Definitions* 里，参见[自定义实体](../gameplay/custom-entities.md#making-it-placeable)。把 *Node Class* 保持为 `Decal`，脚本需要它，编辑器也靠它才知道要把这个实体当作贴花处理。

## 放置一个 { #placing-one }

1. 把卡片拖到一个表面上

    打开检查器旁边的**实体**标签页，把 `infodecal` 卡片拖进某个视图，就像放置任何实体那样（参见[放置实体](../gameplay/placing.md#placing-entities)）。它会落在指针碰到的第一个实心表面上，并转向那个表面，让它朝表面里投射。一个盒子会显示它覆盖的空间，一半在表面前面，一半在后面。

    ![3D 视图里一块砖台阶的边缘上有一个 infodecal，它的盒子越过顶面并向下覆盖正面，盒子里预览着招牌材质](../assets/projected-decals/decal-box-on-step.png)

2. 选好材质

    在检查器里，往 `material` 里输入一个材质名，和面使用的名字相同，比如 `base/stain`。它的颜色、法线贴图和发光都会带过去。

    ![选中 infodecal 的检查器，material 里是 night/shop_sign，size 里是 128 64 64](../assets/projected-decals/decal-inspector-material.png)

3. 调整大小

    `size` 接受三个以地图单位计的数：宽、深和高。宽和高是它在表面上覆盖的那一片，深是盒子伸进表面和伸出表面的距离。默认值是 64 32 64，一个两米见方、前后各伸出半米的盒子。如果保持默认，而材质有[纹理尺寸](../godot/materials.md#texture-size)，那么这一片会改用材质的纹理尺寸，编辑器和 Godot 里都是如此。

4. （可选）调整朝向

    像其他实体一样旋转它。它总是沿着自己的向下轴，也就是局部的 -Y 投射，拖放时这个轴是指向表面里的。

5. 在 Godot 里构建地图

    Godot 构建地图之后，材质会显示在盒子里的每个表面上，参见[构建地图](../godot/building.md)。

    编辑器只会在盒子里显示材质的平面预览，真正的投射由 Godot 绘制，招牌会越过台阶的边缘并向下弯到它的正面。

    ![同一块招牌在运行中的 Godot 游戏里，越过砖台阶的边缘弯到它的正面](../assets/projected-decals/decal-in-game.png)

## 键 { #keys }

| 键 | 作用 |
| --- | --- |
| `material` | 要投射的材质，可以是 `base/stain` 这样的名字，或一个 `res://` 路径。它的反照率、法线、ORM 和自发光纹理，以及它的反照率颜色都会被用到 |
| `texture` | 一张图像的 `res://` 路径，在 `material` 为空时投射 |
| `normal_texture` | 一张法线贴图的 `res://` 路径，配合 `texture` 使用。材质自带的法线贴图会替换它 |
| `size` | 盒子的宽、深和高，以地图单位计，不改的话是 64 32 64 |
| `modulate` | 投射的图像要乘上的一种颜色 |

## 需要留意的事 { #things-to-watch-out-for }

| 情况 | 会发生什么 |
| --- | --- |
| 项目使用 Compatibility 渲染器 | Godot 完全不绘制贴花。把 *Project Settings > Rendering > Renderer* 下的 *Rendering Method* 切换为 Forward+ 或 Mobile |
| 项目使用 Mobile 渲染器 | 任何一个网格上只会显示 8 个贴花，其余的被舍弃 |
| 沿着投射方向延伸的面 | 纹理会被拉成条纹，就像一个正对下方的贴花下面台阶的正面那样。把盒子倾斜，让它以一定角度同时碰到两个面 |
| 靠近盒子两端的表面 | 贴花会朝着盒子的顶部和底部淡出，所以让表面靠近中间，也就是拖放放置的位置 |
| 有东西穿过盒子 | 它也会被贴上，路过的玩家也不例外。盒子的深度不要超过表面所需 |
| 玻璃和镂空 | 它们还是和以前一样透明，贴花无法填上透明的部分 |
| 许多大型贴花 | 每一个都会给它覆盖的每个像素增加工作量，所以叠在一起的地方会拖慢游戏 |

<div class="td-mcp" markdown>

## MCP { #mcp }

通过 [MCP 服务器](../mcp.md)，用 `run_action` 配合 `action: "place_entities"` 和 `args: {"classnames": ["infodecal"], "at": [x, y, z], "normal": [x, y, z]}`，可以把它放置并转向表面，和拖放一样。之后用 `update_entity` 的 `properties` 设置它的键。

</div>
