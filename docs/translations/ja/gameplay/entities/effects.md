---
source: 80f0288013e71dba5bd67c791a5e3e3272a71653
---

# プロップ、ライト、エフェクト { #props-lights-and-effects }

> [!NOTE]
> これらのエンティティは、ゲームプレイエンティティパックに含まれています。
> [インストールとカスタマイズ](README.md#installing-and-customizing)を参照してください。`light`、`light_spot`、プローブ
> (`light_probe`、`env_reflection_probe`、`env_voxel_gi`)はアドオンのコアの一部なので、パックなしでも使えます。

## prop_physics { #prop_physics }

投げたり、ダメージを与えたり、壊したりできる木箱や樽です。体力が0になったとき、`impact_speed`より速い衝撃を受けたとき、`smash`を受けたとき、`ignite`から`fuse`秒がたったときに壊れます。爆発するプロップは、壊れると爆発します。

| キー | デフォルト | 働き |
| --- | --- | --- |
| `model` | | 見た目になるシーン。見た目だけに使います |
| `size` | 16 16 16 | ボックスコリジョンの半分の大きさ。デフォルトで幅1メートルです |
| `collision` | default | defaultとboxは`size`を使い、convex hullとconvex decompositionはモデルに沿います。[コリジョン](collision.md)を参照してください |
| `collision_layer`、`collision_mask` | 空 | `1, 3`のようなレイヤー番号。空ならプロジェクトのデフォルトのままです |
| `mass` | 5.0 | 質量(キログラム) |
| `health` | 30 | 壊れるまでに耐えられるダメージ |
| `explosive` | 0 | 壊れるときに爆発します |
| `explosion_radius`、`explosion_damage` | 192、40 | その爆発の大きさと強さ |
| `fuse` | 0.6 | 火をつけられたプロップが、壊れるまで燃える秒数 |
| `impact_speed` | 0 | これより速い(m/s)速度で何かにぶつかると壊れます。0なら衝撃では壊れません |
| `debris_scene` | | 壊れた場所にスポーンするシーン。板切れなど |

* **インプット:** `smash`、`ignite(activator)`、`push(direction)`、`take_damage(amount, source)`
* **アウトプット:** `damaged(hp)`(残りの体力が付きます)、`broken`

`push`は、1秒あたりのマップ単位の速度を受け取ります。`broken`は爆発のあとに起動するので、それが起動するものは、すでにダメージを見ています。

プロップは、シーンを読み込むたびにモデルとコリジョンのボックスを作ります。Godotのエディターではプレビューとして表示されますが、シーンには保存されません。

## prop_model { #prop_model }

必要ならコリジョン付きの静的モデルです。家具や岩など、動かないプロップに使います。

* **キー:** `model`(.bbmodel、.glb、.gltf、.tscn)、`model_node`、`collision`、`collision_layer`、
  `collision_mask`、`scale` 1.0

`collision`がdefaultなら、メッシュごとに凸包が付きます。箱型の形には十分です。アーチのように、プレイヤーが通り抜けたり上を歩いたりする形には、convex decompositionかtrimeshを選びます。いちばん軽い形状が欲しいときはboxです。[コリジョン](collision.md)を参照してください。コリジョンが生成されるのは、モデルが自分では持っていないときだけです。*モデルプロップ*で置いたモデルと、プレハブの中のモデルは、prop_modelエンティティなので、同じように衝突します。バリエーションを並べて一つのファイルに入れてあるときは、`model_node`で指定したノードだけを残し、プロップの原点へ移します。

## env_explosion { #env_explosion }

壁に仕掛けた爆薬のような、台本どおりの爆発です。`explode`を受けると、`radius`内のすべてのノードの`damage_method`を、`damage * (1 - distance / radius)`の量で呼び、剛体を押しのけ、閃光と爆音のために`effect_scene`をスポーンします。

* **インプット:** `explode(activator)`
* **アウトプット:** `exploded`
* **キー:** `radius` 256、`damage` 50、`force` 8(中心でのm/s。質量には関係しません)、`damage_method` take_damage、
  `effect_scene`

距離は各ノードの原点まで測り、視線のチェックはしないので、壁越しにもダメージが入ります。

## game_text { #game_text }

ワールドの中、HUD、またはその両方に、テキストを1行表示します。ヒント、看板、カットシーンの字幕に使います。

| キー | デフォルト | 働き |
| --- | --- | --- |
| `text` | | 表示するテキスト |
| `place` | world | `world`はエンティティの位置に浮かび、`hud`は画面に表示し、`both`は両方を行います |
| `start_visible` | 0 | マップの読み込みから表示します |
| `duration` | 0 | その秒数のあとに隠します。0なら`hide`まで出したままです |
| `text_color` | 255 255 255 | テキストの色 |
| `world_size` | 0.01 | ワールドのラベルの大きさ。1ピクセルあたりのメートル数です |
| `billboard` | 1 | ワールドのラベルがカメラのほうを向き、壁越しにも描画されます。看板にするときはオフにします。そうするとエンティティの角度のままになり、壁の裏では隠れます |

* **インプット:** `show`、`hide`、`set_text(text)`、`flash(seconds)`
* **アウトプット:** `shown`、`hidden`

`show`と`hide`は、ノードではなくラベルを切り替えます。`flash`は、パラメーターの秒数だけ表示し、なければ`duration`、それもなければ2秒です。

## light と light_spot { #light-and-light_spot }

インプットでオンとオフを切り替えられる、全方向ライトとスポットライトです。電灯のスイッチ、警報、ちらつくランプに使います。`switched`は、状態が実際に変わったときだけ起動します。アドオンのコアの一部なので、どのプロジェクトでも使えます。

* **インプット:** `turn_on`、`turn_off`、`toggle`
* **アウトプット:** `switched(on)`
* **キー:** `light_energy` 1.0(スポットは1.5)、`light_color` 255 255 255、`start_on` 1、`shadows` 0(描画の負荷が増えます)、`fixture`、`fixture_off` hide、
  そして`omni_range` 10、または`spot_range` 15と`spot_angle` 35(円錐の半分の角度、単位は度)、`bake_mode` auto、`light_size` 0、`light_indirect_energy` 1、
  さらに`omni_attenuation` 1、または`spot_attenuation` 1と`spot_angle_attenuation` 1。最後のキーは、同じ名前のGodotのライトのプロパティを設定し、ベイクでも使われます

**フィクスチャ**は、ランプのモデルをライトと連動させます。ランプの光る部分のジオメトリにtargetnameを付けて、`fixture`に入れてください。末尾の`*`は前方一致なので、`hall_tube_*`はすべてのチューブにあたります。ライトがオンのあいだは表示され、オフのあいだは隠れます。配線は要りません。`fixture_off`を`dark`にすると、表示されたままエミッションだけが消えます。`fixture`は、ほかの[ターゲット](../parameters.md#targets)の形式も受け付けます。ノードグループなら`@lamps`、ノードパスなら`/root/...`です。エディターは、ノードパスを除くすべての形式をプレビューします。ノードパスを解決できるのはGodotだけです。

`bake_mode`と`light_size`は、[ライティングをベイク](../../editor/light-baking.md#light-keys)がそのライトをどう扱うかを決めます。`auto`のとき、I/Oで切り替えられるライト、つまりtargetnameがあるか`start_on`が0のライトは、リアルタイムのままになります。

## env_sound { #env_sound }

レベルの中の一点から鳴る音です。警報、機械、イベントに配線した衝突音などに使います。

* **インプット:** `play`、`stop`、`toggle`
* **アウトプット:** `finished`
* **キー:** `sound`(`res://`のオーディオファイル)、`volume_db` 0、`max_distance` 0(無制限)、`autoplay_on` 0(マップの読み込みから再生します)、
  `loop_sound` 0

## env_particles { #env_particles }

炎、煙、火花、ほこり、雨のためのパーティクルエフェクトです。見た目は組み込みなので、自分で作る必要はありません。`burst`は一回分を放出し、そのあとは動いているエフェクトが続きます。

* **インプット:** `start`、`stop`、`toggle`、`burst`
* **アウトプット:** `finished`
* **キー:** `amount` 32(同時に生きているパーティクルの数)、`lifetime` 2.0、`one_shot` 0、`start_emitting` 0、`effect` rise、
  `color` 255 255 255、`size` 0、`blend` auto、`area` 10 1 10

`effect`は、`rise`(光るほこりや燃えさし)、`fire`、`smoke`、`sparks`、`rain`のどれかです。`rain`は、幅、高さ、奥行きが`area`メートルの箱から降ります。`color`は色味を付け、白ならエフェクト本来の色のままです。`size`はパーティクルの大きさ(メートル)で、0ならエフェクト本来の大きさです。`blend`は、光らせるなら`add`、煙なら`mix`です。各エフェクトのデフォルトだけで見た目が整うので、まずは`effect`だけで始めてください。

## infodecal { #infodecal }

投影デカールです。中にあるものすべてにマテリアルを映す箱で、階段、パイプ、でこぼこの地面の上にもまたがる汚れに使います。置き方と気をつけることは、[投影デカール](../../editor/projected-decals.md)で説明しています。

* **キー:** `material`(`base/stain`のような名前か、`res://`のパス)、`texture`、`normal_texture`、`size` 64 32 64、
  `modulate` 255 255 255

`material`はテクスチャと色を伴います。`texture`と`normal_texture`は、マテリアルなしで使う`res://`の画像です。`size`は、箱の幅、奥行き、高さで、マップ単位です。64 32 64のままなら、代わりにマテリアルの[テクスチャサイズ](../../godot/materials.md#texture-size)が幅と高さを決めます。`modulate`はデカールに色味を付けます。

## light_probe、env_reflection_probe、env_voxel_gi { #light_probe-env_reflection_probe-and-env_voxel_gi }

動くオブジェクトを照らし、映り込ませるプローブです。[Godotでのベイクしたライティング](../../godot/baked-lighting.md)を参照してください。どれもインプットとアウトプットはありません。

| エンティティ | 内容 | キー |
| --- | --- | --- |
| `light_probe` | ベイクにライトプローブを一つ加えるポイント。プローブのグリッドが届かない場所に使います | なし |
| `env_reflection_probe` | ブラシの大きさの`ReflectionProbe`になるブラシボリューム | `intensity` 1.0、`update` once、`box_projection` 1、`interior` 0、`max_distance` 0 |
| `env_voxel_gi` | ブラシの大きさの`VoxelGI`になるブラシボリューム。ビルドのたびにGodotのエディターでベイクされます | `subdiv` 128、`energy` 1.0、`bake_on_build` 1 |

`update`を`always`にすると、動くオブジェクトのために反射プローブを毎フレーム取り込みますが、負荷が大きくなります。`box_projection`は、反射を部屋の壁に合わせ、`interior`は、窓のない部屋のために空を除き、`max_distance`は、プローブがボックスの外を何メートル先まで見るかです。`subdiv`は、最長の辺に沿ったボクセル数で、64、128、256、512のどれかです。大きいほど鮮明になりますが、ベイクが遅くなります。
