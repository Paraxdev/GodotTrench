---
source: c6245792b90999ca4b48812eb3d96e7a9c2389d6
---

# 数値と計算 { #math-and-values }

> [!NOTE]
> これらのエンティティは、ゲームプレイエンティティパックに含まれています。
> [インストールとカスタマイズ](README.md#installing-and-customizing)を参照してください。

数値を保存、計算、比較する、見えないポイントエンティティです。スコア、回数、価格などで、スクリプトなしにマップを動かせます。配線のしかたはほかのエンティティと同じです。数値は、`changed(value)`や`result(value)`のようなアウトプットの値として接続を流れ、自分のパラメーターを持たないインプットがそれを受け取ります。[パラメーターなし](../parameters.md#no-parameter)を参照してください。接続に入力したパラメーターは、値より優先されます。

整数は整数のままです。`2 + 3`は`5.0`ではなく`5`になり、`2.5`のようにテキストとして届いた数値も数値として扱われます。数値でないものは、すべてテキストとして扱われます。

## math_value { #math_value }

数値かテキストを一つ、変数のように保存します。ボタンが`add`で足し、`changed`がマップのほかの部分に現在の値を知らせます。`changed`は、値が以前と違うときだけ起動します。

* **インプット:** `set_value(value)`、`add(amount)`(デフォルトは1)、`get_value`、`reset`
* **アウトプット:** `value(value)`、`changed(value)`
* **キー:** `start_value` 0、数値かテキスト

`add`は二つの数値を足し、どちらかがテキストなら連結します。`hello`に`!`を足すと`hello!`になります。`get_value`は、保存されている値で`value`を起動します。変化を待たずに数値が欲しいときのためのものです。`reset`は`start_value`に戻します。値なしの`set_value`は、保存されている値をそのままにします。

## math_calc { #math_calc }

受け取った数値に計算を一つ行い、答えを起動します。何も保持しないので、同じ入力なら必ず同じ答えが出ます。いくつかつなげれば、段階を踏んで計算できます。

* **インプット:** `calculate(value)`。値がなければ0として扱います
* **アウトプット:** `result(value)`
* **キー:** `operation` add、`operand` 1

| `operation` | `result` |
| --- | --- |
| `add`、`subtract`、`multiply` | 値に`operand`を足した値、引いた値、掛けた値 |
| `divide` | 値を`operand`で割った値 |
| `modulo` | 割り算の余り。`operand`が正なら負にならないので、`-1`の`modulo` `3`は`2` |
| `power` | 値の`operand`乗 |
| `min`、`max` | 二つのうち小さいほう、大きいほう |
| `abs`、`negate` | 符号を取った値、符号を反転した値 |
| `round`、`floor`、`ceil` | 最も近い整数(.5は0から遠いほうへ)、切り捨て、切り上げ |

`abs`、`negate`、`round`、`floor`、`ceil`は`operand`を無視します。0での`divide`や`modulo`、負の数の平方根のように答えがないときは、何も起動せず、Godotが警告を一度だけ出力します。

## math_compare { #math_compare }

保存した値を`compare_value`と比べ、どちら側に入ったかを起動します。`on_not_equal`は、違いだけが重要なときのために、`on_less`と`on_greater`に並んで起動します。

* **インプット:** `set_value(value)`、`set_compare_value(value)`、`compare`、`set_and_compare(value)`
* **アウトプット:** `on_less`、`on_equal`、`on_not_equal`、`on_greater`
* **キー:** `compare_value` 0

`set_value`と`set_compare_value`は保存するだけで、`compare`は保存されている値を比べ、`set_and_compare`は両方を行います。ですから、`changed(value)`アウトプットに配線するのはこのインプットです。数値は値で比べるので、`10`は`9`より大きくなります。二つのテキストは辞書順で比べ、大文字が小文字より先になります。`0.1 + 0.2`と`0.3`のようにほぼ等しい小数は、等しいものとして扱われます。

## 例: スコアでドアを開ける { #example-a-score-that-opens-a-door }

コインのボタン三つとレバーで、コインが3枚以上入ったときに金庫のドアが開きます。

1. `score`という名前の`math_value`、`compare_value`が3の`enough`という名前の`math_compare`、`gate`が`and`の`door_gate`という名前の`logic_gate`を置きます。
2. 各コインのボタンの`pressed`を、`score`のインプット`add`に配線します。押すたびに1が加わります。
3. `score`の`changed`を、`enough`のインプット`set_and_compare`に配線します。新しいスコアが渡されます。
4. `enough`の`on_less`を、`door_gate`のインプット`set_a`にパラメーター`false`で配線します。`on_equal`と`on_greater`も同じようにパラメーター`true`で配線します。これでゲートの`a`は、コインが足りているかどうかを表します。
5. レバーの`pressed`を、`door_gate`のインプット`toggle_b`に配線します。
6. `door_gate`の`on_true`をドアのインプット`open`に、`on_false`をインプット`close`に配線します。

3枚目のコインが入ったときにレバーが引かれていると、ドアが開きます。順番が逆でも同じです。レバーを戻すかスコアをリセットすると閉まります。スコアを表示するには、`score`の`changed`を`game_text`のインプット`set_text`に配線します。コインではなくポイントで表示したければ、間に`math_calc`を入れてください。

ロジックのプレイグラウンドマップが、このホールを作っています。さらに、`logic_random`と`logic_case`でライトの抽選を、`logic_flipflop`でランプのスイッチを加えています。デモプロジェクトで`godot/demo/maps/logic_playground.tdm`を開くか、作り方を[`examples/mcp/logic_playground.json`](https://github.com/Paraxdev/TrenchDot/blob/main/examples/mcp/logic_playground.json)で読んでください。

<div class="td-mcp" markdown>

## MCP { #mcp }

上のスクリプトは、`gameplay place`でエンティティを一つずつ置いています。

```json
{ "tool": "gameplay", "args": { "op": "place", "classname": "math_value", "origin": [-280, 40, 40],
  "properties": { "targetname": "score" },
  "outputs": [ { "output": "changed", "target": "enough", "input": "set_and_compare" } ] } }
{ "tool": "gameplay", "args": { "op": "place", "classname": "math_compare", "origin": [-280, 40, -40],
  "properties": { "targetname": "enough", "compare_value": "3" },
  "outputs": [ { "output": "on_greater", "target": "door_gate", "input": "set_a", "parameter": "true" } ] } }
```

</div>
