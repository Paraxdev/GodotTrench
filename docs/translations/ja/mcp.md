---
source: d5a662c31724dba93b77bd318ab3a570c43e95b6
---

# MCPサーバー { #mcp-server }

エディターには[MCP](https://modelcontextprotocol.io)サーバーが組み込まれています。Claude CodeのようなエージェントやただのPythonスクリプトから、マップの確認や編集、スクリーンショットの撮影、UIへの入力の送信ができます。

AIアシスタントにレベルを作らせたりレビューさせたり、繰り返しの編集をスクリプトにしたり、エディターを自動でテストしたりするのに使えます。サーバーは起動するまでオフで、待ち受けるのは自分のマシンだけです。

## 起動する { #starting-it }

| トランスポート | 起動のしかた |
| --- | --- |
| HTTP | エディターを`--mcp-http`で起動するとポート7841、`--mcp-http=PORT`なら別のポートになります。毎回自動で起動したいときは、*ファイル > 環境設定*を開き、*詳細設定*で*MCPサーバー（HTTP、localhost）*の隣の*起動時に有効化*にチェックを入れ、その隣でポートを決めて、再起動します |
| stdio | `--mcp`で起動します。クライアントがセッションごとにエディターを自分で起動し、標準入出力で会話します |

エディターを開いたまま自分でも作業し、エージェントに手伝わせたいときはHTTPを選びます。クライアントに自分専用のエディターを持たせたいときや、同じコンピューターを他の人も使うときは、stdioを選びます。どちらの場合もClaude Codeにつなぐには、次のようにします。

```sh
claude mcp add --transport http trenchdot http://127.0.0.1:7841/mcp   # HTTP
claude mcp add trenchdot -- /path/to/trenchdot --mcp                 # stdio
```

リポジトリの`.mcp.json`は、すでにClaude CodeをHTTPサーバーに向けています。*ファイル > 環境設定*には、動作中のサーバー用のコマンドが表示され、*コマンドをコピー*でクリップボードに入ります。

別のポートでエディターを起動したときは、`TRENCHDOT_MCP_URL`を設定すると、[tools/mcp_client.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_client.py)と[tools/mcp_script.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_script.py)がそちらにつながります。

### HTTPトークン { #the-http-token }

HTTPサーバーには、自分のコンピューター上のどのプログラムからも、同じコンピューターにサインインしている他の人からも届きます。それを防ぐには、*ファイル > 環境設定*の*詳細設定*で*トークンを必須にする*にチェックを入れます。次のリクエストから、サーバーはトークンを`Authorization: Bearer <token>`として送ってきたリクエストにだけ応答します。環境設定のコマンドにはトークンが含まれるので、`.mcp.json`の代わりにそれを使ってください。stdioにトークンは要りません。エディターを起動したクライアントだけが話せるからです。

エディターは、最初に求められたときにランダムなトークンを作り、設定の隣にある、自分だけが読めるファイル`mcp-token`に保存します。場所はWindowsなら`%APPDATA%\TrenchDot\data`、Linuxなら`~/.local/share/trenchdot`、macOSなら`~/Library/Application Support/TrenchDot`です。`--default-prefs`で起動したときは、tempフォルダーの`trenchdot-default-prefs.mcp-token`に専用のものを保存します。

1回の実行だけトークンを自分で決めたいときは、`TRENCHDOT_MCP_TOKEN`を設定するか、`--mcp-token=<token>`で起動します。後者は環境変数より優先されます。どちらも、環境設定の内容にかかわらずトークンをオンにし、*トークンを必須にする*にはチェックが入った状態でグレーアウトして表示されます。`tools/mcp_client.py`と`tools/mcp_script.py`は、`--token`か`TRENCHDOT_MCP_TOKEN`で渡したときにだけトークンを送ります。

## MCPをオフにする { #turning-mcp-off }

エージェントやスクリプトをエディターから完全に締め出したいときは、*ファイル > 環境設定*のMCPの部分の一番上にある*MCP*のチェックを外します。HTTPサーバーはすぐに閉じて開いている接続を切り、stdioのクライアントはどのリクエストにもエラーを受け取り、他のMCP設定はグレーアウトします。この設定はすぐに保存されます。そのため、`--default-prefs`なしで起動するエディターはどれも、またチェックを入れるまでMCPがオフのままです。チェックを入れると、HTTPサーバーが元のポートで戻ってきます。

![環境設定のMCPの部分。MCPにチェックがあり、ポート付きのHTTPサーバーの行と、トークンを必須にするがある](assets/mcp/prefs-mcp.png)

環境設定にかかわらず1回の実行だけMCPをオフにするには、`TRENCHDOT_DISABLE_MCP`に、空でも`0`でもない値を設定します。すると*MCP*はチェックなしでグレーアウトして表示されます。MCPがオフの間、`--mcp`と`--mcp-http`はサーバーを起動しません。エディターは開き、ステータス行にMCPがオフだと出て、stdioのクライアントは待たされる代わりに、そのことを伝えるエラーを受け取ります。

エージェントも、`set_editor`と`"mcp": false`でMCPをオフにできます。オンに戻せるのは、あなただけです。

## ツール { #tools }

| 領域 | 用途 | ツール |
| --- | --- | --- |
| 確認 | 何も変えずに、マップ、エディターの状態、エンティティ定義、ゲームコードからのエンティティの使い方を読む | `get_state`, `summarize_map`, `list_nodes`, `get_node`, `changes_since`, `validate_map`, `get_game_config`, `code_reference` |
| 作成 | ブラシ、メッシュ、地形、エンティティ、インポートしたモデルを追加し、面の辺に沿ってトリムする | `create_brush`, `create_mesh`, `create_terrain`, `create_entity`, `import_model`, `trim` |
| 編集 | 今あるものを変える。エンティティのキー、選択、配置、面、テクスチャ、地形、レイヤーとグループ、worldspawn | `update_entity`, `select`, `transform`, `duplicate`, `set_face`, `mesh_edit`, `texture`, `terrain_edit`, `hierarchy`, `set_map_properties`, `bake_lighting` |
| ペイントとゲームプレイ | モデルを散布し、マテリアルをブレンドし、プレハブライブラリからパーツを貼り付け、ブラシをドア、プラットフォーム、ボタンにする | `scatter`, `blend`, `prefab_library`, `random_fill`, `gameplay` |
| エディター | メニューアクションの実行、ファイルを開く、保存する、エディター設定の変更、カメラの移動、スクリーンショット、入力の送信、プレイヤーが歩ける場所の確認 | `run_action`, `map_file`, `open_project`, `set_editor`, `set_camera`, `screenshot`, `simulate_input`, `walkability` |
| プロジェクト | ゲームプレイエンティティパック、自然モデル、デモを、開いているGodotプロジェクトに追加する | `project_content` |
| スクリプト | 保存したツール呼び出しのリストを再生する。[MCPスクリプト](mcp-scripts.md)を参照 | `run_script` |

各ツールの説明に、そのオペレーションと引数が載っているので、エージェントはサーバーから直接それを学べます。[tools/mcp_client.py](https://github.com/Paraxdev/TrenchDot/blob/main/tools/mcp_client.py)は`list`でそれらを表示し、シェルからツールを呼び出せます。

```sh
python tools/mcp_client.py call screenshot '{"target":"3d","width":1280,"height":720}' --out shot.png
```

## スクリーンショット { #screenshots }

`screenshot`は、エディターウィンドウ全体のPNGを返します。`target`が`3d`、`top`、`front`、`side`のときは、そのビュー1つのPNGです。ビューの場合は、ほかの引数で結果が決まります。

| 引数 | 結果 |
| --- | --- |
| なし | エディターウィンドウ全体 |
| `target`だけ | そのドッキングされたビューをそのまま、オーバーレイ込みで |
| `width`と`height` | 最大4096ピクセルまでオフスクリーンで描画。グリッド、エンティティのボックス、ギズモ、選択は含まない |
| `width`、`height`、`"overlays": true` | オフスクリーンで、エディターのオーバーレイ付き |
| `"overlays": false` | ドッキングされたサイズで、何も重ねないクリーンなショット |

### Godotが描画するもの { #what-godot-renders }

`"source": "godot"`を指定すると、Godotエディターに、本物のライト、環境、フォグ、ポストプロセスでマップを描画させます。そのPNGが返り、ビルド中にGodotが記録したエラーや警告もあれば一緒に届きます。ゲームを起動せずに、ライティングやマテリアルを判断するのに便利です。

```sh
python tools/mcp_client.py call screenshot '{"source":"godot","position":[0,256,512],"look_at":[0,64,0]}' --out godot.png
```

1. TrenchDotで開いているプロジェクトの中に、マップを保存します。
2. そのプロジェクトをGodotエディターで開き、`FuncGodotMap`でマップをビルドするシーンを用意します。
3. `"source": "godot"`を付けて`screenshot`を呼びます。`position`と`look_at`を省くと、3Dビューのカメラを使います。

[ライブモード](godot/live.md)がないときは、Godotはまず、TrenchDotで見えているとおりにマップをビルドします。*Godotでビルド*と同じです。`"build": false`にすると、Godotが持っているままのシーンを撮ります。`width`と`height`の既定値は1280 x 720、`fov`は3Dビューのものです。

> [!NOTE]
> Godotは描画にウィンドウを必要とします。ヘッドレスのGodotは、画像の代わりにエラーを返します。ウィンドウは背面にあっても、最小化されていても構いません。

## マップを読む、レビューする { #reading-and-reviewing-a-map }

| ステップ | ツール |
| --- | --- |
| 概要をつかむ | `summarize_map`が、レイヤー、エンティティクラス、I/Oリンク、マテリアル、足りないアセットを一覧にします |
| ノードを探す | `list_nodes`は、`layer`、`material`、`property`(`{"model": "*crate*"}`)、マップ内の`box`で絞り込みます |
| 1つを見る | `get_node`は1つのノードを返します。`compact: true`にすると頂点と面を省きます |
| 変えたところを見せる | `changes_since`は、最後の保存、`undo_steps`回前、またはマップの`file`と比べます |
| 取り消す | 各呼び出しと各`run_script`は、`MCP: ...`という名前の1つの元に戻すステップになります。`run_action undo`に`steps`を付けると、複数まとめて戻せます |

マップをgitで管理する方法は、[変更の確認](editor/reviewing.md)を参照してください。

## 知っておきたい挙動 { #behavior-worth-knowing }

| 項目 | 挙動 |
| --- | --- |
| ブラウザー | HTTPサーバーは、`Origin`ヘッダー付きのリクエストと、`application/json`ではないPOSTをすべて拒否します。そのため、localhostから配信されたものを含め、Webページからは届きません |
| エラー | 仕事を果たせなかったツールは`isError`を返します。失敗した`run_action`も同じです |
| ファイルダイアログ | ダイアログを開くアクションは拒否されます。パスを付けた`map_file`を使ってください。`run_action save`は、すでにファイルがあるマップが必要です |
| 未保存のタブ | `run_action close_tab`は、`args.discard`がtrueでない限り、未保存の変更があると拒否します。`args.close_others`にすると、代わりにアクティブなタブ以外をすべて閉じます |
| ファイルパス | `map_file`、`run_script`、`import_model`、`changes_since`、`validate_map`の`res://`と相対パスは、開いているプロジェクトから始まり、プロジェクトがなければエディターの作業ディレクトリから始まります。`map_file open`と`open_tab`は、開く前にパスを絶対パスにするので、あとの`save`が別の場所に落ちることはありません |
| エディター設定 | `transform`と`run_action rotate`、`flip`は、選択範囲の中心を軸にします。グリッドにスナップするのは、スナップがオンのときだけです。UVロックがオンの間は、テクスチャを固定したままにします。`transform`は`snap`と`uv_lock`でその1回だけ上書きでき、`duplicate`は`uv_lock`を取ります。`run_action sculpt`と`blend`は、人が変えられるツール設定を使い、`terrain_edit`は固定の既定値を使います |
| 選択 | `select`は、未知のIDを拒否し、非表示やロックされたノードを飛ばして、`skipped`に一覧にします |
| CSG | `csg_subtract`、`csg_merge`、`csg_intersect`、`csg_hollow`、`clip_apply`は`replaced`を返します。古いIDを新しいIDに対応づけたものです |
| タイムアウト | 呼び出しは120秒でタイムアウトします。`run_script`、`project_content`、`bake_lighting`は除きます |
| コンテンツの質問 | 新しいプロジェクトで出る、すぐ使えるコンテンツについての質問は、`get_state`、`screenshot`、`simulate_input`以外の呼び出しが最初に来た時点で閉じ、そのセッションでは二度と開きません |
| スクリプトの変数 | `$name/rest`(波括弧なし)は、`name`が既知の変数だとエラーになります。展開されるのは、`$name`だけか、`${name}/rest`のときだけだからです。`${project}/maps`と書いてください |

## ツールのメモ { #tool-notes }

| ツール | メモ |
| --- | --- |
| `run_action` `copy`、`cut`、`paste` | コピーとカットはクリップボードのテキストを返します。ペーストはそれを`args.text`で受け取り、`offset`か`origin`も付けられます |
| `run_action` `clip_apply` | 平面(`point`と`normal`、または3つの`points`)と`keep`を取ります。`keep`は`front`、`back`、`both`です |
| `run_action` `create_decal` | [デカール](editor/decals.md)を置きます。薄いブラシの代わりにこれを使ってください |
| `create_brush` | `openings`でドアや窓をくり抜きます。`{min, max, count, step, rows, row_step}`で1つを格子状に繰り返します |
| `create_mesh` `rotate` | `[x, y, z]`の度数で、GodotのYXZ順です。`[0, 0, 90]`だと円柱がX方向に寝ます |
| `transform` | translate、rotate、flip、scale_toをこの順に適用し、1つの元に戻すステップにします。`rotate`は、指定しない限りY軸まわりに90度回します |
| `create_terrain` | 他のツールと違い、`origin`は地形の最小の角です。`center`なら、代わりにある点のまわりに置きます |
| `create_brush`、`create_mesh` | 切妻の棟は、ブラシではZ方向、メッシュではX方向に走ります。どちらのツールも`ridge_x`と`ridge_z`を取ります |
| `map_file`のエクスポート | `path`は書き出したファイルで、古い呼び出し元のために残してあります。`exported`も同じものを指し、`map_path`は開いているマップです |
| `terrain_edit` | `op`なしの`probe [x, z]`は、高さを読むだけです。`clear_layer`には`layer`が必要です |
| `validate_map` | ターゲットのGodotクラスやスクリプトの任意のメソッドやプロパティを、入力として受け付けます。`coplanar_faces`は、ちらつく面を指摘します。どの問題にも`bounds`があります。`path`は、保存済みのマップを開かずに検査し、`project: true`はプロジェクト内のすべてのマップを検査します |
| `run_action` `place_entities` | `args.classnames`を取り、`at`と`normal`で、エンティティタブからのドラッグと同じように、面の上に載せます。クラス名なしはエラーです |
| `simulate_input`のドラッグ | `"release": false`のドラッグは、呼び出しが戻ったあともボタンと修飾キーを押したままにします。そのため、スクリーンショットにドラッグ中の様子、たとえばカーソルの横のカードが写ります。あとの呼び出しの`release`イベントが、ポインターのある場所、または`x`と`y`、または`world`でドロップします。`x`と`y`の片方だけの`release`はエラーです。ドラッグと修飾キーは、実際のキーボードでも、そのreleaseか、`new_map`か`close_tab`アクションの成功か、MCPがオフになるまで押されたままです。releaseを忘れないでください。それまでは、クリックとドラッグは拒否されます |
| `run_action` `reload_materials` | テクスチャフォルダーを今すぐ再スキャンします。ただし、新しいファイルは放っておいても見つかります |
| `set_editor` `live_link_port` | プロジェクトが別の`trenchdot/live_link_port`を設定しているGodotエディターと通信します |
| `scatter` `preset` | プリセットに必要な、埋め込みのBlockbenchモデルを追加します。自然パックのないプロジェクトでglTFプリセットを使うとエラーになり、`project_content`が示されます |
| `bake_lighting` | [ライティングのベイク](editor/light-baking.md)と同じようにベイクして、終わるまで待ちます。省いたオプションは、マップが最後にベイクされたときのままです。`bake: false`は`view`を切り替えるだけで、`remove: true`はベイクを捨てます。`out_of_date`は、そのあとマップが変わったかを示します |
| `project_content` | `install: "nature"`か`"demo"`は、リリースからダウンロードして、ファイルが入った時点で応答します。`install: "gameplay"`は、プロジェクトのアドオンからゲームプレイエンティティパックをコピーします。スクリプトの中でも使えます。引数なしだと、プロジェクトにあるものを答えます |
