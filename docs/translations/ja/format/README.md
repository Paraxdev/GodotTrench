---
source: d3f0019b345d8a4b85cb01b22ccb9fd1cdee21d4
---

# .tdmマップ形式 { #the-tdm-map-format }

このページ群は、インポーターやコンバーター、CIチェックのように、`.tdm`ファイルを読み書きするツールを作る方向けです。レベルを作るだけなら、ここは読まなくて大丈夫です。

`.tdm`ファイルには3つのものが入っています。worldspawnのプロパティ、少しのエディターの状態、そして実際の内容を持つノードのツリーです。

エディターはzstdで圧縮したチャンクのバイナリコンテナとして保存するので、ファイルは小さく収まり、壊れたファイルでも開けます。古いエディターは代わりにUTF-8のJSONドキュメントを1つ保存していて、どのリーダーも両方を読み込めます。どちらの形式も同じツリーを持つので、これらのページでは`trenchdot --dump`が出力するJSONとして示します。

| ページ | こんなときに読む |
| --- | --- |
| [コンテナの構造](container.md) | バイナリファイルを読み書きするとき。ファイルヘッダー、チャンク、各値のバイト列へのエンコードを扱います |
| [壊れたファイル](recovery.md) | 壊れたファイルから何が読み込めるか、各リーダーが失われたものをどこで報告するかを知りたいとき |
| [ノード](nodes.md) | ツリーをたどるとき。全ノードに共通する形、idの仕組み、レイヤー、グループ、エンティティ、インスタンスのノードを説明します |
| [ジオメトリ](geometry.md) | ブラシやメッシュを読んだり作ったりするとき。面、テクスチャ投影、ディスプレイスメントも含みます |
| [地形と散布](terrain-scatter.md) | ハイトマップの地形と、散布したモデルの集まりを扱うとき |
| [JSONの構造](json.md) | 読みやすい形式を扱うとき、2つの形式を相互に変換するとき、エディターがクリップボードに入れるものを読むとき |

動くコードを見たいですか。リファレンス実装は[`format.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/format.rs)で、コンテナは[`binary.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/binary.rs)と[`variant.rs`](https://github.com/Paraxdev/TrenchDot/blob/main/crates/td_doc/src/variant.rs)にあります。Godot側では、[`tdm_file.gd`](https://github.com/Paraxdev/trenchdot_func/blob/main/src/trenchdot/tdm_file.gd)がファイルを読み、[`tdm_parser.gd`](https://github.com/Paraxdev/trenchdot_func/blob/main/src/trenchdot/tdm_parser.gd)がそれを組み立てます。

## トップレベル { #top-level }

| キー | 型 | 書き込まれる条件 | 意味 |
| --- | --- | --- | --- |
| `format` | string | 常に | 常に`"trenchdot-map"`です。マップをほかのJSONと区別するための値です |
| `version` | integer | 常に | マップのバージョンで、現在は1です |
| `properties` | stringからstringへのobject | 空でないとき | worldspawnのキーを名前順に並べたものです |
| `editor` | object | すべてがデフォルトではないとき | エディターのビューの状態です。Godotにはエクスポートされません |
| `textures` | object | テクスチャに設定があるとき | マテリアル名ごとの[テクスチャ設定](geometry.md#texture-settings)です |
| `layers` | ノードの配列 | 常に | マップの[レイヤーノード](nodes.md#layer)を順に並べたものです。ほかのすべてはこの下にぶら下がります |
| `lightmap` | object | ベイクしたJSONマップ | ベイクしたライティングです。バイナリマップでは[LMAPチャンク](container.md#lmap)に保存されます |

`layers`から読み込まれるのは`layer`ノードだけで、1つもないマップは読み込み時に`Default`レイヤーが付きます。

worldspawnの値は`.map`ファイルと同じく、常に文字列です。Godotがマップを組み立てるとき、`TrenchDotEnvironment`がこれらのキーから空、フォグ、太陽を作ります。

| キー | 制御するもの |
| --- | --- |
| `sun_angles`, `sun_color`, `sun_energy` | 平行光源の太陽。向き、色、明るさです |
| `ambient_color`, `ambient_energy` | 太陽が届かない面に届く光です。影が真っ黒にならないようにします |
| `sky_top_color`, `sky_horizon_color`, `sky_ground_color`, `sky_energy` | プロシージャルな空のグラデーションと、その明るさです |
| `sky_panorama` | Godotでプロシージャルな空の代わりになる`res://`の画像です。エディターのプレビューとそのベイクでは、引き続き空の色を使います |
| `fog_color`, `fog_density` | 距離フォグです |
| `glow_intensity`, `ssr` | 明るい面のブルームと、`ssr`が`1`のときのスクリーンスペースリフレクションです |

ゲームが独自の`WorldEnvironment`を持っているなら、`environment`を`sun_only`にすると、太陽だけが作られます。`0`か`none`にすると何も作られません。[環境と太陽](../godot/building.md#environment-and-sun)を参照してください。`sky_source`は、インポートしたマップの空の面がどのテクスチャを使っていたかを覚えておくためのキーです。

`editor`オブジェクトは、エディターの中でしか意味を持たないビューの状態です。

| キー | 型 | デフォルト | 意味 |
| --- | --- | --- | --- |
| `cameras` | object、キーは`"1"`から`"9"` | 空 | 保存したカメラ位置で、それぞれ`{"position": [x, y, z], "yaw": r, "pitch": r}`です。角度はラジアンです |
| `cordon` | `{"min": [x, y, z], "max": [x, y, z]}` | なし | マップ単位のコードンのボックスで、大きなマップの一部だけを作業するときに使います |
| `cordon_enabled` | bool | `false` | コードンの外にあるオブジェクトを隠し、コードン付きのエクスポートから外します |
| `bake` | `{"quality", "texel_size", "softness", "backend"}` | なし | マップを最後に[ベイク](../editor/light-baking.md)したときのオプションで、次のベイクはこれを出発点にします |
| `graph_frames` | array | 空 | [ロジックグラフ](../gameplay/logic-graph.md#frames)のフレームで、それぞれ`{"title", "rect": [x, y, width, height], "color"}`です。単位はグラフのポイントです。壊れたものは読み込み時に捨てられます |

## バージョンと互換性 { #versions-and-compatibility }

`format`と`version`はデータについて記述します。ファイルヘッダーにあるコンテナのバージョンは別物で、チャンクのレイアウトが変わったときだけ変わります。

完全には理解できないファイルに対して、各リーダーがどうするかは次のとおりです。

| 状況 | エディター | Godotアドオン |
| --- | --- | --- |
| `format`が違う | ファイルを拒否する | ビルドエラー |
| `version`が新しい | ファイルを拒否する | アドオンの更新を求めるビルドエラー |
| `version`がない | 最も古いバージョンとして読む | 同じ |
| 未知のキー | 無視し、次の保存で消える | 無視する |
| 未知のチャンク | 保持し、保存時にそのまま書き戻す | スキップする |
| 範囲外の面インデックス | ノードを名指ししてマップを拒否する | そのブラシをスキップする |
| サイズの合わない、ディスプレイスメントの`heights` | ノードを名指ししてマップを拒否する | その面を平らに作る |

エディターは保存時に未知のキーを捨てます。古いエディターに失わせたくないデータを追加するなら、マップのバージョンを上げる(古いエディターは開けなくなります)か、独自のチャンクを使う(古いエディターはそのまま保持します)必要があります。[ほかのチャンク](container.md#other-chunks)を参照してください。

## 座標と単位 { #coordinates-and-units }

位置はGodotの軸を使います。Yが上、Xが右、Zが手前です。ビルド時にマップ設定の`inverse_scale_factor`(デフォルトは32)で割るので、マップ単位32が1メートルになります。

| 量 | 単位 |
| --- | --- |
| 位置、サイズ、高さ、間隔、セルサイズ | マップ単位 |
| エンティティ、インスタンス、散布の角度 | 度。Xまわりがpitch、Yまわりがヨー、Zまわりがrollで、Godotの`rotation_degrees`と同じYXZ順に適用します |
| カメラブックマークのヨーとピッチ | ラジアン |
| UVオフセット | テクセル |
| UVスケール | テクセルあたりのマップ単位 |
| メッシュの`uvs` | テクスチャの幅。1がテクスチャ1回分の繰り返しです |
