---
source: 078f68d4f428db30dc2dfcb8561a8991ada14154
---

# テクスチャを変換する { #converting-textures }

Godotは`.vtf`、`.wad`、`.wal`ファイルを読み込めないので、エディターがそれをPNGに変換します。普通の画像では設定を持たせられないときは、`.tres`も一緒に作ります。それぞれ、面のマテリアル名でプロジェクトのテクスチャフォルダに入るので、インポートした面はすぐにそれを見つけます。

マップをインポートすると、そのマップが使うテクスチャについてこの変換が提案されます。[HammerとTrenchBroomのマップをインポートする](importing.md)を参照してください。フォルダ全体を変換するには、先にGodotプロジェクトを開いてから、次の手順を行います。

1. コンバーターを開く

    *ファイル > インポート > テクスチャを変換 (VTF/VMT, WAD, WAL)…*を選びます。

    ![インポートを開いてテクスチャを変換 (VTF/VMT, WAD, WAL)…にカーソルを合わせたファイルメニュー](../assets/importing-textures/file-import-convert-textures.png)

2. フォルダを選ぶ

    `.vmt`と`.vtf`、`.wad`、`.wal`ファイルがあるフォルダを選びます。開いているマップにそのいくつかが足りないときは、**マップ分のみ**と**すべて**のどちらを変換するかをエディターが尋ねます。

    するとステータスバーに「テクスチャ：12個を変換しました」のように変換した数が表示され、それらはマテリアルタブにマテリアル名で現れます。

    ![マテリアルタブでcastleを検索して、インポートしたマップのbrick_wallとstone_stepが表示されている。ツールチップはcastle/brick_wall, 1024 x 1024 px](../assets/importing-textures/converted-in-materials.png)

## 見つかる場所 { #where-they-are-found }

エディターは、各ゲームがテクスチャを置く通常の場所を探します。そのため、ゲームのフォルダごとコピーしたマップなら、設定なしでテクスチャが見つかります。

| 元 | 探す場所 | 付く名前 |
| --- | --- | --- |
| Valveの`.vmt`と`.vtf` | マップの隣、または最大4つ上のフォルダにある`materials`フォルダ | `materials`の下のパス。たとえば`brick/brickwall001a` |
| QuakeとHalf-Lifeの`.wad` | worldspawnの`wad`キー。次に、近くのフォルダにある同じファイル名 | テクスチャ名。たとえば`{grate` |
| Quake 2の`.wal` | 近くにある`textures`フォルダ | `textures`の下のパス。たとえば`e1u1/floor1_1` |
| 単体の`.png`、`.jpg`、`.tga` | QodotやFuncGodotのプロジェクトが同梱するのと同じ、近くにある`textures`フォルダ | `textures`の下のパス。そのままコピーされます |

## Valveのマテリアル { #valve-materials }

Valveの`.vmt`ファイルは、マテリアルをパラメーターの一覧として記述します。次のものはGodotのマテリアルに引き継がれ、残りは省かれます。

| Valveのマテリアル | Godotのマテリアル |
| --- | --- |
| `$basetexture` | アルベド。マテリアルが透けて見えるものでない限り、アルファは捨てられます |
| `$bumpmap`、または水の`$normalmap` | 法線マップ。緑のチャンネルはDirectXからGodotの向きに反転されます |
| `$translucent`、`$additive` | `$translucent`はアルファブレンド、`$additive`は加算ブレンド |
| `$alphatest`、`$alphatestreference` | そのしきい値でのアルファシザー。デフォルトは0.5 |
| `$selfillum`、`$selfillummask` | 光る発光テクスチャ。ベーステクスチャのアルファ、またはマスクでマスクされます |
| `$nocull`、`UnlitGeneric` | `$nocull`は両面、`UnlitGeneric`はシェーディングなし |
| `$color` | アルベドの色 |
| `$surfaceprop` | マテリアルの`metadata/surfaceprop` |
| `include`付きの`patch` | パッチは、少しの変更を加えて別のマテリアルを再利用するものなので、`insert`と`replace`の変更を適用した、インクルード元のマテリアルになります |

## Quakeのテクスチャ { #quake-textures }

`{`で始まるテクスチャはアルファシザーで切り抜かれ、スカイはシェーディングなしになり、`*water`は`water`として保存されます。複数フレームのVTFを含むアニメーションテクスチャは、最初のフレームだけが残ります。

## スカイ { #skies }

マップのテクスチャを変換すると、そのスカイも設定されるので、インポートしたマップには元に近いスカイが付きます。マップにすでに`sky_top_color`があるときは、この設定は行われません。

`materials/skybox`にある`skyname`のスカイボックスを持つSourceのマップには、テクスチャフォルダの`skybox/<skyname>_panorama.png`というパノラマが作られ、`sky_panorama`に設定されます。それ以外のマップは、`sky_source`のテクスチャから`sky_top_color`と`sky_horizon_color`が設定されます。シーンに独自のWorldEnvironment（シーンのスカイ、フォグ、環境光を持つGodotのノード）がないときは、`TrenchDotEnvironment`がこれらのキーからそれを作ります。

## VTFの形式 { #vtf-formats }

コンバーターは、バージョン7.0から7.5のVTFを、次のピクセル形式で読み込みます。DXT1（1ビットのアルファ付き）、DXT3、DXT5、8ビットのRGBとRGBAの各並び順、I8、IA88、A8、RGB565、BGR565、BGRA4444、BGRA5551、BGRX5551、UV88、UVWQ8888、UVLX8888、16ビットのRGBA形式です。それ以外の形式とVTF 7.6は、形式名を示して拒否されるので、どのファイルを手作業で変換すればよいかが分かります。

<div class="td-mcp" markdown>

## MCP { #mcp }

[MCPサーバー](../mcp.md)からは、`map_file {op: convert_textures, path}`でテクスチャのフォルダを変換できます。`only_missing`を付けると、開いているマップに足りないものだけを変換し、`overwrite`を付けると、プロジェクトにすでにあるファイルを置き換えます。

</div>
