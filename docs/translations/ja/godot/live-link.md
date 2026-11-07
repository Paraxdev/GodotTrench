---
source: c998bca32c1c4bd85f0c95739d78a13d4ede986a
---

# Live Linkプロトコル { #live-link-protocol }

Live Linkは、TrenchDotとGodotエディターが話すための通信路です。アドオンがGodotエディターの中で小さなサーバーを動かし、TrenchDotがそこへ接続して、保存時の再ビルド、[ライブモード](live.md)の編集のストリーミング、Godotが動いているかどうかの表示を行います。

このページが必要になることは、あまりありません。アドオンを有効にすれば、リンクはそのまま働きます。ここにあるのは、同じやり方でGodotを動かす自作のツールを書くときや、接続が失敗する理由を調べたいときのためです。保存しても再ビルドされないときは、まず[保存時に再ビルドする](live.md#rebuild-on-save)を見てください。たいていはステータスバーが理由を教えてくれます。

## トランスポート { #transport }

`127.0.0.1:7842`のTCPで、改行区切りのJSONをやり取りします。メッセージには`event`と、任意の`seq`があり、返信はちょうど1行で、`seq`をそのまま返し、`ok`を持ちます。失敗したときは、`ok: false`と`error`を返します。エディターは接続を1本開いたままにして、1秒ごとに`status`のハートビートを送ります。

別のポートを使うには、両側で変えてください。Godotのプロジェクト設定の`trenchdot/live_link_port`と、TrenchDotの*ファイル > 環境設定*にある*Godotライブリンク*です。

実行中のゲームにある[ホットリロード](live.md#hot-reload-in-a-running-game)のオートロードは、`127.0.0.1:7843`で待ち受け、`map_saved`にしか答えません。

## イベント { #events }

クライアントは次のイベントを送ります。返信の列には、`ok`と`seq`のほかにGodotが返すフィールド、または実行する動作を書いています。

| イベント | フィールド | 返信 |
| --- | --- | --- |
| `hello` | | `project`と`godot`(バージョン)。リンクが応答するかを手早く確かめる |
| `status` | | `project`、`godot`(バージョン)、`pid`、`scene`(Godotが表示しているシーンタブ)、そのシーン用の`maps: [{path, epoch}]` |
| `map_saved` | `path` | `rebuilt`(ファイルから再ビルドしたマップノードの数)、`scene`、`waiting`、そしてビルドの`missing_classes`、`settings`、`warnings` |
| `build` | `path`、`text` | `map_saved`と同じ。ただしファイルではなく`text`からビルドする |
| `focus` | | Godotのウィンドウを前面に出す |
| `export_game_config` | | `trenchdot_game.json`を書き出す |
| `scan` | | プロジェクトのファイルのスキャンを始める。終わると、すべてのライブセッションがエンティティ定義を確認する |
| `play` | | 開いているシーンを保存して、メインシーンを再生する。プロジェクトにメインシーンがなければ、`ok: false`と`error` |
| `live_begin` | `path`、`text`、`content` | `epoch`と`unresolved` |
| `live_resync` | `path`、`text` | `epoch`と`unresolved` |
| `live_delta` | `path`、`ops` | `epoch`と`unresolved`。opsが合わないときは`ok: false, resync: true` |
| `live_end` | `path`、`revert` | `revert`があれば、保存済みのファイルから再ビルドする |
| `inspect` | `path`、`ids` | マップのノードidごとの名前、クラス、子の数、位置を持つ`nodes`と、`live`、`pending` |
| `capture` | `path`、`camera`、`width`、`height`、`text` | `png`(base64)、`width`、`height`、`warnings`(ビルド中に記録されたエラーと警告) |
| `walkable` | `path`、`agent`、`text` | `vertices`(x、y、zを平らに並べたリスト)、`polygons`、`islands`(ポリゴンのインデックス。大きい順)、`units_per_meter`、`agent`、`msec` |

`capture`は、`path`を使っている最初のマップノードのワールドを描画します。`camera`は、マップの単位と軸での`position`と`forward`、そして水平方向の視野角を度で表す`fov`を持ちます。`text`があれば、Godotはまずそのマップをビルドし、なければライブセッションが追いつくのを待ちます。返信は約12フレーム後で、そのあいだほかのメッセージには応答します。

`walkable`は、編集中のシーン全体のナビゲーションメッシュを、`agent`(`radius`、`height`、`max_climb`、`max_slope`。メートルと度)について`TrenchDotNav`でベイクして、マップの単位と軸で返します。`text`があれば、最後に同じテキストからビルドしたのでないかぎり、Godotはまずそのマップをビルドします。

パスは絶対パスで、区切りにはスラッシュを使います。対象になるのは、編集中のシーン(Godotが表示しているシーンタブ)にある`FuncGodotMap`ノードのうち、*Auto Rebuild On Save*がオンのものだけです。`scene`はそのシーンの`res://`パスで、シーンが開いていなければ空です。

`waiting`は、そのマップを使っているほかの開いているシーンタブの一覧です。`map_saved`のあとは、タブが表示されたときにファイルをビルドしますが、`build`のあとはビルドしません。プレハブへの`map_saved`は、それを直接、または別のプレハブ経由でインスタンス化している、編集中のシーンのマップも再ビルドし、`rebuilt`に数えます。

`map_saved`や`build`のビルドが報告した内容は3つのリストで届き、それぞれ空でないときだけ付きます。`missing_classes`はどの定義にも説明されていなかったクラス名をソートしたもの、`settings`はビルドが使ったマップ設定のパス、`warnings`はビルドレポートの1行の警告で、最大20件です。ライブの返信の`unresolved`は、そのステップが定義なしでビルドしたクラス名をソートしたもので、こちらもあるときだけ付きます。

`scan`は、インストールしたエンティティパックのように、Godotの外で書かれたファイルに向いています。シーンのビルドに使ったエンティティ定義が、スキャンのあとでマップ設定にあるものと違うライブセッションは、マップをもう一度ビルドします。`scan`や`play`のないアドオンは、それらを不明なイベントとして返します。

`text`はマップ全体のJSONで、マップにベイクがあれば`lightmap`の下にベイクしたライティングが入るので、ライブのビルドでもそれが保たれます。アトラスのテクセルあたり約11バイトが加わり、1024×1024のアトラスなら約11MBになります。そのため、エディターがマップ全体を送り直すのは、セッションの開始時、新しいベイクのあと、大きな変更のあとだけです。

`content`は、保存したファイルのENDチャンクにあるコンテンツidです。これがあるので、変わっていないファイルからビルドしたシーンは、セッションが始まってももう一度ビルドされません。

ポートが使用中のとき(たいていは同じプロジェクトを開いた2つ目のGodotエディターです)、アドオンはOutputパネルに一度だけ警告を出して、3秒ごとにポートを試し直します。もう一方のエディターが閉じると、引き継ぐためです。

## ライブ操作 { #live-ops }

| 操作 | フィールド | 意味 |
| --- | --- | --- |
| `set` | `id`、`parent`、`index`、`node` | ノードを挿入するか置き換える。`.tdm`形式のJSONの形で、子は含まない |
| `remove` | `id` | ノードをサブツリーごと取り除く。送られるのは、いちばん上で取り除かれたidだけ |
| `translate` | `ids`、`offset` | ノードを動かす。ドラッグ中に、ジオメトリの代わりに送られる |
| `properties` | `properties` | worldspawnのプロパティを置き換える |

マップの`epoch`は、ライブセッションの外でビルドされたとき、またはそれを使う`FuncGodotMap`ノードが変わったときに上がります。次の`status`で新しいepochが見えたら、`live_begin`からやり直してください。

生成されたノードには、マップのノードidが`_td_id`メタに入り、レイヤーとグループのノードには`_td_group`に入ります。

## ウィンドウなしでテストする { #testing-without-a-window }

リンクは、ヘッドレスのGodotエディターでも動くので、自動テストに向いています。`FuncGodotMap`のあるシーンを開いて、ポート7842に`inspect`を送ると、Godotが何をビルドしたかが分かります。

```sh
godot --headless --editor --path godot res://path/scene.tscn
```

(補足：新しくチェックアウトした直後は、先に`godot --headless --path godot --import`を2回実行してください。1回目では、アドオンのクラスが登録されるだけです。)

ヘッドレスのGodotは何も描画しないので、`capture`はそこではエラーを返します。キャプチャのテストは、ウィンドウつきのエディターで行ってください。

アドオンのテストは、これを[`godot/tests/editor/live_link_tabs.gd`](https://github.com/Paraxdev/TrenchDot/blob/main/godot/tests/editor/live_link_tabs.gd)で行っています。ヘッドレスのエディターでシーンタブを開く、閉じる、再読み込みする、切り替えるを行い、リンクが報告する内容を確かめます。

<div class="td-mcp" markdown>

## MCP { #mcp }

[MCPサーバー](../mcp.md)を使うと、エージェントやスクリプトでエディターを操作できます。そのツールのいくつかは、Live Link経由で動きます。`screenshot`ツールは、Godotからの画像を求められると`capture`を使います。

マップも編集するヘッドレスのテストでは、ヘッドレスのGodotエディターと同じプロジェクトを指定してTrenchDotをMCPサーバー付きで起動し、MCP経由でマップを編集します。

```sh
trenchdot --mcp-http --project godot
```

</div>
