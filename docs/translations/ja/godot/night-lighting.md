---
source: 3df4b9a4462dd78307ebb473881223aa02ea1321
---

# 夜のライティング { #night-lighting }

夜のマップでは、グローバルライトを下げて、置いたランプが主役になるようにします。

## マップを暗くする { #darken-the-map }

Escapeを押すか、ビューの何もない場所をクリックして、何も選択されていない状態にします。すると右のインスペクターに、マップ全体の設定である*マップ (worldspawn)*が表示されます。

下のキーを設定します。まだ一覧にないキーは、いちばん下の*キー*欄に入力し、その隣に値を入れて、**追加**をクリックします。

| キー | デフォルト | 夜にするには |
| --- | --- | --- |
| `sun_energy`, `sun_color` | 1.1, `255 245 224` | エネルギーを下げて淡い青にすると、太陽が月に見えます |
| `ambient_energy`, `sky_energy` | 1.0, 1.0 | 両方を下げると、ライトの当たらない場所と空が暗くなります |
| `glow_intensity` | 0 | 0より大きくすると、Godotでランプと[発光するマテリアル](materials.md#glowing-materials)にブルームがかかります |
| `ssr` | 0 | 1にするとスクリーンスペースリフレクションがオンになり、濡れた通りやつやのある床がランプを映します |

夜の街区のショーケースが使っている設定は、次のとおりです。

![夜の街区のマップ (worldspawn)を表示したインスペクター。ambient_energy 0.8、glow_intensity 0.9、sky_energy 0.6、淡い青のsun_color、sun_energy 0.16](../assets/night-lighting/worldspawn-night.png)

## ライトプレビューで確認する { #check-it-in-lit-preview }

F3を押すか、ツールバーの「ライトプレビュー」をクリックすると、ライト、空、フォグを含めたマップを見られます。

![ライトプレビューのボタンにカーソルを重ねてツールチップを表示したツールバー。シェーディング: ライトプレビュー (F3)](../assets/night-lighting/toolbar-lit-preview.png)

ランプ、明かりのついた窓、光る看板が、暗い空を背景に浮かび上がるはずです。

![ライトプレビューでの夜の街区の通り。街灯、明かりのついた窓、暗い空の下で光るCorner Shopの看板](../assets/night-lighting/street-lit-preview.jpg)

ライトプレビューは、上のキーのうちグローと反射を除くすべてを表示するので、その2つはGodotで確認してください。ランプの回り込む光を、毎フレームのコストなしで得るには、[ライティングをベイクします](../editor/light-baking.md)。

## ランプの器具 { #lamp-fixtures }

ランプは、`light`または`light_spot`エンティティと、光る電球を含むランプのジオメトリでできています。そのジオメトリにtargetnameを付け、ライトの`fixture`キーに入れます(末尾の`*`は前方一致になります)。

すると、そのジオメトリは配線なしでライトに従い、ライトがオンのあいだは表示され、オフのあいだは隠れます。`fixture_off`を`dark`にすると、隠さずに表示したまま、発光だけをオフにします。

![targetnameがalley_lamp、fixtureがalley_bulb、fixture_offがdark (エミッションオフ)のライトのインスペクター](../assets/night-lighting/light-fixture-keys.png)

ほかのライトのキーは、[プロップ、ライト、エフェクト](../gameplay/entities/effects.md)を参照してください。

<div class="td-mcp" markdown>

## MCP { #mcp }

ショーケースのマップは、エディターを[MCPサーバー](../mcp.md)経由で操作する[MCPスクリプト](../mcp-scripts.md)で作られています。`examples/mcp/night_district.json`と`examples/mcp/withered_city.json`は、この方法で作られた夜のマップの完成版です。

</div>
