---
source: 755d17f52aebf0b32513b6e22a04abb368badeab
---

# 构建地图 { #building-maps }

TrenchDot 保存的是 `.tdm` 文件，它自己从不写 Godot 场景。在 Godot 里，插件的 `FuncGodotMap` 节点读取这个文件，生成网格、碰撞和实体节点。结果会像手工做的节点一样随你的场景一起保存，所以除非你要求，运行时不会重新构建任何东西。

## FuncGodotMap 节点 { #the-funcgodotmap-node }

1. 添加节点

    在 Godot 编辑器里，在你的场景中按 Ctrl+A，搜索 `FuncGodotMap` 并添加它。

2. 让它指向你的地图

    选中这个节点后，在 Godot 的检查器里把 *Local Map File* 设为你保存的 `.tdm`。

3. 构建

    点击检查器顶部的 **Build Map**。你应该能在 Godot 的视口里看到这张地图，节点也会多出放着网格和实体的子节点。用 Ctrl+S 保存场景来保留它们。**Clear Map** 会再把它们移除。

检查器里能看到的设置如下：

| 属性 | 作用 |
| --- | --- |
| Local Map File | 要构建的 `.tdm` 的 `res://` 路径。这是通常的选择。 |
| Global Map File | 项目之外的地图文件。两者都设置时，以这个为准。 |
| Auto Rebuild On Save | 只要 TrenchDot 保存这张地图，并且它的场景是 Godot 编辑器里的当前标签页，就重新构建这个节点，参见[保存时重建](live.md#rebuild-on-save)。默认开启。 |
| Map Settings | 纹理、材质和实体定义从哪里来，以及一个地图单位有多大。留空时，它使用项目设置 `func_godot/default_map_settings`，参见[项目设置](project-setup.md)。用旧版插件添加的地图节点，会在这里存下插件的默认地图设置，它的构建会在 Godot 的输出里对此发出警告，清空这个字段就会跟随项目设置。 |
| Use Baked Lighting | 用[在编辑器里烘焙](baked-lighting.md)的光照给地图打光。默认开启，取消勾选就可以改用别的光照贴图工具。 |
| Warm Up Shaders | 在加载地图时编译它的着色器，这样游玩的头几秒不会卡顿。参见[着色器预热](warm-up.md)。 |

插件只构建 `.tdm` 文件。想使用 Quake 的 `.map` 或 Hammer 的 `.vmf`，先在 TrenchDot 里用*文件 > 导入*打开它，再另存为 `.tdm`。

> [!WARNING]
> 构建会删除地图节点的每一个子节点，[叠加层](overlays.md)除外。请把你自己的节点放在地图节点旁边，或者放进叠加层里。

## 三种构建方式 { #three-ways-to-build }

| 触发方式 | 构建什么 | 什么时候用 |
| --- | --- | --- |
| 在 TrenchDot 里保存 | Godot 编辑器里打开的场景中，每个使用该文件的 `FuncGodotMap` | 日常的循环，保存之后去 Godot 里看结果 |
| *Godot > 在 Godot 中构建* | 地图此刻在 TrenchDot 里的样子，不管保存没保存，也不写文件 | 想在 Godot 里试一个未保存的改动，又不想就此定下来 |
| Godot 里的 **Build Map** | 已保存的文件 | TrenchDot 关着的时候，或者重建一个刚打开的场景 |

前两种需要一个已经打开这个项目的 Godot 编辑器，并且它们只会动那里打开的场景。重建会把那个场景标记为已修改，所以要在 Godot 里保存才能留住结果。

在 Godot 打开这个项目之前，*在 Godot 中构建*一直是灰的。它还要求地图至少保存进项目一次，这样 Godot 才知道是哪个 `FuncGodotMap` 在用它。

![Godot 菜单，鼠标停在在 Godot 中构建上](../assets/building/menu-build-in-godot.png)

## 用代码构建 { #building-from-code }

你也可以在游戏运行时构建地图，比如加载从菜单里选出的关卡：

```gdscript
var map := FuncGodotMap.new()
map.map_settings = load("res://demo/demo_map_settings.tres")
map.local_map_file = "res://demo/maps/scripted_scene.tdm"
add_child(map)
map.build()
```

节点完成时会发出 `build_complete`，地图文件缺失或不是 `.tdm` 时会发出 `build_failed`。

## 环境和太阳 { #environment-and-sun }

worldspawn 上的天空、雾和太阳这些键，会在地图里构建出一个 `WorldEnvironment` 和一盏名为 `sun` 的灯，这样地图不用在 Godot 里做任何设置，看起来就是有光照的。按 Escape 让什么都不被选中，检查器就会显示*Map (worldspawn)*，并列出这些键。它的 `environment` 键决定构建什么。没设置的键不会进入地图，所以一个键都没设置的地图根本不会构建环境。如果用旧版插件做的项目没有列出这些键，请更新插件，再[导出游戏配置](project-setup.md#the-game-config)一次。

| `environment` | 构建 |
| --- | --- |
| `1` | 环境和太阳。这是默认值。 |
| `sun_only` | 只有太阳，适用于自带 `WorldEnvironment` 的游戏 |
| `0` 或 `none` | 什么都不构建 |

如果场景已经有自己的 `WorldEnvironment` 或 `DirectionalLight3D`，地图会保留它，不再添加，所以自带风格的游戏总是优先。当多张地图共用一个场景时，第一个构建的会给它打光。

## 构建报告 { #build-report }

`build()` 返回一个 Dictionary，也保存在 `build_report` 里。它会告诉你每一步花了多久、每个类的实体各构建了多少个，以及地图有多少个顶点。最有用的是构建时没找到的东西的清单，原因通常是拼写错误，或者文件被改了名、或者从没提交过：

| 键 | 它列出什么 | 你在游戏里看到什么 |
| --- | --- | --- |
| `missing_classes` | 没有任何实体定义描述的实体类 | 笔刷实体保留它的网格，但没有脚本也没有碰撞，点实体是一个空的 `Marker3D`，参见[什么都不做的实体](entities-troubleshooting.md) |
| `missing_models` | 不存在的模型路径，以及使用每个路径的实体 | 道具没有模型 |
| `missing_materials` | 既没有材质文件也没有图片的面纹理 | 这些面显示占位纹理 |
| `unknown_tool_textures` | 地图设置里命名不同的 `special/clip` 之类的工具纹理 | 这些面仍然什么都不画，但原点笔刷不会设置它实体的原点 |
| `unresolved_targets` | 目标在地图或它的叠加层里匹配不到任何 targetname 的输出和目标键 | 这个输出触发在空处 |

想在每次构建之后打印它，请在节点的 *Build Flags* 下勾选 *Print Build Report*，或者在代码里把 `FuncGodotMap.BuildFlags.PRINT_REPORT` 加进 `build_flags`。测试或 CI 检查则可以直接读这个 Dictionary：

```gdscript
var report := map.build()
if not report["missing_models"].is_empty():
	push_error("missing models: %s" % [report["missing_models"]])
```

每次构建也会在 Godot 的输出里，针对没找到的东西打印警告：所有缺失的类合成一条，每个缺失的模型、材质或工具纹理各一条，不管有多少个道具共用同一个路径。保存之后，TrenchDot 会在它的问题面板里列出同样的警告。

> [!NOTE]
> 你在 `build()` 之后添加的叠加层，在报告查找目标时还不存在，所以它们的 targetname 会出现在 `unresolved_targets` 下面。

## 损坏的地图 { #damaged-maps }

损坏的地图仍然会用能读取的部分构建，每个读不了的部分都会带一条 `[TDM]` 警告。参见 [.tdm 地图格式](../format/recovery.md)。
