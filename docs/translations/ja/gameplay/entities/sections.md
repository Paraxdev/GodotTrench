---
source: 721efa7c5ececa362093af607d2e2c2faa92ad9b
---

# ストリーミングセクション { #streamed-sections }

> [!NOTE]
> これらのエンティティは、ゲームプレイエンティティパックに含まれています。
> [インストールとカスタマイズ](README.md#installing-and-customizing)を参照してください。

読み込み画面なしで、複数のマップを一つのワールドとして遊ぶためのエンティティです。マップ一つがセクションで、セクション同士は、両方のマップがまったく同じコピーを持つ接続の部屋でつながります。設定の手順は[セクションのストリーミング](../../godot/streaming.md#streaming-sections)で説明しています。

## info_landmark { #info_landmark }

接続の部屋の共有ポイントです。部屋の真ん中に置き、二つのコピーで同じ名前にします。次のセクションが入ってくるときは、そのランドマークが古いほうのランドマークとぴったり重なる位置に置かれます。

* **キー:** `targetname`

## func_section_stream { #func_section_stream }

接続の部屋の半分にあたる、見えないボリュームです。プレイヤーがその中に立っているあいだは、それが指定するセクションが読み込まれます。接続には、両側に一つずつ、二つ必要です。二つが接する線がセクションの入れ替わる場所なので、部屋のどちらの端も見えない場所に置いてください。たとえば、Z字の廊下の真ん中です。

* **キー:** `section`(この側のセクション。`res://levels/pools_2.tscn`のようなシーンパスか、ストリーマーの`sections_dir`にある名前)、`landmark`(部屋の`info_landmark`の`targetname`)

## TDSectionStreamer { #tdsectionstreamer }

エンティティではなく、ストリーミングを動かすノードです。ワールドのシーンに追加して、次を設定します。

| プロパティ | 内容 |
| --- | --- |
| `first_section` | ゲームが始まるセクション。シーンパスか名前 |
| `sections_dir` | 短い名前を探すフォルダー。`name.tscn`か`name.scn`として探します |
| `player_scene` | 最初のセクションの`info_player_start`にスポーンするプレイヤー |
| `player_path` | あるいは、シーンにすでにいるプレイヤー。どちらもないときは、セクションが持ってくる`player`グループの最初のノードが残されます |
| `recenter_distance` | ワールド全体を戻す前に、セクションの連なりが原点から離れてよいメートル数。8000 |

入れ替えのたびに、`section_changed(from, to)`を発行します。
