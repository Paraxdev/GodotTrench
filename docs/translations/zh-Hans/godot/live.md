---
source: 15ac2f37b7d278fa849fb066b91734f671f35658
---

# 保存、实时模式与热重载 { #saving-live-mode-and-hot-reload }

## 保存时重建 { #rebuild-on-save }

让带有你的 `FuncGodotMap` 的场景在 Godot 编辑器里保持打开，然后在 TrenchDot 里直接按 Ctrl+S 保存就行。只要它的标签页是 Godot 当前显示的那一个，并且地图节点打开了 *Auto Rebuild On Save*，Godot 就会立刻重新构建这个场景。另一个标签页里使用这张地图的场景，会在你切换到它的标签页时重新构建。关闭再重新打开标签页，或者重新加载场景，都不会破坏这一点。保存一个[预制体](../editor/prefab-library.md)，会重新构建那个场景里直接或通过另一个预制体用到它的地图。

左下角的状态栏会告诉你 Godot 做了什么。成功时，它应该写着：

![状态栏显示“实时链接：Godot 重新构建了 1 个地图节点”](../assets/live/status-rebuilt.png)

在 Godot 里保存场景来保留结果。如果状态栏说的是别的，这条消息会告诉你原因：

![状态栏显示 room.tscn 使用 room.tdm，但不是 Godot 当前的场景标签页，切换过去时会重新构建](../assets/live/status-not-current-tab.png)

| 状态栏显示 | 怎么做 |
| --- | --- |
| 场景不是 Godot 当前的场景标签页 | 切换到那个标签页，它随即重新构建 |
| 当前场景标签页里没有这张地图的 `FuncGodotMap` | 打开构建这张地图的场景，或者在它的地图节点上打开 *Auto Rebuild On Save* |
| Godot 没有打开任何场景 | 在 Godot 里打开带有这张地图的场景，如果已经打开了，请接着往下读 |
| Godot 没有带着 TrenchDot 插件运行 | 在启用了插件的情况下，用 Godot 编辑器打开项目 |
| Godot 重新构建了地图，但地图设置里缺少某些类的定义，所以它们被构建成普通节点 | 添加这些定义，参见[什么都不做的实体](entities-troubleshooting.md) |
| Godot 重新构建了地图，但带有警告，请看“问题”面板 | 在“问题”面板里阅读它们，它们以“Godot:”开头 |

同一时间只有一个 Godot 编辑器能占用 Live Link 的端口。用同一个项目打开的第二个编辑器，会在它的输出面板里警告 Live Link 无法监听，而 TrenchDot 仍然和第一个编辑器通信，那个编辑器可能没有打开任何场景。关掉你不用的那个编辑器，另一个会在几秒钟内接管这条链接。（注意：*Godot > 在 Godot 编辑器中打开项目*会把已经打开了这个项目的编辑器带到前台，而不是再启动第二个。）

## 实时模式 { #live-mode }

实时模式会在你保存之前，就把你的编辑推送到 Godot 编辑器中打开的场景里，这样你就能在真正的渲染器里判断光照和比例。

1. 在 Godot 里打开场景

    地图必须先保存到项目里一次，而且 Godot 里打开的场景需要有一个用到它的 `FuncGodotMap`。

2. 打开它

    点击工具栏最右端、Godot 机器人旁边的链接按钮，或者使用*Godot > 实时模式*。按钮会亮起来，如果它还在等什么，工具提示会告诉你。

    ![工具栏上，实时模式的链接按钮已打开，它的工具提示说编辑会在保存之前到达 Godot](../assets/live/toolbar-live-mode-on.png)

3. 照常编辑

    每一处改动都会立刻出现在 Godot 里。

    ![TrenchDot 里的编辑在保存之前就出现在 Godot 编辑器里](../screenshots/TrenchDot-demo-live-view.gif)

实时模式需要 Godot Live Link，它默认是开着的。两者都在*文件 > 偏好设置*里，其中 *Godot 实时模式*和工具栏按钮是同一个开关。

![偏好设置里的 Godot Live Link、Godot 实时模式和运行中游戏热重载这几行](../assets/live/prefs-live-link.png)

拖动时节点会跟着一起移动，而且大多数编辑只会重建它们碰到的部分，所以预览能跟得上。在图层上切换 *Omit From Export*，以及编辑预制体实例，则会在你停下来之后重建整张地图。

预览和完整构建很接近，但并不总是完全一样。保存时总是做完整构建，不保存就退出则会让 Godot 回到已保存的文件，所以你在实时模式里试的任何东西，不保存就不会保留。

### 实时模式什么时候重建整张地图 { #when-live-mode-rebuilds-the-whole-map }

当场景和完整构建会得到的结果不再一致时，Godot 会自己重建整张地图。实时模式在某个场景上启动时会立刻这样做，其他情况则等你停下编辑之后：

| 时机 | 原因 |
| --- | --- |
| 实时模式在一个用别的实体定义构建的场景上启动 | 这个场景是在你安装玩法实体包或添加某个定义之前构建的，所以有些实体是普通节点 |
| 实时模式在一个被之前的实时编辑改动过的场景上启动 | 场景里还留着早先某次会话的实时编辑，你从来没有保存过它们 |
| 实时模式运行期间实体定义发生变化 | 你在 Godot 的检查器里保存了一个 FGD 或一个定义，某个脚本类变了，或者来了新文件，比如装好的一个包 |
| 你保存了地图用到的预制体 | 场景里显示的还是预制体原来的样子 |

不处于实时模式时打开的场景，实体保持构建时的样子，其中某个场景是用别的实体定义构建的话，Godot 的输出面板会发出警告。在它里面按下 **Build Map** 然后保存。

### 没有定义的实体 { #entities-without-a-definition }

当实时模式构建一个 Godot 里没有定义其类的实体时，状态栏会指出这个类，以及应该把它加到哪个 FGD 里：

![状态栏显示 Godot 没有 crypt_lamp 的定义，显示为普通节点，请把这个类添加到 res://crypt_settings.tres 的 FGD](../assets/live/status-no-definition.png)

只要地图还在用那个类，实时模式按钮的工具提示里就会一直带着同样的提示：

![实时模式按钮已打开，它的工具提示说编辑会在保存之前到达 Godot，并且 Godot 没有 crypt_lamp 的定义](../assets/live/toolbar-live-mode-no-definition.png)

添加定义并保存，Godot 就会自己重建地图。下一次保存之后，这条提示就会消失。定义该放在哪里，参见[什么都不做的实体](entities-troubleshooting.md)。

## 运行中游戏的热重载 { #hot-reload-in-a-running-game }

想在已经运行的游戏里看到你的保存结果：

1. 添加自动加载

    在 Godot 里进入 *Project > Project Settings > Globals > Autoload*，添加这个脚本：

    ```
    res://addons/func_godot/src/trenchdot/runtime/trenchdot_hot_reload.gd
    ```

2. 从 Godot 编辑器运行游戏

    自动加载只在从 Godot 编辑器启动的游戏里监听，除非你把它的 `enabled_in_exported_builds` 设为 true。

3. 在 TrenchDot 里保存

    游戏会重新构建用到被保存文件的地图，并发出 `map_reloaded(map)`。用这个信号把玩家放回出生点，或者重置游戏状态。状态栏末尾应该写着“，运行中的游戏重新加载了 1 张地图”。

它监听 7843 端口，这个端口必须和编辑器偏好设置里的*运行中游戏热重载*一致。TrenchDot 只在 Live Link 开着的时候才向它发送保存通知。
