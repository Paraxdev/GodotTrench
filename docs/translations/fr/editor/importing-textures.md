---
source: 078f68d4f428db30dc2dfcb8561a8991ada14154
---

# Convertir des textures { #converting-textures }

Godot ne sait pas charger les fichiers `.vtf`, `.wad` ou `.wal`, donc l'éditeur les transforme en PNG, plus un `.tres`
quand une image simple ne peut pas porter les réglages. Chacun atterrit dans le dossier de textures du projet sous le nom
de matériau de la face, si bien que les faces importées le trouvent tout de suite.

Importer une carte propose cette conversion pour les textures qu'elle utilise, voir
[Importer des cartes Hammer et TrenchBroom](importing.md). Pour convertir un dossier entier, ouvrez d'abord le projet
Godot, puis :

1. Ouvrez le convertisseur

    Allez dans *Fichier > Importer > Convertir les textures (VTF/VMT, WAD, WAL)…*.

    ![Le menu Fichier avec Importer ouvert et Convertir les textures (VTF/VMT, WAD, WAL)… survolé](../assets/importing-textures/file-import-convert-textures.png)

2. Choisissez le dossier

    Choisissez le dossier qui contient les fichiers `.vmt` et `.vtf`, `.wad` ou `.wal`. Si la carte ouverte en manque
    certains, l'éditeur demande s'il faut convertir seulement **Celles de la carte** ou **Tout**.

    La barre d'état doit ensuite indiquer combien ont été converties, comme « Textures : 12 converted », et elles
    apparaissent dans l'onglet Matériaux sous leurs noms de matériau.

    ![L'onglet Matériaux recherché avec castle, montrant brick_wall et stone_step d'une carte importée, avec l'infobulle castle/brick_wall, 1024 x 1024 px](../assets/importing-textures/converted-in-materials.png)

## Où elles sont trouvées { #where-they-are-found }

L'éditeur cherche aux endroits habituels où chaque jeu range ses textures, donc une carte copiée avec les dossiers de son
jeu les trouve sans aucune configuration.

| Source | Cherchée | Nommée d'après |
| --- | --- | --- |
| `.vmt` et `.vtf` de Valve | un dossier `materials` à côté de la carte ou jusqu'à quatre dossiers plus haut | le chemin sous `materials`, comme `brick/brickwall001a` |
| `.wad` de Quake et de Half-Life | la clé `wad` de worldspawn, puis le même nom de fichier dans les dossiers proches | le nom de la texture, comme `{grate` |
| `.wal` de Quake 2 | un dossier `textures` proche | le chemin sous `textures`, comme `e1u1/floor1_1` |
| `.png`, `.jpg`, `.tga` en vrac | un dossier `textures` proche, comme les livrent les projets Qodot et FuncGodot | le chemin sous `textures`, copié tel quel |

## Matériaux Valve { #valve-materials }

Un fichier `.vmt` de Valve décrit un matériau comme une liste de paramètres. Ceux-ci passent dans le matériau Godot, le
reste est ignoré :

| Matériau Valve | Matériau Godot |
| --- | --- |
| `$basetexture` | albédo, son alpha supprimé sauf si le matériau est transparent |
| `$bumpmap`, ou `$normalmap` sur l'eau | carte normale, vert inversé de DirectX vers Godot |
| `$translucent`, `$additive` | mélange alpha pour `$translucent`, mélange additif pour `$additive` |
| `$alphatest`, `$alphatestreference` | Alpha Scissor à ce seuil, 0,5 par défaut |
| `$selfillum`, `$selfillummask` | une texture d'émission lumineuse, masquée par l'alpha de la texture de base ou par le masque |
| `$nocull`, `UnlitGeneric` | double face pour `$nocull`, non ombré pour `UnlitGeneric` |
| `$color` | couleur d'albédo |
| `$surfaceprop` | `metadata/surfaceprop` sur le matériau |
| `patch` avec `include` | un patch réutilise un autre matériau avec quelques changements, donc il devient le matériau inclus avec ses changements `insert` et `replace` appliqués |

## Textures Quake { #quake-textures }

Les textures qui commencent par `{` sont découpées avec Alpha Scissor, les ciels sont non ombrés, et `*water` est
enregistrée sous `water`. Les textures animées, y compris les VTF à plusieurs images, gardent leur première image.

## Ciels { #skies }

Convertir les textures d'une carte règle aussi son ciel, pour que la carte importée ait un ciel proche de l'original. Cela
est sauté quand la carte a déjà `sky_top_color`.

Une carte Source dont la skybox `skyname` est dans `materials/skybox` reçoit un panorama, `skybox/<skyname>_panorama.png`
dans le dossier de textures, réglé comme `sky_panorama`. Les autres cartes reçoivent `sky_top_color` et
`sky_horizon_color` à partir de la texture de `sky_source`. Quand la scène n'a pas de WorldEnvironment à elle (le nœud
Godot qui contient le ciel, le brouillard et la lumière ambiante d'une scène), `TrenchDotEnvironment` en construit un à
partir de ces clés.

## Formats VTF { #vtf-formats }

Le convertisseur lit les VTF des versions 7.0 à 7.5 dans ces formats de pixels : DXT1 (avec alpha sur un bit), DXT3, DXT5,
les ordres RGB et RGBA en 8 bits, I8, IA88, A8, RGB565, BGR565, BGRA4444, BGRA5551, BGRX5551, UV88, UVWQ8888, UVLX8888 et
les formats RGBA en 16 bits. Les autres formats, et le VTF 7.6, sont refusés avec le nom du format, pour que vous sachiez
quel fichier convertir à la main.

<div class="td-mcp" markdown>

## MCP { #mcp }

Via le [serveur MCP](../mcp.md), convertissez un dossier de textures avec `map_file {op: convert_textures, path}`.
Ajoutez `only_missing` pour ne convertir que ce qui manque à la carte ouverte, et `overwrite` pour remplacer les fichiers
déjà dans le projet.

</div>
