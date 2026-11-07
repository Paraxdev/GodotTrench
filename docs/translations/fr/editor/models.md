---
source: d2dc61c8ee306263eda3f36520611e0790a27732
---

# Modèles { #models }

Les modèles sont des fichiers 3D faits dans un autre programme, comme Blender ou Blockbench. TrenchDot lit les fichiers
glTF, OBJ, Blockbench, STL, MD2 et MD3.

Vous les trouvez dans l'onglet **Modèles** en bas, à côté de Matériaux et de Prefabs. Il liste tout ce qui se trouve sous
`res://models`, plus les modèles de nature de `res://trenchdot/nature` une fois que vous les avez installés.

![L'onglet Modèles : la zone de recherche, les filtres de dossier et de source, Trier, Slugs, le curseur de taille des vignettes, Ajuster, le multiplicateur de taille, le bouton de réanalyse et Importer…, au-dessus d'une grille de vignettes de modèles de nature](../assets/models/models-panel.png)

Un nouveau projet démarre sans modèles. *Godot > Ajouter du contenu au projet…* ajoute les modèles de nature, et le choix
de la démo apporte aussi les accessoires Poly Haven dans `res://models/polyhaven`. Ou copiez vos propres fichiers dans
`res://models` et appuyez sur le bouton ⟳ pour réanalyser.

## Placer un modèle { #placing-a-model }

Un modèle entre dans la carte de deux façons. Choisissez selon que vous voulez ou non changer sa forme dans l'éditeur.

| Placé comme | Stocké dans la carte | À utiliser pour |
| --- | --- | --- |
| Maillage modifiable | Une copie de la géométrie, qui ne suit plus le fichier | Les formes que vous voulez modifier avec les [outils de maillage](meshes.md) |
| Entité `prop_model` | Seulement un chemin `res://`, Godot charge le fichier | Les accessoires que vous gardez tels quels. Chaque copie suit le fichier quand vous le changez |

### Comme maillage modifiable { #as-an-editable-mesh }

1. (Facultatif) Régler la taille

    Ajuster, dans la barre d'outils Modèles, est coché par défaut. Il met à 2 mètres un modèle dont le plus grand côté fait
    moins d'un quart de mètre ou plus de 256 mètres, pour que des fichiers minuscules ou gigantesques arrivent quand même
    utilisables. Le champ × à côté multiplie la taille en plus de cela. Les deux s'appliquent au moment du placement,
    réglez-les donc avant de faire glisser.

2. Le faire glisser dans une vue

    Faites glisser une vignette de l'onglet Modèles dans n'importe quelle vue. Elle se pose sur la surface sous le pointeur.

    ![Un tonneau de vin glissé de l'onglet Modèles dans la vue 3D, où il se pose sur le sol comme maillage modifiable](../assets/models/drag-model-into-view.gif)

Pour un fichier situé n'importe où sur le disque, utilisez **Importer…** à la fin de la barre d'outils Modèles.

### Comme prop { #as-a-prop }

1. Choisir le fichier

    Allez dans *Fichier > Importer > Prop modèle (.bbmodel, .glb)…* et choisissez le modèle.

    ![Le menu Fichier avec Importer ouvert et Prop modèle (.bbmodel, .glb)… survolé](../assets/models/file-import-model-prop.png)

2. Vérifier l'Inspecteur

    Il se pose au curseur 3D comme entité `prop_model`, et sa clé `model` contient le chemin `res://`.

    ![L'Inspecteur d'une entité prop_model avec la clé model réglée sur un chemin res://models/polyhaven](../assets/models/inspector-prop-model.png)

Un prop doit pointer vers un fichier que Godot sait charger. Choisissez-en un hors du projet et l'éditeur propose de le
copier d'abord dans `res://models`.

Les modèles Blockbench ont deux autres voies d'entrée, *Fichier > Importer > Modèle Blockbench en maillage…* et *Modèle
Blockbench en brushes…*. En brushes, chaque cube devient un brush, groupé sous le nom du fichier.

## Textures d'un maillage placé { #textures-of-a-placed-mesh }

Un maillage modifiable garde l'aspect du modèle. Ses textures sont copiées dans `models/<name>` dans le dossier de
textures du projet, où `<name>` est le chemin du modèle dans l'onglet Modèles, ou son nom de fichier pour un modèle venu
d'ailleurs. Sélectionnez le maillage et l'Inspecteur liste les matériaux avec lesquels il est dessiné :

![L'Inspecteur d'un maillage placé : 5966 sommets et le matériau models/polyhaven/containers/wine_barrel_01/tex0](../assets/models/inspector-mesh-materials.png)

Là où la simple image ne se dessinerait pas comme le modèle, un matériau Godot est aussi ajouté au dossier de matériaux du
projet. Ainsi les feuilles découpées, le pixel art et les parties lumineuses (couleur et carte d'émission d'un modèle glTF
comprises) se dessinent dans Godot comme ici. Pour changer les matériaux, cliquez sur l'un d'eux dans l'onglet Matériaux
pendant que le maillage est sélectionné, *Édition > Annuler* les ramène.

Placer n'écrase jamais un fichier. Placez le même modèle encore une fois et il réutilise ses textures, et garde un
matériau que vous avez réglé dans Godot. Si les textures d'un autre modèle se trouvent déjà dans ce dossier, comme celles
de `rock.bbmodel` à côté de `rock.glb`, les nouvelles vont dans un dossier numéroté tel que `models/rock_2`. Annuler
laisse les fichiers où ils sont. Si les textures ne peuvent pas être écrites, par exemple dans un projet en lecture seule,
le maillage arrive sans texture et la barre d'état dit pourquoi.

Deux textures comptent pour la même d'après leurs pixels, donc une texture qu'un autre programme a seulement
réenregistrée est quand même réutilisée. Vous avez peint par-dessus l'une d'elles ? Les maillages placés avant gardent
votre version, et le placement suivant reçoit des copies neuves dans un dossier numéroté, avec un nouveau matériau. Pour
donner vos changements aux placements suivants, appliquez-leur votre matériau depuis l'onglet Matériaux.

> [!NOTE]
> FuncGodot ne construit le matériau d'une face à partir du `default_material` du projet que s'il n'existe pas de fichier
> de matériau du même nom. Les faces d'un modèle placé qui ont reçu un fichier de matériau se dessinent avec ce fichier,
> donc un shader personnalisé défini comme `default_material` ne les atteint pas. Modifiez le fichier dans Godot pour les
> changer. Un projet dont l'extension de matériau n'est pas `tres` ne reçoit pas de fichiers de matériau, et son matériau
> par défaut s'applique.

## Plus sur une vignette { #more-on-a-thumbnail }

Faites un clic droit sur une vignette pour la placer au curseur, copier son chemin ou trouver le fichier sur le disque.
*Ouvrir la page du pack* apparaît quand son pack a une url.

![Le menu du clic droit sur la vignette wine_barrel_01 avec Placer au curseur survolé, puis Copier le chemin, Afficher dans l'explorateur et Ouvrir la page du pack](../assets/models/thumbnail-menu.png)

## Crédits des packs { #pack-credits }

Les packs de modèles gratuits demandent généralement une mention. Mettez un `pack.json` dans le dossier d'un modèle, ou
dans n'importe quel dossier au-dessus (le plus proche l'emporte), et le panneau sait de quel pack vient le modèle, qui l'a
fait et sous quelle licence. Survolez une vignette pour le voir :

![L'infobulle de la vignette wine_barrel_01 : son chemin, Pack : Poly Haven, l'auteur, Licence : CC0, l'url du pack et le crédit du modèle](../assets/models/thumbnail-credit-tooltip.png)

```json
{
  "name": "Nature Kit",
  "short": "Nature",
  "author": "Some Artist",
  "license": "CC0",
  "url": "https://example.com/nature-kit",
  "credits": [{ "asset": "pine", "author": "Other Artist" }]
}
```

`short` est l'étiquette de chaque vignette tant que Slugs est coché, et se rabat sur `name`. `credits` remplace l'auteur,
la licence ou l'url de modèles isolés, repérés par leur nom de fichier sans extension. Les dossiers sans `pack.json` sont
listés sous le nom de leur dossier de premier niveau.
