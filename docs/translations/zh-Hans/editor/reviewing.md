---
source: 77812fdfe3757a8be2d8e1be3d7fbd73a1ff678a
---

# 审阅更改 { #reviewing-changes }

把你的地图放在 git 里，你就能看到两个版本之间改了什么，和审阅代码的方式一样。对于当前会话内的更改，[历史面板](organizing.md#history)会列出每一次编辑，并可以撤销回其中任何一步。

## git 里的地图 { #maps-in-git }

`.tdm` 文件是二进制的，但 git 仍然能给你看得懂的差异。它会让编辑器把每个版本打印成 JSON，再比较这些 JSON。你只需要设置一次：

1. 在项目的 `.gitattributes` 里加上这一行：

    ```
    *.tdm binary diff=tdm
    ```

2. 告诉 git 怎么打印地图，每个克隆做一次：

    ```sh
    git config diff.tdm.textconv "trenchdot --dump"
    ```

3. （可选）在 `.gitignore` 里加上这几行，让备份和自动保存不进仓库：

    ```
    *.tdm.bak
    *.tdm.autosave
    ```

现在 `git diff` 和 `git log -p` 应该会把地图的更改显示成 JSON，而不是只说二进制文件不同。

> [!WARNING]
> `trenchdot` 必须在你的 `PATH` 里，否则 `git diff` 遇到地图会失败。

想把 JSON 拿到一个文件里，用 `trenchdot --to-json map.tdm map.json` 写出来，再用 `trenchdot --to-tdm map.json map.tdm` 转回二进制地图。参见[命令行](../command-line.md)。

<div class="td-mcp" markdown>

## MCP { #mcp }

MCP 审阅工具会让你看到 AI 代理对你的地图做了什么。

### AI 代理所做的编辑 { #edits-made-by-an-ai-agent }

当代理通过 [MCP](../mcp.md) 编辑你的地图时，每次工具调用都是[历史面板](organizing.md#history)里一个名为 `MCP: ...` 的撤销步骤。整个 `run_script` 也是一步，比如 `MCP: Add crates, 6 steps`。撤销那一步，就能收回代理的工作。

![历史面板顶部是 MCP: Add crates, 6 steps，下面是手动做的编辑，比如重命名、编组和挖空](../assets/reviewing/history-mcp-script.png)

| 想看 | 让代理用 |
| --- | --- |
| 上次保存以来改了什么 | `changes_since`，它会列出新增、删除、移动或以其他方式编辑过的节点 |
| 它上一次调用或脚本改了什么 | `changes_since` 并带上 `undo_steps: 1` |
| 自某个 git 提交以来改了什么 | `changes_since`，把 `file` 设为那个版本，用 `git show HEAD:maps/level.tdm > old.tdm` 得到 |

> [!TIP]
> 让代理把多步任务放进一个带 `label` 的 `run_script`，这样整个任务可以一步撤销，名字也是你认得的。

</div>
