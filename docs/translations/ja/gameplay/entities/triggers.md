---
source: c7fdbd8dcaa1ac8625c1d6e7014b686194c50f36
---

# トリガー { #triggers }

> [!NOTE]
> これらのエンティティは、ゲームプレイエンティティパックに含まれています。
> [インストールとカスタマイズ](README.md#installing-and-customizing)を参照してください。

ボディが入ったときに反応する、見えないブラシのボリュームです。レベルがプレイヤーに気づくしくみで、戸口を通るとカットシーンが始まり、穴に落ちるとダメージを受け、パッドを踏むと飛ばされます。**ボリューム**ツール(Shift+E)で描きます。

## 共通のインプット、アウトプット、キー { #shared-inputs-outputs-and-keys }

以下のトリガーはすべて、それぞれ固有のものに加えて、次を持っています。

* **インプット:** `enable`、`disable`、`toggle`
* **アウトプット:** `triggered(activator)`、`entered(activator)`、`exited(activator)`
* **キー:** `filter_group` player、`start_disabled` 0

`filter_group`は、トリガーが反応するボディを、そのGodotグループのものに限ります。敵や木箱がプレイヤー用のトリガーを作動させることはありません。どんなボディも受け付けたいときは、空にします。

| アウトプット | 起動するとき |
| --- | --- |
| `entered` | 有効なあいだ、受け付けた進入のたびに |
| `triggered` | 受け付けた進入のとき。ボディごとに`cooldown`あたり最大一度です。配線にはたいていこちらを使います |
| `exited` | 退出のとき。トリガーが無効でも起動します。ボディが出ていったときに何かを元に戻すためのものです |

トリガーは、物理レイヤー1のボディを検出します。プレイヤーが別のレイヤーにいるときは、マップ設定(Godotでは`FuncGodotMapSettings`リソース)の**Trigger Collision Mask**にそのレイヤーを加えてください。そうしないと、トリガーは起動しません。

## trigger_once { #trigger_once }

一度起動すると自分を無効にします。カットシーンやヒントのような、一度きりのイベントに使います。`enable`で再び使える状態に戻ります。

* **キー:** `once` 1

## trigger_multiple { #trigger_multiple }

進入のたびに起動します。ボディごとに`cooldown`あたり最大一度です。お店のベルのように、毎回起こることに使います。

* **キー:** `cooldown` 0.5

## trigger_hurt { #trigger_hurt }

ボディが中にいるあいだ、ダメージを与えます。溶岩、トゲ、マップの下のキルゾーンなどに使います。`interval`秒ごとに、ボディの`damage_method(amount, source)`を、`damage * interval`を量にして呼びます。

* **アウトプット:** `hurt(activator)`。ダメージを受けたボディごと、ティックごとに起動します
* **キー:** `damage` 10(1秒あたり)、`damage_method` take_damage、`interval` 0.5、`filter_group` 空

`filter_group`をデフォルトの空のままにすると、プレイヤーだけでなく、そのメソッドを持つものすべてにダメージを与えます。グループを設定したときは、メソッドを持たないボディでも、グループのボディすべてについて`hurt`が起動します。

## trigger_teleport { #trigger_teleport }

ボディを`destination`で指定したエンティティ、ふつうは[`info_teleport_destination`](actors.md#info_teleport_destination)へ移し、そのヨーの向きに回します。`keep_velocity`が1でない限り、ボディを止めます。`keep_velocity`は、プレイヤーが速度を落とさずに落ちたり走り抜けたりするポータルのためのものです。

* **アウトプット:** `teleported(activator)`
* **キー:** `destination`、`keep_velocity` 0、`cooldown` 0.5

## trigger_push { #trigger_push }

`push`でボディを押します。`push`は1秒あたりのマップ単位の速度で、ビューポートのハンドルで向きを決められます。`once`が1だと、進入のたびに一度だけ衝撃を与えます。ジャンプパッドのようなものです。0にすると、風洞やコンベアのように押し続け、剛体は`push`の速さで加速をやめます。

* **アウトプット:** `pushed(activator)`。進入ごとに一度です
* **キー:** `push` 0 384 0、`filter_group` 空、`once` 1

## trigger_call { #trigger_call }

トリガーされたときにメソッドを呼びます。呼ぶ先はデフォルトで`!activator`なので、トリガーからプレイヤーのスクリプトへ直接呼び出せます。たとえば、チェックポイントを保存するときです。`call_target`と`arguments`は、[`logic_call`](scripting.md#logic_call)と同じように動きます。

* **インプット:** `trigger(activator)`
* **アウトプット:** `called(result)`
* **キー:** `call_target` `!activator`、`method`、`arguments` `[]`、`once` 0、`cooldown` 0.5

## trigger_spawn_area { #trigger_spawn_area }

ボリュームの中のランダムな場所にシーンをスポーンします。デフォルトではプレイヤーが入ったときです。プレイヤーが部屋に入ると待ち伏せされる、といった場面に使います。キーは[`info_spawner`](actors.md#info_spawner)と同じように動きます。`start`は、`interval`が0より大きいときにスポーンのタイマーを動かし、ここでの`toggle`は、トリガーではなくそのタイマーを切り替えます。

* **インプット:** `spawn`、`start`、`stop`、`toggle`、`kill_all`
* **アウトプット:** `spawned(node)`、`all_dead`、`exhausted`(`total`に初めて達したときに一度だけ)
* **キー:** `scene`、`count` 3、`max_alive` 10、`total` 0、`interval` 0、`spawn_on_enter` 1、`spawn_group` enemies、
  `snap_to_ground` 1、`cooldown` 0.5

## trigger_area { #trigger_area }

スクリプトを持たないふつうの`Area3D`のボリュームです。部屋に入ったときと出たときのイベントを、ほかのエンティティへ直接配線するのに使います。上の共通のキーはどれも持たず、すべてのボディについて起動するので、それが問題になるときは、呼び出す先のエンティティで絞り込んでください。

* **アウトプット:** `body_entered(body)`、`body_exited(body)`
* **キー:** `targetname`
