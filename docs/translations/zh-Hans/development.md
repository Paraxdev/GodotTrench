---
source: d0d174df9061792086d11223ac4029e9d23f9d56
---

# 开发 { #development }

如果你想从源码构建编辑器、修改插件或者写文档，这一页就是写给你的。

## 构建 { #building }

克隆时要带上子模块，`godot/addons/func_godot` 里的 Godot 插件就是一个子模块：

```sh
git clone --recursive https://github.com/Paraxdev/TrenchDot.git
```

已经不带它克隆过了？运行 `git submodule update --init`。没有插件时编辑器也能构建，但测试和 Godot 项目需要它。

```sh
cargo run -p td_editor --release            # start the editor
cargo run -p td_editor --release -- map.tdm # open a map
```

你需要一块支持 Vulkan、DX12 或 Metal 的 GPU，在 Linux 上还需要 X11 或 Wayland 会话。

最低要求是 Rust 1.95，egui 0.36 需要它。[rust-toolchain.toml](https://github.com/Paraxdev/TrenchDot/blob/main/rust-toolchain.toml) 固定在 1.98.1，也就是 CI 使用的版本，所以 rustup 会在你第一次构建时安装它。

## 检查 { #checks }

每个克隆运行一次 `./tools/fmt --install`。它会启用一个提交前钩子，用来格式化已暂存的 Rust 代码。

```sh
./tools/fmt --check
python tools/gen_licenses.py --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p td_editor --test e2e -- --ignored --test-threads=1
godot --headless --path godot --import                            # twice on a fresh checkout
godot --headless --path godot --script res://tests/run_tests.gd
godot --headless --path godot --script res://tests/build_showcase.gd
```

CI 会运行所有这些，所以推送之前请先运行一遍。

e2e 测试会启动真正的编辑器，并用模拟输入来驱动它。这需要 GPU 和桌面会话，所以除非你传 `--ignored`，否则 `cargo test` 会跳过它们。CI 则在 Linux 上用软件 Vulkan 运行它们。想在没有窗口的情况下测试 Live Link，参见 [Live Link 协议](godot/live-link.md#testing-without-a-window)。

> [!NOTE]
> `godot --headless --script` 什么都不渲染（截图和渲染检查的结果是黑的），也不运行项目的自动加载，只运行脚本本身。请针对数据和节点来写测试，而不是针对像素或自动加载的状态。

想给地图构建计时，运行 `godot --headless --path godot --script res://tests/bench_build.gd -- runs=5 threaded=1`。不要为此使用 `build_showcase.gd`，它会覆盖已提交的展示场景。

## git 里的地图文件 { #map-files-in-git }

`.tdm` 地图是二进制文件，参见 [.tdm 地图格式](format/README.md)。仓库的 `.gitattributes` 已经标记了它们。想要可读的 diff，你只需要每个克隆执行一次[审阅更改](editor/reviewing.md#maps-in-git)里的那行 `git config`。

## 生成的内容 { #generated-content }

有些文件是由脚本生成的，不是手写的。当你改动了某个脚本读取的内容，就要重新运行它。

| 命令 | 重新生成 |
| --- | --- |
| `cargo run -p td_formats --example demo_maps` | `godot/tests/maps` 里的测试地图和 `godot/demo/maps` 里的小型演示地图 |
| `python tools/fetch_demo_textures.py` | 演示用的 CC0 照片纹理，参见[纹理尺寸](godot/materials.md#texture-size) |
| `python tools/fetch_demo_props.py` | `godot/models/polyhaven` 里的 CC0 Poly Haven 道具，并恢复玻璃和火焰的 alpha 贴图。`--fix-alpha` 只恢复它们 |
| `cargo run -p td_samples --bin showcase --release` | 演示用的 Blockbench 模型 |
| `cargo run -p td_editor --example export_addon_fgd` | 插件的核心 FGD 文件，以及由 `builtin_entities.json` 和 `gameplay_entities.json` 生成的玩法实体包模板，写入插件子模块，然后是演示项目里已安装的那份实体包副本 |
| `python tools/gen_licenses.py` | [LICENSES.md](https://github.com/Paraxdev/TrenchDot/blob/main/LICENSES.md) 里的 Rust crate 表 |

## Godot 插件 { #the-godot-addon }

插件有自己的仓库 [trenchdot_func](https://github.com/Paraxdev/trenchdot_func)，检出在 `godot/addons/func_godot`。它的测试放在这里的 `godot/tests` 里，因为它们需要演示项目。要修改插件：

1. 在 `godot/addons/func_godot` 里提交修改，并把它推送到 trenchdot_func。
2. 在这里用 `git add godot/addons/func_godot` 暂存新的子模块提交，并和需要它的 TrenchDot 改动一起提交。

请先推送插件。如果一个 TrenchDot 提交指向了 GitHub 上没有的插件提交，它在 CI 里检出时会失败。

`plugin.cfg` 的 `version` 与 [Cargo.toml](https://github.com/Paraxdev/TrenchDot/blob/main/Cargo.toml) 的工作区版本一致。请把两者一起升级，当项目的插件版本与它不一致时，编辑器会发出警告。

## 翻译 { #translations }

编辑器界面提供英语、德语、法语、西班牙语、葡萄牙语、俄语、简体中文和日语，参见[语言](editor/interface.md#language)。英语是源语言，其余七种放在 [crates/td_editor/locales](https://github.com/Paraxdev/TrenchDot/tree/main/crates/td_editor/locales) 里，每种一个 JSON 文件，在构建编辑器时嵌入。那里的 README 写明了格式、复数形式和一份术语表。

在代码里，英文文本就是键。把字面量包在存放它的地方的 `en!("...")` 里，以及绘制它的地方的 `tr(...)`、`trf!("... {n} ...", n = count)` 或 `tr_n!(count, "{n} brush", "{n} brushes")` 里，并把同一个键加到每个文件里。没有对应条目的文本会以英文显示。`crates/td_editor/src/mcp` 下的代码、日志行和地图内容从不经过它们，MCP 的回复和错误保持英文。`Msg` 会把英文文本和译文放在一起，这样状态栏可以显示其中一个，而 MCP 读取另一个。

```sh
TD_L10N_TODO=/tmp/l10n cargo test -p td_editor --lib i18n
```

只要某个文件缺少代码用到的键，或者还留着代码已经不用的键，这次运行就会失败。设置了 `TD_L10N_TODO` 并指向一个已存在的文件夹时，它还会为每种语言写出一个列出缺失键的文件。另一个测试会扫描源码里跳过了查找的界面文本，所以把原始字面量交给标签会让它失败。这些翻译由 AI 撰写，没有经过核对，母语者的修正只会改动那种语言的文件。像 e2e 测试那样用 `--default-prefs` 启动的编辑器永远是英文。

手册也有翻译，参见[翻译手册](#translating-the-manual)。它的页面会按编辑器显示的方式来称呼菜单项和按钮，所以在你改动某个目录里的值之后，请运行 `python3 tools/translations.py check`。它会列出仍在使用旧措辞的页面。

## 文档 { #documentation }

文档是 `docs/` 里的 Markdown 页面，由 [Zensical](https://zensical.org) 把它们变成网站。仓库根目录的 `zensical.toml` 负责配置它，`docs/SUMMARY.md` 是目录。没有列在那里的页面仍然会被构建，只是在导航里没有位置。`docs/requirements.txt` 固定了 Zensical，`docs/stylesheets/td.css` 放着颜色，`docs/javascripts/td.js` 放着 MCP 开关和术语卡片，`docs/overrides` 替换了几个主题模板。

请在仓库根目录运行这些：

```sh
python3 -m venv .venv                  # once
. .venv/bin/activate                   # .venv\Scripts\activate on Windows
pip install -r docs/requirements.txt   # once, and again when the pin changes
zensical serve                         # live preview of the English site on http://localhost:8000/TrenchDot/
python3 tools/translations.py check    # the translated pages against their English source
python3 tools/build_site.py            # the build CI runs, English at the root of _site and a folder per language
python3 tools/test_docs.py             # tests of the three tools and of the built site
python3 tools/check_site.py            # the link check CI runs on _site
```

`build_site.py` 会为英文构建一次 `zensical build --clean --strict`，再为每种语言各构建一次，所以指向任何语言里都不存在的页面或标题的链接会让它失败。写错的 `SUMMARY.md` 条目、缺失的图片或者被漏出导航的页面能通过它，`tools/check_site.py` 会在每种语言里抓住这些。想只构建某几种语言，就把它们列出来：`python3 tools/build_site.py de ja`。

标题的 id 是它的文字转成小写，空格变成连字符，其他标点被去掉，所以 `## path_corner` 就是 `#path_corner`。编辑器在 [help.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/help.rs) 里的帮助链接指向这些 id，并且有测试来检查它们，所以只有连同链接一起才能重命名标题。

提示框是 GitHub 风格的警示：`> [!NOTE]`、`> [!TIP]` 或 `> [!WARNING]` 单独占一行，文字写在下面被引用的行里。这样它们在 GitHub 上也能正常渲染。属于编号步骤的一切，包括第二段、图片或代码块，都要缩进 4 个空格，否则会掉出列表。

页面的图片放在 `docs/assets/<page>/` 里。

如果新手不会知道某个术语，就在 `GLOSSARY.md` 里给它一个条目：一个 `## term` 标题加上一段话的定义。它在每个页面上第一次出现时就会链接到这里，悬停时显示定义。代码、链接和标题不会被处理。

`.github/workflows/pages.yml` 会为改动了 `docs/`、`zensical.toml`、文档工具或编辑器目录的拉取请求构建网站，并在它们进入 `main` 之后发布到 GitHub Pages。

### 翻译手册 { #translating-the-manual }

网站还会构建成德语、法语、西班牙语、葡萄牙语、俄语、简体中文和日语，也就是编辑器的语言。英语是源语言，其他语言按编辑器的代码来命名：`de`、`fr`、`es`、`pt`、`ru`、`zh-Hans` 和 `ja`。每种语言的网站在 `/TrenchDot/<code>/`，编辑器以那种语言运行时会在那里打开手册。这些翻译由 AI 撰写，没有母语者核对过，所以每个翻译后的网站都会用一条读者可以关闭的横幅说明这一点。欢迎精通该语言的人来修正。

**文件放在哪里。** 翻译后的页面放在 `docs/translations/<code>/` 里，名字和英文页面相同，`docs/editor/brushes.md` 对应 `docs/translations/de/editor/brushes.md`。页面旁边，`site.toml` 放着不是页面的文本：主题语言、网站描述、配色方案按钮和 MCP 开关的名字、横幅、未翻译页面的提示、提示框的标题，以及一张 `[nav]` 表。`[nav]` 表把 `SUMMARY.md` 里的每个英文标题对应到它的译文，表里没有的标题会以英文显示。语言列表是 `zensical.toml` 里的 `[[project.extra.alternate]]`。

**source 行。** 翻译后的页面以一段 front matter 开头，它写明了所依据的英文页面：

```yaml
---
source: 0123456789abcdef0123456789abcdef01234567
---
```

`source` 就是 `git hash-object docs/<page>` 对英文页面打印出的值。只要它还匹配，翻译就是最新的，构建会使用它。一旦英文页面改变，它就不再匹配，翻译就过期了，构建会改为显示英文页面，并在标题后加一条简短的提示。所以修改英文页面永远不会让 CI 失败，也不需要改翻译，页面只是在其他网站上变回英文，直到有人更新它。缺失的页面也以同样的方式回退，所以每个页面在每种语言里都存在，语言按钮总是落到同一个页面上。

**命令。** 它们都在仓库根目录运行。

| 命令 | 作用 |
| --- | --- |
| `python3 tools/translations.py status` | 列出过期、缺失和孤立的页面，以及没有条目的导航标题。它从不失败。加 `--lang de` 看单一语言，加 `--diff` 看每个页面翻译之后英文改了什么。 |
| `python3 tools/translations.py prep --lang de editor/brushes.md` | 打印译者在处理一个页面时需要的东西：front matter、标题 id、带译文的标签、术语表里的词和英文文本。 |
| `python3 tools/translations.py check [--lang de] [page ...]` | 当某个最新的翻译违反了下面的规则时失败。你点名的页面无论处于什么状态都会被检查，所以可以在它变成最新之前先检查。 |
| `python3 tools/translations.py stamp --lang de editor/brushes.md` | 把 `source` 设为英文页面现在的样子，其他什么都不动。 |

**翻译或更新一个页面。** 四步：

1. 对这个页面运行 `prep`。对于过期的页面，它还会显示自旧 `source` 以来的英文 diff。
2. 在 `docs/translations/<code>/` 里写出这个页面，开头放 `prep` 打印的 front matter。逐行保持英文的结构：相同的标题、列表、图片、链接、提示框和 `td-mcp` 块。只翻译文字、图片的替代文字和链接文字，别的都不动，用那种语言的语气，见下文。
3. 对这个页面运行 `check`，修好它报告的问题，直到干净为止。
4. 运行 `python3 tools/build_site.py`、`python3 tools/check_site.py` 和 `python3 tools/translations.py check`，然后把同一种语言的页面一起提交。

只有当英文的改动不需要翻译做任何改动时才用 `stamp`，比如错别字、改了链接目标，或者意思不变的改写。它不会查看页面，所以请先读英文的 diff。页面所讲内容的改变需要编辑翻译，新的 `source` 随之而来。

**语气。** 每种语言都保持[入门](getting-started.md)那种随和的语气，用软件文档在那里使用的称呼形式：

| 语言 | 称呼 |
| --- | --- |
| de | du，小写 |
| fr | vous |
| es | tú，用中性西班牙语，复数用 ustedes，就像目录的 README 所说 |
| pt | você，巴西葡萄牙语 |
| ru | вы，小写 |
| zh-Hans | 你 |
| ja | です/ます 体，不用代词 |

**`check` 会强制执行什么。** 这些规则对每个最新的翻译都成立，`check` 在 CI 里先于构建运行。

* 标题 id 保持英文的那个。每个标题，包括标题页的主标题，都带有它对应的英文标题的 id，`## Größe ändern { #reshape }`，顺序相同，重复后缀也相同（`{ #inputs_1 }`）。Zensical 生成 id 时会丢掉 ASCII 以外的字母，而编辑器的 F1 链接、页面之间的链接、术语表和语言按钮都依赖英文的 id。
* 代码保持英文，并且逐字节一致。围栏代码块连同注释一起照抄，英文页面的每个行内代码片段都要出现在译文里。实体类名、键、输入和输出、MCP 工具名、JSON、文件名、按键名（`Ctrl`、`F1`）和产品名（TrenchDot、Godot、MCP、Live Link）同样不翻译。
* 链接和图片保持它们的目标，只有链接文字和替代文字会变。截图用的是英文版的那些，翻译后的页面指向同样的文件，所以部署的网站里每张图只有一份。
* 标签是编辑器的。粗体片段，或者包含 ` > ` 的斜体片段，只要它的各个部分都是目录 `crates/td_editor/locales/<code>.json` 里的键，就必须按目录的写法出现在翻译后的页面里，各部分用 ` > ` 连接。请按编辑器显示的样子写标签，不要变形，然后围绕它造句。不在目录里的标签，比如 Godot 或插件的按钮，保持英文。
* 提示框、`td-mcp` 块和它们的 `## MCP` 标题留在原处。提示框标记单独写在一行里，`> [!NOTE]`，构建会把翻译后的标题放在它后面。GDScript 代码块保留它们的 `gdscript` 标签。
* 不用破折号、半角连接号、带空格的连字符或表情符号，英文页面的规则在每种语言里都成立。
* 在中文和日语里，每个段落、列表项和表格单元格都是一个源码行。两个 CJK 字符之间的换行在浏览器里会显示成一个空格，所以 `check` 会标出它。全角标点没有问题，但双倍的破折号字符 U+2014 和 U+2015 算作破折号，不允许使用。

**术语表。** `GLOSSARY.md` 和其他页面一样翻译，每个术语保留它的英文 id，`## Aktivator { #activator }`。会变形的语言把其他形式列在 `data-forms` 里，`{ #baked-lighting data-forms="gebackenen Beleuchtung|gebackener Beleuchtung" }`，这样悬停卡片也能找到它们。属于界面名称的术语使用目录里的词，`check` 会比对它们。翻译后的页面在术语第一次出现的地方使用术语表自己的形式，这就是把它链接到卡片的办法。当你给英文术语表添加术语时，如果可以，也请把它加到翻译后的术语表里，在那之前每种语言的术语表都是英文的。

## CI { #ci }

CI 会跳过只改动文档或 Markdown 的 `main` 推送。其他每一次成功的 `main` 推送，都会用全新的编辑器、插件、演示和内容下载包替换[滚动 beta](https://github.com/Paraxdev/TrenchDot/releases/tag/beta)。

编辑器只从 GitHub 发布页下载插件和内容。itch.io 是一个商店，它在自己的渠道 `addon`、`nature-pack`、`demo-content` 等上得到同样的文件。滚动 beta 是唯一的构建，所以它填充这些普通渠道。

| 发布页里的 zip | 由什么安装 |
| --- | --- |
| `func_godot-trenchdot-addon.zip` | *Godot > 安装或更新插件…* |
| `trenchdot-nature-pack.zip` | *Godot > 向项目添加内容…* |
| `trenchdot-demo-content.zip` | *Godot > 向项目添加内容…* |

每个 zip 里装了哪些文件夹，列在 [content.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/content.rs) 和 [addon_install.rs](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_editor/src/addon_install.rs) 里，并且有测试把 `ci.yml` 里的 zip 命令和它们对照检查。

编辑器通过 GitHub API 找到这些 zip。CI 用 `TRENCHDOT_CHANNEL=beta` 构建它，所以它会先看滚动 beta。本地构建会先尝试以自己版本命名的发布，再回退到 beta。想在 zip 发布之前试用它，可以用 `TRENCHDOT_RELEASES_API` 环境变量把编辑器指向另一个服务器。

CI 还会给每个 beta 打上标记，在编辑器的 `TRENCHDOT_BUILD` 里，也在打包的插件的 `plugin.cfg` 里，格式如 `0.1.0-beta.202611041532.d194b57`：发布编号、UTC 提交时间和提交。编辑器就是靠它分辨插件是不是比较旧的 beta，并提供更新。本地构建则保留普通版本号。

<div class="td-mcp" markdown>

## MCP { #mcp }

想用 AI 代理来贡献代码？[AGENTS.md](https://github.com/Paraxdev/TrenchDot/blob/main/AGENTS.md) 列出了规则，以及拉取请求必须通过的检查。

e2e 测试通过 [MCP 服务器](mcp.md)来驱动编辑器，`example_mcp_scripts_replay` 会重放 `examples/mcp` 里的每个脚本。

想在 shell 里驱动你自己的编辑器：

```sh
trenchdot --mcp-http=7850 --default-prefs &
TRENCHDOT_MCP_URL=http://127.0.0.1:7850/mcp python3 tools/mcp_client.py call get_state
```

把 `TRENCHDOT_MCP_TOKEN` 设为两边相同的值，就可以用[令牌](mcp.md#the-http-token)来试一试。

### 文档里的 MCP { #mcp-in-the-docs }

页眉里的 **MCP** 开关默认关闭，它会隐藏所有关于 MCP 的内容，这样从不碰 MCP 的人读页面时不会被打扰。在页面上，请把所有 MCP 文字放进最末尾的同一个 `## MCP` 部分里，放在一个 `td-mcp` 块中：

```html
<div class="td-mcp" markdown>

## MCP

What only MCP users need.

</div>
```

请保留内容周围的空行，GitHub 只有在有空行时才会读取里面的 Markdown。只讲 MCP 的页面属于 `SUMMARY.md` 的 **MCP** 部分，这个开关同样会把它隐藏。

</div>
