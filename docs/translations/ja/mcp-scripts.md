---
source: ac40f8ef73c25c546d11381f2a654f03d847b7aa
---

# MCPスクリプト { #mcp-scripts }

MCPスクリプトは、[MCP](mcp.md)のツール呼び出しのリストを持つJSONファイルで、`run_script`がそれを順に再生します。マップがどう作られたかを読める手順として残せるので、マップをゼロから作り直せますし、プルリクエストでテキストとしてレビューできますし、テストでも再生できます。

[examples/mcp](https://github.com/Paraxdev/TrenchDot/tree/main/examples/mcp)のショーケースマップはこの方法で作られていて、すべてのツールの使用例も兼ねています。

| スクリプト | 作るもの |
| --- | --- |
| `mountain_house.json` | 崖にせり出した山小屋。崖のリフトと森つき |
| `church_school.json` | 教会と学校。3つのボタンで中庭の門が開きます |
| `lighthouse_forest.json` | 回転するビーコンのある灯台、`func_train`に乗った船、湧き出るウィスプ |
| `withered_city.json` | 夜の廃墟となったネオンの街。稲妻、グロー、`ssr`つき |
| `night_district.json` | 夜の工業地帯の通り。ローラードアと高架を走る列車つき |
| `sea_island.json` | [海の島のチュートリアル](tutorials/sea-island.md) |
| `scripted_scene.json` | [カットシーンのチュートリアル](tutorials/cutscene.md) |
| `logic_playground.json` | [数学ノードとロジックノード](gameplay/entities/logic.md)のためのホール。コイン、スコア、金庫のドアのゲート、ライトの抽選 |

## 実行する { #running-them }

デモプロジェクトとそのHTTPサーバーつきでエディターを起動し([起動する](mcp.md#starting-it)を参照)、`tools/mcp_script.py`でスクリプトを再生します。エディターが[トークン](mcp.md#the-http-token)を求めるときは、`--token`か`TRENCHDOT_MCP_TOKEN`で渡してください。`shot`コマンドは、指定したカメラ位置からスクリーンショットを保存し、`--shade`でシェーディングを選びます。

```sh
trenchdot --mcp-http --project godot                        # the editor with the demo project
python tools/mcp_script.py run examples/mcp/sea_island.json   # replay a script
python tools/mcp_script.py shot view.png --pos 4300,900,3300 --look 2200,700,1200 --shade lit
```

スクリプトは`godot/demo/maps/showcase`に保存しますが、`scripted_scene.json`と`logic_playground.json`だけは`godot/demo/maps`に保存します。

実行は、`--continue-on-error`を付けない限り、最初に失敗したステップで止まります。`--url`を使うと、別のポートのエディターをランナーに指定できます。1回の実行は、変更したすべてのマップで1つのアンドゥ単位になり、名前にはスクリプトの`label`か、ファイル名が使われます。

## スクリプトの形式 { #script-format }

```json
{
  "format": "trenchdot-mcp-script",
  "label": "Hollow room",
  "vars": { "save_dir": "${project}/maps" },
  "steps": [
    { "note": "A hollow room." },
    { "tool": "map_file", "args": { "op": "new" } },
    { "tool": "create_brush", "args": { "min": [0, 0, 0], "max": [256, 128, 256] }, "save": "room" },
    { "tool": "select", "args": { "ids": "$room.ids" } },
    { "tool": "run_action", "args": { "action": "csg_hollow" } },
    { "tool": "map_file", "args": { "op": "save", "path": "${save_dir}/room.tdm" } }
  ]
}
```

`vars`は、自分用の変数のデフォルトを設定します。`run_script`に渡した`vars`オブジェクトはそれを上書きするので、1つのスクリプトでも保存先を変えられます。この方法で渡したフォルダーは、スクリプトが読み込みも書き込みもできるフォルダーになります。

`tool`のないステップはコメントです。`save`のあるステップは、その結果をその名前で保存するので、後のステップが、作られたidを使えます。

スクリプトはマップと同じように共有されるので、そのステップができるのは、マップの編集と保存だけです。ステップから、別のプロジェクトやそのマップを開く、Godotを起動する、その中でビルドする、ライブモードをオンにする、MCPをオフにする、クリップボードにコピーやカットをする、未保存の変更を捨てる、ほかのタブを閉じる、ということはできません。アンドゥとリドゥが届くのはスクリプト自身が行ったことだけなので、保存していない作業は、すべてのタブにそのまま残ります。`screenshot`、`simulate_input`、`walkability`、`run_script`もスクリプトの中では実行できず、`gameplay`以外の`project_content`のインストールも同じです。

ステップが読み書きするファイルは、すべて、開いているプロジェクト、スクリプト自身のフォルダー、`vars`で渡したフォルダーのどれかの中になければなりません。その中にあるリンクが別の場所を指している場合も同じです。対象は、開く、チェックする、比較する、保存する、エクスポートするマップ、インポートするマップ、モデル、プレハブ、テクスチャとそれらが指すファイル、そして変換、インポート、エクスポート、プレハブライブラリが書き出すものすべてです。マップやモデルがほかの場所の名前で指しているプレハブ、モデルファイル、テクスチャ、Hammerのインスタンスは除外され、ネットワークパスはエディターが見る前に拒否され、パスなしの保存は、スクリプトの開始時に開いていたマップにだけ書き戻します。

このルールは、マテリアルファイルとその画像、ホットスポットファイル、プレハブライブラリ、自然プリセットにも当てはまります。ファイルがそれらのフォルダーの外にあるテクスチャは、スクリプトの中では存在しないものとして読まれるので、フィットや整列をすると、デフォルトのテクスチャサイズにフォールバックします。そのとき、サマリーの`unread_textures`に、読めなかったファイルが一覧されます。

拒否されたステップは、サマリーの`errors`に理由つきで現れます。サマリーは、ツール呼び出しを`steps`、コメントを`notes`として数えます。

`args`の文字列は、保存した結果と、いくつかの組み込み変数を参照できます。

| 書き方 | 意味 |
| --- | --- |
| `"$name.path"` | 保存したJSONの値。数値は配列のインデックスで、`$room.ids.0`のように使います |
| `"${name.path}"` | 長い文字列の中での、そのテキスト |
| `"$a.ids + $b.ids"` | 両方を1つの配列に連結したもの。`["$a.ids", "$b.ids"]`でも同じです |
| `$last` | 直前のステップの結果 |
| `$project` | 開いているGodotプロジェクトのフォルダー。何も開いていなければエラーです |
| `$script_dir` | スクリプトファイルのフォルダー |
| `$$` | リテラルの`$`。ノードパスなら`"$$Player"`になります |

未知の名前はエラーです。`"$5"`のように名前を始められない`$`は、そのまま残ります。`"42"`のようにテキストで渡したidは、ノードidが期待される場所ならどこでも使えます。

波かっこなしの`$name/rest`は展開されないので、波かっこのない`$project/maps`は、誰も意図していないパスになるのではなく、エラーになります。`"${project}/maps"`を使ってください。

## CSGの後のid { #ids-after-csg }

CSGとクリップのステップは、切ったブラシを、新しいidを持つ新しいブラシに置き換えます。ランナーは、前のステップで保存したidをそれに合わせて書き換えるので、壁に窓を切った後でも`"$wall.ids"`は壁全体を指したままで、取り除かれたブラシは消えます。

書き換えられるのは、ノードidを持つキーの下の値だけです。`id`、`ids`、`selection`、`selected`、`nodes`、`letters`、`copies`、`entity`、`group`、`parent`、`insert_parent`、`current_layer`、`open_groups`、`targets`、そして`_id`や`_ids`で終わるキーです。`hierarchy reparent`の`moved`のような個数はそのままなので、代わりにその`ids`を使ってください。

> [!TIP]
> `scatter`と`blend`の呼び出しでは、ブラシの設定をすべて書き出してください。省いたものは、エディターの現在の散布パネルや、ブレンドツールの設定から取られます。
