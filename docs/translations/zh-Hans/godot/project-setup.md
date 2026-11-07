---
source: 0b6155b82bf9e0fff2345e05627d7c675a868912
---

# 项目设置 { #project-setup }

插件的默认值对你的头几张地图就够用了。一旦你要写自己的实体，或者想换一个纹理文件夹，就给项目配上它自己的资源，因为插件更新会替换掉默认值。

## 安装和更新插件 { #installing-and-updating-the-addon }

每次你打开项目，TrenchDot 都会检查它的插件。当 `addons/func_godot` 不存在，或者版本和编辑器不同时，*设置此项目*窗口会弹出一次，状态栏右下角会一直留着一条黄色的提示。把鼠标悬停上去能看到两个版本，点击它就能把窗口叫回来。

![状态栏上，鼠标悬停在黄色的“插件 0.0.9，编辑器 0.1.0”提示上，显示它的工具提示](../assets/project-setup/status-addon-mismatch.png)

你也可以随时用 *Godot > 安装或更新插件…* 打开这个窗口。

![“Godot”菜单，鼠标悬停在“安装或更新插件…”上](../assets/project-setup/menu-install-update-addon.png)

它只提供你的项目下一步需要的那一步：

| 项目里有 | 窗口提供 |
| --- | --- |
| 没有插件 | **安装插件**，然后是**启用插件** |
| 较旧的插件 | **将插件更新到**编辑器的版本 |
| 较新的插件，或者没有版本号的插件 | **将插件替换为**编辑器的版本 |
| 插件已安装，但没有启用 | **启用插件**，它会把插件加进 `project.godot` 的 `[editor_plugins]`，别的什么都不改 |
| 作为 git 检出或子模块的插件 | 什么都没有。请用 git 更新它，编辑器绝不会替换它 |

![“设置此项目”窗口，说插件 0.0.9 比编辑器旧，带着“将插件更新到 0.1.0”按钮](../assets/project-setup/setup-update-addon.png)

> [!WARNING]
> 更新之前请先关闭 Godot。它在重启之前会一直运行旧的插件，所以只要 Live Link 看到 Godot 打开着这个项目，按钮就会保持灰色。启用插件时也要关着 Godot，否则 Godot 会把它自己那份 `project.godot` 写回去。在 Godot 的 *Project > Project Settings > Plugins* 里勾选它，同样可以。

旧的插件不会被删除，它会移到你项目里的 `.trenchdot/addon-backups/func_godot-<version>`。Godot 会跳过以点开头的文件夹，所以它既不会导入这份备份，也不会把它当成第二个插件列出来。新插件能用之后，就把它删掉。（注意：下载的内容会先解压，然后才换进去，所以失败或取消的更新会让旧插件保持原样。）

如果下载失败，窗口会链接到[发布页](https://github.com/Paraxdev/TrenchDot/releases)和 [itch.io](https://paraxdev.itch.io/trenchdot)，它们提供的是同一个 `func_godot-trenchdot-addon.zip`。把它解压到项目文件夹里，得到 `addons/func_godot`，并且要替换掉旧文件夹，而不是直接解压覆盖在上面。

## 核心实体和玩法实体包 { #core-entities-and-the-gameplay-entities-pack }

每个项目都会从插件得到一个小小的核心：worldspawn、`func_detail`、`func_illusionary`、`func_geo`、`light`、`light_spot`、`light_probe`、`env_reflection_probe`、`env_voxel_gi` 和 `info_player_start`。

门、按钮、升降机、火车、触发器、逻辑、生成器、传送、路径点、道具、特效，以及[实体参考](../gameplay/entities/README.md)里的其余内容，都包含在玩法实体包里。你想要的时候再安装它，它会被复制到 `res://trenchdot/entities`：

| 路径 | 内容 |
| --- | --- |
| `gameplay_fgd.tres` | 这个包的 FGD 文件，列出每个定义 |
| `definitions/` | 每个实体一个定义：它的键、输入、输出和编辑器小工具 |
| `scripts/` | 每个实体运行的 GDScript |
| `pack.json` | 安装了什么，这样更新时才能分辨出你的修改。请保持原样 |

这些副本属于你的项目。你可以修改脚本或定义，删掉你不用的实体，或者把你自己的定义加进 `gameplay_fgd.tres`。插件自己从不定义这些类名或脚本类，所以你在那里做的任何事都不会和它冲突，插件更新也不会撤销它。

### 安装和更新实体包 { #installing-and-updating-the-pack }

1. 安装实体包

    在*设置此项目*窗口里，点击插件下方的**安装玩法实体**。

    ![“设置此项目”窗口，里面是“玩法实体”部分和它的“安装玩法实体”按钮](../assets/project-setup/setup-gameplay-entities.png)

    项目里还只有核心时，同一个按钮在“玩法”菜单的顶部，以及实体面板里。

    ![“玩法”菜单，鼠标悬停在“安装玩法实体”上，显示它的工具提示](../assets/project-setup/menu-install-gameplay-entities.png)

2. 检查是否成功

    窗口现在应该用绿色写着“已安装在 res://trenchdot/entities”，编辑器会立刻提供新的实体。

    ![安装之后的“玩法实体”部分，说已向 res://trenchdot/entities 添加了 86 个文件，并且如果 Godot 打开着这个项目，就重启 Godot](../assets/project-setup/setup-gameplay-installed.png)

    当有 Godot 编辑器打开着这个项目时，TrenchDot 会请它接收新文件，窗口也会这么说。处于[实时模式](live.md#when-live-mode-rebuilds-the-whole-map)的场景随后会自己重建。其他用到你的地图的场景，请按下 **Build Map** 并保存，因为它们是在没有这个包的情况下构建的。Godot 没在运行，或者运行的是较旧的插件时，窗口会说如果 Godot 打开着这个项目就重启它，然后构建并保存那些场景。

3. （可选）以后更新它

    当插件更新带来较新的包时，窗口会提供**更新玩法实体**，“玩法”菜单也会带着改动的文件数把它显示出来。你点击之前，什么都不会更新。更新会添加新文件，并替换你没动过的那些。你编辑过的文件保持原样，编辑器会点出它们的名字，你删掉的文件仍然保持删除。

实体还在插件里的时候做的旧地图继续有效。当某个项目的地图用到了项目里没有任何东西定义的实体包实体时，这个包会自己安装，编辑器打开项目时是这样，Godot 带着更新后的插件启动时也是这样。不论有没有定义，`.tdm` 文件都会保留每个实体。

> [!NOTE]
> 插件里的模板 `addons/func_godot/gameplay_pack` 有一个 `.gdignore` 文件，好让 Godot 跳过它。请保留这个文件，否则包一旦安装，Godot 就会把这个包的每个脚本类找到两遍。

### 哪个定义生效 { #which-definition-wins }

核心 FGD `res://addons/func_godot/fgd/trenchdot/trenchdot_fgd.tres` 会自己读取已安装的包，所以每个基于插件默认值建立的 FGD，不用列出它就能得到这个包。当两个定义用了同一个类名时，后面的生效，顺序如下：

1. FuncGodot 的类和插件的核心
2. `res://trenchdot/entities` 里已安装的包
3. 你自己的 FGD 文件里的实体

想修改一个实体，就编辑它在包里的副本，或者在你自己的 FGD 里定义同一个类名。两种做法都行，但一个类名定义了两次很容易忘记，所以编辑器会告诉你：项目打开时，状态栏会点出生效的那个定义，“问题”面板会对地图用到的每个重复定义的类给出警告。删掉两个中的一个就能解决。

## 你自己的资源 { #your-own-resources }

三种资源协同工作。FGD 文件列出你的地图可以用的实体。地图设置告诉构建纹理在哪里，以及要读哪个 FGD。游戏配置让 TrenchDot 了解这两者，所以编辑时编辑器提供的实体和纹理是同样的。

最快的办法是把 `demo_fgd.tres`、`demo_map_settings.tres` 和 `demo_game_config.tres` 从 [`godot/demo`](https://github.com/Paraxdev/TrenchDot/tree/main/godot/demo) 复制到你的项目里，再改好路径。想自己做的话，在 Godot 编辑器里这样做：

1. 创建一个 `FuncGodotFGDFile`。把 *Base Fgd Files* 设为 `res://addons/func_godot/fgd/trenchdot_default_fgd.tres`，这样你就保留了核心和已安装的包，然后在 *Entity Definitions* 下添加你自己的实体。没有这个基础的话，你的 FGD 里一个都没有，连 worldspawn 构建出来也没有碰撞。
2. 创建一个 `FuncGodotMapSettings`。把 *Entity Fgd* 设为第 1 步的 FGD，把 *Base Texture Dir* 设为你的纹理文件夹（默认是 `res://textures`）。新的地图设置资源一开始用的是 `trenchdot_default_fgd.tres`，它有核心和包，但没有你自己的实体。
3. 创建一个 `TrenchDotGameConfig`。把它的 *Fgd File* 和 *Map Settings* 设为上面那两个资源。（可选）设置 *Game Name*，编辑器会把它显示在工具栏右上角、项目文件夹的旁边。
4. 在 *Project > Project Settings* 里，把 `func_godot/default_map_settings` 指向你的地图设置，把 `trenchdot/game_config` 指向你的游戏配置。每个 *Map Settings* 字段为空的地图节点都会跟随它。用旧版插件添加的节点，会在那个字段里存着插件的默认值，所以请在那里把它清空。

编辑器提供的是游戏配置的 FGD 里的实体，但 Godot 用的是地图设置的 FGD 来构建，所以请让两者保持一致。当它们不同时，Godot 每次导出游戏配置都会在它的输出面板里警告，TrenchDot 的“问题”面板则会点出 Godot 构建所用的 FGD 里缺少的每个实体。

> [!TIP]
> 像演示那样，把地图设置里的 *Entity Name Property* 设为 `targetname`。这样构建出来的节点就会以它们的 targetname 命名，场景树也就好读。

## 游戏配置 { #the-game-config }

TrenchDot 从 `res://trenchdot_game.json` 了解你的实体、纹理文件夹和单位比例，这个文件是由游戏配置写出来的。每当 Godot 发现文件变化时，插件就会再次导出它，编辑器会在一秒内重新加载它。导出的内容还会写明默认的地图设置，以及它们的 FGD 定义的类，编辑器就是靠这个来警告 Godot 会在没有定义的情况下构建的实体。想手动导出，就在 Godot 里用 *Project > Tools > TrenchDot: Export Game Config*，或者以无头方式运行它，比如在 CI 里：

```sh
godot --headless --path <project> --script res://addons/func_godot/src/trenchdot/export_game_config_cli.gd
```

## 项目设置 { #project-settings }

大多数项目只会改第一个和第三个。Live Link 相关的几项，在另一个程序已经占用端口时才重要。

| 设置 | 默认值 | 用途 |
| --- | --- | --- |
| `func_godot/default_map_settings` | 插件的默认值 | 每张没有设置自己的那份的地图所用的地图设置 |
| `func_godot/default_inverse_scale_factor` | 32 | 每米的地图单位数 |
| `trenchdot/game_config` | 插件的配置 | 写出 `trenchdot_game.json` 的那份配置 |
| `trenchdot/auto_export_game_config` | true | 文件变化时再次导出游戏配置 |
| `trenchdot/live_link_enabled` | true | 运行 TrenchDot 与之通信的 Live Link 服务器 |
| `trenchdot/live_link_port` | 7842 | 必须与编辑器偏好设置里的 *Godot Live Link* 一致。改动会立刻生效。取值在 1024 到 65535 之间，其他值会带着警告退回到 7842 |
| `trenchdot/threaded_build` | true | 在 WorkerThreadPool 上转换笔刷和地形区块，这样大地图构建得更快 |
| `trenchdot/live_chunk_size` | 16.0 | 在实时模式下，worldspawn 几何体会被切成这个大小（以米计）的区块，这样一次编辑只会重建它所在的区块 |
| `trenchdot/csharp_entity_dirs` | `res://` | 构建时在哪里查找 C# 实体 |
