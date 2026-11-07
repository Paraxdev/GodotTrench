---
source: 46b4723cec5ef9109b70a2a54e850458f119c7e9
---

# 从编辑器运行 Godot { #running-godot-from-the-editor }

保存、构建和实时模式都通过 Live Link 和插件通信，所以只要有 Godot 编辑器打开了这个项目，它们就能用。TrenchDot 只有在这几处需要找到 Godot 可执行文件：*Godot > 在 Godot 编辑器中打开项目*、工具栏里的 Godot 机器人，以及没有 Godot 编辑器打开项目时的 *Godot > 运行项目* (F5)，因为它们要自己启动 Godot。

![“Godot”菜单，鼠标悬停在“运行项目”上，右边是它的 F5 快捷键，下面是“在 Godot 编辑器中打开项目”](../assets/running-godot/menu-run-project.png)

## 运行项目 { #run-project }

当有 Godot 编辑器打开了这个项目时，*运行项目*会请那个编辑器保存它打开的场景并播放主场景，就像在 Godot 里按下播放键一样。这样游戏拿到的就是 Godot 构建好的一切，包括实时编辑。状态栏应该会写着“Godot 已保存打开的场景并运行主场景”。如果项目没有主场景，它会这么说，请在 Godot 的 *Project > Project Settings > Application > Run* 下设置一个。

没有 Godot 编辑器时，*运行项目*会用 Godot 可执行文件启动游戏。这时它运行的是场景在 Godot 里最后一次保存时的样子。插件较旧时也一样，状态栏还会补充说游戏可能缺少你最新的地图改动。请在 Godot 里保存它们，或者在那里按播放。

## 找到可执行文件 { #finding-the-executable }

进入*文件 > 偏好设置*，看一看 *Godot 可执行文件*。当 TrenchDot 找到了一个，输入框下面会写着“正在使用”和路径。

![偏好设置里的“Godot 可执行文件”输入框，下面是“正在使用”这一行和路径](../assets/running-godot/prefs-godot-using.png)

当它用黄色写着“未找到”时，点击输入框旁边的“…”按钮，在你的磁盘上选择 Godot 可执行文件，也就是以“Godot_v”开头的那个。如果它选错了 Godot，也这样做。在找到之前，“在 Godot 编辑器中打开项目”会保持灰色，没有 Godot 编辑器打开项目时，“运行项目”也一样。

![空的“Godot 可执行文件”输入框，旁边是“…”按钮，下面是黄色的“未找到”这一行](../assets/running-godot/prefs-godot-not-found.png)

如果偏好设置里没有可用的路径，编辑器会按下面的顺序取第一个匹配项：

1. 环境变量 `GODOT`。
2. `PATH` 里的 `godot` 或 `godot4`，然后是像 `Godot_v4.7.2-stable_win64.exe` 这样下载来的版本，新的优先。项目文件夹里有 `.csproj` 时，Mono 版本优先，因为 C# 项目需要它们。控制台版本会被跳过。
3. 常见的安装文件夹：Windows 上是 WinGet、Scoop、Steam 和 `Programs/Godot`，Linux 上是 `~/.local/bin` 和 Steam，macOS 上是 `/Applications/Godot.app`。

## Godot 按钮 { #the-godot-button }

工具栏右端的 Godot 机器人显示连接状态。把鼠标悬停在上面，就能看到它会做什么。

灰色表示没有打开任何项目，或者没有找到 Godot，所以点击它什么也不会发生。

![鼠标悬停在灰色的 Godot 机器人上，工具提示说没有找到 Godot](../assets/running-godot/toolbar-godot-not-found.png)

普通颜色表示找到了 Godot。点击它，就能用 Godot 编辑器打开项目。

![鼠标悬停在普通颜色的 Godot 机器人上，工具提示写着“在 Godot 编辑器中打开项目”](../assets/running-godot/toolbar-godot-found.png)

青色表示有 Godot 编辑器打开了这个项目。点击它，就能把那个编辑器带到前台。

![鼠标悬停在青色的 Godot 机器人上，工具提示说 Godot 4.7.2-stable (official) 已打开此项目，点击显示](../assets/running-godot/toolbar-godot-open.png)

Godot 在构建时，它旁边会显示一个转圈。如果工具提示说运行中的 Godot 插件较旧，请用 *Godot > 安装或更新插件…* 更新它，才能用上实时模式。

## 项目信任 { #project-trust }

Godot 一打开、播放或构建项目，就会运行项目的脚本，而你下载的项目或地图包可能自带脚本。所以当你打开一个 TrenchDot 还不认识、而且里面有代码的项目时，TrenchDot 会先问你。

TrenchDot 会先在项目里查找代码。在这期间，状态栏会显示*正在查找代码*，项目上不会在 Godot 里启动任何东西，所以大项目在询问或打开之前要等一会儿。

![弹出的提示“你信任此项目中文件的作者吗？”，里面有项目的文件夹、受限模式会阻止的事项、上级文件夹的复选框，以及“信任项目”和“以受限模式打开”两个按钮](../assets/running-godot/trust-prompt.png)

对你自己的项目和你信任的人的项目，点击**信任项目**。先勾选*信任上级文件夹中所有文件的作者*，就能信任那个文件夹里的每个项目，比如你存放所有游戏的那个文件夹。不勾选的话，你只信任这一个项目，而在它里面解压出来的项目，比如带有自己的 `project.godot` 的地图包，不会跟着一起被信任。

拿不准的时候，点击**以受限模式打开**。你仍然可以查看和编辑地图，但 TrenchDot 不会在这个项目上启动 Godot、不会在 Godot 里构建，也不会发送实时编辑。*受限模式*随后会以黄色显示在状态栏右侧，点击它就能再回答一次这个问题。

![状态栏里黄色的“受限模式”标签，它的工具提示说信任之前，TrenchDot 不会在 Godot 中运行此项目](../assets/running-godot/restricted-mode-chip.png)

在受限模式下，*运行项目*、Godot 机器人、*在 Godot 中构建*以及打开实时模式，都会询问“运行会执行其脚本。要信任并继续吗？”，你点击**信任项目**之后才会继续。

打开属于另一个你没有信任过的项目的地图时，会保留当前打开的项目，这时提示的按钮里会多出**保留当前项目**。已打开项目文件夹里的 `project.godot` 会被忽略，和 Godot 忽略它的方式一样。当项目之外的某张地图运行它自己的 GDScript 时，状态行会这么说，“问题”面板会显示在哪里。

在 TrenchDot 开始询问信任之前就已经在*最近的项目*里的项目，算作已信任，而你通过 `--project` 启动编辑器时指定的项目，在那次会话里也算。

> [!NOTE]
> 受限模式只管 TrenchDot 启动的东西。你自己用 Godot 打开项目，它的脚本照常运行。

### 没有代码的项目 { #projects-without-code }

不含代码的项目打开时不会提问，因为里面没有 Godot 要运行的东西。这指的是空文件夹、你刚创建的项目，或者只有 TrenchDot 的插件和它安装的玩法实体包的项目。这里的代码指脚本、C# 项目、原生库、里面有脚本的场景和资源、自动加载、编辑器插件，以及 `logic_script` 里是 GDScript 的地图。插件和实体包只有在其中每个文件都是这个版本所安装的那个时，才算 TrenchDot 自己的，所以改过的或较旧的副本算作代码，大到无法快速查完的项目也算。

TrenchDot 每次打开项目时都会重新查看，什么也不记，所以项目一旦有了脚本，它就会再问一次。想让你自己的项目一直保持已信任，请在它没有代码的时候，用*文件 > 保存*把一张地图保存进去。从那以后 TrenchDot 就信任它，即使你之后在 Godot 里添加了脚本也一样。在受限模式下把地图保存进项目，不会信任任何东西。

### 已信任的项目 { #trusted-projects }

打开*文件 > 偏好设置…*，看底部的*信任的项目*，就能看到你信任的每个项目和文件夹。点击其中一个旁边的**移除**，就能收回这份信任。当它是当前打开的项目时，它会立刻进入受限模式，状态栏里会显示黄色标签，除非这个项目没有代码。

![偏好设置里的“信任的项目”部分，列出两个项目和一个包含其中所有项目的文件夹，每个都带有“移除”按钮，其中一个的工具提示写着“之后 Godot 在此运行脚本前 TrenchDot 会再次询问”](../assets/running-godot/trusted-projects.png)

<div class="td-mcp" markdown>

## MCP { #mcp }

代理永远不会遇到这个提示。`open_project` 和 `get_state` 会报告 `restricted`，在你用那个标签信任项目之前，那些会让 Godot 运行或构建项目的工具都会报错失败。`map_file` 的 `open` 在打开一个未信任项目的地图时会保留当前打开的项目，并把那个项目作为 `other_project` 返回。

两个结果里的 `trust` 都会说明一个项目为什么不在受限模式下：`stored`，用 `--project` 时的 `session`，或者 `no_code`。代理用 `map_file`、`run_action` 或在 `run_script` 里做的保存，永远不会存下信任，所以代理在其中工作的无代码项目，一旦有了脚本就会再被问一次。

</div>
