---
source: 602da12da979c292b377fb3a10cadcc22f1babd5
---

# C#のエンティティ { #c-entities }

C#のクラスは、`.tres`の定義なしで、配置できるエンティティになれます。アドオンがソースファイルを読み、シグナルはアウトプットに、exportしたメンバーはキーに、印を付けたメソッドはインプットになります。

C#のエンティティを動かすには、Godotの.NET版が必要です。エディターはどちらの版でもC#のエンティティを一覧に出しますが、Godotがビルドできるのは、.NET版で動いていて、C#プロジェクトがビルド済みで、クラスに`[GlobalClass]`があるときだけです。そうでないと、C#のブラシエンティティはコリジョンのない普通の`Node3D`として、ポイントエンティティは空の`Node3D`としてビルドされ、GodotのOutputパネルに`Node Class ... not found`と、この3つの確認事項が出ます。

## 1. ヘルパーを追加する { #1-add-the-helper }

**リファレンス**パネル(*表示 > パネル > リファレンス*)を開いて、左上の**C# ヘルパーをプロジェクトに追加**を押します。

![リファレンスパネルでC# ヘルパーをプロジェクトに追加にカーソルが重なっている。ツールチップにはTrenchDot.csを書き出す、C#のエンティティが使うブリッジ、と出ている](../assets/csharp/reference-csharp-helper.png)

アトリビュートを収めた`res://addons/func_godot/csharp/TrenchDot.cs`が書き出されます。これがないと、エンティティのコードはコンパイルできません。

## 2. エンティティを宣言する { #2-declare-an-entity }

```csharp
[GlobalClass]
[TrenchDotEntity("npc_guard", Description = "Patrolling guard",
    Color = "#ff4040ff", Size = "-16 0 -16 16 64 16")]
public partial class NpcGuard : CharacterBody3D
{
    [Signal] public delegate void AlertedEventHandler(Node activator, int level); // output "alerted"
    [Export] public float WalkSpeed { get; set; } = 3.5f;                          // key "walk_speed"
    [TrenchDotInput] public void Alert(Node activator) { }                        // input "alert"
}
```

| 部分 | 大切な理由 |
| --- | --- |
| `[GlobalClass]` | 必須です。マップのビルドは、クラス名でノードを作ります |
| `Size` | エディターがエンティティのために描くボックス。マップの単位で、最小値、最大値の順に書きます |
| `Solid = true` | ポイントエンティティではなく、ドアのようなブラシエンティティにします |
| `[TrenchDotInput]` | インプットに印を付けます。どのメソッドにも付いていなければ、名前が大文字で始まるpublicな`void`メソッドがすべてインプットになります |

名前は、マップではsnake_case、C#ではPascalCaseです。キーを自分で読むには、`_func_godot_apply_properties(Dictionary props)`を実装して、`TrenchDot.Float(props, "walk_speed", 3.5f)`のようなヘルパーのリーダーを使ってください。

> [!TIP]
> リファレンスパネルの*C# クラス*タブは、どのエンティティについても、そのシグナル、export、インプットを持つひな形を書き出します。自分のエンティティを手早く始められます。`[TrenchDotEntity]`の行は含まれないので、自分で足してください。

## アドオンが探す場所 { #where-the-addon-looks }

エディターとマップのビルドは、それぞれフォルダーをスキャンしてC#のエンティティクラスを探します。設定場所は2つあります。

| 使う側 | 設定 |
| --- | --- |
| エディター(ゲーム設定のエクスポート) | ゲーム設定リソースの*Csharp Source Dirs* |
| マップのビルド | プロジェクト設定の`trenchdot/csharp_entity_dirs` |

どちらも既定は`res://`で、`addons`、`bin`、`obj`、隠しフォルダーは飛ばします。

> [!NOTE]
> C#のクラスは、ライブラリのエンティティを置き換えられません。そのクラス名をFGDがすでに定義しているときは、FGDが優先されます。

## C#からのI/O { #io-from-c }

```csharp
var guide = TrenchDot.FindTargets(this, "guide")[0];
guide.Connect("finished", Callable.From(() => GD.Print("cutscene over")));

foreach (Node barrel in TrenchDot.FindTargets(this, "barrel"))
    TrenchDot.Invoke(barrel, "ignite", "", this);

TrenchDot.FireOutput(this, "alerted", player);
```

リファレンスパネルの*C# での使い方*タブは、どのエンティティについても、このコードを生成します。

<div class="td-mcp" markdown>

## MCP { #mcp }

[MCPサーバー](../mcp.md)でエディターを操作するエージェントやスクリプトは、`code_reference`ツールから、同じ生成コードを得られます。

</div>
