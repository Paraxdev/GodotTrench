---
source: 755d17f52aebf0b32513b6e22a04abb368badeab
---

# マップをビルドする { #building-maps }

TrenchDotは`.tdm`ファイルを保存するだけで、Godotのシーンは自分では書きません。Godotでは、アドオンの`FuncGodotMap`ノードがそのファイルを読み、メッシュ、コリジョン、エンティティのノードを生成します。結果は、手で作ったノードと同じようにシーンと一緒に保存されるので、求めない限り、実行時に何かが再ビルドされることはありません。

## FuncGodotMapノード { #the-funcgodotmap-node }

1. ノードを追加する

    Godotエディターでシーンを開いてCtrl+Aを押し、`FuncGodotMap`を検索して追加します。

2. マップを指定する

    ノードを選んだ状態で、GodotのインスペクターにあるLocal Map Fileに、保存した`.tdm`を設定します。

3. ビルドする

    インスペクターの上部にある**Build Map**をクリックします。Godotのビューポートにマップが表示され、ノードにはメッシュとエンティティの子ができるはずです。Ctrl+Sでシーンを保存すると、それが残ります。**Clear Map**でそれらをまた取り除けます。

インスペクターに表示される設定は次のとおりです。

| プロパティ | 働き |
| --- | --- |
| Local Map File | ビルドする`.tdm`の`res://`パスです。通常はこちらを使います。 |
| Global Map File | プロジェクトの外にあるマップファイルです。両方が設定されているときは、こちらが優先されます。 |
| Auto Rebuild On Save | TrenchDotがこのマップを保存するたびに、そのシーンがGodotエディターの現在のタブである間、ノードを再ビルドします。[保存時の再ビルド](live.md#rebuild-on-save)を参照してください。デフォルトでオンです。 |
| Map Settings | テクスチャ、マテリアル、エンティティ定義の取得元と、1マップユニットの大きさを決めます。空のままにすると、プロジェクト設定`func_godot/default_map_settings`が使われます。[プロジェクトのセットアップ](project-setup.md)を参照してください。古いアドオンで追加したマップノードには、アドオンのデフォルトのマップ設定がここに保存されていて、ビルドするとGodotの出力に警告が出ます。プロジェクト設定に従わせるには、この欄を空にしてください。 |
| Use Baked Lighting | [エディターでベイクした](baked-lighting.md)ライティングでマップを照らします。デフォルトでオンです。別のライトマッパーを使うときは、チェックを外してください。 |
| Warm Up Shaders | マップの読み込み中にそのシェーダーをコンパイルするので、プレイ開始直後の数秒がカクつきません。[シェーダーのウォームアップ](warm-up.md)を参照してください。 |

アドオンがビルドするのは`.tdm`ファイルだけです。Quakeの`.map`やHammerの`.vmf`を使うには、TrenchDotで*ファイル > インポート*を使って開き、先に`.tdm`として保存してください。

> [!WARNING]
> ビルドすると、[オーバーレイ](overlays.md)を除く、マップノードのすべての子が削除されます。自分のノードは、マップノードの隣か、オーバーレイの中に置いてください。

## 3つのビルド方法 { #three-ways-to-build }

| きっかけ | ビルドされるもの | 使いどころ |
| --- | --- | --- |
| TrenchDotで保存 | Godotエディターで開いているシーンのうち、そのファイルを使うすべての`FuncGodotMap` | 日常の流れで、保存してGodotで結果を見ます |
| *Godot > Godotでビルド* | 今TrenchDotにあるままのマップ。保存済みでなくても、ファイルは書き込まれません | 確定させずに、未保存の変更をGodotで試すとき |
| Godotの**Build Map** | 保存済みのファイル | TrenchDotを閉じているときや、開いたばかりのシーンを再ビルドするとき |

最初の2つは、このプロジェクトを開いたGodotエディターが必要で、そこで開いているシーンにしか作用しません。再ビルドするとそのシーンが変更済みの扱いになるので、結果を残すにはGodotで保存してください。

Godotがプロジェクトを開くまで、*Godotでビルド*はグレーのままです。また、Godotがどの`FuncGodotMap`がそれを使うかを分かるように、マップを一度プロジェクトに保存しておく必要もあります。

![Godotでビルドにカーソルを合わせたGodotメニュー](../assets/building/menu-build-in-godot.png)

## コードからビルドする { #building-from-code }

ゲームの実行中にマップをビルドすることもできます。たとえば、メニューで選んだレベルを読み込むときです。

```gdscript
var map := FuncGodotMap.new()
map.map_settings = load("res://demo/demo_map_settings.tres")
map.local_map_file = "res://demo/maps/scripted_scene.tdm"
add_child(map)
map.build()
```

ノードは、完了すると`build_complete`を、マップファイルがないか`.tdm`でないと`build_failed`を発します。

## 環境と太陽 { #environment-and-sun }

worldspawnのスカイ、フォグ、太陽のキーから、`WorldEnvironment`と`sun`という名前のライトがマップの中に作られるので、Godotで何も設定しなくても、マップは照らされて見えます。Escapeを押して何も選択していない状態にすると、インスペクターにこれらのキーを並べた*マップ (worldspawn)*が表示されます。その`environment`キーが、何を作るかを決めます。設定していないキーはマップに入らないので、どれも設定していないマップでは、環境がまったく作られません。古いアドオンで作ったプロジェクトにこれらのキーが表示されないときは、アドオンを更新して、[ゲーム設定をエクスポート](project-setup.md#the-game-config)し直してください。

| `environment` | 作られるもの |
| --- | --- |
| `1` | 環境と太陽です。これがデフォルトです。 |
| `sun_only` | 太陽だけです。独自の`WorldEnvironment`を持つゲーム向けです |
| `0`または`none` | 何も作りません |

シーンにすでに独自の`WorldEnvironment`や`DirectionalLight3D`があるときは、マップはそれをそのまま使い、何も追加しないので、独自の見た目を持つゲームが常に優先されます。1つのシーンを複数のマップが共有しているときは、最初にビルドされたマップが照らします。

## ビルドレポート { #build-report }

`build()`はDictionaryを返し、それは`build_report`にも保持されます。各ステップにかかった時間、クラスごとに作られたエンティティの数、マップの頂点の数が分かります。いちばん役に立つのは、ビルドが見つけられなかったものの一覧で、たいていはタイプミスや、名前が変わった、あるいは一度もコミットされなかったファイルが原因です。

| キー | 一覧にするもの | ゲームでの見え方 |
| --- | --- | --- |
| `missing_classes` | どのエンティティ定義にも記述がないエンティティクラス | ブラシエンティティはメッシュを保ちますが、スクリプトもコリジョンもありません。ポイントエンティティは空の`Marker3D`になります。[何も起きないエンティティ](entities-troubleshooting.md)を参照してください |
| `missing_models` | 存在しないモデルのパスと、それぞれを使うエンティティ | プロップにモデルがありません |
| `missing_materials` | マテリアルファイルも画像もない面のテクスチャ | 面にプレースホルダーのテクスチャが表示されます |
| `unknown_tool_textures` | `special/clip`のようなツールテクスチャで、マップ設定では別の名前になっているもの | 面は引き続き何も描画しませんが、オリジンのブラシはエンティティのオリジンを設定しません |
| `unresolved_targets` | ターゲットが、マップやそのオーバーレイのどのtargetnameにも一致しない、アウトプットとtargetのキー | アウトプットは何にも届かずに発火します |

ビルドのたびに表示するには、ノードの*Build Flags*の下にある*Print Build Report*にチェックを付けるか、コードで`build_flags`に`FuncGodotMap.BuildFlags.PRINT_REPORT`を加えます。テストやCIのチェックでは、代わりにDictionaryを読めます。

```gdscript
var report := map.build()
if not report["missing_models"].is_empty():
	push_error("missing models: %s" % [report["missing_models"]])
```

ビルドは毎回、見つけられなかったものについてGodotの出力に警告を出します。見つからないクラスは全部まとめて1件、見つからないモデル、マテリアル、ツールテクスチャは、そのパスを何個のプロップが共有していても、1つにつき1件です。保存のあと、TrenchDotは同じ警告を問題パネルに一覧にします。

> [!NOTE]
> `build()`のあとに追加したオーバーレイは、レポートがターゲットを探す時点ではまだないので、そのtargetnameは`unresolved_targets`に出てきます。

## 壊れたマップ { #damaged-maps }

壊れたマップも、読み込めた部分からビルドされ、読み込めなかった部分ごとに`[TDM]`の警告が出ます。[.tdmマップ形式](../format/recovery.md)を参照してください。
