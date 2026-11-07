---
source: 2bf109094dcc42ebb11cf6c736e210c0e4e79b02
---

# スクリプトと呼び出し { #scripts-and-calls }

> [!NOTE]
> これらのエンティティは、ゲームプレイエンティティパックに含まれています。
> [インストールとカスタマイズ](README.md#installing-and-customizing)を参照してください。

単純な配線では届かないもののためのポイントエンティティです。ゲームの状態、自分のゲームコード、アニメーション、そして連鎖が起動しない理由の調査です。

## logic_script { #logic_script }

起動されたときにGDScriptのスニペットを実行します。組み込みのエンティティにはない小さなロジック、たとえばプレイヤーの体力を読むようなことを、オリジナルエンティティを作らずに行えます。

* **インプット:** `run(activator)`、`run_with(parameter)`
* **アウトプット:** `ran(result)`。スニペットの戻り値が付きます
* **キー:** `source` 空、`expression` 空(1行のコードで、`source`が空のときに使われます)

スニペットからは、`this`(エンティティ自身)、`activator`(連鎖を始めたもの)、`parameter`(届いた値)、`io`(`TrenchDotIO`のヘルパー。たとえば名前でエンティティを探せます)、`tree`(`SceneTree`)が見えます。`run_with`では、`activator`は常にnullです。コンパイルエラーは、マップの読み込み時に警告としてログに出ます。次のスニペットはプレイヤーにダメージを与えます。

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

> [!WARNING]
> ネストした行のインデントにはタブを使ってください。スペースだと、インデントが混在しているというエラーでコンパイルに失敗します。

> [!WARNING]
> スニペットは、マップの中に入っているコードです。ゲームが動くたびに、プロジェクト自身のスクリプトと同じ権限で実行されます。ですから、コードを含むマップをダウンロードしたときは、ダウンロードしたプログラムと同じように扱ってください。コンピューター上のどんなことでもできてしまいます。問題パネルは、コードを持つ`logic_script`、コードを渡せるアウトプット、そのどちらかを含むプレハブのすべてについて警告します。マップを遊ぶ前に、それらを読んでください。

## logic_call { #logic_call }

任意のターゲットのメソッドを、固定の引数で呼び出します。`Game`オートロードにスコアを加えるなど、自分のゲームコードに届くために使います。ふつうのアウトプットと違い、ターゲットやメソッドがないときは警告を出すので、タイプミスがログに現れます。C#のメソッドは、PascalCaseの名前で動きます。

* **インプット:** `trigger(activator)`、`call_with(parameter)`
* **アウトプット:** `called(result)`
* **キー:** `call_target` `/root/Game`、`method` 空、`arguments` `[]`

`call_target`は、ノードパス、`@group`、targetname、`!activator`、`!self`のどれかです。`arguments`はJSONの配列で、たとえば`["key_red", "$activator"]`のように書き、[パラメーター](../parameters.md)にあるプレースホルダーが使えます。プレースホルダーは`logic_call`自身で解決されます。`call_with`は、受け取ったパラメーターを`"$parameter"`のある場所に入れるか、それが唯一の引数として渡します。

## logic_animate { #logic_animate }

`target`にある、またはその下にある最初の`AnimationPlayer`でアニメーションを再生します。自分のアニメーションを持つキャラクターやプロップに使います。`target`を`!activator`にすると、連鎖を始めたものがアニメーションします。

* **インプット:** `play(name)`、`stop`、`queue(name)`、`seek(time)`
* **アウトプット:** `finished(anim)`
* **キー:** `target` `!self`、`animation` 空(`play`に名前がないときに再生されます)、`speed` 1.0

## logic_debug { #logic_debug }

`write`を受けると1行を出力し、デバッグビルドでは最近の行を浮かぶラベルとして表示します。連鎖がここに届いているか、どんなパラメーターで届いているかを、ゲームの中で確認できます。`trace_all`をオンにすると、ゲーム内のすべてのアウトプットをログに出すので、連鎖全体を手早く見られます。[デバッグ](../debugging.md)を参照してください。

* **インプット:** `write(parameter)`、`toggle`(ラベル)、`show`、`hide`
* **アウトプット:** `printed(text)`
* **キー:** `message` `{name} received {parameter} from {activator}`(`{time}`も使えます)、`show_label` 1、
  `trace_all` 0
