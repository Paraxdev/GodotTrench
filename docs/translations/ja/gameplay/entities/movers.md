---
source: 63266bb349fb8b02f543d43498bfdee3d8c3f3cd
---

# ドア、ムーバー、ボタン { #doors-movers-and-buttons }

> [!NOTE]
> これらのエンティティは、ゲームプレイエンティティパックに含まれています。
> [インストールとカスタマイズ](README.md#installing-and-customizing)を参照してください。

一つのボディとして動き、上に乗っているものも運ぶブラシエンティティです。たとえばリフトは、プレイヤーを乗せたまま動きます。

> [!NOTE]
> ドアとボタンは、それだけではプレイヤーに反応しません。プレイヤーのスクリプトが、たとえばインタラクション用のレイから`use(activator)`を呼びます。[ボタンでドアを開ける](../../tutorials/button-door.md)を参照してください。I/Oやスクリプトだけで動かしたいものは、`interact`を0にします。

## func_door { #func_door }

スライドドア、ハッチ、シャッターです。開くときは`travel`だけ動き、閉じるときは戻ります。値の設定は、ビューポートでハンドルをドラッグして行います。ロックされたドアは`open`と`use`を無視し、代わりに`locked_use`を起動します。「鍵がかかっています」のようなメッセージを出すときに使います。

| キー | デフォルト | 働き |
| --- | --- | --- |
| `travel` | 0 64 0 | スライドする量。マップ単位です |
| `speed` | 2.0 | メートル毎秒 |
| `wait` | -1 | -1なら開いたまま。正の値なら、その秒数のあとにまた閉じます |
| `locked` | 0 | ロックされた状態で始まります |
| `start_open` | 0 | 開いた位置で始まります |
| `interact` | 1 | プレイヤーが`use`を呼ぶと開きます |

* **インプット:** `open`、`close`、`toggle`、`lock`、`unlock`、`use(activator)`
* **アウトプット:** `opened`、`closed`、`started_opening`、`started_closing`、`locked_use(activator)`

## func_door_rotating { #func_door_rotating }

ビューポートで位置をドラッグして決める蝶番を軸に、回って開くドアやゲートです。`open`はアクティベーターを受け取るので、`open_away`を使えば、開けた人の顔に向かってではなく、離れる向きに開かせられます。

| キー | デフォルト | 働き |
| --- | --- | --- |
| `hinge` | 0 0 0 | ドアの中心から見た蝶番の位置 |
| `axis` | y | 回転軸。`y`は縦の蝶番で、`x`か`z`にするとハッチや跳ね橋になります |
| `open_angle` | 95 | 回る角度(度)。負の値なら逆向きに回ります |
| `speed` | 120 | 1秒あたりの度数 |
| `open_away` | 1 | 開けた人から離れる向きに回ります |
| `wait`、`locked`、`start_open`、`interact` | -1、0、0、1 | `func_door`と同じです |

* **インプット:** `open(activator)`、`close`、`toggle(activator)`、`lock`、`unlock`、`use(activator)`
* **アウトプット:** `func_door`と同じ

`func_gate`は古い別名で、キーは`hinge`、`open_angle` 100、`speed` 90だけです。`func_door_rotating`を使ってください。

## func_platform { #func_platform }

開始位置と`start + travel`のあいだを動くリフトや移動床です。エレベーターや、穴を渡る床に使います。

* **インプット:** `start`、`stop`、`toggle`、`go_to_end`、`go_to_start`
* **アウトプット:** `reached_end`、`reached_start`、`started`
* **キー:** `travel` 0 128 0、`speed` 2.0、`wait` 1.0、`mode` 0、`start_active` 0(マップの読み込みから動きます)

| `mode` | `start`の動作 | よくある使い道 |
| --- | --- | --- |
| 0、トグル | もう一方の端へ動く | ボタンで呼ぶリフト |
| 1、往復 | `stop`まで行き来し、両端で`wait`秒待つ | 動き続ける床 |
| 2、一度だけ | 終点まで動いてそこに留まる | 一度だけ伸びる橋 |

## func_train { #func_train }

`target`から始まる`path_corner`エンティティの連なりをたどります。路面電車やレールの上のカートのように、いくつもの停留所がある経路を走るものに使います。各コーナーの`wait`と`speed`は、到着したときに適用されます。マップの読み込みと同時に動き出してループするので、一度きりの移動にするには、`start_active`か`loop`を0にします。

* **インプット:** `start`、`stop`、`toggle`
* **アウトプット:** `arrived(corner)`、`finished`(ループしないときだけ)
* **キー:** `target`、`speed` 3.0、`loop` 1、`start_active` 1、`orient` 0(1にすると進む向きへ回転します)

## func_button { #func_button }

押されているあいだ`travel`だけ沈み、`pressed`を起動します。`wait`秒後にまた戻り、-1にすると押されたままになるので、一度きりのスイッチになります。ロックされたボタンは、代わりに`locked_use`を起動します。先に電源が要るパネルに便利です。

* **インプット:** `press(activator)`、`release`、`lock`、`unlock`、`use(activator)`
* **アウトプット:** `pressed(activator)`、`released`、`locked_use(activator)`
* **キー:** `travel` 0 -2 0、`wait` 1.0、`locked` 0、`interact` 1
