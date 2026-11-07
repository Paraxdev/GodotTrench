---
source: 5b25efdda7a0b559f9cff81871758a2f55e33a9c
---

# Exporter vers Blender et d'autres outils 3D { #exporting-to-blender-and-other-3d-tools }

Vous pouvez enregistrer l'aspect de votre carte dans un seul fichier de modèle 3D : brushes, maillages, terrain et
modèles, avec leurs textures à l'intérieur. Pratique pour faire le rendu d'un niveau dans Blender, préparer un plan de
bande-annonce, ou confier un blockout à un artiste qui construit par-dessus les maillages définitifs.

> [!WARNING]
> Seules la géométrie et les matériaux sont exportés. La logique des entités, le câblage d'E/S, les triggers, les scripts
> et le comportement du jeu sont laissés de côté, et le fichier ne peut pas être rouvert dans TrenchDot comme une carte.
> Le `.tdm` reste la carte, continuez à modifier celui-ci.

## Étapes { #steps }

1. Ouvrez l'export

    Allez dans *Fichier > Exporter > glTF Binary (.glb) pour Blender et d'autres outils 3D…*, ou tapez « glTF » dans la
    palette de commandes (Ctrl+Shift+P ou F1).

    ![Le menu Fichier avec Exporter ouvert et glTF Binary (.glb) pour Blender et d'autres outils 3D survolé](../assets/exporting/menu-export.png)

2. Choisissez ce qui est inclus

    La fenêtre *Exporter pour des outils 3D* s'ouvre. Cochez ce que vous voulez, voir [les options](#options) plus bas.
    Les nombres à côté de *Modèles de props* et *Instances de scatter* vous disent combien d'objets chacune ajoute. Vous
    pouvez passer le format en OBJ en haut.

    ![La fenêtre Exporter pour des outils 3D avec glTF Binary choisi, Carte entière, Modèles de props (5) coché et Instances de scatter (4081) décoché](../assets/exporting/export-dialog.png)

3. Enregistrez

    Cliquez sur **Exporter…** et choisissez où va le fichier. L'export tourne en arrière-plan avec une fenêtre de
    progression, donc vous pouvez continuer à modifier, et **Annuler** l'arrête sans rien écrire. Quand il est terminé,
    la barre d'état indique combien d'objets, de matériaux et de triangles sont sortis.

4. Ouvrez-le dans Blender

    Utilisez *File > Import > glTF 2.0 (.glb/.gltf)* et choisissez le fichier. Les réglages par défaut sont les bons,
    l'importeur met tout seul le modèle avec Z vers le haut.

Le fichier contient la carte telle qu'elle était quand vous avez cliqué sur **Exporter…**. Les modifications non
enregistrées sont d'abord sauvegardées automatiquement, sauf si la sauvegarde automatique est désactivée dans *Fichier >
Préférences*. Si la barre d'état nomme un matériau pour lequel le projet n'a pas d'image, celui-ci sort sans texture.

## Options { #options }

| Option | Effet |
| --- | --- |
| Carte entière, Sélection seule | Tout, ou seulement les objets sélectionnés, toujours dans leurs calques et leurs groupes |
| Calques et objets masqués | Exporte aussi ce qui est masqué dans les vues. Désactivé, vous obtenez ce que vous voyez |
| Modèles de props | Les modèles que montrent les props et les autres entités point, à leur place, avec leur rotation et leur échelle |
| Instances de scatter | Chaque arbre, rocher et touffe des [ensembles de scatter](scatter.md) comme son propre objet. Désactivé par défaut, car des milliers d'instances font un gros fichier lent à ouvrir |
| Entités ponctuelles comme marqueurs vides | Un objet vide nommé comme l'entité à l'endroit de chaque entité point sans modèle, pratique pour placer des lumières ou des caméras dans Blender |
| Fusionner les brushes sans nom | Les brushes que vous n'avez pas renommés se joignent en un seul objet par calque, groupe ou entité, ce qui garde l'outliner de Blender court. Les brushes nommés restent séparés |

## Ce qui est exporté { #what-is-exported }

| Dans la carte | Dans le fichier |
| --- | --- |
| Calques et groupes | Des objets parents vides avec les mêmes noms, pour que l'organisation de l'Outliner passe dans le fichier |
| Brushes | Un objet chacun, nommé comme dans l'Outliner, `brush12` ou le nom que vous avez donné |
| Entités brush comme les portes et les ascenseurs | Un objet parent nommé d'après l'entité qui contient ses brushes, là où ils se trouvent dans l'éditeur |
| Maillages, decals | Un objet chacun. Les decals reçoivent des matériaux mélangés, double face |
| Terrain | Un objet. Chaque triangle prend la texture de son calque peint le plus fort, donc les mélanges peints paraissent en escalier |
| Instances de prefab | Le contenu du prefab déplacé à sa place, sous un objet nommé d'après l'instance |
| Matériaux | Un par matériau avec sa texture d'albédo, plus les cartes de normales, de rugosité, de métallique, d'occlusion ambiante et d'émission qu'il a. Les matériaux découpés et transparents gardent leur alpha, le pixel art reste net. Les couleurs de dev intégrées sortent comme leurs textures de grille |

Le fichier laisse de côté ce que la construction Godot ne dessine pas : les faces avec des textures d'outil (clip, skip,
origin, sky, trigger, nodraw, hint), les faces cachées contre un autre brush, les triggers et autres entités de volume, et
les calques dont l'omission à l'export est activée. Les lumières, les sons, les points d'apparition, le ciel et le
brouillard ne sont pas exportés non plus.

Seuls les objets qui vont dans le fichier se cachent mutuellement leurs faces. Ainsi un pilier exporté avec *Sélection
seule* garde le dessous qui repose sur le sol, et un calque dont l'omission à l'export est activée ne laisse aucun trou chez
ses voisins.

## Unités et axes { #units-and-axes }

Un mètre dans Godot est un mètre dans Blender. Les unités de la carte sont divisées par les unités par mètre du projet, 32
sauf si les réglages de la carte disent autre chose. Le fichier est Y vers le haut comme Godot, ce qu'attend glTF. Chaque
brush, maillage et terrain a son origine au centre de ses limites, et les modèles gardent l'origine du fichier de modèle.

## Quand utiliser OBJ { #when-to-use-obj }

*Fichier > Exporter > Fichier Wavefront OBJ (.obj)…* sert aux outils qui ne savent pas lire glTF. Il écrit trois choses
côte à côte, par exemple `level.obj`, `level.mtl` avec les matériaux, et un dossier `level_textures` avec des copies des
textures. Gardez-les ensemble quand vous les déplacez. Dans Blender, utilisez *File > Import > Wavefront (.obj)*.

OBJ n'a pas d'objets parents ni de moyen de partager un maillage entre des copies, donc chaque objet est écrit à plat dans
l'espace monde et chaque instance de scatter est une copie complète. Avec les instances de scatter cochées, la fenêtre
avertit dès qu'elles totalisent un million de triangles et dit quelle taille le fichier atteint. Un export OBJ refuse plus
de 10 millions de triangles, un fichier de plus d'un gigaoctet. Les matériaux gardent les textures d'albédo, de normales,
de rugosité et d'émission. Préférez glTF chaque fois que l'autre outil le lit.

Un fichier `.glb` contient au plus 4 Go. glTF écrit chaque modèle une seule fois, quel que soit le nombre d'instances de
scatter qui l'affichent, donc seule une carte immense y arrive, et l'export le dit alors au lieu d'écrire un fichier
cassé.

## Ligne de commande { #command-line }

`trenchdot --export-glb <map> <out.glb>` et `--export-obj <map> <out.obj>` exportent avec les options par défaut sans
ouvrir de fenêtre. Le projet Godot de la carte est trouvé à partir de son dossier. Voir
[Ligne de commande](../command-line.md).

<div class="td-mcp" markdown>

## MCP { #mcp }

`map_file {op: export_glb, path}` écrit un `.glb` et `map_file {op: export_obj, path}` un `.obj` avec son `.mtl` et son
dossier de textures, à partir de la carte ouverte telle qu'elle est, enregistrée ou non. Les options sont des drapeaux :
`selection_only`, `hidden`, `scatter`, `models` (vrai sauf indication contraire), `markers` et `merge_brushes`. Le
résultat liste `objects`, `meshes`, `materials`, `images`, `triangles`, `bytes` et `missing_textures`. `run_action` refuse
`export_glb` et `export_obj`, puisque les entrées de menu ouvrent une boîte de dialogue de fichier.

</div>
