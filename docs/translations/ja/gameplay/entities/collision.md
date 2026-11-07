---
source: d6310713b2a9d65be25e506f0179132b8e0e10fd
---

# コリジョン { #collision }

ゲームプレイパックのブラシエンティティと、モデルエンティティはすべて、何も設定しなくても衝突します。ドア、ボタン、リフト、トレイン、トリガーは、ブラシごとに凸包を持ちます。`prop_model`、`prop_physics`、`npc_walker`、散布のプロップは、モデルから作った形状を持ちます。デフォルトが合わないときだけ、以下のキーに触れてください。

## キー { #the-keys }

エンティティを選び、インスペクターでこれらの行を探します。行にカーソルを重ねると、それぞれの選択肢の働きが読めます。

![func_doorのインスペクター。コリジョンのメニューが開いていて、default、none、box、convex hull、convex decomposition、trimeshが並び、その下にcollision_layerとcollision_maskの行がある](../../assets/collision/inspector.png)

| キー | デフォルト | 働き |
| --- | --- | --- |
| `collision` | default | 形状。default、none、box、convex hull、convex decomposition、trimeshのどれか |
| `collision_layer` | 空 | ボディがいる物理レイヤー。`1, 3`のようなレイヤー番号で指定します。`none`にすると、どのレイヤーにもいなくなります |
| `collision_mask` | 空 | ボディが検出する物理レイヤー。同じ書き方です。`none`にすると、何も検出しません |

レイヤーやマスクが空なら、デフォルトのままです。ブラシエンティティではエンティティ定義のレイヤーとマスクが使われ、トリガーはマップ設定の*Trigger Collision Mask*を使います。モデルエンティティでは、下のプロジェクト設定が使われます。

## 形状の選び方 { #picking-a-shape }

| 形状 | ブラシエンティティ | モデル |
| --- | --- | --- |
| default | クラスの形状。ブラシごとの凸包 | プロジェクト設定。変えていなければ、メッシュごとの凸包。`prop_physics`は`size`から作るボックス |
| none | コリジョンなし | コリジョンなし |
| box | すべてのブラシを囲む一つのボックス | モデルを囲む一つのボックス。いちばん軽い形状です |
| convex hull | ブラシごとの凸包 | メッシュごとの凸包。箱型のモデルには十分です |
| convex decomposition | convex hullと同じ。ブラシはもともと凸形です | アーチのような凹んだモデルに沿う複数の凸包。作るのに時間がかかります |
| trimesh | すべての面。静的ボディだけです | すべての三角形。正確ですが、いちばん重い形状です |

trimeshが使えるのは、静的ボディだけです。ドア、ボタン、リフト、トレインはアニメーション可能なボディで、静的として数えられるので、使えます。トリガー、`prop_physics`、`npc_walker`は、代わりに凸包を作ります。trimeshにしたトリガーは、マップのビルド時に警告が出ます。

モデルがすでにコリジョンのボディを持っている`prop_model`は、そのボディを使い、自分では形状を作りません。

## プロジェクトのデフォルト { #project-defaults }

キーが空のモデルエンティティは、Godotの*Project Settings*の*Trenchdot > Collision*にある三つの設定に従います。TrenchDotプラグインを有効にすると現れます。

| 設定 | デフォルト | 働き |
| --- | --- | --- |
| *Model Shape* | convex | `prop_model`と`npc_walker`で、defaultが指す形状 |
| *Model Layer* | 1 | モデルのボディがいるレイヤー |
| *Model Mask* | 1 | モデルのボディが検出するレイヤー |

散布セットの形状は、散布セット自身の*Collision*の行で選びます。[散布](../../editor/scatter.md)を参照してください。

## 自作のブラシエンティティ { #your-own-brush-entities }

自分で定義したブラシエンティティは、*Node Class*に指定したボディとして衝突します。*Node Class*が空なら、スクリプトが継承するボディになり、スクリプトがない、またはスクリプトが`Node3D`を継承しているときは、`StaticBody3D`になります。`Marker3D`のような別のノードを継承するスクリプトだけは、コリジョンのない状態になり、ビルドで警告が出ます。[GDScriptでオリジナルエンティティを作る](../custom-entities.md)を参照してください。

エディターのビューポートは、コリジョンの形状を描画しません。Godotの*Debug > Visible Collision Shapes*を使うと、ゲームの実行中に表示できます。ライブセッションでも、フルビルドでも同じです。ブラシエンティティと`prop_model`は、形状をビルドしたシーンに保存します。`prop_physics`と`npc_walker`は、モデルを読み込むのと同じように、シーンを読み込むたびに形状を作ります。モデルのシーンがすでに物理ボディを持っている`prop_model`や`npc_walker`は、そのボディを使い、自分では形状を作りません。
