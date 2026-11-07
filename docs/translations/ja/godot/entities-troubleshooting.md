---
source: 1b2a2ceda473607c8db087c552aff86851d08ef2
---

# 何も起きないエンティティ { #entities-that-do-nothing }

Godotが定義のないエンティティをビルドすると、そのエンティティはただの殻になります。ドアのようなブラシエンティティはメッシュを保ちますが、スクリプトもコリジョンもない普通の`Node3D`になり、ポイントエンティティは空の`Marker3D`になります。物理ボディを作らない定義を持つブラシエンティティのように、定義がエンティティに合っていないときも同じことが起きます。こうなったときは、TrenchDotもGodotも教えてくれます。このページでは、それぞれのメッセージを説明します。

## どこに表示されるか { #where-you-see-it }

1. ステータスバー

    シーンをGodotで開いているあいだに、TrenchDotでマップを保存します。Godotが定義のないクラスをビルドしたときは、左下のステータスバーに、そのクラスと、Godotがビルドに使ったマップ設定が表示されます。

    ![Godotがマップノード1個を再ビルドしたが、res://crypt_settings.tresにfunc_buttonとfunc_doorの定義がなく、通常のノードとしてビルドされるというステータスバー](../assets/entities-troubleshooting/status-missing-classes.png)

2. 問題パネル

    左のアウトライナーと履歴の隣にある**問題**タブを開きます。Godotが通常のノードとしてビルドしたクラスが、それぞれ対処法つきで一覧になり、そのビルドの他の警告は「Godot：」で始まります。行をクリックすると、そのエンティティが選択されます。

    ![警告が2件ある問題パネル。Godotがfunc_buttonとfunc_doorを通常のノードとしてビルドしたことと、Base Fgd Filesにどのファイルを追加するかが、それぞれに表示されている](../assets/entities-troubleshooting/issues-missing-classes.png)

    Godotがマップをビルドする前は、問題パネルが、エンティティを、Godotがビルドに使うFGDと比べます。そのFGDは、アドオンが[ゲーム設定](project-setup.md#the-game-config)と一緒にエクスポートしたものです。そのFGDにないエンティティには、「GodotはXでビルドしますが、そのFGDに...がありません」という警告が出ます。

    Godotが、定義がなかった時点でビルドしたシーンを表示しているときは、ステータスバーと問題パネルに「Godotが表示している...は、現在のマップ設定とは別のエンティティ定義でビルドされています」と表示されます。*Godot > Godotでビルド*を選ぶか、Godotで**Build Map**を押して保存すると、もう一度ビルドされます。**プロジェクトを実行**も、実行するシーンについて同じことを伝えます。

3. GodotのOutputパネル

    Godotで`FuncGodotMap`ノードの**Build Map**を押します。Godotエディターの下にあるOutputパネルに、定義のないクラスについて1件と、合っていない定義ごとに1件の`[TrenchDot]`の警告が出ます。[ビルドレポート](building.md#build-report)の全体を見るには、ノードの*Build Flags*の下にある*Print Build Report*にチェックを付けてください。

## 定義のないエンティティ { #entities-without-a-definition }

| メッセージの内容 | 理由 | 対処 |
| --- | --- | --- |
| ゲームプレイエンティティパックをインストールする | ドア、ボタン、トリガー、プロップ、スポナーはパックに含まれていて、プロジェクトにそれがありません | TrenchDotで*ゲームプレイ > ゲームプレイエンティティをインストール*を選びます。[プロジェクトのセットアップ](project-setup.md#installing-and-updating-the-pack)を参照してください |
| FGDにゲームプレイエンティティパックがありません。`trenchdot_default_fgd.tres`をそのBase Fgd Filesに追加してください | FGDにBase Fgd Filesがありません | マップ設定がEntity Fgdに指定しているFGDのBase Fgd Filesに、`res://addons/func_godot/fgd/trenchdot_default_fgd.tres`を追加します |
| パックはインストール済みですが、Entity Fgdがアドオンに含まれるFGDです | マップ設定が、FuncGodot自身のFGDか、アドオンの別のファイルを使っていて、アドオンを更新するとそれが置き換えられます | そのマップ設定のEntity Fgdを、ゲーム設定が使うFGDにします |
| 自作のクラスに定義がありません | ゲーム設定のFGDにはあるが、Godotがビルドに使うマップ設定のEntity Fgdにはありません | そのマップ設定のEntity Fgdを、ゲーム設定が使うFGDにします。[自分のリソース](project-setup.md#your-own-resources)を参照してください |
| マップノードが、プロジェクト設定のものではなく、アドオンのデフォルトのマップ設定を使っています | ノードが古いアドオンで追加され、そのアドオンがデフォルトのマップ設定をシーンに保存していました | Godotでマップノードを選び、インスペクターのMap Settingsを空にします |
| シーンが別のエンティティ定義でビルドされています | パックや定義が追加される前にビルドされ、それ以降はビルドされていません | **Build Map**を押して、シーンを保存します。[ライブモード](live.md#when-live-mode-rebuilds-the-whole-map)では、Godotが自動で行います |

## 合っていない定義 { #definitions-that-do-not-fit }

これらはGodotのOutputパネルに出て、保存のあとは問題パネルに「Godot：」の警告として出ます。最後の2つは、保存する前から問題パネルが指摘します。

| メッセージの内容 | 理由 | 対処 |
| --- | --- | --- |
| `func_crusher builds as a Marker3D, so its brushes have no collision` | 定義にNode Classがなく、そのスクリプトが物理ボディではないノードを継承しています | Node Classを`StaticBody3D`、`AnimatableBody3D`、`Area3D`のいずれかにするか、スクリプトにそのどれかを継承させます |
| `func_crusher: Node Class AnimatedBody3D not found, building it as a Node3D` | タイプミスか、Godotがまだ登録していない`class_name`です | 名前を直します。C#については[C#エンティティ](../gameplay/csharp.md)を参照してください |
| `alarm: res://entities/alarm.gd has no @tool` | エディターはtoolスクリプトしか実行しないので、マップのキーがノードに届きません | スクリプトの先頭に`@tool`を追加します |
| `light is a point entity, so its brushes are dropped` | ブラシエンティティに`FuncGodotFGDPointClass`が使われています | 定義を`FuncGodotFGDSolidClass`にします |
| `func_door is a brush entity without brushes` | ビルドの元になるものがないブラシエンティティです | ブラシを与えるか、定義をポイントクラスにします |

## ビルドはされるが反応しないエンティティ { #entities-that-build-but-do-not-react }

| 見えること | 理由 | 対処 |
| --- | --- | --- |
| トリガーがまったく発火せず、GodotのOutputに、`filter_group`にノードがないためトリガーがプレイヤーを無視したと出る | デフォルトでは、トリガーは物理レイヤー1の、グループ`player`にあるボディにしか反応しません | プレイヤーをグループ`player`に追加します。別のレイヤーなら、マップ設定のTrigger Collision Maskにそのレイヤーを加えます |
| ドアやボタンがまったく動かない | ゲームのコードがそれらの`use()`を呼ぶと動きます | [プレイヤーに押せるようにする](../tutorials/button-door.md#let-the-player-press-it)を参照してください |
| `info_spawner`と`npc_walker`がGodotのエディターで空に見える | モデルのスポーンや読み込みは、実行中のゲームでしか行われません | [スポナーとパス](../gameplay/entities/actors.md)を参照してください |
| `res://trenchdot/entities`のスクリプトについて、GodotのOutputに`Parse Error`と出る | Godotが、パックのスクリプトを、それが使うクラスを知る前にコンパイルしました | Godotを再起動してから、**Build Map**を押して保存します |
