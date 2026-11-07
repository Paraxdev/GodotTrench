---
source: 5168ac40e48ba63850a1be1795dae420bfe6bf5f
---

# ロジック { #logic }

> [!NOTE]
> これらのエンティティは、ゲームプレイエンティティパックに含まれています。
> [インストールとカスタマイズ](README.md#installing-and-customizing)を参照してください。

いつ何を起こすかを決める、見えないポイントエンティティです。トリガーやボタンと、それが動かす相手のあいだに置いて、途中でシグナルの形を整えます。数値を扱うエンティティは[数値と計算](math.md)に、コードやアニメーションを動かすものは[スクリプトと呼び出し](scripting.md)にあります。

## logic_relay { #logic_relay }

有効なあいだ、`trigger`を`triggered`として先へ渡します。いくつものものを一つのリレーに配線しておき、`disable`すれば、連鎖全体をまとめて止められます。

* **インプット:** `trigger(activator)`、`enable`、`disable`、`toggle`
* **アウトプット:** `triggered(activator)`
* **キー:** `start_disabled` 0

## logic_branch { #logic_branch }

真か偽かを覚えておき、`test`を受けると`on_true`か`on_false`を起動します。「電源が入っているときだけドアが開く」といったチェックに使います。ジェネレーターが真にして、ドアのボタンがそれをテストします。

* **インプット:** `set_true`、`set_false`、`toggle`、`test`、`set_and_test(value)`
* **アウトプット:** `on_true`、`on_false`
* **キー:** `start_value` 0

## logic_gate { #logic_gate }

真偽の値を二つ、`a`と`b`として持ち、ゲートで組み合わせます。「電源が入っている**かつ**レバーが引かれているときにドアが開く」といったチェックに使います。インプットが変わって結果が入れ替わるたびに、`changed(on)`のあと`on_true`か`on_false`を起動するので、こちらから問い合わせる必要はありません。`test`は、いまの結果をもう一度起動します。ほかの何かがいますぐ結果を知りたいときのためのものです。

* **インプット:** `set_a(value)`、`set_b(value)`、`toggle_a`、`toggle_b`、`test`
* **アウトプット:** `on_true`、`on_false`、`changed(on)`
* **キー:** `gate` and、`start_a` 0、`start_b` 0

| `gate` | 真になるとき |
| --- | --- |
| `and` | 両方が真 |
| `or` | 少なくとも一方が真 |
| `xor` | ちょうど一方だけが真 |
| `nand` | 両方が真ではない |
| `nor` | どちらも真ではない |
| `xnor` | 両方が同じ |
| `not` | `a`が偽。`b`は無視される |

値なしの`set_a`と`set_b`は真にします。ですから、トリガーの`entered`アウトプットを`set_a`に配線すると、`a`がオンになります。数値は0以外なら真で、テキストは`true`、`yes`、`1`のときだけ真です。開始値では何も起動せず、結果がそのままになる変化でも起動しません。

## logic_flipflop { #logic_flipflop }

トリガーされるたびに切り替わるので、ボタン一つで何かをオンにしたりオフに戻したりできます。オンになるときに`on_a`、オフになるときに`on_b`を起動します。`reset`は、どちらも起動せずに`start_on`の状態へ戻します。それが変化だったときだけ、`changed`を起動します。

* **インプット:** `trigger`、`reset`
* **アウトプット:** `on_a`、`on_b`、`changed(on)`
* **キー:** `start_on` 0(オンなので、最初のトリガーで`on_b`を起動します)

## logic_case { #logic_case }

受け取った値を最大八つのケースと比べ、最初に一致したケースのアウトプットを起動します。どれも一致しなければ`on_default`です。`logic_random`のインデックスや`math_value`のスコアのような数値を、別々のイベントに変えます。

* **インプット:** `in_value(value)`
* **アウトプット:** `on_case_1`から`on_case_8`、`on_default`
* **キー:** `case_1`から`case_8`、空

数値は値で比べるので、ケース`2`は値2にも`2.0`にも一致します。テキストは完全に一致する必要があり、`Red`は`red`と同じではありません。ケースの前後の空白は無視されます。空のケースは飛ばされ、最初の一致が優先され、値がないときは`on_default`へ進みます。

## logic_random { #logic_random }

最大八つのアウトプットから一つをランダムに選ぶか、確率を判定します。`out_1`から`out_3`を三つのライトに配線しておくと、`pick`でそのうち一つが点きます。`roll`は代わりに確率で決めます。たとえば、宝箱にアイテムが入っているかどうかです。

* **インプット:** `pick`、`roll`
* **アウトプット:** `out_1`から`out_8`、`picked(index)`、`on_success`、`on_fail`
* **キー:** `count` 3(1から8)、`no_repeat` 0、`chance` 50(パーセント)

`pick`は、選んだアウトプットの番号を付けて`picked`を起動し、続けてそのアウトプットを起動します。ですから`picked`を[`logic_case`](#logic_case)につなげられます。`no_repeat`を付けると、`count`が1でない限り、前回選んだアウトプットは選びません。`roll`は`chance`パーセントの確率で`on_success`を、そうでなければ`on_fail`を起動します。0なら絶対に成功せず、100なら必ず成功します。

## logic_counter { #logic_counter }

`min`と`max`のあいだで数えます。「三つのボタンをすべて押す」といったパズルに使います。各ボタンが`add`を送り、`hit_max`がドアを開きます。値は範囲に収められ、変化がなければ何も起動しません。変化が上限か下限に届いたときに`hit_max`と`hit_min`を起動し、`reset`では`changed`だけを起動します。

* **インプット:** `add(amount)`(デフォルトは1)、`subtract(amount)`、`set_value(value)`、`reset`
* **アウトプット:** `hit_max`、`hit_min`、`changed(value)`
* **キー:** `min` 0、`max` 3、`start_value` 0

## logic_timer { #logic_timer }

`timer`を何度も起動します。ちらつくライトや敵の波のように、繰り返すものに使います。起動と起動のあいだに`interval`秒待ち、`start_on`が0でない限り、マップを読み込んだときから動き出します。`random_max`が`random_min`より大きいと、待ち時間はその範囲のランダムな値になり、繰り返しが機械的に感じられなくなります。`once`を付けると最初の起動で止まるので、単純な遅延になります。

* **インプット:** `start`、`stop`、`toggle`、`fire`(一度だけ、いますぐ)
* **アウトプット:** `timer`
* **キー:** `interval` 1.0、`random_min` 0、`random_max` 0、`start_on` 1、`once` 0

## logic_auto { #logic_auto }

マップのエンティティがすべて準備できたあとに、`map_spawn`を一度だけ起動します。音楽やイントロのテキストなど、最初から動かしたいものはここに配線します。

* **アウトプット:** `map_spawn`

## logic_sequence { #logic_sequence }

カットシーンのタイムラインです。番号付きのステップを最大八つ、順番に起動し、最初のステップも含めて、それぞれの前に待ちます。各ステップを、そのときに起こることに配線します。ドアが開く、テキストが出る、アクターが歩くなどです。

* **インプット:** `start`、`stop`、`reset`
* **アウトプット:** `step_1`から`step_8`、`step(index)`、`finished`
* **キー:** `steps` 3(1から8)、`interval` 1.0、`times` 空、`start_active` 0(マップの読み込みから動きます)、`loop` 0

`times`にはステップごとの待ち時間を`0 1 0.5`のように並べます。指定のないステップには`interval`が使われます。ループするシーケンスは`finished`を起動しません。`start`は動いているあいだは無視され、`stop`のあとはステップ1からやり直します。[カットシーンのチュートリアル](../../tutorials/cutscene.md)を参照してください。
