---
source: 18fb3370f830b9fda2728d12b95c3b5cef06be64
---

# 入门 { #getting-started }

第 1 到 3 步：首次设置。如果你已经有一个用 TrenchDot 设置好的 Godot 项目，可以跳过这几步。

第 4 和 5 步：在 TrenchDot 中搭建第一个房间，并在 Godot 中看到它。

## 你需要准备什么 { #what-you-need }

* Godot 4.7 或更新版本，如果你想用 C# 编写实体，请用 .NET 版
* 最新的 [TrenchDot 发布版](https://github.com/Paraxdev/TrenchDot/releases/tag/beta)

## 确认 TrenchDot 能找到 Godot 可执行文件 { #ensure-trenchdot-can-see-the-godot-executable }

请确认 TrenchDot 能找到你安装的 Godot。要检查是否一切都被正确识别，打开 *文件 > 偏好设置… > Godot 可执行文件*。

如果 TrenchDot 自己找到了 Godot，输入框下面那一行就已经显示“正在使用”和路径。如果显示的是“未找到”，你就需要自己在磁盘上找到这个可执行文件。

为此，点击输入框旁边的“…”按钮

![Godot 可执行文件输入框旁边的 … 按钮](assets/getting-started/godot-path-browse.png)

然后找到以“Godot_v”开头的可执行文件。

如果你选对了可执行文件，那一行应该显示“正在使用”和你刚选的路径。

![输入框下面的那一行，显示正在使用以及 Godot 可执行文件的路径](assets/getting-started/godot-path-using.png)

## 1. 在 Godot 中创建项目 { #1-create-a-project-in-godot }

打开 Godot，点击左上角的“Create”。

![Godot 的项目管理器，左上角带加号图标的 Create 按钮](assets/getting-started/godot-create-project.png)

选一个项目名称，本教程里用 TrenchDotTest，再（可选地）选一个放项目的文件夹。

然后点击“Create”。

## 2. 在 TrenchDot 中打开项目 { #2-opening-the-project-within-trenchdot }

选择 *Godot > 打开 Godot 项目…*

![TrenchDot 中的 Godot 菜单，打开 Godot 项目… 处于高亮状态](assets/getting-started/menu-open-godot-project.png)

或者点击工具栏右上角的 **打开 Godot 项目…** 按钮

![TrenchDot 工具栏右上角的 打开 Godot 项目… 按钮，打开项目后它会显示项目名称](assets/getting-started/toolbar-open-godot-project.png)

然后选择项目文件夹，或者其中的任意一个文件夹。编辑器会在下次启动时重新打开它。

## 3. 欢迎界面 { #3-the-welcome-screen }

一旦 TrenchDot 发现这是一个新的 Godot 项目，就会显示 *设置此项目* 窗口。

![设置此项目窗口，包含插件、玩法实体和现成内容几个部分](assets/getting-started/setup-window.png)

在这里你可以下载插件，Godot 需要它来把你的地图变成场景，并与 TrenchDot 通信，你也可以按需下载额外内容。当然，你也可以什么都不装就开始。

一些最基本的实体始终包含在内，比如简单的 `info_player_start` 和 `light`。

1. 安装插件

    点击顶部的“安装插件”按钮，把 TrenchDot 的 FuncGodot 分支下载到你的项目中，这对 TrenchDot 至关重要。
    （注：这会从 GitHub 发布页下载。如果你没有网络，请先下载 `func_godot-trenchdot-addon.zip`，再把它解压到项目根目录，手动安装。）

    完成后，点击“启用插件”，让 Godot 加载它。
    （注：如果 Godot 已经打开了这个项目，请先关闭它，否则 Godot 会把它自己的 `project.godot` 副本写回去，插件就会保持关闭。）

2. 安装玩法实体

    点击插件下方的 **安装玩法实体**。

    ![设置此项目窗口的玩法实体部分，里面有安装玩法实体按钮](assets/getting-started/setup-gameplay-entities.png)

    门、按钮、升降机、触发器、道具和生成器都包含在这个包里。没有它，Godot 会把它们构建成普通的形状，既没有碰撞，也什么都不做。“玩法”菜单里的门和触发器助手会在你第一次使用时自动安装这个包，但在 Godot 第一次打开项目之前就装好它，就能保证没有任何场景是在缺少它的情况下构建的。

3. （可选）额外内容

    TrenchDot 有一些可选内容可以预先载入：来自 Poly Haven 和 ambientCG 的 CC0 纹理和模型，以及专为 TrenchDot 制作的模型。许可证和各项内容的来源清单都附在其中。

    如果你不想要任何额外内容，可以选择“不添加额外内容”，最基本的实体仍然会包含在内。

4. 在 Godot 中打开一次项目

    插件启用后，在 Godot 中打开项目（窗口里有一个“在 Godot 编辑器中打开项目”按钮可以做到）。随后插件会在项目根目录创建一个名为 `trenchdot_game.json` 的文件。TrenchDot 就是从这里了解你的纹理和实体的，它会在一秒内加载这个文件。

    每当文件发生变化，插件都会重新写入它，所以不需要手动编辑。在它出现之前，状态栏会提示没有 `trenchdot_game.json`，编辑器只提供核心实体，如果你装了玩法实体，也会提供它们。

想添加你自己的实体，请看[用 GDScript 编写自定义实体](gameplay/custom-entities.md)和 [C# 实体](gameplay/csharp.md)。[项目设置](godot/project-setup.md)解释了列出它们的资源。

## 4. 搭建一个房间 { #4-build-a-room }

编辑器有 4 个主视图：一个可以像在 Godot 里那样飞行浏览的 3D 视图，以及从顶部、前面和侧面看的 3 个线框视图。

笔刷和各种操作都取决于栅格大小。你可以在工具栏里设置栅格大小，数值越小越精细。

![工具栏中的栅格大小设置，设为 16](assets/getting-started/toolbar-grid-size.png)

在空白关卡上，你会看到一小段文字，说明如何绘制。你可以点“隐藏提示”把它藏起来，也可以直接开始画，地图里一有内容它就会消失。

![一张新地图：3D 视图中的卡片提示在顶视图中拖动来画出第一个方盒，顶视图里有一个黄色的“在此拖动画出第一个方块”标签](assets/getting-started/empty-map.png)

1. 画出你的第一个方盒

    用选择工具（Q），在任意视图的空白处拖动。
    你应该会看到一个方盒正在被画出来。按住鼠标左键不放，移动鼠标来调整矩形的形状。

    ![画好第一个方盒的四个视图，顶视图里标出了它的尺寸，512 乘 256](assets/getting-started/draw-first-box.png)

2. 增加高度

    在侧视图中，把鼠标悬停在方盒的上边缘，直到它变成黄色，按住鼠标左键，然后向上移动鼠标，直到方盒高 256 个单位。

    ![向上拖动了方盒上边缘的侧视图，上方和旁边标出了 256](assets/getting-started/side-view-depth.png)

3. 把它挖空

    按 Ctrl+Shift+K，或者使用工具栏里的“挖空”按钮。

    ![悬停在挖空按钮上并显示提示的工具栏](assets/getting-started/toolbar-hollow.png)

    现在方盒被分成了墙壁、地板和天花板，如果你想做一个较小的房间或走廊，这样最合适。接着用飞行控制（按住鼠标右键，再用 WASD）飞进这个挖空的方盒里。

4. 切出一个门洞

    按 Esc 可以取消选中已选的笔刷。什么都没选中之后，在前视图中的方盒内部画一个较小的竖直矩形，形状像一扇门洞。

    ![挖空方盒内部的门笔刷，在前视图中宽 64、高 96](assets/getting-started/doorway-brush.png)

    然后在侧视图中拉长门笔刷的一侧，让它有了深度，一直贯穿整面墙。

    ![在侧视图中拖动门笔刷的一侧，让它穿过墙壁](assets/getting-started/doorway-extend.gif)

    现在，选中这个笔刷后，按 Ctrl+K，或者使用工具栏里的“减去”按钮。

    ![悬停在 CSG 减去按钮上并显示提示的工具栏](assets/getting-started/toolbar-subtract.png)

    你应该会看到笔刷消失，门洞被从墙上切了出来。

    ![3D 视图中切出了门洞的墙](assets/getting-started/doorway-cut.png)

5. 贴上纹理

    你应该有 5 张自带的原型制作用纹理。想只给某一个面（比如门框）贴纹理，把纹理从“材质”标签页拖到这个面上（按住 Shift 可以覆盖整个笔刷）。

    ![把 dev/blue 从材质标签页拖到一个面上，并显示放置预览](assets/getting-started/texture-drop.png)

6. 保存地图

    要在 Godot 中看到改动，我们得先保存地图。按 Ctrl+S，或者使用 *文件 > 保存*。

    保存对话框会在你的 Godot 项目文件夹中打开。保存在项目内的任意位置都行，在项目根目录放一个 `maps` 文件夹比较整洁，比如 `res://maps/room.tdm`。
    （注：文件夹名由你决定，但要把地图保存在项目内，这样 Godot 才能通过它的 `res://` 路径加载它。）

    保存之后，它会出现在 *文件 > 项目中的地图* 下。

    ![文件菜单，里面有保存和项目中的地图，后者在 res://maps 下列出了 room](assets/getting-started/file-menu-maps-in-project.png)

## 5. 在 Godot 中看到它 { #5-see-it-in-godot }

在 Godot 中，关卡是一个场景：一棵节点树，保存为 `.tscn` 文件，其中每个节点是一样东西，比如网格、灯光或摄像机。Godot 用 `res://` 来书写项目内的路径，所以 `res://maps/room.tdm` 就是你项目文件夹里的 `maps/room.tdm` 这个文件。地图由插件中的一个节点 `FuncGodotMap` 变成节点。

1. 在 Godot 编辑器中，选择 *Scene > New Scene*，然后在左侧的 Scene 面板中点击 **3D Scene**。
2. 按 Ctrl+A（*Add Child Node*），搜索 `FuncGodotMap` 并添加它。
3. 选中新节点后，在 Godot 自己的检查器里把 *Local Map File* 设为你的 `.tdm`，检查器就是 Godot 编辑器右侧的面板。它的用法和 TrenchDot 里的检查器一样，但属于 Godot。
4. 点击那个检查器顶部的 **Build Map**，然后用 Ctrl+S 保存场景。

笔刷会变成带碰撞的网格，实体会变成带着脚本的节点，一同保存在场景里。[构建地图](godot/building.md)解释了这个节点的其他设置。如果某个实体变成了没有碰撞的光秃秃的网格，TrenchDot 的状态栏会说明原因，请看[什么都不做的实体](godot/entities-troubleshooting.md)。

让那个场景在 Godot 中保持打开，TrenchDot 里的每次保存都会重新构建它：当它的标签页是当前页时会立刻构建，否则在你切换回来时构建。在 Godot 中保存场景才能保留结果。如果某次保存没有触发重新构建，状态栏会说明原因，请看[保存时重新构建](godot/live.md#rebuild-on-save)。

> [!WARNING]
> 每次构建都会先删除 `FuncGodotMap` 的子节点。请把你自己的节点放在地图节点旁边，或者放进[叠加层](godot/overlays.md)里。

## 试试演示项目 { #try-the-demo-project }

仓库里的 `godot` 文件夹，发布版中也有 `trenchdot-demo-project.zip`，是一个已经装好插件的项目。它的主场景会播放展示地图，这些地图保存在 `godot/demo/maps/showcase` 中。在编辑器里打开这个项目，从 *文件 > 项目中的地图* 下选一张，就能看到一张完成的地图是怎么搭起来的。

*Godot > 向项目添加内容…* 中的演示选项会把同样的地图和场景放进你自己的项目。在 Godot 中打开 `res://demo/demo.tscn`，按 F6 来播放它们。从你的项目里用 TrenchDot 打开时，它们显示时没有纹理，因为它们用的是演示的纹理文件夹 `res://demo/textures`，而不是你项目的。演示项目则会按它们搭建时的样子展示。

| 按键 | 作用 |
| --- | --- |
| WASD、Space、Shift | 行走、跳跃、奔跑 |
| E | 使用门和按钮 |
| 1 到 5 | 在展示地图之间切换 |
| F3 | 显示 I/O 调试叠加层，记录每个输入和输出触发时的日志，并画出触发体积 |

## 下一步 { #next }

想让这个房间有点动静，请阅读[实体 I/O 的工作原理](gameplay/io.md)，并搭建[按钮开门](tutorials/button-door.md)。

你自己的玩家需要满足这个包中实体的两个条件。触发器只会对物理层 1 上、属于 Godot 分组 `player` 的物体做出反应，所以请把你的玩家放在那里。当你的玩家代码对它正在看的那一个调用 `use()` 时，门和按钮才会动，请看[让玩家按下它](tutorials/button-door.md#let-the-player-press-it)。

<div class="td-mcp" markdown>

## MCP { #mcp }

在由 MCP 客户端驱动的编辑器里，内容问题不会自己弹出来：除 `get_state`、`screenshot` 和 `simulate_input` 之外的第一次工具调用会把它关掉，用 `open_project` 打开的项目也不会被询问。`project_content` 会添加同样的内容，请看 [MCP](mcp.md)。

</div>
