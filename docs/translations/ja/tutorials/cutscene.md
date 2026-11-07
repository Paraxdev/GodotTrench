---
source: 12dfe8079d66dd10bb5ca7191aa90e4b3c01fe5b
---

# スクリプトによるカットシーン { #scripted-cutscene }

プレイヤーが部屋に入ると、看板が現れ、案内役が歩き去り、樽が爆発し、ランプが点いて、出口が開きます。専用のコードは要りません。タイマーエンティティが拍ごとに出力を1つ発火し、それぞれの出力を、動かしたいエンティティに配線するだけです。デモプロジェクトの`scripted_scene`マップで見られます。

![ロジックパネルに表示した`scripted_scene`マップ。start_zoneがcutscene.startを発火し、cutsceneのステップ出力が案内役、樽、ヒントへつながり、樽のbroken出力がhp_readoutへつながっている。ランプと出口のドアは下のほうにあり、画像には入っていない](../assets/cutscene/logic-graph.png)

## タイムライン { #the-timeline }

ここで使うエンティティはすべてゲームプレイエンティティパックに入っています。プロジェクトにまだ入っていないときは、先に*ゲームプレイ > ゲームプレイエンティティをインストール*でインストールしてください。

1. 入口を覆う

    入口の上にブラシを描き、`start_zone`という名前の`trigger_once`にします(右クリックして*ブラシエンティティを作成 > trigger_once*)。プレイヤーが最初に入ったときに`triggered`を発火し、そのあと自分で無効になるので、シーンは一度しか再生されません。

2. タイムラインを追加する

    *ゲームプレイ > ロジック*から`logic_sequence`を選ぶと、3Dカーソルの位置に置かれます。名前を`cutscene`にして、`steps`を5、`interval`を1.5にします。開始すると、`step_1`、`step_2`、と1.5秒おきに発火します。

    ![ゲームプレイメニューでロジックが開き、カーソル位置に置くの下でlogic_sequenceにカーソルが重なっている](../assets/cutscene/menu-logic-sequence.png)

3. トリガーから開始する

    `start_zone`の`triggered`出力を`cutscene.start`にリンクします。[ボタンでドアを開ける](button-door.md)と同じように、*ゲームプレイ > ロジック > 選択した2つのエンティティをリンク…*を使います。

4. 拍ごとに配線する

    `cutscene`の各ステップ出力を、その拍で動くエンティティへ、1組ずつリンクします。[ロジックパネル](../gameplay/logic-graph.md)で、出力ピンからインプットピンへドラッグしても同じことができます。

| 出力 | ターゲットとインプット | 起きること |
| --- | --- | --- |
| `step_1` | `hint.show` | `game_text`が「Follow the guide」と表示する |
| `step_2` | `guide.start` | `npc_walker`が`path_corner`の連なりに沿って歩き出す。`path_corner`は、それぞれが次の1つを指すマーカーの列 |
| `step_3` | `barrel.ignite` | 爆発する`prop_physics`の導火線が燃え、爆発して壊れる |
| `step_4` | `lamp.turn_on` | ランプが点く |
| `step_5` | `exit_door.open` | 出口のドアが上へスライドする |

`cutscene`を選ぶと、インスペクターの*出力*に5つすべてが並んでいるはずです。

![cutsceneという名前のlogic_sequenceのインスペクター。stepsは5、intervalは1.5で、出力(I/O, 5)はstep_1からhint.showへの行で始まる](../assets/cutscene/inspector-sequence.png)

シーケンスは最初のステップも含めて、どのステップの前にも待ちます。なので、ステップ1は1.5秒、ステップ5は7.5秒で発火します。間隔をそろえたくないときは、`times`にステップごとの待ち時間を`0.5 2 1 1 3`のように設定します。

## プレイヤーの体力を読む { #reading-the-players-health }

組み込みのインプットでは足りない、ちょっとしたロジックが必要な反応もあります。`logic_script`は、発火されると短いGDScriptを実行します。樽の`broken`出力を`hp_readout.run`にリンクします。これはHUDの`game_text`(名前は`hpmsg`)にプレイヤーの体力を書き込む`logic_script`で、`source`は次のとおりです。

```gdscript
var players = io.find_targets(this, "!player", null)
if players.is_empty():
	return
var hp = int(players[0].health)
for label in io.find_targets(this, "hpmsg", null):
	label.set_text("HP " + str(hp))
	label.show()
```

ここでは`!activator`ではなく`!player`を使います。`!activator`は連鎖を始めた相手ですが、ステップ出力と`broken`は誰も運びません。`!player`なら、`player`グループのノードがいつでも見つかります。`broken`は爆発がダメージを与えたあとに発火するので、体力はすでに最新です。

## 再生する { #playing-it }

シーンスクリプトの[godot/demo/scenes/scripted_scene.gd](https://github.com/Paraxdev/TrenchDot/blob/main/godot/demo/scenes/scripted_scene.gd)は、マップを作り、`info_player_start`にプレイヤーを出現させ、`TrenchDotDebugOverlay`を追加するだけです。ゲーム中に**F3**を押すと、出力が発火するたびに表示されます。ステップが間違った名前に配線されているのを見つけるには、これが一番早い方法です。

<div class="td-mcp" markdown>

## MCP

デモマップは、[MCPサーバー](../mcp.md)経由でエディターを操作して作っています。呼び出しは[examples/mcp/scripted_scene.json](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/scripted_scene.json)にあります。

</div>
