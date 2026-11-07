---
source: 414dc5a3711ba57511bd4ecc18dc19f159af44d7
---

# I/Oチートシート { #io-cheatsheet }

[エンティティI/Oのしくみ](io.md)と[ターゲット、パラメーター、インプット](parameters.md)を1ページにまとめたものです。考え方は分かっていて、細かいところだけ知りたいときに使ってください。

## アウトプットが起動するとき { #when-an-output-fires }

シグナルが発せられ、*回数*に数えられ、*遅延*だけ待ってから、ターゲットが見つけたすべてのノードでインプットが呼ばれます。何も見つからなければ何も起きず、何も見つからなかったtargetnameやノードパスは、一度だけ警告します。

## ターゲット { #targets }

| 書き方 | 届く先 |
| --- | --- |
| `main_door` | 同じマップ内で、そのtargetnameを持つすべてのエンティティ |
| `door_*` | `door_`で始まるすべてのtargetname |
| `@enemies` | ツリー内のどこにあっても、そのGodotグループのすべてのノード |
| `!player` | グループ`player`の最初のノード |
| `!activator` | チェーンを始めた存在。`!caller`も同じ意味 |
| `!self` | そのアウトプットを持つエンティティ |
| `/root/Game` | その絶対パスにあるノード |

## パラメーター { #parameters }

ふつうの値は、int、float、boolの順に試され、そのあと`x y z`が`Vector3`として試されます。どれでもなければ文字列のままです。JSONの`[a, b]`は複数の引数に展開されます。`$activator`、`$self`、`$position`、`$caller_name`は、**受け取る**側のエンティティで埋められます。

パラメーターがないときは、アクティベーターが最初のオブジェクト引数に入り、アウトプットが運んだほかの値が、残りに順に入ります。

## アクティベーターを保つもの { #who-keeps-the-activator }

| シグナル | アクティベーター |
| --- | --- |
| `triggered`、`pressed`、`entered`、`exited`、`hurt`、`teleported`、`pushed` | そのまま引き継ぐ |
| `arrived(corner)`、`spawned(node)`、`reached(train)` | 括弧内のノードに置き換わる |
| `opened`、`timer`、`step_N`、`on_true`、`on_case_N`、`result`、`broken`、`switched`、`map_spawn` | 失われる |

チェーンの途中で失われてプレイヤーが必要なときは、`!player`をターゲットにしてください。

## どのノードにもあるインプット { #inputs-every-node-has }

`kill`、`show`、`hide`、`enable`、`disable`、`toggle`は、エンティティにその名前のメソッドがないかぎり、どのノードでも使えます。プロパティと同じ名前のインプットは、そのプロパティをパラメーターの値にします。

## logic_scriptのスコープ { #logic_script-scope }

[`logic_script`](entities/scripting.md)は、起動されるとGDScriptの断片を実行します。その中で、`this`はlogic_script自身、`activator`はチェーンを始めたノード、`parameter`は渡された値、`io`は`TrenchDotIO`のヘルパー、`tree`は`SceneTree`です。インデントはタブで付けてください。

```gdscript
var p = io.find_targets(this, "!player", null)
if not p.is_empty():
	p[0].take_damage(25, this)
```

## 何も起きないとき { #nothing-happened }

問題パネル、ロジックグラフ、ゲーム中のF3の順に確認して、それから`TrenchDotIO.events()`をログに出してください。[配線のデバッグ](debugging.md)を参照してください。
