---
source: 425ace012cd6d8410cbf64a89bb7dc31275f8980
---

# 配線のデバッグ { #debugging-wiring }

配線のミスは、たいてい静かに失敗します。ドアが閉まったままになるだけです。いちばん手早い確認から、いちばん徹底した確認まで、この順に調べてください。

> [!TIP]
> プレイヤーをふさぎすらしないドアやトリガーは、定義なしでビルドされています。これは配線のミスではありません。[何も起きないエンティティ](../godot/entities-troubleshooting.md)を参照してください。

## 1. 問題パネル { #1-the-issues-panel }

左のアウトライナーと履歴の隣にある**問題**タブを開きます。どのエンティティにもないtargetnameに向いた出力と、ターゲットのクラスにない入力が表示されます。ドアが閉まったままになる原因で、いちばん多いのは、targetnameのタイプミスです。

![問題パネルにエラーが1つ。trigger_onceの出力triggeredが、不明なエンティティ'hall_lihgt'を対象にしている。修正ボタンがある](../assets/debugging/issues-panel.png)

問題をクリックすると、そのエンティティが選択され、ビューに表示されます。そこで、インスペクターで名前を直してください。

> [!WARNING]
> 存在しないターゲットに対する**修正**は、正しい名前を推測せず、接続を削除します。タイプミスのときは、代わりに名前を手で直してください。

## 2. ロジックグラフ { #2-the-logic-graph }

[ロジックグラフ](logic-graph.md)では、誰も持っていないターゲットが、赤の破線のワイヤーでつながった赤いノードとして表示され、この名前のエンティティはないと示されます。

![ロジックグラフ。trigger_onceのtriggered出力が、赤の破線のワイヤーで赤いノードhall_lihgtにつながっている。この名前のエンティティはない](../assets/debugging/graph-missing-target.png)

**シミュレート**は、出力を紙の上で起動して、チェーン全体をハイライトします。[シミュレート](logic-graph.md#simulate)を参照してください。

## 3. ゲーム中のF3 { #3-f3-in-game }

Godotで、シーンに`TrenchDotDebugOverlay`ノードを追加するか、オートロードとして追加します。ゲームを実行してF3を押してください。起動した出力が順に一覧表示され、すべての`Area3D`ボリュームがtargetnameとともに描画されます。トリガーがまったく起動しなかったのか、何もない所に向けて起動したのかが分かります。

トリガーをスポーンしたあとは、もう一度切り替えてください。ボリュームは、オンにしたときに集められます。キーは、オーバーレイの`toggle_key`プロパティです。

## 4. ログ { #4-logging }

`logic_debug`エンティティを置いて、インスペクターで`trace_all`のボックスにチェックを入れます。起動した出力が、すべてGodotのOutputパネルに表示されます。

コードからは、`TrenchDotIO.events().fired`に接続します。これは、ターゲットが何にも解決されなかったものを含め、すべての出力で発せられます。

```gdscript
TrenchDotIO.events().fired.connect(func(source, output, target, input, parameter):
	print(source.name, ".", output, " -> ", target, ".", input))
```

## ミスごとの症状 { #what-each-mistake-looks-like }

| ミス | 症状 | 対処 |
| --- | --- | --- |
| ターゲットが何にも解決されない | 最初に起動したとき、`[TD I/O] <node>.<output>: target '<name>' matches no node` | ターゲットを、エンティティのtargetnameに、大文字小文字も含めて正確に合わせる |
| ターゲットにそのインプットがない | GodotのOutputパネルに`[TD I/O] <node> has no input '<name>'` | ターゲットのクラスにあるインプットを選ぶ。[エンティティリファレンス](entities/README.md)を参照 |
| エンティティが発しない出力 | マップをビルドしたとき一度だけ`[TD I/O] <node> has no signal '<output>'` | クラスにある出力を選ぶか、`TrenchDotIO.fire_output()`でコードから起動する |
| 実行時にスポーンした、または名前を変えたエンティティ | targetnameで見つからない | `TrenchDotIO.invalidate()`を呼ぶ。[実行時の変更](parameters.md#runtime-changes)を参照 |
