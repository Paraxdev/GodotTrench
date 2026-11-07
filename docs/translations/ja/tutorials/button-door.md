---
source: 19de62e149bf31bc7f0453a04737b8fbd5510db4
---

# ボタンでドアを開ける { #button-opens-a-door }

壁のボタンをプレイヤーが押すと、ドアがスライドして開きます。エンティティI/Oの中でも、実用になる一番小さな例です。ボタンが`pressed`という出力を発火し、リンクがそれをドアの`open`インプットの呼び出しに変えます。

> [!TIP]
> 出力、インプット、`targetname`がはじめての方は、先に[エンティティI/Oのしくみ](../gameplay/io.md)を読んでください。数分で読めますし、このページはその内容を前提にしています。

## 配線する { #wire-it }

`func_door`と`func_button`はゲームプレイエンティティパックに入っています。プロジェクトにまだ入っていないときは、*ゲームプレイ > ゲームプレイエンティティをインストール*を使うか、*このプロジェクトを設定*ウィンドウ(*Godot > アドオンをインストールまたは更新…*)の**ゲームプレイエンティティをインストール**を押してください。

1. ドアを作る

    出入口にドアの大きさのブラシを描きます。切り抜き方は[はじめの一歩](../getting-started.md#4-build-a-room)で説明しています。ブラシを右クリックして、*ブラシエンティティを作成 > func_door*を選びます。

    ![オレンジ色のドア用ブラシの右クリックメニュー。ブラシエンティティを作成が開き、func_doorにカーソルが重なっている](../assets/button-door/create-brush-entity-func-door.png)

2. 名前を付ける

    右側のインスペクターで、`targetname`に`main_door`と入力します。この名前で、ほかのエンティティがドアを見つけます。`travel`は`0 64 0`のままにしておくと、ドアは開くときに64ユニット(2 m)上へスライドします。

    ![func_doorのインスペクター。targetnameがmain_door、travelが0 64 0](../assets/button-door/inspector-targetname.png)

3. ボタンを作る

    ドアの隣の壁に小さなブラシを描き、同じ手順で`func_button`にします。

4. 両方を選ぶ

    ボタンをクリックし、続けてドアをCtrlクリックします。ビューでも、左側のアウトライナーでも構いません。壁が邪魔なときは、アウトライナー上部の*フィルター*ボックスに`func`と入力すると、この2つだけが一覧に残ります。

    ![funcで絞り込んだアウトライナー。func_door(main_door)とfunc_buttonの両方が選択されている](../assets/button-door/outliner-filter-func.png)

5. リンクする

    *ゲームプレイ > ロジック > 選択した2つのエンティティをリンク…*を開くか、インスペクター上部の**リンク…**を押します。

    ![ゲームプレイメニューでロジックが開き、選択した2つのエンティティをリンク…にカーソルが重なっている](../assets/button-door/menu-link-two-selected.png)

    発火するエンティティは左側になければいけません。ここではボタンです。ダイアログは先に作ったほうを左に置くので、今はドアが左にあります。2つの名前の間にある小さな入れ替えボタンを押してください。*条件*で`pressed`を、*呼出*で`open`を選び、**リンク**を押します。

    ![エンティティをリンクウィンドウ。左にfunc_button、右にfunc_door(main_door)、条件はpressed、呼出はopen](../assets/button-door/link-dialog.png)

左下のステータスバーに`pressed > main_door.open`と出ているはずです。「pressedのとき、main_doorのopenを呼ぶ」と読みます。

![pressed > main_door.openと表示されたステータスバー](../assets/button-door/status-linked.png)

ボタンだけを選ぶと、インスペクターの*出力*の中にリンクがあります。あとからここで変更したり削除したりできます。

![func_buttonの出力セクション。出力はpressed、ターゲットはmain_door、インプットはopen](../assets/button-door/inspector-button-output.png)

> [!TIP]
> インスペクターの*ゲームプレイ*セクションにもある*ゲームプレイ > ドアと可動物*は、選択したブラシをワンクリックで蝶番ドアやスライドドアにします。ドアに名前を付け、開き方を設定し、プレイヤーが近づくとドアを開ける`trigger_multiple`を手前に置きます。ボタンだけでドアを開けたいときは、そのトリガーを削除してください。
>
> ![ゲームプレイメニューでドアと可動物が開いた状態。左右どちらかに蝶番のある蝶番ドア、上または横にスライドするスライドドア、リフト・移動プラットフォーム](../assets/button-door/menu-doors-and-movers.png)

## プレイヤーに押してもらう { #let-the-player-press-it }

アドオンには独自のインタラクションの仕組みがありません。「使う」操作はゲームごとに違うからです。ドアとボタンには`use(activator)`メソッドがあり、プレイヤーのスクリプトがそれを呼びます。デモのプレイヤーは**E**キーで呼びます。カメラからレイを飛ばして、最初に当たった`use`を持つものに対して`use`を呼びます([godot/demo/player.gd](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/player.gd))。

```gdscript
func use_target() -> void:
	var from := camera.global_position
	var query := PhysicsRayQueryParameters3D.create(from, from - camera.global_basis.z * use_distance)
	query.exclude = [get_rid()]
	query.collide_with_areas = false
	var hit := get_world_3d().direct_space_state.intersect_ray(query)
	var node: Node = hit.get("collider")
	while node:
		if node.has_method(&"use"):
			node.call(&"use", self)
			return
		node = node.get_parent()
```

プレイヤーは自分自身をアクティベーターとして渡すので、`pressed(activator)`はプレイヤーを運びます。連鎖の後ろの出力でターゲットに`!activator`を指定すると、ボタンを押した本人に作用させられます。

## バリエーション { #variations }

| やりたいこと | 変えるところ |
| --- | --- |
| ドアをボタンからだけ開けたい | ドアの`interact`のチェックを外します。外さないと、ドアに向かって**E**を押してもドアが開きます |
| ドアが勝手に閉まる | ドアの`wait`に、開いたままでいる秒数を設定します。既定の-1では開いたままです |
| 鍵のかかったドア | `locked`にチェックを入れます。ドアは`open`と`use`を無視し、代わりに`locked_use`を発火します。出力がドアの`unlock`インプットを呼ぶまで続きます。たとえば鍵の取得から呼ぶ形です |
| 1つのボタンで複数のドア | ドアに`gate_1`、`gate_2`のように名前を付け、`gate_*`をターゲットにします。末尾の`*`は、`gate_`で始まるすべての名前に一致します |
| 開き戸 | 代わりに`func_door_rotating`を使い、ビュー上の蝶番ハンドルを、蝶番を置きたい場所へドラッグします |

## コードで同じ配線をする { #the-same-wiring-in-code }

エディターではなくGDScriptでつなぎたいときは、こう書けば上のリンクと同じことになります。

```gdscript
# pressed(activator) > main_door.open
button.pressed.connect(func(activator):
	for door in TrenchDotIO.find_targets(button, "main_door", activator):
		TrenchDotIO.invoke(door, &"open", "", activator))
```

`pressed`を`door.open`に直接つなぐと失敗します。`pressed`は引数を1つ渡しますが、`open()`は引数を取らないからです。`invoke`はマップのリンクと同じように、メソッドに合わせて引数を調整します。
