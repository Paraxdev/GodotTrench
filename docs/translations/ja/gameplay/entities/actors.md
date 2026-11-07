---
source: 978a2cda95dff6236396e32b6d494bed11f3f94b
---

# スポナーとパス { #spawners-and-paths }

> [!NOTE]
> これらのエンティティは、ゲームプレイエンティティパックに含まれています。
> [インストールとカスタマイズ](README.md#installing-and-customizing)を参照してください。

## info_spawner { #info_spawner }

敵やピックアップなどのシーンを、`radius`の範囲内のランダムな場所にスポーンします。待ち伏せにはトリガーから`spawn`を起動し、絶え間なく出したいときは`interval`を設定して`start`します。部屋全体に散らしたいときは、[`trigger_spawn_area`](triggers.md#trigger_spawn_area)を使ってください。

| キー | デフォルト | 働き |
| --- | --- | --- |
| `scene` | | スポーンする`res://`のシーン |
| `count` | 1 | 一度にスポーンする数 |
| `max_alive` | 5 | 同時に生きている数の上限。超える分のスポーンは飛ばされます |
| `total` | 0 | この数だけスポーンしたら完全に止まります。0なら無制限です |
| `interval` | 0 | タイマーが動いているあいだの、スポーンの間隔(秒)。0なら`spawn`のときだけスポーンします |
| `radius` | 64 | スポナーからどれだけ離れた場所に出てよいか。ビューポートでハンドルをドラッグします |
| `start_active` | 0 | マップの読み込みからタイマーを動かします |
| `spawn_on_ready` | 0 | マップを読み込んだときに一度スポーンします |
| `spawn_group` | enemies | スポーンしたすべてのノードに加えるGodotグループ |
| `snap_to_ground` | 1 | スポーンしたものを、その下の床に落とします |

* **インプット:** `spawn`、`start`、`stop`、`toggle`、`kill_all`
* **アウトプット:** `spawned(node)`、`all_dead`、`exhausted`(`total`に初めて達したときに一度だけ)

`all_dead`は、スポーンしたノードがすべていなくなったときに起動します。「部屋を片付けるとドアが開く」といった仕掛けに使います。スポーンしたノードにはtargetnameがないので、`@enemies`のように`spawn_group`で指します。

デフォルトのキーのままだと、何かが`spawn`を起動するまで何もスポーンしません。トリガーなしでスポーンさせるには、`spawn_on_ready`を設定するか、`interval`と`start_active`を設定します。スポーンは実行中のゲームでだけ行われ、Godotのエディターでは、スポナーは空のノードのままです。

## info_teleport_destination { #info_teleport_destination }

`trigger_teleport`がボディを送る先を示します。到着したプレイヤーに見せたい向きにこれを向けると、そのヨーがプレイヤーの向きになります。キーは`targetname`だけです。

## path_corner { #path_corner }

`func_train`や`npc_walker`の経路にある停留所の一つで、`target`に次の停留所を指定します。**パス**ツール(Shift+P)でクリックしていくと、連なりを置けます。トレインやウォーカーが着くと`reached`を起動するので、その停留所で音やイベントを起こせます。

* **アウトプット:** `reached(train)`
* **キー:** `target`、`wait` 0(ここで待つ秒数)、`speed` 0(ここから先のトレインの新しい速度。0なら変えません)

`speed`が効くのはトレインだけです。ウォーカーは自分の速度を保ちます。

## npc_walker { #npc_walker }

パスコーナーの連なりを、アニメーションを再生しながら歩く、台本どおりのアクターです。AIではなくカットシーン向けで、たとえば窓の前を通りかかる警備員のようなものです。直線に動き、重力も壁も無視するので、歩ける場所にコーナーを置いてください。

| キー | デフォルト | 働き |
| --- | --- | --- |
| `model` | | メッシュと`AnimationPlayer`を持つシーン |
| `target` | | 最初に向かうパスコーナー |
| `speed` | 2.0 | メートル毎秒 |
| `loop` | 0 | 最後のコーナーのあと、最初のコーナーに戻ります |
| `start_active` | 0 | マップの読み込みから歩き出します |
| `walk_anim`、`idle_anim` | walk、idle | 歩いているときと止まっているときに再生するアニメーション |
| `face_travel` | 1 | 歩く向きに体を向けます |
| `collision` | default | モデルから作る形状。ほかのボディがぶつかります。[コリジョン](collision.md)を参照してください |
| `collision_layer`、`collision_mask` | 空 | `1, 3`のようなレイヤー番号。空ならプロジェクトのデフォルトのままです |

* **インプット:** `start`、`stop`、`walk_to(corner)`、`play_anim(name)`、`face(target)`
* **アウトプット:** `arrived(corner)`、`reached_goal`、`finished`(ループしないときは、どちらも最後のコーナーで)

`face`と`walk_to`は、`!player`のようにどんなターゲットでも受け付けます。`walk_to`は、いまの区間をやめて、まっすぐそこへ向かいます。

`model`を読み込むのは実行中のゲームだけなので、Godotのエディターでは空のノードです。
