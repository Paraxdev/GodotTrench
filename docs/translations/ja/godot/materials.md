---
source: 6f3b7a09923ac7b7e6e7e7d49ae11666c6d47d02
---

# マテリアル { #materials }

どの面も、マップ設定の*Base Texture Dir*の中でのパスで、拡張子を除いた形(たとえば`showcase/planks`)でマテリアルを指します。

Godotがマップをビルドするとき、そのフォルダの中、または*Base Material Dir*を設定してあればそちらから`showcase/planks.tres`を探し、見つかればそのまま使います。`.tres`がなければ、アドオンが画像から単純なマテリアルを作ります。TrenchDotは同じ`.tres`をプレビューするので、どちらも同じ見た目になります。

`.tres`が必要になるのは、画像だけでは足りないときだけです。たとえばスケールを変える、光らせる、透かすといった場合です。`albedo_color`だけでテクスチャのない`.tres`も使えて、エディターはその色で描画します。

新しいテクスチャやマテリアルは、数秒以内にマテリアルタブに現れます。再読み込みは要りません。

## テクスチャサイズ { #texture-size }

デフォルトでは、テクスチャの1ピクセルがマップユニット1つ分になるので、64ピクセルのテクスチャは2メートルごとに繰り返されます。ピクセルアートには最適ですが、2メートルの壁を撮った1024ピクセルの写真だと、32メートルにも引き伸ばされてしまいます。

これを直すには、1回の繰り返しが何マップユニット分かをマテリアルに伝えます。

```
[resource]
albedo_texture = ExtResource("1_albedo")
metadata/texture_size = Vector2(64, 64)
```

(Godotのインスペクターの*Metadata*から、`Vector2`または単一の数値として追加することもできます。)

確認するには、マテリアルタブでマテリアルにカーソルを重ねます。ツールチップに、画像サイズと、1回の繰り返しが何ユニットになるかが表示されます。

![マテリアルタブでのnight/shop_signのツールチップ。1024 x 256 px、1回の繰り返しが256 x 64ユニット、emissive x3](../assets/materials/tooltip-texture-size.png)

これで、マップを変えずに、よりシャープな画像に差し替えられます。地形レイヤーはこの設定を無視します。繰り返しは`tile`の設定がすでに決めているからです。

## 発光するマテリアル { #glowing-materials }

マテリアルのファイルで`emission_enabled`を設定すると、そのマテリアルは光ります。エミッションの色が黒なら、エミッションテクスチャは描かれたとおりに光ります。

```
[resource]
albedo_texture = ExtResource("1_albedo")
emission_enabled = true
emission = Color(0, 0, 0, 1)
emission_energy_multiplier = 2.5
emission_texture = ExtResource("2_emission")
```

マテリアルのファイルがないときは、アルベドと同じ名前に`_emission`を付けた画像を隣に置きます。`night/sign.png`の隣に`night/sign_emission.png`を置くと、それが発光マップになります。

発光するマテリアルには、マテリアルタブで黄色の*グロー*タグが付きます。

![マテリアルタブのnightフォルダ。apartment_lit、neon_red、shop_signに黄色のグローのタグが付いている](../assets/materials/glow-tags.png)

F3でライトプレビューにすると、3Dビューで光る様子を確認できます。周りのブルームはGodotでしか表示されません。[夜のライティング](night-lighting.md)を参照してください。

## 透明 { #transparency }

テクスチャのアルファチャンネルで向こう側を透かしたいときは、マテリアルのファイルで`transparency`を設定します。モードごとに、品質とコストのトレードオフが異なります。

| `transparency` | 仕組み | 向いている用途 | 注意点 |
| --- | --- | --- | --- |
| `1` blend | 面を背後のものと混ぜる | ガラス、やわらかい汚れ、煙 | 影を落とさない |
| `2` scissor | ピクセルを完全に表示するか、切り取るか | 格子や葉のような、近くで見る切り抜き | 細い部分が遠くで消える |
| `3` hash | ピクセルをディザリングし、アルファが低いほど表示されるピクセルが減る | 金網のフェンスのような、遠くから見る細い切り抜き | やわらかい縁がノイズになる |

どのモードを使っているかは、マテリアルタブのツールチップで分かります。

![マテリアルタブでのnight/chainlinkのツールチップ。alpha hash、法線マップ、両面](../assets/materials/tooltip-alpha-hash.png)
