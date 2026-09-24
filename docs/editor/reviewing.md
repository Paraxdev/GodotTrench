# Reviewing changes

Keep maps in git to see what changed between versions, the same way you review changes to code. For changes inside the
current session, the [History panel](organizing.md#history) lists every edit and undoes back to any of them.

## Maps in git

`.gtm` files are binary, but git can show readable diffs of them by printing each version as JSON with the editor.

1. Add this line to `.gitattributes` in your project:

   ```
   *.gtm binary diff=gtm
   ```

2. Tell git once per clone how to print a map:

   ```sh
   git config diff.gtm.textconv "godottrench --dump"
   ```

3. Keep backups and autosaves out of the repository with these lines in `.gitignore`:

   ```
   *.gtm.bak
   *.gtm.autosave
   ```

`git diff` and `git log -p` now show map changes as JSON. `godottrench --to-json map.gtm map.json` writes that JSON to
a file, and `godottrench --to-gtm map.json map.gtm` turns it back into a binary map.

> **Warning:** `godottrench` has to be on your `PATH`, or `git diff` fails on maps.

{% mcp %}

## MCP

The MCP review tools show what an AI agent did to your map.

### Edits made by an AI agent

When an agent edits your map over [MCP](../mcp.md), every tool call is one undo step named `MCP: ...` in the
[History panel](organizing.md#history), and a whole `run_script` is one step too, such as `MCP: Add crates, 40 steps`.
Undo that step to take the agent's work back.

| To see | Ask the agent for |
| --- | --- |
| What changed since you last saved | `changes_since`, which lists nodes added, removed, moved or otherwise edited |
| What its last call or script changed | `changes_since` with `undo_steps: 1` |
| What changed since a git commit | `changes_since` with `file` set to that version, written with `git show HEAD:maps/level.gtm > old.gtm` |

> **Tip:** Ask the agent to put a multi-step job in one `run_script` with a `label`, so the whole job undoes in one
> step under a name you recognise.

{% endmcp %}
