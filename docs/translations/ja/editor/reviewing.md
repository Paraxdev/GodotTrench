---
source: 77812fdfe3757a8be2d8e1be3d7fbd73a1ff678a
---

# 変更をレビューする { #reviewing-changes }

マップをgitで管理すれば、コードをレビューするのと同じように、2つのバージョンのあいだで何が変わったかを確認できます。現在のセッション内の変更なら、[履歴パネル](organizing.md#history)がすべての編集を一覧にしていて、どの編集にでも元に戻せます。

## gitでのマップ { #maps-in-git }

`.tdm`ファイルはバイナリですが、gitでも読みやすい差分を表示できます。gitがエディターに各バージョンをJSONとして出力させて、それを比較します。設定は一度だけです。

1. プロジェクトの`.gitattributes`に次の行を追加します。

    ```
    *.tdm binary diff=tdm
    ```

2. マップの出力方法をgitに教えます。クローンごとに一度だけ必要です。

    ```sh
    git config diff.tdm.textconv "trenchdot --dump"
    ```

3. (任意)バックアップと自動保存をリポジトリに入れないようにするには、`.gitignore`に次の行を追加します。

    ```
    *.tdm.bak
    *.tdm.autosave
    ```

これで`git diff`と`git log -p`が、バイナリファイルが違うとだけ表示する代わりに、マップの変更をJSONで表示するはずです。

> [!WARNING]
> `trenchdot`が`PATH`に入っている必要があります。入っていないと、マップで`git diff`が失敗します。

そのJSONをファイルとして取り出すには、`trenchdot --to-json map.tdm map.json`で書き出せます。`trenchdot --to-tdm map.json map.tdm`でバイナリのマップに戻せます。[コマンドライン](../command-line.md)を参照してください。

<div class="td-mcp" markdown>

## MCP { #mcp }

MCPのレビューツールを使うと、AIエージェントがマップに何をしたかを確認できます。

### AIエージェントによる編集 { #edits-made-by-an-ai-agent }

エージェントが[MCP](../mcp.md)でマップを編集すると、ツール呼び出し1回ごとに、[履歴パネル](organizing.md#history)に`MCP: ...`という名前のアンドゥ1ステップができます。`run_script`全体も1ステップで、たとえば`MCP: Add crates, 6 steps`のようになります。そのステップを元に戻すと、エージェントの作業が取り消されます。

![先頭にMCP: Add crates, 6 stepsがあり、その下に名前変更、グループ化、中空化など手作業の編集が並んでいる履歴パネル](../assets/reviewing/history-mcp-script.png)

| 確認したいこと | エージェントに頼むこと |
| --- | --- |
| 最後に保存してから何が変わったか | `changes_since`。追加、削除、移動、その他の編集をされたノードを一覧にします |
| 直前の呼び出しやスクリプトが何を変えたか | `undo_steps: 1`を付けた`changes_since` |
| gitのコミット以降に何が変わったか | `file`にそのバージョンを指定した`changes_since`。バージョンは`git show HEAD:maps/level.tdm > old.tdm`で書き出します |

> [!TIP]
> 複数ステップの作業は、`label`を付けた1回の`run_script`にまとめるようエージェントに頼むと、作業全体が、見分けやすい名前の1ステップで元に戻せます。

</div>
